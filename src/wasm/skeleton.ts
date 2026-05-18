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
import { initParserWasmBrowser } from "./parser_wasm_browser";
import {
  isParserRuntimeReady,
  parseWithParserRuntime,
} from "./parser_runtime";

// ── Public API ───────────────────────────────────────────────────

/** Async: initializes WASM and parses. For first-time use. */
export async function parseDocumentSkeletonFromWasm(
  source: string,
): Promise<DocumentSkeleton> {
  await initParserWasmBrowser();
  return parseDocumentSkeletonFromWasmSync(source);
}

/** Sync: assumes WASM is already initialized via initWasm(). */
export function parseDocumentSkeletonFromWasmSync(
  source: string,
): DocumentSkeleton {
  if (!isParserRuntimeReady()) {
    throw new Error("WASM parser not ready. Call initWasm() first.");
  }
  const raw = parseWithParserRuntime(source) as WasmDocument | WasmError[];
  if (isWasmErrorArray(raw)) {
    return {
      headers: {
        tempo: { field: "tempo", value: 120, line: 0 },
        time: { field: "time", beats: 4, beatUnit: 4, line: 0 },
        grouping: { field: "grouping", values: [1], line: 0 },
      },
      paragraphs: [],
      errors: raw.map((e) => ({
        line: e.line,
        column: e.column,
        message: e.message,
      })),
    };
  }
  return adaptToSkeleton(source, raw);
}

const MEASURE_METADATA_KINDS = new Set<WasmToken["kind"]>([
  "measureRepeat",
  "multiRest",
  "inlineRepeat",
]);

// ── WASM output types ────────────────────────────────────────────

interface WasmDocument {
  headers: WasmHeaders;
  paragraphs: WasmParagraph[];
  errors: WasmError[];
}

function isWasmErrorArray(raw: unknown): raw is WasmError[] {
  return Array.isArray(raw);
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
  barlineLocation?: WasmSourceLocation;
  closingBarline?: WasmBarline;
  closingBarlineLocation?: WasmSourceLocation;
  tokens: WasmToken[];
}

