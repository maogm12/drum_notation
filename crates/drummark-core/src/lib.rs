#![allow(dead_code)]

use wasm_bindgen::prelude::*;
use js_sys::{Array, Object};

extern crate drummark_layout;

pub mod lexer;
pub mod ast;
pub mod parser;
pub mod error;
pub mod to_js;
pub mod fraction;
pub mod resolve;
pub mod validate;
pub mod hairpin;
pub mod nav;
pub mod volta;
pub mod event;
pub mod normalize;
pub mod render_score;

/// Parse a DrumMark source string and return the AST as a JS object.
#[wasm_bindgen]
pub fn parse(source: &str) -> JsValue {
    let parser = parser::Parser::new(source);
    match parser.parse() {
        Ok(document) => to_js::document_to_js(&document),
        Err(errors) => to_js::errors_to_js(&errors),
    }
}

/// Parse and normalize a DrumMark source string in one call.
/// Returns the NormalizedScore as a JS object tree.
#[wasm_bindgen]
pub fn build_normalized_score(source: &str) -> JsValue {
    let parser = parser::Parser::new(source);
    let doc = match parser.parse() {
        Ok(doc) => doc,
        Err(errors) => return to_js::errors_to_js(&errors),
    };
    let score = normalize::normalize_document(&doc);
    normalize_to_js(&score)
}

#[wasm_bindgen]
pub fn build_render_score(source: &str) -> JsValue {
    let parser = parser::Parser::new(source);
    let doc = match parser.parse() {
        Ok(doc) => doc,
        Err(errors) => return to_js::errors_to_js(&errors),
    };
    let score = normalize::normalize_document(&doc);
    let render_score = render_score::derive_render_score(&score);
    render_score_to_js(&render_score)
}

#[wasm_bindgen]
pub fn build_layout_scene(source: &str, options: JsValue) -> JsValue {
    let opts = parse_layout_options(&options);
    let parser = parser::Parser::new(source);
    let doc = match parser.parse() {
        Ok(doc) => doc,
        Err(errors) => {
            let scene = drummark_layout::LayoutScene {
                version: drummark_layout::LAYOUT_SCENE_VERSION.to_string(),
                metrics_version: drummark_layout::CANONICAL_METRICS_VERSION.to_string(),
                pages: vec![],
                issues: errors.iter().map(|error| format!("Line {}, Col {}: {}", error.line, error.column, error.message)).collect(),
            };
            return layout_scene_to_js(&scene);
        }
    };
    let score = normalize::normalize_document(&doc);
    let render_score = render_score::derive_render_score(&score);
    let scene = drummark_layout::build_layout_scene(&render_score, &opts);
    layout_scene_to_js(&scene)
}

