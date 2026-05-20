## Addendum v1.0: Remove Legacy VexFlow Renderer

### Status

Proposed.

This addendum defines the final removal path for VexFlow after the platform-neutral layout engine became the default rendering route.

### Problem

The repository still carries VexFlow as a lazy legacy renderer even though the approved architecture is:

`source -> parser AST -> normalized musical model -> RenderScore -> LayoutScene -> platform adapter`

Current VexFlow remnants create three forms of drag:

- **Runtime ambiguity**: the app still exposes a renderer choice and can dynamically import `src/vexflow`.
- **Build and dependency weight**: `vexflow` remains in `package.json`, Vite dependency optimization, and generated bundles.
- **Test oracle coupling**: some layout parity tests still compare layout output against VexFlow SVG summaries, which keeps VexFlow behavior as an implicit authority.

The repository also still has process text that says score rendering must be handled by VexFlow. That rule is now inconsistent with the approved layout-engine architecture and must be superseded before implementation.

### Goal

Delete VexFlow from the product, dependency graph, and active test strategy.

After this change:

- all app preview rendering uses the layout-engine SVG adapter
- CLI SVG rendering uses the Node layout WASM package
- docs/example rendering uses the layout-engine route
- no production or build script imports `src/vexflow` or `vexflow/bravura`
- no package dependency on `vexflow` remains
- layout correctness is owned by `RenderScore`, `LayoutScene`, adapter snapshots, corpus gates, and CLI SVG checks, not by VexFlow parity

### Non-Goals

This proposal does not:

- redesign `RenderScore`
- expand the supported DrumMark notation surface
- rewrite the layout engine
- require pixel parity with historical VexFlow output
- remove MusicXML export
- remove Bravura or SMuFL assets used by the layout adapter
- introduce manual renderer-side engraving logic outside the approved thin adapter contract

### Superseded Rendering Rule

The old rule:

> All score rendering must be handled exclusively by VexFlow.

is superseded by:

> All score layout decisions must be handled by `drummark-layout` through `RenderScore -> LayoutScene`. Platform adapters may only translate resolved scene geometry into drawing commands, glyphs, paths, text, and accessibility/event metadata. They may not perform spacing, collision resolution, span reconstruction, or notation-specific layout fixes.

The existing graceful-failure rule remains:

- if layout or adapter rendering cannot render a supported input, the UI and CLI must fail closed with an explicit error
- adapters must not draw placeholder score elements to hide layout failures

### Current VexFlow Surfaces To Retire

Known surfaces include:

- `src/vexflow/**`
- dynamic import of `./vexflow` in `src/App.tsx`
- user-facing `Legacy VexFlow` setting and i18n keys
- VexFlow docs path in `build-docs.ts`
- Vite dependency optimization entry for `vexflow`
- `vexflow` package dependency
- VexFlow-specific unit, smoke, probe, and helper tests
- corpus/parity tests that import VexFlow as an oracle
- docs ledgers or reports whose only purpose is VexFlow divergence tracking

Implementation must verify the actual list with `rg "vexflow|VexFlow|vexflow/bravura"` before editing.

### Migration Gates

VexFlow may be removed only after all of these gates pass:

1. **Default layout route is authoritative**
   - app preview renders through `renderScorePagesToSvgs` from `src/renderer/svgRenderer`
   - CLI SVG output renders through `src/renderer/svgRendererNode`
   - docs example rendering uses the layout route and does not instantiate the legacy renderer

2. **No saved setting traps**
   - a saved legacy-renderer preference does not break startup
   - settings migration maps legacy VexFlow preference to the layout engine or ignores it safely
   - user-facing renderer toggle copy is removed or replaced with layout-neutral status copy

3. **Test oracle replacement**
   - scene snapshots and corpus reports remain the stable layout oracle
   - VexFlow divergence reports are either archived or converted into historical notes
   - no active test imports `../vexflow` or `vexflow/bravura`
   - removed VexFlow tests are replaced only where they covered behavior still not asserted by layout scene, adapter, CLI, or corpus tests

4. **Import and bundle boundary cleanup**
   - `package.json`, lockfile, and Vite config no longer reference VexFlow
   - production import-boundary checks prove no VexFlow runtime modules are reachable
   - bundle report confirms VexFlow is absent from built output

5. **Rendering verification**
   - representative examples render via `npm run drummark -- <fixture> --format svg`
   - full split-WASM verification passes or is updated to an equivalent no-VexFlow gate
   - docs generation succeeds through the layout route

