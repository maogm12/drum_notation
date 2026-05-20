# Learnings

This file is the current architecture memory for DrumMark. Older chronological notes were archived to `docs/archived/LEARNINGS_legacy_2026-05-18.md` because the project changed technical direction several times and the old log had started to obscure the active path.

When an older note conflicts with this file, treat this file plus the active spec/proposal docs as authoritative.

## Current Architecture Baseline

### Product Rendering Path

- The default renderer is the platform-neutral Rust layout engine.
- VexFlow remains in the repo as a lazy legacy renderer for comparison, fallback, and migration debugging.
- New default-renderer work should target the Rust layout scene path, not VexFlow internals.
- VexFlow-specific learnings are still useful only for legacy-renderer fixes or visual parity investigations.

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
- Default layout/settings paths must not pull VexFlow runtime imports.
- Integration/parity tests may cross these boundaries only when explicitly scoped as integration/parity tests.

## UI And Settings Copy

- User-facing labels should use musical/product language, not implementation names.
- Prefer labels such as `Layout Engine` and `Legacy VexFlow`.
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
