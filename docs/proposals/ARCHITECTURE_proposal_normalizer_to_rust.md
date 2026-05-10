## Addendum v1.0: Migrate Normalizer to Rust/WASM

### Motivation

The parser migration (Tasks 1-9) moved the DSL parser from Lezer to Rust/WASM, but the normalizer (`normalize.ts`, 939 lines) remains in TypeScript. It consumes the parser's `DocumentSkeleton` → `ScoreAst` → `NormalizedScore`. Moving the normalizer to Rust completes the pipeline: source text → NormalizedScore entirely in Rust, with no intermediate TypeScript transformation.

**Current flow** (after parser migration):
```
Source → WASM Parser → DocumentSkeleton → ast.ts (518 lines) → ScoreAst → normalize.ts (939 lines) → NormalizedScore → renderer / MusicXML
```

**Proposed flow**:
```
Source → WASM (parser + normalizer) → NormalizedScore (JSON) → renderer / MusicXML
```

### Scope

| Included | Excluded |
|----------|----------|
| `normalizeScoreAst()` — the core normalization engine | VexFlow rendering |
| `logic.ts` fraction math (Fraction add/multiply/divide/simplify/compare) | MusicXML export |
| Magic token resolution (glyph → track mapping) | Settings/preferences |
| Modifier legality validation (track × modifier matrix) | CodeMirror highlighting |
| Voice assignment (`voiceForTrack`) | CLI binary (reuse existing) |
| Hairpin collection state machine | Documentation builder |
| Navigation marker resolution (startNav/endNav + barline forcing) | |
| Volta propagation (forward sweep across measures) | |
| Measure-repeat / multi-rest metadata pass-through | |
| Grouping boundary validation | |
| Token-to-event expansion (basic, group, combined, braced) | |
| Beaming (set to `"none"`, beam assignment is in VexFlow) | |
| Duration calculation (dots/halves/stars/tuplet sub-division) | |
| `buildNormalizedScore()` — convenience wrapper | |
| Parity tests comparing Rust NormalizedScore against TS NormalizedScore | |

### Architecture

```
┌────────────────────────────────────────────────────────────┐
│ Rust crate: drummark-core (expanded)                      │
│                                                            │
│  New modules:                                              │
│  ├── fraction.rs      # Fraction type + arithmetic        │
│  ├── normalize.rs     # normalize_score_ast()              │
│  ├── resolve.rs       # Magic token → (track, glyph)      │
│  ├── hairpin.rs       # Hairpin state machine             │
│  ├── nav.rs           # Navigation resolution             │
│  ├── volta.rs         # Volta propagation                 │
│  ├── validate.rs      # Modifier legality, grouping       │
│  └── event.rs         # Token → NormalizedEvent expansion │
│                                                            │
│  Existing modules (unchanged):                             │
│  ├── lexer.rs / parser.rs / ast.rs / to_js.rs             │
│                                                            │
│  WASM output: NormalizedScore as JsValue (via to_js)      │
└────────────────────────────────────────────────────────────┘
```

### Data Flow

```
Source String
    │
    ▼
Parser (Rust) → Document (AST)
    │
    ▼
ast.rs (Rust) → ScoreAst-like internal representation
    │
    ▼
normalize.rs (Rust) → NormalizedScore
    │
    ▼
to_js.rs → JsValue (typed JS object tree)
    │
    ▼
renderer.ts / musicxml.ts (unchanged TypeScript consumers)
```

The `NormalizedScore` JSON schema is the contract between Rust and JS. The WASM module exports a single function `normalize(source: &str) -> JsValue` that parses AND normalizes in one call.

### Key Design Decisions

#### Fraction Type

The existing TS `Fraction = { numerator: number; denominator: number }`. In Rust:

```rust
#[derive(Debug, Clone, Copy)]
pub struct Fraction {
    pub numerator: u32,
    pub denominator: u32,
}

impl Fraction {
    fn simplify(self) -> Self { /* gcd */ }
    fn add(self, other: Self) -> Self { /* ... */ }
    fn multiply(self, other: Self) -> Self { /* ... */ }
    fn divide(self, other: Self) -> Self { /* ... */ }
    fn compare(self, other: Self) -> Ordering { /* ... */ }
    fn to_slot_count(self, divisions: u32, beat_unit: u32, beats: u32) -> f64 { /* ... */ }
}
```

This replaces `logic.ts` functions `addFractions`, `multiplyFractions`, `divideFractions`, `simplify`, `compareFractions`, `fractionsEqual`, `calculateTokenWeightAsFraction`.

#### Magic Token Resolution

The TS `resolveToken()` maps shortcut glyphs (`s`, `b`, `c`, `t1`, etc.) to their canonical (track, glyph, modifiers) tuples based on the active track context. This is a static mapping table plus context-dependent logic. Ported directly as a Rust function:

```rust
fn resolve_token(token: &TokenGlyph, context_track: Option<&str>) -> ResolvedToken {
    // Static magic token table
    // Accent-uppercase mapping (c → C → C:d:accent)
    // Track-family defaults (cymbal→x notehead, drum→d notehead)
    // Sticking track handling
}
```

#### Hairpin State Machine

Per-track, per-measure state machine. `<` opens a hairpin of type "crescendo", `>` opens "decrescendo", `!` closes the active hairpin. Tracked across measures. At end of score, dangling hairpins are closed at the last measure.

```rust
struct HairpinState {
    active_type: Option<HairpinType>,
    active_start: Option<Fraction>,
    start_measure_index: usize,
}

fn collect_hairpins(
    events: &[NormalizedEvent],
    measure_index: usize,
    state: &mut HairpinState,
) -> Vec<HairpinIntent> { ... }
```

#### Volta Propagation

Forward sweep: a measure with `voltaIndices: [1, 2]` seeds an `activeVolta` that propagates to all subsequent measures until cleared by a repeat-end, repeat-both, or `voltaTerminator` barline. Implemented as a single-pass loop over measures.

#### Voice Assignment and Beaming

Voice assignment is static (`voiceForTrack` returns 1 or 2 based on track family). Beaming is set to `"none"` in normalize — actual beam calculation is in VexFlow. Both are trivial to port.

#### Source Position Tracking

Every parser AST node and every normalized event carries a byte offset into the source. This enables exact source-position parity (including `sourceLine` and `sourcePosition` fields) between TS and Rust normalizers.

**Parser layer**: `Document`, `HeaderSection`, `NoteExpr`, `MeasureSection`, `MeasureExpr`, `GroupExpr`, and `Barline` each gain an `offset: u32` field. The parser captures `self.lexer.span().start` when constructing each node. `ParseError` already carries `line`/`column` from the same offset.

