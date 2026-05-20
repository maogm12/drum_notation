# Tasks: Remove Legacy VexFlow Renderer

### Task 1: Create VexFlow Coverage Migration Matrix
- [x] **Status**: Done
- **Scope**: `docs/proposals/ARCHITECTURE_tasks_remove_vexflow.md` or companion appendix, VexFlow-era test inventory
- **Commits**:
  - `docs(renderer): map VexFlow-era coverage to layout-owned tests`
- **Acceptance Criteria**:
  - A checked-in matrix classifies every VexFlow test file and behavior cluster as `obsolete`, `covered`, or `new regression required`
  - The matrix includes at least `src/vexflow/notes.test.ts`, `src/vexflow/articulations.test.ts`, `src/vexflow/renderer.test.ts`, `src/vexflow/render-probe.test.ts`, `src/vexflow/smoke.test.ts`, `src/renderer/vexflowParity.test.ts`, `src/renderer/detailed_diff.test.ts`, VexFlow portions of `src/renderer/corpusGate.test.ts`, and docs-rendering smoke coverage formerly exercised through `build-docs.ts`
  - Behavior clusters explicitly covered: notehead/rest role mapping, articulations/modifiers, hairpins, measure repeats, multi-rests, repeat/final barlines, navigation markers, voltas, tempo/header rendering, secondary voice rests, beaming/triplets, and docs example rendering
  - Any `new regression required` row names the exact test file to add before deleting the old coverage
  - The task is testable by reviewing the matrix against `rg -n "vexflow|VexFlow" src/renderer src/vexflow build-docs.ts`
- **Dependencies**: None

### Task 2: Replace Required VexFlow Coverage With Layout-Owned Regressions
- [x] **Status**: Done
- **Scope**: `src/renderer/`, `src/wasm/`, `src/cli_runtime.test.ts`, `docs/layout-corpus/`, tests named by Task 1
- **Commits**:
  - `test(renderer): add layout-owned regressions for former VexFlow coverage`
- **Acceptance Criteria**:
  - Every Task 1 `new regression required` row has a passing replacement test
  - Replacement tests assert `RenderScore`, `LayoutScene`, SVG adapter semantics, CLI SVG output, or corpus reports; they do not import `src/vexflow` or `vexflow/bravura`
  - `npm test` passes for the replacement test set before any VexFlow implementation files are deleted
  - The task is testable in isolation by temporarily leaving VexFlow source present and verifying the new tests pass without importing it
- **Dependencies**: Task 1

### Task 3: Migrate Settings And App Rendering To Layout-Only
- [x] **Status**: Done
- **Scope**: `src/App.tsx`, `src/hooks/useAppSettings.ts`, `src/hooks/useAppSettings.test.ts`, `src/components/SettingsPanel.tsx`, `src/i18n/keys.ts`, `src/i18n/en.json`, `src/i18n/zh.json`
- **Commits**:
  - `feat(app): remove legacy VexFlow renderer selection`
  - `test(settings): coerce legacy renderer preference to layout route`
- **Acceptance Criteria**:
  - `src/App.tsx` has one preview render branch and never dynamically imports `./vexflow`
  - `resolveAppSettings()` treats saved `useLayoutEngine: false` as layout rendering
  - New persisted settings no longer advertise or write a legacy renderer choice
  - Renderer toggle UI and `Legacy VexFlow` user-facing strings are removed, or replaced with layout-neutral status copy if still useful
  - Settings tests prove null, malformed, legacy-false, and explicit-true settings all render through the layout route
  - This task is testable with settings/unit tests and static search before dependency cleanup
- **Dependencies**: None

### Task 4: Move Docs Rendering To The Node Layout Source API
- [x] **Status**: Done
- **Scope**: `build-docs.ts`, `src/renderer/svgRendererNode.ts` if docs option support is missing, docs build tests if present
- **Commits**:
  - `feat(docs): render examples through Node layout SVG path`
- **Acceptance Criteria**:
  - `build-docs.ts` calls `renderSourceToSvgNode(source, docsRenderOptions)` for score rendering
  - `build-docs.ts` no longer imports `buildNormalizedScore`, `src/vexflow/index`, or initializes VexFlow-specific DOM state
  - Docs render options are explicit and derive from `DEFAULT_RENDER_OPTIONS` or documented docs-specific overrides for scale, page size, margins, header spacing, system spacing, stem length, volta spacing, hairpin offset, secondary-rest visibility, duration spacing compression, and measure-width compression
  - `npm run build-docs` succeeds and generated docs contain layout-rendered SVG for examples
  - This task is testable independently while VexFlow files still exist