### Test Strategy After Removal

The stable rendering test stack becomes:

- `RenderScore` derivation tests for parser/normalizer ownership
- `LayoutScene` snapshot tests for resolved geometry and semantic composites
- SVG adapter tests for thin translation from scene to SVG
- corpus gate reports for supported DrumMark examples
- CLI SVG smoke tests for Node layout bootstrap
- browser/network audits for parser/layout WASM fetch boundaries
- targeted regression tests for each previously VexFlow-covered musical feature

Tests must not compare against VexFlow output after removal. Historical VexFlow reports may remain archived as migration evidence only.

### Implementation Boundaries

The implementation should avoid a broad renderer rewrite. The expected edits are deletion and route simplification:

- remove the legacy renderer branch from the app
- simplify settings state and persistence around renderer choice
- move docs rendering to the layout route
- delete VexFlow implementation files and VexFlow-only tests
- update import-boundary checks and corpus tests to a no-VexFlow model
- remove dependency and build config references
- update active docs/process text that still mandates VexFlow

No new renderer-side positioning rules should be added to compensate for missing VexFlow behavior. If a score feature fails after removal, the fix belongs in `RenderScore`, `drummark-layout`, or a thin adapter translation bug, depending on where the contract is violated.

### Risks

1. **Hidden oracle loss**
   - Some VexFlow tests may cover musical features not yet covered by layout-scene or adapter tests. Deleting them without replacement would reduce regression coverage.
   - Mitigation: classify each VexFlow-only test before deletion as obsolete, covered elsewhere, or requiring a new layout/adapter regression.

2. **Saved preference regressions**
   - Users may have local storage that requests the legacy renderer.
   - Mitigation: migrate or ignore that value and keep the app on layout rendering.

3. **Docs route drift**
   - `build-docs.ts` currently imports the VexFlow renderer directly.
   - Mitigation: route docs through the same layout rendering API used by CLI/app where practical.

4. **Process contradiction**
   - `AGENTS.md` still says VexFlow owns all score rendering.
   - Mitigation: update process docs during implementation to match this approved addendum after human stamp and consolidation.

### Acceptance Criteria

Removal is complete when:

- `rg "vexflow|VexFlow|vexflow/bravura" src build-docs.ts vite.config.ts package.json package-lock.json docs AGENTS.md` finds no active production, build, or current-process dependency references except archived historical notes
- `npm test` passes after obsolete VexFlow tests are removed or replaced
- `npm run build` passes and generated bundle output does not include VexFlow
- `npm run drummark -- docs/examples/overview.drum --format svg` succeeds
- docs generation renders examples through the layout engine
- local storage or settings values that previously selected VexFlow do not prevent app rendering
- `LEARNINGS.md` and active architecture docs identify VexFlow as removed, not legacy-available

### Proposed End State

The repository has one score-rendering architecture:

`RenderScore -> LayoutScene -> thin platform adapter`

VexFlow is not a dependency, not a fallback, not a test oracle, and not a user-visible option.

### Review Round 1

1. **The corpus-gate replacement is under-specified and would leave a dead test contract.**
   The proposal says VexFlow divergence reports are archived or converted into historical notes, but `src/renderer/corpusGate.test.ts` currently has two distinct responsibilities: stable `LayoutScene` snapshots and an `oracleReport` comparison against VexFlow summaries stored in `docs/layout-corpus/corpus_gate_report.json`. Removing VexFlow without defining what happens to `expected.oracleReport` creates a migration deadlock: either the test is deleted and loses coverage, or the report schema still carries a VexFlow-owned field with no producer. The addendum needs a concrete post-removal corpus report shape, for example "delete `oracleReport` from active reports and replace it with layout-only SVG semantic summaries" or "freeze it only in archived evidence, never loaded by active tests."

2. **The saved-setting gate conflicts with current behavior and tests.**
   `src/hooks/useAppSettings.test.ts` currently asserts that an explicit legacy preference is preserved (`useLayoutEngine: false`). The proposal says a saved legacy preference maps to layout or is ignored safely, but it does not explicitly require changing the persistence schema or test expectation. That ambiguity matters because keeping the boolean while deleting the renderer branch can still leak a false value into debug UI, serialized settings, or future conditionals. Require `resolveAppSettings` to coerce any saved legacy renderer value to the layout route, update/remove the preservation test, and ensure the saved settings rewrite does not keep advertising `useLayoutEngine: false` as a supported state.