**Normalizer layer**: `NormalizedEvent` gains `sourceOffset: u32`, carried forward from the parser token's offset. `NormalizedMeasure.sourceLine` is computed as `line_column(offset).0`. `NormalizedMeasure.sourcePosition` is computed from the barline's offset.

**to_js layer**: Offsets are serialized as the existing `SourceLocation { line, column, offset }` type that the TS pipeline already consumes.

**Parity testing**: `sourceLine` on `NormalizedMeasure` is compared between TS and Rust paths. `sourceOffset` on `NormalizedEvent` is a Rust-side diagnostic extension — not part of the TS schema — and is excluded from parity comparison. The 8 core parity cases compare all shared fields directly.

**Effort**: ~80 lines of Rust (parser nodes: ~50 lines for offset capture; normalizer: ~30 lines for offset propagation). Minimal complexity — purely additive.

**Design notes**:

- **`MeasureExpr` (enum)**: Wrapped in a `MeasureExprNode` struct: `{ expr: MeasureExpr, offset: u32 }`. All parser functions that construct `MeasureExpr` variants now return `MeasureExprNode` instead. `MeasureSection.tokens` stores `Vec<MeasureExprNode>`. The `offset` captures the byte position of the token that triggered the expression (e.g., glyph for BasicNote, `[` for Group, `{` for BracedBlock).

- **Offset capture timing**: `self.lexer.span().start` is read at the moment the token is consumed via `self.next()`, BEFORE the next `peek()`/`peek_n()` advances the lexer. This avoids the lookahead buffer skew.

- **`sourcePosition`**: The TS `NormalizedMeasure` type has only `sourceLine: number` (no `sourcePosition` field). The proposal text corrected — parity covers `sourceLine`; `sourceOffset` on events is the Rust-side diagnostic field, visible in `NormalizedEvent` but not in the TS schema (and therefore excluded from parity comparison on events).

- **Parity scope**: `sourceLine` on `NormalizedMeasure` is compared; `sourceOffset` on `NormalizedEvent` is Rust-only metadata (not in TS schema), excluded from parity comparison. This is documented explicitly as a known schema extension.

### Impact

| Metric | Current | After |
|--------|---------|-------|
| **JS normalize code** | 939 lines (`normalize.ts`) | ~0 (removed) |
| **JS logic code** | `logic.ts` fraction math (~410 lines) | ~0 (removed) |
| **JS AST builder** | 518 lines (`ast.ts`) | ~50 lines (thin adapter) |
| **Total JS reduction** | ~1867 lines | → ~50 lines |
| **Rust additions** | — | ~2700 lines (incl. ~80 lines position tracking) |
| **WASM size** | ~34KB gzip | ~55–65KB gzip (estimated) |
| **End-to-end latency** | Parse (WASM) + AST build (JS) + Normalize (JS) | Parse + Normalize (WASM, single call) |

### Migration Strategy (4 Phases)

#### Phase 1: Fraction Math
- [ ] Implement `fraction.rs` with full parity to `logic.ts`
- [ ] Add Rust tests matching `logic.test.ts`

#### Phase 2: Token Resolution & Validation
- [ ] Implement `resolve.rs` (magic token mapping)
- [ ] Implement `validate.rs` (modifier legality, grouping boundaries)
- [ ] Add parity tests against `spec-c07-modifiers.test.ts`, `spec-c08-modifier-legality.test.ts`

#### Phase 3: Core Normalization
- [ ] Implement `event.rs` (token → NormalizedEvent expansion)
- [ ] Implement `hairpin.rs` (hairpin state machine)
- [ ] Implement `nav.rs` (navigation resolution)
- [ ] Implement `volta.rs` (volta propagation)
- [ ] Implement `normalize.rs` (main orchestrator)
- [ ] Wire `buildNormalizedScore()` → WASM export

#### Phase 4: Parity & Integration
- [ ] NormalizedScore JSON parity against existing TS output
- [ ] Run all `spec-c*.test.ts` tests through WASM path
- [ ] VexFlow SVG output parity (smoke tests)
- [ ] MusicXML export parity
- [ ] CLI `--format ir` parity with TS normalizer

### Risk Mitigation

| Risk | Mitigation |
|------|------------|
| **Fraction precision mismatch** | Rust `Fraction` uses same `(u32, u32)` representation. Test against `logic.test.ts` cases. |
| **Magic token resolution differences** | Static mapping table + deterministic track-family logic. Parity tests against `spec-c03-tokens.test.ts`. |
| **Hairpin cross-measure bugs** | State machine is deterministic. Verify against `spec-c22-hairpins.test.ts`. |
| **Volta edge cases** | Forward sweep is simple. Parity tests against `spec-c12-voltas.test.ts`. |
| **NormalizedScore schema drift** | WASM output uses the same JSON schema. `to_js.rs` mirrors the TS types. |

### Dependency Comparison

| | Current (TS) | Proposed (Rust) |
|---|---|---|
| **Fraction math** | `logic.ts` ~200 lines | `fraction.rs` ~100 lines |
| **Normalizer** | `normalize.ts` 939 lines | `normalize.rs` + submodules ~1200 lines |
| **Token resolution** | `resolveFallbackTrack` in `logic.ts` | `resolve.rs` ~150 lines |
| **Validation** | `validateModifierLegality()` in `normalize.ts` | `validate.rs` ~120 lines |
| **AST builder** | `ast.ts` 518 lines | `ast.rs` expanded ~200 lines |
| **Runtime deps** | `logic.ts` + `normalize.ts` + `ast.ts` (JS) | WASM module (single call) |


### Review Round 1

**Reviewer**: Critical Architect (sub-agent)
**Date**: 2026-05-10

---

#### 1. Voice Assignment Description is Factually Wrong

**Proposal / Tasks file (Task 4)** states: "Voice assignment via `voiceForTrack` (cymbal/pedal/perc → 1, drum → 2)".

**Actual implementation** in `logic.ts:142-151`:

```ts
export function voiceForTrack(track: TrackName): VoiceId {
  switch (track) {
    case "BD": case "BD2": case "HF":
      return 2;
    default:
      return 1;
  }
}
```

- "cymbal → 1" is correct for HH, RC, C, etc.
- "percussion → 1" is correct for CB, WB, CL.
- "pedal → 1" is **wrong** — HF is voice **2**.
- "drum → 2" is **wrong** — only BD/BD2 are voice 2. SD, T1–T4 are voice **1**.

This is an explicit track-list decision, not a family-based rule. The voice mapping is: BD, BD2, HF → 2; everything else → 1.