- **Dependencies**: None

### Task 5: Convert Corpus Gate To Layout-Only Reports
- [x] **Status**: Done
- **Scope**: `src/renderer/corpusGate.test.ts`, `docs/layout-corpus/corpus_gate_report.json`, `docs/layout-corpus/`, archived corpus evidence
- **Commits**:
  - `test(renderer): replace VexFlow oracle corpus gate with layout semantic report`
- **Acceptance Criteria**:
  - Active corpus report contains `sceneReport` and `svgSemanticReport`, and no `oracleReport`
  - `corpusGate.test.ts` verifies scene report stability, representative scene snapshots, and layout SVG semantic summary stability
  - The test does not import `../vexflow/renderer`, build VexFlow SVG, or read the VexFlow divergence ledger
  - Historical VexFlow divergence evidence is archived or documented as historical only
  - `npm test -- src/renderer/corpusGate.test.ts` passes
- **Dependencies**: Task 2

### Task 6: Add No-VexFlow Boundary Check And Clean Build Metadata
- [x] **Status**: Done
- **Scope**: `scripts/check_import_boundaries*.mjs`, `package.json`, `package-lock.json`, `vite.config.ts`, `tsconfig.app.json`, `dist/`
- **Commits**:
  - `test(boundaries): reject active VexFlow imports`
  - `chore(deps): remove VexFlow package and build aliases`
- **Acceptance Criteria**:
  - Boundary checks fail on static or dynamic imports resolving to `src/vexflow`, `./vexflow`, `../vexflow`, `vexflow`, or `vexflow/bravura`
  - Allowed exclusions are explicit for archived docs and historical evidence
  - `package.json`, `package-lock.json`, `vite.config.ts`, and `tsconfig.app.json` no longer reference VexFlow
  - Stale `dist/assets/vexflow-*.js` chunks are removed by clean rebuild or output cleanup
  - Boundary checks and `npm run build` pass
- **Dependencies**: Tasks 3, 4, 5

### Task 7: Delete Legacy VexFlow Source And Obsolete Tests
- [x] **Status**: Done
- **Scope**: `src/vexflow/**`, VexFlow-only tests classified by Task 1, VexFlow parity utilities, stale imports
- **Commits**:
  - `chore(renderer): delete legacy VexFlow renderer`
- **Acceptance Criteria**:
  - `src/vexflow/**` is deleted
  - Obsolete VexFlow-only tests are deleted only after Task 1 classification and Task 2 replacements
  - Active source/tests/build scripts contain no import of deleted VexFlow files
  - `npm test` passes
  - `npm run drummark -- docs/examples/overview.drum --format svg` succeeds
- **Dependencies**: Tasks 1, 2, 3, 4, 5, 6

### Task 8: Supersede Current Process And Active Proposal References
- [x] **Status**: Done
- **Scope**: `AGENTS.md`, `docs/RENDER_LAYOUT_CONTRACT.md`, `LEARNINGS.md`, active proposal/task files with uncompleted VexFlow-targeted work
- **Commits**:
  - `docs(renderer): mark VexFlow removal as current architecture`
- **Acceptance Criteria**:
  - `AGENTS.md` rendering rules name `RenderScore -> LayoutScene -> thin adapter`, not VexFlow, as the current rendering ownership model
  - `docs/RENDER_LAYOUT_CONTRACT.md` no longer says VexFlow remains available as a lazy legacy renderer
  - `LEARNINGS.md` states VexFlow is removed, not legacy-available
  - Active proposal/task files with uncompleted VexFlow-targeted work receive terminal supersession notes or are archived as appropriate
  - Search gates distinguish active contradictions from immutable archived history
- **Dependencies**: Task 7

### Task 9: Final Verification, Consolidation, And Archival
- [x] **Status**: Done
- **Scope**: proposal/task docs, canonical architecture docs, branch verification, archival
- **Commits**:
  - `docs(renderer): consolidate VexFlow removal architecture`
  - `docs(renderer): archive completed VexFlow removal proposal`
