## Tasks v0.1: Scene Pagination and Renderer Context Refactor

### Task 1: Page-Aware Adapter Surface
- [ ] **Status**: Pending
- **Scope**: `src/renderer/svgRenderer.ts`, TypeScript tests
- **Input/Output Contract**: Input is an existing `Scene` object with one or more `pages`; output is one SVG string per page from a new page-aware function.
- **Commits**:
  - `feat(renderer): add page-aware scene svg rendering`
  - `test(renderer): cover multi-page scene adapter output`
- **Acceptance Criteria**:
  - `renderScenePagesToSvgs(scene, options)` returns one SVG per `ScenePage`.
  - Returned SVG order follows contiguous `scene.pages` array order.
  - Existing `renderSceneToSvg(scene, options)` remains first-page-only and emits a development warning for multi-page scenes.
  - A TypeScript test proves page 1+ items appear in page-aware adapter output.
- **Dependencies**: None

### Task 2: Renderer Context and Primitive Specs
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is the current scene emission state and geometry values; output is unchanged `SceneItem` / `SceneComposite` data built through context/spec structs instead of long positional helper argument lists.
- **Commits**:
  - `refactor(layout): introduce scene emit contexts`
  - `refactor(layout): replace primitive helper argument lists with specs`
- **Acceptance Criteria**:
  - `cargo clippy --workspace --all-targets -- -D warnings` passes without a crate-level `too_many_arguments` allow.
  - Existing layout tests and snapshots pass.
  - Mutation is owned by `SceneEmitSink`; immutable page/system/measure geometry lives in context/spec structs.
- **Dependencies**: None

### Task 3: Preflight Structural Extents
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is a `PlannedSystem` plus layout options; output is a conservative `SystemVerticalExtent` containing above/below structural margins and system advance, without emitting scene items.
- **Commits**:
  - `feat(layout): compute preflight system vertical extents`
  - `test(layout): cover stacked structural extent estimates`
- **Acceptance Criteria**:
  - Extents are derived from actual planned structural role counts by side.
  - Above-staff stacked fixture with at least two structural groups can force a page break under small page height.
  - Below-staff hairpin plus offset fixture can force a page break under small page height.
  - Extent computation is testable with hand-crafted planned-system inputs.
- **Dependencies**: None

### Task 4: Deterministic Page Assignment
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is ordered `PlannedSystem` values and their `SystemVerticalExtent`s; output is ordered `PlannedPage` / `PagedSystem` assignments with page-local system origins.
- **Commits**:
  - `feat(layout): assign planned systems to pages`
  - `test(layout): verify deterministic page assignment`
- **Acceptance Criteria**:
  - Page assignment is single-pass and does not depend on emitted scene items.
  - `PlannedPage.index` values are contiguous and increasing from 0.
  - First system on page 0 honors title/header spacing; first system on later pages starts at top margin.
  - Too-tall single-system fixture produces an unavoidable overflow marker for later scene issue emission.
- **Dependencies**: Task 3

### Task 5: Multi-Page Scene Emission
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is `PlannedPage` assignments and `RenderScore`; output is a `LayoutScene` with one `ScenePage` per planned page and page-local systems/measures/items/composites.
- **Commits**:
  - `feat(layout): emit layout scene pages`
  - `test(layout): verify multi-page scene invariants`
- **Acceptance Criteria**:
  - Long-score fixture produces `scene.pages.len() > 1`.
  - `ScenePage.index` values match array order.
  - `SceneSystem.page_index` matches containing page.
  - `SceneSystem.index` remains global.
  - All `SceneItem.id` values are globally unique.
  - All `SceneComposite.id` values are globally unique.
  - Non-overflow page item bounds stay within page bounds after structural stacking.
- **Dependencies**: Task 2, Task 4

### Task 6: Cross-Page Span Fragment Invariants
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is page-local scene measures plus logical volta/hairpin spans; output is page-local composites whose child and anchor references resolve within the containing page.
- **Commits**:
  - `feat(layout): fragment spans across pages`
  - `test(layout): cover cross-page volta and hairpin fragments`
