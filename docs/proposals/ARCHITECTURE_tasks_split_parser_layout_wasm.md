## Tasks: Split Parser/Layout WASM and Default Layout Rendering

### Task 1: Define Split WASM Build Topology
- [x] **Status**: Done
- **Scope**: `crates/drummark-core/Cargo.toml`, Rust WASM export cfg gates, `scripts/build_wasm.mjs`, generated package paths
- **Commits**:
  - `build(wasm): split parser and layout wasm package builds`
  - `test(wasm): enforce parser package excludes layout exports`
- **Acceptance Criteria**:
  - `scripts/build_wasm.mjs` builds `src/wasm/parser-pkg-web/`, `src/wasm/layout-pkg-web/`, and `src/wasm/layout-pkg-node/`
  - Parser build command uses `--target web --no-default-features --features parser-wasm`
  - Layout browser build command uses `--target web --no-default-features --features layout-wasm`
  - Layout Node build command uses `--target nodejs --no-default-features --features layout-wasm`
  - `drummark-layout` is optional from the parser build's dependency graph
  - Parser generated declarations contain no layout exports
  - Build script reports raw, gzip, and brotli sizes for parser and layout WASM assets
  - Running the build script alone verifies the generated package directories and declaration boundaries
- **Dependencies**: None

### Task 2: Add Explicit Browser and Node WASM Wrappers
- [x] **Status**: Done
- **Scope**: `src/wasm/`, wrapper tests, test setup imports
- **Commits**:
  - `feat(wasm): add explicit parser and layout wasm wrappers`
  - `test(wasm): initialize split web and node wasm wrappers`
- **Acceptance Criteria**:
  - Browser parser wrapper imports only `src/wasm/parser-pkg-web/`
  - Browser layout wrapper imports only `src/wasm/layout-pkg-web/`
  - Node layout wrapper imports only `src/wasm/layout-pkg-node/`
  - Node parser wrapper exists only if needed and imports only `src/wasm/parser-pkg-node/`
  - Wrapper tests prove parser initialization can parse without importing layout wrapper code
  - Wrapper tests prove layout initialization can produce a `LayoutScene` from source
  - Generic `src/wasm/drummark_wasm.ts` is deleted or reduced to a test-only compatibility shim
- **Dependencies**: Task 1

### Task 3: Introduce Static Import Boundary Enforcement
- [ ] **Status**: Pending
- **Scope**: test utilities or scripts, production source import scanning, CI/local test command integration
- **Commits**:
  - `test(wasm): enforce split wasm import boundaries`
- **Acceptance Criteria**:
  - Static test fails if browser production source imports `src/wasm/drummark_wasm.ts`, `src/wasm/pkg/drummark_core`, Node wrappers, or Node generated packages
  - Static test fails if parser-facing production source imports layout wrappers or layout generated packages
  - Static test fails if CLI runtime imports browser-only wrappers or browser generated packages
  - Integration/parity tests may import both parser and layout wrappers only from explicitly named integration/parity test files
  - The static test can run without invoking WASM builds
- **Dependencies**: Task 2

### Task 4: Carry Source Revision Through Parsed Score State
- [x] **Status**: Done
- **Scope**: `src/App.tsx`, worker message types, parser state types, rapid-edit tests
- **Commits**:
  - `feat(app): bind parsed scores to source revisions`
  - `test(app): prevent stale source renders during rapid edits`
- **Acceptance Criteria**:
  - Active parsed score state carries `score`, `source`, and monotonically increasing `sourceRevision`
  - Async parse results older than the current revision cannot replace newer active score state
  - Layout rendering receives the source attached to the active parsed score revision
  - No production render path reads a module-level layout source cache
  - A rapid-edit test proves the rendered layout uses the source associated with the final accepted score
- **Dependencies**: Task 2

### Task 5: Make Layout Engine the Default with Settings Migration
- [x] **Status**: Done
- **Scope**: settings defaults/loaders, settings UI labels, i18n keys if labels are user-facing, migration tests
- **Commits**:
  - `feat(settings): default preview rendering to layout engine`
  - `test(settings): preserve explicit legacy renderer preferences`
- **Acceptance Criteria**:
  - Users with no saved settings default to `useLayoutEngine: true`
  - Saved settings without an own `useLayoutEngine` property default to `true`
  - Saved settings with own `useLayoutEngine: false` preserve `false`
  - Saved settings with own `useLayoutEngine: true` preserve `true`
  - Corrupt saved settings fall back to layout-engine default
  - Migration uses explicit own-property detection for `useLayoutEngine`
  - User-facing renderer labels use `Layout Engine` and `Legacy VexFlow`