- **Acceptance Criteria**:
  - Proposal file receives appended `### Consolidated Changes`
  - A clean addendum is appended to the active architecture/render contract documentation following the Linear Ledger Protocol
  - `npm test`, `npm run build`, boundary checks, docs build, and representative CLI SVG rendering pass
  - Proposal and tasks files move from `docs/proposals/` to `docs/archived/` after implementation completion
  - Final search confirms no active production, build, test, or current-process VexFlow dependency references remain outside explicitly archived historical material
- **Dependencies**: Tasks 1, 2, 3, 4, 5, 6, 7, 8

## Initial Coverage Migration Matrix

This initial matrix is refined during Task 1 before implementation deletes any tests.

| VexFlow-era surface | Initial classification | Required successor |
| --- | --- | --- |
| `src/vexflow/notes.test.ts` | covered / new regression required | Layout role-mapping tests for notehead and rest glyph roles |
| `src/vexflow/articulations.test.ts` | covered / new regression required | Layout modifier role and SVG adapter semantic-role tests |
| `src/vexflow/renderer.test.ts` | covered / new regression required | Layout structural helper coverage for repeats, barlines, navigation, voltas |
| `src/vexflow/render-probe.test.ts` | mixed | Behavior-cluster rows for hairpins, multi-rests, tempo/header, beaming/triplets, secondary rests, navigation, and docs examples |
| `src/vexflow/smoke.test.ts` | mixed | CLI SVG and corpus semantic report coverage |
| `src/renderer/vexflowParity.test.ts` | obsolete after cutover | Replaced by layout scene/adaptor/corpus gates |
| `src/renderer/detailed_diff.test.ts` | obsolete after cutover | Replaced by layout-only semantic reports or archived as migration tooling |
| VexFlow portion of `src/renderer/corpusGate.test.ts` | obsolete after cutover | `svgSemanticReport` generated from layout SVG only |
| `build-docs.ts` VexFlow smoke path | new regression required | `npm run build-docs` with Node layout renderer |

## Final Coverage Migration Matrix

Task 1 final inventory command:

`rg -n "from \".*vexflow|from './vexflow|from \"\\.\\/vexflow|import\\(\"\\.\\/vexflow|vexflow/bravura|../vexflow|src/vexflow|VexFlow|vexflow" src build-docs.ts scripts vite.config.ts tsconfig.app.json package.json docs/proposals docs/RENDER_LAYOUT_CONTRACT.md AGENTS.md LEARNINGS.md`