---

#### 2. Fraction Type Design: u32 vs IEEE 754 Double

The TS `Fraction` uses JS `number` (IEEE 754 double, 53-bit mantissa). The proposal uses **`u32`**. While practical music-domain values are small (dots ≤ 4, stars/halves ≤ 5 → values within u32), this difference has two unaddressed consequences:

**(a) Overflow detection gap.** The TS function `basicTokenExceedsExactDurationRange` (`logic.ts:13-23`) checks against `token.dots > 52` (mantissa precision guard) and `numeratorExponent > 1023` (IEEE 754 exponent limit). With `u32`, the equivalent check would be different — whether `2^(dots+1) * 2^(stars-halves)` exceeds `u32::MAX`. The proposal says `fraction.rs` must have "full parity to `logic.ts`" (Phase 1) but does not specify how `basicTokenExceedsExactDurationRange` translates to the `u32` domain.

**(b) Denominator overflow in `addFractions`.** With `u32`, `left.denominator * right.denominator` can overflow when two large coprime denominators multiply. TS has no overflow — doubles just lose precision. Rust needs either `u64` intermediates, `checked_mul` with widening-to-`u128`, or else silent overflow produces wrong fractions.

**Recommendation**: Either (a) use `u64` for numerator/denominator fields (safe for all music-domain values, no intermediate overflow), or (b) document a `checked_*` pattern and ensure the test suite covers maximum-practical-value edge cases.

---

#### 3. "Single-Pass" Claim is Overstated

**Task 7 acceptance criteria** states: "Single-pass paragraph → measure → track → token walk".

The actual TS `normalizeScoreAst` (lines 605–935) has **four passes**:

| Pass | Lines | Purpose |
|------|-------|---------|
| 1 (main) | 613–837 | Token expansion, modifier validation, grouping check, nav resolution, hairpin collection, measure construction |
| 2 (volta) | 839–858 | Forward sweep assigning volta seeds to subsequent measures |
| 3 (hairpin) | 868–894 | Dangling hairpin closure + hairpin assignment to measures |
| 4 (tracks) | 913–920 | Unique track collection |

Passes 2–4 are short (8–20 lines each) but have ordering dependencies (pass 2 reads pass 1's `voltaSeeds`; pass 3 reads pass 1's `trackHairpinStates`). Calling this "single-pass" risks a Rust implementation that incorrectly interleaves these phases.

**Recommendation**: Replace "Single-pass" with "Main pass" and list the three post-processing passes explicitly.

---

#### 4. Task 3 Missing Dependency on Task 2

**Tasks file**, Task 3 (Validation) lists dependency: **Task 1** only.

However, `validateModifierLegality` (`normalize.ts:562-603`) calls `resolveToken(token, contextTrack)`, which in turn uses `resolveFallbackTrack`, `STATIC_MAGIC_TOKENS`, `ACCENT_MAGIC_TOKENS`, track-family sets, and `getTrackFamily` — all defined in Task 2's scope. Without Task 2, validation cannot resolve tokens to tracks, so the modifier × track legality matrix (`TRACKS_BY_MODIFIER`) cannot be checked.

**Recommendation**: Add Task 2 as a dependency of Task 3.

---

#### 5. Missing Concern: `to_js.rs` Expansion

The existing `to_js.rs` (240 lines) converts the parser's `DocumentSkeleton` to JS types. After this migration, it must also convert `NormalizedScore` (including `Vec<NormalizedMeasure>`, `Vec<NormalizedEvent>`, `NormalizedHeader`, etc.) to `JsValue`. This is non-trivial:

- `NormalizedEvent` has 14 fields including enums (`NormalizedEventKind`, `TieState`, `BeamState`, `Modifier`, `TupletSpec`)
- `NormalizedMeasure` has 15 fields including optional nested types
- Hairpin, Volta, MeasureRepeat, MultiRest, and navigation types all need JS representations

The proposal estimates Rust additions at ~1500–2000 lines but does not explicitly account for the ~200–300 lines of `to_js.rs` expansion. This should be part of Task 7 or a sub-item.

---

#### 6. Magic Token Resolution: Missing Edge Case Documentation

The proposal states: "Static mapping table plus context-dependent logic. Ported directly as a Rust function." The following resolution edge cases are not documented:

**(a) ANONYMOUS context fallback.** When `contextTrack === "ANONYMOUS"` and the token is NOT in `STATIC_MAGIC_TOKENS` (e.g., bare `d`, `x`, `p`, `g`, `L`), the resolver falls through to `resolveFallbackTrack(token.value)` which defaults to `"HH"` for unrecognized glyphs (`logic.ts:284`). This means `d` in an anonymous track resolves to track HH with glyph `d`.

**(b) Uppercase accent mapping is case-sensitive.** Lowercase `c` is in `STATIC_MAGIC_TOKENS` but NOT in `ACCENT_MAGIC_TOKENS`. The accent modifier is only added for uppercase magic tokens. The proposal says "c → C → C:d:accent" but doesn't clarify that accent is added iff the token value is uppercase and not in the ST sticking track.

**(c) Sticking track (ST) + magic tokens.** When `contextTrack === "ST"` and `token.value ∈ {R, L}`, the token resolves as a sticking glyph on track ST without accent. The accent-bypass condition `!(track === "ST" && stickingToken)` at normalize.ts:224 prevents `R` (which IS in `ACCENT_MAGIC_TOKENS`) from receiving accent. This interaction should be explicitly documented.

**(d) Track override precedence.** `trackOverride` (from braced-token syntax `{T1: d}`) takes highest priority, overriding contextTrack and magic token defaults. The resolution rules should list this explicitly.

---

#### 7. `ast.ts`: Delete vs. Adapter

The proposal says `ast.ts` reduces from 518 lines to "~150 lines (thin adapter)". No task addresses this transformation. The data flow diagram (lines 62–78) shows "Source → WASM (parser + normalizer) → NormalizedScore", which bypasses `ScoreAst` entirely. This implies `ast.ts` is **deleted**, with ~150 lines of JS glue in `src/wasm/` or `buildNormalizedScore`. The proposal should clarify:

- **Option A** (implied): WASM exposes `normalize(source: &str) -> JsValue` directly, `ast.ts` is deleted.
- **Option B**: WASM exposes `normalize_score_ast(DocumentSkeleton) -> JsValue`, `ast.ts` remains as a thin adapter.

---

#### 8. Missing `ast.ts` Functions Not Accounted For

If `ast.ts` is deleted (Option A in item 7), the following functions must move into Rust. None are mentioned in any task:

