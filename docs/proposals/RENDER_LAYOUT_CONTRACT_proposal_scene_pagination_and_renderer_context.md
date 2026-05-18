## Addendum v0.1: Scene Pagination and Renderer Context Refactor

### Problem

The Rust `LayoutScene` builder currently emits exactly one page. Systems are stacked by increasing `y` coordinates and can overflow past `page_height_pt - bottom_margin_pt`. The scene contract already has `pages` and `SceneSystem.page_index`, but the current implementation leaves those fields underused.

The same builder also has many helper functions with long argument lists. Those argument lists obscure ownership boundaries: some parameters describe page/system geometry, some describe current measure geometry, and some describe primitive drawing style.

### Goals

- Render long scores into multiple `ScenePage` entries.
- Keep all coordinates absolute within each page.
- Preserve stable system, measure, item, and composite ids across pagination.
- Keep platform adapters thin: no adapter-side page breaking, line breaking, or collision repair.
- Refactor renderer internals around explicit context/spec structs so high-level functions pass domain objects rather than long positional argument lists.
- Preserve current single-page output for scores that fit on one page.

### Non-Goals

- Do not change DrumMark syntax.
- Do not change `RenderScore`.
- Do not introduce adapter-side layout.
- Do not change the SVG adapter contract beyond consuming multiple existing `pages`.
- Do not add horizontal system wrapping inside a paragraph; current paragraph-to-system behavior remains unchanged unless a later proposal changes it.

### Contract Additions

`LayoutScene.pages` may contain more than one page. Each page owns only the systems, measures, items, and composites whose visible geometry appears on that page.

`ScenePage.index` remains zero-based.

`SceneSystem.page_index` must equal its containing page's `index`.

`SceneSystem.index` remains global across the score, not per-page. This preserves stable `system-{index}` ids.

`SceneMeasure.id` remains `measure-{global_index}`. Expanded display measures, such as two-bar repeats, keep using display global indices as they do now.

Item ids may remain page-local counter output as long as ids are unique within the whole `LayoutScene`. The implementation should keep a single item counter across pages.

### Pagination Algorithm

The layout engine computes `planned_systems` exactly as today, then assigns each planned system to a page before emitting scene items.

For each system:

- The first system on page 0 starts at `top_margin + header_height + header_staff_spacing`.
- Later systems on the same page start at previous system origin plus the existing vertical system advance.
- The first system on pages after page 0 starts at `top_margin`.
- A system fits when its staff bottom plus the reserved system advance does not exceed `page_height - bottom_margin`.
- If a system does not fit and the current page already has at least one system, start a new page and emit the system there.
- If a single system is taller than the available page content area, emit it on the current empty page and add a layout issue rather than dropping it.

Header/title/tempo items are emitted only on page 0 unless a later proposal adds repeated running headers.

### Structural Spans Across Pages

Voltas and hairpins are currently fragmented by system. Pagination must preserve that behavior and additionally isolate fragments by page.

- Span fragment generation samples only measures on the same page.
- A span crossing a page boundary emits a fragment on each page it touches.
- Fragment kind remains based on global span position: `start`, `continuation`, `end`, or `singleSegment`.
- Continuation visual rules for cross-system voltas remain unchanged.

### Renderer Context Refactor

Introduce internal context/spec structs:

- `PageEmitContext`: current page, page index, global item counter, layout options.
- `SystemEmitContext`: system id/index/page index, staff top/bottom/mid, system bounds.
- `MeasureEmitContext`: measure id, measure bounds, left/right pads, source `DisplayMeasure`.
- Primitive specs such as `LineItemSpec`, `GlyphItemSpec`, `TextItemSpec`, `RectItemSpec`, and `PathItemSpec`.

High-level render functions should accept context structs:

- `render_measure_events(..., MeasureEmitContext, ...)`
- `render_nav_markers(..., MeasureEmitContext, ...)`
- `render_right_barline(..., MeasureEmitContext, ...)`
- `push_volta_segment(..., VoltaSegmentSpec)`

