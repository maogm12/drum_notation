import { build_layout_scene } from "../wasm/pkg/drummark_core";
import { initWasm } from "../wasm/drummark_wasm";
import { SETTINGS_RANGES } from "../vexflow/config";

let cachedSource = "";
initWasm().catch(() => {});

export function setLayoutSource(src: string) {
  cachedSource = src;
}

type RenderOptions = {
  staffScale?: number;
  pageWidth?: number;
  showTitle?: boolean;
  topMargin?: number;
  bottomMargin?: number;
  leftMargin?: number;
  rightMargin?: number;
  stemLength?: number;
  systemSpacing?: number;
  debug?: boolean;
};

type Scene = {
  version: string;
  metricsVersion: string;
  pages: ScenePage[];
  issues?: string[];
};

type ScenePage = {
  index: number;
  widthPt: number;
  heightPt: number;
  systems: SceneSystem[];
  measures: SceneMeasure[];
  items: SceneItem[];
  composites: SceneComposite[];
};

type SceneSystem = {
  id: string;
  index: number;
  pageIndex: number;
  xPt: number;
  yPt: number;
  widthPt: number;
  heightPt: number;
  measureIds: string[];
};

type SceneMeasure = {
  id: string;
  globalIndex: number;
  systemId: string;
  xPt: number;
  yPt: number;
  widthPt: number;
  heightPt: number;
};

type SceneItem = {
  id: string;
  measureId?: string;
  anchorItemId?: string;
  role: string;
  kind: "glyphRun" | "lineSegment" | "path" | "polyline" | "rect" | "textRun";
  zIndex: number;
  primitive: Record<string, unknown>;
};

type SceneComposite = {
  id: string;
  kind: "repeatSpan" | "volta" | "hairpin" | "navigation" | "measureRepeat" | "multiRest" | "textBlock";
  fragment: "singleSegment" | "start" | "continuation" | "end";
  childItemIds?: string[];
  label?: string;
  count?: number;
  startAnchorId?: string;
  endAnchorId?: string;
};

export function buildLayoutSceneFromSource(source: string, options?: RenderOptions): Scene {
  const ss = options?.staffScale ?? 0.75;
  const logicalW = (options?.pageWidth ?? 612) / ss;
  const logicalH = 792 / ss;
  const opts = {
    pageWidth: logicalW,
    pageHeight: logicalH,
    topMargin: (options?.topMargin ?? 40) / ss,
    bottomMargin: (options?.bottomMargin ?? 40) / ss,
    leftMargin: (options?.leftMargin ?? 40) / ss,
    rightMargin: (options?.rightMargin ?? 40) / ss,
    staffScale: 1.0,
    pxPerQuarter: 80,
    stemLenPt: options?.stemLength ?? 31,
    systemSpacing: (options?.systemSpacing ?? SETTINGS_RANGES.systemSpacing.default) / ss,
    debug: options?.debug ? 1 : 0,
  };
  const scene = build_layout_scene(source, opts as any) as Scene;
  if (!scene || !Array.isArray(scene.pages)) {
    throw new Error("Layout scene export returned an invalid payload.");
  }
  if (scene.pages.length === 0 && scene.issues?.length) {
    throw new Error(scene.issues.join("\n"));
  }
  return scene;
}