**(a) `validateGroupToken`** (`ast.ts:221-281`). Validates group ratios (only `2:1, 3:1, 4:1, 3:2, 4:2, 5:4, 6:4, 7:4`), checks item durations below 64th notes, checks zero-count groups (which produces `score.ast.errors`).

**(b) `validateGrouping`** (`ast.ts:185-219`). Validates grouping header: sum must equal beats, boundaries must fall on integer slots. Not mentioned in any phase.

**(c) `validateAndBuildRepeats` / `validateMeasureMetadata`** (`ast.ts:283-401`). Validate repeat span nesting, unmatched open/close, measure-repeat conflicts. These produce `RepeatSpan[]` and errors stored in `ScoreAst`.

**(d) `normalizeExplicitMeasure`** (`ast.ts:67-116`). Fills empty measures with rest tokens, appends rest-fill for under-full measures, resolves `barline` from repeat flags. This is pre-processing that must happen in Rust if the pipeline goes directly from parser to normalizer.

These total ~350 lines. If they remain in `ast.ts`, then `ast.ts` cannot be deleted and the ~150-line estimate is wrong.

---

#### 9. `NormalizedScore.ast` Field: Critical Schema Drift Risk

The current `NormalizedScore` type includes `ast: ScoreAst` (`types.ts:424`). The WASM normalizer cannot construct a `ScoreAst` in Rust without mirroring its full type hierarchy (paragraphs, ScoreTrackParagraph, ScoreMeasure, RepeatSpan, etc. — ~100+ lines of type definitions plus construction code). The proposal acknowledges `NormalizedScore` schema as a **contract** but does not discuss the `ast` field at all.

Grep confirms renderer.ts accesses `score.ast.repeatSpans`. If `ast` is removed, repeat spans need their own field on `NormalizedScore`. If kept, Rust must construct it. The proposal must specify which path.

---

#### 10. Size Estimates: Underestimated

| Item | Proposal | Realistic |
|------|----------|-----------|
| Rust additions | 1500–2000 lines | **2200–2500 lines** (+`to_js.rs` expansion ~250, +`ast.ts` validations ~350, +`ScoreAst` mirror types ~150) |
| WASM size | 45–50 KB gzip | Plausible but unvalidated; could push to 55 KB+ with data tables |
| JS reduction | ~1650 → ~150 lines | ~50-80 lines if `ast.ts` deleted; minimal if kept as fallback |

---

#### 11. Task Ordering

Overall ordering (Fraction → Resolution + Validation → Event Expansion → Hairpin + Nav/Volta → Orchestrator → Parity) is sound. Only the missing Task 2 dependency on Task 3 needs fixing (see item 4).

---

#### Verdict

The proposal is well-structured and covers the major subsystems. However, the following are **blocking**:

1. **Voice assignment description is wrong** (item 1) — factual error in acceptance criteria.
2. **`NormalizedScore.ast` field schema drift** (item 9) — unaddressed compatibility issue; consumers depend on `ast.repeatSpans`.
3. **Fraction type overflow semantics** (item 2) — `u32` overflow in arithmetic needs explicit design.
4. **Missing `ast.ts` validation functions** (item 8) — ~350 lines not accounted for in any phase.
5. **Task 3 missing dependency on Task 2** (item 4).

**STATUS: CHANGES_REQUESTED**

### Author Response

#### Re 1: Voice assignment wrong (CRITICAL)

Accepted. Correction: voice mapping is `BD, BD2, HF → 2; everything else → 1`. This is a three-track list, not a family-based rule. Tasks file and proposal text corrected.

#### Re 2: Fraction u32 overflow (CRITICAL)

Accepted. Switch to `u64` for numerator/denominator — eliminates overflow risk for all music-domain values while staying within WASM's i64 capability (no 128-bit ops needed). `checked_mul` with `u64` and `simplify()` (gcd reduction) before every operation keeps intermediates small.

`basicTokenExceedsExactDurationRange`: the TS check guards against IEEE 754 precision loss at extreme dot/star counts. With `u64` fractions, the equivalent is `denominator > 2^53` (where mantissa precision fails in JSON serialization). Documented explicitly in `fraction.rs`.

#### Re 3: "Single-pass" overstated

Accepted. Reword to "Main pass + three post-processing passes". The post-passes are: (a) volta propagation (forward sweep, ~20 lines), (b) hairpin closure (dangling + assignment, ~30 lines), (c) track collection (unique set, ~10 lines). All are short and order-dependent on the main pass output.

#### Re 4: Task 3 missing dep on Task 2

Accepted. Task 3 dependencies updated: `Task 1, Task 2`.

#### Re 5: to_js.rs expansion

Accepted. Add ~250 lines estimate for NormalizedScore → JsValue conversion. Added as explicit sub-item in Task 7 AC: "`NormalizedScore` fully convertible to JsValue via `to_js.rs`".

#### Re 6: Magic token edge cases

All accepted. The four cases (ANONYMOUS fallback, uppercase accent mapping, ST sticking bypass, trackOverride precedence) are documented in `resolve.rs` design. Ported directly from the TS implementation with identical logic.

#### Re 7: ast.ts delete vs adapter

**Chosen: Option A** — WASM directly exposes `normalize(source: &str) -> JsValue` (parser + normalizer in one call). `ast.ts` reduces to a ~50-line adapter that calls WASM and forwards the result. The ScoreAst → NormalizedScore transformation moves entirely into Rust.

#### Re 8: Missing ast.ts validation functions

Accepted. Four functions must move to Rust:
- `validateGroupToken` → `validate.rs` (~60 lines)
- `validateGrouping` → `validate.rs` (~40 lines)
- `validateAndBuildRepeats` / `validateMeasureMetadata` → `normalize.rs` pre-pass (~80 lines)
- `normalizeExplicitMeasure` → `normalize.rs` main pass (~50 lines)

These are added to Tasks 3 and 7. Revised Rust estimate: **2200–2600 lines**.

#### Re 9: NormalizedScore.ast field

Accepted. Two viable paths:

**Path A**: Keep `ast` on NormalizedScore. Rust constructs a minimal ScoreAst mirror type. At minimum: `repeatSpans`, `errors` array, paragraph count. ~100 lines of Rust types.

**Path B**: Remove `ast` from NormalizedScore, add top-level `repeatSpans: RepeatSpan[]` to NormalizedScore. Renderer only accesses `ast.repeatSpans` and `ast.errors` → both available directly on NormalizedScore. ~20 lines change in renderer + ~20 lines in Rust.

**Recommendation: Path B** — it's the simpler change and eliminates the need for a ScoreAst mirror type.

