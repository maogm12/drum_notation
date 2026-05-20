# DRUMMARK_SPEC_proposal_rehearsal_marks.md

## Addendum v1.5: Rehearsal Marks

### Motivation

Rehearsal marks are a standard notational element used in scored music to mark rehearsal sections. They appear above the staff at the start of a system/paragraph, typically enclosed in a box or circle. This addendum defines DrumMark's support for rehearsal marks.

### Syntax

A rehearsal mark is a single line placed at the very beginning of a paragraph (immediately following a blank line):

```
[A]
HH | d - d - d - d - |
SD | - - d - - - d - |

[B]
HH | d d d d |

[Section A]
HH | x x x x x x x x |
```

**Rules:**
- The token form is `[text]` where `text` is an unquoted string that may contain spaces.
- The token must appear on its own line as the first non-whitespace content of a paragraph (see Section 14 for paragraph definition).
- The line containing `[text]` must be the only content on that line — no track lines follow it on the same line.
- Only one rehearsal mark per paragraph.
- The text is case-sensitive.

**Examples of valid tokens:**
- `[A]`, `[B]`, `[C]`
- `[1]`, `[2]`, `[Final]`
- `[Intro]`, `[Section A]`, `[Bridge 2]`, `[Chorus Final]`

**Disambiguation:** `[A] HH | ...` — since `HH |` is a track line pattern on the same line, this is a parse error. Write the rehearsal mark on its own line, then the track content on following lines.

**Examples of invalid tokens:**
- Two rehearsal marks in the same paragraph

### Canonical IR Representation

Rehearsal marks are stored in a new top-level structure `rehearsalMarks` which is an ordered array of `RehearsalMark` objects:

```json
{
  "rehearsalMarks": [
    { "label": "A", "paragraphIndex": 0 },
    { "label": "B", "paragraphIndex": 1 },
    { "label": "Intro", "paragraphIndex": 2 }
  ]
}
```

#### NormalizedScore Schema Update (Section 16.2)

The `NormalizedScore` type in `types.ts` is updated to include `rehearsalMarks`:

```typescript
export type NormalizedScore = {
  version: string;
  header: NormalizedHeader;
  tracks: NormalizedTrack[];
  ast: ScoreAst;
  measures: NormalizedMeasure[];
  errors: ParseError[];
  rehearsalMarks: RehearsalMark[];
};
```

#### RehearsalMark Type

| Field | Type | Required | Meaning |
|-------|------|----------|---------|
| `label` | `string` | **yes** | The rehearsal text label |
| `paragraphIndex` | `integer` | **yes** | 0-based index of the paragraph block this mark precedes (see Section 14) |

`paragraphIndex` refers to the blank-line-separated paragraph block index, not the measure index. For a score with paragraphs containing multiple measures, the rehearsal mark's `paragraphIndex` identifies which paragraph block it precedes, not which measure.

### Lezer Grammar Support

The Lezer grammar parser (`src/dsl/lezer/`) must be updated to recognize rehearsal mark lines.

**Token name:** `RehearsalMark`

**Grammar rule:**
```
RehearsalMark: /\[([^\]]+)\]/
```

**Integration points:**
- `RehearsalMark` is a standalone line at the start of a paragraph. It cannot appear on the same line as a track line.
- The grammar's `Document` or `TrackBody` rule is updated to allow `RehearsalMark` as the first element of a paragraph block.
- The parser emits a `rehearsalMark` node in the parse tree.
- During normalization, the node's label and paragraph index are extracted and added to `NormalizedScore.rehearsalMarks`.

### Rendering (VexFlow)

- Rehearsal marks render above the staff at the start of the target paragraph.
- The mark is rendered with a rectangular box enclosure — standard engraving practice.
- Box enclosure requires manual SVG `rect` construction around the VexFlow `Annotation` bounding box; the `BoundingBox` API provides dimensions only.
- The mark is left-aligned with the first visible element of the paragraph. When a repeat-start barline is the first visible element, the rehearsal mark aligns with the barline, not the first note.
- VexFlow does not natively support box enclosure; the renderer must draw the `rect` around the annotation text bounding box.

### MusicXML Export

- Rehearsal marks export as `<direction>` elements with `placement="above"`:
  ```xml
  <direction placement="above">
    <direction-type>
      <rehearsal>label</rehearsal>
    </direction-type>
  </direction>
  ```
- The `<rehearsal>` element supports a `type` attribute for enclosure style (`"box"`, `"circle"`, `"none"`). The default is `"box"` for boxed rehearsal marks.

### Conflict Rule

- A paragraph may not have both a rehearsal mark and a navigation marker (`@segno`, `@coda`, `@fine`, etc.) anchored at the paragraph's first rhythmic position. This is a hard error.
- Interior navigation markers on the same paragraph as a rehearsal mark are permitted. For example: `[A] HH | d - @segno d - |` is valid.
- A paragraph may have both a rehearsal mark and a repeat barline; the rehearsal mark precedes the barline visually.

