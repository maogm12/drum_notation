import { describe, it, expect, beforeAll } from "vitest";
import { initWasm } from "../wasm/drummark_wasm";
import { parseDocumentSkeletonFromWasmSync } from "../wasm/skeleton";
import { parseDocumentSkeletonFromLezer } from "./lezer_skeleton";
import type { DocumentSkeleton } from "./types";

beforeAll(async () => {
  await initWasm();
});

// ── Test inputs ──────────────────────────────────────────────────

const FIXTURES: Record<string, string> = {
  headers: `title My Score
subtitle Verse
tempo 120
time 4/4
`,
  simple: `time 4/4
note 1/8
grouping 2+2
HH | x - x - |
`,
  hairpins: `time 4/4
note 1/8
grouping 2+2
HH | x < d > ! |
`,
};

// Cases with known structural differences between WASM and Lezer
const KNOWN_DIFFERENCES: Record<string, string> = {
  trackAnonymous: `time 4/4
note 1/8
grouping 2+2
| x - x - |
| --d- |
`,
  combinedHit: `time 4/4
note 1/8
grouping 2+2
SD | x+d+b |
`,
  group: `time 4/4
note 1/8
grouping 2+2
SD | [x d b] |
`,
  suffixChain: `time 4/4
note 1/8
grouping 2+2
SD | x. / * :accent |
`,
  navigation: `time 4/4
note 1/8
grouping 2+2
HH | @segno x |
HH | @dc |
`,
  measureRepeat: `time 4/4
note 1/8
grouping 2+2
HH | x | % |
`,
  multiRest: `time 4/4
note 1/8
grouping 2+2
HH | x | --2-- |
`,
};

// ── Helpers ──────────────────────────────────────────────────────

/** Strip fields that differ between WASM and Lezer by design:
 *  - Source positions (WASM doesn't track line numbers yet)
 *  - Content/raw strings (different generation strategies)
 *  - null vs undefined (WASM always emits null for absent fields)
 *  - Extra WASM-only fields (voltaIndices, voltaTerminator, multiRestCount, measureRepeatSlashes)
 */
function normalize(s: any): any {
  if (s === null || s === undefined) return undefined;
  if (Array.isArray(s)) return s.map(normalize);
  if (typeof s !== "object") return s;

  const out: Record<string, any> = {};
  for (const k of Object.keys(s)) {
    // Skip source-position fields (WASM doesn't track line numbers)
    if (k === "line" || k === "lineNumber" || k === "startLine" ||
        k === "startOffset" || k === "globalIndex") continue;
    // Skip raw/source text (generated differently)
    if (k === "raw" || k === "content" || k === "source") continue;
    // Strip barline: "regular" (Lezer omits it, WASM always emits)
    if (k === "barline" && s[k] === "regular") continue;
    // Skip WASM-only false/null placeholder fields
    if (k === "voltaTerminator" && s[k] === false) continue;
    if ((k === "voltaIndices" || k === "measureRepeatSlashes" ||
         k === "multiRestCount" || k === "trackOverride") && s[k] === null) continue;

    out[k] = normalize(s[k]);
  }
  return out;
}

function normalizeSkeleton(s: DocumentSkeleton): unknown {
  return JSON.parse(JSON.stringify(s, (_, v) => (v === undefined ? null : v)));
}

// ── Tests ────────────────────────────────────────────────────────

describe("WASM vs Lezer parser parity", () => {
  for (const [name, source] of Object.entries(FIXTURES)) {
    it(name, () => {
      const wasm = parseDocumentSkeletonFromWasmSync(source);
      const lezer = parseDocumentSkeletonFromLezer(source);

      const w = normalize(normalizeSkeleton(wasm));
      const l = normalize(normalizeSkeleton(lezer));

      try {
        expect(w).toEqual(l);
      } catch (e) {
        console.error(`${name}:`);
        console.error("  W:", JSON.stringify(w));
        console.error("  L:", JSON.stringify(l));
        throw e;
      }
    });
  }
});

describe("WASM vs Lezer parser parity (known differences)", () => {
  for (const [name, source] of Object.entries(KNOWN_DIFFERENCES)) {
    it.skip(name, () => {
      const wasm = parseDocumentSkeletonFromWasmSync(source);
      const lezer = parseDocumentSkeletonFromLezer(source);
      const w = normalize(normalizeSkeleton(wasm));
      const l = normalize(normalizeSkeleton(lezer));
      expect(w).toEqual(l);
    });
  }
});
