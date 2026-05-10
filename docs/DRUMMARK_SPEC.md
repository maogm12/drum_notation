# DrumMark Specification

## Status

**Definitive** — the authoritative source of truth for the DrumMark DSL, IR, compiler design, and MusicXML export.

This document is the merged successor of `DSL_DESIGN.md` and `IR_SPEC.md`. All language syntax, intermediate representation, compiler pipeline, error format, and export guidance live here. From now on, only this document needs to be read.

---

## 1. Overview

DrumMark is a plain-text notation language for drum scores. It is designed to be fast to write, human-readable, and directly compilable to a deterministic IR.

**Core design principles**:
- **Human-first**: Readable by musicians without a tool.
- **Deterministic**: Same source always produces the same IR.
- **Validatable**: Compiler reports hard errors for any unsupported construct.
- **Renderer-agnostic**: IR output feeds VexFlow rendering, MusicXML export, and future playback.

---

## 2. Architecture & Compile Pipeline

### 2.1 Pipeline

```
DSL Source
  │
  ▼
Tokenizer
  │ token stream
  ▼
Parser
  │ AST
  ▼
Normalizer  ──►  Validation
  │                    │
  ▼                    ▼  (hard errors thrown here)
IR (JSON)
  │
  ├──────────────────┬────────────────────┐
  ▼                  ▼                    ▼
VexFlow Renderer   MusicXML Exporter   Playback Engine
(preview + PDF)    (MuseScore)         (future)
```

- All score rendering goes through VexFlow 5.
- MusicXML is an export-only backend.
- The editor is the source of truth. Preview and export are derived.
- IR is the single canonical interchange format.

### 2.2 Rendering Modes

**Continuous Scroll Preview**
- Single-page infinite-scroll SVG rendering the full score
- No page breaks; the entire score is visible at once
- Used for live preview in the browser

**Page View / PDF Export**
- Score is sliced into pages using the current page size (default: US Letter 8.5×11 in, 612×792 pt)
- Systems are placed top to bottom, one after another, until the current page has no room for the next system
- At that point, a new page is started and systems continue filling it the same way
- Header (title, subtitle, composer) appears on the first page only
- Each page is rendered as a separate SVG via `renderScorePagesToSvgs`

### 2.3 Normalized Events

The normalized event is the single source for rendering and export. Each event contains:

```
track          — canonical score track after input sugar is resolved
paragraphIndex — paragraph index for layout
measureIndex   — measure index within the score
measureInParagraph — measure position within its paragraph
start          — rational musical duration (Fraction), not raw grid slot
duration       — rational musical duration (Fraction)
kind           — hit | rest | sticking
glyph          — atomic glyph token
modifier       — modifier string(s)
tuplet         — tuplet spec or null
tie            — tie spec or null
voice          — 1 (up-stem) | 2 (down-stem) | null (derived)
beam           — begin | continue | end | none | null (derived)
```

Notes:
- `track` is the canonical score track after input sugar is resolved
- Anonymous tracks are expanded before normalization and do not appear as a normalized track
- `c` resolves canonically to `C:d`; `C` resolves to `C:d:accent`
- `start` and `duration` are rational musical durations, not raw grid slot numbers
- Groups that require automatic tie splitting are **rejected** during validation
- Instrument placement is derived by renderers/exporters from `track`, `glyph`, and `modifier`

---

## 3. Headers

### 3.1 Supported Header Fields

```txt
title <text>
subtitle <text>
composer <text>
tempo <number>
time <beats>/<beatUnit>
divisions <number>
grouping <a+b+c+...>
```

**Rules**:
- `title`, `subtitle`, `composer`: optional, free text.
- `tempo`: optional, positive integer, default `120`, interpreted as quarter-note BPM.
- `time`: required, e.g. `4/4` or `4 / 4`.
- `divisions`: required, positive integer, defines the grid resolution per measure.
- `grouping`: optional, e.g. `2+2` or `1 + 1 + 1 + 1`. Sum must equal numerator of `time`. Controls beat structure, default accents, beaming. Defaults inferred from `time` if absent.

### 3.2 Grouping Inference

If the `grouping` header is omitted, the compiler infers a default grouping based on the `time` signature:

| Time | Inferred Grouping |
|------|-------------------|
| `2/4` | `1+1` |
| `3/4` | `1+1+1` |
| `4/4` | `2+2` |
| `2/2` | `1+1` |
| `3/8` | `1+1+1` |
| `6/8` | `3+3` |
| `9/8` | `3+3+3` |
| `12/8` | `3+3+3+3` |

**Rules**:
- **Spaces**: Spaces are optional around the `/` in `time` (e.g., `4 / 4`) and around the `+` in `grouping` (e.g., `1 + 1 + 1 + 1`).
- **Sum**: For an explicit `grouping`, the sum of all parts must exactly equal the numerator of the `time` signature.
- **Irregular Meters**: Signatures like `5/8`, `7/8`, or `5/4` have no default inference and **require** an explicit `grouping` header (e.g., `grouping 3+2`).

---

## 4. Tracks

### 4.1 Supported Track Names

| ID | Family | MIDI Note |
|----|--------|-----------|
| `BD` | drum | 36 |
| `BD2` | drum | 36 |
| `SD` | drum | 38 |
| `T1` | drum | 48 |
| `T2` | drum | 45 |
| `T3` | drum | 41 |
| `T4` | drum | 43 |
| `HH` | cymbal | 42 |
| `HF` | pedal | 44 |
| `RC` | cymbal | 51 |
| `RC2` | cymbal | 59 |
| `C` | cymbal | 49 |
| `C2` | cymbal | 57 |
| `SPL` | cymbal | 55 |
| `CHN` | cymbal | 52 |
| `CB` | percussion | 56 |
| `WB` | percussion | 76 |
| `CL` | percussion | 75 |
| `ST` | auxiliary | — (no MIDI) |

### 4.2 Track Line Syntax

```
<TRACK> | ... |
```

Example:
```
HH | x - x - x - x - |
SD | - - d:cross - d - |
```

### 4.3 Anonymous Track

A line that starts directly with a barline acts as a universal container:

```
| x - s - x - s |
```

The default track for anonymous lines is `HH` for glyph routing.

### 4.4 Track Routing Scopes

Use `@TRACK { ... }` to route a block of notes to a specific track without affecting timing:

```
| @RC { x x x x } |        # A full measure of Ride
| @SD { [3: d d d] } |    # Tuplet group on SD
```

### 4.5 Voice Convention

- Voice 1 (up-stem): `HH`, `RC`, `RC2`, `C`, `C2`, `SPL`, `CHN`, `SD`, `T1`, `T2`, `T3`, `T4`, `CB`, `WB`, `CL`, `ST`
- Voice 2 (down-stem): `BD`, `BD2`, `HF`

### 4.6 Track Registry and Auto Fill

- Any track mentioned via line header (`SD |`), routed block directive (`@SD { ... }`), or summoning prefix (`SD:d`) is **automatically registered** in the score.
- Tracks are ordered based on their first appearance in the document.
- Once a track is registered, it remains active throughout the score.
- If a registered track is omitted in a later paragraph, it is auto-filled with full-measure rests.

### 4.7 Track Families

| Family | Tracks |
|--------|--------|
| cymbal | `HH`, `RC`, `RC2`, `C`, `C2`, `SPL`, `CHN` |
| drum | `SD`, `BD`, `BD2`, `T1`, `T2`, `T3`, `T4` |
| pedal | `HF` |
| percussion | `CB`, `WB`, `CL` |
| auxiliary | `ST` |

---

## 5. Tokens

### 5.1 Atomic Tokens

| Token | Meaning |
|-------|---------|
| `d` | Universal hit (standard notehead) |
| `D` | Universal hit with accent |
| `-` | Rest |
| `x` | Cymbal/Crossstick — maps to `HH:d` in cymbal context, `SD:d:cross` in drum context, `HH:d` in anonymous |
| `s` | `SD:d` |
| `S` | `SD:d:accent` |
| `b` | `BD:d` |
| `B` | `BD:d:accent` |
| `b2` | `BD2:d` |
| `B2` | `BD2:d:accent` |
| `r` | `RC:d` |
| `R` | `RC:d:accent` |
| `r2` | `RC2:d` |
| `R2` | `RC2:d:accent` |
| `c` | `C:d` |
| `C` | `C:d:accent` |
| `c2` | `C2:d` |
| `C2` | `C2:d:accent` |
| `t1`, `t2`, `t3`, `t4` | `T1:d`, `T2:d`, `T3:d`, `T4:d` |
| `o` | `HH:d:open` |
| `O` | `HH:d:open:accent` |
| `spl` | `SPL:d` |
| `SPL` | `SPL:d:accent` |
| `chn` | `CHN:d` |
| `CHN` | `CHN:d:accent` |
| `cb` | `CB:d` |
| `CB` | `CB:d:accent` |
| `wb` | `WB:d` |
| `WB` | `WB:d:accent` |
| `cl` | `CL:d` |
| `CL` | `CL:d:accent` |
| `p` | `(Local):d`; in anonymous track, `HF:d` |
| `g` | `(Local):d:ghost`; in anonymous track, `SD:d:ghost` |
| `R`, `L` | Sticking — used in `ST` track or with `ST:` prefix |

### 5.2 Resolution Priority

When parsing a token, the compiler resolves its target in this order:

1. **Explicit override**: `RC:d` forces delivery to `RC` track.
2. **Static Magic Token**: `s`, `b`, `r`, etc. always map to their global physical target (`s` → `SD`) even inside other track lines.
3. **Context fallback**: `d` or `x` in a named track line uses that line's track; in anonymous `|` defaults to `HH`.

### 5.3 Duration Modifiers

| Symbol | Effect |
|--------|--------|
| `.` | Multiplies duration by 1.5. Multiple dots accumulate (`d..` = 1.75×). |
| `/` | Halves duration. Multiple halves accumulate (`d//` = 0.25×). |
| `*` | Doubles duration. Multiple stars accumulate (`d**` = 4×). No per-token limit; measure validation ensures correctness. |

Combined: `d./` = 0.75× duration. `d.*` = 3× duration. Modifiers are commutative — order does not affect the result.

### 5.4 Rhythmic Math

Each token's weight is computed as:

```
weight = base × (2 - 0.5^dots) × 2^stars / (2^halves)
```

**Base values**: `d` = 1 slot, `-` = 1 slot.

**Dotting** (left-associative):
- `d.` = 1.5
- `d..` = 1 + 0.5 + 0.25 = 1.75
- `d...` = 1 + 0.5 + 0.25 + 0.125 = 1.875

**Halving**:
- `d/` = 0.5
- `d//` = 0.25
- `d///` = 0.125

**Doubling**:
- `d*` = 2
- `d**` = 4
- `d***` = 8

Fractional validation: each token is converted to an absolute Fraction relative to a whole note before summing. Validation MUST use exact rational duration math internally. A measure is valid iff the sum of all token durations equals the full `timeSignature` fraction; equivalently, the accumulated token weight equals `divisions`.

### 5.5 Groups

**Syntax**:
```
[span: item1 item2 ...]
[ item1 item2 ... ]     # shorthand for [1: item1 item2 ...]
```

**Semantics**: Each item's duration = `slotDuration × span / itemCount`.

**Supported group forms**:

