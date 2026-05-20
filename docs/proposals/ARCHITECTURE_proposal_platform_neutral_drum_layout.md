## Addendum v1.0: Platform-Neutral Drum Layout Engine to Replace VexFlow

### Status

Proposed. This addendum supersedes the direction of `ARCHITECTURE_proposal_layout_engine.md` for the VexFlow-replacement effort.

### Problem

The current rendering stack still depends on VexFlow for score engraving. That dependency is disproportionately large for DrumMark's actual scope, but the previously attempted Rust layout effort took the wrong architectural shape:

1. It mixed **layout** and **web rendering** into one contract.
2. It treated `source -> WASM -> SVG-ish draw commands` as the product, which is not platform-neutral.
3. It aimed too broadly at "mini VexFlow" instead of a drum-only engine.
4. It used VexFlow output shape as a de facto oracle instead of defining a stable repository-owned rendering contract.

The result is a prototype that is neither a clean layout kernel nor a thin renderer adapter. It is hard to reason about, hard to port to other platforms, and still coupled to browser-oriented output assumptions.

### Goal

Replace VexFlow with a **drum-only** rendering stack built around a **platform-neutral Rust layout engine**.

The target architecture is:

- Rust computes layout only.
- Rust output is a **platform-independent scene description** in logical units.
- Each platform implements a **small translation layer** from scene description to its drawing API.
- The engine only supports the notation surface DrumMark actually uses for drum scores.

### Non-Goals

This proposal does **not** attempt to build a general music engraving engine.

Out of scope:

- pitched notation beyond the drum/sticking surface
- key signatures, accidentals, chord symbols, lyrics
- piano or multi-staff systems
- arbitrary orchestral engraving rules
- editor syntax highlighting
- playback/audio timing
- direct SVG/Canvas/Skia drawing from Rust
- pixel-identical parity with VexFlow internals

### Core Decision

The Rust engine must output a **Layout Scene**, not browser draw commands and not DOM-oriented primitives.

That means the Rust contract must not contain:

- `g_open` / `g_close`
- browser SVG tag names as the primary abstraction
- CSS classes
- VexFlow-specific geometry conventions
- raw source-string parsing as part of layout

It may contain generic geometric primitives and semantic glyph roles, because those are portable.

### Architecture

```
NormalizedScore
    │
    ▼
Rust crate: drummark-layout
    │
    ├─ score analysis
    ├─ system breaking
    ├─ measure geometry
    ├─ event placement
    ├─ beam / stem geometry
    ├─ repeat / volta / nav geometry
    ├─ text anchors
    └─ collision resolution
    │
    ▼
LayoutScene
    │
    ├─ Web SVG adapter
    ├─ Web Canvas adapter
    ├─ Native Skia adapter
    ├─ CoreGraphics adapter
    └─ other future adapters
```

### Input Boundary

The layout crate consumes a normalized rendering model, not raw DSL source.

The required input contract is:

- parser-agnostic
- stable across WASM / native / test harnesses
- sufficient for rendering drum scores without source rescans

At minimum, this means the input is derived from `NormalizedScore` plus any explicitly required render metadata that is not yet represented there.

The layout crate must not:

- read DrumMark source text
- reconstruct syntax from strings
- depend on browser APIs
- depend on VexFlow behavior

### Output Boundary: `LayoutScene`

`LayoutScene` is a platform-neutral, device-independent scene graph in logical units.

Its units are:

- page-space points for page/system geometry
- staff-space relative geometry where useful for note/staff semantics
- explicit final coordinates after layout resolution

Its primitives are semantic-but-portable:

- `GlyphRun`
- `TextRun`
- `LineSegment`
- `Polyline`
- `Rect`
- `Path` or `Polygon` where needed

Its structure is document-oriented:

- pages
- systems
- measures
- items

Each item carries:

- kind
- geometry
- z-order
- semantic role
- stable identifiers back to score structure where needed

Example roles:

- `staff-line`
- `percussion-clef`
- `time-signature-digit`
- `notehead-normal`
- `notehead-x`
- `rest-eighth`
- `stem`
- `beam`
- `repeat-dot`
- `measure-repeat-mark`
- `multi-rest-bar`
- `hairpin-crescendo`
- `volta-bracket`
- `nav-segno`
- `title`
- `tempo`

The output is intentionally not tied to any one renderer's implementation detail. A web adapter may choose to emit SVG `<text>` or `<path>`, while a native adapter may draw via font glyph APIs or cached vector paths.

### Translation Layer Contract

Each platform adapter is intentionally thin.

The adapter is responsible for:

- mapping glyph roles to concrete font glyphs or vector paths
- converting logical coordinates to device coordinates
- painting primitives on the target surface
- theme/color/stroke application
- optional text measurement fallback if a platform requires it

The adapter is not responsible for:

- beam grouping
- stem direction logic
- system breaking
- note spacing
- collision resolution
- repeat-span semantics

If a platform adapter needs to "fix layout," the layout contract is wrong.

### Scope: Drum-Only Rendering Surface

The engine only needs to support the drum notation features used by DrumMark.

Included rendering surface:

- single percussion staff per system
- drum track vertical mapping
- noteheads for DrumMark drum tokens actually used
- rests
- two-voice drum rendering
- combined hits
- stems
- beams
- tuplets/groups only to the extent they affect rendered geometry
- regular/double/final/repeat barlines
- measure-repeat marks
- multi-measure rests
- sticking
- supported modifiers that affect visible output
- hairpins
- navigation markers
- volta brackets
- title/subtitle/composer/tempo
- measure numbers if retained by product requirements

Excluded unless separately approved:

- general grace-note subsystem beyond the exact DrumMark modifier surface
- arbitrary text annotations not already in the product surface
- rehearsal marks unless already normalized and product-required
- MusicXML export redesign

### Layout Responsibilities in Rust

The Rust engine owns:

1. System breaking
2. Measure width allocation
3. Slot/time to X mapping
4. Track/voice to Y mapping
5. Notehead/stem/beam geometry
6. Repeat and volta geometry
7. Navigation placement
8. Hairpin placement
9. Header/text anchor placement
10. Collision resolution for above/below staff elements

The Rust engine must output resolved geometry, not "hints" that require renderer-side layout decisions.

### Data Model Requirements

The `LayoutScene` contract must satisfy these constraints:

1. **Portable**
   - no DOM assumptions
   - no browser API assumptions
   - no JS-only object shapes as the semantic model

2. **Deterministic**
   - same normalized input + same layout options = same scene output