- **Acceptance Criteria**:
  - Fragment unit is one logical span intersected with one visible system on one page.
  - Cross-page hairpin fixture emits correct `start` / `continuation` / `end` semantics.
  - Cross-page volta fixture emits correct continuation fragments without repeating labels except at logical starts.
  - Every composite child and anchor reference resolves within its containing page.
- **Dependencies**: Task 5

### Task 7: Contract Consolidation
- [ ] **Status**: Pending
- **Scope**: `docs/proposals/RENDER_LAYOUT_CONTRACT_proposal_scene_pagination_and_renderer_context.md`, `docs/RENDER_LAYOUT_CONTRACT.md`, `TASKS.md`
- **Input/Output Contract**: Input is the approved proposal ledger and completed implementation; output is append-only contract documentation and completed task status.
- **Commits**:
  - `docs(layout): consolidate scene pagination contract`
- **Acceptance Criteria**:
  - Proposal file gets appended `### Consolidated Changes`.
  - `docs/RENDER_LAYOUT_CONTRACT.md` gets a clean append-only addendum.
  - This tasks file marks all implementation tasks done.
  - `TASKS.md` Rust cleanup TODO entries for renderer context and pagination are marked done.
- **Dependencies**: Tasks 1-6

### Review Round 1

#### 1. Task 1 does not cover the approved full-score caller/export migration

The approved proposal requires more than adding `renderScenePagesToSvgs(scene, options)`: it also says at least one full-score adapter/export path must use the page-aware API so the compatibility `renderSceneToSvg` path is not the only exercised route. Task 1 only proves page 1+ items appear in the new adapter function and that `renderSceneToSvg` warns. That can pass while the application/PDF/export path still silently truncates multi-page scenes through `renderScoreToSvg` or another first-page-only caller.

Action required: add an acceptance criterion, probably in Task 1 or a separate adapter integration task, that identifies the full-score caller/export path to migrate or test and verifies it consumes `renderScenePagesToSvgs`.

#### 2. Overflow issue emission is split into a marker with no consuming task

Task 4 produces an "unavoidable overflow marker for later scene issue emission," but no later task explicitly consumes that marker and appends the approved non-fatal `LayoutScene.issues` warning with page/system identity plus available versus actual bounds. Task 5 validates page bounds, but it does not mention overflow issue strings or preservation of existing score errors.

This creates hidden coupling: the page assignment task can pass by returning a marker, scene emission can pass normal multi-page invariants, and the approved overflow issue contract remains unimplemented.

Action required: either make Task 4 output the final issue data structure and test its fields, or add explicit Task 5 acceptance criteria that overflow markers are converted into `LayoutScene.issues`, existing issues are preserved, and the warning contains page index, system id/index, visual bottom/height, and available bottom/height.

#### 3. Task 3 needs a sharper formula-level contract to be independently testable

Task 3 says extents are derived from actual planned structural role counts by side, but its input/output contract does not require the controlling v0.4 formula fields: role height sum, `edge_padding * max(0, group_count - 1)`, and max user offset that only expands away from the staff. The fixture-level criteria are useful, but they are downstream symptoms; they do not prove the preflight function itself is correct in isolation.

Action required: add Task 3 acceptance criteria with hand-crafted planned structural roles that directly assert the above-staff and below-staff margin calculation, including multiple same-side groups, edge padding cardinality, and positive/negative user offset behavior.

#### 4. Task 5 and Task 6 have an unclear composite boundary

Task 5 claims to emit a `LayoutScene` with page-local systems/measures/items/composites, while Task 6 later implements cross-page span fragmentation and page-local composite reference invariants. If Task 5 emits real composites before Task 6, it may need the Task 6 span-fragment rules to avoid invalid cross-page anchors. If Task 5 intentionally excludes or stubs cross-page span composites until Task 6, that limitation is not stated.

This violates the Task Independence Rule because Task 5's output contract depends on behavior assigned to Task 6, and Task 6's only stated tests are full fixtures after Task 5 exists.