fn normalize_to_js(score: &normalize::NormalizedScore) -> JsValue {
    use js_sys::{Array, Object};

    let obj = Object::new();
    set(&obj, "version", &JsValue::from_str(&score.version));

    // Header
    let h = Object::new();
    if let Some(ref t) = score.header.title { set(&h, "title", &JsValue::from_str(t)); }
    if let Some(ref t) = score.header.subtitle { set(&h, "subtitle", &JsValue::from_str(t)); }
    if let Some(ref t) = score.header.composer { set(&h, "composer", &JsValue::from_str(t)); }
    set(&h, "tempo", &JsValue::from_f64(score.header.tempo as f64));
    let ts = Object::new();
    set(&ts, "beats", &JsValue::from_f64(score.header.time_beats as f64));
    set(&ts, "beatUnit", &JsValue::from_f64(score.header.time_beat_unit as f64));
    set(&h, "timeSignature", &ts.into());
    set(&h, "divisions", &JsValue::from_f64(score.header.divisions as f64));
    set(&h, "noteValue", &JsValue::from_f64(score.header.note_value as f64));
    let ga = Array::new();
    for &g in &score.header.grouping { ga.push(&JsValue::from_f64(g as f64)); }
    set(&h, "grouping", &ga.into());
    set(&obj, "header", &h.into());

    // Tracks
    let ta = Array::new();
    for t in &score.tracks {
        let to = Object::new();
        set(&to, "id", &JsValue::from_str(&t.id));
        set(&to, "family", &JsValue::from_str(&t.family));
        ta.push(&to.into());
    }
    set(&obj, "tracks", &ta.into());

    // Errors
    let ea = Array::new();
    for e in &score.errors { ea.push(&JsValue::from_str(e)); }
    set(&obj, "errors", &ea.into());

    // Measures
    let ma = Array::new();
    for m in &score.measures {
        let mo = Object::new();
        set(&mo, "index", &JsValue::from_f64(m.index as f64));
        set(&mo, "globalIndex", &JsValue::from_f64(m.global_index as f64));
        set(&mo, "paragraphIndex", &JsValue::from_f64(m.paragraph_index as f64));
        set(&mo, "measureInParagraph", &JsValue::from_f64(m.measure_in_paragraph as f64));
        set(&mo, "noteValue", &JsValue::from_f64(m.note_value as f64));
        if let Some(ref b) = m.barline { set(&mo, "barline", &JsValue::from_str(b)); }
        if let Some(ref b) = m.closing_barline { set(&mo, "closingBarline", &JsValue::from_str(b)); }
        if let Some(ref s) = m.start_nav {
            set(&mo, "startNav", &JsValue::from_str(s.kind_name()));
        }
        if let Some(ref e) = m.end_nav {
            set(&mo, "endNav", &JsValue::from_str(e.kind_name()));
        }
        if let Some(ref v) = m.volta {
            let va = Array::new();
            for n in v { va.push(&JsValue::from_f64(*n as f64)); }
            set(&mo, "volta", &va.into());
        }
        if let Some(slashes) = m.measure_repeat_slashes {
            set(&mo, "measureRepeatSlashes", &JsValue::from_f64(slashes as f64));
        }
        if let Some(count) = m.multi_rest_count {
            set(&mo, "multiRestCount", &JsValue::from_f64(count as f64));
        }
        // Hairpins
        if !m.hairpins.is_empty() {
            let ha = Array::new();
            for hp in &m.hairpins {
                let ho = Object::new();
                set(&ho, "kind", &JsValue::from_str(match hp.kind {
                    hairpin::HairpinKind::Crescendo => "crescendo",
                    hairpin::HairpinKind::Decrescendo => "decrescendo",
                }));
                set(&ho, "start", &frac_js(hp.start));
                set(&ho, "startMeasureIndex", &JsValue::from_f64(hp.start_measure_index as f64));
                set(&ho, "end", &frac_js(hp.end));
                set(&ho, "endMeasureIndex", &JsValue::from_f64(hp.end_measure_index as f64));
                ha.push(&ho.into());
            }
            set(&mo, "hairpins", &ha.into());
        }
        // Events
        let eva = Array::new();
        for ev in &m.events {
            let evo = Object::new();
            set(&evo, "track", &JsValue::from_str(&ev.track));
            set(&evo, "glyph", &JsValue::from_str(&ev.glyph));
            set(&evo, "kind", &JsValue::from_str(match ev.kind {
                event::EventKind::Hit => "hit",
                event::EventKind::Rest => "rest",
                event::EventKind::Sticking => "sticking",
            }));
            set(&evo, "start", &frac_js(ev.start));
            set(&evo, "duration", &frac_js(ev.duration));
            set(&evo, "voice", &JsValue::from_f64(ev.voice as f64));
            if !ev.modifiers.is_empty() {
                let ma_mod = Array::new();
                for m in &ev.modifiers { ma_mod.push(&JsValue::from_str(m)); }
                set(&evo, "modifiers", &ma_mod.into());
            }
            if let Some(ref m) = ev.modifier {
                set(&evo, "modifier", &JsValue::from_str(m));
            }
            eva.push(&evo.into());
        }
        set(&mo, "events", &eva.into());
        ma.push(&mo.into());
    }
    set(&obj, "measures", &ma.into());

    obj.into()
}

fn frac_js(f: crate::fraction::Fraction) -> JsValue {
    let obj = js_sys::Object::new();
    set(&obj, "numerator", &JsValue::from_f64(f.numerator as f64));
    set(&obj, "denominator", &JsValue::from_f64(f.denominator as f64));
    obj.into()
}

fn render_fraction_js(f: drummark_layout::Fraction) -> JsValue {
    let obj = js_sys::Object::new();
    set(&obj, "numerator", &JsValue::from_f64(f.numerator as f64));
    set(&obj, "denominator", &JsValue::from_f64(f.denominator as f64));
    obj.into()
}

