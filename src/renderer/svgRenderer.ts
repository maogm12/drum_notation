import type { NormalizedScore, NormalizedMeasure, NormalizedEvent } from "../dsl/types";
import { trackFamily } from "./layoutMetrics";

// VexFlow-compatible rendering constants
// VexFlow uses 40pt staff height, 30pt noteheads, 10pt staff-space.
const STAFF_HEIGHT = 40;      // pt (unscaled)
const STAFF_SPACE = 10;       // pt
const NOTEHEAD_FONT = 30;     // pt
const CLEF_WIDTH = 30;        // pt
const TIME_SIG_WIDTH = 30;    // pt
const MARGIN_LEFT = 20;       // pt
const MARGIN_TOP = 10;        // pt
const HEADER_HEIGHT = 50;     // pt (title area)

export function renderScoreToSvg(
  score: NormalizedScore,
  _options?: { staffScale?: number; pageWidth?: number; showTitle?: boolean },
): string {
  const pageWidth = _options?.pageWidth ?? 612;
  const showTitle = _options?.showTitle ?? true;
  const marginLeft = MARGIN_LEFT;
  const systemStart = marginLeft + CLEF_WIDTH + TIME_SIG_WIDTH + 10; // after clef + time sig

  let staffY = MARGIN_TOP;
  if (showTitle && score.header?.title) {
    staffY += HEADER_HEIGHT;
  }

  // Measure layout
  let measures: { m: NormalizedMeasure; x: number; width: number }[] = [];
  let cursorX = systemStart;
  const usableWidth = pageWidth - marginLeft * 2 - CLEF_WIDTH - TIME_SIG_WIDTH;

  for (const measure of score.measures) {
    const slots = Math.max(measure.events.length || 4, 4);
    // VexFlow: measure width ≈ slots * 15pt (approximate, will vary by content)
    const width = Math.max(slots * 15, 60);
    if (cursorX + width > systemStart + usableWidth && measures.length > 0) {
      staffY += STAFF_HEIGHT + STAFF_SPACE * 5;
      cursorX = systemStart;
    }
    measures.push({ m: measure, x: cursorX, width });
    cursorX += width;
  }

  const totalHeight = staffY + STAFF_HEIGHT + STAFF_SPACE * 3;
  const scaledW = pageWidth;
  const scaledH = totalHeight;

  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${scaledW}" height="${scaledH}" viewBox="0 0 ${scaledW} ${scaledH}" font-family="Bravura,Academico" font-size="10pt">`;
  svg += `<defs><style>
    .vf-staff { stroke: #333; stroke-width: 1; fill: none; }
    .vf-notehead { fill: #333; font-size: ${NOTEHEAD_FONT}pt; }
    .vf-stem { stroke: #333; stroke-width: 1.5; }
    .vf-bar { stroke: #333; stroke-width: 1.5; }
    .vf-text { fill: #333; stroke: none; }
  </style></defs>`;

  // Title
  const headerTitle = score.header?.title ?? (score as any).ast?.headers?.title?.value;
  if (showTitle && headerTitle) {
    svg += `<text class="vf-text" font-size="18pt" x="${pageWidth / 2}" y="${MARGIN_TOP + 15}" text-anchor="middle">${esc(String(headerTitle))}</text>`;
  }

  // Tempo
  if (score.header?.tempo) {
    svg += `<text class="vf-text" font-size="14pt" x="${marginLeft + 5}" y="${staffY - STAFF_SPACE}">♩ = ${score.header.tempo}</text>`;
  }

  // Percussion clef
  svg += `<text class="vf-notehead" x="${marginLeft + 5}" y="${staffY + STAFF_HEIGHT / 2 + STAFF_SPACE * 0.5}">\u{E069}</text>`;

  // Time signature
  const beats = score.header?.timeSignature?.beats ?? 4;
  const beatUnit = score.header?.timeSignature?.beatUnit ?? 4;
  svg += `<text class="vf-notehead" x="${marginLeft + CLEF_WIDTH + 5}" y="${staffY + STAFF_SPACE * 1.4}">${numGlyph(beats)}</text>`;
  svg += `<text class="vf-notehead" x="${marginLeft + CLEF_WIDTH + 5}" y="${staffY + STAFF_SPACE * 3.4}">${numGlyph(beatUnit)}</text>`;

  // Staff lines and measures
  for (const { m, x, width } of measures) {
    svg += renderStaff(x, staffY, width);

    // Barline at measure start
    svg += renderBarline(x, staffY, m.barline, true);

    // Content
    svg += renderNotes(m, x, staffY, width);
  }

  // Final barline
  const lastM = measures[measures.length - 1];
  if (lastM) {
    svg += renderBarline(lastM.x + lastM.width, staffY, "final", false);
  }

  svg += "</svg>";
  return svg;
}

