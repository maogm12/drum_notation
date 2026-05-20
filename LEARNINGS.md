# Learnings

This file is the current architecture memory for DrumMark. Older chronological notes were archived to `docs/archived/LEARNINGS_legacy_2026-05-18.md` because the project changed technical direction several times and the old log had started to obscure the active path.

When an older note conflicts with this file, treat this file plus the active spec/proposal docs as authoritative.

## Current Architecture Baseline

### Product Rendering Path

- The default renderer is the platform-neutral Rust layout engine.
- VexFlow has been removed as a product renderer, fallback path, dependency, and active test oracle.
- Renderer work should target the Rust layout scene path and thin adapter contract.
- Historical VexFlow-specific notes are useful only as archived migration context, not as implementation guidance.

### Parser Ownership

- Rust/WASM parser output is the production parser path for the app, score worker, docs build, and CLI normalization.
- Lezer and regex/manual parser notes are historical. They may explain why a decision changed, but they are not the oracle for new behavior.
- When Rust/WASM parser behavior disagrees with old Lezer or regex behavior, use the spec and current Rust/WASM parser contract as the authority.
- Parser-facing TypeScript code should go through the explicit parser runtime registry and parser wrappers. It should not import browser-only generated WASM packages directly.

### Split WASM Runtime

- Browser startup initializes parser WASM only.
- Layout WASM must stay lazy and load only when layout rendering is invoked.
- Active generated package directories:
  - `src/wasm/parser-pkg-web/`
  - `src/wasm/layout-pkg-web/`
  - `src/wasm/parser-pkg-node/`
  - `src/wasm/layout-pkg-node/`
- Active wrappers:
  - `src/wasm/parser_wasm_browser.ts`
  - `src/wasm/layout_wasm_browser.ts`
  - `src/wasm/parser_wasm_node.ts`
  - `src/wasm/layout_wasm_node.ts`
- The old combined `src/wasm/pkg/` and `src/wasm/drummark_wasm.ts` path is not the active production contract.

### Layout Scene Contract

- Layout rendering is source-to-scene through layout WASM: `build_layout_scene(source, options)`.
- The TypeScript SVG adapter is intentionally thin. It renders `LayoutScene` primitives and composites; it should not reconstruct score semantics from source text.
- Unknown scene item kinds should fail loudly, not silently disappear.
- Stable layout-engine SVG assertions should use semantic output such as `data-role`, `data-measure-id`, page count, system count, and scene/composite structure.
- Old VexFlow DOM-class assertions such as `vf-notehead`, `vf-bar`, and `vf-staff` are stale for the layout engine.

### Source Coherence

- `setLayoutSource` and module-level layout source caches are obsolete.
- Render calls must carry the source attached to the accepted parsed-score revision.
- Active parsed score state should include `{ score, source, sourceRevision }`.
- Async parse results older than the current revision must not replace newer active score state.
- Rapid-edit rendering tests should verify that layout rendering receives the source from the same accepted revision as the score.

### Pagination

- `LayoutScene.pages` may contain multiple pages.
- App preview must use page-aware APIs such as `renderScorePagesToSvgs()` / `renderScenePagesToSvgs()`.
- Single-page helpers are acceptable for focused adapter tests, but app preview should not wrap only page 1 and call that complete pagination.
- `pageWidth` and `pageHeight` must both cross the JS/WASM boundary after the same staff-scale conversion.

### CLI Rendering

- CLI normalization initializes parser WASM through the Node parser wrapper.
- CLI SVG rendering initializes layout WASM through the Node layout wrapper.
- CLI rendering should fail closed on internal layout or adapter errors. It should not write friendly placeholder SVGs that hide runtime failures.
- Keep `npm run drummark -- <fixture> --format svg` in verification for renderer/bootstrap changes.

## Build And Verification

- `npm run wasm:build` is the authoritative local WASM rebuild command.
- `npm run build` runs the WASM build first, then TypeScript/docs/Vite and bundle reporting.
- Manual `wasm-pack` or one-off cargo/wasm-bindgen command notes in the archive are troubleshooting history, not the preferred workflow.
- `npm run verify:split-wasm` is the current full split-WASM verification gate. It covers:
  - split WASM build and size reporting
  - TypeScript/docs/Vite build
  - import-boundary enforcement
  - split wrapper tests
  - settings migration tests
  - score source-revision tests
  - SVG renderer/adapter regression tests
  - parser/layout semantic parity tests
  - CLI runtime tests
  - browser network audit
  - representative CLI SVG generation

## Static Import Boundaries

- Browser production code must not import Node wrappers or Node generated packages.
- Parser-facing production code must not import layout wrappers or layout generated packages.
- CLI runtime must not import browser wrappers or browser generated packages.
- Default layout/settings paths must not pull legacy renderer runtime imports.
- Integration/parity tests may cross these boundaries only when explicitly scoped as integration/parity tests.

## UI And Settings Copy

- User-facing labels should use musical/product language, not implementation names.
- Prefer musical/product labels over renderer implementation names.
- Avoid labels such as `useLayoutEngine`, `WASM render`, `offsetY`, or source-code field names.
- Settings that cross into layout WASM must preserve explicit zero values. Missing option and option value `0` are different states.

## Current Layout-Specific Notes

