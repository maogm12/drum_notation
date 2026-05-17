import { describe, it, expect } from "vitest";
import { buildNormalizedScore } from "../dsl/normalize";
import { buildLayoutSceneFromSource, renderSceneToSvg, renderScoreToSvg, setLayoutSource } from "./svgRenderer";

const HEADER = `time 4/4
note 1/8
grouping 2+2
`;

function render(dsl: string): string {
  setLayoutSource(dsl);
  const score = buildNormalizedScore(dsl);
  return renderScoreToSvg(score, { pageWidth: 612, showTitle: true });
}

function countRe(svg: string, re: RegExp): number {
  return (svg.match(re) || []).length;
}

function countRole(svg: string, role: string): number {
  return countRe(svg, new RegExp(`data-role="${role}"`, "g"));
}

function renderPrecomputedScene(items: Array<Record<string, unknown>>): string {
  return renderSceneToSvg({
    version: "1",
    metricsVersion: "test",
    pages: [{
      index: 0,
      widthPt: 120,
      heightPt: 80,
      systems: [{ id: "system-0", index: 0, pageIndex: 0, xPt: 0, yPt: 0, widthPt: 120, heightPt: 80, measureIds: ["measure-0"] }],
      measures: [{ id: "measure-0", globalIndex: 0, systemId: "system-0", xPt: 0, yPt: 0, widthPt: 120, heightPt: 80 }],
      items: items as any,
      composites: [],
    }],
  } as any, { staffScale: 1 });
}

