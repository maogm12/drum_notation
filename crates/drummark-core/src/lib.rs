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
        if let Some(ref b) = m.barline { set(&mo, "barline", &JsValue::from_str(b)); }
        if let Some(ref s) = m.start_nav {
            set(&mo, "startNav", &JsValue::from_str(s.kind_name()));
        }
        if let Some(ref e) = m.end_nav {
            set(&mo, "endNav", &JsValue::from_str(e.kind_name()));
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
pub fn build_layout_plan(source: &str) -> JsValue {
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

    // 4. Layout
    let opts = drummark_layout::LayoutOptions::default();
    let systems = drummark_layout::build_systems(&layout_score, &opts);

    // 5. Serialize as pages → systems → drawing instructions
    let page_obj = Object::new();
    // One page for now — all systems
    let sys_arr = Array::new();
    let page_w = 612.0_f64;
    let page_h = 792.0_f64;
    let margin = 30.0_f64;
    let staff_ss = 10.0_f64; // staff space: 10pt

    for sys in &systems {
        let sy = sys.y as f64;
        let s_top = sy + staff_ss;
        let s_bot = sy + staff_ss * 5.0;
        let s_mid = sy + staff_ss * 3.0;

        // Staff lines (5 lines spanning the full system)
        for i in 0..5 {
            let ly = sy + staff_ss * (1.0 + i as f64);
            append_line(&sys_arr, margin, ly, page_w - margin, ly, "#999", 0.6);
        }

        // Opening barline (left edge of system)
        append_line(&sys_arr, margin, s_top, margin, s_bot, "#333", 1.0);

        // Percussion clef
        append_text(&sys_arr, margin + 5.0, s_mid + 6.0, "\u{E069}", "Bravura,Academico", 30.0, "#333");

        // Time signature (first system only)
        if sys.measures.first().map(|m| m.x) == sys.measures.first().map(|m| m.x) {
            let tsx = margin + 35.0;
            let beats = layout_score.header.time_beats;
            let unit = layout_score.header.time_beat_unit;
            append_text(&sys_arr, tsx, sy + staff_ss * 1.6, &num_to_glyph(beats), "Bravura,Academico", 30.0, "#333");
            append_text(&sys_arr, tsx, sy + staff_ss * 3.6, &num_to_glyph(unit), "Bravura,Academico", 30.0, "#333");
        }

        // Measures
        for m in &sys.measures {
            // Measure barline
            append_line(&sys_arr, m.x as f64, s_top, m.x as f64, s_bot, "#333", 1.0);

            // Notes/barlines/etc
            for e in &m.elements {
                match e.kind {
                    drummark_layout::ElementKind::Note => {
                        if let Some(cp) = e.smufl_codepoint {
                            let ny = e.y as f64; // notehead Y
                            append_text(&sys_arr, e.x as f64 - 7.0, ny, &char::from_u32(cp).unwrap_or('?').to_string(), "Bravura,Academico", 30.0, "#333");
                            // Stem
                            let sx = e.x as f64 + 9.0;
                            let up = e.stem_up.unwrap_or(true);
                            if up {
                                append_line(&sys_arr, sx, ny - staff_ss * 3.5, sx, ny, "#333", 1.2);
                            } else {
                                append_line(&sys_arr, sx, ny, sx, ny + staff_ss * 3.5, "#333", 1.2);
                            }
                        }
                    }
                    drummark_layout::ElementKind::Beam => {
                        if let (Some(fx), Some(tx)) = (e.from_x, e.to_x) {
                            let by = e.y as f64;
                            append_line(&sys_arr, fx as f64, by, tx as f64, by, "#333", 4.0);
                        }
                    }
                    drummark_layout::ElementKind::Text => {
                        if let Some(ref t) = e.text {
                            append_text(&sys_arr, e.x as f64, e.y as f64, t, "Academico,serif", 12.0, "#333");
                        }
                    }
                    _ => {}
                }
            }
        }

        // Closing barline (right edge of last measure)
        if let Some(last) = sys.measures.last() {
            let ex = last.x as f64 + last.width as f64;
            append_line(&sys_arr, ex, s_top, ex, s_bot, "#333", 1.0);
            append_line(&sys_arr, ex + 3.0, s_top, ex + 3.0, s_bot, "#333", 3.0);
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

fn num_to_glyph(n: u32) -> String {
    match n {
        0 => "\u{E080}", 1 => "\u{E081}", 2 => "\u{E082}", 3 => "\u{E083}", 4 => "\u{E084}",
        5 => "\u{E085}", 6 => "\u{E086}", 7 => "\u{E087}", 8 => "\u{E088}", 9 => "\u{E089}",
        _ => n.to_string(),
    }.to_string()
}
