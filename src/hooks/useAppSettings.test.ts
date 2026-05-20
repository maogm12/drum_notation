import { describe, expect, it } from "vitest";
import { resolveAppSettings } from "./useAppSettings";

describe("resolveAppSettings", () => {
  it("does not expose a renderer selector for new users", () => {
    expect(resolveAppSettings(null)).not.toHaveProperty("useLayoutEngine");
  });

  it("preserves old saved settings without renderer preference", () => {
    const settings = resolveAppSettings(JSON.stringify({ staffScale: 0.9 }));

    expect(settings.staffScale).toBe(0.9);
    expect(settings).not.toHaveProperty("useLayoutEngine");
  });

  it("drops saved legacy renderer preferences", () => {
    expect(resolveAppSettings(JSON.stringify({ useLayoutEngine: false }))).not.toHaveProperty("useLayoutEngine");
    expect(resolveAppSettings(JSON.stringify({ useLayoutEngine: true }))).not.toHaveProperty("useLayoutEngine");
  });

  it("falls back to layout engine defaults for corrupt settings", () => {
    expect(resolveAppSettings("{")).not.toHaveProperty("useLayoutEngine");
  });
});
