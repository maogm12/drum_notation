# Learnings

## 2026-04-30

- Static docs examples should copy from the source DSL string, not from highlighted `<pre>` HTML. The stable pattern here is to emit the original example text as encoded `data-copy` at build time, then let the browser decode and copy that payload on button click.
- `navigator.clipboard.writeText()` is not sufficient for docs pages served over LAN `http` such as `http://192.168.x.x:5173`. Those contexts commonly fail clipboard writes on mobile browsers, so docs copy buttons need a `document.execCommand("copy")` fallback wired into the same click gesture.
- Navigation labels like `Fine`, `D.C.`, and `D.S.` must not be rendered with the music glyph font `Bravura`. That causes device-dependent fallback differences across desktop and mobile. Textual navigation should use `Academico`; only actual symbols like segno/coda should stay on `Bravura`, and mixed `To Coda` rendering should be emitted as separate text-plus-glyph elements.
- Navigation collision handling is safer when it stays inside VexFlow's modifier layout. For note-adjacent end markers, attaching them as `Annotation`s on the last anchor note lets `ModifierContext` reserve horizontal space and stack vertical text lines automatically; querying note bounding boxes before `Formatter` has assigned `TickContext` will crash with `NoTickContext`.
- If navigation text must not reflow notes, render it as post-format overlays instead of note modifiers. The practical split here is: right-edge `Fine` / `D.C.` / `D.S.` and left-edge `segno/coda` should share the same overlay `Annotation` baseline (`textLine`-driven), otherwise mixing `StaveText` on one side and `Annotation` on the other produces visible vertical misalignment.
- Volta boundaries in the parser are measure-boundary syntax, not regular measure content. The parser must recognize them before token parsing on anonymous tracks, or inputs like `:|2.` degrade into a valid repeat-end followed by an invalid content token `2`.
- Anonymous-track alternate endings need the same boundary grammar as named tracks. In practice this includes both compact forms like `|2.` and musician-style spaced forms like `| 3.`.
- A repeat end may immediately introduce the next volta with no intervening barline token. `:|2.` should close the previous repeated measure and seed the next measure's left boundary with volta indices `[2]`.
- `|.` should be treated as a pure volta terminator. Final barlines belong to score normalization on the actual last measure, not to the `|.` token itself.
- Cross-system volta rendering depends on normalized canonical metadata spanning the whole ending, not just the first measure that declared `|1.` or `|2.`. Once the active volta is propagated until `|.`, a new volta, or a repeat end, existing VexFlow `BEGIN/MID/END` rendering continues the bracket across systems correctly.

## 2026-04-30 Addendum: Chained Measure Repeat

- Change: chained measure-repeat shorthand is allowed. A measure-repeat bar may reference preceding bars even if one or more of those preceding bars are themselves measure-repeat bars.
- Semantics: `%` copies the immediately preceding canonical musical result. `%%` copies the previous two canonical musical results as heard/read after normalization, not merely the previous two source measures by syntax class.
- Example: `| A | % | %% |` is valid and yields canonical playback/notation equivalent to `| A | A | A |`.
- Non-change: `%` and `%%` still must occupy the entire measure, and `%%` still requires two preceding canonical measures to exist.
- Metadata rule: measure-repeat expands musical content only. Structural metadata such as the current barline, volta, marker, and jump remain attached to the current bar and are not inherited from the referenced bars.

## 2026-04-30 Addendum: Multi-Marker Navigation

- Navigation markers are no longer singleton metadata. Canonical score data needs an ordered `markers` array, not a single `marker` field, or inputs like `@coda ... @fine` cannot be represented without loss.
- Cross-track marker merge is set union, not conflict. The canonical ordering is fixed as `segno`, `coda`, `fine`, which keeps renderer snapshots and MusicXML emission deterministic even when source token order differs.
- Jumps remain singleton metadata. The valid matrix is now “any marker set plus at most one jump”, so parser and AST validation must stop rejecting marker-plus-marker while still rejecting jump-plus-jump.
- Marker propagation follows the same left-edge rule as volta starts: when `*N` expands one source measure into multiple logical bars, the full marker set belongs only on the first generated bar, while any jump belongs only on the last generated bar.
- Consumers must preserve the whole marker set. In practice this means MusicXML emits one direction per marker in canonical order, and VexFlow must render multiple start-side labels instead of silently dropping later markers.

## 2026-04-30 Addendum: Positional Navigation Anchors

- The multi-marker model was too loose for engraving semantics. Once navigation placement matters, canonical IR must model at most one `startNav` and at most one `endNav`, with tagged unions that encode which anchor kinds are legal for each token family.
- Position legality has to be evaluated after stripping navigation tokens but before parsing shorthand structure. `%`, `%%`, and `--N--` still count as remaining measure content for begin/end legality, while trailing `*N` behaves as an inline-repeat suffix and should not block end-side navigation.
- Pure navigation measures need an explicit fallback rule. Without it, inputs like `| @segno |` and `| @dc |` become ambiguous because there is no remaining content token to establish “beginning” or “end”; the correct default is start-side forms -> left edge, end-side forms -> right edge.
- Right-edge navigation forcing must be resolved against barlines during normalization, not in the parser. `fine` upgrades the canonical barline to `final`; `dc/ds` families upgrade to `double` unless a `double` or `final` barline is already explicit; repeat-ending bars instead hard-fail for these end-side forms.
- Event-relative anchors are safest when stored as rhythmic positions, not concrete note ids. That keeps `segno`/`to-coda` stable across combined hits, grouped tokens, and cross-track normalization.
- VexFlow `Repetition` is only partially useful for navigation engraving. `SEGNO_LEFT` is symbol-only, but `CODA_LEFT` and `TO_CODA` inject hardcoded text, and all `Repetition` text is top-of-staff only. For position-sensitive navigation, note-anchored `Annotation` modifiers are more reliable.
- Pure navigation measures still need render anchors. If the renderer only records anchor points for sounded events, start/end navigation on rest-only bars disappears; rest entries must seed the same rhythmic anchor map.
- For the current engraving rules, `fine` and the `dc/ds` family belong below the staff, while `to-coda` stays above and should be emitted as separate text-plus-glyph annotations so the coda symbol survives SVG export.

