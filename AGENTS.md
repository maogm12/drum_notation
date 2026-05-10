# AGENTS.md

## Engineering Integrity

- **Research First:** When encountering technical obstacles or unfamiliar APIs, prioritize reading source code and official documentation to understand implementation details and usage patterns.
- **Avoid "Shotgun" Debugging:** Do not make speculative changes (guess-and-check) followed by requests for user verification.
- **Prototype Verification:** Before applying complex fixes or features, implement small-scale prototypes or reproduction scripts to verify assumptions autonomously.
- **Technical Rigor:** Ensure every change is idiomatically correct and does not introduce regressions or syntax errors (like omission placeholders) into the codebase.
- **Knowledge Retention:** After researching source code or documentation to solve a problem, document the findings (API details, internal logic, discovered constraints) in `LEARNINGS.md`. **All updates to `LEARNINGS.md` MUST follow the Append-Only Protocol** to prevent accidental data loss and maintain a chronological record of technical discoveries.
- **Design First**: For any significant DSL or architectural changes, the agent MUST present a design proposal (documented in the relevant specification file or a dedicated design document) and obtain explicit user approval before writing or modifying any implementation code. **All design proposals and their subsequent reviews MUST follow the Linear Ledger Protocol defined below.**
- **Mandatory Post-Change Review**: After every code modification (feature implementation or bug fix), the agent MUST invoke a sub-agent to review the change. The reviewer must verify technical correctness, check for potential side effects, and ensure compliance with existing patterns.

## Specification & Design Review Protocol

To ensure technical integrity and historical traceability, all formal specifications (e.g., `DRUMMARK_SPEC.md`, `DRUM_IR_SPEC.md`) and design proposals must follow this **Proposal-based Review Protocol**. Proposals are authored, reviewed, and iterated in isolated files; only the final approved result lands in the spec.

### 1. Proposal File

- **Create a standalone proposal file** in `docs/proposals/` for each change, named `<SpecName>_proposal_<topic>.md` (e.g., `DRUMMARK_SPEC_proposal_rehearsal_marks.md`).
- The proposal file contains the full Addendum text as it would appear in the spec.
- **Each proposal gets its own file** — concurrent proposals do not block each other.

### 2. Review Iteration (Linear Ledger within the Proposal)

The proposal file itself follows the **Linear Ledger Protocol** for review notes:

- **Strict Physical Append**: Never modify the original proposal text or any previous review round. All review notes and author responses MUST be appended to the **very end of the file**.
- **Chrono-Log Format**: The file grows downward:
    - `## Addendum vX.Y: [Title]` (the original proposal)
    - `### Review Round 1` (reviewer notes)
    - `### Author Response` (author addresses feedback)
    - `### Review Round 2` → `### Author Response` → ... until approval
- **Prohibition of Anchoring**: Do NOT insert content above an existing header. Every response is a new section at the bottom.

### 3. Mandatory Sub-agent Review

After authoring a proposal, the agent MUST invoke a sub-agent to review it:

- **Constructive Hostility**: The reviewer must act as a critical architect, searching for logic deadlocks, ambiguities, or implementation gaps.
- **No Rubber Stamping**: "Looks good" is an automatic failure. The reviewer must provide specific, actionable critiques or verify complex edge cases.
- **Physical Documentation**: The reviewer MUST append their review notes to the proposal file, following the Linear Ledger Protocol.
- **Review Round ID**: Every review must clearly state its round number (e.g., `### Review Round 1`).
- The reviewer must end with a clear status: `STATUS: CHANGES_REQUESTED` or `STATUS: APPROVED`.

### 4. Execution Planning (Tasks File)

After the proposal achieves sub-agent `STATUS: APPROVED`, the author MUST create a companion **tasks file** in `docs/proposals/`, named `<SpecName>_tasks_<topic>.md`. This file defines the implementation plan as a sequence of tasks, each mapping to one or more commits.

Each task entry MUST include a status checkbox and the following fields:

```markdown
### Task N: [Title]
- [ ] **Status**: Pending
- **Scope**: modules/files affected (e.g., parser grammar, IR types, renderer, editor integration, syntax highlighting, docs, CLI)
- **Commits**: list of planned commits with scope prefix (e.g., `feat(parser): add X rule`)
- **Acceptance Criteria**: verifiable conditions (e.g., `npm run drummark --format ir <input>` produces expected IR)
- **Dependencies**: which prior tasks must complete first (if any)
```

When a task is completed during implementation (Section 6), its status is updated to `[x] **Status**: Done`.

The final task MUST always include consolidation of the proposal into the spec (if not already done in Section 5).

