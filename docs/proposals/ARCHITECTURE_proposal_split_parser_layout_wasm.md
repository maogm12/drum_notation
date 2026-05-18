## Addendum v1.0: Split Parser and Layout WASM with Layout Engine as Default Renderer

### Status

Proposed.

### Problem

The current browser WASM package (`drummark_core_bg.wasm`) combines two different runtime responsibilities:

1. DrumMark parser / document skeleton generation, needed by the editor and score worker on startup.
2. Layout scene generation, needed only when rendering through the platform-neutral layout engine.

This makes the startup WASM binary much larger than the parser actually requires. A current size probe showed:

- current combined core+layout WASM: about `466 KB` raw / `164 KB` gzip
- parser-only export surface: about `106 KB` raw / `36 KB` gzip
- `build_layout_scene` export chain: about `350 KB` raw / `125 KB` gzip incremental cost

At the same time, the app still defaults rendering to the legacy VexFlow path even though the new layout engine is now the intended product direction. VexFlow must remain available for comparison, fallback, and migration debugging, but it should no longer be the default rendering path.

### Goal

Make the platform-neutral WASM layout engine the default product renderer while keeping VexFlow available as an explicit legacy path, and split the browser WASM runtime so startup only loads the parser package.

Target architecture:

```
Editor / score worker startup
    -> drummark-parser WASM
    -> parse DocumentSkeleton / NormalizedScore

Default preview render
    -> lazy import layout renderer
    -> lazy load drummark-layout WASM
    -> source + layout options -> LayoutScene
    -> SVG adapter

Legacy preview render
    -> lazy import VexFlow renderer
    -> VexFlow pages
```

### Non-Goals

This proposal does not remove VexFlow from the repository or from the production bundle graph yet.

Out of scope:

- deleting `src/vexflow/`
- removing VexFlow parity or corpus comparison tests
- removing the VexFlow dependency from `package.json`
- changing DrumMark DSL semantics
- changing the `LayoutScene` rendering contract
- introducing a compact binary scene wire format
- changing CLI native output ownership

### Core Decisions

#### 1. Split WASM Packages by Runtime Responsibility

Create two browser-facing WASM packages:

- `drummark_parser`: parser-facing API only
- `drummark_layout`: layout-facing API only

The parser package owns:

- `parse(source: &str) -> JsValue`
- any parser-specific JS object conversion needed by `src/wasm/skeleton.ts`

The layout package owns:

- `build_layout_scene(source: &str, options: JsValue) -> JsValue`
- any internal parser / normalizer / render-score derivation required to preserve the current source-to-scene app contract

This intentionally duplicates parser/normalizer logic in the layout package at first if needed. The first-order goal is runtime split, not perfect Rust crate factoring. A later proposal may introduce shared internal crates if duplication becomes a maintenance problem.

#### 2. Startup Loads Parser WASM Only

`src/main.tsx`, `src/scoreWorker.ts`, and parser-facing wrappers must initialize only the parser WASM package.

The layout WASM package must not be loaded during startup, parse-only editor typing, or XML generation unless the layout renderer is explicitly used.

#### 3. Default Rendering Uses Layout Engine

The app setting default for score preview rendering changes to the layout engine path.

VexFlow remains available as an explicit legacy renderer option. The UI copy should make the distinction clear without suggesting that VexFlow is the recommended path.

#### 4. VexFlow Remains Lazy

VexFlow must stay behind a dynamic import. The main entry HTML must not preload the VexFlow chunk, and the default layout render path must not import from `src/vexflow`.

#### 5. Layout WASM Remains Lazy

The layout renderer adapter may be imported when preview rendering is active, but it must load the layout WASM package lazily from that adapter path rather than through the parser wrapper.

### Package and File Boundaries

The browser-facing package directories should be:

```
src/wasm/parser-pkg/
src/wasm/layout-pkg/
```

The TypeScript wrappers should be:

```
src/wasm/parser_wasm.ts
src/wasm/layout_wasm.ts
```

Compatibility wrappers may exist temporarily, but final active imports should make the distinction explicit:

- parser consumers import parser wrapper APIs
- layout renderer imports layout wrapper APIs
- no active browser code imports a generic combined `drummark_wasm` wrapper

### Rust Crate Boundary Options

The implementation may choose either of these shapes if tests and size criteria pass:

#### Option A: Export-Gated Existing `drummark-core`

Use compile features to build `drummark-core` in parser-only and layout-enabled modes:

- parser build exports only parser API
- layout build exports layout API

This minimizes initial crate movement but requires careful build script control.

#### Option B: Dedicated WASM Facade Crates

Add small facade crates:

- `drummark-parser-wasm`
- `drummark-layout-wasm`

They may depend on shared internal crates and export separate wasm-bindgen surfaces.

This is cleaner long-term but larger as an implementation step.

The approved tasks must choose one option before implementation begins.

### Build Script Contract

`scripts/build_wasm.mjs` must build both browser WASM packages and place them in their final package directories.

The build script must fail if either package cannot be built.

The script must report raw and gzip sizes for both generated `.wasm` files so size regressions are visible during normal development.

### Size Targets

Initial targets:

- parser WASM gzip: at or below `45 KB`
- layout WASM gzip: no larger than the current combined gzip size unless explicitly justified
- default startup path must not fetch layout WASM before the layout renderer is used
- VexFlow chunk must not be fetched on the default layout rendering path

These are guardrails, not permanent budgets. If a target cannot be met, the implementation must document the measured reason in the tasks file before proceeding.

### App Settings and UX

The renderer setting should default to the layout engine.

The legacy renderer option should remain available for comparison and emergency fallback. User-facing labels must use plain language and avoid implementation leakage:

- preferred: `Layout Engine`, `Legacy VexFlow`
- avoid: `useLayoutEngine`, `WASM render`, `new renderer`

Existing persisted settings need a migration rule:

- if the user has an explicit persisted renderer preference, preserve it
- if no explicit preference exists, default to layout engine

### Verification

Required verification includes:

- parser wrapper tests prove parser-only WASM initializes and parses without layout package imports
- layout wrapper tests prove layout WASM initializes lazily and returns `LayoutScene`
- bundle inspection proves default entry does not preload VexFlow
- bundle inspection or browser-level test proves layout WASM is not fetched before layout rendering
- render regression tests continue to cover layout scene adapter behavior
- VexFlow legacy tests continue to pass
- build output reports parser/layout WASM sizes separately

### Risks

#### Shared Rust Code May Be Awkward

Splitting browser packages while keeping shared native parser/normalizer code may require new crate boundaries or feature flags. The tasks must choose a concrete ownership strategy before implementation.

#### Duplicate Parser Logic in Layout WASM

If layout WASM keeps accepting raw source, it may need parser and normalizer code too. That is acceptable for runtime split because layout is lazy-loaded, but the duplication must not leak back into startup.

#### Cache and Worker Initialization

The score worker currently initializes WASM eagerly. After the split, it must initialize only parser WASM. Layout rendering should not happen inside the worker unless a later proposal explicitly moves rendering there.

#### Persisted Settings Could Surprise Users

Users who previously toggled VexFlow or layout engine must retain their explicit choice. Only users without a saved renderer preference should receive the new default.

### Acceptance Criteria

- Browser startup initializes parser WASM only.
- Default score preview uses the layout engine.
- VexFlow remains available as a lazy legacy renderer.
- Parser and layout browser WASM packages are emitted separately.
- Build output reports separate parser/layout WASM sizes.
- Parser WASM gzip is at or below `45 KB`, or an approved task note explains why not.
- `npm run build` succeeds.
- Targeted parser/layout/render tests pass.
- No implementation removes VexFlow runtime dependency yet.

### Review Round 1

The proposal is directionally sound, but it is not yet implementation-safe. It identifies the desired split, but several current-code coupling points make the stated lazy-load guarantees easier to violate than the text acknowledges.

1. The default-renderer change conflicts with current eager layout adapter usage. `src/App.tsx` currently imports `./renderer/svgRenderer` inside the DSL persistence effect just to call `setLayoutSource(dsl)`, independent of renderer selection. `src/renderer/svgRenderer.ts` imports `../wasm/pkg/drummark_core` at module top level and calls `initWasm().catch(() => {})` at module top level. Under the proposed default layout renderer this may look acceptable, but the proposal also requires "layout WASM is not fetched before layout rendering"; the existing `setLayoutSource` path would fetch/import the layout adapter on every DSL change before an explicit render call. The proposal must require deleting or redesigning `setLayoutSource`/cached-source ownership so layout source is passed through render calls only, or otherwise define exactly when preview rendering begins for the startup-fetch criterion.

