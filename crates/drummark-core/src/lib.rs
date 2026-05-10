#![allow(dead_code)]

use wasm_bindgen::prelude::*;

mod lexer;
mod ast;
mod parser;
mod error;
mod to_js;

/// Parse a DrumMark source string and return the AST as a JS object.
///
/// Returns a JsValue representing the DocumentSkeleton tree directly
/// consumable by the TypeScript wrapper layer. No JSON serialization
/// is performed — the object tree is constructed via js_sys primitives.
#[wasm_bindgen]
pub fn parse(source: &str) -> JsValue {
    let parser = parser::Parser::new(source);
    match parser.parse() {
        Ok(document) => to_js::document_to_js(&document),
        Err(errors) => to_js::errors_to_js(&errors),
    }
}
