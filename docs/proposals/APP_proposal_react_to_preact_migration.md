# React to Preact Migration Proposal

## Addendum v1.0: Replace React 19 + ReactDOM with Preact + preact/compat

### Goal

Replace React 19 (`react@^19.2.0`, `react-dom@^19.0.0`) with Preact (`preact@^10.x`) using the `preact/compat` compatibility layer. This reduces the framework payload from ~136 KB raw / ~40 KB gzipped to ~8 KB raw / ~3 KB gzipped, while preserving all existing component code, Radix UI integrations, and JSX syntax.

### Motivation

The project has a small React footprint (6 `.tsx` files, ~1,800 lines of component code) and uses only basic React APIs (hooks, context, memo). No advanced React 18/19 features (Suspense, ErrorBoundary, forwardRef, portals in app code) are used. The cost of React's synthetic event system, Fiber scheduler, and legacy compatibility layers (~128 KB raw) buys nothing for this project.

Preact + `preact/compat` provides a drop-in replacement:
- All standard hooks (`useState`, `useEffect`, `useRef`, `useCallback`, `useMemo`, `memo`, `createContext`, `useContext`) — identical API
- JSX — identical via `preact/compat`, same `jsx: "react-jsx"` tsconfig setting
- Radix UI compatibility — all Radix primitives work with `preact/compat` (they depend on React 18+ APIs that `preact/compat` implements)
- `dangerouslySetInnerHTML` — supported

### Changes

#### 1. Dependencies

| Action | Package | Version |
|--------|---------|---------|
| **Remove** | `react` | `^19.2.0` |
| **Remove** | `react-dom` | `^19.0.0` |
| **Remove** | `@types/react` | `^19.2.2` |
| **Remove** | `@types/react-dom` | `^19.2.2` |
| **Remove** | `@vitejs/plugin-react` | `^5.1.0` |
| **Remove** | `@radix-ui/react-slider` | `^1.3.6` (unused) |
| **Add** | `preact` | `^10.26.0` |
| **Add** | `@preact/preset-vite` | `^2.10.0` (devDependency) |

Radix UI packages (`@radix-ui/react-accordion`, `@radix-ui/react-popover`, `@radix-ui/react-switch`, `@radix-ui/react-tabs`) stay. Their peer dependency on `react` is satisfied by `preact/compat`'s alias.

#### 2. `vite.config.ts`

Replace `@vitejs/plugin-react` with `@preact/preset-vite`:

```diff
- import react from "@vitejs/plugin-react";
+ import preact from "@preact/preset-vite";

  export default defineConfig({
-   plugins: [react()],
+   plugins: [preact()],
```

`@preact/preset-vite` automatically:
- Aliases `react` → `preact/compat`
- Aliases `react-dom` → `preact/compat`
- Aliases `react/jsx-runtime` → `preact/jsx-runtime`
- Injects Preact devtools in development

#### 3. `tsconfig.app.json`

Remove `react` and `react-dom` from `types`, add `preact/compat` shim for `ReactNode`:

```diff
- "types": ["react", "react-dom"],
+ "types": [],
```

`preact/compat` exports types that match `@types/react` for the subset of APIs used. If any type (`ReactNode`, `UIEvent`) causes issues, they can be imported from `preact/compat` directly instead of `react`.

#### 4. Source Files: Import Changes

All `from "react"` imports become `from "preact/compat"`:

| File | Change |
|------|--------|
| `src/main.tsx` | `import React from "react"` → `import { StrictMode } from "preact/compat"`; `ReactDOM.createRoot` → import from `preact/compat` |
| `src/App.tsx` | `import { memo, useCallback, ... } from "react"` → `from "preact/compat"` |
| `src/i18n/context.tsx` | `import { createContext, ... } from "react"` → `from "preact/compat"` |
| `src/hooks/useAppSettings.ts` | `import { useEffect, useState } from "react"` → `from "preact/compat"` |
| `src/components/NumericSettingControl.tsx` | `import { useRef, useState, useEffect } from "react"` → `from "preact/compat"` |
| `src/components/settings-panel.test.tsx` | `import React from "react"` → `from "preact/compat"`; `createRoot from "react-dom/client"` → remove (see test changes below) |

#### 5. `useSyncExternalStore` — Rewrite

`preact/compat` does not export `useSyncExternalStore`. The single usage in `App.tsx` (line 574) subscribes to theme changes from `<html data-theme>` mutations:

```ts
// Before
const resolvedTheme: AppTheme = useSyncExternalStore(
  (listener) => subscribeToThemeChanges(listener),
  () => resolveDocumentTheme(),
  (): AppTheme => "light",
);

// After
const [resolvedTheme, setResolvedTheme] = useState<AppTheme>(() => resolveDocumentTheme());
useEffect(() => {
  setResolvedTheme(resolveDocumentTheme());
  return subscribeToThemeChanges(() => setResolvedTheme(resolveDocumentTheme()));
}, []);
```

The semantic difference is negligible: `useSyncExternalStore` prevents tearing during concurrent rendering, but Preact does not have concurrent rendering. The `useState` + `useEffect` pattern is functionally identical in Preact's synchronous render model.

#### 6. `flushSync` — Remove from Tests

`preact/compat` does not export `flushSync`. The test file `settings-panel.test.tsx` uses it 3 times for synchronous rendering guarantees. Replace with Vitest's `act()` from `preact/test-utils`:

```diff
- import { flushSync } from "react-dom";
+ import { act } from "preact/test-utils";

  function renderSync(jsx: React.ReactElement): HTMLElement {
    const container = document.createElement("div");
    const root = createRoot(container);
-   flushSync(() => {
+   act(() => {
      root.render(<I18nProvider>{jsx}</I18nProvider>);
    });
    return container;
  }
```

Similarly replace the 2 `flushSync` calls in `openAccordionItem`.

Alternatively, since Preact renders synchronously by default, `flushSync` calls can simply be removed — `root.render()` is already synchronous in Preact. The `act()` wrapper is still recommended by `@testing-library/preact` conventions.

#### 7. `React.StrictMode`

Preact compat supports `StrictMode` as a no-op. Can either keep it (harmless) or remove the wrapping element:

```diff
  root.render(
-   <React.StrictMode>
      <I18nProvider>
        <App />
      </I18nProvider>
-   </React.StrictMode>
  );
```

Recommended: remove `StrictMode` since Preact's `StrictMode` does nothing (no double-render detection).

#### 8. `memo` and `type ReactNode`

Preact compat exports `memo` with identical behavior. `ReactNode` type can be imported from `preact/compat`:

```ts
import { type ReactNode, memo } from "preact/compat";
```

If `ReactNode` causes issues, it can be defined locally in the one or two files that use it:

```ts
type ReactNode = import("preact/compat").ReactNode;
```

#### 9. `dangerouslySetInnerHTML`

`preact/compat` supports `dangerouslySetInnerHTML` with identical syntax. No changes needed.

### What Stays Untouched

| Surface | Rationale |
|---------|-----------|
| All Radix UI components | Compatible with `preact/compat` via alias. Tested upstream. |
| `Popover.Portal` + `asChild` | `preact/compat` implements `createPortal` and `cloneElement` — Radix works. |
| VexFlow rendering | Completely unaffected. |
| CodeMirror editor | Completely unaffected. |
| Worker, state, export, print | Completely unaffected. |
| i18n system | Completely unaffected. |
| CSS / theme system | Completely unaffected. |
| All `.test.ts` files (except `settings-panel.test.tsx`) | No React imports. |

### Bundle Impact

| Item | Before (raw) | After (raw) | Delta |
|------|-------------|-------------|-------|
| React | ~6 KB | 0 | -6 KB |
| ReactDOM | ~130 KB | 0 | -130 KB |
| Preact + compat | 0 | ~8 KB | +8 KB |
| **Net framework** | **~136 KB** | **~8 KB** | **~-128 KB** |

Gzipped: ~40 KB → ~3 KB, saving ~37 KB from the main bundle.

Plus removal of unused `@radix-ui/react-slider` (~14 KB raw / ~4 KB gzipped).

### Risks

1. **Radix + Preact compat edge cases**: While Radix officially works with Preact via compat, `Popover.Portal`'s `asChild` pattern uses `cloneElement` which has subtle differences in Preact. Manual verification required.

2. **`useSyncExternalStore` behavioral change**: The rewrite to `useState` + `useEffect` is functionally identical in Preact's synchronous model, but should be tested during theme toggle + system dark mode detection.

3. **Third-party library compatibility**: Any library that internally depends on React's `createElement` or uses `react-reconciler` directly would break. Currently none do — VexFlow, CodeMirror, and Radix are the only external UI deps.

4. **`@preact/preset-vite` maintenance**: The preset is actively maintained and used by the Preact team, but it's a smaller community than React's.

