export const I18N_KEYS = [
  "brand.subtitle",

  "theme.toggle",
  "lang.toggle",
  "nav.docs",

  "tabs.editor",
  "tabs.page",
  "tabs.xml",
  "panes.preview",

  "status.valid",
  "status.errors_one",
  "status.errors_other",
  "status.lines",
  "status.repeats",

  "preview.rendering",
  "preview.error",

  "toolbar.zoomAria",
  "toolbar.zoomTitle",
  "toolbar.fitWidth",
  "toolbar.print",
  "toolbar.settings",
  "toolbar.expandAll",
  "toolbar.collapseAll",
  "toolbar.export",

  "generating.musicxml",
  "xml.emptyState",
  "xml.previewAria",

  "errorPanel.title",
  "errorPanel.close",
  "alert.printPopup",

  "settings.pageLayout",
  "settings.notes",
  "settings.staffLayout",
  "settings.topMargin",
  "settings.bottomMargin",
  "settings.leftMargin",
  "settings.rightMargin",
  "settings.hideVoice2Rests",
  "settings.stemLength",
  "settings.staffScale",
  "settings.systemSpacing",
  "settings.titleHeight",
  "settings.titleGap",
  "settings.voltaOffset",
  "settings.hairpinOffset",
  "settings.decrease",
  "settings.increase",
] as const;

export type I18nKey = typeof I18N_KEYS[number];
