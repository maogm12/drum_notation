import { describe, expect, it } from "vitest";
import VexFlow from "vexflow/bravura";
import type { NormalizedScore } from "../dsl/types";

const { Glyphs, VoltaType } = VexFlow;
import { endNavText, measureRepeatGlyph, startNavText, voltaTypeForMeasure } from "./renderer";

function makeScore(): NormalizedScore {
  return {
    version: "1.0.0",
    header: {
      tempo: 120,
      timeSignature: { beats: 4, beatUnit: 4 },
      divisions: 4,
      grouping: [2, 2],
      noteValue: 16,
    },
    tracks: [],
    ast: {
      headers: {
        tempo: { field: "tempo", value: 120, line: 1 },
        time: { field: "time", beats: 4, beatUnit: 4, line: 2 },
        divisions: { field: "divisions", value: 4, line: 3 },
        grouping: { field: "grouping", values: [2, 2], line: 4 },
      },
      paragraphs: [],
      repeatSpans: [],
      errors: [],
    },
    measures: [
      {
        index: 0,
        globalIndex: 0,
        paragraphIndex: 0,
        measureInParagraph: 0,
        sourceLine: 1,
        events: [],
        noteValue: 16,
        volta: { indices: [1] },
      },
      {
        index: 1,
        globalIndex: 1,
        paragraphIndex: 0,
        measureInParagraph: 1,
        sourceLine: 1,
        events: [],
        noteValue: 16,
        volta: { indices: [1] },
      },
      {
        index: 2,
        globalIndex: 2,
        paragraphIndex: 0,
        measureInParagraph: 2,
        sourceLine: 1,
        events: [],
        noteValue: 16,
      },
    ],
    errors: [],
  };
}

describe("vexflow structural helpers", () => {
  it("maps navigation metadata to stave text labels", () => {
    expect(startNavText({ kind: "segno", anchor: "left-edge" })).toBe("Segno");
    expect(startNavText({ kind: "coda", anchor: "left-edge" })).toBe("Coda");
    expect(endNavText({ kind: "fine", anchor: "right-edge" })).toBe("Fine");
    expect(endNavText({ kind: "to-coda", anchor: "right-edge" })).toBe("To Coda");
    expect(endNavText({ kind: "dc-al-fine", anchor: "right-edge" })).toBe("D.C. al Fine");
  });

  it("maps measure-repeat intent to VexFlow repeat glyphs", () => {
    expect(measureRepeatGlyph(1)).toBe(Glyphs.repeat1Bar);
    expect(measureRepeatGlyph(2)).toBe(Glyphs.repeat2Bars);
  });

  it("derives volta shapes from canonical neighboring measures", () => {
    const score = makeScore();

    expect(voltaTypeForMeasure(score, score.measures[0]!)).toBe(VoltaType.BEGIN);
    expect(voltaTypeForMeasure(score, score.measures[1]!)).toBe(VoltaType.END);
    expect(voltaTypeForMeasure(score, score.measures[2]!)).toBeNull();
  });

  it("does not close a volta shape solely because a measure has repeat-end", () => {
    const score = makeScore();
    score.measures[1] = {
      ...score.measures[1]!,
      barline: "repeat-end",
    };
    score.measures[2] = {
      ...score.measures[2]!,
      volta: { indices: [1] },
    };

    expect(voltaTypeForMeasure(score, score.measures[1]!)).toBe(VoltaType.MID);
    expect(voltaTypeForMeasure(score, score.measures[2]!)).toBe(VoltaType.END);
  });
});
