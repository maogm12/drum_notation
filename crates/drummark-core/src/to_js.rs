use wasm_bindgen::JsValue;
use js_sys::{Array, Object};
use crate::ast::{Document, ParseError};

pub fn document_to_js(doc: &Document) -> JsValue {
    let _ = doc;
    Object::new().into()
}

pub fn errors_to_js(errors: &[ParseError]) -> JsValue {
    let _ = errors;
    Array::new().into()
}
