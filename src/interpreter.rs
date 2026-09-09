use crate::ast::*;
use crate::pdf::{PDF, ReadError};
use crate::error::Error;

use std::collections::HashMap;
use std::ops::Range;

// Despite the existence of a type for functions, they do not have a
// corresponding value, since they are built-in.
#[derive(Clone)]
enum Value {
    Unit,
    String(String),
    PDF(PDF)
}

type State = HashMap<String, Value>;
type InterpreterResult<T> = Result<T, Error>;

pub fn interpret(program: &TypedProgram) -> InterpreterResult<()> {
    let mut state = HashMap::new();
    for statement in program.0.iter() {
        interpret_statement(&mut state, statement)?;
    }
    Ok(())
}

fn interpret_statement(state: &mut State, statement: &Statement<Type>) -> InterpreterResult<()> {
    match statement {
        Statement::Assignment(name, expression, _) => {
            let value = interpret_expression(state, expression)?;
            state.insert(name.clone(), value);
            Ok(())
        },
        Statement::ExpressionStatement(expression, _) => {
            interpret_expression(state, expression)?;
            Ok(())
        }
    }
}

fn interpret_expression(state: &State, expression: &Expression<Type>) -> InterpreterResult<Value> {
    match expression {
        Expression::StringLiteral(s, _) => Ok(Value::String(s.clone())),
        Expression::Variable(name, _, _) => Ok(state.get(name).unwrap().clone()),

        Expression::FunctionCall(function, arguments, range, _) => {
            match function.as_ref() {
                Expression::Variable(name, _, _) if name == "read" => {
                    let path = interpret_expression(state, &arguments[0])?;
                    if let Value::String(p) = path {
                        read(&p, range)
                    } else {
                        unreachable!("arguments to read have been type-checked");
                    }
                },
                Expression::Variable(name, _, _) if name == "write" => {
                    let path = interpret_expression(state, &arguments[0])?;
                    let pdf = interpret_expression(state, &arguments[1])?;
                    if let Value::String(a) = path && let Value::PDF(b) = pdf {
                        write(&a, &b)
                    } else {
                        unreachable!("arguments to write have been type-checked");
                    }
                },
                _ => unreachable!("only built-in function calls are allowed")
            }
        }
    }
}

fn read(path: &str, range: &Range<usize>) -> InterpreterResult<Value> {
    match PDF::read(path) {
        Ok(pdf) => Ok(Value::PDF(pdf)),
        Err(ReadError::FileNotFound) => Err(Error::new(format!("couldn't find file {}", path), range.clone())),
        Err(_) => Err(Error::new(String::from("internal error"), range.clone()))
    }
}

fn write(path: &str, pdf: &PDF) -> InterpreterResult<Value> {
    pdf.write(path);
    Ok(Value::Unit)
}
