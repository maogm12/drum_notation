import { buildScoreAst, type ParseMode } from "./ast";
import { build_normalized_score as wasmNormalize } from "../wasm/pkg/drummark_core";
import {
  addFractions,
  basicTokenExceedsExactDurationRange,
  multiplyFractions,
  divideFractions,
  simplify,
  voiceForTrack,
  calculateTokenWeightAsFraction,
  compareFractions,
  fractionsEqual,
} from "./logic";
import {
  type EndNav,
  type Fraction,
  type HairpinIntent,
  type NormalizedEvent,
  type NormalizedHeader,
  type NormalizedScore,
  type ParsedEndNav,
  type ParsedStartNav,
  type ScoreAst,
  type StartNav,
  type TrackFamily,
  type NormalizedTrack,
  type TrackName,
  type TokenGlyph,
  type BasicGlyph,
  type Modifier,
  TRACKS,
} from "./types";

import { resolveFallbackTrack } from "./logic";

const CYMBAL_TRACKS = new Set<TrackName>(["HH", "RC", "RC2", "C", "C2", "SPL", "CHN"]);
const DRUM_TRACKS = new Set<TrackName>(["SD", "BD", "BD2", "T1", "T2", "T3", "T4"]);
const PEDAL_TRACKS = new Set<TrackName>(["HF"]);
const PERCUSSION_TRACKS = new Set<TrackName>(["CB", "WB", "CL"]);
const STATIC_MAGIC_TOKENS = new Set<BasicGlyph>([
  "s",
  "S",
  "b",
  "B",
  "b2",
  "B2",
  "r",
  "R",
  "r2",
  "R2",
  "c",
  "C",
  "c2",
  "C2",
  "t1",
  "T1",
  "t2",
  "T2",
  "t3",
  "T3",
  "t4",
  "T4",
  "o",
  "O",
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
]);
const ACCENT_MAGIC_TOKENS = new Set<BasicGlyph>([
  "D",
  "X",
  "P",
  "G",
  "S",
  "B",
  "B2",
  "R",
  "R2",
  "C",
  "C2",
  "O",
  "SPL",
  "CHN",
  "CB",
  "WB",
  "CL",
]);
const TRACKS_BY_MODIFIER: Record<Modifier, ReadonlySet<TrackName>> = {
  accent: new Set<TrackName>(TRACKS),
  open: new Set<TrackName>(["HH"]),
  "half-open": new Set<TrackName>(["HH"]),
  close: new Set<TrackName>(["HH", "HF"]),
  choke: new Set<TrackName>(["RC", "RC2", "C", "C2", "SPL", "CHN"]),
  bell: new Set<TrackName>(["RC", "RC2"]),
  rim: new Set<TrackName>(["SD"]),
  cross: new Set<TrackName>(["SD"]),
  flam: new Set<TrackName>(["SD", "T1", "T2", "T3", "T4"]),
  ghost: new Set<TrackName>(["SD", "HH", "T1", "T2", "T3", "T4"]),
  drag: new Set<TrackName>(["SD", "HH", "T1", "T2", "T3", "T4", "RC", "RC2"]),
  roll: new Set<TrackName>(["SD", "HH", "T1", "T2", "T3", "T4", "RC", "RC2", "BD", "BD2"]),
  dead: new Set<TrackName>(["SD", "HH", "T1", "T2", "T3", "T4", "BD", "BD2"]),
};

function gcd(a: number, b: number): number {
  let x = Math.abs(a);
  let y = Math.abs(b);
  while (y !== 0) {
    const next = x % y;
    x = y;
    y = next;
  }
  return x || 1;
}

function groupingBoundaryInSlots(cumulativeGrouping: number, beats: number, divisions: number): Fraction {
  return simplify({
    numerator: cumulativeGrouping * divisions,
    denominator: beats,
  });
}

function resolveParsedStartNav(nav: ParsedStartNav | undefined, tokenStarts: Fraction[]): StartNav | undefined {
  if (!nav) return undefined;
  if (nav.anchor === "left-edge") {
    return { kind: nav.kind, anchor: "left-edge" };
  }
  return {
    kind: "segno",
    anchor: { eventAfter: tokenStarts[nav.anchor.tokenAfter]! },
  };
}

function resolveParsedEndNav(nav: ParsedEndNav | undefined, tokenStarts: Fraction[]): EndNav | undefined {
  if (!nav) return undefined;
  if (nav.anchor === "right-edge") {
    return { kind: nav.kind, anchor: "right-edge" } as EndNav;
  }
  return {
    kind: "to-coda",
    anchor: { eventBefore: tokenStarts[nav.anchor.tokenBefore]! },
  };
}

