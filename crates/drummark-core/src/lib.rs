#![allow(dead_code)]
#![allow(clippy::items_after_test_module)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::too_many_arguments)]

use wasm_bindgen::prelude::*;

#[cfg(feature = "layout-wasm")]
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
#[cfg(feature = "layout-wasm")]
pub mod render_score;

/// Parse a DrumMark source string and return the AST as a JS object.
#[cfg(feature = "parser-wasm")]
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
#[cfg(all(feature = "parser-wasm", feature = "layout-wasm"))]
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

#[cfg(all(feature = "parser-wasm", feature = "layout-wasm"))]
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

#[cfg(feature = "layout-wasm")]
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

#[cfg(all(feature = "parser-wasm", feature = "layout-wasm"))]
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

#[cfg(all(feature = "parser-wasm", feature = "layout-wasm"))]
fn render_fraction_js(f: drummark_layout::Fraction) -> JsValue {
    let obj = js_sys::Object::new();
    set(&obj, "numerator", &JsValue::from_f64(f.numerator as f64));
    set(&obj, "denominator", &JsValue::from_f64(f.denominator as f64));
    obj.into()
}

#[cfg(all(feature = "parser-wasm", feature = "layout-wasm"))]
fn render_score_to_js(score: &drummark_layout::RenderScore) -> JsValue {
    use js_sys::{Array, Object};
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

#[cfg(feature = "layout-wasm")]
fn parse_layout_options(options: &JsValue) -> drummark_layout::LayoutOptions {
    let mut opts = drummark_layout::LayoutOptions::default();
    if !options.is_object() {
        return opts;
    }

    let get_optional_f64 = |key: &str| -> Option<f64> {
        js_sys::Reflect::get(options, &JsValue::from_str(key))
            .ok()
            .and_then(|v| v.as_f64())
    };
    let get_optional_bool = |key: &str| -> Option<bool> {
        js_sys::Reflect::get(options, &JsValue::from_str(key))
            .ok()
            .and_then(|v| v.as_bool())
    };
    let assign_positive = |target: &mut f32, key: &str| {
        if let Some(value) = get_optional_f64(key).filter(|value| *value > 0.0) {
            *target = value as f32;
        }
    };
    let assign_any = |target: &mut f32, key: &str| {
        if let Some(value) = get_optional_f64(key) {
            *target = value as f32;
        }
    };

    assign_positive(&mut opts.page_width_pt, "pageWidth");
    assign_positive(&mut opts.page_height_pt, "pageHeight");
    assign_any(&mut opts.top_margin_pt, "topMargin");
    assign_any(&mut opts.bottom_margin_pt, "bottomMargin");
    assign_any(&mut opts.left_margin_pt, "leftMargin");
    assign_any(&mut opts.right_margin_pt, "rightMargin");
    assign_positive(&mut opts.staff_scale, "staffScale");
    assign_positive(&mut opts.px_per_quarter, "pxPerQuarter");
    assign_positive(&mut opts.stem_len_pt, "stemLenPt");
    assign_any(&mut opts.system_spacing_pt, "systemSpacing");
    assign_any(&mut opts.header_height_pt, "headerHeight");
    assign_any(&mut opts.header_staff_spacing_pt, "headerStaffSpacing");
    assign_any(&mut opts.volta_offset_y, "voltaSpacing");
    assign_any(&mut opts.hairpin_offset_y, "hairpinOffsetY");
    assign_any(
        &mut opts.duration_spacing_compression,
        "durationSpacingCompression",
    );
    assign_any(
        &mut opts.measure_width_compression,
        "measureWidthCompression",
    );
    if let Some(value) = get_optional_bool("hideVoice2Rests") {
        opts.hide_voice2_rests = value;
    }

    opts
}

#[cfg(feature = "layout-wasm")]
fn layout_scene_to_js(scene: &drummark_layout::LayoutScene) -> JsValue {
    drummark_layout::layout_scene_to_js(scene)
}