5. **Dev tooling**: React DevTools won't work. Preact DevTools browser extension is available but less polished.

---

### Review Round 1

**Reviewer**: Critical Architect Review  
**Date**: 2026-05-09

---

#### Finding 1: CRITICAL — Missing `jsxImportSource` will break all TypeScript JSX type-checking

The proposal sets `"types": []` in `tsconfig.app.json` and removes `@types/react` and `@types/react-dom`. However, it does **not** add `"jsxImportSource": "preact"` to the tsconfig.

TypeScript resolves JSX types from the module specified by `jsxImportSource` (defaults to `"react"` when `jsx: "react-jsx"`). Without `@types/react` installed and without `jsxImportSource: "preact"`, **every JSX element in every `.tsx` file will fail type-checking** — TypeScript won't know the types for intrinsic elements (`div`, `button`, `span`, etc.) or fragment syntax (`<>...</>`).

`@preact/preset-vite` only handles **Vite bundling** (esbuild transforms + aliases). It does NOT configure TypeScript's type-checker (`tsc -b`). The `jsxImportSource` must be set in tsconfig explicitly.

**Required fix**: Add `"jsxImportSource": "preact"` to `compilerOptions` in `tsconfig.app.json`. This tells TypeScript to:
1. Resolve JSX runtime from `preact/jsx-runtime` (instead of `react/jsx-runtime`)
2. Use Preact's JSX namespace types (`JSX.Element`, `JSX.IntrinsicElements`, etc.) from `preact/src/jsx`

Note: with `jsxImportSource: "preact"`, Preact's event handler types differ from React's. Preact uses `JSX.TargetedEvent<T, Event>` instead of React's `SyntheticEvent<T>`. This is relevant to Finding 2.

---

#### Finding 2: CRITICAL — `UIEvent<HTMLDivElement>` type is React-specific and won't compile

At `src/App.tsx:383`:
```ts
function handleScroll(e: UIEvent<HTMLDivElement>) { ... }
```

This uses React's synthetic `UIEvent<T>` type with a generic element parameter. This type is defined in `@types/react` and does **not** exist in `preact/compat`. Preact events are native DOM events — native `UIEvent` has no generic type parameter.

The `handleScroll` function is passed as `onScroll={handleScroll}` on a `<div>`. With Preact's JSX types, `onScroll` expects `JSX.UIEventHandler<HTMLDivElement>` → `(event: TargetedUIEvent<HTMLDivElement>) => void`, where `TargetedUIEvent.currentTarget` is typed as `HTMLDivElement`. So the function body `e.currentTarget.scrollTop` works with inferred types.

**Required fix**: Remove the explicit `UIEvent<HTMLDivElement>` annotation and let TypeScript infer the type from the `onScroll` prop. The cleanest change:

Option A (recommended): Remove the annotation entirely, let inference handle it:
```ts
// TypeScript infers e from the onScroll prop type
function handleScroll(e) {
```

Option B (explicit): Use Preact's event type from JSX namespace:
```ts
function handleScroll(e: import("preact/compat").JSX.TargetedUIEvent<HTMLDivElement>) {
```

The inline `MouseEvent` on line 761 and `TouchEvent` on line 766 use native DOM types on `document.addEventListener` calls — those are fine and unaffected.

---

#### Finding 3: HIGH — `flushSync` DOES exist in `preact/compat` ≥10.19.0, invalidating test migration strategy

The proposal states: "`preact/compat` does not export `flushSync`." This is **incorrect** for the target version.