Primitive item constructors should accept one spec object instead of many positional parameters.

### Acceptance Criteria

- `cargo test --workspace` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` passes without a crate-level `too_many_arguments` allow.
- Existing one-page fixture snapshots remain stable unless page metadata changes are explicitly justified.
- A new unit test with enough paragraph systems to exceed one page produces `scene.pages.len() > 1`.
- The multi-page test verifies page indices, per-page systems, and that no system on a non-overflow page extends below `page_height_pt - bottom_margin_pt`.
- A cross-page hairpin or volta fixture emits fragments on each affected page with correct `fragment` semantics.

### Review Round 1

#### 1. Adapter contract is underspecified and currently false in TypeScript

The proposal says "Do not change the SVG adapter contract beyond consuming multiple existing `pages`", but the current `src/renderer/svgRenderer.ts` adapter only renders `scene.pages[0]` and returns one SVG string from `renderSceneToSvg`. A Rust scene with `pages.len() > 1` would be silently truncated by the existing adapter surface. This is not just an implementation detail: callers need to know whether multi-page scene rendering returns one SVG per page, a single concatenated SVG/document, or preserves the existing single-SVG function as page 0 only with a new page-aware API.

Action required: define the adapter behavior explicitly. At minimum specify whether `renderSceneToSvg(scene)` remains first-page-only, changes to render all pages into a vertically stacked SVG, or is accompanied by a new `renderScenePagesToSvgs(scene)` API. Add acceptance criteria that prove page 1+ scene items are visible through the adapter path, not only present in Rust `LayoutScene`.

#### 2. Page-local ownership conflicts with composite anchor references

The proposal correctly says each `ScenePage` owns only visible geometry on that page, but it does not state whether `SceneComposite.start_anchor_id`, `end_anchor_id`, and `child_item_ids` must resolve within the same `ScenePage`. This matters because the current SVG scene adapter builds `measureMap` from only the page's measures. A cross-page hairpin or volta fragment that keeps the logical span's off-page start or end measure as an anchor will produce an unresolved composite in the adapter.

Action required: add an invariant that every composite on a page may reference only item IDs and measure IDs present on that same page, or define a cross-page reference mechanism that adapters must support. For fragmented spans, specify how start/end anchors are clipped at page boundaries: page-start continuation fragments should anchor to the first visible measure/system boundary on that page, and page-end fragments should anchor to the last visible measure/system boundary on that page.

#### 3. Span fragmentation semantics are ambiguous for page plus system breaks

The proposal says spans are currently fragmented by system and pagination must preserve that behavior, but then says a span crossing a page boundary emits "a fragment on each page it touches." Those two rules can be read as either one fragment per page or one fragment per system segment, isolated by page. A long span that covers two systems on page 0 and one system on page 1 needs three visual fragments, not two page-level fragments.

Action required: define the unit of fragmentation as the intersection of the logical span with each visible system on each page. Then define `SpanFragmentKind` against the logical span, not the page, with examples:

- start on page 0/system 0, continue on page 0/system 1, continue on page 1/system 2, end on page 1/system 3.
- single logical span that begins before a page and ends after it should produce only continuation fragments on intermediate page systems.

Also state whether volta labels are shown only on logical starts or repeated on page/system continuations.

#### 4. Fit test uses "reserved system advance" without defining the measured extent

The pagination algorithm says a system fits when "staff bottom plus the reserved system advance" does not exceed the page bottom. The current builder has at least three different vertical concepts: `s_bot`, `SceneSystem.height_pt`, and the advance `100.0 + opts.system_spacing_pt`. Structural items can also be stacked above/below after span generation. Reserving a full next-system advance for the last system on a page can cause premature page breaks; ignoring structural extents can allow visible hairpins/navigation/stacked items to escape the page even when the staff lines fit.

Action required: define the exact fit formula. Recommended: compute a `SystemVerticalExtent` with `origin_y`, `staff_top`, `staff_bottom`, `visual_top`, `visual_bottom`, and `advance_after`, then page-break on `visual_bottom <= page_height - bottom_margin` for the candidate system, with `advance_after` used only to place the next system. If structural stacking remains post-pass, acceptance criteria must check visible item bounds, not just `SceneSystem` bounds.

#### 5. Overflow issue reporting needs a contract

The proposal says a too-tall single system is emitted on an empty page and "add a layout issue", but `LayoutScene.issues` currently also carries score errors. The wording does not define whether this is warning-level, whether it blocks adapters, or what minimum data is included.

Action required: specify that layout overflow warnings are appended to `LayoutScene.issues` without discarding existing score errors, and include stable identifying data such as page index, system id/index, visual height, and available height. Add a unit assertion for this issue path so the branch is not untested.

#### 6. Planned systems "exactly as today" hides coupling with pagination

The proposal keeps paragraph-to-system planning unchanged, which is acceptable, but the implementation still needs a page assignment structure before emitting items. If page assignment is inferred while mutating `ScenePage`, span generation later has to rediscover page membership from emitted measures. That creates hidden coupling between emit order, page ownership, and span code.

Action required: define an intermediate `PlannedPage` / `PagedSystem` output from pagination before scene emission. It should contain page index, global system index, page-local origin Y, and the planned system reference. Span generation should consume emitted page-local measure lists, not the global planned systems, to preserve adapter-thin behavior.

#### 7. Context refactor does not yet constrain mutable ownership

The context structs are directionally useful, but `PageEmitContext: current page, page index, global item counter, layout options` mixes immutable layout metadata with mutable output ownership. If helpers receive both `&mut ScenePage` and a context that also implies "current page", the refactor can still leave aliasing-style confusion and long argument coupling under different names.

Action required: split immutable geometry/options from mutable sinks. For example, use `PageEmitContext` for page index/options and a separate `SceneEmitSink { page: &mut ScenePage, item_counter: &mut usize }`, or define that only the sink owns mutation. The proposal should state this boundary so the clippy cleanup does not become a cosmetic wrapper around the same responsibilities.

#### 8. Test coverage must include adapter and ID uniqueness, not just Rust page counts

The acceptance criteria cover Rust tests and one cross-page span fixture, but they miss two high-risk regressions: duplicate IDs across pages and page 1+ adapter omission. Item IDs are explicitly allowed to remain counter-based; composite IDs and text block IDs are not discussed. Header composites such as `text-block-title` are page 0 only today, but span/measure-repeat/navigation composites can easily duplicate if page-local counters or local segment indices reset.

Action required: add acceptance criteria that assert all `SceneItem.id` values are globally unique, all `SceneComposite.id` values are globally unique, every composite child/anchor reference resolves according to the chosen reference scope, and a TypeScript SVG scene adapter test renders more than the first page.

STATUS: CHANGES_REQUESTED

### Author Response Round 3

The review is accepted. Because prior append operations introduced multiple similarly named `Author Response` sections, this physically final response is the controlling clarification for consolidation. The deterministic preflight pagination model from v0.3 is the intended resolution of Round 2 and supersedes any earlier text that implies post-emission system movement.

No implementation should move systems after item emission. Pagination is based on precomputed conservative extents, then scene emission follows that fixed page assignment.

## Addendum v0.4: Controlling Preflight Stack Formula

### Controlling Algorithm

The implementation must use deterministic single-pass preflight pagination:

1. Build planned systems from measures.
2. Compute actual structural role counts for each planned system before page assignment.
3. Compute conservative `SystemVerticalExtent` values from those role counts.
4. Assign systems to `PlannedPage` values once.
5. Emit page-local scene geometry from the fixed page assignment.
6. Validate emitted bounds in tests/debug checks only; do not repaginate after emission.

This v0.4 algorithm is the authoritative source for final consolidation.

### Structural Stack Extent Formula

Preflight margins must be computed from the actual planned structural roles/counts on each system, not from one unqualified global constant.

For each planned system, collect layout-owned structural elements by vertical side:

- above-staff roles: measure number, navigation start/end markers, volta bracket/label fragments
- below-staff roles: hairpin fragments

For each side, compute:

`stack_margin = base_clearance + sum(role_height for each emitted structural group on that side) + edge_padding * max(0, group_count - 1) + max_user_offset`

Where:

- `role_height` comes from canonical text/glyph metrics or the existing fixed geometry constants for the role.
- `group_count` is the number of same-side structural groups the stacker may need to separate on that system.
- `edge_padding` is `LayoutOptions.edge_padding`.
- `max_user_offset` accounts for user-controlled offsets that move the element farther from the staff on that side. Offsets that move elements toward the staff do not reduce the reserved margin below the no-offset role height.

The formula may overestimate because exact X-overlap stacking is not known until emission. It must not underestimate the maximum stack height created by the currently supported structural roles on the planned system.

### Updated Acceptance Criteria

- Add an above-staff stacked fixture with at least two same-side structural groups on one system, such as measure number plus navigation or volta, and verify the stacked preflight margin can force a page break under small page height.
- Add a below-staff fixture with hairpin extent and user offset under small page height.
- Add a validation helper in tests that every emitted item on non-overflow pages stays within page bounds after structural stacking.
- Keep a unit assertion that single-system unavoidable overflow emits a non-fatal layout issue instead of triggering repagination.

### Author Response

The review is accepted. v0.2 still mixed two algorithms: pre-assignment pagination and post-emission movement. That is too easy to implement as an unstable loop. The revised design below chooses a single-pass conservative preflight model and removes post-emission movement from the contract.

## Addendum v0.3: Deterministic Preflight Pagination

### Page Ordering Invariant

`LayoutScene.pages` must be stored in strictly increasing contiguous `ScenePage.index` order, starting at 0. The TypeScript adapter may trust array order but should assert or test that the emitted scenes satisfy this invariant.

`renderScenePagesToSvgs(scene, options)` returns SVG strings in the same order as `scene.pages`.

### Compatibility API Behavior

`renderSceneToSvg(scene, options)` remains intentionally first-page-only for backward compatibility. It must not be used by full-score export paths once multi-page layout exists.

Observable behavior:

- If `scene.pages.length <= 1`, it behaves as today.
- If `scene.pages.length > 1`, it still returns page 0, but emits a development-time `console.warn` explaining that the scene has multiple pages and full-score callers should use `renderScenePagesToSvgs`.

Acceptance criteria must include migrating or testing the full-score caller path against `renderScenePagesToSvgs`, so the first-page compatibility function is not the only exercised adapter API.

### Single-Pass Preflight Pagination

Pagination must be single-pass over precomputed conservative vertical extents. No post-emission system movement is allowed.

Before page assignment, compute one `SystemVerticalExtent` per planned system:

- `origin_y`: tentative origin on a page
- `staff_top`
- `staff_bottom`
- `visual_top_margin`: conservative distance above `origin_y` needed by measure numbers, navigation, volta labels/brackets, title spillover when applicable, and stacked above-staff structural elements
- `visual_bottom_margin`: conservative distance below `staff_bottom` needed by hairpins and stacked below-staff structural elements
- `visual_bottom = staff_bottom + visual_bottom_margin`
- `advance_after`: existing system-to-system vertical advance

The preflight margins may overestimate. They must not underestimate known layout-owned structural elements. Conservative blank space is acceptable; clipped output is not.

The fit check for a candidate system on the current page is:

`candidate_origin_y + (staff_bottom - origin_y) + visual_bottom_margin <= page_height_pt - bottom_margin_pt`

If the candidate does not fit and the current page already has at least one system, start a new page and place the candidate at that page's first-system origin. Do not emit, stack, then move systems.

If the candidate still does not fit on an empty page, emit it on that empty page and append a non-fatal overflow issue. This is the only unavoidable overflow path.

### Structural Extent Sources

The conservative preflight extent must account for:

- staff line height
- above-staff navigation markers
- above-staff volta brackets and labels
- measure numbers on later systems
- below-staff hairpin open height and offset
- structural stacking padding for same-side edge elements

The implementation may express this as fixed conservative constants derived from current canonical metrics. It does not need to exactly replay `stack_scene_structural_items`, but it must reserve enough vertical space for the currently supported structural roles.

### Removed Post-Emission Movement

Post-emission validation remains useful as a debug/test assertion, but it must not mutate page assignment.

If validation finds item bounds outside the reserved page area:

- on a page containing a single unavoidable-overflow system, keep the scene and ensure an overflow issue exists
- otherwise, fail the relevant unit test; this means preflight extent constants are insufficient and must be fixed

### Revised Acceptance Criteria

- Add a fixture where above-staff structural elements, not staff geometry alone, force a page break under a small page height.
- Add a fixture where below-staff hairpin extent forces a page break under a small page height.
- Verify `LayoutScene.pages` indices are contiguous, increasing, and match array order.
- Verify `renderSceneToSvg` warns on multi-page scenes and remains first-page-only.
- Verify at least one full-score adapter/export path uses `renderScenePagesToSvgs`.

### Author Response

The review is accepted. The original v0.1 text underspecified the platform adapter behavior and allowed multiple interpretations for cross-page references and span fragments. The implementation must not create multi-page Rust data that the TypeScript adapter silently truncates.

The revised design below narrows the contract:

- SVG adapter behavior is explicitly page-aware.
- Every page-local composite must resolve its child and anchor references within the containing `ScenePage`.
- Span fragments are the intersection of a logical span with each visible system on each page, not one fragment per page.
- Page fitting is based on emitted visible bounds, not just staff line geometry.
- Overflow warnings are appended to `LayoutScene.issues` with stable identifying context.
- Pagination creates an intermediate page assignment before scene emission.
- Mutable scene output ownership is separated from immutable geometry/context.

## Addendum v0.2: Scene Pagination and Renderer Context Refactor Clarifications

### Adapter Output Contract

The TypeScript adapter must become page-aware before or alongside Rust multi-page output.

`renderSceneToSvg(scene, options)` remains a compatibility function for existing callers and renders the first page only. It must not pretend to render the full score when `scene.pages.length > 1`.

A new page-aware adapter function must be added:

- `renderScenePagesToSvgs(scene, options): string[]`
- Each returned SVG corresponds to one `ScenePage`.
- Page order follows `ScenePage.index`.
- Page dimensions and `viewBox` come from that page's `widthPt` and `heightPt`.

Callers that need full-score output must use the page-aware API. Tests must verify that page 1+ items are rendered by the adapter path.

### Page-Local Reference Invariant

Every `SceneComposite` stored on a `ScenePage` must reference only IDs visible on that same page:

- every `child_item_ids` entry resolves to an item in `page.items`
- `start_anchor_id`, when present, resolves to a measure or item visible on the same page
- `end_anchor_id`, when present, resolves to a measure or item visible on the same page

Cross-page composites are not allowed. A logical cross-page span is represented by separate page-local fragments.

For continuation fragments:

- a page-start/system-start continuation anchors to the first visible measure of that fragment
- a page-end/system-end continuation anchors to the last visible measure of that fragment
- no composite may anchor to an off-page measure solely because it is the logical span start or end

### Span Fragment Unit

The fragment unit is the intersection of one logical span with one visible system on one page.

Examples:

- A span that starts on page 0/system 0, continues on page 0/system 1, continues on page 1/system 2, and ends on page 1/system 3 emits four fragments: `start`, `continuation`, `continuation`, `end`.
- A span that begins before page 1 and ends after page 1 emits only `continuation` fragments for all systems it touches on page 1.
- A span fully contained in one system emits `singleSegment`.

Volta labels render only on the logical start fragment. Page/system continuations may show continuation hooks when required by existing volta visual rules, but they do not repeat the label unless a later proposal changes that notation rule.

### Page Assignment Before Emission

The implementation must compute page assignment before emitting page items.

Introduce an internal intermediate model:

- `PlannedPage`: page index and ordered `PagedSystem` entries
- `PagedSystem`: global system index, page index, page-local origin Y, and reference to the planned system

Scene emission consumes `PlannedPage` values. Span generation consumes emitted page-local measure lists, preserving the page-local reference invariant.

### Fit Formula and Visible Bounds

Pagination should separate placement advance from fit testing.

For candidate system placement, compute:

- `origin_y`
- `staff_top`
- `staff_bottom`
- initial `visual_top`
- initial `visual_bottom`
- `advance_after`

The candidate fits a non-empty page when its expected `visual_bottom <= page_height_pt - bottom_margin_pt`.

`advance_after` is used to place the next system, not to reject the current last system on a page.

After page items and structural spans are emitted and stacked, the implementation must validate actual visible item bounds for each non-overflow page. If any visible item exceeds `page_height_pt - bottom_margin_pt`, the engine must either move the system to the next page when possible or append a layout issue when the page contains an unavoidable overflow.

### Overflow Issue Contract

Layout overflow warnings are appended to `LayoutScene.issues`; existing score errors are preserved.

An overflow issue string must include at least:

- page index
- system id or global system index
- visible bottom or visual height
- available bottom or available height

The warning is non-fatal. Adapters still render the scene.

### Renderer Context Ownership

Context refactor must split immutable geometry from mutable sinks.

Use immutable contexts/specs for geometry and options:

- `PageEmitContext`: page index, page dimensions, margins/options
- `SystemEmitContext`: global system index, system id, staff top/bottom/mid, bounds
- `MeasureEmitContext`: measure id, measure bounds, left/right pads, source `DisplayMeasure`
- primitive specs: `LineItemSpec`, `GlyphItemSpec`, `TextItemSpec`, `RectItemSpec`, `PathItemSpec`

Use a separate mutable sink for scene output:

- `SceneEmitSink<'a> { page: &'a mut ScenePage, item_counter: &'a mut usize }`