function navKey(nav: StartNav | EndNav | undefined): string | undefined {
  return nav ? JSON.stringify(nav) : undefined;
}

function getTrackFamily(track: TrackName): TrackFamily {
  if (CYMBAL_TRACKS.has(track)) return "cymbal";
  if (DRUM_TRACKS.has(track)) return "drum";
  if (PEDAL_TRACKS.has(track)) return "pedal";
  if (PERCUSSION_TRACKS.has(track)) return "percussion";
  return "auxiliary";
}

type ResolvedToken = {
  track: TrackName;
  glyph: Exclude<BasicGlyph, "-">;
  modifiers: Modifier[];
};

type HairpinState = {
  activeType: HairpinIntent["type"] | null;
  activeStart: Fraction | null;
  startMeasureIndex: number | null;
};

type TrackHairpinResult = {
  hairpins: HairpinIntent[];
};

function applyModifiersToToken(token: TokenGlyph, modifiers: Modifier[]): TokenGlyph {
  if (token.kind === "basic") {
    return { ...token, modifiers: [...token.modifiers, ...modifiers] };
  }
  if (token.kind === "group") {
    return { ...token, modifiers: [...token.modifiers, ...modifiers] };
  }
  if (token.kind === "combined") {
    return {
      ...token,
      items: token.items.map((item) => applyModifiersToToken(item, modifiers)),
    };
  }
  // braced kind doesn't support modifiers
  return token;
}

function resolveToken(
  token: Extract<TokenGlyph, { kind: "basic" }>,
  contextTrack: TrackName | "ANONYMOUS",
): ResolvedToken | null {
  if (token.value === "-") return null;

  let track: TrackName;
  let glyph: Exclude<BasicGlyph, "-"> = "d";
  const modifiers = [...token.modifiers];
  const explicitTrack = token.trackOverride && TRACKS.includes(token.trackOverride as TrackName)
    ? token.trackOverride as TrackName
    : undefined;
  const stickingToken = token.value === "R" || token.value === "L";

  // 1. Resolve Track (Hierarchy)
  if (explicitTrack) {
    track = explicitTrack;
  } else if (contextTrack === "ST" && stickingToken) {
    track = "ST";
  } else if (STATIC_MAGIC_TOKENS.has(token.value)) {
    track = resolveFallbackTrack(token.value);
  } else if (contextTrack !== "ANONYMOUS") {
    track = contextTrack;
  } else {
    track = resolveFallbackTrack(token.value);
  }

  // 2. Resolve Magic Tokens (Mapping to d + modifiers)
  const v = token.value;
  if (ACCENT_MAGIC_TOKENS.has(v) && !(track === "ST" && stickingToken)) {
    if (!modifiers.includes("accent")) modifiers.push("accent");
  }

  if (v === "g" || v === "G") {
    if (!modifiers.includes("ghost")) modifiers.push("ghost");
  }

  if (v === "o" || v === "O") {
    if (!modifiers.includes("open")) modifiers.push("open");
  }

  // 3. Context-aware x/X mapping for Drum Family
  if ((v === "x" || v === "X") && getTrackFamily(track) === "drum") {
    if (!modifiers.includes("cross")) modifiers.push("cross");
  }

  // 4. Notehead Selection (Glyph semantic)
  if (track === "ST") {
    glyph = token.value as Exclude<BasicGlyph, "-">;
  } else if (getTrackFamily(track) === "cymbal") {
    glyph = "x";
  } else {
    glyph = "d";
  }

  return { track, glyph, modifiers };
}

