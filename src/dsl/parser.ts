import { inferGrouping } from "./grouping";
import {
  HEADER_FIELDS,
  MODIFIERS,
  TRACKS,
  type EndNavKind,
  type DocumentSkeleton,
  type MeasureToken,
  type ParseError,
  type ParsedEndNav,
  type ParsedHeaders,
  type ParsedMeasure,
  type ParsedStartNav,
  type ParsedTrackLine,
  type PreprocessedLine,
  type StartNavKind,
  type TrackParagraph,
  type TrackName,
  type SourceTrackName,
  type Modifier,
  type BasicGlyph,
  type MetadataHeader,
  type TokenGlyph,
} from "./types";
import { preprocessSource } from "./preprocess";

type HeaderAccumulator = Partial<ParsedHeaders>;
type RawMeasure = Omit<ParsedTrackLine["measures"][number], "tokens"> & { tokens?: MeasureToken[] };
const SORTED_TRACK_NAMES = [...TRACKS].sort((left, right) => right.length - left.length);
const START_NAVS = ["@segno", "@coda"] as const;
const END_NAVS = ["@fine", "@dc", "@ds", "@dc-al-fine", "@dc-al-coda", "@ds-al-fine", "@ds-al-coda", "@to-coda"] as const;
const DEPRECATED_NAVS = ["@da-capo", "@dal-segno"] as const;
const BASIC_GLYPHS = [
  "-",
  "x",
  "X",
  "d",
  "D",
  "p",
  "P",
  "R",
  "L",
  "o",
  "O",
  "c2",
  "C2",
  "c",
  "C",
  "s",
  "S",
  "b2",
  "B2",
  "b",
  "B",
  "r2",
  "R2",
  "r",
  "t1",
  "T1",
  "t2",
  "T2",
  "t3",
  "T3",
  "t4",
  "T4",
  "g",
  "G",
  "spl",
  "SPL",
  "chn",
  "CHN",
  "cb",
  "CB",
  "wb",
  "WB",
  "cl",
  "CL",
] as const;
const MULTI_CHAR_GLYPHS = BASIC_GLYPHS.filter((glyph) => glyph.length > 1).sort((left, right) => right.length - left.length);

const SUPPORTED_BEAT_UNITS = new Set([2, 4, 8, 16]);

function isPowerOfTwo(n: number): boolean {
  return n > 0 && (n & (n - 1)) === 0;
}

function parseNoteValue(value: string): number | null {
  const match = value.match(/^1\s*\/\s*(\d+)$/);
  if (!match || !match[1]) return null;
  const n = parseInt(match[1], 10);
  if (isPowerOfTwo(n) && n <= 128) return n;
  return null;
}

function parsePositiveInteger(value: string): number | null {
  if (!/^\d+$/.test(value)) {
    return null;
  }

  const parsed = Number(value);
  return parsed > 0 ? parsed : null;
}

export { inferGrouping };

function parseGroupingValue(value: string): number[] | null {
  if (!/^\d+(?:\s*\+\s*\d+)*$/.test(value)) {
    return null;
  }

  const values = value.split("+").map(s => Number(s.trim()));
  return values.every((item) => item > 0) ? values : null;
}

function isMetadataField(field: string): field is MetadataHeader["field"] {
  return field === "title" || field === "subtitle" || field === "composer";
}

function unquoteString(raw: string): { value: string; ok: boolean } {
  const q = raw[0];
  if (q !== '"' && q !== "'") return { value: raw, ok: true };
  const close = raw.indexOf(q, 1);
  if (close === -1) return { value: raw, ok: false };
  return { value: raw.slice(1, close), ok: true };
}