`flushSync` was added to `preact/compat` in Preact 10.19.0 (October 2023, PR #4399). The proposal targets `preact@^10.26.0` — well past 10.19.0.

This means:
1. The test migration in Section 6 is overcomplicated and partially wrong
2. `act()` from `preact/test-utils` is async (returns a Promise) — the synchronous `flushSync()` calls in the test would break with `act()`
3. The correct migration is trivial:

```diff
- import { createRoot } from "react-dom/client";
+ import { createRoot } from "preact/compat";
- import { flushSync } from "react-dom";
+ import { flushSync } from "preact/compat";
```

The `renderSync` function and `openAccordionItem` function bodies remain syntactically identical. No `act()` needed.

**Required fix**: Update Section 6 to use `flushSync` and `createRoot` from `preact/compat` instead of `act()` from `preact/test-utils`.

---

#### Finding 4: HIGH — `Popover.Portal` event bubbling has known Preact compat edge cases

The proposal dismisses this risk with "Preact compat implements createPortal and cloneElement — Radix works." This is insufficiently skeptical.

While Radix UI broadly works with Preact compat, `Popover.Portal` + `createPortal` has **known differences** in Preact:

- **Event dispatching**: Preact dispatches events through the virtual DOM tree, NOT through real DOM. React portals preserve native event propagation through the real DOM. Radix's `DismissableLayer` registers `pointerdown` on `document` (native DOM), which should work since it uses `document.addEventListener` directly. However, the interaction between Preact's VNode-tree event dispatching and Radix's real-DOM listeners has caused [historically documented issues](https://github.com/preactjs/preact/issues/3574) with portal-based components.

- **Focus management**: Radix's `FocusScope` inside portals uses React's synthetic focus/blur events. Preact compat normalizes focus events differently. Focus trapping within the popover (Tab/Shift+Tab cycling) is the most likely breakage point.

**Required fix**: This risk MUST be explicitly manually tested before implementation approval:
1. Open the zoom popover (`Popover.Trigger asChild` at `src/App.tsx:1133`)
2. Click outside the popover — verify it closes
3. Press Escape — verify it closes
4. Tab and Shift+Tab — verify focus is trapped inside the popover
5. Open popover, switch browser tabs away, switch back — verify popover closes
6. Open popover, click inside the popover buttons — verify popover stays open

If any test fails, additional workarounds may be needed (e.g., wrapping `Popover` in a custom component or adding explicit event listeners).

---

#### Finding 5: HIGH — `Popover.Trigger asChild` with `cloneElement` — ref merging risks

At `src/App.tsx:1133`:
```tsx
<Popover.Trigger asChild>
  <button ...>
```

Radix's `asChild` pattern uses `cloneElement(child, mergedProps)` to merge its own props (`ref`, `data-state`, `aria-*`, event handlers) onto the child `<button>`. Preact compat's `cloneElement` has historically had bugs with `ref` merging — specifically, when both the parent (Radix Trigger) and child (button) set a `ref`, Preact may lose one of the refs.

In this specific case, the `<button>` child has NO `ref` prop — only Radix sets one, so there's no conflict. However, this is fragile: if any future code adds a `ref` to a child of an `asChild` component, it would silently break.

**Required fix**: Document this as a known limitation. Audit all `asChild` usages (currently only one at line 1133) to confirm no child `ref` conflicts exist.

---

#### Finding 6: MODERATE — `React.ReactElement` type in test not explicitly addressed

The test at `src/components/settings-panel.test.tsx:26`:
```ts
function renderSync(jsx: React.ReactElement): HTMLElement {
```

The proposal's import changes table says `import React from "react"` → `from "preact/compat"`, but doesn't explicitly address the `React.ReactElement` type usage in the function signature. After migration:
```ts
import { createRoot, flushSync, type ReactElement } from "preact/compat";
function renderSync(jsx: ReactElement): HTMLElement {
```

Or more idiomatically, since `render()` accepts any Preact component child:
```ts
import { type ComponentChild } from "preact";
function renderSync(jsx: ComponentChild): HTMLElement {
```

**Required fix**: Update Section 4's test file row to explicitly show the type migration path.

---

#### Finding 7: MODERATE — `@preact/preset-vite` MPA mode compatibility not verified

The `vite.config.ts` sets `appType: "mpa"` with three HTML entry points (`index.html`, `docs.html`, `docs_zh.html`). `@preact/preset-vite`'s alias injection and devtools script injection apply per-build. The preset should handle MPA mode correctly (aliases work at the Vite resolver level, not per-entry), but this needs explicit verification.

Specifically: in MPA mode, Vite generates separate bundles per entry page. The preset might inject the Preact devtools hook into each entry. With three entries, this could cause:
- Duplicate devtools registration
- Missing injection on docs pages (if the preset only looks at the default entry)
- Incorrect chunk splitting with the `manualChunks` config

**Required fix**: After applying the preset, run `npm run build` and verify:
1. All three HTML outputs (`dist/index.html`, `dist/docs.html`, `dist/docs_zh.html`) have correct Preact compat aliases resolved
2. No duplicate or missing Preact devtools script tags
3. The `manualChunks` config (`vexflow`, `codemirror`) still produces correct chunks with preact aliases
4. `npm run dev` serves all three pages without errors

---

#### Finding 8: MODERATE — `useSyncExternalStore` rewrite has a one-render-lag behavioral difference

The rewrite in Section 5 replaces `useSyncExternalStore` (synchronous snapshot reads during render) with `useState` + `useEffect` (reads after mount, triggers re-render on change).

Behavioral difference: When the external theme changes (system dark mode toggle, MutationObserver on `data-theme`), `useSyncExternalStore` returns the new value synchronously in the same render. `useState` + `useEffect` requires a **second render** — the effect callback fires `setResolvedTheme()` which schedules a re-render.

This is imperceptible because:
- User-initiated theme toggles (clicking the theme button) already trigger a render via `toggleTheme()` → DOM mutation → `subscribeToThemeChanges` callback → `setResolvedTheme()`
- System theme changes (OS dark mode toggle) are infrequent
- Preact renders synchronously in microtask batches

**Required fix**: Document the one-render-lag explicitly in the proposal rather than calling it "negligible" without explanation.

---

#### Finding 9: LOW — `tsconfig.test.json` inherits types change, verify `typecheck:test` still passes

`tsconfig.test.json` extends `tsconfig.app.json` and inherits `"types": []` (after the proposed change). The test file `settings-panel.test.tsx` needs `preact/compat` types for `createRoot`, `flushSync`, and `ReactElement`. When `jsxImportSource` is added to `tsconfig.app.json`, it propagates to the test config, providing JSX types.

**Required fix**: After migration, run `npm run typecheck:test` (i.e., `tsc -p tsconfig.test.json --noEmit`) to confirm test files type-check correctly.

---

#### Finding 10: LOW — SVG attribute casing is DOM-native (currently fine, but no guard)

`App.tsx` contains inline SVG at lines 188-201 with CSS custom properties (`var(--accent-primary)`, `var(--text-main)`). React normalizes SVG attribute casing (e.g., `strokeWidth` → `stroke-width`). Preact passes SVG attributes as-is to the DOM.

The current SVG uses **DOM-native casing** (`stroke-linecap`, `stroke-linejoin`, `stroke-opacity`) — which is correct for both React and Preact. No change needed. But if any future SVG were added with React-style camelCase attributes (e.g., `strokeWidth`, `fillOpacity`), they would silently render incorrectly in Preact.

**Verification**: Current App.tsx SVG is correct. Worth noting as a codebase convention.

---

#### Positive Confirmations

- No `useId`, `useLayoutEffect`, `useTransition`, `useDeferredValue`, `forwardRef`, `startTransition`, `createRef`, `PureComponent`, or `Component` class usages anywhere in `src/`
- VexFlow renderer (`src/vexflow/renderer.ts`) has zero React imports — confirmed unaffected
- `SettingsPanel.tsx` has zero React imports — requires no changes at all (JSX transform handles it)
- `dangerouslySetInnerHTML` at `App.tsx:461` is supported identically by `preact/compat`
- All Radix UI packages (`accordion`, `popover`, `switch`, `tabs`) are compatible with `preact/compat` at the API level
- `memo` wrapping `PagePreview` at `App.tsx:335` is supported by `preact/compat`
- Only 6 `.tsx` files exist total, all identified and accounted for

---

### Summary

| # | Severity | Issue | Section |
|---|----------|-------|---------|
| 1 | CRITICAL | Missing `jsxImportSource: "preact"` in tsconfig | §3 |
| 2 | CRITICAL | `UIEvent<HTMLDivElement>` type won't compile in Preact | §4 (`App.tsx`) |
| 3 | HIGH | `flushSync` IS available in target preact version — test strategy wrong | §6 |
| 4 | HIGH | `Popover.Portal` event/focus edge cases need manual testing | Risks §1 |
| 5 | HIGH | `asChild` + `cloneElement` ref merging risk | Risks §1 |
| 6 | MODERATE | `React.ReactElement` type in test not addressed | §6 |
| 7 | MODERATE | MPA mode + `@preact/preset-vite` not verified | §2 |
| 8 | MODERATE | `useSyncExternalStore` rewrite lag not explicitly documented | §5 |
| 9 | LOW | `tsconfig.test.json` inheritance needs verification | §3 |
| 10 | LOW | SVG camelCase attribute risk (currently fine, but no guard) | §8 |

---

**STATUS: CHANGES_REQUESTED**

The migration is well-scoped and the bundle savings are real, but the proposal has two critical TypeScript configuration gaps (Findings 1, 2) and an incorrect claim about `flushSync` availability (Finding 3) that would cause unnecessary complexity in the test migration. The Radix `Popover.Portal` risk (Finding 4) is under-scrutinized for a UI that uses it as a primary interaction pattern.

All 10 findings must be addressed in an Author Response before this can move to APPROVED.

---

### Author Response

**Date**: 2026-05-09

---

#### Finding 1 (CRITICAL): Missing `jsxImportSource`

**Accepted.** The proposal's Section 3 (`tsconfig.app.json`) incorrectly assumed `@preact/preset-vite` would handle TypeScript type-checking. It only handles Vite bundling. The fix:

```jsonc
// tsconfig.app.json
{
  "compilerOptions": {
    "jsx": "react-jsx",
    "jsxImportSource": "preact",  // <-- ADD
    "types": [],
    // ... rest unchanged
  }
}
```

This causes TypeScript to resolve JSX types from `preact/jsx-runtime` instead of `react/jsx-runtime`. Note: `preact/compat` exports JSX types compatible with `preact/jsx-runtime`, so no conflict between compat aliases and JSX types.

**Updated Section 3** incorporates this change.

---

#### Finding 2 (CRITICAL): `UIEvent<HTMLDivElement>` type

**Accepted.** The explicit `UIEvent<HTMLDivElement>` annotation at `App.tsx:383` will fail type-checking after migration because Preact's JSX types use `TargetedUIEvent<HTMLDivElement>` (from the Preact JSX namespace), not React's synthetic `UIEvent<T>`.

Fix: Remove the annotation, let TypeScript infer from the `onScroll` prop:

```diff
- function handleScroll(e: UIEvent<HTMLDivElement>) {
+ function handleScroll(e: any) {
```

Or even better — TypeScript can infer without `any` in strict mode when the function is used directly as a JSX handler. Since `onScroll` on a `<div>` infers `(e: JSX.TargetedUIEvent<HTMLDivElement>) => void`, this works:

```ts
function handleScroll(e: { currentTarget: HTMLDivElement }) {
```

The function body only accesses `e.currentTarget.scrollTop` — so a minimal structural type suffices.

Import change: `type UIEvent` is removed from the `from "react"` import. The renamed import from `preact/compat` simply drops that member.

The inline `MouseEvent` (line 761) and `TouchEvent` (line 766) on `document.addEventListener` calls use native DOM types — unaffected.

**Updated Section 4** incorporates this.

---

#### Finding 3 (HIGH): `flushSync` IS available in `preact/compat`

**Accepted with thanks for the correction.** I missed that `flushSync` was added in `preact/compat` 10.19.0 (Oct 2023). The target version `^10.26.0` includes it.

**Revised Section 6**: Remove all discussion of `act()` from `preact/test-utils`. The test file migrates trivially:

```diff
// src/components/settings-panel.test.tsx
- import React from "react";
- import { createRoot } from "react-dom/client";
- import { flushSync } from "react-dom";
+ import { createRoot, flushSync } from "preact/compat";
```

The `renderSync` and `openAccordionItem` functions require zero body changes. Note: `React.ReactElement` → `import type { ReactElement } from "preact/compat"` (covered in Finding 6).

---

#### Finding 4 (HIGH): `Popover.Portal` event/focus edge cases

**Accepted.** My previous language was too dismissive. The manual test plan (6 checks) is accepted verbatim and added to the proposal's risk section as a mandatory pre-approval verification task.

---

#### Finding 5 (HIGH): `asChild` + `cloneElement` ref merging

**Accepted.** I agree this is fragile. Current audit: the only `asChild` usage is `App.tsx:1133` with a `<button>` child that has zero `ref` props. No conflict today.

**Action**: Add to Section 8 (Risks) as a documented limitation and add a lint/audit task: any future addition of `ref` to an `asChild` child must be tested against Preact compat.

---

#### Finding 6 (MODERATE): `React.ReactElement` type in test

**Accepted.** The updated test import path is:

```diff
- import React from "react";
+ import type { ReactElement } from "preact/compat";
  import { createRoot, flushSync } from "preact/compat";

- function renderSync(jsx: React.ReactElement): HTMLElement {
+ function renderSync(jsx: ReactElement): HTMLElement {
```

Alternatively, since all callers pass JSX expressions, typing as `preact.ComponentChild` is more permissive and equally correct. I'll use `ReactElement` for minimal diff.

**Updated Section 6** incorporates this.

---

#### Finding 7 (MODERATE): MPA mode + `@preact/preset-vite`

**Accepted.** The MPA compatibility is documented as a verification requirement in the tasks file (not the proposal — build verification is an implementation step). The `@preact/preset-vite` works at the Vite resolver alias level, which applies across all entries. But the explicit build verification check (all 4 items) will be added to the tasks file as an acceptance criterion.

---

#### Finding 8 (MODERATE): `useSyncExternalStore` rewrite has one-render lag

**Accepted.** I'll replace "negligible" with the explicit analysis:

> **Behavioral difference**: `useSyncExternalStore` returns the new value synchronously during the same render when the store mutates. `useState` + `useEffect` triggers a **second render** after the effect callback fires. This is imperceptible because: (a) user theme toggles already go through DOM mutation → listener callback → `setResolvedTheme()` which re-renders; (b) system theme changes (OS dark mode) are infrequent single events; (c) Preact renders synchronously in microtask batches so there's no visible intermediate state.

**Updated Section 5** incorporates this explicit analysis.

---

#### Finding 9 (LOW): `tsconfig.test.json` inheritance

**Accepted.** Adding `"jsxImportSource": "preact"` to `tsconfig.app.json` propagates to `tsconfig.test.json` (which extends it). Running `tsc -p tsconfig.test.json --noEmit` post-migration is a mandatory verification step. Added to Section 3 verification notes.

---

#### Finding 10 (LOW): SVG camelCase attribute risk

**Accepted.** Current SVG in `App.tsx:188-201` uses DOM-native casing and is safe. Added as a codebase convention note: Preact renders SVG attribute names as-is; all future SVG in JSX must use native attribute casing (`stroke-width`, not `strokeWidth`).

---

### Revised Sections Summary

The following proposal sections are updated to incorporate reviewer feedback:

| Section | Changes |
|---------|---------|
| §3 (`tsconfig.app.json`) | Add `"jsxImportSource": "preact"`, add verification note about `tsconfig.test.json` |
| §4 (Source Files) | Remove `UIEvent` from `App.tsx` import; add `ReactElement` type to test import; fix `createRoot` import to `preact/compat/client` |
| §5 (`useSyncExternalStore`) | **Superseded — no rewrite needed.** `useSyncExternalStore(subscribe, getSnapshot)` IS exported from `preact/compat` (2-arg, no server-snapshot). Only drop the 3rd arg and change the import path. |
| §6 (`flushSync` / tests) | `flushSync` from `preact/compat`; `createRoot` from `preact/compat/client` |
| §8 (Risks) | Add `Popover.Portal` manual test plan (6 checks), document `asChild` ref limitation, add SVG casing convention note |

---

### Review Round 2

**Reviewer**: Critical Architect Review  
**Date**: 2026-05-09

---

#### Finding N1: CRITICAL — `useSyncExternalStore` IS available from `preact/compat`, making Section 5's rewrite entirely unnecessary

Verified by source inspection (`node_modules/preact/compat/src/hooks.js`):

```js
export function useSyncExternalStore(subscribe, getSnapshot) {
```

`preact/compat` exports a 2-argument `useSyncExternalStore(subscribe, getSnapshot)`. It does NOT accept the optional 3rd server-snapshot argument (`getServerSnapshot`) that React provides for SSR. Since this project has no SSR, the 3rd argument is never used (it's set to `(): AppTheme => "light"` as a dummy value).

**The correct migration**:
```diff
- import { memo, useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore, type ReactNode, type UIEvent } from "react";
+ import { memo, useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore, type ReactNode } from "preact/compat";

  const resolvedTheme: AppTheme = useSyncExternalStore(
    (listener) => subscribeToThemeChanges(listener),
    () => resolveDocumentTheme(),
-   (): AppTheme => "light",
  );
```

Drop the 3rd argument, change `"react"` to `"preact/compat"`. That's it. Zero behavioral change — Preact compat's `useSyncExternalStore` is a faithful port of React's shim with the same getSnapshot-during-render semantics.

This also renders **Finding 8 (MODERATE)** moot — no `useState` + `useEffect` rewrite, no one-render-lag to document.

**Action**: Section 5 is entirely replaced with: "Drop 3rd argument from `useSyncExternalStore` call. Import from `preact/compat`."

---

#### Finding N2: MODERATE — `createRoot` is at `preact/compat/client`, not `preact/compat`

Verified by source (`node_modules/preact/compat/client.js`):

```js
const { render, hydrate, unmountComponentAtNode } = require('preact/compat');

function createRoot(container) {
    return {
        render: function (children) { render(children, container); },
        unmount: function () { unmountComponentAtNode(container); }
    };
}
exports.createRoot = createRoot;
```

The correct test import path:
```diff
- import { createRoot } from "react-dom/client";
+ import { createRoot } from "preact/compat/client";
```

And the `main.tsx` entry point import:
```diff
- import ReactDOM from "react-dom/client";
+ import { createRoot } from "preact/compat/client";
```

Note: `preact/compat` itself does NOT export `createRoot`. The `@preact/preset-vite` aliases `react-dom` → `preact/compat` and `react-dom/client` → `preact/compat/client`, so code that does `from "react-dom/client"` works automatically through aliases. But since we're removing the alias layer and explicitly importing, the correct path is `preact/compat/client`.

**Action**: Update Section 4 and Section 6 to use `preact/compat/client` for `createRoot` imports.

---

#### Finding N3 (NOTE): `UIEvent<T>` IS also exported from `preact/compat`

Additional discovery: `preact/compat`'s type definitions (`index.d.ts`) declare `UIEvent<T>` with the same generic signature as React's. The structural type fix (`{ currentTarget: HTMLDivElement }`) from Round 1 Finding 2 still works, but the simpler option is to just keep importing it from `preact/compat`.

For `preact/compat` ≥ 10.19.0:
```ts
import { type UIEvent } from "preact/compat";
// UIEvent<HTMLDivElement> works identically
```

**Action**: The Author Response for Finding 2 should prefer the simpler fix: keep importing `UIEvent` from `preact/compat`. The structural type remains the fallback if type resolution fails at `tsc -b` time.

---

#### All Round 1 Findings Re-Evaluated

| # | Status | Notes |
|---|--------|-------|
| F1 | RESOLVED | `jsxImportSource: "preact"` accepted and incorporated |
| F2 | RESOLVED | `UIEvent` from `preact/compat` works; structural type fallback available |
| F3 | RESOLVED (with correction) | `flushSync` from `preact/compat` is correct; `createRoot` path fixed to `preact/compat/client` |
| F4 | RESOLVED | `Popover.Portal` test plan accepted |
| F5 | RESOLVED | `asChild` ref limitation documented |
| F6 | RESOLVED | `ReactElement` from `preact/compat` incorporated |
| F7 | RESOLVED | MPA verification added to tasks |
| F8 | MOOT | `useSyncExternalStore` rewrite not needed — Preact compat exports it natively |
| F9 | RESOLVED | `tsconfig.test.json` verification added |
| F10 | RESOLVED | SVG casing convention noted |

---

### Summary

| # | Severity | Issue |
|---|----------|-------|
| N1 | CRITICAL | `useSyncExternalStore` IS available from `preact/compat` (2-arg). Section 5 entire rewrite is unnecessary — just drop 3rd arg. |
| N2 | MODERATE | `createRoot` is at `preact/compat/client`, not `preact/compat`. Import paths corrected. |
| N3 | NOTE | `UIEvent<T>` IS also available from `preact/compat` — simpler than the structural type fix. |

---

**STATUS: CHANGES_REQUESTED**

Two actionable corrections: (1) remove the Section 5 rewrite and use native `useSyncExternalStore` from Preact compat, and (2) fix `createRoot` import to `preact/compat/client`. Both are trivial. The proposal is otherwise sound.

---

### Author Response (Round 2)

**Date**: 2026-05-09

---

#### Finding N1 (CRITICAL): `useSyncExternalStore` in preact/compat

**Accepted.** I confirmed by source inspection — `preact/compat/src/hooks.js` exports a 2-argument `useSyncExternalStore(subscribe, getSnapshot)` that is a faithful port of React's shim. No behavioral change, no lag, no rewrite needed.

**Revised plan**: Simply drop the 3rd server-snapshot argument and change the import path. Section 5 is struck from the proposal entirely — no code changes needed other than the import alias.

**Updated Section 4 (App.tsx import line)**:
```diff
- import { memo, useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore, type ReactNode, type UIEvent } from "react";
+ import { memo, useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore, type ReactNode, type UIEvent } from "preact/compat";
```

And the 3rd arg removal:
```diff
  const resolvedTheme: AppTheme = useSyncExternalStore(
    (listener) => subscribeToThemeChanges(listener),
    () => resolveDocumentTheme(),
-   (): AppTheme => "light",
  );
```

---

#### Finding N2 (MODERATE): `createRoot` at `preact/compat/client`

**Accepted.** `preact/compat` does not export `createRoot` directly — it's in a separate `preact/compat/client` entry point (mirrors React's `react-dom/client` convention). Import paths corrected:

`src/main.tsx`:
```diff
- import ReactDOM from "react-dom/client";
+ import { createRoot } from "preact/compat/client";
```

`src/components/settings-panel.test.tsx`:
```diff
- import { createRoot } from "react-dom/client";
+ import { createRoot } from "preact/compat/client";
```

---

#### Finding N3 (NOTE): `UIEvent<T>` in preact/compat

**Accepted.** I'll keep the `UIEvent` import from `preact/compat` as the primary path (one-liner diff). The structural type remains the documented fallback if `tsc -b` type resolution fails.

---

### Consolidated Final Import Map

| Source File | React Import | Preact Import |
|---|---|---|
| `main.tsx` | `React from "react"` + `ReactDOM from "react-dom/client"` | `import { StrictMode } from "preact/compat"` + `import { createRoot } from "preact/compat/client"` |
| `App.tsx` | `{ memo, useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore, type ReactNode, type UIEvent } from "react"` | Same names from `"preact/compat"` |
| `i18n/context.tsx` | `{ createContext, useCallback, useContext, useMemo, useState, type ReactNode } from "react"` | Same names from `"preact/compat"` |
| `useAppSettings.ts` | `{ useEffect, useState } from "react"` | Same from `"preact/compat"` |
| `NumericSettingControl.tsx` | `{ useRef, useState, useEffect } from "react"` | Same from `"preact/compat"` |
| `settings-panel.test.tsx` | `React from "react"` + `createRoot from "react-dom/client"` + `flushSync from "react-dom"` | `import { flushSync, type ReactElement } from "preact/compat"` + `import { createRoot } from "preact/compat/client"` |

---

### Review Round 3

**Reviewer**: Critical Architect Review  
**Date**: 2026-05-09

---

#### N1 & N2 Status

☑ **N1 (`useSyncExternalStore`)** — RESOLVED. Native Preact compat export verified. Section 5 rewrite correctly eliminated.

☑ **N2 (`createRoot` path)** — RESOLVED. `createRoot` from `preact/compat/client` verified. Import map corrected.

#### Consolidated Import Map Verification

All imports verified against actual `preact/compat` source files:

| Import | Source File | Line | Status |
|--------|------------|------|--------|
| `StrictMode` | `preact/compat/src/index.js` | 72: `Fragment as StrictMode` | ☑ |
| `useSyncExternalStore` | `preact/compat/src/hooks.js` | 39-42 | ☑ |
| `ReactNode` | `preact/compat/src/index.d.ts` | — | ☑ |
| `ReactElement` | `preact/compat/src/index.d.ts` | — | ☑ |
| `UIEvent<T>` | `preact/compat/src/index.d.ts` | — | ☑ |
| `flushSync` | `preact/compat/src/index.js` | — | ☑ |
| `createRoot` | `preact/compat/client.js` | — | ☑ |

`SettingsPanel.tsx` — confirmed zero React imports, needs no changes.

#### Round 1 Findings Re-Check

All 10 findings resolved or moot:

| # | Status | Rationale |
|---|--------|-----------|
| F1 | ☑ | `jsxImportSource: "preact"` added |
| F2 | ☑ | `UIEvent<T>` available from `preact/compat` |
| F3 | ☑ | `flushSync` from `preact/compat`, `createRoot` from `preact/compat/client` |
| F4 | ☑ | `Popover.Portal` test plan accepted |
| F5 | ☑ | `asChild` ref limitation documented |
| F6 | ☑ | `ReactElement` from `preact/compat` |
| F7 | ☑ | MPA verification in tasks |
| F8 | ☑ | Moot — `useSyncExternalStore` native in Preact |
| F9 | ☑ | `tsconfig.test.json` verification |
| F10 | ☑ | SVG casing convention noted |

---

#### Finding R3-C1: CRITICAL — Missing tsconfig `paths` alias for Radix `.d.ts` resolution

**Scope**: `tsconfig.app.json`

All 4 Radix UI packages have `.d.ts` files that import from `react`:
- `@radix-ui/react-tabs/dist/index.d.ts` → `import * as React from "react"`
- `@radix-ui/react-popover/dist/index.d.ts` → `import * as React from "react"`
- `@radix-ui/react-accordion/dist/index.d.ts` → `import * as React from "react"`
- `@radix-ui/react-switch/dist/index.d.ts` → `import * as React from "react"`

After removing `react` from `node_modules` and removing `@types/react`, TypeScript has no module named `"react"` to resolve these imports. The proposal's claim that `skipLibCheck: true` handles this is **incorrect** — `skipLibCheck` suppresses type *errors* in `.d.ts` files, but does NOT suppress module *resolution* failures. TypeScript will report `TS2307: Cannot find module 'react'` at compile time for Radix's declaration files, and `tsc -b` will fail.

**Required fix**: Add `paths` aliases to `tsconfig.app.json`:

```jsonc
{
  "compilerOptions": {
    "jsx": "react-jsx",
    "jsxImportSource": "preact",
    "types": [],
    "paths": {
      "react": ["./node_modules/preact/compat/src/index.d.ts"],
      "react-dom": ["./node_modules/preact/compat/src/index.d.ts"]
    },
    "baseUrl": ".",
    // ... rest unchanged
  }
}
```

Note: `baseUrl` is required when using non-relative `paths`. Set `"baseUrl": "."` (project root) since `node_modules` is at project root.

Alternatively, the more portable approach used by Preact projects:

```jsonc
{
  "compilerOptions": {
    "types": ["preact/compat/src/index.d.ts"],
    // ...
  }
}
```

But since Radix's `.d.ts` files explicitly `import from "react"` (not a global reference), `types` won't help — they need the module to be resolvable. `paths` is the correct solution.

**Verification**: After adding `paths`, run `tsc -p tsconfig.app.json --noEmit` and confirm zero errors.

---

#### Finding R3-C2: LOW — `StrictMode` import inconsistency in import map

The Consolidated Import Map says `import { StrictMode } from "preact/compat"` for `main.tsx`. This imports `Fragment` re-exported as `StrictMode`. It works but is semantically misleading — `StrictMode` in Preact is a no-op. The proposal should note that removing `StrictMode` wrapping is a one-line cleanup, not a breaking change.

**Action**: Document in proposal that `StrictMode` wrapping can either be kept (harmless) or removed (cleaner). Both are valid.

---

#### Finding R3-C3: LOW — Duplicate Review Round 2 sections in file

There are TWO instances of `### Review Round 2` and `### Author Response (Round 2)` in the file. The first Author Response (after the first Review Round 1) accidentally includes the second round as pasted content within a code block. The file structure is:

```
### Review Round 1
### Author Response
  [Revised Sections Summary table with Review Round 2 embedded]
### Review Round 2
### Author Response (Round 2)
  [Consolidated Final Import Map]
### Review Round 3  <-- this review
```

The content is not lost — it's duplicated. No functional impact on the proposal's correctness. Recommendation: clean up duplicate during consolidation phase.

---

### Summary

| # | Severity | Issue |
|---|----------|-------|
| R3-C1 | CRITICAL | Missing tsconfig `paths` alias — Radix `.d.ts` files can't resolve `import from "react"` after removing `react` |
| R3-C2 | LOW | `StrictMode` import is `Fragment` alias — document cleanup option |
| R3-C3 | LOW | Duplicate Review Round 2 content — cleanup during consolidation |

---

**STATUS: CHANGES_REQUESTED**

One critical fix: add `paths` aliases for `react`/`react-dom` in `tsconfig.app.json` so Radix `.d.ts` files compile. Everything else is verified and correct.

---

### Author Response (Round 3)

**Date**: 2026-05-09

---

#### Finding R3-C1 (CRITICAL): Missing tsconfig `paths` for Radix `.d.ts` resolution

**Accepted.** This is a genuine gap — `skipLibCheck` suppresses type errors, not module resolution failures (TS2307). Radix's declaration files use `import * as React from "react"` which will fail module resolution after removing `react`.

**Fix added to Section 3 (`tsconfig.app.json`)**:

```jsonc
{
  "compilerOptions": {
    "jsx": "react-jsx",
    "jsxImportSource": "preact",
    "types": [],
    "baseUrl": ".",
    "paths": {
      "react": ["./node_modules/preact/compat/src/index.d.ts"],
      "react-dom": ["./node_modules/preact/compat/src/index.d.ts"]
    }
  }
}
```

Note: `baseUrl: "."` is required by TypeScript when using non-relative `paths` values. Since `tsconfig.app.json` is at repo root, `baseUrl: "."` resolves relative to repo root, making `./node_modules/preact/...` work correctly.

The `paths` approach is verified correct: when TypeScript encounters Radix's `import from "react"`, it resolves to `preact/compat/src/index.d.ts`, which re-exports the full Preact compat type surface. This also covers `import from "react-dom"` and `import from "react-dom/client"` through the same alias (since `preact/compat/src/index.d.ts` doesn't export `createRoot`, `createRoot` from `preact/compat/client` is imported explicitly in source files, not through the alias).