function tokenToEvents(
  token: TokenGlyph,
  start: Fraction,
  duration: Fraction,
  contextTrack: TrackName | "ANONYMOUS",
  paragraphIndex: number,
  measureIndex: number,
  measureInParagraph: number,
  inheritedTuplet?: { actual: number; normal: number },
): NormalizedEvent[] {
  if (token.kind === "basic") {
    const resolved = resolveToken(token, contextTrack);
    if (!resolved) return [];

    const primaryModifier = resolved.modifiers.find((m) => m !== "accent");

    const kind: NormalizedEvent["kind"] = resolved.track === "ST" ? "sticking" : "hit";

    return [
      {
        track: resolved.track,
        paragraphIndex,
        measureIndex,
        measureInParagraph,
        start,
        duration,
        kind,
        glyph: resolved.glyph,
        modifiers: resolved.modifiers,
        modifier: primaryModifier,
        voice: voiceForTrack(resolved.track),
        beam: "none",
        ...(inheritedTuplet ? { tuplet: inheritedTuplet } : {}),
      },
    ];
  }

  if (token.kind === "combined") {
    return token.items.flatMap((item) =>
      tokenToEvents(item, start, duration, contextTrack, paragraphIndex, measureIndex, measureInParagraph),
    );
  }

  if (token.kind === "braced") {
    const events: NormalizedEvent[] = [];
    let currentStart = start;
    const totalWeight = calculateTokenWeightAsFraction(token);

    token.items.forEach((item) => {
      const itemWeight = calculateTokenWeightAsFraction(item);
      const itemDuration = multiplyFractions(duration, divideFractions(itemWeight, totalWeight));
      
      events.push(
        ...tokenToEvents(
          item,
          currentStart,
          itemDuration,
          token.track as TrackName,
          paragraphIndex,
          measureIndex,
          measureInParagraph,
        ),
      );
      currentStart = addFractions(currentStart, itemDuration);
    });
    return events;
  }

  if (token.kind === "group") {
    const events: NormalizedEvent[] = [];
    const totalWeight = token.items.reduce(
      (sum, item) => addFractions(sum, calculateTokenWeightAsFraction(item)),
      { numerator: 0, denominator: 1 },
    );
    const reducedDivisor = gcd(token.count, token.span);
    const reducedActual = token.count / reducedDivisor;
    const reducedNormal = token.span / reducedDivisor;
    const groupTuplet =
      token.count > token.span && !(reducedNormal === 1 && (reducedActual === 2 || reducedActual === 4))
        ? { actual: token.count, normal: token.span }
        : undefined;

    let currentStart = start;
    token.items.forEach((item) => {
      const itemWeight = calculateTokenWeightAsFraction(item);
      const itemDuration = multiplyFractions(duration, divideFractions(itemWeight, totalWeight));

      // Apply group modifiers to each item
      const itemWithModifiers = token.modifiers.length > 0
        ? applyModifiersToToken(item, token.modifiers)
        : item;

      events.push(
        ...tokenToEvents(
          itemWithModifiers,
          currentStart,
          itemDuration,
          contextTrack,
          paragraphIndex,
          measureIndex,
          measureInParagraph,
          groupTuplet,
        ),
      );
      currentStart = addFractions(currentStart, itemDuration);
    });
    return events;
  }

  return [];
}

function hairpinSignature(hairpin: HairpinIntent): string {
  return `${hairpin.type}:${hairpin.startMeasureIndex}:${hairpin.start.numerator}/${hairpin.start.denominator}->${hairpin.endMeasureIndex}:${hairpin.end.numerator}/${hairpin.end.denominator}`;
}

function findDurationOverflowToken(token: TokenGlyph): Extract<TokenGlyph, { kind: "basic" }> | null {
  if (token.kind === "basic") {
    return basicTokenExceedsExactDurationRange(token) ? token : null;
  }
  if (token.kind === "combined" || token.kind === "group" || token.kind === "braced") {
    for (const item of token.items) {
      const offender = findDurationOverflowToken(item);
      if (offender) return offender;
    }
  }
  return null;
}

function fractionsKey(fraction: Fraction): string {
  return `${fraction.numerator}/${fraction.denominator}`;
}

function pushHairpinFragment(
  state: HairpinState,
  hairpins: HairpinIntent[],
  endMeasureIndex: number,
  end: Fraction,
): void {
  if (!state.activeType || !state.activeStart || state.startMeasureIndex === null) return;
  if (state.startMeasureIndex === endMeasureIndex && fractionsEqual(state.activeStart, end)) return;
  hairpins.push({
    type: state.activeType,
    start: state.activeStart,
    startMeasureIndex: state.startMeasureIndex,
    end,
    endMeasureIndex,
  });
}

