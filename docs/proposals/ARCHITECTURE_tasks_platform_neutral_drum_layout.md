## Tasks: Platform-Neutral Drum Layout Engine

### Task 1: Define Canonical Render Contracts
- [x] **Status**: Done
- **Scope**: `docs/`, `crates/drummark-layout/`, render-contract type definitions
- **Commits**:
  - `docs(layout): define RenderScore, LayoutScene, and canonical metrics contract`
  - `feat(layout): scaffold contract types for render input, scene output, and metrics tables`
- **Acceptance Criteria**:
  - Repository-owned contract docs exist for `RenderScore`, `LayoutScene`, and canonical metrics ownership
  - `LayoutScene` is defined in a single absolute page-space coordinate system
  - Cross-system span semantics are explicit: `start`, `continuation`, `end`, `single-segment`
  - Semantic composites are first-class for at least `volta`, `hairpin`, `repeat-span`, `measure-repeat`, `multi-rest`, and `text-block`
  - Canonical metrics schema covers all in-scope drum glyph/text roles needed for layout-affecting measurement
  - `cargo test -p drummark-layout` passes with contract-only smoke tests
- **Dependencies**: None

### Task 2: Introduce Explicit `RenderScore` Boundary
- [x] **Status**: Done
- **Scope**: `crates/drummark-core/`, render-facing IR derivation, contract fixtures
- **Commits**:
  - `feat(core): add explicit RenderScore derivation from normalized score`
  - `test(core): add RenderScore fixtures for supported drum corpus slices`
- **Acceptance Criteria**:
  - Layout no longer targets source text or parser-facing AST shapes
  - A dedicated `RenderScore` or equivalent explicit render IR is produced from the normalized musical model
  - `RenderScore` contains deterministic fields for timing slots, voice/staff semantics, attachment anchors, repeat/volta/navigation spans, measure-repeat and multi-rest semantics, and text blocks
  - No open-ended "extra render metadata" escape hatch remains in the boundary
  - Fixture tests can build `RenderScore` from hand-crafted normalized inputs without invoking any renderer
  - `cargo test -p drummark-core` passes
- **Dependencies**: Task 1

### Task 3: Add Canonical Metrics Package for Drum Layout
- [x] **Status**: Done
- **Scope**: `crates/drummark-layout/`, metrics assets/tables, measurement APIs, docs
- **Commits**:
  - `feat(layout): add canonical glyph and text metrics package`
  - `test(layout): verify deterministic measurement for in-scope drum symbols and text roles`
- **Acceptance Criteria**:
  - `drummark-layout` exposes repository-owned measurement APIs for all layout-affecting glyph and text roles in scope
  - Layout-affecting text roles include at least `title`, `subtitle`, `composer`, `tempo`, `sticking`, and count labels
  - No platform adapter is required to measure text or glyphs to influence layout
  - Metrics inputs are versioned in-repo and consumable by Rust without browser APIs
  - Tests prove repeated calls return stable metrics for the same role/style input
  - `cargo test -p drummark-layout` passes
- **Dependencies**: Task 1

### Task 4: Build `LayoutScene` Serialization and Fixture Harness
- [x] **Status**: Done
- **Scope**: `crates/drummark-layout/`, WASM/native exports, fixture harness, scene goldens
- **Commits**:
  - `feat(layout): add scene export APIs for native and wasm consumers`
  - `test(layout): add scene snapshot harness and golden fixtures`
- **Acceptance Criteria**:
  - `drummark-layout` can produce `LayoutScene` from hand-crafted `RenderScore` fixtures without web-only types
  - WASM export and native Rust API both serialize the same logical `LayoutScene` contract
  - Golden tests snapshot `LayoutScene` structure directly rather than pixel output
  - Stable ids, semantic kinds, z-order, and absolute geometry are present in snapshots
  - Fixture harness supports span fragments across system breaks
  - `cargo test -p drummark-layout` passes
- **Dependencies**: Tasks 1, 2, 3

### Task 5: Implement System Breaking and Horizontal Geometry
- [x] **Status**: Done
- **Scope**: `crates/drummark-layout/src/`, system builder, width allocation, slot-to-X mapping
- **Commits**:
  - `feat(layout): implement page, system, measure, and slot geometry`
  - `test(layout): cover system breaking and horizontal spacing with RenderScore fixtures`
- **Acceptance Criteria**:
  - Layout computes page boxes, system breaks, measure widths, and slot-to-X geometry from `RenderScore` plus canonical metrics
  - First-system reservations for percussion clef, time signature, and header anchors are explicit and test-covered
  - Output geometry is emitted in final absolute page-space coordinates
  - Tests cover simple 4/4 spacing, grouped timing, compact structural measures, and at least one forced system break scenario
  - No renderer-side spacing calculation is required for supported cases
  - `cargo test -p drummark-layout` passes
