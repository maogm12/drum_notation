import { describe, expect, it } from "vitest";
import { resolveAppSettings } from "./useAppSettings";

describe("resolveAppSettings", () => {
  it("defaults new users to the layout engine", () => {
    expect(resolveAppSettings(null).useLayoutEngine).toBe(true);
  });

  it("defaults old saved settings without renderer preference to the layout engine", () => {
    const settings = resolveAppSettings(JSON.stringify({ staffScale: 0.9 }));

    expect(settings.staffScale).toBe(0.9);
    expect(settings.useLayoutEngine).toBe(true);
  });

  it("preserves an explicit legacy renderer preference", () => {
    expect(resolveAppSettings(JSON.stringify({ useLayoutEngine: false })).useLayoutEngine).toBe(false);
  });

  it("preserves an explicit layout renderer preference", () => {
    expect(resolveAppSettings(JSON.stringify({ useLayoutEngine: true })).useLayoutEngine).toBe(true);
  });

  it("falls back to layout engine defaults for corrupt settings", () => {
    expect(resolveAppSettings("{").useLayoutEngine).toBe(true);
  });
});
