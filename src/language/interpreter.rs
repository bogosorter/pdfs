use std::collections::HashMap;
use std::ops::Range;
use crate::{
    language::ast::*,
    utils::{
        pdf::{PDF, ReadError},
        error::Error
    }
};


#[derive(Clone)]
enum Value {
    Unit,
    String(String),
    PDF(PDF),
    BuiltIn(BuiltInExpression)
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
        Expression::BuiltIn(t, _) => Ok(Value::BuiltIn(*t)),

        Expression::FunctionCall(function, arguments, range, _) => {
            let interpreted_function = interpret_expression(state, function)?;

            match interpreted_function {
                Value::BuiltIn(BuiltInExpression::Read) => {
                    let path = interpret_expression(state, &arguments[0])?;
                    match path {
                        Value::String(path) => read(&path, range),
                        _ => unreachable!("arguments to read have been type-checked")
                    }
                },
                Value::BuiltIn(BuiltInExpression::Write) => {
                    let path = interpret_expression(state, &arguments[0])?;
                    let pdf = interpret_expression(state, &arguments[1])?;
                    match (path, pdf) {
                        (Value::String(path), Value::PDF(pdf)) => write(&path, &pdf),
                        _ => unreachable!("arguments to write have been type-checked")
                    }
                },
                Value::BuiltIn(BuiltInExpression::Concatenate) => {
                    let left = interpret_expression(state, &arguments[0])?;
                    let right = interpret_expression(state, &arguments[1])?;
                    match (left, right) {
                        (Value::PDF(left), Value::PDF(right)) => concatenate(&left, &right),
                        _ => unreachable!("arguments to write have been type-checked")
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

fn concatenate(left: &PDF, right: &PDF) -> InterpreterResult<Value> {
    let mut pages = left.pages();
    pages.extend(right.pages());
    let result = PDF::from(pages);
    Ok(Value::PDF(result))
}