3. **Build-surface cleanup misses `tsconfig.app.json` and stale generated output.**
   The proposal names `package.json`, lockfile, and Vite config, but this repo also has a TypeScript path alias for `"vexflow"` in `tsconfig.app.json`. Leaving that alias behind keeps VexFlow as a first-class compile-time surface even after dependency removal. There is also already a generated `dist/assets/vexflow-*.js`; acceptance says generated bundle output must not include VexFlow, but the search gate does not include `dist` or specify whether stale generated artifacts must be removed before verification. Add both surfaces explicitly: no TypeScript path alias for VexFlow, and either clean/rebuild `dist` or remove stale generated VexFlow chunks before declaring the bundle gate passed.

4. **The import-boundary gate is not concrete enough to catch dynamic legacy imports.**
   Existing import-boundary tests only enforce WASM split rules. The proposal says production import-boundary checks should prove no VexFlow runtime modules are reachable, but does not require a new rule or define its scope. A plain `rg` gate is useful but weaker than the proposed boundary check, especially around dynamic imports such as `import("./vexflow")`. Require a production no-VexFlow import-boundary rule that scans active production source and build scripts for static and dynamic imports resolving to `src/vexflow`, `./vexflow`, `../vexflow`, or `vexflow/bravura`, with archived docs/tests excluded deliberately rather than accidentally.

5. **Docs rendering migration has an API mismatch that should be resolved in design, not during implementation.**
   `build-docs.ts` currently renders from a `NormalizedScore` through `src/vexflow/index`, while the Node layout route exposed by `src/renderer/svgRendererNode.ts` renders from source (`renderSourceToSvgNode`) and initializes Node layout WASM. The proposal says to route docs through the same layout rendering API "where practical", but does not decide whether docs should render from source directly, share CLI rendering options, or grow a normalized-score Node layout API. That ambiguity risks either duplicate bootstrap code in docs or a hidden behavior split from CLI. The proposal should name the intended docs API and option mapping, including title/page/margin defaults.

6. **The replacement-test requirement is too vague for the large VexFlow test surface.**
   `src/vexflow/render-probe.test.ts`, `smoke.test.ts`, `renderer.test.ts`, and helper tests cover many feature-specific behaviors: hairpins, multi-rests, measure repeats, tempo/header rendering, secondary rests, beaming/triplets, articulations, notehead mapping, and docs examples. The proposal says to classify tests as obsolete, covered, or needing replacement, but it does not require a checked-in classification artifact or minimum replacement matrix. Without that, implementation can delete high-value coverage with only a passing `npm test`. Require a migration ledger/table, probably in the tasks file or proposal appendix, mapping each VexFlow test file or behavior cluster to its successor assertion.

7. **Active proposal/process references need a sharper exception model.**
   The acceptance regex includes `docs` and `AGENTS.md` but allows "archived historical notes." The repo has active proposal/task files that still mention VexFlow, including platform-neutral layout tasks and current renderer proposals. Those are neither production code nor archived historical notes. The proposal should define whether active proposals may retain pre-removal VexFlow references, whether they must be superseded/archived before this implementation, and how `rg` gates distinguish current-process contradictions from legitimate history.

STATUS: CHANGES_REQUESTED

### Author Response

The review is correct. The v1.0 proposal identified the right target state but left too many implementation-critical gates as judgment calls. The following v1.1 amendments are binding and supersede weaker wording above.

#### 1. Corpus Gate Shape After VexFlow

Active corpus reports must become layout-owned and must not carry a VexFlow-derived `oracleReport`.

The post-removal active report shape is:

- `sceneReport`: summary of `LayoutScene` page/system/measure/item/composite counts and role/kind distributions
- `svgSemanticReport`: summary of layout-rendered SVG semantic roles and visible text tokens emitted by the thin adapter
- no `oracleReport` field
- no VexFlow summary fields

`docs/layout-corpus/vexflow_divergence_ledger.md` and the old `oracleReport` data may be archived as historical migration evidence, but active tests must not read them.

`src/renderer/corpusGate.test.ts` must be rewritten so it verifies:

- layout scene report stability
- representative scene snapshots
- layout SVG semantic summary stability

It must not import `../vexflow/renderer`, build a VexFlow SVG, or compare layout output against VexFlow output.