- **Dependencies**: Tasks 2, 3, 4

### Task 6: Implement Drum Event Geometry
- [x] **Status**: Done
- **Scope**: `crates/drummark-layout/src/`, note/rest placement, two-voice handling, stems/beams, sticking, modifiers
- **Commits**:
  - `feat(layout): implement drum event placement and voice geometry`
  - `feat(layout): implement stems, beams, and visible modifier geometry`
  - `test(layout): cover combined hits, two-voice collisions, and modifier attachments`
- **Acceptance Criteria**:
  - Layout emits positioned scene items for drum noteheads, rests, stems, beams, sticking, and visible modifiers in scope
  - Two-voice and combined-hit cases are handled without adapter inference
  - Attachment anchors and stable ids survive into `LayoutScene`
  - Tests cover at least one collision-prone case involving two voices plus sticking or hairpin-adjacent content
  - Scene output remains semantic-first rather than decomposing everything into anonymous lines
  - `cargo test -p drummark-layout` passes
- **Dependencies**: Tasks 2, 3, 4, 5

### Task 7: Implement Structural and Span Composites
- [x] **Status**: Done
- **Scope**: `crates/drummark-layout/src/`, repeats, voltas, navigation, measure-repeat, multi-rest, text blocks, collision stacking
- **Commits**:
  - `feat(layout): implement structural composites and span continuation semantics`
  - `feat(layout): implement text block placement and edge stacking`
  - `test(layout): cover repeat and volta spans across system breaks`
- **Acceptance Criteria**:
  - `LayoutScene` contains first-class composites for repeat spans, volta brackets, hairpins, navigation markers, measure-repeat marks, multi-rests, and text blocks
  - Cross-system span fragments include continuation metadata in the serialized scene
  - Count semantics for measure-repeat and multi-rest are represented separately from paint geometry
  - Title, subtitle, composer, tempo, and count/sticking text placement use canonical metrics, not adapter measurement
  - Collision resolution for above/below-staff structural items is owned by layout and test-covered
  - `cargo test -p drummark-layout` passes
- **Dependencies**: Tasks 2, 3, 4, 5, 6

### Task 8: Build Thin Web Adapter Against `LayoutScene`
- [x] **Status**: Done
- **Scope**: `src/renderer/`, web rendering path, adapter contract tests
- **Commits**:
  - `feat(renderer): implement thin web adapter for LayoutScene`
  - `test(renderer): verify adapter performs translation only`
- **Acceptance Criteria**:
  - The web renderer consumes `LayoutScene` directly and performs only traversal, coordinate conversion, role-to-glyph/path mapping, and paint execution
  - Adapter does not perform measurement, line breaking, collision fixes, semantic span reconstruction, or layout nudging
  - Existing preview/doc output path can render supported scenes through the adapter
  - Adapter tests prove platform translation works from scene fixtures without invoking layout decisions
  - `npm test` passes for renderer-facing coverage
- **Dependencies**: Tasks 1, 3, 4, 5, 6, 7

### Task 9: Migration Oracle, Divergence Ledger, and Corpus Gates
- [x] **Status**: Done
- **Scope**: test harnesses, `docs/`, comparison tooling, corpus fixtures
- **Commits**:
  - `test(layout): add corpus-level scene gates and migration comparisons`
  - `docs(layout): add divergence ledger for intentional VexFlow differences`
- **Acceptance Criteria**:
  - A checked-in supported drum corpus is exercised through the new layout stack
  - Scene-level goldens exist for the supported corpus or approved representative slices
  - VexFlow comparison is used only as a migration oracle, with every intentional divergence recorded in a checked-in ledger
  - Unsupported or VexFlow-specific quirks are documented explicitly rather than silently accepted
  - CI or equivalent local test command can fail on unreviewed scene divergence
  - `cargo test -p drummark-layout` and `npm test` both pass
- **Dependencies**: Tasks 4, 5, 6, 7, 8

### Task 10: Cut Over Rendering and Remove VexFlow
- [ ] **Status**: Pending
- **Scope**: app rendering integration, CLI/doc render path, dependency cleanup, docs
- **Commits**:
  - `feat(renderer): switch product rendering to LayoutScene adapter`
  - `chore(renderer): remove VexFlow runtime dependency`
  - `docs(layout): update renderer ownership after cutover`
- **Acceptance Criteria**:
  - Active product rendering paths no longer depend on VexFlow for supported drum notation
  - Bundle/dependency graph no longer includes VexFlow in active rendering paths
  - Supported corpus renders through the new engine without fallback to VexFlow
  - Verification includes relevant local commands for build/test/render output
  - Ownership docs reflect `RenderScore -> LayoutScene -> adapter` as the live stack
  - `npm test`, `npm run build`, and required Rust tests pass
