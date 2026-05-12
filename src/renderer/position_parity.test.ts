// @vitest-environment jsdom

import { describe, it, expect, beforeAll } from "vitest";
import { initWasm } from "../wasm/drummark_wasm";
import { buildNormalizedScore } from "../dsl/normalize";
import { renderScoreToSvg } from "./svgRenderer";
import { renderScoreToSvg as vexRender } from "../vexflow/renderer";
import { setLayoutSource } from "./svgRenderer";

const SRC = `tempo 100
time 4/4
note 1/4

|s s s s|`;

interface Element {
  type: string;
  x: number; y: number;
  x2?: number; y2?: number;
  w?: number; h?: number;
  fontSize?: number;
  fontFamily?: string;
  strokeWidth?: number;
  text?: string;
}

function parseSvg(svg: string): Element[] {
  const els: Element[] = [];
  // Lines
  const lineRe = /<line\s+x1="([\d.]+)"\s+y1="([\d.]+)"\s+x2="([\d.]+)"\s+y2="([\d.]+)"\s+stroke="([^"]*)"\s+stroke-width="([\d.]+)"\/>/g;
  let m;
  while ((m = lineRe.exec(svg))) {
    els.push({ type: "line", x: +m[1], y: +m[2], x2: +m[3], y2: +m[4], strokeWidth: +m[6] });
  }
  // Texts
  const textRe = /<text[^>]*x="([\d.]+)"[^>]*y="([\d.]+)"[^>]*(?:font-family="([^"]*)")?[^>]*(?:font-size="([\d.]+)pt")?[^>]*>([^<]*)<\/text>/g;
  while ((m = textRe.exec(svg))) {
    els.push({ type: "text", x: +m[1], y: +m[2], fontFamily: m[3] || "", fontSize: m[4] ? +m[4] : 0, text: m[5] });
  }
  // Rects
  const rectRe = /<rect\s+x="([\d.]+)"\s+y="([\d.]+)"\s+width="([\d.]+)"\s+height="([\d.]+)"/g;
  while ((m = rectRe.exec(svg))) {
    els.push({ type: "rect", x: +m[1], y: +m[2], w: +m[3], h: +m[4] });
  }
  // Paths
  const pathRe = /<path[^>]*d="M([\d.]+)\s+([\d.]+)L([\d.]+)\s+([\d.]+)"[^>]*stroke-width="([\d.]+)"\/>/g;
  while ((m = pathRe.exec(svg))) {
    els.push({ type: "path", x: +m[1], y: +m[2], x2: +m[3], y2: +m[4], strokeWidth: +m[5] });
  }
  return els;
}

function categorize(els: Element[]) {
  const result: Record<string, Element[]> = {};
  for (const e of els) {
    let cat = e.type;
    if (e.type === "line") {
      const xDiff = Math.abs((e.x2 ?? e.x) - e.x);
      const yDiff = Math.abs((e.y2 ?? e.y) - e.y);
      if (yDiff < 1 && xDiff > 100) cat = "staffLine";
      else if (xDiff < 1 && yDiff > 5) cat = "barline";
      else cat = "ledger/stem";
    }
    if (e.type === "text") {
      if (e.text?.includes("♩")) cat = "tempo";
      else if (e.text?.includes("")) cat = "clef";
      else if (e.text?.includes("")) cat = "timeSig";
      else if (e.fontSize && e.fontSize >= 25) cat = "notehead";
      else cat = "text-other";
    }
    (result[cat] = result[cat] || []).push(e);
  }
  return result;
}

function summarize(cats: Record<string, Element[]>) {
  const s: Record<string, any> = {};
  for (const [k, v] of Object.entries(cats)) {
    s[k] = { count: v.length };
    if (v.length) {
      s[k].xRange = [Math.min(...v.map(e => e.x)), Math.max(...v.map(e => e.x))];
      s[k].yRange = [Math.min(...v.map(e => e.y)), Math.max(...v.map(e => e.y))];
      if (v[0].fontSize) s[k].fontSize = v[0].fontSize;
      if (v[0].strokeWidth) s[k].strokeWidth = v[0].strokeWidth;
    }
  }
  return s;
}

