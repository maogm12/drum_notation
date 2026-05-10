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
        "SD" => 4.0,   // snare — middle line
        "BD" | "BD2" => 8.0, // bass drum — bottom space
        "HF" => 9.0,   // hi-hat foot — below staff
        "ST" => 0.0,   // sticking — above staff
        "CB" | "WB" | "CL" => 0.0, // percussion
        _ => 4.0,
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
        assert_eq!(staff_y_for_track("SD"), 4.0);
        assert_eq!(staff_y_for_track("BD"), 8.0);
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
