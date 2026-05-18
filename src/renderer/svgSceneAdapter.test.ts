import { describe, expect, it, vi } from "vitest";
import { buildLayoutSceneFromSource, renderScenePagesToSvgs, renderSceneToSvg, renderSourcePagesToSvgs, renderSourceToSvg } from "./svgRenderer";

const SRC = `title Smoke
tempo 120
time 4/4
note 1/8

HH | x x x x |
`;

const HAIRPIN_SRC = `title Hairpin Offset
time 4/4
note 1/8

HH | < x x x x ! |
`;

function hairpinCenterY(scene: any): number {
  const items = scene.pages[0].items;
  const top = items.find((item: any) => item.role === "hairpin-top").primitive;
  const bottom = items.find((item: any) => item.role === "hairpin-bottom").primitive;
  return (top.y1Pt + top.y2Pt + bottom.y1Pt + bottom.y2Pt) / 4;
}

describe("SVG scene adapter", () => {
  it("renders a precomputed scene without source cache", () => {
    const scene = {
      version: "1",
      metricsVersion: "test",
      pages: [
        {
          index: 0,
          widthPt: 100,
          heightPt: 80,
          measures: [
            { id: "measure-0", globalIndex: 0, systemId: "system-0", xPt: 10, yPt: 20, widthPt: 40, heightPt: 20 },
            { id: "measure-1", globalIndex: 1, systemId: "system-0", xPt: 50, yPt: 20, widthPt: 40, heightPt: 20 },
          ],
          items: [
            {
              id: "item-1",
              role: "staff-line",
              kind: "lineSegment",
              zIndex: 0,
              primitive: { x1Pt: 10, y1Pt: 20, x2Pt: 90, y2Pt: 20, stroke: "#333", strokeWidth: 1 },
            },
            {
              id: "item-2",
              role: "title",
              kind: "textRun",
              zIndex: 0,
              primitive: { xPt: 50, yPt: 12, text: "Smoke", fontFamily: "Bravura", fontSizePt: 24, fill: "#333", textAnchor: "middle" },
            },
            {
              id: "item-3",
              role: "accent",
              kind: "textRun",
              zIndex: 1,
              anchorItemId: "item-2",
              primitive: { xPt: 50, yPt: 20, text: ">", fontFamily: "Bravura", fontSizePt: 12, fill: "#333", textAnchor: "middle" },
            },
          ],
          composites: [
            {
              id: "repeat-0",
              kind: "repeatSpan",
              fragment: "singleSegment",
              count: 3,
              startAnchorId: "measure-0",
              endAnchorId: "measure-1",
            },
            {
              id: "volta-0",
              kind: "volta",
              fragment: "singleSegment",
              label: "1.",
              startAnchorId: "measure-0",
              endAnchorId: "measure-1",
            },
          ],
        },
      ],
    } as any;

    const svg = renderSceneToSvg(scene, { staffScale: 1 });
    expect(svg).toContain('<line data-role="staff-line"');
    expect(svg).toContain("Smoke");
    expect(svg).toContain('data-role="repeat-span-line"');
    expect(svg).toContain('data-role="volta-line"');
    expect(svg).toContain('data-anchor-item-id="item-2"');
  });

  it("renders every scene page through the page-aware adapter", () => {
    const scene = {
      version: "1",
      metricsVersion: "test",
      pages: [
        {
          index: 0,
          widthPt: 100,
          heightPt: 80,
          measures: [],
          items: [
            {
              id: "page-0-item",
              role: "page-zero",
              kind: "textRun",
              zIndex: 0,
              primitive: { xPt: 10, yPt: 20, text: "Page Zero", fontFamily: "Bravura", fontSizePt: 12, fill: "#333" },
            },
          ],
          composites: [],
        },
        {
          index: 1,
          widthPt: 100,
          heightPt: 80,
          measures: [
            { id: "measure-2", globalIndex: 2, systemId: "system-1", xPt: 10, yPt: 40, widthPt: 30, heightPt: 20 },
            { id: "measure-3", globalIndex: 3, systemId: "system-1", xPt: 40, yPt: 40, widthPt: 30, heightPt: 20 },
          ],
          items: [
            {
              id: "page-1-item",
              role: "page-one",
              kind: "textRun",
              zIndex: 0,
              primitive: { xPt: 10, yPt: 20, text: "Page One", fontFamily: "Bravura", fontSizePt: 12, fill: "#333" },
            },
          ],
          composites: [
            {
              id: "page-1-repeat",
              kind: "repeatSpan",
              fragment: "singleSegment",
              count: 2,
              startAnchorId: "measure-2",
              endAnchorId: "measure-3",
            },
          ],
        },
      ],
    } as any;

    const svgs = renderScenePagesToSvgs(scene, { staffScale: 1 });
    expect(svgs).toHaveLength(2);
    expect(svgs[0]).toContain("Page Zero");
    expect(svgs[1]).toContain("Page One");
    expect(svgs[1]).toContain('data-role="repeat-span-line"');

    const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});
    expect(renderSceneToSvg(scene, { staffScale: 1 })).toContain("Page Zero");
    expect(warnSpy).toHaveBeenCalledWith(expect.stringContaining("renderSceneToSvg received a multi-page scene"));
    warnSpy.mockRestore();
  });

  it("builds scene from source and renders svg", () => {
    const scene = buildLayoutSceneFromSource(SRC, { pageWidth: 612, staffScale: 0.75 });
    expect(scene.pages.length).toBeGreaterThan(0);
    expect(scene.pages[0]?.items.length).toBeGreaterThan(0);
    expect(scene.pages[0]?.systems.length).toBeGreaterThan(0);
    expect(scene.pages[0]?.systems[0]?.measureIds.length).toBeGreaterThan(0);
    expect(scene.pages[0]?.composites.some((composite) => composite.kind === "textBlock" && composite.label === "tempo")).toBe(true);

    const svg = renderSourceToSvg(SRC, { pageWidth: 612, staffScale: 0.75 });
    expect(svg).toContain("<svg");
    expect(svg).toContain("Smoke");
    expect(svg).toContain('data-role="notehead"');

    const pages = renderSourcePagesToSvgs(SRC, { pageWidth: 612, staffScale: 0.75 });
    expect(pages).toHaveLength(scene.pages.length);
    expect(pages[0]).toContain('data-role="notehead"');
  });

  it("passes hairpin vertical offset into the layout engine", () => {
    const baseline = buildLayoutSceneFromSource(HAIRPIN_SRC, { pageWidth: 612, staffScale: 1, hairpinOffsetY: 0 });
    const lower = buildLayoutSceneFromSource(HAIRPIN_SRC, { pageWidth: 612, staffScale: 1, hairpinOffsetY: 10 });
    const higher = buildLayoutSceneFromSource(HAIRPIN_SRC, { pageWidth: 612, staffScale: 1, hairpinOffsetY: -5 });
    const baselineY = hairpinCenterY(baseline);

    expect(hairpinCenterY(lower) - baselineY).toBeCloseTo(10, 3);
    expect(hairpinCenterY(higher) - baselineY).toBeCloseTo(-5, 3);
  });

  it("passes title area height and title gap into the layout engine", () => {
    const baseline = buildLayoutSceneFromSource(SRC, { pageWidth: 612, staffScale: 1, topMargin: 30, headerHeight: 50, headerStaffSpacing: 60 });
    const taller = buildLayoutSceneFromSource(SRC, { pageWidth: 612, staffScale: 1, topMargin: 30, headerHeight: 80, headerStaffSpacing: 60 });
    const tighter = buildLayoutSceneFromSource(SRC, { pageWidth: 612, staffScale: 1, topMargin: 30, headerHeight: 50, headerStaffSpacing: 20 });

    expect(baseline.pages[0].systems[0].yPt).toBeCloseTo(140, 3);
    expect(taller.pages[0].systems[0].yPt).toBeCloseTo(170, 3);
    expect(tighter.pages[0].systems[0].yPt).toBeCloseTo(100, 3);
  });

  it("fails closed on parse errors", () => {
    expect(() => buildLayoutSceneFromSource("time 4\nHH | x |\n")).toThrow(/Line/);
    expect(() => renderSourceToSvg("time 4\nHH | x |\n")).toThrow(/Line/);
  });

  it("throws on unknown scene item kinds", () => {
    const badScene = {
      version: "1",
      metricsVersion: "test",
      pages: [{ index: 0, widthPt: 10, heightPt: 10, measures: [], items: [{ id: "x", role: "bad", kind: "mystery", zIndex: 0, primitive: {} }], composites: [] }],
    } as any;
    expect(() => renderSceneToSvg(badScene, { staffScale: 1 })).toThrow(/Unsupported scene item kind/);
  });

  it("renders repeat-span fragments without duplicating hooks and labels", () => {
    const scene = {
      version: "1",
      metricsVersion: "test",
      pages: [
        {
          index: 0,
          widthPt: 200,
          heightPt: 100,
          measures: [
            { id: "measure-0", globalIndex: 0, systemId: "system-0", xPt: 10, yPt: 30, widthPt: 40, heightPt: 20 },
            { id: "measure-1", globalIndex: 1, systemId: "system-1", xPt: 10, yPt: 60, widthPt: 40, heightPt: 20 },
          ],
          items: [],
          composites: [
            { id: "repeat-0", kind: "repeatSpan", fragment: "start", count: 2, startAnchorId: "measure-0", endAnchorId: "measure-0" },
            { id: "repeat-1", kind: "repeatSpan", fragment: "end", count: 2, startAnchorId: "measure-1", endAnchorId: "measure-1" },
          ],
        },
      ],
    } as any;

    const svg = renderSceneToSvg(scene, { staffScale: 1 });
    expect((svg.match(/data-role="repeat-span-count"/g) || []).length).toBe(1);
    expect((svg.match(/data-role="repeat-span-start"/g) || []).length).toBe(1);
    expect((svg.match(/data-role="repeat-span-end"/g) || []).length).toBe(1);
  });

  it("renders glyphRun, path, and polyline items", () => {
    const scene = {
      version: "1",
      metricsVersion: "test",
      pages: [
        {
          index: 0,
          widthPt: 120,
          heightPt: 80,
          measures: [],
          items: [
            {
              id: "glyph-1",
              role: "glyph-note",
              kind: "glyphRun",
              zIndex: 0,
              primitive: { xPt: 20, yPt: 20, codepoint: 0xe0a4, fontFamily: "Bravura", fontSizePt: 18, fill: "#333" },
            },
            {
              id: "path-1",
              role: "beam",
              kind: "path",
              zIndex: 0,
              primitive: { d: "M 10 10 L 30 10 L 30 14 L 10 14 Z", fill: "#333" },
            },
            {
              id: "poly-1",
              role: "shape",
              kind: "polyline",
              zIndex: 0,
              primitive: { pointsPt: [[10, 10], [20, 20], [30, 10]] },
            },
          ],
          composites: [],
        },
      ],
    } as any;

    const svg = renderSceneToSvg(scene, { staffScale: 1 });
    expect(svg).toContain('data-role="glyph-note"');
    expect(svg).toContain('data-role="beam"');
    expect(svg).toContain('data-role="shape"');
    expect(svg).toContain("<path");
    expect(svg).toContain("<polyline");
  });
});
