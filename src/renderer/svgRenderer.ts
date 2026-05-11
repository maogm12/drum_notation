import { build_layout_plan } from "../wasm/pkg/drummark_core";
import { initWasm } from "../wasm/drummark_wasm";

let _cachedSource = "";
initWasm().catch(() => {});
export function setLayoutSource(src: string) { _cachedSource = src; }

interface DrawCmd {
  tag: string;
  x?: number; y?: number; x1?: number; y1?: number; x2?: number; y2?: number;
  text?: string; fontFamily?: string; fontSize?: number; fill?: string;
  stroke?: string; strokeWidth?: number;
}

export function renderScoreToSvg(
  _score: any,
  _options?: { staffScale?: number; pageWidth?: number; showTitle?: boolean },
): string {
  let plan: any = { pages: [] };
  try {
    if (_cachedSource) plan = build_layout_plan(_cachedSource) as any;
  } catch (e) {
    return `<svg xmlns="http://www.w3.org/2000/svg" width="612" height="792"><text x="20" y="40" fill="#666">Layout engine loading...</text></svg>`;
  }
  const pages = plan.pages || [];
  if (!pages.length) return `<svg xmlns="http://www.w3.org/2000/svg" width="612" height="792"><text x="20" y="40">No layout data</text></svg>`;

  const page = pages[0];
  const pw = page.width || 612;
  const ph = page.height || 792;
  const cmds: DrawCmd[] = page.systems || [];

  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${pw}" height="${ph}" viewBox="0 0 ${pw} ${ph}">`;

  for (const cmd of cmds) {
    switch (cmd.tag) {
      case "line":
        svg += `<line x1="${cmd.x1}" y1="${cmd.y1}" x2="${cmd.x2}" y2="${cmd.y2}" stroke="${cmd.stroke || '#333'}" stroke-width="${cmd.strokeWidth || 1}"/>`;
        break;
      case "text":
        svg += `<text x="${cmd.x}" y="${cmd.y}" font-family="${cmd.fontFamily || 'Bravura'}" font-size="${cmd.fontSize || 30}pt" fill="${cmd.fill || '#333'}">${esc(cmd.text || '')}</text>`;
        break;
    }
  }

  svg += "</svg>";
  return svg;
}

function esc(s: string) { return s.replace(/&/g, "&amp;").replace(/</g, "&lt;"); }