3. **Inspectable**
   - scene output can be snapshot-tested directly
   - failures can be reasoned about without rendering pixels

4. **Composable**
   - one adapter can render SVG
   - another adapter can render Canvas
   - another adapter can render native surfaces
   - none of them need to rerun layout logic

5. **Stable**
   - scene item kinds and required fields form a repository-owned contract
   - adapter churn should not force layout-core churn unless semantics changed

### Migration Strategy

The VexFlow replacement should be staged, not all-or-nothing.

#### Phase 1: Define the Neutral Contract

Deliverables:

- `LayoutScene` schema
- explicit drum-only supported surface
- golden scene fixtures for representative examples

Success condition:

- layout output can be asserted without rendering any SVG

#### Phase 2: Build a Minimal Rust Layout Kernel

Start with:

- systems
- measures
- noteheads
- stems
- beams
- barlines
- text anchors

Do not start with every edge case.

Success condition:

- core examples render correctly through a test adapter

#### Phase 3: Add Drum-Specific Structural Features

Add:

- repeats
- voltas
- navigation
- measure-repeat marks
- multi-rest
- hairpins
- sticking/modifier placement

Success condition:

- the supported docs/example corpus can render through the new scene path

#### Phase 4: Build Thin Platform Adapters

Web first:

- SVG adapter
- optionally Canvas adapter if still needed

Future platforms:

- use the same `LayoutScene`
- only implement drawing translation

Success condition:

- adapter code is materially smaller and simpler than the VexFlow renderer path

#### Phase 5: Cut Over Product Rendering

Only after scene contract and adapter coverage are stable:

- preview path switches off VexFlow
- CLI/doc render path switches off VexFlow
- VexFlow dependency becomes removable

### Acceptance Bar

This proposal is successful only if all of the following are true:

1. The Rust layout engine is **parser-independent**.
2. The Rust layout engine emits a **platform-neutral scene**, not browser draw commands.
3. Web/native renderers only implement **light translation layers**.
4. The supported drum notation corpus renders through the new stack without falling back to VexFlow.
5. Removing VexFlow materially reduces bundle size and codepath complexity.

### Explicit Rejections

This proposal rejects the following directions:

1. **Source-string layout APIs**
   - `build_layout_plan(source, options)` is not the long-term architecture.

2. **Web-shaped scene output**
   - output centered on SVG tags or browser group instructions is not platform-neutral enough.

3. **General-notation ambition**
   - the engine is for DrumMark drum rendering only.

4. **VexFlow pixel-clone requirement**
   - the goal is repository-owned correctness and stable visual quality, not reproducing VexFlow's internal DOM or exact quirks.

### Risks

1. The current `NormalizedScore` may not yet contain every render-semantic field cleanly enough for a renderer-neutral pipeline. If so, the repository will need a small render-facing IR adjustment before the layout engine can be clean.

2. Beam and collision logic are the highest-risk geometry areas. They should be designed as explicit modules with direct fixture coverage, not hidden inside a monolithic orchestrator.

3. If the scene contract is underspecified, platform adapters will accrete layout logic. That is a failure mode and must be treated as an architectural bug, not an adapter convenience.

### Proposed End State

The final repository state should look like this:

- `drummark-core` owns parsing and normalization
- `drummark-layout` owns platform-neutral layout
- `src/renderer/*` owns thin surface adapters only
- VexFlow is removed from active rendering paths
- new platforms can render DrumMark by consuming `LayoutScene` without reimplementing engraving logic

### Review Round 1

1. The proposed `LayoutScene` boundary is not yet actually platform-neutral because text and glyph measurement ownership is left ambiguous. The adapter is allowed to do "optional text measurement fallback if a platform requires it," but the Rust engine also claims ownership of system breaking, measure width allocation, text anchors, collision resolution, and final resolved geometry. Those claims are incompatible unless the proposal fixes one contract:
   - either layout consumes canonical font/glyph metrics as part of its input and all spacing decisions are made in Rust,
   - or adapters are allowed to measure and feed back into layout, which breaks determinism and means the adapter is no longer thin.
   As written, `LayoutScene` is neutral only for primitives, not for the sizing decisions that define engraving.

2. The input boundary hides a major coupling by saying "NormalizedScore plus any explicitly required render metadata that is not yet represented there." That phrase is doing too much work. It leaves open whether the layout crate owns discovery of beaming groups, sticking attachment points, repeat-span anchors, volta extents, text styles, and staff-slot semantics, or whether those must be precomputed upstream. Without a concrete render-facing contract, migration will devolve into back-channel field accretion between `drummark-core` and `drummark-layout`, which is exactly the hidden coupling this proposal claims to avoid.

3. "Page-space points" plus "staff-space relative geometry where useful" is not a stable scene coordinate contract. Mixing coordinate systems inside one scene object is a classic adapter leak. The proposal needs to define whether `LayoutScene` exports:
   - only absolute final coordinates in one canonical space,
   - or a strict hierarchical transform model with explicit local coordinate spaces.
   "Where useful" is not a contract. It invites per-item special cases and platform-specific interpretation bugs, especially once pagination, zooming, hit-testing, or native adapters enter the picture.

4. The scene primitive set is underspecified for the exact features the proposal claims to include. Example: tuplets, hairpins, volta brackets, navigation markers, and multi-measure rests all have semantic spans and label anchors, not just isolated glyphs and lines. If those are represented only as decomposed geometry, adapters lose semantic information needed for accessibility, selection, or future interaction. If they are represented as semantic composites, the proposal needs to say so. Right now the item model says `kind`, `geometry`, `z-order`, `semantic role`, and stable identifiers, but it does not define whether cross-item relationships are first-class. That is a likely source of scene churn and adapter-side reconstruction.

5. The migration strategy is not realistic enough about parity and rollback boundaries. Phase 2 says "core examples render correctly through a test adapter," and Phase 3 says the supported docs/example corpus can render through the new scene path. That does not define:
   - what the golden oracle is before VexFlow is retired,
   - which rendering differences are acceptable versus regressions,
   - whether old and new renderers can coexist per feature during migration,
   - how unsupported-but-currently-working VexFlow cases fail once cutover starts.
   Without a corpus gate and divergence policy, "render correctly" will collapse into subjective screenshot review.