*Stretched* (`itemCount ≤ span`): Allowed only if each resulting item duration maps to a standard note value, dotted note, or tuplet — and does not require automatic tie splitting.

*Compressed* (`itemCount > span`): Ratios `[2, 1]`, `[4, 1]` → subdivide (no tuplet). All others → tuplet.

*Unsupported*: Any group requiring automatic tie splitting is a hard error.

**Minimum duration**: No guarantees below 64th note.

---

## 6. Modifiers

### 6.1 Supported Modifiers

| Modifier | Allowed on | Visual Effect |
|----------|-----------|--------------|
| `accent` | all tracks | Accent mark (>) above note |
| `open` | `HH` | Open circle on X notehead |
| `half-open` | `HH` | Sizzle; encircled "zz" or half-open circle; CC4 ≈ 64 |
| `close` | `HH`, `HF` | Default hi-hat state |
| `choke` | `C`, `C2`, `RC`, `RC2`, `SPL`, `CHN` | `+` or `×` above note |
| `bell` | `RC`, `RC2` | `B` or dot on ride cymbal |
| `rim` | `SD` | Smaller notehead + "R" optional |
| `cross` | `SD` | X above stem on snare |
| `flam` | `SD`, `T1`, `T2`, `T3`, `T4` | 16th grace note preceding main note |
| `ghost` | `SD`, `HH`, `T1`, `T2`, `T3`, `T4` | Parenthesized notehead |
| `drag` | `SD`, `HH`, `T1`, `T2`, `T3`, `T4`, `RC`, `RC2` | Two 16th grace notes preceding |
| `roll` | `SD`, `HH`, `T1`, `T2`, `T3`, `T4`, `RC`, `RC2`, `BD`, `BD2` | Slash marks on stem |
| `dead` | `SD`, `HH`, `T1`, `T2`, `T3`, `T4`, `BD`, `BD2` | Small "x" notehead, muted attack |

### 6.2 Modifier Syntax

```
<token>:<modifier>
Track:d:<modifier>
```

Examples:
```
HH | d - d:open - d:close - d - |
SD | - - d:cross - d - d:rim:accent - |
RC | - - d:bell - - - d - |
C  | d:choke:accent - - - - - - - |
SD | - - d:ghost - - - - |
HH | d - d:drag - - - - - |
```

---

## 7. Combined Hits

Use `+` to play multiple instruments simultaneously:

```
x+s          # Hi-hat and Snare
b+x          # Bass drum and Hi-hat
HH:d + SD:d  # Explicit combined hit
```

Combined hits produce multiple events at the same `start` position.

---

## 8. Sticking

### 8.1 Sticking Track

Use the `ST` track for hand sticking annotations:

```
ST | R - L - [2: R L R] - | R - L - R - L - |
```

### 8.2 Sticking Semantics

- Sticking tokens in `ST` track do not create MusicXML `<note>` elements with percussion step/octave. They are attached as `<fingering>` or `<direction>` to notes at the same rhythmic position.
- Sticking at a given `start` position applies to **all notes** at that position across all tracks.
- Sticking without a matching note at the same `start` position is ignored in MusicXML export.

---

## 9. Repeats

### 9.1 Repeat Barlines

| Syntax | Meaning |
|--------|---------|
| `\|` | regular barline |
| `\|:` | repeat start |
| `:\|` | repeat end |
| `\|: :\|` | repeat start + end (same measure) |
| `\|\|` | double barline (no measure between) |
| `\|  \|` | double barline with whitespace → empty measure between |
| `\|.` | explicit volta termination |

**Double barline with empty measure**: If whitespace exists between the two bars, it forms an empty measure. `|  |` and `|  |` (any amount of whitespace) both produce a double barline with an empty measure in between. `||` with no whitespace produces a double barline with no measure between.

### 9.2 Repeat Rules

- Repeats are global measure structure, not private to one track.
- Repeat boundaries may be written on any track. A declaration on any track applies to the whole score.
- Nested repeats are not allowed in v1.
- Crossing repeats are not allowed in v1.

### 9.3 Voltas (Alternative Endings)

| Syntax | Meaning |
|--------|---------|
| `\|1.` | Volta for 1st repetition |
| `\|1,2.` | Volta shared by 1st and 2nd repetition |
| `\|.` | Explicit termination of current volta bracket |

**Example**:
```
|: d d d d |1. d d d d :|2. d d d d | d d d d |. d d d d |
```

**Rules**:
- Volta starts at `|N.`` or `|N,M.` barline.
- Volta ends when: (a) a `repeat-end` barline is encountered, (b) a new `volta` with a different index starts, or (c) `|.` is encountered.
- Both `|: :|` and voltas can span multiple paragraphs. Paragraph boundaries (blank lines) only trigger system breaks in the renderer and do not affect the musical logical structure.
- If a volta is followed immediately by `:|`, the bracket ends at that barline.

### 9.4 Measure Repeat (`%`)

A measure containing only `%` shorthand repeats the preceding measures. Each `%` repeats one preceding measure.

```
HH | d d d d | % |      # repeats 1 preceding measure
HH | c c c c | %% |     # repeats 2 preceding measures
BD | b - - - | % |      # repeats 1 preceding measure
```

**Rules**:
- One `%` repeats one preceding measure; two `%%` repeats two preceding measures; and so on.
- The measure containing `%` shorthand must contain only that token (no other notes).
- The referenced measures must not themselves be repeat shorthand measures (no chaining).
- Canonical IR stores measure-repeat intent as `measureRepeat.slashes` (`1` for `%`, `2` for `%%`).

### 9.5 Complex Repeats (Markers & Jumps)

Complex navigation is handled via markers (targets) and jumps (instructions). These are global and can be declared on any track.

| Syntax | Meaning | Visual |
|--------|---------|--------|
| `@segno` | Segno marker | $\S$ |
| `@coda` | Coda marker | $\phi$ |
| `@fine` | Fine marker | "Fine" |
| `@to-coda` | To Coda jump | "To Coda" |
| `@da-capo` | Da Capo | "D.C." |
| `@dal-segno` | Dal Segno | "D.S." |
| `@dc-al-fine` | Da Capo al Fine | "D.C. al Fine" |
| `@dc-al-coda` | Da Capo al Coda | "D.C. al Coda" |
| `@ds-al-fine` | Dal Segno al Fine | "D.S. al Fine" |
| `@ds-al-coda` | Dal Segno al Coda | "D.S. al Coda" |

**Rules**:
- **Placement**: These tokens can appear anywhere within a measure's content (e.g., `| @segno d d d d |`).
- **Global Scope**: Like repeat barlines, a marker or jump declared on one track applies to the entire measure for all tracks.
- **Conflict**: A single measure may contain at most one marker and at most one jump. Conflicting declarations within the same measure (on same or different tracks) are a hard error.
- **Render position**: Markers usually appear above the start of the measure; jumps usually appear above the end of the measure.

---

## 10. Multi-Measure Rest

Multi-measure rest is written as a left dash run, an integer count, and a right dash run.

```
HH | --8-- |
HH | -- 8 -- |
HH | ---8---- |
```

**Rules**:
- The left dash run must contain at least two `-` characters.
- The right dash run must contain at least two `-` characters.
- The left and right dash runs do not need to have equal length.
- Optional horizontal whitespace may appear between the left dash run and the integer.
- Optional horizontal whitespace may appear between the integer and the right dash run.
- `N` is an unsigned decimal integer.
- `N` must be at least `2`.
- Leading zeros are not allowed.
- The entire construct must fit within a single measure boundary `| ... |`.

---

## 11. Inline Measure Repeat

### `*N` — Inline repeat count

`*N` at the end of a measure expands that measure to a total of `N` consecutive measures.

```
HH | dddd *2 |       # 2 measures of dddd
HH | - *3 |           # 3 blank measures
```

This is syntactic sugar. After expansion, there is no record that `*N` was used.

---

## 12. Measure Validation

### 12.1 Total Duration

For each measure, the sum of all token durations must equal one full measure length. Equivalently, the sum of token weights must equal `divisions`. Any mismatch is a hard error.

### 12.2 Grouping Boundary Alignment

No token or group may cross a boundary defined by `grouping`. A hard error is reported if a token's duration overlaps a grouping boundary.

**Example** (error):
```
HH | d. d/ d d |    # 'd.' crosses boundary at slot 2
```

**Correct**:
```
HH | d. -/ d d |    # 'd.' ends at 1.5, followed by a half-rest at 1.5-2.0
```

### 12.3 Whole-Measure Rest

If all entries in a voice are rests and their combined duration equals the full measure, emit one `<rest measure="yes"/>` element instead of splitting at grouping boundaries:

```xml
<note>
  <rest measure="yes"/>
  <duration>32</duration>
  <voice>2</voice>
  <type>whole</type>
  <staff>1</staff>
</note>
```

**Rationale**: A voice that is entirely silent for a full measure does not need to assert the silence slot-by-slot. The grouping structure is irrelevant when there is nothing to render. A single whole-measure rest is semantically equivalent and more compact.

**Rule**: If a voice consists entirely of rests covering a complete measure, emit one whole-measure rest. Otherwise, split rests at grouping boundaries as normal. Applies to both voice 1 and voice 2.

---

## 13. Comments

```
# comment
```

`#` starts a comment that runs to end of line. Comments are ignored by the parser.

---

## 14. Whitespace

- Spaces and tabs are ignored structurally except as token separators.
- Users may add spaces freely for alignment and readability.

These should be treated equivalently by the parser:

```
HH | d - d - |
HH|d-d-|
HH |   d   -   d   -   |
```

### Paragraphs

After the header, track content is organized into paragraphs. Blank lines separate paragraphs. Paragraph primarily affects layout and text organization. Each paragraph starts a new system in the rendered score. Paragraph does not change musical time structure.

---

## 15. Compiler Errors

### 15.1 Parsing Strategy

- Be permissive about whitespace
- Be strict about semantics
- Try to collect multiple errors in one pass
- Do not silently rewrite user intent
- Any unsupported, ambiguous, or inconsistent construct is a hard error

### 15.2 Error Format

Errors should include line, column, and message:

```
Line 8, Col 12: Unknown token `q` on track HH
Line 10, Col 7: Group [3: a b] expects 3 items, got 2
Line 14, Col 1: Repeat boundary conflicts with previous declaration
Line 18, Col 3: Modifier `:choke` is not allowed on SD
Line 21, Col 5: Measure duration (14) does not equal divisions (16)
Line 24, Col 1: Token `d.` crosses grouping boundary at slot 2
Line 27, Col 8: Unknown track `XX`
Line 30, Col 1: Empty measure is not allowed in repeat section
```

### 15.3 Hard Error List

- Unknown header field
- Unknown track
- Illegal token on a track
- Unknown modifier
- Illegal glyph + modifier combination
- Malformed group
- Group item count mismatch
- Measure slot mismatch
- Repeat conflict
- Repeat structure mismatch
- Paragraph measure-count mismatch among explicit tracks
- Multi-measure rest with `N < 2`
- Inline repeat with non-positive `N`

---

## 16. Intermediate Representation (IR)

The compiler emits a JSON IR. All temporal values use the **Fraction** structure.

### 16.1 Basic Types

#### Fraction (Object)
All temporal values (start, duration) MUST be stored as reduced fractions.
- `num`: Non-negative integer.
- `den`: Positive integer.

