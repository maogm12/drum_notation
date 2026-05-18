import { useEffect, useState } from "preact/compat";
import type { PagePadding } from "../renderer/renderOptions";
import { SETTINGS_RANGES } from "../renderer/renderOptions";

export type MainTab = "editor" | "page" | "xml";

export interface AppSettings {
  hideVoice2Rests: boolean;
  useLayoutEngine: boolean;
  pagePadding: PagePadding;
  staffScale: number;
  headerStaffSpacing: number;
  systemSpacing: number;
  stemLength: number;
  voltaSpacing: number;
  hairpinOffsetY: number;
  headerHeight: number;
  activeTab: MainTab;
  tempoOffsetX: number;
  tempoOffsetY: number;
  measureNumberOffsetX: number;
  measureNumberOffsetY: number;
  measureNumberFontSize: number;
  durationSpacingCompression: number;
  measureWidthCompression: number;
}

export const defaultSettings: AppSettings = {
  hideVoice2Rests: false,
  useLayoutEngine: true,
  pagePadding: { top: 30, right: 50, bottom: 30, left: 50 },
  staffScale: 0.75,
  headerStaffSpacing: 60,
  headerHeight: 50,
  systemSpacing: 30,
  stemLength: 31,
  voltaSpacing: 0,
  hairpinOffsetY: 0,
  activeTab: "page",
  tempoOffsetX: 0,
  tempoOffsetY: 0,
  measureNumberOffsetX: 0,
  measureNumberOffsetY: 8,
  measureNumberFontSize: 10,
  durationSpacingCompression: 0.6,
  measureWidthCompression: 0.75,
};

function hasOwn(object: object, key: PropertyKey): boolean {
  return Object.prototype.hasOwnProperty.call(object, key);
}

export function resolveAppSettings(saved: string | null): AppSettings {
  if (!saved) return defaultSettings;
  try {
    const parsed = JSON.parse(saved) as Partial<AppSettings>;
    const hasExplicitRenderer = hasOwn(parsed, "useLayoutEngine");
    const r = SETTINGS_RANGES;
    if (parsed.stemLength === undefined || parsed.stemLength < r.stemLength.min || parsed.stemLength > r.stemLength.max) {
      parsed.stemLength = r.stemLength.default;
    }
    if (parsed.voltaSpacing === undefined || parsed.voltaSpacing < r.voltaSpacing.min || parsed.voltaSpacing > r.voltaSpacing.max) {
      parsed.voltaSpacing = r.voltaSpacing.default;
    }
    if (parsed.hairpinOffsetY === undefined || parsed.hairpinOffsetY < r.hairpinOffsetY.min || parsed.hairpinOffsetY > r.hairpinOffsetY.max) {
      parsed.hairpinOffsetY = r.hairpinOffsetY.default;
    }
    if (parsed.headerHeight === undefined || parsed.headerHeight < r.headerHeight.min || parsed.headerHeight > r.headerHeight.max) {
      parsed.headerHeight = r.headerHeight.default;
    }
    if (parsed.durationSpacingCompression === undefined || parsed.durationSpacingCompression < r.durationSpacingCompression.min || parsed.durationSpacingCompression > r.durationSpacingCompression.max) {
      parsed.durationSpacingCompression = r.durationSpacingCompression.default;
    }
    if (parsed.measureWidthCompression === undefined || parsed.measureWidthCompression < r.measureWidthCompression.min || parsed.measureWidthCompression > r.measureWidthCompression.max) {
      parsed.measureWidthCompression = r.measureWidthCompression.default;
    }
    const resolved = { ...defaultSettings, ...parsed };
    if (!hasExplicitRenderer) {
      resolved.useLayoutEngine = true;
    }
    return resolved;
  } catch {
    return defaultSettings;
  }
}

export function useAppSettings() {
  const [settingsVisible, setSettingsVisible] = useState(true);

  const [settings, setSettings] = useState<AppSettings>(() => {
    const saved = localStorage.getItem("drummark-settings");
    return resolveAppSettings(saved);
  });

  useEffect(() => {
    if (settings.activeTab !== "page") {
      setSettingsVisible(false);
    }
  }, [settings.activeTab]);

  useEffect(() => {
    localStorage.setItem("drummark-settings", JSON.stringify(settings));
  }, [settings]);

  const updateSetting = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    setSettings((prev) => ({ ...prev, [key]: value }));
  };

  const updatePagePadding = (key: keyof PagePadding, value: number) => {
    setSettings((prev) => ({
      ...prev,
      pagePadding: { ...prev.pagePadding, [key]: value },
    }));
  };

  return {
    settings,
    updateSetting,
    updatePagePadding,
    settingsVisible,
    setSettingsVisible,
  };
}