#### Re 10: Size estimates

Accepted. Revised:
- Rust additions: **2200–2600 lines**
- WASM size: ~**50KB gzip** (including fraction tables, magic token tables, modifier matrix)
- JS reduction: `ast.ts` → ~50 lines adapter; `normalize.ts` + `logic.ts` → deleted (~1150 lines total)

#### Re 11: Task ordering

Accepted. Task 3 dependency corrected to include Task 2.

### Review Round 2

**Reviewer**: Critical Architect (sub-agent)
**Date**: 2026-05-10

---

#### Summary Assessment

All 11 Round 1 issues were conceptually accepted and addressed in the Author Response. However, I find **five new issues** — two are blocking (Tasks file sync, `--format ast` gap), three are material but non-blocking.

---

#### Blocking Issues

##### A. Tasks File Not Synchronized with Author Response (BLOCKING)

The Author Response states corrections were applied to the Tasks file for items 1, 3, 4, 5, and 8. Inspection of `/docs/proposals/ARCHITECTURE_tasks_normalizer_to_rust.md` shows **zero updates were made**. Concrete gaps:

| Author Response | Tasks File (actual) |
|---|---|
| Fraction → `u64` (Re 2) | Task 1 AC: `numerator: u32, denominator: u32` (line 9) |
| Task 3 deps: `Task 1, Task 2` (Re 4) | Task 3: `Dependencies: Task 1` (line 44) |
| Voice: `BD, BD2, HF → 2` (Re 1) | Task 4 AC: `cymbal/pedal/perc → 1, drum → 2` (line 57) |
| "Main pass + three post-passes" (Re 3) | Task 7 AC: `Single-pass paragraph → measure → track → token walk` (line 99) |
| `to_js.rs` ~250 lines (Re 5) | Not mentioned in any task |
| ast.ts validations (Re 8) | Not mentioned in any task (Task 3 AC only lists modifier/grouping; Task 7 has no mention) |
| `ast.ts` → ~50-line adapter (Re 7) | Not mentioned in any task |

The Tasks file is the implementation plan. Without these corrections, a sub-agent implementing Task 1 will use `u32`, Task 7 will attempt a single-pass, and Task 3 will lack the token resolver it needs. **The Tasks file must be updated before implementation begins.**

##### B. Path B + `--format ast` CLI Gap (BLOCKING)

The Author Response chooses Path B (remove `ast` from `NormalizedScore`, add top-level `repeatSpans`). This eliminates the `ScoreAst` from the normalizer output. However, the CLI has a `--format ast` option that does:

```ts
// cli_output.ts:7
return JSON.stringify(score.ast, null, 2);
```

If `ast` is removed from `NormalizedScore`, `--format ast` produces nothing. Two sub-issues:

1. **No replacement documented.** Neither the proposal, Author Response, nor Tasks file mentions what happens to `--format ast`. Options: (a) remove the format, (b) add a `parse_only(source) -> JsValue` WASM export that returns the parser AST, (c) keep a minimal `ScoreAst` mirror with only the fields needed by `--format ast` (headers, paragraphs, repeatSpans, errors). The choice must be explicit.

2. **Consumers not acknowledged.** `cli_output.ts:7,11` and tests (`cli_output.test.ts:25`, `cli_runtime.test.ts:63`) all reference `score.ast` / `parsed.ast`. These were not mentioned in the Author Response or in any task.

---

#### Material Issues (Non-Blocking)

##### C. `musicxml.ts` Consumer of `score.ast.headers.*`

The Author Response states Path B removes `ast` from NormalizedScore, and claims "Renderer only accesses `ast.repeatSpans` and `ast.errors`." This is incomplete:

```ts
// musicxml.ts:224-226
const title = score.ast.headers.title?.value ?? "DrumMark";
const subtitle = score.ast.headers.subtitle?.value;
const composer = score.ast.headers.composer?.value;
```

These same values already exist on `score.header.title`, `score.header.subtitle`, `score.header.composer` (built by `normalize.ts:900-902` and mirrored on `NormalizedHeader` type, `types.ts:401-404`). The fix is a 3-line refactor in `musicxml.ts` — trivial, but it must be listed as a task item. Currently not mentioned.

##### D. Revised Size Estimates Inconsistent

The Author Response (Re 10) states: "JS reduction: `ast.ts` → ~50 lines adapter; `normalize.ts` + `logic.ts` → deleted (~1150 lines total)."

Actual line counts (`wc -l`):
- `normalize.ts`: **939** lines
- `logic.ts`: **410** lines
- `ast.ts`: **538** lines
- **Total**: 1887 lines

So `normalize.ts + logic.ts = 1349` lines, not ~1150. After migration: ~50 lines remain. Net reduction: ~1837 lines. The Author Response's figure understates the reduction by ~700 lines. This is not harmful but undermines confidence in the other estimates.

Additionally, the WASM size estimate was revised from "45–50KB" to "~50KB gzip" but the new scope adds:
- `to_js.rs` expansion: ~250 lines
- ast.ts validations: ~230 lines
- Magic token + modifier static tables (data, not just code)
- Fraction math tables

With these additions, 55–60KB gzip is more realistic. "~50KB" is optimistic.

##### E. `normalizeExplicitMeasure` Phrasing

The Author Response (Re 8) says `normalizeExplicitMeasure` goes into "`normalize.rs` main pass (~50 lines)." In the current TS flow, this function (`ast.ts:67-116`) runs in `buildScoreAst` **before** `normalizeScoreAst`. In the WASM flow (parser → normalizer directly), it must run as a **pre-processing step** within the normalizer before the main measure-walk loop. The "main pass" phrasing is ambiguous — it should be explicitly designated as a pre-pass. This is minor; the architecturally correct behavior is clear.

---

#### Path B Regression Audit

I traced every access to `score.ast` in the codebase to verify Path B's claim that only `repeatSpans` and `errors` need lifting:

| Location | Field accessed | Path B impact |
|---|---|---|
| `musicxml.ts:224-226` | `ast.headers.title/.subtitle/.composer` | Refactor to `score.header.*` (3 lines) — **not mentioned** |
| `cli_output.ts:7` | `score.ast` (entire) | **Blocking — see Issue B** |
| `cli_output.ts:11` | `delete score.ast` | Removal (1 line) — trivial |
| `spec-c11-repeat-barlines.test.ts:190,234` | `ast.repeatSpans` | Refactor to `score.repeatSpans` (test-only) |
| `normalize.ts:910` | `ast.headers.grouping.values` | Moves inside Rust normalizer — OK |
| `cli_output.test.ts:25`, `cli_runtime.test.ts:63` | `parsed.ast` | Tests need updating — **not mentioned** |