#### TimeSignature (Object)
- `beats`: Number of beats in a measure.
- `beatUnit`: The note value that represents one beat.

---

### 16.2 Document Hierarchy

```
DrumScore
  └─ version: string
  └─ header: Header
  └─ tracks: Track[]
  └─ measures: Measure[]
```

### 16.3 Header IR

| Field | Type | Required | Meaning |
|-------|------|----------|---------|
| `title` | string | no | Score title |
| `subtitle` | string | no | Score subtitle |
| `composer` | string | no | Composer credit |
| `tempo` | integer | no | Quarter-note BPM |
| `timeSignature` | `TimeSignature` | **yes** | Measure structure |
| `divisions` | integer | **yes** | Grid slots per measure |
| `grouping` | integer[] | no | Beat grouping |

### 16.4 Track IR

| Field | Type | Required | Meaning |
|-------|------|----------|---------|
| `id` | string | **yes** | Track identifier (e.g., "HH") |
| `family` | string | **yes** | `cymbal`, `drum`, `pedal`, `percussion`, `auxiliary` |

### 16.5 Measure IR

A `Measure` is the primary container for events and visual/structural metadata.

```json
{
  "index": 0,
  "events": [ Event, Event, ... ],
  "barline": "regular",
  "marker": "segno",
  "jump": "ds-al-coda",
  "volta": { "indices": [1, 2] },
  "measureRepeat": { "slashes": 1 },
  "multiRest": { "count": 4 }
}
```

| Field | Type | Required | Meaning |
|-------|------|----------|---------|
| `index` | integer | **yes** | 0-based measure index within the score |
| `events` | `Event[]` | **yes** | All events in this measure |
| `barline` | `BarlineType` | **yes** | Visual style of the right-hand barline |
| `marker` | `MarkerType` | no | Navigation marker (e.g., segno, coda) |
| `jump` | `JumpType` | no | Navigation jump instruction (e.g., D.S. al Coda) |
| `volta` | `VoltaIntent` | no | Metadata for alternative endings |
| `measureRepeat` | `MeasureRepeatIntent` | no | Visual repeat shorthand (% or %%) |
| `multiRest` | `MultiRestIntent` | no | Multi-measure rest metadata |

---

### 16.6 BarlineType (Enum)

| Value | Meaning |
|-------|---------|
| `regular` | Standard single barline |
| `double` | Double barline |
| `final` | Termination or heavy double barline |
| `repeat-start` | Start of a repeat section |
| `repeat-end` | End of a repeat section |
| `repeat-both` | Back-to-back repeat (end + start) |

### 16.7 MarkerType (Enum)

| Value | Meaning |
|-------|---------|
| `segno` | Segno symbol ($\S$) |
| `coda` | Coda symbol ($\phi$) |
| `fine` | "Fine" text |

### 16.8 JumpType (Enum)

| Value | Meaning |
|-------|---------|
| `da-capo` | "D.C." |
| `dal-segno` | "D.S." |
| `dc-al-fine` | "D.C. al Fine" |
| `dc-al-coda` | "D.C. al Coda" |
| `ds-al-fine` | "D.S. al Fine" |
| `ds-al-coda` | "D.S. al Coda" |
| `to-coda` | "To Coda" |

### 16.9 VoltaIntent (Object)

| Field | Type | Meaning |
|-------|------|---------|
| `indices` | `integer[]` | 1-based indices for the jump bracket (e.g., `[1]`) |

### 16.8 MeasureRepeatIntent (Object)

| Field | Type | Meaning |
|-------|------|---------|
| `slashes` | `integer` | `1` for `%`, `2` for `%%` |

### 16.9 MultiRestIntent (Object)

| Field | Type | Meaning |
|-------|------|---------|
| `count` | `integer` | Total measures in the rest block (N >= 1) |

---

### 16.10 Event IR

```json
{
  "track": "HH",
  "start": { "num": 0, "den": 16 },
  "duration": { "num": 1, "den": 16 },
  "kind": "hit",
  "glyph": "d",
  "modifiers": [],
  "tuplet": null,
  "tie": null,
  "voice": 1,
  "beam": "begin"
}
```

| Field | Type | Required | Meaning |
|-------|------|----------|---------|
| `track` | string | **yes** | Track ID |
| `start` | `Fraction` | **yes** | Offset from start of measure |
| `duration` | `Fraction` | **yes** | Musical duration |
| `kind` | string | **yes** | `hit`, `rest`, `sticking` |
| `glyph` | string | `kind == "hit"` | Atomic glyph token |
| `modifiers` | string[] | no | List of modifier strings |
| `tuplet` | `TupletSpec` | no | Tuplet metadata if applicable |
| `tie` | `string` | no | `start`, `stop`, `both` |
| `voice` | integer | no | `1` (up), `2` (down) |
| `beam` | `string` | no | `begin`, `continue`, `end`, `none` |

---

### 16.11 TupletSpec (Object)

| Field | Type | Meaning |
|-------|------|---------|
| `actual` | integer | Notes played (e.g., 3) |
| `normal` | integer | Notes in normal time (e.g., 2) |
| `span` | integer | Duration in "normal" units |
| `bracket` | boolean | Whether to draw the bracket |

### 16.12 Range Annotations

| Field | Type | Meaning |
|-------|------|---------|
| `type` | string | `hairpin`, `slur`, `dynamic` |
| `subtype` | string | e.g., `crescendo`, `p`, `ff` |
| `start` | `Fraction` | Start position |
| `end` | `Fraction` | End position (for span types) |

**Cross-measure spanning**: A hairpin that spans measures 0–2 is represented as one fragment per measure, using `{ "num": 1, "den": 1 }` to anchor at measure boundaries.

---

## 17. MusicXML Export

### 17.1 Export Structure

- Export from normalized events, not raw DSL
- Use **one percussion part** for the whole drum kit, not one part per track
- `divisions` in MusicXML may be chosen independently as needed for accurate durations
- `:|` should export as actual repeat barlines when possible

### 17.2 Track → Instrument Mapping

| Track | MusicXML Instrument | MIDI Note |
|-------|-------------------|-----------|
| `HH` | closed hi-hat | 42 |
| `HF` | pedal hi-hat | 44 |
| `SD` | snare | 38 |
| `BD` | bass drum | 36 |
| `BD2` | bass drum | 36 |
| `T1` | high tom | 48 |
| `T2` | mid tom | 45 |
| `T3` | floor tom | 41 |
| `T4` | low tom | 43 |
| `RC` | ride cymbal | 51 |
| `RC2` | ride cymbal 2 | 59 |
| `C` | crash cymbal | 49 |
| `C2` | crash cymbal 2 | 57 |
| `SPL` | splash cymbal | 55 |
| `CHN` | china cymbal | 52 |
| `CB` | cowbell | 56 |
| `WB` | wobble board | 76 |
| `CL` | clap | 75 |

### 17.3 Velocity Mapping

| Track | Default Velocity | Accent Velocity | Ghost Velocity |
|-------|-------------------|-----------------|----------------|
| `BD` / `BD2` | 90 | 127 | 30 |
| `SD` | 85 | 120 | 25 |
| `T1` / `T2` | 80 | 115 | 25 |
| `T3` / `T4` | 82 | 118 | 28 |
| `HH` | 80 | 115 | 20 |
| `HF` | 75 | 110 | 20 |
| `RC` / `RC2` | 78 | 112 | 20 |
| `C` / `C2` | 85 | 120 | 25 |
| `SPL` | 80 | 115 | 20 |
| `CHN` | 83 | 120 | 22 |
| `CB` | 75 | 110 | 20 |
| `WB` / `CL` | 72 | 108 | 18 |

**HiHat open/close**: Note-on for `HH` (42) is sent regardless; a CC4 message follows with value `0` (closed) or `127` (open), sent simultaneously with or 1 tick after the note-on.

### 17.4 Notehead Selection (Renderer & Exporter Reference)

| Family | Default | With `:ghost` | With `:accent` |
|--------|---------|---------------|----------------|
| cymbal | X | X (parenthesized) | X + accent mark |
| drum | filled-circle | filled-circle (parenthesized) | filled-circle + accent mark |
| pedal | filled-circle | filled-circle (parenthesized) | filled-circle + accent mark |
| percussion | filled-circle | filled-circle (parenthesized) | filled-circle + accent mark |

### 17.5 Modifier Export Priority

v0 modifiers are limited to forms with stable MusicXML export semantics.

**Supported and reliably exported**:
- accents
- open/close hi-hat
- tuplets
- flam
- ghost
- drag

**Supported when explicitly included in the whitelist**:
- rim
- cross
- bell
- choke

**Out of scope for the current v0 MusicXML exporter**: A modifier may be valid in the DSL even if this exporter does not yet provide a stable representation for it.

`ghost` and `drag` are exported as grace notes with appropriate notation semantics.

### 17.6 Sticking Export

- `ST` sticking is exported as above-staff fingering text at matching note positions.
- `R` and `L` do not export as percussion notes.
- Sticking text does not advance rhythmic time.
- Matching is based on start position (Fraction), not track identity.
- A sticking annotation at a given start position applies to all notes at that position, regardless of track.
- Sticking without a matching note at the same start position is ignored.

---

## 18. Complete Example

```
title Funk Study No. 1
subtitle Verse groove
composer G. Mao
tempo 96
time 4/4
divisions 16
grouping 2+2

HH |: d - d - o - d - | d - d:close - d:accent - d - :|
SD |  - - d:cross - d - | d:rim:accent - [2: d d:flam d] - - -  |
BD |  d - - - d - - - | d - d - - - d -                     |
HF |  - - - - p - - - | - - - - p:close - -                |

RC | - - d:bell - - - d - | - - - - - - - - |
C  | d:choke:accent - - - - - - - | - - - - d:accent - - - |
ST | R - L - [2: R L R] - | R - L - R - L - |
```

Corresponding IR excerpt (first measure):

```json
{
  "version": "1.0",
  "header": {
    "title": "Funk Study No. 1",
    "subtitle": "Verse groove",
    "composer": "G. Mao",
    "tempo": 96,
    "timeSignature": { "beats": 4, "beatUnit": 4 },
    "divisions": 16,
    "grouping": [2, 2]
  },
  "tracks": [
    { "id": "HH", "family": "cymbal" },
    { "id": "SD", "family": "drum" },
    { "id": "BD", "family": "drum" },
    { "id": "HF", "family": "pedal" },
    { "id": "RC", "family": "cymbal" },
    { "id": "C", "family": "cymbal" },
    { "id": "ST", "family": "auxiliary" }
  ],
  "measures": [
    {
      "index": 0,
      "barline": "repeat-start",
      "events": [
        { "track": "HH", "start": { "num": 0, "den": 16 }, "duration": { "num": 1, "den": 16 }, "kind": "hit", "glyph": "d", "modifiers": [], "tuplet": null, "tie": null, "voice": 1, "beam": "begin" },
        { "track": "HH", "start": { "num": 1, "den": 16 }, "duration": { "num": 1, "den": 16 }, "kind": "rest", "glyph": null, "modifiers": [], "tuplet": null, "tie": null, "voice": 1, "beam": "none" },
        { "track": "HH", "start": { "num": 2, "den": 16 }, "duration": { "num": 1, "den": 16 }, "kind": "hit", "glyph": "d", "modifiers": [], "tuplet": null, "tie": null, "voice": 1, "beam": "continue" },
        { "track": "HH", "start": { "num": 3, "den": 16 }, "duration": { "num": 1, "den": 16 }, "kind": "hit", "glyph": "o", "modifiers": [], "tuplet": null, "tie": null, "voice": 1, "beam": "end" },
        { "track": "HH", "start": { "num": 4, "den": 16 }, "duration": { "num": 1, "den": 16 }, "kind": "hit", "glyph": "d", "modifiers": [], "tuplet": null, "tie": null, "voice": 1, "beam": "begin" },
        { "track": "HH", "start": { "num": 7, "den": 16 }, "duration": { "num": 1, "den": 16 }, "kind": "hit", "glyph": "d", "modifiers": [], "tuplet": null, "tie": null, "voice": 1, "beam": "end" },
        { "track": "SD", "start": { "num": 2, "den": 16 }, "duration": { "num": 1, "den": 16 }, "kind": "hit", "glyph": "d", "modifiers": ["cross"], "tuplet": null, "tie": null, "voice": 1, "beam": "begin" },
        { "track": "SD", "start": { "num": 3, "den": 16 }, "duration": { "num": 1, "den": 16 }, "kind": "hit", "glyph": "d", "modifiers": [], "tuplet": null, "tie": null, "voice": 1, "beam": "end" },
        { "track": "BD", "start": { "num": 0, "den": 16 }, "duration": { "num": 1, "den": 16 }, "kind": "hit", "glyph": "d", "modifiers": [], "tuplet": null, "tie": null, "voice": 2, "beam": "none" }
      ]
    }
  ]
}
```