- **Dependencies**: Task 9

### Task 11: Consolidate, Stamp Results, and Archive Proposal Artifacts
- [ ] **Status**: Pending
- **Scope**: proposal/task docs, architecture/spec docs, archival
- **Commits**:
  - `docs(layout): consolidate approved layout architecture into canonical docs`
  - `docs(layout): archive completed proposal and tasks files`
- **Acceptance Criteria**:
  - Proposal file receives appended `### Consolidated Changes`
  - Canonical architecture/spec documentation is updated append-only with the approved end state
  - Proposal and tasks files move from `docs/proposals/` to `docs/archived/` after implementation completion
  - Final docs clearly state VexFlow replacement scope, platform-neutral layout ownership, and thin-adapter contract
- **Dependencies**: Task 10

### Review Round 1

1. Task 9 is too late in the sequence. The proposal made the migration oracle, divergence ledger, and supported corpus gate a core execution control, but this tasks file places them after the web adapter is already built. That is backwards. Without the corpus definition, scene-golden policy, and divergence ledger earlier, Tasks 5-8 can "pass" against local hand-picked fixtures while drifting away from the supported corpus. The oracle/harness work should begin before or alongside the first layout modules, not after most of the engine is implemented.

2. Task 5 and Task 7 hide a coupling around text/header layout and structural spacing. Task 5 owns first-system reservations for percussion clef, time signature, and header anchors, while Task 7 owns title/subtitle/composer/tempo placement and edge stacking. In practice these are not independent: header text metrics affect first-system reservation and page/system packing. As written, either Task 5 must guess incomplete header geometry, or Task 7 must retroactively change system-breaking behavior. That violates the task-independence rule. The plan needs one task to own the full input/output contract for page/header reservation before system breaking is declared complete.

3. Hairpin ownership is split inconsistently. The approved proposal treats hairpins as first-class semantic composites with system-break semantics, but Task 6 only mentions "hairpin-adjacent content" in a collision case while Task 7 makes hairpins a structural composite. That implies Task 6 cannot truly verify the event-collision cases it claims to cover until Task 7 exists. Either move hairpin-related collision coverage fully into Task 7, or define a narrower Task 6 contract that excludes hairpins entirely.

4. Task 4's acceptance criteria are not concrete enough for schema stability. "Stable ids, semantic kinds, z-order, and absolute geometry are present in snapshots" is necessary but too weak. It does not require snapshot coverage for composite relationships, span-fragment metadata, or canonical metrics references, which are exactly the fields most likely to churn later. A serialization task should prove the scene schema is rich enough for the hard cases before geometry tasks start depending on it.

5. Task 8 does not yet make the "thin adapter" contract falsifiable enough. Saying the adapter "does not perform measurement, line breaking, collision fixes..." is directionally correct, but there is no concrete acceptance hook for proving that. For example:
   - what fixtures prove glyph/text positioning is fully determined by `LayoutScene`?
   - how do tests detect adapter-side span reconstruction or nudging?
   - what renderer API surface is allowed versus forbidden?
   Without a stronger harness, this task can pass by convention rather than evidence.

6. Task 9 weakens the corpus gate with "supported corpus or approved representative slices." That is too soft for a migration-control task. "Representative slices" belongs only where the proposal explicitly allows it; otherwise it becomes an escape hatch to avoid full supported-corpus coverage. The task needs to distinguish:
   - full supported-corpus gating required for cutover,
   - smaller representative slices allowed only for earlier intermediate goldens.
   As written, the final migration gate is ambiguous.

7. Task 10's acceptance criteria are not strong enough to prove VexFlow is actually removed from active rendering ownership. "Bundle/dependency graph no longer includes VexFlow in active rendering paths" leaves room for dead-but-still-installed codepaths, test-only reliance, or fallback hooks. For a cutover task, it should explicitly require no active product/CLI/doc rendering import path to VexFlow and define whether package/dependency removal is mandatory or merely runtime disuse.

8. The plan never isolates canonical metrics evolution from geometry algorithm work. Task 3 defines metrics, but no later task pins which metrics version the golden scenes and corpus gates are asserted against. Since metrics changes can invalidate every scene fixture, you need an explicit rule about metrics versioning/change control in the acceptance surface; otherwise large layout diffs can be explained away as "metrics updates" without disciplined review.

9. Final verification is under-specified. Task 10 says "Verification includes relevant local commands for build/test/render output," but for an architectural migration this needs named commands and a concrete artifact or gate. Otherwise green local checks can still be misleading if they do not exercise the supported corpus through the new stack end-to-end.