function collectHairpinsFromToken(
  token: TokenGlyph,
  measureIndex: number,
  start: Fraction,
  duration: Fraction,
  state: HairpinState,
  hairpins: HairpinIntent[],
  errors: ScoreAst["errors"],
  line: number,
): void {
  if (token.kind === "crescendo_start" || token.kind === "decrescendo_start") {
    const nextType: HairpinIntent["type"] = token.kind === "crescendo_start" ? "crescendo" : "decrescendo";
    if (state.activeType && state.activeStart && fractionsEqual(state.activeStart, start) && state.activeType !== nextType) {
      errors.push({
        line,
        column: 1,
        message: `Conflicting hairpin start types at the same position`,
      });
      state.activeType = nextType;
      state.startMeasureIndex = measureIndex;
      return;
    }
    pushHairpinFragment(state, hairpins, measureIndex, start);
    state.activeType = nextType;
    state.activeStart = start;
    state.startMeasureIndex = measureIndex;
    return;
  }

  if (token.kind === "hairpin_end") {
    if (!state.activeType || !state.activeStart) {
      errors.push({
        line,
        column: 1,
        message: "`!` without preceding `<` or `>`",
      });
      return;
    }
    pushHairpinFragment(state, hairpins, measureIndex, start);
    state.activeType = null;
    state.activeStart = null;
    state.startMeasureIndex = null;
    return;
  }

  if (token.kind === "combined") {
    for (const item of token.items) {
      collectHairpinsFromToken(item, measureIndex, start, duration, state, hairpins, errors, line);
    }
    return;
  }

  if (token.kind === "braced") {
    const totalWeight = calculateTokenWeightAsFraction(token);
    let currentStart = start;
    for (const item of token.items) {
      const itemWeight = calculateTokenWeightAsFraction(item);
      const itemDuration = fractionsEqual(totalWeight, { numerator: 0, denominator: 1 })
        ? { numerator: 0, denominator: 1 }
        : multiplyFractions(duration, divideFractions(itemWeight, totalWeight));
      collectHairpinsFromToken(item, measureIndex, currentStart, itemDuration, state, hairpins, errors, line);
      currentStart = addFractions(currentStart, itemDuration);
    }
    return;
  }

  if (token.kind === "group") {
    const totalWeight = token.items.reduce(
      (sum, item) => addFractions(sum, calculateTokenWeightAsFraction(item)),
      { numerator: 0, denominator: 1 },
    );
    let currentStart = start;
    for (const item of token.items) {
      const itemWeight = calculateTokenWeightAsFraction(item);
      const itemDuration = fractionsEqual(totalWeight, { numerator: 0, denominator: 1 })
        ? { numerator: 0, denominator: 1 }
        : multiplyFractions(duration, divideFractions(itemWeight, totalWeight));
      collectHairpinsFromToken(item, measureIndex, currentStart, itemDuration, state, hairpins, errors, line);
      currentStart = addFractions(currentStart, itemDuration);
    }
  }
}

function collectTrackHairpins(
  measure: ScoreAst["paragraphs"][number]["tracks"][number]["measures"][number],
  globalMeasureIndex: number,
  noteValue: number,
  timeSignature: ScoreAst["headers"]["time"],
  state: HairpinState,
  errors: ScoreAst["errors"],
): TrackHairpinResult {
  const divisions = (timeSignature.beats * noteValue) / timeSignature.beatUnit;
  const slotDuration = simplify({
    numerator: 1,
    denominator: noteValue,
  });
  const hairpins: HairpinIntent[] = [];
  let currentSlotOffset: Fraction = { numerator: 0, denominator: 1 };

  for (const token of measure.tokens) {
    const overflowToken = findDurationOverflowToken(token);
    if (overflowToken) {
      continue;
    }
    const weight = calculateTokenWeightAsFraction(token);
    const tokenStart = multiplyFractions(slotDuration, currentSlotOffset);
    const tokenDuration = multiplyFractions(slotDuration, weight);
    collectHairpinsFromToken(token, globalMeasureIndex, tokenStart, tokenDuration, state, hairpins, errors, measure.sourceLine || 0);
    currentSlotOffset = addFractions(currentSlotOffset, weight);
  }

  const divisionsFrac: Fraction = { numerator: divisions, denominator: 1 };
  if (!fractionsEqual(currentSlotOffset, divisionsFrac) && measure.measureRepeat === undefined && measure.multiRest === undefined) {
    // Duration diagnostics are already emitted elsewhere; keep the state machine aligned with the nominal bar end.
  }

  return {
    hairpins,
  };
}

function mergeMeasureHairpins(
  globalMeasureIndex: number,
  sourceLine: number,
  perTrack: HairpinIntent[],
  errors: ScoreAst["errors"],
): { hairpins?: HairpinIntent[] } {
  const signatures = new Set<string>();
  const merged: HairpinIntent[] = [];
  for (const hairpin of perTrack) {
    const signature = hairpinSignature(hairpin);
    if (signatures.has(signature)) continue;
    signatures.add(signature);
    merged.push(hairpin);
  }

  const startsByPosition = new Map<string, Set<HairpinIntent["type"]>>();
  for (const hairpin of merged) {
    const key = fractionsKey(hairpin.start);
    const types = startsByPosition.get(key) ?? new Set<HairpinIntent["type"]>();
    types.add(hairpin.type);
    startsByPosition.set(key, types);
  }
  for (const [key, types] of startsByPosition) {
    if (types.size > 1) {
      errors.push({
        line: sourceLine,
        column: 1,
        message: `Conflicting hairpin start types at position ${key} in bar ${globalMeasureIndex + 1}`,
      });
    }
  }

  merged.sort((left, right) => compareFractions(left.start, right.start) || compareFractions(left.end, right.end));
  return {
    hairpins: merged.length > 0 ? merged : undefined,
  };
}

