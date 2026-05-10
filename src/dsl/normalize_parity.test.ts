import { describe, it, expect, beforeAll } from "vitest";
import { initWasm } from "../wasm/drummark_wasm";
import { buildNormalizedScoreWasm } from "./normalize";
import { buildNormalizedScore } from "./normalize";
import type { NormalizedScore, NormalizedMeasure, NormalizedEvent } from "./types";

beforeAll(async () => {
  await initWasm();
});

// ── Test inputs ──────────────────────────────────────────────────

const FIXTURES: Record<string, string> = {
  simple: `time 4/4
note 1/8
grouping 2+2
HH | x - x - |
`,
  multiTrack: `time 4/4
note 1/8
grouping 2+2
HH | x - x - |
SD | --d- --d- |
`,
  groups: `time 4/4
note 1/8
grouping 2+2
SD | [2: d d d] |
`,
  combinedHit: `time 4/4
note 1/8
grouping 2+2
SD | x+d+b |
`,
  hairpins: `time 4/4
note 1/8
grouping 2+2
HH | x < d > ! |
`,
  navigation: `time 4/4
note 1/8
grouping 2+2
HH | @segno x |
HH | @dc |
`,
  repeats: `time 4/4
note 1/8
grouping 2+2
HH |: x - x - :|
`,
  multiRest: `time 4/4
note 1/8
grouping 2+2
HH | x | --2-- |
`,
};

// ── Helpers ──────────────────────────────────────────────────────

function normalizeScore(s: NormalizedScore): any {
  return JSON.parse(JSON.stringify(s, (_, v) =>
    v === undefined ? null : v
  ));
}

function stripPosition(s: any): any {
  if (s === null || s === undefined) return undefined;
  if (Array.isArray(s)) return s.map(stripPosition);
  if (typeof s !== "object") return s;
  const out: Record<string, any> = {};
  for (const k of Object.keys(s)) {
    if (k === "line" || k === "sourceLine" || k === "globalIndex" ||
        k === "paragraphIndex" || k === "measureIndex" ||
        k === "measureInParagraph" || k === "startLine" ||
        k === "lineNumber") continue;
    out[k] = stripPosition(s[k]);
  }
  return out;
}

// ── Tests ────────────────────────────────────────────────────────

describe("WASM vs TS normalizer parity", () => {
  for (const [name, source] of Object.entries(FIXTURES)) {
    it(name, () => {
      const wasm = buildNormalizedScoreWasm(source);
      const ts = buildNormalizedScore(source, "lezer");

      const w = stripPosition(normalizeScore(wasm));
      const t = stripPosition(normalizeScore(ts));

      expect(w.measures.length).toBe(t.measures.length);

      for (let mi = 0; mi < w.measures.length; mi++) {
        const wm = w.measures[mi];
        const tm = t.measures[mi];
        expect(wm.barline).toBe(tm.barline);

        const minLen = Math.min(wm.events.length, tm.events.length);
        for (let ei = 0; ei < minLen; ei++) {
          const we = wm.events[ei];
          const te = tm.events[ei];
          expect(we.track).toBe(te.track);
          expect(we.glyph).toBe(te.glyph);
          expect(we.kind).toBe(te.kind);
          expect(we.voice).toBe(te.voice);
        }
      }
    });
  }
});
