import { beforeAll, describe, expect, it } from "vitest";
import { initParserWasmBrowser } from "../wasm/parser_wasm_browser";
import { parseDocumentSkeletonFromWasmSync } from "../wasm/skeleton";
import { buildNormalizedScoreWasm } from "./normalize";

beforeAll(async () => {
  await initParserWasmBrowser();
});

describe("WASM cutover blockers", () => {
  it("rejects malformed paragraph note overrides at parse time", () => {
    const bare = parseDocumentSkeletonFromWasmSync(`time 4/4
note 1/8

note
HH | x - x - |
`);
    expect(bare.errors.length).toBeGreaterThan(0);
    expect(bare.errors[0]?.message).toMatch(/note/i);

    const midParagraph = parseDocumentSkeletonFromWasmSync(`time 4/4
note 1/8
HH | x - x - |
note 1/16
HH | x x x x x x x x |
`);
    expect(midParagraph.errors.length).toBeGreaterThan(0);
    expect(midParagraph.errors.some((e) => /paragraph|unexpected 'note'|note override/i.test(e.message))).toBe(true);
  });

  it("reports malformed headers instead of silently dropping them", () => {
    const malformedTime = parseDocumentSkeletonFromWasmSync(`time 4
HH | x |
`);
    expect(malformedTime.errors.length).toBeGreaterThan(0);
    expect(malformedTime.errors.some((e) => /time/i.test(e.message))).toBe(true);

    const malformedTempo = parseDocumentSkeletonFromWasmSync(`tempo fast
HH | x |
`);
    expect(malformedTempo.errors.length).toBeGreaterThan(0);
    expect(malformedTempo.errors.some((e) => /tempo/i.test(e.message))).toBe(true);

    const malformedGrouping = parseDocumentSkeletonFromWasmSync(`grouping 3+
HH | x |
`);
    expect(malformedGrouping.errors.length).toBeGreaterThan(0);
    expect(malformedGrouping.errors.some((e) => /grouping/i.test(e.message))).toBe(true);
  });

  it("keeps signed inline repeat counts through parse and rejects them in normalization", () => {
    const negative = buildNormalizedScoreWasm(`time 4/4
note 1/8
HH | x - x - *-1 |
`);
    expect(negative.errors).toContainEqual(
      expect.objectContaining({
        message: "Repeat count must be at least 1",
      }),
    );

    const zero = buildNormalizedScoreWasm(`time 4/4
note 1/8
HH | x - x - *0 |
`);
    expect(zero.errors).toContainEqual(
      expect.objectContaining({
        message: "Repeat count must be at least 1",
      }),
    );
  });
});
