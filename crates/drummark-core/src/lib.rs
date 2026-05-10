#![allow(dead_code)]

use wasm_bindgen::prelude::*;

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
