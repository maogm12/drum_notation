## Render Layout Contract

This document defines the repository-owned contract for VexFlow replacement work.

### Ownership Chain

The active architectural target is:

`source -> parser AST -> normalized musical model -> RenderScore -> LayoutScene -> platform adapter`

Where:

- `drummark-core` owns parsing and normalization
- `drummark-core` derives `RenderScore`
- `drummark-layout` owns canonical metrics and layout
- platform renderers consume `LayoutScene` and only translate it to drawing APIs

### `RenderScore`

`RenderScore` is the parser-independent render input contract. It must contain all data required for deterministic drum layout without source rescans.

Required surface:

- header timing and title metadata
- explicit track list with render families
- measures with stable indices and source-line provenance
- resolved note/rest/sticking events with timing fractions
- voice, beam, tuplet, and visible modifier data
- repeat-span, navigation, volta, hairpin, measure-repeat, and multi-rest semantics

`RenderScore` is a closed contract. New layout dependencies must be added explicitly, not tunneled through ad hoc metadata.

### Canonical Metrics

All layout-affecting measurement is repository-owned.

That includes:

- drum notehead glyph metrics
- rest glyph metrics
- repeat/navigation glyph metrics
- title/subtitle/composer/tempo/sticking/count text metrics

Adapters do not measure text or glyphs to influence layout.

### `LayoutScene`

`LayoutScene` is the platform-neutral layout output.

Contract rules:

- coordinates are absolute page-space coordinates
- stable ids are preserved for systems, measures, items, and composites
- semantic composites are first-class for spans and text blocks
- system-break span fragments are encoded explicitly
- scene snapshots are a valid test oracle independent of pixel rendering

Minimum scene structure:

- pages
- systems
- measures
- items
- composites

### Thin Adapter Rule

A platform renderer may only do:

- scene traversal
- unit conversion
- glyph/path lookup
- paint execution
- optional accessibility/event tagging

A platform renderer may not do:

- text or glyph measurement for layout
- line breaking
- collision resolution
- span reconstruction
- position nudging beyond device-space rounding

## Addendum 2026-05-14: Approved Platform-Neutral Layout Constraints

The approved repository contract for the current migration additionally requires:

- `RenderScore` remains the explicit render-facing boundary between normalization and layout
- `LayoutScene` remains the only adapter input for active rendering paths
- canonical metrics and layout-affecting geometry are owned by `drummark-layout`, not by adapters

### Approved Engraving Constraints

- System starts are decomposed into explicit reservation components:
  - opening barline at the staff left boundary
  - repeated percussion clef width plus trailing gap
  - optional time-signature width plus trailing gap
  - first-note entry offset after the rendered components
- No unnamed extra start-padding bucket is allowed outside those explicit components.
- The first playable slot starts immediately after the sum of the rendered components and their canonical inter-component gaps.
- Later systems may not retain phantom time-signature spacing when no time signature is rendered.
- The rightmost closing barline of a system terminates at the visible staff boundary.
- Default tempo uses a quarter-note beat unit unless the source specifies otherwise.
- Tempo output must be reviewable as a resolved composite with distinct beat-unit, equals-sign, and numeric child geometry or equivalent canonical spacing ownership.
- Drum vertical placement must come from one authoritative checked-in mapping table covering all supported render families.
- Up-stems must attach on the notehead's right side with enough outward offset to avoid piercing the glyph body.
- Down-stems must also attach on the notehead's right side unless a specific notehead family has a separately documented exception.
- Stem anchors must be derived from canonical notehead metrics rather than line-centering guesses.
- Unbeamed flags must use dedicated glyph roles or canonical paths, not fallback strokes.
- Slanted beams must be emitted as real beam bodies with vertically cut endcaps, and participating stems must terminate at the resolved beam boundary.

### Migration Gate

The supported-corpus gate for final migration and cutover requires the full supported drum corpus. Representative slices are allowed only for intermediate fixture development, not for final parity approval.

## Addendum 2026-05-18: System Box Pagination Contract

The approved multi-page layout strategy is system-box pagination:

- Plan systems from known page width and content width.
- Render each planned system into a `SystemLayoutBox` in system-local coordinates.
- Compute every system box's `visual_top` and `visual_bottom` from actual emitted item bounds after structural stacking.
- Render page-0 title, subtitle, composer, and tempo content into a separate `HeaderLayoutBox`.
- Paginate ordered system boxes deterministically, then assemble final page scenes by translating local geometry into absolute page-space coordinates.

### Box Placement