Only the sink owns mutation. Context structs must not also imply ownership of mutable page state.

### Expanded Acceptance Criteria

- `cargo test --workspace` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` passes without a crate-level `too_many_arguments` allow.
- Existing one-page fixture snapshots remain stable unless page metadata changes are explicitly justified.
- A new Rust unit test with enough paragraph systems to exceed one page produces `scene.pages.len() > 1`.
- Multi-page Rust tests verify page indices, global system indices, page-local `y_pt`, and that non-overflow visible item bounds do not exceed `page_height_pt - bottom_margin_pt`.
- A cross-page hairpin or volta fixture emits one fragment per touched system/page intersection with correct `fragment` semantics.
- Every composite child and anchor reference resolves within its containing page.
- All `SceneItem.id` values are globally unique within the `LayoutScene`.
- All `SceneComposite.id` values are globally unique within the `LayoutScene`.
- A TypeScript adapter test verifies `renderScenePagesToSvgs(scene)` renders page 1+ items.
- A too-tall single-system fixture emits a non-fatal overflow issue containing page/system identity and available versus actual bounds.

### Review Round 2

#### 1. Post-emission bounds validation still creates a pagination deadlock

The v0.2 fit formula fixes the "reserved advance" issue, but the new post-emission rule introduces an unresolved control-flow problem:

> After page items and structural spans are emitted and stacked, the implementation must validate actual visible item bounds for each non-overflow page. If any visible item exceeds `page_height_pt - bottom_margin_pt`, the engine must either move the system to the next page when possible or append a layout issue when the page contains an unavoidable overflow.

This happens after page assignment and after span fragmentation, but moving a system to the next page changes page assignment, span fragments, continuation kinds, anchors, and potentially stacked structural extents. That means the layout may need to re-fragment and re-stack spans, then revalidate bounds again. The proposal does not define whether pagination is a single pass, a bounded fixpoint loop, or a preflight measurement pass.

Action required: define the algorithmic contract for visible bounds before implementation. Two acceptable shapes:

- Precompute conservative `SystemVerticalExtent` values before page assignment, including all structural stacks that can affect vertical bounds, then paginate once.
- Or define a bounded repagination loop: emit/stack, validate, move overflowing systems when possible, recompute fragments/anchors from scratch, repeat until stable or until only single-system overflow pages remain. Specify the maximum iteration guard and the issue emitted if convergence fails.

Acceptance criteria should include a fixture where a structural element, not staff geometry, forces a page break. Otherwise this branch can remain untested while normal staff-only pagination passes.

#### 2. "Move the system to the next page when possible" needs a precise constraint

The proposal does not say which system is moved when post-emission overflow is found. Moving the overflowing system is not always correct: if the overflow is caused by a span or navigation marker attached to a previous system, the last system on the page may be innocent, and moving it may not resolve the overflow. Conversely, moving a system that starts a cross-page span can change the fragment set and create a different overflow on the next page.

Action required: define the candidate movement rule. For example: only move the candidate system whose own computed visual extent exceeds the page bottom and only if the current page would remain non-empty; otherwise mark the page as unavoidable overflow. If overflow is caused by page-level or previous-system structural stacking, define whether the algorithm moves the last system, recomputes the page, or records an issue.

#### 3. Page-aware adapter ordering is still underspecified for malformed or unsorted scenes

`renderScenePagesToSvgs(scene, options): string[]` says page order follows `ScenePage.index`, while the return type has no page metadata. This is sufficient for ideal scenes, but it leaves edge cases ambiguous: should the adapter sort `scene.pages` by `index`, trust array order and assert monotonic indices, or render sparse page indices with gaps?

Action required: add an invariant that `LayoutScene.pages` is stored in strictly increasing contiguous `ScenePage.index` order, or state that the adapter sorts by index before rendering. Add a test or validation assertion for the chosen invariant. This prevents page order bugs from being hidden behind the page-aware API.

#### 4. Compatibility API behavior needs an observable contract

v0.2 says `renderSceneToSvg(scene, options)` remains first-page-only and "must not pretend to render the full score" when `scene.pages.length > 1`, but it does not define any observable behavior beyond returning page 0. If the function silently returns page 0, that is exactly the truncation failure Review Round 1 objected to, only now documented.

Action required: specify whether the compatibility function emits a warning, exposes metadata, throws in strict mode, or is intentionally silent. If intentional silence is chosen, say so explicitly and add an acceptance criterion that downstream full-score callers are migrated to `renderScenePagesToSvgs`.

STATUS: CHANGES_REQUESTED

### Review Round 3

#### 1. Ledger chronology is currently ambiguous under the proposal protocol

The v0.3 text appears physically before `## Addendum v0.2` and before `### Review Round 2`. Under the repository's Linear Ledger Protocol, review iterations are supposed to grow strictly downward, so the current file order makes the status of v0.3 ambiguous: it reads as if v0.3 was inserted into the middle of the ledger rather than appended as the response to Round 2.

