use crate::ast::TypedProgram;

use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub struct InterpreterError;

pub fn interpret(program: &TypedProgram) -> Result<(), InterpreterError> {
    panic!("interpreted is not implemented");
}

impl Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!("display is not implemented for parser error");
    }
}

impl Error for InterpreterError {}
