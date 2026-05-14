import { build_layout_plan } from "../wasm/pkg/drummark_core";
import { initWasm } from "../wasm/drummark_wasm";

let _cachedSource = "";
initWasm().catch(() => {});
export function setLayoutSource(src: string) { _cachedSource = src; }

interface DrawCmd {
  tag: string;
  x?: number; y?: number; x1?: number; y1?: number; x2?: number; y2?: number;
  width?: number; height?: number;
  text?: string; fontFamily?: string; fontSize?: number; fill?: string;
  textAnchor?: string; fontWeight?: string;
  stroke?: string; strokeWidth?: number;
}

export function renderScoreToSvg(
  _score: any,
  _options?: { staffScale?: number; pageWidth?: number; showTitle?: boolean; topMargin?: number; bottomMargin?: number; leftMargin?: number; rightMargin?: number; debug?: boolean },
): string {
  let plan: any = { pages: [] };
  try {
    if (_cachedSource) {
      const ss = _options?.staffScale ?? 0.75;
      const logicalW = (_options?.pageWidth ?? 612) / ss;
      const logicalH = 792 / ss;
      const opts = {
        pageWidth: logicalW,
        pageHeight: logicalH,
        topMargin: (_options?.topMargin ?? 40) / ss,
        bottomMargin: (_options?.bottomMargin ?? 40) / ss,
        leftMargin: (_options?.leftMargin ?? 40) / ss,
        rightMargin: (_options?.rightMargin ?? 40) / ss,
        staffScale: 1.0,
        pxPerQuarter: 80,
        debug: _options?.debug ? 1 : 0,
      };
      plan = build_layout_plan(_cachedSource, opts as any) as any;
    }
  } catch (e) {
    return `<svg xmlns="http://www.w3.org/2000/svg" width="612" height="792"><text x="20" y="40" fill="#666">Layout engine loading...</text></svg>`;
  }
  const pages = plan.pages || [];
  if (!pages.length) return `<svg xmlns="http://www.w3.org/2000/svg" width="612" height="792"><text x="20" y="40">No layout data</text></svg>`;

  const page = pages[0];
  const ss = _options?.staffScale ?? 0.75;
  const pw = page.width || 612;
  const ph = page.height || 792;
  // ViewBox uses logical coordinates; physical width/height scales back
  const cmds: DrawCmd[] = page.systems || [];
  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${(pw * ss).toFixed(0)}" height="${(ph * ss).toFixed(0)}" viewBox="0 0 ${pw} ${ph}">`;

  for (const cmd of cmds) {
    switch (cmd.tag) {
      case "line":
        svg += `<line x1="${cmd.x1}" y1="${cmd.y1}" x2="${cmd.x2}" y2="${cmd.y2}" stroke="${cmd.stroke || '#333'}" stroke-width="${cmd.strokeWidth || 1}"/>`;
        break;
      case "rect": {
        const f = cmd.fill || "none";
        const s = (cmd as any).stroke ? ` stroke="${(cmd as any).stroke}" stroke-width="${(cmd as any).strokeWidth || 1}"` : "";
        svg += `<rect x="${cmd.x}" y="${cmd.y}" width="${cmd.width}" height="${cmd.height}" fill="${f}"${s}/>`;
        break;
      }
      case "text": {
        const fw = (cmd as any).fontWeight ? ` font-weight="${(cmd as any).fontWeight}"` : "";
        svg += `<text x="${cmd.x}" y="${cmd.y}" dominant-baseline="central" font-family="${cmd.fontFamily || 'Bravura'}" font-size="${cmd.fontSize || 30}pt" fill="${cmd.fill || '#333'}"${fw}${(cmd as any).textAnchor ? ` text-anchor="${(cmd as any).textAnchor}"` : ""}>${esc(cmd.text || '')}</text>`;
        break;
      }
      case "g_open":
        svg += `<g>`;
        break;
      case "g_close":
        svg += `</g>`;
        break;
    }
  }

  svg += "</svg>";
  return svg;
}

function esc(s: string) { return s.replace(/&/g, "&amp;").replace(/</g, "&lt;"); }
