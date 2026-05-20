## Tasks: Migrate Normalizer to Rust

### Task 1: Fraction Math
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-core/src/fraction.rs`
- **Commits**:
  - `feat(normalize): implement Fraction type (u64, gcd-simplified) with add/multiply/divide/compare`
- **Acceptance Criteria**:
  - `Fraction { numerator: u64, denominator: u64 }` with `simplify()` before every operation
  - `add`, `multiply`, `divide`, `simplify`, `compare`, `fractions_equal`
  - `calculate_token_weight_as_fraction(dots, halves, stars, tuplet_span) -> Fraction`
  - `to_slot_count(divisions, beat_unit, beats)` for grid alignment
  - `basic_token_exceeds_exact_duration_range(denominator) -> bool` (compare against 2^53 threshold)
  - All existing `logic.test.ts` fraction cases pass as Rust tests
  - `cargo test` passes
- **Dependencies**: None

### Task 2: Magic Token Resolution
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-core/src/resolve.rs`
- **Commits**:
  - `feat(normalize): implement magic token → (track, glyph, modifiers) resolution`
- **Acceptance Criteria**:
  - Static mapping tables: `STATIC_MAGIC_TOKENS`, `ACCENT_MAGIC_TOKENS`, track-family sets
  - Accent-uppercase resolution: lowercase raw, lowercase+cached-track, uppercase+accent modifiers
  - ANONYMOUS context fallback via `resolveFallbackTrack` (defaults to `"HH"`)
  - Sticking track (ST) bypass: `R`/`L` on ST → no accent added
  - Track override precedence: `trackOverride` takes highest priority
  - `resolveToken(token, contextTrack, trackOverride?) -> ResolvedTrack`
  - Parity with `spec-c03-tokens.test.ts` cases
  - `cargo test` passes
- **Dependencies**: Task 1

### Task 3: Validation
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-core/src/validate.rs`
- **Commits**:
  - `feat(normalize): implement modifier legality, grouping boundary, group token, and repeat validation`
- **Acceptance Criteria**:
  - `validate_modifier_legality(token, track)` using `TRACKS_BY_MODIFIER` matrix
  - `validate_grouping(grouping, beats, divisions)` — sum check + integer boundary check
  - `validate_group_token(n, item_count, token_duration)` — ratio legality + below-64th check
  - `validate_repeat_spans(token, is_measure_repeat)` — nesting + conflict detection
  - Parity with `spec-c07-modifiers.test.ts`, `spec-c08-modifier-legality.test.ts`
  - `cargo test` passes
- **Dependencies**: Task 1, Task 2

### Task 4: Token → Event Expansion
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-core/src/event.rs`
- **Commits**:
  - `feat(normalize): implement token-to-event expansion (basic, group, combined, braced)`
- **Acceptance Criteria**:
  - `token_to_events(token, track, start, divisions, ...) -> Vec<NormalizedEvent>`
  - Basic token: single event with dots/halves/stars duration
  - Group token: tuplet subdivision, proportional duration split, group modifiers
  - Combined hit: multiple simultaneous events, proportionally weighted
  - Braced block: track-override events, duration split across items
  - Voice assignment: `BD, BD2, HF → voice 2; everything else → voice 1`
  - Beaming: set to `"none"` (actual beaming is in VexFlow)
  - Each `NormalizedEvent` carries `sourceOffset: u32` from the parser token
  - Parity with `spec-c04-resolution.test.ts`, `spec-c06-groups.test.ts`
  - `cargo test` passes
- **Dependencies**: Task 1, Task 2

### Task 5: Hairpin State Machine
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-core/src/hairpin.rs`
- **Commits**:
  - `feat(normalize): implement hairpin state machine (crescendo/decrescendo collection)`
- **Acceptance Criteria**:
  - Per-track `HairpinState` machine: `<` opens crescendo, `>` opens decrescendo, `!` closes
  - Cross-measure propagation of active hairpin state
  - Dangling hairpins closed at end of score
  - Cross-track deduplication and conflict detection
  - Parity with `spec-c22-hairpins.test.ts`
  - `cargo test` passes
- **Dependencies**: Task 4

### Task 6: Navigation & Volta
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-core/src/nav.rs`, `crates/drummark-core/src/volta.rs`
- **Commits**:
  - `feat(normalize): implement navigation resolution (startNav/endNav + barline forcing)`
  - `feat(normalize): implement volta propagation (forward sweep)`
- **Acceptance Criteria**:
  - Navigation: `ParsedStartNav/ParsedEndNav` → `StartNav/EndNav` with Fraction anchors
  - Cross-track marker merge (set union, canonical ordering: segno, coda, fine)
  - `fine` forces `barline: "final"`; other end-nav forces `barline: "double"`
  - Volta: forward sweep from seed measure until repeat-end/voltaTerminator
  - Parity with `spec-c11-repeat-barlines.test.ts`, `spec-c14-navigation.test.ts`, `spec-c12-voltas.test.ts`
  - `cargo test` passes