6. The proposal says the adapter must not own beam grouping, stem direction, or collision resolution, but it still allows glyph-role-to-font-path mapping inside the adapter. That is only safe if glyph roles fully determine metrics. In practice, notehead choice, sticking font, tempo text, repeat dots, and navigation symbols all depend on concrete font assets whose advance widths and bounding boxes vary by platform. If those metrics are not part of the layout contract, adapters will inevitably nudge positions. That would violate the claimed "resolved geometry, not hints" rule. This is the main reason I do not yet buy that the current `LayoutScene` boundary is truly platform-neutral.

7. The proposal does not specify how platform-neutrality survives native text shaping differences. Title/subtitle/composer/tempo are explicitly in scope, but text layout across SVG, Canvas, CoreGraphics, and Skia is not interchangeable by default. If text runs are laid out in Rust, the proposal must define the canonical shaping/measurement source. If text runs are only anchored in Rust and shaped by adapters, then adapters own overflow, line breaks, and collision behavior. Right now the contract tries to have it both ways.

8. The crate split is plausible, but the repository end state still blurs normalization and rendering ownership. The proposal says `drummark-core` owns parsing and normalization and `drummark-layout` owns layout, yet the Risks section admits `NormalizedScore` may need render-facing IR adjustment. That is a material architectural dependency and should be elevated into the main design, not left as a risk note. If a render-oriented intermediate contract is required, define it explicitly now; otherwise the first implementation round will discover it informally and bake unstable assumptions into both crates.

9. Edge-case verification is currently missing for the hardest portability cases. I do not see an explicit answer for:
   - how two-voice collisions are represented when stems/beams overlap sticking or hairpins,
   - how repeat/volta spans across system breaks are encoded,
   - how measure-repeat marks and multi-rest labels expose count semantics separate from glyph geometry,
   - how hit-testing or stable item identity survives scene decomposition.
   These are exactly the places where "thin adapter" claims usually fail.

The direction is defensible, but the contract is still too ambiguous to approve as architecture. The main gap is not whether Rust should do layout; it is whether the proposed `LayoutScene` and its input contract are precise enough to prevent measurement/layout logic from leaking back into adapters or upstream normalization.

STATUS: CHANGES_REQUESTED

### Author Response

The prior response was appended in the wrong physical location. This tail response is the authoritative ledger continuation for Review Round 4.

The review is correct. The parity-derived constraints were directionally right, but three of them still left too much room for compliant-looking drift. The following clarifications are binding and supersede any weaker phrasing in Addendum v1.1 above.

#### 1. Drum Vertical Mapping Must Come From One Authoritative Checked-In Table

The required drum-position mapping is not "explicit metadata somewhere." It must come from one repository-owned authoritative mapping surface consumed by layout.

That mapping surface must, for every supported DrumMark render family:

- assign a resolved staff step or ledger-line position
- declare any notehead family needed for that render family
- declare whether the item lives on the staff, in a space, or on a ledger line

Adapters may consume the emitted geometry, but they may not reinterpret staff position by instrument family.

This mapping table is the single source of truth for drum vertical placement. Fixture coverage must prove the full supported family set against that table, not only crash as a spot check. Crash-on-top-ledger-line remains the concrete oracle that exposes the failure immediately, but it is not the only required case.

#### 2. Start-of-System Reservations Must Be Decomposed Into Independent Components

The system-start contract may not use one oversized undifferentiated start zone.

Layout must model distinct reservation components for:

- opening barline thickness at the staff left boundary
- repeated percussion clef width and its required trailing gap
- optional time-signature width and its required trailing gap
- first-note entry offset after the actually rendered preceding components

The governing rules are:

- later systems always reserve opening barline + repeated clef
- later systems reserve time-signature width only if a time signature is actually rendered on that system
- the first playable slot starts immediately after the sum of the rendered components and their canonical inter-component gaps

This is the rule that forbids the current phantom whitespace bug. No unnamed "start padding" bucket is allowed outside those explicit components.

#### 3. Tempo Spacing Must Be a Reviewable Composite Contract

Tempo ownership is not complete unless the scene contract exposes a reviewable tempo composite rather than a vague spacing intention.

The layout engine must emit a tempo composite or equivalent canonical grouped structure whose child geometry distinguishes at minimum:

- beat-unit glyph
- equals-sign glyph or text item
- numeric tempo text

The contract must also carry canonical inter-child spacing ownership, either by:

- explicit child coordinates in the composite, or
- named canonical spacing roles consumed during layout and visible in deterministic scene output

It is not acceptable for different implementations to claim compliance by inventing different hidden spacing rules around the `=` cluster. Review must be able to inspect scene output and determine whether tempo geometry was resolved by layout.

### Consolidated Changes

This terminal section is appended at the physical end of the file and is the only authoritative final consolidation for implementation. Any earlier `### Consolidated Changes` section higher in the ledger is historical and superseded by this terminal synthesis.

The approved architecture is:

`source -> parser AST -> normalized musical model -> RenderScore -> LayoutScene -> platform adapter`

The approved ownership and contract rules are:

- `drummark-core` owns parsing, normalization, and explicit `RenderScore` derivation
- `drummark-layout` owns canonical metrics, deterministic layout decisions, and final scene construction
- `RenderScore` is a closed render-facing IR with explicit timing, voice, attachment, span, text, and count semantics
- `LayoutScene` is serialized in one absolute page-space coordinate system with stable ids, semantic composites, and explicit system-break fragments
- canonical metrics are repository-owned, versioned in-repo, and consumed directly by layout
- platform adapters may not perform measurement, spacing correction, collision resolution, semantic span reconstruction, or layout nudging beyond device-space rounding

The approved engraving constraints are:

- system-start and system-end barlines are resolved layout geometry, not adapter cleanup
- start-of-system reservations are decomposed into explicit components:
  - opening barline thickness at the staff left boundary
  - repeated percussion clef width and its required trailing gap
  - optional time-signature width and its required trailing gap
  - first-note entry offset after the actually rendered preceding components
- later systems always reserve opening barline plus repeated clef, reserve time-signature width only when a time signature is actually rendered there, and may not use any unnamed extra start-padding bucket
- the first playable slot starts immediately after the sum of the rendered components and their canonical inter-component gaps
- the rightmost closing barline of a system terminates at the visible staff boundary
- default tempo uses a quarter-note beat unit unless the source explicitly specifies another beat unit
- tempo geometry is reviewable as a resolved composite with distinct beat-unit, equals-sign, and numeric child geometry or equivalent canonical spacing ownership
- drum vertical placement comes from one authoritative checked-in mapping table covering all supported render families
- up-stems attach on the notehead's right side with enough outward offset to avoid piercing the glyph body
- down-stems also attach on the notehead's right side unless a specific notehead family requires a documented exception
- unbeamed flags use dedicated glyph/path assets rather than fallback strokes
- slanted beams are emitted as real path/polygon bodies with vertically cut endcaps, and participating stems terminate at the resolved beam boundary

