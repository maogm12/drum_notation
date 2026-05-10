import type { NormalizedScore, NormalizedMeasure, NormalizedEvent } from "../dsl/types";
import { trackFamily } from "./layoutMetrics";

// ── Public API ───────────────────────────────────────────────────

export function renderScoreToSvg(
  score: NormalizedScore,
  _options?: { staffScale?: number; pageWidth?: number; showTitle?: boolean },
): string {
  const pageWidth = _options?.pageWidth ?? 612;
  const staffScale = _options?.staffScale ?? 0.75;
  const showTitle = _options?.showTitle ?? true;
  const marginLeft = 50;
  const marginRight = 50;
  const marginTop = 30;
  const staffHeightPx = 40 * staffScale;
  const staffSpace = staffHeightPx / 4;
  const pxPerQuarter = 80 * staffScale;

  // ── Layout ────────────────────────────────────────────────────

  let currentY = marginTop;
  let measures: { m: NormalizedMeasure; x: number; width: number }[] = [];

  if (showTitle && score.header.title) {
    currentY += 30;
  }

  // Simple measure layout: proportionally space measures
  let cursorX = marginLeft + 70; // space for clef + time sig
  const usableWidth = pageWidth - marginLeft - marginRight - 70;

  for (const measure of score.measures) {
    const totalSlots = measure.events.length || 1;
    const quarters = totalSlots / (score.header.divisions || 16) * 4;
    const width = Math.max(quarters * pxPerQuarter, 60);

    if (cursorX + width > marginLeft + usableWidth && measures.length > 0) {
      // New system
      currentY += staffHeightPx + 40;
      cursorX = marginLeft + 70;
    }

    measures.push({ m: measure, x: cursorX, width });
    cursorX += width;
  }

  const totalHeight = currentY + staffHeightPx + marginTop + 20;
  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${pageWidth}" height="${totalHeight}" viewBox="0 0 ${pageWidth} ${totalHeight}">`;
  svg += `<defs><style>
    .staff-line { stroke: #333; stroke-width: 0.5; }
    .barline { stroke: #333; stroke-width: 1; }
    .barline-double { stroke: #333; stroke-width: 1; }
    .barline-final { stroke: #333; stroke-width: 1; }
    .notehead { fill: #333; font-family: Bravura, sans-serif; }
    .rest { fill: #333; font-family: Bravura, sans-serif; }
    .stem { stroke: #333; stroke-width: 1; }
    .beam { stroke: #333; stroke-width: 3; stroke-linecap: round; }
    .nav-text { fill: #333; font-family: Academico, serif; font-size: 12px; }
  </style></defs>`;

  // ── Title ──────────────────────────────────────────────────────
  if (showTitle && score.header.title) {
    svg += `<text x="${pageWidth / 2}" y="${marginTop}" text-anchor="middle" font-family="Academico, serif" font-size="18" fill="#333">${esc(score.header.title)}</text>`;
  }

  // ── Render measures ────────────────────────────────────────────
  let prevSystemY = marginTop + (showTitle && score.header.title ? 30 : 0);

  for (const { m, x, width } of measures) {
    const staffY = prevSystemY;
    svg += renderStaffLines(x, staffY, width, staffHeightPx, staffSpace);
    svg += renderBarline(x, staffY, m.barline, staffHeightPx);
    svg += renderMeasureContent(m, x, staffY, width, staffSpace, pxPerQuarter);
  }

  // ── Final barline ──────────────────────────────────────────────
  const lastM = measures[measures.length - 1];
  if (lastM) {
    const finalX = lastM.x + lastM.width;
    svg += renderBarline(finalX, prevSystemY, "final", staffHeightPx);
  }

  svg += "</svg>";
  return svg;
}

// ── Staff Lines ─────────────────────────────────────────────────

function renderStaffLines(x: number, y: number, width: number, _staffHeightPx: number, staffSpace: number): string {
  let svg = "";
  for (let i = 0; i < 5; i++) {
    const lineY = y + i * staffSpace;
    svg += `<line x1="${x - 5}" y1="${lineY}" x2="${x + width + 5}" y2="${lineY}" class="staff-line"/>`;
  }
  return svg;
}

// ── Barlines ─────────────────────────────────────────────────────

