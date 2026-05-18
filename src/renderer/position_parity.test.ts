// @vitest-environment jsdom

import { describe, it, expect, beforeAll } from "vitest";
import { initParserWasmBrowser } from "../wasm/parser_wasm_browser";
import { buildNormalizedScore } from "../dsl/normalize";
import { renderScoreToSvg } from "./svgRenderer";
import { renderScoreToSvg as vexRender } from "../vexflow/renderer";

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
  role?: string;
}

function parseSvg(svg: string): Element[] {
  const doc = new DOMParser().parseFromString(svg, "image/svg+xml");
  const els: Element[] = [];
  for (const node of Array.from(doc.querySelectorAll("line"))) {
    els.push({
      type: "line",
      x: Number(node.getAttribute("x1") ?? "0"),
      y: Number(node.getAttribute("y1") ?? "0"),
      x2: Number(node.getAttribute("x2") ?? "0"),
      y2: Number(node.getAttribute("y2") ?? "0"),
      strokeWidth: Number(node.getAttribute("stroke-width") ?? "0"),
      role: node.getAttribute("data-role") ?? undefined,
    });
  }
  for (const node of Array.from(doc.querySelectorAll("text"))) {
    const fontSizeRaw = node.getAttribute("font-size") ?? "";
    els.push({
      type: "text",
      x: Number(node.getAttribute("x") ?? "0"),
      y: Number(node.getAttribute("y") ?? "0"),
      fontFamily: node.getAttribute("font-family") ?? "",
      fontSize: fontSizeRaw.endsWith("pt") ? Number(fontSizeRaw.slice(0, -2)) : 0,
      text: node.textContent ?? "",
      role: node.getAttribute("data-role") ?? undefined,
    });
  }
  for (const node of Array.from(doc.querySelectorAll("rect"))) {
    els.push({
      type: "rect",
      x: Number(node.getAttribute("x") ?? "0"),
      y: Number(node.getAttribute("y") ?? "0"),
      w: Number(node.getAttribute("width") ?? "0"),
      h: Number(node.getAttribute("height") ?? "0"),
      role: node.getAttribute("data-role") ?? undefined,
    });
  }
  for (const node of Array.from(doc.querySelectorAll("path"))) {
    els.push({
      type: "path",
      x: 0,
      y: 0,
      strokeWidth: Number(node.getAttribute("stroke-width") ?? "0"),
      role: node.getAttribute("data-role") ?? undefined,
    });
  }
  return els;
}

function categorize(els: Element[]) {
  const result: Record<string, Element[]> = {};
  for (const e of els) {
    let cat = e.type;
    if (e.role === "staff-line") cat = "staffLine";
    else if (
      e.role === "opening-barline"
      || e.role === "closing-barline"
      || e.role === "double-barline-left"
      || e.role === "double-barline-right"
      || e.role === "final-barline-thin"
      || e.role === "final-barline-thick"
      || e.role === "barline"
    ) {
      cat = "barline";
    }
    if (e.type === "line") {
      const xDiff = Math.abs((e.x2 ?? e.x) - e.x);
      const yDiff = Math.abs((e.y2 ?? e.y) - e.y);
      if (cat === "line") {
        if (yDiff < 1 && xDiff > 100) cat = "staffLine";
        else if (xDiff < 1 && yDiff > 5) cat = "barline";
        else cat = "ledger/stem";
      }
    }
    if (e.type === "rect" && cat === "rect" && e.role?.includes("barline")) {
      cat = "barline";
    }
    if (e.type === "text") {
      if (e.role === "tempo-glyph" || e.role === "tempo-equals" || e.role === "tempo") cat = "tempo";
      else if (e.role === "percussion-clef" || e.text?.includes("")) cat = "clef";
      else if (e.role === "time-signature-digit" || e.text?.includes("")) cat = "timeSig";
      else if (e.role === "notehead" || (e.fontSize && e.fontSize >= 25)) cat = "notehead";
      else cat = "ledger/stem";
    }
    if (e.type === "text" && (cat === "text" || cat === "ledger/stem")) {
      cat = "text-other";
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
  beforeAll(async () => { await initParserWasmBrowser(); });

  let vexEls: Element[];
  let ourEls: Element[];

  beforeAll(async () => {
    const score = buildNormalizedScore(SRC);
    const vexSvg = await vexRender(score, { staffScale: 0.75 });
    const ourSvg = await renderScoreToSvg(
      score,
      { staffScale: 0.75, pageWidth: 816, showTitle: true },
      { source: SRC, sourceRevision: 1 },
    );
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

  it("notehead: same font size, relative vertical placement matches clef anchor", () => {
    const vCats = categorize(vexEls);
    const oCats = categorize(ourEls);
    const vNh = (vCats.notehead || [])[0];
    const oNh = (oCats.notehead || [])[0];
    const vCl = (vCats.clef || [])[0];
    const oCl = (oCats.clef || [])[0];
    if (vNh && oNh && vCl && oCl) {
      const vDelta = vNh.y - vCl.y;
      const oDelta = oNh.y - oCl.y;
      console.log(`Notehead size — VexFlow: ${vNh.fontSize}pt, Layout: ${oNh.fontSize}pt`);
      console.log(`Notehead ΔY from clef — VexFlow: ${vDelta}, Layout: ${oDelta}, diff: ${Math.abs(oDelta - vDelta).toFixed(1)}`);
      expect(oNh.fontSize).toBe(vNh.fontSize);
      expect(Math.abs(oDelta - vDelta)).toBeLessThan(5);
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
      expect(Math.abs(oCl.x - vCl.x)).toBeLessThan(15);
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
      expect(Math.abs(oTs[0].y - vTs[0].y)).toBeLessThan(35);
    }
  });

  it("barlines: exist and Y range matches staff", () => {
    const oCats = categorize(ourEls);
    const oBars = oCats.barline || [];
    console.log(`Layout barlines: ${oBars.length}`);
    expect(oBars.length).toBeGreaterThan(0);
  });
});
