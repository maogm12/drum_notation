## Tasks v0.1: System Box Pagination

### Task 1: Page-Aware Adapter Surface
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

### Task 2: Scene Item Bounds Module
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is a `SceneItem`; output is a deterministic bounds rectangle or a test-visible error for unsupported primitives.
- **Commits**:
  - `feat(layout): define scene item bounds`
  - `test(layout): cover bounds for emitted primitive kinds`
- **Acceptance Criteria**:
  - Bounds tests cover `TextRun`, `GlyphRun`, `LineSegment`, `Rect`, `Polyline`, and every `Path` command currently emitted by the layout engine.
  - Stroke width is included for line/rect/polyline bounds where applicable.
  - Unsupported path commands fail tests instead of silently contributing no bounds.
- **Dependencies**: None

### Task 3: Renderer Context and Primitive Specs
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is the current scene emission state and geometry values; output is unchanged scene data built through context/spec structs instead of long positional helper argument lists.
- **Commits**:
  - `refactor(layout): introduce scene emit contexts`
  - `refactor(layout): replace primitive helper argument lists with specs`
- **Acceptance Criteria**:
  - `cargo clippy --workspace --all-targets -- -D warnings` passes without a crate-level `too_many_arguments` allow.
  - Existing layout tests and snapshots pass.
  - Mutation is owned by `SceneEmitSink`; immutable geometry/options live in context/spec structs.
- **Dependencies**: None

### Task 4: System-Local Box Renderer
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is one planned system plus layout options and available width; output is a `SystemLayoutBox` with local systems/measures/items/composites and actual `visual_top` / `visual_bottom`.
- **Commits**:
  - `feat(layout): render planned systems as local boxes`
  - `test(layout): verify system box visual bounds`
- **Acceptance Criteria**:
  - System box coordinates are local, not page-space.
  - `visual_top` and `visual_bottom` are computed from actual emitted item bounds after structural stacking.
  - A fixture with above-staff and below-staff structural elements verifies visual bounds include both sides.
  - Span fragments are generated per logical-span/system intersection and remain local to the system box.
- **Dependencies**: Task 2, Task 3

### Task 5: Deterministic Box Pagination
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is ordered `SystemLayoutBox` values plus page/header/margin options; output is ordered `PlacedSystemBox` values and overflow issue data.
- **Commits**:
  - `feat(layout): paginate system boxes`
  - `test(layout): verify box pagination rules`
- **Acceptance Criteria**:
  - Page 0 cursor starts at `max(top + headerHeight + headerStaffSpacing, headerVisualBottom + headerStaffSpacing)`.
  - Later pages start at `topMargin`.
  - `systemSpacing` is added before non-first systems on a page.
  - Hand-crafted box tests cover normal fit, page break, later-page placement, and single-system overflow.
  - Overflow data includes page index, system id/index, visual height or bottom, and available height or bottom.
- **Dependencies**: Task 4

### Task 6: Page Scene Assembly and ID Remapping
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is `PlacedSystemBox` values and local system boxes; output is final `LayoutScene` pages with page-space coordinates, globally unique ids, and page-local references.
- **Commits**:
  - `feat(layout): assemble page scenes from system boxes`
  - `test(layout): verify page scene id remapping`
- **Acceptance Criteria**:
  - Assembly translates all primitive geometry by explicit `dx` and `dy`.
  - `SceneSystem.y_pt` remains the page-space staff/system origin, not visual top.
  - Local item/composite ids are remapped with deterministic system prefixes.
  - Composite child ids, item anchors, and composite item anchors are rewritten through the remap table.
  - Measure anchors use final measure ids directly.
  - Two boxes with identical local ids assemble into globally unique final ids with valid references.
- **Dependencies**: Task 5

### Task 7: Final Scene Validator
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is a final `LayoutScene`; output is validation success or diagnostic strings for page/order/id/reference/bounds violations.
- **Commits**:
  - `test(layout): add final scene validator`