After authoring the tasks file, the agent MUST invoke a sub-agent to review it, following the **Linear Ledger Protocol** (append-only, Review Rounds + Author Responses). The reviewer checks for completeness, correct ordering, and coverage of all proposal requirements.

#### Task Independence Rule

Each task MUST be independently implementable and testable. Avoid the "normalize.rs trap" — where tasks are defined as logical phases but the implementation reveals they are deeply coupled (e.g., `token_to_events` cannot work without `resolve_token`, which cannot work without `fraction`). Rules:

- **Every task must have a clear input/output contract** — what data goes in, what data comes out.
- **Every task must be testable in isolation** — using hand-crafted mock inputs, without requiring downstream or upstream tasks to be complete.
- **Tasks that compute positions from shared data (e.g., note X from slot→X mapping) depend on foundation tasks, but tasks that compute DIFFERENT element types from the SAME shared data are parallel and independent.** Example: note placement depends on `slot→X`, but barline placement and beam placement are independent of each other — both read slot→X results, neither reads the other's output.
- **Algorithms with no external dependencies (e.g., edge element stacking, fixpoint loop) must be their own task**, testable with a hand-crafted list of mock edge elements.
- **Orchestrator tasks (that call modules in sequence) come last**, after all independent modules are built and tested.

The reviewer MUST check for task independence and reject designs where multiple tasks share hidden coupling.

### 5. Final Approval, Consolidation & Stamp

- Implementation may ONLY begin after **both** the proposal and tasks files achieve sub-agent `STATUS: APPROVED`.
- The author presents both approved files to the user and waits for the user's explicit **stamp** (final human sign-off).
- Once the user stamps, the author MUST:
    1. **Append `### Consolidated Changes`** to the proposal file, synthesizing all agreed-upon changes from the proposal and its review rounds into a single, cohesive summary.
    2. **Append a clean Addendum** to the actual spec file (`DRUMMARK_SPEC.md` etc.) following the Linear Ledger Protocol — no review noise, just the final approved content.
- The spec file itself remains append-only: Addendums are added to its end, never inserted above existing content.

### 6. Implementation

After consolidation is complete, implementation proceeds task-by-task:

- For each task, the agent invokes a sub-agent with the spec, the approved proposal, and the tasks file as context.
- Small, related changes may be batched within a single task; the sub-agent may commit multiple times within a task as needed.
- After committing, the agent marks the task complete in the tasks file (e.g., `STATUS: DONE`) and moves to the next task.
- After each code change, the sub-agent MUST verify acceptance criteria and run `npm run drummark` to confirm no regressions.

### 7. Completion & Archival

- After all tasks are complete, verify the proposal's content is present in the spec file. If not, append it.
- **Move the proposal file and tasks file** to `docs/archived/` as a permanent historical record.
- `docs/proposals/` holds active proposals; `docs/archived/` holds completed ones.

## Rendering Rules

- **Total Delegation:** All score rendering (staves, notes, headers, titles, tempo) must be handled exclusively by VexFlow.
- **No Manual Simulation:** Do not add custom HTML, CSS, or Canvas/SVG drawing code to simulate or "patch" missing score elements that should be part of the VexFlow output.
- **Graceful Failure:** If VexFlow cannot render a specific input, fall back only to empty preview states or clear error messages instead of trying to manually draw placeholders.

## Debugging Tools

- **Initial Diagnosis**: When encountering parser, normalization, or rendering bugs, ALWAYS use `npm run drummark` to isolate the problem.
    - Treat the CLI pipeline as `input -> ast -> ir -> xml/svg`.
    - Use `--format ast` to inspect parser / AST shaping problems before normalization.
    - Use `--format ir` to verify if the issue is in the parser/normalization phase.
    - Use `--format svg` or `--format xml` to verify if the issue is in the rendering/export phase.
    - Typical usage:
      - `npm run drummark -- <input-file> --format ast`
      - `npm run drummark -- <input-file> --format ir`
      - `npm run drummark -- <input-file> --format svg`
      - `npm run drummark -- <input-file> --format xml`
- **Verification**: After applying a fix, use the tool to verify the output in the relevant format.

## Content Design (Labels & Copy)

