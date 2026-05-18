# DrumMark App Tasks

## Phase 0: Project Setup

- [x] Write DSL design document
- [x] Initialize git repository
- [x] Add initial project scaffold
- [x] Choose frontend stack and package manager

## Phase 1: DSL Core

- [x] Define tokenizer output types
- [x] Implement line preprocessing
- [x] Implement comment stripping
- [x] Implement header parsing (`tempo`, `time`, `divisions`)
- [x] Implement metadata header parsing (`title`, `subtitle`, `composer`)
- [x] Implement `grouping` header parsing and validation
- [x] Implement paragraph splitting by blank lines
- [x] Implement track line parsing
- [x] Implement empty-measure rest shorthand (`| |`)
- [x] Implement base token parsing
- [x] Implement modifier parsing
- [x] Implement `o` sugar for `HH`
- [x] Implement `c` crash sugar for `HH`
- [x] Implement `DR` input sugar
- [x] Implement group parsing (`[n/m: ...]`)
- [x] Implement repeat parsing (`|:`, `:|`)
- [x] Build AST types
- [x] Build normalized event model
- [x] Align normalized event model documentation with v0 implementation

## Phase 2: Validation

- [x] Validate known headers
- [x] Validate duplicate metadata headers
- [x] Validate non-empty metadata header values
- [x] Validate supported `time` beat units
- [x] Validate known track names
- [x] Validate per-track token legality
- [x] Validate modifier legality
- [x] Validate `DR` rejects modifiers
- [x] Validate group arity
- [x] Validate supported group ratios and stretched durations
- [x] Reject groups requiring automatic tie splitting
- [x] Reject group durations below 64th note
- [x] Validate measure slot totals against `divisions`
- [x] Validate `grouping` compatibility against `time` and `divisions`
- [x] Validate `DR` paragraph exclusivity with explicit drum tracks
- [x] Validate paragraph measure-count consistency
- [x] Validate repeat counts are at least 2
- [x] Validate repeat boundary consistency across tracks
- [x] Validate whitespace-equivalent measure syntax
- [x] Collect structured errors with line/column info

## Phase 3: Grid Preview

- [x] Render paragraphs as preview rows
- [x] Render measures with clear boundaries
- [x] Render groups spanning multiple slots
- [x] Render modifiers visually
- [x] Render repeat boundaries
- [x] Render `ST` sticking row
- [x] Highlight parse errors in preview

## Phase 4: MusicXML Export

- [x] Map tracks to percussion instruments
- [x] Convert normalized events into MusicXML measures
- [x] Export tuplets from group syntax
- [x] Export repeats where possible
- [x] Degrade `:|xN` for `N > 2` by expansion if needed
- [x] Export a single percussion part
- [x] Export title, subtitle, and composer metadata
- [x] Keep default beaming within `grouping` boundaries
- [x] Exclude `ST` sticking from MusicXML export
- [x] Export supported modifiers with stable MusicXML semantics
- [x] Verify import in MuseScore

## Phase 5: App UI

- [x] Set up editor pane
- [x] Set up preview pane
- [x] Add error panel
- [x] Add `Export MusicXML`
- [x] Add `Export PDF`
- [x] Add staff-style preview tab

## Phase 6: DrumScript Extraction & Semantic Model

- [ ] Extract `src/drumscript/` as a standalone logic core
- [ ] Implement Semantic Intermediate Model (Semantic tracks, Intensity, Techniques)
- [ ] Remove `glyph` dependency from the core model
- [ ] Implement instrument registry for easy extension
- [ ] Implement Voltas/Alternative Endings (`|1.`, `|2.`, `|.`)
- [ ] Support chainable and additive techniques in the model
- [ ] Migrate MusicXML backend to consume Semantic Model
- [ ] Migrate VexFlow backend to consume Semantic Model

## Immediate Next Tasks

- [ ] Extract `drumscript/` directory and define its clean API
- [ ] Implement `|1.`, `|2.` and `|. ` syntax in Parser
- [ ] Update `compiler.ts` to populate `volta` and `voltaStatus`

## Rust Cleanup Todo

- [x] Remove obsolete `drummark-core::build_layout_plan` hand-drawn layout path
- [x] Remove obsolete `drummark-layout::layout_plan` empty WASM export
- [x] Preserve `LayoutOptions::default()` values when JS callers omit optional fields
- [x] Replace lexer multi-rest no-op test with a real assertion
- [x] Clean obvious Clippy findings in `drummark-layout`
- [x] Refactor scene renderer helpers to reduce argument-heavy functions
- [x] Design and implement true multi-page layout for long scores