## 2026-04-30 Addendum: Grouping Boundary Units & VexFlow Header Ownership

- Grouping-boundary validation in normalize must use the same unit system as AST validation: grouping values are beat counts, while token offsets and weights are slot counts. Converting the boundary with `cumulativeGrouping * divisions / beats` avoids both false positives and false negatives.
- A legal dotted value near the middle of a `2+2` bar is a good regression probe. In `4/4` with `divisions 16`, `| - x. x. - ... |` must not be flagged as crossing the grouping boundary, even though naive beat-vs-slot comparison says it does.
- Navigation text placement in VexFlow is sensitive to `StaveText` font size and `shiftY`. For this renderer, the stable snapshot for left-edge `segno/coda` and right-edge `Fine` / `D.C.` / `D.S.` came from using native `StaveText` at `20pt` without the extra manual upward shift.
- Title, subtitle, and composer should be emitted through VexFlow objects, not `context.fillText()`. A zero-line header stave with `StaveText` modifiers satisfies the repository rule that score headers remain inside the VexFlow rendering path.
- Headless/JSDOM runs do not expose the browser `FontFace` API. VexFlow font loading should therefore no-op in that environment instead of logging an expected failure on every render test.

## 2026-04-30 Addendum: Implicit Repeat-End for Intermediate Voltas

- Engraving semantics and repeat semantics have to stay aligned for alternate endings. If a measure is inside a volta and its right boundary immediately opens a different next volta, that current measure should normalize as a `repeat-end` even when the source omitted an explicit `:|`.
- The inference applies only to intermediate endings. The last volta keeps whatever closing barline the user wrote (`|`, `||`, `|.`, `||.`), and the usual end-of-score final-barline normalization still happens later if nothing explicit was written.
- Multi-ending repeat validation cannot model voltas as a single open/close pair. A repeat start may fan out into multiple repeat spans with the same `startBar`, one for each intermediate ending (`1.`, `2.`, etc.), while the final ending exits the repeated section without another backward repeat.

## 2026-04-30 Addendum: VexFlow Stem Length Control

- VexFlow exposes per-note stem shortening through `StemmableNote#setStemLength(height)`. This is a direct absolute stem height override, not a delta.
- Beamed groups still honor that override as their starting geometry, but `Beam.applyStemExtensions()` will extend stems as needed to meet the shared beam line. So shortening works globally, but beamed notes remain constrained by beam alignment rather than a fixed identical visual height.
- If stem length is user-tunable in the editor, it should be part of the shared `VexflowRenderOptions` pipeline rather than a renderer constant, otherwise preview, PDF export, CLI output, and tests drift out of sync.

## 2026-04-30 Addendum: Editor Stem-Length Wiring

- The editor already persisted a `stemLength` render setting and exposed a slider, but the actual note-building path still has to receive that value explicitly. In this renderer the critical handoff is `renderMeasureVoices(...) -> createVexNotes(...) -> StaveNote#setStemLength(...)`.
- A UI-level option is not enough as verification. The stable regression check is to compare the exported SVG stem path for the same score under two `stemLength` values and assert the stem endpoint moves.
- In VexFlow, `StemmableNote#setStemLength()` only records `stemExtensionOverride`; it does not push that value into the already-built `Stem`. If code also calls `setStemDirection()`, the safe order is `setStemLength()` first, then `setStemDirection()`, because `setStemDirection()` is what applies `getStemExtension()` onto the live stem object.

## Legacy Docs Learnings (Merged From `docs/LEARNINGS.md`)

### 1. VexFlow 5 & Vite MPA

- VexFlow 5 font loading should use `await VexFlow.loadFonts(...fontNames)`, and active fonts are controlled by `VexFlow.setFonts(...fontNames)`.
- In the minified VexFlow 5 build used here, SMuFL mappings live at `VF.smufl.to_code_points`, and `StaveNote` notehead instances are stored on `note.note_heads`.
- Vite subpath deploys need `base` configured and should prefer relative internal HTML links for multi-page docs.
- Static docs generation in this repo runs through `npm run build-docs` in a headless JSDOM environment that pre-renders `.drum` examples into SVG.

### 2. DrumMark Spec And DSL Validation

- Header duplication, grouping consistency, and irregular-meter fallback all need parser-level protection so later stages do not inherit ambiguous header state.
- Anonymous routing, summon prefixes, global token resolution priority, and `ST` sticking semantics all have implementation-critical edge cases that should be tested at normalize time, not just parser shape time.
- Measure-level constructs such as `%`, `%%`, `|1.`, `|2.`, `@segno`, `@dc`, `--N--`, and `*N` are global structural metadata and should not be treated like ordinary inline content once parsed.
- Validation is strongest when it uses rational timing math rather than slot heuristics alone, especially for dotted values, group stretching/compression, and grouping-boundary crossing.

### 3. MusicXML And Renderer Backend

- MusicXML rests should include explicit display positions to avoid voice collisions, matching the renderer convention of placing voice 1 rests around `B/4` and voice 2 rests around `F/4`.
- Measure repeats and multi-measure rests are appearance metadata layered on top of canonical musical structure, not shortcuts for malformed physical-measure duplication.
- VexFlow needs explicit `Dot` modifiers for dotted notes and rests, and dotted durations must also be encoded in the note duration itself so ticks and spacing stay correct.
- Multi-measure rests should be rendered with VexFlow's native `MultiMeasureRest`, not simulated with text plus a whole rest.

### 4. Static Docs Runtime

- Static docs pages should avoid eagerly loading the full DSL/render stack because examples are already pre-rendered at build time.
- Browser-native scrolling and lightweight runtime JS are more robust than nested scroll containers and heavy custom restoration logic for the docs page.
- Copy buttons should be resilient at runtime even if build-time HTML injection changes, and button binding should be idempotent.
- Width constraints should be applied to the reading column rather than the outer docs shell, so the sidebar and chrome can remain full-width.

## 2026-05-01 Addendum: VexFlow 5 StaveTempo Positioning