2. The parser/layout API boundary is ambiguous around normalized score ownership. The target architecture says startup parses `DocumentSkeleton / NormalizedScore`, while the layout package owns `build_layout_scene(source, options)` and may internally parse/normalize again. This risks dual normalizer execution with divergent outputs: the worker produces the app's `score`, while the layout renderer ignores that `score` and reparses raw source. The proposal should explicitly choose whether the layout engine contract is source-to-scene only, normalized-score-to-scene, or temporarily both, and define the equivalence tests that prove editor diagnostics/XML output and rendered output use the same parser/normalizer semantics.

3. The Rust crate options leave a hidden deadlock for Option A. `drummark-core` currently exports both parser and `build_layout_scene`, and `build_layout_scene` depends on `drummark-layout`. If parser-only and layout-enabled modes are controlled by feature flags, the proposal must state the default feature set and how TypeScript/wasm-bindgen declarations are generated without stale exports. Otherwise a parser build could still type-export layout symbols, or a layout build could be accidentally linked into parser startup through default features. The tasks should be required to pick one crate/package naming scheme and one feature matrix before implementation.

4. The "layout WASM gzip no larger than current combined gzip size" budget is under-specified. If the layout package intentionally keeps parser/normalizer code to preserve `source -> LayoutScene`, then the realistic comparison target is not just the current combined `164 KB` gzip; it also needs a policy for duplicated generated JS glue, TypeScript declarations, and any shared chunks emitted by Vite. The proposal should require measuring both raw `.wasm` sizes and browser-fetched transfer sizes for initial page load, first layout render, and first legacy render. Otherwise the WASM budget can pass while the actual product startup bundle regresses.

5. Verification needs a browser/network-level acceptance test, not only bundle inspection. Static inspection can miss `new URL(..._bg.wasm)` fetches hidden behind wasm-bindgen glue or dynamic imports triggered by effects. The proposal should require an automated dev/build preview test that records network requests and asserts: parser WASM is fetched at startup, layout WASM is not fetched until the preview render path executes, and the VexFlow chunk is not fetched on the layout path.

6. The persisted-settings migration is not precise enough for the existing `useLayoutEngine: boolean` model. Today `defaultSettings.useLayoutEngine` is `false`, and saved settings are merged with defaults. A saved settings object lacking `useLayoutEngine` is indistinguishable from a new user unless the migration checks `Object.prototype.hasOwnProperty.call(parsed, "useLayoutEngine")`. The proposal should require this explicit-presence rule and a unit test for old saved settings, explicit `false`, explicit `true`, corrupt JSON, and no saved settings.

7. The non-goal "changing CLI native output ownership" conflicts with current shared imports. `src/cli_runtime.ts` uses `renderer/svgRenderer`, which currently imports browser WASM package symbols and parser init wrappers. If the split introduces browser-only `layout_wasm.ts`, the proposal must say whether CLI keeps using the same layout SVG adapter, gets a Node-specific layout wrapper, or stays on the existing combined package. Without this, implementation may accidentally break `npm run drummark -- --format svg`, which the repo instructions require for verification.

8. The proposal does not cover test and setup imports that initialize the combined wrapper. `src/test/setup.ts`, parser tests, renderer corpus tests, and CLI tests import `initWasm` from `drummark_wasm`. The split needs a migration rule for tests: parser tests use parser wrapper, layout tests use layout wrapper, and parity/corpus tests should prove they are not masking accidental combined-wrapper use.

9. The "VexFlow remains lazy" boundary is weakened by non-render imports from `src/vexflow/config` and `src/vexflow/types` in settings and layout adapter code. Type-only imports are fine, but runtime imports from `src/vexflow/config` can pull renderer-adjacent code into default chunks even when the VexFlow renderer remains dynamically imported. The proposal should require moving shared settings ranges/types out of `src/vexflow` into renderer-neutral modules, or prove through bundle inspection that these imports do not drag VexFlow runtime code.

10. The acceptance criteria do not require documenting the public replacement for the generic `drummark_wasm` wrapper. Because many imports currently target that wrapper, the proposal should explicitly require deleting or narrowing it after migration, plus a static test that no active browser production code imports `src/wasm/drummark_wasm.ts` or `src/wasm/pkg/drummark_core`.

STATUS: CHANGES_REQUESTED

### Review Round 4

I reviewed the final Author Response and `## Addendum v1.3` as the intended current proposal.

On proposal substance, v1.3 is approval-ready. It restates the split as an explicit implementation contract rather than an aspirational direction:

1. The Rust/WASM topology is concrete enough to implement and verify. It names web and Node package directories, requires `--no-default-features`, cfg-gated exports, and a build failure if the parser package exposes or links layout code.
2. The browser, parser-facing, and CLI import boundaries are enforceable. v1.3 blocks the old combined wrapper in production browser code, Node packages in browser code, layout wrappers in parser-facing code, and browser packages in CLI runtime.
3. The render API now carries `source` and `sourceRevision` with the active parsed score state, which closes the stale-source race identified earlier.
4. The network audit scenarios are specific enough to avoid timing-dependent interpretations of "before preview rendering." The required cold, incremental, and cumulative labels should produce actionable size data.
5. Parser/layout semantic parity is now explicit for both successful parses and parse failures, with coverage over the right musical structures.
6. CLI SVG ownership is no longer ambiguous: it remains TypeScript-owned and uses a Node layout WASM wrapper for this proposal.

I do not see a remaining architecture-level blocker in v1.3 itself. The next tasks-file phase still needs to decompose this into independently testable tasks, especially around the build script, static import enforcement, network audit, and parity corpus, but those are planning requirements rather than proposal defects.

The ledger repair is not acceptable yet. The Author Response says the current authoritative proposal is "restated below at the physical end of the file," but in the actual file `### Author Response` and `## Addendum v1.3` appear immediately after Review Round 1 and before the existing v1.1, Review Round 2, and Review Round 3 sections. That means the attempted repair repeated the same class of physical-order problem: it did not append the repair after the latest review at the true end of the ledger.

Because prior content must not be moved or edited, the acceptable repair remains append-only: append a new Author Response after this Review Round 4 that acknowledges v1.3 was also placed out of physical order, identifies the line-order mistake, and appends a clean current proposal addendum or consolidated current proposal text at the actual end of the file. Once the current proposal text is physically after all prior review rounds, I would expect approval unless the new appended text materially changes the architecture.

STATUS: CHANGES_REQUESTED

### Author Response

The ledger finding is accepted. The attempted v1.3 repair was also inserted after an earlier review marker rather than after the latest review at the actual end of the file. This response is anchored to the final Review Round 4 text and is the first valid repair placed after all existing review rounds.

The technical content of v1.3 is unchanged. The current authoritative proposal is restated below as v1.4 at the physical end of the ledger.

## Addendum v1.4: Current Proposal for Split Parser/Layout WASM and Default Layout Rendering

This addendum supersedes v1.0, v1.1, v1.2, and v1.3 as the implementation target while preserving the earlier ledger history.

### Goals

- Make the layout engine the default preview renderer for users without an explicit saved renderer preference.
- Keep VexFlow available as a lazy legacy renderer.
- Split browser WASM into parser and layout packages so startup does not load layout code before a layout render is requested.
- Keep CLI SVG output working.
- Measure real browser transfer costs, not only raw `.wasm` file sizes.

### Rust and WASM Package Topology

Use the existing Rust WASM crate with explicit feature-gated builds.

Required build commands:

- browser parser package: `wasm-pack build --target web --no-default-features --features parser-wasm`
- browser layout package: `wasm-pack build --target web --no-default-features --features layout-wasm`
- Node layout package: `wasm-pack build --target nodejs --no-default-features --features layout-wasm`
- Node parser package: `wasm-pack build --target nodejs --no-default-features --features parser-wasm`, only if needed by tests or CLI parser initialization

Generated package directories:

- `src/wasm/parser-pkg-web/`
- `src/wasm/layout-pkg-web/`
- `src/wasm/layout-pkg-node/`
- `src/wasm/parser-pkg-node/`, only if needed

`drummark-layout` must be optional from the parser build's perspective and enabled only by `layout-wasm` or an equivalent non-parser feature. Parser exports must be cfg-gated to parser builds. Layout exports must be cfg-gated to layout builds. Shared helper code may compile for both only when it does not pull `drummark-layout` into the parser build.

The build script must fail if parser package generated bindings expose layout exports or if dependency analysis shows `drummark-layout` linked into the parser package.