The approved migration and execution rules are:

- implementation proceeds only against the approved task list
- fixture/corpus gates are first-class migration controls
- final migration and cutover require the full supported drum corpus
- task completion counts only when the corresponding task items are actually marked done in the tasks file

### Terminal Supersession: VexFlow Comparison Gates

This terminal note is appended after consolidation to preserve historical review context while retiring obsolete comparison gates.

Any remaining text in this proposal that requires VexFlow comparison gates, VexFlow as a parity oracle, VexFlow SVG output, or VexFlow-backed cutover criteria is superseded by `docs/proposals/ARCHITECTURE_proposal_remove_vexflow.md` and `docs/proposals/ARCHITECTURE_tasks_remove_vexflow.md`.

The active post-removal verification model is layout-owned: `RenderScore -> LayoutScene -> thin platform adapter`, semantic SVG output, corpus scene reports, import-boundary enforcement, split-WASM network audit, clean package/build metadata, and representative CLI SVG generation.

### Consolidated Changes

This final section is the authoritative terminal consolidation for the approved proposal. Any earlier `### Consolidated Changes` section or earlier misplaced summary language in this file is superseded by this terminal synthesis.

The approved architecture is:

`source -> parser AST -> normalized musical model -> RenderScore -> LayoutScene -> platform adapter`

The approved ownership and contract rules are:

- `drummark-core` owns parsing, normalization, and explicit `RenderScore` derivation
- `drummark-layout` owns canonical metrics, deterministic layout decisions, and final scene construction
- `RenderScore` is a closed render-facing IR with explicit timing, voice, attachment, span, text, and count semantics
- `LayoutScene` is serialized in one absolute page-space coordinate system with stable ids, semantic composites, and explicit system-break fragments
- canonical metrics are repository-owned, versioned in-repo, and consumed directly by layout
- platform adapters may not perform measurement, spacing correction, collision resolution, semantic span reconstruction, or layout nudging beyond device-space rounding

The approved engraving constraints are:

- system-start and system-end barlines are resolved layout geometry, not adapter cleanup
- start-of-system reservations are decomposed into explicit components:
  - opening barline thickness at the staff left boundary
  - repeated percussion clef width and its required trailing gap
  - optional time-signature width and its required trailing gap
  - first-note entry offset after the actually rendered preceding components
- later systems always reserve opening barline plus repeated clef, reserve time-signature width only when a time signature is actually rendered there, and may not use any unnamed extra start-padding bucket
- the first playable slot starts immediately after the sum of the rendered components and their canonical inter-component gaps
- the rightmost closing barline of a system terminates at the visible staff boundary
- default tempo uses a quarter-note beat unit unless the source explicitly specifies another beat unit
- tempo geometry is reviewable as a resolved composite with distinct beat-unit, equals-sign, and numeric child geometry or equivalent canonical spacing ownership
- drum vertical placement comes from one authoritative checked-in mapping table covering all supported render families
- up-stems attach on the notehead's right side with enough outward offset to avoid piercing the glyph body
- down-stems also attach on the notehead's right side unless a specific notehead family requires a documented exception
- unbeamed flags use dedicated glyph/path assets rather than fallback strokes
- slanted beams are emitted as real path/polygon bodies with vertically cut endcaps, and participating stems terminate at the resolved beam boundary

The approved migration and execution rules are:

- implementation proceeds only against the approved task list
- fixture/corpus gates are first-class migration controls
- final migration and cutover require the full supported drum corpus
- task completion counts only when the corresponding task items are actually marked done in the tasks file

### Consolidated Changes

This terminal section is appended at the physical end of the file and is the only authoritative final consolidation for implementation. Any earlier `### Consolidated Changes` section higher in the ledger is historical and superseded by this terminal synthesis.

The approved architecture is:

`source -> parser AST -> normalized musical model -> RenderScore -> LayoutScene -> platform adapter`

The approved ownership and contract rules are:

- `drummark-core` owns parsing, normalization, and explicit `RenderScore` derivation
- `drummark-layout` owns canonical metrics, deterministic layout decisions, and final scene construction
- `RenderScore` is a closed render-facing IR with explicit timing, voice, attachment, span, text, and count semantics
- `LayoutScene` is serialized in one absolute page-space coordinate system with stable ids, semantic composites, and explicit system-break fragments
- canonical metrics are repository-owned, versioned in-repo, and consumed directly by layout
- platform adapters may not perform measurement, spacing correction, collision resolution, semantic span reconstruction, or layout nudging beyond device-space rounding

The approved engraving constraints are:

- system-start and system-end barlines are resolved layout geometry, not adapter cleanup
- start-of-system reservations are decomposed into explicit components:
  - opening barline thickness at the staff left boundary
  - repeated percussion clef width and its required trailing gap
  - optional time-signature width and its required trailing gap
  - first-note entry offset after the actually rendered preceding components
- later systems always reserve opening barline plus repeated clef, reserve time-signature width only when a time signature is actually rendered there, and may not use any unnamed extra start-padding bucket
- the first playable slot starts immediately after the sum of the rendered components and their canonical inter-component gaps
- the rightmost closing barline of a system terminates at the visible staff boundary
- default tempo uses a quarter-note beat unit unless the source explicitly specifies another beat unit
- tempo geometry is reviewable as a resolved composite with distinct beat-unit, equals-sign, and numeric child geometry or equivalent canonical spacing ownership
- drum vertical placement comes from one authoritative checked-in mapping table covering all supported render families
- up-stems attach on the notehead's right side with enough outward offset to avoid piercing the glyph body
- down-stems also attach on the notehead's right side unless a specific notehead family requires a documented exception
- unbeamed flags use dedicated glyph/path assets rather than fallback strokes
- slanted beams are emitted as real path/polygon bodies with vertically cut endcaps, and participating stems terminate at the resolved beam boundary

The approved migration and execution rules are:

- implementation proceeds only against the approved task list
- fixture/corpus gates are first-class migration controls
- final migration and cutover require the full supported drum corpus
- task completion counts only when the corresponding task items are actually marked done in the tasks file

### Consolidated Changes

The earlier `### Consolidated Changes` section was appended before the final review/response tail completed. This terminal section is the authoritative consolidated synthesis for the approved proposal.

The approved architecture is:

`source -> parser AST -> normalized musical model -> RenderScore -> LayoutScene -> platform adapter`