- Paragraphs own system breaks: one paragraph maps to one system unless a future approved proposal changes that contract.
- Measure-owned scene items must stay attached to their owning measure/system during pagination. Visual-band inclusion is only for unowned system-level items such as staff lines and clefs.
- Structural composites such as voltas, repeat-related spans, measure-repeat signs, multi-rests, and hairpins should be represented semantically in the scene contract rather than reconstructed from paint primitives.
- Beams should be filled path bodies for slanted geometry, not thick stroked lines.
- Unbeamed flags should use SMuFL glyph roles.
- Ordinary rests should be glyph runs with duration-aware rest roles.
- Volta placement uses the top skyline. Hairpin placement uses the bottom skyline.
- Repeat counts are playback semantics; visible bracket houses are volta notation.

## Superseded Buckets In The Archive

- Lezer migration and regex-parser coexistence notes are historical.
- Single-package WASM notes involving `src/wasm/pkg` or `drummark_wasm.ts` are superseded by split packages.
- VexFlow-first renderer planning is superseded for product rendering, though still useful for legacy parity.
- Old notes describing app preview as single-page-only are superseded by page-aware preview rendering.
- Old notes recommending manual `wasm-pack` rebuilds are superseded by `npm run wasm:build`.

## Process Notes

- For technical obstacles, read source and official docs first.
- Verify parser, normalization, and rendering issues through `npm run drummark` in the relevant format:
  - `--format ast`
  - `--format ir`
  - `--format svg`
  - `--format xml`
- For significant DSL or architecture changes, use proposal files under `docs/proposals/`, sub-agent review, tasks files, explicit human stamp, implementation branch, pre-merge review, and archival.
- After this cleanup, append only concise, current-route learnings here. If a note is mainly historical context, place it in an archived document instead of bloating the active baseline.

## 2026-05-20 Tempo Layout Contract

- First-system tempo text is measure-owned layout content, not header-owned content.
- Header extraction for pagination should include only text-block children with no `measure_id`.
- System boxes must preserve measure-owned `TextBlock` composites such as tempo, otherwise semantic scene consumers lose the grouped tempo marker.

## 2026-05-20 Compact Repeat Boundary Parsing

- In Rust/WASM parser input, the compact shared repeat boundary `:|:` is lexed as `RepeatEnd` plus a trailing `Colon`, not as `RepeatEnd` plus `RepeatStart`, because the second half has no `|` character left for the `|:` token.
- The parser must interpret a standalone `Colon` as a repeat-start barline only when `parse_barline()` is already being asked for a measure boundary. Note suffix parsing such as `x:close` remains owned by `parse_suffix_chain()`.
- The legacy TypeScript parser has no lexer token for `:|:` either; handle it after consuming a repeat-end boundary by seeding the next left boundary as `repeat_start` and advancing past the extra colon, instead of adding `:` to the general boundary regex where it would collide with note modifiers.
- SMuFL provides a dedicated `repeatRightLeft` glyph at U+E042 for a right-and-left repeat sign. When rendering adjacent repeat-end/repeat-start boundaries, emit one semantic `repeat-end-start` glyph run with `GlyphRole::RepeatRightLeft` rather than separate `repeatRight` and next-measure `repeatLeft` glyphs.

## 2026-05-20 VexFlow Removal Planning

- At proposal-planning time, VexFlow remnants were legacy-only: `src/App.tsx` could lazy-import `./vexflow`, `build-docs.ts` imported `src/vexflow/index`, Vite optimized `vexflow/bravura`, `package.json` depended on `vexflow`, and some corpus/parity tests still imported VexFlow as an oracle.
- The approved active architecture remains `RenderScore -> LayoutScene -> thin platform adapter`; VexFlow removal should be deletion/route simplification plus oracle replacement, not a new renderer rewrite.
- Before deleting VexFlow-only tests, classify each test as obsolete, already covered by layout/adapter/CLI/corpus tests, or needing a new non-VexFlow regression.

## 2026-05-20 VexFlow Removal Implementation

- The active rendering ownership rule is `RenderScore -> LayoutScene -> thin platform adapter`.
- Legacy VexFlow removal is governed by `docs/proposals/ARCHITECTURE_proposal_remove_vexflow.md` and `docs/proposals/ARCHITECTURE_tasks_remove_vexflow.md`.
- During removal, do not use VexFlow output as an active oracle. Replace old parity coverage with layout scene snapshots, SVG semantic reports, adapter tests, CLI SVG tests, and corpus gates.

## 2026-05-20 Layout Event Spacing

- `measure_geometry()` owns note X placement inside `drummark-layout`; SVG adapters should not shift notes away from barlines.
- Event placement should use both event start slots and event end slots as spacing-cell boundaries. Centering only between adjacent starts incorrectly moves a lone short downbeat toward the middle of the measure.
- The first event in a measure should sit in the center of its rhythmic cell, not on the cell's left boundary. This creates clearance after left barlines/repeat starts while keeping the final event from leaving a full trailing beat of empty space.

## 2026-05-20 wasm-pack wasm-opt Feature Flags

- `wasm-pack` reads `[package.metadata.wasm-pack.profile.release].wasm-opt` from the crate `Cargo.toml`; specifying the array overrides the feature flags passed to `wasm-opt`.
- Recent Rust/LLVM wasm output can include `i32.trunc_sat_f32_s`, which requires Binaryen validation with `--enable-nontrapping-float-to-int`.
- If `wasm-opt` is configured with only `--enable-bulk-memory`, GitHub Actions can fail during `wasm-pack build --target web crates/drummark-core` with `unexpected false: all used features should be allowed`.
- The custom split-WASM build script invokes a standalone `wasm-bindgen` binary. Installing `wasm-pack` is not sufficient for CI; install `wasm-bindgen-cli` matching the locked `wasm-bindgen` crate version before `npm run build`.
