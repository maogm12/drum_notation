# RENDERER_tasks_hairpin_bottom_skyline.md

### Task 1: Add BottomSkyline class and constants
- [x] **Status**: Done
- **Scope**: `src/vexflow/renderer.ts`
- **Commits**: `feat(renderer): add BottomSkyline class mirroring TopSkyline`
- **Acceptance Criteria**:
  - `BottomSkyline` class exists next to `TopSkyline` in renderer.ts
  - Same bucket-based architecture (SKYLINE_BUCKET_WIDTH = 4)
  - `sample()` returns max Y in range (or fallback if unoccupied)
  - `occupy()` sets `Math.max(current, bottomY)` per bucket
  - `fallbackBottom` computed from `stave.getBottomLineY()`
  - `Number.NEGATIVE_INFINITY` initialization; `Number.isFinite` fallback with explanatory comment
  - Constants added: `SKYLINE_GAP_BELOW = 6`, `BEAM_THICKNESS = 3`, `HAIRPIN_FULL_HEIGHT = 16`
  - Verified that `Stave.prototype.getBottomLineY` exists in installed vexflow (runtime prerequisite)
- **Dependencies**: None

### Task 2: Build BottomSkyline in buildSystemLayoutState
- [x] **Status**: Done
- **Scope**: `src/vexflow/renderer.ts` (`buildSystemLayoutState` function, new `occupyNoteInBottomSkyline` function)
- **Commits**: `feat(renderer): build BottomSkyline from note geometry in system layout`
- **Acceptance Criteria**:
  - `SystemLayoutState` type extended with `bottomSkyline: BottomSkyline`
  - `skyline` field NOT renamed (stays `skyline: TopSkyline`)
  - `buildSystemLayoutState` constructs `BottomSkyline` after TopSkyline
  - `fallbackBottom = Math.max(...staves.map(s => s.getBottomLineY()))`
  - `occupyNoteInBottomSkyline` iterates layout notes per measure
  - DOWN stem → `note.getStemExtents().bottomY` (+`BEAM_THICKNESS` if `note.beam`)
  - UP stem / no stem → `Math.max(...note.getYs())`
  - Occupied X range: `absoluteX ± glyphWidth/2 ± 2` (mirrors `occupyNoteInSkyline`)
  - `npm run build` passes with new types
- **Dependencies**: Task 1

### Task 3: Wire BottomSkyline into hairpin placement
- [x] **Status**: Done
- **Scope**: `src/vexflow/renderer.ts` (hairpin rendering loop at line 971, clipping logic)
- **Commits**: `feat(renderer): auto-place hairpins via BottomSkyline`
- **Acceptance Criteria**:
  - `buildSystemLayoutState` call at line 969 remains BEFORE hairpin rendering loop at line 971 (call order is now load-bearing)
  - Rendering loop accesses `bottomSkyline` via `systemLayout.bottomSkyline` (no signature change to `buildHairpinSpans` or `renderSystem`)
  - 3-point skyline sampling per hairpin: left, center, right of `[clipLeftX, clipRightX]`
  - `targetHairpinTopY = Math.max(leftY, centerY, rightY) + SKYLINE_GAP_BELOW`
  - `yShift = targetHairpinTopY - startAnchor.y + (options.hairpinOffsetY ?? 0)`
  - After rendering each hairpin: `bottomSkyline.occupy(clipLeftX, clipRightX, targetHairpinTopY + HAIRPIN_FULL_HEIGHT)`
  - Clipping: `clipH = (targetHairpinTopY + HAIRPIN_FULL_HEIGHT) - clipY + 6` (dynamic, replaces hardcoded +60)
  - Multi-rest / empty measures: BottomSkyline falls back to `getBottomLineY()` (no crash)
  - Hairpins sorted by `startMeasureIndex` ascending for deterministic stacking
- **Dependencies**: Task 2

### Task 4: Update settings defaults and range
- [x] **Status**: Done
- **Scope**: `src/hooks/useAppSettings.ts`, `src/vexflow/types.ts`
- **Commits**: `refactor(settings): change hairpinOffsetY range to 0..20 and default to 0`
- **Acceptance Criteria**:
  - `useAppSettings.ts`: `hairpinOffsetY` default changed from -15 to 0
  - `useAppSettings.ts`: validation range changed from `-40..40` to `0..20`
  - `types.ts` (if comment exists): updated to reflect new semantics
- **Dependencies**: Task 1 (constants only; can run parallel to Tasks 2-3)

### Task 5: Update smoke tests and add BottomSkyline unit tests
- [x] **Status**: Done
- **Scope**: `src/vexflow/smoke.test.ts`
- **Commits**: `test(renderer): update hairpin smoke tests for skyline-based placement`
- **Acceptance Criteria**:
  - `smoke.test.ts` hairpin rendering tests updated for new Y coordinates
  - Expected SVG fragments reflect skyline-determined positions
  - No test regressions in non-hairpin assertions
  - `npm run test` passes all
- **Dependencies**: Task 3

### Task 6: Integration verification
- [x] **Status**: Done
- **Scope**: CLI + build pipeline
- **Commits**: `chore: regression check after hairpin bottom skyline`
- **Acceptance Criteria**:
  - `npm run drummark -- docs/examples/basic.drummark --format svg` produces valid SVG
  - `npm run drummark -- docs/examples/basic.drummark --format ir` unchanged
  - `npm run drummark -- docs/examples/basic.drummark --format ast` unchanged
  - `npm run build` succeeds
  - `npm run test` succeeds