---

## Appendix A: Complete Token Reference

| Token | Track | Modifiers | Notes |
|-------|-------|-----------|-------|
| `d` | local | — | |
| `x` | HH/SD | cross in drum ctx | context-aware |
| `s` | SD | — | |
| `S` | SD | accent | |
| `b` | BD | — | |
| `B` | BD | accent | |
| `b2` | BD2 | — | |
| `B2` | BD2 | accent | |
| `r` | RC | — | |
| `R` | RC | accent | |
| `r2` | RC2 | — | |
| `R2` | RC2 | accent | |
| `c` | C | — | |
| `C` | C | accent | |
| `c2` | C2 | — | |
| `C2` | C2 | accent | |
| `t1`–`t4` | T1–T4 | — | |
| `o` | HH | open | |
| `O` | HH | open, accent | |
| `spl` | SPL | — | |
| `SPL` | SPL | accent | |
| `chn` | CHN | — | |
| `CHN` | CHN | accent | |
| `cb` | CB | — | |
| `CB` | CB | accent | |
| `wb` | WB | — | |
| `WB` | WB | accent | |
| `cl` | CL | — | |
| `CL` | CL | accent | |
| `p` | HF (local fallback) | — | |
| `g` | local | ghost | |
| `R/L` | ST | — | sticking |
| `-` | — | — | rest |

---

## Appendix B: Modifier Legality Matrix

| Modifier | BD | BD2 | SD | T1 | T2 | T3 | T4 | HH | HF | RC | RC2 | C | C2 | SPL | CHN | CB | WB | CL |
|----------|----|----|----|----|----|----|----|----|----|----|----|----|----|-----|-----|----|----|----|----|----|----|
| accent | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| open | | | | | | | | ✓ | | | | | | | | | | | |
| half-open | | | | | | | | ✓ | | | | | | | | | | | |
| close | | | | | | | | | ✓ | ✓ | | | | | | | | | |
| choke | | | | | | | | | | | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | | | | |
| bell | | | | | | | | | | ✓ | ✓ | | | | | | | | |
| rim | | | ✓ | | | | | | | | | | | | | | | | |
| cross | | | ✓ | | | | | | | | | | | | | | | | | |
| flam | | | ✓ | ✓ | ✓ | ✓ | ✓ | | | | | | | | | | | | | |
| ghost | | | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | | | | | | | | | | | |
| drag | | | ✓ | ✓ | ✓ | ✓ | ✓ | | | ✓ | ✓ | | | | | | | | |
| roll | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | | | ✓ | ✓ | | | | | | | | |
| dead | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | | | | | | | | | | | | | |

---

## Appendix C: Future Improvements

The following features are defined in the spec but not yet implemented or not storable in IR:

| Feature | Description | IR Status |
|---------|-------------|-----------|
| `@tempo:<N>` | Inline tempo change mid-score | Not stored in IR |
| `@time:<N/M>` | Inline time signature change mid-score | Not stored in IR |
| `@partial:<N>` | Pickup/anacrusis measure with N slots | Not stored in IR |
| `@divisions:<N>` | Inline divisions change | Rejected — mid-score divisions change is not supported |
| `dashed` barline | Dashed barline visual | Not yet implemented in IR or renderer |

---

## 20. Developer Tooling

To aid in the development and verification of DrumMark compilers and renderers, a CLI tool is provided.

### `drummark` CLI

This tool allows for rapid inspection of the internal state and output of the DrumMark engine.

**Usage**:
```bash
npm run drummark -- <input-file> [--format ir|xml|svg] [--output <path>]
```

If `--output` is not specified for `xml` or `svg` formats, it defaults to `<input-file>.<format>`.

**Formats**:
- `ir` (default): Dumps the normalized Intermediate Representation (JSON). Useful for verifying parsing and normalization logic.
- `xml`: Generates MusicXML output. Useful for verifying MusicXML export logic.
- `svg`: Generates VexFlow SVG rendering. Useful for verifying visual rendering and layout.

---

## 19. Implementation Responsibilities

### 19.1 Responsibilities of Each Consumer

| Consumer | Reads | Computes |
|----------|-------|----------|
| **VexFlow Renderer** | header, tracks, measures, events | notehead shapes, positioning, page layout |
| **MusicXML Exporter** | header, tracks, measures, events | MIDI note numbers, notehead types, tuplet XML elements |
| **Playback Engine** (future) | header, tracks, measures (with repeats/volts expanded) | linear MIDI event stream, velocities, CC messages |

### 19.2 What IR Does NOT Store

The following are intentionally absent — they are concerns of specific consumers, not the IR:

- **MIDI velocity values**: Mapped by exporter from `modifiers` (e.g., `accent` → velocity 120, `ghost` → velocity 30).
- **Notehead shape per track/modifier**: Looked up by renderer from the Track Registry appearance table.
- **Expanded playback sequence**: (Repeat/volta unfolded) computed by playback engine as a separate pass.
- **Visual positioning data**: (X/Y coordinates) computed by the renderer during layout.

---

## Addendum 2026-04-30: Chained Measure Repeat Resolution

### Status

Proposed

### Scope

This addendum refines Section 9.4 Measure Repeat (`%`) and supersedes only the no-chaining restriction there. All other measure-repeat rules remain in force unless explicitly stated below.

### Revised Rules

- Chained measure-repeat shorthand is allowed.
- `%` resolves to the immediately preceding logical measure in score order after recursively resolving measure-repeat content.
- `%%` resolves to the previous two logical measures in score order after recursively resolving measure-repeat content.
- Resolution is based on logical score order only. It is not based on playback order, and it does not unfold repeat, volta, or jump navigation.
- `%` and `%%` must still occupy the entire measure by themselves.
- A measure-repeat bar still requires the referenced number of preceding logical measures to exist.

### Exclusivity Rule

- Measure-repeat intent is a global bar-level construct.
- If any track declares `%` or `%%` on a given logical bar, no other track on that same logical bar may contain ordinary musical content or multi-measure-rest content.
- Other tracks may leave that bar empty so the global measure-repeat intent can be merged canonically.

### Metadata Rule

- Recursive measure-repeat resolution copies musical content only.
- Structural metadata remains local to the destination logical bar and is not inherited from referenced bars. This includes barline, volta, marker, jump, multi-rest, and measure-repeat metadata itself.

### Consumer Rule

- Canonical normalized IR may continue storing measure-repeat as intent metadata rather than expanded events.
- Any consumer that needs referenced content, such as MusicXML export, must resolve measure-repeat content recursively from prior logical measures rather than reading unresolved repeat-intent bars as literal empty-event bars.

### Examples

```drummark
HH | x - - - | % | % |
```

This is valid and resolves logically to three consecutive bars with the same musical content.

```drummark
HH | x - - - | % | %% |
```

This is valid and resolves logically to:

1. bar 1 -> original content
2. bar 2 -> copy of bar 1
3. bar 3 -> copy of bars 1 and 2 as logical antecedents

```drummark
HH | x - - - | x - - - | x - - - |
SD | - - - - | - - - - | %% |
```

This is invalid because the third logical bar mixes global measure-repeat intent with ordinary content on another track.

## Addendum 2026-04-30: Multi-Marker Navigation Measures

### Status

Proposed

### Scope

This addendum refines Section 9.5 Markers and Jumps. It supersedes only the single-marker restriction there. All other navigation rules remain in force unless explicitly stated below.

### Revised Rules

- A single logical measure may contain zero or more navigation markers.
- Supported markers remain `@segno`, `@coda`, and `@fine`.
- A single logical measure may still contain at most one navigation jump.
- Supported jumps remain `@to-coda`, `@da-capo`, `@dal-segno`, `@dc-al-fine`, `@dc-al-coda`, `@ds-al-fine`, and `@ds-al-coda`.
- Marker multiplicity is local set union, not overwrite semantics. If the same marker is declared multiple times on the same logical measure, the canonical result still contains that marker only once.

### Global Merge Rule

- Markers are global bar-level metadata, like repeat barlines and voltas.
- If multiple tracks declare markers on the same logical bar, the canonical normalized result stores the union of those markers.
- Cross-track disagreement is therefore only an error for jumps, not for markers, unless the representation itself becomes ambiguous in a future extension.

### Render and Export Rule

- Markers continue to render at the start side of the logical measure.
- If a measure contains multiple markers, consumers must preserve all of them rather than dropping later ones.
- Jumps continue to render at the end side of the logical measure and remain singleton metadata.

### Examples

```drummark
HH | @coda x x @fine - - |
```

This is valid. The logical measure carries both the `coda` marker and the `fine` marker.

```drummark
HH | @segno x x x x |
SD | @fine d - - - |
```

This is valid. The canonical logical bar carries both `segno` and `fine`.

```drummark
HH | @to-coda x x x x |
SD | @ds-al-coda d - - - |
```

This is invalid because a logical measure still may not contain more than one jump.

## Addendum 2026-04-30B: Multi-Marker Canonical Representation Clarification

### Status

Proposed

### Scope

This addendum refines and operationalizes the immediately preceding addendum on multi-marker navigation measures. Where the earlier addendum is ambiguous, this addendum controls.

### Canonical IR Change

- The singular canonical measure field `marker` is superseded by `markers`.
- `markers` is an ordered array of zero or more `MarkerType` values.
- `jump` remains a singleton field.
- The canonical `MarkerType` domain remains:
  - `segno`
  - `coda`
  - `fine`

### Canonical Ordering Rule

- `markers` must be stored, rendered, and exported in this fixed canonical order:
  1. `segno`
  2. `coda`
  3. `fine`
- Duplicate declarations of the same marker on one logical bar collapse to one element in `markers`.
- Source token order does not override canonical ordering.

### Conflict Rule Replacement

- This addendum explicitly replaces the old Section 9.5 rule that a logical measure may contain at most one marker.
- New rule:
  - A logical measure may contain any number of markers from the supported marker set.
  - A logical measure may still contain at most one jump.
  - Multiple jump declarations on one logical bar are a hard error.