- **Dependencies**: None

### Task 6: Move Shared Render Options Out of VexFlow Modules
- [x] **Status**: Done
- **Scope**: shared renderer option modules, VexFlow imports, app/settings imports, type-only cleanup
- **Commits**:
  - `refactor(renderer): move shared render options to neutral modules`
  - `test(renderer): keep VexFlow runtime out of default settings path`
- **Acceptance Criteria**:
  - Shared setting ranges, render settings types, and page layout option definitions used outside VexFlow live in renderer-neutral modules
  - Default app/settings/layout-renderer production code has no runtime import from `src/vexflow/*`
  - Any remaining VexFlow imports outside the VexFlow renderer are type-only and erased at build time
  - Static or bundle inspection test proves VexFlow runtime is not pulled into the default layout settings path
- **Dependencies**: None

### Task 7: Convert Layout SVG Rendering to Explicit Source Input
- [x] **Status**: Done
- **Scope**: `src/renderer/svgRenderer.ts`, layout scene adapter call sites, CLI runtime, renderer tests
- **Commits**:
  - `feat(renderer): pass source explicitly to layout svg renderer`
  - `fix(cli): initialize layout wasm through node wrapper`
- **Acceptance Criteria**:
  - Production renderer APIs accept explicit `source` and `sourceRevision` for layout rendering
  - `setLayoutSource` and module-level source cache are removed from production code
  - Browser layout rendering initializes through `layout_wasm_browser`
  - CLI SVG rendering initializes through `layout_wasm_node`
  - `npm run drummark -- <representative fixture> --format svg` succeeds
  - Existing layout adapter regression tests pass through the new explicit-source API
- **Dependencies**: Tasks 2, 4

### Task 8: Add Parser/Layout Semantic Parity Corpus
- [ ] **Status**: Pending
- **Scope**: shared fixtures, parser/layout parity tests, structural comparison helpers
- **Commits**:
  - `test(wasm): compare parser and layout wasm semantics`
- **Acceptance Criteria**:
  - Shared corpus covers measure count, barlines, repeats, navigation markers, timing constructs, beams, and multi-measure input
  - Successful-parse parity compares parser/normalizer output with layout scene structural interpretation
  - Parse-failure parity compares parser-facing diagnostics with layout-facing failure behavior
  - Comparison avoids exact SVG coordinate assertions
  - Corpus includes representative notation from existing renderer regression tests
- **Dependencies**: Task 2

### Task 9: Add Browser Network and Transfer Audit
- [ ] **Status**: Pending
- **Scope**: browser automation dependency if needed, production preview audit script, test route/query for preview suspension, size report output
- **Commits**:
  - `test(bundle): audit parser layout and legacy renderer network loads`
- **Acceptance Criteria**:
  - Audit runs against a production build preview
  - Scenario 1 uses preview suspension before renderer invocation and verifies parser WASM loads while layout WASM and VexFlow do not
  - Scenario 2 uses a fresh context for first default layout render and reports cumulative plus layout incremental transfer
  - Scenario 3 uses a fresh context for first legacy VexFlow render and reports cumulative plus VexFlow incremental transfer
  - Scenario 4 measures legacy VexFlow render after default layout assets are cached
  - Report labels raw, gzip, brotli, cache-cold transfer, incremental transfer, and cumulative transfer distinctly
  - Audit fails if VexFlow is fetched on the default layout render path
- **Dependencies**: Tasks 5, 6, 7

### Task 10: Consolidate Verification and Build Gates
- [ ] **Status**: Pending
- **Scope**: npm scripts, local verification docs, final test command set
- **Commits**:
  - `chore(test): wire split wasm verification gates`
- **Acceptance Criteria**:
  - `npm run build` succeeds and reports parser/layout WASM sizes separately
  - Targeted parser/layout wrapper tests pass
  - Settings migration tests pass
  - Static import boundary tests pass
  - Parser/layout semantic parity tests pass
  - Browser network audit passes
  - `npm run drummark -- <representative fixture> --format svg` succeeds
  - VexFlow remains available as a lazy legacy renderer
- **Dependencies**: Tasks 1, 2, 3, 5, 7, 8, 9

### Task 11: Consolidate Proposal Into Architecture Docs
- [ ] **Status**: Pending
- **Scope**: proposal file, canonical architecture/spec docs, archival after merge
- **Commits**:
  - `docs(wasm): consolidate split parser layout architecture`
