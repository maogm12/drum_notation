use wasm_bindgen::JsValue;
use js_sys::{Array, Object};
use crate::ast::*;

pub fn document_to_js(doc: &Document) -> JsValue {
    let obj = Object::new();
    set(&obj, "headers", &headers_to_js(&doc.headers));
    set(&obj, "paragraphs", &paragraphs_to_js(&doc.paragraphs));
    set(&obj, "errors", &errors_to_js(&doc.errors));
    obj.into()
}

pub fn errors_to_js(errors: &[ParseError]) -> JsValue {
    let arr = Array::new();
    for e in errors {
        let obj = Object::new();
        set(&obj, "line", &JsValue::from_f64(e.line as f64));
        set(&obj, "column", &JsValue::from_f64(e.column as f64));
        set(&obj, "message", &JsValue::from_str(&e.message));
        arr.push(&obj);
    }
    arr.into()
}

fn headers_to_js(h: &HeaderSection) -> JsValue {
    let obj = Object::new();
    if let Some(ref v) = h.title { set(&obj, "title", &JsValue::from_str(v)); }
    if let Some(ref v) = h.subtitle { set(&obj, "subtitle", &JsValue::from_str(v)); }
    if let Some(ref v) = h.composer { set(&obj, "composer", &JsValue::from_str(v)); }
    if let Some(v) = h.tempo { set(&obj, "tempo", &JsValue::from_f64(v as f64)); }
    if let Some((b, u)) = h.time { set(&obj, "time", &frac_to_js(b, u)); }
    if let Some(ref v) = h.grouping { set(&obj, "grouping", &vec_u32_to_js(v)); }
    if let Some((b, u)) = h.note { set(&obj, "note", &frac_to_js(b, u)); }
    if let Some(v) = h.divisions { set(&obj, "divisions", &JsValue::from_f64(v as f64)); }
    obj.into()
}

fn paragraphs_to_js(paras: &[Paragraph]) -> JsValue {
    let arr = Array::new();
    for p in paras {
        let obj = Object::new();
        if let Some((b, u)) = p.note {
            set(&obj, "note", &frac_to_js(b, u));
        }
        set(&obj, "lines", &track_lines_to_js(&p.lines));
        arr.push(&obj);
    }
    arr.into()
}

fn track_lines_to_js(lines: &[TrackLine]) -> JsValue {
    let arr = Array::new();
    for line in lines {
        let obj = Object::new();
        // Always include track, even if None (set to JsValue::NULL)
        if let Some(ref t) = line.track {
            set(&obj, "track", &JsValue::from_str(t));
        } else {
            set(&obj, "track", &JsValue::NULL);
        }
        set(&obj, "measures", &measures_to_js(&line.measures));
        arr.push(&obj);
    }
    arr.into()
}

fn measures_to_js(measures: &[MeasureSection]) -> JsValue {
    let arr = Array::new();
    for m in measures {
        let obj = Object::new();
        set(&obj, "barline", &barline_to_js(&m.barline));
        set(&obj, "barlineLocation", &source_location_to_js(&m.barline_location));
        if let Some(ref closing) = m.closing_barline {
            set(&obj, "closingBarline", &barline_to_js(closing));
        }
        if let Some(ref location) = m.closing_barline_location {
            set(&obj, "closingBarlineLocation", &source_location_to_js(location));
        }
        set(&obj, "tokens", &exprs_to_js(&m.tokens));
        arr.push(&obj);
    }
    arr.into()
}

fn source_location_to_js(location: &SourceLocation) -> JsValue {
    let obj = Object::new();
    set(&obj, "line", &JsValue::from_f64(location.line as f64));
    set(&obj, "column", &JsValue::from_f64(location.column as f64));
    set(&obj, "offset", &JsValue::from_f64(location.offset as f64));
    obj.into()
}

fn exprs_to_js(exprs: &[MeasureExpr]) -> JsValue {
    let arr = Array::new();
    for e in exprs {
        arr.push(&expr_to_js(e));
    }
    arr.into()
}