Action required: narrow Task 5 to non-span scene emission/composite invariants, or move the page-local composite/reference invariant into Task 6. Also add a Task 6 isolation criterion that tests span fragmentation from hand-crafted page-local measure lists and logical spans without requiring the full scene emitter/orchestrator.

#### 5. Orchestrator responsibilities are not isolated at the end

The approved design has a clear sequence: plan systems, compute extents, assign pages, emit page-local scene, fragment spans, then expose through adapters. The tasks mostly follow that sequence, but there is no final layout orchestrator task that wires the independent modules together and verifies the CLI/pipeline acceptance criteria. As written, Task 5 risks becoming the orchestrator while also owning scene emission and multi-page invariants, which makes it harder to test independently and easier to hide coupling between pagination, emission, stacking, and issue reporting.

Action required: add or revise a task so orchestration comes after the independently testable pieces. Its acceptance criteria should include the repository-required `npm run drummark -- <fixture> --format ir` or relevant SVG/XML command for a multi-page fixture, plus `cargo test --workspace` and clippy if those are not already tied to implementation completion.

STATUS: CHANGES_REQUESTED

### Author Response

Accepted. Tasks v0.1 hid coupling between scene emission, span fragmentation, overflow issue emission, and adapter integration. Tasks v0.2 below supersede v0.1 for implementation planning.

## Tasks v0.2: Scene Pagination and Renderer Context Refactor

### Task 1: Page-Aware Adapter Surface and Full-Score Caller
- [ ] **Status**: Pending
- **Scope**: `src/renderer/svgRenderer.ts`, full-score SVG/PDF/export caller path, TypeScript tests
- **Input/Output Contract**: Input is a `Scene` with one or more ordered pages; output is one SVG string per page for full-score callers, while the legacy single-SVG function remains first-page-compatible.
- **Commits**:
  - `feat(renderer): add page-aware scene svg rendering`
  - `test(renderer): cover multi-page scene adapter output`
- **Acceptance Criteria**:
  - `renderScenePagesToSvgs(scene, options)` returns one SVG per `ScenePage`.
  - Returned SVG order follows the contiguous `scene.pages` array order.
  - Existing `renderSceneToSvg(scene, options)` remains first-page-only and emits a development warning for multi-page scenes.
  - A TypeScript test proves page 1+ items appear in page-aware adapter output.
  - At least one full-score caller/export path is migrated to or tested against `renderScenePagesToSvgs`, so multi-page output is not silently truncated through the compatibility function.
- **Dependencies**: None

### Task 2: Renderer Context and Primitive Specs
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is the current scene emission state and geometry values; output is unchanged `SceneItem` / `SceneComposite` data built through context/spec structs instead of long positional helper argument lists.
- **Commits**:
  - `refactor(layout): introduce scene emit contexts`
  - `refactor(layout): replace primitive helper argument lists with specs`
- **Acceptance Criteria**:
  - `cargo clippy --workspace --all-targets -- -D warnings` passes without a crate-level `too_many_arguments` allow.
  - Existing layout tests and snapshots pass.
  - Mutation is owned by `SceneEmitSink`; immutable page/system/measure geometry lives in context/spec structs.
- **Dependencies**: None

### Task 3: Preflight Structural Extent Calculator
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is a planned system summary plus explicit structural role counts/offsets by side; output is a conservative `SystemVerticalExtent` with above/below margins, staff bounds, and system advance, without emitting scene items.
- **Commits**:
  - `feat(layout): compute preflight system vertical extents`
  - `test(layout): cover stacked structural extent estimates`
- **Acceptance Criteria**:
  - Hand-crafted above-staff role input verifies `base_clearance + role_height_sum + edge_padding * max(0, group_count - 1)`.
  - Hand-crafted below-staff role input verifies the same edge-padding cardinality.
  - User offsets that move an element away from the staff increase the reserved margin.
  - User offsets that move an element toward the staff do not reduce the reserved margin below the no-offset role height.
  - Above-staff stacked fixture with at least two structural groups can force a page break under small page height.
  - Below-staff hairpin plus offset fixture can force a page break under small page height.
- **Dependencies**: None