function parseHeaderLine(line: PreprocessedLine, headers: HeaderAccumulator, errors: ParseError[]): boolean {
  const match = line.content.match(/^(\S+)(?:\s+(.*))?$/);
  const field = match?.[1] ?? "";
  let rawValue = match?.[2] ?? "";
  let value = rawValue;

  if (!field || !HEADER_FIELDS.includes(field as (typeof HEADER_FIELDS)[number])) {
    return false;
  }

  // Unquote metadata values
  if (isMetadataField(field) && rawValue) {
    const result = unquoteString(rawValue);
    if (!result.ok) {
      errors.push({
        line: line.lineNumber,
        column: line.raw.indexOf(rawValue) + 1,
        message: `Unclosed quote in header \`${field}\``,
      });
      return true;
    }
    value = result.value;
  }

  if (!value) {
    errors.push({
      line: line.lineNumber,
      column: 1,
      message: isMetadataField(field)
        ? `Header \`${field}\` expects non-empty text`
        : `Header \`${field}\` expects a single value`,
    });
    return true;
  }

  if (!isMetadataField(field) && field !== "grouping" && field !== "time" && /\s/.test(value)) {
    errors.push({
      line: line.lineNumber,
      column: 1,
      message: `Header \`${field}\` expects a single value`,
    });
    return true;
  }

  switch (field) {
    case "title":
    case "subtitle":
    case "composer": {
      if (headers[field]) {
        errors.push({
          line: line.lineNumber,
          column: 1,
          message: `Duplicate header \`${field}\``,
        });
        return true;
      }

      headers[field] = { field, value, line: line.lineNumber };
      return true;
    }
    case "tempo": {
      if (headers.tempo) {
        errors.push({
          line: line.lineNumber,
          column: 1,
          message: "Duplicate header `tempo`",
        });
        return true;
      }

      const parsed = parsePositiveInteger(value);

      if (parsed === null) {
        errors.push({
          line: line.lineNumber,
          column: line.raw.indexOf(value) + 1,
          message: "Tempo must be a positive integer",
        });
        return true;
      }

      headers.tempo = { field: "tempo", value: parsed, line: line.lineNumber };
      return true;
    }
    case "time": {
      if (headers.time) {
        errors.push({
          line: line.lineNumber,
          column: 1,
          message: "Duplicate header `time`",
        });
        return true;
      }

      const match = value.match(/^(\d+)\s*\/\s*(\d+)$/);

      if (!match) {
        errors.push({
          line: line.lineNumber,
          column: line.raw.indexOf(value) + 1,
          message: "Time must use the form beats/beatUnit",
        });
        return true;
      }

      const beats = Number(match[1]);
      const beatUnit = Number(match[2]);

      if (beats <= 0 || beatUnit <= 0) {
        errors.push({
          line: line.lineNumber,
          column: line.raw.indexOf(value) + 1,
          message: "Time values must be positive integers",
        });
        return true;
      }

      if (!SUPPORTED_BEAT_UNITS.has(beatUnit)) {
        errors.push({
          line: line.lineNumber,
          column: line.raw.indexOf(value) + 1,
          message: "Beat unit must be one of 2, 4, 8, or 16",
        });
        return true;
      }

      headers.time = { field: "time", beats, beatUnit, line: line.lineNumber };
      return true;
    }
    case "divisions": {
      if (headers.divisions) {
        errors.push({
          line: line.lineNumber,
          column: 1,
          message: "Duplicate header `divisions`",
        });
        return true;
      }

      const parsed = parsePositiveInteger(value);

      if (parsed === null) {
        errors.push({
          line: line.lineNumber,
          column: line.raw.indexOf(value) + 1,
          message: "Divisions must be a positive integer",
        });
        return true;
      }

      headers.divisions = {
        field: "divisions",
        value: parsed,
        line: line.lineNumber,
      };
      return true;
    }
    case "note": {
      if (headers.note) {
        errors.push({
          line: line.lineNumber,
          column: 1,
          message: "Duplicate header `note`",
        });
        return true;
      }

      const parsed = parseNoteValue(value);

      if (parsed === null) {
        errors.push({
          line: line.lineNumber,
          column: line.raw.indexOf(value) + 1,
          message: "Note must be in the form 1/N where N is a power of 2 (1, 2, 4, 8, 16, 32, 64, 128)",
        });
        return true;
      }

      headers.note = {
        field: "note",
        value: parsed,
        line: line.lineNumber,
      };
      return true;
    }
    case "grouping": {
      if (headers.grouping) {
        errors.push({
          line: line.lineNumber,
          column: 1,
          message: "Duplicate header `grouping`",
        });
        return true;
      }

      const parsed = parseGroupingValue(value);

      if (!parsed) {
        errors.push({
          line: line.lineNumber,
          column: line.raw.indexOf(value) + 1,
          message: "Grouping must use the form n+n+...",
        });
        return true;
      }

      headers.grouping = {
        field: "grouping",
        values: parsed,
        line: line.lineNumber,
      };
      return true;
    }
  }

  return false;
}

function finalizeHeaders(headers: HeaderAccumulator, errors: ParseError[]): ParsedHeaders {
  const time =
    headers.time ??
    (() => {
      errors.push({
        line: 1,
        column: 1,
        message: "Missing required header `time`",
      });
      return { field: "time", beats: 4, beatUnit: 4, line: 0 } as const;
    })();
  const inferredGrouping = inferGrouping(time.beats, time.beatUnit);

  const grouping =
    headers.grouping ??
    (() => {
      if (!inferredGrouping) {
        errors.push({
          line: time.line || 1,
          column: 1,
          message: `Missing required header \`grouping\` for time ${time.beats}/${time.beatUnit}`,
        });
        return { field: "grouping", values: [time.beats], line: 0 };
      }

      return { field: "grouping", values: inferredGrouping, line: 0 };
    })();

  if (grouping.values.reduce((sum, value) => sum + value, 0) !== time.beats) {
    errors.push({
      line: grouping.line || 1,
      column: 1,
      message: `Grouping must sum to time numerator ${time.beats}`,
    });
  }

  return {
    ...(headers.title ? { title: headers.title } : {}),
    ...(headers.subtitle ? { subtitle: headers.subtitle } : {}),
    ...(headers.composer ? { composer: headers.composer } : {}),
    tempo: headers.tempo ?? { field: "tempo", value: 120, line: 0 },
    time,
    divisions: headers.divisions,
    note:
      headers.note ??
      (headers.divisions
        ? undefined
        : (() => {
            errors.push({
              line: 1,
              column: 1,
              message: "Missing required header `note` (e.g., note 1/16)",
            });
            return { field: "note", value: 16, line: 0 };
          })()),
    grouping,
  };
}

function isBasicGlyph(value: string): value is BasicGlyph {
  return BASIC_GLYPHS.includes(value as BasicGlyph);
}

function readBasicGlyph(input: string, cursor: number): { glyph: BasicGlyph; next: number } | null {
  for (const glyph of MULTI_CHAR_GLYPHS) {
    if (input.startsWith(glyph, cursor)) {
      return { glyph, next: cursor + glyph.length };
    }
  }

  const glyph = input[cursor];
  if (glyph === undefined) return null;
  return isBasicGlyph(glyph) ? { glyph, next: cursor + 1 } : null;
}

function readModifier(input: string, start: number): { modifier: Modifier; next: number } | null {
  let end = start;

  while (end < input.length) {
    const char = input[end];
    if (char !== undefined && /[a-z-]/.test(char)) {
      end += 1;
    } else {
      break;
    }
  }

  const value = input.slice(start, end);

  if (!MODIFIERS.includes(value as Modifier)) {
    return null;
  }

  return {
    modifier: value as Modifier,
    next: end,
  };
}