### TypeScript Entrypoints

Browser production code must use:

- `src/wasm/parser_wasm_browser.ts`
- `src/wasm/layout_wasm_browser.ts`

Node and CLI code must use:

- `src/wasm/layout_wasm_node.ts`
- `src/wasm/parser_wasm_node.ts`, only if Node parser WASM is required

Browser wrappers import only `*-pkg-web`. Node wrappers import only `*-pkg-node`.

The generic `src/wasm/drummark_wasm.ts` wrapper must not remain an active browser production dependency. It may be deleted or narrowed to a test-only compatibility shim.

### Layout Render API and Source Coherence

The layout render path accepts raw source explicitly at call time:

```ts
renderScoreToSvg(score, settings, { source, sourceRevision })
renderScorePagesToSvgs(score, settings, { source, sourceRevision })
```

The exact function names may follow local style, but production rendering must not use `setLayoutSource` or any module-level source cache.

The app must associate parsed score state with the exact source string and a monotonically increasing source revision:

```ts
{
  score,
  source,
  sourceRevision
}
```

If parsing is asynchronous, stale parse results must not replace newer score/source revisions. The active preview render receives the source attached to the active parsed score revision, not an independently captured editor string.

For this proposal, the layout package owns a `source -> LayoutScene` boundary. Duplicate parser/normalizer code inside the lazy layout package is acceptable because it does not affect startup.

### Default Renderer and Settings Migration

The layout engine becomes the default renderer for users without an explicit saved preference.

Migration rule:

- no saved settings: `useLayoutEngine` defaults to `true`
- saved settings without an own `useLayoutEngine` property: `useLayoutEngine` defaults to `true`
- saved settings with own `useLayoutEngine: false`: preserve `false`
- saved settings with own `useLayoutEngine: true`: preserve `true`
- corrupt saved settings: fall back to default settings with `useLayoutEngine: true`

Implementation must use explicit own-property detection equivalent to:

```ts
Object.prototype.hasOwnProperty.call(parsed, "useLayoutEngine")
```

User-facing labels should be `Layout Engine` and `Legacy VexFlow`, not implementation names.

### Renderer-Neutral Shared Options

Shared setting ranges, render settings types, and page layout option definitions used outside the VexFlow renderer must live in renderer-neutral modules.

Default app code may type-import from VexFlow modules only when erased at build time. Runtime imports from `src/vexflow/*` in default layout settings, app initialization, or layout renderer code are not allowed unless bundle evidence proves VexFlow runtime remains lazy.

### CLI Contract

`npm run drummark -- <input> --format svg` must keep working.

CLI SVG output remains TypeScript-owned for this proposal. The CLI layout path must initialize layout WASM through `src/wasm/layout_wasm_node.ts` and `src/wasm/layout-pkg-node/`. This proposal does not move CLI SVG rendering to a native Rust executable.

### Static Import Boundaries

Static enforcement must fail on production browser imports of:

- `src/wasm/drummark_wasm.ts`
- `src/wasm/pkg/drummark_core`
- `src/wasm/*_wasm_node.ts`
- `src/wasm/*-pkg-node`

Static enforcement must fail on parser-facing imports of:

- `src/wasm/layout_wasm_browser.ts`
- `src/wasm/layout-pkg-web`
- `src/wasm/layout_wasm_node.ts`
- `src/wasm/layout-pkg-node`

Static enforcement must fail on CLI imports of browser-only wrappers or browser generated packages.

Tests may intentionally import both parser and layout wrappers only when the file path or test name makes the integration/parity purpose explicit.

### Network and Size Verification

The build and audit must report:

- raw asset size
- gzip asset size
- brotli asset size
- cache-cold browser transfer
- incremental browser transfer after a named prior scenario
- cumulative browser transfer for the scenario

Required browser/network scenarios:

1. Initial app load with preview inactive or rendering deliberately suspended before renderer invocation.
   - Must fetch parser WASM.
   - Must not fetch layout WASM.
   - Must not fetch VexFlow chunk.

2. First default layout preview render in a fresh browser context.
   - Must fetch parser WASM as needed.
   - Must fetch layout WASM.
   - Must not fetch VexFlow chunk.
   - Must report cumulative transfer and layout-render incremental transfer.

3. First legacy VexFlow render in a fresh browser context.
   - Must fetch parser WASM as needed.
   - Must fetch VexFlow chunk.
   - Must report cumulative transfer and VexFlow incremental transfer.

4. Legacy VexFlow render after an already completed default layout render.
   - Must report incremental transfer after cached parser/layout assets.

The preview suspension mechanism for scenario 1 may be a test route, query parameter, or test-only setting, and must not alter normal production behavior.

If the repo lacks suitable browser automation, tasks may add a development dependency for this audit.

### Semantic Parity Verification

A shared corpus must verify successful-parse parity between parser WASM and the layout WASM source-to-scene path.

The parity check must cover:

- measure count
- barline and repeat markers
- navigation markers such as endings, coda, segno, fine, and D.C./D.S. when present in fixtures
- timing constructs including time signatures, rests, subdivisions, tuplets if currently supported, beams, and multi-measure input
- representative notation from existing renderer regression tests

The comparison does not need exact SVG coordinates. It must assert that the layout scene's structural interpretation agrees with parser/normalizer output on the musical document shape.

Parse-failure parity must also be tested.

### Acceptance Criteria

- Parser package declarations contain no layout exports.
- Layout package declarations expose the layout scene builder expected by the layout wrapper.
- Parser build uses `--no-default-features --features parser-wasm`.
- Layout browser and Node builds use `--no-default-features --features layout-wasm`.
- Parser build does not link `drummark-layout`.
- Default app rendering no longer imports or calls `setLayoutSource`.
- Browser production code imports only browser wrappers and web generated packages.
- CLI runtime imports only Node wrappers and Node generated packages for WASM initialization.
- Active parsed score state carries `source` and `sourceRevision`.
- Rapid-edit rendering test proves layout source and active score revision stay coherent.
- Settings migration tests cover no saved settings, old saved settings without `useLayoutEngine`, explicit `false`, explicit `true`, and corrupt JSON.
- Static import tests enforce the forbidden boundaries listed above.
- Network audit reports all four required scenarios with cold and incremental transfer labels.
- Shared parser/layout parity corpus covers successful parses and parse failures.
- `npm run drummark -- <input> --format svg` passes on at least one representative fixture.
- `npm run build` succeeds and reports parser/layout WASM sizes separately.
- VexFlow remains available as a lazy legacy renderer.

### Review Round 5

The ledger repair is now acceptable. The new Author Response and `## Addendum v1.5` are physically appended after the prior `### Review Round 4` status line at the actual end of the file. The response also explicitly acknowledges the earlier v1.3/v1.4 placement error, so the repair preserves the historical defect instead of attempting to rewrite or obscure it. That satisfies the append-only Linear Ledger requirement for this proposal file.

The duplicate earlier addenda and duplicated review sections remain historically messy, but they are no longer a blocking protocol issue because v1.5 clearly states that it supersedes v1.0 through v1.4 and is now the latest physically appended implementation target. Future consolidation should use v1.5 as the clean source and leave prior ledger content untouched.

Technically, v1.5 remains approval-ready. It keeps the important architecture constraints intact: parser and layout WASM builds are feature-gated with `--no-default-features`, browser and Node wrappers are separated, parser builds must fail if they expose or link layout code, stale layout-source state is replaced by explicit `source` and `sourceRevision`, default-renderer migration preserves explicit legacy choices, VexFlow remains lazy, CLI SVG ownership stays TypeScript-side through Node layout WASM, static import boundaries are enforceable, and network/parity acceptance criteria are specific enough to test.

The next tasks file still needs to prove task independence rather than hide coupling between package generation, wrapper refactors, static boundary checks, browser transfer auditing, and semantic parity fixtures. That is a tasks-planning concern, not a remaining proposal blocker.

STATUS: APPROVED

### Consolidated Changes

The approved implementation target is `## Addendum v1.5: Current Proposal for Split Parser/Layout WASM and Default Layout Rendering`.

The implementation will make the layout engine the default preview renderer for users without an explicit saved renderer preference, while keeping VexFlow available as a lazy `Legacy VexFlow` renderer.

Browser WASM is split into explicit parser and layout packages:

- `src/wasm/parser-pkg-web/`
- `src/wasm/layout-pkg-web/`

CLI/Node layout initialization uses a separate Node package:

- `src/wasm/layout-pkg-node/`

The Rust WASM crate must build these packages through explicit feature sets using `--no-default-features`. The parser package must not expose layout exports or link `drummark-layout`.

