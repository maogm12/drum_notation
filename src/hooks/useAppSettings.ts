import { useEffect, useState } from "preact/compat";
import type { PagePadding } from "../renderer/renderOptions";
import { SETTINGS_RANGES } from "../renderer/renderOptions";

export type MainTab = "editor" | "page" | "xml";

export interface AppSettings {
  hideVoice2Rests: boolean;
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

export function resolveAppSettings(saved: string | null): AppSettings {
  if (!saved) return defaultSettings;
  try {
    const parsed = JSON.parse(saved) as Partial<AppSettings> & { useLayoutEngine?: unknown };
    const { useLayoutEngine: _legacyRenderer, ...rendererNeutralSettings } = parsed;
    void _legacyRenderer;
    const r = SETTINGS_RANGES;
    if (rendererNeutralSettings.stemLength === undefined || rendererNeutralSettings.stemLength < r.stemLength.min || rendererNeutralSettings.stemLength > r.stemLength.max) {
      rendererNeutralSettings.stemLength = r.stemLength.default;
    }
    if (rendererNeutralSettings.voltaSpacing === undefined || rendererNeutralSettings.voltaSpacing < r.voltaSpacing.min || rendererNeutralSettings.voltaSpacing > r.voltaSpacing.max) {
      rendererNeutralSettings.voltaSpacing = r.voltaSpacing.default;
    }
    if (rendererNeutralSettings.hairpinOffsetY === undefined || rendererNeutralSettings.hairpinOffsetY < r.hairpinOffsetY.min || rendererNeutralSettings.hairpinOffsetY > r.hairpinOffsetY.max) {
      rendererNeutralSettings.hairpinOffsetY = r.hairpinOffsetY.default;
    }
    if (rendererNeutralSettings.headerHeight === undefined || rendererNeutralSettings.headerHeight < r.headerHeight.min || rendererNeutralSettings.headerHeight > r.headerHeight.max) {
      rendererNeutralSettings.headerHeight = r.headerHeight.default;
    }
    if (rendererNeutralSettings.durationSpacingCompression === undefined || rendererNeutralSettings.durationSpacingCompression < r.durationSpacingCompression.min || rendererNeutralSettings.durationSpacingCompression > r.durationSpacingCompression.max) {
      rendererNeutralSettings.durationSpacingCompression = r.durationSpacingCompression.default;
    }
    if (rendererNeutralSettings.measureWidthCompression === undefined || rendererNeutralSettings.measureWidthCompression < r.measureWidthCompression.min || rendererNeutralSettings.measureWidthCompression > r.measureWidthCompression.max) {
      rendererNeutralSettings.measureWidthCompression = r.measureWidthCompression.default;
    }
    return { ...defaultSettings, ...rendererNeutralSettings };
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
