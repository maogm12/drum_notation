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


## 2026-05-13 Addendum: Rust Parser Cutover Blockers for Lezer Deprecation

- The repository is not yet in a real post-Lezer state. Parser ownership still lives on the Lezer path in both code and docs:
  - `src/dsl/ast.ts` defaults `parseMode` to `"lezer"` and branches to `parseDocumentSkeletonFromLezer(...)`.
  - `src/normalize.ts` defaults `buildNormalizedScore(...)` to `"lezer"`.
  - `src/scoreWorker.ts` accepts runtime `parseMode` switching.
  - `src/App.tsx` and `src/hooks/useAppSettings.ts` expose a persisted `useWasmParser` product toggle.
  - `docs/PARSER_OWNERSHIP.md` still names Lezer as the authoritative parser.

- Three concrete Rust parser issues block full cutover:
  1. Paragraph-level `note` override detection is too loose. A bare `note` line is accepted and can fabricate a `4/4` override instead of producing a parse error.
  2. Header parsing silently drops malformed values (`time 4`, `tempo fast`, `grouping 3+`) rather than emitting explicit parser errors.
  3. Signed inline-repeat tokens lose their sign during parse (`*-1` becomes `1`), which prevents downstream validation from rejecting the original invalid value correctly.

- The correct cutover criterion is not `WASM equals Lezer`. The correct criterion is `WASM matches the spec and owns production semantics`. Existing Lezer behavior must not remain the oracle once a mismatch is proven to be a Lezer bug.

- A complete Lezer deprecation plan must cover five layers together: Rust parser correctness, JS/WASM integration helpers, runtime API ownership, UI/settings removal, and test/doc/dependency cleanup. Fixing `parser.rs` alone is insufficient.

## 2026-05-13 Addendum: Rust/WASM Cutover Execution Notes

- `src/wasm/skeleton.ts` must treat the Rust token stream as a left-boundary measure representation, not a ready-made JS `ParsedMeasure` shape. In particular:
  - left-boundary `||`, `|.`, `||.`, and `|:.` need to be shifted onto the previous parsed measure's closing metadata
  - `closingBarline` remains the source of truth for repeat-end on the current measure
  - final-line heuristics still need explicit handling for trailing `||`

- The WASM adapter cannot count measure-level metadata tokens (`inlineRepeat`, `measureRepeat`, `multiRest`) as rhythmic content when validating navigation anchors. If it does, valid forms like `@ds-al-coda *1` are misclassified as "not at end of measure."

- Rust group tokens carry all child items, but JS group validation expects `count` to mean duration-consuming items only. Hairpin markers inside a group (`<`, `>`, `!`) must stay in `items` while being excluded from the compressed/stretched ratio count.

- The old regex parser's remaining corpus mismatches reduced to five accepted Lezer/legacy bugs:
  - nested groups in `docs/examples/groups.drum`
  - third-ending retention in `docs/examples/repeats.drum`
  - the same third-ending bug in `examples/李白-李荣浩.drum`
  - dropped hairpin intent plus fabricated errors in `docs/examples/hairpins.drum`
  - paragraph measure-count mismatch anchored to the preceding comment line instead of the first offending track line in `docs/examples/full-example.drum`

- Renderer parity tests that previously looked for VexFlow-specific class names (`vf-notehead`, `vf-bar`, `vf-staff`) were stale against the current layout-engine SVG emitter. The stable contract for `src/renderer/svgRenderer.ts` is primitive SVG output (`<text>`, `<line>`, `<rect>`) after `setLayoutSource(dsl)`, not VexFlow DOM class parity.

- `buildNormalizedScore(...)` is now WASM-owned, so any test constructing a score at module top level can race WASM initialization. `src/cli_output.test.ts` had to move score construction behind `beforeAll(async () => await initWasm())`.

## 2026-05-13 Addendum: Platform-Neutral Layout Proposal Research

- The existing custom-renderer direction is architecturally mis-scoped for a real VexFlow replacement:
  - `crates/drummark-core/src/lib.rs` currently exposes `build_layout_plan(source, options)`, which makes the Rust boundary depend on source text instead of a normalized rendering model.
  - `src/renderer/svgRenderer.ts` consumes a Rust plan that is already shaped like web drawing instructions (`line`, `rect`, `text`, group open/close markers), so the output contract is browser-oriented rather than platform-neutral.

- A correct replacement boundary for multi-platform rendering is `normalized score -> Rust layout engine -> platform-neutral LayoutScene -> thin platform adapter`. If the Rust layer emits browser/SVG-shaped commands directly, every future platform either reimplements layout or inherits web-specific semantics.

- The replacement scope should stay intentionally narrow: only the drum-notation surface the product already uses. A "general music engraving engine" target will create schedule and design drag before the project even reaches VexFlow removal.

## 2026-05-13 Addendum: RenderScore and LayoutScene Foundation Work

- `drummark-layout` previously treated its input as a self-contained `NormalizedScore` copy, which duplicated `drummark-core` normalization semantics and encouraged source-driven layout wiring. Converting that boundary to an explicit `RenderScore` makes the ownership chain visible and gives `drummark-core` one place to map normalized music data into render-facing data.

- The current normalized model's `source_line` field is preserved into `RenderScore`, but it is not a stable 1-based source contract today. For simple parsed fixtures it may legitimately remain `0`, so downstream code should treat it as best-effort provenance rather than guaranteed user-facing line numbering.

- `LayoutScene` contract work exposed two easy-to-miss failure modes in a platform-neutral exporter:
  - measure indices must preserve score-space identity rather than resetting per system
  - line-like geometry cannot be reconstructed with ad hoc `max()` logic from width/height boxes; scene endpoints must remain faithful to resolved absolute geometry