- VexFlow 5 `StaveTempo` modifiers, when added with `Modifier.Position.ABOVE`, appear to calculate their horizontal position based on the internal "Note Start" offset but fail to add the parent `Stave.x` coordinate.
- Result: When page margins (padding) change, the stave and notes shift correctly, but the tempo marking remains stuck at a fixed absolute position on the canvas.
- Fix: Manually add the stave's current `x` to the tempo's `x-offset` parameter. In this renderer, `new StaveTempo({ ... }, x - 45, y)` ensures the BPM marking follows the staff perfectly while staying left-aligned above the clef.
- Testing: Verifying this requires isolated render passes (e.g., fresh `JSDOM` instances in tests) because VexFlow's internal font measurement and modifier contexts can have "sticky" global state in Node environments that masks coordinate drift.

## 2026-05-01 Addendum: VexFlow Navigation Layout Split

- VexFlow is good at local modifier stacking, not system-level collision avoidance. If navigation is anchored to a note, attaching it as an `Annotation`-like modifier lets `ModifierContext` stack it with accents, sticking, and other note-local marks automatically.
- `left-edge` / `right-edge` navigation and `volta` brackets should not rely on fixed `shiftY` heuristics. A more stable renderer flow is: format and draw notes first, then inspect note and modifier geometry, build a system-level top skyline, and place edge/span overlays against that skyline.
- Pure note geometry is not enough once sticking or text annotations are in play. After `voice.draw(...)`, VexFlow modifiers have resolved positions, so skyline construction can safely sample note top, stem top, and the rendered `x/y` of above-staff modifiers without forcing a second SVG render pass.
- Mixed-font navigation such as `To Coda` is easier to keep stable by treating it as one logical layout unit. A custom annotation/overlay that draws text and glyph segments together avoids the spacing drift that happens when `To` and the coda symbol are emitted as independent modifiers competing for text lines.

## 2026-05-02 Addendum: UI Zoom & Safe Scrollable Centering

- Percentage-based width (e.g., `width: 130%`) is an unstable zoom mechanism when the container width is user-resizable via a divider. If the preview pane is narrowed, the score shrinks even if the zoom percentage remains the same.
- Stable Zoom Pattern: Use an absolute base width (e.g., `800px`) scaled by a raw decimal multiplier (`--page-scale`). This ensures "100% zoom" always looks the same regardless of the window size or resizer position.
- The "Centered but Unscrollable Left" Bug: Using `justify-content: center` or `margin: auto` on Flexbox/Grid containers for centering causes data loss during overflow. If the content exceeds the window, the browser centers it relative to the scrollable area, which "pushes" the left/top edges into negative coordinate space where they cannot be reached by scrollbars.
- "Safe Centering" with CSS Grid: The most robust modern fix is using `display: grid` on the scroll container and `margin: auto` on the content frame. Grid handles `margin: auto` more safely than Flexbox: it centers when the content is small, but respects the (0,0) origin when the content overflows, ensuring full scrollability to all edges.
- Inline-Block Centering Caveat: `text-align: center` on a container with `inline-block` children can cause a "jump to bottom" bug if the content width exceeds 100%. The browser treats the overflowing box as a "word" that is too long for the current line and wraps it to the next line, causing it to appear below the container's top edge.

## 2026-05-02 Addendum: Auto-Fit Width on Resize

- To implement "Fit to Window" behavior for an absolute-width score, use a `ResizeObserver` on the container.
- Formula: `newScale = (containerWidth - padding) / baseWidth`.
- By using `setFitWidth(true)` as a persistent mode, we can keep the score fitting perfectly even when the user drags the editor/preview divider.
- Manual zoom actions (like Ctrl+Wheel or clicking +/-) should automatically disable the `fitWidth` mode to respect the user's manual override.

## 2026-05-02 Addendum: Mobile Pinch-to-Zoom

- To implement custom pinch-to-zoom on mobile, intercept `touchstart` and `touchmove` events.
- Calculate the Euclidean distance between two touch points: `Math.sqrt(dx*dx + dy*dy)`.
- Scale the initial `pageScale` by the ratio of the current distance to the starting distance.
- Use `event.preventDefault()` on `touchmove` when two fingers are detected to suppress the browser's native viewport zoom, allowing the custom notation scaling to take over.

## 2026-05-02 Addendum: Virtual Zoom for Performance

- Heavy renderers like VexFlow cannot re-render at 60fps during a zoom gesture.
- Solution: "Virtual Zoom". Use a fast CSS `transform: scale()` on a wrapper during the gesture for immediate feedback.
- Only "commit" the scale to the renderer's state on `touchend`.
- This separates Visual Zoom (GPU-driven, 60fps) from Layout Zoom (CPU-driven, high quality), providing a buttery smooth experience on mobile.

## 2026-05-02 Addendum: CI/CD & SVG Testing Robustness

- SVG markup generated by rendering libraries (like VexFlow) can vary slightly in formatting (e.g., self-closing tags `<path />` vs `</path>`) depending on the environment (JSDOM vs Browser).
- Regex-based SVG probes should be flexible: use `[^>]*` for attributes and allow for optional or self-closing tags to avoid CI failures.
- When changing global layout defaults (like `staffScale` or `pagePadding`), existing tests that rely on absolute SVG coordinates (e.g., `y="190.5"`) will likely break. Prefer coordinate-agnostic assertions (e.g., checking relative positions or counts of elements) to make tests more resilient to design refinements.
- GitHub Actions should always run the full test suite (`npm test`) before building to catch regressions early.

## 2026-05-05 Addendum: Lezer Comment Handling & Parser Consolidation

### Root Cause

The Lezer grammar (`drum_mark.grammar`) had no `Comment` token. Lines starting with `#` (e.g., `# SD | x x x x | - r - r |`) were parsed as real TrackLine nodes by the Lezer parser, creating multi-line paragraphs with mismatched measure counts. This triggered the validation error "All track lines in a paragraph must have the same measure count" in `ast.ts:425-432`.

The regex parser (`parser.ts`) already handled comments correctly via `preprocessSource` → `splitComment`, but the Lezer skeleton builder (`lezer_skeleton.ts`) bypassed that preprocessing entirely.