| VexFlow-era surface or behavior cluster | Final classification | Successor assertion |
| --- | --- | --- |
| `src/vexflow/notes.test.ts`: duration code, VexFlow stave positions, x-notehead families, hi-hat-local crash sugar | covered | `src/renderer/svgParity.test.ts` covers notehead, X notehead, rests by duration, implicit/same-voice/eighth rests; `src/cli_runtime.test.ts` covers active CLI notehead and ledger-line rendering. VexFlow duration-code assertions are obsolete because VexFlow duration strings are no longer a contract. |
| `src/vexflow/articulations.test.ts`: half-open, roll, flam/drag, dead-stroke helpers | covered | `src/renderer/svgParity.test.ts` covers accent and ghost modifier semantic roles; `docs/layout-corpus/scene-snapshots/modifiers.layout-scene.json` and `src/renderer/corpusGate.test.ts` scene report cover supported modifier scene output. VexFlow-specific grace-note helper path is obsolete as a VexFlow internal. |
| `src/vexflow/renderer.test.ts`: navigation labels, measure-repeat glyph mapping, volta shape helpers | covered | `src/renderer/svgParity.test.ts` covers navigation markers, coda navigation, measure repeat, repeat bars, and volta composites; `src/renderer/svgSceneAdapter.test.ts` covers repeat-span and volta adapter translation. |
| `src/vexflow/render-probe.test.ts`: hairpins, voltas, navigation, multi-rests, triplets/beams, sticking, paragraph breaks, measure repeats, rests, spacing, measure widths | covered | `src/renderer/svgParity.test.ts` covers hairpins, multi-rests, navigation, repeat bars, measure repeats, beams, secondary beams, rest behavior, and voltas; `src/renderer/svgSceneAdapter.test.ts` covers hairpin offset, title area/header spacing, measure-owned beam containment, repeat-span fragments, and generic path/glyph/polyline translation; `src/renderer/corpusGate.test.ts` covers representative scene snapshots for `hairpins`, `multi-rest`, `repeats`, `sticking`, and `full-example`; `src/cli_runtime.test.ts` covers active CLI flags, systems, and ledger lines. VexFlow-specific StaveHairpin spy behavior is obsolete. |
| `src/vexflow/smoke.test.ts`: preview smoke, hairpin bottom skyline, docs examples | covered | `src/renderer/svgParity.test.ts`, `src/renderer/svgSceneAdapter.test.ts`, `src/renderer/corpusGate.test.ts`, and `src/cli_runtime.test.ts` cover the active layout route. Docs example rendering is covered by Task 5's `npm run build-docs` acceptance after docs migrate to `renderSourceToSvgNode`. VexFlow clipping/skylines are obsolete internals. |
| `src/renderer/vexflowParity.test.ts` | obsolete | Pixel/position parity with VexFlow is superseded by layout-owned scene snapshots, SVG semantic assertions, and corpus reports. |
| `src/renderer/detailed_diff.test.ts` | obsolete | The detailed VexFlow-vs-layout diff is migration tooling. Active diff oracle becomes layout-only `svgSemanticReport` in `src/renderer/corpusGate.test.ts`. |
| `src/renderer/position_parity.test.ts` | obsolete | VexFlow-vs-layout element-position parity is superseded by `src/renderer/svgParity.test.ts`, `src/renderer/svgSceneAdapter.test.ts`, and scene snapshots. |
| VexFlow portion of `src/renderer/corpusGate.test.ts` | obsolete | Replaced by layout-only `svgSemanticReport` generated from layout SVG. Existing `sceneReport` and scene snapshots remain active. |
| `build-docs.ts` VexFlow render path | covered after Task 5 | Task 5 changes docs rendering to `renderSourceToSvgNode(source, docsRenderOptions)` and verifies `npm run build-docs`. |
| `scripts/audit_render_network.mjs` legacy VexFlow fetch scenarios | new regression required | Update the audit to remove legacy-render scenarios and assert no VexFlow fetches exist on startup or default layout rendering. |
| `src/App.tsx` legacy renderer branch and `useLayoutEngine` setting | new regression required | Task 4 updates `src/hooks/useAppSettings.test.ts` and app/source contract tests so legacy-false settings resolve to layout-only rendering and no `import("./vexflow")` remains. |
| `vite.config.ts`, `tsconfig.app.json`, `package.json`, `package-lock.json`, stale `dist/assets/vexflow-*.js` | new regression required | Task 9 dependency/build cleanup plus no-VexFlow boundary check verify these references are removed. |

### Review Round 1

1. **Task 6 is ordered so it can fail its own acceptance criteria.**
   Task 6 removes `vexflow` from `package.json` / lockfile / Vite / `tsconfig.app.json` and then requires `npm run build` to pass, but Task 7, which deletes `src/vexflow/**`, is blocked on Task 6. That leaves TypeScript source files such as `src/vexflow/renderer.ts` still present and still importing `vexflow/bravura` after the package/type alias is removed. Unless the current build excludes those files, this creates a hidden coupling where Task 6 cannot be verified until Task 7 has already happened. The dependency direction needs to be corrected: either delete or quarantine the legacy source/tests before dependency removal, or split Task 6 into an independently testable boundary-rule task and a later dependency/build cleanup task that runs after deletion.

2. **The no-VexFlow boundary check lacks a precise scan contract while legacy source still exists.**
   The proposal requires explicit exclusions and a rule that catches active-route static and dynamic imports. The task says the check fails on imports resolving to `src/vexflow`, `./vexflow`, `../vexflow`, `vexflow`, or `vexflow/bravura`, but it does not say whether `src/vexflow/**` is scanned before deletion, whether active VexFlow tests are scanned before deletion, or how the check is expected to pass in Task 6 while those files remain. This violates the Task Independence Rule because the input set is ambiguous and the output is only meaningful after another task changes the repository shape. The task needs an explicit input contract such as "scan production routes/build scripts/tests after Tasks X-Y, excluding the legacy directory until Task 7" or it needs to move after Task 7 and fail on any remaining active reference.

3. **Task 1's output contract is weaker than the proposal's migration-matrix requirement.**
   The acceptance criteria say the matrix classifies every VexFlow test file and behavior cluster, but the checked-in initial matrix still contains unresolved states like `covered / new regression required` and `mixed`. That is acceptable as a seed, not as an approved task output. Task 1 should require a final matrix with no mixed classifications, concrete successor test names for every `covered` row, and explicit `obsolete` rationale for each deleted behavior. It should also require the inventory command to include all active VexFlow-importing tests found by search, not only the minimum list; for example `src/renderer/position_parity.test.ts` currently imports the VexFlow renderer and must not fall through the cracks.