- If `RenderScore` is exported to JS/WASM for fixtures or adapters, omitting semantic fields like navigation markers or hairpins silently re-opens the boundary. A render-facing contract is not "explicit" if the Rust struct is richer than the serialized form consumers actually receive.

## 2026-05-13 Addendum: Scene Export Runtime Integration

- On this machine, `wasm-pack` defaults to the Homebrew Rust toolchain, which does not have the `wasm32-unknown-unknown` target installed in its sysroot. Rebuilding the checked-in wasm glue works only if `wasm-pack` is invoked with rustup's toolchain binaries forced via `PATH`, `CARGO`, and `RUSTC`.

- A `build_layout_scene(...)` export must have a stable return contract even on parser failure. Returning a raw error array from Rust while the JS side assumes `Scene.pages` exists causes the runtime to degrade into misleading placeholder SVGs instead of surfacing diagnostics.

- The scene-construction ownership boundary matters in practice, not just in docs. When the new scene export was first implemented directly inside `drummark-core`, it recreated header placement, system spacing, measure width allocation, note spacing, stem geometry, and font choices in the wrong layer. Moving scene construction back into `drummark-layout` restores the intended ownership chain: `core` parses/normalizes/serializes, `layout` owns geometry.

- A thin renderer adapter should fail on unknown scene item kinds, not silently skip them. Otherwise the first future `glyphRun`/`polyline` scene item becomes an invisible regression instead of an actionable contract mismatch.

- CLI rendering should not use the same "friendly fallback" behavior as interactive preview rendering. For command-line verification, scene export or adapter failures must fail closed so users do not get a plausible-looking SVG file that actually encodes an internal runtime error.

## 2026-05-13 Addendum: Render Path Debugging for the New Layout Engine

- The current app/CLI render path does not consume the TypeScript normalized score; it goes straight through Rust WASM via `build_layout_scene(source, options)`. When a visual result disagrees with `npm run drummark --format ir`, the first suspect is parser/normalize drift between the JS and Rust pipelines, not the SVG adapter.

- `SD | d ||` exposed a concrete Rust parser gap: `parse_measure_section()` only preserved closing `:|` / `:|.` markers, so closing `||` and `|.` were being dropped before normalization. The practical effect was that the last measure silently downgraded to the post-pass default final barline. Fixing this required preserving distinct closing barlines in the AST and teaching normalization to upgrade the resolved measure barline from a closing `Double`.

- For debugging scene-based rendering, emitting stable `data-role` and `data-measure-id` attributes from `src/renderer/svgRenderer.ts` is a high-leverage contract. It turns renderer tests from weak primitive-count checks into semantic assertions against roles like `beam`, `double-barline-left`, `nav-start`, `hairpin-top`, and `multi-rest-bar`.

- The Rust render pipeline currently does not emit ordinary rest events for simple dash gaps such as `SD | d - d - |`; those gaps remain implicit spacing rather than first-class `EventKind::Rest` output. Existing renderer checks for visible rest glyphs therefore need to target explicit render-time constructs that already exist today (multi-rests, measure repeats, hairpins, repeats, nav markers) until the rest-event contract is expanded.

- `:|.` and `||.` cannot share one AST/normalize case. `:|.` means `repeat-end + volta terminator`, while `||.` means `double barline + volta terminator`. Collapsing both into one `DoubleVoltaTerminator` variant caused the new explicit `closing_barline` render path to draw `||.` as a repeat ending. The Rust path needs distinct barline variants so right-edge rendering can stay semantically correct.

## 2026-05-14 Addendum: Implicit Rests Belong in RenderScore, Not the Parser

- The right place to materialize ordinary gap rests for the new layout engine is `crates/drummark-core/src/render_score.rs`, not the parser or normalized AST. The parser's `-` tokens still represent source rhythm syntax, while the renderer needs a continuous per-voice timeline. That timeline can be derived from normalized events without changing parse semantics.

- The VexFlow pipeline's behavior is the right reference for this layer:
  - each voice should have a continuous measure timeline with gap rests inserted
  - if an entire voice is empty for the measure, it should collapse to one whole-measure rest
  - that empty-voice case must not be split at grouping boundaries, even when ordinary interior gaps are

- Once implicit rests are derived in `RenderScore`, the SVG scene renderer can stay simple: it just renders `EventKind::Rest` like any other event. The main additional layout requirement is that lower-voice rests need their own vertical lane, otherwise they overlap upper-voice rests and become visually useless.

- Rest glyph selection cannot be based on denominator alone. `1/1` and `1/2` both have denominator `< 4`, but they are whole-rest vs half-rest. The scene renderer needs duration-aware rest selection keyed by the full fraction, at least for the core undotted values.

## 2026-05-14 Addendum: Render-Time Rest Semantics After the First Prototype

- The first implicit-rest prototype overgeneralized from VexFlow and synthesized a full-measure rest for any voice missing from a measure. That is the wrong contract for this repo's score model. The safe rule is narrower: derive rests only for voices that are actually present somewhere in `score.tracks`, then fill their intra-measure gaps and trailing silence.

- In practice that means `BD | b - - - |` should render lower-voice rests for the remaining silence in voice 2, but it must not invent a separate upper-voice rest lane just because voice 1 exists in traditional drum notation. The layout engine should respect the score's active voice set, not a hard-coded "always two voices" assumption.

- Simple gap materialization also has a visible trailing-silence consequence: `HH | x - x - |` in `4/4` with `note 1/8` does not contain only the two interior eighth rests. After the final hit, the remaining half-measure silence is also a real rest span and should render as a half rest. Tests that assert only the interior gap glyphs undercount the new timeline contract.