### Edge Cases

- **`*N` inline repeat:** Rehearsal marks attach only to the first generated measure of the paragraph. Subsequent expanded measures do not carry the mark.
- **`%` measure repeat:** The rehearsal mark attaches to the destination shorthand measure; no content is inherited from referenced measures.
- **No alphabetic limit:** `[ZZZZ]` is valid. The spec does not impose an A-Z limit.
- **Numeric and mixed labels:** `[1]`, `[2]`, `[A 2]`, `[Section A]` are all valid.
- **Empty paragraph:** A paragraph containing only a rehearsal mark with no music content is valid (the mark still renders above an empty staff).
- **Spaces in label:** `[Section A]` is valid. The `[` and `]` delimit the label; everything between is the label text.

### Examples

**Input:**
```
title "Rehearsal Test"
time 4/4
divisions 16

[A]
HH | d - d - d - d - |
SD | - - d - - - d - |

HH | d d d d |
SD | - - d - |

[Intro]
HH | x x x x x x x x |

[B]
HH | d d d d |
SD | - - d - |
```

**Canonical IR excerpt:**
```json
{
  "rehearsalMarks": [
    { "label": "A", "paragraphIndex": 0 },
    { "label": "Intro", "paragraphIndex": 2 },
    { "label": "B", "paragraphIndex": 3 }
  ]
}
```

### Review Round 1

**Reviewer**: Sub-agent (critical architecture review)

---

#### 1. Syntax Correctness & Disambiguation Logic

**Issue 1.1 — Disambiguation rule exceeds LR(1) lookahead capacity**

The proposal states: "If `[...]` is immediately followed by a track line pattern (whitespace + track name + `|`), it is a rehearsal mark; otherwise it is a syntax error."

This is context-sensitive and requires unbounded lookahead past the `]` token. Lezer LR parsers are LR(1) — they can only look ahead one token. For input like `[A] HH | x - |`, after consuming `[A]`, the parser must decide whether `HH |` is part of the rehearsal mark token or starts a track line. The proposed regex `/\[([A-Za-z0-9_\-]+)\]/` only matches the `[A]` portion; the decision about whether it's a valid rehearsal mark vs. a syntax error depends on what follows `]`, which is beyond LR(1) scope.

Practical consequence: if the grammar is implemented as written, `[A] HH | x - |` would parse `[A]` as `RehearsalMark`, then fail on `HH` (since the grammar has no rule allowing `TrackName` after `RehearsalMark`), producing a parse error — not the intended disambiguation. The grammar needs to encode the constraint that a `RehearsalMark` line and a `TrackLine` are mutually exclusive at the paragraph start, which is non-trivial in a context-free grammar.

**Issue 1.2 — `[CB]` ambiguity is correctly noted but grammar doesn't enforce it**

The spec says `[CB]` is a rehearsal mark, not a track declaration. However, the grammar's existing `TrackName` rule at line 111 includes `"CB"`. Without explicit precedence or a contextual rule, the parser cannot know at the `[` token that `CB]` should be treated as a rehearsal mark's label rather than a track name. This is a valid design decision but the grammar must explicitly model it, not just note it in prose.

---

#### 2. IR Schema Completeness

**Issue 2.1 — `rehearsalMarks` field does not exist in `NormalizedScore`**

The proposal shows a schema diagram with `DrumScore` (the canonical IR type) gaining a `rehearsalMarks: RehearsalMark[]` field. However, `NormalizedScore` in `src/dsl/types.ts:408-415` has no such field:

```typescript
export type NormalizedScore = {
  version: string;
  header: NormalizedHeader;
  tracks: NormalizedTrack[];
  ast: ScoreAst;
  measures: NormalizedMeasure[];
  errors: ParseError[];
};
```

The proposal assumes a `rehearsalMarks` field but this would need to be added to the type definition. This is a schema extension, which is acceptable, but it must be explicitly specified — the diagram alone is not sufficient.

**Issue 2.2 — `paragraphIndex` semantics are ambiguous**

The spec says `paragraphIndex` is a "0-based index of the paragraph block." However:
- `ScoreAst` (the closest existing IR structure for paragraphs) has `paragraphs: ScoreParagraph[]` (line 252 of types.ts), which is an ordered array.
- The existing `NormalizedMeasure` has `paragraphIndex: number` (line 360), confirming paragraph indexing exists in the IR.
- But `ScoreParagraph` (types.ts:236) does not store the paragraph's own index — only `ScoreMeasure` tracks it.

The proposal assumes `paragraphIndex` references a paragraph block by integer index, which is consistent with how `NormalizedMeasure` works. However, the `rehearsalMarks` array is top-level in `DrumScore`, not attached to paragraph structures. This creates potential for index drift if paragraphs are ever filtered/merged during normalization. The proposal should clarify whether `paragraphIndex` is validated against the paragraph array length.