function validateModifierLegality(
  token: TokenGlyph,
  contextTrack: TrackName | "ANONYMOUS",
  errors: ScoreAst["errors"],
  line: number,
): void {
  if (token.kind === "basic") {
    const resolved = resolveToken(token, contextTrack);
    if (!resolved) return;

    for (const modifier of resolved.modifiers) {
      if (!TRACKS_BY_MODIFIER[modifier].has(resolved.track)) {
        errors.push({
          line,
          column: 1,
          message: `Modifier \`${modifier}\` is not allowed on track \`${resolved.track}\``,
        });
      }
    }
    return;
  }

  if (token.kind === "combined" || token.kind === "group") {
    // Check if group has modifiers and contains a rest (invalid)
    if (token.kind === "group" && token.modifiers.length > 0) {
      const containsRest = token.items.some((item) => item.kind === "basic" && item.value === "-");
      if (containsRest) {
        errors.push({
          line,
          column: 1,
          message: `Rest cannot have articulation modifiers`,
        });
      }
    }
    token.items.forEach((item) => validateModifierLegality(item, contextTrack, errors, line));
    return;
  }

  if (token.kind === "braced") {
    token.items.forEach((item) => validateModifierLegality(item, token.track as TrackName, errors, line));
  }
}

export function normalizeScoreAst(ast: ScoreAst): NormalizedScore {
  const measures: NormalizedScore["measures"] = [];
  const voltaSeeds: (NormalizedScore["measures"][number]["volta"] | undefined)[] = [];
  const voltaTerminators: boolean[] = [];
  const trackHairpinStates = new Map<string, HairpinState>();
  const completedHairpinsByStartMeasure = new Map<number, HairpinIntent[]>();
  let globalMeasureIndex = 0;

  for (const [paragraphIndex, paragraph] of ast.paragraphs.entries()) {
    for (let measureInParagraph = 0; measureInParagraph < paragraph.measureCount; measureInParagraph += 1) {
      const events: NormalizedEvent[] = [];
      let sourceLine = 0;
      const trackMeasures = paragraph.tracks
        .map((trackLine) => trackLine.measures[measureInParagraph])
        .filter((measure): measure is NonNullable<typeof measure> => measure !== undefined);
      const resolvedTrackNavs: Array<{ startNav?: StartNav; endNav?: EndNav; sourceLine: number }> = [];
      for (const trackLine of paragraph.tracks) {
        const measure = trackLine.measures[measureInParagraph];
        if (!measure) continue;

        sourceLine = measure.sourceLine || sourceLine;
        const overflowTokenInMeasure = measure.tokens
          .map((token) => findDurationOverflowToken(token))
          .find((token): token is Extract<TokenGlyph, { kind: "basic" }> => token !== null);

        if (overflowTokenInMeasure) {
          ast.errors.push({
            line: measure.sourceLine || 0,
            column: 1,
            message: `Token \`${overflowTokenInMeasure.value}\` exceeds the exact duration range under current modifier counts`,
          });
          resolvedTrackNavs.push({
            startNav: resolveParsedStartNav(measure.startNav, []),
            endNav: resolveParsedEndNav(measure.endNav, []),
            sourceLine: measure.sourceLine || sourceLine,
          });
          continue;
        }

        const trackKey = "__global_hairpin_state__";
        const priorState = trackHairpinStates.get(trackKey) ?? {
          activeType: null,
          activeStart: null,
          startMeasureIndex: null,
        };
        const trackHairpins = collectTrackHairpins(
          measure,
          globalMeasureIndex,
          paragraph.noteValue,
          ast.headers.time,
          priorState,
          ast.errors,
        );
        trackHairpinStates.set(trackKey, priorState);
        for (const hairpin of trackHairpins.hairpins) {
          const existing = completedHairpinsByStartMeasure.get(hairpin.startMeasureIndex) ?? [];
          completedHairpinsByStartMeasure.set(hairpin.startMeasureIndex, [...existing, hairpin]);
        }

        const activeNoteValue = paragraph.noteValue;
        const divisions = (ast.headers.time.beats * activeNoteValue) / ast.headers.time.beatUnit;
        const slotDuration = simplify({
          numerator: 1,
          denominator: activeNoteValue,
        });

        let currentSlotOffset: Fraction = { numerator: 0, denominator: 1 };
        const divisionsFrac: Fraction = { numerator: divisions, denominator: 1 };
        const tokenStarts: Fraction[] = [];
        
        for (const token of measure.tokens) {
          validateModifierLegality(token, trackLine.track as TrackName | "ANONYMOUS", ast.errors, measure.sourceLine || 0);

          const weight = calculateTokenWeightAsFraction(token);
          const tokenStart = multiplyFractions(slotDuration, currentSlotOffset);
          const tokenDuration = multiplyFractions(slotDuration, weight);
          tokenStarts.push(tokenStart);

          // Validation: Check grouping boundaries
          const startSlot = currentSlotOffset;
          const endSlot = addFractions(currentSlotOffset, weight);
          
          let cumulativeGrouping = 0;
          for (const groupSize of ast.headers.grouping.values) {
            cumulativeGrouping += groupSize;
            const boundaryFrac = groupingBoundaryInSlots(
              cumulativeGrouping,
              ast.headers.time.beats,
              divisions,
            );
            
            // startSlot < boundaryFrac AND endSlot > boundaryFrac
            if (compareFractions(startSlot, boundaryFrac) < 0 && compareFractions(endSlot, boundaryFrac) > 0) {
              ast.errors.push({
                line: measure.sourceLine || 0,
                column: 1,
                message: `Token \`${token.kind === "basic" ? token.value : "group"}\` crosses grouping boundary at ${cumulativeGrouping} in track ${trackLine.track}`,
              });
            }
          }

          events.push(
            ...tokenToEvents(
              token,
              tokenStart,
              tokenDuration,
              trackLine.track as TrackName | "ANONYMOUS",
              paragraphIndex,
              globalMeasureIndex,
              measureInParagraph,
            ),
          );
          currentSlotOffset = addFractions(currentSlotOffset, weight);
        }

        // Pad if measure is short (validation)
        if (
          measure.measureRepeat === undefined &&
          measure.multiRest === undefined &&
          !fractionsEqual(currentSlotOffset, divisionsFrac)
        ) {
          ast.errors.push({
            line: measure.sourceLine || 0,
            column: 1,
            message: `Track \`${trackLine.track}\` measure ${measureInParagraph + 1} has invalid duration: used ${currentSlotOffset.numerator}/${currentSlotOffset.denominator} slots, expected ${divisions}`,
          });
        }

        resolvedTrackNavs.push({
          startNav: resolveParsedStartNav(measure.startNav, tokenStarts),
          endNav: resolveParsedEndNav(measure.endNav, tokenStarts),
          sourceLine: measure.sourceLine || sourceLine,
        });
      }

      const mergedRepeatStart = trackMeasures.some((measure) => measure.repeatStart);
      const mergedRepeatEnd = trackMeasures.some((measure) => measure.repeatEnd);
      const mergedMeasureRepeat = trackMeasures.find((measure) => measure.measureRepeat !== undefined)?.measureRepeat;
      const mergedMultiRest = trackMeasures.find((measure) => measure.multiRest !== undefined)?.multiRest;
      let mergedBarline =
        mergedRepeatStart && mergedRepeatEnd
          ? "repeat-both"
          : mergedRepeatStart
            ? "repeat-start"
            : mergedRepeatEnd
              ? "repeat-end"
              : trackMeasures.find((measure) => measure.barline === "final")?.barline
                ?? trackMeasures.find((measure) => measure.barline === "double")?.barline
                ?? trackMeasures.find((measure) => measure.barline !== undefined)?.barline;

      const startNavKey = new Set(resolvedTrackNavs.map((item) => navKey(item.startNav)).filter((value): value is string => value !== undefined));
      const endNavKey = new Set(resolvedTrackNavs.map((item) => navKey(item.endNav)).filter((value): value is string => value !== undefined));

      if (startNavKey.size > 1) {
        ast.errors.push({
          line: resolvedTrackNavs[0]?.sourceLine ?? sourceLine,
          column: 1,
          message: `Conflicting start-side navigation at bar ${globalMeasureIndex + 1}`,
        });
      }

      if (endNavKey.size > 1) {
        ast.errors.push({
          line: resolvedTrackNavs[0]?.sourceLine ?? sourceLine,
          column: 1,
          message: `Conflicting end-side navigation at bar ${globalMeasureIndex + 1}`,
        });
      }

      const mergedStartNav = resolvedTrackNavs.find((item) => item.startNav !== undefined)?.startNav;
      const mergedEndNav = resolvedTrackNavs.find((item) => item.endNav !== undefined)?.endNav;

      if (
        mergedEndNav !== undefined &&
        mergedEndNav.kind !== "to-coda" &&
        (mergedBarline === "repeat-end" || mergedBarline === "repeat-both")
      ) {
        ast.errors.push({
          line: resolvedTrackNavs.find((item) => item.endNav !== undefined)?.sourceLine ?? sourceLine,
          column: 1,
          message: `End-side navigation \`${mergedEndNav.kind}\` cannot appear on a repeat-ending bar ${globalMeasureIndex + 1}`,
        });
      }

      if (mergedEndNav?.kind === "fine") {
        mergedBarline = "final";
      } else if (
        mergedEndNav
        && mergedEndNav.kind !== "to-coda"
        && mergedBarline !== "final"
        && mergedBarline !== "double"
        && mergedBarline !== "repeat-end"
        && mergedBarline !== "repeat-both"
      ) {
        mergedBarline = "double";
      }

      const measureMeta = trackMeasures.length === 0
        ? undefined
        : {
            generated: trackMeasures.every((measure) => measure.generated),
            barline: mergedBarline,
            startNav: mergedStartNav,
            endNav: mergedEndNav,
            volta: trackMeasures.find((measure) => measure.volta !== undefined)?.volta,
            voltaTerminator: trackMeasures.some((measure) => measure.voltaTerminator === true),
            measureRepeat: mergedMeasureRepeat,
            multiRest: mergedMultiRest,
            multiRestCount: mergedMultiRest?.count,
          };

      measures.push({
        index: globalMeasureIndex,
        globalIndex: globalMeasureIndex,
        paragraphIndex,
        measureInParagraph,
        sourceLine,
        events,
        generated: measureMeta?.generated,
        barline: measureMeta?.barline,
        startNav: measureMeta?.startNav,
        endNav: measureMeta?.endNav,
        volta: measureMeta?.volta,
        measureRepeat: measureMeta?.measureRepeat,
        multiRest: measureMeta?.multiRest,
        multiRestCount: measureMeta?.multiRestCount,
        noteValue: paragraph.noteValue,
      });
      voltaSeeds.push(measureMeta?.volta ? { indices: [...measureMeta.volta.indices] } : undefined);
      voltaTerminators.push(measureMeta?.voltaTerminator === true);
      globalMeasureIndex++;
    }
  }

  let activeVolta: NormalizedScore["measures"][number]["volta"] | undefined;
  for (let index = 0; index < measures.length; index += 1) {
    const measure = measures[index];
    if (!measure) continue;

    const seed = voltaSeeds[index];
    if (seed) {
      activeVolta = { indices: [...seed.indices] };
    }

    measure.volta = activeVolta ? { indices: [...activeVolta.indices] } : undefined;

    if (
      voltaTerminators[index]
      || measure.barline === "repeat-end"
      || measure.barline === "repeat-both"
    ) {
      activeVolta = undefined;
    }
  }

  // Ensure the last measure has a final barline if not otherwise specified
  if (measures.length > 0) {
    const lastMeasure = measures[measures.length - 1];
    if (lastMeasure && (lastMeasure.barline === undefined || lastMeasure.barline === "regular")) {
      lastMeasure.barline = "final";
    }
  }

  if (measures.length > 0) {
    const finalMeasureDuration = {
      numerator: ast.headers.time.beats,
      denominator: ast.headers.time.beatUnit,
    };
    for (const state of trackHairpinStates.values()) {
      if (!state.activeType || !state.activeStart || state.startMeasureIndex === null) continue;
      const existing = completedHairpinsByStartMeasure.get(state.startMeasureIndex) ?? [];
      pushHairpinFragment(state, existing, measures.length - 1, finalMeasureDuration);
      completedHairpinsByStartMeasure.set(state.startMeasureIndex, existing);
      state.activeType = null;
      state.activeStart = null;
      state.startMeasureIndex = null;
    }
  }

  for (const [startMeasureIndex, hairpins] of completedHairpinsByStartMeasure) {
    const merged = mergeMeasureHairpins(
      startMeasureIndex,
      measures[startMeasureIndex]?.sourceLine ?? 1,
      hairpins,
      ast.errors,
    );
    if (measures[startMeasureIndex]) {
      measures[startMeasureIndex]!.hairpins = merged.hairpins;
    }
  }

  const globalNoteValue = ast.headers.note?.value ?? 
    (ast.headers.divisions ? (ast.headers.divisions.value * ast.headers.time.beatUnit / ast.headers.time.beats) : 16);

  const header: NormalizedHeader = {
    ...(ast.headers.title ? { title: ast.headers.title.value } : {}),
    ...(ast.headers.subtitle ? { subtitle: ast.headers.subtitle.value } : {}),
    ...(ast.headers.composer ? { composer: ast.headers.composer.value } : {}),
    tempo: ast.headers.tempo.value,
    timeSignature: {
      beats: ast.headers.time.beats,
      beatUnit: ast.headers.time.beatUnit,
    },
    divisions: ast.headers.divisions?.value ?? ((ast.headers.time.beats * globalNoteValue) / ast.headers.time.beatUnit),
    noteValue: globalNoteValue,
    grouping: [...ast.headers.grouping.values],
  };

  const trackIds = new Set<TrackName>();
  for (const paragraph of ast.paragraphs) {
    for (const track of paragraph.tracks) {
      if (track.track !== "ANONYMOUS") {
        trackIds.add(track.track);
      }
    }
  }

  const tracks: NormalizedTrack[] = [...trackIds].map((id) => ({
    id,
    family: getTrackFamily(id),
  }));

  return {
    version: "1.0",
    header,
    tracks,
    ast,
    measures,
    errors: ast.errors,
  };
}