- Splitting gap rests only at grouping boundaries is not enough. A renderable rest contract also needs binary-duration decomposition inside each span, otherwise common gaps like `3/8` degrade into one semantically wrong glyph. The current safe baseline is to decompose binary silence into primitive `1`, `1/2`, `1/4`, `1/8`, `1/16`, and `1/32` rest events before layout.

- Once the synth layer can emit `1/32` rests, canonical metrics must carry a real 32nd-rest glyph too. Leaving layout capped at `RestSixteenth` recreates the same cross-layer mismatch in a smaller form: render-score becomes more precise than the scene renderer can actually draw.

## 2026-05-14 Addendum: Scene Composites Must Survive WASM Serialization

- A platform-neutral scene contract is not real if semantic composites disappear at the WASM boundary. `LayoutScene` may contain `SceneComposite` values in Rust, but if `build_layout_scene(...)` serializes only `id` and `kind`, the web adapter cannot render spans such as `volta` or `repeatSpan` without re-inventing layout decisions from source text.

- The minimum useful composite payload for the thin adapter is: `fragment`, `label`/`count`, and `startAnchorId`/`endAnchorId`. With those fields plus page-space measure boxes, the adapter can draw semantic brackets without measuring text or reconstructing span ranges itself.

- `page.items`-only rendering leaves an entire class of notation invisible even when the layout engine is already producing the right semantics. Voltas were the concrete failure mode here: Rust had the notion of `CompositeKind::Volta`, but the app could not show any bracket until both scene serialization and the SVG adapter started consuming `page.composites`.

## 2026-05-14 Addendum: Repeat Spans Need Their Own Rust Post-Pass

- The Rust normalizer previously carried a `repeat_spans` field all the way out to `RenderScore`, but never actually populated it. That kind of "declared but dead" contract is easy to miss because downstream code can look complete while every real corpus example still receives an empty array.

- The correct ownership for repeat-span derivation is a normalization post-pass over canonical measure metadata, after repeat barlines and propagated voltas are already resolved. The logic needs to mirror the score-level rule used in the TypeScript pipeline:
  - open on `repeat-start`
  - emit a span when a matching `repeat-end` is reached
  - keep the logical repeat block open across subsequent alternate endings while `nextVolta` exists
  - finally close once the volta chain ends

- Once repeat spans are split into scene fragments for the adapter, fragment semantics matter just as much as for voltas. `start` fragments own the label and left hook, `end` fragments own the right hook, and `continuation` fragments own neither. Otherwise multi-system repeat brackets visibly duplicate counts and endcaps even though the scene data is technically present.

## 2026-05-14 Addendum: Structural Composites Should Not Be Reconstructed From Paint Items

- If `multi-rest`, `measure-repeat`, or `hairpin` only exist as ordinary scene items, downstream adapters and non-web renderers lose the semantic distinction between “this is a repeated-measure sign with count 2” and “this is just a text glyph `%` positioned near the staff.” That makes the scene look complete while still forcing every consumer to reverse-engineer structure from paint output.

- The layout layer should emit these constructs twice only in the intentional sense:
  - once as paint items for the current adapter to draw
  - once as first-class `SceneComposite` metadata carrying anchors and count/span semantics

- `hairpin` needs the same fragment treatment as `volta` and `repeatSpan`. Even if the current SVG adapter still draws hairpins from line items, the scene contract should already expose cross-system `start` / `continuation` / `end` fragments so future adapters do not need score-specific inference.

## 2026-05-14 Addendum: Parity Bugs Need Contract-Level Fixture Gates

- The layout contract must treat system-start and system-end barlines as resolved geometry, not adapter cleanup:
  - the first measure's opening barline sits on the visible staff left boundary
  - clef and first-system time signature live inside that first measure
  - later systems must not retain phantom time-signature spacing
  - final system barlines close on the staff right boundary instead of protruding past it

- Tempo defaults are easy to get wrong if beat-unit semantics stay implicit. The canonical default tempo beat is quarter note, and the `note = number` cluster needs explicit horizontal padding in layout output so adapters do not invent spacing.

- Stem attachment is a notehead-metrics problem, not a renderer stroke-placement problem. Up-stems and down-stems both need right-side anchoring derived from canonical notehead bounds, otherwise the scene can be structurally correct but visibly stab through the notehead.

- Drum vertical mapping cannot stay "mostly inferred." The supported render families need explicit staff-position or ledger-line mapping in the render/layout contract. Crash-on-top-ledger-line is the concrete parity check that exposes missing mapping data immediately.

- Flags and slanted beams both need canonical geometry assets rather than line fallbacks:
  - unbeamed flags should come from dedicated glyph roles or canonical paths
  - slanted beams should be real polygon/path bodies with stem lengths reprojected to the beam boundary

## 2026-05-14 Addendum: Layout Geometry Must Follow the Legacy Instrument Map

- The quickest reliable source of truth for current drum vertical placement is the existing legacy instrument mapping in `src/vexflow/notes.ts`, not the ad hoc scene-layout defaults. Converting those display-step/display-octave assignments into staff-space positions exposes concrete errors immediately:
  - `C` (crash) belongs on the top ledger line above the staff
  - `HH` is above the top line, not on it
  - `BD`/`BD2` sit higher than the previous layout defaults had them

- A system-start contract is not satisfied just because the correct glyphs exist near the left margin. The measure geometry itself must begin at the staff left boundary, while note-entry spacing inside the first measure is controlled by a decomposed start zone: opening barline, repeated clef, optional time signature, then the first playable slot.