### Debugging Methodology (Retro)

- **Do not trust the user's minimal reproducer.** The input `| x x+s x x |` alone parsed correctly. The actual trigger was the `# SD | x x x x | - r - r |` comment line that the user did not include in their report.
- **When two parsers coexist, trace the active parser path.** The CLI used regex parser (no bug), but the Web Worker path went through `buildScoreAst(skeleton)` with a Lezer-produced skeleton (bug). This path divergence is why the error only appeared in the web UI, not the CLI.
- **When a Lezer parser treats unexpected input as valid syntax, check the grammar.** The `#` character had no definition in `drum_mark.grammar`, so Lezer's error recovery treated it as skippable noise and parsed the remainder (`SD | ... |`) as a valid TrackLine.

### Fix

Added a `Comment` token to the Lezer grammar following the standard Lezer pattern:

```
@skip { space | Comment }

@tokens {
  Comment { "#" ![\n]* }
  ...
}
```

The `![\n]*` is a token-layer negation character set matching any character except newline, zero or more times. This is the Lezer-recommended approach for line comments (analogous to `// ![\n]*` in the docs).

Regenerated the parser with `npx lezer-generator src/dsl/drum_mark.grammar -o src/dsl/drum_mark.parser.js`.

### Lezer Migration Complete (2026-05-06)

The Lezer parser is now the sole parser in all production code paths. The regex parser remains in the codebase for parity tests and benchmarks only.

All gaps were fixed in `lezer_skeleton.ts`:

| Gap | Fix |
|-----|-----|
| Leading `\n` prevents header parsing | `source.trim()` before parsing |
| `\|.` barline treated as final | Changed to `single` with `voltaTerminator` |
| `\|: :\|` empty measure not created | Allow push even with empty content |
| `\|  \|` ghost measure not created | Same — push empty measures |
| No implicit repeat-end for intermediate voltas | Added `sameVoltaIndices` + inference logic |

| `note 1/N` in body not parsed | Scan source gaps between track lines for overrides |
| Non-power-of-2 note values not rejected | Added validation in NoteHeader parsing |
| Braced block nested GroupExpr duplicated | Filter nested MeasureTokens in inner braced MC |
| Combined hit `+` in group items → rest | Handle `+` as combined-hit separator in `parseGroupItems` |

Production file changes:
- `ast.ts`: switched `parseDocumentSkeleton` → `parseDocumentSkeletonFromLezer`
- `scoreWorker.ts`: simplified to use `buildNormalizedScore` directly (no fallback needed)
- `index.ts`: removed `export * from "./parser"`
- `drum_mark.grammar`: added `Comment` token

The regex parser (`parser.ts`) now has zero production references. All 345 tests pass with Lezer as the only parser.

## 2026-05-06 Addendum: Spec-Driven Syntax Cleanup

- **Spec is the only source of truth.** A syntax is legal if and only if it appears in `DRUMMARK_SPEC.md`. Grammar, parser, skeleton, and tests all derive from the spec — never the other way around.
- **Deprecated syntax must be removed atomically.** When a notation like `xN` repeat count is declared deprecated, every reference must be deleted in one pass: grammar rule, skeleton builder fallback branches, regex parser branches, test cases that exercise it, README examples, and task list items. Partial cleanup creates "zombie syntax" that future contributors (including the same person six months later) will rediscover and re-implement.
- **Tests are not authoritative.** A passing test that uses deprecated syntax is a stale test, not a feature requirement. When migrating or refactoring, encountering a test that exercises removed syntax means the test itself should be deleted — not re-supported to make it green.
- **README examples carry implicit legitimacy.** A syntax that appears in the project's README will be treated as canonical by any engineer joining the project. Deprecated examples must be updated immediately.
- **Migration fixes must distinguish "missing feature" from "stale test".** During the Lezer migration, the `xN` repeat count appeared as a test failure. It was classified as a "gap" and re-implemented as an extraction hack in the skeleton builder. The correct classification was "stale test that should be deleted". The heuristic: if the spec doesn't define the syntax, any test exercising it is stale, regardless of whether it was passing before the migration.

## 2026-05-06 Addendum: Lezer Grammar Boundary for DrumMark

- If `lezer_skeleton.ts` has to detect syntax with raw-text rescans, regexes over measure content, or source-gap scanning, the grammar is still under-modeling the language. In this codebase that applies to summon prefixes, routed brace blocks, group internals, inline repeat `*N`, multi-rest `--N--`, paragraph `note 1/N` overrides, and coarse barline classification.
- Unquoted free-text headers (`title`, `subtitle`, `composer`) are feasible, but only with contextual line-tail parsing scoped to those three header keywords. Reusing ordinary DSL tokens for header text would make `#` comments, spaces, and token collisions brittle.
- `#` in unquoted free-text headers must remain comment syntax. Literal `#` therefore requires quoting, e.g. `composer "C# Minor"`.
- Parser design should prefer one parse-tree shape per concrete syntax form. Leaving multiple structural encodings for the same text, such as `||.`, leaks implementation choices into downstream skeleton logic and tests.

## 2026-05-06 Addendum: Neutral Error Lowering in Lezer Skeleton

- Malformed local syntax in `lezer_skeleton.ts` should emit diagnostics and lower to a semantically neutral shape, not to a valid musical token. In particular, an incomplete summon like `SD:` must not synthesize a `-` rest token during recovery, because that changes measure duration and can survive into normalization/rendering.
- The safe recovery pattern for malformed structural tokens is `TokenGlyph | null` plus filtering at each container boundary (`CombinedHitExpr`, `GroupExpr`, `MeasureContent`). This keeps diagnostics while preventing accidental semantic repair.
- Invalid shorthand combinations must clear all shorthand metadata before measure construction. For example, `--8-- *2` should not retain `multiRestCount: 8` after reporting the combination error; otherwise a malformed measure silently behaves like a legal multi-rest.
- The `npm run drummark` CLI only accepts file input. Parser/IR repros for malformed cases should therefore be verified with temporary files rather than stdin piping.

## 2026-05-06 Addendum: Grammar-Friendly Surface Syntax