interface WasmSourceLocation {
  line: number;
  column: number;
  offset: number;
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
  source: string,
  wasm: WasmDocument,
): DocumentSkeleton {
  const time: [number, number] = wasm.headers.time ?? [4, 4];
  const sourceLines = source.split(/\r?\n/);
  const sourceTrackLineNumbers = collectTrackSourceLineNumbers(source);
  let sourceTrackLineIndex = 0;
  const adapterErrors: ParseError[] = [];
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

  for (const wp of wasm.paragraphs) {
    const paragraphLineNumbers = sourceTrackLineNumbers.slice(
      sourceTrackLineIndex,
      sourceTrackLineIndex + wp.lines.length,
    );
    const startLine = paragraphLineNumbers[0] ?? 1;
    const lines: ParsedTrackLine[] = [];

    for (const [lineOffset, wl] of wp.lines.entries()) {
      const lineNumber = paragraphLineNumbers[lineOffset] ?? startLine;
      const rawLine = sourceLines[lineNumber - 1] ?? "";
      const measures: ParsedMeasure[] = [];

      for (const wm of wl.measures) {
        const openingLocation = wasmLocationToSourceLocation(wm.barlineLocation);
        const previousMeasure = measures[measures.length - 1];
        if (previousMeasure) {
          switch (wm.barline.type) {
            case "||":
              previousMeasure.barline = "double";
              break;
            case "|.":
              previousMeasure.voltaTerminator = true;
              break;
            case "||.":
              previousMeasure.barline = "double";
              previousMeasure.voltaTerminator = true;
              break;
            case "|:.":
              previousMeasure.voltaTerminator = true;
              break;
          }
        }

        // Extract measure-level metadata and filter non-display tokens
        let measureRepeatSlashes: number | undefined;
        let multiRestCount: number | undefined;
        let inlineRepeatTimes: number | undefined;
        let startNav: ParsedMeasure["startNav"];
        let endNav: ParsedMeasure["endNav"];
        const displayTokens: WasmToken[] = [];
        const nonNavTokenTotal = wm.tokens.filter(
          (t) => t.kind !== "navMarker" && t.kind !== "navJump" && !MEASURE_METADATA_KINDS.has(t.kind),
        ).length;
        let nonNavSeen = 0;

        for (const t of wm.tokens) {
          if (t.kind === "measureRepeat") {
            measureRepeatSlashes = t.count;
          } else if (t.kind === "multiRest") {
            multiRestCount = t.count;
          } else if (t.kind === "inlineRepeat") {
            inlineRepeatTimes = t.times;
          } else if (t.kind === "navMarker") {
            const pureNavigationMeasure = nonNavTokenTotal === 0;
            const nonNavAfter = nonNavTokenTotal - nonNavSeen;
            const column = rawLine.indexOf(`@${t.name}`) + 1;
            if (startNav !== undefined) {
              adapterErrors.push({
                line: lineNumber,
                column: column > 0 ? column : 1,
                message: "Measure contains multiple start-side navigation markers",
              });
              continue;
            }
            if (t.name === "coda") {
              if (!pureNavigationMeasure && nonNavSeen !== 0) {
                adapterErrors.push({
                  line: lineNumber,
                  column: column > 0 ? column : 1,
                  message: "`@coda` may appear only at the beginning of a measure",
                });
                continue;
              }
              startNav = { kind: "coda", anchor: "left-edge" };
              continue;
            }
            if (!pureNavigationMeasure && nonNavAfter === 0) {
              adapterErrors.push({
                line: lineNumber,
                column: column > 0 ? column : 1,
                message: "`@segno` may not appear at the end of a measure",
              });
              continue;
            }
            startNav =
              pureNavigationMeasure || nonNavSeen === 0
                ? { kind: "segno", anchor: "left-edge" }
                : { kind: "segno", anchor: { tokenAfter: nonNavSeen } };
          } else if (t.kind === "navJump") {
            const pureNavigationMeasure = nonNavTokenTotal === 0;
            const nonNavAfter = nonNavTokenTotal - nonNavSeen;
            const column = rawLine.indexOf(`@${t.name}`) + 1;
            if (endNav !== undefined) {
              adapterErrors.push({
                line: lineNumber,
                column: column > 0 ? column : 1,
                message: "Measure contains multiple end-side navigation instructions",
              });
              continue;
            }
            if (t.name === "to-coda") {
              if (!pureNavigationMeasure && nonNavSeen === 0) {
                adapterErrors.push({
                  line: lineNumber,
                  column: column > 0 ? column : 1,
                  message: "`@to-coda` may not appear at the beginning of a measure",
                });
                continue;
              }
              endNav =
                pureNavigationMeasure || nonNavAfter === 0
                  ? { kind: "to-coda", anchor: "right-edge" }
                  : { kind: "to-coda", anchor: { tokenBefore: nonNavSeen - 1 } };
              continue;
            }
            if (!pureNavigationMeasure && nonNavAfter !== 0) {
              adapterErrors.push({
                line: lineNumber,
                column: column > 0 ? column : 1,
                message: `\`@${t.name}\` may appear only at the end of a measure`,
              });
              continue;
            }
            endNav = { kind: t.name as Exclude<NonNullable<ParsedMeasure["endNav"]>["kind"], "to-coda">, anchor: "right-edge" };
          } else {
            displayTokens.push(t);
            if (!MEASURE_METADATA_KINDS.has(t.kind)) {
              nonNavSeen += 1;
            }
          }
        }

        const tokens: TokenGlyph[] = displayTokens.map(adaptToken);
        const content = displayTokens.map(t => tokenToString(t)).join(" ") || "";

        // Barline metadata
        let barline: BarlineType | undefined;
        let repeatStart = false;
        let repeatEnd = false;
        let repeatEndLocation: ParsedMeasure["repeatEndLocation"];
        let voltaIndices: number[] | undefined;
        let voltaTerminator = false;

        const bl = wm.barline;
        switch (bl.type) {
          case "|":
            break;
          case "||":
            break;
          case "|:":
            repeatStart = true;
            break;
          case ":|":
            repeatEnd = true;
            repeatEndLocation = openingLocation;
            break;
          case "|.":
            break;
          case "||.":
            break;
          case "|:.":
            repeatStart = true;
            break;
          case "volta":
            voltaIndices = bl.numbers;
            if (bl.prefix === "|:") repeatStart = true;
            if (bl.prefix === ":|") {
              repeatEnd = true;
              repeatEndLocation = openingLocation;
            }
            break;
        }

        const closing = wm.closingBarline;
        if (closing) {
          const closingLocation = wasmLocationToSourceLocation(wm.closingBarlineLocation);
          switch (closing.type) {
            case "||":
              barline = "double";
              break;
            case ":|":
              repeatEnd = true;
              repeatEndLocation = closingLocation;
              break;
            case "|.":
              voltaTerminator = true;
              break;
            case "||.":
              barline = "double";
              voltaTerminator = true;
              break;
          }
        }

        const parsedMeasure: ParsedMeasure = {
          content,
          tokens,
          repeatStart,
          repeatEnd,
          repeatEndLocation,
          repeatTimes: repeatEnd ? inlineRepeatTimes : undefined,
          repeatCount: inlineRepeatTimes && inlineRepeatTimes > 0 ? inlineRepeatTimes : undefined,
          barline,
          voltaIndices,
          voltaTerminator,
          measureRepeatSlashes,
          multiRestCount,
          startNav,
          endNav,
        };

        if (inlineRepeatTimes === undefined || displayTokens.length === 0) {
          measures.push(parsedMeasure);
          continue;
        }

        if (inlineRepeatTimes < 1) {
          adapterErrors.push({
            line: lineNumber,
            column: 1,
            message: "Repeat count must be at least 1",
          });
          measures.push(parsedMeasure);
          continue;
        }

        for (let i = 0; i < inlineRepeatTimes; i += 1) {
          measures.push({
            ...parsedMeasure,
            repeatStart: i === 0 ? parsedMeasure.repeatStart : false,
            repeatEnd: i === inlineRepeatTimes - 1 ? parsedMeasure.repeatEnd : false,
            repeatEndLocation: i === inlineRepeatTimes - 1 ? parsedMeasure.repeatEndLocation : undefined,
            repeatTimes: i === inlineRepeatTimes - 1 ? parsedMeasure.repeatTimes : undefined,
            barline: i === inlineRepeatTimes - 1 ? parsedMeasure.barline : undefined,
            startNav: i === 0 ? parsedMeasure.startNav : undefined,
            endNav: i === inlineRepeatTimes - 1 ? parsedMeasure.endNav : undefined,
            voltaIndices: i === 0 ? parsedMeasure.voltaIndices : undefined,
            voltaTerminator: i === inlineRepeatTimes - 1 ? parsedMeasure.voltaTerminator : undefined,
            measureRepeatSlashes: i === 0 ? parsedMeasure.measureRepeatSlashes : undefined,
            multiRestCount: i === 0 ? parsedMeasure.multiRestCount : undefined,
          });
        }
      }

      for (let i = 0; i < measures.length - 1; i += 1) {
        const current = measures[i];
        const next = measures[i + 1];
        if (
          current &&
          next &&
          current.repeatEnd !== true &&
          current.voltaIndices?.length &&
          next.voltaIndices?.length &&
          current.voltaIndices.join(",") !== next.voltaIndices.join(",")
        ) {
          current.repeatEnd = true;
        }
      }

      const lastMeasure = measures[measures.length - 1];
      const trimmedLine = rawLine.trim();
      if (lastMeasure) {
        if (trimmedLine.endsWith("||.")) {
          lastMeasure.barline = "double";
          lastMeasure.voltaTerminator = true;
        } else if (trimmedLine.endsWith("||")) {
          lastMeasure.barline = "double";
        } else if (trimmedLine.endsWith("|.")) {
          lastMeasure.voltaTerminator = true;
        } else if (trimmedLine.endsWith(":|")) {
          lastMeasure.repeatEnd = true;
          lastMeasure.repeatEndLocation ??= {
            line: lineNumber,
            column: rawLine.lastIndexOf(":|") + 1 || 1,
            offset: 0,
          };
        }
      }

      lines.push({
        track: (wl.track ?? "ANONYMOUS") as TrackName | "ANONYMOUS",
        lineNumber,
        measures,
        source: {
          kind: "content",
          lineNumber,
          raw: "",
          content: rawLine,
          startOffset: 0,
        },
      });
    }

    sourceTrackLineIndex += wp.lines.length;

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
  errors.push(...adapterErrors);

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
      return {
        kind: "combined",
        items: token.hits.map(adaptToken),
      };
    }
    case "group": {
      const count = token.items.filter(
        (item) => item.kind !== "crescendo" && item.kind !== "decrescendo" && item.kind !== "hairpinEnd",
      ).length;
      return {
        kind: "group",
        count,
        span: token.n ?? 1,
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

function wasmLocationToSourceLocation(location: WasmSourceLocation | undefined): ParsedMeasure["repeatEndLocation"] {
  if (!location) return undefined;
  return {
    line: location.line,
    column: location.column,
    offset: location.offset,
  };
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

function collectTrackSourceLineNumbers(source: string): number[] {
  const lineNumbers: number[] = [];
  const lines = source.split(/\r?\n/);

  for (let index = 0; index < lines.length; index += 1) {
    const trimmed = lines[index]?.trim() ?? "";
    if (!trimmed || trimmed.startsWith("#")) {
      continue;
    }
    if (/^(title|subtitle|composer|tempo|time|grouping|note|divisions)\b/.test(trimmed)) {
      continue;
    }
    if (trimmed.includes("|")) {
      lineNumbers.push(index + 1);
    }
  }

  return lineNumbers;
}