The architecture direction is sound, but this tasks file still leaves hidden coupling around header/text geometry, delays the migration oracle too long, and keeps several acceptance criteria at the level of intent instead of proof.

STATUS: CHANGES_REQUESTED

### Author Response

The prior response was appended in the wrong physical location. This tail response is the authoritative ledger continuation for Review Round 4.

The review is correct. The prior amendment described the right parity targets, but it did not map them tightly enough onto the authoritative task list. The following task-level clarifications are binding and supersede any weaker or ambiguous language above.

#### 1. Task Ownership Is Explicit

The parity findings map onto the approved task list as follows:

- Task 3 owns canonical assets and tables:
  - drum-family vertical-position table
  - notehead-family mapping
  - flag glyph/path coverage
  - tempo spacing roles if spacing is metrics-backed
- Task 4 owns fixture and scene-oracle coverage for these cases:
  - system-start component geometry
  - tempo composite structure
  - flagged-note scene structure
  - slanted-beam scene structure
- Task 5 owns start-of-system reservation decomposition:
  - opening barline
  - repeated clef
  - optional time signature
  - first-note entry offset
- Task 6 owns system-breaking use of that reservation contract and right-edge staff closure
- Task 7 owns note/stem/beam/flag geometry plus authoritative drum-family placement in emitted scene items
- Task 8 owns tempo composite placement, text-block interaction, and other structural composites
- Task 10 owns full-corpus parity gating for the newly required fixture cases

No later review should refer to a non-existent standalone "text task" or "header task." The list above is the authoritative mapping.

#### 2. Tempo Has One End-to-End Acceptance Path

Tempo parity is reviewable only through the combined acceptance surface of Tasks 3, 4, and 8:

- Task 3 must provide the canonical beat-unit asset and any spacing roles used by layout
- Task 4 must snapshot tempo composite structure with distinct beat glyph, equals sign, and numeric text geometry
- Task 8 must prove final scene placement uses quarter-note default semantics and resolved spacing with no adapter-side correction

Task 8 does not pass if any of those scene elements are missing, collapsed into one opaque text string, or spaced only by adapter behavior.

#### 3. Flags Have One End-to-End Acceptance Path

Flag parity is reviewable only through the combined acceptance surface of Tasks 3, 4, and 7:

- Task 3 must define dedicated flag glyph roles or canonical flag paths
- Task 4 must snapshot flagged unbeamed notes in a form that distinguishes flag geometry from a generic stroke
- Task 7 must emit flagged-note scene items using those glyph/path-backed assets

Task 7 fails if unbeamed flag output can still be satisfied by an anonymous short line near the stem.

#### 4. The Full-Corpus Gate Supersedes the Earlier Weaker Wording

The earlier Task 9 wording about "supported corpus or approved representative slices" is no longer authoritative for final migration gating.

The binding rule is:

- representative slices are allowed only for intermediate development work under Task 4
- Task 10 final migration comparison and parity gating require the full supported drum corpus

Any remaining weaker wording elsewhere in this file is superseded by this rule and must not be used during implementation review or cutover approval.

### Terminal Supersession: VexFlow Cutover Tasks

This terminal note is appended to preserve the ledger while retiring stale task wording.

Any pending or incomplete task text in this file that says to cut over rendering through VexFlow, compare against VexFlow, remove VexFlow as part of this older platform-neutral layout task list, or use VexFlow parity as a final gate is superseded by `docs/proposals/ARCHITECTURE_proposal_remove_vexflow.md` and `docs/proposals/ARCHITECTURE_tasks_remove_vexflow.md`.

The active acceptance surface for renderer cutover is layout-owned scene output, SVG adapter semantic assertions, corpus reports, import-boundary checks, bundle/network checks, and CLI SVG generation. VexFlow-specific pending rows in this historical task file are no longer active implementation work.

### Review Round 5

The new tail response closes the outstanding Round 4 task-planning objections.

1. Task ownership is now mapped onto actual task numbers, and the phantom task-name problem is resolved. The response explicitly assigns the parity work across Tasks 3, 4, 5, 6, 7, 8, and 10 and forbids future review language that refers to non-existent standalone tasks.

2. Tempo now has a reviewable end-to-end acceptance path. The response assigns asset/spacing-role ownership to Task 3, scene-structure proof to Task 4, and final placement semantics to Task 8, which is sufficient to prevent the requirement from dissolving across unrelated tasks.

3. Flags now have a reviewable end-to-end acceptance path. The response assigns canonical asset definition to Task 3, scene-structure snapshots to Task 4, and emitted flagged-note geometry to Task 7, and it explicitly states that Task 7 fails if fallback short-line output is still possible.