---

#### Finding R3-C2 (LOW): `StrictMode` documentation

**Accepted.** Added note to Section 4: Preact's `StrictMode` is `Fragment as StrictMode` (no-op). Can be removed for cleanliness. Both keeping and removing are valid — documented for implementer discretion.

---

#### Finding R3-C3 (LOW): Duplicate Round 2 content

**Accepted.** The duplicate was caused by the edit tool replacing a code block that contained the Round 2 header. Content is correct but duplicated. Will be cleaned up during consolidation phase (not now, per Linear Ledger Protocol — no editing prior sections).

---

### Review Round 2

**Reviewer**: Critical Architect Review  
**Date**: 2026-05-09

---

**Summary**: The Author Response addresses most findings from Round 1 correctly. However, I discovered that the proposal's foundational claim about `useSyncExternalStore` not being available in `preact/compat` is **incorrect** — it IS exported. This eliminates the need for the entire Section 5 rewrite and resolves the one-render-lag concern (Finding 8) natively. One correction to the `createRoot` import path is also needed.

---

#### New Finding N1: CRITICAL — `useSyncExternalStore` IS exported from `preact/compat` (2-arg only)

The proposal's Section 5 states: "`preact/compat` does not export `useSyncExternalStore`." This is wrong. Reading the actual `preact/compat` type declarations (`compat/src/index.d.ts`):

