// @vitest-environment jsdom

import { describe, it, expect, beforeAll } from "vitest";
import { initParserWasmBrowser } from "../wasm/parser_wasm_browser";
import { buildNormalizedScore } from "../dsl/normalize";
import { renderScoreToSvg } from "./svgRenderer";
import { renderScoreToSvg as vexRender } from "../vexflow/renderer";
import fs from "fs";

const SRC = `title Rock
tempo 92
time 4/4
note 1/4

| s s s s |

| s s s s |`;

const RENDER_OPTS = { staffScale: 1.0, pageWidth: 612, pageHeight: 792, showTitle: true, topMargin: 40, bottomMargin: 40, leftMargin: 40, rightMargin: 40 };

interface El {
  tag: string;
  x: number; y: number;
  x2?: number; y2?: number;
  w?: number; h?: number;
  fontSize?: number;
  fontFamily?: string;
  strokeWidth?: number;
  stroke?: string;
  fill?: string;
  text?: string;
  textAnchor?: string;
}

function parseSvg(svg: string): El[] {
  const els: El[] = [];
  // Lines
  const lineRe = /<line\s+x1="([\d.]+)"\s+y1="([\d.]+)"\s+x2="([\d.]+)"\s+y2="([\d.]+)"\s+stroke="([^"]*)"\s+stroke-width="([\d.]+)"\/>/g;
  let m;
  while ((m = lineRe.exec(svg))) els.push({ tag: "line", x: +m[1], y: +m[2], x2: +m[3], y2: +m[4], stroke: m[5], strokeWidth: +m[6] });
  // Texts (try multiple attr orders)
  const textRe1 = /<text\s+[^>]*x="([\d.]+)"[^>]*y="([\d.]+)"[^>]*font-family="([^"]*)"[^>]*font-size="([\d.]+)pt"[^>]*fill="([^"]*)"(?:[^>]*text-anchor="([^"]*)")?[^>]*>([^<]*)/g;
  const textRe2 = /<text\s+[^>]*x="([\d.]+)"[^>]*y="([\d.]+)"[^>]*>/g;
  while ((m = textRe1.exec(svg))) {
    const _fontFam = m[3], _fontSz = +m[4], _fill = m[5], _anchor = m[6] || "";
    // Extract the text content between > and <
    const contentMatch = svg.substring(m.index).match(/>([^<]*)<\/text>/);
    const text = contentMatch ? contentMatch[1] : "";
    els.push({ tag: "text", x: +m[1], y: +m[2], fontFamily: m[3], fontSize: +m[4], fill: m[5], textAnchor: m[6] || "", text });
  }
  // Rects
  const rectRe = /<rect\s+x="([\d.]+)"\s+y="([\d.]+)"\s+width="([\d.]+)"\s+height="([\d.]+)"[^>]*\/>/g;
  while ((m = rectRe.exec(svg))) els.push({ tag: "rect", x: +m[1], y: +m[2], w: +m[3], h: +m[4] });
  // Paths
  const pathRe = /<path[^>]*d="M([\d.]+)\s+([\d.]+)L([\d.]+)\s+([\d.]+)"[^>]*stroke-width="([\d.]+)"[^>]*>/g;
  while ((m = pathRe.exec(svg))) els.push({ tag: "path", x: +m[1], y: +m[2], x2: +m[3], y2: +m[4], strokeWidth: +m[5] });
  return els;
}

function classify(e: El): string {
  if (e.tag === "text") {
    if (e.fontSize && e.fontSize >= 25 && e.fontFamily?.includes("Bravura")) return "notehead";
    if (e.text?.includes("♩") || e.text?.includes("=") && e.fontSize && e.fontSize <= 14) return "tempo";
    if (e.text?.includes("")) return "clef";
    if (e.text?.includes("")) return "timeSig";
    if (e.fontSize && e.fontSize >= 14 && e.fontSize < 30) return "title";
    return "text-other";
  }
  if (e.tag === "line") {
    const dy = Math.abs((e.y2 ?? e.y) - e.y);
    const dx = Math.abs((e.x2 ?? e.x) - e.x);
    if (dy < 1 && dx > 50) return "staffLine";
    if (dx < 1 && dy > 5) return "barline";
    if (dx < 1 && dy > 0) return "stem";
    return "line-other";
  }
  if (e.tag === "path") {
    const dy = Math.abs((e.y2 ?? e.y) - e.y);
    const dx = Math.abs((e.x2 ?? e.x) - e.x);
    if (dy < 1 && dx > 50) return "staffLine";
    if (dx < 1 && dy > 5) return "barline";
    if (dx < 1 && dy > 0) return "stem";
    return "path-other";
  }
  if (e.tag === "rect") return "rect";
  return "other";
}