The approved ownership model is:

- `drummark-core` owns parsing, normalization, and explicit `RenderScore` derivation
- `drummark-layout` owns canonical metrics, deterministic layout decisions, and final scene construction
- platform adapters consume `LayoutScene` and may only translate resolved scene geometry into drawing commands

The approved contract rules are:

- `RenderScore` is a closed render-facing IR with explicit timing, voice, attachment, span, text, and count semantics
- `LayoutScene` is serialized in one absolute page-space coordinate system with stable ids, semantic composites, and explicit system-break fragments
- canonical metrics are repository-owned, versioned in-repo, and used directly by layout
- adapters may not perform measurement, spacing correction, collision resolution, semantic span reconstruction, or layout nudging beyond device-space rounding

The approved engraving constraints are:

- system-start and system-end barlines are resolved layout geometry, not adapter cleanup
- start-of-system reservations are decomposed into independent components:
  - opening barline thickness at the staff left boundary
  - repeated percussion clef width and its required trailing gap
  - optional time-signature width and its required trailing gap
  - first-note entry offset after the actually rendered preceding components
- later systems always reserve opening barline plus repeated clef, reserve time-signature width only when a time signature is actually rendered there, and may not use any unnamed extra start-padding bucket
- the first playable slot starts immediately after the sum of the rendered components and their canonical inter-component gaps
- the rightmost closing barline of a system terminates at the visible staff boundary
- default tempo uses a quarter-note beat unit unless the source explicitly specifies another beat unit
- tempo geometry is reviewable as a resolved composite with distinct beat-unit, equals-sign, and numeric child geometry or equivalent canonical spacing ownership
- drum vertical placement comes from one authoritative checked-in mapping table covering all supported render families
- up-stems attach on the notehead's right side with enough outward offset to avoid piercing the glyph body
- down-stems also attach on the notehead's right side unless a specific notehead family requires a documented exception
- unbeamed flags use dedicated glyph/path assets rather than fallback strokes
- slanted beams are emitted as real path/polygon bodies with vertically cut endcaps, and participating stems terminate at the resolved beam boundary

The approved migration and execution rules are:

- implementation proceeds only against the approved task list
- fixture/corpus gates are first-class migration controls
- final migration and cutover require the full supported drum corpus
- task completion counts only when the corresponding task items are actually marked done in the tasks file

### Review Round 5

The new tail response closes the outstanding Round 4 proposal objections.

1. Drum-family mapping now has a single authoritative checked-in source of truth. The response replaces the earlier loose "`RenderScore` or canonical layout metadata" wording with one repository-owned mapping table consumed by layout, and it requires full-family fixture coverage rather than crash-only spot validation.

2. The system-start reservation contract is now decomposed into independent components with separate reservation rules. Opening barline, repeated clef, optional time signature, and first-note entry offset are all explicitly modeled, and the prohibition on unnamed extra start padding directly closes the phantom-whitespace loophole.

3. Tempo is now reviewable as a concrete composite contract. The response requires distinct child geometry for beat-unit glyph, equals sign, and numeric text, and it defines where spacing ownership lives so scene inspection can verify layout-resolved tempo geometry.

STATUS: APPROVED
### Consolidated Changes

The approved architecture for VexFlow replacement is a platform-neutral ownership chain:

`source -> parser AST -> normalized musical model -> RenderScore -> LayoutScene -> platform adapter`

The consolidated requirements are:

- `drummark-core` owns parsing, normalization, and explicit `RenderScore` derivation
- `drummark-layout` owns all layout-affecting measurement, geometry resolution, and scene construction
- platform adapters are thin and may only translate resolved scene geometry into drawing commands
- no adapter may perform measurement, spacing correction, collision resolution, semantic span reconstruction, or layout nudging beyond device-space rounding

The consolidated `RenderScore` / `LayoutScene` contract requirements are:

- `RenderScore` is a closed render-facing IR with explicit timing, voice, attachment, span, text, and count semantics
- `LayoutScene` is serialized in one absolute page-space coordinate system with stable ids, semantic composites, and explicit system-break fragments
- canonical metrics are repository-owned and versioned in-repo
- scene serialization and migration verification use checked-in fixtures, goldens, and a divergence ledger

The consolidated engraving constraints from parity findings are:

- system-start and system-end barlines are resolved layout geometry, not adapter cleanup
- start-of-system reservations are decomposed into opening barline, repeated clef, optional time signature, and first-note entry offset with no unnamed phantom padding
- the default tempo beat unit is quarter note, and tempo spacing must be reviewable as resolved composite geometry
- drum vertical placement comes from one authoritative checked-in mapping table for all supported render families
- stem anchors are derived from notehead metrics and attach on the resolved notehead side without piercing glyph bodies
- unbeamed flags use dedicated glyph/path assets rather than fallback strokes
- slanted beams are emitted as real path/polygon bodies, and stem lengths are reprojected to the beam boundary

The consolidated execution rules are:

- implementation proceeds only against the approved task list
- fixture/corpus gates are first-class migration controls
- final migration and cutover require the full supported drum corpus
- task completion is recognized only when the corresponding task items are actually marked done in the tasks file

### Author Response

The review is correct. The parity-derived constraints were directionally right, but three of them still left too much room for compliant-looking drift. The following clarifications are binding and supersede any weaker phrasing in Addendum v1.1 above.

#### 1. Drum Vertical Mapping Must Come From One Authoritative Checked-In Table

The required drum-position mapping is not "explicit metadata somewhere." It must come from one repository-owned authoritative mapping surface consumed by layout.

That mapping surface must, for every supported DrumMark render family:

- assign a resolved staff step or ledger-line position
- declare any notehead family needed for that render family
- declare whether the item lives on the staff, in a space, or on a ledger line

Adapters may consume the emitted geometry, but they may not reinterpret staff position by instrument family.

This mapping table is the single source of truth for drum vertical placement. Fixture coverage must prove the full supported family set against that table, not only crash as a spot check. Crash-on-top-ledger-line remains the concrete oracle that exposes the failure immediately, but it is not the only required case.

#### 2. Start-of-System Reservations Must Be Decomposed Into Independent Components

The system-start contract may not use one oversized undifferentiated start zone.

Layout must model distinct reservation components for:

- opening barline thickness at the staff left boundary
- repeated percussion clef width and its required trailing gap
- optional time-signature width and its required trailing gap
- first-note entry offset after the actually rendered preceding components

The governing rules are:

