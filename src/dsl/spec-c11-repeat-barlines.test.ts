import { describe, expect, it } from "vitest";
import { buildScoreAst } from "./ast";
import { buildNormalizedScore } from "./normalize";
import { parseDocumentSkeleton } from "./parser";
import { parseDocumentSkeletonFromWasmSync } from "../wasm/skeleton";

describe("spec C11: repeat barlines", () => {
  it("distinguishes regular, repeat-start, repeat-end, and final barlines in parser output", () => {
    const doc = parseDocumentSkeleton(`time 4/4
divisions 4

|: x | s :| b |.`);

    expect(doc.errors).toEqual([]);
    const measures = doc.paragraphs[0].lines[0].measures;

    expect(measures).toHaveLength(3);
    expect(measures[0]).toMatchObject({
      content: "x",
      repeatStart: true,
      repeatEnd: false,
      voltaTerminator: undefined,
    });
    expect(measures[1]).toMatchObject({
      content: "s",
      repeatStart: false,
      repeatEnd: true,
      repeatTimes: 2,
      voltaTerminator: undefined,
    });
    expect(measures[2]).toMatchObject({
      content: "b",
      repeatStart: false,
      repeatEnd: false,
      barline: undefined,
      voltaTerminator: true,
    });
  });

  it("treats `|.` as a volta terminator without forcing a final barline", () => {
    const score = buildNormalizedScore(`time 4/4
divisions 4

|1. x - - - |. x - - - |`);

    expect(score.errors).toEqual([]);
    expect(score.measures).toHaveLength(2);
    expect(score.measures[0]).toMatchObject({
      barline: "regular",
      volta: { indices: [1] },
    });
    expect(score.measures[1]).toMatchObject({
      barline: "final",
    });
    expect(score.measures[1]?.volta).toBeUndefined();
  });

  it("infers a repeat-end barline when a volta is followed by a different next volta", () => {
    const doc = parseDocumentSkeleton(`time 4/4
divisions 4

|: x - - - |1. x - - - |2. o - - - |3. c - - - |`);

    expect(doc.errors).toEqual([]);
    const measures = doc.paragraphs[0].lines[0].measures;

    expect(measures[1]).toMatchObject({
      repeatStart: false,
      repeatEnd: true,
      repeatTimes: 2,
      voltaIndices: [1],
    });
    expect(measures[2]).toMatchObject({
      repeatStart: false,
      repeatEnd: true,
      repeatTimes: 2,
      voltaIndices: [2],
    });
    expect(measures[3]).toMatchObject({
      repeatEnd: false,
      repeatTimes: undefined,
      voltaIndices: [3],
    });
  });

  it("treats `|: :|` as a single repeat-both empty measure", () => {
    const ast = buildScoreAst(`time 4/4
divisions 4

|: :|`);

    expect(ast.errors).toEqual([]);
    expect(ast.repeatSpans).toEqual([{ startBar: 0, endBar: 0, times: 2 }]);

    const measure = ast.paragraphs[0].tracks[0].measures[0];
    expect(measure).toMatchObject({
      repeatStart: true,
      repeatEnd: true,
      generated: true,
      barline: "repeat-both",
    });
    expect(measure.tokens).toHaveLength(4);
  });

  it("parses compact `:|:` as a shared repeat-end and repeat-start boundary", () => {
    const doc = parseDocumentSkeleton(`time 4/4
divisions 4

|: ssss :|: ssss :|`);

    expect(doc.errors).toEqual([]);
    const measures = doc.paragraphs[0].lines[0].measures;
    expect(measures).toHaveLength(2);
    expect(measures[0]).toMatchObject({
      repeatStart: true,
      repeatEnd: true,
    });
    expect(measures[1]).toMatchObject({
      repeatStart: true,
      repeatEnd: true,
    });
  });

  it("treats `||` as a double barline with no empty measure between", () => {
    const score = buildNormalizedScore(`time 4/4
divisions 4

| x || s |`);

    expect(score.errors).toEqual([]);
    expect(score.measures).toHaveLength(2);
    expect(score.measures[0]).toMatchObject({
      barline: "double",
      generated: false,
    });
    expect(score.measures[1]).toMatchObject({
      barline: "final",
      generated: false,
    });
  });

  it("treats `|  |` as an empty generated measure between two regular barlines", () => {
    const score = buildNormalizedScore(`time 4/4
divisions 4

| x |  | s |`);

    expect(score.errors).toEqual([]);
    expect(score.measures).toHaveLength(3);
    expect(score.measures[0]).toMatchObject({
      barline: "regular",
      generated: false,
    });
    expect(score.measures[1]).toMatchObject({
      barline: "regular",
      generated: true,
    });
    expect(score.measures[1].events).toEqual([]);
    expect(score.measures[2]).toMatchObject({
      barline: "final",
      generated: false,
    });
  });
});