- Right-edge barlines should be modeled against a boundary coordinate, not a barline-left coordinate. If the final/double barline rectangle starts at the boundary and then extends by its width, the barline visibly protrudes past the staff. The safer contract is: the barline's right edge lands on the boundary, so final/double/thick right-edge bars are positioned by subtracting their width from the boundary x.

## 2026-05-14 Addendum: Checked-In WASM Glue Is Part of the Renderer Contract

- The web renderer does not call Rust sources directly in tests; it imports the checked-in package under `src/wasm/pkg`. A Rust-side layout fix can therefore pass `cargo test` while `npm test` still exercises stale scene behavior if the wasm glue is not rebuilt.

- In this repo, a failing renderer parity test that still shows pre-fix SVG output after a Rust layout change should first be treated as a stale `src/wasm/pkg` suspicion, not as evidence that the Rust fix failed.

- On this machine, the reliable rebuild path remains `wasm-pack build crates/drummark-core --target web --out-dir ../../src/wasm/pkg` with rustup toolchain binaries forced through `PATH`, `CARGO`, and `RUSTC`; otherwise `wasm-pack` may resolve to the Homebrew toolchain and miss the correct wasm target setup.

## 2026-05-14 Addendum: Shared Render Enums Must Be Closed at the WASM Serialization Boundary

- If task acceptance or local workflow relies on root-level `cargo test -p ...`, the repository needs a real root Cargo workspace. Without a top-level `Cargo.toml`, task text that names `cargo test -p drummark-layout` is not literally satisfiable from the repo root even if the crate itself builds.

- `drummark-layout` enum growth is not local. Adding new `GlyphRole` or `TextRole` variants for canonical metrics also requires updating the `drummark-core/src/lib.rs` JS serialization matches that lower `LayoutScene` into wasm-bindgen objects. Otherwise Rust unit tests in the layout crate can stay green while `cargo test -p drummark-core` and `wasm-pack build` fail with non-exhaustive pattern errors at the boundary.

## 2026-05-14 Addendum: Scene Goldens Need a Canonical Wire Model, Not Duplicate Lowerings

- A platform-neutral `LayoutScene` fixture harness becomes brittle if Rust snapshots and wasm-export payloads are produced by separate hand-maintained lowering code. The stable pattern here is to let `drummark-layout` own one canonical wire-contract lowering, then derive both:
  - native scene snapshots for goldens
  - JS/wasm payloads for adapters

- A useful scene golden for layout migration should mix geometry and semantics in one fixture. The high-leverage combination here was:
  - header text blocks, especially tempo as a multi-item composite
  - a flagged unbeamed note, so the snapshot proves `polyline` flag structure instead of anonymous strokes
  - a slanted beam, so scene structure captures beam geometry independently of final SVG paint
  - repeat/volta/hairpin fragments spanning multiple systems

- If the web adapter consumes fields like `page.systems`, those fields need to exist in the TypeScript scene contract too. Leaving them out of `src/renderer/svgRenderer.ts` creates a false boundary where wasm exports more structure than the type system admits, which weakens adapter-contract tests.

## 2026-05-14 Addendum: Width-Driven System Breaking Changes Span Fragment Cardinality

- Once system breaking becomes width-driven instead of paragraph-driven, cross-system composite tests must stop assuming a fixed fragment count. A repeat span, volta, or hairpin that used to split into `start/end` over two systems may legitimately become `start/continuation/.../end` when narrower estimated widths force more systems.

- Scene goldens that are meant to validate contract structure should encode whatever the current planner really emits, not preserve an older “two systems” mental model. The useful invariant is fragment semantics and anchor continuity, not a hard-coded number of line breaks.

- Beam-group heuristics that key off rendered X distance are sensitive to width-planning changes. If a fixture is supposed to prove beam structure for serialization/golden purposes, choose rhythmic spacing that still satisfies the beam-group heuristic under wider measures, or the fixture will accidentally turn into a flag fixture after unrelated layout work.

## 2026-05-14 Addendum: System Breaking Must Budget Fixed Start-Zone Costs Before Scaling Content

- A width-driven planner cannot treat the full staff width as scalable measure content. First-measure start-zone costs are fixed geometry:
  - opening barline
  - repeated percussion clef
  - optional first-system time signature
  - baseline left/right playable padding

- If those fixed costs are only subtracted later during event placement, the planner can accept a system that "fits" on paper while the first measure's actual slot-to-X area is silently compressed. The stable contract is:
  - estimate inner playable width from rhythm/grouping density
  - add fixed reservation costs per measure when deciding breaks
  - scale only the inner playable width, not the reservations

- Grouped-timing tests need to prove geometry, not just presence. A useful acceptance pattern is:
  - compare quarter-note beat gaps across two groups with different density
  - assert the denser group receives wider beat spacing
  - add a near-threshold break case where the first-system reservation alone is what forces the extra system

## 2026-05-14 Addendum: Event Geometry Needs Slot Clusters and Explicit Attachment Anchors

- Scene-level event geometry is not complete if layout only emits noteheads and loose strokes. For drum notation, the stable contract is:
  - events are grouped by measure slot before drawing
  - same-voice combined hits can share one stem
  - opposing voices at the same slot must be horizontally separated by layout, not by adapter heuristics

- Attachment relationships need to survive serialization as first-class data. Adding `anchorItemId` to scene items gives the adapter and goldens a checked-in way to preserve:
  - stem -> notehead ownership
  - flag -> stem ownership
  - beam -> stem ownership
  - accent/sticking -> notehead ownership

- Source-driven renderer parity should not pretend the parser/normalizer already derive beam semantics when `RenderEvent.beam` is still `none` on that path. If the goal is to verify adapter paint behavior for beams, use precomputed scene fixtures; if the goal is to verify layout beam grouping, use hand-crafted `RenderScore` fixtures in Rust where beam intent is explicit.