- **Acceptance Criteria**:
  - Proposal file receives appended `### Consolidated Changes`
  - Canonical architecture/spec documentation receives the approved clean addendum append-only
  - Proposal and tasks files remain active until implementation branch passes pre-merge review
  - After reviewed merge, proposal and tasks files move to `docs/archived/`
- **Dependencies**: Task 10

### Review Round 1

I reviewed this tasks file against the operative approved proposal, `## Addendum v1.5`, and its approving `### Review Round 5`. The plan is close in coverage, but it is not yet safe to approve because several tasks are not independently implementable/testable in the order written, and the consolidation step conflicts with the repository's proposal workflow.

1. The tasks do not include explicit input/output contracts, even though the Task Independence Rule requires every task to state what data goes in and what comes out. Most tasks have scope, commits, acceptance criteria, and dependencies, but no task-level contract. This matters here because several tasks operate on the same shared surfaces: generated WASM packages, wrapper modules, app parsed-score state, renderer APIs, and network audit output. The tasks should add an explicit `Input/Output Contract` or equivalent field. Examples: Task 4 should define input as parser worker results tagged with source/revision and output as active parsed score state; Task 8 should define input as shared fixtures plus parser/layout wrapper outputs and output as structural parity assertions; Task 9 should define input as a production preview URL plus four named scenarios and output as a request/transfer ledger.

2. Task 11 is ordered after implementation verification, but the repo protocol requires consolidation after proposal/tasks approval and human stamp, before implementation begins. The current Task 11 says the canonical architecture/spec docs receive the clean addendum only after Task 10, and it depends on the entire implementation. That reverses the required workflow: once the user stamps the approved proposal and tasks, the proposal file should receive `### Consolidated Changes` and the canonical architecture/spec addendum should be appended before the implementation branch proceeds. Archival after reviewed merge is correctly late, but consolidation and archival are two different phases and should not be collapsed into one final task.

3. Task 3 cannot pass independently at its current point in the sequence. It depends only on Task 2, but its acceptance criteria require production import boundaries that are not true until later tasks remove VexFlow runtime imports, replace `setLayoutSource`, route layout rendering through browser wrappers, and route CLI SVG through the Node wrapper. If Task 3 is implemented immediately as a failing static gate, it will block Task 4/6/7 from doing the migrations that make the gate pass. Split this into either an early scanner with fixture-based self-tests and a later enforcement task, or move final enforcement after Tasks 6 and 7.

4. Task 4 overclaims relative to Task 7. Task 4 says "Layout rendering receives the source attached to the active parsed score revision" and "No production render path reads a module-level layout source cache," but Task 7 is the task that changes renderer APIs, removes `setLayoutSource`, and deletes the module-level cache. As written, Task 4 cannot satisfy its own acceptance criteria without performing Task 7's renderer work. To preserve independence, Task 4 should stop at the app/worker state boundary and prove with a mock render callback that the active source/revision would be passed. Task 7 should own the production renderer API conversion and cache deletion.

5. Task 1 does not fully cover the v1.5 package/declaration acceptance criteria. It verifies parser declarations contain no layout exports, but the proposal also requires layout package declarations to expose the layout scene builder expected by the layout wrapper. Task 2 proves the wrapper can produce a `LayoutScene`, but that is downstream and could mask a declaration/API mismatch with hand-written wrapper assumptions. Add an explicit Task 1 build-level check for layout generated declarations exposing the expected scene builder.

6. Task 1 says the build script reports raw, gzip, and brotli sizes for parser and layout WASM assets, but v1.5 requires reporting raw/gzip/brotli asset size, cache-cold transfer, incremental transfer, and cumulative transfer. Task 9 covers the browser transfer side, but the tasks do not define a single artifact/schema that connects build asset-size output with network audit output. Without that contract, Task 10 can pass by checking two unrelated reports with inconsistent labels. Add a shared report format or explicit acceptance criteria tying the build-size report and browser-transfer audit labels together.

7. Task 9's failure criteria are incomplete. It verifies Scenario 1 does not fetch layout WASM or VexFlow, and it fails if VexFlow is fetched on the default layout render path, but it should also explicitly fail if layout WASM is fetched before renderer invocation in Scenario 1. The proposal's core lazy-load requirement is symmetric: layout WASM must not load during startup/preview suspension, and VexFlow must not load on the default layout path.

