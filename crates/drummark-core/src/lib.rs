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

fn set(obj: &js_sys::Object, key: &str, val: &JsValue) {
    js_sys::Reflect::set(obj, &JsValue::from_str(key), val).unwrap();
}

// ── Combined: Parse + Normalize + Layout → LayoutPlan ──────────

#[wasm_bindgen]
pub fn build_layout_plan(source: &str, options: JsValue) -> JsValue {
    // 0. Parse layout options from JS
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
    let layout_score = drummark_layout::NormalizedScore {
        header: drummark_layout::NormalizedHeader {
            tempo: score.header.tempo,
            time_beats: score.header.time_beats,
            time_beat_unit: score.header.time_beat_unit,
            divisions: score.header.divisions,
            note_value: score.header.note_value,
            grouping: score.header.grouping.clone(),
            title: score.header.title.clone(),
            subtitle: score.header.subtitle.clone(),
            composer: score.header.composer.clone(),
        },
        tracks: score.tracks.iter().map(|t| drummark_layout::NormalizedTrack {
            id: t.id.clone(),
            family: t.family.clone(),
        }).collect(),
        measures: score.measures.iter().map(|m| drummark_layout::NormalizedMeasure {
            index: m.index,
            global_index: m.global_index,
            paragraph_index: m.paragraph_index,
            measure_in_paragraph: m.measure_in_paragraph,
            events: m.events.iter().map(|ev| drummark_layout::NormalizedEvent {
                track: ev.track.clone(),
                start: drummark_layout::Fraction { numerator: ev.start.numerator as u32, denominator: ev.start.denominator as u32 },
                duration: drummark_layout::Fraction { numerator: ev.duration.numerator as u32, denominator: ev.duration.denominator as u32 },
                kind: match ev.kind {
                    event::EventKind::Hit => drummark_layout::EventKind::Hit,
                    event::EventKind::Rest => drummark_layout::EventKind::Rest,
                    event::EventKind::Sticking => drummark_layout::EventKind::Sticking,
                },
                glyph: ev.glyph.clone(),
                modifiers: ev.modifiers.clone(),
                modifier: ev.modifier.clone(),
                voice: ev.voice,
                beam: ev.beam.clone(),
                tuplet: ev.tuplet,
            }).collect(),
            barline: m.barline.clone(),
            start_nav: None,
            end_nav: None,
            volta_indices: m.volta.clone(),
            hairpins: vec![],
            measure_repeat_slashes: m.measure_repeat_slashes,
            multi_rest_count: m.multi_rest_count,
            note_value: m.note_value,
        }).collect(),
        errors: score.errors.clone(),
    };

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
    let content_start = margin + 35.0 + 35.0; // clef + time sig area
    let mut sys_y = opts.top_margin_pt as f64 + 50.0; // after title area

    // ── Title / Subtitle / Composer / Tempo ────────────────────

    if let Some(ref t) = layout_score.header.title {
        append_text_anchor(&sys_arr, center_x, 25.0, t, "Academico,serif", 18.0, "#333", "middle");
    }
    if let Some(ref t) = layout_score.header.subtitle {
        append_text_anchor(&sys_arr, center_x, 42.0, t, "Academico,serif", 12.0, "#333", "middle");
    }
    if let Some(ref t) = layout_score.header.composer {
        append_text_anchor(&sys_arr, page_w - margin, 25.0, t, "Academico,serif", 10.0, "#333", "end");
    }
    if layout_score.header.tempo > 0 {
        let tempo_text = format!("♩ = {}", layout_score.header.tempo);
        append_text_anchor(&sys_arr, margin, sys_y - 10.0, &tempo_text, "Academico,serif", 12.0, "#333", "start");
    }

    let mut sys_idx = 0;
    for measures in &para_systems {
        let is_first_system = sys_idx == 0;
        sys_idx += 1;
        let sy = sys_y;
        sys_y += staff_ss * 8.0; // staff height + gap
        let s_top = sy + staff_ss;
        let s_bot = sy + staff_ss * 5.0;
        let s_mid = sy + staff_ss * 3.0;

        // Staff lines
        for i in 0..5 {
            let ly = sy + staff_ss * (1.0 + i as f64);
            append_line(&sys_arr, margin, ly, page_w - margin, ly, "#999", 0.6);
        }

        // Percussion clef — dominant-baseline="central" centers on y
        append_text(&sys_arr, margin + 5.0, s_mid, "\u{E069}", "Bravura,Academico", 30.0, "#333");

        // Time signature — fills spaces 1-4 (full staff height)
        if is_first_system {
            let tsx = margin + 35.0;
            let beats = layout_score.header.time_beats;
            let unit = layout_score.header.time_beat_unit;
            append_text(&sys_arr, tsx, sy + staff_ss * 2.0, &num_to_glyph(beats), "Bravura,Academico", 30.0, "#333");
            append_text(&sys_arr, tsx, sy + staff_ss * 4.0, &num_to_glyph(unit), "Bravura,Academico", 30.0, "#333");
        }

        // Measures — equal width. No barline between clef/ts and first measure.
        let available_w = (page_w - margin * 2.0 - 70.0).max(100.0);
        let mw = available_w / measures.len().max(1) as f64;
        let mut mx = content_start;

        for (mi, m) in measures.iter().enumerate() {
            // Barline between measures (skip first — clef/ts area is the left boundary)
            if mi > 0 {
                append_line(&sys_arr, mx, s_top, mx, s_bot, "#333", 1.0);
            }

            // Notes — distribute evenly across measure width
            let hit_count = m.events.iter().filter(|e| e.kind == drummark_layout::EventKind::Hit).count().max(1);
            let note_spacing = (mw - 24.0) / hit_count as f64;
            let mut note_idx = 0;
            for ev in &m.events {
                if ev.kind == drummark_layout::EventKind::Hit {
                    let nx = mx + 12.0 + note_spacing * note_idx as f64;
                    let ny = s_top + staff_ss * 2.0;
                    let cp = 0xE0A4u32;
                    append_text(&sys_arr, nx - 7.0, ny, &char::from_u32(cp).unwrap_or('?').to_string(), "Bravura,Academico", 30.0, "#333");
                    append_line(&sys_arr, nx + 9.0, ny - staff_ss * 3.5, nx + 9.0, ny, "#333", 1.2);
                    note_idx += 1;
                }
            }

            mx += mw;
        }

        // Closing barline
        append_line(&sys_arr, mx, s_top, mx, s_bot, "#333", 1.0);
        append_line(&sys_arr, mx + 3.0, s_top, mx + 3.0, s_bot, "#333", 3.0);
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

fn num_to_glyph(n: u32) -> String {
    match n {
        0 => "\u{E080}".to_string(), 1 => "\u{E081}".to_string(), 2 => "\u{E082}".to_string(),
        3 => "\u{E083}".to_string(), 4 => "\u{E084}".to_string(), 5 => "\u{E085}".to_string(),
        6 => "\u{E086}".to_string(), 7 => "\u{E087}".to_string(), 8 => "\u{E088}".to_string(),
        9 => "\u{E089}".to_string(),
        _ => n.to_string(),
    }
}