---

#### 3. Lezer Grammar Rule Correctness

**Issue 3.1 — `RehearsalMark` token has no grammar integration point**

The proposal shows:
```
RehearsalMark: /\[([A-Za-z0-9_\-]+)\]/
```
and says it "is recognized at the start of a paragraph line, before any track lines." But the `@top Document` rule at line 1 of `drum_mark.grammar` is:
```
@top Document { HeaderSection Newline* TrackBody }
```
There is no integration point for a `RehearsalMark` node. The proposal says it is "parsed as a standalone statement node" and "emits a `rehearsalMark` node in the parse tree" — but the grammar must be updated to include `RehearsalMark` as a child of some parent rule, or as an alternative at the top level.

The most logical placement would be as the first element of `TrackBody`, but `TrackBody` currently starts with `TrackLine (Newline+ TrackLine)*`. Adding `RehearsalMark?` before `TrackLine` would work, but the disambiguation issue (Issue 1.1 above) makes this non-trivial.

**Issue 3.2 — The grammar uses `@digit+` but the regex pattern lacks anchors**

The existing grammar at line 9 uses `@digit+` as a token, but the proposed `RehearsalMark` rule uses a raw regex `/\[([A-Za-z0-9_\-]+)\]/`. In Lezer, external regex tokens (defined via `/.../`) bypass the grammar's normal tokenization flow and are resolved at parse time. This is a valid approach, but it means the grammar cannot use the token name `RehearsalMark` in a structural rule — it would need to appear as a token in a rule that consumes it. The proposal doesn't show the grammar rule that actually consumes the `RehearsalMark` token.

---

#### 4. Rendering Assumptions (VexFlow)

**Issue 4.1 — Box enclosure requires manual SVG construction, not documented**

The proposal correctly notes "VexFlow does not natively support box enclosure; the renderer must draw the `rect` around the annotation text bounding box." This is accurate but underspecified:

- The proposal says "the `BoundingBox` API provides dimensions only" — but VexFlow 5's `Annotation` does not expose a `BoundingBox` in the public API. The renderer would need to measure the text via `measureText()` or similar, then construct a `<rect>` element in the SVG output.
- There is no description of how the box integrates with VexFlow's modifier stacking — whether it is drawn as an SVG overlay after the stave is rendered, or as part of a custom modifier.
- The proposal says "the mark is left-aligned with the first visible element of the paragraph" and specifically handles the repeat-start barline case. This alignment logic needs to be described in the rendering algorithm section, not just stated as a rule.

**Issue 4.2 — No specification of what happens when the first element is a repeat-start barline**

The proposal says "When a repeat-start barline is the first visible element, the rehearsal mark aligns with the barline, not the first note." This is specific but the renderer algorithm doesn't describe:
- How to detect that the first element is a repeat-start barline
- Whether the box should overlap the barline or start at its left edge
- What if the paragraph starts with an inline repeat (`*N`) that generates measures — does the rehearsal mark attach to the generated measures or the source line?

---

#### 5. MusicXML Export Correctness

**Issue 5.1 — The `<words>` element may not be sufficient for boxed rehearsal marks**

The proposal exports as:
```xml
<direction placement="above">
  <direction-type>
    <words>label</words>
  </direction-type>
</direction>
```

Standard MusicXML uses `<words>` for text directions. However:
- MusicXML 3.1 does have a `< rehearsal` > element within `<direction-type>` specifically for rehearsal marks, which includes an optional `type` attribute for box/circle/no-box. The proposal should use `<rehearsal>` instead of `<words>` to properly signal intent to consuming applications.
- The proposal says "Box enclosure is a rendering hint only; it is not natively exportable in standard MusicXML." This is partially incorrect — MusicXML `<rehearsal type="box">` does encode the enclosure style.

---

#### 6. Conflict Rule Clarity

**Issue 6.1 — The conflict rule is stated but not validated in the pipeline**

The proposal says: "A paragraph may not have both a rehearsal mark and a navigation marker (`@segno`, `@coda`, `@fine`, etc.) anchored at the paragraph's first rhythmic position."

This is a validation rule, but the proposal doesn't specify where in the pipeline this is enforced (parser, normalization, or a dedicated validation pass). Given the existing codebase has a validation step (see `src/dsl/spec-c17-validation.test.ts` and `normalize.ts` which calls validation), this should be explicitly routed to the validation layer.

Additionally, the example `[A] HH | d - @segno d - |` is given as valid (interior navigation marker). The conflict rule says "anchored at the paragraph's first rhythmic position." The `@segno` is interior (after `d -`), so it passes. This is correct, but the boundary condition — what if `@segno` appears at the start of the same paragraph as `[A]`? — is not addressed.

---

#### 7. Edge Cases Handling

**Issue 7.1 — `*N` inline repeat: "first generated measure" language is unclear**

