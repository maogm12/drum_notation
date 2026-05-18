import { describe, expect, it } from "vitest";
import { buildNormalizedScore } from "../dsl/normalize";
import { buildLayoutSceneFromSource } from "../renderer/svgRenderer";

type Scene = Awaited<ReturnType<typeof buildLayoutSceneFromSource>>;

const successfulFixtures = [
  {
    name: "basic timing and beams",
    source: `time 4/4
note 1/8
grouping 2+2

HH | x - x - |
BD | b - - - |
`,
  },
  {
    name: "barlines and repeats",
    source: `time 4/4
note 1/8
grouping 2+2

|: x x x x | x x x x :| % |
`,
  },
  {
    name: "navigation markers",
    source: `time 4/4
note 1/4
grouping 1+1+1+1

| @segno x x x x | x x x @fine | @dc |
`,
  },
  {
    name: "volta and hairpin",
    source: `time 4/4
note 1/8
grouping 2+2

|: < x x x x ! |1. x x x x :|2. x x x x |
`,
  },
  {
    name: "multi-measure input",
    source: `time 4/4
note 1/4
grouping 1+1+1+1

| x x x x |

| --2-- |
`,
  },
] as const;

const invalidFixtures = [
  {
    name: "bad time signature",
    source: `time 4
HH | x |
`,
  },
  {
    name: "bad tuplet",
    source: `time bad
HH | x |
`,
  },
] as const;

function sceneMeasures(scene: Scene) {
  return scene.pages.flatMap((page) => page.measures);
}

function countSceneRole(scene: Scene, role: string) {
  return scene.pages
    .flatMap((page) => page.items)
    .filter((item) => item.role === role).length;
}

function countSceneComposite(scene: Scene, kind: string) {
  return scene.pages
    .flatMap((page) => page.composites)
    .filter((composite) => composite.kind === kind).length;
}

describe("parser/layout wasm semantic parity", () => {
  it.each(successfulFixtures)("keeps structural shape for $name", async ({ source }) => {
    const score = buildNormalizedScore(source);
    const scene = await buildLayoutSceneFromSource(source, {
      pageWidth: 612,
      pageHeight: 792,
      staffScale: 1,
      showTitle: true,
    });

    expect(score.errors).toEqual([]);
    expect(sceneMeasures(scene)).toHaveLength(score.measures.length);

    const scoreHasRepeat = score.measures.some(
      (measure) => Boolean(measure.measureRepeatSlashes)
        || String(measure.barline ?? "").includes("repeat"),
    );
    if (scoreHasRepeat) {
      expect(
        countSceneRole(scene, "repeat-start")
        + countSceneRole(scene, "repeat-end")
        + countSceneRole(scene, "measure-repeat")
        + countSceneRole(scene, "barline"),
      ).toBeGreaterThan(0);
    }

    const scoreHasNavigation = score.measures.some((measure) => measure.startNav || measure.endNav);
    if (scoreHasNavigation) {
      expect(countSceneComposite(scene, "navigation")).toBeGreaterThan(0);
    }

    const scoreHasVolta = score.measures.some((measure) => (measure.volta?.length ?? 0) > 0);
    if (scoreHasVolta) {
      expect(countSceneComposite(scene, "volta")).toBeGreaterThan(0);
    }

    const scoreHasHairpin = score.measures.some((measure) => (measure.hairpins?.length ?? 0) > 0);
    if (scoreHasHairpin) {
      expect(countSceneComposite(scene, "hairpin")).toBeGreaterThan(0);
    }

    const scoreHasBeamedEvents = score.measures.some((measure) =>
      measure.events.some((event) => event.beam && event.beam !== "none"),
    );
    if (scoreHasBeamedEvents) {
      expect(countSceneRole(scene, "beam")).toBeGreaterThan(0);
    }
  });

  it.each(invalidFixtures)("keeps failure classification for $name", async ({ source }) => {
    const score = buildNormalizedScore(source);

    expect(score.errors.length).toBeGreaterThan(0);
    await expect(buildLayoutSceneFromSource(source)).rejects.toThrow();
  });
});