- Marker-plus-jump combinations remain legal. A logical measure may contain any valid marker set together with one jump.

### Global Merge Rule

- Marker declarations merge across tracks by set union on the logical bar.
- The merged canonical result is then sorted by the canonical ordering rule above.
- Jump declarations merge across tracks only if they are identical.
- If two tracks declare different jumps on the same logical bar, that is a hard error.
- If one or more tracks declare markers and another track declares one jump on the same logical bar, the merged logical bar carries both the marker set and the jump.

### Structural Propagation Rule

- Markers are left-edge structural metadata.
- Jumps are right-edge structural metadata.
- For inline repeat expansion using `*N`:
  - the entire marker set is attached only to the first generated logical measure
  - the jump, if present, is attached only to the last generated logical measure
- For bare `*N` expansion, the same left-edge and right-edge propagation rule applies.
- For measure-repeat and multi-rest shorthand bars, the marker set remains attached to the destination logical bar exactly as written and is not inherited from referenced bars.

### Render and Export Rule

- Consumers must preserve the entire ordered marker set.
- VexFlow rendering places marker labels at the start side of the measure in canonical order.
- MusicXML export emits one `<direction>` block per marker in canonical order before note content for that measure.
- Jumps continue to render and export independently at the end side of the measure.

### Canonical Examples

```drummark
HH | @coda x x @fine - - @to-coda |
```

Canonical logical-bar metadata:

```json
{
  "markers": ["coda", "fine"],
  "jump": "to-coda"
}
```

```drummark
HH | @segno x x x x |
SD | @fine d - - - |
BD | - - @to-coda b - |
```

Canonical logical-bar metadata:

```json
{
  "markers": ["segno", "fine"],
  "jump": "to-coda"
}
```

```drummark
HH | @segno @fine x - - - *3 |
```

This expands to three logical bars. The canonical metadata distribution is:

1. bar 1: `markers = ["segno", "fine"]`
2. bar 2: no markers, no jump
3. bar 3: no markers, no jump

### Review Round 2

- The multi-marker design is approved.
- Canonical representation uses ordered `markers` arrays rather than a singular `marker` field.
- Cross-track marker merge is set union followed by canonical ordering.
- Jumps remain singleton metadata and still hard-fail on conflicting declarations.
- Inline repeat propagation remains directional: markers on the first generated bar, jumps on the last generated bar.

STATUS: APPROVED

## Addendum 2026-04-30C: Positional Navigation Anchors and Barline Forcing

### Status

Proposed

### Scope

This addendum refines navigation syntax and rendering semantics for `segno`, `coda`, `fine`, `dc`, `ds`, and `to-coda`. Where this addendum conflicts with earlier marker/jump rules, this addendum controls.

### Spelling Changes

- `@da-capo` is removed and replaced by `@dc`.
- `@dal-segno` is removed and replaced by `@ds`.
- `@dc-al-fine`, `@dc-al-coda`, `@ds-al-fine`, and `@ds-al-coda` remain supported.
- `@to-coda` remains supported.

### Positional Rule Summary

- `@segno` may appear anywhere in a measure except the final token position.
- `@coda` may appear only at the beginning of a measure.
- `@fine` may appear only at the end of a measure.
- `@dc` and `@ds` may appear only at the end of a measure.
- `@to-coda` may appear anywhere except the beginning of a measure.

### Anchor Semantics

- `@segno`
  - At measure start: anchors to the left barline of the measure.
  - In measure interior: anchors to the immediately following event.
  - At measure end: invalid.
- `@coda`
  - Anchors to the left edge of the measure only.
- `@fine`
  - Anchors to the right edge of the measure only.
- `@dc` and `@ds`
  - Anchor to the right edge of the measure only.
- `@to-coda`
  - In measure interior: anchors to the immediately preceding event.
  - At measure end: anchors to the right edge of the measure.
  - At measure start: invalid.

### Cardinality Rule

- A measure may contain at most one start-side navigation marker.
- The start-side marker set is:
  - `segno`
  - `coda`
- A measure may contain at most one end-side navigation instruction.
- The end-side instruction set is:
  - `fine`
  - `dc`
  - `ds`
  - `dc-al-fine`
  - `dc-al-coda`
  - `ds-al-fine`
  - `ds-al-coda`
  - `to-coda`
- Therefore:
  - `segno` and `coda` may not coexist on one logical bar.
  - `fine`, `dc`, `ds`, and `to-coda` may not coexist with one another on one logical bar.

### Canonical Representation

- Canonical normalized IR no longer treats navigation as unordered marker sets.
- Canonical normalized measures must preserve positional navigation metadata with explicit anchor class.
- The minimum required normalized semantics are:
  - one optional start-side marker: `segno` or `coda`
  - one optional end-side instruction: `fine`, `dc`, `ds`, `dc-al-fine`, `dc-al-coda`, `ds-al-fine`, `ds-al-coda`, or `to-coda`
  - when needed by a consumer, enough anchor information to distinguish left-edge, right-edge, event-after, and event-before placement

### Barline Forcing Rule

- `@fine` forces the measure's right barline to `final`.
- `@dc` and `@ds` force the measure's right barline to at least `double` when no explicit right-side double or final barline is present.
- `@dc-al-fine`, `@dc-al-coda`, `@ds-al-fine`, and `@ds-al-coda` follow the same right-edge forcing rule as `@dc` and `@ds`: if the measure does not already end with an explicit `double` or `final` barline, normalize it to `double`.
- `@to-coda` does not force a barline change by itself.

### Render Rule

- `@segno`
  - At measure start: render above the left barline.
  - On the first measure of a system where no visible left attachment is available, consumers may align it to the first sounded event while preserving start-of-measure semantics.
  - In measure interior: render above the immediately following event.
- `@coda`
  - Render above the left barline.
  - If the measure is first in a rendered system and no visible left attachment is available, render above the first beat event.
- `@fine`
  - Render at the measure end and pair it with a final barline.
- `@dc` and `@ds`
  - Render at the measure end and pair them with a double barline unless a final barline is already present.
- `@to-coda`
  - Render as `To` plus the coda symbol.
  - In measure interior: center the symbol block on the immediately preceding event.
  - At measure end: center the symbol block on the right barline.

### Global Merge Rule

- Navigation declarations remain global bar-level metadata.
- If different tracks declare compatible start-side and end-side navigation on the same logical bar, they merge.
- If different tracks declare incompatible start-side navigation on the same logical bar, that is a hard error.
- If different tracks declare incompatible end-side navigation on the same logical bar, that is a hard error.

### Deprecation Rule

- `@da-capo` and `@dal-segno` are immediately invalid in this revision.
- Parsers must reject them with a directive to use `@dc` or `@ds`.

### Examples

```drummark
HH | @segno x x x x |
```

Valid. `segno` is a start-side marker anchored to the measure start.

```drummark
HH | x @segno x x x |
```

Valid. `segno` is anchored to the following event.

```drummark
HH | @coda x x x x |
```

Valid. `coda` is a start-side marker anchored to the measure start.

```drummark
HH | x x x @fine |
```

Valid. `fine` is an end-side instruction and forces a final barline.

```drummark
HH | x x x @dc |
```

Valid. `dc` is an end-side instruction and forces a double barline unless the bar already ends with a final barline.

```drummark
HH | x x @to-coda x |
HH | x x x @to-coda |
```

Both are valid. The first anchors `to-coda` to the preceding event; the second anchors it to the right edge.

## Addendum 2026-04-30D: Positional Navigation Canonical Schema Clarification

### Status

Proposed

### Scope

This addendum operationalizes Addendum 2026-04-30C and explicitly supersedes any incompatible multi-marker examples or rules from Addendum 2026-04-30B.

### Superseded Prior Cases

- The following previously legalized combinations are no longer valid:
  - `segno` plus `coda` on one logical bar
  - `fine` plus `to-coda` on one logical bar
  - `fine` plus `dc` or `ds` family instructions on one logical bar
- Addendum 2026-04-30B remains in force only where it does not conflict with 2026-04-30C or this clarification.

### Canonical Navigation Schema

- Canonical normalized measures must expose positional navigation through two explicit fields:
  - `startNav?: StartNav`
  - `endNav?: EndNav`
- `StartNav`:
  - `kind: "segno" | "coda"`
  - `anchor: "left-edge" | { eventAfter: Fraction }`
- `EndNav`:
  - `kind: "fine" | "dc" | "ds" | "dc-al-fine" | "dc-al-coda" | "ds-al-fine" | "ds-al-coda" | "to-coda"`
  - `anchor: "right-edge" | { eventBefore: Fraction }`
- `Fraction` here refers to the rhythmic start position of the anchored event within the normalized measure.
- Event-relative navigation anchors attach to rhythmic position, not to one specific expanded event instance inside a chord, combined hit, or multi-track brace.

### Position Legality After Navigation Extraction

- Position legality is evaluated after removing navigation tokens from the measure token stream.
- Therefore:
  - `@segno` at start means no non-navigation token precedes it.
  - `@segno` in interior means at least one non-navigation token follows it.
  - `@coda` at start means no non-navigation token precedes it.
  - `@fine`, `@dc`, `@ds`, `@dc-al-fine`, `@dc-al-coda`, `@ds-al-fine`, and `@ds-al-coda` at end mean no non-navigation token follows them.
  - `@to-coda` in interior means at least one non-navigation token precedes it.
  - `@to-coda` at end means no non-navigation token follows it.
- On shorthand measures such as `%`, `%%`, or multi-rest, the shorthand token counts as a non-navigation token for these legality checks.

### Merge Rule

- A logical bar may merge declarations from multiple tracks only if they resolve to at most one `startNav` and at most one `endNav`.
- Compatible duplicates collapse:
  - same `kind`
  - same `anchor`
- Incompatible declarations are a hard error:
  - two different start-side kinds
  - two different end-side kinds
  - same kind with different anchors

### Barline Interaction Rule

- `@fine`, `@dc`, `@ds`, `@dc-al-fine`, `@dc-al-coda`, `@ds-al-fine`, and `@ds-al-coda` are not permitted on measures whose right boundary is already a repeat end:
  - `repeat-end`
  - `repeat-both`
  - compact repeat-end-plus-volta forms such as `:|2.`
- This is a hard error, not a precedence fight.
- `@to-coda` is permitted on repeat-ending measures because it does not force a barline rewrite.

### Barline Forcing Semantics

- `@fine` changes the canonical right barline of that logical measure to `final`, even if later logical measures still exist in the score.
- `@dc`, `@ds`, `@dc-al-fine`, `@dc-al-coda`, `@ds-al-fine`, and `@ds-al-coda` change the canonical right barline of that logical measure to `double` unless an explicit `final` or `double` barline is already present.
- Existing explicit `final` remains `final`.
- `|.` remains only a volta terminator and does not itself create `double` or `final`.

### Renderer Fallback Rule

- Canonical semantics are determined only by `startNav` / `endNav` and their anchors.
- When a renderer cannot physically attach a left-edge symbol to a visible left barline because the measure begins a new rendered system, it may fall back visually to the first event position while preserving the canonical `left-edge` anchor semantics.

### Examples

```drummark
HH | @segno x x x x |
```

Canonical metadata:

```json
{
  "startNav": { "kind": "segno", "anchor": "left-edge" }
}
```

```drummark
HH | x @segno x x x |
```

Canonical metadata:

```json
{
  "startNav": { "kind": "segno", "anchor": { "eventAfter": { "numerator": 1, "denominator": 4 } } }
}
```

```drummark
HH | x x @to-coda x |
```

Canonical metadata:

```json
{
  "endNav": { "kind": "to-coda", "anchor": { "eventBefore": { "numerator": 1, "denominator": 4 } } }
}
```

```drummark
HH | x x x @fine |
```

Canonical metadata:

```json
{
  "endNav": { "kind": "fine", "anchor": "right-edge" },
  "barline": "final"
}
```

## Addendum 2026-04-30E: Tagged Navigation Union Refinement

### Status

Proposed

### Scope

This addendum replaces the over-permissive navigation schema in Addendum 2026-04-30D with kind-specific tagged unions. Where D and this addendum differ, this addendum controls.

### Refined Canonical Schema

- `StartNav` must be exactly one of:
  - `{ kind: "coda", anchor: "left-edge" }`
  - `{ kind: "segno", anchor: "left-edge" | { eventAfter: Fraction } }`
- `EndNav` must be exactly one of:
  - `{ kind: "to-coda", anchor: "right-edge" | { eventBefore: Fraction } }`
  - `{ kind: "fine", anchor: "right-edge" }`
  - `{ kind: "dc", anchor: "right-edge" }`
  - `{ kind: "ds", anchor: "right-edge" }`
  - `{ kind: "dc-al-fine", anchor: "right-edge" }`
  - `{ kind: "dc-al-coda", anchor: "right-edge" }`
  - `{ kind: "ds-al-fine", anchor: "right-edge" }`
  - `{ kind: "ds-al-coda", anchor: "right-edge" }`

### Consequence

- The following canonical states are impossible and therefore invalid:
  - `{ kind: "coda", anchor: { eventAfter: ... } }`
  - `{ kind: "fine", anchor: { eventBefore: ... } }`
  - `{ kind: "dc", anchor: { eventBefore: ... } }`
  - `{ kind: "ds", anchor: { eventBefore: ... } }`
  - any `dc/ds-al-*` form with `{ eventBefore: ... }`

### Review Round 5

- This tagged-union refinement is intended to close the final schema gap from Review Round 4 by making legal anchor kinds enforceable directly in canonical IR shape rather than only by surrounding prose.

## Addendum 2026-04-30F: Pure Navigation Measure Default Anchors

### Status

Proposed

### Scope

This addendum refines positional legality for measures that contain navigation syntax only and no non-navigation content tokens after navigation extraction.

### Rule

- If a measure contains no non-navigation content tokens after navigation extraction, it is a pure navigation measure.
- In a pure navigation measure:
  - `@segno` and `@coda` default to measure-start semantics.
  - `@fine`, `@to-coda`, `@dc`, `@ds`, `@dc-al-fine`, `@dc-al-coda`, `@ds-al-fine`, and `@ds-al-coda` default to measure-end semantics.
- Therefore the following are valid in pure navigation measures:
  - `| @segno |`
  - `| @coda |`
  - `| @fine |`
  - `| @to-coda |`
  - `| @dc |`
  - `| @ds |`

### Canonical Consequence

- In pure navigation measures:
  - `@segno` normalizes to `{ kind: "segno", anchor: "left-edge" }`
  - `@coda` normalizes to `{ kind: "coda", anchor: "left-edge" }`
  - `@fine` normalizes to `{ kind: "fine", anchor: "right-edge" }`
  - `@to-coda` normalizes to `{ kind: "to-coda", anchor: "right-edge" }`
  - `@dc` normalizes to `{ kind: "dc", anchor: "right-edge" }`
  - `@ds` normalizes to `{ kind: "ds", anchor: "right-edge" }`

### Non-Change

- Cardinality rules remain unchanged:
  - at most one start-side navigation marker
  - at most one end-side navigation instruction

## Addendum 2026-04-30G: Implicit Repeat-End for Intermediate Voltas

### Status

Proposed

### Scope

This addendum refines repeat and volta interaction for alternate endings whose engraved right barline should show a backward repeat even when the source omits an explicit `:|`.

### Rule

- If a measure is inside a volta and its right boundary immediately opens a different following volta, that measure is an intermediate ending.
- An intermediate ending must normalize as if its right boundary were a repeat end.
- This rule applies whether the source wrote the repeat explicitly:
  - `|1. ... :|2. ...`
- or omitted it:
  - `|1. ... |2. ...`
- The inferred repeat-end uses the default repeat count `2` unless an explicit repeat count syntax is present.

### Final Ending Rule

- The last volta in a repeated section does not receive inferred repeat-end semantics.
- Its closing barline remains exactly the user-written boundary:
  - `|` stays regular
  - `||` stays double
  - `|.` stays volta terminator
  - `||.` stays double plus volta terminator
- If the last volta is also the final measure of the score and no explicit terminal barline was written, the existing end-of-score final-barline normalization still applies.

### Canonical Consequence

- `|: A |1. B |2. C |`
  normalizes barline intent as:
  - bar 1: `repeat-start`
  - bar 2: `repeat-end`, volta `[1]`
  - bar 3: `final`, volta `[2]`
- `|: A |1. B :|2. C |3. D ||`
  normalizes barline intent as:
  - bar 1: `repeat-start`
  - bar 2: `repeat-end`, volta `[1]`
  - bar 3: `repeat-end`, volta `[2]`
  - bar 4: `double`, volta `[3]`

### Repeat-Span Consequence

- Multi-ending alternate endings may produce multiple canonical repeat spans that share the same `repeat-start`.
- Therefore `|: A |1. B :|2. C :|3. D ||` yields two repeat spans:
  - start bar 1 -> end bar 2
  - start bar 1 -> end bar 3
- The final ending exits the repeated section without adding another backward-repeat span unless one is explicitly written.

## Addendum 2026-04-30H: Clarification of Addendum G

### Status

Proposed

### Scope

This addendum refines Addendum 2026-04-30G and controls wherever G was ambiguous.

### Clarification 1: `|.` and `||.` Are Excluded from Implicit Repeat-End Inference

- Implicit repeat-end inference applies only when a volta measure's right boundary is a plain barline that immediately opens a different following volta.
- Therefore this inference applies to:
  - `|1. ... |2. ...`
  - `|2. ... |3. ...`
- This inference does not apply to right boundaries that are already explicit volta terminators:
  - `|.`
  - `||.`
- So `|1. ... |. |2. ...` is not a valid surrogate for `|1. ... :|2. ...`.
- The only valid meanings of `|.` and `||.` remain:
  - terminate the current volta bracket
  - optionally carry a double-barline appearance in the `||.` case

### Clarification 2: No New Repeat-Count Syntax Is Introduced

- Addendum G does not introduce any new repeat-count syntax or metadata field.
- Implicit repeat-end inference produces canonical repeat-end semantics equivalent to a plain `:|`.
- Therefore the inferred repeat uses the existing default backward-repeat count of `2`, exactly as plain `:|` already does elsewhere in the specification.

### Non-Change

- Explicit compact syntax remains valid and unchanged:
  - `:|2.`
  - `:|3.`
  - `:|1,2.`
- The last volta in a repeated section still keeps the user-written closing boundary and does not receive implicit repeat-end semantics.

## Addendum 2026-05-02: Base Rhythmic Unit and Paragraph Overrides

### Status

Proposed

### Scope

This addendum introduces the `note 1/N` syntax to define the base rhythmic unit (grid resolution), deprecates the `divisions` header, and enables paragraph-level overrides.


### New Syntax: `note 1/N`

- The `note 1/N` syntax explicitly defines the musical duration of a single grid slot (one character column or one basic token like `d` or `-`).
- `N` must be a power of 2 ($2^n$ where $n \in \{0, 1, 2, 3, 4, 5, 6, 7\}$).
- Supported values for `N`: `1`, `2`, `4`, `8`, `16`, `32`, `64`, `128`.
- Examples:
  - `note 1/4` (Quarter note grid)
  - `note 1/8` (Eighth note grid)
  - `note 1/16` (Sixteenth note grid)

### Scoping Rules

- **Global Header**: `note 1/N` can be used in the document header. It sets the default grid resolution for the entire score.
- **Paragraph Override**: A line containing only `note 1/N` at the very beginning of a paragraph (immediately following a blank line) overrides the global setting for that paragraph only.
- Subsequent paragraphs revert to the global header value unless they also contain an explicit override.

### Deprecation of `divisions`

- The `divisions` header is deprecated in favor of `note 1/N`.
- For backward compatibility, `divisions X` will be interpreted as `note 1/N` where $1/N = \text{MeasureDuration} / X$, provided $N$ is a supported power of 2.
- If $N$ calculated from `divisions` is not a power of 2, the compiler should issue a warning or error depending on the strictness level.

### Rhythmic Validation

- The number of slots required to fill a measure is calculated dynamically:
  - `ExpectedSlots = MeasureDuration / (1/N)`
- For a 4/4 measure (Duration = 1) with `note 1/16`, `ExpectedSlots = 16`.
- For a 6/8 measure (Duration = 3/4) with `note 1/16`, `ExpectedSlots = 12`.
- A measure is valid if and only if the sum of all token durations (expressed in fractions of a whole note) equals the `MeasureDuration`. This is equivalent to ensuring the total weight of tokens equals `ExpectedSlots`.

### Canonical Representation

- The `NormalizedHeader` and IR will store `noteValue: number` (the denominator $N$).
- `divisions` may be omitted from IR if `noteValue` is present, or kept as a derived value for specific consumers.
- Each `NormalizedMeasure` or `ScoreParagraph` must carry its effective `noteValue`.

## Addendum 2026-05-02-B: Clarification of Rhythmic Units in Groups

### Status

Proposed

### Scope

This addendum clarifies the definition of the `span` parameter in Rhythmic Groups (`[span: ...]`) following the introduction of the `note 1/N` syntax.

### Redefinition of "Slot" as "Note Unit"

- With the deprecation of the global `divisions` header, the term "slot" or "grid unit" is formally redefined as a **Note Unit**.
- One **Note Unit** is equivalent to the duration defined by the active `note 1/N` setting (e.g., if `note 1/16` is active, 1 Note Unit = 1/16th note).

### Interpretation of `span` in Groups

- In the group syntax `[span: item1 item2 ...]`, the `span` integer represents the number of **Note Units** (slots) the group occupies.
- **Formula Update**: Each item's absolute duration is calculated as:
  `ItemDuration = NoteDuration * (span / itemCount)`
  where `NoteDuration` is the duration of the active base note (e.g., 1/16).
- **Example**: Under `note 1/16`, the group `[2: d d d]` occupies 2 Note Units (which equals two 1/16th notes, or one 1/8th note). The three `d` tokens are compressed into that duration as a triplet.

### Supersession

- This definition supersedes the descriptions in **Section 5.4 (Rhythmic Math)** and **Section 5.5 (Groups)** regarding the relationship between token weights and `divisions`. 
- Validation should now compare total measure weight against the expected slot count: `MeasureDuration / NoteDuration`.

## Addendum 2026-05-02-C: Duration Multiplication Modifier

### Status

Proposed

### Motivation