export function renderSceneToSvg(scene: Scene, options?: RenderOptions): string {
  const page = scene.pages?.[0];
  if (!page) {
    const reason = scene.issues?.[0] || "No layout data";
    throw new Error(reason);
  }
  const ss = options?.staffScale ?? 0.75;
  const width = page.widthPt || 612;
  const height = page.heightPt || 792;
  const items = [...page.items].sort((a, b) => a.zIndex - b.zIndex);
  const measureMap = new Map((page.measures || []).map((measure) => [measure.id, measure]));
  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${(width * ss).toFixed(0)}" height="${(height * ss).toFixed(0)}" viewBox="0 0 ${width} ${height}">`;

  for (const item of items) {
    const roleAttr = ` data-role="${esc(item.role)}"`;
    const measureAttr = item.measureId ? ` data-measure-id="${esc(item.measureId)}"` : "";
    const anchorAttr = item.anchorItemId ? ` data-anchor-item-id="${esc(item.anchorItemId)}"` : "";
    switch (item.kind) {
      case "glyphRun": {
        const p = item.primitive as {
          xPt: number;
          yPt: number;
          codepoint?: number;
          fontFamily?: string;
          fontSizePt?: number;
          fill?: string;
        };
        const glyph = p.codepoint ? String.fromCodePoint(p.codepoint) : "";
        svg += `<text${roleAttr}${measureAttr}${anchorAttr} x="${p.xPt}" y="${p.yPt}" font-family="${p.fontFamily || "Bravura"}" font-size="${p.fontSizePt || 12}pt" fill="${p.fill || "#333"}">${esc(glyph)}</text>`;
        break;
      }
      case "lineSegment": {
        const p = item.primitive as {
          x1Pt: number;
          y1Pt: number;
          x2Pt: number;
          y2Pt: number;
          stroke?: string;
          strokeWidth?: number;
          strokeLineCap?: string;
        };
        const capAttr = p.strokeLineCap ? ` stroke-linecap="${p.strokeLineCap}"` : "";
        svg += `<line${roleAttr}${measureAttr}${anchorAttr} x1="${p.x1Pt}" y1="${p.y1Pt}" x2="${p.x2Pt}" y2="${p.y2Pt}" stroke="${p.stroke || "#333"}" stroke-width="${p.strokeWidth || 1}"${capAttr}/>`;
        break;
      }
      case "rect": {
        const p = item.primitive as {
          xPt: number;
          yPt: number;
          widthPt: number;
          heightPt: number;
          fill?: string;
          stroke?: string;
          strokeWidth?: number;
        };
        const stroke = p.stroke ? ` stroke="${p.stroke}" stroke-width="${p.strokeWidth || 1}"` : "";
        svg += `<rect${roleAttr}${measureAttr}${anchorAttr} x="${p.xPt}" y="${p.yPt}" width="${p.widthPt}" height="${p.heightPt}" fill="${p.fill || "none"}"${stroke}/>`;
        break;
      }
      case "path": {
        const p = item.primitive as {
          d: string;
          fill?: string;
          stroke?: string;
          strokeWidth?: number;
        };
        const stroke = p.stroke ? ` stroke="${p.stroke}" stroke-width="${p.strokeWidth || 1}"` : "";
        svg += `<path${roleAttr}${measureAttr}${anchorAttr} d="${esc(p.d)}" fill="${p.fill || "none"}"${stroke}/>`;
        break;
      }
      case "polyline": {
        const p = item.primitive as {
          pointsPt: Array<[number, number]>;
        };
        const points = (p.pointsPt || []).map(([x, y]) => `${x},${y}`).join(" ");
        svg += `<polyline${roleAttr}${measureAttr}${anchorAttr} points="${points}" fill="none" stroke="#333" stroke-width="1"/>`;
        break;
      }
      case "textRun": {
        const p = item.primitive as {
          xPt: number;
          yPt: number;
          text: string;
          fontFamily?: string;
          fontSizePt?: number;
          fill?: string;
          textAnchor?: string;
          fontWeight?: string;
        };
        const anchor = p.textAnchor ? ` text-anchor="${p.textAnchor}"` : "";
        const weight = p.fontWeight ? ` font-weight="${p.fontWeight}"` : "";
        svg += `<text${roleAttr}${measureAttr}${anchorAttr} x="${p.xPt}" y="${p.yPt}" font-family="${p.fontFamily || "Bravura"}" font-size="${p.fontSizePt || 12}pt" fill="${p.fill || "#333"}"${anchor}${weight}>${esc(p.text)}</text>`;
        break;
      }
      default:
        throw new Error(`Unsupported scene item kind: ${(item as { kind: string }).kind}`);
    }
  }

  for (const composite of page.composites || []) {
    svg += renderCompositeToSvg(composite, measureMap);
  }

  svg += "</svg>";
  return svg;
}

export function renderSourceToSvg(source: string, options?: RenderOptions): string {
  return renderSceneToSvg(buildLayoutSceneFromSource(source, options), options);
}