function renderBarline(x: number, y: number, type?: string | null, _staffHeightPx?: number): string {
  const h = _staffHeightPx ?? 40;
  switch (type) {
    case "double": case "final":
      return `<line x1="${x}" y1="${y}" x2="${x}" y2="${y + h}" class="barline"/>`
        + `<line x1="${x + 4}" y1="${y}" x2="${x + 4}" y2="${y + h}" class="barline"/>`;
    case "repeat-start":
      return `<line x1="${x + 4}" y1="${y}" x2="${x + 4}" y2="${y + h}" class="barline"/>`
        + `<line x1="${x + 10}" y1="${y}" x2="${x + 10}" y2="${y + h}" class="barline"/>`
        + `<text x="${x + 14}" y="${y + h / 2 + 4}" class="nav-text">:</text>`;
    case "repeat-end":
      return `<text x="${x - 4}" y="${y + h / 2 + 4}" text-anchor="end" class="nav-text">:</text>`
        + `<line x1="${x}" y1="${y}" x2="${x}" y2="${y + h}" class="barline"/>`
        + `<line x1="${x + 6}" y1="${y}" x2="${x + 6}" y2="${y + h}" class="barline"/>`;
    case "repeat-both":
      return renderBarline(x, y, "repeat-end", h) + renderBarline(x + 12, y, "repeat-start", h);
    default:
      return `<line x1="${x}" y1="${y}" x2="${x}" y2="${y + h}" class="barline"/>`;
  }
}

// ── Measure Content ──────────────────────────────────────────────

function renderMeasureContent(
  m: NormalizedMeasure,
  measureX: number,
  staffY: number,
  _measureWidth: number,
  staffSpace: number,
  _pxPerQuarter: number,
): string {
  let svg = "";
  let prevX = measureX + 8;

  for (const ev of m.events) {
    const x = prevX + 12;
    const y = noteY(ev.track, staffY, staffSpace);
    const isRest = ev.kind !== "hit" && ev.kind !== "sticking";
    const notehead = isRest ? noteheadGlyph(ev) : noteGlyph(ev);

    if (isRest) {
      svg += `<text x="${x}" y="${y + staffSpace * 0.5}" class="rest" font-size="${staffSpace * 2}px" text-anchor="middle">${notehead}</text>`;
    } else {
      svg += `<text x="${x}" y="${y + staffSpace * 0.7}" class="notehead" font-size="${staffSpace * 2.5}px" text-anchor="middle">${notehead}</text>`;
      // Stem
      const stemUp = ev.voice === 1;
      const stemY1 = y - (stemUp ? staffSpace * 3 : 0);
      const stemY2 = y + (stemUp ? 0 : staffSpace * 3);
      svg += `<line x1="${x + staffSpace}" y1="${stemY1}" x2="${x + staffSpace}" y2="${stemY2}" class="stem"/>`;
    }

    prevX = x;
  }
  return svg;
}

// ── Note Y Position ─────────────────────────────────────────────

function noteY(track: string, staffY: number, staffSpace: number): number {
  const pos: Record<string, number> = {
    HH: 0, RC: 1, RC2: 1, C: 2, C2: 2, SPL: -1, CHN: -1,
    T1: 3, T2: 4, T3: 5, T4: 6, SD: 4, BD: 8, BD2: 8, HF: 9,
    ST: -1, CB: 0, WB: 0, CL: 0,
  };
  return staffY + (pos[track] ?? 4) * (staffSpace / 2);
}

// ── Glyph Mapping ────────────────────────────────────────────────

function noteGlyph(ev: NormalizedEvent): string {
  const family = trackFamily(ev.track);
  if (family === "cymbal") return "\u{E0A9}"; // X notehead
  for (const m of ev.modifiers || []) {
    if (m === "open") return "\u{E0B3}";
    if (m === "cross") return "\u{E0A9}";
    if (m === "bell") return "\u{E0DB}";
    if (m === "rim") return "\u{E0CE}";
  }
  return "\u{E0A4}"; // standard notehead
}

function noteheadGlyph(ev: NormalizedEvent): string {
  const d = ev.duration?.denominator ?? 4;
  if (d >= 32) return "\u{E4E7}";
  if (d >= 16) return "\u{E4E6}";
  if (d >= 8)  return "\u{E4E5}";
  if (d >= 4)  return "\u{E4E4}";
  return "\u{E4E3}";
}

// ── Helpers ──────────────────────────────────────────────────────

function esc(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