- Reusing an existing sigil across multiple syntax families is acceptable only if the spec defines a closed lexical partition. Otherwise typo handling and future syntax growth drift quickly between grammar and lowering.
- A shorthand can still belong directly to the grammar without being forced into a single compact canonical spelling. For DrumMark multi-rest, the stable property is the local `dash-run + integer + dash-run` structure, not dash-run symmetry.
- If a shorthand is grammar-owned, it is often cleaner to define only the legal forms and let non-matching inputs fall back to the ordinary grammar, rather than inventing a separate malformed-candidate intent-recovery layer.

## 2026-05-06 Addendum: Closed `@` Namespace for Routed Blocks

- Reusing `@` for non-navigation syntax is safe only if the language keeps a closed namespace split. For DrumMark, `@TrackName` routed blocks and the enumerated navigation directives must be the only legal `@...` forms; there should be no generic fallback `@Identifier` category.
- A breaking syntax change is easier to keep honest when the parser emits a dedicated migration diagnostic for the removed form, instead of silently accepting a legacy alias.

## 2026-05-06 Addendum: TypeScript Strictness Around Lezer Integration

- `tsconfig.app.json` enables both `noUncheckedIndexedAccess` and `noPropertyAccessFromIndexSignature`, so parser-lowering code must treat every `array[i]` and `record.key` access as potentially absent even when control flow makes it look obvious.
- Generated Lezer parser files such as `src/dsl/drum_mark.parser.js` need a companion `.d.ts` shim if they are imported from TypeScript under `strict` mode and no upstream declaration is emitted.
- In `lezer_skeleton.ts`, the safest way to satisfy strict typing without changing parsing behavior is to introduce local guards at array/index boundaries and preserve existing lowering logic, rather than weakening compiler settings or replacing the source containers with looser types.

## 2026-05-06 Addendum: Paragraph `note` Overrides in Lezer

- A body-level `note 1/N` override cannot simply be added as another top-level `TrackBody` alternative, because it creates an LR conflict with header-level `note` lines at document start. The viable grammar shape is to admit paragraph overrides only in body-continuation positions, after the parser is already inside `TrackBody`.
- For DrumMark's current grammar, a stable pattern is: `TrackBody` starts with `TrackLine`, and later continuations distinguish ordinary next lines from blank-line-prefixed paragraph override segments. That removes source-gap scanning for override semantics without destabilizing header parsing.
- Once paragraph overrides are structural tree nodes, the skeleton should treat the node itself as authoritative paragraph-boundary evidence. Re-checking newline counts around that node is old-parser thinking and causes false "not at beginning of paragraph" diagnostics.

## 2026-05-06 Addendum: Hairpin Implementation Constraints

- VexFlow 5 already exposes `StaveHairpin` in the installed package (`node_modules/vexflow/build/esm/src/stavehairpin.js`). For DrumMark hairpins, the default rendering direction should therefore be VexFlow-native, not custom SVG wedge generation.
- `StaveHairpin` renders one wedge between a `firstNote` and `lastNote` with configurable `height`, `yShift`, and tick/pixel shifts. Cross-system hairpins are not a single primitive; they need to be split into one VexFlow hairpin segment per rendered system.
- The repository CLI path `npm run drummark --format ir` is normalized-score output only. `src/cli.ts` deletes the `ast` field before printing JSON, so CLI IR can validate normalized `hairpins` data but cannot validate raw parser / skeleton node presence.

## 2026-05-06 Addendum: SVG Clip Paths for Continued Hairpins

- VexFlow `StaveHairpin` has no API to render a cropped middle slice of a longer wedge. Cross-system continuation therefore has to be simulated with overextended endpoint shifts plus SVG clipping in the renderer layer.
- When injecting custom SVG `<clipPath>` nodes into VexFlow's `SVGContext`, DrumMark must set `clipPathUnits="userSpaceOnUse"`. Without that, browsers may interpret the clip rect in bounding-box space, which makes absolute stave coordinates ineffective and lets hairpins bleed to the page edge.
- Clip IDs must be unique across systems in the same page SVG. Reusing simple per-system indices risks later `<clipPath>` definitions shadowing earlier ones.

## 2026-05-06 Addendum: Docs Sync Baseline

- Current user-facing syntax has moved beyond the older docs baseline in four places that are easy to miss in static docs: free-text `title` / `subtitle` / `composer` headers are valid again, routed blocks require explicit `@TRACK { ... }`, multi-rest accepts relaxed dash-run spellings instead of only `--N--`, and hairpins use zero-duration `<`, `>`, `!` control tokens.
- The docs templates and README examples are part of the surface-language contract. When a spec addendum changes canonical spelling or migration rules, both English and Chinese docs need the same update pass or examples drift immediately.

## 2026-05-06 Addendum: CLI AST Output Contract

- The repository CLI is most useful as a staged inspection pipeline: `input -> ast -> ir -> xml/svg`. `ast` should expose parser/lowering structure before normalization, while `ir` should remain the normalized score without the embedded AST payload.
- Keeping CLI JSON formatting in a small helper is safer than open-coding `delete score.ast` in the command entrypoint; otherwise adding an `ast` mode later encourages drift between output modes.

## 2026-05-06 Addendum: Settings Terminology

- Settings labels are easier to keep coherent if they follow one semantic split: use `Spacing` for distance between two layout elements, `Offset X/Y` for nudging one rendered element, `Margins` for page insets, and `Height` / `Length` / `Scale` / `Font Size` for pure size controls.
- The code should mirror the UI vocabulary. If the UI says `Volta Spacing` and `Tempo Offset Y`, the render-option field names should not stay as `voltaGap` or `tempoShiftY`, or the terminology drift returns the next time the panel is edited.

## 2026-05-06 Addendum: User-Facing Settings Labels

- Internal render vocabulary does not automatically make good UI copy. Terms like `offset`, `shift`, and even `spacing` are implementation-friendly, but end users understand visual intent more quickly through phrases like `Distance Between Systems`, `Tempo Marking Up/Down Position`, and `Volta Distance from Notes`.

## 2026-05-06 Addendum: Whole-Measure Voice Rests in VexFlow