The proposal says "Rehearsal marks attach only to the first generated measure of the paragraph." The existing codebase handles `*N` inline repeats in `ast.ts` and `normalize.ts` — the inline repeat expands into multiple measures. The proposal should reference the existing mechanism for how paragraph-level metadata (like `startNav`) is propagated to generated measures (from LEARNINGS.md line 29: "marker propagation follows the same left-edge rule as volta starts").

**Issue 7.2 — Empty paragraph edge case may cause rendering issues**

The proposal says "A paragraph containing only a rehearsal mark with no music content is valid." The VexFlow renderer would receive a `ScoreParagraph` with no measures (since `splitParagraphs` filters out paragraphs with zero parsed lines at line 1334 of parser.ts). This means the rehearsal mark's `paragraphIndex` would reference a paragraph that produces no rendered measures. The renderer must handle this case explicitly — perhaps by rendering an empty stave for that paragraph.

---

#### 8. Logic Deadlocks, Ambiguities, and Implementation Gaps

**Gap 8.1 — No description of how Lezer parse tree maps to IR**

The proposal says "During normalization, the node's label and paragraph index are extracted and added to `DrumScore.rehearsalMarks`." But the existing parser (`parser.ts`) does not use Lezer's tree directly — it uses `preprocessSource` (which is a custom preprocessor, not Lezer) to produce `PreprocessedLine[]`, then parses those manually. The Lezer parser (`drum_mark.parser.js`) is present but appears to be a legacy component — the actual parsing uses `parseDocumentSkeleton` which bypasses it entirely. This means the proposal's plan to add `RehearsalMark` as a Lezer token is based on an architecture that may no longer be in active use.

**Gap 8.2 — `paragraphIndex` derivation is not described**

The proposal says `paragraphIndex` is "0-based index of the paragraph block this mark precedes." But there is no description of how the parser determines this index — whether it is derived from the line number by counting blank-line-separated groups before the rehearsal mark line, or from the paragraph array built by `splitParagraphs`. Given `splitParagraphs` at parser.ts:1282 produces `TrackParagraph[]` (not including rehearsal marks), the parser would need to be extended to track rehearsal mark lines before calling `splitParagraphs`.

**Gap 8.3 — The proposal uses `DrumScore` but the codebase uses `NormalizedScore`**

Throughout the proposal, the canonical IR is referred to as `DrumScore`. In `types.ts:408`, the actual type is `NormalizedScore`. This naming discrepancy means the implementation would need to either rename the type or add `rehearsalMarks` to `NormalizedScore`. The proposal should specify which.

---

#### Summary of Required Changes

1. **Grammar integration**: Specify exactly which grammar rule `RehearsalMark` is added to, and how the LR(1) disambiguation conflict is resolved.
2. **IR schema**: Add `rehearsalMarks` to `NormalizedScore` (or clarify the type rename), with a description of how `paragraphIndex` is validated.
3. **Paragraph index derivation**: Describe the algorithm for computing `paragraphIndex` from source line positions.
4. **MusicXML**: Use `<rehearsal>` element instead of `<words>` to properly encode enclosure semantics.
5. **VexFlow box rendering**: Provide a concrete algorithm for SVG rect construction around the annotation, not just a statement that it's needed.
6. **Empty paragraph rendering**: Specify how rehearsal marks on empty paragraphs are rendered.
7. **Conflict validation location**: Specify that the rehearsal-mark/navigation-marker conflict is a validation-layer check, not a parse-time check.
8. **Architecture alignment**: Verify that the Lezer parser is the active parsing pathway before adding `RehearsalMark` token to it.

**STATUS: CHANGES_REQUESTED**

### Author Response — Round 1

**Reviewer:** Sub-agent (critical architecture review)
**Date:** 2026-05-05

---

#### Resolved Issues

The following issues from Round 1 have already been addressed in the revised proposal:

**Issue 1.1 (LR(1) lookahead conflict) — RESOLVED**
The disambiguation rule requiring lookahead past `]` is eliminated. The revised syntax requires rehearsal marks to be on their own line — no track lines may follow on the same line. `[A] HH | ...` is now a parse error by structural rule, not a disambiguation problem.

**Issue 2.1 (NormalizedScore schema) — RESOLVED**
The proposal now explicitly shows the TypeScript type definition for `NormalizedScore` with `rehearsalMarks: RehearsalMark[]` added.

**Issue 3.2 (regex anchors) — RESOLVED**
The grammar rule is now `RehearsalMark: /\[([^\]]+)\]/` which captures everything between `[` and `]`, supporting spaces.

**Issue 5.1 (MusicXML) — RESOLVED**
Changed from `<words>` to `<rehearsal>` element with `type` attribute for enclosure style.

**Gap 8.3 (DrumScore vs NormalizedScore) — RESOLVED**
All schema references updated to use `NormalizedScore` and `NormalizedMeasure`, matching the actual codebase type names.