4. The full-corpus gate now clearly supersedes the weaker earlier wording. The response confines representative slices to intermediate Task 4 work and assigns final full-corpus migration gating to Task 10, which closes the prior conflict for implementation review purposes.

STATUS: APPROVED

### Author Response

The review is correct. The prior amendment described the right parity targets, but it did not map them tightly enough onto the authoritative task list. The following task-level clarifications are binding and supersede any weaker or ambiguous language above.

#### 1. Task Ownership Is Explicit

The parity findings map onto the approved task list as follows:

- Task 3 owns canonical assets and tables:
  - drum-family vertical-position table
  - notehead-family mapping
  - flag glyph/path coverage
  - tempo spacing roles if spacing is metrics-backed
- Task 4 owns fixture and scene-oracle coverage for these cases:
  - system-start component geometry
  - tempo composite structure
  - flagged-note scene structure
  - slanted-beam scene structure
- Task 5 owns start-of-system reservation decomposition:
  - opening barline
  - repeated clef
  - optional time signature
  - first-note entry offset
- Task 6 owns system-breaking use of that reservation contract and right-edge staff closure
- Task 7 owns note/stem/beam/flag geometry plus authoritative drum-family placement in emitted scene items
- Task 8 owns tempo composite placement, text-block interaction, and other structural composites
- Task 10 owns full-corpus parity gating for the newly required fixture cases

No later review should refer to a non-existent standalone "text task" or "header task." The list above is the authoritative mapping.

#### 2. Tempo Has One End-to-End Acceptance Path

Tempo parity is reviewable only through the combined acceptance surface of Tasks 3, 4, and 8:

- Task 3 must provide the canonical beat-unit asset and any spacing roles used by layout
- Task 4 must snapshot tempo composite structure with distinct beat glyph, equals sign, and numeric text geometry
- Task 8 must prove final scene placement uses quarter-note default semantics and resolved spacing with no adapter-side correction

Task 8 does not pass if any of those scene elements are missing, collapsed into one opaque text string, or spaced only by adapter behavior.

#### 3. Flags Have One End-to-End Acceptance Path

Flag parity is reviewable only through the combined acceptance surface of Tasks 3, 4, and 7:

- Task 3 must define dedicated flag glyph roles or canonical flag paths
- Task 4 must snapshot flagged unbeamed notes in a form that distinguishes flag geometry from a generic stroke
- Task 7 must emit flagged-note scene items using those glyph/path-backed assets

Task 7 fails if unbeamed flag output can still be satisfied by an anonymous short line near the stem.

#### 4. The Full-Corpus Gate Supersedes the Earlier Weaker Wording

The earlier Task 9 wording about "supported corpus or approved representative slices" is no longer authoritative for final migration gating.

The binding rule is:

- representative slices are allowed only for intermediate development work under Task 4
- Task 10 final migration comparison and parity gating require the full supported drum corpus

Any remaining weaker wording elsewhere in this file is superseded by this rule and must not be used during implementation review or cutover approval.

### Author Response

The review is correct. The task list above had the right major work items, but it still left three dangerous ambiguities:

- migration control came too late
- header/text reservation crossed task boundaries
- some acceptance criteria were still "intent-shaped" instead of falsifiable

The following amendments are binding and supersede any ambiguous sequencing or acceptance language in the original task list above.

#### 1. Revised Execution Order

The authoritative execution order is now:

1. Task 1: Canonical render contracts
2. Task 2: Explicit `RenderScore` boundary
3. Task 3: Canonical metrics package
4. Task 4: Scene schema plus fixture/oracle harness
5. Task 5: Header/page reservation model
6. Task 6: System breaking and horizontal geometry
7. Task 7: Drum event geometry
8. Task 8: Structural and span composites
9. Task 9: Thin web adapter
10. Task 10: Corpus gate, divergence ledger, and migration comparisons
11. Task 11: Cut over rendering and remove VexFlow
12. Task 12: Consolidate and archive

This means the original "Task 9" migration-control work is no longer late-stage-only. The fixture corpus, scene-golden machinery, and divergence-ledger structure begin with Task 4 and become a hard gate before geometry work expands.

#### 2. Task 4 Is Upgraded Into the Oracle Foundation

Task 4 is no longer just serialization smoke coverage. It now owns the foundational migration harness:

- checked-in `LayoutScene` schema snapshots
- checked-in representative fixture corpus for early module development
- checked-in divergence ledger scaffold
- test helpers that assert:
  - stable ids
  - semantic composite structure
  - span-fragment metadata
  - canonical metrics references or style-role bindings where applicable

Intermediate work may use representative fixture slices. Final cutover may not.

#### 3. Header/Text Reservation Is Separated From General System Breaking

