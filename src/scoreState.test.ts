import { describe, expect, it } from "vitest";
import type { NormalizedScore } from "./dsl";
import {
  acceptCurrentParsedScoreResult,
  createParsedScoreState,
} from "./scoreState";

const score = { errors: [] } as unknown as NormalizedScore;

describe("parsed score state", () => {
  it("keeps score source and source revision together", () => {
    expect(createParsedScoreState(score, "time 4/4", 3)).toEqual({
      score,
      source: "time 4/4",
      sourceRevision: 3,
    });
  });

  it("rejects stale parse results during rapid edits", () => {
    const stale = createParsedScoreState(score, "time 3/4", 1);
    const current = createParsedScoreState(score, "time 4/4", 2);

    expect(acceptCurrentParsedScoreResult(2, stale)).toBeNull();
    expect(acceptCurrentParsedScoreResult(2, current)).toBe(current);
  });
});