This is not just clerical. The final consolidation step needs a clear chronological source of truth, and a reviewer should not have to infer that an earlier physical section supersedes later Round 2 findings.

Action required: without editing existing text, append an author response after this Round 3 that clearly restates the controlling v0.3 design at the actual end of the file, or otherwise appends a concise clarification that the deterministic preflight pagination text is the latest intended resolution of Round 2. Future reviewers and consolidation should use the physically final appended version/clarification as authoritative.

#### 2. Deterministic preflight resolves the post-emission movement deadlock, but stacked extent cardinality is still underspecified

The substantive algorithm change is the right direction: v0.3 chooses a single-pass conservative preflight model and explicitly forbids post-emission system movement. That resolves the Round 2 deadlock around moving systems after span fragmentation, and it also resolves the adapter ordering and compatibility-observability points by defining contiguous page order and a development warning for `renderSceneToSvg`.

The remaining ambiguity is how `visual_top_margin` and `visual_bottom_margin` are made conservative for stacked same-side structural elements. The text says the implementation may use fixed conservative constants and must include "structural stacking padding for same-side edge elements," but it does not say whether that margin is:

- a single worst-case constant for all currently supported combinations,
- computed from the actual roles present on that planned system,
- computed from the actual number of same-side elements that will be stacked, or
- intentionally capped with overflow allowed beyond the cap.