function parseMeasureTokens(
  content: string,
  track: SourceTrackName | "ANONYMOUS",
  lineNumber: number,
  columnOffset: number,
  errors: ParseError[],
  allowGroups: boolean,
): MeasureToken[] {
  const tokens: MeasureToken[] = [];
  let cursor = 0;

  const skipSpaces = () => {
    while (cursor < content.length && /\s/.test(content[cursor]!)) {
      cursor++;
    }
  };

  while (cursor < content.length) {
    skipSpaces();
    if (cursor >= content.length) break;

    // 1. Check for routed block directive: @TRACK { ... }
    const routedTrackNames = [...SORTED_TRACK_NAMES];
    let handledRoutedBlock = false;
    if (content[cursor] === "@") {
      for (const id of routedTrackNames) {
        if (content.startsWith(`@${id}`, cursor)) {
          const nextCharPos = cursor + id.length + 1;
          const remainder = content.slice(nextCharPos);
          const sepMatch = remainder.match(/^\s*\{/);
          if (!sepMatch) {
            break;
          }

          const idEnd = cursor + id.length + 1 + sepMatch[0]!.length;
          let braceLevel = 1;
          let innerEnd = idEnd;
          while (innerEnd < content.length && braceLevel > 0) {
            if (content[innerEnd] === "{") braceLevel++;
            else if (content[innerEnd] === "}") braceLevel--;
            innerEnd++;
          }

          if (braceLevel > 0) {
            errors.push({
              line: lineNumber,
              column: columnOffset + cursor + id.length + 2,
              message: "Unterminated braced scope",
            });
            cursor = idEnd;
            handledRoutedBlock = true;
            break;
          }

          const innerContent = content.slice(idEnd, innerEnd - 1);
          const innerTokens = parseMeasureTokens(innerContent, id as TrackName, lineNumber, columnOffset + idEnd, errors, allowGroups);
          tokens.push({ kind: "braced", track: id, items: innerTokens });
          cursor = innerEnd;
          handledRoutedBlock = true;
          break;
        }
      }
    }

    if (handledRoutedBlock) continue;

    // 2. Check for Track Identifier: legacy TRACK { ... } or TRACK: ...
    const trackNames = [...SORTED_TRACK_NAMES, "ANONYMOUS"];
    let matchedTrackId: string | null = null;
    for (const id of trackNames) {
      if (content.startsWith(id, cursor)) {
        const nextCharPos = cursor + id.length;
        // Check if id is followed by { or : or space+{:
        const remainder = content.slice(nextCharPos);
        const sepMatch = remainder.match(/^\s*([:{])/);
        if (sepMatch) {
          matchedTrackId = id;
          const separator = sepMatch[1] as "{" | ":";
          const fullMatchLength = id.length + sepMatch[0]!.length;

          if (separator === "{") {
            errors.push({
              line: lineNumber,
              column: columnOffset + cursor,
              message: `Legacy routed block syntax \`${id} { ... }\` has been removed; use \`@${id} { ... }\` instead.`,
            });
            const idEnd = cursor + fullMatchLength;
            let braceLevel = 1;
            let innerEnd = idEnd;
            while (innerEnd < content.length && braceLevel > 0) {
              if (content[innerEnd] === "{") braceLevel++;
              else if (content[innerEnd] === "}") braceLevel--;
              innerEnd++;
            }

            if (braceLevel > 0) {
              errors.push({ 
                line: lineNumber, 
                column: columnOffset + cursor + id.length + 1, 
                message: "Unterminated braced scope" 
              });
              cursor = idEnd;
              break;
            }

            const innerContent = content.slice(idEnd, innerEnd - 1);
            const innerTokens = parseMeasureTokens(innerContent, id as TrackName, lineNumber, columnOffset + idEnd, errors, allowGroups);
            tokens.push({ kind: "braced", track: id, items: innerTokens });
            cursor = innerEnd;
            matchedTrackId = "HANDLED"; 
            break;
          } else {
            // TRACK: prefix, advance cursor past prefix for generic parsing
            cursor += fullMatchLength;
          }
          break;
        }
      }
    }

    if (matchedTrackId === "HANDLED") continue;
    if (cursor >= content.length) break;

    // 3. Check for Group: [ ... ]
    if (content[cursor] === "[") {
      if (!allowGroups) {
        errors.push({ line: lineNumber, column: columnOffset + cursor + 1, message: "Nested groups are not allowed" });
        break;
      }

      const closeIndex = content.indexOf("]", cursor);
      if (closeIndex === -1) {
        errors.push({ line: lineNumber, column: columnOffset + cursor + 1, message: "Unterminated group" });
        break;
      }

      const body = content.slice(cursor + 1, closeIndex);
      const colonMatch = body.match(/^(\d+)\s*:\s*(.*)$/);
      let span: number, inner: string;

      if (colonMatch) {
        span = Number(colonMatch[1]);
        inner = colonMatch[2] ?? "";
        if (span < 1) {
          errors.push({ line: lineNumber, column: columnOffset + cursor + 1, message: "Group span must be at least 1" });
          cursor = closeIndex + 1;
          continue;
        }
        if (!inner.trim()) {
          errors.push({ line: lineNumber, column: columnOffset + cursor + 1, message: "Empty group" });
          cursor = closeIndex + 1;
          continue;
        }
      } else if (body.trim()) {
        span = 1;
        inner = body.trim();
      } else {
        errors.push({ line: lineNumber, column: columnOffset + cursor + 1, message: "Empty group" });
        cursor = closeIndex + 1;
        continue;
      }

      const items = parseMeasureTokens(inner, track, lineNumber, columnOffset + cursor + 2 + (colonMatch ? body.indexOf(inner) : 0), errors, false);
      cursor = closeIndex + 1;

      // Parse trailing modifiers after the group (e.g., [2:s]:flam)
      const groupModifiers: Modifier[] = [];
      while (content[cursor] === ":") {
        const modResult = readModifier(content, cursor + 1);
        if (!modResult) {
          let modifierEnd = cursor + 1;
          while (modifierEnd < content.length) {
            const char = content[modifierEnd];
            if (char !== undefined && /[a-z-]/.test(char)) {
              modifierEnd += 1;
            } else {
              break;
            }
          }

          const rawModifier = content.slice(cursor + 1, modifierEnd);
          if (rawModifier) {
            errors.push({
              line: lineNumber,
              column: columnOffset + cursor + 1,
              message: `Unknown modifier \`${rawModifier}\``,
            });
            cursor = modifierEnd;
            continue;
          }

          break;
        }
        groupModifiers.push(modResult.modifier);
        cursor = modResult.next;
      }

      tokens.push({ kind: "group", count: items.length, span, items, modifiers: groupModifiers });
      continue;
    }

    // 3. Generic Token Parsing: Glyph[( :Mod | . | / | * )*][+]
    // Modifiers and duration suffixes can be interleaved in any order
    const parsePart = (ptr: number, inheritedTrackOverride?: string): { token: TokenGlyph; next: number } | null => {
      const glyphResult = readBasicGlyph(content, ptr);
      if (!glyphResult) return null;

      let nextPtr = glyphResult.next;
      const modifiers: Modifier[] = [];
      let dots = 0, halves = 0, stars = 0;

      // Loop to handle interleaved modifiers and duration suffixes
      while (nextPtr < content.length) {
        // Handle modifiers (starting with :)
        if (content[nextPtr] === ":") {
          const modResult = readModifier(content, nextPtr + 1);
          if (!modResult) {
            let modifierEnd = nextPtr + 1;
            while (modifierEnd < content.length) {
              const char = content[modifierEnd];
              if (char !== undefined && /[a-z-]/.test(char)) {
                modifierEnd += 1;
              } else {
                break;
              }
            }

            const rawModifier = content.slice(nextPtr + 1, modifierEnd);
            if (rawModifier) {
              errors.push({
                line: lineNumber,
                column: columnOffset + nextPtr + 1,
                message: `Unknown modifier \`${rawModifier}\``,
              });
              nextPtr = modifierEnd;
              continue;
            }

            break;
          }
          modifiers.push(modResult.modifier);
          nextPtr = modResult.next;
          continue;
        }

        // Handle duration suffixes (., /, *)
        if (content[nextPtr] === ".") { dots++; nextPtr++; }
        else if (content[nextPtr] === "/") { halves++; nextPtr++; }
        else if (content[nextPtr] === "*") { stars++; nextPtr++; }
        else break;
      }

      return {
        token: { kind: "basic", value: glyphResult.glyph, dots, halves, stars, modifiers, trackOverride: inheritedTrackOverride },
        next: nextPtr
      };
    };

    const firstPart = parsePart(cursor, matchedTrackId || undefined);
    if (!firstPart) {
      errors.push({ line: lineNumber, column: columnOffset + cursor + 1, message: `Unknown token \`${content[cursor]}\` on track ${track}` });
      cursor += 1;
      continue;
    }

    const skipWhitespaceFrom = (ptr: number) => {
      let next = ptr;
      while (next < content.length && /\s/.test(content[next]!)) {
        next += 1;
      }
      return next;
    };

    let nextCursor = firstPart.next;
    let combinedCursor = skipWhitespaceFrom(nextCursor);
    if (content[combinedCursor] === "+") {
      const items: TokenGlyph[] = [firstPart.token];
      while (content[combinedCursor] === "+") {
        combinedCursor += 1;
        combinedCursor = skipWhitespaceFrom(combinedCursor);
        let subTrackOverride: string | undefined;
        for (const tid of [...SORTED_TRACK_NAMES, "ANONYMOUS"]) {
           if (content.startsWith(tid, combinedCursor) && content[combinedCursor + tid.length] === ":") {
             subTrackOverride = tid;
             combinedCursor += tid.length + 1;
             break;
           }
        }
        
        const subPart = parsePart(combinedCursor, subTrackOverride);
        if (!subPart) break;
        items.push(subPart.token);
        nextCursor = subPart.next;
        combinedCursor = skipWhitespaceFrom(nextCursor);
      }
      tokens.push({ kind: "combined", items });
    } else {
      tokens.push(firstPart.token);
    }

    cursor = nextCursor;
  }

  return tokens;
}

function parseTrackName(line: PreprocessedLine, errors: ParseError[]): { track: SourceTrackName | "ANONYMOUS"; rest: string } | null {
  if (line.content.startsWith("|")) {
    return { track: "ANONYMOUS", rest: line.content };
  }

  const match = line.content.match(/^([A-Z0-9]+)\s*(.*)$/);

  if (!match) {
    errors.push({
      line: line.lineNumber,
      column: 1,
      message: "Expected a track line",
    });
    return null;
  }

  const track = match[1];

  if (!TRACKS.includes(track as TrackName)) {
    errors.push({
      line: line.lineNumber,
      column: 1,
      message: `Unknown track \`${track}\``,
    });
    return null;
  }

  return {
    track: track as SourceTrackName,
    rest: match[2] ?? "",
  };
}

function parseMeasureTail(remainder: string, line: PreprocessedLine, errors: ParseError[]): RawMeasure[] {
  const measures: RawMeasure[] = [];
  let cursor = 0;
  const remainderStart = line.content.length - remainder.length;
  let currentLeftBoundary:
    | { kind: "barline" | "repeat_start"; voltaIndices?: number[]; voltaTerminator?: boolean }
    | null = null;

  const sameVoltaIndices = (left?: number[], right?: number[]) =>
    left !== undefined
    && right !== undefined
    && left.length === right.length
    && left.every((value, index) => value === right[index]);

  const parseBoundary = (
    index: number,
  ): {
    length: number;
    kind: "barline" | "repeat_start" | "repeat_end";
    times?: number;
    voltaIndices?: number[];
    voltaTerminator?: boolean;
    double?: boolean;
    final?: boolean;
    column: number;
    offset: number;
  } | null => {
    const location = {
      column: remainderStart + index + 1,
      offset: line.startOffset + remainderStart + index,
    };
    const repeatEndWithVoltaMatch = remainder.slice(index).match(/^:\|\s*(\d+(?:,\d+)*)\./);
    if (repeatEndWithVoltaMatch?.[1] !== undefined) {
      return {
        length: repeatEndWithVoltaMatch[0].length,
        kind: "repeat_end",
        times: 2,
        voltaIndices: repeatEndWithVoltaMatch[1].split(",").map(Number),
        ...location,
      };
    }

    const voltaStartMatch = remainder.slice(index).match(/^\|\s*(\d+(?:,\d+)*)\./);
    if (voltaStartMatch?.[1] !== undefined) {
      return {
        length: voltaStartMatch[0].length,
        kind: "barline",
        voltaIndices: voltaStartMatch[1].split(",").map(Number),
        ...location,
      };
    }

    if (remainder.startsWith("|.", index)) {
      return { length: 2, kind: "barline", voltaTerminator: true, ...location };
    }

    if (remainder.startsWith("|:", index)) {
      return { length: 2, kind: "repeat_start", ...location };
    }

    if (remainder.startsWith(":|", index)) {
      const timesMatch = remainder.slice(index + 2).match(/^x(\d+)/);
      if (timesMatch?.[1] !== undefined) {
        return {
          length: 2 + 1 + timesMatch[1].length,
          kind: "repeat_end",
          times: parseInt(timesMatch[1], 10),
          ...location,
        };
      }
      return { length: 2, kind: "repeat_end", times: 2, ...location };
    }

    if (remainder.startsWith("||.", index)) {
      return { length: 3, kind: "barline", double: true, voltaTerminator: true, ...location };
    }

    if (remainder.startsWith("||", index)) {
      return { length: 2, kind: "barline", double: true, ...location };
    }

    if (remainder.startsWith("|", index)) {
      return { length: 1, kind: "barline", ...location };
    }

    return null;
  };

  while (cursor < remainder.length) {
    while (cursor < remainder.length) {
      const char = remainder[cursor];
      if (char !== undefined && /\s/.test(char)) {
        cursor += 1;
      } else {
        break;
      }
    }

    if (cursor >= remainder.length) {
      break;
    }

    if (currentLeftBoundary === null) {
      const startBoundary = parseBoundary(cursor);

      if (!startBoundary || (startBoundary.kind !== "barline" && startBoundary.kind !== "repeat_start")) {
        errors.push({
          line: line.lineNumber,
          column: line.content.indexOf(remainder.slice(cursor)) + 1,
          message: "Expected `|` or `|:` to start a measure",
        });
        break;
      }

      currentLeftBoundary = {
        kind: startBoundary.kind,
        ...(startBoundary.voltaIndices ? { voltaIndices: startBoundary.voltaIndices } : {}),
        ...(startBoundary.voltaTerminator ? { voltaTerminator: true } : {}),
      };
      cursor += startBoundary.length;
    }

    const endBoundaryMatch = remainder.slice(cursor).match(/:\|\s*\d+(?:,\d+)*\.|\|\s*\d+(?:,\d+)*\.|\|\.\s*\d+(?:,\d+)*\.|\|\.|\|\|\.|\|:|:\|x\d+|:\||\|/);

    if (!endBoundaryMatch || endBoundaryMatch.index === undefined) {
      errors.push({
        line: line.lineNumber,
        column: line.content.indexOf(remainder.slice(cursor)) + 1,
        message: "Unterminated measure",
      });
      break;
    }

    const endIndex = cursor + endBoundaryMatch.index;
    const endBoundary = parseBoundary(endIndex);

    if (!endBoundary) {
      errors.push({
        line: line.lineNumber,
        column: endIndex + 1,
        message: "Invalid measure boundary",
      });
      break;
    }

    const content = remainder.slice(cursor, endIndex).trim();

    // Regular measure (no special shorthand)
    const inferredRepeatEnd =
      currentLeftBoundary.voltaIndices !== undefined
      && endBoundary.kind === "barline"
      && endBoundary.voltaIndices !== undefined
      && !sameVoltaIndices(currentLeftBoundary.voltaIndices, endBoundary.voltaIndices);

    measures.push({
      content,
      repeatStart: currentLeftBoundary.kind === "repeat_start",
      repeatEnd: endBoundary.kind === "repeat_end" || inferredRepeatEnd,
      repeatEndLocation:
        endBoundary.kind === "repeat_end"
          ? { line: line.lineNumber, column: endBoundary.column, offset: endBoundary.offset }
          : undefined,
      repeatTimes:
        endBoundary.kind === "repeat_end" || inferredRepeatEnd
          ? endBoundary.times ?? 2
          : undefined,
      barline: endBoundary.final ? "final" : endBoundary.double ? "double" : undefined,
      voltaIndices: currentLeftBoundary.voltaIndices,
      voltaTerminator: currentLeftBoundary.voltaTerminator || endBoundary.voltaTerminator,
    });

    const nextCursor = endIndex + endBoundary.length;
    const compactRepeatEndStart = endBoundary.kind === "repeat_end" && remainder.startsWith(":", nextCursor);
    currentLeftBoundary = {
      kind: endBoundary.kind === "repeat_start" || compactRepeatEndStart ? "repeat_start" : "barline",
      ...(endBoundary.voltaIndices ? { voltaIndices: endBoundary.voltaIndices } : {}),
      ...(endBoundary.voltaTerminator ? { voltaTerminator: true } : {}),
    };
    cursor = nextCursor + (compactRepeatEndStart ? 1 : 0);
  }

  return measures;
}

function splitTopLevelParts(content: string): Array<{ text: string; start: number }> {
  const parts: Array<{ text: string; start: number }> = [];
  let squareDepth = 0;
  let braceDepth = 0;
  let tokenStart: number | null = null;

  for (let index = 0; index < content.length; index += 1) {
    const char = content[index];
    if (char === undefined) continue;

    if (tokenStart === null) {
      if (/\s/.test(char)) {
        continue;
      }
      tokenStart = index;
    }

    if (char === "[" ) squareDepth += 1;
    else if (char === "]") squareDepth = Math.max(0, squareDepth - 1);
    else if (char === "{") braceDepth += 1;
    else if (char === "}") braceDepth = Math.max(0, braceDepth - 1);

    const next = content[index + 1];
    const atBoundary = next === undefined || (/\s/.test(next) && squareDepth === 0 && braceDepth === 0);
    if (tokenStart !== null && atBoundary) {
      parts.push({ text: content.slice(tokenStart, index + 1), start: tokenStart });
      tokenStart = null;
    }
  }

  return parts;
}

function isInlineRepeatSuffixToken(text: string): boolean {
  return /^\*(-?\d+)$/.test(text);
}

function extractNavigationTokens(content: string, line: PreprocessedLine, errors: ParseError[]) {
  const parts = splitTopLevelParts(content);
  const remaining: string[] = [];
  let startNav: ParsedStartNav | undefined;
  let endNav: ParsedEndNav | undefined;

  const partColumn = (offset: number) => {
    const contentStart = line.content.indexOf(content);
    return Math.max(1, contentStart + offset + 1);
  };

  const nonNavParts = parts.filter((part) =>
    !(START_NAVS as readonly string[]).includes(part.text)
    && !(END_NAVS as readonly string[]).includes(part.text)
    && !(DEPRECATED_NAVS as readonly string[]).includes(part.text),
  );
  const anchorParts = nonNavParts.filter((part) => !isInlineRepeatSuffixToken(part.text));
  const pureNavigationMeasure = anchorParts.length === 0;

  let nonNavSeen = 0;
  for (const part of parts) {
    const column = partColumn(part.start);
    const anchorSeen = nonNavParts.slice(0, nonNavSeen).filter((candidate) => !isInlineRepeatSuffixToken(candidate.text)).length;
    const nonNavAfter = anchorParts.length - anchorSeen;

    if ((DEPRECATED_NAVS as readonly string[]).includes(part.text)) {
      errors.push({
        line: line.lineNumber,
        column,
        message: part.text === "@da-capo" ? "Use `@dc` instead of `@da-capo`" : "Use `@ds` instead of `@dal-segno`",
      });
      continue;
    }

    if ((START_NAVS as readonly string[]).includes(part.text)) {
      if (startNav !== undefined) {
        errors.push({
          line: line.lineNumber,
          column,
          message: "Measure contains multiple start-side navigation markers",
        });
        continue;
      }

      const kind = part.text.slice(1) as StartNavKind;
      if (kind === "coda") {
        if (!pureNavigationMeasure && anchorSeen !== 0) {
          errors.push({
            line: line.lineNumber,
            column,
            message: "`@coda` may appear only at the beginning of a measure",
          });
          continue;
        }
        startNav = { kind: "coda", anchor: "left-edge" };
        continue;
      }

      if (!pureNavigationMeasure && nonNavAfter === 0) {
        errors.push({
          line: line.lineNumber,
          column,
          message: "`@segno` may not appear at the end of a measure",
        });
        continue;
      }

      startNav = pureNavigationMeasure || anchorSeen === 0
        ? { kind: "segno", anchor: "left-edge" }
        : { kind: "segno", anchor: { tokenAfter: anchorSeen } };
      continue;
    }

    if ((END_NAVS as readonly string[]).includes(part.text)) {
      if (endNav !== undefined) {
        errors.push({
          line: line.lineNumber,
          column,
          message: "Measure contains multiple end-side navigation instructions",
        });
        continue;
      }

      const kind = part.text.slice(1) as EndNavKind;
      if (kind === "to-coda") {
        if (!pureNavigationMeasure && anchorSeen === 0) {
          errors.push({
            line: line.lineNumber,
            column,
            message: "`@to-coda` may not appear at the beginning of a measure",
          });
          continue;
        }
        endNav = pureNavigationMeasure || nonNavAfter === 0
          ? { kind: "to-coda", anchor: "right-edge" }
          : { kind: "to-coda", anchor: { tokenBefore: anchorSeen - 1 } };
        continue;
      }

      if (!pureNavigationMeasure && nonNavAfter !== 0) {
        errors.push({
          line: line.lineNumber,
          column,
          message: `\`${part.text}\` may appear only at the end of a measure`,
        });
        continue;
      }

      endNav = { kind, anchor: "right-edge" } as ParsedEndNav;
      continue;
    }

    remaining.push(part.text);
    nonNavSeen += 1;
  }

  return {
    content: remaining.join(" "),
    startNav,
    endNav,
  };
}

function parseTrackLine(line: PreprocessedLine, errors: ParseError[]): ParsedTrackLine | null {
  const parsed = parseTrackName(line, errors);

  if (!parsed) {
    return null;
  }

  const measures = parseMeasureTail(parsed.rest, line, errors);

  if (measures.length === 0) {
    errors.push({
      line: line.lineNumber,
      column: 1,
      message: `Track \`${parsed.track}\` does not contain any measures`,
    });
    return null;
  }

  const expandedMeasures = measures.flatMap((measure, _measureIndex) => {
      const navigation = extractNavigationTokens(measure.content, line, errors);
      const normalizedContent = navigation.content;
      const measureRepeatMatch = normalizedContent.match(/^(%+)$/);
      if (measureRepeatMatch?.[1] !== undefined) {
        return [{
          content: normalizedContent,
          tokens: [],
          repeatStart: measure.repeatStart,
          repeatEnd: measure.repeatEnd,
          repeatTimes: measure.repeatTimes,
          barline: measure.barline,
          startNav: navigation.startNav,
          endNav: navigation.endNav,
          voltaIndices: measure.voltaIndices,
          voltaTerminator: measure.voltaTerminator,
          measureRepeatSlashes: measureRepeatMatch[1].length,
        }];
      }

      if (normalizedContent.includes("%")) {
        errors.push({
          line: line.lineNumber,
          column: 1,
          message: "Measure repeat shorthand must occupy the entire measure",
        });
        return [];
      }

      const multiRestMatch = normalizedContent.match(/^--+\s*((?:1\d+)|(?:[2-9]\d*))\s*--+$/);
      if (multiRestMatch?.[1] !== undefined) {
        const count = parseInt(multiRestMatch[1], 10);
        return [{
          content: normalizedContent,
          tokens: [],
          repeatStart: measure.repeatStart,
          repeatEnd: measure.repeatEnd,
          repeatTimes: measure.repeatTimes,
          barline: measure.barline,
          startNav: navigation.startNav,
          endNav: navigation.endNav,
          voltaIndices: measure.voltaIndices,
          voltaTerminator: measure.voltaTerminator,
          multiRestCount: count,
        }];
      }

      // Check for *N inline repeat: "xxxx *3" means the content occupies 3 total measures
      // Non-greedy match: d*3 → content="d" (with stars on token), *3 is inline repeat
      // The star in the content is parsed as token modifier, *N at end is inline repeat
      const inlineRepeatMatch = normalizedContent.match(/^(.*?)\s*\*\s*(-?\d+)\s*$/);
      const m1 = inlineRepeatMatch?.[1];
      const m2 = inlineRepeatMatch?.[2];
      if (inlineRepeatMatch && m1 !== undefined && m2 !== undefined && m1.trim() !== "") {
        const repeatCount = parseInt(m2, 10);
        if (repeatCount >= 1) {
          const measureContent = m1.trim();
          const parsedTokens = parseMeasureTokens(
            measureContent,
            parsed.track,
            line.lineNumber,
            line.content.indexOf(measureContent) + 1,
            errors,
            true,
          );
          const expanded: ParsedMeasure[] = [];
          for (let i = 0; i < repeatCount; i++) {
            expanded.push({
              content: measureContent,
              tokens: parsedTokens,
              repeatStart: i === 0 ? measure.repeatStart : false,
              repeatEnd: i === repeatCount - 1 ? measure.repeatEnd : false,
              repeatTimes: i === repeatCount - 1 ? measure.repeatTimes : undefined,
              barline: i === repeatCount - 1 ? measure.barline : undefined,
              startNav: i === 0 ? navigation.startNav : undefined,
              endNav: i === repeatCount - 1 ? navigation.endNav : undefined,
              voltaIndices: i === 0 ? measure.voltaIndices : undefined,
              voltaTerminator: i === repeatCount - 1 ? measure.voltaTerminator : undefined,
            });
          }
          return expanded;
        } else {
          errors.push({
            line: line.lineNumber,
            column: 1,
            message: "Repeat count must be at least 1",
          });
          return [];
        }
      }

      // Check for bare *N repeat marker - creates N total empty measures
      const bareRepeatMatch = normalizedContent.match(/^\*(-?\d+)$/);
      const bm1 = bareRepeatMatch?.[1];
      if (bareRepeatMatch && bm1 !== undefined) {
        const count = parseInt(bm1, 10);
        if (count < 1) {
          errors.push({
            line: line.lineNumber,
            column: 1,
            message: "Repeat count must be at least 1",
          });
          return [];
        }
        const expanded: ParsedMeasure[] = [];
        for (let i = 0; i < count; i++) {
          expanded.push({
            content: "",
            tokens: [],
            repeatStart: i === 0 ? measure.repeatStart : false,
            repeatEnd: i === count - 1 ? measure.repeatEnd : false,
            repeatTimes: i === count - 1 ? measure.repeatTimes : undefined,
            barline: i === count - 1 ? measure.barline : undefined,
            startNav: i === 0 ? navigation.startNav : undefined,
            endNav: i === count - 1 ? navigation.endNav : undefined,
            voltaIndices: i === 0 ? measure.voltaIndices : undefined,
            voltaTerminator: i === count - 1 ? measure.voltaTerminator : undefined,
          });
        }
        return expanded;
      }

      const measureStart = line.content.indexOf(normalizedContent);
      const parsedTokens = parseMeasureTokens(
        normalizedContent,
        parsed.track,
        line.lineNumber,
        measureStart === -1 ? 1 : measureStart + 1,
        errors,
        true,
      );

      // Inline repeat: replicate this measure N times
      if (measure.repeatCount !== undefined && measure.repeatCount > 1) {
        const expanded: ParsedMeasure[] = [];
        for (let i = 0; i < measure.repeatCount; i++) {
          expanded.push({
            content: measure.content,
            tokens: parsedTokens,
            repeatStart: i === 0 ? measure.repeatStart : false,
            repeatEnd: i === measure.repeatCount - 1 ? measure.repeatEnd : false,
            repeatTimes: i === measure.repeatCount - 1 ? measure.repeatTimes : undefined,
            barline: i === measure.repeatCount - 1 ? measure.barline : undefined,
          });
        }
        return expanded;
      }

      return [{
        content: normalizedContent,
        tokens: parsedTokens,
        repeatStart: measure.repeatStart,
        repeatEnd: measure.repeatEnd,
        repeatTimes: measure.repeatTimes,
        barline: measure.barline,
        startNav: navigation.startNav,
        endNav: navigation.endNav,
        voltaIndices: measure.voltaIndices,
        voltaTerminator: measure.voltaTerminator,
        multiRestCount: measure.multiRestCount,
      }];
    });

  if (expandedMeasures.length >= 2) {
    const lastMeasure = expandedMeasures[expandedMeasures.length - 1];
    const previousMeasure = expandedMeasures[expandedMeasures.length - 2];
    const metadataOnlyTrailingMeasure =
      lastMeasure !== undefined &&
      previousMeasure !== undefined &&
      lastMeasure.content === "" &&
      (lastMeasure.tokens?.length ?? 0) === 0 &&
      !lastMeasure.repeatStart &&
      !lastMeasure.repeatEnd &&
      lastMeasure.measureRepeatSlashes === undefined &&
      lastMeasure.multiRestCount === undefined &&
      (
        lastMeasure.startNav !== undefined ||
        lastMeasure.endNav !== undefined ||
        lastMeasure.voltaIndices !== undefined ||
        lastMeasure.voltaTerminator === true ||
        lastMeasure.barline !== undefined
      );

    if (metadataOnlyTrailingMeasure) {
      if (lastMeasure.startNav !== undefined) previousMeasure.startNav = lastMeasure.startNav;
      if (lastMeasure.endNav !== undefined) previousMeasure.endNav = lastMeasure.endNav;
      if (lastMeasure.voltaIndices !== undefined) previousMeasure.voltaIndices = lastMeasure.voltaIndices;
      if (lastMeasure.voltaTerminator) previousMeasure.voltaTerminator = true;
      if (lastMeasure.barline !== undefined) previousMeasure.barline = lastMeasure.barline;
      expandedMeasures.pop();
    }
  }

  return {
    track: parsed.track,
    lineNumber: line.lineNumber,
    measures: expandedMeasures,
    source: line,
  };
}

function splitParagraphs(lines: PreprocessedLine[], errors: ParseError[]): TrackParagraph[] {
  const rawParagraphs: PreprocessedLine[][] = [];
  let current: PreprocessedLine[] = [];

  for (const line of lines) {
    if (line.kind === "blank") {
      if (current.length > 0) {
        rawParagraphs.push(current);
        current = [];
      }
      continue;
    }

    if (line.kind === "content" || line.kind === "comment") {
      current.push(line);
    }
  }

  if (current.length > 0) {
    rawParagraphs.push(current);
  }

  return rawParagraphs
    .map((paragraphLines) => {
      let noteValue: number | undefined;
      const filteredLines: PreprocessedLine[] = [];

      for (const line of paragraphLines) {
        if (line.kind === "comment") continue;
        if (line.kind === "content") {
          const match = line.content.match(/^note\s+1\s*\/\s*(\d+)$/);
          if (match && match[1]) {
            const n = parseInt(match[1], 10);
            if (isPowerOfTwo(n) && n <= 128) {
              noteValue = n;
            } else {
              errors.push({
                line: line.lineNumber,
                column: 1,
                message: "Note must be in the form 1/N where N is a power of 2 (1, 2, 4, 8, 16, 32, 64, 128)",
              });
            }
            continue; // Skip the note override line from track parsing
          }
          filteredLines.push(line);
        }
      }

      const parsedLines = filteredLines
        .map((line) => parseTrackLine(line, errors))
        .filter((line): line is ParsedTrackLine => line !== null);

      if (parsedLines.length === 0) return null;

      const paragraph: TrackParagraph = {
        startLine: paragraphLines[0]!.lineNumber,
        lines: parsedLines,
        noteValue,
      };
      return paragraph;
    })
    .filter((paragraph): paragraph is TrackParagraph => paragraph !== null);
}

export function parseDocumentSkeleton(source: string): DocumentSkeleton {
  const lines = preprocessSource(source);
  const errors: ParseError[] = [];
  const headers: HeaderAccumulator = {};

  // If headers are separated from body by a blank line, the body may begin with
  // a paragraph-level `note 1/N` override before the first track line.
  let bodyStartIndex = lines.length;
  let sawNonBlank = false;
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    if (!line) continue;

    if (line.kind === "blank") {
      if (sawNonBlank) {
        const nextContentIndex = lines.findIndex((candidate, candidateIndex) =>
          candidateIndex > index && candidate.kind === "content",
        );
        if (nextContentIndex !== -1) {
          bodyStartIndex = nextContentIndex;
          break;
        }
      }
      continue;
    }

    sawNonBlank = true;
    if (line.kind === "content" && line.content.includes("|")) {
      bodyStartIndex = index;
      break;
    }
  }

  // Parse headers from all content lines preceding the first track line
  for (let index = 0; index < bodyStartIndex; index += 1) {
    const line = lines[index];
    if (!line || line.kind !== "content") continue;

    const isHeader = parseHeaderLine(line, headers, errors);
    if (!isHeader) {
      const firstToken = line.content.split(/\s+/)[0];
      if (firstToken && /^[a-z][a-z0-9-]*$/i.test(firstToken) && /^[a-z]/.test(firstToken) && !HEADER_FIELDS.includes(firstToken as (typeof HEADER_FIELDS)[number])) {
        errors.push({
          line: line.lineNumber,
          column: 1,
          message: `Unknown header \`${firstToken}\``,
        });
      }
    }
  }

  const bodyLines = lines.slice(bodyStartIndex);
  const unexpectedHeader = bodyLines.find((line) => {
    if (line.kind !== "content") return false;
    const firstToken = line.content.split(/\s+/)[0];
    if (!firstToken || !HEADER_FIELDS.includes(firstToken as (typeof HEADER_FIELDS)[number])) {
      return false;
    }
    // 'note' is allowed at the start of a paragraph in bodyLines
    if (firstToken === "note") {
      const match = line.content.match(/^note\s+1\s*\/\s*(\d+)$/);
      if (match) return false;
    }
    return true;
  });

  if (unexpectedHeader) {
    errors.push({
      line: unexpectedHeader.lineNumber,
      column: 1,
      message: "Headers must appear before track content",
    });
  }

  return {
    headers: finalizeHeaders(headers, errors),
    paragraphs: splitParagraphs(bodyLines, errors),
    errors,
  };
}
