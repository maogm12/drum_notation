// ── Fraction ─────────────────────────────────────────────────────

/// Musical fraction (numerator/denominator) for start times and durations.
#[derive(Debug, Clone, Copy)]
pub struct Fraction {
    pub numerator: u32,
    pub denominator: u32,
}

// ── NormalizedScore (self-contained, no parser dependency) ──────

#[derive(Debug, Clone)]
pub struct NormalizedScore {
    pub header: NormalizedHeader,
    pub tracks: Vec<NormalizedTrack>,
    pub measures: Vec<NormalizedMeasure>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NormalizedHeader {
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

#[derive(Debug, Clone)]
pub struct NormalizedTrack {
    pub id: String,
    pub family: String,
}

#[derive(Debug, Clone)]
pub struct NormalizedMeasure {
    pub index: u32,
    pub global_index: u32,
    pub paragraph_index: u32,
    pub measure_in_paragraph: u32,
    pub events: Vec<NormalizedEvent>,
    pub barline: Option<String>,
    pub start_nav: Option<NavMarker>,
    pub end_nav: Option<NavJump>,
    pub volta_indices: Option<Vec<u32>>,
    pub hairpins: Vec<Hairpin>,
    pub measure_repeat_slashes: Option<u32>,
    pub multi_rest_count: Option<u32>,
    pub note_value: u32,
}

#[derive(Debug, Clone)]
pub struct NormalizedEvent {
    pub track: String,
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
pub enum EventKind {
    Hit,
    Rest,
    Sticking,
}

#[derive(Debug, Clone)]
pub enum NavMarker {
    Segno,
    Coda,
}

#[derive(Debug, Clone)]
pub enum NavJump {
    Fine,
    DC,
    DS,
    DCalFine, DCalCoda,
    DSalFine, DSalCoda,
    ToCoda,
}

#[derive(Debug, Clone)]
pub struct Hairpin {
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

// ── SMuFL Glyph Metrics ──────────────────────────────────────────

/// SMuFL codepoint identifier for a notehead or rest glyph.
#[derive(Debug, Clone, Copy)]
pub struct GlyphMetrics {
    /// SMuFL codepoint (e.g., `\u{E0A4}` for black notehead).
    pub codepoint: u32,
    /// Width in staff-space units.
    pub width_ss: f32,
    /// Height in staff-space units.
    pub height_ss: f32,
    /// Stem connection offset in SS from notehead center (negative = above center).
    pub stem_offset_y: f32,
}

/// Returns SMuFL glyph metrics for a notehead given track + modifiers.
pub fn notehead_glyph(track: &str, modifiers: &[String], _glyph: &str) -> GlyphMetrics {
    let family = track_family(track);

    // Cymbal tracks use X notehead
    if family == "cymbal" {
        return GlyphMetrics { codepoint: 0xE0A9, width_ss: 1.0, height_ss: 1.0, stem_offset_y: 0.0 };
    }

    // Drum tracks: check modifiers for special noteheads
    for m in modifiers {
        match m.as_str() {
            "open" => return GlyphMetrics { codepoint: 0xE0B3, width_ss: 1.0, height_ss: 1.0, stem_offset_y: 0.0 },
            "cross" => return GlyphMetrics { codepoint: 0xE0A9, width_ss: 1.0, height_ss: 1.0, stem_offset_y: 0.0 },
            "bell" => return GlyphMetrics { codepoint: 0xE0DB, width_ss: 1.0, height_ss: 1.0, stem_offset_y: 0.0 },
            "rim" => return GlyphMetrics { codepoint: 0xE0CE, width_ss: 1.0, height_ss: 1.0, stem_offset_y: 0.0 },
            _ => {}
        }
    }

    // Standard drum notehead
    GlyphMetrics { codepoint: 0xE0A4, width_ss: 1.0, height_ss: 1.0, stem_offset_y: 0.0 }
}

/// Returns SMuFL metrics for a rest glyph by duration denominator.
pub fn rest_glyph(denominator: u32) -> GlyphMetrics {
    match denominator {
        d if d >= 32 => GlyphMetrics { codepoint: 0xE4E7, width_ss: 0.8, height_ss: 1.2, stem_offset_y: 0.0 },
        d if d >= 16 => GlyphMetrics { codepoint: 0xE4E6, width_ss: 0.8, height_ss: 1.2, stem_offset_y: 0.0 },
        d if d >= 8  => GlyphMetrics { codepoint: 0xE4E5, width_ss: 0.8, height_ss: 1.5, stem_offset_y: 0.0 },
        d if d >= 4  => GlyphMetrics { codepoint: 0xE4E4, width_ss: 0.8, height_ss: 2.0, stem_offset_y: 0.0 },
        _             => GlyphMetrics { codepoint: 0xE4E3, width_ss: 0.8, height_ss: 1.0, stem_offset_y: 0.0 },
    }
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
    // Per-element Y offsets (positive = downward in staff space)
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
            volta_offset_y: -15.0,
            nav_offset_y: -10.0,
            hairpin_offset_y: 10.0,
            sticking_offset_y: -8.0,
            accent_offset_y: -6.0,
            text_offset_y: -40.0,
            tempo_offset_y: -25.0,
            measure_num_offset_y: -4.0,
            edge_padding: 4.0,
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
    // VexFlow-compatible percussion clef positions (staff-space units from top staff line)
    match track {
        "HH" => 0.0,   // top line — cymbal
        "RC" | "RC2" => 1.0,
        "C" | "C2" => 2.0,
        "SPL" => -1.0,
        "CHN" => -1.0,
        "T1" => 3.0,   // toms
        "T2" => 4.0,
        "T3" => 5.0,
        "T4" => 6.0,
        "SD" => 3.0,   // snare — third space (between lines 3-4 from top)
        "BD" | "BD2" => 6.0, // bass drum — first space (between lines 4-5)
        "HF" => 9.0,   // hi-hat foot — below staff
        "ST" => 0.0,   // sticking — above staff
        "CB" | "WB" | "CL" => 0.0, // percussion
        _ => 3.0,
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
        assert_eq!(staff_y_for_track("HH"), 0.0);
        assert_eq!(staff_y_for_track("SD"), 3.0);
        assert_eq!(staff_y_for_track("BD"), 6.0);
        assert_eq!(staff_y_for_track("T1"), 3.0);
    }

    #[test]
    fn test_notehead_glyph() {
        let g = notehead_glyph("HH", &[], "x");
        assert_eq!(g.codepoint, 0xE0A9); // cymbal → X notehead
        let g = notehead_glyph("SD", &[], "d");
        assert_eq!(g.codepoint, 0xE0A4); // drum → standard notehead
        let g = notehead_glyph("SD", &["cross".to_string()], "d");
        assert_eq!(g.codepoint, 0xE0A9); // cross mod → X notehead
    }

    #[test]
    fn test_default_options() {
        let opts = LayoutOptions::default();
        assert_eq!(opts.page_width_pt, 612.0);
        assert_eq!(opts.px_per_quarter, 80.0);
        assert_eq!(opts.volta_offset_y, -15.0);
    }

    #[test]
    fn test_staff_space() {
        let ss = StaffSpace::default();
        assert_eq!(ss.pt_per_ss, 8.0);
        assert_eq!(ss.to_pixels(40.0), 10.0); // 40pt staff / 4 = 10px per ss
    }
}

// ── Slot → X Mapping (Task 2) ───────────────────────────────────

/// Converts a uniform slot grid position to a horizontal X coordinate (in px).
/// The engine uses proportional spacing with content-weighted bonuses.
pub struct SlotMapper {
    pub px_per_quarter: f32,
}

impl SlotMapper {
    pub fn new(px_per_quarter: f32) -> Self { Self { px_per_quarter } }

    /// Map a slot index within a beat to a horizontal offset from the beat start.
    /// slots_per_beat = `divisions / beats` for this measure.
    pub fn slot_x_within_beat(&self, slot: u32, slots_per_beat: u32, beat_width: f32) -> f32 {
        let frac = slot as f32 / slots_per_beat as f32;
        frac * beat_width
    }

    /// Full measure width in pixels. Content-weighted: denser rhythms get more space.
    pub fn measure_width(&self, total_slots: u32, slots_per_quarter: u32, is_compact: bool) -> f32 {
        if is_compact { return 40.0; }
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
    pub priority: u8,  // for edge stacking (0=innermost)
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
pub fn place_notes(measure: &NormalizedMeasure, mapper: &SlotMapper, opts: &LayoutOptions) -> Vec<LayoutElement> {
    let mut elements = Vec::new();
    for ev in &measure.events {
        let x = mapper.slot_x_within_beat(
            to_slots(&ev.start, measure.note_value),
            slots_per_beat(measure),
            beat_width_for(measure, &ev.start),
        );
        let y = staff_y_for_track(&ev.track) + if ev.voice == 2 { 0.0 } else { 0.0 };
        let metrics = if ev.kind == EventKind::Rest {
            rest_glyph(ev.duration.denominator)
        } else {
            notehead_glyph(&ev.track, &ev.modifiers, &ev.glyph)
        };

        elements.push(LayoutElement {
            kind: if ev.kind == EventKind::Rest { ElementKind::Rest } else { ElementKind::Note },
            x, y,
            width: metrics.width_ss * 10.0,
            height: metrics.height_ss * 10.0,
            smufl_codepoint: Some(metrics.codepoint),
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
            for j in (i+1)..elements.len() {
                let (a, b) = if elements[i].priority < elements[j].priority {
                    (&elements[i].clone(), &elements[j].clone())
                } else {
                    (&elements[j].clone(), &elements[i].clone())
                };

                // Check X overlap
                let a_right = a.x + a.width;
                let b_right = b.x + b.width;
                let x_overlap = a.x < b_right && a_right > b.x;
                if !x_overlap { continue; }

                // Check Y overlap
                let a_bottom = a.y + a.height;
                let b_bottom = b.y + b.height;
                let y_overlap = a.y < b_bottom && a_bottom > b.y;
                if !y_overlap { continue; }

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

        if !any_overlap { break; }
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
    let usable_width = opts.page_width_pt - opts.left_margin_pt - opts.right_margin_pt - 30.0 - 40.0;

    for measure in &score.measures {
        let is_compact = measure.multi_rest_count.is_some() || measure.measure_repeat_slashes.is_some();
        let total_slots = measure.events.len() as u32; // simplified
        let width = mapper.measure_width(total_slots.max(1), 4, is_compact);

        if cursor_x + width > opts.left_margin_pt + usable_width && !current_system.measures.is_empty() {
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
    (f.numerator * note_value as u32) / f.denominator.max(1)
}

fn slots_per_beat(_measure: &NormalizedMeasure) -> u32 { 4 } // simplified
fn beat_width_for(_measure: &NormalizedMeasure, _start: &Fraction) -> f32 { 80.0 }

// ── LayoutPlan + WASM Export (Task 8) ────────────────────────────

use wasm_bindgen::prelude::*;
use js_sys::{Array, Object};

#[wasm_bindgen]
pub fn layout_plan(_score: JsValue, _options_json: JsValue) -> JsValue {
    let obj = Object::new();
    js_sys::Reflect::set(&obj, &JsValue::from_str("systems"), &Array::new()).unwrap();
    obj.into()
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
        index: 0, global_index: 0, paragraph_index: 0, measure_in_paragraph: 0,
        events: vec![NormalizedEvent {
            track: "HH".into(), start: Fraction{numerator:0,denominator:1},
            duration: Fraction{numerator:1,denominator:8}, kind: EventKind::Hit,
            glyph: "x".into(), modifiers: vec![], modifier: None, voice: 1,
            beam: "none".into(), tuplet: None,
        }],
        barline: Some("regular".into()), start_nav: None, end_nav: None,
        volta_indices: None, hairpins: vec![], measure_repeat_slashes: None,
        multi_rest_count: None, note_value: 8,
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
        LayoutElement { kind: ElementKind::NavMarker, x: 50.0, y: -15.0, width: 10.0, height: 10.0, smufl_codepoint: None, voice: None, stem_up: None, barline_type: None, text: None, from_x: None, to_x: None, priority: 6, can_shift_y: true, can_shift_x: false },
        LayoutElement { kind: ElementKind::Volta, x: 50.0, y: -20.0, width: 100.0, height: 8.0, smufl_codepoint: None, voice: None, stem_up: None, barline_type: None, text: None, from_x: None, to_x: None, priority: 7, can_shift_y: false, can_shift_x: false },
    ];
    let warnings = stack_edge_elements(&mut elements, 4.0);
    assert!(warnings.is_empty(), "unexpected warnings: {:?}", warnings);
    // Nav should be pushed above volta
    assert!(elements[0].y < -20.0, "nav should be above volta");
}

#[test]
fn test_barlines() {
    let measure = NormalizedMeasure {
        index: 0, global_index: 0, paragraph_index: 0, measure_in_paragraph: 0,
        events: vec![], barline: Some("|:".into()), start_nav: None, end_nav: None,
        volta_indices: None, hairpins: vec![], measure_repeat_slashes: None,
        multi_rest_count: None, note_value: 8,
    };
    let elements = place_barlines(&measure, 50.0);
    assert_eq!(elements.len(), 1);
    assert_eq!(elements[0].kind, ElementKind::Barline);
    assert_eq!(elements[0].barline_type.as_deref(), Some("|:"));
}
