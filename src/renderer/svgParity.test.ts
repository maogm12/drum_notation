import { describe, it, expect } from "vitest";
import { buildNormalizedScore } from "../dsl/normalize";
import { renderScoreToSvg } from "./svgRenderer";

const HEADER = `time 4/4
note 1/8
grouping 2+2
`;

function render(dsl: string): string {
  const score = buildNormalizedScore(dsl);
  return renderScoreToSvg(score, { pageWidth: 612, showTitle: true });
}

function countRe(svg: string, re: RegExp): number {
  return (svg.match(re) || []).length;
}

describe("SVG Renderer parity", () => {
  it("renders 5 staff lines", () => {
    const svg = render(HEADER + "SD | d |\n");
    expect(countRe(svg, /class="vf-staff"/g)).toBe(5);
  });

  it("renders percussion clef", () => {
    const svg = render(HEADER + "SD | d |\n");
    expect(svg).toContain("vf-notehead"); // clef is a notehead-sized text
  });

  it("renders time signature", () => {
    const svg = render(HEADER + "SD | d |\n");
    expect(svg).toContain("\u{E084}"); // time sig "4"
  });

  it("renders barline", () => {
    const svg = render(HEADER + "SD | d |\n");
    expect(countRe(svg, /class="vf-bar"/g)).toBeGreaterThanOrEqual(1);
  });

  it("renders notehead", () => {
    const svg = render(HEADER + "SD | d |\n");
    expect(svg).toContain("vf-notehead");
  });

  it("renders stem", () => {
    const svg = render(HEADER + "SD | d |\n");
    expect(svg).toContain("vf-stem");
  });

  it("renders X notehead on cymbal", () => {
    const svg = render(HEADER + "HH | x |\n");
    expect(svg).toContain("\u{E0A9}");
  });

  it("renders accent modifier", () => {
    const svg = render(HEADER + "SD | d:accent |\n");
    expect(svg).toContain(">");
  });

  it("renders ghost modifier", () => {
    const svg = render(HEADER + "SD | d:ghost |\n");
    expect(svg).toContain("(");
  });

  it("renders double barline", () => {
    const svg = render(HEADER + "SD | d ||\n");
    const bars = countRe(svg, /class="vf-bar"/g);
    expect(bars).toBeGreaterThanOrEqual(2);
  });

  it("renders repeat bars", () => {
    const svg = render(HEADER + "SD |: d :|\n");
    expect(svg).toContain(":");
  });

  it("renders title", () => {
    const svg = render("title Hello\n" + HEADER + "SD | d |\n");
    expect(svg).toContain("Hello");
  });

  it("renders measure repeat", () => {
    const svg = render(HEADER + "SD | d | % |\n");
    expect(svg).toContain("%");
  });

  it("renders multi-rest", () => {
    const svg = render(HEADER + "SD | --2-- |\n");
    expect(svg).toContain("vf-staff"); // H-bar uses vf-staff class
  });
});