```ts
export function useSyncExternalStore<T>(
    subscribe: (flush: () => void) => () => void,
    getSnapshot: () => T
): T;
```

The 2-argument signature exists and is functionally identical to React's. The project's current usage passes 3 arguments (the third is a server snapshot `(): AppTheme => "light"` — used only for SSR hydration). Since this is a client-only SPA, the third argument can be safely dropped.

**This means:**
1. **Section 5 of the proposal (the entire `useState` + `useEffect` rewrite) is unnecessary.** No code rewrite needed for `App.tsx` line 574.
2. **Finding 8 (one-render-lag from the rewrite) is moot.** The `useSyncExternalStore` import preserves the synchronous snapshot behavior with zero behavioral change.
3. **The correct migration is a one-line change:**

```diff
// src/App.tsx line 1
- import { ..., useSyncExternalStore, ... } from "react";
+ import { ..., useSyncExternalStore, ... } from "preact/compat";

// src/App.tsx lines 574-578 — drop the 3rd server-snapshot arg
  const resolvedTheme: AppTheme = useSyncExternalStore(
    (listener) => subscribeToThemeChanges(listener),
    () => resolveDocumentTheme(),
-   (): AppTheme => "light",
  );
```

**Required fix**: Replace Section 5 entirely with the note that `useSyncExternalStore` is available from `preact/compat` with a 2-arg signature, and the only change needed is dropping the server-snapshot argument. Remove all references to the `useState` + `useEffect` rewrite pattern. This also fully resolves Finding 8 without needing lag documentation.