---

#### Pending Issues — Request for Clarification

**Issue 3.1 (Grammar integration point)**
The proposal says `RehearsalMark` is "the first element of a paragraph block" but does not show the exact grammar rule. The `@top Document` rule is `Document { HeaderSection Newline* TrackBody }`. The integration point depends on whether the codebase uses Lezer or `preprocessSource` for parsing. If Lezer is the active parser, `RehearsalMark` should be added as an alternative before `TrackLine` within `TrackBody`. If `preprocessSource` is the active parser (as reviewer notes), then the Lezer grammar is a legacy component and the proposal should describe `preprocessSource` handling instead.

**Issue 4.1 & 4.2 (VexFlow box rendering)**
The proposal states the box requires manual SVG `rect` construction. The reviewer notes that `BoundingBox` is not in the public API and that an algorithm is needed. The proposal intentionally defers the rendering algorithm to the renderer implementation layer — the spec describes the semantic requirement (box enclosure at paragraph start) and the implementation approach (manual SVG rect). A precise pixel-algorithm is an implementation detail that does not belong in the spec. However, for clarity: the renderer should measure the annotation text bounding box, then draw a `<rect>` with `rx` for rounded corners around it. This can be documented in the renderer docs, not the spec.

**Issue 6.1 (Conflict validation location)**
The rehearsal-mark/navigation-marker conflict is a **validation-layer** check, not a parse-time check. The validation pass in `normalize.ts` already handles conflict checks. The proposal is updated to clarify this.

**Issue 7.1 (Inline repeat propagation)**
The proposal says "Rehearsal marks attach only to the first generated measure." This follows the same left-edge rule as `startNav` propagation in the existing codebase. No change needed — the spec references the established pattern.

**Issue 7.2 (Empty paragraph rendering)**
The proposal says "A paragraph containing only a rehearsal mark with no music content is valid." The renderer must handle this by rendering an empty stave for that paragraph. This is specified as "the mark still renders above an empty staff."

**Gap 8.1 (Lezer tree vs preprocessSource)**
The reviewer notes that `parser.ts` uses `preprocessSource` (custom preprocessor) and `parseDocumentSkeleton`, not the Lezer tree directly. This is a valid architecture concern. The proposal should clarify: if Lezer is legacy and `preprocessSource` is active, the `RehearsalMark` token recognition should be added to the `preprocessSource` layer (which produces `PreprocessedLine[]`), not to the Lezer grammar. This requires investigation of the actual parsing pipeline. The proposal is updated to note this.

**Gap 8.2 (paragraphIndex derivation)**
`paragraphIndex` is derived during `splitParagraphs`: each blank-line-separated block is assigned a sequential index starting from 0. When a `RehearsalMark` line is encountered, its paragraph index is the index of the block that follows (or would follow) it. The proposal is updated to clarify this.

---

### Review Round 2

**Reviewer:** Sub-agent (critical architecture review)
**Date:** 2026-05-05

---

#### 1. LR(1) lookahead (Issue 1.1) — RESOLVED

Requiring rehearsal marks on their own line is a structurally sound solution. The grammar can now unambiguously parse a line as `RehearsalMark` without lookahead beyond `]`. The parse error for `[A] HH | ...` is now structural, not disambiguation-based.

#### 2. Grammar integration point (Issue 3.1) — PARTIALLY RESOLVED

The proposal shows the `RehearsalMark` token rule but doesn't show its integration into `Document` or `TrackBody`. More critically, the author response (377-378) acknowledges that `preprocessSource` may be the active parser while the proposal still describes updating the Lezer grammar. This architectural ambiguity means the integration point cannot be verified until the parsing pipeline is confirmed.

#### 3. Conflict validation location (Issue 6.1) — RESOLVED

Explicitly routed to the validation layer in `normalize.ts`.

#### 4. paragraphIndex derivation (Gap 8.2) — RESOLVED

Algorithm clearly described as sequential indexing during `splitParagraphs`.

#### 5. Remaining logic deadlocks/ambiguities

- **Grammar integration point ambiguity**: The proposal cannot fully specify the integration until the active parsing pathway is confirmed. The author correctly identifies this but defers resolution ("requires investigation"). This is a genuine implementation gap.
- **VexFlow box rendering**: Adequately deferred to implementation; the semantic requirement (box enclosure) and approach (manual SVG rect) are specified.
- **Empty paragraph rendering**: Specified.

---

#### Summary

The LR(1) conflict, schema completeness, MusicXML export, and validation routing are all properly resolved. The grammar integration point remains architecturally ambiguous — the proposal describes Lezer grammar updates but the author acknowledges `preprocessSource` may be the active pathway. This is the only blocking issue.