The `ast` field on `NormalizedScore` is `types.ts:424`. With Path B, this field is removed, and a new `repeatSpans: RepeatSpan[]` field is added. The `errors` field already exists at top level (`types.ts:426`). The `header` metadata already exists at `types.ts:422`. Path B is technically sound *if* the consumer refactors are tracked. Currently only 2 of 5 consumers are accounted for.

---

#### Revised Size Estimate Audit

| Item | Original Proposal | Revised Author Response | Auditor's Verified |
|---|---|---|---|
| Rust additions | 1500–2000 | 2200–2600 | 2200–2600 (plausible, given additions) |
| WASM gzip | 45–50 KB | ~50 KB | **55–65 KB** more likely (see Issue D) |
| JS reduction | ~1650 → ~150 | ~1150 (normalize+logic) | **1887 → ~50** (~1837 removed) |
| `to_js.rs` expansion | not estimated | ~250 lines | Unchanged — plausible |
| `ast.ts` validations | not accounted | ~230 lines total | Unchanged — plausible |

---

#### Verdict

The Author Response addresses all 11 Round 1 issues at the conceptual level. The core design is sound. However:

1. **The Tasks file is stale** — zero of the promised corrections were applied. This makes the implementation plan inconsistent with the agreed-upon design. **(BLOCKING)**
2. **The `--format ast` CLI gap is unaddressed** — removing `ast` from `NormalizedScore` breaks a CLI feature and tests without a documented replacement. **(BLOCKING)**
3. **The `musicxml.ts` consumer is not acknowledged** — a trivial but necessary refactor is missing from the task plan.

All five new issues require explicit acknowledgment and correction in the Author Response (and the Tasks file must be physically updated).

**STATUS: CHANGES_REQUESTED**

### Review Round 3

**Reviewer**: Critical Architect (sub-agent)
**Date**: 2026-05-10

---

All 5 Round 2 issues were addressed in Author Response. Not re-reviewed.

### Review Round 4

**Reviewer**: Critical Architect (sub-agent)
**Date**: 2026-05-10

**Scope**: Only the new "Source Position Tracking" section (proposal lines 147–159) and corresponding Task 4 / Task 7 / Task 9 updates.

---

#### 1. `MeasureExpr` is an enum — cannot carry an `offset` field (BLOCKING)

The proposal states (line 151):

> `MeasureExpr` ... gain[s] an `offset: u32` field.

`MeasureExpr` is defined as a bare Rust enum with 14 variants (`ast.rs:51-66`). In Rust, you cannot add a struct-level field to an enum. The only path to attaching an offset to every measure expression is one of:

| Approach | Cost | Notes |
|----------|------|-------|
| **(a) Wrapper struct** `SpannedMeasureExpr { expr: MeasureExpr, offset: u32 }` | Changes every consumer of `MeasureExpr` in parser and `to_js.rs`; `MeasureSection.tokens` becomes `Vec<SpannedMeasureExpr>` | Most explicit; cleanest separation of concerns |
| **(b) Store offset on inner structs** (`NoteExpr`, `GroupExpr`) + capture in `parse_measure_expr` for enum-only variants | `parse_measure_expr` must return `(MeasureExpr, u32)` tuple; callers must thread it | Works, but enum-only variants (Crescendo, HairpinEnd, NavMarker, etc.) still have no home for the offset — the caller must preserve the tuple |
| **(c) Parallel `Vec<u32>`** in `MeasureSection` | `MeasureSection { offsets: Vec<u32>, tokens: Vec<MeasureExpr> }` — two parallel vectors that must stay in sync | Fragile; offset and token indices can diverge on mutation |

None of these approaches is discussed or selected in the proposal or tasks file. An implementer will hit this immediately.

**Recommendation**: Specify the chosen approach. Option (a) (wrapper struct) is the most Rust-idiomatic and least error-prone. If chosen, update `MeasureSection.tokens: Vec<MeasureExpr>` → `Vec<SpannedMeasureExpr>`, and update all parser consumers and `to_js.rs` accordingly. Account for the extra ~30 lines of refactoring.

---

#### 2. `sourcePosition` is a phantom field — does not exist in the TS schema (BLOCKING)

The proposal (line 153) and tasks file (Task 7 line 113, Task 9 line 146) refer to a field called `sourcePosition` on `NormalizedMeasure`:

> `NormalizedMeasure.sourcePosition` is computed from the barline's offset.

But the TS type `NormalizedMeasure` (`types.ts:368-385`) has **no such field**. The only position-related field is:

```ts
sourceLine: number;  // line 373
```

A full-text search (`grep sourcePosition`) confirms zero matches in the entire `src/` tree. The field exists only in the proposal and tasks file.

**Recommendation**: Either:
- (a) Rename all occurrences to `sourceLine` (the actual field name), or
- (b) Define `sourcePosition` as a new field (presumably a `u32` byte offset?) and add it to the TS `NormalizedMeasure` type as part of Task 8. If choosing (b), clarify the type (`number` in TS, `u32` in Rust) and semantics.

---

#### 3. Parity testing claim contradicts existing TS output (BLOCKING)

The proposal (line 157) and Task 9 (lines 146-157) claim:

> No field stripping — source positions must match between TS and Rust paths.

> The 8 core parity test cases must produce byte-for-byte identical JSON output.

But the current TS `NormalizedEvent` (`types.ts:351-366`) has **no per-event position field**. The TS `NormalizedMeasure` has only `sourceLine` — no `sourceOffset`, no `sourcePosition`. The proposal introduces two new fields (`sourceOffset` on NormalizedEvent, plus the phantom `sourcePosition`) that the existing TS normalizer does not produce.

For the Rust output and TS output to be byte-for-byte identical, the TS normalizer must be modified to also emit these new fields. Otherwise the Rust path produces extra keys that fail a strict equality comparison. If the intent is to add the fields and modify the TS normalizer, that work is not scoped in any task.

**Existing parity test behavior** (`wasm_parity.test.ts:85-86`): currently strips `line`, `lineNumber`, `startLine`, `startOffset`, `globalIndex`. That is the opposite of "no field stripping."

**Recommendation**: Either:
- (a) **Patch the TS normalizer** to emit `sourceOffset` on each NormalizedEvent and any new measure-level field — add this to Task 8's scope with explicit line estimates (~40 lines), or
- (b) **Scope parity differently**: the new fields match Rust-vs-Rust golden files, not Rust-vs-TS. Then the TS normalizer does not need modification and the parity claim is scoped to "TS-parser-vs-Rust-parser" only (which already works).