describe("Detailed render diff", () => {
  beforeAll(async () => { await initParserWasmBrowser(); });

  it("prints detailed comparison", async () => {
    const score = buildNormalizedScore(SRC);
    const vexSvg = await vexRender(score, RENDER_OPTS);
    const ourSvg = await renderScoreToSvg(score, RENDER_OPTS as any, {
      source: SRC,
      sourceRevision: 1,
    });

    // Save for inspection
    fs.writeFileSync("/tmp/vex_detailed.svg", vexSvg);
    fs.writeFileSync("/tmp/our_detailed.svg", ourSvg);

    const vEls = parseSvg(vexSvg);
    const oEls = parseSvg(ourSvg);

    let report = "=".repeat(70) + "\nVEXFLOW\n" + "=".repeat(70) + "\n";
    report += reportCats(vEls);
    report += "\n" + "=".repeat(70) + "\nLAYOUT ENGINE\n" + "=".repeat(70) + "\n";
    report += reportCats(oEls);
    report += "\n" + "=".repeat(70) + "\nDIFFS\n" + "=".repeat(70) + "\n";
    report += doDiff(vEls, oEls);

    console.log(report);
    fs.writeFileSync("/tmp/render_diff.txt", report);
    expect(true).toBe(true);
  });
});

function reportCats(els: El[]): string {
  let s = "";
  const cats = ["title", "tempo", "clef", "timeSig", "notehead", "stem", "staffLine", "barline", "rect", "text-other", "line-other"];
  for (const cat of cats) {
    const items = els.filter(e => classify(e) === cat);
    if (!items.length) continue;
    s += `\n[${cat}] ×${items.length}\n`;
    if (cat === "title" || cat === "tempo" || cat === "clef") {
      for (const it of items.slice(0,4))
        s += `  x=${it.x.toFixed(1)} y=${it.y.toFixed(1)} sz=${it.fontSize}pt ff="${it.fontFamily}" fill="${it.fill}" "${it.text?.substring(0,20)}"\n`;
    } else if (cat === "timeSig") {
      const ys = [...new Set(items.map(e => e.y.toFixed(1)))].sort();
      s += `  x=${items[0].x.toFixed(1)} ys=${ys.join(",")} sz=${items[0].fontSize}pt\n`;
    } else if (cat === "notehead") {
      const xs = items.map(e => e.x).sort((a,b)=>a-b);
      const ys = [...new Set(items.map(e => e.y))];
      s += `  X: ${xs.map(x=>x.toFixed(1)).join(", ")}\n  Y: ${ys.map(y=>y.toFixed(1)).join(", ")}\n  sz=${items[0].fontSize}pt\n`;
    } else if (cat === "stem") {
      const lens = items.map(e => Math.abs((e.y2??e.y)-e.y));
      s += `  len: ${lens.map(l=>l.toFixed(1)).join(", ")} sw=${items[0].strokeWidth}\n`;
    } else if (cat === "staffLine") {
      const ys = [...new Set(items.map(e => e.y))].sort((a,b)=>a-b);
      s += `  Y: ${ys.map(y=>y.toFixed(1)).join(", ")} sw=${items[0]?.strokeWidth}\n`;
    } else if (cat === "barline") {
      const xs = items.map(e => Math.min(e.x, e.x2??e.x)).sort((a,b)=>a-b);
      s += `  X: ${xs.map(x=>x.toFixed(1)).join(", ")} sw=${items[0]?.strokeWidth}\n`;
    } else if (cat === "rect") {
      for (const r of items.slice(0,6))
        s += `  x=${r.x.toFixed(1)} y=${r.y.toFixed(1)} w=${r.w!.toFixed(1)} h=${r.h!.toFixed(1)}\n`;
    }
  }
  return s;
}

