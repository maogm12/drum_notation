// @vitest-environment jsdom

import { beforeAll, describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { EXAMPLE_CORPUS_FILES } from "../dsl/example_corpus";
import { buildLayoutSceneFromSource, renderSourcePagesToSvgs } from "./svgRenderer";
import { initParserWasmBrowser } from "../wasm/parser_wasm_browser";

const ROOT = dirname(dirname(fileURLToPath(import.meta.url)));
const REPO_ROOT = dirname(ROOT);
const REPORT_PATH = join(REPO_ROOT, "docs", "layout-corpus", "corpus_gate_report.json");
const REPRESENTATIVE_SCENE_FILES = [
  "docs/examples/headers.drum",
  "docs/examples/repeats.drum",
  "docs/examples/hairpins.drum",
  "docs/examples/multi-rest.drum",
  "docs/examples/modifiers.drum",
  "docs/examples/sticking.drum",
  "docs/examples/full-example.drum",
] as const;
type SceneSummary = {
  pages: number;
  systems: number;
  measures: number;
  items: number;
  composites: number;
  itemRoles: Record<string, number>;
  compositeKinds: Record<string, number>;
  fragmentKinds: Record<string, number>;
};

function sceneSummary(scene: any): SceneSummary {
  const itemRoles: Record<string, number> = {};
  const compositeKinds: Record<string, number> = {};
  const fragmentKinds: Record<string, number> = {};
  let systems = 0;
  let measures = 0;
  let items = 0;
  let composites = 0;

  for (const page of scene.pages || []) {
    systems += page.systems?.length || 0;
    measures += page.measures?.length || 0;
    items += page.items?.length || 0;
    composites += page.composites?.length || 0;
    for (const item of page.items || []) {
      itemRoles[item.role] = (itemRoles[item.role] || 0) + 1;
    }
    for (const composite of page.composites || []) {
      compositeKinds[composite.kind] = (compositeKinds[composite.kind] || 0) + 1;
      const fragmentKey = `${composite.kind}:${composite.fragment}`;
      fragmentKinds[fragmentKey] = (fragmentKinds[fragmentKey] || 0) + 1;
    }
  }

  return {
    pages: scene.pages?.length || 0,
    systems,
    measures,
    items,
    composites,
    itemRoles,
    compositeKinds,
    fragmentKinds,
  };
}

function countRole(svg: string, role: string): number {
  return (svg.match(new RegExp(`data-role="${role}"`, "g")) || []).length;
}

function svgSemanticSummary(svg: string): Record<string, number | string> {
  const semanticTextTokens = [...svg.matchAll(/<text[^>]*>([^<]*)<\/text>/g)]
    .map((match) => match[1].trim())
    .filter((text) => text.length > 0 && /[A-Za-z0-9@>%\u4e00-\u9fff]/.test(text))
    .sort();

  return {
    lineCount: (svg.match(/<line /g) || []).length,
    rectCount: (svg.match(/<rect /g) || []).length,
    textCount: (svg.match(/<text /g) || []).length,
    polylineCount: (svg.match(/<polyline /g) || []).length,
    openingBarlines: countRole(svg, "opening-barline"),
    genericBarlines: countRole(svg, "barline"),
    finalBarlineThin: countRole(svg, "final-barline-thin"),
    finalBarlineThick: countRole(svg, "final-barline-thick"),
    doubleBarlineLeft: countRole(svg, "double-barline-left"),
    doubleBarlineRight: countRole(svg, "double-barline-right"),
    noteheads: countRole(svg, "notehead"),
    stems: countRole(svg, "stem"),
    rests: countRole(svg, "rest"),
    measureRepeats: countRole(svg, "measure-repeat"),
    multiRestBars: countRole(svg, "multi-rest-bar"),
    multiRestCounts: countRole(svg, "multi-rest-count"),
    navStarts: countRole(svg, "nav-start"),
    navEnds: countRole(svg, "nav-end"),
    hairpinTop: countRole(svg, "hairpin-top"),
    hairpinBottom: countRole(svg, "hairpin-bottom"),
    repeatSpanLines: countRole(svg, "repeat-span-line"),
    voltaLines: countRole(svg, "volta-line"),
    sticking: countRole(svg, "sticking"),
    accents: countRole(svg, "accent"),
    semanticTextTokens: semanticTextTokens.join(" || "),
  };
}

describe("Layout corpus gate", () => {
  beforeAll(async () => {
    await initParserWasmBrowser();
  });

  it("keeps the supported corpus scene report stable", async () => {
    const expected = JSON.parse(readFileSync(REPORT_PATH, "utf8"));
    const actualSceneReport = [];

    for (const file of EXAMPLE_CORPUS_FILES) {
      const source = readFileSync(join(REPO_ROOT, file), "utf8");
      actualSceneReport.push({
        file,
        summary: sceneSummary(await buildLayoutSceneFromSource(source, { staffScale: 0.75, pageWidth: 612, showTitle: true })),
      });
    }

    expect(actualSceneReport).toEqual(expected.sceneReport);
  });

  it("keeps approved representative scene snapshots stable", async () => {
    for (const file of REPRESENTATIVE_SCENE_FILES) {
      const source = readFileSync(join(REPO_ROOT, file), "utf8");
      const actual = JSON.stringify(
        await buildLayoutSceneFromSource(source, { staffScale: 0.75, pageWidth: 612, showTitle: true }),
        null,
        2,
      );
      const snapshotPath = join(
        REPO_ROOT,
        "docs",
        "layout-corpus",
        "scene-snapshots",
        `${file.split("/").pop()?.replace(".drum", "")}.layout-scene.json`,
      );
      expect(actual).toBe(readFileSync(snapshotPath, "utf8"));
    }
  });

  it("keeps the supported corpus SVG semantic report stable", async () => {
    const expected = JSON.parse(readFileSync(REPORT_PATH, "utf8"));
    const actualSvgSemanticReport = [];

    for (const file of EXAMPLE_CORPUS_FILES) {
      const source = readFileSync(join(REPO_ROOT, file), "utf8");
      const layoutSvg = (await renderSourcePagesToSvgs(source, { staffScale: 0.75, pageWidth: 612, showTitle: true })).join("\n");
      actualSvgSemanticReport.push({
        file,
        summary: svgSemanticSummary(layoutSvg),
      });
    }

    expect(actualSvgSemanticReport).toEqual(expected.svgSemanticReport);
  });
});