### Task 4: Deterministic Page Assignment and Overflow Data
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is ordered `PlannedSystem` values and `SystemVerticalExtent`s; output is ordered `PlannedPage` / `PagedSystem` assignments plus structured unavoidable-overflow records.
- **Commits**:
  - `feat(layout): assign planned systems to pages`
  - `test(layout): verify deterministic page assignment`
- **Acceptance Criteria**:
  - Page assignment is single-pass and does not depend on emitted scene items.
  - `PlannedPage.index` values are contiguous and increasing from 0.
  - First system on page 0 honors title/header spacing; first system on later pages starts at top margin.
  - Too-tall single-system fixture produces structured overflow data containing page index, system id/index, actual visual bottom/height, and available bottom/height.
  - Page assignment unit tests use hand-crafted extents without requiring scene emission.
- **Dependencies**: Task 3

### Task 5: Page-Local Non-Span Scene Emission
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is `PlannedPage` assignments and `RenderScore`; output is a `LayoutScene` with page-local systems/measures/items and non-span composites. Cross-page hairpin/volta composites are excluded until Task 6.
- **Commits**:
  - `feat(layout): emit non-span layout scene pages`
  - `test(layout): verify multi-page scene shell invariants`
- **Acceptance Criteria**:
  - Long-score fixture produces `scene.pages.len() > 1`.
  - `ScenePage.index` values match array order.
  - `SceneSystem.page_index` matches containing page.
  - `SceneSystem.index` remains global.
  - All `SceneItem.id` values are globally unique.
  - Non-span `SceneComposite.id` values emitted in this task are globally unique.
  - Structured overflow data from Task 4 is converted into `LayoutScene.issues` while preserving existing score issues.
- **Dependencies**: Task 2, Task 4

### Task 6: Page-Local Span Fragment Module
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is page-local scene measures plus logical volta/hairpin spans; output is page-local span composites whose child and anchor references resolve within the containing page.
- **Commits**:
  - `feat(layout): fragment spans across pages`
  - `test(layout): cover cross-page volta and hairpin fragments`
- **Acceptance Criteria**:
  - Fragment unit is one logical span intersected with one visible system on one page.
  - Hand-crafted page-local measure lists and logical spans test fragment kinds without requiring full scene emission.
  - Cross-page hairpin fixture emits correct `start` / `continuation` / `end` semantics.
  - Cross-page volta fixture emits correct continuation fragments without repeating labels except at logical starts.
  - Every span composite child and anchor reference resolves within its containing page.
  - All span `SceneComposite.id` values are globally unique.
- **Dependencies**: Task 5

### Task 7: Layout Scene Orchestrator and End-to-End Verification
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`, `crates/drummark-core`, CLI/adapter verification fixtures
- **Input/Output Contract**: Input is a `RenderScore` plus `LayoutOptions`; output is the final page-aware `LayoutScene` assembled by calling the independent modules in order.
- **Commits**:
  - `feat(layout): orchestrate page-aware layout scene`
  - `test(layout): verify page-aware layout end to end`
- **Acceptance Criteria**:
  - Orchestrator order is: plan systems, compute extents, assign pages, emit non-span scene pages, emit page-local span fragments, stack/validate, return issues.
  - `cargo test --workspace` passes.
  - `cargo clippy --workspace --all-targets -- -D warnings` passes.
  - `npm run drummark -- <multi-page-fixture> --format svg` or equivalent SVG verification confirms page-aware output is reachable from the public pipeline.
  - Non-overflow page item bounds stay within page bounds after structural stacking.
  - Every composite child and anchor reference resolves within its containing page.
- **Dependencies**: Tasks 1-6

### Task 8: Contract Consolidation
- [ ] **Status**: Pending
- **Scope**: `docs/proposals/RENDER_LAYOUT_CONTRACT_proposal_scene_pagination_and_renderer_context.md`, `docs/RENDER_LAYOUT_CONTRACT.md`, `TASKS.md`
- **Input/Output Contract**: Input is the approved proposal ledger and completed implementation; output is append-only contract documentation and completed task status.
- **Commits**:
  - `docs(layout): consolidate scene pagination contract`
- **Acceptance Criteria**:
  - Proposal file gets appended `### Consolidated Changes`.
  - `docs/RENDER_LAYOUT_CONTRACT.md` gets a clean append-only addendum.
  - This tasks file marks all implementation tasks done.
  - `TASKS.md` Rust cleanup TODO entries for renderer context and pagination are marked done.