4. **Task 2 is not independently testable from the matrix because it has no named replacement-test contract.**
   The task says every `new regression required` row must have a passing replacement test, but the task plan itself does not define the minimum replacement targets or how each behavior cluster maps to `RenderScore`, `LayoutScene`, adapter semantics, CLI SVG, or corpus reports. That pushes planning into implementation and makes review unable to tell whether coverage is complete before code is changed. The task should require Task 1's final matrix to name exact test files and assertions, and Task 2 should consume only those named rows as its input contract.

5. **Task 8 is too late for process-rule consistency.**
   The approved proposal says the old "all rendering through VexFlow" process rule is superseded before implementation, and AGENTS/process contradictions were a named migration risk. This plan leaves `AGENTS.md`, `docs/RENDER_LAYOUT_CONTRACT.md`, and `LEARNINGS.md` updates until after the renderer is deleted. That means several implementation tasks would run while the active repository instructions still mandate VexFlow rendering. Move the current-process supersession earlier, or split Task 8 so the binding process-rule update happens before implementation tasks while proposal-stream archival/supersession notes can remain near the end.

6. **Final verification omits one migration gate from the approved proposal.**
   The proposal requires full split-WASM verification to pass or be replaced by an equivalent no-VexFlow gate. Task 9 lists tests, build, boundary checks, docs build, and CLI SVG rendering, but does not mention the split-WASM/browser-network audit or its approved replacement. Add that gate explicitly, including the command or test file that proves startup/layout fetch boundaries no longer include VexFlow.

STATUS: CHANGES_REQUESTED

### Author Response

The review is correct. The original task order made dependency cleanup depend on code deletion while code deletion depended on dependency cleanup, and the matrix output contract left too much planning to implementation. The following v1.1 task amendments supersede the initial task order where they conflict.

#### Revised Task Order

The implementation order is now:

1. Task 1: final coverage migration matrix
2. Task 2: early process-rule supersession
3. Task 3: replacement regressions named by the matrix
4. Task 4: settings and app layout-only route
5. Task 5: docs Node layout source API
6. Task 6: corpus layout-only reports
7. Task 7: no-VexFlow boundary rule, initially excluding the legacy directory only
8. Task 8: delete legacy source and obsolete tests
9. Task 9: dependency/build metadata cleanup and clean rebuild
10. Task 10: active proposal/task supersession notes
11. Task 11: final verification, consolidation, and archival

#### Revised Task 1: Final Coverage Migration Matrix

Task 1 output must be a final matrix, not a seed.

Additional acceptance criteria:

- no row may retain `mixed`, `covered / new regression required`, or any other unresolved classification
- every `covered` row names the exact successor test file and assertion surface
- every `new regression required` row names the exact test file to add and the behavior it must assert
- every `obsolete` row includes a short rationale explaining why the behavior was VexFlow-internal or superseded
- the inventory command must include all active VexFlow-importing tests and build surfaces, not only the minimum list
- the inventory must include currently known non-minimum surfaces such as `src/renderer/position_parity.test.ts` if search confirms they import VexFlow

Task 2 may not begin until this final matrix has no unresolved classification.

#### Revised Task 2: Early Process-Rule Supersession

This task moves before implementation edits to app/rendering behavior.

- [ ] **Status**: Pending
- **Scope**: `AGENTS.md`, `docs/RENDER_LAYOUT_CONTRACT.md`, `LEARNINGS.md`
- **Commits**:
  - `docs(renderer): supersede VexFlow rendering rule before removal work`
- **Acceptance Criteria**:
  - `AGENTS.md` no longer instructs agents that all score rendering must be handled exclusively by VexFlow
  - `AGENTS.md` names `RenderScore -> LayoutScene -> thin adapter` as current rendering ownership
  - `docs/RENDER_LAYOUT_CONTRACT.md` appends or updates current-route text stating this proposal supersedes the legacy VexFlow availability rule after stamp
  - `LEARNINGS.md` records that removal implementation is governed by this approved proposal
  - This task does not delete VexFlow code or dependencies; it only removes contradictory current-process instructions
- **Dependencies**: Task 1 and human stamp/consolidation

The old Task 8 is split. Active process-rule updates happen here. Supersession notes for other active proposal/task files remain near the end.

#### Revised Task 3: Replacement Regressions Consume Named Matrix Rows