## 2026-05-14 Addendum: Structural Scene Contracts Need Explicit Child Geometry and Layout-Owned Edge Stacking

- Structural composites are not actually first-class if the scene only serializes anchors and asks the adapter to redraw the bracket or span from scratch. For repeat spans, volta brackets, and navigation markers, the stable contract is:
  - layout emits the semantic composite
  - layout also emits the concrete child scene items that paint that composite
  - the composite stores `childItemIds` so tests and adapters can verify ownership without reconstructing geometry

- Cross-system continuation semantics are best validated at two levels:
  - fragment metadata on the composite (`start`, `continuation`, `end`)
  - checked-in child item cardinality per fragment in a golden snapshot
  This catches the common regression where continuation metadata survives but the drawable child geometry silently disappears.

- Edge collision resolution for navigation text, repeat spans, volta labels, measure numbers, and hairpins belongs to layout. The adapter should not decide which structural item moves farther away from the staff. A robust layout contract is:
  - group related child items into structural edge groups
  - assign deterministic stacking priority by semantic role
  - translate the whole group after collision detection so labels, hooks, and lines remain internally aligned

## 2026-05-14 Addendum: Corpus Gates Need Separate Layout Goldens and VexFlow Oracle Reports

- A migration gate should not overload one artifact to do two jobs. The stable split is:
  - a layout-owned corpus scene report that snapshots what `LayoutScene` actually emits for the supported corpus
  - a separate VexFlow-oracle divergence report that records how the legacy renderer differs on the same corpus

- This split matters because the two artifacts answer different review questions:
  - scene report drift means the new engine changed its own contract
  - oracle report drift means the relationship between the new engine and the legacy migration oracle changed

- VexFlow is a useful migration oracle only at the level it can actually expose. For this repo, serialized VexFlow SVG does not preserve layout-side role ownership like `repeat-span-line`, `volta-line`, `nav-start`, or `sticking`. Those need to be documented as approved oracle limitations in a checked-in divergence ledger instead of being silently excluded from review.

## 2026-05-15 Addendum: Flags Belong to SMuFL Glyph Metrics and Paragraphs Own System Breaks

- Unbeamed flags should stay in the same canonical glyph contract as noteheads and rests. In this repo that means emitting `ScenePrimitive::GlyphRun` with SMuFL flag roles, not ad-hoc polylines:
  - `flag8thUp`/`Down` = `U+E240` / `U+E241`
  - `flag16thUp`/`Down` = `U+E242` / `U+E243`
  - `flag32ndUp`/`Down` = `U+E244` / `U+E245`

- `paragraph_index` is not a soft hint for system planning. The layout contract is stricter: one paragraph maps to exactly one system, and width planning may only compress measure contents inside that paragraph-owned system.

- In this environment, renderer parity tests cannot rely on rebuilding `src/wasm/pkg` because `wasm-pack build` currently fails without the `wasm32-unknown-unknown` target installed. When that target is missing, the honest fallback is:
  - verify Rust-side integration with `cargo test -p drummark-layout`
  - verify SVG rendering of new scene primitives with precomputed scene fixtures at the TypeScript layer

## 2026-05-15 Addendum: The Active WASM Path Can Be Rebuilt Without wasm-pack

- On this machine, `wasm-pack` still resolves its sysroot/toolchain checks against Homebrew Rust even when `rustup` has the correct `wasm32-unknown-unknown` target installed. The practical rebuild path for the checked-in web package is:
  - `cargo build --manifest-path crates/drummark-core/Cargo.toml --target wasm32-unknown-unknown --release` with the rustup toolchain first on `PATH`
  - `~/.cargo/bin/wasm-bindgen --target web --out-dir src/wasm/pkg --omit-default-module-path target/wasm32-unknown-unknown/release/drummark_core.wasm`

- This matters because product/CLI rendering in this repo does not consume Rust sources directly; it consumes the checked-in `src/wasm/pkg` bundle. A fix can be fully correct in Rust and still appear broken in the app until that bundle is rebuilt.

## 2026-05-15 Addendum: Down-Flag Glyphs Should Not Be Pre-Shifted Left

- VexFlow positions unbeamed down-flags by starting the glyph at `stem_x - stem_width / 2`, not by offsetting it left by the full glyph width. The Bravura `flag8thDown`/`flag16thDown`/`flag32ndDown` glyph outlines already extend to the right of that anchor.

- For this repo's scene contract, that means the down-flag `GlyphRun.x_pt` should stay on the stem anchor and let the SMuFL glyph geometry create the right-side overhang. Pre-shifting the glyph left by `flag_metric.width_pt` in layout inverts the visual attachment.

## 2026-05-15 Addendum: Ledger Lines Must Be Emitted as Separate Scene Items

- Drum staff position mapping alone is not enough for out-of-staff notes. When a track lands on an integer staff position above the top line (`-1`, `-2`, ...) or below the bottom line (`5`, `6`, ...), layout must emit explicit short `ledger-line` segments centered on the notehead.

- Space positions just outside the staff (`-0.5`, `4.5`, `5.5`, ...) do not create new ledger lines by themselves; they inherit the nearest already-required ledger lines. Example: crash at `-1.0` needs one top ledger line, and a bottom note at `6.5` needs lines at `5.0` and `6.0`.

## 2026-05-15 Addendum: Score Text Font Ownership Lives in the Scene Contract

- In this branch, score-font assignment is not just an SVG adapter concern. The authoritative source for most score text is `crates/drummark-layout/src/lib.rs` via `canonical_text_metric()` plus a few explicit `push_text_item(..., font_family, ...)` call sites for noteheads, rests, clef, time signature, and tempo glyphs.