- **Dependencies**: Tasks 1-7

### Review Round 2

Tasks v0.2 resolves the Review Round 1 blockers.

1. Full-score adapter caller coverage is now explicit in Task 1. The task no longer stops at the low-level adapter and legacy compatibility path; it requires at least one full-score caller/export path to be migrated to or tested against `renderScenePagesToSvgs`, which closes the truncation gap identified in Round 1.

2. Overflow issue consumption is now represented across Task 4 and Task 5. Task 4 owns deterministic page assignment and produces structured overflow records with page index, system identity, actual bounds, and available bounds. Task 5 owns conversion of those records into `LayoutScene.issues` while preserving existing score issues. That split is acceptable because the producer/consumer boundary is now named and testable.

3. Task 3 now has formula-level extent tests. The acceptance criteria directly cover role height sums, `edge_padding * max(0, group_count - 1)`, same-side group cardinality, and user offsets expanding only away from the staff. This satisfies the independent calculator requirement rather than relying only on downstream page-break fixtures.

4. The Task 5/Task 6 composite boundary is now clear. Task 5 is limited to page-local systems, measures, items, and non-span composites; cross-page hairpin/volta composites are explicitly excluded until Task 6. Task 6 also adds hand-crafted page-local measure/span tests, so span fragmentation can be verified as a module instead of only through full scene emission.

5. Final orchestration is isolated in Task 7. The task wires the independent modules together after their individual contracts are established, and it includes workspace tests, clippy, public pipeline SVG verification, page-bound validation, and page-local composite reference validation. This addresses the prior risk that Task 5 would become an implicit orchestrator.

Task Independence Rule check: the revised sequence has clear input/output contracts and test boundaries. Task 4 can be tested from hand-crafted extents, Task 6 can be tested from hand-crafted page-local measure lists and logical spans, and Task 7 is correctly positioned as the final orchestrator. Dependencies now describe data flow rather than hidden implementation coupling.

STATUS: APPROVED

### Author Response After User Review

The user rejected the preflight-estimation approach as unreliable. Accepted. Tasks v0.3 below supersede Tasks v0.2 and align with proposal Addendum v0.6: bounded real-layout pagination loop.

## Tasks v0.3: Bounded Real-Layout Pagination Loop

### Task 1: Page-Aware Adapter Surface and Full-Score Caller
- [ ] **Status**: Pending
- **Scope**: `src/renderer/svgRenderer.ts`, full-score SVG/PDF/export caller path, TypeScript tests
- **Input/Output Contract**: Input is a `Scene` with one or more ordered pages; output is one SVG string per page for full-score callers, while the legacy single-SVG function remains first-page-compatible.
- **Commits**:
  - `feat(renderer): add page-aware scene svg rendering`
  - `test(renderer): cover multi-page scene adapter output`
- **Acceptance Criteria**:
  - `renderScenePagesToSvgs(scene, options)` returns one SVG per `ScenePage`.
  - Existing `renderSceneToSvg(scene, options)` remains first-page-only and emits a development warning for multi-page scenes.
  - A TypeScript test proves page 1+ items appear in page-aware adapter output.
  - At least one full-score caller/export path is migrated to or tested against `renderScenePagesToSvgs`.
- **Dependencies**: None

### Task 2: Renderer Context and Primitive Specs
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is the current scene emission state and geometry values; output is unchanged scene data built through context/spec structs instead of long positional helper argument lists.
- **Commits**:
  - `refactor(layout): introduce scene emit contexts`
  - `refactor(layout): replace primitive helper argument lists with specs`