// ── Staff ────────────────────────────────────────────────────────

function renderStaff(x: number, y: number, width: number): string {
  let s = "";
  for (let i = 0; i < 5; i++) {
    const lineY = y + STAFF_SPACE + i * STAFF_SPACE;
    s += `<line x1="${x}" y1="${lineY}" x2="${x + width}" y2="${lineY}" class="vf-staff"/>`;
  }
  return s;
}

// ── Barlines ─────────────────────────────────────────────────────

function renderBarline(x: number, y: number, type?: string | null, _isRight?: boolean): string {
  const y1 = y + STAFF_SPACE;
  const y2 = y + STAFF_SPACE * 5;

  switch (type) {
    case "double": case "final":
      return `<line x1="${x}" y1="${y1}" x2="${x}" y2="${y2}" class="vf-bar"/>`
        + `<line x1="${x + 3}" y1="${y1}" x2="${x + 3}" y2="${y2}" class="vf-bar" stroke-width="3"/>`;
    case "repeat-start":
      return `<line x1="${x + 3}" y1="${y1}" x2="${x + 3}" y2="${y2}" class="vf-bar"/>`
        + `<line x1="${x + 8}" y1="${y1}" x2="${x + 8}" y2="${y2}" class="vf-bar" stroke-width="3"/>`
        + `<text x="${x + 14}" y="${y + STAFF_SPACE * 3.5}" font-size="12pt" class="vf-text" text-anchor="middle">:</text>`;
    case "repeat-end":
      return `<text x="${x + 2}" y="${y + STAFF_SPACE * 3.5}" font-size="12pt" class="vf-text" text-anchor="middle">:</text>`
        + `<line x1="${x + 8}" y1="${y1}" x2="${x + 8}" y2="${y2}" class="vf-bar" stroke-width="3"/>`
        + `<line x1="${x + 13}" y1="${y1}" x2="${x + 13}" y2="${y2}" class="vf-bar"/>`;
    case "repeat-both":
      return renderBarline(x, y, "repeat-end")
        + renderBarline(x + 22, y, "repeat-start");
    default:
      return `<line x1="${x}" y1="${y1}" x2="${x}" y2="${y2}" class="vf-bar"/>`;
  }
}

// ── Notes ────────────────────────────────────────────────────────