The existing duration modifiers (`/` for halving, `.` for dotting/1.5×) have no symmetric inverse for doubling. Users wanting a note with twice the base duration must write `[2: d]` which is verbose. A lightweight doubling modifier would complete the modifier set.

### Proposal

Introduce `*` (asterisk) as a duration multiplication modifier that multiplies the base token weight by powers of 2.

| Symbol | Effect |
|--------|--------|
| `*` | Multiplies duration by 2^n where n is the number of stars. |


**Formula Update**:
```
weight = base × (2 - 0.5^dots) × (2^stars) / (2^halves)
```

**Examples**:
- `d*` = 2× duration (1 slot → 2 slots)
- `d**` = 4× duration (1 slot → 4 slots)
- `d***` = 8× duration
- `d*.` or `d.*` = 3× duration (2 × 1.5 = 3)
- `d*/` or `d/*` = 1× duration (2 × 0.5 = 1, cancels out)

### Interaction with Inline Repeat `*N`

The `*` symbol is also used for inline repeat (`*N` at measure end). Disambiguation rule:

- When parsing a mease, if `*N` is at the end of the measure, allocate the `*` before `N` for inline repeat, the rest of the content is then used for token parse.

**Key examples**:
```
| d* |           # d* is one token, doubled duration
| d* *2 |        # d* is one token (doubled), *2 is inline repeat (2 measures)
| d*3 |          # d is one token (basic), *3 is inline repeat (3 measures)
| d* - - - |     # d* is one token (doubled), rest fills remaining slots
| d* - - - *2 |  # d* is one token (doubled), rest, *2 repeats measure twice
```

### Order Independence

The three modifier characters (`*`, `.`, `/`) are all multiplicative. They can appear in any order:
- `d*.` = `d.*` = `d*` then `.` = 3× duration
- `d*/` = `d/*` = `d*` then `/` = 1× duration (cancels)
- `d./` = `d/.` = `d.` then `/` = 0.75× duration

The formula produces the same result regardless of order.

### Validation

There is no per-token star limit. A token with many `*` modifiers simply has a large weight. Measure validation ensures the total weight of all tokens in a measure does not exceed the measure capacity (`MeasureDuration / NoteDuration`). If a token's weight would cause the measure to overflow, a normal "measure duration mismatch" error is raised.
An implementation may also emit a numeric overflow diagnostic if the resulting duration implied by the modifier counts exceeds the exact arithmetic range of that implementation. This is not a syntactic star cap.


### Interaction with Rest Token

The `*` modifier is valid on the rest token `-`. A doubled rest (`-*`) occupies 2 slots of silence.


### Supersession

This addendum supplements **Section 5.3 (Duration Modifiers)** and **Section 5.4 (Rhythmic Math)** with a complete symmetric modifier set: halving (`/`), base, dotting (`.`), and doubling (`*`).

### Examples

| Token | Base | Stars | Weight | Notes |
|-------|------|-------|--------|-------|
| `d` | 1 | 0 | 1 | |
| `d*` | 1 | 1 | 2 | doubled |
| `d**` | 1 | 2 | 4 | quadrupled |
| `d***` | 1 | 3 | 8 | octupled |
| `d*.` | 1 | 1 | 3 | doubled × dotted |
| `d.*` | 1 | 1 | 3 | same (order independent) |
| `d./` | 1 | 0 | 0.75 | dotted then halved |
| `d*/.` | 1 | 1 | 1.5 | doubled × dotted then halved |
| `d/*.` | 1 | 1 | 1.5 | same (order independent) |
| `-*` | 1 | 1 | 2 | doubled rest |
| `d*3` | 1 | 1 | 2 | doubled note |

## Addendum v1.4: Trailing Modifiers and Single-Note Group Modifier Attachment

### Motivation

Two related features are needed:

1. **Trailing modifiers after duration suffixes**: The `:modifier` syntax (`:accent`, `:ghost`, etc.) should work after `./*` duration suffixes. Currently `d.*:accent` fails because the parser consumes `.*` first, then `:accent` is seen as an unknown token. Users must write `d:accent.*` instead, which is counterintuitive for combining articulation modifiers with duration modifiers.

2. **Group modifier attachment**: A `[N:note]` group can accept `:modifiers` after the closing bracket. Trailing modifiers apply to all notes inside the group. This allows `[2:s]:flam` (snare with flam, stretched to 2 slots) and `[2:dd]:accent` (both drums accented).

### Proposed Syntax

**Unified Token Structure:**
```
Glyph ( Suffix )*
```

- **Glyph**: The base note (`d`, `x`, `-`, etc.) or a Summoned Hit (`s`, `b`, etc.).
- **Suffix**: Any duration modifier (`.`, `*`, `/`) or articulation modifier (`:name`).
- **Order Independence**: Suffixes can appear in any order and can be interleaved. All duration modifiers are multiplied together, and all articulation modifiers are collected into the event's modifier set.

**Examples of Identical Tokens:**
- `d:accent.*` == `d.*:accent`
- `d:ghost.*/:accent` == `d:ghost:accent.*/` == `d.*/:ghost:accent`

**Modifier Chaining**: Modifiers chain via `:` on both basic tokens and group suffixes. `d:flam:accent` and `d:ghost:drag` are both valid — every `:name` after the first is a post-modifier attached to the preceding glyph or group.

**Group Modifier Attachment:**
```
[N:content]( Suffix )*
```
- Trailing suffixes apply to **all notes** inside the group.
- If the group contains a rest, articulation modifiers are a syntax error.

### Examples

| Token | Interpretation |
|-------|----------------|
| `d.*:accent` | note `d`, doubled, dotted, with `:accent` |
| `d:accent.*` | same as above |
| `d:ghost:accent.*/` | note `d`, doubled and dotted (3x), then halved (1.5x), with `:ghost` and `:accent` |
| `[2:s]:flam` | single note `s` stretched to 2 slots, with `:flam` |
| `[2:s.*]:rim` | single note `s` dotted+doubled, stretched to 2 slots, with `:rim` |
| `[2:s+b]:accent` | combined hit `s+b` stretched to 2 slots, each note gets `:accent` |
| `[2:dd]:accent` | two notes `d` stretched to 2 slots, both get `:accent` |
| `[2:-]:accent` | **invalid** — rests cannot have articulation modifiers |

## Addendum 2026-05-04: String Quoting for Text Headers

### Status

Proposed

### Scope

This addendum refines Section 3.1 (Supported Header Fields) to require that `title`, `subtitle`, and `composer` values be quoted strings. It also announces the future deprecation of the regex-based parser.

### String Quoting Rule

- `title`, `subtitle`, and `composer` header values MUST be enclosed in single quotes (`'...'`) or double quotes (`"..."`).
- Unquoted values are no longer supported and will produce a parse error.

**Valid**:
```
title "Funk Study No. 1"
subtitle 'Verse groove'
composer "G. Mao"
```

**Invalid** (will produce a hard error):
```
title Funk Study No. 1
subtitle Verse groove
composer G. Mao
```

### Why

The regex-based DSL parser previously accepted unquoted title/subtitle/composer values by consuming remaining text on the line. The Lezer grammar parser requires explicit string tokens for text values to avoid ambiguity with other keywords and token types. The quoting requirement unifies both parsers and eliminates edge cases where trailing text could be interpreted as DSL tokens.

### Canonical IR

The IR `Header` type already stores `title`, `subtitle`, and `composer` as `string` fields. The quoting is a syntactic requirement at the DSL level only — the IR is unchanged. Canonical IR string values do not include the quote delimiters.

### Regex Parser Deprecation

- The regex-based parser (`src/dsl/regex/`) is deprecated and will be removed in a future release.
- All new development targets the Lezer grammar parser exclusively.
- Users should migrate to the Lezer parser if not already using it (it is the default in the current editor).
- The spec examples have been updated to reflect quoted string syntax.

### Supersession

This addendum supersedes the `title <text>`, `subtitle <text>`, and `composer <text>` syntax descriptions in Section 3.1, which previously implied unquoted free text.

## Addendum 2026-05-06: Lezer Grammar Formalization and Free-Text Header Restoration

### Status

Approved

### Scope

This addendum restores unquoted free-text support for `title`, `subtitle`, and `composer`, and formalizes the parser boundary for local DrumMark syntax that must be represented directly in the Lezer grammar rather than reconstructed in `lezer_skeleton.ts`.

### Free-Text Header Rule

- `title`, `subtitle`, and `composer` accept either quoted or unquoted values.
- In the unquoted form, the header value is the remainder of the line after the keyword, trimmed of leading and trailing whitespace.
- In the unquoted form, `#` starts a comment. Therefore literal `#` requires quoted syntax.
- Empty values are invalid.

**Valid**:
```txt
title Backbeat Study
subtitle with ghost notes
composer "C# Minor"
```

**Invalid**:
```txt
title
composer
```

### Grammar Ownership of Local Syntax

The Lezer grammar must structurally represent the following local syntax forms:

- summon prefix, e.g. `SD:d`
- routed block directive, e.g. `@RC { x x x x }`
- rhythmic group structure, including optional span and trailing group modifiers
- inline repeat suffix `*N` at measure level
- multi-measure rest dash-run/integer/dash-run form
- paragraph-leading `note 1/N` override
- local barline classes

These forms must not be reconstructed from raw-text rescans, measure-content regexes, or source-gap scanning in `lezer_skeleton.ts`.

### Inline Repeat

- `*N` is a measure-level trailing construct.
- It may appear either after normal measure content or as the sole content of an otherwise empty measure body.
- `d*`, `d**`, etc. remain token-local duration modifiers.

Examples:
```txt
| d* *2 |
| - *3 |
| *4 |
```

### Multi-Measure Rest

- Multi-rest is a mutually exclusive measure-body form.
- It cannot be combined with ordinary measure tokens, measure-repeat shorthand, or inline repeat.
- Existing navigation directives may still coexist with a multi-rest measure, subject to the navigation-placement rules elsewhere in this spec.
- Additional adjacent tokens or suffixes are parse errors.

### Paragraph `note 1/N` Override

- A paragraph-level `note 1/N` override attaches to the paragraph it immediately precedes.
- It may appear only as the first non-comment, non-blank content of that paragraph.
- At most one override may precede a paragraph's track content.
- An override appearing after the first track line of a paragraph is a parse error.

### Local Barline Mapping

The grammar-local barline mapping is:

- `|` -> `RegularBarline`
- `||` -> `DoubleBarline`
- `|:` -> `RepeatStartBarline`
- `:|` -> `RepeatEndBarline`
- `|.` -> `VoltaTerminatorBarline`
- `||.` -> `DoubleVoltaTerminatorBarline`
- `|N.` / `|N,M.` -> `VoltaBarline(indices=[...], base="|")`
- `|:N.` / `|:N,M.` -> `VoltaBarline(indices=[...], base="|:")`
- `:|N.` / `:|N,M.` -> `VoltaBarline(indices=[...], base=":|")`

Malformed volta barlines are parse errors.

### Boundary Between Grammar and Skeleton

The grammar is responsible for local syntax shape. The skeleton remains responsible for contextual lowering and semantic behavior, including:

- anonymous-track fallback resolution
- navigation legality and anchor derivation
- inferred repeat-end behavior for intermediate voltas
- grouping defaulting
- duration and grouping-boundary validation

### Supersession

This addendum supersedes Addendum 2026-05-04 ("String Quoting for Text Headers") and any implementation strategy that relies on reconstructing the syntax forms listed above from raw text in `lezer_skeleton.ts`.

