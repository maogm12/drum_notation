// @vitest-environment jsdom

import { describe, expect, it } from "vitest";
import { buildNormalizedScore } from "../dsl/normalize";
import { renderScoreToSvg } from "./renderer";

const BASE_OPTIONS = {
  pagePadding: { top: 30, right: 50, bottom: 30, left: 50 },
  pageWidth: 612,
  pageHeight: 792,
  staffScale: 0.75,
  headerHeight: 50,
  headerStaffSpacing: 60,
  systemSpacing: 30,
  stemLength: 31,
  voltaSpacing: -15,
  hideVoice2Rests: false,
  tempoOffsetX: 0,
  tempoOffsetY: 0,
  measureNumberOffsetX: 0,
  measureNumberOffsetY: 8,
  measureNumberFontSize: 10,
  durationSpacingCompression: 0.6,
  measureWidthCompression: 0.75,
};

describe("preview smoke", () => {
  it("renders a fixture with headers, voices, dynamics, and navigation", async () => {
    const dsl = `title  Smoke Test
subtitle  Acceptance
composer  Audit
time 4/4
divisions 4

| @segno HH | b - x - | d - @coda wb | @fine`;

    const svg = await renderScoreToSvg(
      buildNormalizedScore(dsl),
      BASE_OPTIONS,
    );

    expect(svg).toContain("<svg");
    expect(svg).toContain("vf-stavenote");
    expect(svg).toContain("Smoke Test");
    expect(svg).toContain("vf-edge-navigation");
    expect(svg).toContain("edge-navigation-fine");
  });

  it("renders with hideVoice2Rests producing different output than default", async () => {
    const dsl = `time 4/4
divisions 4

| HH | b - d - |`;

    const defaultSvg = await renderScoreToSvg(
      buildNormalizedScore(dsl),
      { ...BASE_OPTIONS, hideVoice2Rests: false },
    );

    const hiddenSvg = await renderScoreToSvg(
      buildNormalizedScore(dsl),
      { ...BASE_OPTIONS, hideVoice2Rests: true },
    );

    expect(defaultSvg).toContain("<svg");
    expect(hiddenSvg).toContain("<svg");
    expect(defaultSvg).not.toBe(hiddenSvg);
  });

  it("renders a hairpin fixture with explicit offsets", async () => {
    const dsl = `time 4/4
divisions 4

| HH < HH > HH < |`;

    const svg = await renderScoreToSvg(
      buildNormalizedScore(dsl),
      { ...BASE_OPTIONS, hairpinOffsetY: -15 },
    );

    expect(svg).toContain("<svg");
  });

  it("renders a multi-system score to exercise layout planning", async () => {
    const dsl = `time 4/4
divisions 4

| HH | b - | d - | wb |
| HH | b - | d - | wb |
| HH | b - | d - | wb |
| HH | b - | d - | wb |`;

    const svg = await renderScoreToSvg(
      buildNormalizedScore(dsl),
      BASE_OPTIONS,
    );

    expect(svg).toContain("<svg");
    expect(svg).toContain("vf-stavenote");
  });

  it("renders a single-tack rest measure without crashing", async () => {
    const dsl = `time 4/4
divisions 4

| - - - - |`;

    const svg = await renderScoreToSvg(
      buildNormalizedScore(dsl),
      BASE_OPTIONS,
    );

    expect(svg).toContain("<svg");
  });
});

describe("hairpin bottom skyline", () => {
  it("does not create clip path for single-system hairpins", async () => {
    const dsl = `time 4/4
divisions 4

| HH | < d d d d | d d d d | != |`;

    const svg = await renderScoreToSvg(buildNormalizedScore(dsl), BASE_OPTIONS);
    expect(svg).toContain("<svg");
    expect(svg).not.toContain("clipPath");
  });

  it("renders cross-system hairpins without crashing", async () => {
    const dsl = `time 4/4
divisions 4

| HH | < d d d d | d d d d | d d d d | d d d d | d d d d | d d d d | d d d d | != | d d d d |`;

    const svg = await renderScoreToSvg(buildNormalizedScore(dsl), {
      ...BASE_OPTIONS,
      pageWidth: 360,
      measureWidthCompression: 1,
    });
    expect(svg).toContain("<svg");
  });

  it("places hairpin below an accented bass drum note", async () => {
    const dsl = `time 4/4
divisions 4

BD | < b b B b != |`;

    const svg = await renderScoreToSvg(buildNormalizedScore(dsl), BASE_OPTIONS);
    expect(svg).toContain("<svg");
  });

  it("pushes hairpin further down when hairpinOffsetY is increased", async () => {
    const dsl = `time 4/4
divisions 4

| HH | < d d d d | d d d d | != |`;

    const svg0 = await renderScoreToSvg(buildNormalizedScore(dsl), { ...BASE_OPTIONS, hairpinOffsetY: 0 });
    const svg10 = await renderScoreToSvg(buildNormalizedScore(dsl), { ...BASE_OPTIONS, hairpinOffsetY: 10 });
    expect(svg0).toContain("<svg");
    expect(svg10).toContain("<svg");
    expect(svg0).not.toBe(svg10);
  });
});

describe("docs example smoke", () => {
  const examples = [
    "overview",
    "headers",
    "tracks",
    "tokens",
    "modifiers",
    "groups",
    "combined-hits",
    "sticking",
    "repeats",
    "multi-rest",
    "inline-repeat",
    "hairpins",
    "validation",
    "full-example",
    "common-rock",
    "common-16th",
    "common-shuffle",
    "common-triplet-fill",
    "common-ghost",
    "common-cross-stick",
    "common-half-time",
    "common-open-hh",
  ];

  for (const name of examples) {
    it(`renders docs example ${name}`, async () => {
      const fs = await import("node:fs");
      const path = await import("node:path");
      const { fileURLToPath } = await import("node:url");

      const __dirname = path.dirname(fileURLToPath(import.meta.url));
      const fixturePath = path.resolve(
        __dirname,
        "../../docs/examples",
        `${name}.drum`,
      );

      const source = fs.readFileSync(fixturePath, "utf8");
      const score = buildNormalizedScore(source);
      const svg = await renderScoreToSvg(score, BASE_OPTIONS);

      expect(svg).toContain("<svg");
      expect(svg.length).toBeGreaterThan(1000);
    });
  }
});