fn render_score_to_js(score: &drummark_layout::RenderScore) -> JsValue {
    let obj = Object::new();
    set(&obj, "version", &JsValue::from_str(&score.version));

    let header = Object::new();
    set(&header, "tempo", &JsValue::from_f64(score.header.tempo as f64));
    set(&header, "timeBeats", &JsValue::from_f64(score.header.time_beats as f64));
    set(&header, "timeBeatUnit", &JsValue::from_f64(score.header.time_beat_unit as f64));
    set(&header, "divisions", &JsValue::from_f64(score.header.divisions as f64));
    set(&header, "noteValue", &JsValue::from_f64(score.header.note_value as f64));
    if let Some(ref title) = score.header.title { set(&header, "title", &JsValue::from_str(title)); }
    if let Some(ref subtitle) = score.header.subtitle { set(&header, "subtitle", &JsValue::from_str(subtitle)); }
    if let Some(ref composer) = score.header.composer { set(&header, "composer", &JsValue::from_str(composer)); }
    let grouping = Array::new();
    for group in &score.header.grouping {
        grouping.push(&JsValue::from_f64(*group as f64));
    }
    set(&header, "grouping", &grouping.into());
    set(&obj, "header", &header.into());

    let tracks = Array::new();
    for track in &score.tracks {
        let entry = Object::new();
        set(&entry, "id", &JsValue::from_str(&track.id));
        set(&entry, "family", &JsValue::from_str(&track.family));
        tracks.push(&entry.into());
    }
    set(&obj, "tracks", &tracks.into());

    let measures = Array::new();
    for measure in &score.measures {
        let entry = Object::new();
        set(&entry, "index", &JsValue::from_f64(measure.index as f64));
        set(&entry, "globalIndex", &JsValue::from_f64(measure.global_index as f64));
        set(&entry, "paragraphIndex", &JsValue::from_f64(measure.paragraph_index as f64));
        set(&entry, "measureInParagraph", &JsValue::from_f64(measure.measure_in_paragraph as f64));
        set(&entry, "sourceLine", &JsValue::from_f64(measure.source_line as f64));
        set(&entry, "noteValue", &JsValue::from_f64(measure.note_value as f64));
        set(&entry, "voltaTerminator", &JsValue::from_bool(measure.volta_terminator));
        if let Some(ref barline) = measure.barline {
            set(&entry, "barline", &JsValue::from_str(barline));
        }
        if let Some(ref closing_barline) = measure.closing_barline {
            set(&entry, "closingBarline", &JsValue::from_str(closing_barline));
        }
        if let Some(ref start_nav) = measure.start_nav {
            set(&entry, "startNav", &JsValue::from_str(match start_nav {
                drummark_layout::NavMarker::Segno => "segno",
                drummark_layout::NavMarker::Coda => "coda",
            }));
        }
        if let Some(ref end_nav) = measure.end_nav {
            set(&entry, "endNav", &JsValue::from_str(match end_nav {
                drummark_layout::NavJump::Fine => "fine",
                drummark_layout::NavJump::DC => "dc",
                drummark_layout::NavJump::DS => "ds",
                drummark_layout::NavJump::DCalFine => "dc-al-fine",
                drummark_layout::NavJump::DCalCoda => "dc-al-coda",
                drummark_layout::NavJump::DSalFine => "ds-al-fine",
                drummark_layout::NavJump::DSalCoda => "ds-al-coda",
                drummark_layout::NavJump::ToCoda => "to-coda",
            }));
        }
        if let Some(ref volta) = measure.volta_indices {
            let values = Array::new();
            for index in volta {
                values.push(&JsValue::from_f64(*index as f64));
            }
            set(&entry, "voltaIndices", &values.into());
        }
        let hairpins = Array::new();
        for hairpin in &measure.hairpins {
            let hairpin_obj = Object::new();
            set(&hairpin_obj, "kind", &JsValue::from_str(match hairpin.kind {
                drummark_layout::HairpinKind::Crescendo => "crescendo",
                drummark_layout::HairpinKind::Decrescendo => "decrescendo",
            }));
            set(&hairpin_obj, "start", &render_fraction_js(hairpin.start));
            set(&hairpin_obj, "end", &render_fraction_js(hairpin.end));
            set(&hairpin_obj, "startMeasureIndex", &JsValue::from_f64(hairpin.start_measure_index as f64));
            set(&hairpin_obj, "endMeasureIndex", &JsValue::from_f64(hairpin.end_measure_index as f64));
            hairpins.push(&hairpin_obj.into());
        }
        set(&entry, "hairpins", &hairpins.into());
        if let Some(count) = measure.measure_repeat_slashes {
            set(&entry, "measureRepeatSlashes", &JsValue::from_f64(count as f64));
        }
        if let Some(count) = measure.multi_rest_count {
            set(&entry, "multiRestCount", &JsValue::from_f64(count as f64));
        }
        let events = Array::new();
        for event in &measure.events {
            let event_obj = Object::new();
            set(&event_obj, "track", &JsValue::from_str(&event.track));
            set(&event_obj, "trackFamily", &JsValue::from_str(&event.track_family));
            set(&event_obj, "glyph", &JsValue::from_str(&event.glyph));
            set(&event_obj, "start", &render_fraction_js(event.start));
            set(&event_obj, "duration", &render_fraction_js(event.duration));
            set(&event_obj, "voice", &JsValue::from_f64(event.voice as f64));
            set(&event_obj, "beam", &JsValue::from_str(&event.beam));
            set(&event_obj, "kind", &JsValue::from_str(match event.kind {
                drummark_layout::EventKind::Hit => "hit",
                drummark_layout::EventKind::Rest => "rest",
                drummark_layout::EventKind::Sticking => "sticking",
            }));
            if let Some(ref modifier) = event.modifier {
                set(&event_obj, "modifier", &JsValue::from_str(modifier));
            }
            if let Some((count, span)) = event.tuplet {
                let tuplet = Object::new();
                set(&tuplet, "count", &JsValue::from_f64(count as f64));
                set(&tuplet, "span", &JsValue::from_f64(span as f64));
                set(&event_obj, "tuplet", &tuplet.into());
            }
            let modifiers = Array::new();
            for modifier in &event.modifiers {
                modifiers.push(&JsValue::from_str(modifier));
            }
            set(&event_obj, "modifiers", &modifiers.into());
            events.push(&event_obj.into());
        }
        set(&entry, "events", &events.into());
        measures.push(&entry.into());
    }
    set(&obj, "measures", &measures.into());

    let repeats = Array::new();
    for repeat in &score.repeat_spans {
        let entry = Object::new();
        set(&entry, "startMeasure", &JsValue::from_f64(repeat.start_measure as f64));
        set(&entry, "endMeasure", &JsValue::from_f64(repeat.end_measure as f64));
        set(&entry, "times", &JsValue::from_f64(repeat.times as f64));
        repeats.push(&entry.into());
    }
    set(&obj, "repeatSpans", &repeats.into());

    let errors = Array::new();
    for error in &score.errors {
        errors.push(&JsValue::from_str(error));
    }
    set(&obj, "errors", &errors.into());

    obj.into()
}