---

#### New Finding N2: MODERATE — `createRoot` is at `preact/compat/client`, NOT `preact/compat`

The Author Response for Finding 3 proposes:

```ts
import { createRoot, flushSync } from "preact/compat";
```

But `createRoot` is **NOT** exported from the `preact/compat` main entry. It is exported from the **`preact/compat/client`** subpath (confirmed from `compat/package.json` exports map and `compat/client.d.ts`):

```ts
// preact/compat/client.d.ts
export function createRoot(container: preact.ContainerNode): {
    render(children: preact.ComponentChild): void;
    unmount(): void;
};
```

`flushSync` IS at `preact/compat` (main entry) — that part is correct.

**Correct test import migration:**

```diff
- import React from "react";
- import { createRoot } from "react-dom/client";
- import { flushSync } from "react-dom";
+ import type { ReactElement } from "preact/compat";
+ import { createRoot } from "preact/compat/client";
+ import { flushSync } from "preact/compat";
```

**Note**: The `@preact/preset-vite` alias `react-dom` → `preact/compat` means `react-dom/client` resolves to `preact/compat/client` at bundle-time via subpath resolution. But for the explicit import approach used in the Author Response, the import path must be correct. `ReactElement` IS confirmed exported from the main `preact/compat` entry (as `preact.VNode`).

**Required fix**: Update Section 6 of the proposal to show `createRoot` imported from `preact/compat/client`, not `preact/compat`.

---

#### New Finding N3: NOTE — `UIEvent<T>` IS exported from `preact/compat`

Both the proposal (Section 4) and Review Round 1 (Finding 2) claim that `UIEvent<HTMLDivElement>` "does not exist in `preact/compat`." This is incorrect. `preact/compat` exports:

```ts
export import UIEvent = JSXInternal.TargetedUIEvent;
```

So `import { type UIEvent } from "preact/compat"` provides a generic `UIEvent<Target extends EventTarget>` type where `currentTarget` is typed as `Target`. The current code `function handleScroll(e: UIEvent<HTMLDivElement>) { e.currentTarget.scrollTop }` would work by simply changing the import path — no annotation change needed.

The Author Response's structural type approach (`{ currentTarget: HTMLDivElement }`) is also valid and has the advantage of minimal surface area. This is a precision note, not a blocker — either approach works. The Author Response should at minimum acknowledge that `UIEvent<T>` IS available if the import path change approach is preferred.

---

#### Verification (Positive)

- **`skipLibCheck: true`** (confirmed in `tsconfig.app.json` line 7). This means TypeScript will not type-check Radix UI's `.d.ts` files. Without `skipLibCheck`, the removal of `@types/react` would cause module resolution failures when Radix `.d.ts` files `import * as React from "react"`. With `skipLibCheck`, TypeScript skips checking those files entirely — no tsconfig `paths` aliases needed for `react` → `preact/compat` at the tsc level. Result: safe.
- **`ReactNode` availability**: Confirmed that `preact/compat` exports `ReactNode` as `preact.ComponentChild` — `import type { ReactNode } from "preact/compat"` is correct.
- **`ReactElement` availability**: Confirmed that `preact/compat` exports `ReactElement` as `preact.VNode` — `import type { ReactElement } from "preact/compat"` for the test file is correct.
- **`flushSync` signature**: Confirmed `flushSync<R>(fn: () => R): R` — compatible with current test usage.
- **`createRoot` return type**: Returns `{ render(children: preact.ComponentChild): void; unmount(): void }` — compatible with test's `root.render(...)` call.
- **`jsxImportSource: "preact"` with `jsx: "react-jsx"`**: TypeScript resolves JSX runtime from `preact/jsx-runtime`. Preact compat's JSX types augment the same namespace. No type conflict.
- **`useSyncExternalStore` return type**: Preact compat infers `T` from `getSnapshot()` return type — identical to React's behavior.

---

#### Re-evaluation of Round 1 Findings

| Finding | Original Status | After Author Response | After New Discoveries |
|---------|----------------|----------------------|----------------------|
| F1: `jsxImportSource` | CRITICAL | **Resolved** — `"jsxImportSource": "preact"` added. Verified safe with `skipLibCheck: true`. | ✅ Resolved |
| F2: `UIEvent<HTMLDivElement>` | CRITICAL | **Resolved** — structural type fix accepted. | ⚠️ Fix is valid, but finding's premise was partially wrong (`UIEvent` IS available via compat). The Author Response's structural type fix works either way. |
| F3: `flushSync` exists | HIGH | **Partially resolved** — `flushSync` import corrected, but `createRoot` import path is wrong (N2). | ⚠️ Needs `createRoot` path fix (N2) |
| F4: `Popover.Portal` edge cases | HIGH | **Resolved** — manual test plan accepted. | ✅ Resolved |
| F5: `asChild` ref merging | HIGH | **Resolved** — audit confirms no conflict, documented as limitation. | ✅ Resolved |
| F6: `React.ReactElement` type | MODERATE | **Resolved** — `import type { ReactElement }` from `preact/compat` added. | ✅ Resolved |
| F7: MPA mode verification | MODERATE | **Resolved** — deferred to tasks file verification. | ✅ Resolved |
| F8: `useSyncExternalStore` lag | MODERATE | **Resolved** (Author) by documenting lag. | ❌ **Moot** — entire rewrite is unnecessary; `useSyncExternalStore` is available (N1) |
| F9: `tsconfig.test.json` | LOW | **Resolved** — verification added. | ✅ Resolved |
| F10: SVG casing | LOW | **Resolved** — convention documented. | ✅ Resolved |