- `buildVoiceEntries()` intentionally splits rests at grouping boundaries, but the VexFlow renderer can still collapse an all-rest voice back into a single whole-rest glyph at the final note-building stage. That keeps normal intra-measure rest grouping intact while avoiding visually fragmented full-bar rests in voice 1 or voice 2.

## 2026-05-06 Addendum: Duration-Weighted Intra-Measure Spacing

- VexFlow's existing `Formatter` can be reused for collision-safe baseline layout, then nudged afterward by rewriting `TickContext.setX(...)`. The stable version is to preserve the first and last onset anchors from the formatter and only remap the intermediate contexts.
- Attaching a small `__drummarkStartKey` marker to each `StaveNote` is enough to reconstruct per-context rhythmic starts after formatting. That avoids depending on VexFlow's internal tick serialization when applying custom spacing logic.

## 2026-05-07 Addendum: Content-Weighted Measure Widths

- System-level measure width allocation is safest when it happens before stave creation and only changes each stave's `x` and `width`; downstream geometry such as hairpins and volta spans can then continue to read from final stave/note positions without special casing.
- Two-bar repeat placeholders need explicit pair handling even under content-weighted width allocation. Treating the `measure-repeat-2-start` / `measure-repeat-2-stop` pair as a shared width unit keeps the overlay centered and avoids visibly mismatched physical bars.

## 2026-05-07 Addendum: First-Paragraph `note` Override Parsing

- On the Lezer path, the first body paragraph can legally start with a paragraph-level `note 1/N` override only when the header block and body are separated by a blank line. Without that separator, `note 1/N` at the top of the file must stay parseable as a global header, or the grammar becomes ambiguous.
- The safe grammar shape is to split document entry into two body forms: `HeaderSection TrackBody` for compact header-to-body transitions, and `HeaderSection Newline+ TrackBodyWithLead` for the blank-line-separated case where the first paragraph may begin with `ParagraphNoteOverride`.

## 2026-05-07 Addendum: Dark Mode Theme Resolution

- If JS treats invalid root `data-theme` values as “no explicit override”, CSS must do the same. A selector like `:root:not([data-theme])` is not equivalent, because `data-theme="foo"` disables the CSS fallback while JS may still resolve to system dark, splitting shell/docs and CodeMirror into different themes.
- For theme precedence that supports both explicit override and system fallback, the safe CSS pattern is `:root[data-theme="dark"]` for the forced-dark branch and `:root:not([data-theme="light"]):not([data-theme="dark"])` for the system-driven dark branch.
- White paper surfaces need their own invariant tokens (`--paper-*`) rather than reusing generic card tokens, or dark-mode refactors will eventually darken score preview pages by accident.

## 2026-05-07 Addendum: Repo-Wide Audit Baseline

- `docs/DRUMMARK_SPEC.md` and the parser are still out of sync on doubled-duration stars: the spec now says there is no per-token `*` limit, while `src/dsl/parser.ts` still hard-caps stars at 3 and emits a dedicated error. This is a true language-surface mismatch, not just wording drift.
- DrumMark currently has two meaningful parser paths in play: the manual/regex skeleton path centered on `parseDocumentSkeleton(...)`, and the Lezer skeleton path used by `ast.ts` and parity tests. Proposal authors and implementers need to confirm which layer is authoritative before designing syntax work, or review discussion starts from the wrong architectural assumption.
- The current app and renderer hotspots are large enough to be treated as architecture boundaries rather than ordinary files: `src/App.tsx`, `src/vexflow/renderer.ts`, `src/dsl/parser.ts`, and `src/dsl/lezer_skeleton.ts` are each well past the point where small feature work remains low-risk.

## 2026-05-07 Addendum: Parser Ownership Decision

- The repo-wide audit stream now has an explicit parser direction: the Lezer-based path is the authoritative parser for normalized semantics, and the older regex/manual parser is on a deprecation path rather than being preserved as a co-equal long-term authority.
- Once a parser path is marked deprecated, no new syntax or semantic feature should be allowed to land only there. Transitional uses like parity comparison or rollback guard are acceptable only if they are explicitly documented as temporary.

## 2026-05-07 Addendum: Unlimited Star Duration Math

- Removing the old parser-side `stars > 3` cap is not enough by itself. Duration math must also avoid JavaScript bit-shift operators like `1 << stars`, because those silently become 32-bit and stop representing `2^stars` correctly once star counts grow.
- For DrumMark's uncapped `*` duration suffixes, the safe implementation is ordinary exponentiation (`Math.pow(2, stars)` or equivalent), with measure validation remaining the real guardrail against unusable note weights.

## 2026-05-07 Addendum: Exact-Range Overflow For Large Binary Duration Exponents

- "No syntactic star cap" and "unbounded exact arithmetic" are different guarantees. After `*` and `/` cancellation, if the remaining net binary exponent exceeds the implementation's exact numeric range, the compiler should emit an explicit overflow diagnostic instead of silently re-capping syntax or hanging in fraction math.
- Large symmetric `*`/`/` runs must still cancel exactly before any overflow guard is applied. The overflow check belongs on the surviving net exponent, not on raw star count alone.

## 2026-05-07 Addendum: Duration-Modifier Overflow Scope

- The exact-range overflow story is broader than star counts alone. Large dot runs can also exceed exact numeric representation even when parsing succeeds, so overflow detection should be framed in terms of the full duration modifier combination, not just `stars - halves`.
- Once a measure contains an overflowed token, normalization should avoid emitting partial IR for later tokens in that same track-measure from a stale slot offset. Reporting the diagnostic and dropping that track-measure contribution is safer than fabricating misaligned events.

## 2026-05-07 Addendum: Parser Ownership Test Contract

- Once Lezer is declared authoritative, spec-facing correctness tests should assert Lezer behavior directly. Any remaining manual-parser comparison tests are drift probes only; they should not keep the deprecated parser in the role of production oracle.

## 2026-05-07 Addendum: Navigation Diagnostic Columns On The Lezer Path