function renderNotes(m: NormalizedMeasure, measureX: number, staffY: number, _w: number): string {
  let s = "";
  let px = measureX + 12;

  // Measure repeat
  if ((m as any).measureRepeat) {
    const slashes = (m as any).measureRepeat.slashes ?? 1;
    const label = slashes === 2 ? "%%" : "%";
    s += `<text x="${measureX + _w / 2}" y="${staffY + STAFF_SPACE * 3.5}" text-anchor="middle" font-size="20pt" class="vf-text">${label}</text>`;
    return s;
  }

  // Multi-rest
  if ((m as any).multiRest) {
    const count = (m as any).multiRest.count ?? 2;
    const midX = measureX + _w / 2;
    s += `<line x1="${measureX + 10}" y1="${staffY + STAFF_SPACE * 3}" x2="${measureX + _w - 10}" y2="${staffY + STAFF_SPACE * 3}" class="vf-staff" stroke-width="3"/>`;
    s += `<text x="${midX + 4}" y="${staffY + STAFF_SPACE * 2.5}" font-size="14pt" class="vf-text">${count}</text>`;
    return s;
  }

  // Generate events if empty (fill with rests for empty measures)
  const events = m.events.length > 0 ? m.events : (m as any)._fillEvents || [];

  for (const ev of events) {
    const noteX = px + 6;
    const isRest = ev.kind === "rest";

    if (isRest) {
      // Rest glyph from SMuFL
      s += `<text class="vf-notehead" x="${noteX}" y="${staffY + STAFF_SPACE * 3.5}">${restGlyph(ev)}</text>`;
    } else {
      const y = noteY(ev.track, staffY);
      const glyph = noteGlyph(ev);

      // Notehead
      const ny = y + STAFF_SPACE * 0.5;
      s += `<text class="vf-notehead" x="${noteX - NOTEHEAD_FONT * 0.22}" y="${ny}">${glyph}</text>`;

      // Stem
      const up = ev.voice !== 2;
      const stemTop = ny - STAFF_SPACE * 3;
      const stemBot = ny;
      s += `<line x1="${noteX + NOTEHEAD_FONT * 0.3}" y1="${up ? stemTop : stemBot}" x2="${noteX + NOTEHEAD_FONT * 0.3}" y2="${up ? stemBot : stemTop}" class="vf-stem"/>`;

      // Accent modifier
      if (ev.modifiers?.includes("accent")) {
        s += `<text class="vf-text" font-size="16pt" x="${noteX}" y="${ny - STAFF_SPACE * 1.2}" text-anchor="middle">></text>`;
      }
      // Ghost modifier
      if (ev.modifiers?.includes("ghost")) {
        s += `<text class="vf-text" font-size="14pt" x="${noteX - 5}" y="${ny}">(</text>`;
        s += `<text class="vf-text" font-size="14pt" x="${noteX + 5}" y="${ny}">)</text>`;
      }

      // Beams
      if (ev.beam && ev.beam !== "none") {
        // Draw a beam line between consecutive beam notes (simplified)
        s += `<line x1="${noteX - 2}" y1="${up ? stemTop : stemTop - 2}" x2="${noteX + 16}" y2="${up ? stemTop : stemTop - 2}" class="vf-stem" stroke-width="4"/>`;
      }
    }

    px += 20; // advance position
  }
  return s;
}

// ── Note Y (VexFlow positions) ───────────────────────────────────

function noteY(track: string, staffY: number): number {
  // VexFlow: staffY + STAFF_SPACE is the top line.
  // Note positions in staff-space units (0 = top line, positive = downward):
  const pos: Record<string, number> = {
    HH: 0, HF: -1, SPL: -1, CHN: -1, ST: -1,
    RC: 1, RC2: 1, C: 2, C2: 2,
    T1: 3, T2: 4, SD: 4, T3: 5, T4: 6,
    BD: 8, BD2: 8,
    CB: 0, WB: -1, CL: 0,
  };
  const ssPos = pos[track] ?? 4;
  return staffY + STAFF_SPACE + ssPos * (STAFF_SPACE / 2);
}

// ── Glyphs ───────────────────────────────────────────────────────

function noteGlyph(ev: NormalizedEvent): string {
  const family = trackFamily(ev.track);
  if (family === "cymbal") return "\u{E0A9}"; // X notehead
  for (const m of ev.modifiers || []) {
    if (m === "open") return "\u{E0B3}";
    if (m === "cross") return "\u{E0A9}";
    if (m === "bell") return "\u{E0DB}";
    if (m === "rim") return "\u{E0CE}";
  }
  return "\u{E0A4}"; // standard black notehead
}

function restGlyph(ev: NormalizedEvent): string {
  const d = ev.duration?.denominator ?? 4;
  if (d >= 32) return "\u{E4E7}";
  if (d >= 16) return "\u{E4E6}";
  if (d >= 8)  return "\u{E4E5}";
  if (d >= 4)  return "\u{E4E4}";
  return "\u{E4E3}";
}

function numGlyph(n: number): string {
  const map: Record<number, string> = {
    0: "\u{E080}", 1: "\u{E081}", 2: "\u{E082}", 3: "\u{E083}", 4: "\u{E084}",
    5: "\u{E085}", 6: "\u{E086}", 7: "\u{E087}", 8: "\u{E088}", 9: "\u{E089}",
  };
  return map[n] ?? String(n);
}

function esc(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
