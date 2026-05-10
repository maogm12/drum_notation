import { memo, useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore, type ReactNode, type UIEvent } from "preact/compat";
import { buildNormalizedScore, type ParseError } from "./dsl";
import { type NormalizedScore } from "./dsl";
import type { VexflowRenderOptions, PagePadding } from "./vexflow";
import { resolveDocumentTheme, subscribeToThemeChanges, type AppTheme } from "./theme";
import { useAppSettings } from "./hooks/useAppSettings";
import { SettingsPanel } from "./components/SettingsPanel";
import { useT } from "./i18n/context";
import * as Tabs from "@radix-ui/react-tabs";
import * as Popover from "@radix-ui/react-popover";
import type { MainTab } from "./hooks/useAppSettings";

function toggleTheme() {
  const resolved = resolveDocumentTheme();
  document.documentElement.setAttribute("data-theme", resolved === "dark" ? "light" : "dark");
}

const legacySeedDsl = `tempo 96
time 4/4
note 1/16

HH |: x - x - o - x - | x - x:close - X - x - :|
SD |  - - d:cross - d - | D:rim - [2: d d:flam d] - - -  |
BD |  p - - - p - - - | p - p - - - p -                     |
HF |  - - - - p - - - | - - - - p:close - -                |

RC |  - - x:bell - - - - - | - - - - x - - - |
ST |  [2: R L R] - - -     | R - L - R - L - |`;

const seedDsl = `title "Advanced Funk"
subtitle "Performance Study"
composer "G. Mao"
tempo 120
time 4/4
note 1/16
grouping 2+2

HH |: x x x x x x x x :| x x x x x x o x |
HF | - - - - - - p - | - - - - - - p - |
SD | - - d - - - D - | - - d - - - d - |
BD | b - - - b - - - | b - - - B - - - |

| d d d d *2 |

ST | R - L - [2: R L R] - | R - L - R - L - |
RC | r r r r r r r r | r r r r r r r r |
C  | - - - - - - - c | - - - - - - - C |

| @segno c2 - cl - *2 | %% | @fine |`;

const pdfPageWidth = 612;
const pdfPageHeight = 792;

function downloadBlob(filename: string, blob: Blob) {
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  anchor.click();
  URL.revokeObjectURL(url);
}

function downloadTextFile(filename: string, content: string, mimeType: string) {
  downloadBlob(filename, new Blob([content], { type: mimeType }));
}

function safeExportBasename(title: string | undefined) {
  const filename = title ? title : "drummark";
  return filename
    .toLowerCase()
    .replace(/[^\p{L}\p{N}]+/gu, "-")
    .replace(/^-+|-+$/g, "") || "drummark";
}

function useDebouncedValue<T>(value: T, delayMs: number) {
  const [debouncedValue, setDebouncedValue] = useState(value);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      setDebouncedValue(value);
    }, delayMs);

    return () => {
      window.clearTimeout(timer);
    };
  }, [delayMs, value]);

  return debouncedValue;
}

function SearchPlusIcon() {
  return (
    <svg aria-hidden="true" fill="none" height="16" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24" width="16">
      <circle cx="11" cy="11" r="7" />
      <path d="M21 21l-4.3-4.3" />
      <path d="M11 8v6" />
      <path d="M8 11h6" />
    </svg>
  );
}

function SearchMinusIcon() {
  return (
    <svg aria-hidden="true" fill="none" height="16" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24" width="16">
      <circle cx="11" cy="11" r="7" />
      <path d="M21 21l-4.3-4.3" />
      <path d="M8 11h6" />
    </svg>
  );
}

function SearchIcon() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="11" cy="11" r="8"></circle>
      <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
    </svg>
  );
}

function PrinterIcon() {
  return (
    <svg aria-hidden="true" fill="none" height="18" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24" width="18">
      <polyline points="6 9 6 2 18 2 18 9" />
      <path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2" />
      <rect height="8" width="12" x="6" y="14" />
    </svg>
  );
}

function SettingsIcon() {
  return (
    <svg aria-hidden="true" fill="none" height="18" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24" width="18">
      <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
      <circle cx="12" cy="12" r="3" />
    </svg>
  );
}

function SaveIcon() {
  return (
    <svg aria-hidden="true" fill="none" height="18" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24" width="18">
      <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
      <polyline points="17 21 17 13 7 13 7 21" />
      <polyline points="7 3 7 8 15 8" />
    </svg>
  );
}

function CollapseAllIcon() {
  return (
    <svg aria-hidden="true" fill="none" height="18" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24" width="18">
      <polyline points="4 14 10 14 10 20" />
      <polyline points="20 10 14 10 14 4" />
      <line x1="10" y1="14" x2="21" y2="3" />
      <line x1="3" y1="21" x2="14" y2="10" />
    </svg>
  );
}

function ExpandAllIcon() {
  return (
    <svg aria-hidden="true" fill="none" height="18" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24" width="18">
      <polyline points="15 3 21 3 21 9" />
      <polyline points="9 21 3 21 3 15" />
      <line x1="21" y1="3" x2="14" y2="10" />
      <line x1="3" y1="21" x2="10" y2="14" />
    </svg>
  );
}

function BookIcon() {
  return (
    <svg aria-hidden="true" fill="none" height="18" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24" width="18">
      <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
      <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
    </svg>
  );
}