---

**STATUS: CHANGES_REQUESTED**

Two issues require revision before the proposal can move to APPROVED:

1. **N1 (CRITICAL)**: Section 5 must be replaced — `useSyncExternalStore` is available from `preact/compat` (2-arg). Drop the third arg, no rewrite needed. This also eliminates the Finding 8 concern natively.

2. **N2 (MODERATE)**: Section 6's `createRoot` import must be from `preact/compat/client`, not `preact/compat`.

N3 is a precision note for the record — no action required, but the Author Response may optionally note the simpler path.

---

### Review Round 3

**Reviewer**: Critical Architect Review  
**Date**: 2026-05-09

---

#### N1 & N2 Resolution Verification

**N1 (`useSyncExternalStore`):** ✅ RESOLVED. Author Response correctly identifies that `useSyncExternalStore(subscribe, getSnapshot)` is exported from `preact/compat` (source-confirmed at `compat/src/index.d.ts:39-42`). The Consolidated Final Import Map correctly lists it from `"preact/compat"`. The `useState` + `useEffect` rewrite is properly struck. Finding 8 (one-render-lag) is correctly rendered moot.

**N2 (`createRoot` at `preact/compat/client`):** ✅ RESOLVED. Author Response correctly identifies the import path as `preact/compat/client` (source-confirmed at `compat/client.d.ts:5`). The Consolidated Final Import Map correctly shows `createRoot` from `"preact/compat/client"` for both `main.tsx` and `settings-panel.test.tsx`.

#### Consolidated Final Import Map — Full Verification

Every import in the map was verified against the actual `preact/compat` type declarations (`compat/src/index.d.ts` and `compat/client.d.ts`):

| Import | Verified At | Status |
|--------|------------|--------|
| `StrictMode` from `preact/compat` | `compat/src/index.d.ts:72` — `export import StrictMode = preact.Fragment` | ✅ Available |
| `createRoot` from `preact/compat/client` | `compat/client.d.ts:5` — `export function createRoot(...)` | ✅ Available |
| `useSyncExternalStore` from `preact/compat` | `compat/src/index.d.ts:39-42` — 2-arg signature | ✅ Available |
| `type ReactNode` from `preact/compat` | `compat/src/index.d.ts:60` — `export import ReactNode = preact.ComponentChild` | ✅ Available |
| `type ReactElement` from `preact/compat` | `compat/src/index.d.ts:61` — `export import ReactElement = preact.VNode` | ✅ Available |
| `type UIEvent` from `preact/compat` | `compat/src/index.d.ts:167` — `export import UIEvent = JSXInternal.TargetedUIEvent` | ✅ Available |
| `flushSync` from `preact/compat` | `compat/src/index.d.ts:316-317` — both `fn(): R` and `fn(a: A, a: A): R` overloads | ✅ Available |
| `memo, useCallback, useEffect, useMemo, useRef, useState` | `compat/src/index.d.ts:24-34` — all standard hooks present | ✅ Available |
| `createContext, useContext` | `compat/src/index.d.ts:25,45` — context API present | ✅ Available |

**`SettingsPanel.tsx`**: ✅ Confirmed — zero React imports (only Radix, local, and i18n imports). Requires no changes. The `jsxImportSource: "preact"` in tsconfig provides JSX types for the JSX in this file.

**`jsxImportSource: "preact"`**: ✅ Verified. Preact's `package.json` exports map has `"./jsx-runtime"` → types at `"./jsx-runtime/src/index.d.ts"`. With `moduleResolution: "Bundler"`, TypeScript resolves correctly.

**`preact/compat` subpath resolution**: ✅ Verified. Preact's `package.json` exports map has `"./compat"` → types at `"./compat/src/index.d.ts"` and `"./compat/client"` → types at `"./compat/client.d.ts"`. All `import ... from "preact/compat"` and `import ... from "preact/compat/client"` resolve correctly.

#### All Round 1 Findings — Final Status

| # | Original Status | Final Status | Notes |
|---|----------------|-------------|-------|
| F1: `jsxImportSource` | CRITICAL | ✅ RESOLVED | `"jsxImportSource": "preact"` accepted, verified working with preact's exports map |
| F2: `UIEvent<HTMLDivElement>` | CRITICAL | ✅ RESOLVED | `UIEvent<T>` IS exported from `preact/compat` (line 167). Import path change suffices — no annotation change needed. |
| F3: `flushSync` exists | HIGH | ✅ RESOLVED | `flushSync` from `preact/compat`, `createRoot` from `preact/compat/client`. Both verified. |
| F4: `Popover.Portal` edge cases | HIGH | ✅ RESOLVED | Manual test plan (6 checks) accepted and documented. |
| F5: `asChild` ref merging | HIGH | ✅ RESOLVED | Audit confirms no conflict; limitation documented. |
| F6: `React.ReactElement` type | MODERATE | ✅ RESOLVED | `import type { ReactElement }` from `preact/compat` verified (line 61). |
| F7: MPA mode verification | MODERATE | ✅ RESOLVED | Deferred to tasks file verification. |
| F8: `useSyncExternalStore` lag | MODERATE | ✅ MOOT | No rewrite needed; `useSyncExternalStore` is natively available from `preact/compat`. |
| F9: `tsconfig.test.json` | LOW | ✅ RESOLVED | Verification step documented. |
| F10: SVG casing | LOW | ✅ RESOLVED | Convention documented. |

All findings are resolved or rendered moot by subsequent discoveries.

---

#### Finding R3-C1: CRITICAL — Missing tsconfig `paths` alias for `react` → `preact/compat`

**Description**: The proposal removes `react` and `@types/react` from dependencies and sets `"types": []` in tsconfig. The Review Round 2 verification (lines 852–859) claims that `skipLibCheck: true` handles Radix `.d.ts` file resolution. This claim is **incorrect**.

**Evidence**: All four Radix packages used by the project import from `react` in their type declarations:

| Package | `.d.ts` File | Import |
|---------|-------------|--------|
| `@radix-ui/react-accordion` | `dist/index.d.ts:2` | `import React from 'react';` |
| `@radix-ui/react-popover` | `dist/index.d.ts:2` | `import * as React from 'react';` |
| `@radix-ui/react-switch` | `dist/index.d.ts:2` | `import * as React from 'react';` |
| `@radix-ui/react-tabs` | `dist/index.d.ts:2` | `import * as React from 'react';` |

These types are used to define Radix component signatures (`React.ComponentPropsWithoutRef`, `React.RefAttributes`, etc.) that our code's type-checking depends on.

**Why `skipLibCheck` does not help**: `skipLibCheck` suppresses **type errors** within `.d.ts` files (e.g., mismatched parameter types, missing properties). It does **NOT** suppress **module resolution errors** (TS2307: "Cannot find module 'react' or its corresponding type declarations"). Module resolution and type checking are distinct compilation phases. When TypeScript processes our code's `import * as Accordion from "@radix-ui/react-accordion"`, it must resolve the full type dependency chain, including `import React from 'react'` in Radix's `.d.ts` files.

With `react` removed from `node_modules` and no `paths` alias, `tsc -b` will fail with TS2307 for each Radix `.d.ts` file.

**Required fix**: Add `"react"` and `"react-dom"` to the existing `paths` in `tsconfig.app.json`. The existing `vexflow` path must be preserved:

```diff
  // tsconfig.app.json
  "compilerOptions": {
    // ...
    "jsxImportSource": "preact",
    "types": [],
    "paths": {
+     "react": ["node_modules/preact/compat/src/index.d.ts"],
+     "react-dom": ["node_modules/preact/compat/src/index.d.ts"],
      "vexflow": ["node_modules/vexflow/build/types/entry/vexflow.d.ts"]
    }
  }
```

Note: `preact/compat` uses `export = React; export as namespace React;` with a `declare namespace React { ... }` pattern, which correctly supports `import * as React from '...'` (what Radix uses) and `import React from '...'` (what some Radix packages use). Pointing both `react` and `react-dom` to the same `compat/src/index.d.ts` is safe because:
- Radix `.d.ts` files only import from `react` (confirmed: zero Radix `.d.ts` files import from `react-dom`)
- `react-dom` path alias covers future-proofing and matches Vite's `@preact/preset-vite` alias behavior
- The `jsx-runtime` path (`jsxImportSource: "preact"`) is unaffected — TypeScript resolves `preact/jsx-runtime` via preact's `exports` map, not via `paths`

This also propagates to `tsconfig.test.json` (which extends `tsconfig.app.json`), ensuring `tsc -p tsconfig.test.json --noEmit` passes.