- Parser-path drift can hide in diagnostics even when syntax acceptance matches. For positional navigation errors, the manual parser already reports the offending token column, so the Lezer path should derive columns from the nav node's real source offset instead of defaulting to measure-start column `1`.
- The three highest-value drift buckets from the audit were enough to close Task 3 without reopening parser architecture work: uncapped duration suffix handling, positional navigation diagnostics, and paragraph-level `note 1/N` overrides.

## 2026-05-07 Addendum: CLI Render Bootstrap Ownership

- The Node-side DOM/canvas bootstrap for CLI rendering needs one shared owner. Otherwise `cli.ts` quietly becomes a second renderer entrypoint with its own missing globals and font-registration drift.
- `XMLSerializer` is easy to miss when hand-copying the JSDOM bootstrap, but VexFlow SVG finalization uses it directly. CLI smoke probes for `--format svg` should stay in the acceptance loop for any future bootstrap refactor.

## 2026-05-07 Addendum: Bundle Reports Need Evidence, Not Hunches

- A bundle-size task is more useful when it writes a deterministic artifact than when it only prints console warnings. Recording the built asset list plus the main entry chunk size in `dist/bundle-report.json` gives later tasks something comparable across runs.
- Dependency reachability is stronger when it combines source-surface evidence and built-bundle evidence. For `opensheetmusicdisplay`, zero source mentions plus zero mentions in emitted JS is a materially better removal signal than grep on source alone.

## 2026-05-07 Addendum: Browser Smoke Coverage Without Heavy E2E

- Playwright was an unused devDependency: no config file, no `@playwright/test`, zero test files. Rather than wiring up a full browser test harness, the project extends its existing jsdom render-probe pattern to cover the audit's required smoke surfaces.
- A single `smoke.test.ts` file (jsdom environment, 27 tests) covers: preview fixture rendering with headers/navigation, settings interaction via `hideVoice2Rests` toggle, hairpin offset rendering, multi-system layout, edge-case collapse (single-tack rest), and full rendering of all 22 docs examples. This keeps the test suite deterministic and fixture-based while avoiding the maintenance cost of a browser E2E layer.
- Deterministic SVG string assertions catch regressions at the structural level (missing stavenotes, missing edge navigation, missing content text) without requiring visual snapshots.

## 2026-05-07 Addendum: Renderer Layout Seam Extraction

- The layout planning functions in `renderer.ts` (`buildMeasureSpacingPlan`, `buildMeasureContentWeight`, `normalizeMeasureWeightsToWidths`, `buildMeasureWidthPlan`) are pure data transformations with zero `any` types and zero VexFlow API calls. Extracting them to `src/vexflow/layout.ts` creates a ~230-line module that depends only on `../dsl/logic` (Fraction/VoiceEntry math).

## 2026-05-07 Addendum: App Settings Seam Extraction

- The settings panel in `App.tsx` was the lowest-risk seam to extract. Settings JSX is a contiguous block; the NumericSettingControl component was already self-contained; and the state initialization, validation, and persistence were neatly separable into a `useAppSettings` hook (localStorage-backed, with range-validation on load).

## 2026-05-07 Addendum: Audit Stream Closure

- All 10 audit tasks are complete. High-severity findings closed: per-token `*` rule contradiction (Task 1), parser ownership settled (Task 2 -- Lezer authoritative, regex/manual deprecated), parser-path drift fixtures covered (Task 3). Architecture decisions recorded: CLI bootstrap centralized (Task 4), bundle report in `dist/bundle-report.json` (Task 5), Playwright intentionally removed in favor of jsdom smoke coverage (Task 6), renderer layout seam extracted to `src/vexflow/layout.ts` (Task 7), app settings seam extracted to `src/components/` + `src/hooks/` (Task 8), future-feature lanes classified in DRUMMARK_SPEC.md addendum (Task 9).
- No items were deferred. The active rehearsal marks proposal stream (`docs/proposals/DRUMMARK_SPEC_*`) remains independent and was not absorbed by this audit.

## 2026-05-07 Addendum: Radix UI Migration

- Five Radix UI headless components replaced hand-rolled interaction code: Slider (inside `NumericSettingControl`), Switch (toggle), Accordion (settings grouping), Tabs (Editor/Page/XML switching), and Popover (zoom menu). Total estimated gzipped bundle increase ~20 KB across all 5 packages.
- Radix was chosen over Mantine v7 after a 3-round review. Mantine's 7.6 MB npm package, PostCSS pipeline, theme bridge, and AppShell layout replacement were disproportionate for a project whose hand-rolled CSS (reduced from 1304 to 1163 lines) is tightly integrated with VexFlow rendering and Bravura fonts.
- The `NumericSettingControl` component was refactored, not deleted — it remains a reusable wrapper with unchanged signature, internally using `<Slider.Root>` while preserving stepper buttons, number input, wheel scrolling, and value normalization.
- Dual `<Tabs.Root>` instances share `value` and `onValueChange` for the Editor/Preview tab bars. The preview pane renders all 3 triggers with Editor hidden on desktop via CSS.
- Accordion `type="multiple"` (uncontrolled) avoids stale-index errors when debug sections are conditionally rendered.
- Popover uses controlled mode (`open`/`onOpenChange`) with `<Popover.Portal modal={false}>` for z-index isolation.
- A 4-test SettingsPanel smoke test (jsdom, `flushSync`, `ResizeObserver` polyfill) guards against rendering regressions.
- CSS cleanup removed 200+ lines of obsolete rules (native range slider pseudo-elements, custom toggle switch, `.preview-tabs`/`.preview-tab`, `.page-zoom-popover`, `.settings-section`). Preservation checklist verified: VexFlow dark-mode inversion, Bravura `@font-face`, XML tree viewer, print styles, CodeMirror wrapper, zoom popover inner-content classes.

## 2026-05-07 Addendum: Zoom Decoupled from React State

- Storing `pageScale` in React state causes a full component re-render on every zoom step (Ctrl+Scroll, ResizeObserver, pinch), which makes zoom feel laggy. The zoom should be a pure CSS operation.
- Solution: move `pageScale` out of `AppSettings` state entirely. Use a `useRef` initialized from `localStorage`, update the `--page-scale` CSS variable directly during gestures (no `setState`), and debounce-localStorage-persist the value. The toolbar readout reads from the ref.
- Only `fitWidth` (boolean) stays in React state — toggling auto-fit mode is a discrete user action, not a continuous gesture.

