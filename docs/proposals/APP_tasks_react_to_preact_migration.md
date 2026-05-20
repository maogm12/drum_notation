# React to Preact Migration Tasks

### Task 1: Update Dependencies and Config
- [ ] **Status**: Pending
- **Scope**: `package.json`, `tsconfig.app.json`, `vite.config.ts`
- **Commits**: `chore: replace React with Preact — deps and config`
- **Acceptance Criteria**:
  - `react`, `react-dom`, `@types/react`, `@types/react-dom`, `@vitejs/plugin-react` removed from dependencies
  - `@radix-ui/react-slider` removed (unused)
  - `preact@^10.26.0` added as dependency
  - `@preact/preset-vite@^2.10.0` added as devDependency
  - `npm install` succeeds
  - `tsconfig.app.json` updated: `"jsxImportSource": "preact"`, `"types": []`, add `"paths"` aliases for `react`/`react-dom` → `preact/compat/src/index.d.ts`
  - `vite.config.ts`: `@vitejs/plugin-react` → `@preact/preset-vite`
  - `npm run build` succeeds
  - `npm run typecheck` (tsc -b) passes with zero errors
  - `npm test` passes (all vitest tests)
  - Bundle report: main bundle reduced by ~128 KB raw (~37 KB gzipped)
- **Dependencies**: None

### Task 2: Migrate App Entry (`main.tsx`)
- [ ] **Status**: Pending
- **Scope**: `src/main.tsx`
- **Commits**: `refactor: migrate main.tsx React imports to Preact`
- **Acceptance Criteria**:
  - `import React from "react"` → removed (`StrictMode` from `preact/compat` or removed entirely)
  - `import ReactDOM from "react-dom/client"` → `import { createRoot } from "preact/compat/client"`
  - App renders correctly at `/DrumMark/`
  - `npm run build` and `npm test` pass
- **Dependencies**: Task 1

### Task 3: Migrate App Component (`App.tsx`)
- [ ] **Status**: Pending
- **Scope**: `src/App.tsx`
- **Commits**: `refactor: migrate App.tsx React imports to Preact`
- **Acceptance Criteria**:
  - `import { ... } from "react"` → same names from `"preact/compat"`
  - `useSyncExternalStore` 3rd argument (`(): AppTheme => "light"`) removed
  - `UIEvent<HTMLDivElement>` import kept from `preact/compat` (type-compatible)
  - Editor, preview, XML view all render correctly
  - Theme toggle works (light/dark/system)
  - Tab switching works (Editor/Page/XML)
  - Zoom popover opens, closes on outside-click and Escape, traps focus
  - Resizer drag works
  - Print and export functionality unchanged
  - All interactive UI states verified manually
  - `npm run build` and `npm test` pass
- **Dependencies**: Task 1

### Task 4: Migrate Supporting Components
- [ ] **Status**: Pending
- **Scope**: `src/i18n/context.tsx`, `src/hooks/useAppSettings.ts`, `src/components/NumericSettingControl.tsx`
- **Commits**: `refactor: migrate remaining components React imports to Preact`
- **Acceptance Criteria**:
  - `i18n/context.tsx`: `from "react"` → `from "preact/compat"`, i18n works (locale switch, translations)
  - `useAppSettings.ts`: `from "react"` → `from "preact/compat"`, settings persist to localStorage
  - `NumericSettingControl.tsx`: `from "react"` → `from "preact/compat"`, stepper controls work
  - Settings panel renders, all sliders functional
  - `npm run build` and `npm test` pass
- **Dependencies**: Task 1

### Task 5: Migrate Tests
- [ ] **Status**: Pending
- **Scope**: `src/components/settings-panel.test.tsx`
- **Commits**: `test: migrate settings-panel test React imports to Preact`
- **Acceptance Criteria**:
  - `import React from "react"` → `import { flushSync, type ReactElement } from "preact/compat"`
  - `import { createRoot } from "react-dom/client"` → `import { createRoot } from "preact/compat/client"`
  - `import { flushSync } from "react-dom"` → removed (now from `preact/compat`)
  - `React.ReactElement` → `ReactElement`
  - All 4 test cases pass: renders without crashing (debug=false), renders debug sections (debug=true), toggle renders inside Notation, toggle shows unchecked state
  - `npm test` (full suite, ~440 tests) passes
- **Dependencies**: Task 4

### Task 6: Popover.Portal Manual Verification
- [ ] **Status**: Pending
- **Scope**: Manual testing only, no code changes expected
- **Commits**: None unless fix needed
- **Acceptance Criteria**:
  - Open zoom popover (click zoom button in Page view)
  - Click outside popover → closes ✓
  - Press Escape → closes ✓
  - Tab and Shift+Tab → focus trapped inside popover ✓
  - Switch browser tabs away and back → popover closes ✓
  - Click inside popover buttons → popover stays open ✓
  - If any test fails: document workaround, implement fix, re-test
- **Dependencies**: Task 3

### Task 7: MPA Build Verification
- [ ] **Status**: Pending
- **Scope**: Build output verification
- **Commits**: None unless fix needed
- **Acceptance Criteria**:
  - `npm run build` produces all three HTML outputs: `dist/index.html`, `dist/docs.html`, `dist/docs_zh.html`
  - All three pages load without JS errors when served from `/DrumMark/`
  - `manualChunks` (`vexflow`, `codemirror`) produce correct chunks with preact aliases
  - `npm run dev` serves all three pages without errors
  - No duplicate or missing Preact devtools script tags
- **Dependencies**: Task 3

### Task 8: Bundle Size Report
- [ ] **Status**: Pending
- **Scope**: `dist/bundle-report.json`
- **Commits**: `docs: update bundle report after Preact migration`
- **Acceptance Criteria**:
  - `npm run bundle:report` produces updated report
  - Main JS bundle size reduced from ~440 KB → ~312 KB (~29% reduction raw)
  - Main JS bundle gzipped reduced from ~140 KB → ~103 KB
  - Report shows zero `react`/`react-dom` in bundle
  - `opensheetmusicdisplay reachable: no` unchanged
- **Dependencies**: Task 7

### Task 9: Final Regression Check
- [ ] **Status**: Pending
- **Scope**: Full pipeline
- **Commits**: None unless fix needed
- **Acceptance Criteria**:
  - `npm test` — all tests pass
  - `npm run typecheck` — zero errors
  - `npm run typecheck:test` — zero errors
  - `npm run lint` — zero new warnings
  - `npm run build` — succeeds, all assets produced
  - `npm run drummark -- <example-input> --format svg` — correct SVG output
  - Manual smoke test: editor → type drummark → see preview → switch to Page view → zoom → export
- **Dependencies**: Tasks 1-8

### Terminal Supersession: VexFlow Chunk Expectation

This terminal note is appended to retire stale build-output expectations without rewriting the original task ledger.

Any acceptance criterion in this file that expects a `manualChunks` entry or emitted bundle chunk for `vexflow` is superseded by `docs/proposals/ARCHITECTURE_proposal_remove_vexflow.md` and `docs/proposals/ARCHITECTURE_tasks_remove_vexflow.md`.

The active build expectation is the opposite: production bundles, network audits, package metadata, Vite config, and TypeScript aliases must not retain VexFlow as a dependency, chunk, or runtime fetch target.