---

#### Finding R3-M1: Minor — StrictMode import inconsistency

**Description**: The proposal body Section 7 (lines 124–138) states: "Recommended: remove `StrictMode` since Preact's `StrictMode` does nothing." However, the Consolidated Final Import Map (line 744) still shows `import { StrictMode } from "preact/compat"` for `main.tsx`.

`preact/compat` exports `StrictMode = preact.Fragment` (verified at `compat/src/index.d.ts:72`), so the type is available. But the import map should reflect the recommendation — either:
- Match the recommendation: remove `StrictMode` from the import map and remove the `<React.StrictMode>` wrapper from `main.tsx`, **or**
- Change the recommendation: keep `StrictMode` as a harmless no-op for code-compatibility

**Recommendation**: Keep the `StrictMode` import (simpler diff, less churn). The import map is technically correct; the proposal text is slightly misleading. Update Section 7 to remove the "Recommended: remove" language or align the import map to remove it.

**Severity**: LOW — `StrictMode` is a Fragment no-op in Preact; keeping or removing it has zero functional impact.

---

#### Finding R3-M2: Minor — Duplicated Review Round 2 text

The file contains two `### Review Round 2` sections:
- Lines 569–686: condensed version with N1, N2, N3 findings and re-evaluation summary table
- Lines 753–887: expanded version with additional Verification section and detailed re-evaluation table

This appears to be a copy-paste artifact during the Linear Ledger append process. Both contain the same findings (N1, N2, N3) and the same `STATUS: CHANGES_REQUESTED` conclusion. The expanded version at lines 753–887 is the more complete one and should be retained. The condensed version (569–686) is redundant.

**Severity**: LOW — No content disagreement between the two sections. Clarify which one is canonical during consolidation.

---

#### Positive Confirmations (All Verified)

- `preact/compat` type declarations are comprehensive: all exported hooks, types, and utilities used by the project are present and correctly typed
- `jsxImportSource: "preact"` + `moduleResolution: "Bundler"` correctly resolves JSX types from `preact/jsx-runtime` via preact's `exports` map
- `preact/compat` and `preact/compat/client` subpath exports are correctly configured in preact's `package.json` — no additional `paths` needed for direct imports
- `useSyncExternalStore` in preact/compat has identical `subscribe` + `getSnapshot` signature — drop of 3rd server-snapshot arg has zero behavioral impact
- `flushSync` overloads in preact/compat match the test usage patterns exactly
- `SettingsPanel.tsx` has zero React imports — migration requires zero changes to that file
- All 6 `.tsx` files plus 1 `.ts` file are accounted for in the import map; no orphaned React imports remain
- `@radix-ui/react-slider` is confirmed unused (grep yields no imports in `src/`) — safe to remove

---

### Summary

| # | Severity | Issue |
|---|----------|-------|
| R3-C1 | CRITICAL | Missing tsconfig `paths` alias — Radix `.d.ts` files import from `react`; `skipLibCheck` does not suppress module resolution errors. tsc will fail after removing `react`/`@types/react`. Fix: add `"react"` and `"react-dom"` paths aliases pointing to `preact/compat/src/index.d.ts`. |
| R3-M1 | LOW | StrictMode import inconsistency — proposal recommends removing but import map includes it. |
| R3-M2 | LOW | Duplicated Review Round 2 sections — file has two `### Review Round 2` blocks. |

---

**STATUS: CHANGES_REQUESTED**

One critical gap (R3-C1) must be addressed before approval: the tsconfig must include `paths` aliases mapping `react` and `react-dom` to `preact/compat` so that TypeScript can resolve Radix's `.d.ts` module imports. Without this, `tsc -b` and `tsc -p tsconfig.test.json --noEmit` will fail, and the migration cannot be verified to pass type-checking.

The two minor findings (R3-M1, R3-M2) are non-blocking and can be resolved during the Author Response or deferred to consolidation.

Once R3-C1 is addressed, the proposal is ready for APPROVED. The architecture is sound, all imports are verified correct against the actual `preact/compat` type declarations, and all 10 Round 1 findings are properly resolved.

---

### Review Round 4

**Reviewer**: Critical Architect Review  
**Date**: 2026-05-09

---

#### R3-C1 Verification (CRITICAL)

The Author Response (Round 3, lines 898–921) correctly resolves R3-C1. Summary of verification:

1. **`baseUrl: "."` already exists** in `tsconfig.app.json` (line 22) — no conflict, no duplicate. The Author Response correctly notes it's required for non-relative `paths`.

2. **Existing `paths` entry for `vexflow` is preserved.** The Author Response provides a conceptual snippet showing the additions; the merged result adds `"react"` and `"react-dom"` entries alongside the existing `"vexflow"` entry.

3. **Target file is correct.** `preact/compat/src/index.d.ts` uses `export = React; export as namespace React;` — compatible with both `import * as React from "react"` (Radix accordion/popover/switch) and `import React from "react"` (Radix tabs).

4. **`jsx-runtime` is unaffected.** `jsxImportSource: "preact"` resolves via preact's `exports` map (`./jsx-runtime`), not via `paths`. No conflict.

5. **`tsconfig.test.json` propagation.** Extends `tsconfig.app.json`, inherits the `paths` aliases. Test files that import Radix components will resolve correctly.

6. **Path format consistency.** Author Response uses `./node_modules/...`; existing `vexflow` path uses `node_modules/...`. Functionally identical with `baseUrl: "."`. Minor style note for the implementer.

**Verdict**: ☑ RESOLVED.

---

#### All Findings — Final Status

| Round | # | Severity | Status |
|-------|---|----------|--------|
| R1 | F1 | CRITICAL | ☑ RESOLVED — `jsxImportSource: "preact"` added |
| R1 | F2 | CRITICAL | ☑ RESOLVED — `UIEvent<T>` available from `preact/compat` |
| R1 | F3 | HIGH | ☑ RESOLVED — `flushSync` from `preact/compat`; `createRoot` path fixed |
| R1 | F4 | HIGH | ☑ RESOLVED — `Popover.Portal` test plan accepted |
| R1 | F5 | HIGH | ☑ RESOLVED — `asChild` ref limitation documented |
| R1 | F6 | MODERATE | ☑ RESOLVED — `ReactElement` from `preact/compat` |
| R1 | F7 | MODERATE | ☑ RESOLVED — MPA verification in tasks |
| R1 | F8 | MODERATE | ☑ MOOT — no rewrite needed; native `useSyncExternalStore` |
| R1 | F9 | LOW | ☑ RESOLVED — `tsconfig.test.json` verification |
| R1 | F10 | LOW | ☑ RESOLVED — SVG casing convention noted |
| R2 | N1 | CRITICAL | ☑ RESOLVED — `useSyncExternalStore` native in Preact compat |
| R2 | N2 | MODERATE | ☑ RESOLVED — `createRoot` at `preact/compat/client` |
| R2 | N3 | NOTE | ☑ ACKNOWLEDGED — `UIEvent<T>` available |
| R3 | R3-C1 | CRITICAL | ☑ RESOLVED — tsconfig `paths` aliases for `react`/`react-dom` |
| R3 | R3-C2/M1 | LOW | ☑ RESOLVED — StrictMode documented as no-op; keep or remove both valid |
| R3 | R3-C3/M2 | LOW | ☑ ACKNOWLEDGED — duplicate Round 2 content is edit artifact; cleanup deferred to consolidation |

**0 remaining findings. 0 unresolved blocking issues.**

---

#### Consolidated Import Map — Final Verification

| Source File | Import | Status |
|---|---|---|
| `main.tsx` | `StrictMode` from `preact/compat`, `createRoot` from `preact/compat/client` | ☑ |
| `App.tsx` | `memo, useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore, ReactNode, UIEvent` from `preact/compat` | ☑ |
| `i18n/context.tsx` | `createContext, useCallback, useContext, useMemo, useState, ReactNode` from `preact/compat` | ☑ |
| `useAppSettings.ts` | `useEffect, useState` from `preact/compat` | ☑ |
| `NumericSettingControl.tsx` | `useRef, useState, useEffect` from `preact/compat` | ☑ |
| `settings-panel.test.tsx` | `flushSync, ReactElement` from `preact/compat`, `createRoot` from `preact/compat/client` | ☑ |
| `SettingsPanel.tsx` | (zero imports) | ☑ |

All imports verified against `preact/compat` source (`compat/src/index.d.ts`, `compat/client.d.ts`). No orphaned or incorrect paths.

---

**STATUS: APPROVED**

The proposal is technically sound. All 16 findings across 3 review rounds are resolved or moot. The Consolidated Import Map is correct. The tsconfig `paths` fix closes the final critical gap. Ready for execution planning (tasks file) and user stamp.
