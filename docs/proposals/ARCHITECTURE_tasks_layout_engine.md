## Tasks: Custom Lightweight Staff Layout Engine

### Task 1: Crate Scaffold + Coordinate Types + LayoutOptions
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/`
- **Commits**:
  - `feat(layout): scaffold standalone drummark-layout crate with coordinate types and LayoutOptions`
- **Acceptance Criteria**:
  - Standalone Cargo crate, no dependency on `drummark-core`
  - `LayoutOptions` struct: page size, margins, staff scale, `px_per_quarter`, per-element Y offsets, `edge_padding`
  - `StaffSpace` type (default: 8pt at 40pt staff, interoperable with SMuFL staff-space units)
  - `StaffY` enum: drum kit vertical positions (HH=0, T1=-2, SD=4, BD=8, etc.)
  - `glyph_metrics()` function: returns `{ width_ss, height_ss, stem_offset_y }` per SMuFL codepoint (hardcoded table matching SMuFL standard)
  - `cargo build` succeeds, `cargo test` passes
- **Dependencies**: None

### Task 2: Slot → X + Measure + System Layout
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/system.rs`
- **Commits**:
  - `feat(layout): implement slot-to-X mapping, measure width, and system layout`
- **Acceptance Criteria**:
  - **Slot→X**: `fn slot_x(slot, slots_per_beat, beat_width) -> f32` with content-weighted spacing
  - Weight bonus: ≤ 1/16 note density → +15% width; beat-group boundaries → +8% width
  - `beat_width` varies per beat-group (e.g., `3+2` → beat 1 = 3/5 of measure width)
  - **Measure width**: regular measures → `total_slots * px_per_slot`; compact: measure-repeat (40pt), multi-rest (60pt)
  - **SystemBuilder**: `fn build_systems(measures, page_width) -> Vec<System>` with break logic
  - **System**: `{ y, height, measures: [{ x, width, elements: [] }] }`
  - Clef + time sig space reserved at start of first system (30pt + 40pt)
  - Unit tests: 4/4 at 80 px/quarter → measure=320pt; 3/4 → 240pt; system break at 612pt
  - Independent of Tasks 3-6 (can stub with empty measures)
  - `cargo test` passes
- **Dependencies**: Task 1

### Task 3: Note & Rest Placement
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/notes.rs`
- **Commits**:
  - `feat(layout): implement note, rest, sticking, and combined hit placement`
- **Acceptance Criteria**:
  - **Note**: X from `slot_x(event.start)`, Y from `StaffY::for_track(track, family)`, stem direction from voice
  - **Combined hit**: same-X, same-voice events → single stem, stacked noteheads
  - **Glyph selection**: each note element carries a SMuFL codepoint identifier from the `glyph_metrics()` table (Task 1), selected by track + modifier
  - **Rest**: Y at voice-appropriate staff position, X from `slot_x(event.start)`
  - **Sticking**: Y = staff_top + `sticking_offset_y`, X aligned to note
  - Voice shift = 0 by default (track mapping already separates voices)
  - Input: single `NormalizedMeasure`, output: `Vec<LayoutElement>` (notes, rests, sticking, combined-hit chords only)
  - Tested in isolation: feed a hand-rolled measure, verify element positions
  - `cargo test` passes
- **Dependencies**: Task 1, Task 2 (uses `slot_x` and `StaffY`)

### Task 4: Modifier & Grace Note Placement
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/modifiers.rs`
- **Commits**:
  - `feat(layout): implement modifier annotation and grace note placement`
- **Acceptance Criteria**:
  - Each modifier gets a Y rule: accent `>` (+6pt above notehead), ghost `( )` (surround notehead), etc.
  - Flam → single-slash grace note; drag → double-slash; roll → 3 tremolo slashes on stem
  - Input: note elements from Task 3 output, output: modifier elements (grace notes, annotations, tremolos)
  - Tested independently with hand-rolled note elements
  - `cargo test` passes
- **Dependencies**: Task 1

### Task 5: Beam Layout
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/beams.rs`
- **Commits**:
  - `feat(layout): implement beam group detection and layout`
- **Acceptance Criteria**:
  - **Grouping**: consecutive beamable notes (≤ eighth) in same voice → one beam group
  - **Beam Y**: average of first/last note Y in group
  - **Slope**: `(last_y - first_y) / (last_x - first_x)`, max 20°
  - **Notehead intersection test**: beam must not cross any notehead in group
  - **Stem length**: note → beam distance (one per note in group)
  - **Tuplet bracket**: adjacent to beam or at stem ends
  - Input: note elements from Task 3, output: beam + stem elements
  - Tested independently with hand-rolled note arrays
  - `cargo test` passes
- **Dependencies**: Task 1

### Task 6: Structural Elements
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/structural.rs`
- **Commits**:
  - `feat(layout): implement barline, hairpin, volta, navigation, clef, time sig placement`