fn set(obj: &js_sys::Object, key: &str, val: &JsValue) {
    js_sys::Reflect::set(obj, &JsValue::from_str(key), val).unwrap();
}

fn parse_layout_options(options: &JsValue) -> drummark_layout::LayoutOptions {
    if options.is_object() {
        let get_f64 = |key: &str| -> f64 {
            js_sys::Reflect::get(options, &JsValue::from_str(key))
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
        };
        let get_optional_f64 = |key: &str| -> Option<f64> {
            js_sys::Reflect::get(options, &JsValue::from_str(key))
                .ok()
                .and_then(|v| v.as_f64())
        };
        let width = get_f64("pageWidth");
        let height = get_f64("pageHeight");
        let top = get_f64("topMargin");
        let bottom = get_f64("bottomMargin");
        let left = get_f64("leftMargin");
        let right = get_f64("rightMargin");
        let scale = get_f64("staffScale");
        let px_q = get_f64("pxPerQuarter");
        let stem_len = get_f64("stemLenPt");
        let sys_spacing = get_optional_f64("systemSpacing");
        let header_height = get_optional_f64("headerHeight");
        let header_staff_spacing = get_optional_f64("headerStaffSpacing");
        let volta_spacing = get_optional_f64("voltaSpacing");
        let hairpin_offset = get_optional_f64("hairpinOffsetY");
        let dur_compression = get_optional_f64("durationSpacingCompression");
        let measure_compression = get_optional_f64("measureWidthCompression");
        let hide_v2_rests = js_sys::Reflect::get(options, &JsValue::from_str("hideVoice2Rests"))
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if width > 0.0 && height > 0.0 {
            drummark_layout::LayoutOptions {
                page_width_pt: width as f32,
                page_height_pt: height as f32,
                top_margin_pt: top as f32,
                bottom_margin_pt: bottom as f32,
                left_margin_pt: left as f32,
                right_margin_pt: right as f32,
                staff_scale: if scale > 0.0 { scale as f32 } else { 0.75 },
                px_per_quarter: if px_q > 0.0 { px_q as f32 } else { 80.0 },
                stem_len_pt: if stem_len > 0.0 { stem_len as f32 } else { 31.0 },
                system_spacing_pt: sys_spacing.unwrap_or(30.0) as f32,
                header_height_pt: header_height.unwrap_or(50.0) as f32,
                header_staff_spacing_pt: header_staff_spacing.unwrap_or(60.0) as f32,
                volta_offset_y: volta_spacing.unwrap_or(0.0) as f32,
                hairpin_offset_y: hairpin_offset.unwrap_or(0.0) as f32,
                hide_voice2_rests: hide_v2_rests,
                duration_spacing_compression: dur_compression.unwrap_or(0.6) as f32,
                measure_width_compression: measure_compression.unwrap_or(0.75) as f32,
                ..drummark_layout::LayoutOptions::default()
            }
        } else {
            drummark_layout::LayoutOptions::default()
        }
    } else {
        drummark_layout::LayoutOptions::default()
    }
}