The TypeScript runtime must use explicit wrapper boundaries:

- browser parser wrapper imports only the web parser package
- browser layout wrapper imports only the web layout package
- CLI layout wrapper imports only the Node layout package
- the old generic combined wrapper must not remain in active browser production imports

The layout render API must receive `source` and `sourceRevision` explicitly at render time. Production code must remove `setLayoutSource` and any module-level layout source cache. Active parsed score state must carry the coherent tuple `{ score, source, sourceRevision }`, and stale async parse results must not replace newer state.

Settings migration preserves explicit user choices:

- no saved preference defaults to layout engine
- saved settings without an own `useLayoutEngine` property default to layout engine
- explicit `useLayoutEngine: false` remains legacy VexFlow
- explicit `useLayoutEngine: true` remains layout engine
- corrupt settings fall back to layout engine

Shared renderer settings and option ranges used outside VexFlow must live in renderer-neutral modules, so default app/settings/layout code does not import VexFlow runtime.

Verification requires static import enforcement, parser/layout semantic parity for successful and failed sources, browser network/transfer audits for startup/layout/legacy scenarios, separate WASM asset-size reporting, and continued CLI SVG support.

### Author Response

The ledger finding is accepted. A previous append operation matched an earlier `STATUS: CHANGES_REQUESTED` marker instead of the physical end of the file, so the v1.2 response and addendum are present but out of chronological order. That ordering mistake is not repaired by moving or editing earlier text. The current authoritative proposal is therefore restated below at the physical end of the file.

## Addendum v1.3: Current Proposal for Split Parser/Layout WASM and Default Layout Rendering

This addendum supersedes v1.0, v1.1, and v1.2 as the implementation target while preserving the earlier ledger history.

### Goals

- Make the layout engine the default preview renderer for users without an explicit saved renderer preference.
- Keep VexFlow available as a lazy legacy renderer.
- Split browser WASM into parser and layout packages so startup does not load layout code before a layout render is requested.
- Keep CLI SVG output working.
- Measure real browser transfer costs, not only raw `.wasm` file sizes.

### Rust and WASM Package Topology

Use the existing Rust WASM crate with explicit feature-gated builds.

Required build commands:

- browser parser package: `wasm-pack build --target web --no-default-features --features parser-wasm`
- browser layout package: `wasm-pack build --target web --no-default-features --features layout-wasm`
- Node layout package: `wasm-pack build --target nodejs --no-default-features --features layout-wasm`
- Node parser package: `wasm-pack build --target nodejs --no-default-features --features parser-wasm`, only if needed by tests or CLI parser initialization

Generated package directories:

- `src/wasm/parser-pkg-web/`
- `src/wasm/layout-pkg-web/`
- `src/wasm/layout-pkg-node/`
- `src/wasm/parser-pkg-node/`, only if needed

`drummark-layout` must be optional from the parser build's perspective and enabled only by `layout-wasm` or an equivalent non-parser feature. Parser exports must be cfg-gated to parser builds. Layout exports must be cfg-gated to layout builds. Shared helper code may compile for both only when it does not pull `drummark-layout` into the parser build.

The build script must fail if parser package generated bindings expose layout exports or if dependency analysis shows `drummark-layout` linked into the parser package.

### TypeScript Entrypoints

Browser production code must use:

- `src/wasm/parser_wasm_browser.ts`
- `src/wasm/layout_wasm_browser.ts`

Node and CLI code must use:

- `src/wasm/layout_wasm_node.ts`
- `src/wasm/parser_wasm_node.ts`, only if Node parser WASM is required

Browser wrappers import only `*-pkg-web`. Node wrappers import only `*-pkg-node`.

The generic `src/wasm/drummark_wasm.ts` wrapper must not remain an active browser production dependency. It may be deleted or narrowed to a test-only compatibility shim.

### Layout Render API and Source Coherence

The layout render path accepts raw source explicitly at call time:

```ts
renderScoreToSvg(score, settings, { source, sourceRevision })
renderScorePagesToSvgs(score, settings, { source, sourceRevision })
```

The exact function names may follow local style, but production rendering must not use `setLayoutSource` or any module-level source cache.

The app must associate parsed score state with the exact source string and a monotonically increasing source revision:

```ts
{
  score,
  source,
  sourceRevision
}
```

If parsing is asynchronous, stale parse results must not replace newer score/source revisions. The active preview render receives the source attached to the active parsed score revision, not an independently captured editor string.

For this proposal, the layout package owns a `source -> LayoutScene` boundary. Duplicate parser/normalizer code inside the lazy layout package is acceptable because it does not affect startup.

### Default Renderer and Settings Migration

The layout engine becomes the default renderer for users without an explicit saved preference.

Migration rule:

- no saved settings: `useLayoutEngine` defaults to `true`
- saved settings without an own `useLayoutEngine` property: `useLayoutEngine` defaults to `true`
- saved settings with own `useLayoutEngine: false`: preserve `false`
- saved settings with own `useLayoutEngine: true`: preserve `true`
- corrupt saved settings: fall back to default settings with `useLayoutEngine: true`

Implementation must use explicit own-property detection equivalent to:

```ts
Object.prototype.hasOwnProperty.call(parsed, "useLayoutEngine")
```

User-facing labels should be `Layout Engine` and `Legacy VexFlow`, not implementation names.

### Renderer-Neutral Shared Options

Shared setting ranges, render settings types, and page layout option definitions used outside the VexFlow renderer must live in renderer-neutral modules.

Default app code may type-import from VexFlow modules only when erased at build time. Runtime imports from `src/vexflow/*` in default layout settings, app initialization, or layout renderer code are not allowed unless bundle evidence proves VexFlow runtime remains lazy.

### CLI Contract

`npm run drummark -- <input> --format svg` must keep working.

CLI SVG output remains TypeScript-owned for this proposal. The CLI layout path must initialize layout WASM through `src/wasm/layout_wasm_node.ts` and `src/wasm/layout-pkg-node/`. This proposal does not move CLI SVG rendering to a native Rust executable.

### Static Import Boundaries

Static enforcement must fail on production browser imports of:

- `src/wasm/drummark_wasm.ts`
- `src/wasm/pkg/drummark_core`
- `src/wasm/*_wasm_node.ts`
- `src/wasm/*-pkg-node`

Static enforcement must fail on parser-facing imports of:

- `src/wasm/layout_wasm_browser.ts`
- `src/wasm/layout-pkg-web`
- `src/wasm/layout_wasm_node.ts`
- `src/wasm/layout-pkg-node`

Static enforcement must fail on CLI imports of browser-only wrappers or browser generated packages.

Tests may intentionally import both parser and layout wrappers only when the file path or test name makes the integration/parity purpose explicit.

### Network and Size Verification

The build and audit must report:

- raw asset size
- gzip asset size
- brotli asset size
- cache-cold browser transfer
- incremental browser transfer after a named prior scenario
- cumulative browser transfer for the scenario

Required browser/network scenarios:

1. Initial app load with preview inactive or rendering deliberately suspended before renderer invocation.
   - Must fetch parser WASM.
   - Must not fetch layout WASM.
   - Must not fetch VexFlow chunk.

2. First default layout preview render in a fresh browser context.
   - Must fetch parser WASM as needed.
   - Must fetch layout WASM.
   - Must not fetch VexFlow chunk.
   - Must report cumulative transfer and layout-render incremental transfer.

3. First legacy VexFlow render in a fresh browser context.
   - Must fetch parser WASM as needed.
   - Must fetch VexFlow chunk.
   - Must report cumulative transfer and VexFlow incremental transfer.

4. Legacy VexFlow render after an already completed default layout render.
   - Must report incremental transfer after cached parser/layout assets.

The preview suspension mechanism for scenario 1 may be a test route, query parameter, or test-only setting, and must not alter normal production behavior.

If the repo lacks suitable browser automation, tasks may add a development dependency for this audit.

### Semantic Parity Verification

A shared corpus must verify successful-parse parity between parser WASM and the layout WASM source-to-scene path.

The parity check must cover:

- measure count
- barline and repeat markers
- navigation markers such as endings, coda, segno, fine, and D.C./D.S. when present in fixtures
- timing constructs including time signatures, rests, subdivisions, tuplets if currently supported, beams, and multi-measure input
- representative notation from existing renderer regression tests

The comparison does not need exact SVG coordinates. It must assert that the layout scene's structural interpretation agrees with parser/normalizer output on the musical document shape.

Parse-failure parity must also be tested.

### Acceptance Criteria

