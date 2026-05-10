import * as Switch from "@radix-ui/react-switch";
import * as Accordion from "@radix-ui/react-accordion";
import { NumericSettingControl } from "./NumericSettingControl";
import type { AppSettings } from "../hooks/useAppSettings";
import type { PagePadding } from "../vexflow/types";
import { SETTINGS_RANGES } from "../vexflow/config";
import { useT } from "../i18n/context";

function Numeric({
  labelKey,
  value,
  min,
  max,
  step,
  onChange,
  unit,
}: {
  labelKey: string;
  value: number;
  min: number;
  max: number;
  step: number;
  onChange: (value: number) => void;
  unit?: string;
}) {
  const { t } = useT();
  const label = t(labelKey as Parameters<typeof t>[0]);
  return (
    <NumericSettingControl
      label={label}
      value={value}
      min={min}
      max={max}
      step={step}
      onChange={onChange}
      unit={unit}
      ariaLabelDecrease={t("settings.decrease", { label })}
      ariaLabelIncrease={t("settings.increase", { label })}
    />
  );
}

export function SettingsPanel({
  settings,
  updateSetting,
  updatePagePadding,
  debugMode,
}: {
  settings: AppSettings;
  updateSetting: <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => void;
  updatePagePadding: (key: keyof PagePadding, value: number) => void;
  debugMode: boolean;
}) {
  const { t } = useT();
  return (
    <Accordion.Root type="multiple" className="settings-accordion" defaultValue={["page-layout", "notation", "staff-header"]}>
      <Accordion.Item value="page-layout">
        <Accordion.Trigger className="settings-trigger">
          {t("settings.pageLayout")}
          <span className="settings-accordion-chevron" aria-hidden />
        </Accordion.Trigger>
        <Accordion.Content className="settings-content">
          <Numeric labelKey="settings.topMargin" value={settings.pagePadding.top} min={0} max={800} step={1} onChange={(value) => updatePagePadding("top", value)} />
          <Numeric labelKey="settings.bottomMargin" value={settings.pagePadding.bottom} min={0} max={800} step={1} onChange={(value) => updatePagePadding("bottom", value)} />
          <Numeric labelKey="settings.leftMargin" value={settings.pagePadding.left} min={0} max={400} step={1} onChange={(value) => updatePagePadding("left", value)} />
          <Numeric labelKey="settings.rightMargin" value={settings.pagePadding.right} min={0} max={400} step={1} onChange={(value) => updatePagePadding("right", value)} />
        </Accordion.Content>
      </Accordion.Item>

      <Accordion.Item value="notation">
        <Accordion.Trigger className="settings-trigger">
          {t("settings.notes")}
          <span className="settings-accordion-chevron" aria-hidden />
        </Accordion.Trigger>
        <Accordion.Content className="settings-content">
          <label className="setting-row toggle">
            <span>{t("settings.hideVoice2Rests")}</span>
            <Switch.Root
              className="toggle-root"
              checked={settings.hideVoice2Rests}
              onCheckedChange={(checked) => updateSetting("hideVoice2Rests", checked)}
            >
              <Switch.Thumb className="toggle-thumb" />
            </Switch.Root>
          </label>
          <Numeric labelKey="settings.stemLength" value={settings.stemLength} min={15} max={50} step={1} onChange={(value) => updateSetting("stemLength", value)} />
        </Accordion.Content>
      </Accordion.Item>

      <Accordion.Item value="staff-header">
        <Accordion.Trigger className="settings-trigger">
          {t("settings.staffLayout")}
          <span className="settings-accordion-chevron" aria-hidden />
        </Accordion.Trigger>
        <Accordion.Content className="settings-content">
          <Numeric labelKey="settings.staffScale" value={Math.round(settings.staffScale * 100)} min={30} max={150} step={5} unit="%" onChange={(value) => updateSetting("staffScale", value / 100)} />
          <Numeric labelKey="settings.systemSpacing" value={settings.systemSpacing} min={SETTINGS_RANGES.systemSpacing.min} max={SETTINGS_RANGES.systemSpacing.max} step={1} onChange={(value) => updateSetting("systemSpacing", value)} />
          <Numeric labelKey="settings.titleHeight" value={settings.headerHeight} min={SETTINGS_RANGES.headerHeight.min} max={SETTINGS_RANGES.headerHeight.max} step={1} onChange={(value) => updateSetting("headerHeight", value)} />
          <Numeric labelKey="settings.titleGap" value={settings.headerStaffSpacing} min={SETTINGS_RANGES.headerStaffSpacing.min} max={SETTINGS_RANGES.headerStaffSpacing.max} step={1} onChange={(value) => updateSetting("headerStaffSpacing", value)} />
          <Numeric labelKey="settings.voltaOffset" value={settings.voltaSpacing} min={SETTINGS_RANGES.voltaSpacing.min} max={SETTINGS_RANGES.voltaSpacing.max} step={1} onChange={(value) => updateSetting("voltaSpacing", value)} />
          <Numeric labelKey="settings.hairpinOffset" value={settings.hairpinOffsetY} min={SETTINGS_RANGES.hairpinOffsetY.min} max={SETTINGS_RANGES.hairpinOffsetY.max} step={1} onChange={(value) => updateSetting("hairpinOffsetY", value)} />
        </Accordion.Content>
      </Accordion.Item>

      {debugMode && (
        <Accordion.Item value="debug" className="settings-debug-section">
          <Accordion.Trigger className="settings-trigger debug-trigger">
            Advanced Debugging
            <span className="settings-accordion-chevron" aria-hidden />
          </Accordion.Trigger>
           <Accordion.Content className="settings-content">
            <label className="setting-row toggle">
              <span>{t("settings.useLayoutEngine")}</span>
              <Switch.Root
                className="toggle-root"
                checked={settings.useLayoutEngine}
                onCheckedChange={(checked) => updateSetting("useLayoutEngine", checked)}
              >
                <Switch.Thumb className="toggle-thumb" />
              </Switch.Root>
            </label>
            <label className="setting-row toggle">
              <span>{t("settings.useWasmParser")}</span>
              <Switch.Root
                className="toggle-root"
                checked={settings.useWasmParser}
                onCheckedChange={(checked) => updateSetting("useWasmParser", checked)}
              >
                <Switch.Thumb className="toggle-thumb" />
              </Switch.Root>
            </label>
            <div className="settings-group-label">Coordinate Offsets</div>
            <NumericSettingControl
              label="Tempo X"
              value={settings.tempoOffsetX}
              min={-100}
              max={100}
              step={1}
              onChange={(value) => updateSetting("tempoOffsetX", value)}
            />
            <NumericSettingControl
              label="Tempo Y"
              value={settings.tempoOffsetY}
              min={-100}
              max={100}
              step={1}
              onChange={(value) => updateSetting("tempoOffsetY", value)}
            />
            <NumericSettingControl
              label="Measure Num X"
              value={settings.measureNumberOffsetX}
              min={-100}
              max={100}
              step={1}
              onChange={(value) => updateSetting("measureNumberOffsetX", value)}
            />
            <NumericSettingControl
              label="Measure Num Y"
              value={settings.measureNumberOffsetY}
              min={-100}
              max={100}
              step={1}
              onChange={(value) => updateSetting("measureNumberOffsetY", value)}
            />
            <NumericSettingControl
              label="Measure Num Size"
              value={settings.measureNumberFontSize}
              min={4}
              max={24}
              step={1}
              onChange={(value) => updateSetting("measureNumberFontSize", value)}
            />
            <div className="settings-group-label">Experimental</div>
            <NumericSettingControl
              label="Note Spacing Compression"
              value={settings.durationSpacingCompression}
              min={0}
              max={1.5}
              step={0.05}
              onChange={(value) => updateSetting("durationSpacingCompression", value)}
            />
            <NumericSettingControl
              label="Measure Width Compression"
              value={settings.measureWidthCompression}
              min={0}
              max={1.5}
              step={0.05}
              onChange={(value) => updateSetting("measureWidthCompression", value)}
            />
          </Accordion.Content>
        </Accordion.Item>
      )}
    </Accordion.Root>
  );
}
