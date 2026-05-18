import type { NormalizedScore } from "./dsl";

export type ParsedScoreState = {
  score: NormalizedScore;
  source: string;
  sourceRevision: number;
};

export type ParsedScoreResult = ParsedScoreState;

export function createParsedScoreState(
  score: NormalizedScore,
  source: string,
  sourceRevision: number,
): ParsedScoreState {
  return { score, source, sourceRevision };
}

export function acceptCurrentParsedScoreResult(
  currentSourceRevision: number,
  result: ParsedScoreResult,
): ParsedScoreResult | null {
  if (result.sourceRevision !== currentSourceRevision) {
    return null;
  }
  return result;
}
