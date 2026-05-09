import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from "preact/compat";
import type { I18nKey } from "./keys";
import en from "./en.json";
import zh from "./zh.json";

export type Locale = "en" | "zh";

const bundles: Record<Locale, Record<string, string>> = { en, zh };

interface I18nContextValue {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: I18nKey, params?: Record<string, string | number>) => string;
}

const I18nContext = createContext<I18nContextValue>(null!);

function resolveLocale(): Locale {
  try {
    const stored = localStorage.getItem("drummark-locale");
    if (stored === "en" || stored === "zh") return stored;
  } catch { /* localStorage unavailable */ }
  if (typeof navigator !== "undefined") {
    const lang = navigator.language?.slice(0, 2);
    if (lang === "zh") return "zh";
  }
  return "en";
}

function paramReplace(template: string, params?: Record<string, string | number>): string {
  if (!params) return template;
  return template.replace(/\{\{(\w+)\}\}/g, (_, key: string) => {
    const val = params[key];
    return val !== undefined ? String(val) : `{{${key}}}`;
  });
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(resolveLocale);

  const setLocale = useCallback((next: Locale) => {
    setLocaleState(next);
    try {
      localStorage.setItem("drummark-locale", next);
    } catch { /* ignore */ }
  }, []);

  const t = useCallback(
    (key: I18nKey, params?: Record<string, string | number>): string => {
      const bundle = bundles[locale];
      const template = bundle?.[key];
      if (!template) {
        console.warn(`Missing i18n key: ${key} (locale: ${locale})`);
        return key;
      }
      return paramReplace(template, params);
    },
    [locale],
  );

  const value = useMemo<I18nContextValue>(
    () => ({ locale, setLocale, t }),
    [locale, setLocale, t],
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useT() {
  return useContext(I18nContext);
}