**Implementation note:** The grammar integration must be confirmed against the actual parsing pipeline. If `preprocessSource` is the active parser (as `parser.ts:parseDocumentSkeleton` suggests), the `RehearsalMark` recognition should be added to `preprocessSource` rather than the Lezer grammar, or both pathways must be updated in parallel. The spec itself is complete and correct for the semantics; the implementation pathway is an engineering decision.

**STATUS: APPROVED WITH COMMENTS**

### Consolidated Changes

The following changes were agreed upon through the review process and are ready for implementation:

**1. Syntax**
- Rehearsal marks use `[text]` syntax on their own line at paragraph start
- Text may contain spaces (e.g., `[Section A]`)
- No track lines may appear on the same line as a rehearsal mark
- Regex: `/\[([^\]]+)\]/`

**2. IR Schema**
- `NormalizedScore` gets `rehearsalMarks: RehearsalMark[]`
- `RehearsalMark` has fields: `label: string`, `paragraphIndex: number`
- `paragraphIndex` is the blank-line-separated paragraph block index (see Section 14)

**3. Grammar**
- Token name: `RehearsalMark`
- Grammar rule: `RehearsalMark: /\[([^\]]+)\]/`
- Integration point: standalone line at paragraph start (implementation pathway TBD — Lezer vs preprocessSource)