- **Acceptance Criteria**:
  - Validator checks contiguous page order.
  - Validator checks system page indices.
  - Validator checks global item/composite id uniqueness.
  - Validator checks page-local composite child references and measure anchors.
  - Validator checks page-local item anchor references.
  - Validator checks bounded items stay within page dimensions on non-overflow pages.
  - Validator test coverage includes page 0 with header items and at least one later page.
- **Dependencies**: Task 6

### Task 8: Layout Scene Orchestrator
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`, `crates/drummark-core`, CLI/adapter verification fixtures
- **Input/Output Contract**: Input is a `RenderScore` plus `LayoutOptions`; output is the final page-aware `LayoutScene` built by calling independent modules in order.
- **Commits**:
  - `feat(layout): orchestrate system box pagination`
  - `test(layout): verify system box pagination end to end`
- **Acceptance Criteria**:
  - Orchestrator order is: plan systems, render header box, render system boxes, paginate boxes, assemble page scenes, validate final scene, return issues.
  - Long-score fixture produces `scene.pages.len() > 1`.
  - Cross-page hairpin fixture remains page-local after pagination.
  - Cross-page volta fixture remains page-local and does not repeat labels except on logical starts.
  - Existing score issues are preserved.
  - `LAYOUT_WARNING overflow ...` issues are emitted for single-system overflow.
  - `cargo test --workspace` passes.
  - `cargo clippy --workspace --all-targets -- -D warnings` passes.
  - `npm run drummark -- <multi-page-fixture> --format svg` or equivalent SVG verification confirms page-aware output is reachable from the public pipeline.
- **Dependencies**: Tasks 1-7

### Task 9: Contract Consolidation
- [ ] **Status**: Pending
- **Scope**: `docs/proposals/RENDER_LAYOUT_CONTRACT_proposal_system_box_pagination.md`, `docs/RENDER_LAYOUT_CONTRACT.md`, `TASKS.md`
- **Input/Output Contract**: Input is the approved proposal ledger and completed implementation; output is append-only contract documentation and completed task status.
- **Commits**:
  - `docs(layout): consolidate system box pagination contract`
- **Acceptance Criteria**:
  - Proposal file gets appended `### Consolidated Changes`.
  - `docs/RENDER_LAYOUT_CONTRACT.md` gets a clean append-only addendum.
  - This tasks file marks all implementation tasks done.
  - `TASKS.md` Rust cleanup TODO entries for renderer context and pagination are marked done.
- **Dependencies**: Tasks 1-8

### Review Round 1

The task plan is close to the approved v0.2 proposal, but it still hides several proposal requirements inside broad downstream tasks instead of giving them independent input/output contracts and acceptance tests.

1. Header layout is not independently implemented or testable.

The approved proposal introduces a separate `HeaderLayoutBox` with actual item bounds for title, subtitle, composer, and tempo, and uses `max(top_margin_pt + header_height_pt + header_staff_spacing_pt, header_visual_bottom + header_staff_spacing_pt)` to prevent page-0 overlap. The tasks mention `headerVisualBottom` in Task 5 and "render header box" in Task 8, but there is no task whose output is a `HeaderLayoutBox`, no bounds test for header primitives, and no acceptance criterion proving tempo/header items cannot overlap the first system under non-default offsets.

Required change: add a dedicated header-box task, or expand an existing pre-pagination task with an explicit `RenderScore/LayoutOptions -> HeaderLayoutBox` contract and tests for title/subtitle/composer/tempo bounds plus the first-system cursor rule.

2. Composite anchor policy is inconsistent with the approved proposal and adapter reality.

Addendum v0.2 restricts adapter-rendered composite `start_anchor_id` / `end_anchor_id` to page-local measures unless the TypeScript adapter is updated to resolve item anchors. Task 6 says "Composite child ids, item anchors, and composite item anchors are rewritten through the remap table," which appears to permit composite item anchors without a matching adapter update. Task 7 validates composite measure anchors but does not reject or validate composite item anchors. Task 1 only proves page 1+ raw items render, not that page 1+ composites with measure anchors render correctly.