describe("SVG Renderer parity", () => {
  it("renders 5 staff lines", () => {
    const svg = render(HEADER + "SD | d |\n");
    expect(countRe(svg, /<line /g)).toBeGreaterThanOrEqual(5);
  });

  it("renders percussion clef", () => {
    const svg = render(HEADER + "SD | d |\n");
    expect(svg).toContain("");
  });

  it("renders time signature", () => {
    const svg = render(HEADER + "SD | d |\n");
    expect(svg).toContain("\u{E084}"); // time sig "4"
  });

  it("renders barline", () => {
    const svg = render(HEADER + "SD | d |\n");
    expect(countRole(svg, "opening-barline")).toBe(1);
    expect(countRole(svg, "final-barline-thin") + countRole(svg, "barline") + countRole(svg, "closing-barline")).toBeGreaterThanOrEqual(1);
  });

  it("renders notehead", () => {
    const svg = render(HEADER + "SD | d |\n");
    expect(svg).toContain("");
  });

  it("renders stem", () => {
    const svg = render(HEADER + "SD | d |\n");
    expect(countRole(svg, "stem")).toBeGreaterThanOrEqual(1);
  });

  it("renders unbeamed flags as SMuFL glyphs", () => {
    const svg = renderPrecomputedScene([
      { id: "stem-0", measureId: "measure-0", role: "stem", kind: "lineSegment", zIndex: 0, primitive: { x1Pt: 18, y1Pt: 10, x2Pt: 18, y2Pt: 40, stroke: "#333", strokeWidth: 1.5 } },
      { id: "flag-0", measureId: "measure-0", anchorItemId: "stem-0", role: "flag", kind: "glyphRun", zIndex: 1, primitive: { xPt: 18, yPt: 10, glyphRole: "flag8thUp", glyphCount: 1, codepoint: 0xE240, fontFamily: "Bravura", fontSizePt: 16, fill: "#333" } },
    ]);
    expect(countRole(svg, "flag")).toBe(1);
    expect(svg).toContain("\u{E240}");
    expect(svg).not.toContain("<polyline");
  });

  it("renders eighth-note beams", () => {
    const svg = renderPrecomputedScene([
      { id: "beam-0", measureId: "measure-0", role: "beam", kind: "path", zIndex: 0, primitive: { d: "M 10 20 L 40 20 L 40 24 L 10 24 Z", fill: "#333" } },
    ]);
    expect(countRole(svg, "beam")).toBeGreaterThanOrEqual(1);
    expect(svg).toContain("<path");
  });

  it("renders sixteenth-note secondary beams", () => {
    const svg = renderPrecomputedScene([
      { id: "beam-0", measureId: "measure-0", role: "beam", kind: "path", zIndex: 0, primitive: { d: "M 10 20 L 40 20 L 40 24 L 10 24 Z", fill: "#333" } },
      { id: "beam-1", measureId: "measure-0", role: "beam-secondary", kind: "path", zIndex: 0, primitive: { d: "M 10 26 L 40 26 L 40 30 L 10 30 Z", fill: "#333" } },
    ]);
    expect(countRole(svg, "beam-secondary")).toBeGreaterThanOrEqual(1);
  });

  it("renders X notehead on cymbal", () => {
    const svg = render(HEADER + "HH | x |\n");
    expect(svg).toContain("");
  });

  it("renders accent modifier", () => {
    const svg = render(HEADER + "SD | d:accent |\n");
    expect(countRole(svg, "accent")).toBe(1);
    expect(svg).toContain("");
  });

  it("renders ghost modifier", () => {
    const svg = render(HEADER + "SD | d:ghost |\n");
    expect(svg).toContain("");
  });

  it("renders double barline", () => {
    const svg = render(HEADER + "SD | d ||\n");
    expect(countRole(svg, "double-barline-left")).toBe(1);
    expect(countRole(svg, "double-barline-right")).toBe(1);
  });

  it("renders repeat bars", () => {
    const svg = render(HEADER + "SD |: d :|\n");
    expect(countRole(svg, "repeat-start")).toBe(1);
    expect(countRole(svg, "repeat-end")).toBe(1);
  });

  it("renders repeat-span and volta composites", () => {
    const svg = render("time 4/4\nnote 1/4\ngrouping 1+1+1+1\n|: s s s s |1. s s [ss] s :|2. s s [ssss] s |\n");
    expect(countRole(svg, "repeat-span-line")).toBeGreaterThanOrEqual(1);
    expect(countRole(svg, "repeat-span-count")).toBeGreaterThanOrEqual(1);
    expect(countRole(svg, "volta-line")).toBeGreaterThanOrEqual(2);
    expect(countRole(svg, "volta-label")).toBeGreaterThanOrEqual(2);
    expect(svg).toContain("1.");
    expect(svg).toContain("2.");
  });

  it("renders title", () => {
    const svg = render("title Hello\n" + HEADER + "SD | d |\n");
    expect(svg).toContain("Hello");
  });

  it("renders measure repeat", () => {
    const svg = render(HEADER + "SD | d | % |\n");
    expect(countRole(svg, "measure-repeat")).toBe(1);
  });

  it("expands two-bar repeats into two display measures and uses the dedicated glyph", () => {
    const source = HEADER + "HH | x - - - | x x - - | %% |\n";
    const scene = buildLayoutSceneFromSource(source, { pageWidth: 612, staffScale: 1 });
    expect(scene.pages[0]?.measures).toHaveLength(4);
    const svg = renderSceneToSvg(scene, { staffScale: 1 });
    expect(countRole(svg, "measure-repeat")).toBe(1);
    expect(svg).toContain("\u{E501}");
  });

  it("renders multi-rest", () => {
    const svg = render(HEADER + "SD | --2-- |\n");
    expect(countRole(svg, "multi-rest-bar")).toBe(1);
    expect(countRole(svg, "multi-rest-count")).toBe(1);
  });

  it("renders navigation markers", () => {
    const svg = render(HEADER + "SD | @segno d - - - | d - - - @dc |\n");
    expect(countRole(svg, "nav-start")).toBe(1);
    expect(countRole(svg, "nav-end")).toBe(1);
    expect(svg).toContain("@segno");
    expect(svg).toContain("@dc");
  });

  it("renders hairpins", () => {
    const svg = render("time 4/4\nnote 1/8\ngrouping 2+2\nHH | < x x x x ! - - - |\n");
    expect(countRole(svg, "hairpin-top")).toBe(1);
    expect(countRole(svg, "hairpin-bottom")).toBe(1);
  });

  it("renders rests by duration", () => {
    const svg = render(HEADER + "SD | --2-- |\n");
    expect(countRole(svg, "rest")).toBe(0);
    expect(countRole(svg, "multi-rest-bar")).toBe(1);
  });

  it("renders implicit lower-voice rests for rhythmic gaps", () => {
    const svg = render("time 4/4\nnote 1/8\ngrouping 2+2\nHH | x x x x x x x x |\nBD | p - - - p - - - |\n");
    expect(countRole(svg, "rest")).toBe(4);
    expect(svg).toContain("\u{E4E5}");
    expect(svg).toContain("\u{E4E6}");
  });

  it("renders same-voice rests without inventing an upper voice", () => {
    const svg = render("time 4/4\ndivisions 4\ngrouping 2+2\nBD | b - - - |\n");
    expect(countRole(svg, "rest")).toBe(3);
    expect(svg).toContain("\u{E4E5}");
    expect(svg).toContain("\u{E4E6}");
    expect(svg).toContain("\u{E4E4}");
  });

  it("renders eighth-rest glyphs for simple eighth-note gaps and trailing silence", () => {
    const svg = render("time 4/4\nnote 1/8\ngrouping 2+2\nHH | x - x - |\n");
    expect(countRole(svg, "rest")).toBe(3);
    expect(svg).toContain("\u{E4E6}");
    expect(svg).toContain("\u{E4E4}");
  });
});