- Parser package declarations contain no layout exports.
- Layout package declarations expose the layout scene builder expected by the layout wrapper.
- Parser build uses `--no-default-features --features parser-wasm`.
- Layout browser and Node builds use `--no-default-features --features layout-wasm`.
- Parser build does not link `drummark-layout`.
- Default app rendering no longer imports or calls `setLayoutSource`.
- Browser production code imports only browser wrappers and web generated packages.
- CLI runtime imports only Node wrappers and Node generated packages for WASM initialization.
- Active parsed score state carries `source` and `sourceRevision`.
- Rapid-edit rendering test proves layout source and active score revision stay coherent.
- Settings migration tests cover no saved settings, old saved settings without `useLayoutEngine`, explicit `false`, explicit `true`, and corrupt JSON.
- Static import tests enforce the forbidden boundaries listed above.
- Network audit reports all four required scenarios with cold and incremental transfer labels.
- Shared parser/layout parity corpus covers successful parses and parse failures.
- `npm run drummark -- <input> --format svg` passes on at least one representative fixture.
- `npm run build` succeeds and reports parser/layout WASM sizes separately.
- VexFlow remains available as a lazy legacy renderer.

### Author Response

The review is accepted. v1.1 still left several topology decisions as implementation discretion. Those decisions are now part of the proposal contract.

Response to review point 1: the Cargo feature mechanics must be explicit. Browser builds must use `--no-default-features`; `drummark-layout` must be optional from the parser build's point of view; wasm-bindgen exports must be cfg-gated so hidden TypeScript declarations are not the only protection.

Response to review point 2: Node/CLI packages must be generated explicitly. CLI SVG will continue to use the TypeScript SVG adapter in this proposal, but it will initialize layout through a `wasm-bindgen --target nodejs` layout package. This avoids mixing browser package initialization with Node runtime behavior.

Response to review point 3: the app must carry a source revision with parsed score state. Layout rendering receives the source attached to the active score revision, not an independently captured editor string.

Response to review point 4: the network audit must use controlled scenarios. It cannot rely on racing the default preview effect.

Response to review point 5: static import enforcement must cover all old and new forbidden directions, including browser code importing Node wrappers and parser-facing code importing layout wrappers.

Response to review point 6: semantic parity criteria must be explicit for successful parses, not just failures.

Response to review point 7: transfer-size reporting must separate cold contexts and cumulative/incremental measurements.

## Addendum v1.2: Package Topology and Observable Runtime Contract

This version refines v1.1 with implementation-critical package and runtime details.

### Cargo Feature Mechanics

The existing Rust WASM crate must support explicit feature-gated builds:

- browser parser package: `wasm-pack build --target web --no-default-features --features parser-wasm`
- browser layout package: `wasm-pack build --target web --no-default-features --features layout-wasm`
- Node layout package: `wasm-pack build --target nodejs --no-default-features --features layout-wasm`
- Node parser package, if needed by tests or CLI: `wasm-pack build --target nodejs --no-default-features --features parser-wasm`

`drummark-layout` must be an optional dependency from the parser build's perspective and enabled only by `layout-wasm` or an equivalent non-parser feature.

WASM exports must be cfg-gated:

- parser exports are compiled only for `parser-wasm`
- layout exports are compiled only for `layout-wasm`
- shared helper code may compile for both only when it does not pull `drummark-layout` into `parser-wasm`

The build script must fail if parser package generated bindings expose layout exports, or if parser package dependency analysis shows `drummark-layout` linked into the parser build.

### Generated Package Directories

The build emits separate generated packages:

- `src/wasm/parser-pkg-web/`
- `src/wasm/layout-pkg-web/`
- `src/wasm/layout-pkg-node/`
- `src/wasm/parser-pkg-node/`, only if needed by tests or CLI parser initialization

Browser wrappers import only `*-pkg-web` packages.

Node wrappers import only `*-pkg-node` packages.

The browser app must never import a Node wrapper or Node package. CLI runtime must never import a browser wrapper or browser package.

### CLI Runtime Decision

CLI SVG output remains TypeScript-owned for this proposal and continues to use the existing SVG adapter surface where practical.

The CLI layout path must call `src/wasm/layout_wasm_node.ts`, which initializes `src/wasm/layout-pkg-node/`.

If parser WASM is required in CLI tests, it must use `src/wasm/parser_wasm_node.ts` and `src/wasm/parser-pkg-node/`.

The proposal does not move CLI SVG rendering to a native Rust executable. That can be considered later, but it is not part of this change.

### Source Revision Contract

The app must associate parsed score state with the exact source string and a monotonically increasing source revision.

The active preview render receives the source attached to the active parsed score revision. It must not read a mutable editor string or module-level source cache independently of that score.

If parsing is asynchronous, stale parse results must not replace newer score/source revisions. The layout renderer must therefore receive a coherent pair:

```ts
{
  score,
  source,
  sourceRevision
}
```

The exact type name may follow local style, but the invariant is mandatory: layout `source` and app `score` represent the same revision.

Verification must include a rapid-edit test where multiple source changes are queued and the rendered layout uses the source associated with the final accepted score, not an earlier or later editor snapshot.

### Network Audit Scenarios

The browser/network audit must run in controlled cache-cold contexts.

Required scenarios:

1. Initial app load with preview inactive or rendering deliberately suspended before renderer invocation.
   - Must fetch parser WASM.
   - Must not fetch layout WASM.
   - Must not fetch VexFlow chunk.

2. First default layout preview render in a fresh browser context.
   - Must fetch parser WASM as needed.
   - Must fetch layout WASM.
   - Must not fetch VexFlow chunk.
   - Must report both cumulative transfer and layout-render incremental transfer.

3. First legacy VexFlow render in a fresh browser context.
   - Must fetch parser WASM as needed.
   - Must fetch VexFlow chunk.
   - Must report cumulative transfer and VexFlow incremental transfer.

4. Legacy VexFlow render after an already completed default layout render.
   - Must report incremental transfer after cached parser/layout assets.

The audit must define how preview is suspended for scenario 1, such as a test route, query parameter, or test-only setting. The mechanism must not affect normal production behavior.

### Static Import Boundaries

Static enforcement must fail on these production browser imports:

- old combined wrapper: `src/wasm/drummark_wasm.ts`
- old combined generated package: `src/wasm/pkg/drummark_core`
- Node wrappers: `src/wasm/*_wasm_node.ts`
- Node generated packages: `src/wasm/*-pkg-node`

Static enforcement must fail on these parser-facing imports:

- `src/wasm/layout_wasm_browser.ts`
- `src/wasm/layout-pkg-web`
- `src/wasm/layout_wasm_node.ts`
- `src/wasm/layout-pkg-node`

Static enforcement must fail on CLI imports of browser-only wrappers or browser generated packages.

Test files may intentionally import both parser and layout wrappers only when the test name or file path makes the integration/parity purpose explicit.

### Parser/Layout Semantic Parity

A shared corpus must verify successful-parse parity between parser WASM and layout WASM's internal source-to-scene path.

The parity check must cover at least:

- measure count
- barline and repeat markers
- navigation markers such as endings, coda, segno, fine, and D.C./D.S. when present in fixtures
- timing constructs including time signatures, rests, subdivisions, tuplets if currently supported, beams, and multi-measure input
- representative notation used by existing renderer regression tests

The comparison does not need to assert exact SVG coordinates. It must assert that the layout scene's structural interpretation agrees with the parser/normalizer output on the musical document shape.

Parse-failure parity must also be tested: invalid source should produce compatible diagnostics between parser-facing behavior and layout-facing behavior.

### Size Reporting Semantics

Size reports must label each number as one of:

- raw asset size
- gzip asset size
- brotli asset size
- cache-cold browser transfer
- incremental browser transfer after named prior scenario
- cumulative browser transfer for the scenario

First legacy render must be measured in both a fresh context and after the default layout path has already loaded, so shared chunks do not hide the true lazy-load cost.

### Additional Acceptance Criteria

- Parser build uses `--no-default-features --features parser-wasm`.
- Layout browser build uses `--no-default-features --features layout-wasm`.
- Layout Node build uses `--no-default-features --features layout-wasm`.
- Parser build does not link `drummark-layout`.
- Browser production code imports only browser wrappers and web generated packages.
- CLI runtime imports only Node wrappers and Node generated packages for WASM initialization.
- Active parsed score state carries `source` and `sourceRevision`.
- Rapid-edit rendering test proves layout source and active score revision stay coherent.
- Network audit reports all four required scenarios with cold and incremental transfer labels.
- Shared parser/layout parity corpus covers successful parses and parse failures.

