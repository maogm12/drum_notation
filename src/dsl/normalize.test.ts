import { describe, expect, it } from "vitest";
import { buildNormalizedScore } from "./normalize";

describe("buildNormalizedScore", () => {
  it("resolves context-aware aliases and fallback instruments", () => {
    const score = buildNormalizedScore(`time 4/4
divisions 4

HH | x - - - |
SD | x - - - |
| x s b g |`);

    expect(score.errors).toEqual([]);
    
    // HH | x -> HH:d
    const hhEvent = score.measures[0].events.find(e => e.track === "HH" && e.start.numerator === 0);
    expect(hhEvent?.glyph).toBe("x"); // Cymbal family
    expect(hhEvent?.modifier).toBeUndefined();

    // SD | x -> SD:d:cross
    const sdEvent = score.measures[0].events.find(e => e.track === "SD" && e.start.numerator === 0);
    expect(sdEvent?.track).toBe("SD");
    expect(sdEvent?.modifier).toBe("cross");

    // Anonymous | x -> HH:d
    const hhAt0 = score.measures[0].events.filter(e => e.track === "HH" && e.start.numerator === 0);
    expect(hhAt0).toHaveLength(2);

    // Anonymous | s -> SD:d at slot 1 (1/4)
    const anonS = score.measures[0].events.find(e => e.track === "SD" && e.start.numerator === 1 && e.start.denominator === 4);
    expect(anonS).toBeDefined();

    // Anonymous | b -> BD:d at slot 2 (2/4 -> 1/2)
    const anonB = score.measures[0].events.find(e => e.track === "BD" && e.start.numerator === 1 && e.start.denominator === 2);
    expect(anonB).toBeDefined();

    // Anonymous | g -> SD:d:ghost at slot 3 (3/4)
    const anonG = score.measures[0].events.find(e => e.track === "SD" && e.start.numerator === 3);
    expect(anonG?.modifier).toBe("ghost");
  });

  it("handles braced scopes and timing accurately", () => {
    const score = buildNormalizedScore(`time 4/4
divisions 4

| @RC{d d} @SD{[d d d]} |`);


    expect(score.errors).toEqual([]);
    const events = score.measures[0].events;
    
    // @RC{d d} takes 2 slots (0, 1)
    expect(events.filter(e => e.track === "RC")).toHaveLength(2);
    expect(events.find(e => e.track === "RC" && e.start.numerator === 0)).toBeDefined();
    expect(events.find(e => e.track === "RC" && e.start.numerator === 1)).toBeDefined();

    // @SD{[d d d]} takes 1 slot (at slot 2)
    const sdEvents = events.filter(e => e.track === "SD");
    expect(sdEvents).toHaveLength(3);
    // First SD event at start 2/4 (simplified to 1/2)
    expect(sdEvents[0].start).toMatchObject({ numerator: 1, denominator: 2 });
  });

  it("merges multiple modifiers correctly (accent priority)", () => {
    const score = buildNormalizedScore(`time 4/4
divisions 4
| s:rim:accent |`);

    const event = score.measures[0].events[0];
    expect(event.track).toBe("SD");
    expect(event.kind).toBe("hit");
    expect(event.modifiers).toContain("accent");
    expect(event.modifier).toBe("rim");
  });

  it("normalizes expanded summon tokens and track families", () => {
    const score = buildNormalizedScore(`time 9/4
divisions 9
grouping 1+1+1+1+1+1+1+1+1

| b2 r2 c2 t4 spl chn cb wb cl |`);

    expect(score.errors).toEqual([]);
    expect(score.measures[0].events.map((event) => event.track)).toEqual([
      "BD2",
      "RC2",
      "C2",
      "T4",
      "SPL",
      "CHN",
      "CB",
      "WB",
      "CL",
    ]);
    expect(score.tracks.map((track) => track.id)).toEqual([
      "BD2",
      "RC2",
      "C2",
      "T4",
      "SPL",
      "CHN",
      "CB",
      "WB",
      "CL",
    ]);
  });

  it("carries structured measure metadata into normalized measures", () => {
    const score = buildNormalizedScore(`time 4/4
divisions 4

|: x x x x :| @segno % |1. x - - - | --2-- @to-coda |.`);

    expect(score.errors).toEqual([]);
    expect(score.measures[0]).toMatchObject({ barline: "repeat-both" });
    expect(score.measures[1]).toMatchObject({
      startNav: { kind: "segno", anchor: "left-edge" },
      measureRepeat: { slashes: 1 },
    });
    expect(score.measures[2]).toMatchObject({
      volta: { indices: [1] },
    });
    expect(score.measures[3]).toMatchObject({
      endNav: { kind: "to-coda", anchor: "right-edge" },
      multiRest: { count: 2 },
    });
  });

  it("propagates volta metadata until a terminator, new volta, or repeat-both", () => {
    const score = buildNormalizedScore(`time 4/4
divisions 4

|: x x x x |1. x - - - | x - - - :|2. o - - - | x - - - |. x - - - |`);

    expect(score.errors).toEqual([]);
    expect(score.measures.map((measure) => ({
      barline: measure.barline,
      volta: measure.volta?.indices,
    }))).toEqual([
      { barline: "repeat-start", volta: undefined },
      { barline: "regular", volta: [1] },
      { barline: "repeat-end", volta: [1] },
      { barline: "regular", volta: [2] },
      { barline: "regular", volta: [2] },
      { barline: "final", volta: undefined },
    ]);
  });
});