Required change: make the task plan choose one policy. The smaller compliant path is to require composite start/end anchors to remain measure IDs, add a validator check rejecting adapter-rendered composite item anchors, and add a TypeScript adapter test for a page 1+ measure-anchored composite. If item anchors are intended, Task 1 must include the adapter resolver update and page 1+ item-anchor composite test.

3. The page-aware adapter task is ordered before the scene contract needed to test the real risk.

Task 1 can add the API with a synthetic scene, so it is implementable, but it cannot verify the approved proposal's page-local composite/reference behavior until page assembly exists. The current plan defers only "or equivalent SVG verification" to Task 8, which is too vague for the adapter contract edge case from the proposal review.

Required change: keep Task 1 for the API surface if desired, but add a later adapter integration acceptance criterion after Task 6/7 that renders page 1+ composites, not just primitives, through `renderScenePagesToSvgs`.

4. Pagination is unnecessarily dependent on the full system renderer.

Task 5 is correctly described as a hand-crafted-box algorithm, but it depends on Task 4. Under the Task Independence Rule, deterministic pagination over box rectangles should be independently testable with mock `SystemLayoutBox` values once the data type exists. Coupling Task 5 to Task 4 makes the algorithm wait on emitted score rendering even though it does not need notes, spans, or structural stacking.

Required change: introduce a small foundation task for shared data contracts (`SystemLayoutBox`, `PlacedSystemBox`, overflow warning shape, and possibly `HeaderLayoutBox`) before Tasks 4 and 5, then make Task 5 depend only on that foundation and the header-box contract, not on the full system renderer.

5. Final-scene validation can mask overflow too broadly.

Task 7 says bounded items must stay within page dimensions "on non-overflow pages." The approved proposal's residual note says overflow pages should be exempt only for the specific overflowing system/items, not used to mask unrelated header, ID, reference, or other systems' bounds failures on the same page. A page with one oversized system may still contain header items or other references that must validate.

Required change: refine Task 7 acceptance criteria so overflow warnings suppress only bounds failures attributable to the explicitly overflowing system box; page order, ID uniqueness, page-local references, header bounds, and unrelated systems remain validated.

6. Final consolidation is present.

Task 9 includes `### Consolidated Changes`, append-only contract documentation, task status completion, and `TASKS.md` cleanup. That satisfies the protocol's final consolidation requirement.

STATUS: CHANGES_REQUESTED

### Author Response

Accepted. The plan needed a separate foundation layer so pagination can be tested with mock boxes, and it needed the header box and composite anchor policy to be explicit instead of implied by the orchestrator.

Changes in v0.2:

- Add a dedicated data-contract foundation task for `SystemLayoutBox`, `HeaderLayoutBox`, `PlacedSystemBox`, and overflow warning formatting.
- Split header box rendering into its own independently testable task.
- Make deterministic pagination depend only on the shared contracts, not the full system renderer.
- Choose the smaller composite policy: adapter-rendered composite `start_anchor_id` / `end_anchor_id` remain measure ids. Item anchors are not part of this proposal.
- Add adapter coverage for a page 1+ measure-anchored composite.
- Refine final validation so overflow suppresses only bounds failures for the specific overflowing system box.

## Tasks v0.2: System Box Pagination

### Task 1: Shared Layout Box Contracts
- [x] **Status**: Done
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is no runtime score data; output is typed internal contracts for `SystemLayoutBox`, `HeaderLayoutBox`, `PlacedSystemBox`, and stable overflow issue formatting.
- **Commits**:
  - `feat(layout): define system box pagination contracts`
  - `test(layout): cover overflow warning formatting`
- **Acceptance Criteria**:
  - `SystemLayoutBox` carries global system index/id, local staff origin, local visual bounds, width, local measures, local systems, items, and composites.
  - `HeaderLayoutBox` carries page-0 header items and actual visual bounds.
  - `PlacedSystemBox` carries page index, `page_x`, `page_y`, and enough metadata to assemble final `SceneSystem` values without rescanning unrelated state.
  - Overflow warnings use the stable `LAYOUT_WARNING overflow page=... system=... visualHeight=... availableHeight=...` string schema.
  - Unit tests verify overflow warning strings preserve existing score issues rather than replacing them.