function doDiff(vEls: El[], oEls: El[]): string {
  let s = "";
  function get(c: string, els: El[]) { return els.filter(e => classify(e) === c); }

  // Title
  const vTi = get("title", vEls)[0];
  const oTi = get("title", oEls)[0];
  if (!vTi && !oTi) s += "title: both MISSING\n";
  else if (vTi && oTi) {
    const d = [];
    if (Math.abs(vTi.x - oTi.x) > 0.5) d.push(`x ${vTi.x.toFixed(1)}→${oTi.x.toFixed(1)}`);
    if (Math.abs(vTi.y - oTi.y) > 0.5) d.push(`y ${vTi.y.toFixed(1)}→${oTi.y.toFixed(1)}`);
    if (vTi.fontSize !== oTi.fontSize) d.push(`sz ${vTi.fontSize}→${oTi.fontSize}`);
    if (vTi.fontFamily !== oTi.fontFamily) d.push(`font ${vTi.fontFamily}→${oTi.fontFamily}`);
    s += "title: " + (d.length ? d.join(", ") : "SAME") + "\n";
  } else s += `title: miss V=${!!vTi} O=${!!oTi}\n`;

  // Tempo
  const vTp = get("tempo", vEls)[0];
  const oTp = get("tempo", oEls)[0];
  if (!vTp && !oTp) s += "tempo: both MISSING\n";
  else if (vTp && oTp) {
    const d = [];
    if (Math.abs(vTp.x - oTp.x) > 0.5) d.push(`x ${vTp.x.toFixed(1)}→${oTp.x.toFixed(1)}`);
    if (Math.abs(vTp.y - oTp.y) > 0.5) d.push(`y ${vTp.y.toFixed(1)}→${oTp.y.toFixed(1)}`);
    if (vTp.fontSize !== oTp.fontSize) d.push(`sz ${vTp.fontSize}→${oTp.fontSize}`);
    s += "tempo: " + (d.length ? d.join(", ") : "SAME") + "\n";
  } else s += `tempo: miss V=${!!vTp} O=${!!oTp}\n`;

  // Staff lines
  const vSL = get("staffLine", vEls);
  const oSL = get("staffLine", oEls);
  const vSY = [...new Set(vSL.map(e=>e.y))].sort((a,b)=>a-b);
  const oSY = [...new Set(oSL.map(e=>e.y))].sort((a,b)=>a-b);
  s += `staffLines: V=${vSL.length} (path-based=${vSL.filter(e=>e.tag==="path").length}) O=${oSL.length} (line-based)\n`;
  s += `  V Y: ${vSY.map(y=>y.toFixed(1)).join(", ")}\n`;
  s += `  O Y: ${oSY.map(y=>y.toFixed(1)).join(", ")}\n`;

  // Clef, TimeSig
  const vCl = get("clef", vEls);
  const oCl = get("clef", oEls);
  if (vCl.length || oCl.length) {
    s += `clef: V x=${vCl[0]?.x.toFixed(1)??"?"} y=${vCl[0]?.y.toFixed(1)??"?"} O x=${oCl[0]?.x.toFixed(1)??"?"} y=${oCl[0]?.y.toFixed(1)??"?"}\n`;
  } else s += "clef: both MISSING\n";

  const vTs = get("timeSig", vEls);
  const oTs = get("timeSig", oEls);
  if (vTs.length || oTs.length) s += `timeSig: V=${vTs.length} O=${oTs.length}\n`;
  else s += "timeSig: MISSING\n";

  // Noteheads
  const vNh = get("notehead", vEls);
  const oNh = get("notehead", oEls);
  if (vNh.length || oNh.length) {
    s += `noteheads: V=${vNh.length} O=${oNh.length}\n`;
    const vX = vNh.map(e=>e.x).sort((a,b)=>a-b);
    const oX = oNh.map(e=>e.x).sort((a,b)=>a-b);
    s += `  V X: ${vX.map(x=>x.toFixed(1)).join(", ")}\n`;
    s += `  O X: ${oX.map(x=>x.toFixed(1)).join(", ")}\n`;
    const vY = [...new Set(vNh.map(e=>e.y))];
    const oY = [...new Set(oNh.map(e=>e.y))];
    s += `  V Y: ${vY.map(y=>y.toFixed(1)).join(", ")}\n`;
    s += `  O Y: ${oY.map(y=>y.toFixed(1)).join(", ")}\n`;
    s += `  sz: V=${vNh[0]?.fontSize}pt O=${oNh[0]?.fontSize}pt\n`;
  }

  // Stems
  const vSt = get("stem", vEls);
  const oSt = get("stem", oEls);
  if (vSt.length || oSt.length) {
    const vLn = vSt.map(e=>Math.abs((e.y2??e.y)-e.y));
    const oLn = oSt.map(e=>Math.abs((e.y2??e.y)-e.y));
    s += `stems: V=${vSt.length} len=${vLn.map(l=>l.toFixed(1)).join(",")} sw=${vSt[0]?.strokeWidth} O=${oSt.length} len=${oLn.map(l=>l.toFixed(1)).join(",")} sw=${oSt[0]?.strokeWidth}\n`;
  }

  // Barlines
  const vBl = get("barline", vEls);
  const oBl = get("barline", oEls);
  s += `barlines: V=${vBl.length} O=${oBl.length}\n`;
  
  // Rects
  const vRc = get("rect", vEls);
  const oRc = get("rect", oEls);
  s += `rects: V=${vRc.length} O=${oRc.length}\n`;

  return s;
}