fn expr_to_js(e: &MeasureExpr) -> JsValue {
    let obj = Object::new();
    match e {
        MeasureExpr::BasicNote(note) => {
            set(&obj, "kind", &JsValue::from_str("basic"));
            set(&obj, "glyph", &JsValue::from_str(&note.glyph));
            if note.dots > 0 { set(&obj, "dots", &JsValue::from_f64(note.dots as f64)); }
            if note.halves > 0 { set(&obj, "halves", &JsValue::from_f64(note.halves as f64)); }
            if note.stars > 0 { set(&obj, "stars", &JsValue::from_f64(note.stars as f64)); }
            if !note.modifiers.is_empty() {
                set(&obj, "modifiers", &strings_to_js(&note.modifiers));
            }
        }
        MeasureExpr::SummonedNote { track, note } => {
            set(&obj, "kind", &JsValue::from_str("summoned"));
            set(&obj, "track", &JsValue::from_str(track));
            set(&obj, "glyph", &JsValue::from_str(&note.glyph));
            if note.dots > 0 { set(&obj, "dots", &JsValue::from_f64(note.dots as f64)); }
            if note.halves > 0 { set(&obj, "halves", &JsValue::from_f64(note.halves as f64)); }
            if note.stars > 0 { set(&obj, "stars", &JsValue::from_f64(note.stars as f64)); }
            if !note.modifiers.is_empty() {
                set(&obj, "modifiers", &strings_to_js(&note.modifiers));
            }
        }
        MeasureExpr::RoutedBracedBlock { track, content } => {
            set(&obj, "kind", &JsValue::from_str("routedBraced"));
            set(&obj, "track", &JsValue::from_str(track));
            set(&obj, "content", &exprs_to_js(content));
        }
        MeasureExpr::InlineBracedBlock(content) => {
            set(&obj, "kind", &JsValue::from_str("inlineBraced"));
            set(&obj, "content", &exprs_to_js(content));
        }
        MeasureExpr::Group(g) => {
            set(&obj, "kind", &JsValue::from_str("group"));
            if let Some(n) = g.n {
                set(&obj, "n", &JsValue::from_f64(n as f64));
            }
            set(&obj, "items", &exprs_to_js(&g.items));
            if !g.modifiers.is_empty() {
                set(&obj, "modifiers", &strings_to_js(&g.modifiers));
            }
        }
        MeasureExpr::CombinedHit(hits) => {
            set(&obj, "kind", &JsValue::from_str("combinedHit"));
            set(&obj, "hits", &exprs_to_js(hits));
        }
        MeasureExpr::MeasureRepeat(count) => {
            set(&obj, "kind", &JsValue::from_str("measureRepeat"));
            set(&obj, "count", &JsValue::from_f64(*count as f64));
        }
        MeasureExpr::MultiRest(count) => {
            set(&obj, "kind", &JsValue::from_str("multiRest"));
            set(&obj, "count", &JsValue::from_f64(*count as f64));
        }
        MeasureExpr::InlineRepeat(times) => {
            set(&obj, "kind", &JsValue::from_str("inlineRepeat"));
            set(&obj, "times", &JsValue::from_f64(*times as f64));
        }
        MeasureExpr::Crescendo => {
            set(&obj, "kind", &JsValue::from_str("crescendo"));
        }
        MeasureExpr::Decrescendo => {
            set(&obj, "kind", &JsValue::from_str("decrescendo"));
        }
        MeasureExpr::HairpinEnd => {
            set(&obj, "kind", &JsValue::from_str("hairpinEnd"));
        }
        MeasureExpr::NavMarker(name) => {
            set(&obj, "kind", &JsValue::from_str("navMarker"));
            set(&obj, "name", &JsValue::from_str(name));
        }
        MeasureExpr::NavJump(name) => {
            set(&obj, "kind", &JsValue::from_str("navJump"));
            set(&obj, "name", &JsValue::from_str(name));
        }
    }
    obj.into()
}

fn notes_to_js(notes: &[NoteExpr]) -> JsValue {
    let arr = Array::new();
    for n in notes {
        let obj = Object::new();
        set(&obj, "glyph", &JsValue::from_str(&n.glyph));
        if n.dots > 0 { set(&obj, "dots", &JsValue::from_f64(n.dots as f64)); }
        if n.halves > 0 { set(&obj, "halves", &JsValue::from_f64(n.halves as f64)); }
        if n.stars > 0 { set(&obj, "stars", &JsValue::from_f64(n.stars as f64)); }
        if !n.modifiers.is_empty() {
            set(&obj, "modifiers", &strings_to_js(&n.modifiers));
        }
        arr.push(&obj);
    }
    arr.into()
}

fn barline_to_js(b: &Barline) -> JsValue {
    let obj = Object::new();
    match b {
        Barline::Regular => {
            set(&obj, "type", &JsValue::from_str("|"));
        }
        Barline::Double => {
            set(&obj, "type", &JsValue::from_str("||"));
        }
        Barline::RepeatStart => {
            set(&obj, "type", &JsValue::from_str("|:"));
        }
        Barline::RepeatEnd => {
            set(&obj, "type", &JsValue::from_str(":|"));
        }
        Barline::VoltaTerminator => {
            set(&obj, "type", &JsValue::from_str("|."));
        }
        Barline::RepeatEndVoltaTerminator => {
            set(&obj, "type", &JsValue::from_str(":|."));
        }
        Barline::DoubleVoltaTerminator => {
            set(&obj, "type", &JsValue::from_str("||."));
        }
        Barline::VoltaRepeatStart => {
            set(&obj, "type", &JsValue::from_str("|:."));
        }
        Barline::Volta { prefix, numbers } => {
            set(&obj, "type", &JsValue::from_str("volta"));
            set(&obj, "prefix", &JsValue::from_str(prefix));
            set(&obj, "numbers", &vec_u32_to_js(numbers));
        }
    }
    obj.into()
}

fn strings_to_js(strings: &[String]) -> JsValue {
    let arr = Array::new();
    for s in strings {
        arr.push(&JsValue::from_str(s));
    }
    arr.into()
}

fn vec_u32_to_js(nums: &[u32]) -> JsValue {
    let arr = Array::new();
    for &n in nums {
        arr.push(&JsValue::from_f64(n as f64));
    }
    arr.into()
}

fn frac_to_js(beats: u32, beat_unit: u32) -> JsValue {
    let arr = Array::new();
    arr.push(&JsValue::from_f64(beats as f64));
    arr.push(&JsValue::from_f64(beat_unit as f64));
    arr.into()
}

fn set(obj: &Object, key: &str, val: &JsValue) {
    js_sys::Reflect::set(obj, &JsValue::from_str(key), val).unwrap();
}
