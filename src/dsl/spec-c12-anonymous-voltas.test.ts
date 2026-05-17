import { describe, expect, it } from "vitest";
import { buildScoreAst } from "./ast";
import { buildNormalizedScore } from "./normalize";

describe("anonymous-track voltas", () => {
  it("parses `:|2.` and spaced `| 3.` as alternate endings on anonymous tracks", () => {
    const source = `title Repeat Structure
time 4/4
divisions 4

|: x x x x |1. x x x o :|2. x x/x/ x x |

| x/x/ x x x/x/ | 3. o o o o |`;

    const ast = buildScoreAst(source);
    expect(ast.errors).toEqual([]);
    expect(ast.repeatSpans).toEqual([{ startBar: 0, endBar: 1, times: 2 }]);

    const score = buildNormalizedScore(source);
    expect(score.errors).toEqual([]);
    expect(score.measures.map((measure) => ({
      barline: measure.barline,
      volta: measure.volta?.indices,
    }))).toEqual([
      { barline: "repeat-start", volta: undefined },
      { barline: "repeat-end", volta: [1] },
      { barline: "regular", volta: [2] },
      { barline: "regular", volta: [2] },
      { barline: "final", volta: [3] },
    ]);
  });

  it("continues an ending bracket across a paragraph break after repeat-end", () => {
    const source = `time 4/4
divisions 8

|: ssss ssss |1. ssSs ssSs :|2. cCcc cccc |

| ssss ssss |3. bbbb bbbb |.`;

    const score = buildNormalizedScore(source);
    expect(score.errors).toEqual([]);
    expect(score.measures.map((measure) => ({
      barline: measure.barline,
      paragraph: measure.paragraphIndex,
      volta: measure.volta?.indices,
    }))).toEqual([
      { barline: "repeat-start", paragraph: 0, volta: undefined },
      { barline: "repeat-end", paragraph: 0, volta: [1] },
      { barline: "regular", paragraph: 0, volta: [2] },
      { barline: "regular", paragraph: 1, volta: [2] },
      { barline: "final", paragraph: 1, volta: [3] },
    ]);
  });

  it("reports repeat-end inside a continuing ending at the offending :|", () => {
    const source = `time 4/4
divisions 8

|: ssss ssss |1. ssSs ssSs :|2. cCcc cccc :|

| ssss ssss |3. bbbb bbbb |.`;

    const score = buildNormalizedScore(source);
    expect(score.errors).toContainEqual({
      line: 4,
      column: 43,
      message: "Repeat end cannot appear before volta 2 continues; move `:|` to the ending's final measure or terminate the volta before it",
    });
  });
});