describe("spec C11: |:. volta-terminator + repeat-start compound barline", () => {
  it("parses |:. as a barline with repeatStart: true and sets voltaTerminator on the preceding measure", () => {
    const doc = parseDocumentSkeletonFromWasmSync(`time 4/4
divisions 4

|: x - - - |1. x - - - |:. x - - - :|`);

    expect(doc.errors).toEqual([]);
    const measures = doc.paragraphs[0].lines[0].measures;

    expect(measures).toHaveLength(3);
    expect(measures[0]).toMatchObject({ repeatStart: true });
    expect(measures[1]).toMatchObject({
      voltaIndices: [1],
      voltaTerminator: true,
    });
    expect(measures[2]).toMatchObject({
      repeatStart: true,
      repeatEnd: true,
    });
  });

  it("produces repeat-both when |:. is followed by :|", () => {
    const score = buildNormalizedScore(`time 4/4
divisions 4

|:. x - - - :|`);

    expect(score.errors).toEqual([]);
    expect(score.measures).toHaveLength(1);
    expect(score.measures[0]).toMatchObject({
      barline: "repeat-both",
    });
    expect(score.measures[0].volta).toBeUndefined();
  });

  it("produces correct repeat spans with voltas and |:.", () => {
    const score = buildNormalizedScore(`time 4/4
divisions 4

|: x - - - |1. x - - - :|2. x - - - |:. x - - - :|`);

    expect(score.errors).toEqual([]);
    expect(score.ast.repeatSpans).toEqual([
      { startBar: 0, endBar: 1, times: 2 },
      { startBar: 3, endBar: 3, times: 2 },
    ]);
    expect(score.measures[2]).toMatchObject({
      volta: { indices: [2] },
      barline: "regular",
    });
    expect(score.measures[3]).toMatchObject({
      barline: "repeat-both",
      volta: undefined,
    });
  });

  it("reports nested repeat start when |:. opens repeat without closing prior repeat", () => {
    const score = buildNormalizedScore(`time 4/4
divisions 4

|: x - - - |1. x - - - |:. x - - - :|`);

    expect(score.errors.length).toBeGreaterThan(0);
    expect(score.errors[0].message).toContain("Nested repeat start");
  });

  it("is a no-op for volta-termination when no active volta exists", () => {
    const score = buildNormalizedScore(`time 4/4
divisions 4

|:. x - - - :|`);

    expect(score.errors).toEqual([]);
    expect(score.measures[0]).toMatchObject({
      barline: "repeat-both",
      volta: undefined,
    });
  });

  it("parses compact :|: as repeat-end followed by repeat-start", () => {
    const score = buildNormalizedScore(`time 4/4
divisions 4

|: ssss :|: ssss :|`);

    expect(score.errors).toEqual([]);
    expect(score.ast.repeatSpans).toEqual([
      { startBar: 0, endBar: 0, times: 2 },
      { startBar: 1, endBar: 1, times: 2 },
    ]);
    expect(score.measures).toHaveLength(2);
    expect(score.measures[0]).toMatchObject({ barline: "repeat-both" });
    expect(score.measures[1]).toMatchObject({ barline: "repeat-both" });
  });

  it("does not interfere with implicit repeat-end inference", () => {
    const score = buildNormalizedScore(`time 4/4
divisions 4

|: x - - - |1. x - - - |2. o - - - |:. c - - - :|`);

    expect(score.errors).toEqual([]);
    expect(score.ast.repeatSpans).toEqual([
      { startBar: 0, endBar: 1, times: 2 },
      { startBar: 3, endBar: 3, times: 2 },
    ]);
    expect(score.measures[1]).toMatchObject({
      barline: "repeat-end",
      volta: { indices: [1] },
    });
    expect(score.measures[2]).toMatchObject({
      barline: "regular",
      volta: { indices: [2] },
    });
    expect(score.measures[3]).toMatchObject({
      barline: "repeat-both",
      volta: undefined,
    });
  });
});