export function buildNormalizedScore(source: string, parseMode: ParseMode = "lezer"): NormalizedScore {
  if (parseMode === "wasm") {
    return buildNormalizedScoreWasm(source);
  }
  return normalizeScoreAst(buildScoreAst(source, parseMode));
}

export function buildNormalizedScoreWasm(source: string): NormalizedScore {
  const raw = wasmNormalize(source) as any;
  // If raw is an errors array (parse failed), wrap it
  if (Array.isArray(raw)) {
    return {
      version: "1.0",
      header: {
        tempo: 120,
        timeSignature: { beats: 4, beatUnit: 4 },
        divisions: 16,
        noteValue: 8,
        grouping: [1],
      },
      tracks: [],
      ast: {} as any,
      measures: [],
      errors: raw.map((e: any) => ({ line: e.line ?? 1, column: e.column ?? 1, message: e.message ?? String(e) })),
    };
  }

  // Adapt WASM NormalizedScore to JS NormalizedScore
  const header = raw.header || {};
  return {
    version: raw.version || "1.0",
    header: {
      title: header.title,
      subtitle: header.subtitle,
      composer: header.composer,
      tempo: header.tempo || 120,
      timeSignature: header.timeSignature || { beats: 4, beatUnit: 4 },
      divisions: header.divisions || 16,
      noteValue: header.noteValue || 8,
      grouping: header.grouping || [1],
    },
    tracks: (raw.tracks || []).map((t: any) => ({
      id: t.id,
      family: t.family,
    })),
    ast: {} as any,
    measures: (raw.measures || []).map((m: any, i: number) => ({
      index: m.index ?? i,
      globalIndex: m.globalIndex ?? i,
      paragraphIndex: m.paragraphIndex ?? 0,
      measureInParagraph: m.measureInParagraph ?? 0,
      sourceLine: m.sourceLine ?? 0,
      events: (m.events || []).map((ev: any) => ({
        track: ev.track || "HH",
        paragraphIndex: ev.paragraphIndex ?? 0,
        measureIndex: ev.measureIndex ?? 0,
        measureInParagraph: ev.measureInParagraph ?? 0,
        start: ev.start || { numerator: 0, denominator: 1 },
        duration: ev.duration || { numerator: 0, denominator: 1 },
        kind: ev.kind || "hit",
        glyph: ev.glyph || "x",
        modifiers: ev.modifiers || [],
        modifier: ev.modifier,
        voice: (ev.voice || 1) as 1 | 2,
        beam: ev.beam || "none",
        tuplet: ev.tuplet,
      })),
      generated: m.generated ?? false,
      barline: m.barline,
      startNav: m.startNav ? { kind: m.startNav, anchor: "left-edge" as const } : undefined,
      endNav: m.endNav ? { kind: m.endNav, anchor: "right-edge" as const } : undefined,
      volta: m.volta ? { indices: m.volta } : undefined,
      hairpins: (m.hairpins || []).map((h: any) => ({
        type: h.kind === "crescendo" ? "crescendo" : "decrescendo",
        start: h.start || { numerator: 0, denominator: 1 },
        startMeasureIndex: h.startMeasureIndex ?? 0,
        end: h.end || { numerator: 0, denominator: 1 },
        endMeasureIndex: h.endMeasureIndex ?? 0,
      })),
      measureRepeat: m.measureRepeatSlashes ? { slashes: m.measureRepeatSlashes } : undefined,
      multiRest: m.multiRestCount ? { count: m.multiRestCount } : undefined,
      noteValue: m.noteValue ?? 8,
    })),
    errors: (raw.errors || []).map((e: any) => ({
      line: e.line ?? 1,
      column: e.column ?? 1,
      message: typeof e === "string" ? e : (e.message || "unknown error"),
    })),
  };
}
