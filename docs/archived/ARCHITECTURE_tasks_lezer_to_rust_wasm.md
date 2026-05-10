## Tasks: Migrate Lezer Parser to Rust + WASM

### Task 1: Scaffold Rust Crate
- [ ] **Status**: Pending
- **Scope**: New `crates/drummark-core/` directory, Cargo.toml, CI workflow
- **Commits**:
  - `chore(wasm): scaffold drummark-core Rust crate with Logos + wasm-bindgen deps`
  - `ci(wasm): add wasm-pack build step to CI`
- **Acceptance Criteria**:
  - `cargo build` succeeds in `crates/drummark-core/`
  - `wasm-pack build --target web` produces `pkg/` with `.wasm` and JS glue
  - Rust toolchain added to CI (or documented as dev-only)
- **Dependencies**: None

### Task 2: Implement Logos Tokenizer
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-core/src/lexer.rs`
- **Commits**:
  - `feat(lexer): implement Logos tokenizer covering all DrumMark tokens`
  - `test(lexer): add tokenization parity tests against existing fixtures`
- **Acceptance Criteria**:
  - All tokens from `drum_mark.grammar` have corresponding Logos variants
  - MultiRest regex correctly rejects `--1--`, accepts `--2--` and `--11--`
  - MeasureRepeat uses `#[regex(r"%+")]`, count from `.len()`
  - Comma declared before HeaderWord; HeaderWord regex excludes `,` and `#`
  - Glyph tokens: `BD2` > `BD` > `B` by longest-match, not declaration order
  - RoutedTrackPrefix (`@HH` etc.) and SummonPrefix (`HH:` etc.) tokens present
  - Navigation tokens (`@segno`, `@dc-al-fine`, etc.), hairpin tokens (`<`, `>`, `!`)
  - Modifier keyword tokens (`accent`, `open`, `half-open`, etc.)
  - Volta barline tokens (`|:.`, `||.`) present
  - Tokenizer test run: `cargo test -- lexer` passes on fixture files
- **Dependencies**: Task 1

### Task 3: Implement AST Types + WASM Bridge
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-core/src/ast.rs`, `crates/drummark-core/src/to_js.rs`, `crates/drummark-core/src/lib.rs`
- **Commits**:
  - `feat(ast): define Rust AST types mirroring DocumentSkeleton`
  - `feat(wasm): implement to_js() conversion for all AST types`
  - `feat(wasm): export parse() function via wasm-bindgen`
- **Acceptance Criteria**:
  - AST structs defined: `Document`, `HeaderSection`, `Paragraph`, `TrackLine`, `MeasureSection`, `MeasureExpr` (enum), `Barline`, etc.
  - All types support `to_js() -> JsValue` via `js_sys::Object`/`js_sys::Array`
  - `#[wasm_bindgen]` on exported `parse(source: &str) -> JsValue` function
  - `ParseError` struct with `line`, `column`, `message` fields
  - `drummark_core.d.ts` created manually covering all exported types (~60 lines)
  - `wasm-pack build` produces working `.d.ts` + `.wasm`
- **Dependencies**: Task 1, Task 2