- **Dependencies**: None

### Task 2: Page-Aware Adapter Surface
- [x] **Status**: Done
- **Scope**: `src/renderer/svgRenderer.ts`, full-score SVG/PDF/export caller path, TypeScript tests
- **Input/Output Contract**: Input is a `Scene` with one or more ordered pages; output is one SVG string per page for full-score callers, while the legacy single-SVG function remains first-page-compatible.
- **Commits**:
  - `feat(renderer): add page-aware scene svg rendering`
  - `test(renderer): cover multi-page scene adapter output`
- **Acceptance Criteria**:
  - `renderScenePagesToSvgs(scene, options)` returns one SVG per `ScenePage`.
  - Existing `renderSceneToSvg(scene, options)` remains first-page-only and emits a development warning for multi-page scenes.
  - A TypeScript test proves page 1+ primitive items appear in page-aware adapter output.
  - A TypeScript test proves a page 1+ measure-anchored composite renders through `renderScenePagesToSvgs`.
  - Full-score export/caller paths use or are tested against `renderScenePagesToSvgs`.
- **Dependencies**: None

### Task 3: Scene Item Bounds Module
- [x] **Status**: Done
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is a `SceneItem`; output is a deterministic bounds rectangle or a test-visible error for unsupported primitives.
- **Commits**:
  - `feat(layout): define scene item bounds`
  - `test(layout): cover bounds for emitted primitive kinds`
- **Acceptance Criteria**:
  - Bounds tests cover `TextRun`, `GlyphRun`, `LineSegment`, `Rect`, `Polyline`, and every `Path` command currently emitted by the layout engine.
  - Stroke width is included for line/rect/polyline bounds where applicable.
  - Unsupported path commands fail tests instead of silently contributing no bounds.
- **Dependencies**: Task 1

### Task 4: Header Layout Box Renderer
- [x] **Status**: Done
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is score metadata plus `LayoutOptions`; output is a page-0-only `HeaderLayoutBox` with header items and actual visual bounds.
- **Commits**:
  - `feat(layout): render header layout box`
  - `test(layout): verify header box bounds`
- **Acceptance Criteria**:
  - Header box includes title, subtitle, composer, and tempo items when present.
  - `visual_top` and `visual_bottom` are computed from actual header item bounds.
  - Tests cover non-default header/tempo offsets and prove the derived first-system cursor clears the header visual bottom.
  - Later pages receive no header items from this task.
- **Dependencies**: Task 3

### Task 5: Renderer Context and Primitive Specs
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is the current scene emission state and geometry values; output is unchanged scene data built through context/spec structs instead of long positional helper argument lists.
- **Commits**:
  - `refactor(layout): introduce scene emit contexts`
  - `refactor(layout): replace primitive helper argument lists with specs`
- **Acceptance Criteria**:
  - `cargo clippy --workspace --all-targets -- -D warnings` passes without a crate-level `too_many_arguments` allow.
  - Existing layout tests and snapshots pass.
  - Mutation is owned by `SceneEmitSink`; immutable geometry/options live in context/spec structs.
- **Dependencies**: Task 1

### Task 6: System-Local Box Renderer
- [x] **Status**: Done
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is one planned system plus layout options and available width; output is a `SystemLayoutBox` with local systems/measures/items/composites and actual `visual_top` / `visual_bottom`.
- **Commits**:
  - `feat(layout): render planned systems as local boxes`
  - `test(layout): verify system box visual bounds`
- **Acceptance Criteria**:
  - System box coordinates are local, not page-space.
  - `visual_top` and `visual_bottom` are computed from actual emitted item bounds after structural stacking.
  - A fixture with above-staff and below-staff structural elements verifies visual bounds include both sides.
  - Span fragments are generated per logical-span/system intersection and remain local to the system box.
  - Composite `start_anchor_id` and `end_anchor_id` emitted for adapter-rendered spans remain measure ids.
