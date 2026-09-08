use crate::ast::Program;

use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub struct ParserError;

pub fn parse(text: &str) -> Result<Program, ParserError> {
    panic!("parse is not implemented");
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!("display is not implemented for parser error");
    }
}

impl Error for ParserError {}