fn layout_scene_to_js(scene: &drummark_layout::LayoutScene) -> JsValue {
    drummark_layout::layout_scene_to_js(scene)
}

// ── Combined: Parse + Normalize + Layout → LayoutPlan ──────────

#[wasm_bindgen]
pub fn build_layout_plan(source: &str, options: JsValue) -> JsValue {
    // 0. Parse layout options from JS
    let mut show_debug_bbox = false;
    let opts = if options.is_object() {
        let get_f64 = |key: &str| -> f64 {
            js_sys::Reflect::get(&options, &JsValue::from_str(key))
                .ok().and_then(|v| v.as_f64()).unwrap_or(0.0)
        };
        let width = get_f64("pageWidth");
        let height = get_f64("pageHeight");
        let top = get_f64("topMargin");
        let bottom = get_f64("bottomMargin");
        let left = get_f64("leftMargin");
        let right = get_f64("rightMargin");
        let scale = get_f64("staffScale");
        let px_q = get_f64("pxPerQuarter");
        let header_height = get_f64("headerHeight");
        let header_staff_spacing = get_f64("headerStaffSpacing");
        let hairpin_offset = get_f64("hairpinOffsetY");
        show_debug_bbox = get_f64("debug") > 0.0;
        if width > 0.0 && height > 0.0 {
            drummark_layout::LayoutOptions {
                page_width_pt: width as f32,
                page_height_pt: height as f32,
                top_margin_pt: top as f32,
                bottom_margin_pt: bottom as f32,
                left_margin_pt: left as f32,
                right_margin_pt: right as f32,
                staff_scale: if scale > 0.0 { scale as f32 } else { 0.75 },
                px_per_quarter: if px_q > 0.0 { px_q as f32 } else { 80.0 },
                header_height_pt: if header_height > 0.0 { header_height as f32 } else { 50.0 },
                header_staff_spacing_pt: header_staff_spacing as f32,
                hairpin_offset_y: hairpin_offset as f32,
                ..drummark_layout::LayoutOptions::default()
            }
        } else {
            drummark_layout::LayoutOptions::default()
        }
    } else {
        drummark_layout::LayoutOptions::default()
    };
    // 1. Parse
    let parser = parser::Parser::new(source);
    let doc = match parser.parse() {
        Ok(doc) => doc,
        Err(errors) => return to_js::errors_to_js(&errors),
    };

    // 2. Normalize
    let score = normalize::normalize_document(&doc);

    // 3. Convert to layout-engine NormalizedScore
    let layout_score = render_score::derive_render_score(&score);

    // 4. Build systems grouped by paragraph_index
    let mut para_systems: Vec<Vec<&drummark_layout::NormalizedMeasure>> = Vec::new();
    let mut current_para = -1i32;
    for m in &layout_score.measures {
        if m.paragraph_index as i32 != current_para {
            para_systems.push(Vec::new());
            current_para = m.paragraph_index as i32;
        }
        para_systems.last_mut().unwrap().push(m);
    }

    // 5. Serialize as pages → systems → drawing instructions
    let sys_arr = Array::new();
    let page_w = opts.page_width_pt as f64;
    let page_h = opts.page_height_pt as f64;
    let margin = opts.left_margin_pt as f64;
    let staff_ss = 10.0_f64;
    let center_x = page_w / 2.0;
    let content_start = margin + 103.0; // clef + time sig + gap
    // VexFlow-compatible Y offset: accounts for title area + stave internal margin  
    let header_area_h = 130.0;
    let mut sys_y = opts.top_margin_pt as f64 + header_area_h;

    // ── Title / Subtitle / Composer / Tempo ────────────────────

    if let Some(ref t) = layout_score.header.title {
        append_text_bold(&sys_arr, center_x, 72.0, t, "Bravura", 24.0, "#333", "middle");
    }
    if let Some(ref t) = layout_score.header.subtitle {
        append_text_anchor(&sys_arr, center_x, 96.0, t, "Bravura", 12.0, "#333", "middle");
    }
    if let Some(ref t) = layout_score.header.composer {
        append_text_anchor(&sys_arr, page_w - margin, 72.0, t, "Bravura", 10.0, "#333", "end");
    }
    if layout_score.header.tempo > 0 {
        let tempo_y = 160.0;
        append_text(&sys_arr, margin + 32.0, tempo_y, "\u{E0A4}", "Bravura", 25.0, "#333");
        append_text(&sys_arr, margin + 57.0, tempo_y, "=", "Bravura", 14.0, "#333");
        append_text(&sys_arr, margin + 68.0, tempo_y, &layout_score.header.tempo.to_string(), "Bravura", 14.0, "#333");
    }

    let mut sys_idx = 0;
    for measures in &para_systems {
        let is_first_system = sys_idx == 0;
        sys_idx += 1;
        let sy = sys_y;
        sys_y += 130.0; // staff height (40) + inter-system gap (90) = 130
        let s_top = sy + staff_ss;
        let s_bot = sy + staff_ss * 5.0;
        let s_mid = sy + staff_ss * 3.0;

        // Staff lines
        for i in 0..5 {
            let ly = sy + staff_ss * (1.0 + i as f64);
            append_line(&sys_arr, margin, ly, page_w - margin, ly, "#333", 1.0);
        }

        // Percussion clef — dominant-baseline="central" centers on y
        append_text(&sys_arr, margin + 18.0, s_mid, "\u{E069}", "Bravura", 30.0, "#333");

        // Time signature — fills spaces 1-4 (full staff height)
        if is_first_system {
            let tsx = margin + 62.0;
            let beats = layout_score.header.time_beats;
            let unit = layout_score.header.time_beat_unit;
            append_text(&sys_arr, tsx, sy + staff_ss * 2.0, &num_to_glyph(beats), "Bravura", 30.0, "#333");
            append_text(&sys_arr, tsx, sy + staff_ss * 4.0, &num_to_glyph(unit), "Bravura", 30.0, "#333");
        }

        // Measures — equal width. No barline between clef/ts and first measure.
        let available_w = (page_w - margin * 2.0 - 70.0).max(100.0);
        let mw = available_w / measures.len().max(1) as f64;
        let mut mx = content_start;

        // Left barline (opening) — 1pt extra height to match VexFlow
        append_rect(&sys_arr, margin, s_top, 1.0, s_bot - s_top + 1.0, "#333");

        // Measure number for non-first systems
        if !is_first_system {
            append_text(&sys_arr, margin, sy - staff_ss, &format!("{}", measures[0].paragraph_index + 1), "Bravura", 11.0, "#333");
        }

        for (mi, m) in measures.iter().enumerate() {
            // Barline between measures (rect)
            if mi > 0 {
                append_rect(&sys_arr, mx, s_top, 1.0, s_bot - s_top + 1.0, "#333");
            }

            // Notes — distribute evenly across measure width
            let hit_count = m.events.iter().filter(|e| e.kind == drummark_layout::EventKind::Hit).count().max(1);
            let note_spacing = (mw - 24.0) / hit_count as f64;
            let mut note_idx = 0;
            for ev in &m.events {
                if ev.kind == drummark_layout::EventKind::Hit {
                    let nx = mx + 12.0 + note_spacing * note_idx as f64;
                    let track_ss = drummark_layout::staff_y_for_track(&ev.track);
                    let ny = s_top + track_ss as f64 * staff_ss;
                    let cp = 0xE0A4u32;
                    let nh_x = nx - 7.0;
                    // SMuFL noteheadBlack anchors in staff-space units (Bravura metadata)
                    //   stemUpSE:  (1.18,  0.168) — stem-up connection at top-right
                    //   stemDownNW:(0.0,  -0.168) — stem-down connection at bottom-left
                    // Fallback to glyphBBoxes if anchor not available:
                    //   bBoxNE: (1.18, 0.5), bBoxSW: (0.0, -0.5)
                    let nh_font_size = 30.0;
                    let smufl_ss = nh_font_size / 4.0; // 1 SMuFL ss in pt = font-size / 4
                    let stem_up = !matches!(ev.track.as_str(), "BD" | "BD2" | "HF");
                    // Anchor-based stem connection point (preferred over bbox edges)
                    let (anchor_x, anchor_y) = if stem_up {
                        (1.18, 0.168)  // stemUpSE
                    } else {
                        (0.0, -0.168)  // stemDownNW
                    };
                    // Connection point in our coordinates:
                    //   x = origin + anchor_x * smufl_ss  (both use positive-x = rightward)
                    //   y = origin - anchor_y * smufl_ss  (SMuFL y-up, SVG y-down: negate)
                    let stem_cx = nh_x + anchor_x * smufl_ss;
                    let stem_cy = ny - anchor_y * smufl_ss;
                    // Stem length: 3.5 staff spaces (~one octave)
                    let stem_len = staff_ss * 3.5;
                    let stem_y1 = if stem_up { stem_cy - stem_len } else { stem_cy };
                    let stem_y2 = if stem_up { stem_cy } else { stem_cy + stem_len };

                    // Debug bounding box (glyphBBoxes-based, outline only)
                    if show_debug_bbox {
                        // bBoxSW: (0.0, -0.5), bBoxNE: (1.18, 0.5) from Bravura metadata
                        let bb_x = nh_x + 0.0 * smufl_ss;
                        let bb_w = (1.18 - 0.0) * smufl_ss;
                        let bb_top = ny - 0.5 * smufl_ss;
                        let bb_h = (0.5 - (-0.5)) * smufl_ss;
                        append_rect_stroke(&sys_arr, bb_x, bb_top, bb_w, bb_h, "red", 1.0);
                    }
                    // Group notehead + stem
                    append_group_start(&sys_arr);
                    append_text(&sys_arr, nh_x, ny, &char::from_u32(cp).unwrap_or('?').to_string(), "Bravura", 30.0, "#333");
                    append_line(&sys_arr, stem_cx, stem_y1, stem_cx, stem_y2, "#333", 1.5);
                    append_group_end(&sys_arr);
                    note_idx += 1;
                }
            }

            mx += mw;
        }

        // Closing barline (single for non-last, double for last system)
        let is_last = sys_idx as usize == para_systems.len();
        let bar_h = s_bot - s_top + 1.0;
        append_rect(&sys_arr, mx, s_top, 1.0, bar_h, "#333");
        if is_last {
            append_rect(&sys_arr, mx + 4.0, s_top, 3.0, bar_h, "#333");
        }
    }

    let pages_arr = Array::new();
    let p_obj = Object::new();
    set(&p_obj, "width", &JsValue::from_f64(page_w));
    set(&p_obj, "height", &JsValue::from_f64(page_h));
    set(&p_obj, "systems", &sys_arr);
    pages_arr.push(&p_obj);

    let result = Object::new();
    set(&result, "pages", &pages_arr);
    result.into()
}

