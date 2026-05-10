import type {
  DocumentSkeleton,
  TrackParagraph,
  ParsedTrackLine,
  ParsedMeasure,
  TokenGlyph,
  ParsedHeaders,
  ParseError,
  TrackName,
  Modifier,
  BasicGlyph,
  BarlineType,
} from "../dsl/types";
import { initWasm, parse as wasmParse, isWasmReady } from "./drummark_wasm";

// ── Public API ───────────────────────────────────────────────────

/** Async: initializes WASM and parses. For first-time use. */
export async function parseDocumentSkeletonFromWasm(
  source: string,
): Promise<DocumentSkeleton> {
  await initWasm();
  return parseDocumentSkeletonFromWasmSync(source);
}

/** Sync: assumes WASM is already initialized via initWasm(). */
export function parseDocumentSkeletonFromWasmSync(
  source: string,
): DocumentSkeleton {
  if (!isWasmReady()) {
    throw new Error("WASM parser not ready. Call initWasm() first.");
  }
  const raw = wasmParse(source) as WasmDocument;
  return adaptToSkeleton(source, raw);
}

// ── WASM output types ────────────────────────────────────────────

interface WasmDocument {
  headers: WasmHeaders;
  paragraphs: WasmParagraph[];
  errors: WasmError[];
}

interface WasmHeaders {
  title?: string;
  subtitle?: string;
  composer?: string;
  tempo?: number;
  time?: [number, number];
  grouping?: number[];
  note?: [number, number];
  divisions?: number;
}

interface WasmParagraph {
  note?: [number, number];
  lines: WasmTrackLine[];
}

interface WasmTrackLine {
  track?: string;
  measures: WasmMeasure[];
}

interface WasmMeasure {
  barline: WasmBarline;
  tokens: WasmToken[];
}

type WasmBarline =
  | { type: "|" }
  | { type: "||" }
  | { type: "|:" }
  | { type: ":|" }
  | { type: "|." }
  | { type: "||." }
  | { type: "|:." }
  | { type: "volta"; prefix: string; numbers: number[] };

type WasmToken =
  | { kind: "basic"; glyph: string; dots?: number; halves?: number; stars?: number; modifiers?: string[] }
  | { kind: "summoned"; track: string; glyph: string; dots?: number; halves?: number; stars?: number; modifiers?: string[] }
  | { kind: "routedBraced"; track: string; content: WasmToken[] }
  | { kind: "inlineBraced"; content: WasmToken[] }
  | { kind: "group"; n?: number; items: WasmToken[]; modifiers?: string[] }
  | { kind: "combinedHit"; hits: WasmToken[] }
  | { kind: "measureRepeat"; count: number }
  | { kind: "multiRest"; count: number }
  | { kind: "inlineRepeat"; times: number }
  | { kind: "crescendo" }
  | { kind: "decrescendo" }
  | { kind: "hairpinEnd" }
  | { kind: "navMarker"; name: string }
  | { kind: "navJump"; name: string };

interface WasmError {
  line: number;
  column: number;
  message: string;
}

// ── Adapter ──────────────────────────────────────────────────────