- **Acceptance Criteria**:
  - `cargo clippy --workspace --all-targets -- -D warnings` passes without a crate-level `too_many_arguments` allow.
  - Existing layout tests and snapshots pass.
  - Mutation is owned by `SceneEmitSink`; immutable page/system/measure geometry lives in context/spec structs.
- **Dependencies**: None

### Task 3: Initial Page Assignment Module
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is ordered `PlannedSystem` values plus page/options geometry; output is ordered page assignments preserving score order before real-layout repair.
- **Commits**:
  - `feat(layout): create initial page assignments`
  - `test(layout): verify initial page assignment invariants`
- **Acceptance Criteria**:
  - Assignment preserves global system order.
  - Page indices are contiguous and increasing from 0.
  - First system on page 0 honors title/header spacing; first system on later pages starts at top margin.
  - Unit tests use hand-crafted planned systems and do not require scene emission.
- **Dependencies**: None

### Task 4: Page Assignment Repair Loop
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is a page assignment plus a function that emits/measures actual page bounds; output is a repaired page assignment, overflow records, and optional guard-reached record.
- **Commits**:
  - `feat(layout): repair page assignments from real bounds`
  - `test(layout): cover bounded pagination repair loop`
- **Acceptance Criteria**:
  - Repair scans pages in order after each full rebuild/measure.
  - Single-system overflow pages are recorded and skipped while later multi-system overflow pages remain repairable.
  - First overflowing multi-system page is repaired by moving its last system to the beginning of the next page assignment.
  - The loop rebuilds/measures from the latest assignment after every repair.
  - A multi-move fixture converges within `planned_system_count + 1` iterations.
  - A single-system overflow fixture emits non-fatal overflow data and does not loop forever.
  - A guard-reached test hook emits a non-fatal issue from an internally consistent latest assignment.
- **Dependencies**: Task 3

### Task 5: Page-Local Non-Span Scene Emission
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is a page assignment and `RenderScore`; output is a `LayoutScene` with page-local systems/measures/items and non-span composites. Cross-page hairpin/volta composites are excluded until Task 6.
- **Commits**:
  - `feat(layout): emit non-span layout scene pages`
  - `test(layout): verify multi-page scene shell invariants`
- **Acceptance Criteria**:
  - Long-score fixture produces `scene.pages.len() > 1` before span emission is required.
  - `ScenePage.index` values match array order.
  - `SceneSystem.page_index` matches containing page.
  - `SceneSystem.index` remains global.
  - All `SceneItem.id` values are globally unique.
  - Non-span `SceneComposite.id` values emitted in this task are globally unique.
  - Existing score issues are preserved in `LayoutScene.issues`.
- **Dependencies**: Task 2, Task 3

### Task 6: Page-Local Span Fragment Module
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is page-local scene measures plus logical volta/hairpin spans; output is page-local span composites whose child and anchor references resolve within the containing page.
- **Commits**:
  - `feat(layout): fragment spans across pages`
  - `test(layout): cover cross-page volta and hairpin fragments`
- **Acceptance Criteria**:
  - Fragment unit is one logical span intersected with one visible system on one page.
  - Hand-crafted page-local measure lists and logical spans test fragment kinds without requiring full scene emission.
  - Cross-page hairpin fixture emits correct `start` / `continuation` / `end` semantics.
  - Cross-page volta fixture emits correct continuation fragments without repeating labels except at logical starts.
  - Every span composite child and anchor reference resolves within its containing page.
  - All span `SceneComposite.id` values are globally unique.
- **Dependencies**: Task 5