- **Dependencies**: Task 3, Task 5

### Task 7: Deterministic Box Pagination
- [x] **Status**: Done
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is ordered mock or real `SystemLayoutBox` values plus `HeaderLayoutBox` and page/header/margin options; output is ordered `PlacedSystemBox` values and overflow issue data.
- **Commits**:
  - `feat(layout): paginate system boxes`
  - `test(layout): verify box pagination rules`
- **Acceptance Criteria**:
  - Page 0 cursor starts at `max(top + headerHeight + headerStaffSpacing, headerVisualBottom + headerStaffSpacing)`.
  - Later pages start at `topMargin`.
  - `systemSpacing` is added before non-first systems on a page.
  - Pagination tests use hand-crafted boxes and do not require full system rendering.
  - Hand-crafted box tests cover normal fit, page break, later-page placement, and single-system overflow.
  - Overflow data includes page index, system id/index, visual height, and available height.
- **Dependencies**: Task 1

### Task 8: Page Scene Assembly and ID Remapping
- [x] **Status**: Done
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is `HeaderLayoutBox`, `PlacedSystemBox` values, and local system boxes; output is final `LayoutScene` pages with page-space coordinates, globally unique ids, and page-local references.
- **Commits**:
  - `feat(layout): assemble page scenes from system boxes`
  - `test(layout): verify page scene id remapping`
- **Acceptance Criteria**:
  - Assembly translates all primitive geometry by explicit `dx` and `dy`.
  - `SceneSystem.y_pt` remains the page-space staff/system origin, not visual top.
  - Header items are copied only to page 0 and remain outside any `SceneSystem`.
  - Local item/composite ids are remapped with deterministic system prefixes.
  - Composite child ids and item-local references are rewritten through the remap table.
  - Composite `start_anchor_id` and `end_anchor_id` are validated as final measure ids for adapter-rendered composites.
  - Two boxes with identical local ids assemble into globally unique final ids with valid references.
- **Dependencies**: Task 1, Task 4, Task 7

### Task 9: Final Scene Validator
- [x] **Status**: Done
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Input/Output Contract**: Input is a final `LayoutScene`; output is validation success or diagnostic strings for page/order/id/reference/bounds violations.
- **Commits**:
  - `test(layout): add final scene validator`
- **Acceptance Criteria**:
  - Validator checks contiguous page order.
  - Validator checks system page indices.
  - Validator checks global item/composite id uniqueness.
  - Validator checks page-local composite child references and measure anchors.
  - Validator rejects adapter-rendered composite item anchors unless the TypeScript adapter is extended in a later proposal.
  - Validator checks page-local item references.
  - Bounds validation is suppressed only for items belonging to the explicitly overflowing system box named by a `LAYOUT_WARNING overflow ...` issue.
  - Page order, ID uniqueness, page-local references, header bounds, and unrelated systems remain validated even on a page containing one overflowing system.
  - Validator test coverage includes page 0 with header items and at least one later page.
- **Dependencies**: Task 8

### Task 10: Layout Scene Orchestrator
- [x] **Status**: Done
- **Scope**: `crates/drummark-layout/src/lib.rs`, `crates/drummark-core`, CLI/adapter verification fixtures
- **Input/Output Contract**: Input is a `RenderScore` plus `LayoutOptions`; output is the final page-aware `LayoutScene` built by calling independent modules in order.
- **Commits**:
  - `feat(layout): orchestrate system box pagination`
  - `test(layout): verify system box pagination end to end`
- **Acceptance Criteria**:
  - Orchestrator order is: plan systems, render header box, render system boxes, paginate boxes, assemble page scenes, validate final scene, return issues.
  - Long-score fixture produces `scene.pages.len() > 1`.
  - Cross-page hairpin fixture remains page-local after pagination.
  - Cross-page volta fixture remains page-local and does not repeat labels except on logical starts.
  - Existing score issues are preserved.
  - `LAYOUT_WARNING overflow ...` issues are emitted for single-system overflow.
  - `cargo test --workspace` passes.
  - `cargo clippy --workspace --all-targets -- -D warnings` passes.
  - `npm run drummark -- <multi-page-fixture> --format svg` or equivalent SVG verification confirms page-aware output is reachable from the public pipeline.
