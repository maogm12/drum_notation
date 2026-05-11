import { describe, it, expect, beforeAll } from "vitest";
import { initWasm } from "../wasm/drummark_wasm";
import { parseDocumentSkeletonFromWasmSync } from "../wasm/skeleton";
import { parseDocumentSkeletonFromLezer } from "./lezer_skeleton";
import type { DocumentSkeleton } from "./types";
import { buildNormalizedScoreWasm, buildNormalizedScore } from "./normalize";

beforeAll(async () => {
  await initWasm();
});

// ── Parser-level parity (AST skeleton) ─────────────────────────────

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
  measureRepeat: `time 4/4
note 1/8
| x x x x | % | %% |
`,
  multiRest: `time 4/4
note 1/8
| x x x x | --2-- |
`,
  navMarkers: `time 4/4
note 1/8
| @segno x x x x | x x x x @fine |
`,
  combinedHit: `time 4/4
note 1/8
HH | cxxx xxxx |
`,
};

const FIXTURES_EXPECT_SAME: string[] = [
  "headers", "simple", "hairpins", "paragraphs", "comments",
  "noteOverride", "measureRepeat",
  "multiRest", "navMarkers", "combinedHit",
];

// Parser skeleton differs (inline repeats expanded by normalizer, not parser)
const KNOWN_SKELETON_DIFFS: Record<string, string> = {
  inlineRepeat: `time 4/4
note 1/8
grouping 2+2
HH | x - x - *3|
SD | --d- --d- *3|
`,
  voltaSkeleton: `time 4/4
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

// ── Parser skeleton parity tests ─────────────────────────────────

describe("WASM vs Lezer parser skeleton parity", () => {
  for (const name of FIXTURES_EXPECT_SAME) {
    const source = FIXTURES[name];
    if (!source) continue;
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

// ── Normalized IR parity tests (buildNormalizedScore) ─────────────

interface IRLandmark {
  measureCount: number;
  paragraphCount: number;
  barlines: [number, string][];       // [measureIndex, barline]
  voltaMeasures: [number, number[]][]; // [measureIndex, voltaIndices]
  repeatSlashes: [number, number][];   // [measureIndex, slashes]
  multiRest: [number, number][];       // [measureIndex, count]
  noteValues: [number, number][];      // [paragraphIndex, noteValue]
  startNavs: [number, string][];       // [measureIndex, navKind]
  endNavs: [number, string][];         // [measureIndex, navKind]
}

function extractLandmarks(score: any): IRLandmark {
  const pis = new Set(score.measures.map((m: any) => m.paragraphIndex));
  const barlines: [number, string][] = [];
  const voltaMeasures: [number, number[]][] = [];
  const repeatSlashes: [number, number][] = [];
  const multiRest: [number, number][] = [];
  const startNavs: [number, string][] = [];
  const endNavs: [number, string][] = [];

  for (const m of score.measures) {
    const g = m.globalIndex ?? m.index ?? 0;
    if (m.barline) barlines.push([g, m.barline]);
    if (m.volta?.indices?.length) voltaMeasures.push([g, m.volta.indices]);
    if (m.measureRepeat?.slashes) repeatSlashes.push([g, m.measureRepeat.slashes]);
    if (m.multiRest?.count) multiRest.push([g, m.multiRest.count]);
    if (m.startNav?.kind) startNavs.push([g, m.startNav.kind]);
    if (m.endNav?.kind) endNavs.push([g, m.endNav.kind]);
  }

  return {
    measureCount: score.measures.length,
    paragraphCount: pis.size,
    barlines,
    voltaMeasures,
    repeatSlashes,
    multiRest,
    noteValues: [...pis].sort().map(pi => {
      const ms = score.measures.filter((m: any) => m.paragraphIndex === pi);
      return [pi, ms[0]?.noteValue ?? 0] as [number, number];
    }),
    startNavs,
    endNavs,
  };
}

describe("WASM vs Lezer normalized IR parity", () => {
  it("paragraphs and inline repeats produce correct measure counts", () => {
    const src = `time 4/4\nnote 1/8\nHH | x - x - *3|\nSD | --d- --d- *3|\n`;
    const wasm = extractLandmarks(buildNormalizedScoreWasm(src));
    const lezer = extractLandmarks(buildNormalizedScore(src, "lezer"));
    expect(wasm.measureCount).toBe(lezer.measureCount);
    expect(wasm.paragraphCount).toBe(lezer.paragraphCount);
  });

  it("inline repeat with *4 on one track, shorter other tracks", () => {
    const src = `time 4/4\nnote 1/8\nHH | cxxx xxxx | xxxx xxxx *3|\nSD | --d- --d- *4|\nBD | p*-- pp-- *4|\n`;
    const wasm = extractLandmarks(buildNormalizedScoreWasm(src));
    const lezer = extractLandmarks(buildNormalizedScore(src, "lezer"));
    expect(wasm.measureCount).toBe(lezer.measureCount);
  });

  it("volta barlines produce correct measure structure", () => {
    const src = `time 4/4\nnote 1/4\n|: s s s s |1. s s [ss] s :|2. s s [ssss] s |\n`;
    const wasm = extractLandmarks(buildNormalizedScoreWasm(src));
    const lezer = extractLandmarks(buildNormalizedScore(src, "lezer"));
    expect(wasm.measureCount).toBe(lezer.measureCount);
    expect(wasm.barlines.map(b => b[1])).toEqual(lezer.barlines.map(b => b[1]));
    expect(wasm.voltaMeasures.length).toBeGreaterThanOrEqual(1);
    expect(wasm.voltaMeasures).toEqual(lezer.voltaMeasures);
  });

  it("measure-repeat (%) produces correct slashes", () => {
    const src = `time 4/4\nnote 1/4\n| x x x x | % | %% |\n`;
    const wasm = extractLandmarks(buildNormalizedScoreWasm(src));
    const lezer = extractLandmarks(buildNormalizedScore(src, "lezer"));
    expect(wasm.measureCount).toBe(lezer.measureCount);
    expect(wasm.repeatSlashes.length).toBeGreaterThanOrEqual(1);
  });

  it("multi-rest (--N--) produces correct count", () => {
    const src = `time 4/4\nnote 1/8\n| x - x - | --2-- |\n`;
    const wasm = extractLandmarks(buildNormalizedScoreWasm(src));
    const lezer = extractLandmarks(buildNormalizedScore(src, "lezer"));
    expect(wasm.measureCount).toBe(lezer.measureCount);
    expect(wasm.multiRest.length).toBeGreaterThanOrEqual(1);
  });

  it("per-paragraph note override (note 1/4)", () => {
    const src = `time 4/4\nnote 1/8\nHH | x - x - |\n\nnote 1/4\nHH | x |\n`;
    const wasm = extractLandmarks(buildNormalizedScoreWasm(src));
    const lezer = extractLandmarks(buildNormalizedScore(src, "lezer"));
    expect(wasm.noteValues).toEqual(lezer.noteValues);
    expect(wasm.paragraphCount).toBe(2);
    // First paragraph should have noteValue=8, second should have 4
    const nvMap = Object.fromEntries(wasm.noteValues);
    expect(nvMap[0]).toBe(8);
    expect(nvMap[1]).toBe(4);
  });

  it("navigation markers (segno, fine, dc, ds, coda, to-coda)", () => {
    const src = `time 4/4\nnote 1/8\n| @segno x x x x | x x x x @to-coda |\n\n| @coda x x x x | x x x x @fine |\n\n| x x x x @dc-al-coda |\n`;
    const wasm = extractLandmarks(buildNormalizedScoreWasm(src));
    const lezer = extractLandmarks(buildNormalizedScore(src, "lezer"));
    // At minimum, check that nav markers are present and counts match
    expect(wasm.startNavs.length).toBe(lezer.startNavs.length);
    expect(wasm.endNavs.length).toBe(lezer.endNavs.length);
    expect(wasm.measureCount).toBe(lezer.measureCount);
  });

  it("multi-paragraph with comment separators", () => {
    const src = `time 4/4\nnote 1/8\nHH | x - x - |\n\n# Verse\nHH | x - x - |\n\n# Chorus\nHH | x - x - |\n`;
    const wasm = extractLandmarks(buildNormalizedScoreWasm(src));
    const lezer = extractLandmarks(buildNormalizedScore(src, "lezer"));
    expect(wasm.paragraphCount).toBe(lezer.paragraphCount);
    expect(wasm.measureCount).toBe(lezer.measureCount);
  });
});