function DrumIcon() {
  return (
    <svg aria-hidden="true" className="app-logo" width="32" height="32" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path d="M19 22V6C19 6 24 7 24 12" stroke="var(--accent-primary)" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"/>
      <circle cx="12" cy="22" r="7" stroke="var(--text-main)" strokeWidth="2"/>
      <circle cx="12" cy="22" r="8.5" stroke="var(--text-main)" strokeWidth="0.5" strokeOpacity="0.4"/>
      <circle cx="12" cy="13.5" r="1" fill="var(--text-main)"/>
      <circle cx="12" cy="30.5" r="1" fill="var(--text-main)"/>
      <circle cx="3.5" cy="22" r="1" fill="var(--text-main)"/>
      <circle cx="20.5" cy="22" r="1" fill="var(--text-main)"/>
      <line x1="7" y1="20" x2="17" y2="20" stroke="var(--text-main)" strokeWidth="0.5" strokeOpacity="0.6"/>
      <line x1="7" y1="22" x2="17" y2="22" stroke="var(--text-main)" strokeWidth="0.5" strokeOpacity="0.6"/>
      <line x1="7" y1="24" x2="17" y2="24" stroke="var(--text-main)" strokeWidth="0.5" strokeOpacity="0.6"/>
    </svg>
  );
}

function SunIcon() {
  return (
    <svg aria-hidden="true" fill="none" height="18" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24" width="18">
      <circle cx="12" cy="12" r="5" />
      <line x1="12" y1="1" x2="12" y2="3" />
      <line x1="12" y1="21" x2="12" y2="23" />
      <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
      <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
      <line x1="1" y1="12" x2="3" y2="12" />
      <line x1="21" y1="12" x2="23" y2="12" />
      <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
      <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
    </svg>
  );
}

function MoonIcon() {
  return (
    <svg aria-hidden="true" fill="none" height="18" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24" width="18">
      <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
    </svg>
  );
}


function DslEditor({ value, onChange, errors, theme }: { value: string; onChange: (value: string) => void; errors: ParseError[]; theme: AppTheme }) {
  const hostRef = useRef<HTMLDivElement | null>(null);
  const viewRef = useRef<any>(null);
  const linterCompartmentRef = useRef<any>(null);
  const themeCompartmentRef = useRef<any>(null);
  const cmModulesRef = useRef<any>(null);
  const [ready, setReady] = useState(false);
  const onChangeRef = useRef(onChange);

  useEffect(() => {
    onChangeRef.current = onChange;
  }, [onChange]);

  useEffect(() => {
    let cancelled = false;
    Promise.all([
      import("./drummark"),
      import("@codemirror/state"),
      import("@codemirror/view"),
      import("@codemirror/commands"),
      import("@codemirror/lint"),
    ]).then(([drummark, cmState, cmView, cmCommands, cmLint]) => {
      if (cancelled) return;
      cmModulesRef.current = { drummark, cmState, cmView, cmCommands, cmLint };
      setReady(true);
    });
    return () => { cancelled = true; };
  }, []);

  useEffect(() => {
    if (!ready || !hostRef.current) return;
    const { drummark, cmState, cmView, cmCommands, cmLint } = cmModulesRef.current;

    linterCompartmentRef.current = new cmState.Compartment();
    themeCompartmentRef.current = new cmState.Compartment();

    const contentAttrs: Record<string, string> = {
      spellcheck: "false",
      autocorrect: "off",
      autocapitalize: "off",
      "data-gramm": "false",
    };

    const view = new cmView.EditorView({
      state: cmState.EditorState.create({
        doc: value,
        extensions: [
          cmView.lineNumbers(),
          cmView.highlightActiveLine(),
          cmView.highlightActiveLineGutter(),
          cmCommands.history(),
          cmView.keymap.of(cmCommands.historyKeymap),
          cmState.EditorState.tabSize.of(2),
          cmView.EditorView.contentAttributes.of(contentAttrs),
          drummark.drumMarkLanguage,
          drummark.drumMarkSyntaxHighlighting,
          themeCompartmentRef.current.of(drummark.getDrumMarkEditorTheme(theme)),
          linterCompartmentRef.current.of(
            cmLint.linter((v: any) => {
              return errors.map((err) => {
                const lineNum = Math.min(Math.max(1, err.line), v.state.doc.lines);
                const line = v.state.doc.line(lineNum);
                const pos = Math.min(line.from + Math.max(0, err.column - 1), line.to);
                return {
                  from: pos,
                  to: Math.min(pos + 1, line.to),
                  severity: "error" as const,
                  message: err.message,
                };
              });
            }),
          ),
          cmView.EditorView.updateListener.of((update: any) => {
            if (update.docChanged) {
              onChangeRef.current(update.state.doc.toString());
            }
          }),
        ],
      }),
      parent: hostRef.current,
    });

    viewRef.current = view;

    return () => {
      view.destroy();
      viewRef.current = null;
    };
  }, [ready]);

  useEffect(() => {
    const view = viewRef.current;
    const compartment = themeCompartmentRef.current;
    const modules = cmModulesRef.current;
    if (!view || !compartment || !modules) return;

    view.dispatch({
      effects: compartment.reconfigure(modules.drummark.getDrumMarkEditorTheme(theme)),
    });
  }, [theme]);

  useEffect(() => {
    const view = viewRef.current;
    const compartment = linterCompartmentRef.current;
    const modules = cmModulesRef.current;
    if (!view || !compartment || !modules) return;

    view.dispatch({
      effects: compartment.reconfigure(
        modules.cmLint.linter((v: any) => {
          return errors.map((err) => {
            const lineNum = Math.min(Math.max(1, err.line), v.state.doc.lines);
            const line = v.state.doc.line(lineNum);
            const pos = Math.min(line.from + Math.max(0, err.column - 1), line.to);
            return {
              from: pos,
              to: Math.min(pos + 1, line.to),
              severity: "error" as const,
              message: err.message,
            };
          });
        }),
      ),
    });
  }, [errors]);

  if (!ready) {
    return (
      <div className="editor-shell">
        <div className="editor-container editor-loading" />
      </div>
    );
  }

  return (
    <div className="editor-shell">
      <div className="editor-container" ref={hostRef} />
    </div>
  );
}