- **Acceptance Criteria**:
  - **Barlines**: regular, double, repeat-start, repeat-end, repeat-both, final at measure boundaries
  - **Measure-repeat**: N=1 → centered `%`; N=2 → two-bar repeat overlay
  - **Multi-rest**: H-bar + count, centered
  - **Hairpin**: X from start/end events, Y = `staff_bottom + hairpin_offset_y`
  - **Volta**: bracket spanning measure sequence, Y = staff_top + volta_offset_y
  - **Navigation**: segno/coda at left edge, fine/DC/DS at right edge, Y = staff_top + nav_offset_y
  - **Clef**: percussion clef at system start (30pt width)
  - **Time sig**: at system start after clef (40pt width)
  - Input: measure definitions (from NormalizedScore) + system geometry (from Task 2), output: structural LayoutElements
  - Tested independently per element type
  - `cargo test` passes
- **Dependencies**: Task 2 (measure boundaries)

### Task 6.5: Text & Header Placement
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/text.rs`
- **Commits**:
  - `feat(layout): implement title, subtitle, composer, tempo, and measure number placement`
- **Acceptance Criteria**:
  - Title: centered above first system at `staff_top + text_offset_y`
  - Subtitle: below title
  - Composer: right-aligned
  - Tempo: left-aligned above first system
  - Measure numbers: above each barline at `staff_top + measure_num_offset_y`
  - Font sizes configurable (title: 24pt, subtitle: 18pt, tempo: 14pt, measure: 10pt)
  - Input: header data + system layout from Task 2, output: text LayoutElements
  - Tested independently with mock header + system data
  - `cargo test` passes
- **Dependencies**: Task 2

### Task 7: Edge Element Stacking
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/stacking.rs`
- **Commits**:
  - `feat(layout): implement priority-based edge element stacking with fixpoint collision resolution`
- **Acceptance Criteria**:
  - Priority order: modifier → sticking → hairpin → text/rehearsal → navigation → volta
  - Fixpoint loop: pairwise overlap check, push lower-priority outward, repeat until no overlaps or max 5 passes
  - Mobility flags: `can_shift_y: bool`, `can_shift_x: bool` per element type
  - Volta: `can_shift_y: limited` (±0.5 staff space)
  - Tempo + measure numbers: `can_shift_y: yes`, `can_shift_x: limited`
  - Unresolvable overlaps → warning emitted, best-effort position
  - Input: ordered list of edge elements, output: same elements with resolved Y positions
  - Tested independently with hand-crafted collision scenarios
  - `cargo test` passes
- **Dependencies**: None (pure algorithm, takes generic element list)

### Task 8: Layout Orchestrator + WASM Export
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-layout/src/lib.rs`
- **Commits**:
  - `feat(layout): implement layout orchestrator calling modules 2-7 in sequence`
  - `feat(wasm): export layout_plan(score_jsv, options_jsv) -> JsValue`
- **Acceptance Criteria**:
  - `fn layout_score(score: &NormalizedScore, opts: &LayoutOptions) -> LayoutPlan`
  - Calls: System (Task 2) → Notes (Task 3) → Modifiers (Task 4) → Beams (Task 5) → Structural (Task 6) → Text (Task 6.5) → Stacking (Task 7)
  - `NormalizedScore` struct defined directly in `drummark-layout` (self-contained types, no dependency on `drummark-core`)
  - `LayoutPlan` → `JsValue` via `js_sys::Object`/`js_sys::Array` (no serde, no JSON round-trip)
  - `#[wasm_bindgen] pub fn layout_plan(score: JsValue, options: JsValue) -> JsValue` — accepts JsValue trees from JS
  - `wasm-pack build --target web` produces working `.wasm` + JS glue
  - Integration test: hand-crafted NormalizedScore → layout → verify element counts
  - `cargo test` passes
- **Dependencies**: Tasks 1–7

### Task 9: Canvas 2D Renderer
- [ ] **Status**: Pending
- **Scope**: `src/renderer/canvas.ts`
- **Commits**:
  - `feat(renderer): implement Canvas 2D renderer consuming LayoutPlan`
- **Acceptance Criteria**:
  - Renders every element kind in LayoutPlan (notes, rests, barlines, beams, modifiers, hairpins, voltas, navigation, text)
  - Bravura font loaded via `FontFace` API or CSS `@font-face`
  - Staff-space → pixel: `ss * staff_scale * staff_height_px / 4`
  - `<canvas>` replaces VexFlow `<div>` in preview pane
  - Works in continuous scroll and paged modes
  - Tested against golden LayoutPlan fixtures
- **Dependencies**: Task 8

### Task 10: Integration & Parity
- [ ] **Status**: Pending
- **Scope**: Pipeline wiring, structural parity tests, VexFlow removal
- **Commits**:
  - `feat(pipeline): wire LayoutPlan canvas renderer into preview pane`
  - `chore(deps): remove VexFlow from package.json`
  - `test(renderer): add structural parity tests`
- **Acceptance Criteria**:
  - Preview uses Canvas renderer (not VexFlow)
  - `npm run build-docs` produces visually correct output for all 22 examples
  - Structural parity: element counts, barline types, voice assignments, measure counts match VexFlow output for 10 examples
  - All existing 468 JS tests pass (or adapted)
  - No VexFlow in `package.json`; `npm ls vexflow` returns empty
- **Dependencies**: Task 9
### Supersession Note: 2026-05-20 VexFlow Removal

This older layout-engine task stream is superseded for renderer-removal work by `ARCHITECTURE_proposal_remove_vexflow.md`.

Any uncompleted parity or removal task in this file that compares against VexFlow is historical only; active verification is layout-owned scene, SVG semantic, CLI, and corpus coverage.