- later systems always reserve opening barline + repeated clef
- later systems reserve time-signature width only if a time signature is actually rendered on that system
- the first playable slot starts immediately after the sum of the rendered components and their canonical inter-component gaps

This is the rule that forbids the current phantom whitespace bug. No unnamed "start padding" bucket is allowed outside those explicit components.

#### 3. Tempo Spacing Must Be a Reviewable Composite Contract

Tempo ownership is not complete unless the scene contract exposes a reviewable tempo composite rather than a vague spacing intention.

The layout engine must emit a tempo composite or equivalent canonical grouped structure whose child geometry distinguishes at minimum:

- beat-unit glyph
- equals-sign glyph or text item
- numeric tempo text

The contract must also carry canonical inter-child spacing ownership, either by:

- explicit child coordinates in the composite, or
- named canonical spacing roles consumed during layout and visible in deterministic scene output

It is not acceptable for different implementations to claim compliance by inventing different hidden spacing rules around the `=` cluster. Review must be able to inspect scene output and determine whether tempo geometry was resolved by layout.

### Author Response

The review is correct: the proposal direction is sound, but the contract was still too soft. The following clarifications are binding and supersede any ambiguous phrasing in the original addendum above.

#### 1. Measurement Ownership Is Resolved

`drummark-layout` owns all layout-affecting measurement used for:

- system breaking
- measure width allocation
- stem / beam placement
- collision resolution
- text anchor placement
- repeat / volta / hairpin span extents

Adapters do **not** measure text or glyphs in order to influence layout. Adapters only draw the already-laid-out scene.

Therefore:

- there is **no adapter measurement feedback loop**
- there is **no adapter-side spacing correction**
- there is **no platform-specific nudge pass**

If a platform cannot draw the canonical metrics exactly, that is a renderer asset problem, not an adapter-layout responsibility split.

#### 2. Canonical Metrics Source

The layout engine must use a repository-owned canonical metrics package for all layout-relevant glyph and text measurements.

That package includes:

- percussion notehead metrics used by DrumMark
- rest symbol metrics used by DrumMark
- repeat / navigation / ornament symbol metrics used by DrumMark
- canonical text style metrics for title / subtitle / composer / tempo / sticking / count labels

The metrics may come from curated font assets or checked-in metric tables, but they must be versioned in-repo and consumed by Rust directly. Platform adapters map scene roles to platform drawing resources that are required to match those canonical metrics closely enough for faithful rendering. They do not get to redefine width, ascent, descent, or anchor behavior.

#### 3. Input Boundary Is Tightened

The layout crate does **not** consume source text.

The layout crate consumes a dedicated render-facing input contract, not an open-ended "NormalizedScore plus whatever else we later need."

That contract may be:

- a narrowed, rendering-owned subset of `NormalizedScore`, or
- an explicit new `RenderScore` / `LayoutInput` IR derived from `NormalizedScore`

This proposal now prefers the second option.

`RenderScore` must explicitly contain all data needed for deterministic layout, including at minimum:

- staff / voice / slot timing semantics
- resolved note/rest events
- beaming / tuplet grouping decisions if owned upstream, or explicit enough timing/group data for Rust to derive them deterministically
- sticking attachments
- dynamics / hairpin anchors
- repeat-span / volta-span / navigation anchors
- measure-repeat and multi-rest semantics
- text blocks with style roles
- visibility and attachment semantics for every renderable feature in scope

No hidden field accretion between `drummark-core` and `drummark-layout` is acceptable. Any new layout dependency must be added to the explicit render IR contract.

#### 4. Scene Coordinate Contract Is Tightened

`LayoutScene` exports **final absolute coordinates in a single canonical page-space**. There is no mixed "staff-space where useful" fallback in the serialized scene contract.

If internal layout code uses local coordinates or transforms, that is an internal Rust implementation detail only. The serialized platform-facing scene contains:

- page dimensions
- absolute geometry for every drawable or semantic composite
- z-order
- stable ids

This avoids adapter interpretation differences and keeps hit-testing and pagination consistent.

#### 5. Scene Model Must Support Semantic Composites

`LayoutScene` is not just a bag of decomposed primitives. It must support semantic composites for cross-item structures, including at minimum:

- tuplets
- hairpins
- volta brackets
- navigation markers
- repeat spans
- multi-rests
- measure-repeat marks
- text blocks

Each composite must expose:

- stable id
- semantic kind
- child geometry
- anchor relationships
- any count / label / continuation metadata needed across system breaks

Adapters may flatten composites into drawing commands for paint, but they must not reconstruct composite meaning from unrelated primitives.

#### 6. Span and System-Break Rules Must Be First-Class

The scene contract must explicitly represent span continuation across systems/pages for:

- voltas
- repeat regions
- hairpins
- multi-rest labels if they extend or relocate

This means the layout engine owns system-break segmentation for span items and emits scene objects that already describe whether a fragment is:

- start
- continuation
- end
- single-segment

Adapters only draw the emitted fragments.

#### 7. Migration Oracle and Divergence Policy

The migration plan needs a hard oracle. During migration, acceptance is not "looks correct."

The required gates are:

1. A checked-in fixture corpus covering the repository's supported drum examples.
2. Scene-level golden outputs for the new engine on that corpus.
3. Side-by-side comparison against the current VexFlow path for supported features, with every intentional divergence recorded in a checked-in divergence ledger.
4. A rule that unsupported VexFlow quirks do not block cutover if they are documented as non-goals or confirmed VexFlow-specific behavior outside the approved drum scope.

VexFlow remains only a temporary migration oracle, not the long-term rendering contract.

#### 8. Thin Adapter Definition

A renderer counts as "thin" only if it is limited to:

- scene traversal
- coordinate unit conversion
- role-to-glyph/path/paint mapping
- draw-order execution
- optional non-layout concerns like accessibility tags or platform event wiring

A renderer is **not** thin if it performs:

- measurement
- line breaking
- collision resolution
- stem or beam geometry decisions
- repeat / volta span reconstruction
- text wrapping or overflow policy
- semantic attachment inference

If any platform requires those behaviors outside Rust, the architecture has failed and the contract must be revised before approval.

#### 9. Render IR Dependency Is Now Explicit

The original Risks section understated the likely need for a render-facing IR. This proposal now treats that as an expected architectural element, not a contingency.

The intended ownership chain is:

`source -> parser AST -> normalized musical model -> RenderScore -> LayoutScene -> platform adapter`

This preserves separation:

- parsing/DSL concerns do not leak into layout
- layout does not need source-level recovery logic
- platform renderers do not need semantic reconstruction