---

#### 4. `self.lexer.span().start` is unreliable after `peek_n(N)` with `N > 0` (BLOCKING)

The proposal (line 151) says:

> The parser captures `self.lexer.span().start` when constructing each node.

In logos, `Lexer::span()` returns the span of the **last token iterated from the lexer**. The parser has a peek buffer (`peek_buf`). When `peek_n(1)` or `peek_n(2)` is called, multiple tokens are read from the lexer into the buffer. After that, `self.lexer.span()` points to the **last** buffered token, not the one the parser is about to consume via `next()`.

Concrete failure case in `parse_group` (`parser.rs:593-621`):

```rust
// parse_group
let n = if let Some(Token::Integer(num)) = self.peek() {   // fills buffer[0]
    if self.peek_n(1) == Some(Token::Colon) {              // fills buffer[1]; span → Colon
        self.next().ok(); // returns Integer from buffer[0]; span still → Colon
        self.next().ok(); // returns Colon from buffer[1];  span still → Colon
        Some(num)
    } else { None }   // <-- if peek_n(1) != Colon, buffer is NOT drained
} else { None };
// After this block, if ":Colon" was NOT peeked, buffer may have leftover tokens
// and span() may point to a token NOT being consumed
```

Additionally, `parse_header_line` calls `peek_n(2)` for `time` header parsing (`parser.rs:296`). While header offsets are less critical for normalization, any subsequent `next()` that returns from buffer after `peek_n(2)` will see a stale span.

**Severity assessment**: For the group path, after draining the `n:` prefix, the buffer is empty and subsequent peeks refill cleanly — so the *common* group path is safe. However, the *guard path* (integer followed by non-colon) leaves the buffer in an inconsistent state. And the header-time peek path could cause stale spans for any measure expressions parsed in the same parser session (though headers always precede measures, and `peek_n(2)` for time takes 3 tokens which are all consumed before control returns to `parse_header_line`, which calls `consume_newline()`).

**Recommendation**: Instead of relying on live `self.lexer.span()`, capture the span inside `next()` itself and return it alongside the token, or store it in the peek buffer entry. This is the only reliable approach when a lookahead buffer is used.

```
// Option: store span in buffer
peek_buf: Vec<(Token, Range<usize>)>,

fn next(&mut self) -> Result<(Token, u32), ParseError> {
    // ... returns token and span.start
}
```

This eliminates all race conditions between the lexer span and the peek buffer. Effort: ~30 lines.

---

#### 5. No task covers adding `offset` fields to parser AST nodes

The proposal says 7 AST node types gain `offset: u32` fields. But no task explicitly includes this work. The tasks:

| Task | Scope | AST offset fields mentioned? |
|------|-------|---------------------------|
| Task 4 | `event.rs` | No — only AC that NormalizedEvent carries `sourceOffset` |
| Task 7 | `normalize.rs`, `to_js.rs` | No — parser AST is not in scope |

Adding `offset` fields to `Document`, `HeaderSection`, `NoteExpr`, `MeasureSection`, `GroupExpr`, and `Barline` plus solving the `MeasureExpr` enum problem (Issue 1) is a meaningful chunk of parser work (~40–60 lines) that belongs in **Task 7's scope** (since the normalizer consumes these offsets). Alternatively, it should be a separate prerequisite task.

---

#### 6. `sourceOffset` semantics for compound nodes are undefined

The proposal says `NormalizedEvent.sourceOffset` is "carried forward from the parser token's offset." But which token for compound expressions?

| AST node | Which offset? | Ambiguity |
|----------|--------------|-----------|
| `BasicNote` | Glyph token start | Clear |
| `CombinedHit` | First glyph's offset? Last? | Undefined — `+` joins 2+ BasicNotes |
| `GroupExpr` | `[` offset? First item? | Undefined |
| `Crescendo` / `Decrescendo` | `<` / `>` offset | Clear |
| `HairpinEnd` | `!` offset | Clear |
| `NavMarker` / `NavJump` | `@` token offset | Clear |
| `MeasureRepeat` / `MultiRest` | `%` / `--` offset | Clear |
| `InlineBracedBlock` | `{` offset | Clear |

For `CombinedHit`, the normalizer emits **one** NormalizedEvent per hit (each glyph gets its own event). So each hit's event should carry its own glyph's offset. That means tracking per-glyph offsets within `CombinedHit`, not the CombinedHit's aggregate offset.

For `GroupExpr`, a `[3: x d b]` emits 3 NormalizedEvents. Should all carry the `[` offset, or should each carry the offset of `x`, `d`, `b` respectively? The proposal gives no guidance.

**Recommendation**: Each NormalizedEvent should carry the offset of its **originating basic glyph token** (same as `BasicNote.glyph` offset). For compound containers (Group, CombinedHit), the offset propagates to each sub-event from the sub-glyph's offset, not the container. Document this rule explicitly.

---

#### 7. `to_js` serialization specification is incomplete

The proposal (line 155):

> **to_js layer**: Offsets are serialized as the existing `SourceLocation { line, column, offset }` type that the TS pipeline already consumes.

The `SourceLocation` type (`types.ts:116-120`) is:

```ts
export type SourceLocation = {
  line: number;
  column: number;
  offset: number;
};
```

But the proposal says `NormalizedEvent` gains `sourceOffset: u32` — a single number. Then in `to_js`, this becomes a three-field `SourceLocation` object. This requires:
1. Rust must compute `line` and `column` from the offset (using `line_column()`, already available at `parser.rs:184`)
2. Rust must store the raw offset AND either (a) compute line/column at serialization time, or (b) compute line/column during normalization and store all three
3. The field name on the JS side is unclear: is it `sourceLocation` (the SourceLocation type) or `sourceOffset` expanded into fields?

No task addresses this `to_js` conversion logic.

**Recommendation**: Specify:
- The Rust `NormalizedEvent` carries `source_offset: u32` (byte offset)
- The Rust `NormalizedMeasure` carries `source_line: u32`
- `to_js.rs` for NormalizedEvent: calls `line_column(source_offset)` → emits `sourceLocation: { line, column, offset }` on the JS object
- `to_js.rs` for NormalizedMeasure: emits `sourceLine: number` (already consistent)
- Add this serialization logic to Task 7 scope (~20 lines of `to_js.rs`)

---

#### Verdict

The offset propagation concept is sound directionally, but four **blocking** implementation gaps prevent execution:

1. **MeasureExpr enum field** (Issue 1) — no approach specified; Rust won't allow this.
2. **`sourcePosition` phantom field** (Issue 2) — references a field that doesn't exist in the TS type.
3. **Parity claim vs. TS output** (Issue 3) — TS normalizer doesn't produce the new fields; byte-for-byte parity is impossible without patching the TS normalizer or rescoping the parity target.
4. **`self.lexer.span()` unreliability** (Issue 4) — lookahead buffer makes live span() calls non-deterministic; needs span-in-buffer fix.

The three non-blocking issues (5, 6, 7) are implementation-plan gaps that an experienced Rust developer could resolve, but they should be documented in the tasks to avoid re-discovery.

**STATUS: CHANGES_REQUESTED**

### Review Round 5

**Reviewer**: Critical Architect (sub-agent)
**Date**: 2026-05-10

**Scope**: Verify all 7 Round 4 issues resolved. Check only: (1) MeasureExprNode spec, (2) sourcePosition phantom, (3) parity scope, (4) span() with lookahead, (5) AST offset task, (6) compound node semantics, (7) to_js serialization.

---

#### 1. MeasureExprNode — PASS

Proposal line 163 specifies `MeasureExprNode { expr, offset }` wrapper. Tasks line 105 AC: "`MeasureExpr` wrapped in `MeasureExprNode { expr, offset }`". Tasks line 103 commit: "add offset tracking to AST nodes (Document, HeaderSection, NoteExpr, MeasureExprNode, GroupExpr, MeasureSection, Barline)". The enum-field problem is resolved.

#### 2. sourcePosition phantom field — MINOR RESIDUAL

The Design Notes (line 167) correctly state the TS type has only `sourceLine`, and tasks file (Task 7 line 116, Task 9 line 159) references only `sourceLine`. However, the uncorrected proposal body at **line 153** still reads: `NormalizedMeasure.sourcePosition is computed from the barline's offset.` This is a stale reference that contradicts the Design Notes at line 167. The tasks file is clean. Not blocking, but cleanup of line 153 is recommended for consistency.

#### 3. Parity scope — PASS

Proposal lines 157 and 169 scope parity to "shared fields" and explicitly exclude `sourceOffset` on events. Tasks file lines 149-161: "identical NormalizedScore on all fields present in the TS schema", "sourceOffset on NormalizedEvent is Rust-only — verified present and correct, not compared against TS". Scope is unambiguously correct.

#### 4. span() with lookahead — PASS (acceptable specification)

Proposal line 165: "read at the moment the token is consumed via `self.next()`, BEFORE the next `peek()`/`peek_n()` advances the lexer." The Round 4 concern was that live `self.lexer.span()` returns the last lexer-iterated token, which may be a lookahead token, not the consumed one. The design note correctly specifies the *desired behavior* (capture at consumption time). The *mechanism* (span-in-buffer, or span-returned-from-next) is an implementation detail. An experienced Rust developer implementing a peek buffer will naturally store spans alongside buffered tokens to satisfy this spec. The specification is sufficient for design approval; implementation is left to the implementer's judgment.

#### 5. AST offset task — PASS

Tasks file line 103 commit explicitly adds `feat(parser): add offset tracking to AST nodes`. Tasks file line 105 AC lists the specific types. The formerly missing task is present and correctly scoped within Task 7.

#### 6. Compound node semantics (sub-glyph offsets) — MINOR GAP

The `MeasureExprNode` wrap (line 163) specifies the container-trigger offset: "glyph for BasicNote, `[` for Group, `{` for BracedBlock." However, Round 4 Issue 6 asked specifically about per-glyph offsets within compound containers:

| Container | Question |
|-----------|----------|
| `GroupExpr([3: x d b])` | Should each of the 3 sub-events carry `[` offset or their own glyph offset? |
| `CombinedHit(x+c)` | Should `x` and `c` events carry the `+` join offset or the glyph offset? |

Round 4 recommended: "Each NormalizedEvent should carry the offset of its originating basic glyph token." The current proposal doesn't explicitly state that `GroupExpr.items` or `CombinedHit` sub-glyphs also have individually tracked offsets. An implementer reading the `MeasureExprNode` spec would naturally wrap sub-expressions in `MeasureExprNode` as well — but this is implicit, not explicit.

This issue was flagged as non-blocking in Round 4. If the implementer extends `MeasureExprNode` wrapping to nested expressions within containers, the per-glyph offset propagation follows naturally. No change is required for approval, but the author may wish to add a sentence clarifying that sub-glyphs within compound containers also receive their own `MeasureExprNode` wrapping.

#### 7. to_js serialization — MINOR GAP

Proposal line 155: "Offsets are serialized as the existing `SourceLocation { line, column, offset }` type." Tasks file line 102 has a commit for "expand to_js.rs for NormalizedScore → JsValue conversion," but no AC explicitly states the offset → SourceLocation conversion. Unresolved details:

1. **Field naming**: Will the JS-side field be named `sourceLocation` (three-field object) or `sourceOffset` (single number)? The `SourceLocation` type is referenced but the exact key name is not specified.
2. **line/column computation**: The proposal implies `line_column(source_offset)` is called during serialization, but does not state this explicitly. The existing `line_column()` function is available (`parser.rs:184`), so an implementer would find it.
3. **Scope attribution**: The ~20 lines for this conversion is not explicitly in any task AC; it is implied by the `to_js.rs` expansion commit.

This issue was non-blocking in Round 4. The design direction is clear enough that an implementer can resolve these details. No change required for approval.

---

#### Verdict

All four Round 4 **blocking** issues (1, 2, 3, 4) are materially resolved:

- **Issue 1** (MeasureExpr enum): Resolved via `MeasureExprNode` wrapper, specified in both proposal and tasks.
- **Issue 2** (sourcePosition phantom): Design Notes correctly reference the real field `sourceLine`. Stale text on line 153 is a cosmetic inconsistency (not blocking).
- **Issue 3** (parity vs. TS output): Parity correctly scoped to shared fields; `sourceOffset` explicitly excluded from comparison.
- **Issue 4** (lexer.span() unreliability): Capture timing specified. Mechanism left to implementation — acceptable for a design doc.

The three Round 4 **non-blocking** issues (5, 6, 7) are partially addressed:

- **Issue 5** (AST offset task): Fully resolved — Task 7 commit + AC now cover this.
- **Issue 6** (compound nodes): Container-level offsets specified via `MeasureExprNode`. Per-glyph offsets within containers are implicitly derivable but not explicitly stated. Non-blocking.
- **Issue 7** (to_js serialization): Direction clear, detail gaps (field naming, explicit line_column call). Non-blocking — implementer can resolve.

The design is implementable. No new blocking issues found.

**STATUS: APPROVED**