- **Dependencies**: Task 4

### Task 7: Core Normalization Engine
- [ ] **Status**: Pending
- **Scope**: `crates/drummark-core/src/normalize.rs`, `crates/drummark-core/src/to_js.rs` (expanded)
- **Commits**:
  - `feat(normalize): implement normalizeScoreAst orchestrator (main pass + 3 post-passes)`
  - `feat(normalize): implement normalizeExplicitMeasure pre-processing`
  - `feat(wasm): expand to_js.rs for NormalizedScore → JsValue conversion`
  - `feat(wasm): export buildNormalizedScore(source) -> JsValue via WASM binding`
  - `feat(parser): add offset tracking to AST nodes (Document, HeaderSection, NoteExpr, MeasureExprNode, GroupExpr, MeasureSection, Barline)`
- **Acceptance Criteria**:
  - Parser AST nodes gain `offset: u32` fields; `MeasureExpr` wrapped in `MeasureExprNode { expr, offset }`
  - Offset captured at `self.lexer.span().start` when token consumed, before next lookahead
  - `normalize_explicit_measure(measure)` → pre-pass: fill empty with rests, append rest-fill, resolve barline
  - Main pass: paragraph → measure → track → token walk, accumulating events + errors
  - Post-pass 1: volta propagation (forward sweep, ~20 lines)
  - Post-pass 2: hairpin closure + measure assignment (~30 lines)
  - Post-pass 3: unique track collection with family (~10 lines)
  - `NormalizedHeader` built from `ScoreAst.headers`
  - `NormalizedScore` output has: `version`, `header`, `tracks`, `measures`, `errors`, `repeatSpans`
  - All 14 fields of `NormalizedEvent`, 15 fields of `NormalizedMeasure` correctly serialized to JsValue
  - Source positions (`sourceOffset`) propagated from parser tokens through to NormalizedEvent
  - `NormalizedMeasure.sourceLine` computed from barline offset
  - WASM export: `build_normalized_score(source) -> JsValue` (parser + normalizer in one call)
  - `cargo test` passes
- **Dependencies**: Tasks 1–6

### Task 8: JS Adapter & Pipeline Integration
- [ ] **Status**: Pending
- **Scope**: `src/dsl/ast.ts`, `src/dsl/normalize.ts`, `src/cli_runtime.ts`, `src/vexflow/renderer.ts`
- **Commits**:
  - `refactor(pipeline): replace buildNormalizedScore with WASM direct call`
  - `refactor(types): add repeatSpans to NormalizedScore top-level, remove ast field`
  - `fix(cli): update --format ast to use new schema`
  - `fix(renderer): use score.repeatSpans instead of score.ast.repeatSpans`
  - `fix(musicxml): use score.header instead of score.ast.headers`
  - `chore: remove normalize.ts, logic.ts, ast.ts (replaced by WASM)`
- **Acceptance Criteria**:
  - `buildNormalizedScore(source)` calls WASM `build_normalized_score` directly
  - `--format ast` produces functional output with new schema
  - VexFlow renderer renders correctly with new `repeatSpans` location
  - MusicXML export produces valid output with new `header` access
  - `npm run drummark --format ir/svg/xml` all produce correct output
  - All 460+ existing tests pass (or adjusted for schema changes)
  - `normalize.ts`, `logic.ts`, `ast.ts` deleted (~1349 lines removed)
  - `cargo test` + `npm test` both pass
- **Dependencies**: Task 7

### Task 9: Parity Testing
- [ ] **Status**: Pending
- **Scope**: New parity test file, golden-file comparison
- **Commits**:
  - `test(normalize): add NormalizedScore parity tests (Rust vs TS golden files)`
  - `fix(normalize): resolve any parity discrepancies found`
- **Acceptance Criteria**:
  - 8 core parity test cases (identical NormalizedScore on all fields present in the TS schema):
    - simple single-track 4-bar
    - multi-track with anonymous lines
    - groups/tuplets
    - combined hits
    - hairpins (crescendo + decrescendo + close)
    - navigation markers (@segno, @dc, @fine)
    - repeat barlines with voltas
    - measure-repeat + multi-rest
  - All 8 produce semantically identical JSON on shared fields
  - `sourceLine` on NormalizedMeasure must match between TS and Rust
  - `sourceOffset` on NormalizedEvent is Rust-only (not in TS schema) — verified present and correct, not compared against TS
  - Additional edge cases mirroring `spec-c*.test.ts` fixtures
  - `npm test` includes WASM path parity
- **Dependencies**: Task 8
### Supersession Note: 2026-05-20 VexFlow Removal

Any uncompleted task in this file that names VexFlow rendering as an acceptance target is superseded by `ARCHITECTURE_proposal_remove_vexflow.md`.

Normalizer verification should target parser/IR/layout-owned outputs, not VexFlow rendering.