#### 2. Settings Migration Contract

`useLayoutEngine: false` stops being a supported saved state once VexFlow is removed.

Implementation must:

- update `resolveAppSettings()` so any saved legacy renderer value resolves to the layout route
- update or remove tests that currently assert explicit legacy preference preservation
- prevent UI/debug state from advertising `useLayoutEngine: false` as meaningful
- remove renderer toggle copy and i18n keys unless a non-renderer status label remains useful
- ensure new persisted settings do not write a legacy renderer preference back out

This supersedes the older split-WASM rule that explicit saved VexFlow preferences remain respected.

#### 3. Complete Build Surface Cleanup

The surfaces-to-retire list includes:

- `tsconfig.app.json` path aliases or type aliases for `vexflow`
- stale generated `dist` VexFlow chunks
- Vite optimization/manual-chunk entries for VexFlow
- package and lockfile entries for VexFlow

The bundle gate requires a clean rebuild. Before declaring success, implementation must remove stale generated VexFlow chunks or rebuild `dist` from a clean output directory so `dist/assets/vexflow-*.js` cannot remain as a false positive.

#### 4. Concrete No-VexFlow Import Boundary Rule

A dedicated no-VexFlow boundary check must be added or an existing boundary script must gain this rule.

The rule scans active production source, app build scripts, active tests, and current process docs as appropriate for:

- static imports from `src/vexflow`, `./vexflow`, `../vexflow`, or deeper VexFlow paths
- dynamic imports resolving to `src/vexflow`, `./vexflow`, or `../vexflow`
- package imports from `vexflow` or `vexflow/bravura`

Allowed exclusions must be explicit:

- `docs/archived/**`
- old generated build artifacts only before the clean-build gate runs
- historical notes that are not consumed by active tests or process rules

The rule must fail on accidental current-route references rather than relying only on manual `rg` inspection.

#### 5. Docs Rendering API

Docs rendering should use the Node layout source API, not grow a normalized-score VexFlow-shaped replacement.

`build-docs.ts` must render each `.drum` example by calling:

`renderSourceToSvgNode(source, docsRenderOptions)`

from `src/renderer/svgRendererNode.ts`.

The docs render options must be explicit and shared with the CLI defaults where practical:

- `staffScale`: current docs/default scale
- `pageWidth`: current docs/default page width
- `showTitle`: true for examples that include title metadata
- page margins, header spacing, system spacing, stem length, volta spacing, hairpin offset, secondary-rest visibility, duration spacing compression, and measure-width compression must come from `DEFAULT_RENDER_OPTIONS` or documented docs-specific overrides

Docs generation should not manually initialize VexFlow or construct a `NormalizedScore` solely for rendering.

#### 6. Required VexFlow Test Migration Matrix

Implementation must create a checked-in migration matrix before deleting VexFlow tests. The matrix may live in the approved tasks file or a companion appendix referenced by the tasks file.

Minimum rows:

- `src/vexflow/notes.test.ts`
- `src/vexflow/articulations.test.ts`
- `src/vexflow/renderer.test.ts`
- `src/vexflow/render-probe.test.ts`
- `src/vexflow/smoke.test.ts`
- `src/vexflow/renderer.test.ts`
- `src/renderer/vexflowParity.test.ts`
- `src/renderer/detailed_diff.test.ts`
- VexFlow portions of `src/renderer/corpusGate.test.ts`
- docs rendering smoke coverage formerly exercised through `build-docs.ts`

For each row or behavior cluster, the matrix must state one of:

- `obsolete`: behavior was VexFlow-internal and no longer part of the contract
- `covered`: name the replacement layout/adapter/CLI/corpus test
- `new regression required`: name the new test to add before deleting the old test

Behavior clusters that need explicit coverage decisions include:

- notehead and rest role mapping
- articulations/modifiers
- hairpins
- measure repeats
- multi-rests
- repeat and final barlines
- navigation markers
- voltas
- tempo/header rendering
- secondary voice rests
- beaming and triplets
- docs example rendering

#### 7. Active Proposal and Process Reference Model

Active docs must distinguish current process from historical proposal ledger.

Implementation must update current process and contract docs:

- `AGENTS.md`
- `docs/RENDER_LAYOUT_CONTRACT.md`
- `LEARNINGS.md`
- any active proposal/task file whose status would otherwise still prescribe future VexFlow work