### Author Response

The review is accepted. The original proposal described the desired split but left too much room for the current combined wrapper and eager layout adapter to survive under new names. The implementation must treat lazy loading as an API contract, not a bundler side effect.

Response to review point 1: `setLayoutSource` and layout-source global cache must be removed from browser production rendering. The app owns the current DSL string and passes it into each layout render call explicitly. The layout renderer module must not initialize WASM at module load. Preview rendering begins when the preview effect invokes the active renderer for a concrete score/source pair.

Response to review point 2: the first implementation will keep the layout package as a `source -> LayoutScene` boundary. This intentionally duplicates parser/normalizer code inside the lazy layout package so startup can remain parser-only. The split must add parity tests proving parser failures and normalized semantics remain aligned for a shared corpus. A later proposal may move to a serialized render-score boundary if size or duplicate work becomes the next limiting factor.

Response to review point 3: the implementation will use feature-gated builds of the existing Rust WASM crate, not dedicated facade crates, for this proposal. The build script must compile two explicit feature sets into two package directories, and the generated TypeScript declarations must be verified so parser symbols do not include layout exports.

Response to review point 4: size reporting must measure browser-observed transfer groups, not only `.wasm` files. The acceptance data must include initial load, first default layout render, and first legacy VexFlow render.

Response to review point 5: verification must include a browser/network audit. Static chunk inspection remains useful, but it is not sufficient.

Response to review point 6: persisted renderer migration must use explicit-property detection for `useLayoutEngine`; merged defaults alone are insufficient.

Response to review point 7: CLI rendering must keep working through a Node-specific layout WASM wrapper or equivalent Node-safe entry. Browser wrappers must not import Node-only modules.

Response to review point 8: tests must migrate away from the generic combined wrapper according to the behavior under test. Parser tests initialize parser WASM; layout tests initialize layout WASM; integration tests may use both intentionally.

Response to review point 9: shared renderer options and setting ranges must move out of `src/vexflow` into renderer-neutral modules unless proven to be type-only imports erased from the runtime bundle.

Response to review point 10: `src/wasm/drummark_wasm.ts` must either be deleted or narrowed to a test-only compatibility shim. Active browser production code must not import the generic wrapper or the combined generated package.

## Addendum v1.1: Binding Decisions and Lazy-Load Contract

This version refines v1.0 with mandatory implementation decisions.

### Rust Build Strategy

Use the existing Rust WASM crate with explicit feature-gated browser builds.

The build script must produce:

- `src/wasm/parser-pkg/`
- `src/wasm/layout-pkg/`

The script must invoke both builds with explicit feature sets. It must not rely on crate default features for browser packages.

Required feature matrix:

- `parser-wasm`: exports parser/normalization APIs needed by the editor, diagnostics, worker, and non-layout previews; excludes `drummark-layout`.
- `layout-wasm`: enables `drummark-layout` and exports `build_layout_scene(source, options)`; may include parser/normalizer code because the layout package is lazy.

Generated TypeScript declaration files are part of the contract. Parser package declarations must not expose layout exports. Layout package declarations must expose only layout rendering exports and the shared initialization surface needed by wasm-bindgen.

### Browser TypeScript Entrypoints

Browser production code must use explicit wrappers:

- `src/wasm/parser_wasm_browser.ts`
- `src/wasm/layout_wasm_browser.ts`

Node and CLI code must use explicit Node-safe wrappers:

- `src/wasm/parser_wasm_node.ts`, if parser WASM is needed outside the browser
- `src/wasm/layout_wasm_node.ts`

The browser wrappers may dynamically import generated package modules, but they must not import Node-only modules. The Node wrappers may use Node-specific initialization where required, but they must not be reachable from Vite browser production entrypoints.

The generic `src/wasm/drummark_wasm.ts` wrapper must not remain as an active browser production dependency. It may be deleted or converted into a narrow test compatibility shim, with a static test proving no browser production source imports it or `src/wasm/pkg/drummark_core`.

### Layout Render API

The layout render path accepts raw source explicitly:

```ts
renderScoreToSvg(score, settings, { source })
renderScorePagesToSvgs(score, settings, { source })
```

The exact function names may follow existing local style, but the source argument must be passed at call time. The renderer must not depend on a module-level cached source.

`setLayoutSource` and the layout-source global cache are removed from production code.

The layout package owns `source -> LayoutScene` in this proposal. The already-parsed `score` can still be used by the app for editor state, status, and legacy renderer inputs, but layout rendering must pass the same current source string into the layout wrapper.

### Default Renderer and Settings Migration

The layout engine becomes the default renderer for users without an explicit saved preference.

Migration rule:

- no saved settings: `useLayoutEngine` defaults to `true`
- saved settings without an own `useLayoutEngine` property: `useLayoutEngine` defaults to `true`
- saved settings with own `useLayoutEngine: false`: preserve `false`
- saved settings with own `useLayoutEngine: true`: preserve `true`
- corrupt saved settings: fall back to default settings with `useLayoutEngine: true`

The implementation must use explicit own-property detection equivalent to:

```ts
Object.prototype.hasOwnProperty.call(parsed, "useLayoutEngine")
```

### Renderer-Neutral Shared Options

Any shared setting ranges, render settings types, or page layout option definitions used outside the VexFlow renderer must live in renderer-neutral modules, for example under `src/renderer/` or `src/rendering/`.

Default app code may type-import from VexFlow modules only when the import is erased at build time. Runtime imports from `src/vexflow/*` in default layout settings, app initialization, or layout renderer code are not allowed unless bundle evidence proves VexFlow runtime remains lazy.

### CLI Contract

`npm run drummark -- <input> --format svg` must keep working after the split.

CLI SVG generation may reuse the same layout SVG adapter, but it must initialize layout WASM through a Node-safe wrapper. Browser-only generated package imports are not allowed in CLI runtime code unless they are proven to work under Node and do not break Vite browser splitting.

### Test Migration Contract

Tests must import the wrapper that matches the behavior under test:

- parser and worker tests: parser wrapper
- layout scene and SVG adapter tests: layout wrapper
- end-to-end tests that intentionally compare both paths: both wrappers explicitly

No test should pass merely because it imports the old combined wrapper globally in setup.

### Network and Size Verification

The build must report:

- parser `.wasm` raw, gzip, and brotli sizes
- layout `.wasm` raw, gzip, and brotli sizes
- browser-observed transfer size for initial load
- browser-observed transfer size for first default layout render
- browser-observed transfer size for first legacy VexFlow render

Verification must include a browser/network audit against a production build preview. It must assert:

- parser WASM is fetched on startup
- layout WASM is not fetched before preview rendering invokes the layout path
- layout WASM is fetched when the default preview renders
- VexFlow chunk is not fetched on the default layout render path
- VexFlow chunk is fetched only when the legacy renderer is selected

If the repo does not already have a suitable browser automation dependency, the tasks may add one as a development dependency for this audit.

### Additional Acceptance Criteria

- Parser package declarations contain no layout exports.
- Layout package declarations expose the layout scene builder expected by the layout wrapper.
- Default app rendering no longer imports or calls `setLayoutSource`.
- A static test fails if browser production source imports `src/wasm/drummark_wasm.ts` or `src/wasm/pkg/drummark_core`.
- Settings migration tests cover no saved settings, old saved settings without `useLayoutEngine`, explicit `false`, explicit `true`, and corrupt JSON.
- `npm run drummark -- <input> --format svg` passes on at least one representative fixture.

### Review Round 2

v1.1 resolves the main Round 1 concerns about eager layout imports, `setLayoutSource`, persisted-setting migration, renderer-neutral settings, generic wrapper removal, and the need for browser/network verification. The proposal is now much closer to implementable, but I still see blockers that should be closed before this is approved.

1. The chosen Rust strategy is still not fully specified enough to prevent a failed or leaky split. v1.1 names `parser-wasm` and `layout-wasm`, but it does not require the concrete Cargo feature mechanics that make that possible: `drummark-layout` must become an optional dependency for parser builds, layout exports must be behind `#[cfg(feature = "layout-wasm")]`, parser exports must be behind `#[cfg(feature = "parser-wasm")]` or equivalent, and the build script must use `--no-default-features --features ...` for both browser packages. Without those requirements, the existing `drummark-core` dependency graph can still link `drummark-layout` into the parser package even if TypeScript declarations happen to hide layout symbols.