That distinction matters because deterministic preflight only works if the preflight value is at least as large as the later stacker can produce. A fixed constant derived from canonical metrics can still underestimate if a system has multiple same-side elements whose count affects stacked height. The proposal should define the contract as a deterministic formula or bound, for example: group layout-owned structural elements by side for the planned system, reserve `base_offset + element_height_sum + padding * max(0, count - 1)` using the same role set the stacker will emit, or explicitly state a proven maximum role combination for the current renderer.

Action required: append a clarification that preflight margins are derived from the actual planned structural element roles/counts, or define a documented maximum supported stack that the fixed constants cover. Add an acceptance criterion with at least two same-side structural elements on one system so the test proves stacked padding, not just a single hairpin or single above-staff marker, participates in page breaking.

STATUS: CHANGES_REQUESTED

### Review Round 4

#### 1. Chronology remains unresolved because the response was inserted above existing ledger entries

The new `### Author Response Round 3` and `## Addendum v0.4` text does explicitly say it is controlling, but it is physically located before `## Addendum v0.3`, `## Addendum v0.2`, `### Review Round 2`, and `### Review Round 3`. That does not satisfy the Round 3 action item, which specifically required the controlling clarification to be appended after Round 3 or otherwise made physically final.

Under the repository's Linear Ledger Protocol, later reviewers and consolidation should be able to read downward and treat the final appended material as the latest authority. Right now, the file still requires a reader to jump backward from Round 3 to line 148 and accept an inserted section that appears earlier than the review it claims to answer. That leaves the same historical ambiguity Round 3 identified.