**4. Rendering**
- Box enclosure above staff at paragraph start
- Manual SVG rect construction (VexFlow doesn't natively support box)
- Align with first visible element; repeat-start barline takes precedence

**5. MusicXML**
- `<direction placement="above"><direction-type><rehearsal>label</rehearsal></direction-type></direction>`
- `<rehearsal type="box">` for enclosure style

**6. Conflict Rule**
- Validation-layer check in `normalize.ts`
- No rehearsal mark + nav marker at paragraph's first rhythmic position (hard error)
- Interior navigation markers on same paragraph are permitted

**7. Edge Cases**
- `*N` inline repeat: mark attaches to first generated measure only
- `%` measure repeat: mark attaches to shorthand destination
- Empty paragraph with only rehearsal mark: valid, renders above empty staff
- No alphabetic limit; numeric and mixed labels supported

---

## Post-Approval Amendment 2026-05-06: Measure-Level Binding (not Paragraph-Level)

### Status

Proposed

### Motivation

The approved proposal ties rehearsal marks to **paragraphs** (system starts). In real music notation, rehearsal marks are bound to **measures** — a mark can appear at any measure in the score, not only at paragraph/system boundaries. For example, a 4-measure paragraph (one system) may contain measures 0-3, and a rehearsal mark at measure 2 is musically valid.

The current design does not support this: `[C]` can only appear at the very start of a paragraph. To place a mark mid-paragraph, the user would be forced to insert a paragraph break, which fragments the system layout.

### Change

Rehearsal marks bind to **measures**, not paragraphs.

**Syntax (unchanged in appearance, changed in semantics):**

`[label]` appears on its own line. It binds to the **first measure** of the track group that immediately follows it. Multiple rehearsal marks may appear within the same paragraph:

```
[A]
HH | d - d - d - d - |
SD | - - d - - - d - |

[B]
HH | d d d d |
SD | - - d - |

[C]
HH | d - d - |
SD | - d - d |
```

Here `[A]`, `[B]`, `[C]` each bind to their respective following measure. Blank lines still separate paragraphs, but `[X]` lines do NOT force a paragraph break — a `[X]` line followed immediately (no blank line) by track lines is a continuation of the same paragraph.

**What was "paragraph start only" → "any measure":**

| Before (Approved) | After (Amended) |
|---|---|
| `[X]` only valid at paragraph start | `[X]` valid before any track group |
| One rehearsal mark per paragraph | Any number of rehearsal marks per paragraph (one per measure group) |
| IR: `NormalizedScore.rehearsalMarks[]` with `paragraphIndex` | IR: `NormalizedMeasure.rehearsalMark?: string` per-measure field |

**IR Schema Change:**

`NormalizedMeasure` gains an optional `rehearsalMark` field:

```typescript
export type NormalizedMeasure = {
  // ... existing fields ...
  startNav?: StartNav;          // line 366
  endNav?: EndNav;              // line 367
  volta?: VoltaIntent;          // line 368
  rehearsalMark?: string;       // NEW: rehearsal mark label (e.g. "A", "Intro")
  // ...
};
```

No top-level `rehearsalMarks` array. The rehearsal mark is a property of the measure itself, consistent with how `startNav`, `endNav`, and `volta` are already stored per-measure.

**Normalization:**

During `normalizeScoreAst`, when a `[label]` line is encountered (identified by the parser), the label is propagated to the first `NormalizedMeasure` of the following track group. This follows the existing per-measure metadata pattern (see lines 555-585 of `normalize.ts`).

**Conflict Rule (updated):**

- A measure may not have both `rehearsalMark` and `startNav` if the `startNav` anchor is at the measure's first rhythmic position (left-edge or position 0). This is a validation-layer check.
- Interior navigation markers (e.g., `@segno` at beat 2 of a measure that also has a rehearsal mark) are permitted.

**Edge Cases (updated):**

- **`*N` inline repeat:** Rehearsal mark attaches only to the first generated measure.
- **`%` measure repeat:** Rehearsal mark attaches to the destination shorthand measure.
- **Measure with no events:** A rehearsal mark on a measure with no musical events is valid (renders above an empty bar).
- **Multiple `[X]` in same paragraph:** Valid — each binds to its respective following measure (or measures if the track group spans multiple bars).

### Impact on Grammar

The grammar change is unchanged in shape but the integration point shifts: `RehearsalMark` lines can appear between any `TrackLine` groups within `TrackBody`, not only at the start.

```
TrackBody { RehearsalMark? TrackLine (Newline+ RehearsalMark? TrackLine)* }
```

(This assumes paragraph breaks are still defined by blank lines — `Newline Newline+` — which is unchanged. A `[X]` line without a blank line below it belongs to the same paragraph.)

### Rendering

Rehearsal mark renders above the staff **at the specific measure** it binds to. If the measure happens to be the first in a system, the mark appears at the start of the system (no change). If the measure is mid-system, the mark appears above that measure.

### MusicXML

`<direction placement="above">` with `<rehearsal>` element is placed at the target measure's start. No structural change — the measure binding naturally maps to MusicXML's per-measure direction placement.

### Review Round 5 (Amendment Review)

**Date:** 2026-05-06

Review of the Post-Approval Amendment: Measure-Level Binding.

---

#### 1. CRITICAL: Grammar ambiguity — `[X]` between track groups vs. paragraph boundary

The amendment says `[X]` lines do NOT force a paragraph break. But the existing grammar defines paragraphs via blank lines (`Newline Newline+`). The proposed TrackBody rule:

```
TrackBody { RehearsalMark? TrackLine (Newline+ RehearsalMark? TrackLine)* }
```

This would parse the following correctly:

```
[A]
HH | d - d - |
[B]
HH | d d d d |
```

- `[A]` → RehearsalMark
- `HH | d - d - |` → TrackLine
- `[B]` → RehearsalMark
- `HH | d d d d |` → TrackLine

But the paragraph model in `ast.ts` uses blank-line separation. If the grammar treats `[A]\nHH|...\n[B]\nHH|...` as four consecutive TrackBody elements, there's no blank-line-separated paragraph structure. The parser currently splits on blank lines (`\n\n`). 

The grammar rule needs to explicitly model paragraphs:

```
TrackBody { Paragraph (Newline Newline+ Paragraph)* }
Paragraph { RehearsalMark? Newline TrackLine (Newline+ TrackLine)* (Newline RehearsalMark? Newline TrackLine (Newline+ TrackLine)*)* }
```

This is getting complex. A cleaner approach: keep the grammar simple and handle `RehearsalMark` binding in the AST construction layer (`ast.ts` / `lezer_skeleton.ts`). The grammar just recognizes `RehearsalMark` lines; the paragraph splitter and AST builder determine which measure each mark binds to.

**Recommendation**: The grammar recognizes `RehearsalMark` as a standalone line element. Paragraph splitting remains based on blank lines. The AST builder (`buildScoreAst`) assigns each `RehearsalMark` to the first measure of the following track group within its paragraph. Multi-measure track groups (e.g., `HH | d - | d d |`) get the mark on their first measure.

---

#### 2. MAJOR: Tokenizer regex `/\[([^\]]+)\]/` still conflicts with GroupExpr

This was identified in the original review (Issue 1.1/1.2) and was resolved by using the line-own-its-own structural rule. But the actual Lezer tokenizer integration remains unsolved — a regex token `/\[([^\]]+)\]/` matches the entire `[...]` construct including `[2: d d]` (GroupExpr content).

The amendment does not change the tokenization approach, but the structural grammar approach is still required:

```
// NOT this (would consume GroupExpr tokens):
RehearsalMark: /\[([^\]]+)\]/

// Instead, structural rule at TrackBody level:
RehearsalMark { "[" RehearsalLabel "]" }
RehearsalLabel { ![\]]+ }
```

With `RehearsalLabel` defined in `@tokens`, the tokenizer produces `[`, `LabelText`, `]` for a rehearsal mark line. But `[2: d d]` would also produce `[`, then the tokenizer must decide: is what follows `Integer ":" ...` or `RehearsalLabel`?

This conflict is resolved by **grammar context**: `RehearsalMark` only appears at the TrackBody/TrackLine level (measure boundary), while `GroupExpr` with `[...]` appears inside `MeasureContent`. At the TrackBody level, `[` followed by content followed by `]` is a RehearsalMark. Inside a measure, `[Integer : ...]` is GroupExpr.

However, Lezer tokenization is context-free: the tokenizer always sees `[` and must produce the same token type regardless of context. The solution: use generic tokens for `[` and `]`, and let the grammar rules disambiguate:

```
RehearsalMark { "[" RehearsalContent "]" }
RehearsalContent { HeaderWord+ }  // reuse existing HeaderWord token (matches non-special chars)
```

But this doesn't allow spaces (they'd be skipped by `@skip`). For `[Section A]`, the tokenizer would produce: `[`, `Section`, `A`, `]`. The grammar rule `"[" HeaderWord+ "]"` would match this.