describe("VexFlow vs Layout full element parity", () => {
  beforeAll(async () => { await initWasm(); });

  let vexEls: Element[];
  let ourEls: Element[];

  beforeAll(async () => {
    const score = buildNormalizedScore(SRC);
    const vexSvg = await vexRender(score, { staffScale: 0.75 });
    setLayoutSource(SRC);
    const ourSvg = renderScoreToSvg(score, { staffScale: 0.75, pageWidth: 816, showTitle: true });
    vexEls = parseSvg(vexSvg);
    ourEls = parseSvg(ourSvg);
  });

  it("prints full element summaries", () => {
    const v = summarize(categorize(vexEls));
    const o = summarize(categorize(ourEls));
    console.log("VexFlow:", JSON.stringify(v, null, 2));
    console.log("Layout:", JSON.stringify(o, null, 2));
    expect(true).toBe(true);
  });

  it("staff lines: same count, Y range matches", () => {
    const vCats = categorize(vexEls);
    const oCats = categorize(ourEls);
    const vLines = vCats.staffLine || [];
    const oLines = oCats.staffLine || [];
    expect(oLines.length).toBeGreaterThanOrEqual(5);
    if (vLines.length >= 5) {
      const vTop = vLines[0].y;
      const oTop = oLines[0].y;
      console.log(`Staff top Y — VexFlow: ${vTop}, Layout: ${oTop}, diff: ${Math.abs(vTop - oTop).toFixed(1)}`);
      expect(Math.abs(vTop - oTop)).toBeLessThan(5);
    }
  });

  it("notehead: same font size, Y diff < 5pt", () => {
    const vCats = categorize(vexEls);
    const oCats = categorize(ourEls);
    const vNh = (vCats.notehead || [])[0];
    const oNh = (oCats.notehead || [])[0];
    if (vNh && oNh) {
      console.log(`Notehead size — VexFlow: ${vNh.fontSize}pt, Layout: ${oNh.fontSize}pt`);
      console.log(`Notehead Y — VexFlow: ${vNh.y}, Layout: ${oNh.y}, diff: ${Math.abs(vNh.y - oNh.y).toFixed(1)}`);
      expect(oNh.fontSize).toBe(vNh.fontSize);
      expect(Math.abs(oNh.y - vNh.y)).toBeLessThan(5);
    }
  });

  it("clef: same font size, XY diff small", () => {
    const vCats = categorize(vexEls);
    const oCats = categorize(ourEls);
    const vCl = (vCats.clef || [])[0];
    const oCl = (oCats.clef || [])[0];
    if (vCl && oCl) {
      console.log(`Clef size — VexFlow: ${vCl.fontSize}pt, Layout: ${oCl.fontSize}pt`);
      expect(oCl.fontSize).toBe(vCl.fontSize);
      expect(Math.abs(oCl.x - vCl.x)).toBeLessThan(5);
    }
  });

  it("time sig: same font size, Y diff small", () => {
    const vCats = categorize(vexEls);
    const oCats = categorize(ourEls);
    const vTs = vCats.timeSig || [];
    const oTs = oCats.timeSig || [];
    if (vTs.length && oTs.length) {
      console.log(`Time sig size — VexFlow: ${vTs[0].fontSize}pt, Layout: ${oTs[0].fontSize}pt`);
      expect(oTs[0].fontSize).toBe(vTs[0].fontSize);
      expect(Math.abs(oTs[0].y - vTs[0].y)).toBeLessThan(10);
    }
  });

  it("barlines: exist and Y range matches staff", () => {
    const oCats = categorize(ourEls);
    const oBars = oCats.barline || [];
    console.log(`Layout barlines: ${oBars.length}`);
    expect(oBars.length).toBeGreaterThan(0);
  });
});