Action required: append a new author response at the physical end of the file, after this Round 4, that restates that v0.4 is the controlling design for consolidation. Do not move or edit the existing inserted v0.4 text; the fix can be a concise physically final clarification plus, if desired, a short pointer to the inserted v0.4 section.

#### 2. Structural stack extent is now implementable and testable

The v0.4 stack formula resolves the remaining structural extent issue. It requires preflight margins to derive from actual planned structural roles/counts, groups elements by side, includes role heights, `edge_padding * max(0, group_count - 1)`, and user offsets that move elements farther from the staff. It also adds acceptance criteria for a same-side stacked above-staff fixture and a below-staff hairpin offset fixture.

That is a defensible contract for deterministic single-pass pagination. It is conservative without demanding a full replay of the emission stacker, and the tests can prove the previously missing case: stacked structural elements affecting page breaking rather than only a single marker or hairpin.

No further structural stack changes are required before implementation.

STATUS: CHANGES_REQUESTED

### Author Response Round 4

Accepted. This response is physically appended after Review Round 4 and is therefore the controlling ledger entry for consolidation.

The controlling design is the deterministic single-pass preflight pagination model described in `## Addendum v0.4: Controlling Preflight Stack Formula`. For implementation and consolidation, use v0.4 as authoritative and ignore earlier text that implies post-emission system movement or ambiguous stack constants.

No existing text is moved or edited. This final clarification exists only to satisfy the Linear Ledger Protocol's chronological reading requirement.

### Review Round 5

The physically final `### Author Response Round 4` resolves the Round 4 chronology objection. It is appended after Review Round 4, explicitly states that it is the controlling ledger entry for consolidation, identifies `## Addendum v0.4: Controlling Preflight Stack Formula` as authoritative, and clarifies that earlier conflicting text should be ignored for implementation and consolidation.

No chronology blocker remains. Since Round 4 already found the structural stack extent contract implementable and testable, there is no remaining blocker within this review scope.

STATUS: APPROVED