- There is also a parallel legacy WASM/JS scene serializer in `crates/drummark-core/src/lib.rs` that still hardcodes score text fonts. If score-font policy changes, both paths need to move together or the app bundle and layout goldens drift apart.

## 2026-05-15 Addendum: Dev and Build Need a Checked-In WASM Rebuild Step

- This repo's frontend imports the generated package in `src/wasm/pkg` rather than Rust sources directly. Rust changes in `crates/drummark-core` or `crates/drummark-layout` do not show up in `npm run dev` until the checked-in WASM bundle is regenerated.

- The practical fix is to make WASM rebuild a first-class npm script and hook it into `predev` / `prebuild`, rather than relying on contributors to remember an external two-command cargo + wasm-bindgen sequence.

## 2026-05-15 Addendum: Mixed Homebrew and rustup Toolchains Break WASM Builds

- On this machine, PATH resolves `rustc` to Homebrew Rust (`/Users/gmao/brew/bin/rustc` 1.94), while `cargo` and installed targets are managed by rustup (`stable-aarch64-apple-darwin` 1.95). That split causes `cargo build --target wasm32-unknown-unknown` to fail with `can't find crate for core` even though `rust-std-wasm32-unknown-unknown` is installed.

- The reliable repo-local fix is to force the WASM build script to use a matched rustup pair: resolve `cargo` via `rustup which cargo`, derive the sibling `rustc`, and run cargo with `RUSTC=<same-toolchain-rustc>`. Rebuilding `src/wasm/pkg` succeeds once both binaries come from the same toolchain.

## 2026-05-15 Addendum: Slanted Beams Need Filled Path Primitives, Not Thick Lines

- In the layout scene contract, beams should be emitted as filled quadrilateral `path` items rather than thick `lineSegment`s. A thick line looks acceptable only for horizontal beams; once the beam is slanted, the stem-to-beam joint reads as a stroked cap instead of a solid engraved beam.

- The practical shape for this repo is a simple four-point polygon from the first stem tip to the last stem tip plus a constant vertical thickness offset (`+thickness` for up-stems, `-thickness` for down-stems). The SVG adapter must render this as `<path fill=...>` so no stroke cap or join artifacts leak through.

## 2026-05-15 Addendum: `%%` Must Expand Display Measures But Preserve Source Measure Semantics

- A two-bar repeat shorthand cannot stay inside one display slot. Layout must expand one source measure with `measure_repeat_slashes == Some(2)` into two display measures, emit the dedicated SMuFL repeat-2-bars glyph once, and anchor the composite from the first display measure to the second.

- That expansion is display-only. Source-facing semantics such as later-system measure numbers still need to use the underlying source measure index (`measure.measure.global_index`), not the expanded display index, or every following system number drifts after the first `%%`.

## 2026-05-15 Addendum: Beam Grouping Mirrors VexFlow Grouping Segments

- VexFlow does not rely on the normalized event `beam` string for ordinary automatic beaming. It builds `VoiceEntry` runs and creates a `Beam` only while consecutive beamable notes remain in the same `groupingSegmentIndex()` result. A rest or non-beamable duration flushes the current run.

- The Rust layout engine should use the same rule: assign beam groups per voice from the canonical grouping slot boundaries, render groups with more than one anchor as filled beam paths, and render single-anchor runs as flag glyphs. This keeps `grouping 2+2` visually aligned with the VexFlow path and prevents accidental beaming across rests.

## 2026-05-16 Addendum: SMuFL Glyph Anchors Are Not Visual Centers

- Bravura's `bravura_metadata.json` separates glyph geometry into `glyphBBoxes` and optional `glyphsWithAnchors`. Layout code should preserve that split: bbox center may be used for visual centering, but it is not a stem or flag attachment anchor.

- For noteheads, real stem attachment comes from anchors such as `stemUpSE` and `stemDownNW`. For flags, the corresponding anchors are `stemUpNW` and `stemDownSW`. Rests, repeat marks, clefs, navigation glyphs, and time-signature digits in the current table do not expose stem anchors and should store `None` rather than synthetic bbox centers.

- The checked-in Bravura metadata does not expose advance widths. Until a separate font advance source is introduced, `CanonicalGlyphMetric.width_ss` is the bbox width for the glyphs this layout engine uses.

## 2026-05-16 Addendum: Repeat Barlines Have Dedicated SMuFL Glyphs

- Bravura exposes repeat barlines as full-height glyphs: `repeatLeft` (`U+E040`) and `repeatRight` (`U+E041`). Their `glyphBBoxes` span 4 staff spaces vertically, so a 40pt font size aligns naturally to a 10pt staff-space system when the glyph baseline/origin is placed at the staff bottom.

- These repeat glyphs do not define anchors in `glyphsWithAnchors`; layout only needs their bbox width for horizontal reservation and edge placement. The older hand-drawn rect-plus-dot implementation should not be used when the SMuFL font is available.

## 2026-05-16 Addendum: Repeat Barlines Need Optical Size Adjustment in SVG Text

- Although `repeatLeft` / `repeatRight` have a 4-staff-space metadata bbox, browser SVG renders `font-size="Npt"` in CSS points, where `1pt = 4/3` viewBox user units. For a 10-unit staff space and a 40-unit four-space staff height, the repeat font size should therefore be `40 / (4/3) = 30pt`, not the literal 40pt implied by metadata units alone.

- First-system start repeats need two independent horizontal controls: pull the glyph toward the clef/time-signature preamble, then reserve a separate trailing gap before note content. Tying both to a single "width" value makes the repeat either too far from the preamble or too close to the first note.

