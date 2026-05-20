use js_sys::{Array, Object};
use wasm_bindgen::prelude::*;

pub const RENDER_SCORE_VERSION: &str = "1";
pub const LAYOUT_SCENE_VERSION: &str = "1";
pub const CANONICAL_METRICS_VERSION: &str = "2026-05-13";
const BASE_FONT_SIZE_PT: f32 = 30.0;

// ── Core Render Contract ────────────────────────────────────────

/// Musical fraction (numerator/denominator) for start times and durations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fraction {
    pub numerator: u32,
    pub denominator: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderScore {
    pub version: String,
    pub header: RenderHeader,
    pub tracks: Vec<RenderTrack>,
    pub measures: Vec<RenderMeasure>,
    pub errors: Vec<String>,
    pub repeat_spans: Vec<RepeatSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderHeader {
    pub tempo: u32,
    pub time_beats: u32,
    pub time_beat_unit: u32,
    pub divisions: u32,
    pub note_value: u32,
    pub grouping: Vec<u32>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub composer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderTrack {
    pub id: String,
    pub family: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderMeasure {
    pub index: u32,
    pub global_index: u32,
    pub paragraph_index: u32,
    pub measure_in_paragraph: u32,
    pub source_line: u32,
    pub events: Vec<RenderEvent>,
    pub barline: Option<String>,
    pub closing_barline: Option<String>,
    pub start_nav: Option<NavMarker>,
    pub end_nav: Option<NavJump>,
    pub volta_indices: Option<Vec<u32>>,
    pub hairpins: Vec<HairpinSpan>,
    pub measure_repeat_slashes: Option<u32>,
    pub multi_rest_count: Option<u32>,
    pub note_value: u32,
    pub volta_terminator: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderEvent {
    pub track: String,
    pub track_family: String,
    pub start: Fraction,
    pub duration: Fraction,
    pub kind: EventKind,
    pub glyph: String,
    pub modifiers: Vec<String>,
    pub modifier: Option<String>,
    pub voice: u8,
    pub beam: String,
    pub tuplet: Option<(u32, u32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepeatSpan {
    pub start_measure: u32,
    pub end_measure: u32,
    pub times: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    Hit,
    Rest,
    Sticking,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavMarker {
    Segno,
    Coda,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavJump {
    Fine,
    DC,
    DS,
    DCalFine,
    DCalCoda,
    DSalFine,
    DSalCoda,
    ToCoda,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HairpinSpan {
    pub kind: HairpinKind,
    pub start: Fraction,
    pub end: Fraction,
    pub start_measure_index: u32,
    pub end_measure_index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HairpinKind {
    Crescendo,
    Decrescendo,
}

// Compatibility aliases while the old source-driven path still exists.
pub type NormalizedScore = RenderScore;
pub type NormalizedHeader = RenderHeader;
pub type NormalizedTrack = RenderTrack;
pub type NormalizedMeasure = RenderMeasure;
pub type NormalizedEvent = RenderEvent;
pub type Hairpin = HairpinSpan;

// ── Canonical Metrics Contract ──────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphRole {
    NoteheadBlack,
    NoteheadX,
    NoteheadDiamond,
    NoteheadCircleX,
    NoteheadRim,
    Flag8thUp,
    Flag8thDown,
    Flag16thUp,
    Flag16thDown,
    Flag32ndUp,
    Flag32ndDown,
    PercussionClef,
    TimeSignatureDigit,
    RestWhole,
    RestHalf,
    RestQuarter,
    RestEighth,
    RestSixteenth,
    RestThirtySecond,
    RepeatLeft,
    RepeatRight,
    RepeatDot,
    ArticAccentAbove,
    ArticAccentBelow,
    MeasureRepeatMark1Bar,
    MeasureRepeatMark2Bars,
    MultiRestBar,
    NavigationSegno,
    NavigationCoda,
    MetNoteQuarterUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextRole {
    Title,
    Subtitle,
    Composer,
    Tempo,
    PercussionClef,
    TimeSignatureDigit,
    Sticking,
    CountLabel,
    MeasureNumber,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphPoint {
    pub x_ss: f32,
    pub y_ss: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanonicalGlyphMetric {
    pub role: GlyphRole,
    pub smufl_codepoint: u32,
    /// Glyph width used by layout, in staff-space units.
    ///
    /// Bravura's checked-in SMuFL metadata exposes bounding boxes and anchors,
    /// but not advance widths, so this is currently the bbox width for each
    /// glyph we use.
    pub width_ss: f32,
    /// Bounding box bottom-left (staff-space units, from SMuFL metadata).
    pub bbox_sw_x_ss: f32,
    pub bbox_sw_y_ss: f32,
    /// Bounding box top-right (staff-space units, from SMuFL metadata).
    pub bbox_ne_x_ss: f32,
    pub bbox_ne_y_ss: f32,
    /// SMuFL `glyphsWithAnchors.stemUpSE` / `stemUpNW`, when present.
    pub stem_up_anchor_ss: Option<GlyphPoint>,
    /// SMuFL `glyphsWithAnchors.stemDownNW` / `stemDownSW`, when present.
    pub stem_down_anchor_ss: Option<GlyphPoint>,
}

impl CanonicalGlyphMetric {
    /// Convert a staff-space value to points at the given font size.
    /// SMuFL is designed at 4 staff-spaces per em.
    fn ss_to_pt(ss: f32, font_size_pt: f32) -> f32 {
        ss * font_size_pt / 4.0
    }

    /// Width used by layout (staff-space units).
    pub fn width_ss(&self) -> f32 {
        self.width_ss
    }

    /// Width derived from the bounding box (staff-space units).
    pub fn bbox_width_ss(&self) -> f32 {
        self.bbox_ne_x_ss - self.bbox_sw_x_ss
    }

    /// Height derived from the bounding box (staff-space units).
    pub fn bbox_height_ss(&self) -> f32 {
        self.bbox_ne_y_ss - self.bbox_sw_y_ss
    }

    /// Visual center X (staff-space units) — midpoint of the bbox.
    pub fn bbox_center_x_ss(&self) -> f32 {
        (self.bbox_sw_x_ss + self.bbox_ne_x_ss) / 2.0
    }

    /// Visual center Y (staff-space units) — midpoint of the bbox.
    pub fn bbox_center_y_ss(&self) -> f32 {
        (self.bbox_sw_y_ss + self.bbox_ne_y_ss) / 2.0
    }

    pub fn width_pt(&self, font_size_pt: f32) -> f32 {
        Self::ss_to_pt(self.width_ss(), font_size_pt)
    }

    pub fn bbox_height_pt(&self, font_size_pt: f32) -> f32 {
        Self::ss_to_pt(self.bbox_height_ss(), font_size_pt)
    }

    pub fn bbox_center_x_pt(&self, font_size_pt: f32) -> f32 {
        Self::ss_to_pt(self.bbox_center_x_ss(), font_size_pt)
    }

    pub fn bbox_center_y_pt(&self, font_size_pt: f32) -> f32 {
        Self::ss_to_pt(self.bbox_center_y_ss(), font_size_pt)
    }

    pub fn stem_anchor_for_direction(&self, stem_up: bool) -> Option<GlyphPoint> {
        if stem_up {
            self.stem_up_anchor_ss
        } else {
            self.stem_down_anchor_ss
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanonicalTextMetric {
    pub role: TextRole,
    pub font_family: &'static str,
    pub font_size_pt: f32,
    pub line_height_pt: f32,
    pub average_advance_pt: f32,
    pub ascent_pt: f32,
    pub descent_pt: f32,
}

// ── Platform-Neutral Scene Contract ─────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutScene {
    pub version: String,
    pub metrics_version: String,
    pub pages: Vec<ScenePage>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScenePage {
    pub index: u32,
    pub width_pt: f32,
    pub height_pt: f32,
    pub systems: Vec<SceneSystem>,
    pub measures: Vec<SceneMeasure>,
    pub items: Vec<SceneItem>,
    pub composites: Vec<SceneComposite>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneSystem {
    pub id: String,
    pub index: u32,
    pub page_index: u32,
    pub x_pt: f32,
    pub y_pt: f32,
    pub width_pt: f32,
    pub height_pt: f32,
    pub measure_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneMeasure {
    pub id: String,
    pub index: u32,
    pub global_index: u32,
    pub system_id: String,
    pub x_pt: f32,
    pub y_pt: f32,
    pub width_pt: f32,
    pub height_pt: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneItemKind {
    GlyphRun,
    TextRun,
    LineSegment,
    Rect,
    Polyline,
    Path,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneItem {
    pub id: String,
    pub measure_id: Option<String>,
    pub anchor_item_id: Option<String>,
    pub role: String,
    pub kind: SceneItemKind,
    pub z_index: i32,
    pub primitive: ScenePrimitive,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScenePrimitive {
    GlyphRun(GlyphRun),
    TextRun(TextRun),
    LineSegment(LineSegment),
    Rect(RectShape),
    Polyline(Polyline),
    Path(PathShape),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlyphRun {
    pub x_pt: f32,
    pub y_pt: f32,
    pub glyph_role: GlyphRole,
    pub glyph_count: u32,
    pub smufl_codepoint: Option<u32>,
    pub font_family: String,
    pub font_size_pt: f32,
    pub fill: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextRun {
    pub x_pt: f32,
    pub y_pt: f32,
    pub text_role: TextRole,
    pub text: String,
    pub font_family: String,
    pub font_size_pt: f32,
    pub fill: String,
    pub text_anchor: Option<String>,
    pub font_weight: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineSegment {
    pub x1_pt: f32,
    pub y1_pt: f32,
    pub x2_pt: f32,
    pub y2_pt: f32,
    pub stroke: String,
    pub stroke_width: f32,
    pub stroke_line_cap: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RectShape {
    pub x_pt: f32,
    pub y_pt: f32,
    pub width_pt: f32,
    pub height_pt: f32,
    pub fill: String,
    pub stroke: Option<String>,
    pub stroke_width: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Polyline {
    pub points_pt: Vec<(f32, f32)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathShape {
    pub d: String,
    pub fill: String,
    pub stroke: Option<String>,
    pub stroke_width: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagPathRole {
    EighthUp,
    EighthDown,
    SixteenthUp,
    SixteenthDown,
    ThirtySecondUp,
    ThirtySecondDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositeKind {
    RepeatSpan,
    Volta,
    Hairpin,
    Navigation,
    MeasureRepeat,
    MultiRest,
    TextBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanFragmentKind {
    SingleSegment,
    Start,
    Continuation,
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneComposite {
    pub id: String,
    pub kind: CompositeKind,
    pub fragment: SpanFragmentKind,
    pub child_item_ids: Vec<String>,
    pub label: Option<String>,
    pub count: Option<u32>,
    pub start_anchor_id: Option<String>,
    pub end_anchor_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
struct SystemLayoutBox {
    system_index: u32,
    system_id: String,
    local_system_origin_y: f32,
    staff_top: f32,
    staff_bottom: f32,
    visual_top: f32,
    visual_bottom: f32,
    width_pt: f32,
    measures: Vec<SceneMeasure>,
    systems: Vec<SceneSystem>,
    items: Vec<SceneItem>,
    composites: Vec<SceneComposite>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
struct HeaderLayoutBox {
    items: Vec<SceneItem>,
    composites: Vec<SceneComposite>,
    visual_top: f32,
    visual_bottom: f32,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
struct PlacedSystemBox {
    system_index: u32,
    system_id: String,
    page_index: u32,
    page_x: f32,
    page_y: f32,
    local_visual_top: f32,
    local_system_origin_y: f32,
    width_pt: f32,
    measure_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
struct BoxPaginationResult {
    placements: Vec<PlacedSystemBox>,
    issues: Vec<String>,
}

#[allow(dead_code)]
fn layout_overflow_warning(
    page_index: u32,
    system_id: &str,
    visual_height: f32,
    available_height: f32,
) -> String {
    format!(
        "LAYOUT_WARNING overflow page={page_index} system={system_id} visualHeight={visual_height:.2} availableHeight={available_height:.2}"
    )
}

#[allow(dead_code)]
fn paginate_system_boxes(
    boxes: &[SystemLayoutBox],
    header: &HeaderLayoutBox,
    opts: &LayoutOptions,
) -> BoxPaginationResult {
    let mut placements = Vec::new();
    let mut issues = Vec::new();
    let content_bottom = opts.page_height_pt - opts.bottom_margin_pt;
    let available_height = (content_bottom - opts.top_margin_pt).max(0.0);
    let mut page_index = 0_u32;
    let mut cursor_y = page0_first_system_cursor(opts, header);
    let mut systems_on_page = 0usize;

    for system_box in boxes {
        let visual_height = system_box.visual_bottom - system_box.visual_top;
        let mut placement_y = cursor_y
            + if systems_on_page == 0 {
                0.0
            } else {
                opts.system_spacing_pt
            };
        if systems_on_page > 0 && placement_y + visual_height > content_bottom {
            page_index += 1;
            systems_on_page = 0;
            cursor_y = opts.top_margin_pt;
            placement_y = cursor_y;
        }

        if systems_on_page == 0 && placement_y + visual_height > content_bottom {
            issues.push(layout_overflow_warning(
                page_index,
                &system_box.system_id,
                visual_height,
                available_height,
            ));
        }

        placements.push(PlacedSystemBox {
            system_index: system_box.system_index,
            system_id: system_box.system_id.clone(),
            page_index,
            page_x: opts.left_margin_pt,
            page_y: placement_y,
            local_visual_top: system_box.visual_top,
            local_system_origin_y: system_box.local_system_origin_y,
            width_pt: system_box.width_pt,
            measure_ids: system_box
                .measures
                .iter()
                .map(|measure| measure.id.clone())
                .collect(),
        });
        cursor_y = placement_y + visual_height;
        systems_on_page += 1;
    }

    BoxPaginationResult { placements, issues }
}

#[allow(dead_code)]
fn bounds_for_items(items: &[SceneItem]) -> Result<Option<SceneItemBounds>, String> {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let mut found = false;
    for item in items {
        let bounds = scene_item_bounds(item)?;
        min_x = min_x.min(bounds.x);
        min_y = min_y.min(bounds.y);
        max_x = max_x.max(bounds.x + bounds.width);
        max_y = max_y.max(bounds.y + bounds.height);
        found = true;
    }
    Ok(found.then_some(SceneItemBounds {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    }))
}

#[allow(dead_code)]
fn page0_first_system_cursor(opts: &LayoutOptions, header: &HeaderLayoutBox) -> f32 {
    let fixed_cursor = opts.top_margin_pt + opts.header_height_pt + opts.header_staff_spacing_pt;
    let visual_cursor = header.visual_bottom + opts.header_staff_spacing_pt;
    fixed_cursor.max(visual_cursor)
}

#[derive(Debug, Clone, PartialEq)]
struct WireLayoutScene {
    version: String,
    metrics_version: String,
    pages: Vec<WireScenePage>,
    issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct WireScenePage {
    index: u32,
    width_pt: f32,
    height_pt: f32,
    systems: Vec<WireSceneSystem>,
    measures: Vec<WireSceneMeasure>,
    items: Vec<WireSceneItem>,
    composites: Vec<WireSceneComposite>,
}

#[derive(Debug, Clone, PartialEq)]
struct WireSceneSystem {
    id: String,
    index: u32,
    page_index: u32,
    x_pt: f32,
    y_pt: f32,
    width_pt: f32,
    height_pt: f32,
    measure_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct WireSceneMeasure {
    id: String,
    index: u32,
    global_index: u32,
    system_id: String,
    x_pt: f32,
    y_pt: f32,
    width_pt: f32,
    height_pt: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct WireSceneItem {
    id: String,
    measure_id: Option<String>,
    anchor_item_id: Option<String>,
    role: String,
    kind: &'static str,
    z_index: i32,
    primitive: WireScenePrimitive,
}

#[derive(Debug, Clone, PartialEq)]
enum WireScenePrimitive {
    GlyphRun {
        x_pt: f32,
        y_pt: f32,
        glyph_role: &'static str,
        glyph_count: u32,
        codepoint: Option<u32>,
        font_family: String,
        font_size_pt: f32,
        fill: String,
    },
    TextRun {
        x_pt: f32,
        y_pt: f32,
        text_role: &'static str,
        text: String,
        font_family: String,
        font_size_pt: f32,
        fill: String,
        text_anchor: Option<String>,
        font_weight: Option<String>,
    },
    LineSegment {
        x1_pt: f32,
        y1_pt: f32,
        x2_pt: f32,
        y2_pt: f32,
        stroke: String,
        stroke_width: f32,
        stroke_line_cap: Option<String>,
    },
    Rect {
        x_pt: f32,
        y_pt: f32,
        width_pt: f32,
        height_pt: f32,
        fill: String,
        stroke: Option<String>,
        stroke_width: Option<f32>,
    },
    Polyline {
        points_pt: Vec<(f32, f32)>,
    },
    Path {
        d: String,
        fill: String,
        stroke: Option<String>,
        stroke_width: Option<f32>,
    },
}

#[derive(Debug, Clone, PartialEq)]
struct WireSceneComposite {
    id: String,
    kind: &'static str,
    fragment: &'static str,
    child_item_ids: Vec<String>,
    label: Option<String>,
    count: Option<u32>,
    start_anchor_id: Option<String>,
    end_anchor_id: Option<String>,
}

// ── Track Families ───────────────────────────────────────────────

pub fn track_family(track: &str) -> &str {
    match track {
        "HH" | "RC" | "RC2" | "C" | "C2" | "SPL" | "CHN" => "cymbal",
        "SD" | "BD" | "BD2" | "T1" | "T2" | "T3" | "T4" | "ST" => "drum",
        "HF" => "pedal",
        "CB" | "WB" | "CL" => "percussion",
        _ => "auxiliary",
    }
}

// ── Canonical Metrics ────────────────────────────────────────────

fn glyph_metric(
    role: GlyphRole,
    smufl_codepoint: u32,
    bbox_sw: [f32; 2],
    bbox_ne: [f32; 2],
    stem_up_anchor: Option<[f32; 2]>,
    stem_down_anchor: Option<[f32; 2]>,
) -> CanonicalGlyphMetric {
    CanonicalGlyphMetric {
        role,
        smufl_codepoint,
        width_ss: bbox_ne[0] - bbox_sw[0],
        bbox_sw_x_ss: bbox_sw[0],
        bbox_sw_y_ss: bbox_sw[1],
        bbox_ne_x_ss: bbox_ne[0],
        bbox_ne_y_ss: bbox_ne[1],
        stem_up_anchor_ss: stem_up_anchor.map(|point| GlyphPoint {
            x_ss: point[0],
            y_ss: point[1],
        }),
        stem_down_anchor_ss: stem_down_anchor.map(|point| GlyphPoint {
            x_ss: point[0],
            y_ss: point[1],
        }),
    }
}

pub fn canonical_glyph_metric(role: GlyphRole) -> CanonicalGlyphMetric {
    match role {
        GlyphRole::NoteheadBlack => glyph_metric(
            role,
            0xE0A4,
            [0.0, -0.5],
            [1.18, 0.5],
            Some([1.49, 0.16]),
            Some([0.1, -0.16]),
        ),
        GlyphRole::NoteheadX => glyph_metric(
            role,
            0xE0A9,
            [0.0, -0.5],
            [1.16, 0.5],
            Some([1.49, 0.5]),
            Some([0.0, -0.5]),
        ),
        GlyphRole::NoteheadDiamond => glyph_metric(
            role,
            0xE0B2,
            [0.0, -0.5],
            [1.0, 0.5],
            Some([1.0, 0.0]),
            Some([0.0, 0.0]),
        ),
        GlyphRole::NoteheadCircleX => glyph_metric(
            role,
            0xE0B3,
            [0.0, -0.5],
            [0.996, 0.5],
            Some([0.996, 0.0]),
            Some([0.0, 0.0]),
        ),
        GlyphRole::NoteheadRim => glyph_metric(
            role,
            0xE0CE,
            [-0.32, -0.66],
            [1.5, 0.668],
            Some([1.18, 0.164]),
            Some([0.0, -0.172]),
        ),
        GlyphRole::Flag8thUp => glyph_metric(
            role,
            0xE240,
            [0.0, -3.2407685],
            [1.056, 0.036],
            Some([0.0, -0.04]),
            None,
        ),
        GlyphRole::Flag8thDown => glyph_metric(
            role,
            0xE241,
            [0.0, -0.056],
            [1.224, 3.2328966],
            None,
            Some([0.0, 0.132]),
        ),
        GlyphRole::Flag16thUp => glyph_metric(
            role,
            0xE242,
            [0.0, -3.252],
            [1.116, 0.008],
            Some([0.0, -0.088]),
            None,
        ),
        GlyphRole::Flag16thDown => glyph_metric(
            role,
            0xE243,
            [0.0, -0.036],
            [1.1635807, 3.2480257],
            None,
            Some([0.0, 0.128]),
        ),
        GlyphRole::Flag32ndUp => glyph_metric(
            role,
            0xE244,
            [0.0, -3.248],
            [1.044, 0.596],
            Some([0.0, 0.376]),
            None,
        ),
        GlyphRole::Flag32ndDown => glyph_metric(
            role,
            0xE245,
            [0.0, -0.688],
            [1.092, 3.248],
            None,
            Some([0.0, -0.448]),
        ),
        GlyphRole::PercussionClef => {
            glyph_metric(role, 0xE069, [0.0, -1.0], [1.528, 1.0], None, None)
        }
        GlyphRole::TimeSignatureDigit => {
            glyph_metric(role, 0xE080, [0.08, -1.0], [1.8, 1.004], None, None)
        }
        GlyphRole::RestWhole => {
            glyph_metric(role, 0xE4E3, [0.0, -0.54], [1.128, 0.036], None, None)
        }
        GlyphRole::RestHalf => {
            glyph_metric(role, 0xE4E4, [0.0, -0.008], [1.128, 0.568], None, None)
        }
        GlyphRole::RestQuarter => {
            glyph_metric(role, 0xE4E5, [0.004, -1.5], [1.08, 1.492], None, None)
        }
        GlyphRole::RestEighth => {
            glyph_metric(role, 0xE4E6, [0.0, -1.004], [0.988, 0.696], None, None)
        }
        GlyphRole::RestSixteenth => {
            glyph_metric(role, 0xE4E7, [0.0, -2.0], [1.28, 0.716], None, None)
        }
        GlyphRole::RestThirtySecond => {
            glyph_metric(role, 0xE4E8, [0.0, -2.0], [1.452, 1.704], None, None)
        }
        GlyphRole::RepeatLeft => glyph_metric(role, 0xE040, [0.0, 0.0], [1.464, 4.0], None, None),
        GlyphRole::RepeatRight => {
            glyph_metric(role, 0xE041, [0.004, 0.0], [1.468, 4.0], None, None)
        }
        GlyphRole::RepeatDot => glyph_metric(role, 0xE044, [0.0, -0.2], [0.4, 0.2], None, None),
        GlyphRole::ArticAccentAbove => {
            glyph_metric(role, 0xE4A0, [0.0, 0.004], [1.356, 0.98], None, None)
        }
        GlyphRole::ArticAccentBelow => {
            glyph_metric(role, 0xE4A1, [0.0, -0.976], [1.356, 0.0], None, None)
        }
        GlyphRole::MeasureRepeatMark1Bar => {
            glyph_metric(role, 0xE500, [0.0, -1.0], [2.128, 1.116], None, None)
        }
        GlyphRole::MeasureRepeatMark2Bars => {
            glyph_metric(role, 0xE501, [0.0, -1.0], [3.048, 1.116], None, None)
        }
        GlyphRole::MultiRestBar => {
            glyph_metric(role, 0xE4EE, [0.0, -1.084], [3.128, 1.044], None, None)
        }
        GlyphRole::NavigationSegno => {
            glyph_metric(role, 0xE047, [0.016, -0.108], [2.2, 3.036], None, None)
        }
        GlyphRole::NavigationCoda => {
            glyph_metric(role, 0xE048, [-0.016, -0.632], [3.82, 3.592], None, None)
        }
        GlyphRole::MetNoteQuarterUp => {
            glyph_metric(role, 0xE1D5, [0.0, -0.564], [1.328, 2.752], None, None)
        }
    }
}

pub fn canonical_text_metric(role: TextRole) -> CanonicalTextMetric {
    match role {
        TextRole::Title => CanonicalTextMetric {
            role,
            font_family: "Academico",
            font_size_pt: 24.0,
            line_height_pt: 28.0,
            average_advance_pt: 11.0,
            ascent_pt: 18.0,
            descent_pt: 6.0,
        },
        TextRole::Subtitle => CanonicalTextMetric {
            role,
            font_family: "Academico",
            font_size_pt: 18.0,
            line_height_pt: 22.0,
            average_advance_pt: 8.0,
            ascent_pt: 14.0,
            descent_pt: 4.0,
        },
        TextRole::Composer => CanonicalTextMetric {
            role,
            font_family: "Academico",
            font_size_pt: 14.0,
            line_height_pt: 18.0,
            average_advance_pt: 7.0,
            ascent_pt: 11.0,
            descent_pt: 3.0,
        },
        TextRole::Tempo => CanonicalTextMetric {
            role,
            font_family: "Academico",
            font_size_pt: 14.0,
            line_height_pt: 18.0,
            average_advance_pt: 7.0,
            ascent_pt: 11.0,
            descent_pt: 3.0,
        },
        TextRole::PercussionClef => CanonicalTextMetric {
            role,
            font_family: "Bravura",
            font_size_pt: 30.0,
            line_height_pt: 32.0,
            average_advance_pt: 14.0,
            ascent_pt: 24.0,
            descent_pt: 6.0,
        },
        TextRole::TimeSignatureDigit => CanonicalTextMetric {
            role,
            font_family: "Bravura",
            font_size_pt: 30.0,
            line_height_pt: 32.0,
            average_advance_pt: 10.0,
            ascent_pt: 24.0,
            descent_pt: 6.0,
        },
        TextRole::Sticking => CanonicalTextMetric {
            role,
            font_family: "Academico",
            font_size_pt: 12.0,
            line_height_pt: 14.0,
            average_advance_pt: 6.0,
            ascent_pt: 9.0,
            descent_pt: 3.0,
        },
        TextRole::CountLabel => CanonicalTextMetric {
            role,
            font_family: "Bravura",
            font_size_pt: 12.0,
            line_height_pt: 14.0,
            average_advance_pt: 6.0,
            ascent_pt: 9.0,
            descent_pt: 3.0,
        },
        TextRole::MeasureNumber => CanonicalTextMetric {
            role,
            font_family: "Academico",
            font_size_pt: 10.0,
            line_height_pt: 12.0,
            average_advance_pt: 5.0,
            ascent_pt: 8.0,
            descent_pt: 2.0,
        },
    }
}

pub fn canonical_flag_path(
    role: FlagPathRole,
    stem_x: f32,
    stem_tip_y: f32,
) -> Vec<Vec<(f32, f32)>> {
    match role {
        FlagPathRole::EighthUp => vec![vec![
            (stem_x, stem_tip_y),
            (stem_x + 5.0, stem_tip_y + 1.5),
            (stem_x + 8.0, stem_tip_y + 6.0),
            (stem_x + 4.0, stem_tip_y + 9.5),
        ]],
        FlagPathRole::EighthDown => vec![vec![
            (stem_x, stem_tip_y),
            (stem_x + 5.0, stem_tip_y - 1.5),
            (stem_x + 8.0, stem_tip_y - 6.0),
            (stem_x + 4.0, stem_tip_y - 9.5),
        ]],
        FlagPathRole::SixteenthUp => vec![
            vec![
                (stem_x, stem_tip_y),
                (stem_x + 5.0, stem_tip_y + 1.5),
                (stem_x + 8.0, stem_tip_y + 6.0),
                (stem_x + 4.0, stem_tip_y + 9.5),
            ],
            vec![
                (stem_x, stem_tip_y + 5.0),
                (stem_x + 5.0, stem_tip_y + 6.5),
                (stem_x + 8.0, stem_tip_y + 11.0),
                (stem_x + 4.0, stem_tip_y + 14.5),
            ],
        ],
        FlagPathRole::SixteenthDown => vec![
            vec![
                (stem_x, stem_tip_y),
                (stem_x + 5.0, stem_tip_y - 1.5),
                (stem_x + 8.0, stem_tip_y - 6.0),
                (stem_x + 4.0, stem_tip_y - 9.5),
            ],
            vec![
                (stem_x, stem_tip_y - 5.0),
                (stem_x + 5.0, stem_tip_y - 6.5),
                (stem_x + 8.0, stem_tip_y - 11.0),
                (stem_x + 4.0, stem_tip_y - 14.5),
            ],
        ],
        FlagPathRole::ThirtySecondUp => vec![
            vec![
                (stem_x, stem_tip_y),
                (stem_x + 5.0, stem_tip_y + 1.5),
                (stem_x + 8.0, stem_tip_y + 6.0),
                (stem_x + 4.0, stem_tip_y + 9.5),
            ],
            vec![
                (stem_x, stem_tip_y + 5.0),
                (stem_x + 5.0, stem_tip_y + 6.5),
                (stem_x + 8.0, stem_tip_y + 11.0),
                (stem_x + 4.0, stem_tip_y + 14.5),
            ],
            vec![
                (stem_x, stem_tip_y + 10.0),
                (stem_x + 5.0, stem_tip_y + 11.5),
                (stem_x + 8.0, stem_tip_y + 16.0),
                (stem_x + 4.0, stem_tip_y + 19.5),
            ],
        ],
        FlagPathRole::ThirtySecondDown => vec![
            vec![
                (stem_x, stem_tip_y),
                (stem_x + 5.0, stem_tip_y - 1.5),
                (stem_x + 8.0, stem_tip_y - 6.0),
                (stem_x + 4.0, stem_tip_y - 9.5),
            ],
            vec![
                (stem_x, stem_tip_y - 5.0),
                (stem_x + 5.0, stem_tip_y - 6.5),
                (stem_x + 8.0, stem_tip_y - 11.0),
                (stem_x + 4.0, stem_tip_y - 14.5),
            ],
            vec![
                (stem_x, stem_tip_y - 10.0),
                (stem_x + 5.0, stem_tip_y - 11.5),
                (stem_x + 8.0, stem_tip_y - 16.0),
                (stem_x + 4.0, stem_tip_y - 19.5),
            ],
        ],
    }
}

fn scene_item_kind_name(kind: SceneItemKind) -> &'static str {
    match kind {
        SceneItemKind::GlyphRun => "glyphRun",
        SceneItemKind::TextRun => "textRun",
        SceneItemKind::LineSegment => "lineSegment",
        SceneItemKind::Rect => "rect",
        SceneItemKind::Polyline => "polyline",
        SceneItemKind::Path => "path",
    }
}

fn glyph_role_name(role: GlyphRole) -> &'static str {
    match role {
        GlyphRole::NoteheadBlack => "noteheadBlack",
        GlyphRole::NoteheadX => "noteheadX",
        GlyphRole::NoteheadDiamond => "noteheadDiamond",
        GlyphRole::NoteheadCircleX => "noteheadCircleX",
        GlyphRole::NoteheadRim => "noteheadRim",
        GlyphRole::Flag8thUp => "flag8thUp",
        GlyphRole::Flag8thDown => "flag8thDown",
        GlyphRole::Flag16thUp => "flag16thUp",
        GlyphRole::Flag16thDown => "flag16thDown",
        GlyphRole::Flag32ndUp => "flag32ndUp",
        GlyphRole::Flag32ndDown => "flag32ndDown",
        GlyphRole::PercussionClef => "percussionClef",
        GlyphRole::TimeSignatureDigit => "timeSignatureDigit",
        GlyphRole::RestWhole => "restWhole",
        GlyphRole::RestHalf => "restHalf",
        GlyphRole::RestQuarter => "restQuarter",
        GlyphRole::RestEighth => "restEighth",
        GlyphRole::RestSixteenth => "restSixteenth",
        GlyphRole::RestThirtySecond => "restThirtySecond",
        GlyphRole::RepeatLeft => "repeatLeft",
        GlyphRole::RepeatRight => "repeatRight",
        GlyphRole::RepeatDot => "repeatDot",
        GlyphRole::ArticAccentAbove => "articAccentAbove",
        GlyphRole::ArticAccentBelow => "articAccentBelow",
        GlyphRole::MeasureRepeatMark1Bar => "measureRepeatMark1Bar",
        GlyphRole::MeasureRepeatMark2Bars => "measureRepeatMark2Bars",
        GlyphRole::MultiRestBar => "multiRestBar",
        GlyphRole::NavigationSegno => "navigationSegno",
        GlyphRole::NavigationCoda => "navigationCoda",
        GlyphRole::MetNoteQuarterUp => "metNoteQuarterUp",
    }
}

fn text_role_name(role: TextRole) -> &'static str {
    match role {
        TextRole::Title => "title",
        TextRole::Subtitle => "subtitle",
        TextRole::Composer => "composer",
        TextRole::Tempo => "tempo",
        TextRole::PercussionClef => "percussionClef",
        TextRole::TimeSignatureDigit => "timeSignatureDigit",
        TextRole::Sticking => "sticking",
        TextRole::CountLabel => "countLabel",
        TextRole::MeasureNumber => "measureNumber",
    }
}

fn composite_kind_name(kind: CompositeKind) -> &'static str {
    match kind {
        CompositeKind::RepeatSpan => "repeatSpan",
        CompositeKind::Volta => "volta",
        CompositeKind::Hairpin => "hairpin",
        CompositeKind::Navigation => "navigation",
        CompositeKind::MeasureRepeat => "measureRepeat",
        CompositeKind::MultiRest => "multiRest",
        CompositeKind::TextBlock => "textBlock",
    }
}

fn fragment_kind_name(kind: SpanFragmentKind) -> &'static str {
    match kind {
        SpanFragmentKind::SingleSegment => "singleSegment",
        SpanFragmentKind::Start => "start",
        SpanFragmentKind::Continuation => "continuation",
        SpanFragmentKind::End => "end",
    }
}

// ── SMuFL Glyph Metrics ──────────────────────────────────────────

/// Returns SMuFL glyph metrics for a notehead given track + modifiers.
pub fn notehead_glyph(track: &str, modifiers: &[String], _glyph: &str) -> CanonicalGlyphMetric {
    // Modifier-based noteheads take priority over track family defaults
    for m in modifiers {
        match m.as_str() {
            "open" => return canonical_glyph_metric(GlyphRole::NoteheadCircleX),
            "cross" => return canonical_glyph_metric(GlyphRole::NoteheadX),
            "bell" => return canonical_glyph_metric(GlyphRole::NoteheadDiamond),
            "rim" => return canonical_glyph_metric(GlyphRole::NoteheadRim),
            _ => {}
        }
    }

    let family = track_family(track);

    // Cymbal tracks and hi-hat pedal default to X notehead
    if family == "cymbal" || track == "HF" {
        return canonical_glyph_metric(GlyphRole::NoteheadX);
    }

    // Standard drum notehead
    canonical_glyph_metric(GlyphRole::NoteheadBlack)
}

/// Returns SMuFL metrics for a rest glyph by duration denominator.
pub fn rest_glyph_for_fraction(duration: Fraction) -> CanonicalGlyphMetric {
    match (duration.numerator, duration.denominator) {
        (1, 1) => canonical_glyph_metric(GlyphRole::RestWhole),
        (1, 2) => canonical_glyph_metric(GlyphRole::RestHalf),
        (1, 4) => canonical_glyph_metric(GlyphRole::RestQuarter),
        (1, 8) => canonical_glyph_metric(GlyphRole::RestEighth),
        (1, 16) => canonical_glyph_metric(GlyphRole::RestSixteenth),
        (1, 32) => canonical_glyph_metric(GlyphRole::RestThirtySecond),
        (_, d) if d >= 32 => canonical_glyph_metric(GlyphRole::RestThirtySecond),
        (_, d) if d >= 16 => canonical_glyph_metric(GlyphRole::RestEighth),
        (_, d) if d >= 8 => canonical_glyph_metric(GlyphRole::RestQuarter),
        (_, d) if d >= 4 => canonical_glyph_metric(GlyphRole::RestHalf),
        _ => canonical_glyph_metric(GlyphRole::RestWhole),
    }
}

pub fn rest_glyph(denominator: u32) -> CanonicalGlyphMetric {
    rest_glyph_for_fraction(Fraction {
        numerator: 1,
        denominator,
    })
}

// ── Layout Options ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LayoutOptions {
    // Page
    pub page_width_pt: f32,
    pub page_height_pt: f32,
    pub top_margin_pt: f32,
    pub bottom_margin_pt: f32,
    pub left_margin_pt: f32,
    pub right_margin_pt: f32,
    // Staff
    pub staff_scale: f32,
    pub px_per_quarter: f32,
    // Header/title area. Matches the TS renderer: first system starts at
    // top margin + title area height + title gap.
    pub header_height_pt: f32,
    pub header_staff_spacing_pt: f32,
    // Per-element Y offsets in staff space. Volta spacing is positive upward from the top skyline.
    pub volta_offset_y: f32,
    pub nav_offset_y: f32,
    pub hairpin_offset_y: f32,
    pub sticking_offset_y: f32,
    pub accent_offset_y: f32,
    pub text_offset_y: f32,
    pub tempo_offset_y: f32,
    pub measure_num_offset_y: f32,
    // Padding between edge elements
    pub edge_padding: f32,
    // Stem
    pub stem_len_pt: f32,
    // Inter-system spacing (pt)
    pub system_spacing_pt: f32,
    // Whether to hide lower-voice rests (matching VexFlow hideVoice2Rests)
    pub hide_voice2_rests: bool,
    // Note spacing compression: higher values give more space to longer durations.
    // VexFlow default: 0.6
    pub duration_spacing_compression: f32,
    // Measure width compression: higher values widen busy measures more.
    // VexFlow default: 0.75
    pub measure_width_compression: f32,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            page_width_pt: 612.0,
            page_height_pt: 792.0,
            top_margin_pt: 30.0,
            bottom_margin_pt: 30.0,
            left_margin_pt: 50.0,
            right_margin_pt: 50.0,
            staff_scale: 0.75,
            px_per_quarter: 80.0,
            header_height_pt: 50.0,
            header_staff_spacing_pt: 60.0,
            volta_offset_y: 0.0,
            nav_offset_y: -10.0,
            hairpin_offset_y: 0.0,
            sticking_offset_y: -8.0,
            accent_offset_y: -6.0,
            text_offset_y: -40.0,
            tempo_offset_y: -10.0,
            measure_num_offset_y: -4.0,
            edge_padding: 4.0,
            stem_len_pt: 31.0,
            system_spacing_pt: 30.0,
            hide_voice2_rests: false,
            duration_spacing_compression: 0.6,
            measure_width_compression: 0.75,
        }
    }
}

// ── Staff-Space Workhorse ────────────────────────────────────────

/// 1 staff space = distance between two staff lines. Default: 8pt at 40pt staff.
#[derive(Debug, Clone, Copy)]
pub struct StaffSpace {
    pub pt_per_ss: f32,
}

impl Default for StaffSpace {
    fn default() -> Self {
        Self { pt_per_ss: 8.0 }
    }
}

impl StaffSpace {
    pub fn to_pixels(&self, staff_height_px: f32) -> f32 {
        staff_height_px / 4.0 // 4 staff spaces per staff height
    }

    pub fn to_pt(&self, ss: f32) -> f32 {
        ss * self.pt_per_ss
    }

    pub fn from_pt(&self, pt: f32) -> f32 {
        pt / self.pt_per_ss
    }
}

// ── Staff Y Positions ────────────────────────────────────────────

/// Vertical position of each drum kit element in staff-space units
/// (0 = top staff line, positive = downward).
pub fn staff_y_for_track(track: &str) -> f32 {
    // VexFlow-compatible staff positions derived from the legacy instrument map.
    // Units are measured from the top staff line in staff-space units where
    // 0.5 = adjacent line/space step and 1.0 = distance between staff lines.
    match track {
        "HH" => -0.5,
        "RC" => 0.0,
        "RC2" | "T1" => 0.5,
        "C" => -1.0,
        "C2" => -1.5,
        "SPL" => -2.5,
        "CHN" => -2.0,
        "SD" => 1.5,
        "T2" => 1.0,
        "T3" => 2.5,
        "T4" | "CL" => 3.0,
        "BD" => 3.5,
        "BD2" => 4.0,
        "HF" => 4.5,
        "CB" => 2.0,
        "WB" => 6.5,
        "ST" => -3.0,
        _ => 1.5,
    }
}

/// Staff height in staff-space units (always 8 for a standard 5-line staff).
pub const STAFF_HEIGHT_SS: f32 = 8.0;
/// Staff top Y in staff-space (0).
pub const STAFF_TOP_SS: f32 = 0.0;
/// Staff bottom Y in staff-space (top + height).
pub const STAFF_BOTTOM_SS: f32 = STAFF_HEIGHT_SS;

// ── Staff-Space Glyph Metrics (font-agnostic) ────────────────────

/// Glyph metrics for every notehead/rest variant, in staff-space units.
pub fn glyph_metrics(codepoint: u32) -> (f32, f32, f32) {
    // All SMuFL standard noteheads are ~1.0 × 1.0 ss. Rest widths vary.
    match codepoint {
        // Rests
        0xE4E3 => (0.8, 1.0, 0.0), // whole rest
        0xE4E4 => (0.8, 2.0, 0.0), // half rest
        0xE4E5 => (0.8, 1.5, 0.0), // quarter rest
        0xE4E6 => (0.8, 1.2, 0.0), // 16th rest
        0xE4E7 => (0.8, 1.2, 0.0), // 32nd rest
        // Noteheads
        _ => (1.0, 1.0, 0.0), // all noteheads are 1.0 × 1.0 ss
    }
}

// ── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_family() {
        assert_eq!(track_family("HH"), "cymbal");
        assert_eq!(track_family("SD"), "drum");
        assert_eq!(track_family("BD"), "drum");
        assert_eq!(track_family("HF"), "pedal");
        assert_eq!(track_family("CB"), "percussion");
    }

    #[test]
    fn test_staff_y() {
        assert_eq!(staff_y_for_track("HH"), -0.5);
        assert_eq!(staff_y_for_track("SD"), 1.5);
        assert_eq!(staff_y_for_track("BD"), 3.5);
        assert_eq!(staff_y_for_track("T1"), 0.5);
        assert_eq!(staff_y_for_track("C"), -1.0);
    }

    #[test]
    fn test_notehead_glyph() {
        let g = notehead_glyph("HH", &[], "x");
        assert_eq!(g.smufl_codepoint, 0xE0A9); // cymbal → X notehead
        let g = notehead_glyph("HF", &[], "d");
        assert_eq!(g.smufl_codepoint, 0xE0A9); // hi-hat pedal → X notehead
        let g = notehead_glyph("SD", &[], "d");
        assert_eq!(g.smufl_codepoint, 0xE0A4); // drum → standard notehead
        let g = notehead_glyph("SD", &["cross".to_string()], "d");
        assert_eq!(g.smufl_codepoint, 0xE0A9); // cross mod → X notehead
    }

    #[test]
    fn test_ledger_line_offsets_cover_top_and_bottom_positions() {
        assert_eq!(
            ledger_line_offsets_for_staff_position(-0.5),
            Vec::<f32>::new()
        );
        assert_eq!(ledger_line_offsets_for_staff_position(-1.0), vec![-1.0]);
        assert_eq!(ledger_line_offsets_for_staff_position(-1.5), vec![-1.0]);
        assert_eq!(
            ledger_line_offsets_for_staff_position(-2.0),
            vec![-1.0, -2.0]
        );
        assert_eq!(
            ledger_line_offsets_for_staff_position(4.5),
            Vec::<f32>::new()
        );
        assert_eq!(ledger_line_offsets_for_staff_position(5.0), vec![5.0]);
        assert_eq!(ledger_line_offsets_for_staff_position(6.5), vec![5.0, 6.0]);
    }

    #[test]
    fn test_rest_glyph_by_fraction() {
        assert_eq!(
            rest_glyph_for_fraction(Fraction {
                numerator: 1,
                denominator: 8
            })
            .smufl_codepoint,
            0xE4E6
        );
        assert_eq!(
            rest_glyph_for_fraction(Fraction {
                numerator: 1,
                denominator: 16
            })
            .smufl_codepoint,
            0xE4E7
        );
        assert_eq!(
            rest_glyph_for_fraction(Fraction {
                numerator: 1,
                denominator: 32
            })
            .smufl_codepoint,
            0xE4E8
        );
    }

    #[test]
    fn test_canonical_metrics_are_stable() {
        let glyph_once = canonical_glyph_metric(GlyphRole::NoteheadX);
        let glyph_twice = canonical_glyph_metric(GlyphRole::NoteheadX);
        assert_eq!(glyph_once, glyph_twice);

        let text_once = canonical_text_metric(TextRole::Tempo);
        let text_twice = canonical_text_metric(TextRole::Tempo);
        assert_eq!(text_once, text_twice);

        let clef_glyph_once = canonical_glyph_metric(GlyphRole::PercussionClef);
        let clef_glyph_twice = canonical_glyph_metric(GlyphRole::PercussionClef);
        assert_eq!(clef_glyph_once, clef_glyph_twice);
        assert_eq!(clef_glyph_once.smufl_codepoint, 0xE069);

        let time_sig_glyph_once = canonical_glyph_metric(GlyphRole::TimeSignatureDigit);
        let time_sig_glyph_twice = canonical_glyph_metric(GlyphRole::TimeSignatureDigit);
        assert_eq!(time_sig_glyph_once, time_sig_glyph_twice);
        assert_eq!(time_sig_glyph_once.smufl_codepoint, 0xE080);

        let clef_text_once = canonical_text_metric(TextRole::PercussionClef);
        let clef_text_twice = canonical_text_metric(TextRole::PercussionClef);
        assert_eq!(clef_text_once, clef_text_twice);
        assert_eq!(clef_text_once.font_size_pt, 30.0);

        let time_sig_text_once = canonical_text_metric(TextRole::TimeSignatureDigit);
        let time_sig_text_twice = canonical_text_metric(TextRole::TimeSignatureDigit);
        assert_eq!(time_sig_text_once, time_sig_text_twice);
        assert_eq!(time_sig_text_once.font_size_pt, 30.0);
    }

    #[test]
    fn test_canonical_flag_glyphs_exist() {
        assert_eq!(
            canonical_glyph_metric(GlyphRole::Flag8thUp).smufl_codepoint,
            0xE240
        );
        assert_eq!(
            canonical_glyph_metric(GlyphRole::Flag8thDown).smufl_codepoint,
            0xE241
        );
        assert_eq!(
            canonical_glyph_metric(GlyphRole::Flag16thUp).smufl_codepoint,
            0xE242
        );
        assert_eq!(
            canonical_glyph_metric(GlyphRole::Flag16thDown).smufl_codepoint,
            0xE243
        );
        assert_eq!(
            canonical_glyph_metric(GlyphRole::Flag32ndUp).smufl_codepoint,
            0xE244
        );
        assert_eq!(
            canonical_glyph_metric(GlyphRole::Flag32ndDown).smufl_codepoint,
            0xE245
        );
    }

    #[test]
    fn test_canonical_glyph_metrics_preserve_metadata_anchors() {
        let notehead = canonical_glyph_metric(GlyphRole::NoteheadBlack);
        assert_eq!(notehead.bbox_sw_x_ss, 0.0);
        assert_eq!(notehead.bbox_ne_x_ss, 1.18);
        assert_eq!(
            notehead.stem_up_anchor_ss,
            Some(GlyphPoint {
                x_ss: 1.49,
                y_ss: 0.16
            })
        );
        assert_eq!(
            notehead.stem_down_anchor_ss,
            Some(GlyphPoint {
                x_ss: 0.1,
                y_ss: -0.16
            })
        );

        let rest = canonical_glyph_metric(GlyphRole::RestQuarter);
        assert_eq!(rest.stem_up_anchor_ss, None);
        assert_eq!(rest.stem_down_anchor_ss, None);

        let flag = canonical_glyph_metric(GlyphRole::Flag8thDown);
        assert_eq!(
            flag.stem_down_anchor_ss,
            Some(GlyphPoint {
                x_ss: 0.0,
                y_ss: 0.132
            })
        );
    }

    #[test]
    fn test_default_options() {
        let opts = LayoutOptions::default();
        assert_eq!(opts.page_width_pt, 612.0);
        assert_eq!(opts.px_per_quarter, 80.0);
        assert_eq!(opts.header_height_pt, 50.0);
        assert_eq!(opts.header_staff_spacing_pt, 60.0);
        assert_eq!(opts.volta_offset_y, 0.0);
    }

    #[test]
    fn test_staff_space() {
        let ss = StaffSpace::default();
        assert_eq!(ss.pt_per_ss, 8.0);
        assert_eq!(ss.to_pixels(40.0), 10.0); // 40pt staff / 4 = 10px per ss
    }

    fn cross_system_fixture_score() -> RenderScore {
        RenderScore {
            version: RENDER_SCORE_VERSION.to_string(),
            header: RenderHeader {
                tempo: 120,
                time_beats: 4,
                time_beat_unit: 4,
                divisions: 16,
                note_value: 8,
                grouping: vec![2, 2],
                title: Some("Fixture".into()),
                subtitle: Some("Scene".into()),
                composer: Some("Codex".into()),
            },
            tracks: vec![
                RenderTrack {
                    id: "HH".into(),
                    family: "cymbal".into(),
                },
                RenderTrack {
                    id: "SD".into(),
                    family: "drum".into(),
                },
            ],
            measures: vec![
                RenderMeasure {
                    index: 0,
                    global_index: 0,
                    paragraph_index: 0,
                    measure_in_paragraph: 0,
                    source_line: 1,
                    events: vec![RenderEvent {
                        track: "HH".into(),
                        track_family: "cymbal".into(),
                        start: Fraction {
                            numerator: 0,
                            denominator: 1,
                        },
                        duration: Fraction {
                            numerator: 1,
                            denominator: 32,
                        },
                        kind: EventKind::Hit,
                        glyph: "x".into(),
                        modifiers: vec![],
                        modifier: None,
                        voice: 1,
                        beam: "none".into(),
                        tuplet: None,
                    }],
                    barline: Some("regular".into()),
                    closing_barline: Some("regular".into()),
                    start_nav: Some(NavMarker::Segno),
                    end_nav: None,
                    volta_indices: Some(vec![1]),
                    hairpins: vec![HairpinSpan {
                        kind: HairpinKind::Crescendo,
                        start: Fraction {
                            numerator: 0,
                            denominator: 1,
                        },
                        end: Fraction {
                            numerator: 3,
                            denominator: 4,
                        },
                        start_measure_index: 0,
                        end_measure_index: 3,
                    }],
                    measure_repeat_slashes: None,
                    multi_rest_count: None,
                    note_value: 8,
                    volta_terminator: false,
                },
                RenderMeasure {
                    index: 1,
                    global_index: 1,
                    paragraph_index: 1,
                    measure_in_paragraph: 0,
                    source_line: 2,
                    events: vec![
                        RenderEvent {
                            track: "HH".into(),
                            track_family: "cymbal".into(),
                            start: Fraction {
                                numerator: 0,
                                denominator: 1,
                            },
                            duration: Fraction {
                                numerator: 1,
                                denominator: 16,
                            },
                            kind: EventKind::Hit,
                            glyph: "x".into(),
                            modifiers: vec![],
                            modifier: None,
                            voice: 1,
                            beam: "begin".into(),
                            tuplet: None,
                        },
                        RenderEvent {
                            track: "SD".into(),
                            track_family: "drum".into(),
                            start: Fraction {
                                numerator: 1,
                                denominator: 16,
                            },
                            duration: Fraction {
                                numerator: 1,
                                denominator: 16,
                            },
                            kind: EventKind::Hit,
                            glyph: "d".into(),
                            modifiers: vec![],
                            modifier: None,
                            voice: 1,
                            beam: "end".into(),
                            tuplet: None,
                        },
                    ],
                    barline: Some("regular".into()),
                    closing_barline: Some("regular".into()),
                    start_nav: None,
                    end_nav: None,
                    volta_indices: Some(vec![1]),
                    hairpins: vec![],
                    measure_repeat_slashes: None,
                    multi_rest_count: None,
                    note_value: 8,
                    volta_terminator: false,
                },
                RenderMeasure {
                    index: 2,
                    global_index: 2,
                    paragraph_index: 2,
                    measure_in_paragraph: 0,
                    source_line: 3,
                    events: vec![RenderEvent {
                        track: "HH".into(),
                        track_family: "cymbal".into(),
                        start: Fraction {
                            numerator: 0,
                            denominator: 1,
                        },
                        duration: Fraction {
                            numerator: 1,
                            denominator: 4,
                        },
                        kind: EventKind::Hit,
                        glyph: "x".into(),
                        modifiers: vec![],
                        modifier: None,
                        voice: 1,
                        beam: "none".into(),
                        tuplet: None,
                    }],
                    barline: Some("regular".into()),
                    closing_barline: Some("regular".into()),
                    start_nav: None,
                    end_nav: None,
                    volta_indices: Some(vec![1]),
                    hairpins: vec![],
                    measure_repeat_slashes: None,
                    multi_rest_count: None,
                    note_value: 8,
                    volta_terminator: false,
                },
                RenderMeasure {
                    index: 3,
                    global_index: 3,
                    paragraph_index: 3,
                    measure_in_paragraph: 0,
                    source_line: 4,
                    events: vec![RenderEvent {
                        track: "SD".into(),
                        track_family: "drum".into(),
                        start: Fraction {
                            numerator: 0,
                            denominator: 1,
                        },
                        duration: Fraction {
                            numerator: 1,
                            denominator: 4,
                        },
                        kind: EventKind::Hit,
                        glyph: "d".into(),
                        modifiers: vec!["accent".into()],
                        modifier: Some("accent".into()),
                        voice: 1,
                        beam: "none".into(),
                        tuplet: None,
                    }],
                    barline: Some("regular".into()),
                    closing_barline: Some("final".into()),
                    start_nav: None,
                    end_nav: Some(NavJump::DSalCoda),
                    volta_indices: Some(vec![1]),
                    hairpins: vec![],
                    measure_repeat_slashes: None,
                    multi_rest_count: None,
                    note_value: 8,
                    volta_terminator: false,
                },
            ],
            errors: vec![],
            repeat_spans: vec![RepeatSpan {
                start_measure: 0,
                end_measure: 3,
                times: 2,
            }],
        }
    }

    fn regular_measure(index: u32, paragraph_index: u32, event_count: u32) -> RenderMeasure {
        let events = (0..event_count)
            .map(|event_index| RenderEvent {
                track: "HH".into(),
                track_family: "cymbal".into(),
                start: Fraction {
                    numerator: event_index,
                    denominator: event_count.max(1),
                },
                duration: Fraction {
                    numerator: 1,
                    denominator: event_count.max(1) * 2,
                },
                kind: EventKind::Hit,
                glyph: "x".into(),
                modifiers: vec![],
                modifier: None,
                voice: 1,
                beam: "none".into(),
                tuplet: None,
            })
            .collect::<Vec<_>>();

        RenderMeasure {
            index,
            global_index: index,
            paragraph_index,
            measure_in_paragraph: index,
            source_line: index + 1,
            events,
            barline: Some("regular".into()),
            closing_barline: Some("regular".into()),
            start_nav: None,
            end_nav: None,
            volta_indices: None,
            hairpins: vec![],
            measure_repeat_slashes: None,
            multi_rest_count: None,
            note_value: 8,
            volta_terminator: false,
        }
    }

    fn simple_layout_score(measures: Vec<RenderMeasure>) -> RenderScore {
        RenderScore {
            version: RENDER_SCORE_VERSION.to_string(),
            header: RenderHeader {
                tempo: 120,
                time_beats: 4,
                time_beat_unit: 4,
                divisions: 16,
                note_value: 8,
                grouping: vec![2, 2],
                title: None,
                subtitle: None,
                composer: None,
            },
            tracks: vec![RenderTrack {
                id: "HH".into(),
                family: "cymbal".into(),
            }],
            measures,
            errors: vec![],
            repeat_spans: vec![],
        }
    }

    fn line_for_role<'a>(page: &'a ScenePage, role: &str) -> &'a LineSegment {
        let item = page
            .items
            .iter()
            .find(|item| item.role == role)
            .unwrap_or_else(|| panic!("expected {role} line item"));
        let ScenePrimitive::LineSegment(line) = &item.primitive else {
            panic!("expected {role} to be a line segment");
        };
        line
    }

    fn line_for_id<'a>(page: &'a ScenePage, id: &str) -> &'a LineSegment {
        let item = page
            .items
            .iter()
            .find(|item| item.id == id)
            .unwrap_or_else(|| panic!("expected line item {id}"));
        let ScenePrimitive::LineSegment(line) = &item.primitive else {
            panic!("expected {id} to be a line segment");
        };
        line
    }

    fn hairpin_center_y(page: &ScenePage) -> f32 {
        let top = line_for_role(page, "hairpin-top");
        let bottom = line_for_role(page, "hairpin-bottom");
        (top.y1_pt + top.y2_pt + bottom.y1_pt + bottom.y2_pt) / 4.0
    }

    #[test]
    fn test_scene_fixture_supports_span_fragments_across_system_breaks() {
        let scene = build_layout_scene(&cross_system_fixture_score(), &LayoutOptions::default());
        let volta_fragments = scene
            .pages
            .iter()
            .flat_map(|page| page.composites.iter())
            .filter(|composite| composite.kind == CompositeKind::Volta)
            .map(|composite| composite.fragment)
            .collect::<Vec<_>>();
        let hairpin_fragments = scene
            .pages
            .iter()
            .flat_map(|page| page.composites.iter())
            .filter(|composite| composite.kind == CompositeKind::Hairpin)
            .map(|composite| composite.fragment)
            .collect::<Vec<_>>();

        assert_eq!(
            volta_fragments,
            vec![
                SpanFragmentKind::Start,
                SpanFragmentKind::Continuation,
                SpanFragmentKind::Continuation,
                SpanFragmentKind::End
            ]
        );
        assert_eq!(
            hairpin_fragments,
            vec![
                SpanFragmentKind::Start,
                SpanFragmentKind::Continuation,
                SpanFragmentKind::Continuation,
                SpanFragmentKind::End
            ]
        );
    }

    #[test]
    fn test_single_system_hairpin_is_conical() {
        let mut measure = regular_measure(0, 0, 4);
        measure.hairpins = vec![HairpinSpan {
            kind: HairpinKind::Crescendo,
            start: Fraction {
                numerator: 0,
                denominator: 1,
            },
            end: Fraction {
                numerator: 1,
                denominator: 1,
            },
            start_measure_index: 0,
            end_measure_index: 0,
        }];
        let scene = build_layout_scene(
            &simple_layout_score(vec![measure]),
            &LayoutOptions::default(),
        );
        let page = &scene.pages[0];
        let top = line_for_role(page, "hairpin-top");
        let bottom = line_for_role(page, "hairpin-bottom");

        assert!((bottom.y1_pt - top.y1_pt).abs() < 0.01);
        assert!(bottom.y2_pt - top.y2_pt > 8.0);
    }

    #[test]
    fn test_hairpin_vertical_offset_moves_down_when_positive() {
        let mut measure = regular_measure(0, 0, 4);
        measure.hairpins = vec![HairpinSpan {
            kind: HairpinKind::Crescendo,
            start: Fraction {
                numerator: 0,
                denominator: 1,
            },
            end: Fraction {
                numerator: 1,
                denominator: 1,
            },
            start_measure_index: 0,
            end_measure_index: 0,
        }];
        let score = simple_layout_score(vec![measure]);

        let baseline = build_layout_scene(&score, &LayoutOptions::default());
        let below = build_layout_scene(
            &score,
            &LayoutOptions {
                hairpin_offset_y: 10.0,
                ..LayoutOptions::default()
            },
        );
        let above = build_layout_scene(
            &score,
            &LayoutOptions {
                hairpin_offset_y: -5.0,
                ..LayoutOptions::default()
            },
        );

        let baseline_y = hairpin_center_y(&baseline.pages[0]);
        assert!((hairpin_center_y(&below.pages[0]) - baseline_y - 10.0).abs() < 0.01);
        assert!((hairpin_center_y(&above.pages[0]) - baseline_y + 5.0).abs() < 0.01);
    }

    #[test]
    fn test_cross_system_hairpin_continuation_keeps_partial_opening() {
        let scene = build_layout_scene(&cross_system_fixture_score(), &LayoutOptions::default());
        let page = &scene.pages[0];
        let continuation = page
            .composites
            .iter()
            .find(|composite| {
                composite.kind == CompositeKind::Hairpin
                    && composite.fragment == SpanFragmentKind::Continuation
            })
            .expect("expected continuation hairpin fragment");
        let top = line_for_id(page, &continuation.child_item_ids[0]);
        let bottom = line_for_id(page, &continuation.child_item_ids[1]);

        assert!(bottom.y1_pt - top.y1_pt > 0.5);
        assert!(bottom.y2_pt - top.y2_pt > bottom.y1_pt - top.y1_pt);
    }

    #[test]
    fn test_volta_segment_type_does_not_end_on_repeat_end_when_next_measure_matches() {
        let mut source_measures = [
            regular_measure(0, 0, 1),
            regular_measure(1, 0, 1),
            regular_measure(2, 0, 1),
        ];
        source_measures[0].volta_indices = Some(vec![2]);
        source_measures[1].volta_indices = Some(vec![2]);
        source_measures[1].barline = Some("repeat-end".into());
        source_measures[2].volta_indices = Some(vec![2]);

        let display_measures = source_measures
            .iter()
            .map(|measure| DisplayMeasure {
                measure,
                global_index: measure.global_index,
                paragraph_index: measure.paragraph_index,
                barline: measure.barline.clone(),
                closing_barline: measure.closing_barline.clone(),
                start_nav: measure.start_nav.clone(),
                end_nav: measure.end_nav.clone(),
                hairpins: measure.hairpins.clone(),
                repeat_part: None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            volta_type_for_measure(&display_measures, 1),
            VoltaSegmentType::Mid
        );
        assert_eq!(
            volta_type_for_measure(&display_measures, 2),
            VoltaSegmentType::End
        );
    }

    #[test]
    fn test_structural_span_fragments_emit_child_items_and_navigation() {
        let scene = build_layout_scene(&cross_system_fixture_score(), &LayoutOptions::default());
        let items = scene
            .pages
            .iter()
            .flat_map(|page| page.items.iter())
            .collect::<Vec<_>>();
        let composites = scene
            .pages
            .iter()
            .flat_map(|page| page.composites.iter())
            .collect::<Vec<_>>();

        assert!(composites
            .iter()
            .all(|composite| composite.kind != CompositeKind::RepeatSpan));
        assert!(items
            .iter()
            .all(|item| !item.role.starts_with("repeat-span")));

        let volta_fragments = composites
            .iter()
            .copied()
            .filter(|composite| composite.kind == CompositeKind::Volta)
            .collect::<Vec<_>>();
        assert!(!volta_fragments.is_empty());
        assert!(volta_fragments
            .iter()
            .all(|fragment| !fragment.child_item_ids.is_empty()));
        assert_eq!(
            items
                .iter()
                .filter(|item| item.role == "volta-start-hook")
                .count(),
            4
        );
        assert_eq!(
            items
                .iter()
                .filter(|item| item.role == "volta-label")
                .count(),
            1
        );

        let navigation = composites
            .iter()
            .copied()
            .filter(|composite| composite.kind == CompositeKind::Navigation)
            .collect::<Vec<_>>();
        assert_eq!(navigation.len(), 2);
        assert_eq!(navigation[0].label.as_deref(), Some("segno"));
        assert_eq!(navigation[1].label.as_deref(), Some("D.S. al Coda"));
        assert!(navigation
            .iter()
            .all(|composite| !composite.child_item_ids.is_empty()));
        assert!(items.iter().any(|item| {
            item.role == "nav-start"
                && matches!(
                    &item.primitive,
                    ScenePrimitive::GlyphRun(GlyphRun {
                        glyph_role: GlyphRole::NavigationSegno,
                        ..
                    })
                )
        }));
        assert!(items.iter().any(|item| {
            item.role == "nav-end"
                && matches!(
                    &item.primitive,
                    ScenePrimitive::TextRun(TextRun { text, .. }) if text == "D.S. al Coda"
                )
        }));
    }

    #[test]
    fn test_canonical_text_metrics_drive_structural_and_attachment_text() {
        let scene = build_layout_scene(&cross_system_fixture_score(), &LayoutOptions::default());
        let items = scene
            .pages
            .iter()
            .flat_map(|page| page.items.iter())
            .collect::<Vec<_>>();
        let count_metric = canonical_text_metric(TextRole::CountLabel);

        {
            let nav_start = items
                .iter()
                .copied()
                .find(|item| item.role == "nav-start")
                .expect("expected scene item with role nav-start");
            let ScenePrimitive::GlyphRun(glyph) = &nav_start.primitive else {
                panic!("expected glyph primitive for nav-start");
            };
            assert_eq!(glyph.glyph_role, GlyphRole::NavigationSegno);
            assert_eq!(glyph.font_family, "Bravura");
            assert_eq!(glyph.font_size_pt, 20.0);
        }
        {
            let nav_end = items
                .iter()
                .copied()
                .find(|item| item.role == "nav-end")
                .expect("expected scene item with role nav-end");
            let ScenePrimitive::TextRun(text) = &nav_end.primitive else {
                panic!("expected text primitive for nav-end");
            };
            assert_eq!(text.text_role, TextRole::CountLabel);
            assert_eq!(text.font_family, "Academico");
            assert_eq!(text.font_size_pt, count_metric.font_size_pt);
        }

        let volta_label = items
            .iter()
            .copied()
            .find(|item| item.role == "volta-label")
            .expect("expected volta label item");
        let ScenePrimitive::TextRun(volta_text) = &volta_label.primitive else {
            panic!("expected text primitive for volta label");
        };
        assert_eq!(volta_text.text_role, TextRole::CountLabel);
        assert_eq!(volta_text.font_family, "Academico");
        assert_eq!(volta_text.font_size_pt, VOLTA_TEXT_SIZE_PT);

        let accent_item = items
            .iter()
            .copied()
            .find(|item| item.role == "accent")
            .expect("expected accent scene item");
        let ScenePrimitive::GlyphRun(accent_glyph) = &accent_item.primitive else {
            panic!("expected glyph primitive for accent");
        };
        assert_eq!(accent_glyph.glyph_role, GlyphRole::ArticAccentAbove);
        assert_eq!(accent_glyph.font_family, "Bravura");
        assert_eq!(accent_glyph.font_size_pt, BASE_FONT_SIZE_PT);

        let sticking_score = RenderScore {
            version: RENDER_SCORE_VERSION.to_string(),
            header: RenderHeader {
                tempo: 0,
                time_beats: 4,
                time_beat_unit: 4,
                divisions: 16,
                note_value: 8,
                grouping: vec![1, 1, 1, 1],
                title: None,
                subtitle: None,
                composer: None,
            },
            tracks: vec![RenderTrack {
                id: "HH".into(),
                family: "cymbal".into(),
            }],
            measures: vec![RenderMeasure {
                index: 0,
                global_index: 0,
                paragraph_index: 0,
                measure_in_paragraph: 0,
                source_line: 1,
                events: vec![
                    RenderEvent {
                        track: "HH".into(),
                        track_family: "cymbal".into(),
                        start: Fraction {
                            numerator: 0,
                            denominator: 1,
                        },
                        duration: Fraction {
                            numerator: 1,
                            denominator: 8,
                        },
                        kind: EventKind::Hit,
                        glyph: "x".into(),
                        modifiers: vec![],
                        modifier: None,
                        voice: 1,
                        beam: "none".into(),
                        tuplet: None,
                    },
                    RenderEvent {
                        track: "HH".into(),
                        track_family: "cymbal".into(),
                        start: Fraction {
                            numerator: 0,
                            denominator: 1,
                        },
                        duration: Fraction {
                            numerator: 1,
                            denominator: 8,
                        },
                        kind: EventKind::Sticking,
                        glyph: "R".into(),
                        modifiers: vec![],
                        modifier: None,
                        voice: 1,
                        beam: "none".into(),
                        tuplet: None,
                    },
                ],
                barline: Some("regular".into()),
                closing_barline: Some("regular".into()),
                start_nav: None,
                end_nav: None,
                volta_indices: None,
                hairpins: vec![],
                measure_repeat_slashes: None,
                multi_rest_count: None,
                note_value: 8,
                volta_terminator: false,
            }],
            errors: vec![],
            repeat_spans: vec![],
        };
        let sticking_scene = build_layout_scene(&sticking_score, &LayoutOptions::default());
        let sticking_item = sticking_scene.pages[0]
            .items
            .iter()
            .find(|item| item.role == "sticking")
            .expect("expected sticking scene item");
        let ScenePrimitive::TextRun(sticking_text) = &sticking_item.primitive else {
            panic!("expected text primitive for sticking");
        };
        let sticking_metric = canonical_text_metric(TextRole::Sticking);
        assert_eq!(sticking_text.text_role, TextRole::Sticking);
        assert_eq!(sticking_text.font_family, sticking_metric.font_family);
        assert_eq!(sticking_text.font_size_pt, sticking_metric.font_size_pt);
    }

    #[test]
    fn test_layout_owned_structural_stacking_avoids_overlap() {
        let scene = build_layout_scene(&cross_system_fixture_score(), &LayoutOptions::default());
        let page = &scene.pages[0];

        let measure_number = page
            .items
            .iter()
            .find(|item| item.role == "measure-number")
            .expect("expected measure number item");
        let nav_start = page
            .items
            .iter()
            .find(|item| item.role == "nav-start")
            .expect("expected navigation start item");
        let volta_label = page
            .items
            .iter()
            .find(|item| item.role == "volta-label")
            .expect("expected volta label item");
        let hairpin_top = page
            .items
            .iter()
            .find(|item| item.role == "hairpin-top")
            .expect("expected hairpin item");
        let notehead = page
            .items
            .iter()
            .find(|item| item.role == "notehead" && item.measure_id.as_deref() == Some("measure-0"))
            .expect("expected notehead item");

        let (_, measure_number_y, _, _) = item_bounds(measure_number).unwrap();
        let (_, nav_y, _, nav_h) = item_bounds(nav_start).unwrap();
        assert!(item_bounds(volta_label).is_some());
        let (_, hairpin_y, _, _) = item_bounds(hairpin_top).unwrap();
        let (_, notehead_y, _, notehead_h) = item_bounds(notehead).unwrap();

        assert!(nav_y + nav_h <= measure_number_y - 4.0);
        assert!(hairpin_y >= notehead_y + notehead_h + 4.0);
    }

    #[test]
    fn test_navigation_uses_anchor_aware_bounds_and_clears_notes() {
        let mut items = Vec::new();
        let mut counter = 0usize;
        let mut sink = SceneEmitSink::new(&mut items, &mut counter);
        let note_id = sink.push_text_item(TextItemSpec {
            measure_id: Some("measure-0"),
            role: "notehead",
            x: 500.0,
            y: 210.0,
            text_role: TextRole::Tempo,
            text: "\u{E0A4}".to_string(),
            font_family: "Bravura",
            font_size_pt: 30.0,
            fill: "#333",
            text_anchor: None,
            font_weight: None,
        });
        let mut composites = Vec::new();
        render_nav_markers(
            &mut sink,
            &mut composites,
            &DeferredNavMarker {
                measure_id: "measure-0".to_string(),
                global_index: 0,
                start_nav: None,
                end_nav: Some(NavJump::DSalCoda),
                x: 50.0,
                width: 520.0,
                top: 220.0,
            },
        );
        drop(sink);

        let nav_end = items
            .iter()
            .find(|item| item.role == "nav-end")
            .expect("expected end navigation item");
        let notehead = items
            .iter()
            .find(|item| item.id == note_id)
            .expect("expected colliding notehead candidate");

        let (nav_x, nav_y, nav_w, nav_h) = item_bounds(nav_end).unwrap();
        let (note_x, note_y, note_w, _) = item_bounds(notehead).unwrap();
        assert!(
            nav_x < note_x + note_w && nav_x + nav_w > note_x,
            "fixture should exercise horizontal nav/note overlap: nav=({nav_x:.1},{nav_y:.1},{nav_w:.1},{nav_h:.1}) note=({note_x:.1},{note_y:.1},{note_w:.1})"
        );
        assert!(
            nav_y + nav_h <= note_y - 4.0,
            "end navigation should float above the overlapping notehead"
        );
    }

    #[test]
    fn test_cross_system_scene_snapshot_matches_golden() {
        let scene = build_layout_scene(&cross_system_fixture_score(), &LayoutOptions::default());
        let actual = layout_scene_snapshot(&scene);
        let expected = include_str!("../tests/goldens/cross_system_scene_snapshot.txt");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_same_paragraph_stays_on_one_system_even_when_page_is_narrow() {
        let score = simple_layout_score(vec![
            regular_measure(0, 0, 1),
            regular_measure(1, 0, 1),
            regular_measure(2, 0, 1),
        ]);
        let opts = LayoutOptions {
            page_width_pt: 260.0,
            ..LayoutOptions::default()
        };

        let scene = build_layout_scene(&score, &opts);
        assert_eq!(scene.pages[0].systems.len(), 1);
        assert_eq!(
            scene.pages[0].systems[0].measure_ids,
            vec!["measure-0", "measure-1", "measure-2"]
        );
    }

    #[test]
    fn test_each_paragraph_becomes_its_own_system() {
        let score = simple_layout_score(vec![regular_measure(0, 0, 1), regular_measure(1, 1, 1)]);
        let opts = LayoutOptions {
            page_width_pt: 240.0,
            left_margin_pt: 20.0,
            right_margin_pt: 20.0,
            px_per_quarter: 10.0,
            ..LayoutOptions::default()
        };

        let scene = build_layout_scene(&score, &opts);
        assert_eq!(
            scene.pages[0].systems.len(),
            2,
            "each paragraph must map to its own system"
        );
        assert_eq!(scene.pages[0].systems[0].measure_ids, vec!["measure-0"]);
        assert_eq!(scene.pages[0].systems[1].measure_ids, vec!["measure-1"]);
    }

    #[test]
    fn test_compact_structural_measure_is_narrower_than_regular_measure() {
        let mut compact = regular_measure(1, 0, 1);
        compact.events.clear();
        compact.multi_rest_count = Some(4);
        let score = simple_layout_score(vec![regular_measure(0, 0, 4), compact]);

        let scene = build_layout_scene(&score, &LayoutOptions::default());
        let regular_width = scene.pages[0]
            .measures
            .iter()
            .find(|measure| measure.id == "measure-0")
            .unwrap()
            .width_pt;
        let compact_width = scene.pages[0]
            .measures
            .iter()
            .find(|measure| measure.id == "measure-1")
            .unwrap()
            .width_pt;

        assert!(compact_width < regular_width);
    }

    fn notehead_positions(scene: &LayoutScene, measure_id: &str) -> Vec<f32> {
        let mut positions = scene.pages[0]
            .items
            .iter()
            .filter(|item| {
                item.role == "notehead" && item.measure_id.as_deref() == Some(measure_id)
            })
            .filter_map(|item| match &item.primitive {
                ScenePrimitive::TextRun(text) => Some(text.x_pt),
                _ => None,
            })
            .collect::<Vec<_>>();
        positions.sort_by(|a, b| a.partial_cmp(b).unwrap());
        positions
    }

    fn items_by_role<'a>(scene: &'a LayoutScene, role: &str) -> Vec<&'a SceneItem> {
        scene.pages[0]
            .items
            .iter()
            .filter(|item| item.role == role)
            .collect()
    }

    fn text_y_by_role(scene: &LayoutScene, role: &str) -> f32 {
        let item = items_by_role(scene, role)
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("expected {role} text item"));
        let ScenePrimitive::TextRun(text) = &item.primitive else {
            panic!("expected {role} to be text");
        };
        text.y_pt
    }

    fn test_hit(track: &str, start: Fraction, duration: Fraction, voice: u8) -> RenderEvent {
        RenderEvent {
            track: track.into(),
            track_family: track_family(track).into(),
            start,
            duration,
            kind: EventKind::Hit,
            glyph: if track_family(track) == "cymbal" {
                "x".into()
            } else {
                "d".into()
            },
            modifiers: vec![],
            modifier: None,
            voice,
            beam: "none".into(),
            tuplet: None,
        }
    }

    fn test_rest(start: Fraction, duration: Fraction, voice: u8) -> RenderEvent {
        RenderEvent {
            track: "HH".into(),
            track_family: "cymbal".into(),
            start,
            duration,
            kind: EventKind::Rest,
            glyph: "r".into(),
            modifiers: vec![],
            modifier: None,
            voice,
            beam: "none".into(),
            tuplet: None,
        }
    }

    #[test]
    fn test_simple_four_four_spacing_is_even() {
        let measure = RenderMeasure {
            index: 0,
            global_index: 0,
            paragraph_index: 0,
            measure_in_paragraph: 0,
            source_line: 1,
            events: vec![
                RenderEvent {
                    track: "HH".into(),
                    track_family: "cymbal".into(),
                    start: Fraction {
                        numerator: 0,
                        denominator: 1,
                    },
                    duration: Fraction {
                        numerator: 1,
                        denominator: 4,
                    },
                    kind: EventKind::Hit,
                    glyph: "x".into(),
                    modifiers: vec![],
                    modifier: None,
                    voice: 1,
                    beam: "none".into(),
                    tuplet: None,
                },
                RenderEvent {
                    track: "HH".into(),
                    track_family: "cymbal".into(),
                    start: Fraction {
                        numerator: 1,
                        denominator: 4,
                    },
                    duration: Fraction {
                        numerator: 1,
                        denominator: 4,
                    },
                    kind: EventKind::Hit,
                    glyph: "x".into(),
                    modifiers: vec![],
                    modifier: None,
                    voice: 1,
                    beam: "none".into(),
                    tuplet: None,
                },
                RenderEvent {
                    track: "HH".into(),
                    track_family: "cymbal".into(),
                    start: Fraction {
                        numerator: 1,
                        denominator: 2,
                    },
                    duration: Fraction {
                        numerator: 1,
                        denominator: 4,
                    },
                    kind: EventKind::Hit,
                    glyph: "x".into(),
                    modifiers: vec![],
                    modifier: None,
                    voice: 1,
                    beam: "none".into(),
                    tuplet: None,
                },
                RenderEvent {
                    track: "HH".into(),
                    track_family: "cymbal".into(),
                    start: Fraction {
                        numerator: 3,
                        denominator: 4,
                    },
                    duration: Fraction {
                        numerator: 1,
                        denominator: 4,
                    },
                    kind: EventKind::Hit,
                    glyph: "x".into(),
                    modifiers: vec![],
                    modifier: None,
                    voice: 1,
                    beam: "none".into(),
                    tuplet: None,
                },
            ],
            barline: Some("regular".into()),
            closing_barline: Some("final".into()),
            start_nav: None,
            end_nav: None,
            volta_indices: None,
            hairpins: vec![],
            measure_repeat_slashes: None,
            multi_rest_count: None,
            note_value: 8,
            volta_terminator: false,
        };
        let score = simple_layout_score(vec![measure]);
        let scene = build_layout_scene(&score, &LayoutOptions::default());
        let xs = notehead_positions(&scene, "measure-0");
        let gaps = xs
            .windows(2)
            .map(|pair| pair[1] - pair[0])
            .collect::<Vec<_>>();

        assert_eq!(xs.len(), 4);
        assert!(
            (gaps[0] - gaps[1]).abs() < 0.5,
            "quarter-note gaps should match: {gaps:?}"
        );
        assert!(
            (gaps[1] - gaps[2]).abs() < 0.5,
            "quarter-note gaps should match: {gaps:?}"
        );
    }

    #[test]
    fn test_grouping_allocates_more_width_to_dense_first_half() {
        let measure = RenderMeasure {
            index: 0,
            global_index: 0,
            paragraph_index: 0,
            measure_in_paragraph: 0,
            source_line: 1,
            events: vec![
                RenderEvent {
                    track: "HH".into(),
                    track_family: "cymbal".into(),
                    start: Fraction {
                        numerator: 0,
                        denominator: 1,
                    },
                    duration: Fraction {
                        numerator: 1,
                        denominator: 4,
                    },
                    kind: EventKind::Hit,
                    glyph: "x".into(),
                    modifiers: vec![],
                    modifier: None,
                    voice: 1,
                    beam: "none".into(),
                    tuplet: None,
                },
                RenderEvent {
                    track: "SD".into(),
                    track_family: "drum".into(),
                    start: Fraction {
                        numerator: 1,
                        denominator: 8,
                    },
                    duration: Fraction {
                        numerator: 1,
                        denominator: 8,
                    },
                    kind: EventKind::Rest,
                    glyph: "r".into(),
                    modifiers: vec![],
                    modifier: None,
                    voice: 1,
                    beam: "none".into(),
                    tuplet: None,
                },
                RenderEvent {
                    track: "HH".into(),
                    track_family: "cymbal".into(),
                    start: Fraction {
                        numerator: 1,
                        denominator: 4,
                    },
                    duration: Fraction {
                        numerator: 1,
                        denominator: 4,
                    },
                    kind: EventKind::Hit,
                    glyph: "x".into(),
                    modifiers: vec![],
                    modifier: None,
                    voice: 1,
                    beam: "none".into(),
                    tuplet: None,
                },
                RenderEvent {
                    track: "SD".into(),
                    track_family: "drum".into(),
                    start: Fraction {
                        numerator: 3,
                        denominator: 8,
                    },
                    duration: Fraction {
                        numerator: 1,
                        denominator: 8,
                    },
                    kind: EventKind::Rest,
                    glyph: "r".into(),
                    modifiers: vec![],
                    modifier: None,
                    voice: 1,
                    beam: "none".into(),
                    tuplet: None,
                },
                RenderEvent {
                    track: "HH".into(),
                    track_family: "cymbal".into(),
                    start: Fraction {
                        numerator: 1,
                        denominator: 2,
                    },
                    duration: Fraction {
                        numerator: 1,
                        denominator: 4,
                    },
                    kind: EventKind::Hit,
                    glyph: "x".into(),
                    modifiers: vec![],
                    modifier: None,
                    voice: 1,
                    beam: "none".into(),
                    tuplet: None,
                },
                RenderEvent {
                    track: "HH".into(),
                    track_family: "cymbal".into(),
                    start: Fraction {
                        numerator: 3,
                        denominator: 4,
                    },
                    duration: Fraction {
                        numerator: 1,
                        denominator: 4,
                    },
                    kind: EventKind::Hit,
                    glyph: "x".into(),
                    modifiers: vec![],
                    modifier: None,
                    voice: 1,
                    beam: "none".into(),
                    tuplet: None,
                },
            ],
            barline: Some("regular".into()),
            closing_barline: Some("final".into()),
            start_nav: None,
            end_nav: None,
            volta_indices: None,
            hairpins: vec![],
            measure_repeat_slashes: None,
            multi_rest_count: None,
            note_value: 8,
            volta_terminator: false,
        };
        let mut score = simple_layout_score(vec![measure]);
        score.header.grouping = vec![2, 2];
        let scene = build_layout_scene(&score, &LayoutOptions::default());
        let measure_box = scene.pages[0]
            .measures
            .iter()
            .find(|measure| measure.id == "measure-0")
            .unwrap();
        let xs = notehead_positions(&scene, "measure-0");
        let midpoint = measure_box.x_pt + measure_box.width_pt * 0.5;
        let first_group_gap = xs[1] - xs[0];
        let second_group_gap = xs[3] - xs[2];

        assert_eq!(xs.len(), 4);
        assert!(
            xs[2] > midpoint,
            "the beat-3 note should start past the visual midpoint when the first group is denser"
        );
        assert!(
            first_group_gap > second_group_gap + 1.0,
            "dense first-half grouping should allocate wider beat spacing: {xs:?}"
        );
    }

    #[test]
    fn test_header_height_and_gap_match_ts_system_start_semantics() {
        let mut score = simple_layout_score(vec![regular_measure(0, 0, 1)]);
        score.header.title = Some("Title".into());
        score.header.subtitle = Some("Subtitle".into());
        score.header.composer = Some("Composer".into());

        let baseline = build_layout_scene(&score, &LayoutOptions::default());
        let custom_height = build_layout_scene(
            &score,
            &LayoutOptions {
                header_height_pt: 80.0,
                ..LayoutOptions::default()
            },
        );
        let custom_gap = build_layout_scene(
            &score,
            &LayoutOptions {
                header_staff_spacing_pt: 20.0,
                ..LayoutOptions::default()
            },
        );

        assert!(baseline.pages[0].systems[0].y_pt > 140.0);
        assert!(custom_height.pages[0].systems[0].y_pt > baseline.pages[0].systems[0].y_pt);
        assert!(custom_gap.pages[0].systems[0].y_pt < baseline.pages[0].systems[0].y_pt);

        assert_eq!(
            text_y_by_role(&baseline, "title"),
            text_y_by_role(&custom_height, "title")
        );
        assert_eq!(
            text_y_by_role(&custom_height, "subtitle") - text_y_by_role(&baseline, "subtitle"),
            30.0
        );
        assert_eq!(
            text_y_by_role(&custom_height, "composer") - text_y_by_role(&baseline, "composer"),
            30.0
        );
        assert_eq!(
            text_y_by_role(&custom_gap, "subtitle"),
            text_y_by_role(&baseline, "subtitle")
        );
    }

    #[test]
    fn test_beams_follow_grouping_segments() {
        let mut measure = regular_measure(0, 0, 0);
        measure.events = vec![
            test_hit(
                "HH",
                Fraction {
                    numerator: 0,
                    denominator: 1,
                },
                Fraction {
                    numerator: 1,
                    denominator: 8,
                },
                1,
            ),
            test_hit(
                "HH",
                Fraction {
                    numerator: 1,
                    denominator: 8,
                },
                Fraction {
                    numerator: 1,
                    denominator: 8,
                },
                1,
            ),
            test_hit(
                "HH",
                Fraction {
                    numerator: 1,
                    denominator: 2,
                },
                Fraction {
                    numerator: 1,
                    denominator: 8,
                },
                1,
            ),
            test_hit(
                "HH",
                Fraction {
                    numerator: 5,
                    denominator: 8,
                },
                Fraction {
                    numerator: 1,
                    denominator: 8,
                },
                1,
            ),
        ];
        let mut score = simple_layout_score(vec![measure]);
        score.header.grouping = vec![2, 2];

        let scene = build_layout_scene(&score, &LayoutOptions::default());
        assert_eq!(items_by_role(&scene, "beam").len(), 2);
        assert_eq!(items_by_role(&scene, "flag").len(), 0);
    }

    #[test]
    fn test_secondary_beams_break_around_eighth_notes() {
        let mut measure = regular_measure(0, 0, 0);
        measure.events = vec![
            test_hit(
                "SD",
                Fraction {
                    numerator: 0,
                    denominator: 1,
                },
                Fraction {
                    numerator: 1,
                    denominator: 16,
                },
                1,
            ),
            test_hit(
                "SD",
                Fraction {
                    numerator: 1,
                    denominator: 16,
                },
                Fraction {
                    numerator: 1,
                    denominator: 8,
                },
                1,
            ),
            test_hit(
                "SD",
                Fraction {
                    numerator: 3,
                    denominator: 16,
                },
                Fraction {
                    numerator: 1,
                    denominator: 16,
                },
                1,
            ),
        ];
        let mut score = simple_layout_score(vec![measure]);
        score.header.grouping = vec![4];

        let scene = build_layout_scene(&score, &LayoutOptions::default());

        assert_eq!(items_by_role(&scene, "beam").len(), 1);
        assert_eq!(items_by_role(&scene, "beam-secondary").len(), 2);
    }

    #[test]
    fn test_rests_break_grouping_beams() {
        let mut measure = regular_measure(0, 0, 0);
        measure.events = vec![
            test_hit(
                "HH",
                Fraction {
                    numerator: 0,
                    denominator: 1,
                },
                Fraction {
                    numerator: 1,
                    denominator: 8,
                },
                1,
            ),
            test_rest(
                Fraction {
                    numerator: 1,
                    denominator: 8,
                },
                Fraction {
                    numerator: 1,
                    denominator: 8,
                },
                1,
            ),
            test_hit(
                "HH",
                Fraction {
                    numerator: 1,
                    denominator: 4,
                },
                Fraction {
                    numerator: 1,
                    denominator: 8,
                },
                1,
            ),
        ];
        let mut score = simple_layout_score(vec![measure]);
        score.header.grouping = vec![4];

        let scene = build_layout_scene(&score, &LayoutOptions::default());
        assert_eq!(items_by_role(&scene, "beam").len(), 0);
        assert_eq!(items_by_role(&scene, "flag").len(), 2);
    }

    #[test]
    fn test_combined_hit_shares_a_single_stem() {
        let measure = RenderMeasure {
            index: 0,
            global_index: 0,
            paragraph_index: 0,
            measure_in_paragraph: 0,
            source_line: 1,
            events: vec![
                RenderEvent {
                    track: "HH".into(),
                    track_family: "cymbal".into(),
                    start: Fraction {
                        numerator: 0,
                        denominator: 1,
                    },
                    duration: Fraction {
                        numerator: 1,
                        denominator: 8,
                    },
                    kind: EventKind::Hit,
                    glyph: "x".into(),
                    modifiers: vec![],
                    modifier: None,
                    voice: 1,
                    beam: "begin".into(),
                    tuplet: None,
                },
                RenderEvent {
                    track: "SD".into(),
                    track_family: "drum".into(),
                    start: Fraction {
                        numerator: 0,
                        denominator: 1,
                    },
                    duration: Fraction {
                        numerator: 1,
                        denominator: 8,
                    },
                    kind: EventKind::Hit,
                    glyph: "d".into(),
                    modifiers: vec![],
                    modifier: None,
                    voice: 1,
                    beam: "begin".into(),
                    tuplet: None,
                },
            ],
            barline: Some("regular".into()),
            closing_barline: Some("final".into()),
            start_nav: None,
            end_nav: None,
            volta_indices: None,
            hairpins: vec![],
            measure_repeat_slashes: None,
            multi_rest_count: None,
            note_value: 8,
            volta_terminator: false,
        };
        let scene = build_layout_scene(
            &simple_layout_score(vec![measure]),
            &LayoutOptions::default(),
        );
        let noteheads = items_by_role(&scene, "notehead");
        let stems = items_by_role(&scene, "stem");

        assert_eq!(noteheads.len(), 2);
        assert_eq!(
            stems.len(),
            1,
            "combined hits in the same voice should share one stem"
        );
        assert!(
            stems[0].anchor_item_id.is_some(),
            "shared stem should anchor to a notehead"
        );
    }

    #[test]
    fn test_two_voice_collision_case_preserves_attachment_anchors() {
        let measure = RenderMeasure {
            index: 0,
            global_index: 0,
            paragraph_index: 0,
            measure_in_paragraph: 0,
            source_line: 1,
            events: vec![
                RenderEvent {
                    track: "HH".into(),
                    track_family: "cymbal".into(),
                    start: Fraction {
                        numerator: 0,
                        denominator: 1,
                    },
                    duration: Fraction {
                        numerator: 1,
                        denominator: 8,
                    },
                    kind: EventKind::Hit,
                    glyph: "x".into(),
                    modifiers: vec!["accent".into()],
                    modifier: Some("accent".into()),
                    voice: 1,
                    beam: "none".into(),
                    tuplet: None,
                },
                RenderEvent {
                    track: "BD".into(),
                    track_family: "drum".into(),
                    start: Fraction {
                        numerator: 0,
                        denominator: 1,
                    },
                    duration: Fraction {
                        numerator: 1,
                        denominator: 8,
                    },
                    kind: EventKind::Hit,
                    glyph: "d".into(),
                    modifiers: vec!["accent".into()],
                    modifier: Some("accent".into()),
                    voice: 2,
                    beam: "none".into(),
                    tuplet: None,
                },
                RenderEvent {
                    track: "ST".into(),
                    track_family: "text".into(),
                    start: Fraction {
                        numerator: 0,
                        denominator: 1,
                    },
                    duration: Fraction {
                        numerator: 1,
                        denominator: 8,
                    },
                    kind: EventKind::Sticking,
                    glyph: "R".into(),
                    modifiers: vec![],
                    modifier: None,
                    voice: 1,
                    beam: "none".into(),
                    tuplet: None,
                },
            ],
            barline: Some("regular".into()),
            closing_barline: Some("final".into()),
            start_nav: None,
            end_nav: None,
            volta_indices: None,
            hairpins: vec![],
            measure_repeat_slashes: None,
            multi_rest_count: None,
            note_value: 8,
            volta_terminator: false,
        };
        let scene = build_layout_scene(
            &simple_layout_score(vec![measure]),
            &LayoutOptions::default(),
        );
        let noteheads = items_by_role(&scene, "notehead");
        let stems = items_by_role(&scene, "stem");
        let accents = items_by_role(&scene, "accent");
        let sticking = items_by_role(&scene, "sticking")
            .into_iter()
            .next()
            .expect("expected sticking");
        let mut xs = noteheads
            .iter()
            .filter_map(|item| match &item.primitive {
                ScenePrimitive::TextRun(text) => Some(text.x_pt),
                _ => None,
            })
            .collect::<Vec<_>>();
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());

        assert_eq!(noteheads.len(), 2);
        assert_eq!(stems.len(), 2, "opposing voices should keep separate stems");
        assert!(
            xs[1] - xs[0] >= 6.0,
            "opposing voices on the same slot should be horizontally separated: {xs:?}"
        );
        assert!(
            accents.iter().all(|accent| accent.anchor_item_id.is_some()),
            "accents should preserve their note anchors"
        );
        let accent_roles = accents
            .iter()
            .map(|accent| match &accent.primitive {
                ScenePrimitive::GlyphRun(glyph) => glyph.glyph_role,
                _ => panic!("accent should be glyph"),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            accent_roles,
            vec![GlyphRole::ArticAccentAbove, GlyphRole::ArticAccentBelow]
        );
        assert!(
            sticking.anchor_item_id.is_some(),
            "sticking should preserve its anchor"
        );
        assert!(
            stems.iter().all(|stem| stem.anchor_item_id.is_some()),
            "stems should preserve note anchors"
        );
    }

    #[test]
    fn test_accent_uses_smufl_glyph_centered_on_notehead_and_clears_stem_tip() {
        let measure = RenderMeasure {
            index: 0,
            global_index: 0,
            paragraph_index: 0,
            measure_in_paragraph: 0,
            source_line: 1,
            events: vec![
                RenderEvent {
                    track: "HH".into(),
                    track_family: "cymbal".into(),
                    start: Fraction {
                        numerator: 0,
                        denominator: 1,
                    },
                    duration: Fraction {
                        numerator: 1,
                        denominator: 8,
                    },
                    kind: EventKind::Hit,
                    glyph: "x".into(),
                    modifiers: vec!["accent".into()],
                    modifier: Some("accent".into()),
                    voice: 1,
                    beam: "none".into(),
                    tuplet: None,
                },
                RenderEvent {
                    track: "HH".into(),
                    track_family: "cymbal".into(),
                    start: Fraction {
                        numerator: 1,
                        denominator: 8,
                    },
                    duration: Fraction {
                        numerator: 1,
                        denominator: 8,
                    },
                    kind: EventKind::Hit,
                    glyph: "x".into(),
                    modifiers: vec![],
                    modifier: None,
                    voice: 1,
                    beam: "none".into(),
                    tuplet: None,
                },
            ],
            barline: Some("regular".into()),
            closing_barline: Some("final".into()),
            start_nav: None,
            end_nav: None,
            volta_indices: None,
            hairpins: vec![],
            measure_repeat_slashes: None,
            multi_rest_count: None,
            note_value: 8,
            volta_terminator: false,
        };
        let scene = build_layout_scene(
            &simple_layout_score(vec![measure]),
            &LayoutOptions::default(),
        );
        let accent = items_by_role(&scene, "accent")
            .into_iter()
            .next()
            .expect("expected accent");
        let notehead = items_by_role(&scene, "notehead")
            .into_iter()
            .next()
            .expect("expected notehead");
        let stem = items_by_role(&scene, "stem")
            .into_iter()
            .next()
            .expect("expected stem");

        let ScenePrimitive::GlyphRun(accent_glyph) = &accent.primitive else {
            panic!("accent should be glyph");
        };
        let ScenePrimitive::TextRun(note_text) = &notehead.primitive else {
            panic!("notehead should be text");
        };
        let ScenePrimitive::LineSegment(stem_line) = &stem.primitive else {
            panic!("stem should be line");
        };

        assert_eq!(accent_glyph.glyph_role, GlyphRole::ArticAccentAbove);
        let note_center = note_text.x_pt
            + rendered_glyph_width(GlyphRole::NoteheadX, note_text.font_size_pt) * 0.5;
        let accent_center = accent_glyph.x_pt
            + rendered_glyph_width(GlyphRole::ArticAccentAbove, accent_glyph.font_size_pt) * 0.5;
        assert!((note_center - accent_center).abs() < 0.01);
        assert!(accent_glyph.y_pt < stem_line.y1_pt);
    }
}

// ── Slot → X Mapping (Task 2) ───────────────────────────────────

/// Converts a uniform slot grid position to a horizontal X coordinate (in px).
/// The engine uses proportional spacing with content-weighted bonuses.
pub struct SlotMapper {
    pub px_per_quarter: f32,
}

impl SlotMapper {
    pub fn new(px_per_quarter: f32) -> Self {
        Self { px_per_quarter }
    }

    /// Map a slot index within a beat to a horizontal offset from the beat start.
    /// slots_per_beat = `divisions / beats` for this measure.
    pub fn slot_x_within_beat(&self, slot: u32, slots_per_beat: u32, beat_width: f32) -> f32 {
        let frac = slot as f32 / slots_per_beat as f32;
        frac * beat_width
    }

    /// Full measure width in pixels. Content-weighted: denser rhythms get more space.
    pub fn measure_width(&self, total_slots: u32, slots_per_quarter: u32, is_compact: bool) -> f32 {
        if is_compact {
            return 40.0;
        }
        let quarters = total_slots as f32 / slots_per_quarter as f32;
        quarters * self.px_per_quarter
    }

    /// Beat width for a specific beat group.
    pub fn beat_width(&self, beat_slots: u32, slots_per_quarter: u32) -> f32 {
        let quarters = beat_slots as f32 / slots_per_quarter as f32;
        // Dense beats (≤ 1/16) get +15% bonus
        let density_bonus = if beat_slots > 1 { 1.15 } else { 1.0 };
        quarters * self.px_per_quarter * density_bonus
    }
}

#[derive(Debug, Clone)]
struct GroupGeometry {
    end_slot: u32,
    width_pt: f32,
    /// Position of each event start within the group, as fraction of group width.
    /// Maps slot → cumulative offset fraction (0..1). Used by x_for_fraction.
    segment_offsets: Vec<f32>,
    segment_slots: Vec<u32>,
}

#[derive(Debug, Clone)]
struct MeasureGeometry {
    inner_left_pt: f32,
    inner_width_pt: f32,
    groups: Vec<GroupGeometry>,
}

impl MeasureGeometry {
    fn x_for_fraction(&self, header: &RenderHeader, fraction: Fraction) -> f32 {
        if self.groups.is_empty() || self.inner_width_pt <= 0.0 {
            return self.inner_left_pt;
        }

        let slot = fraction_to_measure_slot(
            fraction,
            header.time_beats,
            header.time_beat_unit,
            header.divisions,
        );
        let mut group_start_x = self.inner_left_pt;

        for group in &self.groups {
            if slot < group.end_slot {
                if group.segment_slots.is_empty() {
                    return group_start_x;
                }
                // Binary search for the segment containing this slot
                let seg = match group.segment_slots.binary_search(&slot) {
                    Ok(i) => i,
                    Err(i) => i.saturating_sub(1),
                };
                let offset_frac = group.segment_offsets[seg.min(group.segment_offsets.len() - 1)];
                return group_start_x + offset_frac * group.width_pt;
            }
            group_start_x += group.width_pt;
        }

        self.inner_left_pt + self.inner_width_pt
    }
}

// ── Layout Element Type (Tasks 3-6) ─────────────────────────────

/// A single element on the layout plan.
#[derive(Debug, Clone)]
pub struct LayoutElement {
    pub kind: ElementKind,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub smufl_codepoint: Option<u32>,
    pub voice: Option<u8>,
    pub stem_up: Option<bool>,
    pub barline_type: Option<String>,
    pub text: Option<String>,
    pub from_x: Option<f32>,
    pub to_x: Option<f32>,
    pub priority: u8, // for edge stacking (0=innermost)
    pub can_shift_y: bool,
    pub can_shift_x: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementKind {
    Note,
    Rest,
    Barline,
    Sticking,
    Modifier,
    GraceNote,
    Beam,
    Stem,
    Hairpin,
    Volta,
    NavMarker,
    Text,
    Clef,
    TimeSignature,
}

// ── Note/Rest Placement (Task 3) ────────────────────────────────

/// Place notes and rests from a single measure's events.
pub fn place_notes(
    measure: &NormalizedMeasure,
    mapper: &SlotMapper,
    _opts: &LayoutOptions,
) -> Vec<LayoutElement> {
    let mut elements = Vec::new();
    for ev in &measure.events {
        let x = mapper.slot_x_within_beat(
            to_slots(&ev.start, measure.note_value),
            slots_per_beat(measure),
            beat_width_for(measure, &ev.start),
        );
        let y = staff_y_for_track(&ev.track);
        let metrics = if ev.kind == EventKind::Rest {
            rest_glyph(ev.duration.denominator)
        } else {
            notehead_glyph(&ev.track, &ev.modifiers, &ev.glyph)
        };

        elements.push(LayoutElement {
            kind: if ev.kind == EventKind::Rest {
                ElementKind::Rest
            } else {
                ElementKind::Note
            },
            x,
            y,
            width: metrics.width_ss() * 10.0,
            height: metrics.bbox_height_ss() * 10.0,
            smufl_codepoint: Some(metrics.smufl_codepoint),
            voice: Some(ev.voice),
            stem_up: Some(ev.voice == 1),
            barline_type: None,
            text: None,
            from_x: None,
            to_x: None,
            priority: 0,
            can_shift_y: false,
            can_shift_x: false,
        });
    }
    elements
}

// ── Measure Barlines (Task 6) ───────────────────────────────────

pub fn place_barlines(measure: &NormalizedMeasure, measure_x: f32) -> Vec<LayoutElement> {
    let mut elements = Vec::new();
    let bar_type = measure.barline.as_deref().unwrap_or("regular");
    elements.push(LayoutElement {
        kind: ElementKind::Barline,
        x: measure_x,
        y: 0.0,
        width: 2.0,
        height: crate::STAFF_HEIGHT_SS * 10.0,
        smufl_codepoint: None,
        voice: None,
        stem_up: None,
        barline_type: Some(bar_type.to_string()),
        text: None,
        from_x: None,
        to_x: None,
        priority: 0,
        can_shift_y: false,
        can_shift_x: false,
    });
    elements
}

// ── Edge Element Stacking (Task 7) ───────────────────────────────

/// Push lower-priority edge elements outward when they overlap.
/// Returns the resolved elements with Y positions adjusted.
pub fn stack_edge_elements(elements: &mut [LayoutElement], edge_padding: f32) -> Vec<String> {
    let mut warnings = Vec::new();
    let max_iters = 5;

    for _iter in 0..max_iters {
        let mut any_overlap = false;

        for i in 0..elements.len() {
            for j in (i + 1)..elements.len() {
                let (a, b) = if elements[i].priority < elements[j].priority {
                    (&elements[i].clone(), &elements[j].clone())
                } else {
                    (&elements[j].clone(), &elements[i].clone())
                };

                // Check X overlap
                let a_right = a.x + a.width;
                let b_right = b.x + b.width;
                let x_overlap = a.x < b_right && a_right > b.x;
                if !x_overlap {
                    continue;
                }

                // Check Y overlap
                let a_bottom = a.y + a.height;
                let b_bottom = b.y + b.height;
                let y_overlap = a.y < b_bottom && a_bottom > b.y;
                if !y_overlap {
                    continue;
                }

                any_overlap = true;
                let overlap = a_bottom.min(b_bottom) - a.y.max(b.y);
                let push = overlap + edge_padding;

                // Try to push lower-priority element (b)
                if elements[j].can_shift_y {
                    elements[j].y += push;
                } else if elements[i].can_shift_y {
                    elements[i].y -= push;
                } else {
                    warnings.push(format!("unresolved overlap at x={:.1}", a.x));
                }
            }
        }

        if !any_overlap {
            break;
        }
    }

    warnings
}

// ── System Layout (Task 2) ──────────────────────────────────────

/// A single system (one line of music) containing measures.
#[derive(Debug, Clone)]
pub struct System {
    pub y: f32,
    pub height: f32,
    pub measures: Vec<MeasureLayout>,
}

#[derive(Debug, Clone)]
pub struct MeasureLayout {
    pub x: f32,
    pub width: f32,
    pub elements: Vec<LayoutElement>,
}

#[derive(Clone, Debug)]
struct BeamAnchor {
    x: f32,
    stem_x: f32,
    stem_tip_y: f32,
    voice: u8,
    group: u32,
    level: u8,
    up: bool,
    stem_item_id: String,
}

#[derive(Clone, Copy, Debug)]
struct BeamLineSegment {
    start_x: f32,
    end_x: f32,
}

#[derive(Clone)]
struct SlotEvent<'a> {
    slot: u32,
    event_x: f32,
    event: &'a RenderEvent,
}

#[derive(Clone, Copy, Debug)]
struct BeamRunState {
    segment: usize,
    group: u32,
}

#[derive(Clone)]
struct NotePlacement {
    note_id: String,
    note_x: f32,
    note_y: f32,
    note_center_x: f32,
    has_accent: bool,
    stem_up_anchor_ss: Option<GlyphPoint>,
    stem_down_anchor_ss: Option<GlyphPoint>,
}

/// Build systems from a NormalizedScore.
pub fn build_systems(score: &NormalizedScore, opts: &LayoutOptions) -> Vec<System> {
    let mapper = SlotMapper::new(opts.px_per_quarter);
    let mut systems = Vec::new();
    let mut current_system = System {
        y: opts.top_margin_pt,
        height: STAFF_HEIGHT_SS * 10.0 * opts.staff_scale,
        measures: Vec::new(),
    };
    let mut cursor_x = opts.left_margin_pt + 30.0 + 40.0; // clef + time sig
    let usable_width =
        opts.page_width_pt - opts.left_margin_pt - opts.right_margin_pt - 30.0 - 40.0;

    for measure in &score.measures {
        let is_compact =
            measure.multi_rest_count.is_some() || measure.measure_repeat_slashes.is_some();
        let total_slots = measure.events.len() as u32; // simplified
        let width = mapper.measure_width(total_slots.max(1), 4, is_compact);

        if cursor_x + width > opts.left_margin_pt + usable_width
            && !current_system.measures.is_empty()
        {
            systems.push(current_system);
            current_system = System {
                y: opts.top_margin_pt + (systems.len() as f32 + 1.0) * (opts.staff_scale * 80.0),
                height: STAFF_HEIGHT_SS * 10.0 * opts.staff_scale,
                measures: Vec::new(),
            };
            cursor_x = opts.left_margin_pt + 30.0 + 40.0;
        }

        let mut elements = Vec::new();
        elements.extend(place_notes(measure, &mapper, opts));
        elements.extend(place_barlines(measure, cursor_x));

        current_system.measures.push(MeasureLayout {
            x: cursor_x,
            width,
            elements,
        });
        cursor_x += width;
    }

    if !current_system.measures.is_empty() {
        systems.push(current_system);
    }
    systems
}

// ── Helpers ──────────────────────────────────────────────────────

fn to_slots(f: &Fraction, note_value: u32) -> u32 {
    (f.numerator * note_value) / f.denominator.max(1)
}

fn slots_per_beat(_measure: &NormalizedMeasure) -> u32 {
    4
} // simplified
fn beat_width_for(_measure: &NormalizedMeasure, _start: &Fraction) -> f32 {
    80.0
}

struct SystemStartReservation {
    opening_barline_thickness: f32,
    clef_width: f32,
    clef_trailing_gap: f32,
    time_signature_width: f32,
    time_signature_trailing_gap: f32,
}

const MEASURE_RIGHT_PAD_PT: f32 = 14.0;
const NON_INITIAL_MEASURE_LEFT_PAD_PT: f32 = 14.0;
const SVG_POINT_TO_USER_UNIT: f32 = 4.0 / 3.0;
const REPEAT_BARLINE_FONT_SIZE_PT: f32 = 30.0;
const FIRST_MEASURE_START_REPEAT_PREAMBLE_PULL_PT: f32 = 10.0;
const START_REPEAT_TRAILING_GAP_PT: f32 = 22.0;
const VOLTA_TEXT_SIZE_PT: f32 = 12.0;
const VOLTA_LINE_HEIGHT_PT: f32 = 15.0;
const VOLTA_LINE_THICKNESS_PT: f32 = 1.0;
const VOLTA_SKYLINE_GAP_PT: f32 = 4.0;

impl SystemStartReservation {
    fn width(&self) -> f32 {
        self.opening_barline_thickness
            + self.clef_width
            + self.clef_trailing_gap
            + self.time_signature_width
            + self.time_signature_trailing_gap
    }
}

fn system_start_reservation(is_first_system: bool) -> SystemStartReservation {
    SystemStartReservation {
        opening_barline_thickness: 1.0,
        clef_width: 25.0,
        clef_trailing_gap: 18.0,
        time_signature_width: if is_first_system { 24.0 } else { 0.0 },
        time_signature_trailing_gap: if is_first_system { 18.0 } else { 0.0 },
    }
}

fn is_start_repeat_barline(barline: Option<&str>) -> bool {
    matches!(barline, Some("repeat-start") | Some("repeat-both"))
}

fn start_repeat_reserved_width() -> f32 {
    repeat_barline_rendered_width(GlyphRole::RepeatLeft) + START_REPEAT_TRAILING_GAP_PT
}

fn first_measure_start_repeat_x(measure_x: f32, is_first_system: bool) -> f32 {
    measure_x + system_start_reservation(is_first_system).width()
        - FIRST_MEASURE_START_REPEAT_PREAMBLE_PULL_PT
}

fn start_repeat_vertical_origin(top: f32, bottom: f32) -> f32 {
    let height_pt = repeat_barline_rendered_height(GlyphRole::RepeatLeft);
    top + (bottom - top - height_pt) * 0.5 + height_pt
}

fn repeat_barline_rendered_width(role: GlyphRole) -> f32 {
    rendered_glyph_width(role, REPEAT_BARLINE_FONT_SIZE_PT)
}

fn repeat_barline_rendered_height(role: GlyphRole) -> f32 {
    rendered_glyph_height(role, REPEAT_BARLINE_FONT_SIZE_PT)
}

fn rendered_glyph_width(role: GlyphRole, font_size_pt: f32) -> f32 {
    canonical_glyph_metric(role).width_pt(font_size_pt) * SVG_POINT_TO_USER_UNIT
}

fn rendered_glyph_height(role: GlyphRole, font_size_pt: f32) -> f32 {
    let metric = canonical_glyph_metric(role);
    metric.bbox_height_ss() * (font_size_pt / 4.0) * SVG_POINT_TO_USER_UNIT
}

fn measure_left_pad(
    measure_index_in_system: usize,
    is_first_system: bool,
    barline: Option<&str>,
) -> f32 {
    if measure_index_in_system == 0 {
        let repeat_start_width = if is_start_repeat_barline(barline) {
            start_repeat_reserved_width() - FIRST_MEASURE_START_REPEAT_PREAMBLE_PULL_PT
        } else {
            0.0
        };
        system_start_reservation(is_first_system).width() + repeat_start_width
    } else {
        NON_INITIAL_MEASURE_LEFT_PAD_PT
    }
}

#[derive(Debug)]
struct PlannedSystem<'a> {
    measures: Vec<&'a DisplayMeasure<'a>>,
    widths: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MeasureRepeatDisplayPart {
    Single,
    TwoBarStart,
    TwoBarStop,
}

#[derive(Debug, Clone)]
struct DisplayMeasure<'a> {
    measure: &'a RenderMeasure,
    global_index: u32,
    paragraph_index: u32,
    barline: Option<String>,
    closing_barline: Option<String>,
    start_nav: Option<NavMarker>,
    end_nav: Option<NavJump>,
    hairpins: Vec<HairpinSpan>,
    repeat_part: Option<MeasureRepeatDisplayPart>,
}

#[derive(Debug, Clone)]
struct ExpandedLayoutData<'a> {
    measures: Vec<DisplayMeasure<'a>>,
}

fn normalized_grouping(header: &RenderHeader) -> Vec<u32> {
    let fallback = vec![1; header.time_beats.max(1) as usize];
    if header.grouping.is_empty() {
        return fallback;
    }

    let grouping_sum: u32 = header.grouping.iter().sum();
    if grouping_sum == header.time_beats {
        header.grouping.clone()
    } else {
        fallback
    }
}

fn fraction_to_measure_slot(
    fraction: Fraction,
    time_beats: u32,
    time_beat_unit: u32,
    divisions: u32,
) -> u32 {
    let numerator =
        fraction.numerator as u64 * divisions.max(1) as u64 * time_beat_unit.max(1) as u64;
    let denominator = fraction.denominator.max(1) as u64 * time_beats.max(1) as u64;
    ((numerator + denominator / 2) / denominator) as u32
}

fn grouping_segment_index_for_slot(header: &RenderHeader, slot: u32) -> usize {
    let grouping = normalized_grouping(header);
    let slots_per_beat_unit = (header.divisions / header.time_beats.max(1)).max(1);
    let mut boundary = 0_u32;
    for (index, beat_units) in grouping.iter().enumerate() {
        boundary += (*beat_units).max(1) * slots_per_beat_unit;
        if slot < boundary {
            return index;
        }
    }
    grouping.len().saturating_sub(1)
}

fn is_beamable_duration(duration: Fraction) -> bool {
    let divisor = gcd_u32(duration.numerator, duration.denominator).max(1);
    duration.denominator / divisor >= 8
}

fn gcd_u32(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a
}

struct MeasureGeometryInput {
    measure_x: f32,
    measure_width: f32,
    left_pad: f32,
    right_pad: f32,
    duration_compression: f32,
}

fn measure_geometry(
    header: &RenderHeader,
    measure: &RenderMeasure,
    mapper: &SlotMapper,
    input: &MeasureGeometryInput,
) -> MeasureGeometry {
    let inner_left_pt = input.measure_x + input.left_pad;
    let inner_width_pt = (input.measure_width - input.left_pad - input.right_pad).max(1.0);
    let slots_per_beat_unit = (header.divisions / header.time_beats.max(1)).max(1);
    let grouping = normalized_grouping(header);
    let mut groups = Vec::new();
    let mut weighted_width_sum = 0.0_f32;
    let mut start_slot = 0_u32;

    // Collect all event start slots for the measure (once)
    let mut all_starts: Vec<u32> = measure
        .events
        .iter()
        .map(|event| {
            fraction_to_measure_slot(
                event.start,
                header.time_beats,
                header.time_beat_unit,
                header.divisions,
            )
        })
        .collect();
    all_starts.sort();
    all_starts.dedup();

    for beat_units in grouping {
        let group_slots = beat_units.max(1) * slots_per_beat_unit;
        let end_slot = start_slot + group_slots;
        let base_quarters = beat_units as f32 * 4.0 / header.time_beat_unit.max(1) as f32;

        // Content weight for measure-width compression
        let group_starts: Vec<u32> = all_starts
            .iter()
            .copied()
            .filter(|s| *s >= start_slot && *s < end_slot)
            .collect();
        let segment_count = if group_starts.is_empty() {
            1
        } else {
            group_starts.len().max(1)
        };
        let content_weight =
            1.0 + input.duration_compression * (segment_count as f32).max(1.0).log2();
        let weighted_width = base_quarters * mapper.px_per_quarter * content_weight;
        weighted_width_sum += weighted_width;

        // Duration-weighted segment offsets within this group
        let mut segment_slots: Vec<u32> = group_starts;
        // Ensure we have at least start_slot as first segment
        if segment_slots.first() != Some(&start_slot) {
            segment_slots.insert(0, start_slot);
        }
        // Add group end as last segment boundary
        if segment_slots.last() != Some(&end_slot) {
            segment_slots.push(end_slot);
        }

        let mut segment_offsets = Vec::with_capacity(segment_slots.len());
        if segment_slots.len() <= 2 {
            // One segment: linear
            segment_offsets.push(0.0);
        } else {
            // Compute segment durations and weights
            let slot_span = (end_slot - start_slot).max(1) as f32;
            let mut raw_weights = Vec::with_capacity(segment_slots.len() - 1);
            for i in 0..segment_slots.len() - 1 {
                let seg_slots = (segment_slots[i + 1] - segment_slots[i]) as f32;
                let seg_duration = seg_slots / slot_span;
                raw_weights.push(seg_duration);
            }

            // Apply compression: weight = 1 + compression * log2(ratio + 1)
            let min_dur = raw_weights
                .iter()
                .fold(f32::MAX, |a, &b| if b > 0.0 { a.min(b) } else { a });
            let min_dur = min_dur.max(0.01);
            let weights: Vec<f32> = raw_weights
                .iter()
                .map(|&d| {
                    let ratio = d / min_dur;
                    1.0 + input.duration_compression * (ratio + 1.0).log2()
                })
                .collect();

            let total_weight: f32 = weights.iter().sum();
            let mut cum = 0.0_f32;
            segment_offsets.push(0.0);
            for &w in &weights[..weights.len() - 1] {
                cum += w / total_weight.max(1e-6);
                segment_offsets.push(cum);
            }
        }

        groups.push(GroupGeometry {
            end_slot,
            width_pt: weighted_width,
            segment_offsets,
            segment_slots,
        });
        start_slot = end_slot;
    }

    let scale = inner_width_pt / weighted_width_sum.max(1.0);
    for group in &mut groups {
        group.width_pt *= scale;
    }

    MeasureGeometry {
        inner_left_pt,
        inner_width_pt,
        groups,
    }
}

fn estimated_measure_width(
    header: &RenderHeader,
    measure: &RenderMeasure,
    mapper: &SlotMapper,
    compression: f32,
) -> f32 {
    if measure.multi_rest_count.is_some() || measure.measure_repeat_slashes.is_some() {
        return mapper.measure_width(1, 1, true);
    }

    let grouping = normalized_grouping(header);
    let slots_per_beat_unit = (header.divisions / header.time_beats.max(1)).max(1);

    // Collect all unique event start slots
    let mut starts: Vec<u32> = measure
        .events
        .iter()
        .map(|event| {
            fraction_to_measure_slot(
                event.start,
                header.time_beats,
                header.time_beat_unit,
                header.divisions,
            )
        })
        .collect();
    starts.sort();
    starts.dedup();
    let segment_count = starts.len().max(1);

    // Modifier bonuses (matching VexFlow)
    let has_tuplet = measure.events.iter().any(|event| event.tuplet.is_some());
    let sticking_count = measure
        .events
        .iter()
        .filter(|event| matches!(event.kind, EventKind::Sticking))
        .count();
    let modifier_bonus =
        (if has_tuplet { 0.15 } else { 0.0 }) + (if sticking_count >= 3 { 0.1 } else { 0.0 });

    grouping
        .into_iter()
        .scan(0_u32, |start_slot, beat_units| {
            let base_quarters = beat_units as f32 * 4.0 / header.time_beat_unit.max(1) as f32;
            let content_weight =
                1.0 + compression * (segment_count as f32).max(1.0).log2() + modifier_bonus;
            *start_slot += beat_units.max(1) * slots_per_beat_unit;
            Some(base_quarters * mapper.px_per_quarter * content_weight)
        })
        .sum()
}

fn left_edge_barline(barline: Option<&str>) -> Option<String> {
    match barline {
        Some("repeat-start") | Some("repeat-both") => Some("repeat-start".to_string()),
        _ => None,
    }
}

fn right_edge_barline(barline: Option<&str>) -> Option<String> {
    match barline {
        Some("repeat-end") | Some("repeat-both") => Some("repeat-end".to_string()),
        Some("double") => Some("double".to_string()),
        Some("final") => Some("final".to_string()),
        _ => None,
    }
}

fn expand_layout_data<'a>(score: &'a RenderScore) -> ExpandedLayoutData<'a> {
    let mut display_slots: Vec<Vec<u32>> = Vec::with_capacity(score.measures.len());
    let mut next_index = 0_u32;
    for measure in &score.measures {
        if measure.measure_repeat_slashes == Some(2) {
            display_slots.push(vec![next_index, next_index + 1]);
            next_index += 2;
        } else {
            display_slots.push(vec![next_index]);
            next_index += 1;
        }
    }

    let map_start = |original: u32| -> u32 {
        display_slots
            .get(original as usize)
            .and_then(|slots| slots.first().copied())
            .unwrap_or(original)
    };
    let map_end = |original: u32| -> u32 {
        display_slots
            .get(original as usize)
            .and_then(|slots| slots.last().copied())
            .unwrap_or(original)
    };
    let map_hairpins = |hairpins: &[HairpinSpan]| -> Vec<HairpinSpan> {
        hairpins
            .iter()
            .map(|hairpin| HairpinSpan {
                kind: hairpin.kind,
                start: hairpin.start,
                end: hairpin.end,
                start_measure_index: map_start(hairpin.start_measure_index),
                end_measure_index: map_end(hairpin.end_measure_index),
            })
            .collect()
    };

    let mut measures = Vec::new();
    let mut paragraph_measure_counts: std::collections::BTreeMap<u32, u32> =
        std::collections::BTreeMap::new();
    for (measure_index, measure) in score.measures.iter().enumerate() {
        let slots = &display_slots[measure_index];
        for (slot_index, display_index) in slots.iter().enumerate() {
            let paragraph_counter = paragraph_measure_counts
                .entry(measure.paragraph_index)
                .or_insert(0);
            *paragraph_counter += 1;

            let repeat_part = match measure.measure_repeat_slashes {
                Some(1) => Some(MeasureRepeatDisplayPart::Single),
                Some(2) if slot_index == 0 => Some(MeasureRepeatDisplayPart::TwoBarStart),
                Some(2) => Some(MeasureRepeatDisplayPart::TwoBarStop),
                _ => None,
            };

            let (barline, closing_barline, start_nav, end_nav, hairpins) = match repeat_part {
                Some(MeasureRepeatDisplayPart::TwoBarStart) => (
                    left_edge_barline(measure.barline.as_deref()),
                    left_edge_barline(measure.closing_barline.as_deref()),
                    measure.start_nav.clone(),
                    None,
                    map_hairpins(&measure.hairpins),
                ),
                Some(MeasureRepeatDisplayPart::TwoBarStop) => (
                    right_edge_barline(measure.barline.as_deref()),
                    right_edge_barline(measure.closing_barline.as_deref()),
                    None,
                    measure.end_nav.clone(),
                    Vec::new(),
                ),
                _ => (
                    measure.barline.clone(),
                    measure.closing_barline.clone(),
                    measure.start_nav.clone(),
                    measure.end_nav.clone(),
                    map_hairpins(&measure.hairpins),
                ),
            };

            measures.push(DisplayMeasure {
                measure,
                global_index: *display_index,
                paragraph_index: measure.paragraph_index,
                barline,
                closing_barline,
                start_nav,
                end_nav,
                hairpins,
                repeat_part,
            });
        }
    }

    ExpandedLayoutData { measures }
}

fn finalize_planned_system<'a>(
    systems: &mut Vec<PlannedSystem<'a>>,
    current_measures: Vec<&'a DisplayMeasure<'a>>,
    current_inner_estimates: Vec<f32>,
    is_first_system: bool,
    available_width: f32,
) {
    if current_measures.is_empty() {
        return;
    }
    let fixed_width: f32 = current_inner_estimates
        .iter()
        .enumerate()
        .map(|(index, _)| {
            let left = measure_left_pad(
                index,
                is_first_system,
                current_measures[index].barline.as_deref(),
            );
            left + MEASURE_RIGHT_PAD_PT
        })
        .sum();
    let current_inner_sum: f32 = current_inner_estimates.iter().sum();
    let scale = ((available_width - fixed_width).max(1.0) / current_inner_sum.max(1.0)).max(0.01);
    let widths = current_inner_estimates
        .into_iter()
        .enumerate()
        .map(|(index, width)| {
            let left = measure_left_pad(
                index,
                is_first_system,
                current_measures[index].barline.as_deref(),
            );
            width * scale + left + MEASURE_RIGHT_PAD_PT
        })
        .collect();
    systems.push(PlannedSystem {
        measures: current_measures,
        widths,
    });
}

fn plan_scene_systems<'a>(
    header: &RenderHeader,
    measures: &'a [DisplayMeasure<'a>],
    opts: &LayoutOptions,
) -> Vec<PlannedSystem<'a>> {
    let mapper = SlotMapper::new(opts.px_per_quarter);
    let available_width =
        (opts.page_width_pt - opts.left_margin_pt - opts.right_margin_pt).max(100.0);
    let mut systems: Vec<PlannedSystem<'a>> = Vec::new();
    let mut current_measures: Vec<&'a DisplayMeasure<'a>> = Vec::new();
    let mut current_inner_estimates: Vec<f32> = Vec::new();
    let mut current_paragraph: Option<u32> = None;
    let mut next_is_first_system = true;

    for measure in measures {
        let estimate = estimated_measure_width(
            header,
            measure.measure,
            &mapper,
            opts.measure_width_compression,
        );
        let paragraph_break =
            current_paragraph.is_some() && current_paragraph != Some(measure.paragraph_index);
        if !current_measures.is_empty() && paragraph_break {
            finalize_planned_system(
                &mut systems,
                current_measures,
                current_inner_estimates,
                next_is_first_system,
                available_width,
            );
            current_measures = Vec::new();
            current_inner_estimates = Vec::new();
            next_is_first_system = false;
        }

        current_paragraph = Some(measure.paragraph_index);
        current_measures.push(measure);
        current_inner_estimates.push(estimate);
    }

    finalize_planned_system(
        &mut systems,
        current_measures,
        current_inner_estimates,
        next_is_first_system,
        available_width,
    );

    systems
}

// ── Platform-Neutral Scene Output ───────────────────────────────

#[allow(dead_code)]
fn render_header_layout_box(header: &RenderHeader, opts: &LayoutOptions) -> HeaderLayoutBox {
    let page_w = opts.page_width_pt;
    let margin = opts.left_margin_pt;
    let center_x = page_w / 2.0;
    let header_bottom_y = opts.top_margin_pt + opts.header_height_pt;
    let mut item_counter = 0usize;
    let mut items = Vec::new();
    let mut composites = Vec::new();
    let mut sink = SceneEmitSink::new(&mut items, &mut item_counter);

    let title_metric = canonical_text_metric(TextRole::Title);
    let subtitle_metric = canonical_text_metric(TextRole::Subtitle);
    let composer_metric = canonical_text_metric(TextRole::Composer);
    let tempo_metric = canonical_text_metric(TextRole::Tempo);
    let title_y = opts.top_margin_pt + title_metric.ascent_pt + 18.0;
    let subtitle_y = header_bottom_y + subtitle_metric.ascent_pt + 12.0;
    let composer_y = header_bottom_y + composer_metric.ascent_pt + 12.0;
    let tempo_y = header_bottom_y + opts.header_staff_spacing_pt + opts.tempo_offset_y;

    if let Some(ref text) = header.title {
        let title_id = sink.push_text_item(TextItemSpec {
            measure_id: None,
            role: "title",
            x: center_x,
            y: title_y,
            text_role: TextRole::Title,
            text: text.clone(),
            font_family: title_metric.font_family,
            font_size_pt: title_metric.font_size_pt,
            fill: "#333",
            text_anchor: Some("middle"),
            font_weight: Some("bold"),
        });
        composites.push(SceneComposite {
            id: "text-block-title".to_string(),
            kind: CompositeKind::TextBlock,
            fragment: SpanFragmentKind::SingleSegment,
            child_item_ids: vec![title_id],
            label: Some("title".to_string()),
            count: None,
            start_anchor_id: None,
            end_anchor_id: None,
        });
    }
    if let Some(ref text) = header.subtitle {
        let subtitle_id = sink.push_text_item(TextItemSpec {
            measure_id: None,
            role: "subtitle",
            x: center_x,
            y: subtitle_y,
            text_role: TextRole::Subtitle,
            text: text.clone(),
            font_family: subtitle_metric.font_family,
            font_size_pt: subtitle_metric.font_size_pt,
            fill: "#333",
            text_anchor: Some("middle"),
            font_weight: None,
        });
        composites.push(SceneComposite {
            id: "text-block-subtitle".to_string(),
            kind: CompositeKind::TextBlock,
            fragment: SpanFragmentKind::SingleSegment,
            child_item_ids: vec![subtitle_id],
            label: Some("subtitle".to_string()),
            count: None,
            start_anchor_id: None,
            end_anchor_id: None,
        });
    }
    if let Some(ref text) = header.composer {
        let composer_id = sink.push_text_item(TextItemSpec {
            measure_id: None,
            role: "composer",
            x: page_w - margin,
            y: composer_y,
            text_role: TextRole::Composer,
            text: text.clone(),
            font_family: composer_metric.font_family,
            font_size_pt: composer_metric.font_size_pt,
            fill: "#333",
            text_anchor: Some("end"),
            font_weight: None,
        });
        composites.push(SceneComposite {
            id: "text-block-composer".to_string(),
            kind: CompositeKind::TextBlock,
            fragment: SpanFragmentKind::SingleSegment,
            child_item_ids: vec![composer_id],
            label: Some("composer".to_string()),
            count: None,
            start_anchor_id: None,
            end_anchor_id: None,
        });
    }
    if header.tempo > 0 {
        let tempo_glyph_x = margin + 9.0;
        let tempo_glyph_width =
            canonical_glyph_metric(GlyphRole::MetNoteQuarterUp).width_ss() * 25.0 / 4.0;
        let tempo_equals_x = tempo_glyph_x + tempo_glyph_width + 8.0;
        let tempo_value_text = header.tempo.to_string();
        let tempo_value_x = tempo_equals_x + canonical_text_width(TextRole::Tempo, "=") + 6.0;
        let tempo_glyph_id = sink.push_text_item(TextItemSpec {
            measure_id: None,
            role: "tempo-glyph",
            x: tempo_glyph_x,
            y: tempo_y,
            text_role: TextRole::Tempo,
            text: "\u{ECA5}".to_string(),
            font_family: "Bravura",
            font_size_pt: 25.0,
            fill: "#333",
            text_anchor: None,
            font_weight: None,
        });
        let tempo_equals_id = sink.push_text_item(TextItemSpec {
            measure_id: None,
            role: "tempo-equals",
            x: tempo_equals_x,
            y: tempo_y,
            text_role: TextRole::Tempo,
            text: "=".to_string(),
            font_family: tempo_metric.font_family,
            font_size_pt: tempo_metric.font_size_pt,
            fill: "#333",
            text_anchor: None,
            font_weight: None,
        });
        let tempo_value_id = sink.push_text_item(TextItemSpec {
            measure_id: None,
            role: "tempo",
            x: tempo_value_x,
            y: tempo_y,
            text_role: TextRole::Tempo,
            text: tempo_value_text,
            font_family: tempo_metric.font_family,
            font_size_pt: tempo_metric.font_size_pt,
            fill: "#333",
            text_anchor: None,
            font_weight: None,
        });
        composites.push(SceneComposite {
            id: "text-block-tempo".to_string(),
            kind: CompositeKind::TextBlock,
            fragment: SpanFragmentKind::SingleSegment,
            child_item_ids: vec![tempo_glyph_id, tempo_equals_id, tempo_value_id],
            label: Some("tempo".to_string()),
            count: Some(header.tempo),
            start_anchor_id: None,
            end_anchor_id: None,
        });
    }

    let item_bounds = bounds_for_items(&items).ok().flatten();
    let visual_top = item_bounds
        .map(|bounds| bounds.y)
        .unwrap_or(opts.top_margin_pt);
    let visual_bottom = item_bounds
        .map(|bounds| bounds.y + bounds.height)
        .unwrap_or(opts.top_margin_pt + opts.header_height_pt);

    HeaderLayoutBox {
        items,
        composites,
        visual_top,
        visual_bottom,
    }
}

pub fn build_layout_scene(score: &RenderScore, opts: &LayoutOptions) -> LayoutScene {
    let page_w = opts.page_width_pt;
    let page_h = opts.page_height_pt;
    let margin = opts.left_margin_pt;
    let staff_ss = 10.0_f32;
    let center_x = page_w / 2.0;
    let system_left = margin;
    let system_right = page_w - margin;
    let header_bottom_y = opts.top_margin_pt + opts.header_height_pt;
    let mut sys_y = header_bottom_y + opts.header_staff_spacing_pt;
    let mut item_counter = 0usize;
    let mapper = SlotMapper::new(opts.px_per_quarter);
    let expanded = expand_layout_data(score);

    let planned_systems = plan_scene_systems(&score.header, &expanded.measures, opts);

    let mut page = ScenePage {
        index: 0,
        width_pt: page_w,
        height_pt: page_h,
        systems: Vec::new(),
        measures: Vec::new(),
        items: Vec::new(),
        composites: Vec::new(),
    };
    let mut sink = SceneEmitSink::new(&mut page.items, &mut item_counter);

    let title_metric = canonical_text_metric(TextRole::Title);
    let subtitle_metric = canonical_text_metric(TextRole::Subtitle);
    let composer_metric = canonical_text_metric(TextRole::Composer);
    let title_y = opts.top_margin_pt + title_metric.ascent_pt + 18.0;
    let subtitle_y = header_bottom_y + subtitle_metric.ascent_pt + 12.0;
    let composer_y = header_bottom_y + composer_metric.ascent_pt + 12.0;

    if let Some(ref text) = score.header.title {
        let title_id = sink.push_text_item(TextItemSpec {
            measure_id: None,
            role: "title",
            x: center_x,
            y: title_y,
            text_role: TextRole::Title,
            text: text.clone(),
            font_family: title_metric.font_family,
            font_size_pt: title_metric.font_size_pt,
            fill: "#333",
            text_anchor: Some("middle"),
            font_weight: Some("bold"),
        });
        page.composites.push(SceneComposite {
            id: "text-block-title".to_string(),
            kind: CompositeKind::TextBlock,
            fragment: SpanFragmentKind::SingleSegment,
            child_item_ids: vec![title_id],
            label: Some("title".to_string()),
            count: None,
            start_anchor_id: None,
            end_anchor_id: None,
        });
    }
    if let Some(ref text) = score.header.subtitle {
        let subtitle_id = sink.push_text_item(TextItemSpec {
            measure_id: None,
            role: "subtitle",
            x: center_x,
            y: subtitle_y,
            text_role: TextRole::Subtitle,
            text: text.clone(),
            font_family: subtitle_metric.font_family,
            font_size_pt: subtitle_metric.font_size_pt,
            fill: "#333",
            text_anchor: Some("middle"),
            font_weight: None,
        });
        page.composites.push(SceneComposite {
            id: "text-block-subtitle".to_string(),
            kind: CompositeKind::TextBlock,
            fragment: SpanFragmentKind::SingleSegment,
            child_item_ids: vec![subtitle_id],
            label: Some("subtitle".to_string()),
            count: None,
            start_anchor_id: None,
            end_anchor_id: None,
        });
    }
    if let Some(ref text) = score.header.composer {
        let composer_id = sink.push_text_item(TextItemSpec {
            measure_id: None,
            role: "composer",
            x: page_w - margin,
            y: composer_y,
            text_role: TextRole::Composer,
            text: text.clone(),
            font_family: composer_metric.font_family,
            font_size_pt: composer_metric.font_size_pt,
            fill: "#333",
            text_anchor: Some("end"),
            font_weight: None,
        });
        page.composites.push(SceneComposite {
            id: "text-block-composer".to_string(),
            kind: CompositeKind::TextBlock,
            fragment: SpanFragmentKind::SingleSegment,
            child_item_ids: vec![composer_id],
            label: Some("composer".to_string()),
            count: None,
            start_anchor_id: None,
            end_anchor_id: None,
        });
    }
    let mut deferred_navs = Vec::new();

    for (sys_idx, system) in planned_systems.iter().enumerate() {
        let is_first_system = sys_idx == 0;
        let is_last = sys_idx + 1 == planned_systems.len();
        let system_id = format!("system-{sys_idx}");
        let sy = sys_y;
        sys_y += 100.0 + opts.system_spacing_pt;
        let s_top = sy + staff_ss;
        let s_bot = sy + staff_ss * 5.0;
        let s_mid = sy + staff_ss * 3.0;
        let mut mx = system_left;
        let mut measure_ids = Vec::new();

        for i in 0..5 {
            let ly = sy + staff_ss * (1.0 + i as f32);
            sink.push_line_item(LineItemSpec {
                measure_id: None,
                role: "staff-line",
                x1: system_left,
                y1: ly,
                x2: system_right,
                y2: ly,
                stroke: "#333",
                stroke_width: 1.0,
                stroke_line_cap: None,
            });
        }
        let clef_metric = canonical_text_metric(TextRole::PercussionClef);
        sink.push_text_item(TextItemSpec {
            measure_id: None,
            role: "percussion-clef",
            x: margin + 5.0,
            y: s_mid,
            text_role: TextRole::PercussionClef,
            text: "\u{E069}".to_string(),
            font_family: "Bravura",
            font_size_pt: clef_metric.font_size_pt,
            fill: "#333",
            text_anchor: None,
            font_weight: None,
        });
        if is_first_system {
            let tsx = margin + 35.0;
            let time_sig_metric = canonical_text_metric(TextRole::TimeSignatureDigit);
            sink.push_text_item(TextItemSpec {
                measure_id: None,
                role: "time-signature-digit",
                x: tsx,
                y: sy + staff_ss * 2.0,
                text_role: TextRole::TimeSignatureDigit,
                text: num_to_glyph(score.header.time_beats),
                font_family: time_sig_metric.font_family,
                font_size_pt: time_sig_metric.font_size_pt,
                fill: "#333",
                text_anchor: None,
                font_weight: None,
            });
            sink.push_text_item(TextItemSpec {
                measure_id: None,
                role: "time-signature-digit",
                x: tsx,
                y: sy + staff_ss * 4.0,
                text_role: TextRole::TimeSignatureDigit,
                text: num_to_glyph(score.header.time_beat_unit),
                font_family: time_sig_metric.font_family,
                font_size_pt: time_sig_metric.font_size_pt,
                fill: "#333",
                text_anchor: None,
                font_weight: None,
            });
        }
        if is_first_system && score.header.tempo > 0 {
            let first_measure_id = format!("measure-{}", system.measures[0].global_index);
            let tempo_metric = canonical_text_metric(TextRole::Tempo);
            let tempo_y = sy + opts.tempo_offset_y;
            let tempo_glyph_x = margin + 9.0;
            let tempo_glyph_width =
                canonical_glyph_metric(GlyphRole::MetNoteQuarterUp).width_ss() * 25.0 / 4.0;
            let tempo_equals_x = tempo_glyph_x + tempo_glyph_width + 8.0;
            let tempo_value_text = score.header.tempo.to_string();
            let tempo_value_x = tempo_equals_x + canonical_text_width(TextRole::Tempo, "=") + 6.0;
            let tempo_glyph_id = sink.push_text_item(TextItemSpec {
                measure_id: Some(&first_measure_id),
                role: "tempo-glyph",
                x: tempo_glyph_x,
                y: tempo_y,
                text_role: TextRole::Tempo,
                text: "\u{ECA5}".to_string(),
                font_family: "Bravura",
                font_size_pt: 25.0,
                fill: "#333",
                text_anchor: None,
                font_weight: None,
            });
            let tempo_equals_id = sink.push_text_item(TextItemSpec {
                measure_id: Some(&first_measure_id),
                role: "tempo-equals",
                x: tempo_equals_x,
                y: tempo_y,
                text_role: TextRole::Tempo,
                text: "=".to_string(),
                font_family: tempo_metric.font_family,
                font_size_pt: tempo_metric.font_size_pt,
                fill: "#333",
                text_anchor: None,
                font_weight: None,
            });
            let tempo_value_id = sink.push_text_item(TextItemSpec {
                measure_id: Some(&first_measure_id),
                role: "tempo",
                x: tempo_value_x,
                y: tempo_y,
                text_role: TextRole::Tempo,
                text: tempo_value_text,
                font_family: tempo_metric.font_family,
                font_size_pt: tempo_metric.font_size_pt,
                fill: "#333",
                text_anchor: None,
                font_weight: None,
            });
            page.composites.push(SceneComposite {
                id: "text-block-tempo".to_string(),
                kind: CompositeKind::TextBlock,
                fragment: SpanFragmentKind::SingleSegment,
                child_item_ids: vec![tempo_glyph_id, tempo_equals_id, tempo_value_id],
                label: Some("tempo".to_string()),
                count: Some(score.header.tempo),
                start_anchor_id: None,
                end_anchor_id: None,
            });
        }
        let measure_number_metric = canonical_text_metric(TextRole::MeasureNumber);
        if !is_first_system {
            sink.push_text_item(TextItemSpec {
                measure_id: None,
                role: "measure-number",
                x: margin,
                y: sy,
                text_role: TextRole::MeasureNumber,
                text: format!("{}", system.measures[0].measure.global_index + 1),
                font_family: measure_number_metric.font_family,
                font_size_pt: measure_number_metric.font_size_pt,
                fill: "#333",
                text_anchor: None,
                font_weight: None,
            });
        }

        for (mi, (measure, mw)) in system.measures.iter().zip(system.widths.iter()).enumerate() {
            let measure_id = format!("measure-{}", measure.global_index);
            measure_ids.push(measure_id.clone());

            let left_pad = measure_left_pad(mi, is_first_system, measure.barline.as_deref());
            if mi == 0 {
                render_system_opening_barline(&mut sink, Some(&measure_id), mx, s_top, s_bot);
                if is_start_repeat_barline(measure.barline.as_deref()) {
                    render_start_repeat_barline(
                        &mut sink,
                        Some(&measure_id),
                        first_measure_start_repeat_x(mx, is_first_system),
                        s_top,
                        s_bot,
                    );
                }
            } else {
                render_left_barline(
                    &mut sink,
                    Some(&measure_id),
                    mx,
                    s_top,
                    s_bot,
                    measure.barline.as_deref(),
                );
            }

            page.measures.push(SceneMeasure {
                id: measure_id.clone(),
                index: measure.global_index,
                global_index: measure.global_index,
                system_id: system_id.clone(),
                x_pt: mx,
                y_pt: sy,
                width_pt: *mw,
                height_pt: s_bot - sy,
            });

            if let Some(count) = measure.measure.multi_rest_count {
                let center_y = s_top + (s_bot - s_top) * 0.5;
                let pad = (*mw * 0.1).max(8.0);
                let bar_left = mx + pad;
                let bar_right = mx + *mw - pad;
                let bar_thickness = staff_ss * 0.5;
                let serif_height = staff_ss * 2.0;
                let serif_thickness = 2.0;
                let bar_id = sink.push_line_item(LineItemSpec {
                    measure_id: Some(&measure_id),
                    role: "multi-rest-bar",
                    x1: bar_left,
                    y1: center_y,
                    x2: bar_right,
                    y2: center_y,
                    stroke: "#333",
                    stroke_width: bar_thickness,
                    stroke_line_cap: Some("butt"),
                });
                let left_serif_id = sink.push_line_item(LineItemSpec {
                    measure_id: Some(&measure_id),
                    role: "multi-rest-serif",
                    x1: bar_left,
                    y1: center_y - serif_height * 0.5,
                    x2: bar_left,
                    y2: center_y + serif_height * 0.5,
                    stroke: "#333",
                    stroke_width: serif_thickness,
                    stroke_line_cap: Some("butt"),
                });
                let right_serif_id = sink.push_line_item(LineItemSpec {
                    measure_id: Some(&measure_id),
                    role: "multi-rest-serif",
                    x1: bar_right,
                    y1: center_y - serif_height * 0.5,
                    x2: bar_right,
                    y2: center_y + serif_height * 0.5,
                    stroke: "#333",
                    stroke_width: serif_thickness,
                    stroke_line_cap: Some("butt"),
                });
                let count_glyph: String = count
                    .to_string()
                    .chars()
                    .map(|c| char::from_u32(0xE080 + c.to_digit(10).unwrap()).unwrap())
                    .collect();
                let time_sig_metric = canonical_text_metric(TextRole::TimeSignatureDigit);
                let count_y = s_top - staff_ss * 0.5 - time_sig_metric.font_size_pt * 0.5;
                let count_id = sink.push_text_item(TextItemSpec {
                    measure_id: Some(&measure_id),
                    role: "multi-rest-count",
                    x: mx + *mw * 0.5,
                    y: count_y,
                    text_role: TextRole::TimeSignatureDigit,
                    text: count_glyph,
                    font_family: time_sig_metric.font_family,
                    font_size_pt: time_sig_metric.font_size_pt,
                    fill: "#333",
                    text_anchor: Some("middle"),
                    font_weight: None,
                });
                page.composites.push(SceneComposite {
                    id: format!("multi-rest-{}", measure.global_index),
                    kind: CompositeKind::MultiRest,
                    fragment: SpanFragmentKind::SingleSegment,
                    child_item_ids: vec![bar_id, left_serif_id, right_serif_id, count_id],
                    label: None,
                    count: Some(count),
                    start_anchor_id: Some(measure_id.clone()),
                    end_anchor_id: Some(measure_id.clone()),
                });
            } else if let Some(repeat_part) = measure.repeat_part {
                match repeat_part {
                    MeasureRepeatDisplayPart::Single => {
                        let repeat_metric =
                            canonical_glyph_metric(GlyphRole::MeasureRepeatMark1Bar);
                        let repeat_id = sink.push_glyph_item(GlyphItemSpec {
                            measure_id: Some(&measure_id),
                            role: "measure-repeat",
                            x: mx + *mw * 0.5 - repeat_metric.bbox_center_x_pt(30.0),
                            y: s_mid + repeat_metric.bbox_center_y_pt(30.0),
                            glyph_role: GlyphRole::MeasureRepeatMark1Bar,
                            font_family: "Bravura",
                            font_size_pt: 30.0,
                            fill: "#333",
                        });
                        page.composites.push(SceneComposite {
                            id: format!("measure-repeat-{}", measure.global_index),
                            kind: CompositeKind::MeasureRepeat,
                            fragment: SpanFragmentKind::SingleSegment,
                            child_item_ids: vec![repeat_id],
                            label: None,
                            count: Some(1),
                            start_anchor_id: Some(measure_id.clone()),
                            end_anchor_id: Some(measure_id.clone()),
                        });
                    }
                    MeasureRepeatDisplayPart::TwoBarStart => {
                        let next_width = system.widths.get(mi + 1).copied().unwrap_or(*mw);
                        let span_center_x = mx + (*mw + next_width) * 0.5;
                        let repeat_metric =
                            canonical_glyph_metric(GlyphRole::MeasureRepeatMark2Bars);
                        let repeat_id = sink.push_glyph_item(GlyphItemSpec {
                            measure_id: Some(&measure_id),
                            role: "measure-repeat",
                            x: span_center_x - repeat_metric.bbox_center_x_pt(30.0),
                            y: s_mid + repeat_metric.bbox_center_y_pt(30.0),
                            glyph_role: GlyphRole::MeasureRepeatMark2Bars,
                            font_family: "Bravura",
                            font_size_pt: 30.0,
                            fill: "#333",
                        });
                        let end_anchor_id = format!("measure-{}", measure.global_index + 1);
                        page.composites.push(SceneComposite {
                            id: format!("measure-repeat-{}", measure.global_index),
                            kind: CompositeKind::MeasureRepeat,
                            fragment: SpanFragmentKind::SingleSegment,
                            child_item_ids: vec![repeat_id],
                            label: None,
                            count: Some(2),
                            start_anchor_id: Some(measure_id.clone()),
                            end_anchor_id: Some(end_anchor_id),
                        });
                    }
                    MeasureRepeatDisplayPart::TwoBarStop => {}
                }
            } else {
                render_measure_events(
                    &mut sink,
                    RenderMeasureEventsInput {
                        measure_id: &measure_id,
                        header: &score.header,
                        measure: measure.measure,
                        geometry: MeasureGeometryInput {
                            measure_x: mx,
                            measure_width: *mw,
                            left_pad,
                            right_pad: MEASURE_RIGHT_PAD_PT,
                            duration_compression: opts.duration_spacing_compression,
                        },
                        staff_top: s_top,
                        staff_bottom: s_bot,
                        mapper: &mapper,
                        stem_len_pt: opts.stem_len_pt,
                        hide_voice2_rests: opts.hide_voice2_rests,
                    },
                );
            }

            deferred_navs.push(DeferredNavMarker {
                measure_id: measure_id.clone(),
                global_index: measure.global_index,
                start_nav: measure.start_nav.clone(),
                end_nav: measure.end_nav.clone(),
                x: mx,
                width: *mw,
                top: s_top,
            });
            render_right_barline(
                &mut sink,
                RightBarlineSpec {
                    measure_id: Some(&measure_id),
                    x: mx + *mw,
                    top: s_top,
                    bottom: s_bot,
                    barline: measure
                        .closing_barline
                        .as_deref()
                        .or(measure.barline.as_deref()),
                    is_last_measure_of_score: mi + 1 == system.measures.len() && is_last,
                },
            );
            mx += *mw;
        }

        page.systems.push(SceneSystem {
            id: system_id,
            index: sys_idx as u32,
            page_index: 0,
            x_pt: system_left,
            y_pt: sy,
            width_pt: system_right - system_left,
            height_pt: s_bot - sy,
            measure_ids,
        });
    }

    push_volta_composites(
        &mut sink,
        &mut page.composites,
        &page.measures,
        &expanded.measures,
        opts,
    );
    render_hairpin_fragments(
        &mut sink,
        &mut page.composites,
        &page.measures,
        &expanded.measures,
        opts.hairpin_offset_y,
    );
    for nav_spec in &deferred_navs {
        render_nav_markers(&mut sink, &mut page.composites, nav_spec);
    }
    let _ = sink;
    stack_scene_structural_items(&mut page.items, &page.composites, opts.edge_padding);

    paginate_unpaginated_page(
        page,
        LayoutScene {
            version: LAYOUT_SCENE_VERSION.to_string(),
            metrics_version: CANONICAL_METRICS_VERSION.to_string(),
            pages: Vec::new(),
            issues: score.errors.clone(),
        },
        opts,
    )
}

fn paginate_unpaginated_page(
    page: ScenePage,
    mut scene: LayoutScene,
    opts: &LayoutOptions,
) -> LayoutScene {
    let header_box = header_box_from_page(&page);
    let system_boxes = system_boxes_from_page(&page, opts);
    let pagination = paginate_system_boxes(&system_boxes, &header_box, opts);
    let mut pages = (0..=pagination
        .placements
        .iter()
        .map(|placement| placement.page_index)
        .max()
        .unwrap_or(0))
        .map(|index| ScenePage {
            index,
            width_pt: opts.page_width_pt,
            height_pt: opts.page_height_pt,
            systems: Vec::new(),
            measures: Vec::new(),
            items: Vec::new(),
            composites: Vec::new(),
        })
        .collect::<Vec<_>>();

    pages[0].items.extend(header_box.items.clone());
    pages[0].composites.extend(header_box.composites.clone());

    for placement in &pagination.placements {
        let Some(system_box) = system_boxes
            .iter()
            .find(|candidate| candidate.system_id == placement.system_id)
        else {
            continue;
        };
        let page = &mut pages[placement.page_index as usize];
        assemble_placed_system_box(page, system_box, placement);
    }

    scene.pages = pages;
    scene.issues.extend(pagination.issues);
    scene.issues.extend(validate_layout_scene(&scene));
    scene
}

fn validate_layout_scene(scene: &LayoutScene) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let overflow_systems = overflow_systems_by_page(scene);
    for (expected, page) in scene.pages.iter().enumerate() {
        if page.index != expected as u32 {
            diagnostics.push(format!(
                "LAYOUT_ERROR page-order expected={} actual={}",
                expected, page.index
            ));
        }
        let item_ids = page
            .items
            .iter()
            .map(|item| item.id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        let measure_ids = page
            .measures
            .iter()
            .map(|measure| measure.id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        for system in &page.systems {
            if system.page_index != page.index {
                diagnostics.push(format!(
                    "LAYOUT_ERROR system-page system={} page={} actual={}",
                    system.id, page.index, system.page_index
                ));
            }
        }
        for item in &page.items {
            if let Some(anchor_id) = item.anchor_item_id.as_deref() {
                if !item_ids.contains(anchor_id) {
                    diagnostics.push(format!(
                        "LAYOUT_ERROR item-anchor item={} anchor={}",
                        item.id, anchor_id
                    ));
                }
            }
            if let Ok(bounds) = scene_item_bounds(item) {
                if bounds.x < -0.01
                    || bounds.y < -0.01
                    || bounds.x + bounds.width > page.width_pt + 0.01
                    || bounds.y + bounds.height > page.height_pt + 0.01
                {
                    let overflow_item = item_system_id(page, item)
                        .map(|system_id| overflow_systems.contains(&(page.index, system_id)))
                        .unwrap_or(false);
                    if !overflow_item {
                        diagnostics.push(format!("LAYOUT_ERROR item-bounds item={}", item.id));
                    }
                }
            }
        }
        for composite in &page.composites {
            for child_id in &composite.child_item_ids {
                if !item_ids.contains(child_id.as_str()) {
                    diagnostics.push(format!(
                        "LAYOUT_ERROR composite-child composite={} child={}",
                        composite.id, child_id
                    ));
                }
            }
            for anchor_id in [
                composite.start_anchor_id.as_deref(),
                composite.end_anchor_id.as_deref(),
            ]
            .into_iter()
            .flatten()
            {
                if !measure_ids.contains(anchor_id) {
                    diagnostics.push(format!(
                        "LAYOUT_ERROR composite-anchor composite={} anchor={}",
                        composite.id, anchor_id
                    ));
                }
            }
        }
    }

    let mut global_item_ids = std::collections::BTreeSet::new();
    let mut global_composite_ids = std::collections::BTreeSet::new();
    for page in &scene.pages {
        for item in &page.items {
            if !global_item_ids.insert(item.id.as_str()) {
                diagnostics.push(format!("LAYOUT_ERROR duplicate-item id={}", item.id));
            }
        }
        for composite in &page.composites {
            if !global_composite_ids.insert(composite.id.as_str()) {
                diagnostics.push(format!(
                    "LAYOUT_ERROR duplicate-composite id={}",
                    composite.id
                ));
            }
        }
    }
    diagnostics
}

fn overflow_systems_by_page(scene: &LayoutScene) -> std::collections::BTreeSet<(u32, String)> {
    scene
        .issues
        .iter()
        .filter_map(|issue| {
            let mut page_index = None;
            let mut system_id = None;
            if !issue.starts_with("LAYOUT_WARNING overflow ") {
                return None;
            }
            for token in issue.split_whitespace() {
                if let Some(value) = token.strip_prefix("page=") {
                    page_index = value.parse::<u32>().ok();
                } else if let Some(value) = token.strip_prefix("system=") {
                    system_id = Some(value.to_string());
                }
            }
            Some((page_index?, system_id?))
        })
        .collect()
}

fn item_system_id(page: &ScenePage, item: &SceneItem) -> Option<String> {
    if let Some(measure_id) = item.measure_id.as_deref() {
        if let Some(measure) = page
            .measures
            .iter()
            .find(|measure| measure.id == measure_id)
        {
            return Some(measure.system_id.clone());
        }
    }

    page.systems
        .iter()
        .find(|system| item.id.starts_with(&format!("{}-", system.id)))
        .map(|system| system.id.clone())
}

fn header_box_from_page(page: &ScenePage) -> HeaderLayoutBox {
    let items_by_id = page
        .items
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect::<std::collections::HashMap<_, _>>();
    let header_item_ids = page
        .composites
        .iter()
        .filter(|composite| composite.kind == CompositeKind::TextBlock)
        .filter(|composite| {
            composite.child_item_ids.iter().all(|id| {
                items_by_id
                    .get(id.as_str())
                    .is_some_and(|item| item.measure_id.is_none())
            })
        })
        .flat_map(|composite| composite.child_item_ids.iter().cloned())
        .collect::<std::collections::BTreeSet<_>>();
    let items = page
        .items
        .iter()
        .filter(|item| header_item_ids.contains(&item.id))
        .cloned()
        .collect::<Vec<_>>();
    let composites = page
        .composites
        .iter()
        .filter(|composite| composite.kind == CompositeKind::TextBlock)
        .filter(|composite| {
            composite
                .child_item_ids
                .iter()
                .all(|id| header_item_ids.contains(id))
        })
        .cloned()
        .collect::<Vec<_>>();
    let bounds = bounds_for_items(&items).ok().flatten();
    HeaderLayoutBox {
        items,
        composites,
        visual_top: bounds.map(|bounds| bounds.y).unwrap_or(page.height_pt),
        visual_bottom: bounds.map(|bounds| bounds.y + bounds.height).unwrap_or(0.0),
    }
}

fn system_boxes_from_page(page: &ScenePage, opts: &LayoutOptions) -> Vec<SystemLayoutBox> {
    page.systems
        .iter()
        .enumerate()
        .map(|(index, system)| {
            let prev_y = index
                .checked_sub(1)
                .and_then(|prev| page.systems.get(prev))
                .map(|prev| prev.y_pt);
            let next_y = page.systems.get(index + 1).map(|next| next.y_pt);
            system_box_from_page_system(page, system, opts, prev_y, next_y)
        })
        .collect()
}

fn system_box_from_page_system(
    page: &ScenePage,
    system: &SceneSystem,
    opts: &LayoutOptions,
    previous_system_y: Option<f32>,
    next_system_y: Option<f32>,
) -> SystemLayoutBox {
    let measure_ids = system
        .measure_ids
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let measures = page
        .measures
        .iter()
        .filter(|measure| measure.system_id == system.id)
        .map(|measure| {
            let mut local = measure.clone();
            local.x_pt -= opts.left_margin_pt;
            local
        })
        .collect::<Vec<_>>();
    let staff_top = system.y_pt + 10.0;
    let staff_bottom = system.y_pt + system.height_pt;
    let band_top = previous_system_y
        .map(|previous_y| (previous_y + system.y_pt) * 0.5)
        .unwrap_or(system.y_pt - 90.0);
    let band_bottom = next_system_y
        .map(|next_y| (system.y_pt + next_y) * 0.5)
        .unwrap_or(system.y_pt + system.height_pt + 90.0);
    let mut items = page
        .items
        .iter()
        .filter(|item| {
            if let Some(measure_id) = item.measure_id.as_ref() {
                return measure_ids.contains(measure_id);
            }
            if matches!(item.role.as_str(), "title" | "subtitle" | "composer") {
                return false;
            }
            item_bounds(item)
                .map(|(_, y, _, height)| {
                    let center_y = y + height * 0.5;
                    center_y >= band_top && center_y <= band_bottom
                })
                .unwrap_or(false)
        })
        .cloned()
        .collect::<Vec<_>>();
    for item in &mut items {
        translate_scene_item(item, -opts.left_margin_pt, 0.0);
    }
    let item_ids = items
        .iter()
        .map(|item| item.id.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let composites = page
        .composites
        .iter()
        .filter(|composite| {
            let children_match = composite
                .child_item_ids
                .iter()
                .all(|child_id| item_ids.contains(child_id));
            let start_matches = composite
                .start_anchor_id
                .as_ref()
                .is_none_or(|id| measure_ids.contains(id));
            let end_matches = composite
                .end_anchor_id
                .as_ref()
                .is_none_or(|id| measure_ids.contains(id));
            children_match && start_matches && end_matches
        })
        .cloned()
        .collect::<Vec<_>>();
    let bounds = bounds_for_items(&items).ok().flatten();
    let visual_top = bounds.map(|bounds| bounds.y).unwrap_or(system.y_pt);
    let visual_bottom = bounds
        .map(|bounds| bounds.y + bounds.height)
        .unwrap_or(system.y_pt + system.height_pt);
    let mut local_system = system.clone();
    local_system.x_pt = 0.0;

    SystemLayoutBox {
        system_index: system.index,
        system_id: system.id.clone(),
        local_system_origin_y: system.y_pt,
        staff_top,
        staff_bottom,
        visual_top,
        visual_bottom,
        width_pt: system.width_pt,
        measures,
        systems: vec![local_system],
        items,
        composites,
    }
}

fn assemble_placed_system_box(
    page: &mut ScenePage,
    system_box: &SystemLayoutBox,
    placement: &PlacedSystemBox,
) {
    let dx = placement.page_x;
    let dy = placement.page_y - placement.local_visual_top;
    let item_remap = system_box
        .items
        .iter()
        .map(|item| {
            (
                item.id.clone(),
                format!("system-{}-{}", system_box.system_index, item.id),
            )
        })
        .collect::<std::collections::HashMap<_, _>>();

    for system in &system_box.systems {
        let mut final_system = system.clone();
        final_system.page_index = placement.page_index;
        final_system.x_pt += dx;
        final_system.y_pt += dy;
        page.systems.push(final_system);
    }
    for measure in &system_box.measures {
        let mut final_measure = measure.clone();
        final_measure.x_pt += dx;
        final_measure.y_pt += dy;
        page.measures.push(final_measure);
    }
    for item in &system_box.items {
        let mut final_item = item.clone();
        final_item.id = item_remap
            .get(&item.id)
            .cloned()
            .unwrap_or_else(|| item.id.clone());
        if let Some(anchor) = final_item.anchor_item_id.clone() {
            final_item.anchor_item_id = item_remap.get(&anchor).cloned();
        }
        translate_scene_item(&mut final_item, dx, dy);
        page.items.push(final_item);
    }
    for composite in &system_box.composites {
        let mut final_composite = composite.clone();
        final_composite.id = format!("system-{}-{}", system_box.system_index, final_composite.id);
        final_composite.child_item_ids = final_composite
            .child_item_ids
            .iter()
            .filter_map(|id| item_remap.get(id).cloned())
            .collect();
        page.composites.push(final_composite);
    }
}

fn translate_scene_item(item: &mut SceneItem, dx: f32, dy: f32) {
    match &mut item.primitive {
        ScenePrimitive::TextRun(text) => {
            text.x_pt += dx;
            text.y_pt += dy;
        }
        ScenePrimitive::LineSegment(line) => {
            line.x1_pt += dx;
            line.y1_pt += dy;
            line.x2_pt += dx;
            line.y2_pt += dy;
        }
        ScenePrimitive::Rect(rect) => {
            rect.x_pt += dx;
            rect.y_pt += dy;
        }
        ScenePrimitive::Polyline(polyline) => {
            for (x, y) in &mut polyline.points_pt {
                *x += dx;
                *y += dy;
            }
        }
        ScenePrimitive::Path(path) => translate_path(&mut path.d, dx, dy),
        ScenePrimitive::GlyphRun(glyph) => {
            glyph.x_pt += dx;
            glyph.y_pt += dy;
        }
    }
}

fn translate_path(d: &mut String, dx: f32, dy: f32) {
    let tokens = d.split_whitespace().collect::<Vec<_>>();
    if tokens.is_empty() {
        return;
    }
    let mut translated = Vec::with_capacity(tokens.len());
    let mut coordinate_index = 0usize;
    for token in tokens {
        if let Ok(value) = token.parse::<f32>() {
            let adjusted = if coordinate_index.is_multiple_of(2) {
                value + dx
            } else {
                value + dy
            };
            translated.push(format!("{adjusted:.3}"));
            coordinate_index += 1;
        } else {
            translated.push(token.to_string());
        }
    }
    *d = translated.join(" ");
}

fn _layout_scene_from_page(page: ScenePage, issues: Vec<String>) -> LayoutScene {
    LayoutScene {
        version: LAYOUT_SCENE_VERSION.to_string(),
        metrics_version: CANONICAL_METRICS_VERSION.to_string(),
        pages: vec![page],
        issues,
    }
}

fn push_volta_composites(
    sink: &mut SceneEmitSink<'_>,
    composites: &mut Vec<SceneComposite>,
    page_measures: &[SceneMeasure],
    measures: &[DisplayMeasure<'_>],
    opts: &LayoutOptions,
) {
    let mut system_start = 0usize;
    while system_start < page_measures.len() {
        let system_id = page_measures[system_start].system_id.clone();
        let mut system_end = system_start;
        while system_end + 1 < page_measures.len()
            && page_measures[system_end + 1].system_id == system_id
        {
            system_end += 1;
        }

        push_system_volta_composites(
            sink,
            composites,
            &page_measures[system_start..=system_end],
            measures,
            opts,
            system_id == "system-0",
        );

        system_start = system_end + 1;
    }
}

fn push_system_volta_composites(
    sink: &mut SceneEmitSink<'_>,
    composites: &mut Vec<SceneComposite>,
    system_measures: &[SceneMeasure],
    measures: &[DisplayMeasure<'_>],
    opts: &LayoutOptions,
    is_first_system: bool,
) {
    let mut block_start = 0usize;
    while block_start < system_measures.len() {
        if display_measure_for_scene(measures, &system_measures[block_start])
            .and_then(|measure| measure.measure.volta_indices.as_ref())
            .is_none()
        {
            block_start += 1;
            continue;
        }

        let mut block_end = block_start;
        while block_end + 1 < system_measures.len()
            && display_measure_for_scene(measures, &system_measures[block_end + 1])
                .and_then(|measure| measure.measure.volta_indices.as_ref())
                .is_some()
        {
            block_end += 1;
        }

        let block_x1 = volta_segment_left_x(
            &system_measures[block_start],
            display_measure_for_scene(measures, &system_measures[block_start]),
            block_start == 0,
            is_first_system,
        );
        let block_x2 = system_measures[block_end].x_pt + system_measures[block_end].width_pt;
        let occupied_top = top_skyline_sample(
            sink.items,
            &system_measures[block_start..=block_end],
            block_x1,
            block_x2,
            system_measures[block_start].y_pt - 60.0,
        );
        struct PendingVoltaRun {
            start: usize,
            end: usize,
            label: Vec<u32>,
            show_left_hook: bool,
            show_label: bool,
            show_right: bool,
            fragment: SpanFragmentKind,
            line_y: f32,
        }
        let mut runs = Vec::new();
        let mut index = block_start;
        while index <= block_end {
            let Some(display_measure) =
                display_measure_for_scene(measures, &system_measures[index])
            else {
                index += 1;
                continue;
            };
            let Some(label) = display_measure.measure.volta_indices.as_ref() else {
                index += 1;
                continue;
            };

            let mut end = index;
            while end < block_end
                && display_measure_for_scene(measures, &system_measures[end + 1])
                    .and_then(|measure| measure.measure.volta_indices.as_ref())
                    == Some(label)
            {
                end += 1;
            }

            let start_measure = display_measure.global_index;
            let end_measure = display_measure_for_scene(measures, &system_measures[end])
                .map(|measure| measure.global_index)
                .unwrap_or(start_measure);
            let start_type = volta_type_for_measure(measures, start_measure);
            let end_type = volta_type_for_measure(measures, end_measure);
            let show_label = matches!(
                start_type,
                VoltaSegmentType::Begin | VoltaSegmentType::BeginEnd
            );
            let show_left_hook = show_label || index == 0;
            let show_right = matches!(end_type, VoltaSegmentType::End | VoltaSegmentType::BeginEnd);
            let fragment = volta_fragment_kind(show_label, show_right);
            let line_y = volta_line_y_for_segment(
                sink.items,
                &system_measures[index..=end],
                measures,
                label,
                occupied_top,
                opts.volta_offset_y,
                show_left_hook,
                show_label,
                show_right,
                index == 0,
                is_first_system,
            );
            runs.push(PendingVoltaRun {
                start: index,
                end,
                label: label.clone(),
                show_left_hook,
                show_label,
                show_right,
                fragment,
                line_y,
            });

            index = end + 1;
        }
        let block_line_y = runs
            .iter()
            .map(|run| run.line_y)
            .fold(f32::INFINITY, f32::min);
        for run in &runs {
            push_volta_segment(
                sink,
                composites,
                VoltaSegmentSpec {
                    segment_measures: &system_measures[run.start..=run.end],
                    measures,
                    label: &run.label,
                    line_y: block_line_y,
                    show_left_hook: run.show_left_hook,
                    show_label: run.show_label,
                    show_right: run.show_right,
                    fragment: run.fragment,
                    starts_at_system_left: run.start == 0,
                    is_first_system,
                },
            );
        }

        block_start = block_end + 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn volta_line_y_for_segment(
    items: &[SceneItem],
    segment_measures: &[SceneMeasure],
    measures: &[DisplayMeasure<'_>],
    label: &[u32],
    occupied_top: f32,
    volta_offset_y: f32,
    show_left_hook: bool,
    show_label: bool,
    show_right: bool,
    starts_at_system_left: bool,
    is_first_system: bool,
) -> f32 {
    let first = segment_measures
        .first()
        .expect("volta segment has measures");
    let last = segment_measures.last().expect("volta segment has measures");
    let first_display = display_measure_for_scene(measures, first);
    let x1 = volta_segment_left_x(first, first_display, starts_at_system_left, is_first_system);
    let x2 = last.x_pt + last.width_pt;
    let mut line_y = occupied_top - VOLTA_SKYLINE_GAP_PT - VOLTA_LINE_THICKNESS_PT;

    if show_left_hook {
        line_y = line_y.min(volta_line_y_for_child(
            items,
            segment_measures,
            x1 - VOLTA_LINE_THICKNESS_PT,
            x1 + VOLTA_LINE_THICKNESS_PT,
            VOLTA_LINE_HEIGHT_PT,
        ));
    }
    if show_right {
        line_y = line_y.min(volta_line_y_for_child(
            items,
            segment_measures,
            x2 - VOLTA_LINE_THICKNESS_PT,
            x2 + VOLTA_LINE_THICKNESS_PT,
            VOLTA_LINE_HEIGHT_PT,
        ));
    }
    if show_label {
        let label_text = format!(
            "{}.",
            label
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );
        let label_x = x1 + 5.0;
        let label_width = canonical_text_width(TextRole::CountLabel, &label_text);
        let count_metric = canonical_text_metric(TextRole::CountLabel);
        let label_bottom_extent = VOLTA_TEXT_SIZE_PT + 2.0 + count_metric.descent_pt;
        line_y = line_y.min(volta_line_y_for_child(
            items,
            segment_measures,
            label_x,
            label_x + label_width,
            label_bottom_extent,
        ));
    }

    line_y - volta_offset_y
}

fn volta_line_y_for_child(
    items: &[SceneItem],
    segment_measures: &[SceneMeasure],
    x1: f32,
    x2: f32,
    child_bottom_extent: f32,
) -> f32 {
    top_skyline_sample_optional(items, segment_measures, x1, x2)
        .map(|top| top - VOLTA_SKYLINE_GAP_PT - child_bottom_extent)
        .unwrap_or(f32::INFINITY)
}

struct VoltaSegmentSpec<'a> {
    segment_measures: &'a [SceneMeasure],
    measures: &'a [DisplayMeasure<'a>],
    label: &'a [u32],
    line_y: f32,
    show_left_hook: bool,
    show_label: bool,
    show_right: bool,
    fragment: SpanFragmentKind,
    starts_at_system_left: bool,
    is_first_system: bool,
}

fn push_volta_segment(
    sink: &mut SceneEmitSink<'_>,
    composites: &mut Vec<SceneComposite>,
    spec: VoltaSegmentSpec<'_>,
) {
    if spec.segment_measures.is_empty() {
        return;
    }
    let first = spec.segment_measures.first().unwrap();
    let last = spec.segment_measures.last().unwrap();
    let label_text = format!(
        "{}.",
        spec.label
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    let first_display = display_measure_for_scene(spec.measures, first);
    let x1 = volta_segment_left_x(
        first,
        first_display,
        spec.starts_at_system_left,
        spec.is_first_system,
    );
    let x2 = last.x_pt + last.width_pt;

    let mut child_item_ids = Vec::new();
    child_item_ids.push(sink.push_line_item(LineItemSpec {
        measure_id: Some(&first.id),
        role: "volta-line",
        x1,
        y1: spec.line_y,
        x2,
        y2: spec.line_y,
        stroke: "#333",
        stroke_width: 1.0,
        stroke_line_cap: None,
    }));
    if spec.show_left_hook {
        child_item_ids.push(sink.push_line_item(LineItemSpec {
            measure_id: Some(&first.id),
            role: "volta-start-hook",
            x1,
            y1: spec.line_y,
            x2: x1,
            y2: spec.line_y + VOLTA_LINE_HEIGHT_PT,
            stroke: "#333",
            stroke_width: 1.0,
            stroke_line_cap: None,
        }));
    }
    if spec.show_label {
        child_item_ids.push(sink.push_text_item(TextItemSpec {
            measure_id: Some(&first.id),
            role: "volta-label",
            x: x1 + 5.0,
            y: spec.line_y + VOLTA_TEXT_SIZE_PT + 2.0,
            text_role: TextRole::CountLabel,
            text: label_text.clone(),
            font_family: "Academico",
            font_size_pt: VOLTA_TEXT_SIZE_PT,
            fill: "#333",
            text_anchor: None,
            font_weight: None,
        }));
    }
    if spec.show_right {
        child_item_ids.push(sink.push_line_item(LineItemSpec {
            measure_id: Some(&last.id),
            role: "volta-end-hook",
            x1: x2,
            y1: spec.line_y,
            x2,
            y2: spec.line_y + VOLTA_LINE_HEIGHT_PT,
            stroke: "#333",
            stroke_width: 1.0,
            stroke_line_cap: None,
        }));
    }
    composites.push(SceneComposite {
        id: format!("volta-{}-{}", first.id, last.id),
        kind: CompositeKind::Volta,
        fragment: spec.fragment,
        child_item_ids,
        label: Some(label_text),
        count: None,
        start_anchor_id: Some(first.id.clone()),
        end_anchor_id: Some(last.id.clone()),
    });
}

fn volta_segment_left_x(
    first: &SceneMeasure,
    first_display: Option<&DisplayMeasure<'_>>,
    starts_at_system_left: bool,
    is_first_system: bool,
) -> f32 {
    if starts_at_system_left {
        let barline = first_display.and_then(|measure| measure.barline.as_deref());
        first.x_pt + measure_left_pad(0, is_first_system, barline)
    } else {
        first.x_pt
    }
}

fn top_skyline_sample(
    items: &[SceneItem],
    block_measures: &[SceneMeasure],
    x1: f32,
    x2: f32,
    fallback_top: f32,
) -> f32 {
    top_skyline_sample_optional(items, block_measures, x1, x2).unwrap_or(fallback_top)
}

fn top_skyline_sample_optional(
    items: &[SceneItem],
    block_measures: &[SceneMeasure],
    x1: f32,
    x2: f32,
) -> Option<f32> {
    let left = x1.min(x2);
    let right = x1.max(x2);
    let measure_ids = block_measures
        .iter()
        .map(|measure| measure.id.as_str())
        .collect::<std::collections::HashSet<_>>();
    let system_top = block_measures
        .iter()
        .map(|measure| measure.y_pt)
        .fold(f32::INFINITY, f32::min);
    let system_bottom = block_measures
        .iter()
        .map(|measure| measure.y_pt + measure.height_pt)
        .fold(f32::NEG_INFINITY, f32::max);
    let mut top = f32::INFINITY;
    for item in items {
        if is_decoration_role(&item.role) {
            continue;
        }
        if item.role.starts_with("volta") {
            continue;
        }
        let in_block_measure = item
            .measure_id
            .as_deref()
            .is_some_and(|measure_id| measure_ids.contains(measure_id));
        if let Some((item_x, item_y, item_width, _)) = item_bounds(item) {
            let in_system_band = item.measure_id.is_none()
                && item_y >= system_top - 60.0
                && item_y <= system_bottom + 20.0;
            if !in_block_measure && !in_system_band {
                continue;
            }
            let item_right = item_x + item_width;
            if item_x < right && item_right > left {
                top = top.min(item_y);
            }
        }
    }
    if top.is_finite() {
        Some(top)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VoltaSegmentType {
    Begin,
    Mid,
    End,
    BeginEnd,
}

fn display_measure_for_scene<'a>(
    measures: &'a [DisplayMeasure<'_>],
    scene_measure: &SceneMeasure,
) -> Option<&'a DisplayMeasure<'a>> {
    measures
        .iter()
        .find(|measure| measure.global_index == scene_measure.global_index)
}

fn volta_key(measure: &DisplayMeasure<'_>) -> Option<String> {
    measure.measure.volta_indices.as_ref().map(|indices| {
        indices
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",")
    })
}

fn volta_type_for_measure(measures: &[DisplayMeasure<'_>], global_index: u32) -> VoltaSegmentType {
    let current = measures
        .iter()
        .find(|measure| measure.global_index == global_index);
    let current_key = current.and_then(volta_key);
    let previous_key = global_index
        .checked_sub(1)
        .and_then(|previous| {
            measures
                .iter()
                .find(|measure| measure.global_index == previous)
        })
        .and_then(volta_key);
    let next_key = measures
        .iter()
        .find(|measure| measure.global_index == global_index + 1)
        .and_then(volta_key);
    let begins = current_key != previous_key;
    let ends = current_key != next_key;

    match (begins, ends) {
        (true, true) => VoltaSegmentType::BeginEnd,
        (true, false) => VoltaSegmentType::Begin,
        (false, true) => VoltaSegmentType::End,
        (false, false) => VoltaSegmentType::Mid,
    }
}

fn volta_fragment_kind(show_left: bool, show_right: bool) -> SpanFragmentKind {
    match (show_left, show_right) {
        (true, true) => SpanFragmentKind::SingleSegment,
        (true, false) => SpanFragmentKind::Start,
        (false, true) => SpanFragmentKind::End,
        (false, false) => SpanFragmentKind::Continuation,
    }
}

fn measure_fragments_for_range(
    page_measures: &[SceneMeasure],
    start_measure: u32,
    end_measure: u32,
) -> Vec<Vec<&SceneMeasure>> {
    let mut matches: Vec<&SceneMeasure> = page_measures
        .iter()
        .filter(|measure| {
            measure.global_index >= start_measure && measure.global_index <= end_measure
        })
        .collect();
    matches.sort_by_key(|measure| measure.global_index);

    let mut fragments: Vec<Vec<&SceneMeasure>> = Vec::new();
    for measure in matches {
        if fragments
            .last()
            .map(|fragment| {
                fragment
                    .last()
                    .map(|last| last.system_id == measure.system_id)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
        {
            fragments.last_mut().unwrap().push(measure);
        } else {
            fragments.push(vec![measure]);
        }
    }
    fragments
}

fn canonical_text_width(role: TextRole, text: &str) -> f32 {
    let metric = canonical_text_metric(role);
    metric.average_advance_pt * text.chars().count() as f32
}

fn span_fragment_kind(index: usize, total: usize) -> SpanFragmentKind {
    if total <= 1 {
        SpanFragmentKind::SingleSegment
    } else if index == 0 {
        SpanFragmentKind::Start
    } else if index + 1 == total {
        SpanFragmentKind::End
    } else {
        SpanFragmentKind::Continuation
    }
}

struct RenderMeasureEventsInput<'a> {
    measure_id: &'a str,
    header: &'a RenderHeader,
    measure: &'a RenderMeasure,
    geometry: MeasureGeometryInput,
    staff_top: f32,
    staff_bottom: f32,
    mapper: &'a SlotMapper,
    stem_len_pt: f32,
    hide_voice2_rests: bool,
}

fn render_measure_events(sink: &mut SceneEmitSink<'_>, input: RenderMeasureEventsInput<'_>) {
    let mut beam_anchors: Vec<BeamAnchor> = Vec::new();
    let geometry = measure_geometry(input.header, input.measure, input.mapper, &input.geometry);
    let mut slot_events = input
        .measure
        .events
        .iter()
        .map(|event| SlotEvent {
            slot: fraction_to_measure_slot(
                event.start,
                input.header.time_beats,
                input.header.time_beat_unit,
                input.header.divisions,
            ),
            event_x: geometry.x_for_fraction(input.header, event.start),
            event,
        })
        .collect::<Vec<_>>();
    slot_events.sort_by(|a, b| {
        a.slot
            .cmp(&b.slot)
            .then_with(|| a.event.voice.cmp(&b.event.voice))
            .then_with(|| {
                staff_y_for_track(&a.event.track)
                    .partial_cmp(&staff_y_for_track(&b.event.track))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    let mut index = 0usize;
    let mut beam_states_by_voice: std::collections::BTreeMap<u8, BeamRunState> =
        std::collections::BTreeMap::new();
    let mut next_beam_group = 0_u32;
    while index < slot_events.len() {
        let slot = slot_events[index].slot;
        let event_x = slot_events[index].event_x;
        let slot_start = index;
        while index < slot_events.len() && slot_events[index].slot == slot {
            index += 1;
        }
        let slot_group = &slot_events[slot_start..index];
        let beam_groups_by_voice = beam_groups_for_slot(
            input.header,
            slot,
            slot_group,
            &mut beam_states_by_voice,
            &mut next_beam_group,
        );
        render_slot_group(
            sink,
            RenderSlotGroupInput {
                measure_id: input.measure_id,
                slot_group,
                beam_groups_by_voice: &beam_groups_by_voice,
                event_x,
                staff_top: input.staff_top,
                beam_anchors: &mut beam_anchors,
                stem_len_pt: input.stem_len_pt,
                hide_voice2_rests: input.hide_voice2_rests,
            },
        );
    }

    render_beam_groups(
        sink,
        input.measure_id,
        beam_anchors,
        input.geometry.measure_width,
        input.staff_bottom,
    );
}

struct RenderSlotGroupInput<'a, 'b> {
    measure_id: &'a str,
    slot_group: &'a [SlotEvent<'a>],
    beam_groups_by_voice: &'a std::collections::BTreeMap<u8, u32>,
    event_x: f32,
    staff_top: f32,
    beam_anchors: &'b mut Vec<BeamAnchor>,
    stem_len_pt: f32,
    hide_voice2_rests: bool,
}

fn render_slot_group(sink: &mut SceneEmitSink<'_>, input: RenderSlotGroupInput<'_, '_>) {
    let hit_voice_count = input
        .slot_group
        .iter()
        .filter(|slot_event| matches!(slot_event.event.kind, EventKind::Hit))
        .map(|slot_event| slot_event.event.voice)
        .collect::<std::collections::BTreeSet<_>>()
        .len();

    let mut note_anchors_by_voice: std::collections::BTreeMap<u8, Vec<NotePlacement>> =
        std::collections::BTreeMap::new();

    for voice in input
        .slot_group
        .iter()
        .map(|slot_event| slot_event.event.voice)
        .collect::<std::collections::BTreeSet<_>>()
    {
        let voice_hits = input
            .slot_group
            .iter()
            .filter(|slot_event| {
                slot_event.event.voice == voice && matches!(slot_event.event.kind, EventKind::Hit)
            })
            .collect::<Vec<_>>();
        if !voice_hits.is_empty() {
            let voice_shift = if hit_voice_count > 1 {
                if voice == 1 {
                    -4.0
                } else {
                    4.0
                }
            } else {
                0.0
            };
            let placements = render_hit_cluster(
                sink,
                RenderHitClusterInput {
                    measure_id: input.measure_id,
                    event_x: input.event_x,
                    voice_shift,
                    staff_top: input.staff_top,
                    voice_hits: &voice_hits,
                    beam_group: input.beam_groups_by_voice.get(&voice).copied(),
                    beam_anchors: input.beam_anchors,
                    stem_len_pt: input.stem_len_pt,
                },
            );
            note_anchors_by_voice.insert(voice, placements);
        }

        for rest in input.slot_group.iter().filter(|slot_event| {
            slot_event.event.voice == voice && matches!(slot_event.event.kind, EventKind::Rest)
        }) {
            if input.hide_voice2_rests && rest.event.voice == 2 {
                continue;
            }
            let rest_metric = rest_glyph_for_fraction(rest.event.duration);
            let rest_role = rest_metric.role;
            let rest_font_size = BASE_FONT_SIZE_PT;
            let rest_y = if rest.event.voice == 2 {
                input.staff_top + 30.0
            } else {
                input.staff_top + 20.0
            };
            sink.push_glyph_item(GlyphItemSpec {
                measure_id: Some(input.measure_id),
                role: "rest",
                x: input.event_x,
                y: rest_y,
                glyph_role: rest_role,
                font_family: "Bravura",
                font_size_pt: rest_font_size,
                fill: "#333",
            });
        }
    }

    let default_anchor = note_anchors_by_voice.values().find_map(|placements| {
        placements
            .first()
            .map(|placement| placement.note_id.clone())
    });
    let default_anchor_y = note_anchors_by_voice
        .values()
        .flat_map(|placements| placements.iter().map(|placement| placement.note_y))
        .fold(None, |acc: Option<f32>, y| {
            Some(acc.map_or(y, |current| current.min(y)))
        });

    let sticking_metric = canonical_text_metric(TextRole::Sticking);
    for sticking in input
        .slot_group
        .iter()
        .filter(|slot_event| matches!(slot_event.event.kind, EventKind::Sticking))
    {
        sink.push_text_item(TextItemSpec {
            measure_id: Some(input.measure_id),
            role: "sticking",
            x: input.event_x,
            y: input.staff_top - sticking_metric.descent_pt,
            text_role: TextRole::Sticking,
            text: sticking.event.glyph.clone(),
            font_family: sticking_metric.font_family,
            font_size_pt: sticking_metric.font_size_pt,
            fill: "#333",
            text_anchor: Some("middle"),
            font_weight: Some("bold"),
        });
        if let Some(item) = sink.last_item_mut() {
            item.anchor_item_id = default_anchor.clone();
        }
        if let Some(anchor_y) = default_anchor_y {
            if let Some(ScenePrimitive::TextRun(text)) =
                sink.last_item_mut().map(|item| &mut item.primitive)
            {
                text.y_pt = anchor_y - sticking_metric.line_height_pt - 4.0;
            }
        }
    }
}

fn beam_groups_for_slot(
    header: &RenderHeader,
    slot: u32,
    slot_group: &[SlotEvent<'_>],
    states_by_voice: &mut std::collections::BTreeMap<u8, BeamRunState>,
    next_group: &mut u32,
) -> std::collections::BTreeMap<u8, u32> {
    let mut result = std::collections::BTreeMap::new();
    let voices = slot_group
        .iter()
        .map(|slot_event| slot_event.event.voice)
        .collect::<std::collections::BTreeSet<_>>();

    for voice in voices {
        let voice_events = slot_group
            .iter()
            .filter(|slot_event| slot_event.event.voice == voice)
            .collect::<Vec<_>>();
        let has_rest = voice_events
            .iter()
            .any(|slot_event| matches!(slot_event.event.kind, EventKind::Rest));
        let beamable_hit = voice_events
            .iter()
            .filter(|slot_event| matches!(slot_event.event.kind, EventKind::Hit))
            .find(|slot_event| is_beamable_duration(slot_event.event.duration));

        if has_rest || beamable_hit.is_none() {
            states_by_voice.remove(&voice);
            continue;
        }

        let segment = grouping_segment_index_for_slot(header, slot);
        let group = match states_by_voice.get(&voice).copied() {
            Some(state) if state.segment == segment => state.group,
            _ => {
                let group = *next_group;
                *next_group += 1;
                group
            }
        };
        states_by_voice.insert(voice, BeamRunState { segment, group });
        result.insert(voice, group);
    }

    result
}

struct RenderHitClusterInput<'a, 'b> {
    measure_id: &'a str,
    event_x: f32,
    voice_shift: f32,
    staff_top: f32,
    voice_hits: &'a [&'a SlotEvent<'a>],
    beam_group: Option<u32>,
    beam_anchors: &'b mut Vec<BeamAnchor>,
    stem_len_pt: f32,
}

fn render_hit_cluster(
    sink: &mut SceneEmitSink<'_>,
    input: RenderHitClusterInput<'_, '_>,
) -> Vec<NotePlacement> {
    let note_font_size = 30.0_f32;
    let stem_up = input
        .voice_hits
        .first()
        .map(|slot_event| slot_event.event.voice != 2)
        .unwrap_or(true);
    let base_note_x = input.event_x - 7.0 + input.voice_shift;
    let mut placements = input
        .voice_hits
        .iter()
        .map(|slot_event| {
            let track_ss = staff_y_for_track(&slot_event.event.track);
            let note_y = track_ss * 10.0;
            (*slot_event, note_y)
        })
        .collect::<Vec<_>>();
    placements.sort_by(|(_, ay), (_, by)| ay.partial_cmp(by).unwrap_or(std::cmp::Ordering::Equal));

    let mut note_placements = Vec::new();
    for (slot_event, note_y_offset) in &placements {
        let glyph_metric = notehead_glyph(
            &slot_event.event.track,
            &slot_event.event.modifiers,
            &slot_event.event.glyph,
        );
        let note_glyph = char::from_u32(glyph_metric.smufl_codepoint)
            .unwrap_or('?')
            .to_string();
        let actual_note_y = input.staff_top + *note_y_offset;
        let note_id = sink.push_text_item(TextItemSpec {
            measure_id: Some(input.measure_id),
            role: "notehead",
            x: base_note_x,
            y: actual_note_y,
            text_role: TextRole::Tempo,
            text: note_glyph,
            font_family: "Bravura",
            font_size_pt: note_font_size,
            fill: "#333",
            text_anchor: None,
            font_weight: None,
        });
        let ledger_half_overhang_pt = 3.0_f32;
        for ledger_y_offset in ledger_line_offsets_for_staff_position(*note_y_offset / 10.0) {
            let ledger_y = input.staff_top + ledger_y_offset * 10.0;
            sink.push_line_item(LineItemSpec {
                measure_id: Some(input.measure_id),
                role: "ledger-line",
                x1: base_note_x - ledger_half_overhang_pt,
                y1: ledger_y,
                x2: base_note_x
                    + canonical_glyph_metric(glyph_role_for_codepoint(
                        glyph_metric.smufl_codepoint,
                    ))
                    .width_ss()
                        * note_font_size
                        / 4.0
                    + ledger_half_overhang_pt,
                y2: ledger_y,
                stroke: "#333",
                stroke_width: 1.25,
                stroke_line_cap: None,
            });
            if let Some(item) = sink.last_item_mut() {
                item.anchor_item_id = Some(note_id.clone());
            }
        }
        let note_role = glyph_role_for_codepoint(glyph_metric.smufl_codepoint);
        let note_center_x = base_note_x + rendered_glyph_width(note_role, note_font_size) * 0.5;
        let has_accent = slot_event
            .event
            .modifiers
            .iter()
            .any(|modifier| modifier == "accent");
        note_placements.push(NotePlacement {
            note_id: note_id.clone(),
            note_x: base_note_x,
            note_y: actual_note_y,
            note_center_x,
            has_accent,
            stem_up_anchor_ss: glyph_metric.stem_up_anchor_ss,
            stem_down_anchor_ss: glyph_metric.stem_down_anchor_ss,
        });
    }

    let mut accent_reference_y = None;
    if let Some(first_hit) = input.voice_hits.first() {
        let needs_stem =
            first_hit.event.duration.denominator >= 4 || first_hit.event.tuplet.is_some();
        if needs_stem {
            let smufl_ss = note_font_size / 4.0;
            let attach_note = if stem_up {
                note_placements.iter().min_by(|a, b| {
                    a.note_y
                        .partial_cmp(&b.note_y)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            } else {
                note_placements.iter().max_by(|a, b| {
                    a.note_y
                        .partial_cmp(&b.note_y)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            };
            if let Some(attach_note) = attach_note {
                let fallback_anchor = if stem_up {
                    GlyphPoint {
                        x_ss: 1.18,
                        y_ss: 0.168,
                    }
                } else {
                    GlyphPoint {
                        x_ss: 0.0,
                        y_ss: -0.168,
                    }
                };
                let stem_anchor = if stem_up {
                    attach_note.stem_up_anchor_ss
                } else {
                    attach_note.stem_down_anchor_ss
                }
                .unwrap_or(fallback_anchor);
                let stem_attach_x = attach_note.note_x + stem_anchor.x_ss * smufl_ss;
                let stem_attach_y = attach_note.note_y - stem_anchor.y_ss * smufl_ss;
                let stem_x = stem_attach_x;
                let stem_y1 = if stem_up {
                    stem_attach_y - input.stem_len_pt
                } else {
                    stem_attach_y
                };
                let stem_y2 = if stem_up {
                    stem_attach_y
                } else {
                    stem_attach_y + input.stem_len_pt
                };
                accent_reference_y = Some(if stem_up { stem_y1 } else { stem_y2 });
                let stem_id = sink.push_line_item(LineItemSpec {
                    measure_id: Some(input.measure_id),
                    role: "stem",
                    x1: stem_x,
                    y1: stem_y1,
                    x2: stem_x,
                    y2: stem_y2,
                    stroke: "#333",
                    stroke_width: 1.5,
                    stroke_line_cap: None,
                });
                if let Some(item) = sink.last_item_mut() {
                    item.anchor_item_id = Some(attach_note.note_id.clone());
                }
                let beam_level = if first_hit.event.duration.denominator >= 32 {
                    3
                } else if first_hit.event.duration.denominator >= 16 {
                    2
                } else if first_hit.event.duration.denominator >= 8 {
                    1
                } else {
                    0
                };
                if let Some(group) = input.beam_group.filter(|_| beam_level > 0) {
                    input.beam_anchors.push(BeamAnchor {
                        x: input.event_x,
                        stem_x,
                        stem_tip_y: if stem_up { stem_y1 } else { stem_y2 },
                        voice: first_hit.event.voice,
                        group,
                        level: beam_level,
                        up: stem_up,
                        stem_item_id: stem_id,
                    });
                }
            }
        }
    }

    let fallback_accent_y = if stem_up {
        note_placements
            .iter()
            .map(|placement| placement.note_y)
            .fold(f32::INFINITY, f32::min)
            - 18.0
    } else {
        note_placements
            .iter()
            .map(|placement| placement.note_y)
            .fold(f32::NEG_INFINITY, f32::max)
            + 18.0
    };
    render_accent_glyphs(
        sink,
        input.measure_id,
        &note_placements,
        stem_up,
        accent_reference_y.unwrap_or(fallback_accent_y),
    );

    note_placements
}

fn render_accent_glyphs(
    sink: &mut SceneEmitSink<'_>,
    measure_id: &str,
    note_placements: &[NotePlacement],
    stem_up: bool,
    reference_y: f32,
) {
    let accent_role = if stem_up {
        GlyphRole::ArticAccentAbove
    } else {
        GlyphRole::ArticAccentBelow
    };
    let accent_font_size = BASE_FONT_SIZE_PT;
    let accent_gap = 4.0_f32;
    let accent_width = rendered_glyph_width(accent_role, accent_font_size);
    let accent_y = if stem_up {
        reference_y - accent_gap
    } else {
        reference_y + accent_gap
    };

    for placement in note_placements
        .iter()
        .filter(|placement| placement.has_accent)
    {
        sink.push_glyph_item(GlyphItemSpec {
            measure_id: Some(measure_id),
            role: "accent",
            x: placement.note_center_x - accent_width * 0.5,
            y: accent_y,
            glyph_role: accent_role,
            font_family: "Bravura",
            font_size_pt: accent_font_size,
            fill: "#333",
        });
        if let Some(item) = sink.last_item_mut() {
            item.anchor_item_id = Some(placement.note_id.clone());
        }
    }
}

fn glyph_role_for_codepoint(codepoint: u32) -> GlyphRole {
    match codepoint {
        0xE0A9 => GlyphRole::NoteheadX,
        0xE0B2 => GlyphRole::NoteheadDiamond,
        0xE0B3 => GlyphRole::NoteheadCircleX,
        0xE0CE => GlyphRole::NoteheadRim,
        _ => GlyphRole::NoteheadBlack,
    }
}

fn ledger_line_offsets_for_staff_position(track_ss: f32) -> Vec<f32> {
    let mut lines = Vec::new();
    if track_ss <= -1.0 {
        let mut line_ss = -1.0_f32;
        while line_ss >= track_ss.ceil() {
            lines.push(line_ss);
            line_ss -= 1.0;
        }
    } else if track_ss >= 5.0 {
        let mut line_ss = 5.0_f32;
        while line_ss <= track_ss.floor() {
            lines.push(line_ss);
            line_ss += 1.0;
        }
    }
    lines
}

fn adjust_stem_tip(items: &mut [SceneItem], stem_id: &str, target_y: f32, stem_up: bool) {
    for item in items.iter_mut() {
        if item.id == stem_id {
            if let ScenePrimitive::LineSegment(ref mut line) = &mut item.primitive {
                if stem_up {
                    line.y1_pt = target_y;
                } else {
                    line.y2_pt = target_y;
                }
            }
            return;
        }
    }
}

fn render_beam_groups(
    sink: &mut SceneEmitSink<'_>,
    measure_id: &str,
    mut anchors: Vec<BeamAnchor>,
    _measure_width: f32,
    _staff_bottom: f32,
) {
    anchors.sort_by(|a, b| {
        a.voice
            .cmp(&b.voice)
            .then_with(|| a.group.cmp(&b.group))
            .then_with(|| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
    });

    let mut current: Vec<BeamAnchor> = Vec::new();
    let mut flush_group = |group: &mut Vec<BeamAnchor>| {
        if group.is_empty() {
            return;
        }
        if group.len() == 1 {
            let anchor = &group[0];
            let flag_role = match (anchor.up, anchor.level) {
                (true, level) if level >= 3 => GlyphRole::Flag32ndUp,
                (false, level) if level >= 3 => GlyphRole::Flag32ndDown,
                (true, level) if level >= 2 => GlyphRole::Flag16thUp,
                (false, level) if level >= 2 => GlyphRole::Flag16thDown,
                (true, _) => GlyphRole::Flag8thUp,
                (false, _) => GlyphRole::Flag8thDown,
            };
            let flag_metric = canonical_glyph_metric(flag_role);
            let smufl_ss = BASE_FONT_SIZE_PT / 4.0;
            let flag_anchor =
                flag_metric
                    .stem_anchor_for_direction(anchor.up)
                    .unwrap_or(GlyphPoint {
                        x_ss: 0.0,
                        y_ss: 0.0,
                    });
            let flag_x = anchor.stem_x - flag_anchor.x_ss * smufl_ss;
            let flag_y = anchor.stem_tip_y + flag_anchor.y_ss * smufl_ss;
            let flag_id = sink.push_glyph_item(GlyphItemSpec {
                measure_id: Some(measure_id),
                role: "flag",
                x: flag_x,
                y: flag_y,
                glyph_role: flag_role,
                font_family: "Bravura",
                font_size_pt: BASE_FONT_SIZE_PT,
                fill: "#333",
            });
            if let Some(item) = sink.last_item_mut() {
                item.anchor_item_id = Some(anchor.stem_item_id.clone());
                debug_assert_eq!(item.id, flag_id);
            }
            group.clear();
            return;
        }

        let first = group.first().unwrap().clone();
        let last = group.last().unwrap().clone();
        let primary_y = first.stem_tip_y;
        let raw_end_y = last.stem_tip_y;
        let beam_slope = best_beam_slope(first.stem_x, primary_y, last.stem_x, raw_end_y);
        let end_y = primary_y + beam_slope * (last.stem_x - first.stem_x);

        // Stretch intermediate stems to reach the beam line
        if group.len() > 2 {
            let x1 = first.stem_x;
            let xn = last.stem_x;
            let dx = xn - x1;
            let dy = end_y - primary_y;
            for anchor in &group[1..group.len() - 1] {
                let t = if dx.abs() > 0.001 {
                    (anchor.stem_x - x1) / dx
                } else {
                    0.5
                };
                let target_tip_y = primary_y + dy * t;
                adjust_stem_tip(sink.items, &anchor.stem_item_id, target_tip_y, anchor.up);
            }
        }

        let beam_id = sink.push_path_item(PathItemSpec {
            measure_id: Some(measure_id),
            role: "beam",
            d: beam_path_d(first.stem_x, primary_y, last.stem_x, end_y, first.up, 4.0),
            fill: "#333",
            stroke: None,
            stroke_width: None,
        });
        if let Some(item) = sink.last_item_mut() {
            item.anchor_item_id = Some(first.stem_item_id.clone());
            debug_assert_eq!(item.id, beam_id);
        }
        let max_level = group.iter().map(|anchor| anchor.level).max().unwrap_or(1);
        for level in 2..=max_level {
            for segment in beam_line_segments_for_level(group, level) {
                let level_offset = if first.up {
                    6.0 * (level - 1) as f32
                } else {
                    -6.0 * (level - 1) as f32
                };
                let start_y =
                    beam_y_at_x(segment.start_x, first.stem_x, primary_y, last.stem_x, end_y)
                        + level_offset;
                let segment_end_y =
                    beam_y_at_x(segment.end_x, first.stem_x, primary_y, last.stem_x, end_y)
                        + level_offset;
                let secondary_id = sink.push_path_item(PathItemSpec {
                    measure_id: Some(measure_id),
                    role: "beam-secondary",
                    d: beam_path_d(
                        segment.start_x,
                        start_y,
                        segment.end_x,
                        segment_end_y,
                        first.up,
                        4.0,
                    ),
                    fill: "#333",
                    stroke: None,
                    stroke_width: None,
                });
                if let Some(item) = sink.last_item_mut() {
                    item.anchor_item_id = Some(first.stem_item_id.clone());
                    debug_assert_eq!(item.id, secondary_id);
                }
            }
        }
        group.clear();
    };

    for anchor in anchors {
        let starts_new_group = current.is_empty()
            || current
                .last()
                .map(|prev| {
                    prev.voice != anchor.voice || prev.up != anchor.up || prev.group != anchor.group
                })
                .unwrap_or(false);
        if starts_new_group {
            if !current.is_empty() {
                flush_group(&mut current);
            }
            current.push(anchor.clone());
        } else {
            current.push(anchor.clone());
        }
    }
    flush_group(&mut current);
}

const BEAM_MAX_SLOPE: f32 = 0.25;
const BEAM_MIN_SLOPE: f32 = -0.25;
const BEAM_SLOPE_ITERATIONS: u32 = 20;
const BEAM_SLOPE_COST: f32 = 100.0;

/// Finds the best beam slope by trying candidates in [BEAM_MIN_SLOPE, BEAM_MAX_SLOPE].
/// Cost is a combination of stem extension and distance from the ideal (half-natural) slope.
/// Matches VexFlow's `Beam.calculateSlope()`.
fn best_beam_slope(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    if dx.abs() < 0.001 {
        return 0.0;
    }
    let initial_slope = (y2 - y1) / dx;
    let ideal_slope = initial_slope * 0.5;
    let increment = (BEAM_MAX_SLOPE - BEAM_MIN_SLOPE) / BEAM_SLOPE_ITERATIONS as f32;

    let mut best_slope = initial_slope.clamp(BEAM_MIN_SLOPE, BEAM_MAX_SLOPE);
    let mut min_cost = f32::MAX;

    let mut slope = BEAM_MIN_SLOPE;
    for _ in 0..=BEAM_SLOPE_ITERATIONS {
        let distance_from_ideal = (ideal_slope - slope).abs();
        let cost = BEAM_SLOPE_COST * distance_from_ideal;
        if cost < min_cost {
            min_cost = cost;
            best_slope = slope;
        }
        slope += increment;
    }

    best_slope
}

fn beam_y_at_x(x: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    if dx.abs() < 0.001 {
        return y1;
    }
    y1 + (y2 - y1) * ((x - x1) / dx)
}

fn beam_line_segments_for_level(group: &[BeamAnchor], level: u8) -> Vec<BeamLineSegment> {
    const PARTIAL_BEAM_LENGTH_PT: f32 = 10.0;

    let mut segments = Vec::new();
    let mut active_start: Option<f32> = None;

    for (index, anchor) in group.iter().enumerate() {
        let gets_beam = anchor.level >= level;
        let next_gets_beam = group
            .get(index + 1)
            .map(|next| next.level >= level)
            .unwrap_or(false);

        if gets_beam {
            if let Some(start_x) = active_start {
                segments.push(BeamLineSegment {
                    start_x,
                    end_x: anchor.stem_x,
                });
                active_start = next_gets_beam.then_some(anchor.stem_x);
            } else if next_gets_beam {
                active_start = Some(anchor.stem_x);
            } else {
                // Isolated beamable note: draw a partial stub.
                // Direction: if any PREVIOUS anchor has this beam level → left;
                // otherwise (first in group) → right.
                // This matches VexFlow's partial beam direction logic.
                let has_prev_beam = group[..index].iter().any(|prev| prev.level >= level);
                let direction: f32 = if has_prev_beam { -1.0 } else { 1.0 };
                segments.push(BeamLineSegment {
                    start_x: anchor.stem_x,
                    end_x: anchor.stem_x + PARTIAL_BEAM_LENGTH_PT * direction,
                });
            }
        } else {
            active_start = None;
        }
    }

    segments
}

fn beam_path_d(x1: f32, y1: f32, x2: f32, y2: f32, up: bool, thickness: f32) -> String {
    let offset = if up { thickness } else { -thickness };
    format!(
        "M {:.3} {:.3} L {:.3} {:.3} L {:.3} {:.3} L {:.3} {:.3} Z",
        x1,
        y1,
        x2,
        y2,
        x2,
        y2 + offset,
        x1,
        y1 + offset,
    )
}

fn render_left_barline(
    sink: &mut SceneEmitSink<'_>,
    measure_id: Option<&str>,
    x: f32,
    top: f32,
    bottom: f32,
    barline: Option<&str>,
) {
    match barline {
        Some("repeat-start") | Some("repeat-both") => {
            render_start_repeat_barline(sink, measure_id, x, top, bottom)
        }
        _ => {}
    }
}

fn render_system_opening_barline(
    sink: &mut SceneEmitSink<'_>,
    measure_id: Option<&str>,
    x: f32,
    top: f32,
    bottom: f32,
) {
    sink.push_rect_item(RectItemSpec {
        measure_id,
        role: "opening-barline",
        x,
        y: top,
        width: 1.0,
        height: bottom - top + 1.0,
        fill: "#333",
        stroke: None,
        stroke_width: None,
    });
}

fn render_start_repeat_barline(
    sink: &mut SceneEmitSink<'_>,
    measure_id: Option<&str>,
    x: f32,
    top: f32,
    bottom: f32,
) {
    sink.push_glyph_item(GlyphItemSpec {
        measure_id,
        role: "repeat-start",
        x,
        y: start_repeat_vertical_origin(top, bottom),
        glyph_role: GlyphRole::RepeatLeft,
        font_family: "Bravura",
        font_size_pt: REPEAT_BARLINE_FONT_SIZE_PT,
        fill: "#333",
    });
}

struct RightBarlineSpec<'a> {
    measure_id: Option<&'a str>,
    x: f32,
    top: f32,
    bottom: f32,
    barline: Option<&'a str>,
    is_last_measure_of_score: bool,
}

fn render_right_barline(sink: &mut SceneEmitSink<'_>, spec: RightBarlineSpec<'_>) {
    let h = spec.bottom - spec.top + 1.0;
    match spec.barline {
        Some("repeat-end") | Some("repeat-both") => {
            let y = start_repeat_vertical_origin(spec.top, spec.bottom);
            sink.push_glyph_item(GlyphItemSpec {
                measure_id: spec.measure_id,
                role: "repeat-end",
                x: spec.x - repeat_barline_rendered_width(GlyphRole::RepeatRight),
                y,
                glyph_role: GlyphRole::RepeatRight,
                font_family: "Bravura",
                font_size_pt: REPEAT_BARLINE_FONT_SIZE_PT,
                fill: "#333",
            });
        }
        Some("double") => {
            sink.push_rect_item(RectItemSpec {
                measure_id: spec.measure_id,
                role: "double-barline-left",
                x: spec.x - 4.0,
                y: spec.top,
                width: 1.0,
                height: h,
                fill: "#333",
                stroke: None,
                stroke_width: None,
            });
            sink.push_rect_item(RectItemSpec {
                measure_id: spec.measure_id,
                role: "double-barline-right",
                x: spec.x - 1.0,
                y: spec.top,
                width: 1.0,
                height: h,
                fill: "#333",
                stroke: None,
                stroke_width: None,
            });
        }
        Some("final") => {
            sink.push_rect_item(RectItemSpec {
                measure_id: spec.measure_id,
                role: "final-barline-thin",
                x: spec.x - 4.0,
                y: spec.top,
                width: 1.0,
                height: h,
                fill: "#333",
                stroke: None,
                stroke_width: None,
            });
            sink.push_rect_item(RectItemSpec {
                measure_id: spec.measure_id,
                role: "final-barline-thick",
                x: spec.x - 3.0,
                y: spec.top,
                width: 3.0,
                height: h,
                fill: "#333",
                stroke: None,
                stroke_width: None,
            });
        }
        _ => {
            sink.push_rect_item(RectItemSpec {
                measure_id: spec.measure_id,
                role: if spec.is_last_measure_of_score {
                    "closing-barline"
                } else {
                    "barline"
                },
                x: spec.x - 1.0,
                y: spec.top,
                width: 1.0,
                height: h,
                fill: "#333",
                stroke: None,
                stroke_width: None,
            });
            if spec.is_last_measure_of_score {
                sink.push_rect_item(RectItemSpec {
                    measure_id: spec.measure_id,
                    role: "final-barline",
                    x: spec.x - 3.0,
                    y: spec.top,
                    width: 3.0,
                    height: h,
                    fill: "#333",
                    stroke: None,
                    stroke_width: None,
                });
            }
        }
    }
}

struct DeferredNavMarker {
    measure_id: String,
    global_index: u32,
    start_nav: Option<NavMarker>,
    end_nav: Option<NavJump>,
    x: f32,
    width: f32,
    top: f32,
}

/// Returns true for roles that are purely decorative (background, staff infrastructure)
/// and should not be considered when computing skyline for content-positioned markers.
const SKYLINE_Y_RANGE_ABOVE: f32 = 60.0;
const SKYLINE_Y_RANGE_BELOW: f32 = 30.0;

fn is_decoration_role(role: &str) -> bool {
    matches!(
        role,
        "tempo-glyph"
            | "tempo-equals"
            | "tempo"
            | "staff-line"
            | "percussion-clef"
            | "time-signature-digit"
            | "measure-number"
            | "title"
            | "subtitle"
            | "composer"
    )
}

fn skyline_top_for_range(
    items: &[SceneItem],
    x1: f32,
    x2: f32,
    reference_top: f32,
    fallback: f32,
) -> f32 {
    let left = x1.min(x2);
    let right = x1.max(x2);
    let mut top = f32::INFINITY;
    for item in items {
        if is_decoration_role(&item.role) {
            continue;
        }
        if item.role.starts_with("volta") {
            continue;
        }
        if let Some((item_x, item_y, item_width, _)) = item_bounds(item) {
            // Only consider items within a reasonable Y band of the reference.
            // Items far above (e.g. volta lines from other systems on the
            // pre-pagination page) must not push this marker upward.
            if item_y < reference_top - SKYLINE_Y_RANGE_ABOVE
                || item_y > reference_top + SKYLINE_Y_RANGE_BELOW
            {
                continue;
            }
            let item_right = item_x + item_width;
            if item_x < right && item_right > left {
                top = top.min(item_y);
            }
        }
    }
    if top.is_finite() {
        top
    } else {
        fallback
    }
}

fn render_nav_markers(
    sink: &mut SceneEmitSink<'_>,
    composites: &mut Vec<SceneComposite>,
    spec: &DeferredNavMarker,
) {
    let count_metric = canonical_text_metric(TextRole::CountLabel);
    const NAV_TEXT_FONT: &str = "Academico";
    const NAV_GAP: f32 = 6.0;
    if let Some(ref start_nav) = spec.start_nav {
        let (label, glyph_role) = match start_nav {
            NavMarker::Segno => ("segno", GlyphRole::NavigationSegno),
            NavMarker::Coda => ("coda", GlyphRole::NavigationCoda),
        };
        let glyph_width = rendered_glyph_width(glyph_role, 20.0);
        let x_start = spec.x + 4.0;
        let default_y = spec.top - 8.0;
        let occupied_top = skyline_top_for_range(
            &sink.items,
            x_start,
            x_start + glyph_width,
            spec.top,
            default_y + NAV_GAP,
        );
        let glyph_metric = canonical_glyph_metric(glyph_role);
        let nav_y = occupied_top - NAV_GAP + glyph_metric.bbox_sw_y_ss * (20.0 / 4.0);
        let nav_id = sink.push_glyph_item(GlyphItemSpec {
            measure_id: Some(spec.measure_id.as_str()),
            role: "nav-start",
            x: x_start,
            y: nav_y,
            glyph_role,
            font_family: "Bravura",
            font_size_pt: 20.0,
            fill: "#333",
        });
        composites.push(SceneComposite {
            id: format!("navigation-start-{}", spec.global_index),
            kind: CompositeKind::Navigation,
            fragment: SpanFragmentKind::SingleSegment,
            child_item_ids: vec![nav_id],
            label: Some(label.to_string()),
            count: None,
            start_anchor_id: Some(spec.measure_id.clone()),
            end_anchor_id: Some(spec.measure_id.clone()),
        });
    }
    if let Some(ref end_nav) = spec.end_nav {
        let label = match end_nav {
            NavJump::Fine => "Fine",
            NavJump::DC => "D.C.",
            NavJump::DS => "D.S.",
            NavJump::DCalFine => "D.C. al Fine",
            NavJump::DCalCoda => "D.C. al Coda",
            NavJump::DSalFine => "D.S. al Fine",
            NavJump::DSalCoda => "D.S. al Coda",
            NavJump::ToCoda => "To Coda",
        };
        let child_item_ids = match end_nav {
            NavJump::ToCoda => {
                let right_x = spec.x + spec.width - 4.0;
                let glyph_font_size = 16.0;
                let coda_width = rendered_glyph_width(GlyphRole::NavigationCoda, glyph_font_size);
                let to_text_width = canonical_text_width(TextRole::CountLabel, "To");
                let combined_x_start = right_x - coda_width - 4.0 - to_text_width;
                let combined_x_end = right_x;
                let default_glyph_y = spec.top - 8.0;
                let default_text_y = spec.top - count_metric.descent_pt - 1.0;
                let occupied_top = skyline_top_for_range(
                    &sink.items,
                    combined_x_start,
                    combined_x_end,
                    spec.top,
                    default_glyph_y + NAV_GAP,
                );
                let coda_metric = canonical_glyph_metric(GlyphRole::NavigationCoda);
                let default_glyph_bottom =
                    default_glyph_y - coda_metric.bbox_sw_y_ss * (glyph_font_size / 4.0);
                let default_text_bottom = default_text_y + count_metric.descent_pt;
                let default_group_bottom = default_glyph_bottom.max(default_text_bottom);
                let delta = occupied_top - NAV_GAP - default_group_bottom;
                let glyph_y = default_glyph_y + delta;
                let text_y = default_text_y + delta;
                let glyph_id = sink.push_glyph_item(GlyphItemSpec {
                    measure_id: Some(spec.measure_id.as_str()),
                    role: "nav-end-symbol",
                    x: right_x - coda_width,
                    y: glyph_y,
                    glyph_role: GlyphRole::NavigationCoda,
                    font_family: "Bravura",
                    font_size_pt: glyph_font_size,
                    fill: "#333",
                });
                let text_id = sink.push_text_item(TextItemSpec {
                    measure_id: Some(spec.measure_id.as_str()),
                    role: "nav-end",
                    x: right_x - coda_width - 4.0,
                    y: text_y,
                    text_role: TextRole::CountLabel,
                    text: "To".to_string(),
                    font_family: NAV_TEXT_FONT,
                    font_size_pt: count_metric.font_size_pt,
                    fill: "#333",
                    text_anchor: Some("end"),
                    font_weight: Some("bold"),
                });
                vec![text_id, glyph_id]
            }
            _ => {
                let text_width = canonical_text_width(TextRole::CountLabel, label);
                let x_start = spec.x + spec.width - 4.0 - text_width;
                let x_end = spec.x + spec.width - 4.0;
                let default_y = spec.top - count_metric.descent_pt - 1.0;
                let occupied_top = skyline_top_for_range(
                    &sink.items,
                    x_start,
                    x_end,
                    spec.top,
                    default_y + NAV_GAP,
                );
                let nav_y = occupied_top - NAV_GAP - count_metric.descent_pt;
                let nav_id = sink.push_text_item(TextItemSpec {
                    measure_id: Some(spec.measure_id.as_str()),
                    role: "nav-end",
                    x: spec.x + spec.width - 4.0,
                    y: nav_y,
                    text_role: TextRole::CountLabel,
                    text: label.to_string(),
                    font_family: NAV_TEXT_FONT,
                    font_size_pt: count_metric.font_size_pt,
                    fill: "#333",
                    text_anchor: Some("end"),
                    font_weight: Some("bold"),
                });
                vec![nav_id]
            }
        };
        composites.push(SceneComposite {
            id: format!("navigation-end-{}", spec.global_index),
            kind: CompositeKind::Navigation,
            fragment: SpanFragmentKind::SingleSegment,
            child_item_ids,
            label: Some(label.to_string()),
            count: None,
            start_anchor_id: Some(spec.measure_id.clone()),
            end_anchor_id: Some(spec.measure_id.clone()),
        });
    }
}

fn render_hairpin_fragments(
    sink: &mut SceneEmitSink<'_>,
    composites: &mut Vec<SceneComposite>,
    page_measures: &[SceneMeasure],
    measures: &[DisplayMeasure<'_>],
    hairpin_offset_y: f32,
) {
    const HAIRPIN_OPEN_HEIGHT_PT: f32 = 10.0;
    const HAIRPIN_GAP_BELOW_PT: f32 = 0.0;

    for measure in measures {
        for hairpin in &measure.hairpins {
            let fragments = measure_fragments_for_range(
                page_measures,
                hairpin.start_measure_index,
                hairpin.end_measure_index,
            );
            let fragment_total = fragments.len();
            let total_start = hairpin.start_measure_index as f32 + fraction_to_f32(hairpin.start);
            let mut total_end = hairpin.end_measure_index as f32 + fraction_to_f32(hairpin.end);
            if total_end <= total_start {
                total_end = total_start + 0.05;
            }
            let total_span = total_end - total_start;
            for (fragment_index, fragment) in fragments.iter().enumerate() {
                if fragment.is_empty() {
                    continue;
                }
                let first = fragment.first().unwrap();
                let last = fragment.last().unwrap();
                let start_progress = if first.global_index == hairpin.start_measure_index {
                    fraction_to_f32(hairpin.start)
                } else {
                    0.0
                };
                let end_progress = if last.global_index == hairpin.end_measure_index {
                    fraction_to_f32(hairpin.end).max(start_progress + 0.05)
                } else {
                    1.0
                };
                let start_x = if fragment_index == 0 {
                    first.x_pt + 14.0 + start_progress * (first.width_pt - 28.0)
                } else {
                    first.x_pt + 14.0
                };
                let end_x = if fragment_index + 1 == fragment_total {
                    last.x_pt + 14.0 + end_progress * (last.width_pt - 28.0)
                } else {
                    last.x_pt + last.width_pt - 12.0
                };
                if end_x <= start_x {
                    continue;
                }
                let fragment_start_abs = first.global_index as f32 + start_progress;
                let fragment_end_abs = last.global_index as f32 + end_progress;
                let left_progress =
                    ((fragment_start_abs - total_start) / total_span).clamp(0.0, 1.0);
                let right_progress =
                    ((fragment_end_abs - total_start) / total_span).clamp(0.0, 1.0);
                let left_open_height = hairpin_open_height_at_progress(
                    hairpin.kind,
                    left_progress,
                    HAIRPIN_OPEN_HEIGHT_PT,
                );
                let right_open_height = hairpin_open_height_at_progress(
                    hairpin.kind,
                    right_progress,
                    HAIRPIN_OPEN_HEIGHT_PT,
                );
                let top_y = bottom_skyline_sample(
                    sink.items,
                    fragment,
                    start_x,
                    end_x,
                    first.y_pt + first.height_pt,
                ) + HAIRPIN_GAP_BELOW_PT
                    + hairpin_offset_y;
                let center_y = top_y + HAIRPIN_OPEN_HEIGHT_PT * 0.5;
                let left_top_y = center_y - left_open_height * 0.5;
                let left_bottom_y = center_y + left_open_height * 0.5;
                let right_top_y = center_y - right_open_height * 0.5;
                let right_bottom_y = center_y + right_open_height * 0.5;
                let top_id = sink.push_line_item(LineItemSpec {
                    measure_id: Some(&first.id),
                    role: "hairpin-top",
                    x1: start_x,
                    y1: left_top_y,
                    x2: end_x,
                    y2: right_top_y,
                    stroke: "#333",
                    stroke_width: 1.2,
                    stroke_line_cap: None,
                });
                let bottom_id = sink.push_line_item(LineItemSpec {
                    measure_id: Some(&first.id),
                    role: "hairpin-bottom",
                    x1: start_x,
                    y1: left_bottom_y,
                    x2: end_x,
                    y2: right_bottom_y,
                    stroke: "#333",
                    stroke_width: 1.2,
                    stroke_line_cap: None,
                });
                composites.push(SceneComposite {
                    id: format!(
                        "hairpin-{}-{}-{}",
                        hairpin.start_measure_index, hairpin.end_measure_index, fragment_index
                    ),
                    kind: CompositeKind::Hairpin,
                    fragment: span_fragment_kind(fragment_index, fragment_total),
                    child_item_ids: vec![top_id, bottom_id],
                    label: Some(match hairpin.kind {
                        HairpinKind::Crescendo => "crescendo".to_string(),
                        HairpinKind::Decrescendo => "decrescendo".to_string(),
                    }),
                    count: None,
                    start_anchor_id: Some(first.id.clone()),
                    end_anchor_id: Some(last.id.clone()),
                });
            }
        }
    }
}

fn hairpin_open_height_at_progress(kind: HairpinKind, progress: f32, max_height: f32) -> f32 {
    let clamped = progress.clamp(0.0, 1.0);
    match kind {
        HairpinKind::Crescendo => max_height * clamped,
        HairpinKind::Decrescendo => max_height * (1.0 - clamped),
    }
}

fn bottom_skyline_sample(
    items: &[SceneItem],
    block_measures: &[&SceneMeasure],
    x1: f32,
    x2: f32,
    fallback_bottom: f32,
) -> f32 {
    let left = x1.min(x2);
    let right = x1.max(x2);
    let measure_ids = block_measures
        .iter()
        .map(|measure| measure.id.as_str())
        .collect::<std::collections::HashSet<_>>();
    let system_top = block_measures
        .iter()
        .map(|measure| measure.y_pt)
        .fold(f32::INFINITY, f32::min);
    let system_bottom = block_measures
        .iter()
        .map(|measure| measure.y_pt + measure.height_pt)
        .fold(f32::NEG_INFINITY, f32::max);
    let mut bottom = f32::NEG_INFINITY;
    for item in items {
        if item.role.starts_with("hairpin") {
            continue;
        }
        let in_block_measure = item
            .measure_id
            .as_deref()
            .is_some_and(|measure_id| measure_ids.contains(measure_id));
        if let Some((item_x, item_y, item_width, item_height)) = item_bounds(item) {
            let in_system_band = item.measure_id.is_none()
                && item_y >= system_top - 20.0
                && item_y <= system_bottom + 60.0;
            if !in_block_measure && !in_system_band {
                continue;
            }
            let item_right = item_x + item_width;
            if item_x < right && item_right > left {
                bottom = bottom.max(item_y + item_height);
            }
        }
    }
    if bottom.is_finite() {
        bottom
    } else {
        fallback_bottom
    }
}

#[derive(Clone)]
struct EdgeGroup {
    item_ids: Vec<String>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    priority: u8,
    below_staff: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SceneItemBounds {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl SceneItemBounds {
    fn as_tuple(self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }
}

fn stack_scene_structural_items(
    items: &mut [SceneItem],
    composites: &[SceneComposite],
    edge_padding: f32,
) {
    let item_index = items
        .iter()
        .enumerate()
        .map(|(index, item)| (item.id.clone(), index))
        .collect::<std::collections::HashMap<_, _>>();
    let mut groups = Vec::new();
    let mut volta_groups = std::collections::BTreeMap::<i32, Vec<String>>::new();

    for composite in composites {
        if composite.kind == CompositeKind::Volta {
            if let Some((_, y, _, _)) =
                bounding_box_for_ids(items, &item_index, &composite.child_item_ids)
            {
                let key = (y * 100.0).round() as i32;
                volta_groups
                    .entry(key)
                    .or_default()
                    .extend(composite.child_item_ids.iter().cloned());
            }
            continue;
        }
        let priority = match composite.kind {
            CompositeKind::Navigation => Some((1_u8, false)),
            CompositeKind::RepeatSpan => Some((2_u8, false)),
            CompositeKind::Hairpin => Some((1_u8, true)),
            _ => None,
        };
        let Some((priority, below_staff)) = priority else {
            continue;
        };
        if composite.child_item_ids.is_empty() {
            continue;
        }
        if let Some((x, y, width, height)) =
            bounding_box_for_ids(items, &item_index, &composite.child_item_ids)
        {
            groups.push(EdgeGroup {
                item_ids: composite.child_item_ids.clone(),
                x,
                y,
                width,
                height,
                priority,
                below_staff,
            });
        }
    }

    for (_, item_ids) in volta_groups {
        if let Some((x, y, width, height)) = bounding_box_for_ids(items, &item_index, &item_ids) {
            groups.push(EdgeGroup {
                item_ids,
                x,
                y,
                width,
                height,
                priority: 3,
                below_staff: false,
            });
        }
    }

    for role in ["measure-number"] {
        for item in items.iter().filter(|item| item.role == role) {
            if let Some((x, y, width, height)) = item_bounds(item) {
                groups.push(EdgeGroup {
                    item_ids: vec![item.id.clone()],
                    x,
                    y,
                    width,
                    height,
                    priority: 0,
                    below_staff: false,
                });
            }
        }
    }

    groups.sort_by(|a, b| a.priority.cmp(&b.priority));
    let mut shifted: Vec<EdgeGroup> = Vec::new();
    for mut group in groups {
        loop {
            let overlap = shifted
                .iter()
                .filter(|other| other.below_staff == group.below_staff)
                .find(|other| {
                    let x_overlap =
                        group.x < other.x + other.width && group.x + group.width > other.x;
                    let y_overlap =
                        group.y < other.y + other.height && group.y + group.height > other.y;
                    x_overlap && y_overlap
                })
                .cloned();
            let Some(other) = overlap else { break };
            if group.below_staff {
                group.y = other.y + other.height + edge_padding;
            } else {
                group.y = other.y - group.height - edge_padding;
            }
        }
        if let Some((_, original_y, _, _)) =
            bounding_box_for_ids(items, &item_index, &group.item_ids)
        {
            translate_item_ids(items, &item_index, &group.item_ids, group.y - original_y);
        }
        shifted.push(group);
    }
}

fn bounding_box_for_ids(
    items: &[SceneItem],
    item_index: &std::collections::HashMap<String, usize>,
    ids: &[String],
) -> Option<(f32, f32, f32, f32)> {
    let bounds = ids
        .iter()
        .filter_map(|id| {
            item_index
                .get(id)
                .and_then(|index| item_bounds(&items[*index]))
        })
        .collect::<Vec<_>>();
    if bounds.is_empty() {
        return None;
    }
    let min_x = bounds
        .iter()
        .map(|(x, _, _, _)| *x)
        .fold(f32::INFINITY, f32::min);
    let min_y = bounds
        .iter()
        .map(|(_, y, _, _)| *y)
        .fold(f32::INFINITY, f32::min);
    let max_x = bounds
        .iter()
        .map(|(x, _, width, _)| x + width)
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y = bounds
        .iter()
        .map(|(_, y, _, height)| y + height)
        .fold(f32::NEG_INFINITY, f32::max);
    Some((min_x, min_y, max_x - min_x, max_y - min_y))
}

fn item_bounds(item: &SceneItem) -> Option<(f32, f32, f32, f32)> {
    match &item.primitive {
        ScenePrimitive::TextRun(text) => {
            let metric = canonical_text_metric(text.text_role);
            let width = canonical_text_width(text.text_role, &text.text);
            let x = match text.text_anchor.as_deref() {
                Some("middle") => text.x_pt - width * 0.5,
                Some("end") => text.x_pt - width,
                _ => text.x_pt,
            };
            Some((
                x,
                text.y_pt - metric.ascent_pt,
                width,
                metric.line_height_pt,
            ))
        }
        ScenePrimitive::LineSegment(line) => Some((
            line.x1_pt.min(line.x2_pt),
            line.y1_pt.min(line.y2_pt),
            (line.x2_pt - line.x1_pt).abs().max(line.stroke_width),
            (line.y2_pt - line.y1_pt).abs().max(line.stroke_width),
        )),
        ScenePrimitive::Rect(rect) => Some((rect.x_pt, rect.y_pt, rect.width_pt, rect.height_pt)),
        ScenePrimitive::Polyline(polyline) => {
            let min_x = polyline
                .points_pt
                .iter()
                .map(|(x, _)| *x)
                .fold(f32::INFINITY, f32::min);
            let min_y = polyline
                .points_pt
                .iter()
                .map(|(_, y)| *y)
                .fold(f32::INFINITY, f32::min);
            let max_x = polyline
                .points_pt
                .iter()
                .map(|(x, _)| *x)
                .fold(f32::NEG_INFINITY, f32::max);
            let max_y = polyline
                .points_pt
                .iter()
                .map(|(_, y)| *y)
                .fold(f32::NEG_INFINITY, f32::max);
            Some((min_x, min_y, max_x - min_x, max_y - min_y))
        }
        ScenePrimitive::Path(path) => path_bounds(&path.d).map(SceneItemBounds::as_tuple),
        ScenePrimitive::GlyphRun(glyph) => {
            let metric = canonical_glyph_metric(glyph.glyph_role);
            let ss_to_pt = glyph.font_size_pt / 4.0;
            Some((
                glyph.x_pt + metric.bbox_sw_x_ss * ss_to_pt,
                glyph.y_pt - metric.bbox_ne_y_ss * ss_to_pt,
                metric.bbox_width_ss() * ss_to_pt,
                metric.bbox_height_ss() * ss_to_pt,
            ))
        }
    }
}

#[allow(dead_code)]
fn scene_item_bounds(item: &SceneItem) -> Result<SceneItemBounds, String> {
    match &item.primitive {
        ScenePrimitive::TextRun(text) => {
            let metric = canonical_text_metric(text.text_role);
            let width = canonical_text_width(text.text_role, &text.text);
            let x = match text.text_anchor.as_deref() {
                Some("middle") => text.x_pt - width * 0.5,
                Some("end") => text.x_pt - width,
                _ => text.x_pt,
            };
            Ok(SceneItemBounds {
                x,
                y: text.y_pt - metric.ascent_pt,
                width,
                height: metric.line_height_pt,
            })
        }
        ScenePrimitive::LineSegment(line) => {
            let pad = line.stroke_width * 0.5;
            let min_x = line.x1_pt.min(line.x2_pt) - pad;
            let min_y = line.y1_pt.min(line.y2_pt) - pad;
            let max_x = line.x1_pt.max(line.x2_pt) + pad;
            let max_y = line.y1_pt.max(line.y2_pt) + pad;
            Ok(SceneItemBounds {
                x: min_x,
                y: min_y,
                width: max_x - min_x,
                height: max_y - min_y,
            })
        }
        ScenePrimitive::Rect(rect) => {
            let pad = if rect.stroke.is_some() {
                rect.stroke_width.unwrap_or(1.0) * 0.5
            } else {
                0.0
            };
            Ok(SceneItemBounds {
                x: rect.x_pt - pad,
                y: rect.y_pt - pad,
                width: rect.width_pt + pad * 2.0,
                height: rect.height_pt + pad * 2.0,
            })
        }
        ScenePrimitive::Polyline(polyline) => {
            if polyline.points_pt.is_empty() {
                return Err(format!("SceneItem {} has an empty polyline", item.id));
            }
            let min_x = polyline
                .points_pt
                .iter()
                .map(|(x, _)| *x)
                .fold(f32::INFINITY, f32::min);
            let min_y = polyline
                .points_pt
                .iter()
                .map(|(_, y)| *y)
                .fold(f32::INFINITY, f32::min);
            let max_x = polyline
                .points_pt
                .iter()
                .map(|(x, _)| *x)
                .fold(f32::NEG_INFINITY, f32::max);
            let max_y = polyline
                .points_pt
                .iter()
                .map(|(_, y)| *y)
                .fold(f32::NEG_INFINITY, f32::max);
            Ok(SceneItemBounds {
                x: min_x,
                y: min_y,
                width: max_x - min_x,
                height: max_y - min_y,
            })
        }
        ScenePrimitive::Path(path) => {
            let mut bounds = path_bounds(&path.d)
                .ok_or_else(|| format!("SceneItem {} has an unsupported path", item.id))?;
            if path.stroke.is_some() {
                let pad = path.stroke_width.unwrap_or(1.0) * 0.5;
                bounds.x -= pad;
                bounds.y -= pad;
                bounds.width += pad * 2.0;
                bounds.height += pad * 2.0;
            }
            Ok(bounds)
        }
        ScenePrimitive::GlyphRun(glyph) => {
            let metric = canonical_glyph_metric(glyph.glyph_role);
            let ss_to_pt = glyph.font_size_pt / 4.0;
            Ok(SceneItemBounds {
                x: glyph.x_pt + metric.bbox_sw_x_ss * ss_to_pt,
                y: glyph.y_pt - metric.bbox_ne_y_ss * ss_to_pt,
                width: metric.bbox_width_ss() * ss_to_pt,
                height: metric.bbox_height_ss() * ss_to_pt,
            })
        }
    }
}

fn translate_item_ids(
    items: &mut [SceneItem],
    item_index: &std::collections::HashMap<String, usize>,
    ids: &[String],
    dy: f32,
) {
    for id in ids {
        if let Some(index) = item_index.get(id) {
            translate_item(&mut items[*index], dy);
        }
    }
}

fn translate_item(item: &mut SceneItem, dy: f32) {
    match &mut item.primitive {
        ScenePrimitive::TextRun(text) => text.y_pt += dy,
        ScenePrimitive::LineSegment(line) => {
            line.y1_pt += dy;
            line.y2_pt += dy;
        }
        ScenePrimitive::Rect(rect) => rect.y_pt += dy,
        ScenePrimitive::Polyline(polyline) => {
            for (_, y) in &mut polyline.points_pt {
                *y += dy;
            }
        }
        ScenePrimitive::Path(path) => translate_path_y(&mut path.d, dy),
        ScenePrimitive::GlyphRun(glyph) => glyph.y_pt += dy,
    }
}

fn path_bounds(d: &str) -> Option<SceneItemBounds> {
    let numbers = d
        .split(|ch: char| !(ch.is_ascii_digit() || ch == '.' || ch == '-'))
        .filter(|segment| !segment.is_empty())
        .filter_map(|segment| segment.parse::<f32>().ok())
        .collect::<Vec<_>>();
    if numbers.len() < 2 {
        return None;
    }
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for pair in numbers.chunks(2) {
        if let [x, y] = pair {
            min_x = min_x.min(*x);
            min_y = min_y.min(*y);
            max_x = max_x.max(*x);
            max_y = max_y.max(*y);
        }
    }
    Some(SceneItemBounds {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    })
}

fn translate_path_y(d: &mut String, dy: f32) {
    let tokens = d.split_whitespace().collect::<Vec<_>>();
    if tokens.is_empty() {
        return;
    }
    let mut translated = Vec::with_capacity(tokens.len());
    let mut coordinate_index = 0usize;
    for token in tokens {
        if let Ok(value) = token.parse::<f32>() {
            let adjusted = if coordinate_index % 2 == 1 {
                value + dy
            } else {
                value
            };
            translated.push(format!("{adjusted:.3}"));
            coordinate_index += 1;
        } else {
            translated.push(token.to_string());
        }
    }
    *d = translated.join(" ");
}

fn fraction_to_f32(fraction: Fraction) -> f32 {
    fraction.numerator as f32 / fraction.denominator.max(1) as f32
}

struct SceneEmitSink<'a> {
    items: &'a mut Vec<SceneItem>,
    counter: &'a mut usize,
}

impl<'a> SceneEmitSink<'a> {
    fn new(items: &'a mut Vec<SceneItem>, counter: &'a mut usize) -> Self {
        Self { items, counter }
    }

    fn next_id(&mut self) -> String {
        let id = format!("item-{}", self.counter);
        *self.counter += 1;
        id
    }

    fn last_item_mut(&mut self) -> Option<&mut SceneItem> {
        self.items.last_mut()
    }

    fn push_rect_item(&mut self, spec: RectItemSpec<'_>) {
        let id = self.next_id();
        self.items.push(SceneItem {
            id,
            measure_id: spec.measure_id.map(ToString::to_string),
            anchor_item_id: None,
            role: spec.role.to_string(),
            kind: SceneItemKind::Rect,
            z_index: 0,
            primitive: ScenePrimitive::Rect(RectShape {
                x_pt: spec.x,
                y_pt: spec.y,
                width_pt: spec.width,
                height_pt: spec.height,
                fill: spec.fill.to_string(),
                stroke: spec.stroke.map(ToString::to_string),
                stroke_width: spec.stroke_width,
            }),
        });
    }

    fn push_text_item(&mut self, spec: TextItemSpec<'_>) -> String {
        let id = self.next_id();
        self.items.push(SceneItem {
            id: id.clone(),
            measure_id: spec.measure_id.map(ToString::to_string),
            anchor_item_id: None,
            role: spec.role.to_string(),
            kind: SceneItemKind::TextRun,
            z_index: 0,
            primitive: ScenePrimitive::TextRun(TextRun {
                x_pt: spec.x,
                y_pt: spec.y,
                text_role: spec.text_role,
                text: spec.text,
                font_family: spec.font_family.to_string(),
                font_size_pt: spec.font_size_pt,
                fill: spec.fill.to_string(),
                text_anchor: spec.text_anchor.map(ToString::to_string),
                font_weight: spec.font_weight.map(ToString::to_string),
            }),
        });
        id
    }

    fn push_line_item(&mut self, spec: LineItemSpec<'_>) -> String {
        let id = self.next_id();
        self.items.push(SceneItem {
            id: id.clone(),
            measure_id: spec.measure_id.map(ToString::to_string),
            anchor_item_id: None,
            role: spec.role.to_string(),
            kind: SceneItemKind::LineSegment,
            z_index: 0,
            primitive: ScenePrimitive::LineSegment(LineSegment {
                x1_pt: spec.x1,
                y1_pt: spec.y1,
                x2_pt: spec.x2,
                y2_pt: spec.y2,
                stroke: spec.stroke.to_string(),
                stroke_width: spec.stroke_width,
                stroke_line_cap: spec.stroke_line_cap.map(ToString::to_string),
            }),
        });
        id
    }

    fn push_path_item(&mut self, spec: PathItemSpec<'_>) -> String {
        let id = self.next_id();
        self.items.push(SceneItem {
            id: id.clone(),
            measure_id: spec.measure_id.map(ToString::to_string),
            anchor_item_id: None,
            role: spec.role.to_string(),
            kind: SceneItemKind::Path,
            z_index: 0,
            primitive: ScenePrimitive::Path(PathShape {
                d: spec.d,
                fill: spec.fill.to_string(),
                stroke: spec.stroke.map(ToString::to_string),
                stroke_width: spec.stroke_width,
            }),
        });
        id
    }

    fn push_glyph_item(&mut self, spec: GlyphItemSpec<'_>) -> String {
        let id = self.next_id();
        let metric = canonical_glyph_metric(spec.glyph_role);
        self.items.push(SceneItem {
            id: id.clone(),
            measure_id: spec.measure_id.map(ToString::to_string),
            anchor_item_id: None,
            role: spec.role.to_string(),
            kind: SceneItemKind::GlyphRun,
            z_index: 0,
            primitive: ScenePrimitive::GlyphRun(GlyphRun {
                x_pt: spec.x,
                y_pt: spec.y,
                glyph_role: spec.glyph_role,
                glyph_count: 1,
                smufl_codepoint: Some(metric.smufl_codepoint),
                font_family: spec.font_family.to_string(),
                font_size_pt: spec.font_size_pt,
                fill: spec.fill.to_string(),
            }),
        });
        id
    }
}

struct RectItemSpec<'a> {
    measure_id: Option<&'a str>,
    role: &'a str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    fill: &'a str,
    stroke: Option<&'a str>,
    stroke_width: Option<f32>,
}

struct TextItemSpec<'a> {
    measure_id: Option<&'a str>,
    role: &'a str,
    x: f32,
    y: f32,
    text_role: TextRole,
    text: String,
    font_family: &'a str,
    font_size_pt: f32,
    fill: &'a str,
    text_anchor: Option<&'a str>,
    font_weight: Option<&'a str>,
}

struct LineItemSpec<'a> {
    measure_id: Option<&'a str>,
    role: &'a str,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    stroke: &'a str,
    stroke_width: f32,
    stroke_line_cap: Option<&'a str>,
}

struct PathItemSpec<'a> {
    measure_id: Option<&'a str>,
    role: &'a str,
    d: String,
    fill: &'a str,
    stroke: Option<&'a str>,
    stroke_width: Option<f32>,
}

struct GlyphItemSpec<'a> {
    measure_id: Option<&'a str>,
    role: &'a str,
    x: f32,
    y: f32,
    glyph_role: GlyphRole,
    font_family: &'a str,
    font_size_pt: f32,
    fill: &'a str,
}

fn num_to_glyph(n: u32) -> String {
    match n {
        0 => "\u{E080}".to_string(),
        1 => "\u{E081}".to_string(),
        2 => "\u{E082}".to_string(),
        3 => "\u{E083}".to_string(),
        4 => "\u{E084}".to_string(),
        5 => "\u{E085}".to_string(),
        6 => "\u{E086}".to_string(),
        7 => "\u{E087}".to_string(),
        8 => "\u{E088}".to_string(),
        9 => "\u{E089}".to_string(),
        _ => n.to_string(),
    }
}

fn to_wire_scene(scene: &LayoutScene) -> WireLayoutScene {
    WireLayoutScene {
        version: scene.version.clone(),
        metrics_version: scene.metrics_version.clone(),
        pages: scene
            .pages
            .iter()
            .map(|page| WireScenePage {
                index: page.index,
                width_pt: page.width_pt,
                height_pt: page.height_pt,
                systems: page
                    .systems
                    .iter()
                    .map(|system| WireSceneSystem {
                        id: system.id.clone(),
                        index: system.index,
                        page_index: system.page_index,
                        x_pt: system.x_pt,
                        y_pt: system.y_pt,
                        width_pt: system.width_pt,
                        height_pt: system.height_pt,
                        measure_ids: system.measure_ids.clone(),
                    })
                    .collect(),
                measures: page
                    .measures
                    .iter()
                    .map(|measure| WireSceneMeasure {
                        id: measure.id.clone(),
                        index: measure.index,
                        global_index: measure.global_index,
                        system_id: measure.system_id.clone(),
                        x_pt: measure.x_pt,
                        y_pt: measure.y_pt,
                        width_pt: measure.width_pt,
                        height_pt: measure.height_pt,
                    })
                    .collect(),
                items: page
                    .items
                    .iter()
                    .map(|item| WireSceneItem {
                        id: item.id.clone(),
                        measure_id: item.measure_id.clone(),
                        anchor_item_id: item.anchor_item_id.clone(),
                        role: item.role.clone(),
                        kind: scene_item_kind_name(item.kind),
                        z_index: item.z_index,
                        primitive: match &item.primitive {
                            ScenePrimitive::GlyphRun(glyph) => WireScenePrimitive::GlyphRun {
                                x_pt: glyph.x_pt,
                                y_pt: glyph.y_pt,
                                glyph_role: glyph_role_name(glyph.glyph_role),
                                glyph_count: glyph.glyph_count,
                                codepoint: glyph.smufl_codepoint,
                                font_family: glyph.font_family.clone(),
                                font_size_pt: glyph.font_size_pt,
                                fill: glyph.fill.clone(),
                            },
                            ScenePrimitive::TextRun(text) => WireScenePrimitive::TextRun {
                                x_pt: text.x_pt,
                                y_pt: text.y_pt,
                                text_role: text_role_name(text.text_role),
                                text: text.text.clone(),
                                font_family: text.font_family.clone(),
                                font_size_pt: text.font_size_pt,
                                fill: text.fill.clone(),
                                text_anchor: text.text_anchor.clone(),
                                font_weight: text.font_weight.clone(),
                            },
                            ScenePrimitive::LineSegment(line) => WireScenePrimitive::LineSegment {
                                x1_pt: line.x1_pt,
                                y1_pt: line.y1_pt,
                                x2_pt: line.x2_pt,
                                y2_pt: line.y2_pt,
                                stroke: line.stroke.clone(),
                                stroke_width: line.stroke_width,
                                stroke_line_cap: line.stroke_line_cap.clone(),
                            },
                            ScenePrimitive::Rect(rect) => WireScenePrimitive::Rect {
                                x_pt: rect.x_pt,
                                y_pt: rect.y_pt,
                                width_pt: rect.width_pt,
                                height_pt: rect.height_pt,
                                fill: rect.fill.clone(),
                                stroke: rect.stroke.clone(),
                                stroke_width: rect.stroke_width,
                            },
                            ScenePrimitive::Polyline(polyline) => WireScenePrimitive::Polyline {
                                points_pt: polyline.points_pt.clone(),
                            },
                            ScenePrimitive::Path(path) => WireScenePrimitive::Path {
                                d: path.d.clone(),
                                fill: path.fill.clone(),
                                stroke: path.stroke.clone(),
                                stroke_width: path.stroke_width,
                            },
                        },
                    })
                    .collect(),
                composites: page
                    .composites
                    .iter()
                    .map(|composite| WireSceneComposite {
                        id: composite.id.clone(),
                        kind: composite_kind_name(composite.kind),
                        fragment: fragment_kind_name(composite.fragment),
                        child_item_ids: composite.child_item_ids.clone(),
                        label: composite.label.clone(),
                        count: composite.count,
                        start_anchor_id: composite.start_anchor_id.clone(),
                        end_anchor_id: composite.end_anchor_id.clone(),
                    })
                    .collect(),
            })
            .collect(),
        issues: scene.issues.clone(),
    }
}

pub fn layout_scene_snapshot(scene: &LayoutScene) -> String {
    let wire = to_wire_scene(scene);
    let mut out = String::new();
    out.push_str(&format!("version={}\n", wire.version));
    out.push_str(&format!("metricsVersion={}\n", wire.metrics_version));
    if !wire.issues.is_empty() {
        out.push_str("issues:\n");
        for issue in &wire.issues {
            out.push_str(&format!("  - {}\n", issue));
        }
    }
    for page in &wire.pages {
        out.push_str(&format!(
            "page index={} widthPt={:.3} heightPt={:.3}\n",
            page.index, page.width_pt, page.height_pt
        ));
        for system in &page.systems {
            out.push_str(&format!(
                "  system id={} index={} pageIndex={} xPt={:.3} yPt={:.3} widthPt={:.3} heightPt={:.3} measureIds=[{}]\n",
                system.id,
                system.index,
                system.page_index,
                system.x_pt,
                system.y_pt,
                system.width_pt,
                system.height_pt,
                system.measure_ids.join(",")
            ));
        }
        for measure in &page.measures {
            out.push_str(&format!(
                "  measure id={} index={} globalIndex={} systemId={} xPt={:.3} yPt={:.3} widthPt={:.3} heightPt={:.3}\n",
                measure.id,
                measure.index,
                measure.global_index,
                measure.system_id,
                measure.x_pt,
                measure.y_pt,
                measure.width_pt,
                measure.height_pt
            ));
        }
        for item in &page.items {
            out.push_str(&format!(
                "  item id={} measureId={} anchorItemId={} role={} kind={} zIndex={}",
                item.id,
                item.measure_id.as_deref().unwrap_or("-"),
                item.anchor_item_id.as_deref().unwrap_or("-"),
                item.role,
                item.kind,
                item.z_index
            ));
            match &item.primitive {
                WireScenePrimitive::GlyphRun {
                    x_pt,
                    y_pt,
                    glyph_role,
                    glyph_count,
                    codepoint,
                    font_family,
                    font_size_pt,
                    fill,
                } => {
                    out.push_str(&format!(
                        " primitive={{glyphRole={} glyphCount={} codepoint={} xPt={:.3} yPt={:.3} fontFamily={} fontSizePt={:.3} fill={}}}",
                        glyph_role,
                        glyph_count,
                        codepoint.map(|value| value.to_string()).unwrap_or_else(|| "-".to_string()),
                        x_pt,
                        y_pt,
                        font_family,
                        font_size_pt,
                        fill
                    ));
                }
                WireScenePrimitive::TextRun {
                    x_pt,
                    y_pt,
                    text_role,
                    text,
                    font_family,
                    font_size_pt,
                    fill,
                    text_anchor,
                    font_weight,
                } => {
                    out.push_str(&format!(
                        " primitive={{textRole={} text={:?} xPt={:.3} yPt={:.3} fontFamily={} fontSizePt={:.3} fill={} textAnchor={} fontWeight={}}}",
                        text_role,
                        text,
                        x_pt,
                        y_pt,
                        font_family,
                        font_size_pt,
                        fill,
                        text_anchor.as_deref().unwrap_or("-"),
                        font_weight.as_deref().unwrap_or("-")
                    ));
                }
                WireScenePrimitive::LineSegment {
                    x1_pt,
                    y1_pt,
                    x2_pt,
                    y2_pt,
                    stroke,
                    stroke_width,
                    stroke_line_cap: _,
                } => {
                    out.push_str(&format!(
                        " primitive={{x1Pt={:.3} y1Pt={:.3} x2Pt={:.3} y2Pt={:.3} stroke={} strokeWidth={:.3}}}",
                        x1_pt, y1_pt, x2_pt, y2_pt, stroke, stroke_width
                    ));
                }
                WireScenePrimitive::Rect {
                    x_pt,
                    y_pt,
                    width_pt,
                    height_pt,
                    fill,
                    stroke,
                    stroke_width,
                } => {
                    out.push_str(&format!(
                        " primitive={{xPt={:.3} yPt={:.3} widthPt={:.3} heightPt={:.3} fill={} stroke={} strokeWidth={}}}",
                        x_pt,
                        y_pt,
                        width_pt,
                        height_pt,
                        fill,
                        stroke.as_deref().unwrap_or("-"),
                        stroke_width.map(|value| format!("{value:.3}")).unwrap_or_else(|| "-".to_string())
                    ));
                }
                WireScenePrimitive::Polyline { points_pt } => {
                    let points = points_pt
                        .iter()
                        .map(|(x, y)| format!("{x:.3},{y:.3}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    out.push_str(&format!(" primitive={{pointsPt=[{}]}}", points));
                }
                WireScenePrimitive::Path {
                    d,
                    fill,
                    stroke,
                    stroke_width,
                } => {
                    out.push_str(&format!(
                        " primitive={{d={:?} fill={} stroke={} strokeWidth={}}}",
                        d,
                        fill,
                        stroke.as_deref().unwrap_or("-"),
                        stroke_width
                            .map(|value| format!("{value:.3}"))
                            .unwrap_or_else(|| "-".to_string())
                    ));
                }
            }
            out.push('\n');
        }
        for composite in &page.composites {
            out.push_str(&format!(
                "  composite id={} kind={} fragment={} childItemIds=[{}] label={} count={} startAnchorId={} endAnchorId={}\n",
                composite.id,
                composite.kind,
                composite.fragment,
                composite.child_item_ids.join(","),
                composite.label.as_deref().unwrap_or("-"),
                composite.count.map(|value| value.to_string()).unwrap_or_else(|| "-".to_string()),
                composite.start_anchor_id.as_deref().unwrap_or("-"),
                composite.end_anchor_id.as_deref().unwrap_or("-")
            ));
        }
    }
    out
}

pub fn layout_scene_to_js(scene: &LayoutScene) -> JsValue {
    let wire = to_wire_scene(scene);
    let result = Object::new();
    js_sys::Reflect::set(
        &result,
        &JsValue::from_str("version"),
        &JsValue::from_str(&wire.version),
    )
    .unwrap();
    js_sys::Reflect::set(
        &result,
        &JsValue::from_str("metricsVersion"),
        &JsValue::from_str(&wire.metrics_version),
    )
    .unwrap();

    let pages = Array::new();
    for page in wire.pages {
        let page_obj = Object::new();
        js_sys::Reflect::set(
            &page_obj,
            &JsValue::from_str("index"),
            &JsValue::from_f64(page.index as f64),
        )
        .unwrap();
        js_sys::Reflect::set(
            &page_obj,
            &JsValue::from_str("widthPt"),
            &JsValue::from_f64(page.width_pt as f64),
        )
        .unwrap();
        js_sys::Reflect::set(
            &page_obj,
            &JsValue::from_str("heightPt"),
            &JsValue::from_f64(page.height_pt as f64),
        )
        .unwrap();

        let systems = Array::new();
        for system in page.systems {
            let system_obj = Object::new();
            js_sys::Reflect::set(
                &system_obj,
                &JsValue::from_str("id"),
                &JsValue::from_str(&system.id),
            )
            .unwrap();
            js_sys::Reflect::set(
                &system_obj,
                &JsValue::from_str("index"),
                &JsValue::from_f64(system.index as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &system_obj,
                &JsValue::from_str("pageIndex"),
                &JsValue::from_f64(system.page_index as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &system_obj,
                &JsValue::from_str("xPt"),
                &JsValue::from_f64(system.x_pt as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &system_obj,
                &JsValue::from_str("yPt"),
                &JsValue::from_f64(system.y_pt as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &system_obj,
                &JsValue::from_str("widthPt"),
                &JsValue::from_f64(system.width_pt as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &system_obj,
                &JsValue::from_str("heightPt"),
                &JsValue::from_f64(system.height_pt as f64),
            )
            .unwrap();
            let measure_ids = Array::new();
            for measure_id in system.measure_ids {
                measure_ids.push(&JsValue::from_str(&measure_id));
            }
            js_sys::Reflect::set(
                &system_obj,
                &JsValue::from_str("measureIds"),
                &measure_ids.into(),
            )
            .unwrap();
            systems.push(&system_obj);
        }
        js_sys::Reflect::set(&page_obj, &JsValue::from_str("systems"), &systems.into()).unwrap();

        let measures = Array::new();
        for measure in page.measures {
            let measure_obj = Object::new();
            js_sys::Reflect::set(
                &measure_obj,
                &JsValue::from_str("id"),
                &JsValue::from_str(&measure.id),
            )
            .unwrap();
            js_sys::Reflect::set(
                &measure_obj,
                &JsValue::from_str("index"),
                &JsValue::from_f64(measure.index as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &measure_obj,
                &JsValue::from_str("globalIndex"),
                &JsValue::from_f64(measure.global_index as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &measure_obj,
                &JsValue::from_str("systemId"),
                &JsValue::from_str(&measure.system_id),
            )
            .unwrap();
            js_sys::Reflect::set(
                &measure_obj,
                &JsValue::from_str("xPt"),
                &JsValue::from_f64(measure.x_pt as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &measure_obj,
                &JsValue::from_str("yPt"),
                &JsValue::from_f64(measure.y_pt as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &measure_obj,
                &JsValue::from_str("widthPt"),
                &JsValue::from_f64(measure.width_pt as f64),
            )
            .unwrap();
            js_sys::Reflect::set(
                &measure_obj,
                &JsValue::from_str("heightPt"),
                &JsValue::from_f64(measure.height_pt as f64),
            )
            .unwrap();
            measures.push(&measure_obj);
        }
        js_sys::Reflect::set(&page_obj, &JsValue::from_str("measures"), &measures.into()).unwrap();

        let items = Array::new();
        for item in page.items {
            let item_obj = Object::new();
            js_sys::Reflect::set(
                &item_obj,
                &JsValue::from_str("id"),
                &JsValue::from_str(&item.id),
            )
            .unwrap();
            if let Some(measure_id) = item.measure_id {
                js_sys::Reflect::set(
                    &item_obj,
                    &JsValue::from_str("measureId"),
                    &JsValue::from_str(&measure_id),
                )
                .unwrap();
            }
            if let Some(anchor_item_id) = item.anchor_item_id {
                js_sys::Reflect::set(
                    &item_obj,
                    &JsValue::from_str("anchorItemId"),
                    &JsValue::from_str(&anchor_item_id),
                )
                .unwrap();
            }
            js_sys::Reflect::set(
                &item_obj,
                &JsValue::from_str("role"),
                &JsValue::from_str(&item.role),
            )
            .unwrap();
            js_sys::Reflect::set(
                &item_obj,
                &JsValue::from_str("kind"),
                &JsValue::from_str(item.kind),
            )
            .unwrap();
            js_sys::Reflect::set(
                &item_obj,
                &JsValue::from_str("zIndex"),
                &JsValue::from_f64(item.z_index as f64),
            )
            .unwrap();
            let primitive = Object::new();
            match item.primitive {
                WireScenePrimitive::GlyphRun {
                    x_pt,
                    y_pt,
                    glyph_role,
                    glyph_count,
                    codepoint,
                    font_family,
                    font_size_pt,
                    fill,
                } => {
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("xPt"),
                        &JsValue::from_f64(x_pt as f64),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("yPt"),
                        &JsValue::from_f64(y_pt as f64),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("glyphRole"),
                        &JsValue::from_str(glyph_role),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("glyphCount"),
                        &JsValue::from_f64(glyph_count as f64),
                    )
                    .unwrap();
                    if let Some(codepoint) = codepoint {
                        js_sys::Reflect::set(
                            &primitive,
                            &JsValue::from_str("codepoint"),
                            &JsValue::from_f64(codepoint as f64),
                        )
                        .unwrap();
                    }
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("fontFamily"),
                        &JsValue::from_str(&font_family),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("fontSizePt"),
                        &JsValue::from_f64(font_size_pt as f64),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("fill"),
                        &JsValue::from_str(&fill),
                    )
                    .unwrap();
                }
                WireScenePrimitive::TextRun {
                    x_pt,
                    y_pt,
                    text_role,
                    text,
                    font_family,
                    font_size_pt,
                    fill,
                    text_anchor,
                    font_weight,
                } => {
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("xPt"),
                        &JsValue::from_f64(x_pt as f64),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("yPt"),
                        &JsValue::from_f64(y_pt as f64),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("textRole"),
                        &JsValue::from_str(text_role),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("text"),
                        &JsValue::from_str(&text),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("fontFamily"),
                        &JsValue::from_str(&font_family),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("fontSizePt"),
                        &JsValue::from_f64(font_size_pt as f64),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("fill"),
                        &JsValue::from_str(&fill),
                    )
                    .unwrap();
                    if let Some(text_anchor) = text_anchor {
                        js_sys::Reflect::set(
                            &primitive,
                            &JsValue::from_str("textAnchor"),
                            &JsValue::from_str(&text_anchor),
                        )
                        .unwrap();
                    }
                    if let Some(font_weight) = font_weight {
                        js_sys::Reflect::set(
                            &primitive,
                            &JsValue::from_str("fontWeight"),
                            &JsValue::from_str(&font_weight),
                        )
                        .unwrap();
                    }
                }
                WireScenePrimitive::LineSegment {
                    x1_pt,
                    y1_pt,
                    x2_pt,
                    y2_pt,
                    stroke,
                    stroke_width,
                    stroke_line_cap,
                } => {
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("x1Pt"),
                        &JsValue::from_f64(x1_pt as f64),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("y1Pt"),
                        &JsValue::from_f64(y1_pt as f64),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("x2Pt"),
                        &JsValue::from_f64(x2_pt as f64),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("y2Pt"),
                        &JsValue::from_f64(y2_pt as f64),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("stroke"),
                        &JsValue::from_str(&stroke),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("strokeWidth"),
                        &JsValue::from_f64(stroke_width as f64),
                    )
                    .unwrap();
                    if let Some(cap) = stroke_line_cap {
                        js_sys::Reflect::set(
                            &primitive,
                            &JsValue::from_str("strokeLineCap"),
                            &JsValue::from_str(&cap),
                        )
                        .unwrap();
                    }
                }
                WireScenePrimitive::Rect {
                    x_pt,
                    y_pt,
                    width_pt,
                    height_pt,
                    fill,
                    stroke,
                    stroke_width,
                } => {
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("xPt"),
                        &JsValue::from_f64(x_pt as f64),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("yPt"),
                        &JsValue::from_f64(y_pt as f64),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("widthPt"),
                        &JsValue::from_f64(width_pt as f64),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("heightPt"),
                        &JsValue::from_f64(height_pt as f64),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("fill"),
                        &JsValue::from_str(&fill),
                    )
                    .unwrap();
                    if let Some(stroke) = stroke {
                        js_sys::Reflect::set(
                            &primitive,
                            &JsValue::from_str("stroke"),
                            &JsValue::from_str(&stroke),
                        )
                        .unwrap();
                    }
                    if let Some(stroke_width) = stroke_width {
                        js_sys::Reflect::set(
                            &primitive,
                            &JsValue::from_str("strokeWidth"),
                            &JsValue::from_f64(stroke_width as f64),
                        )
                        .unwrap();
                    }
                }
                WireScenePrimitive::Polyline { points_pt } => {
                    let points = Array::new();
                    for (x, y) in points_pt {
                        let point = Array::new();
                        point.push(&JsValue::from_f64(x as f64));
                        point.push(&JsValue::from_f64(y as f64));
                        points.push(&point.into());
                    }
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("pointsPt"),
                        &points.into(),
                    )
                    .unwrap();
                }
                WireScenePrimitive::Path {
                    d,
                    fill,
                    stroke,
                    stroke_width,
                } => {
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("d"),
                        &JsValue::from_str(&d),
                    )
                    .unwrap();
                    js_sys::Reflect::set(
                        &primitive,
                        &JsValue::from_str("fill"),
                        &JsValue::from_str(&fill),
                    )
                    .unwrap();
                    if let Some(stroke) = stroke {
                        js_sys::Reflect::set(
                            &primitive,
                            &JsValue::from_str("stroke"),
                            &JsValue::from_str(&stroke),
                        )
                        .unwrap();
                    }
                    if let Some(stroke_width) = stroke_width {
                        js_sys::Reflect::set(
                            &primitive,
                            &JsValue::from_str("strokeWidth"),
                            &JsValue::from_f64(stroke_width as f64),
                        )
                        .unwrap();
                    }
                }
            }
            js_sys::Reflect::set(
                &item_obj,
                &JsValue::from_str("primitive"),
                &primitive.into(),
            )
            .unwrap();
            items.push(&item_obj);
        }
        js_sys::Reflect::set(&page_obj, &JsValue::from_str("items"), &items.into()).unwrap();

        let composites = Array::new();
        for composite in page.composites {
            let composite_obj = Object::new();
            js_sys::Reflect::set(
                &composite_obj,
                &JsValue::from_str("id"),
                &JsValue::from_str(&composite.id),
            )
            .unwrap();
            js_sys::Reflect::set(
                &composite_obj,
                &JsValue::from_str("kind"),
                &JsValue::from_str(composite.kind),
            )
            .unwrap();
            js_sys::Reflect::set(
                &composite_obj,
                &JsValue::from_str("fragment"),
                &JsValue::from_str(composite.fragment),
            )
            .unwrap();
            let child_ids = Array::new();
            for child_id in composite.child_item_ids {
                child_ids.push(&JsValue::from_str(&child_id));
            }
            js_sys::Reflect::set(
                &composite_obj,
                &JsValue::from_str("childItemIds"),
                &child_ids.into(),
            )
            .unwrap();
            if let Some(label) = composite.label {
                js_sys::Reflect::set(
                    &composite_obj,
                    &JsValue::from_str("label"),
                    &JsValue::from_str(&label),
                )
                .unwrap();
            }
            if let Some(count) = composite.count {
                js_sys::Reflect::set(
                    &composite_obj,
                    &JsValue::from_str("count"),
                    &JsValue::from_f64(count as f64),
                )
                .unwrap();
            }
            if let Some(start_anchor_id) = composite.start_anchor_id {
                js_sys::Reflect::set(
                    &composite_obj,
                    &JsValue::from_str("startAnchorId"),
                    &JsValue::from_str(&start_anchor_id),
                )
                .unwrap();
            }
            if let Some(end_anchor_id) = composite.end_anchor_id {
                js_sys::Reflect::set(
                    &composite_obj,
                    &JsValue::from_str("endAnchorId"),
                    &JsValue::from_str(&end_anchor_id),
                )
                .unwrap();
            }
            composites.push(&composite_obj);
        }
        js_sys::Reflect::set(
            &page_obj,
            &JsValue::from_str("composites"),
            &composites.into(),
        )
        .unwrap();
        pages.push(&page_obj);
    }
    js_sys::Reflect::set(&result, &JsValue::from_str("pages"), &pages.into()).unwrap();

    let issues = Array::new();
    for issue in wire.issues {
        issues.push(&JsValue::from_str(&issue));
    }
    js_sys::Reflect::set(&result, &JsValue::from_str("issues"), &issues.into()).unwrap();
    result.into()
}

// ── LayoutPlan Tests ─────────────────────────────────────────────

#[test]
fn test_slot_mapper() {
    let m = SlotMapper::new(80.0);
    let width = m.measure_width(16, 4, false);
    assert!(width > 200.0, "measure with 16 slots should be >200px");
}

#[test]
fn test_place_notes() {
    let measure = NormalizedMeasure {
        index: 0,
        global_index: 0,
        paragraph_index: 0,
        measure_in_paragraph: 0,
        source_line: 1,
        events: vec![NormalizedEvent {
            track: "HH".into(),
            start: Fraction {
                numerator: 0,
                denominator: 1,
            },
            track_family: "cymbal".into(),
            duration: Fraction {
                numerator: 1,
                denominator: 8,
            },
            kind: EventKind::Hit,
            glyph: "x".into(),
            modifiers: vec![],
            modifier: None,
            voice: 1,
            beam: "none".into(),
            tuplet: None,
        }],
        barline: Some("regular".into()),
        closing_barline: Some("regular".into()),
        start_nav: None,
        end_nav: None,
        volta_indices: None,
        hairpins: vec![],
        measure_repeat_slashes: None,
        multi_rest_count: None,
        note_value: 8,
        volta_terminator: false,
    };
    let mapper = SlotMapper::new(80.0);
    let opts = LayoutOptions::default();
    let elements = place_notes(&measure, &mapper, &opts);
    assert_eq!(elements.len(), 1);
    assert_eq!(elements[0].kind, ElementKind::Note);
    assert_eq!(elements[0].smufl_codepoint, Some(0xE0A9));
}

#[test]
fn test_stacking_no_overlap() {
    let mut elements = vec![
        LayoutElement {
            kind: ElementKind::NavMarker,
            x: 50.0,
            y: -15.0,
            width: 10.0,
            height: 10.0,
            smufl_codepoint: None,
            voice: None,
            stem_up: None,
            barline_type: None,
            text: None,
            from_x: None,
            to_x: None,
            priority: 6,
            can_shift_y: true,
            can_shift_x: false,
        },
        LayoutElement {
            kind: ElementKind::Volta,
            x: 50.0,
            y: -20.0,
            width: 100.0,
            height: 8.0,
            smufl_codepoint: None,
            voice: None,
            stem_up: None,
            barline_type: None,
            text: None,
            from_x: None,
            to_x: None,
            priority: 7,
            can_shift_y: false,
            can_shift_x: false,
        },
    ];
    let warnings = stack_edge_elements(&mut elements, 4.0);
    assert!(warnings.is_empty(), "unexpected warnings: {:?}", warnings);
    // Nav should be pushed above volta
    assert!(elements[0].y < -20.0, "nav should be above volta");
}

#[test]
fn test_barlines() {
    let measure = NormalizedMeasure {
        index: 0,
        global_index: 0,
        paragraph_index: 0,
        measure_in_paragraph: 0,
        source_line: 1,
        events: vec![],
        barline: Some("|:".into()),
        closing_barline: Some("|:".into()),
        start_nav: None,
        end_nav: None,
        volta_indices: None,
        hairpins: vec![],
        measure_repeat_slashes: None,
        multi_rest_count: None,
        note_value: 8,
        volta_terminator: false,
    };
    let elements = place_barlines(&measure, 50.0);
    assert_eq!(elements.len(), 1);
    assert_eq!(elements[0].kind, ElementKind::Barline);
    assert_eq!(elements[0].barline_type.as_deref(), Some("|:"));
}

#[test]
fn test_contract_scene_smoke() {
    let score = RenderScore {
        version: RENDER_SCORE_VERSION.to_string(),
        header: RenderHeader {
            tempo: 120,
            time_beats: 4,
            time_beat_unit: 4,
            divisions: 16,
            note_value: 8,
            grouping: vec![1, 1, 1, 1],
            title: Some("Smoke".into()),
            subtitle: None,
            composer: None,
        },
        tracks: vec![RenderTrack {
            id: "HH".into(),
            family: "cymbal".into(),
        }],
        measures: vec![RenderMeasure {
            index: 0,
            global_index: 0,
            paragraph_index: 0,
            measure_in_paragraph: 0,
            source_line: 1,
            events: vec![RenderEvent {
                track: "HH".into(),
                track_family: "cymbal".into(),
                start: Fraction {
                    numerator: 0,
                    denominator: 1,
                },
                duration: Fraction {
                    numerator: 1,
                    denominator: 8,
                },
                kind: EventKind::Hit,
                glyph: "x".into(),
                modifiers: vec![],
                modifier: None,
                voice: 1,
                beam: "none".into(),
                tuplet: None,
            }],
            barline: Some("regular".into()),
            closing_barline: Some("regular".into()),
            start_nav: None,
            end_nav: None,
            volta_indices: None,
            hairpins: vec![],
            measure_repeat_slashes: None,
            multi_rest_count: None,
            note_value: 8,
            volta_terminator: false,
        }],
        errors: vec![],
        repeat_spans: vec![RepeatSpan {
            start_measure: 0,
            end_measure: 0,
            times: 2,
        }],
    };
    let scene = build_layout_scene(&score, &LayoutOptions::default());
    assert_eq!(scene.version, LAYOUT_SCENE_VERSION);
    assert_eq!(scene.metrics_version, CANONICAL_METRICS_VERSION);
    assert_eq!(scene.pages.len(), 1);
    assert_eq!(scene.pages[0].systems.len(), 1);
    assert_eq!(scene.pages[0].measures.len(), 1);
    assert_eq!(scene.pages[0].measures[0].index, 0);
    assert!(scene.pages[0]
        .composites
        .iter()
        .all(|c| c.kind != CompositeKind::RepeatSpan));
    assert!(scene.pages[0]
        .items
        .iter()
        .all(|item| !item.role.starts_with("repeat-span")));
    assert!(scene.pages[0]
        .composites
        .iter()
        .any(|c| c.kind == CompositeKind::TextBlock && c.label.as_deref() == Some("title")));
    assert!(scene.pages[0]
        .composites
        .iter()
        .any(|c| c.kind == CompositeKind::TextBlock && c.label.as_deref() == Some("tempo")));
    assert!(scene.pages[0]
        .items
        .iter()
        .filter(|item| { matches!(item.role.as_str(), "tempo-glyph" | "tempo-equals" | "tempo") })
        .all(|item| item.measure_id.as_deref() == Some("measure-0")));
}

#[test]
fn test_system_box_pagination_contracts_and_overflow_warning_schema() {
    let system_box = SystemLayoutBox {
        system_index: 2,
        system_id: "system-2".into(),
        local_system_origin_y: 12.0,
        staff_top: 22.0,
        staff_bottom: 62.0,
        visual_top: -8.0,
        visual_bottom: 80.0,
        width_pt: 500.0,
        measures: vec![SceneMeasure {
            id: "measure-7".into(),
            index: 7,
            global_index: 7,
            system_id: "system-2".into(),
            x_pt: 0.0,
            y_pt: 12.0,
            width_pt: 100.0,
            height_pt: 50.0,
        }],
        systems: vec![SceneSystem {
            id: "system-2".into(),
            index: 2,
            page_index: 0,
            x_pt: 0.0,
            y_pt: 12.0,
            width_pt: 500.0,
            height_pt: 50.0,
            measure_ids: vec!["measure-7".into()],
        }],
        items: Vec::new(),
        composites: Vec::new(),
    };
    assert_eq!(system_box.visual_bottom - system_box.visual_top, 88.0);
    assert_eq!(system_box.local_system_origin_y, 12.0);

    let header_box = HeaderLayoutBox {
        items: Vec::new(),
        composites: Vec::new(),
        visual_top: 10.0,
        visual_bottom: 92.0,
    };
    assert_eq!(header_box.visual_bottom, 92.0);

    let placed = PlacedSystemBox {
        system_index: system_box.system_index,
        system_id: system_box.system_id.clone(),
        page_index: 1,
        page_x: 50.0,
        page_y: 120.0,
        local_visual_top: system_box.visual_top,
        local_system_origin_y: system_box.local_system_origin_y,
        width_pt: system_box.width_pt,
        measure_ids: system_box.systems[0].measure_ids.clone(),
    };
    assert_eq!(placed.page_x, 50.0);
    assert_eq!(placed.measure_ids, ["measure-7"]);

    let mut issues = vec!["Line 1: existing parser issue".to_string()];
    issues.push(layout_overflow_warning(1, &placed.system_id, 900.0, 700.0));
    assert_eq!(issues[0], "Line 1: existing parser issue");
    assert_eq!(
        issues[1],
        "LAYOUT_WARNING overflow page=1 system=system-2 visualHeight=900.00 availableHeight=700.00"
    );
}

#[test]
fn test_scene_item_bounds_cover_emitted_primitive_kinds() {
    let text = SceneItem {
        id: "text".into(),
        measure_id: None,
        anchor_item_id: None,
        role: "title".into(),
        kind: SceneItemKind::TextRun,
        z_index: 0,
        primitive: ScenePrimitive::TextRun(TextRun {
            x_pt: 100.0,
            y_pt: 50.0,
            text_role: TextRole::Title,
            text: "AB".into(),
            font_family: "Academico".into(),
            font_size_pt: 24.0,
            fill: "#333".into(),
            text_anchor: Some("middle".into()),
            font_weight: None,
        }),
    };
    let text_bounds = scene_item_bounds(&text).unwrap();
    assert_eq!(text_bounds.x, 89.0);
    assert_eq!(text_bounds.y, 32.0);
    assert_eq!(text_bounds.height, 28.0);

    let glyph = SceneItem {
        id: "glyph".into(),
        measure_id: None,
        anchor_item_id: None,
        role: "notehead".into(),
        kind: SceneItemKind::GlyphRun,
        z_index: 0,
        primitive: ScenePrimitive::GlyphRun(GlyphRun {
            x_pt: 10.0,
            y_pt: 20.0,
            glyph_role: GlyphRole::NoteheadBlack,
            glyph_count: 1,
            smufl_codepoint: Some(0xE0A4),
            font_family: "Bravura".into(),
            font_size_pt: 20.0,
            fill: "#333".into(),
        }),
    };
    let glyph_bounds = scene_item_bounds(&glyph).unwrap();
    assert_eq!(glyph_bounds.x, 10.0);
    assert_eq!(glyph_bounds.y, 17.5);
    assert!((glyph_bounds.width - 5.9).abs() < 0.001);

    let line = SceneItem {
        id: "line".into(),
        measure_id: None,
        anchor_item_id: None,
        role: "staff-line".into(),
        kind: SceneItemKind::LineSegment,
        z_index: 0,
        primitive: ScenePrimitive::LineSegment(LineSegment {
            x1_pt: 10.0,
            y1_pt: 20.0,
            x2_pt: 30.0,
            y2_pt: 20.0,
            stroke: "#333".into(),
            stroke_width: 2.0,
            stroke_line_cap: None,
        }),
    };
    assert_eq!(
        scene_item_bounds(&line).unwrap(),
        SceneItemBounds {
            x: 9.0,
            y: 19.0,
            width: 22.0,
            height: 2.0
        }
    );

    let rect = SceneItem {
        id: "rect".into(),
        measure_id: None,
        anchor_item_id: None,
        role: "beam".into(),
        kind: SceneItemKind::Rect,
        z_index: 0,
        primitive: ScenePrimitive::Rect(RectShape {
            x_pt: 4.0,
            y_pt: 5.0,
            width_pt: 10.0,
            height_pt: 3.0,
            fill: "#333".into(),
            stroke: Some("#333".into()),
            stroke_width: Some(2.0),
        }),
    };
    assert_eq!(
        scene_item_bounds(&rect).unwrap(),
        SceneItemBounds {
            x: 3.0,
            y: 4.0,
            width: 12.0,
            height: 5.0
        }
    );

    let polyline = SceneItem {
        id: "polyline".into(),
        measure_id: None,
        anchor_item_id: None,
        role: "shape".into(),
        kind: SceneItemKind::Polyline,
        z_index: 0,
        primitive: ScenePrimitive::Polyline(Polyline {
            points_pt: vec![(5.0, 12.0), (20.0, -2.0), (7.0, 4.0)],
        }),
    };
    assert_eq!(
        scene_item_bounds(&polyline).unwrap(),
        SceneItemBounds {
            x: 5.0,
            y: -2.0,
            width: 15.0,
            height: 14.0
        }
    );

    let path = SceneItem {
        id: "path".into(),
        measure_id: None,
        anchor_item_id: None,
        role: "beam".into(),
        kind: SceneItemKind::Path,
        z_index: 0,
        primitive: ScenePrimitive::Path(PathShape {
            d: "M 10 10 L 30 12 L 28 16 L 8 14 Z".into(),
            fill: "#333".into(),
            stroke: Some("#333".into()),
            stroke_width: Some(2.0),
        }),
    };
    assert_eq!(
        scene_item_bounds(&path).unwrap(),
        SceneItemBounds {
            x: 7.0,
            y: 9.0,
            width: 24.0,
            height: 8.0
        }
    );

    let empty_polyline = SceneItem {
        id: "empty".into(),
        measure_id: None,
        anchor_item_id: None,
        role: "shape".into(),
        kind: SceneItemKind::Polyline,
        z_index: 0,
        primitive: ScenePrimitive::Polyline(Polyline { points_pt: vec![] }),
    };
    assert!(scene_item_bounds(&empty_polyline).is_err());
}

#[test]
fn test_header_layout_box_bounds_and_page0_cursor_use_actual_visual_bottom() {
    let header = RenderHeader {
        tempo: 120,
        time_beats: 4,
        time_beat_unit: 4,
        divisions: 16,
        note_value: 8,
        grouping: vec![1, 1, 1, 1],
        title: Some("Title".into()),
        subtitle: Some("Subtitle".into()),
        composer: Some("Composer".into()),
    };
    let opts = LayoutOptions {
        top_margin_pt: 10.0,
        header_height_pt: 20.0,
        header_staff_spacing_pt: 8.0,
        tempo_offset_y: 40.0,
        ..LayoutOptions::default()
    };
    let header_box = render_header_layout_box(&header, &opts);

    assert_eq!(header_box.items.len(), 6);
    assert!(header_box
        .composites
        .iter()
        .any(|composite| composite.label.as_deref() == Some("tempo")));
    assert!(header_box.visual_bottom > opts.top_margin_pt + opts.header_height_pt);

    let fixed_cursor = opts.top_margin_pt + opts.header_height_pt + opts.header_staff_spacing_pt;
    let cursor = page0_first_system_cursor(&opts, &header_box);
    assert!(cursor > fixed_cursor);
    assert_eq!(
        cursor,
        header_box.visual_bottom + opts.header_staff_spacing_pt
    );

    let empty_header = RenderHeader {
        tempo: 0,
        title: None,
        subtitle: None,
        composer: None,
        ..header
    };
    let empty_box = render_header_layout_box(&empty_header, &opts);
    assert!(empty_box.items.is_empty());
    assert_eq!(page0_first_system_cursor(&opts, &empty_box), fixed_cursor);
}

#[test]
fn test_paginate_system_boxes_with_mock_boxes() {
    fn mock_box(index: u32, height: f32) -> SystemLayoutBox {
        let measure_id = format!("measure-{index}");
        SystemLayoutBox {
            system_index: index,
            system_id: format!("system-{index}"),
            local_system_origin_y: 10.0,
            staff_top: 20.0,
            staff_bottom: 60.0,
            visual_top: -5.0,
            visual_bottom: height - 5.0,
            width_pt: 160.0,
            measures: vec![SceneMeasure {
                id: measure_id.clone(),
                index,
                global_index: index,
                system_id: format!("system-{index}"),
                x_pt: 0.0,
                y_pt: 10.0,
                width_pt: 160.0,
                height_pt: 50.0,
            }],
            systems: Vec::new(),
            items: Vec::new(),
            composites: Vec::new(),
        }
    }

    let opts = LayoutOptions {
        page_width_pt: 200.0,
        page_height_pt: 220.0,
        top_margin_pt: 20.0,
        bottom_margin_pt: 20.0,
        left_margin_pt: 12.0,
        header_height_pt: 30.0,
        header_staff_spacing_pt: 10.0,
        system_spacing_pt: 8.0,
        ..LayoutOptions::default()
    };
    let header = HeaderLayoutBox {
        items: Vec::new(),
        composites: Vec::new(),
        visual_top: 0.0,
        visual_bottom: 80.0,
    };

    let result = paginate_system_boxes(
        &[mock_box(0, 40.0), mock_box(1, 60.0), mock_box(2, 60.0)],
        &header,
        &opts,
    );
    assert_eq!(result.issues, Vec::<String>::new());
    assert_eq!(result.placements[0].page_index, 0);
    assert_eq!(result.placements[0].page_y, 90.0);
    assert_eq!(result.placements[0].page_x, 12.0);
    assert_eq!(result.placements[1].page_index, 0);
    assert_eq!(result.placements[1].page_y, 138.0);
    assert_eq!(result.placements[2].page_index, 1);
    assert_eq!(result.placements[2].page_y, 20.0);

    let overflow = paginate_system_boxes(&[mock_box(9, 250.0)], &header, &opts);
    assert_eq!(overflow.placements[0].page_index, 0);
    assert_eq!(
        overflow.issues,
        ["LAYOUT_WARNING overflow page=0 system=system-9 visualHeight=250.00 availableHeight=180.00"]
    );
}

#[test]
fn test_final_scene_validator_checks_ids_and_page_local_references() {
    let mut scene = LayoutScene {
        version: LAYOUT_SCENE_VERSION.to_string(),
        metrics_version: CANONICAL_METRICS_VERSION.to_string(),
        pages: vec![ScenePage {
            index: 0,
            width_pt: 200.0,
            height_pt: 200.0,
            systems: vec![SceneSystem {
                id: "system-0".into(),
                index: 0,
                page_index: 0,
                x_pt: 10.0,
                y_pt: 40.0,
                width_pt: 100.0,
                height_pt: 50.0,
                measure_ids: vec!["measure-0".into()],
            }],
            measures: vec![SceneMeasure {
                id: "measure-0".into(),
                index: 0,
                global_index: 0,
                system_id: "system-0".into(),
                x_pt: 10.0,
                y_pt: 40.0,
                width_pt: 100.0,
                height_pt: 50.0,
            }],
            items: vec![SceneItem {
                id: "item-0".into(),
                measure_id: Some("measure-0".into()),
                anchor_item_id: None,
                role: "staff-line".into(),
                kind: SceneItemKind::LineSegment,
                z_index: 0,
                primitive: ScenePrimitive::LineSegment(LineSegment {
                    x1_pt: 10.0,
                    y1_pt: 50.0,
                    x2_pt: 110.0,
                    y2_pt: 50.0,
                    stroke: "#333".into(),
                    stroke_width: 1.0,
                    stroke_line_cap: None,
                }),
            }],
            composites: vec![SceneComposite {
                id: "composite-0".into(),
                kind: CompositeKind::Volta,
                fragment: SpanFragmentKind::SingleSegment,
                child_item_ids: vec!["item-0".into()],
                label: Some("1.".into()),
                count: None,
                start_anchor_id: Some("measure-0".into()),
                end_anchor_id: Some("measure-0".into()),
            }],
        }],
        issues: Vec::new(),
    };
    assert!(validate_layout_scene(&scene).is_empty());

    scene.pages[0].items[0].anchor_item_id = Some("missing".into());
    scene.pages[0].composites[0].end_anchor_id = Some("item-0".into());
    let duplicate_item = scene.pages[0].items[0].clone();
    scene.pages[0].items.push(duplicate_item);
    let diagnostics = validate_layout_scene(&scene).join("\n");
    assert!(diagnostics.contains("LAYOUT_ERROR item-anchor"));
    assert!(diagnostics.contains("LAYOUT_ERROR composite-anchor"));
    assert!(diagnostics.contains("LAYOUT_ERROR duplicate-item"));
}

#[test]
fn test_final_scene_validator_suppresses_only_named_overflow_system_bounds() {
    let mut scene = LayoutScene {
        version: LAYOUT_SCENE_VERSION.to_string(),
        metrics_version: CANONICAL_METRICS_VERSION.to_string(),
        pages: vec![ScenePage {
            index: 0,
            width_pt: 100.0,
            height_pt: 100.0,
            systems: vec![
                SceneSystem {
                    id: "system-0".into(),
                    index: 0,
                    page_index: 0,
                    x_pt: 0.0,
                    y_pt: 0.0,
                    width_pt: 100.0,
                    height_pt: 200.0,
                    measure_ids: vec!["measure-0".into()],
                },
                SceneSystem {
                    id: "system-1".into(),
                    index: 1,
                    page_index: 0,
                    x_pt: 0.0,
                    y_pt: 0.0,
                    width_pt: 100.0,
                    height_pt: 50.0,
                    measure_ids: vec!["measure-1".into()],
                },
            ],
            measures: vec![
                SceneMeasure {
                    id: "measure-0".into(),
                    index: 0,
                    global_index: 0,
                    system_id: "system-0".into(),
                    x_pt: 0.0,
                    y_pt: 0.0,
                    width_pt: 100.0,
                    height_pt: 200.0,
                },
                SceneMeasure {
                    id: "measure-1".into(),
                    index: 1,
                    global_index: 1,
                    system_id: "system-1".into(),
                    x_pt: 0.0,
                    y_pt: 0.0,
                    width_pt: 100.0,
                    height_pt: 50.0,
                },
            ],
            items: vec![
                SceneItem {
                    id: "system-0-item-0".into(),
                    measure_id: Some("measure-0".into()),
                    anchor_item_id: None,
                    role: "staff-line".into(),
                    kind: SceneItemKind::LineSegment,
                    z_index: 0,
                    primitive: ScenePrimitive::LineSegment(LineSegment {
                        x1_pt: 0.0,
                        y1_pt: 150.0,
                        x2_pt: 80.0,
                        y2_pt: 150.0,
                        stroke: "#333".into(),
                        stroke_width: 1.0,
                        stroke_line_cap: None,
                    }),
                },
                SceneItem {
                    id: "system-1-item-0".into(),
                    measure_id: Some("measure-1".into()),
                    anchor_item_id: None,
                    role: "staff-line".into(),
                    kind: SceneItemKind::LineSegment,
                    z_index: 0,
                    primitive: ScenePrimitive::LineSegment(LineSegment {
                        x1_pt: 0.0,
                        y1_pt: 120.0,
                        x2_pt: 80.0,
                        y2_pt: 120.0,
                        stroke: "#333".into(),
                        stroke_width: 1.0,
                        stroke_line_cap: None,
                    }),
                },
            ],
            composites: Vec::new(),
        }],
        issues: vec![layout_overflow_warning(0, "system-0", 200.0, 100.0)],
    };

    let diagnostics = validate_layout_scene(&scene).join("\n");
    assert!(!diagnostics.contains("system-0-item-0"));
    assert!(diagnostics.contains("system-1-item-0"));

    scene.issues.clear();
    let diagnostics = validate_layout_scene(&scene).join("\n");
    assert!(diagnostics.contains("system-0-item-0"));
    assert!(diagnostics.contains("system-1-item-0"));
}

#[test]
fn test_system_box_orchestrator_outputs_multiple_pages_for_long_scores() {
    let event = RenderEvent {
        track: "HH".into(),
        track_family: "cymbal".into(),
        start: Fraction {
            numerator: 0,
            denominator: 1,
        },
        duration: Fraction {
            numerator: 1,
            denominator: 4,
        },
        kind: EventKind::Hit,
        glyph: "x".into(),
        modifiers: vec![],
        modifier: None,
        voice: 1,
        beam: "none".into(),
        tuplet: None,
    };
    let measures = (0..8)
        .map(|index| RenderMeasure {
            index,
            global_index: index,
            paragraph_index: index,
            measure_in_paragraph: 0,
            source_line: index + 1,
            events: vec![event.clone()],
            barline: Some("regular".into()),
            closing_barline: Some("regular".into()),
            start_nav: None,
            end_nav: None,
            volta_indices: None,
            hairpins: vec![],
            measure_repeat_slashes: None,
            multi_rest_count: None,
            note_value: 4,
            volta_terminator: false,
        })
        .collect::<Vec<_>>();
    let score = RenderScore {
        version: RENDER_SCORE_VERSION.to_string(),
        header: RenderHeader {
            tempo: 120,
            time_beats: 4,
            time_beat_unit: 4,
            divisions: 4,
            note_value: 4,
            grouping: vec![1, 1, 1, 1],
            title: Some("Long".into()),
            subtitle: None,
            composer: None,
        },
        tracks: vec![RenderTrack {
            id: "HH".into(),
            family: "cymbal".into(),
        }],
        measures,
        errors: vec!["existing issue".into()],
        repeat_spans: vec![],
    };
    let scene = build_layout_scene(&score, &LayoutOptions::default());
    assert!(scene.pages.len() > 1);
    assert!(scene.issues.contains(&"existing issue".to_string()));
    assert!(!scene
        .issues
        .iter()
        .any(|issue| issue.starts_with("LAYOUT_ERROR")));
}

#[test]
fn test_volta_composites_are_emitted() {
    let score = RenderScore {
        version: RENDER_SCORE_VERSION.to_string(),
        header: RenderHeader {
            tempo: 120,
            time_beats: 4,
            time_beat_unit: 4,
            divisions: 16,
            note_value: 8,
            grouping: vec![1, 1, 1, 1],
            title: None,
            subtitle: None,
            composer: None,
        },
        tracks: vec![RenderTrack {
            id: "HH".into(),
            family: "cymbal".into(),
        }],
        measures: vec![
            RenderMeasure {
                index: 0,
                global_index: 0,
                paragraph_index: 0,
                measure_in_paragraph: 0,
                source_line: 1,
                events: vec![],
                barline: Some("repeat-start".into()),
                closing_barline: Some("repeat-start".into()),
                start_nav: None,
                end_nav: None,
                volta_indices: Some(vec![1]),
                hairpins: vec![],
                measure_repeat_slashes: None,
                multi_rest_count: None,
                note_value: 8,
                volta_terminator: false,
            },
            RenderMeasure {
                index: 1,
                global_index: 1,
                paragraph_index: 0,
                measure_in_paragraph: 1,
                source_line: 1,
                events: vec![],
                barline: Some("repeat-end".into()),
                closing_barline: Some("repeat-end".into()),
                start_nav: None,
                end_nav: None,
                volta_indices: Some(vec![1]),
                hairpins: vec![],
                measure_repeat_slashes: None,
                multi_rest_count: None,
                note_value: 8,
                volta_terminator: true,
            },
        ],
        errors: vec![],
        repeat_spans: vec![RepeatSpan {
            start_measure: 0,
            end_measure: 1,
            times: 2,
        }],
    };

    let scene = build_layout_scene(&score, &LayoutOptions::default());
    let voltas = scene.pages[0]
        .composites
        .iter()
        .filter(|composite| composite.kind == CompositeKind::Volta)
        .collect::<Vec<_>>();
    assert_eq!(voltas.len(), 1);
    assert_eq!(voltas[0].label.as_deref(), Some("1."));
    assert_eq!(voltas[0].fragment, SpanFragmentKind::SingleSegment);
    assert_eq!(voltas[0].start_anchor_id.as_deref(), Some("measure-0"));
    assert_eq!(voltas[0].end_anchor_id.as_deref(), Some("measure-1"));
}

#[test]
fn test_adjacent_voltas_share_y_and_positive_offset_moves_up() {
    let event = RenderEvent {
        track: "HH".into(),
        track_family: "cymbal".into(),
        start: Fraction {
            numerator: 0,
            denominator: 1,
        },
        duration: Fraction {
            numerator: 1,
            denominator: 4,
        },
        kind: EventKind::Hit,
        glyph: "x".into(),
        modifiers: vec![],
        modifier: None,
        voice: 1,
        beam: "none".into(),
        tuplet: None,
    };
    let measure = |index: u32, volta: u32| RenderMeasure {
        index,
        global_index: index,
        paragraph_index: 0,
        measure_in_paragraph: index,
        source_line: 1,
        events: vec![event.clone()],
        barline: Some("regular".into()),
        closing_barline: Some("regular".into()),
        start_nav: None,
        end_nav: None,
        volta_indices: Some(vec![volta]),
        hairpins: vec![],
        measure_repeat_slashes: None,
        multi_rest_count: None,
        note_value: 4,
        volta_terminator: false,
    };
    let score = RenderScore {
        version: RENDER_SCORE_VERSION.to_string(),
        header: RenderHeader {
            tempo: 120,
            time_beats: 4,
            time_beat_unit: 4,
            divisions: 4,
            note_value: 4,
            grouping: vec![1, 1, 1, 1],
            title: None,
            subtitle: None,
            composer: None,
        },
        tracks: vec![RenderTrack {
            id: "HH".into(),
            family: "cymbal".into(),
        }],
        measures: vec![measure(0, 1), measure(1, 2)],
        errors: vec![],
        repeat_spans: vec![],
    };

    let line_ys = |scene: &LayoutScene| {
        scene
            .pages
            .iter()
            .flat_map(|page| page.items.iter())
            .filter(|item| item.role == "volta-line")
            .filter_map(|item| match &item.primitive {
                ScenePrimitive::LineSegment(line) => Some(line.y1_pt),
                _ => None,
            })
            .collect::<Vec<_>>()
    };

    let default_scene = build_layout_scene(&score, &LayoutOptions::default());
    let default_ys = line_ys(&default_scene);
    assert_eq!(default_ys.len(), 2);
    assert!((default_ys[0] - default_ys[1]).abs() < 0.01);
    let stem_top = default_scene.pages[0]
        .items
        .iter()
        .filter(|item| item.role == "stem")
        .filter_map(|item| match &item.primitive {
            ScenePrimitive::LineSegment(line) => Some(line.y1_pt.min(line.y2_pt)),
            _ => None,
        })
        .fold(f32::INFINITY, f32::min);
    assert!(
        default_ys[0] <= stem_top - VOLTA_SKYLINE_GAP_PT - VOLTA_LINE_THICKNESS_PT + 0.01,
        "volta line should clear the note skyline"
    );
    assert!(
        default_ys[0] > stem_top - (VOLTA_LINE_HEIGHT_PT + VOLTA_TEXT_SIZE_PT + 2.0),
        "volta line should not reserve hook and text height above the skyline"
    );

    let spaced_opts = LayoutOptions {
        volta_offset_y: 10.0,
        ..LayoutOptions::default()
    };
    let spaced_scene = build_layout_scene(&score, &spaced_opts);
    let spaced_ys = line_ys(&spaced_scene);
    assert_eq!(spaced_ys.len(), 2);
    assert!((spaced_ys[0] - spaced_ys[1]).abs() < 0.01);
    assert!(spaced_ys[0].is_finite());
}

#[test]
fn test_two_bar_measure_repeat_expands_into_two_display_measures() {
    let score = RenderScore {
        version: RENDER_SCORE_VERSION.to_string(),
        header: RenderHeader {
            tempo: 120,
            time_beats: 4,
            time_beat_unit: 4,
            divisions: 16,
            note_value: 8,
            grouping: vec![1, 1, 1, 1],
            title: None,
            subtitle: None,
            composer: None,
        },
        tracks: vec![RenderTrack {
            id: "HH".into(),
            family: "cymbal".into(),
        }],
        measures: vec![
            RenderMeasure {
                index: 0,
                global_index: 0,
                paragraph_index: 0,
                measure_in_paragraph: 0,
                source_line: 1,
                events: vec![],
                barline: Some("regular".into()),
                closing_barline: Some("regular".into()),
                start_nav: None,
                end_nav: None,
                volta_indices: None,
                hairpins: vec![],
                measure_repeat_slashes: None,
                multi_rest_count: None,
                note_value: 8,
                volta_terminator: false,
            },
            RenderMeasure {
                index: 1,
                global_index: 1,
                paragraph_index: 0,
                measure_in_paragraph: 1,
                source_line: 1,
                events: vec![],
                barline: Some("regular".into()),
                closing_barline: Some("final".into()),
                start_nav: None,
                end_nav: None,
                volta_indices: None,
                hairpins: vec![],
                measure_repeat_slashes: Some(2),
                multi_rest_count: None,
                note_value: 8,
                volta_terminator: false,
            },
        ],
        errors: vec![],
        repeat_spans: vec![],
    };

    let scene = build_layout_scene(&score, &LayoutOptions::default());
    assert_eq!(scene.pages[0].measures.len(), 3);
    let repeat_items = scene.pages[0]
        .items
        .iter()
        .filter_map(|item| match &item.primitive {
            ScenePrimitive::GlyphRun(glyph) if item.role == "measure-repeat" => Some(glyph),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(repeat_items.len(), 1);
    assert_eq!(
        repeat_items[0].glyph_role,
        GlyphRole::MeasureRepeatMark2Bars
    );
    let repeat_composite = scene.pages[0]
        .composites
        .iter()
        .find(|composite| composite.kind == CompositeKind::MeasureRepeat)
        .expect("expected measure-repeat composite");
    assert_eq!(repeat_composite.count, Some(2));
    assert_eq!(
        repeat_composite.start_anchor_id.as_deref(),
        Some("measure-1")
    );
    assert_eq!(repeat_composite.end_anchor_id.as_deref(), Some("measure-2"));
}

#[test]
fn test_structural_composites_are_emitted() {
    let score = RenderScore {
        version: RENDER_SCORE_VERSION.to_string(),
        header: RenderHeader {
            tempo: 120,
            time_beats: 4,
            time_beat_unit: 4,
            divisions: 16,
            note_value: 8,
            grouping: vec![1, 1, 1, 1],
            title: None,
            subtitle: None,
            composer: None,
        },
        tracks: vec![RenderTrack {
            id: "HH".into(),
            family: "cymbal".into(),
        }],
        measures: vec![
            RenderMeasure {
                index: 0,
                global_index: 0,
                paragraph_index: 0,
                measure_in_paragraph: 0,
                source_line: 1,
                events: vec![],
                barline: Some("regular".into()),
                closing_barline: Some("regular".into()),
                start_nav: None,
                end_nav: None,
                volta_indices: None,
                hairpins: vec![HairpinSpan {
                    kind: HairpinKind::Crescendo,
                    start: Fraction {
                        numerator: 0,
                        denominator: 1,
                    },
                    end: Fraction {
                        numerator: 1,
                        denominator: 1,
                    },
                    start_measure_index: 0,
                    end_measure_index: 1,
                }],
                measure_repeat_slashes: Some(2),
                multi_rest_count: None,
                note_value: 8,
                volta_terminator: false,
            },
            RenderMeasure {
                index: 1,
                global_index: 1,
                paragraph_index: 0,
                measure_in_paragraph: 1,
                source_line: 1,
                events: vec![],
                barline: Some("regular".into()),
                closing_barline: Some("regular".into()),
                start_nav: None,
                end_nav: None,
                volta_indices: None,
                hairpins: vec![],
                measure_repeat_slashes: None,
                multi_rest_count: Some(4),
                note_value: 8,
                volta_terminator: false,
            },
        ],
        errors: vec![],
        repeat_spans: vec![],
    };

    let scene = build_layout_scene(&score, &LayoutOptions::default());
    let hairpin = scene.pages[0]
        .composites
        .iter()
        .find(|c| c.kind == CompositeKind::Hairpin)
        .expect("expected hairpin composite");
    assert_eq!(hairpin.fragment, SpanFragmentKind::SingleSegment);
    assert_eq!(hairpin.label.as_deref(), Some("crescendo"));
    assert_eq!(hairpin.start_anchor_id.as_deref(), Some("measure-0"));
    assert_eq!(hairpin.end_anchor_id.as_deref(), Some("measure-2"));
    assert!(scene.pages[0]
        .composites
        .iter()
        .any(|c| c.kind == CompositeKind::MeasureRepeat && c.count == Some(2)));
    assert!(scene.pages[0]
        .composites
        .iter()
        .any(|c| c.kind == CompositeKind::MultiRest && c.count == Some(4)));
}

#[test]
fn test_system_boundaries_align_with_staff_edges() {
    let score = RenderScore {
        version: RENDER_SCORE_VERSION.to_string(),
        header: RenderHeader {
            tempo: 120,
            time_beats: 4,
            time_beat_unit: 4,
            divisions: 16,
            note_value: 8,
            grouping: vec![1, 1, 1, 1],
            title: None,
            subtitle: None,
            composer: None,
        },
        tracks: vec![RenderTrack {
            id: "HH".into(),
            family: "cymbal".into(),
        }],
        measures: vec![RenderMeasure {
            index: 0,
            global_index: 0,
            paragraph_index: 0,
            measure_in_paragraph: 0,
            source_line: 1,
            events: vec![RenderEvent {
                track: "HH".into(),
                track_family: "cymbal".into(),
                start: Fraction {
                    numerator: 0,
                    denominator: 1,
                },
                duration: Fraction {
                    numerator: 1,
                    denominator: 4,
                },
                kind: EventKind::Hit,
                glyph: "x".into(),
                modifiers: vec![],
                modifier: None,
                voice: 1,
                beam: "none".into(),
                tuplet: None,
            }],
            barline: Some("regular".into()),
            closing_barline: Some("final".into()),
            start_nav: None,
            end_nav: None,
            volta_indices: None,
            hairpins: vec![],
            measure_repeat_slashes: None,
            multi_rest_count: None,
            note_value: 8,
            volta_terminator: false,
        }],
        errors: vec![],
        repeat_spans: vec![],
    };

    let opts = LayoutOptions::default();
    let scene = build_layout_scene(&score, &opts);
    let opening = scene.pages[0]
        .items
        .iter()
        .find(|item| item.role == "opening-barline")
        .expect("expected opening barline");
    let final_thick = scene.pages[0]
        .items
        .iter()
        .find(|item| item.role == "final-barline-thick")
        .expect("expected final thick barline");

    match (&opening.primitive, &final_thick.primitive) {
        (ScenePrimitive::Rect(opening_rect), ScenePrimitive::Rect(final_rect)) => {
            assert!((opening_rect.x_pt - opts.left_margin_pt).abs() < 0.01);
            assert!(
                ((final_rect.x_pt + final_rect.width_pt)
                    - (opts.page_width_pt - opts.right_margin_pt))
                    .abs()
                    < 0.01
            );
        }
        _ => panic!("barlines should be rectangles"),
    }
}

#[test]
fn test_first_measure_repeat_start_sits_after_system_preamble() {
    let score = RenderScore {
        version: RENDER_SCORE_VERSION.to_string(),
        header: RenderHeader {
            tempo: 120,
            time_beats: 4,
            time_beat_unit: 4,
            divisions: 16,
            note_value: 8,
            grouping: vec![1, 1, 1, 1],
            title: None,
            subtitle: None,
            composer: None,
        },
        tracks: vec![RenderTrack {
            id: "SD".into(),
            family: "drum".into(),
        }],
        measures: vec![RenderMeasure {
            index: 0,
            global_index: 0,
            paragraph_index: 0,
            measure_in_paragraph: 0,
            source_line: 1,
            events: vec![RenderEvent {
                track: "SD".into(),
                track_family: "drum".into(),
                start: Fraction {
                    numerator: 0,
                    denominator: 1,
                },
                duration: Fraction {
                    numerator: 1,
                    denominator: 4,
                },
                kind: EventKind::Hit,
                glyph: "d".into(),
                modifiers: vec![],
                modifier: None,
                voice: 1,
                beam: "none".into(),
                tuplet: None,
            }],
            barline: Some("repeat-start".into()),
            closing_barline: Some("final".into()),
            start_nav: None,
            end_nav: None,
            volta_indices: None,
            hairpins: vec![],
            measure_repeat_slashes: None,
            multi_rest_count: None,
            note_value: 8,
            volta_terminator: false,
        }],
        errors: vec![],
        repeat_spans: vec![],
    };

    let opts = LayoutOptions::default();
    let scene = build_layout_scene(&score, &opts);
    let opening = scene.pages[0]
        .items
        .iter()
        .find(|item| item.role == "opening-barline")
        .expect("expected system opening barline");
    let repeat_start = scene.pages[0]
        .items
        .iter()
        .find(|item| item.role == "repeat-start")
        .expect("expected start repeat barline");
    let notehead = scene.pages[0]
        .items
        .iter()
        .find(|item| item.role == "notehead")
        .expect("expected notehead");

    let ScenePrimitive::Rect(opening_rect) = &opening.primitive else {
        panic!("opening barline should be a rect");
    };
    let ScenePrimitive::GlyphRun(repeat_glyph) = &repeat_start.primitive else {
        panic!("repeat start should be a glyph");
    };
    let (note_x, _, _, _) = item_bounds(notehead).expect("notehead should have bounds");
    let repeat_top = repeat_glyph.y_pt - repeat_barline_rendered_height(GlyphRole::RepeatLeft);
    let repeat_bottom = repeat_glyph.y_pt;

    assert!((opening_rect.x_pt - opts.left_margin_pt).abs() < 0.01);
    assert_eq!(repeat_glyph.glyph_role, GlyphRole::RepeatLeft);
    assert_eq!(repeat_glyph.font_size_pt, REPEAT_BARLINE_FONT_SIZE_PT);
    assert!(repeat_glyph.x_pt > opening_rect.x_pt + 60.0);
    assert!((repeat_top - opening_rect.y_pt).abs() < 0.01);
    assert!((repeat_bottom - (opening_rect.y_pt + opening_rect.height_pt - 1.0)).abs() < 0.01);
    assert!(note_x > repeat_glyph.x_pt + repeat_barline_rendered_width(GlyphRole::RepeatLeft));
}

#[test]
fn test_later_system_uses_smaller_start_zone_than_first_system() {
    let measures = [0_u32, 1_u32]
        .into_iter()
        .map(|index| RenderMeasure {
            index,
            global_index: index,
            paragraph_index: index,
            measure_in_paragraph: 0,
            source_line: 1,
            events: vec![RenderEvent {
                track: "HH".into(),
                track_family: "cymbal".into(),
                start: Fraction {
                    numerator: 0,
                    denominator: 1,
                },
                duration: Fraction {
                    numerator: 1,
                    denominator: 4,
                },
                kind: EventKind::Hit,
                glyph: "x".into(),
                modifiers: vec![],
                modifier: None,
                voice: 1,
                beam: "none".into(),
                tuplet: None,
            }],
            barline: Some("regular".into()),
            closing_barline: Some(if index == 1 {
                "final".into()
            } else {
                "regular".into()
            }),
            start_nav: None,
            end_nav: None,
            volta_indices: None,
            hairpins: vec![],
            measure_repeat_slashes: None,
            multi_rest_count: None,
            note_value: 8,
            volta_terminator: false,
        })
        .collect();

    let score = RenderScore {
        version: RENDER_SCORE_VERSION.to_string(),
        header: RenderHeader {
            tempo: 120,
            time_beats: 4,
            time_beat_unit: 4,
            divisions: 16,
            note_value: 8,
            grouping: vec![1, 1, 1, 1],
            title: None,
            subtitle: None,
            composer: None,
        },
        tracks: vec![RenderTrack {
            id: "HH".into(),
            family: "cymbal".into(),
        }],
        measures,
        errors: vec![],
        repeat_spans: vec![],
    };

    let scene = build_layout_scene(&score, &LayoutOptions::default());
    let first_x = scene.pages[0]
        .items
        .iter()
        .find(|item| item.role == "notehead" && item.measure_id.as_deref() == Some("measure-0"))
        .and_then(|item| match &item.primitive {
            ScenePrimitive::TextRun(text) => Some(text.x_pt),
            _ => None,
        })
        .expect("expected first-system notehead");
    let second_x = scene.pages[0]
        .items
        .iter()
        .find(|item| item.role == "notehead" && item.measure_id.as_deref() == Some("measure-1"))
        .and_then(|item| match &item.primitive {
            ScenePrimitive::TextRun(text) => Some(text.x_pt),
            _ => None,
        })
        .expect("expected later-system notehead");

    assert!(
        second_x < first_x,
        "later systems should not retain first-system time-signature padding"
    );
}

#[test]
fn test_later_system_measure_number_uses_absolute_measure_index() {
    let score = RenderScore {
        version: RENDER_SCORE_VERSION.to_string(),
        header: RenderHeader {
            tempo: 120,
            time_beats: 4,
            time_beat_unit: 4,
            divisions: 16,
            note_value: 8,
            grouping: vec![1, 1, 1, 1],
            title: None,
            subtitle: None,
            composer: None,
        },
        tracks: vec![RenderTrack {
            id: "HH".into(),
            family: "cymbal".into(),
        }],
        measures: vec![
            RenderMeasure {
                index: 3,
                global_index: 3,
                paragraph_index: 0,
                measure_in_paragraph: 0,
                source_line: 1,
                events: vec![],
                barline: Some("regular".into()),
                closing_barline: Some("regular".into()),
                start_nav: None,
                end_nav: None,
                volta_indices: None,
                hairpins: vec![],
                measure_repeat_slashes: None,
                multi_rest_count: None,
                note_value: 8,
                volta_terminator: false,
            },
            RenderMeasure {
                index: 7,
                global_index: 7,
                paragraph_index: 1,
                measure_in_paragraph: 0,
                source_line: 2,
                events: vec![],
                barline: Some("final".into()),
                closing_barline: Some("final".into()),
                start_nav: None,
                end_nav: None,
                volta_indices: None,
                hairpins: vec![],
                measure_repeat_slashes: None,
                multi_rest_count: None,
                note_value: 8,
                volta_terminator: false,
            },
        ],
        errors: vec![],
        repeat_spans: vec![],
    };

    let scene = build_layout_scene(&score, &LayoutOptions::default());
    let measure_number = scene.pages[0]
        .items
        .iter()
        .find(|item| item.role == "measure-number")
        .expect("expected measure number on later system");
    let ScenePrimitive::TextRun(text) = &measure_number.primitive else {
        panic!("measure number should be text");
    };
    assert_eq!(text.text, "8");
}

#[test]
fn test_down_stem_keeps_notehead_on_right_and_flag_on_stem_right_side() {
    let score = RenderScore {
        version: RENDER_SCORE_VERSION.to_string(),
        header: RenderHeader {
            tempo: 120,
            time_beats: 4,
            time_beat_unit: 4,
            divisions: 16,
            note_value: 8,
            grouping: vec![1, 1, 1, 1],
            title: None,
            subtitle: None,
            composer: None,
        },
        tracks: vec![RenderTrack {
            id: "BD".into(),
            family: "drum".into(),
        }],
        measures: vec![RenderMeasure {
            index: 0,
            global_index: 0,
            paragraph_index: 0,
            measure_in_paragraph: 0,
            source_line: 1,
            events: vec![RenderEvent {
                track: "BD".into(),
                track_family: "drum".into(),
                start: Fraction {
                    numerator: 0,
                    denominator: 1,
                },
                duration: Fraction {
                    numerator: 1,
                    denominator: 8,
                },
                kind: EventKind::Hit,
                glyph: "d".into(),
                modifiers: vec![],
                modifier: None,
                voice: 2,
                beam: "none".into(),
                tuplet: None,
            }],
            barline: Some("final".into()),
            closing_barline: Some("final".into()),
            start_nav: None,
            end_nav: None,
            volta_indices: None,
            hairpins: vec![],
            measure_repeat_slashes: None,
            multi_rest_count: None,
            note_value: 8,
            volta_terminator: false,
        }],
        errors: vec![],
        repeat_spans: vec![],
    };

    let scene = build_layout_scene(&score, &LayoutOptions::default());
    let notehead = scene.pages[0]
        .items
        .iter()
        .find(|item| item.role == "notehead")
        .expect("expected notehead");
    let stem = scene.pages[0]
        .items
        .iter()
        .find(|item| item.role == "stem")
        .expect("expected stem");
    let flag = scene.pages[0]
        .items
        .iter()
        .find(|item| item.role == "flag")
        .expect("expected flag");

    let note_x = match &notehead.primitive {
        ScenePrimitive::TextRun(text) => text.x_pt,
        _ => panic!("notehead should be text"),
    };
    let stem_x = match &stem.primitive {
        ScenePrimitive::LineSegment(line) => line.x1_pt,
        _ => panic!("stem should be line"),
    };
    let (flag_x, flag_role) = match &flag.primitive {
        ScenePrimitive::GlyphRun(glyph) => (glyph.x_pt, glyph.glyph_role),
        _ => panic!("flag should be glyph"),
    };

    assert!(
        stem_x < note_x + 4.0,
        "down stem should anchor on the notehead left side"
    );
    assert!(
        flag_x >= stem_x - 0.75,
        "down flag glyph should start on the stem and extend on its right side"
    );
    assert_eq!(flag_role, GlyphRole::Flag8thDown);
}

#[test]
fn test_crash_maps_to_top_ledger_line() {
    let score = RenderScore {
        version: RENDER_SCORE_VERSION.to_string(),
        header: RenderHeader {
            tempo: 120,
            time_beats: 4,
            time_beat_unit: 4,
            divisions: 16,
            note_value: 8,
            grouping: vec![1, 1, 1, 1],
            title: None,
            subtitle: None,
            composer: None,
        },
        tracks: vec![RenderTrack {
            id: "C".into(),
            family: "cymbal".into(),
        }],
        measures: vec![RenderMeasure {
            index: 0,
            global_index: 0,
            paragraph_index: 0,
            measure_in_paragraph: 0,
            source_line: 1,
            events: vec![RenderEvent {
                track: "C".into(),
                track_family: "cymbal".into(),
                start: Fraction {
                    numerator: 0,
                    denominator: 1,
                },
                duration: Fraction {
                    numerator: 1,
                    denominator: 4,
                },
                kind: EventKind::Hit,
                glyph: "x".into(),
                modifiers: vec![],
                modifier: None,
                voice: 1,
                beam: "none".into(),
                tuplet: None,
            }],
            barline: Some("final".into()),
            closing_barline: Some("final".into()),
            start_nav: None,
            end_nav: None,
            volta_indices: None,
            hairpins: vec![],
            measure_repeat_slashes: None,
            multi_rest_count: None,
            note_value: 8,
            volta_terminator: false,
        }],
        errors: vec![],
        repeat_spans: vec![],
    };

    let scene = build_layout_scene(&score, &LayoutOptions::default());
    let notehead_y = scene.pages[0]
        .items
        .iter()
        .find(|item| item.role == "notehead")
        .and_then(|item| match &item.primitive {
            ScenePrimitive::TextRun(text) => Some(text.y_pt),
            _ => None,
        })
        .expect("expected crash notehead");
    let staff_top = scene.pages[0]
        .items
        .iter()
        .find(|item| item.role == "staff-line")
        .and_then(|item| match &item.primitive {
            ScenePrimitive::LineSegment(line) => Some(line.y1_pt),
            _ => None,
        })
        .expect("expected staff line");
    let ledger_y = scene.pages[0]
        .items
        .iter()
        .find(|item| item.role == "ledger-line")
        .and_then(|item| match &item.primitive {
            ScenePrimitive::LineSegment(line) => Some(line.y1_pt),
            _ => None,
        })
        .expect("expected top ledger line");

    assert!((notehead_y - (staff_top - 10.0)).abs() < 0.01);
    assert!((ledger_y - (staff_top - 10.0)).abs() < 0.01);
}

#[test]
fn test_bottom_ledger_lines_render_for_notes_below_staff() {
    let score = RenderScore {
        version: RENDER_SCORE_VERSION.to_string(),
        header: RenderHeader {
            tempo: 120,
            time_beats: 4,
            time_beat_unit: 4,
            divisions: 16,
            note_value: 8,
            grouping: vec![1, 1, 1, 1],
            title: None,
            subtitle: None,
            composer: None,
        },
        tracks: vec![RenderTrack {
            id: "WB".into(),
            family: "percussion".into(),
        }],
        measures: vec![RenderMeasure {
            index: 0,
            global_index: 0,
            paragraph_index: 0,
            measure_in_paragraph: 0,
            source_line: 1,
            events: vec![RenderEvent {
                track: "WB".into(),
                track_family: "percussion".into(),
                start: Fraction {
                    numerator: 0,
                    denominator: 1,
                },
                duration: Fraction {
                    numerator: 1,
                    denominator: 4,
                },
                kind: EventKind::Hit,
                glyph: "d".into(),
                modifiers: vec![],
                modifier: None,
                voice: 1,
                beam: "none".into(),
                tuplet: None,
            }],
            barline: Some("final".into()),
            closing_barline: Some("final".into()),
            start_nav: None,
            end_nav: None,
            volta_indices: None,
            hairpins: vec![],
            measure_repeat_slashes: None,
            multi_rest_count: None,
            note_value: 8,
            volta_terminator: false,
        }],
        errors: vec![],
        repeat_spans: vec![],
    };

    let scene = build_layout_scene(&score, &LayoutOptions::default());
    let staff_top = scene.pages[0]
        .items
        .iter()
        .find(|item| item.role == "staff-line")
        .and_then(|item| match &item.primitive {
            ScenePrimitive::LineSegment(line) => Some(line.y1_pt),
            _ => None,
        })
        .expect("expected staff line");
    let ledger_ys = scene.pages[0]
        .items
        .iter()
        .filter(|item| item.role == "ledger-line")
        .filter_map(|item| match &item.primitive {
            ScenePrimitive::LineSegment(line) => Some(line.y1_pt),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(ledger_ys.len(), 2);
    assert!(ledger_ys
        .iter()
        .any(|y| (*y - (staff_top + 50.0)).abs() < 0.01));
    assert!(ledger_ys
        .iter()
        .any(|y| (*y - (staff_top + 60.0)).abs() < 0.01));
}
// PATCH_INSERT_FOR_GOLDEN_REGENERATION