### Task 7: Layout Scene Orchestrator and Real-Bounds Verification
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`, `crates/drummark-core`, CLI/adapter verification fixtures
- **Input/Output Contract**: Input is a `RenderScore` plus `LayoutOptions`; output is the final page-aware `LayoutScene` created by repeatedly emitting real scene geometry and repairing assignment until stable or guarded.
- **Commits**:
  - `feat(layout): orchestrate bounded page-aware layout`
  - `test(layout): verify real-bounds pagination end to end`
- **Acceptance Criteria**:
  - Orchestrator order is: plan systems, create initial assignment, emit full scene from assignment, measure actual bounds, repair assignment, rebuild from scratch, repeat until stable or guarded.
  - No implementation path moves already-emitted systems/items/composites between pages.
  - Overflow and guard records from Task 4 are converted into `LayoutScene.issues` while preserving existing issues.
  - Structural-overflow fixture is repaired by moving the last system to the next page.
  - Cross-page hairpin and volta fragments remain page-local after at least one repair iteration.
  - All item/composite ids remain globally unique after rebuilds.
  - `cargo test --workspace` passes.
  - `cargo clippy --workspace --all-targets -- -D warnings` passes.
  - `npm run drummark -- <multi-page-fixture> --format svg` or equivalent SVG verification confirms page-aware output is reachable from the public pipeline.
- **Dependencies**: Tasks 1-6

### Task 8: Contract Consolidation
- [ ] **Status**: Pending
- **Scope**: `docs/proposals/RENDER_LAYOUT_CONTRACT_proposal_scene_pagination_and_renderer_context.md`, `docs/RENDER_LAYOUT_CONTRACT.md`, `TASKS.md`
- **Input/Output Contract**: Input is the approved proposal ledger and completed implementation; output is append-only contract documentation and completed task status.
- **Commits**:
  - `docs(layout): consolidate scene pagination contract`
- **Acceptance Criteria**:
  - Proposal file gets appended `### Consolidated Changes` synthesizing v0.6 as controlling design.
  - `docs/RENDER_LAYOUT_CONTRACT.md` gets a clean append-only addendum.
  - This tasks file marks all implementation tasks done.
  - `TASKS.md` Rust cleanup TODO entries for renderer context and pagination are marked done.
- **Dependencies**: Tasks 1-7

### Review Round 3

Tasks v0.3 align with proposal Addendum v0.6 and resolve the user rejection of the deterministic preflight model. The task sequence now targets the bounded real-layout pagination loop: initial assignment, real scene emission/measurement, repair from actual bounds, full rebuild from the latest assignment, and guarded termination.

1. Adapter/full-score caller coverage remains explicit. Task 1 requires `renderScenePagesToSvgs(scene, options)`, preserves the first-page compatibility behavior with a warning, verifies page 1+ adapter output, and requires at least one full-score caller/export path to migrate to or be tested against the page-aware API. That covers the proposal's truncation-prevention requirement.

2. The repair loop is independently testable and matches v0.6. Task 4 takes a page assignment plus an emit/measure function, which lets it be tested without depending on the full scene emitter. Its acceptance criteria cover ordered scans after rebuild/measure, skipping single-system overflow pages while continuing to later repairable pages, moving the last system from the first overflowing multi-system page, rebuilding/measuring from the latest assignment after every repair, multi-move convergence, unavoidable overflow data, and guard-reached output from an internally consistent latest assignment.

3. Non-span emission, span fragmentation, and orchestration are separated cleanly. Task 5 owns page-local systems/measures/items and non-span composites only. Task 6 owns page-local hairpin/volta span fragments with hand-crafted page-local measure/span tests. Task 7 is correctly the first task that composes the full bounded loop and verifies that cross-page spans remain page-local after at least one repair iteration.

4. The mandatory rebuild rule is covered at the right level. Task 4 requires rebuild/measure after every repair through its injected measurement boundary, and Task 7 explicitly forbids moving already-emitted systems/items/composites between pages. That preserves the proposal's page-local anchor, child reference, id regeneration, and span-fragment invariants.

5. Consolidation is present and scoped. Task 8 requires appending consolidated v0.6 changes to the proposal, appending a clean contract addendum, marking this tasks file done, and updating the Rust cleanup TODOs in `TASKS.md`.

Task Independence Rule check: the plan now uses data-flow boundaries rather than hidden coupling. Task 3 can be tested from hand-crafted planned systems; Task 4 can be tested with mock emit/measure functions; Task 5 can emit page shells/non-span composites from assignments; Task 6 can fragment spans from page-local measure inputs; Task 7 is the orchestrator after the independent modules exist. No task depends on downstream implementation details to prove its own contract.

STATUS: APPROVED