2. The Node/CLI WASM contract remains incomplete. The addendum introduces `parser_wasm_node.ts` and `layout_wasm_node.ts`, but the build outputs only name browser package directories. A Node-safe wrapper cannot be evaluated without specifying whether it consumes the same `--target web` wasm-bindgen output, a separate `--target nodejs`/bundler output, or a native Rust CLI path. This is an implementation blocker for `npm run drummark -- <input> --format svg`, because the current CLI imports the browser-oriented wrapper and `renderer/svgRenderer`. The proposal should explicitly define the generated package location and wasm-bindgen target for Node, or explicitly require CLI SVG to avoid JS WASM initialization by calling a native Rust path.

3. The render API accepts both `score` and `{ source }`, but the consistency contract between them is under-tested. Since layout rendering reparses `source` while VexFlow uses `score`, stale React closures or async parse/render races can render a source that does not correspond to the displayed parser state. The proposal should require either a source/version token carried with the parsed score or an acceptance test that changes DSL rapidly and proves the layout render uses the exact source associated with the active score.

4. The browser/network audit needs an explicit trigger boundary for "before preview rendering invokes the layout path." Because the default renderer is the layout engine, normal app startup may immediately render a preview for persisted DSL. If that is acceptable, the test cannot also assert that layout WASM is absent after app mount unless preview rendering is disabled or delayed. The proposal should define the audited scenario precisely, for example startup with preview inactive or before the preview effect runs, then first active preview render. Otherwise implementers can satisfy or fail the assertion depending on timing rather than architecture.

5. Static import enforcement is too narrow. v1.1 blocks `src/wasm/drummark_wasm.ts` and `src/wasm/pkg/drummark_core`, but the new split also needs to prevent browser production code from importing Node wrappers and prevent parser-facing production code from importing `layout_wasm_browser`. The static test should cover forbidden directionality across all new wrappers, not just the old combined package.

6. The acceptance criteria still do not require a parser/layout semantic parity fixture for successful parses. The Author Response promises parity for parser failures and normalized semantics, but v1.1 only lists declaration, lazy-load, settings, and CLI criteria. Add an explicit criterion for a shared corpus where parser WASM normalization and layout WASM source-to-scene normalization agree on measure count, barline/repeat/navigation semantics, and representative timing constructs. This is the main protection against duplicated parser/normalizer code drifting immediately after the split.

7. Size reporting should distinguish cache-cold transfer from cached repeat render. The addendum asks for transfer sizes for initial load, first layout render, and first legacy render, which is good, but first legacy render after default layout may include already-cached shared chunks. The audit should specify a fresh context per scenario or clearly report cumulative versus incremental transfer; otherwise the VexFlow lazy-load data can be misleading.

These are fixable specification gaps, but they are not merely task-file details because they define the package topology and observable lazy-load contract that implementation must target.

STATUS: CHANGES_REQUESTED

### Review Round 3

v1.2 closes the substantive Round 2 architecture blockers:

1. Cargo feature mechanics are now explicit. The proposal requires `--no-default-features`, cfg-gated parser/layout exports, optional `drummark-layout` from the parser build's perspective, and a build failure if parser bindings or dependency analysis show layout leakage.
2. Node/CLI topology is now concrete enough to implement. The proposal names `wasm-pack --target nodejs`, `src/wasm/layout-pkg-node/`, and `src/wasm/layout_wasm_node.ts`, and it explicitly keeps CLI SVG TypeScript-owned for this proposal.
3. Source/score coherence is now a first-class invariant. The `score`, `source`, and `sourceRevision` tuple plus the rapid-edit test close the stale-closure/race concern.
4. The network audit now has controlled scenarios, including preview suspension before renderer invocation, fresh-context default layout render, fresh-context legacy render, and legacy render after layout assets are cached.
5. Static import boundaries now cover old combined packages, browser-to-Node leaks, parser-facing layout imports, and CLI-to-browser leaks.
6. Successful and failing parser/layout semantic parity are now explicit acceptance requirements, with enough fixture coverage to catch immediate drift in duplicated parser/normalizer logic.
7. Size reporting now distinguishes raw/gzip/brotli assets, cache-cold transfer, incremental transfer, and cumulative scenario transfer, including fresh and post-layout legacy render measurements.

I do not see remaining proposal-level technical blockers in the v1.2 content. The implementation task file still needs to turn these constraints into independently testable tasks, but that is the next protocol phase rather than a reason to block this proposal's architecture.

There is, however, a ledger blocker. The physical file order is not linear: the Author Response to Round 2 and `## Addendum v1.2` appear before the existing v1.1 / Review Round 2 block, instead of being appended after Round 2. Because the protocol requires strict physical append and prohibits inserting content above an existing header, this file is not yet approval-ready as a historical record. Since earlier content must not be edited or moved, the repair should also be append-only: append an author response after this review that explicitly acknowledges the ordering error, identifies the out-of-order v1.2 section as the intended response to Review Round 2, and appends a clean, current proposal addendum or consolidated proposal text at the physical end before requesting another review.

STATUS: CHANGES_REQUESTED

### Review Round 4

I reviewed the final Author Response and `## Addendum v1.3` as the intended current proposal.

On proposal substance, v1.3 is approval-ready. It restates the split as an explicit implementation contract rather than an aspirational direction:

1. The Rust/WASM topology is concrete enough to implement and verify. It names web and Node package directories, requires `--no-default-features`, cfg-gated exports, and a build failure if the parser package exposes or links layout code.
2. The browser, parser-facing, and CLI import boundaries are enforceable. v1.3 blocks the old combined wrapper in production browser code, Node packages in browser code, layout wrappers in parser-facing code, and browser packages in CLI runtime.
3. The render API now carries `source` and `sourceRevision` with the active parsed score state, which closes the stale-source race identified earlier.
4. The network audit scenarios are specific enough to avoid timing-dependent interpretations of "before preview rendering." The required cold, incremental, and cumulative labels should produce actionable size data.
5. Parser/layout semantic parity is now explicit for both successful parses and parse failures, with coverage over the right musical structures.
6. CLI SVG ownership is no longer ambiguous: it remains TypeScript-owned and uses a Node layout WASM wrapper for this proposal.

I do not see a remaining architecture-level blocker in v1.3 itself. The next tasks-file phase still needs to decompose this into independently testable tasks, especially around the build script, static import enforcement, network audit, and parity corpus, but those are planning requirements rather than proposal defects.

The ledger repair is not acceptable yet. The Author Response says the current authoritative proposal is "restated below at the physical end of the file," but in the actual file `### Author Response` and `## Addendum v1.3` appear immediately after Review Round 1 and before the existing v1.1, Review Round 2, and Review Round 3 sections. That means the attempted repair repeated the same class of physical-order problem: it did not append the repair after the latest review at the true end of the ledger.

Because prior content must not be moved or edited, the acceptable repair remains append-only: append a new Author Response after this Review Round 4 that acknowledges v1.3 was also placed out of physical order, identifies the line-order mistake, and appends a clean current proposal addendum or consolidated current proposal text at the actual end of the file. Once the current proposal text is physically after all prior review rounds, I would expect approval unless the new appended text materially changes the architecture.

STATUS: CHANGES_REQUESTED

### Author Response

The ledger finding is accepted. The attempted v1.3 and v1.4 repairs were inserted after earlier duplicated review text rather than after the latest review at the actual end of the file. This response is appended after the current final Review Round 4 and is the valid repair placed after all existing review rounds.

The technical content is unchanged. The current authoritative proposal is restated below as v1.5 at the physical end of the ledger.

## Addendum v1.5: Current Proposal for Split Parser/Layout WASM and Default Layout Rendering

This addendum supersedes v1.0, v1.1, v1.2, v1.3, and v1.4 as the implementation target while preserving the earlier ledger history.

### Goals

- Make the layout engine the default preview renderer for users without an explicit saved renderer preference.
- Keep VexFlow available as a lazy legacy renderer.
- Split browser WASM into parser and layout packages so startup does not load layout code before a layout render is requested.
- Keep CLI SVG output working.
- Measure real browser transfer costs, not only raw `.wasm` file sizes.

### Rust and WASM Package Topology

Use the existing Rust WASM crate with explicit feature-gated builds.

Required build commands:

- browser parser package: `wasm-pack build --target web --no-default-features --features parser-wasm`
- browser layout package: `wasm-pack build --target web --no-default-features --features layout-wasm`
- Node layout package: `wasm-pack build --target nodejs --no-default-features --features layout-wasm`
- Node parser package: `wasm-pack build --target nodejs --no-default-features --features parser-wasm`, only if needed by tests or CLI parser initialization

Generated package directories:

- `src/wasm/parser-pkg-web/`
- `src/wasm/layout-pkg-web/`
- `src/wasm/layout-pkg-node/`
- `src/wasm/parser-pkg-node/`, only if needed

`drummark-layout` must be optional from the parser build perspective and enabled only by `layout-wasm` or an equivalent non-parser feature. Parser exports must be cfg-gated to parser builds. Layout exports must be cfg-gated to layout builds. Shared helper code may compile for both only when it does not pull `drummark-layout` into the parser build.

The build script must fail if parser package generated bindings expose layout exports or if dependency analysis shows `drummark-layout` linked into the parser package.

### TypeScript Entrypoints

Browser production code must use:

- `src/wasm/parser_wasm_browser.ts`
- `src/wasm/layout_wasm_browser.ts`

Node and CLI code must use:

- `src/wasm/layout_wasm_node.ts`
- `src/wasm/parser_wasm_node.ts`, only if Node parser WASM is required

Browser wrappers import only `*-pkg-web`. Node wrappers import only `*-pkg-node`.

The generic `src/wasm/drummark_wasm.ts` wrapper must not remain an active browser production dependency. It may be deleted or narrowed to a test-only compatibility shim.

### Layout Render API and Source Coherence

The layout render path accepts raw source explicitly at call time:

```ts
renderScoreToSvg(score, settings, { source, sourceRevision })
renderScorePagesToSvgs(score, settings, { source, sourceRevision })
```

The exact function names may follow local style, but production rendering must not use `setLayoutSource` or any module-level source cache.

The app must associate parsed score state with the exact source string and a monotonically increasing source revision:

```ts
{
  score,
  source,
  sourceRevision
}
```

If parsing is asynchronous, stale parse results must not replace newer score/source revisions. The active preview render receives the source attached to the active parsed score revision, not an independently captured editor string.

For this proposal, the layout package owns a `source -> LayoutScene` boundary. Duplicate parser/normalizer code inside the lazy layout package is acceptable because it does not affect startup.

### Default Renderer and Settings Migration

The layout engine becomes the default renderer for users without an explicit saved preference.

Migration rule:

- no saved settings: `useLayoutEngine` defaults to `true`
- saved settings without an own `useLayoutEngine` property: `useLayoutEngine` defaults to `true`
- saved settings with own `useLayoutEngine: false`: preserve `false`
- saved settings with own `useLayoutEngine: true`: preserve `true`
- corrupt saved settings: fall back to default settings with `useLayoutEngine: true`

Implementation must use explicit own-property detection equivalent to:

```ts
Object.prototype.hasOwnProperty.call(parsed, "useLayoutEngine")
```

User-facing labels should be `Layout Engine` and `Legacy VexFlow`, not implementation names.

### Renderer-Neutral Shared Options

Shared setting ranges, render settings types, and page layout option definitions used outside the VexFlow renderer must live in renderer-neutral modules.

Default app code may type-import from VexFlow modules only when erased at build time. Runtime imports from `src/vexflow/*` in default layout settings, app initialization, or layout renderer code are not allowed unless bundle evidence proves VexFlow runtime remains lazy.

### CLI Contract

`npm run drummark -- <input> --format svg` must keep working.

CLI SVG output remains TypeScript-owned for this proposal. The CLI layout path must initialize layout WASM through `src/wasm/layout_wasm_node.ts` and `src/wasm/layout-pkg-node/`. This proposal does not move CLI SVG rendering to a native Rust executable.

### Static Import Boundaries

Static enforcement must fail on production browser imports of:

- `src/wasm/drummark_wasm.ts`
- `src/wasm/pkg/drummark_core`
- `src/wasm/*_wasm_node.ts`
- `src/wasm/*-pkg-node`

Static enforcement must fail on parser-facing imports of:

- `src/wasm/layout_wasm_browser.ts`
- `src/wasm/layout-pkg-web`
- `src/wasm/layout_wasm_node.ts`
- `src/wasm/layout-pkg-node`

Static enforcement must fail on CLI imports of browser-only wrappers or browser generated packages.

Tests may intentionally import both parser and layout wrappers only when the file path or test name makes the integration/parity purpose explicit.

### Network and Size Verification

The build and audit must report:

- raw asset size
- gzip asset size
- brotli asset size
- cache-cold browser transfer
- incremental browser transfer after a named prior scenario
- cumulative browser transfer for the scenario

Required browser/network scenarios:

1. Initial app load with preview inactive or rendering deliberately suspended before renderer invocation.
   - Must fetch parser WASM.
   - Must not fetch layout WASM.
   - Must not fetch VexFlow chunk.

2. First default layout preview render in a fresh browser context.
   - Must fetch parser WASM as needed.
   - Must fetch layout WASM.
   - Must not fetch VexFlow chunk.
   - Must report cumulative transfer and layout-render incremental transfer.

3. First legacy VexFlow render in a fresh browser context.
   - Must fetch parser WASM as needed.
   - Must fetch VexFlow chunk.
   - Must report cumulative transfer and VexFlow incremental transfer.

4. Legacy VexFlow render after an already completed default layout render.
   - Must report incremental transfer after cached parser/layout assets.

The preview suspension mechanism for scenario 1 may be a test route, query parameter, or test-only setting, and must not alter normal production behavior.

If the repo lacks suitable browser automation, tasks may add a development dependency for this audit.

### Semantic Parity Verification

A shared corpus must verify successful-parse parity between parser WASM and the layout WASM source-to-scene path.

The parity check must cover:

- measure count
- barline and repeat markers
- navigation markers such as endings, coda, segno, fine, and D.C./D.S. when present in fixtures
- timing constructs including time signatures, rests, subdivisions, tuplets if currently supported, beams, and multi-measure input
- representative notation from existing renderer regression tests

The comparison does not need exact SVG coordinates. It must assert that the layout scene structural interpretation agrees with parser/normalizer output on the musical document shape.

Parse-failure parity must also be tested.

### Acceptance Criteria

- Parser package declarations contain no layout exports.
- Layout package declarations expose the layout scene builder expected by the layout wrapper.
- Parser build uses `--no-default-features --features parser-wasm`.
- Layout browser and Node builds use `--no-default-features --features layout-wasm`.
- Parser build does not link `drummark-layout`.
- Default app rendering no longer imports or calls `setLayoutSource`.
- Browser production code imports only browser wrappers and web generated packages.
- CLI runtime imports only Node wrappers and Node generated packages for WASM initialization.
- Active parsed score state carries `source` and `sourceRevision`.
- Rapid-edit rendering test proves layout source and active score revision stay coherent.
- Settings migration tests cover no saved settings, old saved settings without `useLayoutEngine`, explicit `false`, explicit `true`, and corrupt JSON.
- Static import tests enforce the forbidden boundaries listed above.
- Network audit reports all four required scenarios with cold and incremental transfer labels.
- Shared parser/layout parity corpus covers successful parses and parse failures.
- `npm run drummark -- <input> --format svg` passes on at least one representative fixture.
- `npm run build` succeeds and reports parser/layout WASM sizes separately.
- VexFlow remains available as a lazy legacy renderer.

### Review Round 5

Ledger placement after the current authoritative v1.5 is acceptable at this physical end-of-file location. Note: an earlier append attempt also added a Review Round 5 after an older duplicated v1.4 block because that block repeated the same acceptance-criteria tail; that earlier insertion is now part of the historical ledger and must remain untouched. This section is the operative Review Round 5 because it is physically after the actual EOF-appended Author Response and `## Addendum v1.5`.

The v1.5 repair itself is acceptable. The Author Response and addendum are physically after the latest prior Review Round 4 and its status line, and the response explicitly acknowledges that v1.3/v1.4 were previously placed out of order. That satisfies the append-only repair requirement: the flawed history remains visible, and the current implementation target now appears after all prior review rounds.

The proposal remains technically approval-ready. v1.5 preserves the key constraints needed for a sound implementation: feature-gated parser/layout WASM builds using `--no-default-features`, separate web and Node generated packages, explicit parser-build failure if layout exports or `drummark-layout` leak in, replacement of module-level layout source state with per-render `source` and `sourceRevision`, default layout migration that preserves explicit legacy choices, renderer-neutral shared settings boundaries, TypeScript-owned CLI SVG output through Node layout WASM, static import enforcement, browser transfer audits, and parser/layout semantic parity for successful and failed parses.

The remaining risk belongs in the tasks phase: the tasks file must split build generation, wrapper replacement, static checks, network audit, source-coherence tests, and parity fixtures into independently testable units with clear input/output contracts. That is not a proposal blocker.

STATUS: APPROVED
