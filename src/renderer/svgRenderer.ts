import { build_layout_plan } from "../wasm/pkg/drummark_core";

const NOTEHEAD = 30;
let _cachedSource = "";

export function setLayoutSource(src: string) { _cachedSource = src; }

export function renderScoreToSvg(
  _score: any,
  _options?: { staffScale?: number; pageWidth?: number; showTitle?: boolean },
): string {
  const pageWidth = _options?.pageWidth ?? 612;
  const pageHeight = 792;

  // Layout engine: use WASM if source available, fall back to static plan
  let plan: any = { systems: [] };
  try {
    if (_cachedSource) plan = build_layout_plan(_cachedSource) as any;
  } catch {
    // WASM not ready — layout engine unavailable, return empty SVG
  }

  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${pageWidth}" height="${pageHeight}" viewBox="0 0 ${pageWidth} ${pageHeight}" font-family="Bravura,Academico" font-size="10pt">`;
  svg += `<defs><style>
    .vf-staff { stroke: #999; stroke-width: 0.6; fill: none; }
    .vf-notehead { fill: #333; font-size: ${NOTEHEAD}pt; }
    .vf-bar { stroke: #333; stroke-width: 1; }
    .vf-stem { stroke: #333; stroke-width: 1.2; }
    .vf-text { fill: #333; stroke: none; }
  </style></defs>`;

  for (const sys of plan.systems || []) {
    const sy = sys.y || 0;
    const sTop = sy + 10;
    const sBot = sy + 50;

    // Staff lines
    for (let i = 0; i < 5; i++) {
      const ly = sy + 10 + i * 10;
      svg += `<line x1="30" y1="${ly}" x2="${pageWidth - 30}" y2="${ly}" class="vf-staff"/>`;
    }

    // Opening barline
    svg += `<line x1="30" y1="${sTop}" x2="30" y2="${sBot}" class="vf-bar"/>`;

    // Clef
    svg += `<text class="vf-notehead" x="35" y="${sy + 35}">\u{E069}</text>`;

    // Closing barline
    const lastM = sys.measures?.[sys.measures.length - 1];
    if (lastM) {
      const ex = lastM.x + lastM.width;
      svg += `<line x1="${ex}" y1="${sTop}" x2="${ex}" y2="${sBot}" class="vf-bar"/>`;
      svg += `<line x1="${ex + 3}" y1="${sTop}" x2="${ex + 3}" y2="${sBot}" class="vf-bar" stroke-width="3"/>`;
    }
  }

  svg += "</svg>";
  return svg;
}