- **Translate variable names to human English.** If a label contains a term that only makes sense by reading the source code (`offsetY`, `compression`, `spacing`), it is implementation leakage. Every label must answer "what does this control do to my score?" in the user's own vocabulary.
- **Avoid redundant section headers.** If a section title already communicates the domain ("Page Layout"), a subgroup label repeating the same concept ("Margins") adds noise without information.
- **Use concrete direction words, not coordinate axes.** `X` and `Y` are implementation detail. Prefer "Horizontal" / "Vertical" or "Left/Right" / "Up/Down" for user-facing labels. (Debug-only labels are exempt.)
- **Avoid orphan prepositions.** A label like "Volta Distance" leaves the user asking "distance from what?". Add the missing object ("Volta Offset") or rephrase entirely.
- **Shorten verb phrases to nouns where context is clear.** "Distance from Title Area to Staff" is verbose; "Title Gap" says the same thing in half the characters. In a 280px sidebar, label length is a layout constraint, not just a style preference.
- **Use the user's musical vocabulary, not the renderer's.** "Lower-voice rests" is implementation terminology (VexFlow voice 1/2). The user understands "secondary voice rests."
- **Group labels should name the domain, not the section.** A debug subsection called "Coordinate Offsets" is cold and redundant (the parent already says "Debug"). Use domain terms the user recognizes, or omit the subgroup label entirely.

## UI Design (Settings Panel)

- **Segmentation beats separators.** Never put `border-top` on every setting row — the resulting "striped" look is visual noise. Proximity alone communicates group membership. Reserve visible dividers for group boundaries, drawn on the group label element.
- **Segmented stepper controls.** Three separated boxes (`[-] [input] [+]`) read as unrelated controls. Fuse them with `gap: 0`, shared borders, and connected border-radius (square middle, rounded ends). This matches iOS/macOS stepper conventions.
- **Directional button cues.** Identical `+` and `-` buttons are scannable but not glanceable. Use directional hover tints: red-tone background for decrement, primary-blue background for increment.
- **Debug visual enclosure.** A debug section in the same visual hierarchy as production controls invites accidental tweaks. Enclose it with: distinct background tint (`--bg-warning-soft`), dashed top separator, left accent bar, and rounded corners.
- **Focus visibility is non-negotiable.** `outline: none` without a replacement indicator breaks keyboard navigation. Use `:focus-visible` with a `box-shadow` ring in the primary accent color and `z-index: 1` to prevent clipping by adjacent segmented-control borders.
- **Long labels need a contract.** On a constrained sidebar, labels must use `flex-shrink: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis`, not `flex-shrink: 0; white-space: nowrap`.

## UI Design (Mobile ≤768px)

- Mobile adaptations are additive overrides in a `@media` block, not a separate stylesheet.
- Only change dimensions (min-height, width, font-size, gap) — never re-declare border, background, or structural properties.
- Touch targets must meet 44×44pt minimum.
- Row height scales from 44px (desktop) to 52px (mobile).
- Apply the same focus-ring, hover-tint, and segmented-control structure across all breakpoints.

## CSS Architecture (Theme Compatibility)

- Directional tints must define a `:root[data-theme="dark"]` variant. Light-mode hardcoded colors (e.g., `#eff6ff`) have no meaning in dark mode — use translucent accent values (`rgba(59, 130, 246, 0.15)`) instead.
- The system-dark `@media (prefers-color-scheme: dark)` block must mirror explicit dark-theme variants using the established `:root:not([data-theme="light"]):not([data-theme="dark"])` selector pattern.

## Internationalization (i18n)

### Architecture
- All user-facing UI strings live in `src/i18n/en.json` (source of truth) and `src/i18n/zh.json` (Chinese translations).
- Keys are typed via `src/i18n/keys.ts` — `I18nKey` is a union of all valid key names, preventing typos at compile time.
- Runtime: `src/i18n/context.tsx` exports `<I18nProvider>` (wraps app root in `main.tsx`) and `useT()` hook. `t(key, params?)` replaces `{{param}}` placeholders.

### Adding a New Key
1. Add the key to `I18N_KEYS` array in `src/i18n/keys.ts`
2. Add the English value to `src/i18n/en.json`
3. Add the Chinese value (or `""` stub) to `src/i18n/zh.json`
4. Use `const { t } = useT()` in the component, then `t("key.name")` in JSX

### Adding a Language
1. Add the locale code to the `Locale` type in `src/i18n/context.tsx`
2. Create `src/i18n/<locale>.json` with all keys from `en.json`
3. Add to the `bundles` map in `context.tsx`
4. Add locale detection logic in `resolveLocale()`

### Pluralization
- No pluralization engine. Use `_one` / `_other` suffix convention (e.g., `status.errors_one`, `status.errors_other`).
- Caller is responsible for selecting the correct key based on count.
- Chinese has no plural morphology — map both `*_one` and `*_other` to the same template.

### Scope
- **Translated**: UI chrome (tabs, buttons, tooltips, aria-labels), status bar, settings labels, preview states, alerts.
- **Not translated**: Debug section labels, music notation terms (D.C., Fine, Coda — standard Italian), CLI output, parser error messages, docs page.