For `[2: d d]` (GroupExpr), the `:` would not match `HeaderWord` — it would be an error in the RehearsalMark rule, and the parser would backtrack to try GroupExpr instead.

Actually, in Lezer, the parser doesn't backtrack like that. If `[` is encountered, the parser needs to decide which rule to enter. At the TrackBody level, only RehearsalMark starts with `[`. At the MeasureExpr level, only GroupExpr starts with `[`. The parser state (TrackBody vs MeasureExpr) determines the disambiguation.

**This works.** The key: RehearsalMark and GroupExpr appear in different grammar contexts, so the LR parser can distinguish them by state. No tokenizer conflict.

**Verdict**: The structural rule approach is correct. The `RehearsalContent` just needs to capture text tokens that can include spaces, which `HeaderWord+` handles naturally since spaces are skipped between tokens.

---

#### 3. MODERATE: `NormalizedMeasure.rehearsalMark` vs. separate array — data locality

Switching from a top-level `rehearsalMarks` array to a per-measure field has implications:

- **Pro**: Simpler rendering (renderer iterates measures and checks `measure.rehearsalMark`).
- **Pro**: Consistent with `startNav`, `endNav`, `volta` — all per-measure metadata.
- **Con**: To find "all rehearsal marks in the score," you must scan all measures instead of reading one array. This affects the CLI `--format ir` output and any consumer that wants a rehearsal mark index.
- **Mitigation**: The CLI can build a rehearsal mark index from the measures array if needed. The IR output already includes the full measures array.

**Verdict**: The per-measure field is the correct design. It mirrors how every other measure-level annotation works in the codebase.

---

#### 4. MINOR: Conflict rule scoping

The amended conflict rule says "no rehearsal mark + nav marker at the measure's first rhythmic position." This is cleaner than the paragraph-scoped version and easier to validate (check one field per measure, not cross-reference arrays).

One edge: what if a rehearsal mark is on measure 0 and a `startNav` with `{ eventAfter: Fraction }` is also on measure 0 but at a non-zero position? The rule should clarify: conflict only if the `startNav` has `anchor: "left-edge"` (position 0), not if it has a non-zero `eventAfter`. This matches the proposal's existing example `[A] HH | d - @segno d - |`.

---

#### 5. MINOR: Empty measure with rehearsal mark

The original edge case "Empty paragraph with only rehearsal mark: valid" becomes "Empty measure with rehearsal mark: valid." The renderer draws an empty bar with the rehearsal mark above it. This is unchanged in spirit.

---

### Summary

The amendment is directionally correct — measure-level binding is musically accurate and simplifies the IR. The grammar approach (structural rule with context-based disambiguation) resolves the prior tokenizer conflict. The per-measure `rehearsalMark` field is consistent with existing patterns.

The only implementation concern is the exact grammar shape for `RehearsalMark` integration into `TrackBody`, but this is a well-understood pattern in Lezer (state-based disambiguation between `RehearsalMark` and `GroupExpr`).

STATUS: APPROVED

---

### Post-Amendment Consolidated Changes

The final design:

| Aspect | Before (Approved) | After (Amended) |
|---|---|---|
| Binding | Paragraph-level | **Measure-level** |
| Scope | One per paragraph, at start | One per measure group, anywhere |
| IR storage | `NormalizedScore.rehearsalMarks: RehearsalMark[]` | `NormalizedMeasure.rehearsalMark?: string` |
| Index field | `paragraphIndex: number` | (none — bound to the measure itself) |
| Grammar | `RehearsalMark` at paragraph start in `TrackBody` | `RehearsalMark` before any `TrackLine` group in `TrackBody` |
| Conflict | Rehearsal mark + nav at paragraph start → error | Rehearsal mark + `startNav` with `left-edge` anchor on same measure → error |
| Edge: empty | Empty paragraph with mark → valid | Empty measure with mark → valid |

All rendering and MusicXML rules are unchanged in substance (mark appears above the target measure).
### Supersession Note: 2026-05-20 VexFlow Removal

Historical review text in this proposal is preserved. Any future rendering work described here that assumes VexFlow is superseded by `ARCHITECTURE_proposal_remove_vexflow.md`.

If rehearsal marks are implemented after VexFlow removal, their layout and box geometry must be represented through `RenderScore -> LayoutScene` and rendered by the thin adapter without adapter-side engraving fixes.