const PagePreview = memo(function PagePreview({
  score,
  pagePadding,
  staffScale,
  headerHeight,
  headerStaffSpacing,
  systemSpacing,
  stemLength,
  voltaSpacing,
  hairpinOffsetY,
  hideVoice2Rests,
  tempoOffsetX,
  tempoOffsetY,
  measureNumberOffsetX,
  measureNumberOffsetY,
  measureNumberFontSize,
  durationSpacingCompression,
  measureWidthCompression,
  active,
  theme,
}: {
  score: NormalizedScore | null;
  pagePadding: PagePadding;
  staffScale: number;
  headerHeight: number;
  headerStaffSpacing: number;
  systemSpacing: number;
  stemLength: number;
  voltaSpacing: number;
  hairpinOffsetY: number;
  hideVoice2Rests: boolean;
  tempoOffsetX: number;
  tempoOffsetY: number;
  measureNumberOffsetX: number;
  measureNumberOffsetY: number;
  measureNumberFontSize: number;
  durationSpacingCompression: number;
  measureWidthCompression: number;
  active: boolean;
  theme: AppTheme;
}) {
  const { t } = useT();
  const shellRef = useRef<HTMLDivElement | null>(null);
  const scrollPosRef = useRef({ top: 0, left: 0 });
  const [renderedMarkup, setRenderedMarkup] = useState("");
  const [isRendering, setIsRendering] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function handleScroll(e: UIEvent<HTMLDivElement>) {
    scrollPosRef.current = {
      top: e.currentTarget.scrollTop,
      left: e.currentTarget.scrollLeft,
    };
  }

  useEffect(() => {
    if (!active || !score) return;

    const targetTop = scrollPosRef.current.top;
    const targetLeft = scrollPosRef.current.left;
    setIsRendering(true);

    const opts: VexflowRenderOptions = {
      pagePadding,
      staffScale,
      pageWidth: pdfPageWidth,
      pageHeight: pdfPageHeight,
      headerHeight,
      headerStaffSpacing,
      systemSpacing,
      stemLength,
      voltaSpacing,
      hairpinOffsetY,
      hideVoice2Rests,
      tempoOffsetX,
      tempoOffsetY,
      measureNumberOffsetX,
      measureNumberOffsetY,
      measureNumberFontSize,
      durationSpacingCompression,
      measureWidthCompression,
    };

    import("./vexflow")
      .then(({ renderScorePagesToSvgs }) => renderScorePagesToSvgs(score, opts))
      .then((pages) => {
        const markup = pages.map((svg, i) => `<section class="staff-preview-page" data-page="${i+1}">${svg}</section>`).join("");
        setRenderedMarkup(markup);
        setIsRendering(false);

        if (shellRef.current) {
          shellRef.current.scrollTop = targetTop;
          shellRef.current.scrollLeft = targetLeft;
        }

        setError(null);
      })
      .catch((renderError) => {
        setIsRendering(false);
        const msg = renderError instanceof Error ? renderError.message : String(renderError);
        console.error("VexFlow render error:", renderError);
        setError(msg || t("preview.error"));
      });
  }, [score, systemSpacing, stemLength, voltaSpacing, hairpinOffsetY, headerStaffSpacing, headerHeight, active, hideVoice2Rests, pagePadding, staffScale, tempoOffsetX, tempoOffsetY, measureNumberOffsetX, measureNumberOffsetY, measureNumberFontSize, durationSpacingCompression, measureWidthCompression]);

  if (!score) {
    return (
      <div className={`staff-preview-shell${theme === "dark" ? " staff-preview-shell-dark" : ""}`} ref={shellRef} onScroll={handleScroll}>
        <div className="staff-printable-frame">
          <div className="staff-printable">
            <div className="staff-preview page-view">
              <section className="staff-preview-page" data-page="1" />
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={`staff-preview-shell${theme === "dark" ? " staff-preview-shell-dark" : ""}`} ref={shellRef} onScroll={handleScroll}>
      {error ? <div className="staff-error">{error}</div> : null}
      {isRendering && !renderedMarkup ? (
        <div className="staff-rendering">{t("preview.rendering")}</div>
      ) : null}
      <div className="staff-printable-frame">
        <div className="staff-printable">
          <div className="staff-preview page-view" dangerouslySetInnerHTML={{ __html: renderedMarkup }} />
        </div>
      </div>
    </div>
  );
});

function MusicXmlPreview({ xml, collapsed, toggle }: {
  xml: string;
  collapsed: Set<string>;
  toggle: (path: string) => void;
}) {
  const { t } = useT();
  const parser = new DOMParser();
  const doc = parser.parseFromString(xml, "text/xml");
  const parseError = doc.querySelector("parsererror");

  if (parseError) {
    return (
      <div className="xml-preview" aria-label={t("xml.previewAria")}>
        <pre>{xml}</pre>
      </div>
    );
  }

  return (
    <div className="xml-preview" aria-label={t("xml.previewAria")}>
      {renderXmlTreeLines(doc.documentElement, 0, "", collapsed, toggle, 0)}
    </div>
  );
}

function renderXmlTreeLines(node: Node, depth: number, path: string, collapsed: Set<string>, toggle: (path: string) => void, index: number): ReactNode {
  if (node.nodeType === Node.TEXT_NODE) {
    const text = node.textContent?.trim();
    if (!text) return null;
    return (
      <div key={`t-${index}`} className="xml-line" style={{ paddingLeft: depth * 16 }}>
        <span className="xml-text">{text}</span>
      </div>
    );
  }

  if (node.nodeType !== Node.ELEMENT_NODE) return null;

  const el = node as Element;
  const tagName = el.nodeName;
  const nodePath = path ? `${path}/${tagName}` : tagName;
  const isCollapsed = collapsed.has(nodePath);

  const childNodes = Array.from(el.childNodes);
  const childElements = childNodes.filter((n) => n.nodeType === Node.ELEMENT_NODE);
  const textContent = childNodes
    .filter((n) => n.nodeType === Node.TEXT_NODE)
    .map((n) => n.textContent?.trim())
    .filter(Boolean)
    .join("");

  const attrs = Array.from(el.attributes).map((attr) => (
    <span key={attr.name} className="xml-attr">
      <span className="xml-attr-name">{attr.name}</span>
      <span className="xml-attr-eq">="</span>
      <span className="xml-attr-value">{attr.value}</span>
      <span className="xml-attr-eq">"</span>
    </span>
  ));

  // Leaf element: text content only, no child elements → inline
  if (childElements.length === 0 && textContent) {
    return (
      <div key={`e-${index}`} className="xml-line" style={{ paddingLeft: depth * 16 }}>
        <span className="xml-toggle xml-toggle-placeholder"/>
        <span className="xml-bracket">{"<"}</span>
        <span className="xml-tag">{tagName}</span>
        {attrs}
        <span className="xml-bracket">{">"}</span>
        <span className="xml-text">{textContent}</span>
        <span className="xml-bracket">{"</"}{tagName}{">"}</span>
      </div>
    );
  }

  const hasChildren = childElements.length > 0;

  return (
    <div key={`e-${index}`}>
      <div className="xml-line" style={{ paddingLeft: depth * 16 }}>
        {hasChildren ? (
          <span className="xml-toggle" onClick={() => toggle(nodePath)}>
            <span className="xml-arrow">{isCollapsed ? "▶" : "▼"}</span>
          </span>
        ) : (
          <span className="xml-toggle xml-toggle-placeholder"/>
        )}
        <span className="xml-bracket">{"<"}</span>
        <span className="xml-tag">{tagName}</span>
        {attrs}
        <span className="xml-bracket">{isCollapsed && hasChildren ? "> [...]" : (hasChildren ? ">" : "/>")}</span>
      </div>
      {hasChildren && !isCollapsed && childElements.map((child, i) =>
        renderXmlTreeLines(child, depth + 1, `${nodePath}/${i}`, collapsed, toggle, i),
      )}
      {hasChildren && !isCollapsed && (
        <div className="xml-line" style={{ paddingLeft: depth * 16 }}>
          <span className="xml-bracket">{"</"}{tagName}{">"}</span>
        </div>
      )}
    </div>
  );
}


export function App() {
  const resolvedTheme: AppTheme = useSyncExternalStore(
    (listener) => subscribeToThemeChanges(listener),
    () => resolveDocumentTheme(),
  );
  const [dsl, setDsl] = useState(() => {
    const saved = localStorage.getItem("drummark-dsl");
    if (!saved || saved === legacySeedDsl) {
      return seedDsl;
    }
    return saved;
  });
  const {
    settings,
    updateSetting,
    updatePagePadding,
    settingsVisible,
    setSettingsVisible,
  } = useAppSettings();
  const { t, locale, setLocale } = useT();
  const [pageZoomMenuOpen, setPageZoomMenuOpen] = useState(false);
  const pageZoomMenuOpenRef = useRef(pageZoomMenuOpen);
  useEffect(() => { pageZoomMenuOpenRef.current = pageZoomMenuOpen; }, [pageZoomMenuOpen]);
  const [xmlCollapsed, setXmlCollapsed] = useState<Set<string>>(new Set());
  const debugMode = new URLSearchParams(window.location.search).has("debug");

  const xmlToggle = (path: string) => {
    setXmlCollapsed((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  };
  const xmlIsAllCollapsed = xmlCollapsed.size > 0;
  const xmlToggleAll = () => {
    if (xmlIsAllCollapsed) {
      setXmlCollapsed(new Set());
    } else {
      setXmlCollapsed(new Set(["score-partwise", "part-list", "part"]));
    }
  };

  const [showErrors, setShowErrors] = useState(false);

  useEffect(() => {
    if (pageZoomMenuOpenRef.current) {
      setPageZoomMenuOpen(false);
    }
  }, [settings.activeTab]);
  
  const [editorWidth, setEditorWidth] = useState(() => {
    const saved = localStorage.getItem("drummark-editor-width");
    return saved ? parseInt(saved, 10) : 600;
  });
  const isResizingRef = useRef(false);
  const workerRef = useRef<Worker | null>(null);
  const requestIdRef = useRef(0);
  const latestHandledRequestIdRef = useRef(0);
  const [isScorePending, setIsScorePending] = useState(false);
  const [analysis, setAnalysis] = useState(() => {
    const initialScore = buildNormalizedScore(dsl);
    return { score: initialScore };
  });
  const [staffXml, setStaffXml] = useState<string | null>(null);
  const [isXmlPending, setIsXmlPending] = useState(false);
  const xmlRequestIdRef = useRef(0);
  const latestXmlIdRef = useRef(0);
  const pendingExportRef = useRef(false);
  const exportBasenameRef = useRef(safeExportBasename(undefined));

  const score = analysis.score;
  const hasRenderableScore = useMemo(
    () => score.ast.paragraphs.some((paragraph) => paragraph.measureCount > 0 && paragraph.tracks.length > 0),
    [score],
  );
  const analysisInput = useMemo(
    () => ({
      dsl,
      hideVoice2Rests: settings.hideVoice2Rests,
      useWasmParser: settings.useWasmParser,
    }),
    [dsl, settings.hideVoice2Rests, settings.useWasmParser],
  );
  const debouncedAnalysisInput = useDebouncedValue(
    analysisInput,
    120,
  );
  const canExport = !isScorePending && hasRenderableScore && score.errors.length === 0;

  useEffect(() => {
    const worker = new Worker(new URL("./scoreWorker.ts", import.meta.url), { type: "module" });
    workerRef.current = worker;

    worker.onmessage = (event: MessageEvent<{ type: string; id: number; score?: NormalizedScore; xml?: string }>) => {
      const { type, id, score: nextScore, xml: nextXml } = event.data;

      if (type === "parse" && nextScore) {
        if (id < latestHandledRequestIdRef.current) return;
        latestHandledRequestIdRef.current = id;
        setAnalysis((prev) => ({ ...prev, score: nextScore }));
        setIsScorePending(id !== requestIdRef.current);
      } else if (type === "xml" && nextXml !== undefined) {
        if (id < latestXmlIdRef.current) return;
        latestXmlIdRef.current = id;
        setStaffXml(nextXml);
        setIsXmlPending(false);
        if (pendingExportRef.current) {
          pendingExportRef.current = false;
          downloadTextFile(
            `${exportBasenameRef.current}.musicxml`,
            nextXml,
            "application/vnd.recordare.musicxml+xml",
          );
        }
      }
    };

    return () => {
      worker.terminate();
      workerRef.current = null;
    };
  }, []);

  useEffect(() => {
    const worker = workerRef.current;
    if (!worker) {
      return;
    }

    const nextId = requestIdRef.current + 1;
    requestIdRef.current = nextId;
    setIsScorePending(true);
    worker.postMessage({
      type: "parse" as const,
      id: nextId,
      dsl: debouncedAnalysisInput.dsl,
      hideVoice2Rests: debouncedAnalysisInput.hideVoice2Rests,
      parseMode: settings.useWasmParser ? "wasm" : "lezer",
    });
  }, [debouncedAnalysisInput]);

  // Request XML when switching to XML tab or when score changes while on XML tab
  const requestXml = useCallback(() => {
    const worker = workerRef.current;
    if (!worker) return;
    const id = ++xmlRequestIdRef.current;
    setIsXmlPending(true);
    worker.postMessage({
      type: "generateXml" as const,
      id,
      hideVoice2Rests: settings.hideVoice2Rests,
    });
  }, [settings.hideVoice2Rests]);

  useEffect(() => {
    if (settings.activeTab === "xml" && score && !isScorePending) {
      requestXml();
    }
  }, [settings.activeTab, score, isScorePending, requestXml]);

  useEffect(() => {
    exportBasenameRef.current = safeExportBasename(score.ast.headers.title?.value);
  }, [score]);

  useEffect(() => {
    localStorage.setItem("drummark-dsl", dsl);
  }, [dsl]);

  useEffect(() => {
    localStorage.setItem("drummark-editor-width", String(editorWidth));
  }, [editorWidth]);

  const handleMouseDown = useCallback(() => {
    isResizingRef.current = true;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    document.body.style.webkitUserSelect = "none";
    document.body.style.webkitTouchCallout = "none";
  }, []);

  const handleTouchStart = useCallback(() => {
    isResizingRef.current = true;
    document.body.style.userSelect = "none";
    document.body.style.webkitUserSelect = "none";
    document.body.style.webkitTouchCallout = "none";
  }, []);

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isResizingRef.current) return;
      setEditorWidth(Math.max(320, Math.min(window.innerWidth - 320, e.clientX)));
    };

    const handleTouchMove = (e: TouchEvent) => {
      if (!isResizingRef.current) return;
      setEditorWidth(Math.max(320, Math.min(window.innerWidth - 320, e.touches[0]!.clientX)));
    };

    const handleMouseUp = () => {
      isResizingRef.current = false;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
      document.body.style.webkitUserSelect = "";
      document.body.style.webkitTouchCallout = "";
    };

    const handleTouchEnd = () => {
      isResizingRef.current = false;
      document.body.style.userSelect = "";
      document.body.style.webkitUserSelect = "";
      document.body.style.webkitTouchCallout = "";
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
    window.addEventListener("touchmove", handleTouchMove, { passive: true });
    window.addEventListener("touchend", handleTouchEnd);
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
      window.removeEventListener("touchmove", handleTouchMove);
      window.removeEventListener("touchend", handleTouchEnd);
    };
  }, []);

  function handleMusicXmlExport() {
    if (staffXml) {
      downloadTextFile(`${safeExportBasename(score.ast.headers.title?.value)}.musicxml`, staffXml, "application/vnd.recordare.musicxml+xml");
    } else if (!isXmlPending) {
      pendingExportRef.current = true;
      requestXml();
    }
  }

  function handlePrint() {
    const printWindow = window.open("", "_blank");
    if (!printWindow) {
      window.alert(t("alert.printPopup"));
      return;
    }

    const title = score.ast.headers.title?.value ?? "DrumMark Score";
    const styles = Array.from(document.querySelectorAll("style, link[rel='stylesheet']"))
      .map(el => el.outerHTML)
      .join("\n");

    const scoreHtml = document.querySelector(".staff-preview.page-view")?.innerHTML || "";

    printWindow.document.write(`
      <!DOCTYPE html>
      <html>
        <head>
          <title>${title}</title>
          ${styles}
          <style>
            @media print {
              @page { margin: 0; size: auto; }
              body { margin: 0; padding: 0; background: white; }
              .staff-printable-frame { margin: 0 !important; padding: 0 !important; }
              .staff-preview-page {
                margin: 0 !important;
                padding: 0 !important;
                page-break-after: always;
                border: none !important;
                box-shadow: none !important;
                background: white !important;
              }
              .staff-preview-page:last-child { page-break-after: auto; }
              svg { width: 100% !important; height: auto !important; }
            }
            body { 
              margin: 0; 
              padding: 20px; 
              display: flex; 
              flex-direction: column; 
              align-items: center; 
              background: #f0f2f5; 
            }
            .staff-preview-page {
              background: white;
              box-shadow: 0 4px 12px rgba(0,0,0,0.1);
              margin-bottom: 20px;
              width: 100%;
              max-width: 800px;
            }
          </style>
        </head>
        <body>
          ${scoreHtml}
          <script>
            window.onload = () => {
              // Give some time for fonts/SVGs to stabilize
              setTimeout(() => {
                window.print();
                window.close();
              }, 500);
            };
          </script>
        </body>
      </html>
    `);
    printWindow.document.close();
  }

  const [fitWidth, setFitWidth] = useState(true);

  const pageSurfaceBodyRef = useRef<HTMLDivElement | null>(null);

  const savedScale = localStorage.getItem("drummark-pageScale");
  const pageScaleRef = useRef(savedScale ? Math.max(0.2, Math.min(3.0, parseFloat(savedScale) || 0.8)) : 0.8);
  const pageZoomPercent = Math.round(pageScaleRef.current * 100);

  const scalePersistTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const applyScaleCss = (scale: number) => {
    const clamped = Math.max(0.2, Math.min(3.0, Math.round(scale * 100) / 100));
    pageSurfaceBodyRef.current?.style.setProperty("--page-scale", clamped.toString());
    pageScaleRef.current = clamped;
    if (scalePersistTimerRef.current) clearTimeout(scalePersistTimerRef.current);
    scalePersistTimerRef.current = setTimeout(() => {
      localStorage.setItem("drummark-pageScale", clamped.toString());
    }, 500);
  };

  useEffect(() => {
    return () => {
      if (scalePersistTimerRef.current) clearTimeout(scalePersistTimerRef.current);
    };
  }, []);

  useEffect(() => {
    pageSurfaceBodyRef.current?.style.setProperty("--page-scale", pageScaleRef.current.toString());
    return () => {
      if (scalePersistTimerRef.current) clearTimeout(scalePersistTimerRef.current);
    };
  }, []);

  useEffect(() => {
    if (!fitWidth || !pageSurfaceBodyRef.current) return;

    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (entry) {
        const isMobile = window.innerWidth <= 768;
        const padding = isMobile ? 0 : 80;
        const containerWidth = entry.contentRect.width - padding;
        const baseWidth = 800;
        const newScale = Math.max(0.2, Math.min(3.0, Math.round((containerWidth / baseWidth) * 20) / 20));
        pageSurfaceBodyRef.current?.style.setProperty("--page-scale", newScale.toString());
        pageScaleRef.current = newScale;
        if (scalePersistTimerRef.current) clearTimeout(scalePersistTimerRef.current);
        scalePersistTimerRef.current = setTimeout(() => {
          localStorage.setItem("drummark-pageScale", newScale.toString());
        }, 500);
      }
    });

    observer.observe(pageSurfaceBodyRef.current);
    return () => observer.disconnect();
  }, [fitWidth]);

  function adjustPageScale(delta: number) {
    setFitWidth(false);
    const newScale = Math.max(0.2, Math.min(3.0, Math.round((pageScaleRef.current + delta) * 100) / 100));
    applyScaleCss(newScale);
  }

    const touchStateRef = useRef({ distance: 0, initialScale: 1, centerX: 0, centerY: 0 });
    const activeTabRef = useRef(settings.activeTab);
    useEffect(() => { activeTabRef.current = settings.activeTab; }, [settings.activeTab]);

    useEffect(() => {
    const handleGlobalWheel = (event: WheelEvent) => {
      if (event.ctrlKey || event.metaKey) {
        event.preventDefault();
        if (activeTabRef.current !== "page") return;
        const shell = pageSurfaceBodyRef.current?.querySelector(".staff-preview-shell") as HTMLElement | null;
        if (!shell?.contains(event.target as Node)) return;

        const oldScale = pageScaleRef.current;
        const delta = event.deltaY < 0 ? 0.1 : -0.1;
        const newScale = Math.max(0.2, Math.min(3.0, Math.round((oldScale + delta) * 100) / 100));
        if (newScale === oldScale) return;

        const BASE = 800;
        const oldWidth = BASE * oldScale;
        const newWidth = BASE * newScale;
        const ratio = newScale / oldScale;
        const shellWidth = shell.clientWidth;

        const oldCenterOffset = Math.max(0, (shellWidth - oldWidth) / 2);
        const newCenterOffset = Math.max(0, (shellWidth - newWidth) / 2);

        const rect = shell.getBoundingClientRect();
        const mx = event.clientX - rect.left;
        const my = event.clientY - rect.top;

        const cursorInContentX = mx - oldCenterOffset + shell.scrollLeft;
        const cursorInContentY = my + shell.scrollTop;

        const targetScrollX = newCenterOffset + cursorInContentX * ratio - mx;
        const targetScrollY = cursorInContentY * ratio - my;

        setFitWidth(false);
        applyScaleCss(newScale);

        void shell.scrollWidth;
        shell.scrollLeft = targetScrollX;
        shell.scrollTop = targetScrollY;
      }
    };

    const handleTouchStart = (event: TouchEvent) => {
      if (event.touches.length === 2 && pageSurfaceBodyRef.current?.contains(event.target as Node)) {
        const t1 = event.touches[0];
        const t2 = event.touches[1];
        if (!t1 || !t2) return;
        const dx = t1.pageX - t2.pageX;
        const dy = t1.pageY - t2.pageY;
          touchStateRef.current = {
          distance: Math.sqrt(dx * dx + dy * dy),
          initialScale: pageScaleRef.current,
          centerX: (t1.pageX + t2.pageX) / 2,
          centerY: (t1.pageY + t2.pageY) / 2,
        };
      }
    };

    const handleTouchMove = (event: TouchEvent) => {
      if (event.touches.length === 2 && pageSurfaceBodyRef.current?.contains(event.target as Node)) {
        const t1 = event.touches[0];
        const t2 = event.touches[1];
        if (!t1 || !t2) return;

        event.preventDefault(); 
        setFitWidth(false);

        const dx = t1.pageX - t2.pageX;
        const dy = t1.pageY - t2.pageY;
        const distance = Math.sqrt(dx * dx + dy * dy);

        if (touchStateRef.current.distance > 0) {
          const ratio = distance / touchStateRef.current.distance;
          const oldScale = pageScaleRef.current;
          const newScale = Math.max(0.2, Math.min(3.0, touchStateRef.current.initialScale * ratio));
          
          const scaleRatio = newScale / oldScale;
          applyScaleCss(newScale);

          const shell = pageSurfaceBodyRef.current?.querySelector(".staff-preview-shell") as HTMLElement | null;
          if (shell) {
            const BASE = 800;
            const oldWidth = BASE * oldScale;
            const newWidth = BASE * newScale;
            const shellWidth = shell.clientWidth;

            const oldCenterOffset = Math.max(0, (shellWidth - oldWidth) / 2);
            const newCenterOffset = Math.max(0, (shellWidth - newWidth) / 2);

            const rect = shell.getBoundingClientRect();
            const mx = touchStateRef.current.centerX - rect.left;
            const my = touchStateRef.current.centerY - rect.top;

            const cursorInContentX = mx - oldCenterOffset + shell.scrollLeft;
            const cursorInContentY = my + shell.scrollTop;

            const targetScrollX = newCenterOffset + cursorInContentX * scaleRatio - mx;
            const targetScrollY = cursorInContentY * scaleRatio - my;

            void shell.scrollWidth;
            shell.scrollLeft = targetScrollX;
            shell.scrollTop = targetScrollY;
          }
        }
      }
    };

    const handleTouchEnd = () => {
      if (touchStateRef.current.distance > 0) {
        applyScaleCss(pageScaleRef.current);
        touchStateRef.current.distance = 0;
      }
    };

    window.addEventListener("wheel", handleGlobalWheel, { passive: false });
    window.addEventListener("touchstart", handleTouchStart, { passive: true });
    window.addEventListener("touchmove", handleTouchMove, { passive: false });
    window.addEventListener("touchend", handleTouchEnd, { passive: true });

    return () => {
      window.removeEventListener("wheel", handleGlobalWheel);
      window.removeEventListener("touchstart", handleTouchStart);
      window.removeEventListener("touchmove", handleTouchMove);
      window.removeEventListener("touchend", handleTouchEnd);
    };
    }, []); // Empty dependency array means listeners are stable and never re-bind

  return (
    <main className="app-shell">
      <header className="app-header">
        <div className="header-branding">
          <DrumIcon />
          <div>
            <h1>
              DrumMark
            </h1>
            <p>{t("brand.subtitle")}</p>
          </div>
        </div>
        <div className="header-actions">
          <button className="export-button" onClick={() => setLocale(locale === "en" ? "zh" : "en")} type="button" aria-label={t("lang.toggle")}>
            {locale === "en" ? "中文" : "EN"}
          </button>
          <button className="theme-toggle-button" onClick={toggleTheme} type="button" aria-label={t("theme.toggle")}>
            <span className="theme-icon theme-icon-light"><SunIcon /></span>
            <span className="theme-icon theme-icon-dark"><MoonIcon /></span>
          </button>
          <a className="export-button" href="docs.html"><BookIcon /> {t("nav.docs")}</a>
        </div>
      </header>

      <section className="workspace">
        <section className={`pane editor-pane${settings.activeTab === "editor" ? " active" : ""}`} style={{ width: editorWidth }}>
          <header className="pane-header">
            <span className="pane-title">{t("tabs.editor")}</span>
            <div className="preview-header-actions mobile-only-actions">
              <Tabs.Root className="editor-pane-tabs" value={settings.activeTab} onValueChange={(v) => updateSetting("activeTab", v as MainTab)}>
                <Tabs.List className="tabs-list">
                  <Tabs.Trigger className="tabs-trigger" value="editor">{t("tabs.editor")}</Tabs.Trigger>
                  <Tabs.Trigger className="tabs-trigger" value="page">{t("tabs.page")}</Tabs.Trigger>
                  <Tabs.Trigger className="tabs-trigger" value="xml">{t("tabs.xml")}</Tabs.Trigger>
                </Tabs.List>
              </Tabs.Root>
            </div>
          </header>
          <DslEditor value={dsl} onChange={setDsl} errors={score.errors} theme={resolvedTheme} />
        </section>

        <div className="resizer" onMouseDown={handleMouseDown} onTouchStart={handleTouchStart} style={{ touchAction: "none" }} />

        <section className={`pane preview-pane${settings.activeTab !== "editor" ? " active" : ""}`} aria-label={t("panes.preview")}>
          <header className="pane-header">
            <span className="pane-title">{t("panes.preview")}</span>
            <div className="preview-header-actions">
              <Tabs.Root className="preview-pane-tabs" value={settings.activeTab} onValueChange={(v) => updateSetting("activeTab", v as MainTab)}>
                <Tabs.List className="tabs-list">
                  <Tabs.Trigger className="tabs-trigger tab-hide-desktop" value="editor">{t("tabs.editor")}</Tabs.Trigger>
                  <Tabs.Trigger className="tabs-trigger" value="page">{t("tabs.page")}</Tabs.Trigger>
                  <Tabs.Trigger className="tabs-trigger" value="xml">{t("tabs.xml")}</Tabs.Trigger>
                </Tabs.List>
              </Tabs.Root>
            </div>
          </header>
          
          <div className="preview-container">
            <div className="preview-content">
              <div className={`preview-surface${settings.activeTab === "page" ? " active" : ""}`} aria-hidden={settings.activeTab !== "page"}>
                <div className="surface-toolbar page-surface-toolbar">
                  <div className="toolbar-group">
                    <Popover.Root open={pageZoomMenuOpen} onOpenChange={setPageZoomMenuOpen}>
                      <Popover.Trigger asChild>
                        <button aria-label={t("toolbar.zoomAria")} className="surface-icon-button" type="button" title={t("toolbar.zoomTitle", { percent: pageZoomPercent })}>
                          {Math.abs(pageScaleRef.current - 1.0) < 0.001 ? <SearchIcon /> : (pageScaleRef.current < 1 ? <SearchMinusIcon /> : <SearchPlusIcon />)}
                        </button>
                      </Popover.Trigger>
                      <Popover.Portal>
                        <Popover.Content className="zoom-popover-content" sideOffset={4}>
                          <div className="page-zoom-readout">{fitWidth ? t("toolbar.fitWidth") : `${pageZoomPercent}%`}</div>
                          <div className="page-zoom-buttons">
                            <button className="page-zoom-action" onClick={() => adjustPageScale(-0.1)} type="button">-</button>
                            <button className="page-zoom-reset" onClick={() => { setFitWidth(false); applyScaleCss(1); setPageZoomMenuOpen(false); }} type="button">100%</button>
                            <button className="page-zoom-action" onClick={() => adjustPageScale(0.1)} type="button">+</button>
                            <button className="page-zoom-reset fit-width-button" onClick={() => setFitWidth(true)} type="button">{t("toolbar.fitWidth")}</button>
                          </div>
                        </Popover.Content>
                      </Popover.Portal>
                    </Popover.Root>
                    <button className="surface-icon-button" onClick={handlePrint} type="button" title={t("toolbar.print")}>
                      <PrinterIcon />
                    </button>
                    <button className={`surface-icon-button${settingsVisible ? " active" : ""}`} onClick={() => setSettingsVisible(!settingsVisible)} type="button" title={t("toolbar.settings")}>
                      <SettingsIcon />
                    </button>
                  </div>
                </div>
                <div className="page-surface-body" ref={pageSurfaceBodyRef}>
                  {settings.activeTab === "page" ? (
                    <PagePreview
                      score={hasRenderableScore ? score : null}
                      pagePadding={settings.pagePadding}
                      staffScale={settings.staffScale}
                      headerHeight={settings.headerHeight}
                      headerStaffSpacing={settings.headerStaffSpacing}
                      systemSpacing={settings.systemSpacing}
                      stemLength={settings.stemLength}
                      voltaSpacing={settings.voltaSpacing}
                      hairpinOffsetY={settings.hairpinOffsetY}
                      hideVoice2Rests={settings.hideVoice2Rests}
                      tempoOffsetX={settings.tempoOffsetX}
                      tempoOffsetY={settings.tempoOffsetY}
                      measureNumberOffsetX={settings.measureNumberOffsetX}
                      measureNumberOffsetY={settings.measureNumberOffsetY}
                      measureNumberFontSize={settings.measureNumberFontSize}
                      durationSpacingCompression={settings.durationSpacingCompression}
                      measureWidthCompression={settings.measureWidthCompression}
                      active={true}
                      theme={resolvedTheme}
                    />
                  ) : null}
                </div>
              </div>
              <div className={`preview-surface${settings.activeTab === "xml" ? " active" : ""}`} aria-hidden={settings.activeTab !== "xml"}>
                <div className="surface-toolbar xml-surface-toolbar">
                  <div className="toolbar-group">
                    <button className="surface-icon-button" onClick={xmlToggleAll} type="button" title={xmlIsAllCollapsed ? t("toolbar.expandAll") : t("toolbar.collapseAll")}>
                      {xmlIsAllCollapsed ? <ExpandAllIcon /> : <CollapseAllIcon />}
                    </button>
                    <button className="surface-icon-button" disabled={!canExport || isXmlPending} onClick={handleMusicXmlExport} type="button" title={isXmlPending ? t("generating.musicxml") : t("toolbar.export")}>
                      <SaveIcon />
                    </button>
                  </div>
                </div>
                {settings.activeTab === "xml" ? (
                  isXmlPending ? (
                    <div className="xml-preview xml-pending" aria-label={t("xml.previewAria")}>
                      <span>{t("generating.musicxml")}</span>
                    </div>
                  ) : staffXml ? (
                    <MusicXmlPreview xml={staffXml} collapsed={xmlCollapsed} toggle={xmlToggle} />
                  ) : (
                    <div className="xml-preview xml-pending" aria-label={t("xml.previewAria")}>
                      <span>{t("xml.emptyState")}</span>
                    </div>
                  )
                ) : null}
              </div>
              </div>

            <aside className={`settings-panel${settingsVisible ? " active" : ""}`}>
              <SettingsPanel
                settings={settings}
                updateSetting={updateSetting}
                updatePagePadding={updatePagePadding}
                debugMode={debugMode}
              />
            </aside>
          </div>
        </section>
      </section>

      <footer className="status-bar">
        <div className="status-left">
          {score.errors.length > 0 ? (
            <button 
              className={`status-error-toggle${showErrors ? " active" : ""}`}
              onClick={() => setShowErrors(!showErrors)}
              type="button"
            >
              {t(score.errors.length === 1 ? "status.errors_one" : "status.errors_other", { count: score.errors.length })}
            </button>
          ) : (
            <span className="status-success">{t("status.valid")}</span>
          )}
        </div>
        <div className="status-right">{t("status.lines", { count: score.ast.paragraphs.length })} • {t("status.repeats", { count: score.ast.repeatSpans.length })}</div>
      </footer>

      {score.errors.length > 0 && showErrors && (
        <div className="error-list">
          <div className="error-list-header">
            <span>{t("errorPanel.title")}</span>
            <button onClick={() => setShowErrors(false)}>{t("errorPanel.close")}</button>
          </div>
          <div className="error-list-content">
            {score.errors.map((error, index) => (
              <div className="error-item" key={`${error.line}-${error.column}-${index}`}>
                <span className="error-loc">[{error.line}:{error.column}]</span>
                <span>{error.message}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </main>
  );
}