The review correctly identified hidden coupling between first-system reservation and text placement.

Therefore:

- a dedicated task now owns page/header reservation semantics before general system breaking is considered complete
- that task defines the reservation contract for:
  - title
  - subtitle
  - composer
  - tempo
  - first-system clef/time-signature start zone
- general system-breaking work depends on that reservation contract and may not guess missing header geometry

This removes retroactive repacking risk.

#### 4. Hairpin Ownership Is Moved Fully Into Structural/Span Work

Hairpins are not part of the drum-event geometry task.

Task 7 covers only:

- note/rest placement
- combined hits
- stems/beams
- sticking
- visible note-local modifiers

Hairpins move entirely into the structural/span composites task, where continuation, system-break segmentation, and collision ownership can be tested coherently.

Any Task 7 collision fixture may include reserved neighboring space for future hairpins, but may not claim to validate hairpin semantics.

#### 5. Thin Adapter Acceptance Must Be Proved, Not Asserted

The web-adapter task is amended to require explicit proof hooks:

- adapter tests must render from precomputed `LayoutScene` fixtures with no call path back into layout or measurement APIs
- fixture comparisons must prove the adapter preserves:
  - absolute coordinates
  - scene ordering
  - composite membership
  - span fragment boundaries
- code review acceptance for this task fails if the adapter adds any API that requests:
  - text measurement for layout
  - glyph measurement for layout
  - semantic reconstruction from decomposed primitives
  - coordinate nudging beyond device-space rounding

Allowed adapter behavior is limited to:

- traversal
- unit conversion
- glyph/path lookup
- paint emission
- optional accessibility/event tagging

#### 6. Final Corpus Gate Must Cover the Full Supported Corpus

The phrase "or approved representative slices" is too weak for final migration acceptance.

The corrected rule is:

- representative slices are allowed only for early fixture development and isolated module tests
- final migration gating and cutover require the full supported drum corpus

No final cutover task may pass on a subset gate.

#### 7. VexFlow Removal Must Be Structural, Not Merely Behavioral

The cutover task is amended to require all of the following:

- no active product preview path imports VexFlow
- no active CLI/doc rendering path imports VexFlow
- no runtime fallback hook to VexFlow remains for supported drum notation
- VexFlow is removed from runtime dependencies unless a documented non-rendering residual use is explicitly approved elsewhere

If VexFlow remains anywhere after cutover, the remaining ownership must be named and justified in docs.

#### 8. Metrics Versioning Is Now a First-Class Acceptance Surface

Canonical metrics are not a silent implementation detail.

The task sequence now assumes:

- scene goldens are asserted against a named canonical metrics version or checked-in metrics snapshot
- any metrics-table change that materially changes scene output must update:
  - affected scene goldens
  - divergence records if migration comparisons move
  - review rationale for why the metrics change is intended

"The metrics changed" is not by itself an acceptable explanation for unreviewed layout drift.

#### 9. Named Verification Commands Are Required at Cutover

Final cutover verification must name the exact local commands used to prove the new stack:

- Rust layout tests
- JS/web renderer tests
- application build
- at least one corpus-driving render command through the active renderer path

The final task file need not guess the exact command strings now, but implementation review may not accept vague "relevant commands" language. The cutover task must close with explicit commands and green results.

#### 10. Revised Task Mapping

To remove ambiguity, the intended task ownership is now:

- Task 1: contract definitions
- Task 2: `RenderScore`
- Task 3: canonical metrics
- Task 4: `LayoutScene` schema + fixture/oracle harness + divergence-ledger scaffold
- Task 5: header/page reservation model
- Task 6: system breaking and horizontal geometry
- Task 7: drum event geometry
- Task 8: structural/span composites, text blocks, collision stacking
- Task 9: thin web adapter
- Task 10: full supported-corpus gate + VexFlow migration comparisons
- Task 11: product cutover + VexFlow removal
- Task 12: consolidation and archival

This mapping is the authoritative plan for implementation sequencing and later task-status updates.

### Review Round 2

The appended `### Author Response` resolves the task-planning blockers well enough for implementation sequencing.

1. The revised execution order fixes the biggest planning flaw. Moving the oracle foundation up to Task 4 and pushing the full supported-corpus gate ahead of cutover means geometry and adapter work are no longer validated only against ad hoc local fixtures. That makes the migration control real instead of retrospective.

2. Task independence is materially improved. Splitting header/page reservation into its own task before general system breaking removes the earlier hidden coupling between text metrics and horizontal packing. That gives Task 5 and Task 6 clean enough input/output boundaries for implementation and review.

3. Hairpin ownership is now coherent. Keeping hairpins entirely in the structural/span task avoids the prior cross-task semantic leak where event-geometry tests would have depended on span behavior that did not exist yet.

