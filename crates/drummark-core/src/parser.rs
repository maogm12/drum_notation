use crate::ast::{Document, ParseError};

pub struct Parser<'a> {
    source: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn parse(self) -> Result<Document, Vec<ParseError>> {
        Err(vec![ParseError {
            line: 1,
            column: 1,
            message: "parser not yet implemented".to_string(),
        }])
    }
}