Task 3 replaces the original Task 2.

Additional acceptance criteria:

- input contract is the final Task 1 matrix
- each `new regression required` row is implemented exactly in the test file named by Task 1, or Task 1 receives an append-only correction before implementation continues
- each `covered` row is verified by running the named successor test, not by assumption
- a short test-run note is appended to the matrix or tasks file when each replacement row is satisfied
- no replacement test imports VexFlow

This makes the task independently testable: leave all VexFlow source in place, run only the named successor tests, and verify they pass without VexFlow imports.

#### Revised Task 7: Boundary Rule Before Deletion

The no-VexFlow boundary rule is now split from package/dependency cleanup.

- [ ] **Status**: Pending
- **Scope**: `scripts/check_import_boundaries*.mjs`, active route scan configuration
- **Commits**:
  - `test(boundaries): reject active VexFlow route imports`
- **Acceptance Criteria**:
  - Boundary rule scans active production routes, build scripts, current process docs, and active non-legacy tests
  - Before legacy deletion, the only temporary exclusion for active source is `src/vexflow/**` plus VexFlow-only tests still awaiting Task 8 deletion
  - The exclusion list is explicit and contains no broad `src/**` or `test/**` wildcard
  - The rule fails on `src/App.tsx` dynamic import of `./vexflow`, `build-docs.ts` imports from `src/vexflow`, package imports from `vexflow` / `vexflow/bravura` outside the excluded legacy surfaces, and active test imports not classified for deletion
  - The rule can pass before dependency removal because it does not scan the still-present legacy implementation directory until Task 8 removes it
- **Dependencies**: Tasks 3, 4, 5, 6

#### Revised Task 8: Delete Legacy Source And Obsolete Tests

Task 8 now runs before dependency/build metadata cleanup.

Additional acceptance criteria:

- `src/vexflow/**` and all matrix-classified obsolete VexFlow-only tests are deleted
- the boundary rule's temporary `src/vexflow/**` and VexFlow-only-test exclusions are removed or narrowed after deletion
- active tests pass before package dependency removal, proving behavioral cleanup is complete while the old dependency is still available
- any remaining VexFlow reference after deletion must be either an explicitly archived historical note or an active failure to fix before Task 9

#### Revised Task 9: Dependency And Build Metadata Cleanup

This is split out from the old Task 6 and depends on Task 8.

- [ ] **Status**: Pending
- **Scope**: `package.json`, `package-lock.json`, `vite.config.ts`, `tsconfig.app.json`, `dist/`, boundary scripts
- **Commits**:
  - `chore(deps): remove VexFlow package and build aliases`
- **Acceptance Criteria**:
  - `package.json`, `package-lock.json`, `vite.config.ts`, and `tsconfig.app.json` no longer reference VexFlow
  - boundary rule has no temporary legacy-source exclusion left for `src/vexflow/**`
  - stale `dist/assets/vexflow-*.js` chunks are removed through clean rebuild or output cleanup
  - `npm run build` passes after dependency removal
  - `npm test` passes after dependency removal
- **Dependencies**: Task 8

#### Revised Task 10: Supersede Active Proposal/Task Streams

This task keeps the late-stage active proposal cleanup from the old Task 8, but no longer includes binding process-rule updates.

- [ ] **Status**: Pending
- **Scope**: active proposal/task files with uncompleted VexFlow-targeted work
- **Commits**:
  - `docs(renderer): supersede active VexFlow proposal tasks`
- **Acceptance Criteria**:
  - active proposals/tasks with uncompleted future VexFlow work receive terminal append-only supersession notes or are archived as appropriate
  - immutable review ledger history is preserved
  - current-process search gates fail on unresolved future VexFlow tasks but allow archived history
- **Dependencies**: Task 9

#### Revised Task 11: Final Verification Gate

Task 11 adds the missing split-WASM/no-VexFlow verification gate.

Additional acceptance criteria:

- `npm run verify:split-wasm` passes, or the task records and runs an approved equivalent no-VexFlow verification command
- browser/network audit proves startup and default layout render do not fetch VexFlow because no VexFlow route exists
- boundary checks pass with no temporary legacy-source exclusions
- final `rg` search confirms no active production, build, test, or current-process VexFlow dependency references remain outside explicitly archived historical material

The final command set must include at least:

- `npm test`
- `npm run build`
- `npm run verify:split-wasm` or approved no-VexFlow equivalent
- `npm run build-docs`
- `npm run drummark -- docs/examples/overview.drum --format svg`

### Review Round 2

The v1.1 task amendments resolve the Round 1 task-planning blockers and now satisfy the Task Independence Rule.

1. **Ordering is corrected.**
   The revised sequence separates boundary-rule work, legacy source deletion, dependency/build metadata cleanup, and final verification. Task 7 can pass while `src/vexflow/**` still exists because it explicitly excludes only the legacy implementation and deletion-classified tests. Task 8 then removes those surfaces before Task 9 removes the package, aliases, and generated chunks. This removes the previous build/dependency deadlock.

2. **Boundary scan scope is now testable before and after deletion.**
   Revised Task 7 has a concrete input set: active production routes, build scripts, current process docs, and active non-legacy tests. Its temporary exclusions are narrow and explicit, and it names representative failures such as `src/App.tsx` dynamic import of `./vexflow`, `build-docs.ts` imports from `src/vexflow`, package imports, and active test imports not classified for deletion. Revised Task 9 also requires the temporary `src/vexflow/**` exclusion to be gone, so the boundary rule has a final no-legacy-source state.

3. **The matrix finality issue is resolved.**
   Revised Task 1 now requires a final matrix with no `mixed`, `covered / new regression required`, or unresolved classifications. It also requires exact successor test files/assertion surfaces, obsolete rationales, and an inventory that includes all active VexFlow-importing tests, including non-minimum examples such as `src/renderer/position_parity.test.ts` if search confirms them.

4. **Replacement tests now have a real input contract.**
   Revised Task 3 consumes the final Task 1 matrix as its input, implements each `new regression required` row in the named file, verifies each `covered` row with the named successor test, and forbids VexFlow imports. That makes the task independently testable while legacy source remains present.

5. **Process-rule timing is fixed.**
   Revised Task 2 moves `AGENTS.md`, `docs/RENDER_LAYOUT_CONTRACT.md`, and `LEARNINGS.md` process-rule supersession ahead of app/rendering implementation work, after the required human stamp/consolidation. Late proposal-stream cleanup remains in Task 10, which is the right separation between binding current-process instructions and archival housekeeping.

6. **Final verification now includes the missing split-WASM/no-VexFlow gate.**
   Revised Task 11 adds `npm run verify:split-wasm` or an approved equivalent, requires a browser/network audit proving startup and default layout render do not fetch VexFlow, requires boundary checks with no temporary legacy-source exclusions, and keeps the full test/build/docs/CLI command set.

I do not see remaining hidden coupling that blocks implementation. The remaining risk is execution discipline: Task 1 must actually produce the final named matrix before Task 3 starts, and Task 7's exclusion list must be removed or narrowed after Task 8 rather than left as a permanent escape hatch. Those are already covered by the amended acceptance criteria.

STATUS: APPROVED

### Implementation Verification Log

Completed implementation tasks through removal and final verification on branch `proposal/remove-vexflow`.

Executed gates:

- `npm test`
- `npm run build`
- `npm run build-docs`
- `npm run verify:split-wasm`
- `npm run drummark -- docs/examples/overview.drum --format svg` via the verify gate

Observed network-audit result:

- startup with preview suspended fetched parser WASM and did not fetch layout WASM or a legacy renderer chunk
- first default layout render fetched layout WASM and did not fetch a legacy renderer chunk
- saved legacy renderer preference was ignored by the app route and did not fetch a legacy renderer chunk

Removal checks:

- `src/vexflow/**` deleted
- VexFlow parity/diff tests deleted
- `package.json`, `package-lock.json`, `vite.config.ts`, and `tsconfig.app.json` no longer reference the removed package
- Vite build output contains no legacy renderer chunk
- active rendering/docs/corpus routes use layout-owned APIs

Task 9 remains pending only for post-review consolidation/archive mechanics after branch integration.

### Final Branch Review

The branch-level post-implementation review approved the removal after the stale active proposal/task references received terminal supersession notes.

Final approved verification:

- `npm test` passed
- `npm run build` passed
- `npm run build-docs` passed
- `npm run verify:split-wasm` passed, including boundary tests, network audit, and CLI SVG smoke
- network audit reported `legacyRendererFetches: 0`, including the saved legacy preference scenario
- static checks found no active `src/vexflow/**`, package dependency, build alias, production import, active test oracle, or VexFlow bundle chunk

STATUS: APPROVED