Active proposal files may retain historical VexFlow references only when they are part of immutable review ledger history. If a still-active proposal or tasks file contains an uncompleted future task that targets VexFlow, this removal implementation must either:

- archive/supersede that proposal stream, or
- append a terminal note to that file stating that the VexFlow-targeted task is superseded by this removal proposal

Search gates must treat current-process contradictions as failures while allowing immutable archived history.

### Review Round 2

The v1.1 amendments resolve the Round 1 blockers without creating a new architectural deadlock.

1. **Corpus ownership is now concrete enough to implement.**
   The active report shape explicitly drops `oracleReport` and VexFlow summary fields, replaces them with `sceneReport` and `svgSemanticReport`, and forbids active tests from reading archived VexFlow evidence. That removes the earlier producer/consumer deadlock in `corpusGate.test.ts`.

2. **Settings migration now has a single supported runtime state.**
   The amendment makes `useLayoutEngine: false` unsupported, requires `resolveAppSettings()` coercion, removes preservation-test ambiguity, and prevents new persisted settings from writing a legacy renderer preference back out. This closes the prior leak where a deleted renderer branch could still survive as serialized state.

3. **Build cleanup now covers the hidden compile and artifact surfaces.**
   `tsconfig.app.json`, stale `dist` chunks, Vite entries, package metadata, and the clean rebuild requirement are named. The clean-output condition is important because otherwise an old generated chunk could make the bundle gate noisy or misleading.

4. **The import-boundary rule is now actionable.**
   The proposal names static imports, dynamic imports, package imports, and explicit exclusions. The phrase "as appropriate" around scanning active tests/current docs is slightly soft, but it does not block implementation because the required failing patterns and allowed exclusions are specific enough for a boundary script or test.

5. **Docs rendering has a chosen API.**
   Requiring `build-docs.ts` to call `renderSourceToSvgNode(source, docsRenderOptions)` avoids a VexFlow-shaped normalized-score replacement. The option list is sufficiently detailed for the tasks file to pin exact defaults and documented overrides.

6. **Coverage migration is no longer hand-wavy.**
   A checked-in matrix with minimum files and behavior clusters prevents silent deletion of valuable VexFlow-era tests. The duplicate `src/vexflow/renderer.test.ts` row is harmless editorial noise, not a semantic conflict.

7. **Historical references have an exception model.**
   The amendment distinguishes immutable review history from current-process contradictions and requires superseding active VexFlow-targeted tasks. That gives the implementation a clear way to preserve ledger history while still failing current-route references.

Remaining implementation pressure belongs in the tasks file review: ensure the no-VexFlow boundary check and migration matrix are their own independently testable tasks, and ensure the docs-rendering task verifies actual docs output, not just API compilation. These are planning requirements, not unresolved defects in the proposal text.

STATUS: APPROVED

### Consolidated Changes

The approved removal architecture is:

`RenderScore -> LayoutScene -> thin platform adapter`

VexFlow is removed as a product renderer, fallback path, dependency, and active test oracle. The layout engine owns all score layout decisions through `RenderScore -> LayoutScene`; platform adapters may only translate resolved scene geometry into drawing commands, glyphs, paths, text, and accessibility/event metadata.

Implementation must remove:

- app preview dynamic import and renderer selection for `src/vexflow`
- saved-setting support for `useLayoutEngine: false`
- user-facing `Legacy VexFlow` strings and renderer toggle UI
- docs rendering through `src/vexflow/index`
- VexFlow corpus/parity oracle tests and active divergence-ledger consumption
- `src/vexflow/**`
- `vexflow` package dependency, Vite references, TypeScript path aliases, and stale generated chunks

Implementation must replace VexFlow-era coverage with layout-owned verification:

- final coverage migration matrix before deleting VexFlow tests
- replacement tests for any behavior not already covered by `RenderScore`, `LayoutScene`, SVG adapter, CLI SVG, or corpus reports
- active corpus report shape with `sceneReport` and `svgSemanticReport`, and no `oracleReport`
- no-VexFlow import-boundary rule that catches active static and dynamic imports
- final split-WASM/no-VexFlow verification, build, tests, docs build, and representative CLI SVG rendering

Current process docs must be updated so they no longer instruct agents or implementers that score rendering belongs to VexFlow. Active proposal/task streams with uncompleted VexFlow-targeted work must receive terminal supersession notes or be archived, while immutable historical ledger text remains preserved.