## Addendum: Relaxed Multi-Measure Rest Spelling

Multi-rest is written as a left dash run, an integer count, and a right dash run:

```txt
HH | --8-- |
HH | -- 8 -- |
HH | ---8---- |
HH | ---- 12 -- |
```

This addendum supersedes the earlier exact-shape `--N--` rule and the later symmetric compact spelling with a broader relaxed family.

Rules:

- The left dash run must contain at least two `-` characters.
- The right dash run must contain at least two `-` characters.
- The left and right dash runs do not need to have equal length.
- Optional horizontal whitespace may appear between the left dash run and the integer.
- Optional horizontal whitespace may appear between the integer and the right dash run.
- `N` is an unsigned decimal integer.
- `N` must be at least `2`.
- Leading zeros are not allowed.

Examples of invalid syntax:

```txt
HH | - 4 - |
HH | --1-- |
HH | --08-- |
HH | -- +8 -- |
```

Measure exclusivity:

- Multi-rest occupies the entire non-navigation rhythmic content of the measure.
- It may not be combined with ordinary note tokens, groups, combined hits, summon prefixes, braced blocks, `%`, `%%`, another multi-rest, or inline repeat suffix syntax.
- Existing navigation directives may still coexist with a multi-rest measure, subject to the navigation-placement rules elsewhere in this spec.

Grammar ownership:

- Multi-rest is a dedicated whole-measure shorthand form.
- Legal multi-rest exists if and only if the measure body matches the multi-rest rule.
- Inputs that do not match that rule are simply not multi-rest; the implementation does not need to guess user intent or define a separate malformed-candidate class.

## Addendum: Explicit `@TRACK { ... }` Routed Block Syntax

Long-span routed blocks use explicit directive syntax:

```txt
@RC { x x x x }
@SD { [3: d d d] }
@BD { - d - d }
```

Rules:

- `@` immediately introduces a routed-block directive.
- The token after `@` must be a valid `TrackName`.
- The directive applies only to the immediately following braced block.
- Canonical spelling is `@TRACK { ... }`.
- Horizontal whitespace between `@TRACK` and `{` is allowed.
- A newline or comment between `@TRACK` and the following `{ ... }` block is not allowed.
- If `@TRACK` is not followed by a braced block on the same logical line, it is a parse error.

Namespace and semantics:

- `@TrackName { ... }` is a routed-block directive class.
- `@segno`, `@coda`, `@fine`, `@dc`, `@ds`, `@dc-al-fine`, `@dc-al-coda`, `@ds-al-fine`, `@ds-al-coda`, and `@to-coda` remain the full navigation directive class.
- There is no generic `@Identifier` category.
- Any other `@...` form is a parse error.
- Routed-block directives are not navigation markers or jumps.
- They do not participate in navigation placement legality.
- They do not produce navigation anchors.

Scope:

- A routed-block directive is a measure-expression form.
- It is legal anywhere an inline braced block measure expression is legal.
- It is legal in measure content and nested braced measure content.
- It is not a valid group item inside `[ ... ]`, unless a future addendum explicitly extends group-item syntax.
- Measure-level suffix forms such as `*N` still apply only at measure-body level and are unaffected by the routed-block syntax itself.

Track registration:

- Any track mentioned via line header (`SD | ... |`), routed block directive (`@RC { ... }`), or summon prefix (`SD:d`) is automatically registered in the score.
- Automatic registration for routed blocks occurs only for syntactically complete routed-block directives.

Removed syntax:

```txt
RC { x x x x }
SD { [3: d d d] }
```

These legacy bare routed-block forms are removed and should produce a dedicated migration diagnostic equivalent to:

```txt
Legacy routed block syntax `RC { ... }` has been removed; use `@RC { ... }` instead.
```

## Addendum 2026-05-06-D: Crescendo / Decrescendo Hairpins

Syntax:

- `<` starts a crescendo hairpin.
- `>` starts a decrescendo / diminuendo hairpin.
- `!` ends the active hairpin at the current rhythmic position.

Hairpin tokens are zero-duration measure expressions. They are legal in ordinary measure content, inline braced content, and rhythmic groups `[ ... ]` / `[N: ... ]`.

Normalization:

- Hairpins normalize to `NormalizedMeasure.hairpins?: HairpinIntent[]`.
- `HairpinIntent` has shape `{ type: "crescendo" | "decrescendo"; start: Fraction; end: Fraction }`.
- `start` and `end` use the same musical-time `Fraction` convention as `NormalizedEvent.start`.
- If a hairpin is not explicitly closed with `!`, it closes at the end of the current measure and carries forward into the next measure as the same active type.
- Carry-forward propagates across paragraph boundaries. Rendering may split at system boundaries, but semantic carry-forward is not reset by layout.
- Hairpin tokens inside rhythmic groups participate in the same measure-level state machine as ordinary measure tokens. They consume no rhythmic weight and do not advance group-local position.

Validation:

- At most one hairpin start position may be declared globally per measure position.
- Same-type declarations at the same position across tracks collapse to one logical hairpin declaration.
- Different hairpin types at the same position across tracks are an error.
- Hairpin declarations at different positions across tracks within the same logical fragment are an error.
- A group is invalid if, after filtering out hairpin tokens, it contains no duration-consuming items.

Rendering:

- Hairpins are rendered through VexFlow primitives, specifically `StaveHairpin`.
- Multi-measure hairpins may merge only within the same rendered system.
- A system break splits rendering into multiple `StaveHairpin` segments while preserving semantic continuity.
- Manual SVG / HTML / Canvas simulation of hairpin wedges is not part of the score-rendering model.

MusicXML:

- Hairpins export as `<direction><direction-type><wedge .../></direction-type></direction>`.
- `<` maps to `type="crescendo"`, `>` maps to `type="diminuendo"`.
- Explicit or implicit termination maps to `type="stop"`.
- Multi-measure spans use `crescendo` / `diminuendo` in the first measure, `continue` in intermediate measures, and `stop` in the terminating measure.

## Addendum 2026-05-07: Future Feature Lane Classification

### Status

Applied — documents the classification of Appendix C features by implementation readiness.

### Scope

Classifies each Appendix C item and the rehearsal marks proposal into one of four lanes. This addendum does not change any feature specification; it only records implementation-readiness metadata.

### Classification

| Lane | Definition |
|------|-----------|
| **F-active** | An approved proposal stream exists; implementation tasks are pending in a separate tasks file. |
| **F-spec** | Defined in `Appendix C` but no IR/render design beyond the spec table text. Requires a separate design proposal before tasking. |
| **F-spec-rejected** | Explicitly ruled out in the spec. Documented here to prevent re-proposal without addressing the underlying constraint. |
| **F-discovery** | Feature idea not yet reaching `Appendix C`. May exist only as verbal discussion or editorial note. Empty as of this addendum. |

### Lane Assignments

| Lane | Feature | Evidence |
|------|---------|----------|
| **F-active** | Rehearsal marks (`[label]`) | Approved proposal: `docs/proposals/DRUMMARK_SPEC_proposal_rehearsal_marks.md`; tasks: `docs/proposals/DRUMMARK_SPEC_tasks_rehearsal_marks.md` |
| **F-spec** | Inline tempo change (`@tempo:<N>`) | `Appendix C`; no grammar, no IR, no source mentions |
| **F-spec** | Inline time-signature change (`@time:<N/M>`) | `Appendix C`; no grammar, no IR, no source mentions |
| **F-spec** | Pickup/anacrusis measure (`@partial:<N>`) | `Appendix C`; no grammar, no IR, no source mentions |
| **F-spec** | Dashed barline (`dashed`) | `Appendix C`; previously in legacy `BarlineType` but intentionally removed from current `src/dsl/types.ts` |
| **F-spec-rejected** | Inline divisions change (`@divisions:<N>`) | `Appendix C`; explicitly marked "Rejected" |
| **F-discovery** | (none) | No feature ideas currently in the discovery lane |

## Addendum 2026-05-08: Volta-Terminator + Repeat-Start Coalescing (`|:.`)

### Status

Proposed

### Motivation

When a repeat section contains voltas, a common scenario is "volta ends, new repeat starts":

```
|: d d d d |1. d d d d :|2. d d d d |. |: d d d d :|
```

Currently `|.` (volta terminator) and `|:` (repeat start) are two separate `MeasureSection` nodes, creating a spurious empty measure between the volta ending and the new repeat. No syntax exists to express both at the same barline boundary.

### Syntax

```
|:.
```

The compound barline follows the existing `|:X.` pattern:

| Token | Meaning |
|-------|---------|
| `|:` | repeat-start |
| `|:1.` | repeat-start + open volta 1 |
| `|:2.` | repeat-start + open volta 2 |
| **`|:.`** | **repeat-start + terminate volta** |
| `|.` | terminate volta (without repeat-start) |

**Example:**

```
|: d d d d |1. d d d d :|2. d d d d |:. d d d d :|
```

### Semantics

`|:.` carries both semantics simultaneously:

| Property | Value |
|----------|-------|
| `openRepeatStart` | `true` |
| `closeVoltaTerminator` | `true` |
| `closeBarlineType` | `"repeatStart"` |
| `closeRepeatEnd` | `false` |

**Sharp Edge**: `|:.` does **NOT** close any open repeat. The prior repeat MUST be explicitly closed with `:|` before `|:.` appears, or a "nested repeat start" error will fire. Example:

```drummark
|: A |1. B |:. C :|              // ERROR: nested repeat start
|: A |1. B :|2. C |:. D :|       // OK: first repeat closed by :| after bar 1
```

### Implicit Repeat-End Exclusion

`|:.` inherits the existing rule from Addendum 2026-04-30H: `|.` does NOT trigger implicit repeat-end inference. The compound's `closeBarlineType` is `"repeatStart"` (not `"single"`), so it automatically fails the inference guard in the skeleton builder.

### Local Barline Mapping Extension

Extends the Local Barline Mapping from Addendum 2026-05-06 with:

- `|:.` -> `VoltaTerminatorRepeatStartBarline`

### Grammar Rule

`VoltaTerminatorRepeatStartBarline { "|:." }` placed before `VoltaBarline` and `RepeatStartBarline` in `BarlineNode` alternatives.

## Addendum v1.6: Rust/WASM Parser Backend

### Architecture

A third parser backend (`"wasm"`) is implemented as a Rust crate (`crates/drummark-core`) compiled to WebAssembly. The parser is a hand-written recursive descent parser using the `logos` crate for tokenization. The WASM output uses `wasm-bindgen` + `js-sys` for direct JS object construction (no JSON serialization).

### Tokenization

- Logos-based lexer with longest-match disambiguation
- ~130 token variants covering all grammar tokens
- Error characters emitted as `FreeText` tokens for header values and unknown content

### Pipeline Integration

`ast.ts` accepts `parseMode: "lezer" | "regex" | "wasm"` (default: `"lezer"`). WASM is initialized on application startup (`main.tsx`) and via `initSync` for Node.js test environments.

### Native CLI

A native binary is available at `cargo run -- <file> --format json`. The binary reads DrumMark source and outputs the parsed AST as JSON.

### WASM Size

- Compiled WASM: ~101KB uncompressed, ~34KB gzipped
- JS glue: ~11KB
- No `serde` / `serde_json` dependency