8. Task 8's parity acceptance criteria are directionally correct but still underspecified for parse failures. v1.5 requires parse-failure parity, but the task does not say what counts as equivalent failure behavior: same success/failure classification, comparable diagnostic count, stable error category, or exact message. Since parser-facing diagnostics and layout-facing failures may not have identical message surfaces, define the minimum structural comparison now so implementation does not invent a weak test after the fact.

9. Task 5 is independent from the WASM split, but it changes the default renderer before the lazy layout path is fully migrated. If implemented in sequence on the proposal branch, intermediate app behavior may default users into a renderer path that still uses old imports/cache behavior until Task 7 lands. That can be acceptable on an unmerged branch, but the task should state its test boundary clearly: migration/default tests only, with production network/default-render verification deferred to Tasks 9 and 10. Otherwise the task appears to assert product readiness earlier than the branch actually has it.

10. The plan covers the major proposal requirements, but the dependency graph should be tightened. Task 10 depends on Task 9, and Task 9 depends on Task 7, so Task 4 is indirectly covered; however the current graph hides important readiness relationships. Final static enforcement should depend on the migrations it enforces, browser audit should depend on final static import boundaries or explicitly justify why it does not, and consolidation-before-implementation should not depend on verification tasks.

Required changes before approval:

- Add explicit input/output contracts to every task.
- Split Task 11 into pre-implementation consolidation and post-merge archival, or otherwise reorder consolidation before implementation per protocol.
- Fix Task 3/4/7 hidden coupling so each task can pass in isolation with mocks or moved acceptance criteria.
- Add the missing layout declaration check, explicit layout-prefetch audit failure, and parse-failure parity comparison contract.
- Clarify the size/transfer report schema shared by Tasks 1, 9, and 10.

STATUS: CHANGES_REQUESTED

### Author Response

The review is accepted. The initial task list mixed implementation gates with migration gates and did not state task-level input/output contracts. The revised task list below supersedes the initial list.

Key changes:

- Every task now includes an explicit input/output contract.
- Consolidation is moved to the pre-implementation phase after human stamp, while archival remains post-merge.
- Static import work is split into an early scanner/harness and a later final enforcement task.
- Source revision state and renderer API migration are separated so each can be tested independently.
- Build-size reporting and browser-transfer auditing share one report schema.
- Parse-failure parity and layout prefetch failure criteria are explicit.

## Revised Tasks v2: Split Parser/Layout WASM and Default Layout Rendering

### Task 0: Consolidate Approved Proposal After Human Stamp
- [x] **Status**: Done
- **Scope**: proposal file, canonical architecture/spec docs
- **Input/Output Contract**: Input is the approved proposal file, approved tasks file, and explicit human stamp. Output is appended `### Consolidated Changes` in the proposal file plus an append-only clean addendum in canonical architecture/spec docs.
- **Commits**:
  - `docs(wasm): consolidate split parser layout proposal`
- **Acceptance Criteria**:
  - Proposal file receives appended `### Consolidated Changes`
  - Canonical architecture/spec documentation receives the approved clean addendum append-only
  - No implementation code is changed in this task
  - Implementation branch may begin only after this task is complete
- **Dependencies**: Approved proposal, approved tasks, explicit human stamp