export function renderScoreToSvg(_score: unknown, options?: RenderOptions): string {
  if (!cachedSource) {
    return `<svg xmlns="http://www.w3.org/2000/svg" width="612" height="792"><text x="20" y="40">No layout source</text></svg>`;
  }
  try {
    return renderSourceToSvg(cachedSource, options);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return `<svg xmlns="http://www.w3.org/2000/svg" width="612" height="792"><text x="20" y="40" fill="#666">${esc(message)}</text></svg>`;
  }
}

function esc(s: string) {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function renderCompositeToSvg(
  composite: SceneComposite,
  measureMap: Map<string, SceneMeasure>,
): string {
  const start = composite.startAnchorId ? measureMap.get(composite.startAnchorId) : undefined;
  const end = composite.endAnchorId ? measureMap.get(composite.endAnchorId) : undefined;
  if (!start || !end) {
    return "";
  }

  switch (composite.kind) {
    case "repeatSpan":
      if (composite.childItemIds?.length) {
        return "";
      }
      return renderRepeatSpanComposite(composite, start, end);
    case "volta":
      if (composite.childItemIds?.length) {
        return "";
      }
      return renderVoltaComposite(composite, start, end);
    case "hairpin":
    case "navigation":
    case "measureRepeat":
    case "multiRest":
    case "textBlock":
      return "";
    default:
      throw new Error(`Unsupported scene composite kind: ${String((composite as { kind: string }).kind)}`);
  }
}

function renderRepeatSpanComposite(
  composite: SceneComposite,
  start: SceneMeasure,
  end: SceneMeasure,
): string {
  const x1 = start.xPt + 4;
  const x2 = end.xPt + end.widthPt - 4;
  const y = Math.min(start.yPt, end.yPt) - 18;
  const countText =
    composite.count && composite.count > 1 && (composite.fragment === "singleSegment" || composite.fragment === "start")
      ? `${composite.count}x`
      : "";
  let svg = "";
  svg += `<line data-role="repeat-span-line" x1="${x1}" y1="${y}" x2="${x2}" y2="${y}" stroke="#333" stroke-width="1.2"/>`;
  if (composite.fragment !== "continuation" && composite.fragment !== "end") {
    svg += `<line data-role="repeat-span-start" x1="${x1}" y1="${y}" x2="${x1}" y2="${y + 8}" stroke="#333" stroke-width="1.2"/>`;
  }
  if (composite.fragment === "singleSegment" || composite.fragment === "end") {
    svg += `<line data-role="repeat-span-end" x1="${x2}" y1="${y}" x2="${x2}" y2="${y + 8}" stroke="#333" stroke-width="1.2"/>`;
  }
  if (countText) {
    svg += `<text data-role="repeat-span-count" x="${(x1 + x2) / 2}" y="${y - 4}" font-family="Bravura" font-size="10pt" fill="#333" text-anchor="middle">${esc(countText)}</text>`;
  }
  return svg;
}

function renderVoltaComposite(
  composite: SceneComposite,
  start: SceneMeasure,
  end: SceneMeasure,
): string {
  const x1 = start.xPt + 2;
  const x2 = end.xPt + end.widthPt - 2;
  const y = Math.min(start.yPt, end.yPt) - 10;
  let svg = "";
  svg += `<line data-role="volta-line" x1="${x1}" y1="${y}" x2="${x2}" y2="${y}" stroke="#333" stroke-width="1.2"/>`;
  if (composite.fragment !== "continuation") {
    svg += `<line data-role="volta-start-hook" x1="${x1}" y1="${y}" x2="${x1}" y2="${y + 10}" stroke="#333" stroke-width="1.2"/>`;
  }
  if (composite.fragment === "singleSegment" || composite.fragment === "end") {
    svg += `<line data-role="volta-end-hook" x1="${x2}" y1="${y}" x2="${x2}" y2="${y + 10}" stroke="#333" stroke-width="1.2"/>`;
  }
  if (composite.label && composite.fragment !== "continuation" && composite.fragment !== "end") {
    svg += `<text data-role="volta-label" x="${x1 + 4}" y="${y - 4}" font-family="Bravura" font-size="10pt" fill="#333">${esc(composite.label)}</text>`;
  }
  return svg;
}