fn append_line(arr: &Array, x1: f64, y1: f64, x2: f64, y2: f64, stroke: &str, sw: f64) {
    let obj = Object::new();
    set(&obj, "tag", &JsValue::from_str("line"));
    set(&obj, "x1", &JsValue::from_f64(x1));
    set(&obj, "y1", &JsValue::from_f64(y1));
    set(&obj, "x2", &JsValue::from_f64(x2));
    set(&obj, "y2", &JsValue::from_f64(y2));
    set(&obj, "stroke", &JsValue::from_str(stroke));
    set(&obj, "strokeWidth", &JsValue::from_f64(sw));
    arr.push(&obj);
}

fn append_rect(arr: &Array, x: f64, y: f64, w: f64, h: f64, fill: &str) {
    let obj = Object::new();
    set(&obj, "tag", &JsValue::from_str("rect"));
    set(&obj, "x", &JsValue::from_f64(x));
    set(&obj, "y", &JsValue::from_f64(y));
    set(&obj, "width", &JsValue::from_f64(w));
    set(&obj, "height", &JsValue::from_f64(h));
    set(&obj, "fill", &JsValue::from_str(fill));
    arr.push(&obj);
}

fn append_rect_stroke(arr: &Array, x: f64, y: f64, w: f64, h: f64, stroke: &str, sw: f64) {
    let obj = Object::new();
    set(&obj, "tag", &JsValue::from_str("rect"));
    set(&obj, "x", &JsValue::from_f64(x));
    set(&obj, "y", &JsValue::from_f64(y));
    set(&obj, "width", &JsValue::from_f64(w));
    set(&obj, "height", &JsValue::from_f64(h));
    set(&obj, "stroke", &JsValue::from_str(stroke));
    set(&obj, "strokeWidth", &JsValue::from_f64(sw));
    set(&obj, "fill", &JsValue::from_str("none"));
    arr.push(&obj);
}

