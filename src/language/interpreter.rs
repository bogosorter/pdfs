use std::collections::HashMap;
use std::ops::Range;
use crate::{
    language::ast::*, utils::{
        error::Error, pdf::{OutOfBounds, PDF, Page, ReadError}
    }
};


#[derive(Clone)]
enum Value {
    Unit,
    Integer(i32),
    String(String),
    Page(Page),
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
        Statement::Iteration(loop_variable, iterator, statements, _) => {
            let interpreted_iterator = interpret_expression(state, iterator)?;
            let pages = if let Value::PDF(pdf) = interpreted_iterator {
                pdf.pages()
            } else {
                unreachable!("iterator has been type-checked to be a pdf");
            };

            for page in pages {
                state.insert(loop_variable.clone(), Value::Page(page));
                for s in statements {
                    interpret_statement(state, s)?;
                }
            }

            Ok(())
        },
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
        Expression::IntegerLiteral(i, _) => Ok(Value::Integer(*i)),
        Expression::StringLiteral(s, _) => Ok(Value::String(s.clone())),
        Expression::Variable(name, _, _) => Ok(state.get(name).unwrap().clone()),
        Expression::BuiltIn(BuiltInExpression::BlankPage, _) => Ok(Value::Page(Page::blank())),
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
        },

        Expression::Index(base, index, range) => {
            let interpreted_base = interpret_expression(state, base)?;
            let interpreted_index = interpret_expression(state, index)?;
            match (interpreted_base, interpreted_index) {
                (Value::PDF(pdf), Value::Integer(i)) => get_page(&pdf, i, range),
                _ => unreachable!("can only index PDFs with Integers")
            }
        },

        Expression::Slice(base, start, end, step, range) => {
            let interpreted_base = interpret_expression(state, base)?;

            let interpreted_start =
                if let Some(e) = start {
                    let interpreted = interpret_expression(state, e)?;
                    match interpreted {
                        Value::Integer(i) => Some(i),
                        _ => unreachable!("can only index PDFs with Integers")
                    }
                } else {
                    None
                };

            let interpreted_end =
                if let Some(e) = end {
                    let interpreted = interpret_expression(state, e)?;
                    match interpreted {
                        Value::Integer(i) => Some(i),
                        _ => unreachable!("can only index PDFs with Integers")
                    }
                } else {
                    None
                };

            let interpreted_step =
                if let Some(e) = step {
                    let interpreted = interpret_expression(state, e)?;
                    match interpreted {
                        Value::Integer(i) => Some(i),
                        _ => unreachable!("can only index PDFs with Integers")
                    }
                } else {
                    None
                };

            match interpreted_base {
                Value::PDF(pdf) => get_slice(&pdf, interpreted_start, interpreted_end, interpreted_step, range),
                _ => unreachable!("can only index PDFs")
            }
        },

        Expression::PDFConstructor(pages, _) => {
            let mut interpreted_pages = Vec::new();
            for page in pages {
                let interpreted_page = interpret_expression(state, page)?;
                match interpreted_page {
                    Value::Page(p) => interpreted_pages.push(p),
                    _ => unreachable!()
                }
            }

            Ok(Value::PDF(PDF::from(interpreted_pages)))
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

fn get_page(pdf: &PDF, i: i32, range: &Range<usize>) -> InterpreterResult<Value> {
    match pdf.page(i) {
        Ok(page) => Ok(Value::Page(page)),
        Err(OutOfBounds) =>
            Err(Error::new(format!("trying to access a page that does not exist in document (page {})", i), range.clone()))
    }
}

fn get_slice(pdf: &PDF, start: Option<i32>, end: Option<i32>, step: Option<i32>, range: &Range<usize>) -> InterpreterResult<Value> {
    match pdf.slice(start, end, step) {
        Ok(pdf) => Ok(Value::PDF(pdf)),
        Err(OutOfBounds) =>
            Err(Error::new(format!("trying to access a range that does not exist in document"), range.clone()))
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
