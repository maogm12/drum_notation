# DRUMMARK_SPEC_tasks_rehearsal_marks.md

## Execution Plan: Rehearsal Marks (Measure-Level Binding)

### Task 1: Grammar — Add RehearsalMark structural rule to TrackBody
- [ ] **Status**: Pending
- **Scope**: `src/dsl/drum_mark.grammar`
- **Commits**:
  - `feat(grammar): add RehearsalMark structural rule for measure-level rehearsal marks`
  - **Design**: `RehearsalMark { "[" RehearsalContent "]" }` where `RehearsalContent` reuses `HeaderWord+` (already handles multi-word space-separated text). NOT a single-regex tokenizer rule — the regex `/\[([^\]]+)\]/` would greedily consume `GroupExpr` content.
  - **Disambiguation**: `RehearsalMark` appears at `TrackBody`/`TrackLine` level; `GroupExpr` appears inside `MeasureExpr`. Lezer's LR parser distinguishes them by grammar state. No tokenizer conflict.
  - **TrackBody rule**: `TrackBody { RehearsalMark? TrackLine (Newline+ RehearsalMark? TrackLine)* }` — rehearsal marks can appear before any track group, not just at paragraph start.
  - **Validation**: `[A] HH | ...` (track on same line) must be a parse error. Enforced by `RehearsalMark` being its own line — the next token after `]` must be `Newline`.
- **Acceptance Criteria**:
  - `npm run drummark` parses `[A]\nHH | d - d - |` correctly.
  - `npm run drummark` parses multiple rehearsal marks in same paragraph (`[A]\nHH|...|\n[B]\nHH|...|`) correctly.
  - `npm run drummark` parses `[Section A]\nHH | x x |` (spaces in label).
  - `npm run drummark` rejects `[A] HH | d - |` as parse error.
  - Existing `.drummark` files parse without regression.
- **Dependencies**: none

### Task 2: IR Types — Add rehearsalMark field to NormalizedMeasure
- [ ] **Status**: Pending
- **Scope**: `src/dsl/types.ts`
- **Commits**:
  - `feat(ir): add rehearsalMark field to NormalizedMeasure`
  - Add `rehearsalMark?: string` to `NormalizedMeasure` (alongside `startNav`, `endNav`, `volta`).
  - No separate `RehearsalMark` type or top-level `rehearsalMarks` array — the mark is a per-measure property.
  - Also add `rehearsalMark?: string` to `ParsedMeasure` (line ~150) so it flows through `ScoreMeasure`.
- **Acceptance Criteria**:
  - TypeScript compilation passes.
  - `NormalizedMeasure.rehearsalMark` is an optional string field.
- **Dependencies**: none (can run parallel to Task 1)

### Task 3: Lezer Skeleton & AST — Extract rehearsal marks from parse tree
- [ ] **Status**: Pending
- **Scope**: `src/dsl/lezer_skeleton.ts`, `src/dsl/ast.ts`
- **Commits**:
  - `feat(parser): extract RehearsalMark nodes from Lezer parse tree into ScoreMeasure`
  - Map `RehearsalMark` grammar nodes → extract label text → attach to the first `ScoreMeasure` of the following track group.
  - Multiple `RehearsalMark` nodes in the same paragraph each bind to their respective following measure group.
- **Acceptance Criteria**:
  - `npm run drummark --format ir <file-with-rehearsal>` shows `rehearsalMark` field on the correct measures.
  - Single paragraph with `[A]\nHH|...|\n[B]\nHH|...|` produces two measures, each with the correct label.
- **Dependencies**: Task 1, Task 2

### Task 4: Normalization — Propagate rehearsalMark to NormalizedMeasure
- [ ] **Status**: Pending
- **Scope**: `src/dsl/normalize.ts`
- **Commits**:
  - `feat(normalize): propagate rehearsalMark through normalization to NormalizedMeasure`
  - In `normalizeScoreAst`, merge `rehearsalMark` from `trackMeasures` into `measureMeta` (line ~563) and stamp onto `NormalizedMeasure` (line ~580).
  - Add validation: `rehearsalMark` + `startNav` with `anchor: "left-edge"` on the same measure → hard error. Non-left-edge `startNav` anchors (`eventAfter: Fraction`) are permitted.
