// @vitest-environment jsdom

import { beforeAll, describe, expect, it } from "vitest";
import { render } from "preact/compat";
import { SettingsPanel } from "./SettingsPanel";
import { defaultSettings } from "../hooks/useAppSettings";
import { I18nProvider } from "../i18n/context";

beforeAll(() => {
  if (!globalThis.window) return;
  if (!(window as any).ResizeObserver) {
    (window as any).ResizeObserver = class {
      observe() {}
      unobserve() {}
      disconnect() {}
    };
  }
  if (!window.requestAnimationFrame) {
    window.requestAnimationFrame = (cb: FrameRequestCallback) => setTimeout(cb, 0) as unknown as number;
    window.cancelAnimationFrame = (id: number) => clearTimeout(id);
  }
});

function renderSync(jsx: any): HTMLElement {
  const container = document.createElement("div");
  render(<I18nProvider>{jsx}</I18nProvider>, container);
  return container;
}

function openAccordionItem(container: HTMLElement, triggerText: string) {
  const triggers = container.querySelectorAll(".settings-trigger");
  for (const trigger of triggers) {
    if (trigger.textContent?.includes(triggerText)) {
      (trigger as HTMLButtonElement).click();
      return;
    }
  }
}

describe("SettingsPanel smoke", () => {
  const noop = (() => {}) as (value: any) => void;

  it("renders without crashing (debugMode=false)", () => {
    const container = renderSync(
      <SettingsPanel
        settings={defaultSettings}
        updateSetting={noop}
        updatePagePadding={noop}
        debugMode={false}
      />,
    );
    expect(container.innerHTML).toContain("Notes");
    expect(container.innerHTML).toContain("Page Layout");
    expect(container.innerHTML).toContain("Staff &amp; Layout");
  });

  it("renders debug sections when debugMode=true", () => {
    const container = renderSync(
      <SettingsPanel
        settings={defaultSettings}
        updateSetting={noop}
        updatePagePadding={noop}
        debugMode={true}
      />,
    );
    expect(container.innerHTML).toContain("Advanced Debugging");
  });

  it("hideVoice2Rests toggle renders inside Notation accordion", () => {
    const container = renderSync(
      <SettingsPanel
        settings={{ ...defaultSettings, hideVoice2Rests: true }}
        updateSetting={noop}
        updatePagePadding={noop}
        debugMode={false}
      />,
    );
    expect(container.querySelector(".toggle-root")).toBeTruthy();
  });

  it("hideVoice2Rests toggle shows unchecked state", () => {
    const container = renderSync(
      <SettingsPanel
        settings={{ ...defaultSettings, hideVoice2Rests: false }}
        updateSetting={noop}
        updatePagePadding={noop}
        debugMode={false}
      />,
    );
    const toggle = container.querySelector(".toggle-root");
    expect(toggle).toBeTruthy();
    expect(toggle!.getAttribute("data-state")).toBe("unchecked");
  });
});
