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
  paragraphs: `time 4/4
note 1/8
grouping 2+2
HH | x - x - |
SD | --d- --d- |

HH | x - x - |
SD | --d- --d- |
`,
  comments: `time 4/4
note 1/8
grouping 2+2
HH | x - x - |

# a comment line
HH | x - x - |
`,
  noteOverride: `time 4/4
note 1/8
grouping 2+2
HH | x - x - |

note 1/4
HH | x |
`,
};

const KNOWN_DIFFERENCES: Record<string, string> = {
  trackAnonymous: `time 4/4
...
`,
  suffixChain: `time 4/4
...
`,
  inlineRepeat: `time 4/4
note 1/8
grouping 2+2
HH | x - x - *3|
SD | --d- --d- *3|
`,
  volta: `time 4/4
note 1/8
grouping 2+2
HH |: x - x - |1. x - :|2. --d- ||
`,
};

// ── Helpers ──────────────────────────────────────────────────────

function normalize(s: any): any {
  if (s === null || s === undefined) return undefined;
  if (Array.isArray(s)) return s.map(normalize);
  if (typeof s !== "object") return s;

  const out: Record<string, any> = {};
  for (const k of Object.keys(s)) {
    if (k === "line" || k === "lineNumber" || k === "startLine" ||
        k === "startOffset" || k === "globalIndex") continue;
    if (k === "raw" || k === "content" || k === "source") continue;
    if (k === "barline" && s[k] === "regular") continue;
    if (k === "voltaTerminator" && s[k] === false) continue;
    if (s[k] === null || s[k] === undefined) continue;
    if (k === "voltaIndices" || k === "measureRepeatSlashes" ||
        k === "multiRestCount" || k === "trackOverride") {
      if (s[k] === null) continue;
    }
    const v = normalize(s[k]);
    if (v !== undefined) out[k] = v;
  }
  return Object.keys(out).length > 0 ? out : undefined;
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

describe("WASM vs Lezer (structural differences — not bugs)", () => {
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