4. The schema/oracle task is now concrete enough. Upgrading Task 4 to own snapshots, representative development fixtures, divergence-ledger scaffold, and assertions for composite/span metadata gives later tasks a stable contract surface to build against.

5. The thin-adapter gate is now enforceable rather than aspirational. The response adds direct proof hooks: precomputed-scene fixtures, no path back into layout/measurement APIs, and preservation checks for coordinates, ordering, composite membership, and span boundaries. That is sufficient to make "thin" reviewable.

6. The corpus gate is now explicit about scope. Representative slices are limited to early development, while final migration and cutover require the full supported corpus. That closes the earlier escape hatch.

7. VexFlow removal is now structural. The response requires no active preview/CLI/doc imports, no runtime fallback, and dependency removal unless a separately approved residual use exists. That is strong enough for a cutover task.

8. Metrics drift is now governed. Requiring goldens and divergence records to move in lockstep with intended metrics changes is enough to keep "the metrics changed" from becoming a blanket excuse for layout churn.

9. Final verification is now concrete enough for implementation review. Requiring named Rust tests, renderer tests, build verification, and a corpus-driving render command closes the earlier "relevant commands" ambiguity.

No remaining ordering or task-boundary issue blocks approval. The remaining risk is implementation complexity in the layout engine itself, not ambiguity in the task plan.

STATUS: APPROVED

### Author Response

Current parity findings exposed six acceptance gaps that need to be made explicit in the plan. The task list remains valid, but the following amendments are binding and supersede any weaker acceptance language above.

#### 1. Header/System-Boundary Geometry Must Be Tested As Real Engraving Cases

The reservation work is not complete unless fixtures prove all of the following:

- the first measure's left barline sits on the staff's left boundary
- the percussion clef remains inside that first measure, to the right of the opening barline
- the first system may include a time signature after the clef
- later systems do not retain blank time-signature reservation when no time signature is rendered
- the final barline of a system closes exactly on the staff's right boundary

This acceptance surface belongs primarily to the header/page reservation and system-breaking tasks, not the adapter task.

#### 2. Tempo Construction Needs an Explicit Fixture Gate

The text/block task is not complete unless tempo fixtures prove:

- default tempo uses a quarter-note beat unit
- tempo beat glyph, equals sign, and numeric value have explicit reserved spacing
- tempo geometry is already resolved in scene output without adapter-side spacing fixes

#### 3. Drum Event Geometry Must Pin Stem Anchors and Staff Positions

The drum-event task must explicitly verify:

- up-stem and down-stem anchors are offset to the notehead's right side using canonical metrics
- supported drum families map to the correct resolved staff positions
- crash/cymbal fixtures cover the top-ledger-line case as a non-optional oracle

It is not sufficient to prove only that "snare looks right" or that stems exist in roughly correct places.

#### 4. Flag Rendering Requires Canonical Glyph Coverage

The canonical-metrics and drum-event tasks together must guarantee:

- dedicated flag glyph roles or canonical flag paths exist for stem-up and stem-down tails
- unbeamed flagged notes are rendered from those canonical assets rather than fallback strokes

Any implementation that draws a generic short line in place of a flag fails acceptance.

#### 5. Slanted Beam Geometry Must Be Verified at the Scene Level

The drum-event task is not complete unless beam fixtures prove:

- slanted beams are emitted as path/polygon bodies with real thickness
- beam endcaps remain vertically cut
- participating stems are reprojected to the resolved beam boundary
- no stem overshoots through the beam body

This must be asserted from `LayoutScene` geometry directly, not by trusting approximate SVG screenshots.

#### 6. Corpus and Migration Gates Must Include These Cases Explicitly

The oracle/corpus tasks may not rely on generic "common examples" alone. The checked-in fixture corpus must contain explicit parity cases for:

- first-system opening barline plus clef/time-signature packing
- later-system start without phantom time-signature spacing
- right-edge final barline closure
- default tempo glyph and spacing
- crash/top-ledger-line placement
- flagged unbeamed notes
- slanted beam stem trimming

### Review Round 3

1. The newly appended guidance is not correctly mapped onto the authoritative task list. It assigns ownership to a "header/page reservation" task and a "text/block task", but this file currently has neither as standalone tasks. Under the actual plan, the relevant work is split across Task 5, Task 6, and Task 7. That mismatch makes later status updates and acceptance review ambiguous.

2. Tempo parity now has hidden coupling with no single end-to-end owner. The new text spreads the requirement across metrics, text/block placement, and system/header reservation, but the task plan does not identify which task is responsible for proving the full tempo composite contract: default quarter-note beat selection, spacing around `=`, and final scene geometry with no adapter fixes. As written, multiple tasks can pass independently while tempo remains wrong in product output.