## 2026-05-16 Addendum: Accents Should Use SMuFL Articulation Glyphs

- Bravura exposes accent articulations as `articAccentAbove` (`U+E4A0`) and `articAccentBelow` (`U+E4A1`). They have separate above/below bboxes and no anchor metadata, so placement should center the glyph bbox over the notehead bbox rather than using a text `>` character.

- Accent placement depends on stem direction: up-stem accents belong above the stem tip or beam, while down-stem voice-2 accents belong below the stem tip or beam. Rendering accents while drawing the notehead is too early because the code has not computed stem and beam positions yet.

## 2026-05-16 Addendum: Layout `systemSpacing` Is Extra Gap, Not System Height

- The user-facing `systemSpacing` setting matches the VexFlow path: it is the extra gap after the fixed logical staff system band, not the full distance between system origins.

- In the layout scene engine, the origin-to-origin advance should therefore be `100.0 + system_spacing_pt`. Using the visible five-line staff height (`40.0`) as the base compresses multi-system scores because ornaments, stems, labels, and skyline content still need the same logical system band as the VexFlow renderer.

## 2026-05-16 Addendum: Zero-Valued WASM Options Must Stay Explicit

- Layout options crossing the JS/WASM boundary need to distinguish "field missing" from "field present with value 0". `systemSpacing: 0` is a valid user setting, so parsing it through `unwrap_or(0.0)` and then treating `<= 0` as default creates a non-monotonic control where 0 jumps to the default but 1 becomes nearly zero.

- Optional numeric settings should be read as `Option<f64>` and defaulted only when the property is absent or non-numeric. Range validation belongs at the UI/settings layer, not in the WASM bridge.

- The JS scene adapter must also preserve that distinction. When `renderScoreToSvg()` or tests omit `systemSpacing`, the adapter should pass the UI default; when the caller explicitly passes `0`, it should pass `0` through to WASM.

## 2026-05-16 Addendum: Volta Rendering Mirrors TS Block Semantics

- The TypeScript/VexFlow renderer does not build voltas as one global label range and then split it. It first finds contiguous volta blocks within the current system, computes one shared line Y for the block, then emits same-label spans inside that block.

- Left and right hooks are derived from global neighboring measures, not from fragment index. A segment shows the left hook and label only when its label begins at that measure; it shows the right hook only when the label ends there or the measure is a repeat end. Continuation fragments across systems intentionally have no left label/hook until the next real begin.

- VexFlow's volta label baseline is below the bracket line (`topY + VOLTA_TEXT_SIZE + 2`) and uses the navigation text font, not the SMuFL music font. Rust layout scene output should keep the label as semantic `CountLabel` text, but use `Academico` at 12pt for visual parity with the TS overlay.

## 2026-05-16 Addendum: Repeat Counts Are Not Volta Brackets

- The normalized `repeat_spans` list describes repeat playback ranges and counts. It must not be rendered as a visible bracket or `2x` house in the score.

- Visible bracket houses are volta notation only. The layout scene should emit `volta` composites from explicit numbered endings (`|1.`, `|2.`, etc.) and leave ordinary repeat spans to repeat barline rendering and playback semantics.

## 2026-05-16 Addendum: Volta Offset Uses the Top Skyline

- Volta bracket Y placement should be based on the highest already-rendered element under the volta block, not on the staff/system origin. That keeps adjacent numbered-ending houses on one shared Y within the same contiguous volta block.

- A volta offset of `0` means the bracket enclosure is tight to that top skyline. Positive values move the bracket upward and increase clearance (`svg y` decreases); negative values move it downward toward the system.

## 2026-05-17 Addendum: Cross-System Voltas Need Continuation Hooks

- When a numbered ending continues onto a later system, the continuation fragment should still draw a left hook at that system's start. The label should remain only on the true begin fragment, but the hook makes the later system's half of the bracket visibly read as the same house.

- Fragment semantics should still be derived from the global volta begin/end state. A continuation hook is a visual affordance for system breaks, not a new volta begin.

## 2026-05-17 Addendum: Repeat-End Does Not Terminate Volta Brackets

- A plain repeat-end barline (`:|`) closes the repeat playback range, but it does not by itself terminate the visible numbered-ending bracket. The active volta continues across paragraph/system breaks until an explicit volta terminator (`|.` / `||.`), a repeat-both boundary, or a new volta seed changes the active label.

- Layout should derive right hooks from neighboring measure volta labels, not from repeat-end status. If measure N has volta `[2]`, measure N is `repeat-end`, and measure N+1 also has volta `[2]`, the first fragment stays open and the continuation fragment closes only where the `[2]` span actually ends.

## 2026-05-17 Addendum: Rust Parser Must Preserve Barline Source Locations

- Editor diagnostics that point at structural barlines cannot be reconstructed reliably from normalized measures alone. The Rust parser should attach source locations to opening and closing barlines before exporting the WASM skeleton.

- For continuing-volta validation, the important location is the offending closing repeat-end token. `MeasureSection.closing_barline_location` should point at the `:|` start column, and the TS adapter should carry that through as `repeatEndLocation` so CodeMirror can underline the exact token.

## 2026-05-17 Addendum: Rust Hairpins Need Bottom Skyline and Fragment Progress

- Rust layout hairpins should use a bottom skyline, not a fixed staff-bottom offset. The skyline sample must include noteheads, stems, beams, flags, rests, articulations, and staff lines in the hairpin's visible X range, then place the hairpin below the lowest occupied element with a small gap.

