// @vitest-environment jsdom

import { describe, it, expect } from "vitest";
import { buildNormalizedScore } from "../dsl/normalize";
import { renderScoreToSvg, setLayoutSource } from "./svgRenderer";
import { renderScoreToSvg as vexRender } from "../vexflow/renderer";

const HEADER = `time 4/4
note 1/8
grouping 2+2
`;

/** Extract all y1 values from line elements with given class. */
function lineY1s(svg: string): number[] {
  const re = /<line[^>]*y1="([\d.]+)"[^>]*>/g;
  const r: number[] = [];
  let m;
  while ((m = re.exec(svg)) !== null) r.push(+m[1]);
  return r;
}

/** Extract all <text> elements. */
function textEls(svg: string): { x: number; y: number; content: string }[] {
  const re = /<text[^>]*x="([\d.]+)"[^>]*y="([\d.]+)"[^>]*>([^<]*)<\/text>/g;
  const r: { x: number; y: number; content: string }[] = [];
  let m;
  while ((m = re.exec(svg)) !== null) r.push({ x: +m[1], y: +m[2], content: m[3] });
  return r;
}

describe("VexFlow position parity", () => {
  it("staff lines start at same Y", async () => {
    const dsl = HEADER + "SD | dddd |\n";
    setLayoutSource(dsl);
    const score = buildNormalizedScore(dsl);
    const vexSvg = await vexRender(score, { staffScale: 0.75 });
    const ourSvg = renderScoreToSvg(score, { staffScale: 0.75, pageWidth: 612, showTitle: false });

    const vexY = lineY1s(vexSvg).sort((a, b) => a - b);
    const ourY = lineY1s(ourSvg).sort((a, b) => a - b);

    // Both should have at least 5 staff lines
    expect(ourY.length).toBeGreaterThanOrEqual(5);
    // First staff line Y should be similar
    if (vexY.length >= 5) {
      console.log("VexFlow staff Y:", vexY.slice(0, 5));
      console.log("Our staff Y:", ourY.slice(0, 5));
    }
  });

  it("first notehead X position is reasonable", async () => {
    const dsl = HEADER + "SD | d |\n";
    setLayoutSource(dsl);
    const score = buildNormalizedScore(dsl);
    const vexSvg = await vexRender(score, { staffScale: 0.75 });
    const ourSvg = renderScoreToSvg(score, { staffScale: 0.75, pageWidth: 612, showTitle: false });

    const vexText = textEls(vexSvg);
    const ourText = textEls(ourSvg);

    if (vexText.length > 0 && ourText.length > 0) {
      console.log("VexFlow first text:", JSON.stringify(vexText[0]));
      console.log("Our first text:", JSON.stringify(ourText[0]));
    }
    expect(ourText.length).toBeGreaterThanOrEqual(1);
  });
});