#### 10. Portability Edge Cases Are Now Required Design Targets

Approval and implementation must explicitly cover these cases in fixtures and scene semantics:

- two-voice collisions with sticking, beams, rests, and hairpins
- repeat / volta spans across system breaks
- measure-repeat and multi-rest count semantics distinct from glyph geometry
- stable item ids for hit-testing, selection, and future interaction
- title / subtitle / composer / tempo placement under canonical text metrics

If the scene contract cannot express those cases without adapter inference, it is not complete enough to replace VexFlow.

### Review Round 2

The appended `### Author Response` resolves the architecture-level blockers well enough for implementation planning.

1. Measurement ownership is now explicit and no longer self-contradictory. The proposal clearly assigns all layout-affecting measurement to `drummark-layout`, forbids adapter measurement feedback, and defines adapter-side nudging as an architectural violation. That closes the main ambiguity that previously made the "platform-neutral" claim suspect.

2. The canonical metrics source is now concrete enough. Requiring repository-owned, versioned metrics consumed directly by Rust gives planning a real dependency boundary. There is still execution risk in producing and maintaining those metrics, but that is an implementation burden, not a contract ambiguity.

3. The input boundary is now precise enough to prevent hidden coupling. Replacing the vague "`NormalizedScore` plus whatever else we need" language with an explicit preferred `RenderScore` / `LayoutInput` IR is the correct move. The response also makes field accretion an explicit anti-pattern, which is the key governance rule this proposal was missing.

4. The scene coordinate model is now stable. Committing the serialized scene to a single absolute page-space contract removes the earlier mixed-space leak and gives adapters a deterministic geometry surface.

5. Semantic composites and system-break spans are now first-class, not implied. That is sufficient to make `LayoutScene` more than a bag of paint primitives and materially improves confidence that repeat/volta/hairpin/multi-rest structures can survive renderer translation without adapter reconstruction.

6. Migration realism is now acceptable. The proposal now requires a checked-in corpus, scene-level goldens, side-by-side VexFlow comparison during migration, and a divergence ledger. That is a real oracle strategy rather than subjective screenshot review.

7. The "thin adapter" rule is now operational instead of aspirational. The response lists both allowed and prohibited adapter behaviors in a way that can drive task review and post-change review.

8. The ownership chain is finally explicit: `normalized musical model -> RenderScore -> LayoutScene -> platform adapter`. That is precise enough to support task decomposition across `drummark-core`, the layout crate, and renderer adapters.

Residual risk remains in execution, especially around canonical text/glyph metrics and the exact `RenderScore` schema, but those are now the right things to design in the companion tasks/proposal flow rather than unresolved architecture ambiguity in this document.

STATUS: APPROVED

## Addendum v1.1: Engraving Constraints from Parity Findings

The approved architecture above is still correct, but current implementation findings exposed several scene-level engraving constraints that must be made explicit. The following constraints are binding and supersede any looser interpretation of the earlier addendum.

#### 1. System-Start and System-End Barlines Are Part of the Layout Contract

For every rendered system:

- the left edge of the visible staff is also the left edge of the first measure's opening barline
- the percussion clef sits inside the first measure, to the right of that opening barline
- the first system may additionally place the time signature inside the first measure, after the clef
- later systems must not reserve time-signature width if no time signature is actually rendered there
- the right edge of the last measure's closing barline must terminate at the visible staff boundary, not protrude beyond it

This means the layout engine must model a `system start zone` and `system end closure` explicitly. Adapters may not infer extra leading or trailing whitespace around clef, time signature, or final barlines.

#### 2. Tempo Glyph Semantics Must Be Canonical

Default tempo notation uses a quarter-note beat unit unless the source explicitly encodes another value.

Therefore:

- the canonical tempo beat glyph for the default case is quarter note, not half note
- tempo layout must reserve explicit horizontal padding around the `=` cluster so the beat glyph, equals sign, and numeric value do not visually collide

Tempo mark construction is layout-owned semantic output, not adapter improvisation.

#### 3. Stem Attachment Must Follow Notehead Geometry, Not Stroke Centerlines

Stem X placement is part of resolved note geometry.

The layout engine must emit stem anchors such that:

- up-stems attach on the notehead's right side with enough outward offset to avoid piercing the glyph body
- down-stems also attach on the notehead's right side unless a specific notehead family requires a documented exception
- the offset is derived from canonical notehead metrics, not hard-coded SVG line centering

If a stem visually runs through the notehead, the emitted geometry is wrong.

#### 4. Drum Vertical Mapping Must Be Explicit Per Render Family

The renderer may not treat snare placement as the default and derive the rest loosely.

`RenderScore` or canonical layout metadata must map each supported drum render family to a resolved staff position, including ledger-line requirements where applicable. At minimum, fixtures and metrics must cover the supported DrumMark surface for:

- snare
- kick
- hi-hat variants
- tom families
- crash / cymbal families
- ride families

Crash is the concrete parity oracle here: it belongs on the top ledger line in the current drum layout surface. If that mapping is absent or implicit, the scene contract is incomplete.

#### 5. Flags Are Glyphs, Not Fallback Strokes

Single-note tails for unbeamed durations must use dedicated canonical flag glyph roles, not an improvised line segment.

That applies to both:

- stem-up flags
- stem-down flags

The scene contract may express flags as glyph roles or canonical paths, but not as a generic "short line near the stem" fallback.

#### 6. Slanted Beams Own Stem Reprojection

Whenever a beam is slanted, stem lengths must be recomputed against the resolved beam body.

The contract requirements are:

- beams are emitted as closed path or polygon geometry representing a real beam thickness
- the left and right beam edges remain vertically cut, forming a parallelogram rather than a centerline stroke
- each participating stem terminates exactly at the beam boundary after slope resolution
- no stem may visually overshoot through the beam body

Adapters may paint the supplied beam polygon/path, but they may not derive beam thickness, slope compensation, or stem trimming.

### Review Round 3

1. The new drum-position constraint is directionally correct but still under-specified as a binding architecture rule. Section 4 says the mapping must be explicit per render family, but only names `crash` as a concrete oracle. That still leaves room for a partial implementation where crash is fixed and the rest remain inferred or adapter-dependent. The addendum needs to require one checked-in authoritative mapping surface for all supported drum render families, not just "explicit metadata somewhere."