- **Dependencies**: Task 5

---

### Review Round 1

#### Issue 1: Task 3 scope is vague on integration point

Task 3 says `buildHairpinSpans` accepts `bottomSkyline: BottomSkyline` parameter, but the skyline query + yShift computation actually happens in the rendering loop (lines 971-1018), not inside `buildHairpinSpans`. `buildHairpinSpans` only builds `HairpinSpan[]` objects with firstNote/lastNote/clip boundaries — it does not compute yShift.

The rendering loop already has access to `systemLayout` via closure; it can capture `systemLayout.bottomSkyline` directly. `buildHairpinSpans` signature does not need to change.

Fix: Task 3 should specify that `bottomSkyline` is passed into the rendering loop (via `systemLayout.bottomSkyline` closure), not into `buildHairpinSpans`.

#### Issue 2: Missing explicit call-order prerequisite

The current code at line 969-971 has the correct order (`buildSystemLayoutState` before `buildHairpinSpans`), but this ordering is now load-bearing: BottomSkyline must be fully populated (with note geometry occupied) BEFORE the hairpin rendering loop queries it. The tasks file should call this out explicitly.

Fix: Add to Task 3 acceptance criteria: "`buildSystemLayoutState` call at line 969 remains before hairpin rendering loop at line 971 — no reordering regression."

#### Issue 3: Tasks file doesn't mention the `stave.getBottomLineY()` prerequisite

Task 1 just says `fallbackBottom computed from stave.getBottomLineY()`, but doesn't verify this API exists in the installed VexFlow version. The VexFlow stave module does expose `getBottomLineY()` (confirmed in `stave.js` line 192), but a quick smoke check at implementation time avoids a runtime crash.

Fix: Add to Task 1 acceptance criteria: "Verified that `Stave.prototype.getBottomLineY` exists in installed vexflow."

#### Issue 4: No task covers the renderSystem function signature change

`renderSystem` currently doesn't receive `bottomSkyline`. With the new design, `bottomSkyline` is embedded in `systemLayout` returned by `buildSystemLayoutState`. The rendering loop gets it from `systemLayout.bottomSkyline`. This is already implied by Task 2 (extending `SystemLayoutState`), but the rendering loop's access to `bottomSkyline` through `systemLayout` should be in Task 3 acceptance criteria.

Fix: Add to Task 3: "Rendering loop accesses `systemLayout.bottomSkyline` (no new parameter on renderSystem)."

#### Minor Issues

**5. Task 2 should verify backward compat** — `buildSystemLayoutState` return type change is additive (`bottomSkyline?: BottomSkyline` or required). Existing callers (line 969 in renderSystem, any test mocks) must still compile. Add `npm run build` to Task 2 acceptance criteria (already present).

**6. Task 4 dependencies** — Listed as "Task 3 (to understand final yShift computation)" which is correct, but the settings change can be done in parallel with Task 3 since it only changes default values and validation range. Consider making Task 4 independent (depends on nothing, or depends on Task 1 for constants).

#### Design Strengths

- Task granularity is appropriate — each task is a coherent unit of work with clear deliverables
- Dependencies form a clean linear chain (1→2→3→4/5→6)
- Acceptance criteria are specific and verifiable
- The tasks file stays within the proposal's scope boundaries (no parser/DSL/IR spill)

#### STATUS: CHANGES_REQUESTED

Issues #1 (function signature), #2 (call order), and #3 (API verification) need refinement. All are documentation-level fixes, not design changes.

### Author Response

**Response to #1**: Agreed. `buildHairpinSpans` signature remains unchanged. The rendering loop captures `bottomSkyline` from `systemLayout.bottomSkyline` which was already built in `buildSystemLayoutState`. Updated Task 3 acceptance criteria to reflect this.

**Response to #2**: Added explicit acceptance criteria to Task 3: "call order at line 969-971 preserved (buildSystemLayoutState before hairpin loop)."

**Response to #3**: Added API verification note to Task 1: "Verified `Stave.prototype.getBottomLineY` exists."

**Response to #4**: Added to Task 3: "Rendering loop accesses `bottomSkyline` via `systemLayout.bottomSkyline` (no signature change to renderSystem)."

**Response to #6**: Task 4 can be done in parallel with Task 3. Changed dependency from Task 3 to Task 1 (constants only).

### Review Round 2

All four issues addressed via Author Response + task file updates below. The tasks are now precise about integration mechanics and carry no design ambiguity.

**Verification:**

| Issue | Status |
|-------|--------|
| #1 signature confusion | ✅ Fixed — `buildHairpinSpans` unchanged; rendering loop uses closure |
| #2 call order | ✅ Fixed — explicit acceptance criteria in Task 3 |
| #3 API verification | ✅ Fixed — added to Task 1 |
| #4 renderSystem access | ✅ Fixed — explicit in Task 3 |
| #5 backward compat | ✅ Already covered |
| #6 Task 4 dependency | ✅ Changed to Task 1 |

#### STATUS: APPROVED

Tasks file is ready for user stamp.
### Supersession Note: 2026-05-20 VexFlow Removal

This VexFlow-specific task stream is superseded by `ARCHITECTURE_proposal_remove_vexflow.md`.

Hairpin skyline behavior is now owned by `drummark-layout` and verified through layout scene, SVG adapter, CLI, and corpus coverage rather than `src/vexflow/**`.