- Hairpin geometry should be drawn as a true wedge: for a crescendo, the opening height grows from zero at the global start to full height at the global end; for a decrescendo, it shrinks. Same-system hairpins therefore have a closed end and an open end, not crossed or parallel lines.

- Cross-system hairpin fragments should compute their left/right aperture from the fragment's progress through the whole hairpin. This mirrors the TS clipping behavior without needing clip paths: each Rust fragment draws only its visible segment with partial openings at continuation boundaries.

## 2026-05-17 Addendum: Hairpin Vertical Offset Is Relative to Bottom Skyline

- The user-facing `hairpinOffsetY` setting is a direct vertical delta from the Rust bottom skyline result. `0` means the hairpin's top edge starts at the lowest occupied element under its X range, positive values increase SVG Y and move it downward, and negative values decrease SVG Y and move it upward.

- The JS layout-engine adapter must pass `hairpinOffsetY` through to WASM. Keeping the setting only in VexFlow options makes the React control appear live while the Rust scene renderer silently uses its default.

- The UI range must allow negative values because tighter placement is part of the control contract, not an invalid input.

## 2026-05-17 Addendum: VexFlow Secondary Beams Are Layer-Specific Segments

- VexFlow draws beam levels independently. The primary beam connects notes shorter than a quarter note across the beam group, but the secondary 16th-note beam only connects notes shorter than an eighth note. An eighth note inside a group therefore interrupts the secondary beam while leaving the primary beam continuous.

- Isolated short notes inside a higher beam level are rendered with partial beam stubs. For a `16th, 8th, 16th` group, VexFlow emits one continuous primary beam and two short secondary beam segments, not one secondary beam spanning the whole group.

- Rust layout should model beam anchors with a level count, then generate beam path segments per level instead of checking whether any anchor in the group requires a secondary beam.

## 2026-05-17 Addendum: Title Area Settings Match VexFlow Header Geometry

- VexFlow starts the first system at `pagePadding.top + headerHeight + headerStaffSpacing` after scaling settings into logical staff space. The Rust scene renderer should expose the same two layout options instead of using a fixed header reservation.

- `headerHeight` moves the header bottom. Subtitle and composer anchor to that moving bottom, while the title remains fixed near the top of the title area. `headerStaffSpacing` moves only the systems/tempo region below the title area; it should not move title, subtitle, or composer text.

- The JS layout-engine adapter must divide both settings by `staffScale` before sending them to WASM, matching the TS renderer's `getScaledDimensions()` behavior.

## 2026-05-17 Addendum: Rust Scene Rests Must Be Glyph Runs

- The layout scene SVG adapter renders `glyphRun` items by converting their numeric `codepoint` to a SMuFL character. Rest scene items should therefore use `GlyphRun` with `RestWhole` / `RestHalf` / `RestQuarter` / `RestEighth` / `RestSixteenth` / `RestThirtySecond` roles, not `TextRun` carrying a private-use character.

- The generated WASM package under `src/wasm/pkg` is ignored by git, so local preview and CLI checks can continue to use stale layout code unless `npm run wasm:build` has been run after changes in `crates/drummark-layout`.

- A minimal SVG verification for ordinary rests is `npm run drummark -- /tmp/rest-test.drum --format svg` followed by checking for `data-role="rest"` and SMuFL rest codepoints such as `U+E4E6` for eighth rests and `U+E4E4` for half rests.

## 2026-05-17 Addendum: Rust Layout Scene Is the Active WASM Path

- The active front-end Rust renderer imports `build_layout_scene` from the core WASM package and expects the platform-neutral scene contract (`version`, `metricsVersion`, `pages`, `systems`, `measures`, `items`, `composites`). The older `build_layout_plan` hand-drawn path in `drummark-core` and the empty `layout_plan` compatibility export in `drummark-layout` are obsolete and should not be revived without a new contract.

- Rust layout option parsing should start from `LayoutOptions::default()` and only override fields that are explicitly supplied by JS. Otherwise partial option objects can accidentally zero out page margins or spacing while still passing the page-size validity check.

- The current layout scene builder still emits a single page. Adding real page breaking changes the scene pagination contract and should be handled as a designed layout feature rather than a small cleanup.

## 2026-05-18 Addendum: WASM Preview Pagination Requires the Page-Aware Adapter

- The Rust layout scene can contain multiple `pages`, but `renderSceneToSvg()` and `renderScoreToSvg()` intentionally render only `pages[0]` and warn on multi-page scenes. Front-end preview pagination must call `renderScenePagesToSvgs()` / `renderScorePagesToSvgs()` and wrap each returned SVG in its own page section.

- The App preview has two independent render branches. The VexFlow branch already uses `renderScorePagesToSvgs`; the `useLayoutEngine` WASM branch must mirror that behavior instead of wrapping a single SVG as page 1.

- `pageHeight` is part of the pagination contract. The JS layout adapter must pass `pageHeight / staffScale` to WASM just like `pageWidth / staffScale`, otherwise tests or settings that use non-default page heights cannot exercise page breaks correctly.

## 2026-05-18 Addendum: System Box Extraction Must Respect Measure Ownership First

- During Rust scene pagination, `system_box_from_page_system()` must treat `measure_id` as authoritative. If an item has a `measure_id`, it belongs only to the system containing that measure and must not fall through to visual-band inclusion.

- Visual-band inclusion is only appropriate for unowned system-level items such as staff lines, clefs, and measure numbers. Letting measure-owned items fall through can duplicate beams/stems from one system into the next system when their Y bounds happen to overlap the next extraction band.

- A minimal regression input is `time 4/4`, `note 1/8`, first paragraph `| p p b b |`, blank line, second paragraph `| ss|`. The correct scene has two beam items for `measure-0` and one beam item for `measure-1`.