function adaptToSkeleton(
  _source: string,
  wasm: WasmDocument,
): DocumentSkeleton {
  const time: [number, number] = wasm.headers.time ?? [4, 4];
  const headers: ParsedHeaders = {
    title: wasm.headers.title
      ? { field: "title", value: wasm.headers.title, line: 0 }
      : undefined,
    subtitle: wasm.headers.subtitle
      ? { field: "subtitle", value: wasm.headers.subtitle, line: 0 }
      : undefined,
    composer: wasm.headers.composer
      ? { field: "composer", value: wasm.headers.composer, line: 0 }
      : undefined,
    tempo: {
      field: "tempo",
      value: wasm.headers.tempo ?? 120,
      line: 0,
    },
    time: {
      field: "time",
      beats: time[0],
      beatUnit: time[1],
      line: 0,
    },
    grouping: {
      field: "grouping",
      values: wasm.headers.grouping ?? inferGrouping(time[0], time[1]),
      line: 0,
    },
    note: wasm.headers.note
      ? { field: "note", value: wasm.headers.note[1], line: 0 }
      : undefined,
    divisions: wasm.headers.divisions
      ? { field: "divisions", value: wasm.headers.divisions, line: 0 }
      : undefined,
  };

  const paragraphs: TrackParagraph[] = [];
  let lineCounter = 1;

  for (const wp of wasm.paragraphs) {
    const startLine = lineCounter;
    const lines: ParsedTrackLine[] = [];

    for (const wl of wp.lines) {
      const measures: ParsedMeasure[] = [];

      for (const wm of wl.measures) {
        const tokens: TokenGlyph[] = wm.tokens.map(adaptToken);

        // Build content string for measure
        const content = wm.tokens.map(t => tokenToString(t)).join(" ") || "";

        // Barline metadata
        let barline: BarlineType | undefined;
        let repeatStart = false;
        let repeatEnd = false;
        let voltaIndices: number[] | undefined;
        let voltaTerminator = false;
        let measureRepeatSlashes: number | undefined;
        let multiRestCount: number | undefined;

        const bl = wm.barline;
        switch (bl.type) {
          case "|":
            barline = "regular";
            break;
          case "||":
            barline = "double";
            break;
          case "|:":
            barline = "regular";
            repeatStart = true;
            break;
          case ":|":
            barline = "regular";
            repeatEnd = true;
            break;
          case "|.":
            barline = "regular";
            voltaTerminator = true;
            break;
          case "||.":
            barline = "double";
            voltaTerminator = true;
            break;
          case "|:.":
            barline = "regular";
            repeatStart = true;
            voltaTerminator = true;
            break;
          case "volta":
            barline = bl.prefix === "|:" ? "regular" : "regular";
            voltaIndices = bl.numbers;
            if (bl.prefix === "|:") repeatStart = true;
            break;
        }

        // Extract measure-repeat and multi-rest from tokens
        for (const t of wm.tokens) {
          if (t.kind === "measureRepeat") {
            measureRepeatSlashes = t.count;
          }
          if (t.kind === "multiRest") {
            multiRestCount = t.count;
          }
        }

        measures.push({
          content,
          tokens,
          repeatStart,
          repeatEnd,
          barline,
          voltaIndices,
          voltaTerminator,
          measureRepeatSlashes,
          multiRestCount,
        });
      }

      lines.push({
        track: (wl.track ?? "ANONYMOUS") as TrackName | "ANONYMOUS",
        lineNumber: lineCounter++,
        measures,
        source: {
          kind: "content",
          lineNumber: lineCounter - 1,
          raw: "",
          content: "",
          startOffset: 0,
        },
      });
    }

    paragraphs.push({
      startLine,
      lines,
      noteValue: wp.note ? wp.note[1] : undefined,
    });
  }

  const errors: ParseError[] = wasm.errors.map((e) => ({
    line: e.line,
    column: e.column,
    message: e.message,
  }));

  return { headers, paragraphs, errors };
}

// ── Token Adapters ───────────────────────────────────────────────

function adaptToken(token: WasmToken): TokenGlyph {
  switch (token.kind) {
    case "basic":
    case "summoned": {
      const glyph = (token as { glyph: string }).glyph as BasicGlyph;
      const dots = token.dots ?? 0;
      const halves = token.halves ?? 0;
      const stars = token.stars ?? 0;
      const modifiers = (token.modifiers ?? []) as Modifier[];
      const trackOverride = (token as { track: string }).track as TrackName;
      return { kind: "basic", value: glyph, dots, halves, stars, modifiers, trackOverride };
    }
    case "combinedHit": {
      return { kind: "combined", items: token.hits.map(adaptToken) };
    }
    case "group": {
      return {
        kind: "group",
        count: token.n ?? 0,
        span: token.items.length,
        items: token.items.map(adaptToken),
        modifiers: (token.modifiers ?? []) as Modifier[],
      };
    }
    case "routedBraced": {
      const trackName = token.track as TrackName;
      return {
        kind: "braced",
        track: trackName,
        items: token.content.map(adaptToken),
      };
    }
    case "inlineBraced": {
      // Inline braced blocks don't have a track override; emit as braced with ANONYMOUS track
      return { kind: "braced", track: "SD" as TrackName, items: token.content.map(adaptToken) };
    }
    case "crescendo":
      return { kind: "crescendo_start" };
    case "decrescendo":
      return { kind: "decrescendo_start" };
    case "hairpinEnd":
      return { kind: "hairpin_end" };
    // Measure-repeat, multi-rest, inline-repeat, nav markers:
    // These are measure-level metadata, not tokens.
    // They get extracted in the measure loop above.
    // Return a placeholder basic token for now.
    default:
      return { kind: "basic", value: "-" as BasicGlyph, dots: 0, halves: 0, stars: 0, modifiers: [] };
  }
}

function tokenToString(t: WasmToken): string {
  switch (t.kind) {
    case "basic":
      return t.glyph;
    case "summoned":
      return t.glyph;
    case "combinedHit":
      return t.hits.map((h) => (h.kind === "basic" || h.kind === "summoned" ? h.glyph : "?")).join("+");
    case "group":
      return `[${t.items.map(tokenToString).join(" ")}]`;
    case "crescendo":
      return "<";
    case "decrescendo":
      return ">";
    case "hairpinEnd":
      return "!";
    default:
      return "?";
  }
}

// ── Grouping Inference ───────────────────────────────────────────

function inferGrouping(beats: number, _beatUnit: number): number[] {
  if (beats === 1) return [1];
  if (beats === 2) return [1, 1];
  if (beats === 3) return [1, 1, 1];
  if (beats === 4) return [2, 2];
  if (beats === 5) return [3, 2];
  if (beats === 6) return [3, 3];
  if (beats === 7) return [3, 2, 2];
  if (beats === 8) return [4, 4];
  if (beats === 9) return [3, 3, 3];
  if (beats === 12) return [3, 3, 3, 3];
  return Array.from({ length: beats }, () => 1);
}