fn append_group_start(arr: &Array) {
    let obj = Object::new();
    set(&obj, "tag", &JsValue::from_str("g_open"));
    arr.push(&obj);
}

fn append_group_end(arr: &Array) {
    let obj = Object::new();
    set(&obj, "tag", &JsValue::from_str("g_close"));
    arr.push(&obj);
}

fn append_text(arr: &Array, x: f64, y: f64, text: &str, font: &str, size: f64, fill: &str) {
    let obj = Object::new();
    set(&obj, "tag", &JsValue::from_str("text"));
    set(&obj, "x", &JsValue::from_f64(x));
    set(&obj, "y", &JsValue::from_f64(y));
    set(&obj, "text", &JsValue::from_str(text));
    set(&obj, "fontFamily", &JsValue::from_str(font));
    set(&obj, "fontSize", &JsValue::from_f64(size));
    set(&obj, "fill", &JsValue::from_str(fill));
    arr.push(&obj);
}

fn append_text_anchor(arr: &Array, x: f64, y: f64, text: &str, font: &str, size: f64, fill: &str, anchor: &str) {
    let obj = Object::new();
    set(&obj, "tag", &JsValue::from_str("text"));
    set(&obj, "x", &JsValue::from_f64(x));
    set(&obj, "y", &JsValue::from_f64(y));
    set(&obj, "text", &JsValue::from_str(text));
    set(&obj, "fontFamily", &JsValue::from_str(font));
    set(&obj, "fontSize", &JsValue::from_f64(size));
    set(&obj, "fill", &JsValue::from_str(fill));
    set(&obj, "textAnchor", &JsValue::from_str(anchor));
    arr.push(&obj);
}