### Task 1: Define Split WASM Build Topology
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-core/Cargo.toml`, Rust WASM export cfg gates, `scripts/build_wasm.mjs`, generated package paths, size report schema
- **Input/Output Contract**: Input is the Rust crate graph and wasm-bindgen exports. Output is generated parser/layout web packages, layout Node package, declaration-boundary checks, and an asset-size report using the shared size/transfer schema.
- **Commits**:
  - `build(wasm): split parser and layout wasm package builds`
  - `test(wasm): enforce generated wasm declaration boundaries`
- **Acceptance Criteria**:
  - `scripts/build_wasm.mjs` builds `src/wasm/parser-pkg-web/`, `src/wasm/layout-pkg-web/`, and `src/wasm/layout-pkg-node/`
  - Parser build command uses `--target web --no-default-features --features parser-wasm`
  - Layout browser build command uses `--target web --no-default-features --features layout-wasm`
  - Layout Node build command uses `--target nodejs --no-default-features --features layout-wasm`
  - `drummark-layout` is optional from the parser build dependency graph
  - Parser generated declarations contain no layout exports
  - Layout generated declarations expose the expected layout scene builder
  - Build output reports raw, gzip, and brotli WASM asset sizes using labels compatible with Task 9
- **Dependencies**: Task 0

### Task 2: Add Explicit Browser and Node WASM Wrappers
- [ ] **Status**: Pending
- **Scope**: `src/wasm/`, wrapper tests, test setup imports
- **Input/Output Contract**: Input is generated split WASM packages. Output is explicit browser and Node wrapper APIs with tests proving parser-only and layout source-to-scene initialization.
- **Commits**:
  - `feat(wasm): add explicit parser and layout wasm wrappers`
  - `test(wasm): initialize split web and node wasm wrappers`
- **Acceptance Criteria**:
  - Browser parser wrapper imports only `src/wasm/parser-pkg-web/`
  - Browser layout wrapper imports only `src/wasm/layout-pkg-web/`
  - Node layout wrapper imports only `src/wasm/layout-pkg-node/`
  - Node parser wrapper exists only if needed and imports only `src/wasm/parser-pkg-node/`
  - Parser wrapper test parses source without importing layout wrapper code
  - Layout wrapper test produces a `LayoutScene` from source
  - Generic `src/wasm/drummark_wasm.ts` is deleted or reduced to a test-only compatibility shim
- **Dependencies**: Task 1

### Task 3: Build Static Import Scanner Harness
- [x] **Status**: Done
- **Scope**: import-scanning script/test, self-test fixtures, npm test wiring for scanner only
- **Input/Output Contract**: Input is a configurable list of source roots and forbidden import rules. Output is a scanner that can pass/fail fixture directories without requiring production code to satisfy final split boundaries yet.
- **Commits**:
  - `test(wasm): add import boundary scanner harness`
- **Acceptance Criteria**:
  - Scanner self-tests cover allowed imports, forbidden browser-to-Node imports, forbidden parser-to-layout imports, and forbidden CLI-to-browser imports
  - Scanner can run without invoking WASM builds
  - Production enforcement rules are defined but not yet required to pass against the full source tree
  - Final production enforcement is explicitly deferred to Task 8
- **Dependencies**: Task 0

### Task 4: Carry Source Revision Through Parsed Score State
- [ ] **Status**: Pending
- **Scope**: `src/App.tsx`, worker message types, parser state types, mock-render tests
- **Input/Output Contract**: Input is parser worker results tagged with source and revision. Output is active parsed score state containing `{ score, source, sourceRevision }` and a mock render callback receiving that coherent tuple.
- **Commits**:
  - `feat(app): bind parsed scores to source revisions`
  - `test(app): ignore stale parse results during rapid edits`
- **Acceptance Criteria**:
  - Active parsed score state carries `score`, `source`, and monotonically increasing `sourceRevision`
  - Async parse results older than the current revision cannot replace newer active score state
  - Mock render callback receives the source attached to the active parsed score revision
  - Rapid-edit test proves stale source revisions do not reach the mock render boundary
  - Production renderer API conversion and cache deletion are not part of this task
- **Dependencies**: Task 0

### Task 5: Make Layout Engine the Default with Settings Migration
- [ ] **Status**: Pending
- **Scope**: settings defaults/loaders, settings UI labels, i18n keys if labels are user-facing, migration tests
- **Input/Output Contract**: Input is persisted settings JSON or absence/corruption of it. Output is resolved app settings with explicit renderer preference preservation.
- **Commits**:
  - `feat(settings): default preview rendering to layout engine`
  - `test(settings): preserve explicit legacy renderer preferences`
- **Acceptance Criteria**:
  - Users with no saved settings default to `useLayoutEngine: true`
  - Saved settings without an own `useLayoutEngine` property default to `true`
  - Saved settings with own `useLayoutEngine: false` preserve `false`
  - Saved settings with own `useLayoutEngine: true` preserve `true`
  - Corrupt saved settings fall back to layout-engine default
  - Migration uses explicit own-property detection for `useLayoutEngine`
  - User-facing renderer labels use `Layout Engine` and `Legacy VexFlow`
  - Product network/default-render readiness is deferred to Tasks 9 and 10
- **Dependencies**: Task 0

### Task 6: Move Shared Render Options Out of VexFlow Modules
- [ ] **Status**: Pending
- **Scope**: shared renderer option modules, VexFlow imports, app/settings imports, type-only cleanup
- **Input/Output Contract**: Input is existing shared settings/types imported from `src/vexflow/*`. Output is renderer-neutral modules consumed by app/settings/layout code without VexFlow runtime imports.
- **Commits**:
  - `refactor(renderer): move shared render options to neutral modules`
  - `test(renderer): keep VexFlow runtime out of default settings path`
- **Acceptance Criteria**:
  - Shared setting ranges, render settings types, and page layout option definitions used outside VexFlow live in renderer-neutral modules
  - Default app/settings/layout-renderer production code has no runtime import from `src/vexflow/*`
  - Remaining VexFlow imports outside the VexFlow renderer are type-only and erased at build time
  - Scanner or bundle inspection proves VexFlow runtime is not pulled into the default layout settings path
- **Dependencies**: Tasks 0, 3

### Task 7: Convert Layout SVG Rendering to Explicit Source Input
- [ ] **Status**: Pending
- **Scope**: `src/renderer/svgRenderer.ts`, layout scene adapter call sites, CLI runtime, renderer tests
- **Input/Output Contract**: Input is active parsed score tuple `{ score, source, sourceRevision }` plus render settings. Output is SVG/pages rendered through browser or Node layout wrapper without module-level source cache.
- **Commits**:
  - `feat(renderer): pass source explicitly to layout svg renderer`
  - `fix(cli): initialize layout wasm through node wrapper`
- **Acceptance Criteria**:
  - Production renderer APIs accept explicit `source` and `sourceRevision` for layout rendering
  - `setLayoutSource` and module-level source cache are removed from production code
  - Browser layout rendering initializes through `layout_wasm_browser`
  - CLI SVG rendering initializes through `layout_wasm_node`
  - Rapid-edit rendering test uses the real renderer boundary and proves source/score revision coherence
  - `npm run drummark -- <representative fixture> --format svg` succeeds
  - Existing layout adapter regression tests pass through the explicit-source API
- **Dependencies**: Tasks 2, 4

### Task 8: Enforce Production Import Boundaries
- [x] **Status**: Done
- **Scope**: import scanner production rules, browser/parser/CLI source roots, test integration
- **Input/Output Contract**: Input is production source files after wrapper, settings, and renderer migrations. Output is a passing static import boundary gate for active production code.
- **Commits**:
  - `test(wasm): enforce production split wasm import boundaries`
- **Acceptance Criteria**:
  - Static test fails if browser production source imports `src/wasm/drummark_wasm.ts`, `src/wasm/pkg/drummark_core`, Node wrappers, or Node generated packages
  - Static test fails if parser-facing production source imports layout wrappers or layout generated packages
  - Static test fails if CLI runtime imports browser-only wrappers or browser generated packages
  - Integration/parity tests may import both parser and layout wrappers only from explicitly named integration/parity test files
  - Gate passes against the production source tree
- **Dependencies**: Tasks 2, 3, 6, 7

### Task 9: Add Parser/Layout Semantic Parity Corpus
- [x] **Status**: Done
- **Scope**: shared fixtures, parser/layout parity tests, structural comparison helpers
- **Input/Output Contract**: Input is a shared source corpus plus parser wrapper output and layout wrapper output. Output is structural parity assertions for successful parses and failure-parity assertions for invalid sources.
- **Commits**:
  - `test(wasm): compare parser and layout wasm semantics`
- **Acceptance Criteria**:
  - Shared corpus covers measure count, barlines, repeats, navigation markers, timing constructs, beams, and multi-measure input
  - Successful-parse parity compares parser/normalizer output with layout scene structural interpretation
  - Parse-failure parity requires same success/failure classification, comparable diagnostic count, and stable error category when available
  - Exact diagnostic text equality is not required unless both paths intentionally share the same message surface
  - Comparison avoids exact SVG coordinate assertions
  - Corpus includes representative notation from existing renderer regression tests
- **Dependencies**: Task 2

### Task 10: Add Browser Network and Transfer Audit
- [x] **Status**: Done
- **Scope**: browser automation dependency if needed, production preview audit script, test route/query for preview suspension, shared size/transfer report
- **Input/Output Contract**: Input is a production preview URL and four named audit scenarios. Output is a size/transfer ledger using the same labels as Task 1 plus pass/fail assertions for forbidden network requests.
- **Commits**:
  - `test(bundle): audit parser layout and legacy renderer network loads`
- **Acceptance Criteria**:
  - Audit runs against a production build preview
  - Scenario 1 uses preview suspension before renderer invocation and fails if layout WASM or VexFlow is fetched
  - Scenario 1 verifies parser WASM is fetched
  - Scenario 2 uses a fresh context for first default layout render and reports cumulative plus layout incremental transfer
  - Scenario 2 fails if VexFlow is fetched on the default layout render path
  - Scenario 3 uses a fresh context for first legacy VexFlow render and reports cumulative plus VexFlow incremental transfer
  - Scenario 4 measures legacy VexFlow render after default layout assets are cached
  - Report labels raw, gzip, brotli, cache-cold transfer, incremental transfer, and cumulative transfer distinctly
- **Dependencies**: Tasks 5, 7, 8

### Task 11: Consolidate Verification and Build Gates
- [x] **Status**: Done
- **Scope**: npm scripts, local verification docs, final test command set
- **Input/Output Contract**: Input is the completed implementation tasks and their reports. Output is one documented local verification gate for build, tests, CLI SVG, import boundaries, parity, and network audit.
- **Commits**:
  - `chore(test): wire split wasm verification gates`
- **Acceptance Criteria**:
  - `npm run build` succeeds and reports parser/layout WASM sizes separately
  - Targeted parser/layout wrapper tests pass
  - Settings migration tests pass
  - Static import boundary tests pass
  - Parser/layout semantic parity tests pass
  - Browser network audit passes
  - `npm run drummark -- <representative fixture> --format svg` succeeds
  - VexFlow remains available as a lazy legacy renderer
- **Dependencies**: Tasks 1, 2, 5, 7, 8, 9, 10

### Task 12: Archive Proposal Artifacts After Merge
- [ ] **Status**: Pending
- **Scope**: proposal and tasks files, `docs/archived/`
- **Input/Output Contract**: Input is a reviewed implementation branch merged back to main. Output is proposal and tasks artifacts moved from `docs/proposals/` to `docs/archived/`.
- **Commits**:
  - `docs(wasm): archive split parser layout proposal`
- **Acceptance Criteria**:
  - Proposal and tasks files move to `docs/archived/` only after reviewed merge
  - Archived files preserve full proposal/task ledger history
  - `docs/proposals/` no longer contains this completed proposal pair
- **Dependencies**: Task 11 and reviewed mainline merge

### Review Round 2

I reviewed `## Revised Tasks v2` against the operative approved proposal, `## Addendum v1.5`, and its approving `### Review Round 5`. The revised task list fixes the major Round 1 blockers: every task now has an input/output contract, consolidation is correctly moved before implementation after human stamp, static scanning is split into harness and final enforcement, source-revision state is separated from renderer API conversion, and archival is correctly post-merge.

There are still a few planning defects that should be corrected before approval:

1. Task 1 has a stale cross-reference in its size-report acceptance criteria. It says build output uses labels compatible with Task 9, but Task 9 is now semantic parity and the browser transfer audit is Task 10. Because Tasks 1 and 10 are intended to share the size/transfer report vocabulary, this should be corrected to Task 10 or phrased as "the shared size/transfer schema" to avoid tying implementation to the wrong task.

2. Task 10 does not explicitly assert all required fetch events from v1.5. The approved proposal requires Scenario 2 to fetch layout WASM and Scenario 3 to fetch the VexFlow chunk. The task currently says Scenario 2 reports layout incremental transfer and Scenario 3 reports VexFlow incremental transfer, which implies those events but does not make them pass/fail conditions. Add explicit acceptance criteria that Scenario 2 fails if layout WASM is not fetched on first default layout render, and Scenario 3 fails if the VexFlow chunk is not fetched on first legacy render.

3. The dedicated implementation branch requirement is not represented. v1.5 does not define this, but the repository workflow requires implementation to proceed on a proposal branch after consolidation. Task 0 correctly gates implementation after human stamp and consolidation, and Task 12 handles post-merge archival, but no task or acceptance criterion states that implementation uses a dedicated branch before code changes. Add this either to Task 0 as the handoff output or as a small pre-implementation task after consolidation.

The task independence and dependency graph are otherwise acceptable. Task 3 is independently testable with scanner fixtures before production enforcement. Task 4 stops at the app/mock-render boundary, while Task 7 owns production renderer conversion and cache removal. Task 8 depends on the migrations it enforces. Task 10 depends on the final import boundary and renderer migrations, and Task 11's dependencies cover the implementation surface either directly or transitively.

Coverage is also close to complete: split WASM package generation, wrapper migration, parser/layout declaration checks, source coherence, settings migration, renderer-neutral options, CLI SVG through Node layout WASM, production import boundaries, semantic parity, browser transfer audits, final verification gates, and archival are all represented. The remaining requested changes are precision fixes rather than a redesign.

STATUS: CHANGES_REQUESTED

### Author Response

The review is accepted. The remaining issues are precision fixes rather than task-structure changes.

The revised task list below supersedes Revised Tasks v2 only for the affected tasks and dependency wording. All unchanged Revised Tasks v2 tasks retain their existing contracts and acceptance criteria.

## Revised Tasks v3 Delta

### Task 0 Amendment: Consolidate Approved Proposal After Human Stamp
- [ ] **Status**: Pending
- **Scope**: proposal file, canonical architecture/spec docs, implementation branch setup
- **Input/Output Contract**: Input is the approved proposal file, approved tasks file, and explicit human stamp. Output is appended `### Consolidated Changes`, an append-only clean addendum in canonical architecture/spec docs, and a dedicated implementation branch ready for code changes.
- **Additional Acceptance Criteria**:
  - Dedicated implementation branch is created or selected before any implementation code changes
  - Branch name should follow the proposal topic, for example `proposal/split-parser-layout-wasm`

### Task 1 Amendment: Define Split WASM Build Topology
- [ ] **Status**: Pending
- **Scope**: shared size/transfer schema wording
- **Input/Output Contract**: Input is the Rust crate graph and wasm-bindgen exports. Output is generated parser/layout web packages, layout Node package, declaration-boundary checks, and an asset-size report using the shared size/transfer schema consumed by Task 10.
- **Replacement Acceptance Criterion**:
  - Build output reports raw, gzip, and brotli WASM asset sizes using the shared size/transfer schema consumed by Task 10

### Task 10 Amendment: Add Browser Network and Transfer Audit
- [ ] **Status**: Pending
- **Scope**: browser automation dependency if needed, production preview audit script, test route/query for preview suspension, shared size/transfer report, required fetch assertions
- **Input/Output Contract**: Input is a production preview URL and four named audit scenarios. Output is a size/transfer ledger using the same labels as Task 1 plus pass/fail assertions for required and forbidden network requests.
- **Additional Acceptance Criteria**:
  - Scenario 2 fails if layout WASM is not fetched on first default layout render
  - Scenario 3 fails if the VexFlow chunk is not fetched on first legacy render
  - Scenario 3 fails if layout WASM is fetched solely because the legacy renderer was selected in a fresh context

### Dependency Amendment

Task 1 through Task 12 depend on Task 0's human-stamped consolidation and dedicated implementation branch handoff, either directly or by following the implementation branch workflow. Tasks that already list Task 0 keep that explicit dependency; implementation tasks without a direct Task 0 dependency are still not allowed to begin before Task 0 completes.

### Review Round 3

I reviewed the appended Author Response and `## Revised Tasks v3 Delta` against the approved proposal `## Addendum v1.5` and the three blockers from Review Round 2.

The stale Task 1 cross-reference is resolved. The amended Task 1 contract and replacement acceptance criterion now tie raw/gzip/brotli asset-size reporting to the shared size/transfer schema consumed by Task 10, which matches v1.5's requirement that build-size output and browser-transfer audit labels remain coherent.

The missing required-fetch assertions in Task 10 are resolved. The amendment makes Scenario 2 fail if layout WASM is not fetched on first default layout render and makes Scenario 3 fail if the VexFlow chunk is not fetched on first legacy render. The added fresh-context guard against layout WASM loading solely because legacy rendering was selected is also consistent with the lazy-loading intent in v1.5.

The dedicated implementation branch requirement is resolved. Task 0 now includes implementation branch setup in its scope and output, requires a dedicated branch before implementation code changes, and the dependency amendment makes Task 0's human-stamped consolidation and branch handoff a gate for all implementation tasks.

I do not see remaining task-planning blockers. The v2 task list plus v3 delta now covers the approved proposal's build topology, wrapper boundaries, source-coherent layout rendering, settings migration, renderer-neutral options, CLI SVG contract, static import boundaries, browser transfer audit, semantic parity checks, final verification, and post-merge archival with independently testable task boundaries.

STATUS: APPROVED

### Implementation Completion Note

Tasks 1 through 11 are complete on branch `proposal/split-parser-layout-wasm`.

Verification completed on 2026-05-18 with:

- `npm run verify:split-wasm`

The verification gate covered:

- split WASM build and size reporting
- production TypeScript build and docs build
- import-boundary scanner tests
- split parser/layout wrapper tests
- settings migration tests
- score source-revision tests
- SVG renderer and adapter regression tests
- parser/layout semantic parity tests
- CLI runtime tests
- browser network audit for parser/layout/VexFlow lazy loading
- representative CLI SVG generation from `docs/examples/overview.drum`

Task 12 remains pending because archival is explicitly post-merge.
