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

    // 5. Serialize to JsValue
    let sys_arr = Array::new();
    for sys in &systems {
        let s_obj = Object::new();
        set(&s_obj, "y", &JsValue::from_f64(sys.y as f64));
        set(&s_obj, "height", &JsValue::from_f64(sys.height as f64));

        let m_arr = Array::new();
        for m in &sys.measures {
            let m_obj = Object::new();
            set(&m_obj, "x", &JsValue::from_f64(m.x as f64));
            set(&m_obj, "width", &JsValue::from_f64(m.width as f64));

            let e_arr = Array::new();
            for e in &m.elements {
                let e_obj = Object::new();
                set(&e_obj, "kind", &JsValue::from_str(match e.kind {
                    drummark_layout::ElementKind::Note => "note",
                    drummark_layout::ElementKind::Rest => "rest",
                    drummark_layout::ElementKind::Barline => "barline",
                    drummark_layout::ElementKind::Sticking => "sticking",
                    drummark_layout::ElementKind::Modifier => "modifier",
                    drummark_layout::ElementKind::GraceNote => "graceNote",
                    drummark_layout::ElementKind::Beam => "beam",
                    drummark_layout::ElementKind::Stem => "stem",
                    _ => "other",
                }));
                set(&e_obj, "x", &JsValue::from_f64(e.x as f64));
                set(&e_obj, "y", &JsValue::from_f64(e.y as f64));
                set(&e_obj, "width", &JsValue::from_f64(e.width as f64));
                set(&e_obj, "height", &JsValue::from_f64(e.height as f64));
                if let Some(cp) = e.smufl_codepoint {
                    set(&e_obj, "codepoint", &JsValue::from_f64(cp as f64));
                }
                if let Some(v) = e.voice { set(&e_obj, "voice", &JsValue::from_f64(v as f64)); }
                if let Some(v) = e.stem_up { set(&e_obj, "stemUp", &JsValue::from_f64(if v { 1.0 } else { 0.0 })); }
                if let Some(ref t) = e.text { set(&e_obj, "text", &JsValue::from_str(t)); }
                e_arr.push(&e_obj);
            }
            set(&m_obj, "elements", &e_arr);
            m_arr.push(&m_obj);
        }
        set(&s_obj, "measures", &m_arr);
        sys_arr.push(&s_obj);
    }

    let result = Object::new();
    set(&result, "systems", &sys_arr);
    result.into()
}