- **Dependencies**: Tasks 2-9

### Task 11: Contract Consolidation
- [ ] **Status**: Pending
- **Scope**: `docs/proposals/RENDER_LAYOUT_CONTRACT_proposal_system_box_pagination.md`, `docs/RENDER_LAYOUT_CONTRACT.md`, `TASKS.md`
- **Input/Output Contract**: Input is the approved proposal ledger and completed implementation; output is append-only contract documentation and completed task status.
- **Commits**:
  - `docs(layout): consolidate system box pagination contract`
- **Acceptance Criteria**:
  - Proposal file gets appended `### Consolidated Changes`.
  - `docs/RENDER_LAYOUT_CONTRACT.md` gets a clean append-only addendum.
  - This tasks file marks all implementation tasks done.
  - `TASKS.md` Rust cleanup TODO entries for renderer context and pagination are marked done.
- **Dependencies**: Tasks 1-10

### Review Round 2

Tasks v0.2 address the Round 1 blockers and are sufficiently independent for implementation.

1. Header layout is now independently scoped and testable.

Task 1 introduces `HeaderLayoutBox` as a shared contract, and Task 4 gives header rendering its own `RenderScore/LayoutOptions`-style input/output boundary: score metadata plus `LayoutOptions` in, page-0 `HeaderLayoutBox` with actual visual bounds out. The Task 4 acceptance criteria cover title, subtitle, composer, tempo, non-default offsets, and the first-system cursor clearing the header visual bottom. That closes the earlier gap where header behavior was implied only by pagination/orchestration.

2. Composite anchor policy is explicit and matches the approved proposal.

Tasks v0.2 choose the smaller compliant policy: adapter-rendered composite `start_anchor_id` and `end_anchor_id` remain measure ids. Task 6 requires emitted adapter-rendered spans to use measure ids, Task 8 validates final composite anchors as measure ids, and Task 9 rejects adapter-rendered composite item anchors unless a later adapter proposal adds item-anchor resolution. This is consistent with the approved proposal's composite anchor scope.

3. Page-aware adapter coverage now includes the page 1+ composite risk.

Task 2 still allows the adapter surface to be implemented with synthetic scenes, which is independently testable, but its acceptance criteria now require both page 1+ primitive output and a page 1+ measure-anchored composite rendered through `renderScenePagesToSvgs`. Task 10 also keeps public pipeline SVG verification, so the adapter API and end-to-end path are both covered at the right layers.

4. Pagination is decoupled from real system rendering.

Task 1 provides the shared box contracts before rendering, and Task 7 depends only on Task 1. Its contract explicitly accepts mock or real `SystemLayoutBox` values plus a `HeaderLayoutBox`, and its tests must use hand-crafted boxes without full system rendering. That satisfies the Task Independence Rule for the pagination algorithm.

5. Overflow validation is scoped to the named overflowing system.

Task 9 now suppresses bounds validation only for items belonging to the explicitly overflowing system box named by a `LAYOUT_WARNING overflow ...` issue. It also requires page order, ID uniqueness, page-local references, header bounds, and unrelated systems to remain validated on the same page. This addresses the prior risk that an overflow page would mask unrelated final-scene defects.

6. Ordering and completeness are coherent.

The task order separates contracts, adapter API, primitive bounds, header layout, renderer refactor, system-local rendering, mock-box pagination, assembly/remapping, validation, orchestration, and consolidation. Tasks that can be tested with mocks are not forced to wait on full rendering, while the orchestrator correctly comes after all independent modules. The final consolidation task remains present and includes the required append-only proposal/spec updates.

Residual implementation notes, not approval blockers: keep `local_system_origin_y` and header bounds coordinate space explicit in the Rust types, and ensure the page 1+ composite adapter test asserts the composite is actually drawn rather than only present in input data.

STATUS: APPROVED
