use crate::ast::*;
use crate::pdf::PDF;

use std::error::Error;
use std::fmt::Display;
use std::collections::HashMap;

#[derive(Debug)]
pub struct InterpreterError;

// Despite the existence of a type for functions, they do not have a
// corresponding value, since they are built-in.
#[derive(Clone)]
enum Value {
    Unit,
    String(String),
    PDF(PDF)
}

type State = HashMap<String, Value>;
type InterpreterResult<T> = Result<T, InterpreterError>;

pub fn interpret(program: &TypedProgram) -> InterpreterResult<()> {
    let mut state = HashMap::new();
    for statement in program.0.iter() {
        interpret_statement(&mut state, statement)?;
    }
    Ok(())
}

fn interpret_statement(state: &mut State, statement: &Statement<Type>) -> InterpreterResult<()> {
    match statement {
        Statement::Assignment(name, expression) => {
            let value = interpret_expression(state, expression)?;
            state.insert(name.clone(), value);
            Ok(())
        },
        Statement::ExpressionStatement(expression) => {
            interpret_expression(state, expression)?;
            Ok(())
        }
    }
}

fn interpret_expression(state: &State, expression: &Expression<Type>) -> InterpreterResult<Value> {
    match expression {
        Expression::StringLiteral(s) => Ok(Value::String(s.clone())),
        Expression::Variable(name, _) => Ok(state.get(name).unwrap().clone()),

        Expression::FunctionCall(function, arguments, _) => {
            match function.as_ref() {
                Expression::Variable(name, _) if name == "read" => {
                    let path = interpret_expression(state, &arguments[0])?;
                    if let Value::String(p) = path {
                        read(&p)
                    } else {
                        unreachable!("arguments to read have been type-checked");
                    }
                },
                Expression::Variable(name, _) if name == "write" => {
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

fn read(path: &str) -> InterpreterResult<Value> {
    match PDF::read(path) {
        Ok(pdf) => Ok(Value::PDF(pdf)),
        Err(_) => Err(InterpreterError)
    }
}

fn write(path: &str, pdf: &PDF) -> InterpreterResult<Value> {
    pdf.write(path);
    Ok(Value::Unit)
}

impl Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "internal error")
    }
}

impl Error for InterpreterError {}