- **Acceptance Criteria**:
  - `npm run drummark --format ir` on the proposal examples produces `rehearsalMark` on the correct measures.
  - Rehearsal mark + `@segno` at left-edge on same measure produces validation error.
  - `[A] HH | d - @segno d - |` (interior nav) is valid.
- **Dependencies**: Task 3

### Task 5: VexFlow Rendering — Boxed rehearsal marks above target measure
- [ ] **Status**: Pending
- **Scope**: `src/vexflow/renderer.ts`
- **Commits**:
  - `feat(renderer): render boxed rehearsal marks above the bound measure`
  - In `renderSystem`, after drawing each measure, check `measure.rehearsalMark`. If present, render a boxed label above the staff.
  - Box: manual SVG `<rect>` around the label text bounding box (VexFlow has no native box enclosure).
  - Left-align the mark with the measure's first visible element (repeat barline takes precedence over first note).
  - Register box in skyline to avoid collision with other annotations.
  - Measure with no events but a rehearsal mark: draw an empty bar with the mark above.
- **Acceptance Criteria**:
  - `npm run drummark --format svg` on rehearsal mark example produces visible boxed `[A]`, `[B]` above the correct measures.
  - Mid-paragraph rehearsal mark renders above the correct measure (not just at system start).
  - Rehearsal mark + repeat-start barline: mark aligns with barline edge.
  - `npm run drummark` confirms no regressions on existing test files.
- **Dependencies**: Task 4

### Task 6: MusicXML Export — Add rehearsal elements per measure
- [ ] **Status**: Pending
- **Scope**: `src/dsl/musicxml.ts`
- **Commits**:
  - `feat(musicxml): export rehearsal marks as MusicXML rehearsal elements per measure`
  - When `measure.rehearsalMark` is set, emit at the measure's start:
    ```xml
    <direction placement="above">
      <direction-type>
        <rehearsal>label</rehearsal>
      </direction-type>
    </direction>
    ```
  - Default enclosure: `"box"` (omit `enclosure` attribute or set `enclosure="square"`).
- **Acceptance Criteria**:
  - `npm run drummark --format xml` on rehearsal mark example produces valid MusicXML with `<rehearsal>` elements at correct measures.
- **Dependencies**: Task 4

### Task 7: Syntax Highlighting — Add rehearsal mark token colors
- [ ] **Status**: Pending
- **Scope**: `src/drummark.ts` (or relevant editor/highlighting module)
- **Commits**:
  - `feat(highlight): add syntax highlighting for [label] rehearsal marks`
- **Acceptance Criteria**:
  - `[A]`, `[Intro]` lines are visually distinct in the editor.
- **Dependencies**: Task 1

### Task 8: Consolidate into SPEC
- [ ] **Status**: Pending
- **Scope**: `docs/DRUMMARK_SPEC.md`
- **Commits**:
  - `docs(spec): consolidate rehearsal marks addendum into DRUMMARK_SPEC.md`
  - Append clean Addendum reflecting the measure-level binding (Post-Amendment Consolidated Changes).
- **Acceptance Criteria**:
  - DRUMMARK_SPEC.md contains the final approved rehearsal mark spec: `[label]` syntax, `NormalizedMeasure.rehearsalMark` field, per-measure rendering, and MusicXML.
- **Dependencies**: Task 4 (core feature working)
### Supersession Note: 2026-05-20 VexFlow Removal

Any uncompleted task in this file that targets `src/vexflow/**` or VexFlow-specific rendering is superseded by `ARCHITECTURE_proposal_remove_vexflow.md`.

Rehearsal-mark rendering implementation, if resumed, must target the current `RenderScore -> LayoutScene -> thin adapter` architecture rather than VexFlow.