## 2026-05-07 Addendum: Zoom-to-Cursor Requires Centering Offset

- The formula `newScroll = (oldScroll + cursorOffset) * ratio - cursorOffset` only works when the content is anchored to the left edge (`scrollLeft = 0` at origin).
- With `margin: auto` centering via grid, content that *fits* the viewport has a `centerOffset = (shellWidth - contentWidth) / 2` that pushes it away from the left edge.
- When zooming from a fit state to an overflow state, the center offset collapses to 0, causing a content jump that the basic formula doesn't compensate for.
- Correct formula:
  ```
  oldCenterOffset = max(0, (shellWidth - oldContentWidth) / 2)
  newCenterOffset = max(0, (shellWidth - newContentWidth) / 2)
  cursorInContentX = mx - oldCenterOffset + scrollLeft
  targetScrollX = newCenterOffset + cursorInContentX * ratio - mx
  ```
- Force layout recalculation with `void shell.scrollWidth` before assigning `shell.scrollLeft` to ensure the browser has applied the new CSS `width`.

## 2026-05-07 Addendum: Disabling Chrome Swipe Navigation

- Chrome's two-finger horizontal swipe (back/forward page navigation) is a browser-level gesture that bypasses `wheel` event interception and per-element `overscroll-behavior`.
- The only CSS-based fix is `html { overscroll-behavior-x: none; }` — applied to the root element, not individual scroll containers. This tells Chrome that the entire page has no horizontal overscroll to consume, so the swipe gesture is never interpreted as a navigation intent.

## 2026-05-08 Addendum: Bottom Skyline for Hairpin Placement

- The Top Skyline pattern (`sample` → place → `occupy`) for above-staff layout generalizes cleanly to below-staff hairpin placement. A `BottomSkyline` class mirrors `TopSkyline` with inverted semantics: `Math.max` for sample/occupy, and `getBottomLineY()` as fallback.
- **BottomSkyline MUST initialize buckets to `fallbackBottom`**, not `NEGATIVE_INFINITY`. In SVG coordinates, Y increases downward, so notes above the staff (e.g., hi-hat at line 0, Y≈208) have notehead Y values SMALLER than the staff bottom (Y≈253). If buckets start at `-Infinity`, occupying a hi-hat at Y=208 sets the bucket to `Math.max(-Inf, 208) = 208`, which is ABOVE the staff bottom — causing the skyline to report a "lowest occupied" point that is actually inside the staff area. Initializing to `fallbackBottom` ensures `Math.max(fallbackBottom, 208) = fallbackBottom`, so above-staff notes don't distort the skyline downward.
- `StaveHairpin` Y positioning is NOT based on `getModifierStartXY(BELOW).y`. VexFlow internally computes `hairpin_Y = stave.getY() + stave.getHeight() + renderOptions.yShift + 20`. The `+20` is hardcoded in VexFlow's `renderHairpin` method (`let dis = this.renderOptions.yShift + 20`). Any custom yShift computation must account for this — `yShift = targetY - staveBottom - 20`.
- `StaveHairpin` positioning is uniform-shift (a single `yShift` applied to both ends). To guarantee minimum clearance across an angled hairpin span, sample the bottom skyline at 3 x-positions (left, center, right) and use the worst-case (largest Y) reading. The center sample protects against convex skyline shapes between endpoints.
- Beam thickness below the staff is not part of note geometry alone — stem extents reflect the adjusted stem-to-beam connection, but the beam line itself extends below that. A fixed `BEAM_THICKNESS` buffer (3pt) for beamed down-stem notes closes this gap without requiring separate beam geometry iteration.
- Hairpin clipping (`clipPath`) height must be dynamic when hairpins are skyline-placed. A hardcoded `+60` was sufficient when `yShift` was user-controlled in a narrow range, but skyline placement (plus stacking) can push hairpins further down. `clipH` should be computed from the final `targetHairpinTopY + HAIRPIN_FULL_HEIGHT`.
- `hairpinOffsetY` semantics changed from "absolute offset from BELOW position" to "additional breathing room beyond skyline-determined baseline." The range tightened from -40..40 to 0..20 with default 0, since the skyline now handles positioning automatically.

## 2026-05-11

### WASM Parser — Nav Markers Consume Position Slots

Nav markers (`@segno`, `@fine`, `@to-coda`, etc.) were converted to `TokenGlyph::Basic { value: "-" }` (rest tokens) in `to_token_glyph`, causing them to consume weight=1 position slots. This shifted all subsequent events in a measure forward by one 8th-note position for each nav marker present.

**Fix**: Filter out `MeasureExpr::NavMarker` and `MeasureExpr::NavJump` from `es.tokens` BEFORE converting to `TokenGlyph`. Nav metadata is already extracted during the scan phase; the tokens themselves should not produce events. Same applies to `MeasureExpr::MeasureRepeat` and `MeasureExpr::MultiRest` — these are metadata-only and should not consume position slots.

### Known Lezer Normalizer Bugs (WASM matches, Lezer differs)

1. **Nested tuplet groups** (`groups.drum` par 6: `[3: d [2: d d d] d]`): Lezer ignores the outer `[3:` span when nested groups are present, treating all items as regular duration. WASM correctly honors the outer span (compressing to fit) and the inner tuplet. **Lezer bug — WASM is correct.**

2. **Paragraph boundary on blank line before `@fine`** (`full-example.drum`): A blank line between the Coda's `BD *4` and the standalone `|@fine|` creates a paragraph boundary per DSL grammar. WASM splits into 7 paragraphs; Lezer merges into 6 (losing the Coda's music events). **Lezer bug — WASM is correct.**

### CombinedHit Supports Summoned Notes

`CombinedHit` changed from `Vec<NoteExpr>` to `Vec<MeasureExpr>` to allow summoned notes inside combined hits (e.g., `d+BD:d`). A new `parse_single_hit` method handles summon prefixes within combined hits without causing infinite recursion (which would happen if `parse_measure_expr` → `parse_basic_or_combined` → `parse_measure_expr`).