fn append_text_bold(arr: &Array, x: f64, y: f64, text: &str, font: &str, size: f64, fill: &str, anchor: &str) {
    let obj = Object::new();
    set(&obj, "tag", &JsValue::from_str("text"));
    set(&obj, "x", &JsValue::from_f64(x));
    set(&obj, "y", &JsValue::from_f64(y));
    set(&obj, "text", &JsValue::from_str(text));
    set(&obj, "fontFamily", &JsValue::from_str(font));
    set(&obj, "fontSize", &JsValue::from_f64(size));
    set(&obj, "fill", &JsValue::from_str(fill));
    set(&obj, "textAnchor", &JsValue::from_str(anchor));
    set(&obj, "fontWeight", &JsValue::from_str("bold"));
    arr.push(&obj);
}

fn num_to_glyph(n: u32) -> String {
    match n {
        0 => "\u{E080}".to_string(), 1 => "\u{E081}".to_string(), 2 => "\u{E082}".to_string(),
        3 => "\u{E083}".to_string(), 4 => "\u{E084}".to_string(), 5 => "\u{E085}".to_string(),
        6 => "\u{E086}".to_string(), 7 => "\u{E087}".to_string(), 8 => "\u{E088}".to_string(),
        9 => "\u{E089}".to_string(),
        _ => n.to_string(),
    }
}