3. Flag rendering has the same coupling problem. The new text says the canonical-metrics and drum-event tasks "together must guarantee" flag glyph coverage, but Task 6's acceptance criteria still do not explicitly fail an implementation that falls back to anonymous strokes for unbeamed flags. The task plan needs one reviewable acceptance surface that requires glyph/path-backed flag output in `LayoutScene`, not just upstream asset availability.

4. The appended corpus note does not resolve an existing contradiction in the task plan. The new prose requires explicit parity fixtures, but Task 9 still says scene goldens exist for "the supported corpus or approved representative slices." That remains incompatible with the earlier approved rule that final migration gating must use the full supported corpus. Because the authoritative task body still contains the weaker escape hatch, the mapping is not yet internally consistent.

STATUS: CHANGES_REQUESTED

### Review Round 4

No new `### Author Response` was appended after Review Round 3 in this file, so the prior task-mapping objections remain open.

1. The file still uses phantom task names in the appended guidance. "header/page reservation task" and "text/block task" do not exist in the authoritative task list at the top of this file, so ownership is still not mapped onto actual task numbers.

2. Tempo still lacks one end-to-end owner in the task plan. The current text continues to split tempo correctness across reservation, metrics, and text-placement language without naming a single task that must prove default beat-unit selection, `=` spacing, and final scene geometry together.

3. Flags still lack a single reviewable acceptance surface. The file still says canonical-metrics and drum-event tasks "together" guarantee flag behavior, but Task 6 itself still does not explicitly reject fallback stroke-based flag output in `LayoutScene`.

4. The weaker corpus wording is still present in the authoritative task body. Task 9 still says scene goldens exist for "the supported corpus or approved representative slices," so the full-corpus gate has not actually superseded the weaker wording in the plan itself.

STATUS: CHANGES_REQUESTED
### Author Response

The prior response was appended in the wrong physical location. This tail response is the authoritative ledger continuation for Review Round 4.

The review is correct. The prior amendment described the right parity targets, but it did not map them tightly enough onto the authoritative task list. The following task-level clarifications are binding and supersede any weaker or ambiguous language above.

#### 1. Task Ownership Is Explicit

The parity findings map onto the approved task list as follows:

- Task 3 owns canonical assets and tables:
  - drum-family vertical-position table
  - notehead-family mapping
  - flag glyph/path coverage
  - tempo spacing roles if spacing is metrics-backed
- Task 4 owns fixture and scene-oracle coverage for these cases:
  - system-start component geometry
  - tempo composite structure
  - flagged-note scene structure
  - slanted-beam scene structure
- Task 5 owns start-of-system reservation decomposition:
  - opening barline
  - repeated clef
  - optional time signature
  - first-note entry offset
- Task 6 owns system-breaking use of that reservation contract and right-edge staff closure
- Task 7 owns note/stem/beam/flag geometry plus authoritative drum-family placement in emitted scene items
- Task 8 owns tempo composite placement, text-block interaction, and other structural composites
- Task 10 owns full-corpus parity gating for the newly required fixture cases

No later review should refer to a non-existent standalone "text task" or "header task." The list above is the authoritative mapping.

#### 2. Tempo Has One End-to-End Acceptance Path

Tempo parity is reviewable only through the combined acceptance surface of Tasks 3, 4, and 8:

- Task 3 must provide the canonical beat-unit asset and any spacing roles used by layout
- Task 4 must snapshot tempo composite structure with distinct beat glyph, equals sign, and numeric text geometry
- Task 8 must prove final scene placement uses quarter-note default semantics and resolved spacing with no adapter-side correction

Task 8 does not pass if any of those scene elements are missing, collapsed into one opaque text string, or spaced only by adapter behavior.

#### 3. Flags Have One End-to-End Acceptance Path

Flag parity is reviewable only through the combined acceptance surface of Tasks 3, 4, and 7:

- Task 3 must define dedicated flag glyph roles or canonical flag paths
- Task 4 must snapshot flagged unbeamed notes in a form that distinguishes flag geometry from a generic stroke
- Task 7 must emit flagged-note scene items using those glyph/path-backed assets

Task 7 fails if unbeamed flag output can still be satisfied by an anonymous short line near the stem.

#### 4. The Full-Corpus Gate Supersedes the Earlier Weaker Wording

The earlier Task 9 wording about "supported corpus or approved representative slices" is no longer authoritative for final migration gating.

The binding rule is:

- representative slices are allowed only for intermediate development work under Task 4
- Task 10 final migration comparison and parity gating require the full supported drum corpus

Any remaining weaker wording elsewhere in this file is superseded by this rule and must not be used during implementation review or cutover approval.