2. The system-start constraint does not fully close the exact whitespace bug it is trying to ban. Section 1 forbids reserving time-signature width on later systems, but it does not separately define the reservation contract for opening barline, repeated clef, and optional time signature. As written, an implementation could still reserve one oversized undifferentiated "start zone" after the clef on later systems and claim compliance because it is not specifically labeled as time-signature width. The contract needs distinct start-of-system components with independent reservation rules.

3. The tempo-spacing rule is still not precise enough to be reviewable. "Reserve explicit horizontal padding around the `=` cluster" establishes ownership, but not the contract surface: whether that padding comes from canonical spacing roles, fixed metrics entries, or explicit child geometry in a tempo composite. Without that, different implementations can claim compliance while producing different scene structures.

STATUS: CHANGES_REQUESTED

### Review Round 4

No new `### Author Response` was appended after Review Round 3 in this file, so the prior objections remain open.

1. The file still does not require one authoritative checked-in mapping source for all supported drum families. The current wording continues to allow the mapping to live in loosely defined "`RenderScore` or canonical layout metadata", which is not the same as a single reviewable source of truth.

2. The system-start reservation contract is still not decomposed into independently owned components. Opening barline, repeated clef, optional time signature, and any inter-item padding are still collapsed into an underspecified `system start zone`, so the phantom-leading-space bug is not contractually ruled out.

3. The tempo composite is still not defined in a reviewable way. The file still says tempo must reserve padding around the `=` cluster, but it does not state whether that padding is encoded as canonical spacing roles, metrics entries, or explicit child geometry in a semantic tempo object. That leaves multiple incompatible scene contracts claiming compliance.

STATUS: CHANGES_REQUESTED
### Author Response

The prior response was appended in the wrong physical location. This tail response is the authoritative ledger continuation for Review Round 4.

The review is correct. The parity-derived constraints were directionally right, but three of them still left too much room for compliant-looking drift. The following clarifications are binding and supersede any weaker phrasing in Addendum v1.1 above.

#### 1. Drum Vertical Mapping Must Come From One Authoritative Checked-In Table

The required drum-position mapping is not "explicit metadata somewhere." It must come from one repository-owned authoritative mapping surface consumed by layout.

That mapping surface must, for every supported DrumMark render family:

- assign a resolved staff step or ledger-line position
- declare any notehead family needed for that render family
- declare whether the item lives on the staff, in a space, or on a ledger line

Adapters may consume the emitted geometry, but they may not reinterpret staff position by instrument family.

This mapping table is the single source of truth for drum vertical placement. Fixture coverage must prove the full supported family set against that table, not only crash as a spot check. Crash-on-top-ledger-line remains the concrete oracle that exposes the failure immediately, but it is not the only required case.

#### 2. Start-of-System Reservations Must Be Decomposed Into Independent Components

The system-start contract may not use one oversized undifferentiated start zone.

Layout must model distinct reservation components for:

- opening barline thickness at the staff left boundary
- repeated percussion clef width and its required trailing gap
- optional time-signature width and its required trailing gap
- first-note entry offset after the actually rendered preceding components

The governing rules are:

- later systems always reserve opening barline + repeated clef
- later systems reserve time-signature width only if a time signature is actually rendered on that system
- the first playable slot starts immediately after the sum of the rendered components and their canonical inter-component gaps

This is the rule that forbids the current phantom whitespace bug. No unnamed "start padding" bucket is allowed outside those explicit components.

#### 3. Tempo Spacing Must Be a Reviewable Composite Contract

Tempo ownership is not complete unless the scene contract exposes a reviewable tempo composite rather than a vague spacing intention.

The layout engine must emit a tempo composite or equivalent canonical grouped structure whose child geometry distinguishes at minimum:

- beat-unit glyph
- equals-sign glyph or text item
- numeric tempo text

The contract must also carry canonical inter-child spacing ownership, either by:

- explicit child coordinates in the composite, or
- named canonical spacing roles consumed during layout and visible in deterministic scene output

It is not acceptable for different implementations to claim compliance by inventing different hidden spacing rules around the `=` cluster. Review must be able to inspect scene output and determine whether tempo geometry was resolved by layout.

### Consolidated Changes

This terminal section is appended at the physical end of the file and is the only authoritative final consolidation for implementation. Any earlier `### Consolidated Changes` section higher in the ledger is historical and superseded by this terminal synthesis.

The approved architecture is:

`source -> parser AST -> normalized musical model -> RenderScore -> LayoutScene -> platform adapter`

The approved ownership and contract rules are:

- `drummark-core` owns parsing, normalization, and explicit `RenderScore` derivation
- `drummark-layout` owns canonical metrics, deterministic layout decisions, and final scene construction
- `RenderScore` is a closed render-facing IR with explicit timing, voice, attachment, span, text, and count semantics
- `LayoutScene` is serialized in one absolute page-space coordinate system with stable ids, semantic composites, and explicit system-break fragments
- canonical metrics are repository-owned, versioned in-repo, and consumed directly by layout
- platform adapters may not perform measurement, spacing correction, collision resolution, semantic span reconstruction, or layout nudging beyond device-space rounding

The approved engraving constraints are:

- system-start and system-end barlines are resolved layout geometry, not adapter cleanup
- start-of-system reservations are decomposed into explicit components:
  - opening barline thickness at the staff left boundary
  - repeated percussion clef width and its required trailing gap
  - optional time-signature width and its required trailing gap
  - first-note entry offset after the actually rendered preceding components
- later systems always reserve opening barline plus repeated clef, reserve time-signature width only when a time signature is actually rendered there, and may not use any unnamed extra start-padding bucket
- the first playable slot starts immediately after the sum of the rendered components and their canonical inter-component gaps
- the rightmost closing barline of a system terminates at the visible staff boundary
- default tempo uses a quarter-note beat unit unless the source explicitly specifies another beat unit
- tempo geometry is reviewable as a resolved composite with distinct beat-unit, equals-sign, and numeric child geometry or equivalent canonical spacing ownership
- drum vertical placement comes from one authoritative checked-in mapping table covering all supported render families
- up-stems attach on the notehead's right side with enough outward offset to avoid piercing the glyph body
- down-stems also attach on the notehead's right side unless a specific notehead family requires a documented exception
- unbeamed flags use dedicated glyph/path assets rather than fallback strokes
- slanted beams are emitted as real path/polygon bodies with vertically cut endcaps, and participating stems terminate at the resolved beam boundary

The approved migration and execution rules are:

- implementation proceeds only against the approved task list
- fixture/corpus gates are first-class migration controls
- final migration and cutover require the full supported drum corpus
- task completion counts only when the corresponding task items are actually marked done in the tasks file