`SystemLayoutBox` carries global system identity, local staff origin, local visual bounds, width, local measures, local systems, items, and composites.

`HeaderLayoutBox` carries page-0 header items and actual visual bounds. Page 0's first system cursor is:

`max(top_margin_pt + header_height_pt + header_staff_spacing_pt, header_visual_bottom + header_staff_spacing_pt)`

Later pages start at `top_margin_pt`. A non-first system on a page receives `system_spacing_pt` before placement.

`PlacedSystemBox` carries page index, `page_x`, `page_y`, and the system metadata needed to assemble `SceneSystem` records. Page assembly applies:

`dx = page_x`

`dy = page_y - local_visual_top`

The translation applies to systems, measures, items, composites, line endpoints, rect origins, text/glyph origins, polyline points, path coordinates, and explicit path bounds.

Final `SceneSystem.y_pt` remains the page-space staff/system origin, computed from the local staff origin plus `dy`; it is not the visual top.

### Bounds and Overflow

Every primitive emitted by the layout engine must have deterministic bounds. Bounds cover text runs, glyph runs, line segments, rects, polylines, and all path commands emitted by the engine. Unsupported or unbounded primitives are test failures.

A system taller than an empty page is placed anyway and emits a non-fatal issue using this schema:

`LAYOUT_WARNING overflow page=<index> system=<id> visualHeight=<pt> availableHeight=<pt>`

Existing parser and normalization issues remain preserved in `LayoutScene.issues`.

### References and Adapters

System-local item and composite IDs are remapped during page assembly with deterministic `system-{system_index}-` prefixes. Composite child IDs and item references are rewritten through the remap table. Measure anchors use final measure IDs directly.

For this contract, adapter-rendered composite `start_anchor_id` and `end_anchor_id` must be page-local measure IDs. Item anchors remain valid for individual item attachment, but composite item anchors require a future adapter contract update.

The TypeScript adapter exposes `renderScenePagesToSvgs(scene, options): string[]`, returning one SVG per `ScenePage`. The legacy `renderSceneToSvg(scene, options)` remains page-0-compatible and emits a development warning when asked to render a multi-page scene.

### Validation Gate

Layout tests must validate final scenes for:

- contiguous page indices matching array order
- system page indices matching containing pages
- global item and composite ID uniqueness
- page-local composite child references
- page-local composite measure anchors
- page-local item references
- bounded item containment within page dimensions

Overflow suppresses only bounds failures for the explicitly overflowing system named by a `LAYOUT_WARNING overflow ...` issue. Page order, ID uniqueness, page-local references, header bounds, and unrelated system bounds remain validated.

## Addendum 2026-05-18: Split Parser/Layout WASM and Default Layout Rendering

The approved web runtime architecture separates parser startup from layout rendering:

- parser WASM is the startup package for parser, worker, diagnostics, and editor state
- layout WASM is loaded only when the layout renderer is invoked
- VexFlow remains available as a lazy legacy renderer

### Package Boundaries

Browser production code uses web packages only:

- `src/wasm/parser-pkg-web/`
- `src/wasm/layout-pkg-web/`

CLI and Node initialization use Node packages only:

- `src/wasm/layout-pkg-node/`
- `src/wasm/parser-pkg-node/`, only if needed

The Rust WASM crate is built with explicit features:

- parser: `--target web --no-default-features --features parser-wasm`
- browser layout: `--target web --no-default-features --features layout-wasm`
- Node layout: `--target nodejs --no-default-features --features layout-wasm`

The parser package must not expose layout exports or link `drummark-layout`.

### Render Source Coherence

The app's active parsed score state carries:

`{ score, source, sourceRevision }`

Layout rendering receives `source` and `sourceRevision` explicitly with the score. Production rendering does not use a module-level source cache or `setLayoutSource`.

If parsing is asynchronous, stale parse results cannot replace newer active score/source revisions.

### Default Renderer

The layout engine is the default renderer for users without an explicit saved renderer preference. Explicit saved VexFlow preferences remain respected.

User-facing renderer labels are:

- `Layout Engine`
- `Legacy VexFlow`

### Lazy Runtime Gates

Default app/settings/layout production code must not import VexFlow runtime modules. Shared render settings and option ranges used outside VexFlow live in renderer-neutral modules.

Verification must prove:

- startup can fetch parser WASM without fetching layout WASM or VexFlow
- first default layout render fetches layout WASM and not VexFlow
- first legacy render fetches VexFlow
- CLI SVG output uses the Node layout WASM package
- parser/layout semantic parity holds for successful and failed source fixtures
