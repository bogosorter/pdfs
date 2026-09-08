use crate::ast::{Program, TypedProgram};

use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub struct TypeCheckerError;

pub fn type_check(program: &Program) -> Result<TypedProgram, TypeCheckerError> {
    panic!("type_check is not implemented");
}

impl Display for TypeCheckerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!("display is not implemented for type checker error");
    }
}

impl Error for TypeCheckerError {}