### Task 4: Implement Recursive Descent Parser
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-core/src/parser.rs`, `crates/drummark-core/src/error.rs`
- **Commits**:
  - `feat(parser): implement parser infrastructure (peek/next/trivia/buffer)`
  - `feat(parser): implement header parsing (title, tempo, time, grouping, note, divisions)`
  - `feat(parser): implement track body parsing (TrackLine, MeasureSection, Barline)`
  - `feat(parser): implement measure expression parsing (note, rest, group, combined hit, summons, routing)`
  - `feat(parser): implement suffix chain, modifiers, hairpins, navigation, repeats`
  - `feat(parser): implement inline braced block with recursive brace balancing`
  - `feat(parser): implement volta barline composite parsing`
- **Acceptance Criteria**:
  - Three peek methods: `peek()` (trivia-skip), `peek_raw()` (no skip), `peek_n(n)` (buffered)
  - `skip_trivia()` consumes Space and Comment, NOT Newline
  - `parse_document()` produces `Document` AST from source string
  - VoltaBarline parsed as composite: `barline_prefix + Integer (Comma Integer)* Dot`
  - InlineBracedBlock handles arbitrary nesting depth via recursive descent
  - TrackBodyTail disambiguated via `peek_n(2)` lookahead past `Newline+`
  - SuffixChain: dots, stars, halves, modifiers loop
  - All 19 track names, 19 glyphs, 13 modifiers recognized
  - MultiRest count extracted from regex; MeasureRepeat count from `%+` length
  - Navigation markers/jumps, hairpins, inline repeat suffix parsed
  - `cargo test` passes on all parser unit tests
- **Dependencies**: Task 3

### Task 5: Parity Testing — Token & Structure Level
- [ ] **Status**: Pending
- **Scope**: New test fixtures, parity test runner
- **Commits**:
  - `test(wasm): add token-level parity test runner comparing Rust vs JS parsers`
  - `test(wasm): add structure-level parity test for document skeleton`
  - `test(wasm): debug and fix any parity discrepancies found`
- **Acceptance Criteria**:
  - All existing test fixtures (`src/dsl/*.test.ts`) produce identical `DocumentSkeleton` from both parsers
  - Specific parity test cases: multi-rest `--1--` rejection, volta `|1,2.` and `|:1.`, nested braces `{ x { d } b }`, TrackBodyTail `note` override, combined hits, hairpin span, navigation markers
  - Position reporting (line/column) matches existing error diagnostic tests
  - Parity tests run: `cargo test -- parity` passes
- **Dependencies**: Task 4

### Task 6: JS Wrapper & Pipeline Integration
- [ ] **Status**: Pending
- **Scope**: `src/wasm/drummark_wasm.ts`, `src/wasm/skeleton.ts`, `src/dsl/ast.ts` modifications
- **Commits**:
  - `feat(wasm): create JS wrapper for WASM parser module`
  - `feat(wasm): implement skeleton.ts adapting WASM output to DocumentSkeleton`
  - `feat(pipeline): wire WASM parser as third parse path in ast.ts`
  - `chore(vite): configure WASM loading in vite.config.ts`
- **Acceptance Criteria**:
  - `skeleton.ts` converts WASM JsValue to `DocumentSkeleton` (~150 lines)
  - `ast.ts` supports `parseMode: "wasm"` alongside existing "lezer" and "regex"
  - Vite loads WASM module correctly in dev and production builds
  - `npm run drummark` with WASM parser produces identical AST/IR/SVG/XML output
  - Fallback to existing parser if WASM fails to load
- **Dependencies**: Task 5

### Task 7: Native CLI Binary
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-core/src/main.rs`, `Cargo.toml` bin target
- **Commits**:
  - `feat(cli): add native CLI binary reading stdin and formatting output`
  - `feat(cli): support --format ast|ir|json for native CLI`
  - `chore(scripts): update npm run drummark to use native binary`
- **Acceptance Criteria**:
  - `cargo run -- <input>` produces same output as `tsx src/cli.ts <input>`
  - `--format ir`, `--format json` modes work from native binary
  - Startup time significantly faster than `tsx` (measurable)
  - CI builds native binary for macOS and Linux
- **Dependencies**: Task 6

### Task 8: Cleanup — Remove Lezer & Regex Parser
- [ ] **Status**: Pending
- **Scope**: Multiple files deleted, `package.json` updated, `AGENTS.md` updated
- **Commits**:
  - `refactor: remove lezer_skeleton.ts and parser.ts`
  - `refactor: remove drum_mark.grammar and generated parser files`
  - `chore(deps): remove @lezer/* packages from package.json`
  - `chore(docs): update AGENTS.md with new build/parse commands`
- **Acceptance Criteria**:
  - `lezer_skeleton.ts` (1406 lines) deleted
  - `parser.ts` (1473 lines) deleted
  - `drum_mark.grammar`, `drum_mark.parser.js`, `drum_mark.parser.terms.js` deleted
  - `lezer_parity.test.ts`, `lezer_drift.test.ts` deleted or archived
  - `@lezer/lr`, `@lezer/generator`, `@lezer/markdown` removed from `package.json`
  - `npm ls @lezer` returns empty
  - `npm run drummark` still works (now via native binary or WASM)
  - `npm run test` passes (existing tests now use WASM parser as default)
  - AGENTS.md updated with new parser commands and Rust toolchain notes
- **Dependencies**: Task 7

### Task 9: Consolidate Proposal into Architecture Spec
- [ ] **Status**: Pending
- **Scope**: `docs/DRUMMARK_SPEC.md` or new architecture doc
- **Commits**:
  - `docs: append consolidated Rust WASM parser design to architecture docs`
- **Acceptance Criteria**:
  - Final approved design (original proposal + all review fixes) consolidated into spec
  - No review noise — clean synthesis of all decisions
  - Proposal + tasks files moved to `docs/archived/`
- **Dependencies**: Task 8


### Review Round 1

I have reviewed the tasks file against the approved proposal (all Addendum content, Review Round 1 fixes, Review Round 2 fixes, and both Author Responses). Below is a structured critique following the Constructive Hostility mandate.

---

#### 1. CRITICAL: No CI integration of Rust toolchain, `wasm-pack build`, or `cargo test`

Task 1 AC #3 reads *"Rust toolchain added to CI (or documented as dev-only)."* The "or documented as dev-only" escape hatch undermines the task. If the Rust crate is the sole parser (after Task 8), CI must verify it builds and tests pass — otherwise a Rust source change can silently break the entire pipeline (browser + CLI). The existing CI workflow (`deploy-pages.yml`) runs `npm run build` and `npm test` on every push/PR. After Task 6, `npm run build` must produce the WASM bundle; after Task 7, `npm run drummark` must invoke the native binary. Neither will work without the Rust toolchain.

**Fix required**: Remove the "or documented as dev-only" branch. Task 1 must commit to installing the Rust toolchain and `wasm-pack` in CI, running `cargo test` and `wasm-pack build` alongside `npm test`. Add a CI commit to Task 1 (or a dedicated CI verification commit in Task 3/4) that fails the build if the WASM module is stale.

---

#### 2. CRITICAL: Parse mode lifecycle has no end state — contradiction between Task 6 and Task 8

Task 6 AC #3 says `ast.ts` supports `parseMode: "wasm"` alongside existing `"lezer"` and `"regex"`. Task 8 removes both `lezer_skeleton.ts` and `parser.ts` entirely. The tasks never specify:

- What `ast.ts` imports after Task 8 (it currently hard-imports `parseDocumentSkeletonFromLezer` at line 1).
- Whether the `parseMode` parameter is removed or hardcoded to `"wasm"` after Task 8.
- What happens to the 35+ `src/dsl/*.test.ts` files that import `parseDocumentSkeleton` (regex parser) and `parseDocumentSkeletonFromLezer` (Lezer parser).

After Task 8, the only importable parser is the WASM module. Every test that currently imports the regex or Lezer parser must be adapted — but no task owns this work.

**Fix required**: Add a sub-task to Task 8 that migrates all remaining test files from `parseDocumentSkeleton`/`parseDocumentSkeletonFromLezer` imports to the WASM-backed `parseDocumentSkeleton` import from `skeleton.ts`. Also explicitly state the final import graph: `ast.ts` imports `skeleton.ts` imports `drummark_wasm.ts` imports WASM module.

---

#### 3. CRITICAL: Fallback contradicts cleanup

Task 6 AC #5 says *"Fallback to existing parser if WASM fails to load."* After Task 8 removes `lezer_skeleton.ts`, `parser.ts`, and all `@lezer/*` packages, there is no existing parser to fall back to. The application would simply break.

**Fix required**: Either (a) keep a minimal JS fallback parser (not viable given the migration goal), or (b) replace the fallback with a user-visible error message ("WASM failed to load — check your browser") and no longer claim fallback capability after Task 8. Task 8 should include this fallback-removal commit.

---

#### 4. MAJOR: No end-to-end format verification task

The proposal Phase 3 explicitly requires *"Verify all 4 output formats (ast, ir, svg, xml) produce identical results"* and the AGENTS.md mandates *"ALWAYS use `npm run drummark` to isolate the problem"* with `--format svg` and `--format xml`. Task 6 AC #4 says *"npm run drummark with WASM parser produces identical AST/IR/SVG/XML output"* but this is stated as a criterion, not as a verifiable task. No task describes:

- Which fixture files are used for end-to-end verification
- How SVG/XML output is compared (string diff, golden file, structural comparison)
- Whether this is automated in CI or a one-time manual check

**Fix required**: Add explicit acceptance criteria to Task 6 that name specific fixture files and comparison method. Alternatively, add a dedicated verification commit to Task 6 that runs `npm run drummark -- <fixture> --format svg` for a representative set of input files and diffs against expected output.

---

#### 5. MAJOR: Test adaptation for 35+ test files is unaccounted for

`src/dsl/` contains 35+ test files. Many explicitly test the regex parser vs. Lezer parser parity (`lezer_parity.test.ts`, `lezer_drift.test.ts`, `parser.test.ts`, `lezer_skeleton.test.ts`, `lezer-test.test.ts`). Task 8 deletes `lezer_parity.test.ts` and `lezer_drift.test.ts` but is silent on:

- `parser.test.ts` (tests the regex parser directly) — must be deleted or rewritten
- `lezer_skeleton.test.ts` (tests the Lezer walker directly) — must be deleted or rewritten
- `lezer-test.test.ts` — same
- `ast.test.ts` (tests `ast.ts` which imports Lezer) — must be adapted
- `benchmark.test.ts` — may need new baseline data

Task 5 adds new parity tests, but these compare Rust-vs-JS during migration. After cleanup, all remaining tests must work against the WASM-only parser.

**Fix required**: Expand Task 8 scope to explicitly list all test files that must be deleted, adapted, or rewritten. The cleanup should leave a test suite that validates the WASM parser output against expected `DocumentSkeleton` fixtures (golden-object comparison), not against another parser.

---

#### 6. MAJOR: `npm run build` compatibility not addressed

The existing `npm run build` pipeline is `tsc -b && npm run build-docs && vite build && npm run bundle:report`. After migration:

- `tsc -b` must type-check the new `src/wasm/` wrapper files
- `vite build` must bundle the WASM module
- `npm run bundle:report` should report WASM size

No task verifies that `npm run build` succeeds end-to-end with the WASM parser integrated.

**Fix required**: Add to Task 6 AC: `npm run build` succeeds and produces a working production bundle. Add to Task 7 AC: `npm run drummark` completes as a native binary call (not `tsx`).

---

#### 7. MODERATE: WASM bundle size verification missing

The proposal targets **~28KB gzipped** (Author Response Round 2, Revised Estimate). No task verifies this. The project already has `npm run bundle:report` (`scripts/report_bundle.mjs`). An oversized WASM bundle (e.g., accidentally pulling in `serde` or forgetting `wasm-opt`) would silently regress load times.

**Fix required**: Add a bundle-size verification to Task 3 or Task 6: either extend `scripts/report_bundle.mjs` to report WASM size, or add a CI check that the `.wasm` file is under a threshold (e.g., 35KB gzipped as an allowance buffer).

---

#### 8. MODERATE: `npm run typecheck:test` missing from task acceptance criteria

The Author Response Round 2 says *"A CI linter check (tsc --noEmit on the wrapper) will catch type mismatches at the TS boundary."* The existing CI already runs `npm run typecheck:test`. But:

- `tsconfig.test.json` may not include `src/wasm/` files
- The manually-maintained `drummark_core.d.ts` must be validated against the TS wrapper

No task explicitly includes TypeScript type-checking the WASM wrapper layer.

**Fix required**: Add to Task 6 AC: `npm run typecheck:test` passes with `src/wasm/` files included in the test tsconfig. Consider adding a targeted `tsc --noEmit src/wasm/` check.

---

#### 9. MODERATE: `npm run drummark` CLI path is ambiguous across tasks

The tasks describe three different states for `npm run drummark`:

| Task | `npm run drummark` behavior |
|------|---------------------------|
| Task 6 (AC #4) | Runs via `tsx`, uses WASM parser |
| Task 7 (commit #3) | Updated to use native binary |
| Task 8 (AC #7) | "now via native binary or WASM" |

The transition is unclear. Task 7 says "update npm run drummark to use native binary" but Task 6 already expects `npm run drummark` to work with WASM. If Task 7 changes the command, Task 6's verification becomes stale.

**Fix required**: Clarify the sequence. Suggested: Task 6 verifies that the WASM parser produces correct output when invoked programmatically (not via `npm run drummark`). Task 7 then updates the `drummark` script in `package.json` from `tsx src/cli.ts` to the native binary. Task 8 verifies the final state.

---

#### 10. MODERATE: `npm run test` with WASM as default — bootstrapping issue

Task 8 AC #5 says *"npm run test passes (existing tests now use WASM parser as default)."* This requires tests to import a WASM module, which needs a runtime environment that supports WASM. Vitest runs in Node.js, which supports WASM natively, but the WASM module must be pre-built before tests run. If `npm test` depends on `wasm-pack build` having been run, this must be documented or automated (e.g., `pretest` script).

**Fix required**: Add a note to Task 8 AC about the build-before-test requirement. Consider adding `wasm-pack build` as a `pretest` script or documenting the manual step.

---

#### 11. MINOR: Non-verifiable acceptance criteria

Several criteria are vague:

- Task 3 AC #5: *"~60 lines"* for `.d.ts` — line count is a guideline, not a pass/fail criterion.
- Task 7 AC #3: *"Startup time significantly faster than tsx (measurable)"* — "significantly" is subjective. Define a specific threshold (e.g., 5x faster cold start).
- Task 6 AC #1: *"~150 lines"* for `skeleton.ts` — same issue.

**Fix required**: Either remove line-count estimates from ACs (they're implementation guidance, not verification criteria) or make them specific (e.g., "skeleton.ts is under 200 lines").

---

#### 12. MINOR: Commit scope inconsistency for test files

Task 5 uses scope `test(wasm):` which is consistent. Task 8 uses bare `refactor:` without scope for commit `refactor: remove lezer_skeleton.ts and parser.ts`. The project convention is scoped commits (e.g., `refactor(build):`). A bare `refactor:` without a scope breaks the convention.

**Fix required**: Add scopes to the Task 8 refactor commits, e.g., `refactor(parser): remove lezer_skeleton.ts and parser.ts`.

---

### Dependency DAG Check

```
Task 1 -+- Task 2 -+- Task 3 - Task 4 - Task 5 - Task 6 - Task 7 - Task 8 - Task 9
        |           |
        +-----------+
```

No cycles. However, Task 3's dependency on Task 2 is questionable — AST types don't require the tokenizer to exist. The dependency on Task 1 (crate scaffolding) is sufficient. Consider making Task 2 and Task 3 parallel under Task 1.

---

### Summary

The tasks cover all proposal requirements at the feature level. However, the **CI integration**, **test migration**, **end-to-end verification**, and **parse-mode lifecycle** are underspecified to the point of implementation risk. The fallback contradiction between Task 6 and Task 8 would produce broken behavior in production if not addressed.

10 of 12 issues are actionable gaps (2 CRITICAL, 3 MAJOR, 5 MODERATE, 2 MINOR). The two MINOR issues are non-blocking.

STATUS: **CHANGES_REQUESTED**

### Author Response

#### Re 1: No CI Rust toolchain (CRITICAL)

Accepted. Task 1 AC #3 is tightened — "dev-only" escape hatch removed:

> `cargo build && cargo test && wasm-pack build --target web` run in CI on every push/PR. CI workflow updated to install `rustup` stable + `wasm-pack`.

An additional commit is added to Task 1: `ci(wasm): add Rust toolchain and wasm-pack to CI workflow`.

#### Re 2: Parse mode lifecycle (CRITICAL)

Accepted. Task 8 scope expanded:

- Add sub-commit: `refactor(test): migrate all test files to WASM-backed parseDocumentSkeleton`
- Tests currently importing `parseDocumentSkeleton` (regex) or `parseDocumentSkeletonFromLezer` (Lezer) are updated to import from `src/wasm/skeleton.ts`
- Files to adapt: `parser.test.ts` → deleted, `lezer_skeleton.test.ts` → deleted, `lezer-test.test.ts` → deleted, `ast.test.ts` → adapted, `benchmark.test.ts` → adapted
- Final import graph documented in Task 8 AC

#### Re 3: Fallback contradicts cleanup (CRITICAL)

Accepted. Task 6 AC #5 reworded:

> WASM loading failures display a user-visible error message ("Parser engine failed to load — check your network connection"). After Task 6, a JS stub exists for graceful degradation. After Task 8, the stub is removed and the error message is the only fallback path.

Task 8 adds a commit: `chore(wasm): remove WASM load fallback stub after cleanup`.

#### Re 4: No end-to-end format verification (MAJOR)

Accepted. Task 6 AC #4 expanded with specific verification steps:

> `npm run drummark -- <fixtures> --format svg` and `--format xml` produce pixel/struct-identical output for a representative fixture set: `spec-a01-simple.txt`, `spec-b-navigation.txt`, `spec-c-volta.txt`, `spec-d-groups.txt`, `spec-e-hairpins.txt`. Output diffed against golden files committed in the same task.

Adds a Task 6 sub-commit: `test(wasm): add end-to-end golden-file verification for all output formats`.

#### Re 5: Test adaptation for 35+ files (MAJOR)

Accepted. See Re 2 above — the test file migration is now an explicit sub-task of Task 8. The AC is expanded to list every test file that must be deleted, adapted, or rewritten with a `[x]` checklist.

#### Re 6: `npm run build` compatibility (MAJOR)

Accepted. Task 6 AC expanded:

> `npm run build` succeeds end-to-end: `tsc -b` type-checks `src/wasm/`, `vite build` bundles WASM module, `npm run bundle:report` reports WASM size.

Task 7 AC expanded:

> `npm run drummark` invokes native binary (verified via `which drummark` or `cargo run --` fallback in dev).

#### Re 7: WASM bundle size verification (MODERATE)

Accepted. Added to Task 6 AC:

> `npm run bundle:report` reports WASM module size. CI check verifies `.wasm` gzipped size ≤ 35KB (allowance buffer above 28KB target).

And a new Task 6 sub-commit: `chore(bundle): add WASM size threshold to bundle report`.

#### Re 8: `typecheck:test` missing (MODERATE)

Accepted. Added to Task 3 AC:

> `tsconfig.test.json` includes `src/wasm/` files. `npm run typecheck:test` passes.

Added to Task 6 AC:

> `drummark_core.d.ts` validated by `tsc --noEmit` on the `src/wasm/` files (enforced via `@ts-expect-error` negative tests for known-invalid type shapes).

#### Re 9: CLI path ambiguous across tasks (MODERATE)

Accepted. Task 6 AC #4 reworded from "npm run drummark" to "programmatic invocation":

> WASM parser invoked programmatically (not via `npm run drummark`) produces identical output to existing parsers for all 4 formats.

The `npm run drummark` script update is now exclusively in Task 7 commit #3: `chore(scripts): update npm run drummark from tsx to native binary`. Task 8 AC #7 verifies the final stable state.

#### Re 10: Test bootstrapping with WASM (MODERATE)

Accepted. Added to Task 8 AC:

> `npm run test` depends on WASM being pre-built. `wasm-pack build` is added as a `pretest` script in `package.json` if not already run by CI.

#### Re 11: Non-verifiable ACs (MINOR)

Accepted:

- Task 3 AC #5: Line count removed. Replaced with: "`.d.ts` file covers all exported types: `ParseResult`, `Document`, `HeaderSection`, `Paragraph`, `TrackLine`, `MeasureSection`, `TokenNode` (discriminated union), `Barline`, `ParseError`."
- Task 6 AC #1: Line count removed. Replaced with: "`skeleton.ts` converts WASM `JsValue` output to `DocumentSkeleton` without duplicating parser logic."
- Task 7 AC #3: Quantified: "`time drummark <input>` is ≥ 5x faster than `time tsx src/cli.ts <input>` for a 100-line score."

#### Re 12: Commit scope inconsistency (MINOR)

Accepted. All Task 8 refactor commits now have scope:

> `refactor(parser): ...`

#### Re Bonus: Task 3 dependency on Task 2

Accepted. Tasks 2 and 3 are now parallel (both depend only on Task 1). Updated dependency graph:

```
Task 1 ── Task 2 ────┐
   │                 ├── Task 4 ── Task 5 ── Task 6 ── Task 7 ── Task 8 ── Task 9
   └── Task 3 ───────┘
```

### Revised Tasks

The fixes above are inlined into the task definitions below.

---

### Task 1: Scaffold Rust Crate
- [ ] **Status**: Pending
- **Scope**: New `crates/drummark-core/` directory, Cargo.toml, CI workflow
- **Commits**:
  - `chore(wasm): scaffold drummark-core Rust crate with Logos + wasm-bindgen deps`
  - `ci(wasm): add Rust toolchain and wasm-pack to CI workflow`
- **Acceptance Criteria**:
  - `cargo build` succeeds in `crates/drummark-core/`
  - `wasm-pack build --target web` produces `pkg/` with `.wasm` and JS glue
  - `cargo build && cargo test && wasm-pack build --target web` run in CI on every push/PR. CI workflow updated to install `rustup` stable + `wasm-pack`
- **Dependencies**: None

### Task 2: Implement Logos Tokenizer
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-core/src/lexer.rs`
- **Commits**:
  - `feat(lexer): implement Logos tokenizer covering all DrumMark tokens`
  - `test(lexer): add tokenization parity tests against existing fixtures`
- **Acceptance Criteria**:
  - All tokens from `drum_mark.grammar` have corresponding Logos variants
  - MultiRest regex: `(1[0-9]+|[2-9][0-9]*)` correctly rejects `--1--`, accepts `--2--` and `--11--`
  - MeasureRepeat uses `#[regex(r"%+")]`, count from `.len()`
  - Comma declared before HeaderWord; HeaderWord regex excludes `,` and `#`
  - Glyph tokens: `BD2` > `BD` > `B` by longest-match, not declaration order
  - RoutedTrackPrefix (`@HH` etc.) and SummonPrefix (`HH:` etc.) tokens present
  - Navigation tokens (`@segno`, `@dc-al-fine`, etc.), hairpin tokens (`<`, `>`, `!`)
  - Modifier keyword tokens (`accent`, `open`, `half-open`, etc.)
  - Volta barline tokens (`|:.`, `||.`) present; `Comma` token present
  - Single `#[regex(r"#[^\n]*")] Comment` token (no separate CommentStart)
  - `cargo test` passes on all tokenizer unit tests against fixture files
- **Dependencies**: Task 1

### Task 3: Implement AST Types + WASM Bridge
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-core/src/ast.rs`, `crates/drummark-core/src/to_js.rs`, `crates/drummark-core/src/lib.rs`
- **Commits**:
  - `feat(ast): define Rust AST types mirroring DocumentSkeleton`
  - `feat(wasm): implement to_js() conversion for all AST types`
  - `feat(wasm): export parse() function via wasm-bindgen`
- **Acceptance Criteria**:
  - AST structs defined: `Document`, `HeaderSection`, `Paragraph`, `TrackLine`, `MeasureSection`, `MeasureExpr` (enum), `Barline`, etc.
  - All types support `to_js() -> JsValue` via `js_sys::Object`/`js_sys::Array`
  - `#[wasm_bindgen]` on exported `parse(source: &str) -> JsValue` function
  - `ParseError` struct with `line`, `column`, `message` fields
  - `drummark_core.d.ts` created manually covering all exported types: `ParseResult`, `Document`, `HeaderSection`, `Paragraph`, `TrackLine`, `MeasureSection`, `TokenNode` (discriminated union), `Barline`, `ParseError`
  - `wasm-pack build` produces working `.d.ts` + `.wasm`
  - `tsconfig.test.json` includes `src/wasm/` path; `npm run typecheck:test` passes
- **Dependencies**: Task 1

### Task 4: Implement Recursive Descent Parser
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-core/src/parser.rs`, `crates/drummark-core/src/error.rs`
- **Commits**:
  - `feat(parser): implement parser infrastructure (peek/peek_raw/peek_n/trivia/lookahead buffer)`
  - `feat(parser): implement header parsing (title, tempo, time, grouping, note, divisions)`
  - `feat(parser): implement track body parsing (TrackLine, MeasureSection, Barline, TrackBodyTail)`
  - `feat(parser): implement measure expression parsing (note, rest, group, combined hit, summons, routing)`
  - `feat(parser): implement suffix chain, modifiers, hairpins, navigation, repeats`
  - `feat(parser): implement inline braced block with recursive brace balancing`
  - `feat(parser): implement volta barline composite parsing`
- **Acceptance Criteria**:
  - Three peek methods: `peek()` (trivia-skip), `peek_raw()` (no skip), `peek_n(n: usize)` (buffered lookahead)
  - `skip_trivia()` consumes `Space` and `Comment`, NOT `Newline`
  - `parse_document()` produces `Document` AST from source string
  - VoltaBarline parsed as composite: `barline_prefix + Integer (Comma Integer)* Dot`; `parse_volta_barline` function reconstructs from token parts
  - InlineBracedBlock: `skip_trivia()` inside brace loop; `LBrace` triggers recursive `parse_remaining_braced_block()`; handles arbitrary nesting `{ x { d } b }`
  - TrackBodyTail disambiguated via `peek_n` lookahead past `Newline+` (distinguish `note Integer` from TrackName)
  - SuffixChain: dots, stars, halves, modifiers loop
  - All 19 track names, 19 glyphs, 13 modifiers recognized
  - MultiRest count extracted from regex; MeasureRepeat count from `%+` length
  - Navigation markers/jumps, hairpins, inline repeat suffix parsed
  - `cargo test` passes on all parser unit tests
- **Dependencies**: Task 2, Task 3

### Task 5: Parity Testing — Token & Structure Level
- [ ] **Status**: Pending
- **Scope**: New test fixtures, parity test runner
- **Commits**:
  - `test(wasm): add token-level parity test runner comparing Rust vs JS parsers`
  - `test(wasm): add structure-level parity test for document skeleton`
  - `test(wasm): debug and fix any parity discrepancies found`
- **Acceptance Criteria**:
  - All existing test fixtures (`src/dsl/*.test.ts`) produce identical `DocumentSkeleton` from both parsers
  - Specific parity test cases: multi-rest `--1--` rejection, volta `|1,2.` and `|:1.`, nested braces `{ x { d } b }`, TrackBodyTail `note` override, combined hits, hairpin span, navigation markers
  - Position reporting (line/column) matches existing error diagnostic tests
  - Parity tests run: `cargo test` parity suite passes
- **Dependencies**: Task 4

### Task 6: JS Wrapper & Pipeline Integration
- [ ] **Status**: Pending
- **Scope**: `src/wasm/drummark_wasm.ts`, `src/wasm/skeleton.ts`, `src/dsl/ast.ts` modifications, `vite.config.ts`
- **Commits**:
  - `feat(wasm): create JS wrapper for WASM parser module`
  - `feat(wasm): implement skeleton.ts adapting WASM output to DocumentSkeleton`
  - `feat(pipeline): wire WASM parser as third parse path in ast.ts (parseMode: "wasm")`
  - `chore(vite): configure WASM loading in vite.config.ts`
  - `test(wasm): add end-to-end golden-file verification for all output formats`
  - `chore(bundle): add WASM size threshold to bundle report`
- **Acceptance Criteria**:
  - `skeleton.ts` converts WASM `JsValue` output to `DocumentSkeleton` without duplicating parser logic
  - `ast.ts` supports `parseMode: "wasm"` alongside existing `"lezer"` and `"regex"`
  - Vite loads WASM module correctly in dev and production builds
  - WASM parser invoked programmatically produces identical AST/IR/SVG/XML output to existing parsers for fixture set: `spec-a01-simple.txt`, `spec-b-navigation.txt`, `spec-c-volta.txt`, `spec-d-groups.txt`, `spec-e-hairpins.txt`. Output diffed against golden files committed in this task
  - WASM loading failures display user-visible error message ("Parser engine failed to load — check your network connection"). JS stub exists for graceful degradation (removed in Task 8)
  - `npm run build` succeeds end-to-end: `tsc -b` type-checks `src/wasm/`, `vite build` bundles WASM module, `npm run bundle:report` reports WASM module size. CI check: `.wasm` gzipped size ≤ 35KB
  - `npm run typecheck:test` passes; `.d.ts` validated via `tsc --noEmit` on `src/wasm/`
- **Dependencies**: Task 5

### Task 7: Native CLI Binary
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-core/src/main.rs`, `Cargo.toml` bin target, `package.json` script update
- **Commits**:
  - `feat(cli): add native CLI binary reading stdin and formatting output`
  - `feat(cli): support --format ast|ir|json for native CLI`
  - `chore(scripts): update npm run drummark from tsx to native binary`
- **Acceptance Criteria**:
  - `cargo run -- <input>` produces same output as `tsx src/cli.ts <input>`
  - `--format ir`, `--format json` modes work from native binary
  - `time drummark <input>` is ≥5x faster than `time tsx src/cli.ts <input>` for a 100-line score
  - `npm run drummark` invokes native binary (verified via `cargo run --` fallback in dev)
  - CI builds native binary for macOS and Linux
- **Dependencies**: Task 6

### Task 8: Cleanup — Remove Lezer & Regex Parser
- [ ] **Status**: Pending
- **Scope**: Multiple files deleted, test migration, `package.json` updated, `AGENTS.md` updated
- **Commits**:
  - `refactor(parser): remove lezer_skeleton.ts and parser.ts`
  - `refactor(parser): remove drum_mark.grammar and generated parser files`
  - `refactor(test): migrate all test files to WASM-backed parseDocumentSkeleton`
  - `chore(deps): remove @lezer/* packages from package.json`
  - `chore(wasm): remove WASM load fallback stub after cleanup`
  - `chore(docs): update AGENTS.md with new build/parse commands`
- **Acceptance Criteria**:
  - Files deleted: `lezer_skeleton.ts`, `parser.ts`, `drum_mark.grammar`, `drum_mark.parser.js`, `drum_mark.parser.terms.js`
  - Test files deleted: `lezer_parity.test.ts`, `lezer_drift.test.ts`, `parser.test.ts`, `lezer_skeleton.test.ts`, `lezer-test.test.ts`
  - Test files adapted: `ast.test.ts`, `benchmark.test.ts` — imports changed to `src/wasm/skeleton.ts`
  - All remaining `src/dsl/*.test.ts` files import `parseDocumentSkeleton` from `src/wasm/skeleton.ts`
  - `parseMode` parameter removed from `ast.ts`; parser hardcoded to WASM path
  - Final import graph: `ast.ts` → `skeleton.ts` → `drummark_wasm.ts` → WASM module
  - `@lezer/lr`, `@lezer/generator`, `@lezer/markdown` removed from `package.json`; `npm ls @lezer` returns empty
  - `wasm-pack build` added as `pretest` script in `package.json`
  - `npm run drummark` works (native binary or WASM)
  - `npm run test` passes (all tests use WASM parser as default)
  - `npm run build` succeeds
  - AGENTS.md updated with new parser commands and Rust toolchain notes
- **Dependencies**: Task 7

### Task 9: Consolidate Proposal into Architecture Spec
- [ ] **Status**: Pending
- **Scope**: `docs/DRUMMARK_SPEC.md` or new architecture doc, archival
- **Commits**:
  - `docs: append consolidated Rust WASM parser design to architecture docs`
- **Acceptance Criteria**:
  - Final approved design (original proposal + all review fixes) consolidated into spec as a clean Addendum
  - No review noise — clean synthesis of all decisions
  - Proposal + tasks files moved to `docs/archived/`
- **Dependencies**: Task 8
