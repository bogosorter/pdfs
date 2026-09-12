use std::collections::HashMap;
use std::ops::Range;
use crate::{
    language::ast::*,
    utils::error::Error
};


type State = HashMap<String, Type>;
type TypingResult<T> = Result<T, Error>;

pub fn type_check(program: &UntypedProgram) -> TypingResult<TypedProgram> {
    let mut state = HashMap::new();

    let mut typed_statements = Vec::new();
    for statement in program.0.iter() {
        let typed = type_check_statement(&mut state, &statement)?;
        typed_statements.push(typed);
    }

    Ok(Program(typed_statements))
}

fn type_check_statement(state: &mut State, statement: &Statement<()>) -> TypingResult<Statement<Type>> {
    match statement {
        Statement::Assignment(name, expression, range) => {
            let typed_expression = type_check_expression(state, expression)?;

            if typed_expression.t() == Type::Unit {
                return type_error(&format!("cannot assign values of type {}", Type::Unit), range);
            }

            state.insert(name.clone(), typed_expression.t());
            Ok(Statement::Assignment(name.clone(), typed_expression, range.clone()))
        },
        Statement::ExpressionStatement(expression, range) => {
            let typed_expression = type_check_expression(state, expression)?;
            Ok(Statement::ExpressionStatement(typed_expression, range.clone()))
        }
    }
}

fn type_check_expression(state: &State, expression: &Expression<()>) -> TypingResult<Expression<Type>> {
    match expression {
        Expression::IntegerLiteral(i, range) => Ok(Expression::IntegerLiteral(*i, range.clone())),
        Expression::StringLiteral(s, range) => Ok(Expression::StringLiteral(s.clone(), range.clone())),

        Expression::Variable(name, range, _) => {
            if let Some(t) = state.get(name) {
                Ok(Expression::Variable(name.clone(), range.clone(), t.clone()))
            } else if name == "read" {
                Ok(Expression::BuiltIn(BuiltInExpression::Read, range.clone()))
            } else if name == "write" {
                Ok(Expression::BuiltIn(BuiltInExpression::Write, range.clone()))
            } else if name == ">>" {
                Ok(Expression::BuiltIn(BuiltInExpression::Concatenate, range.clone()))
            } else {
                type_error(&format!("variable {name} is not defined"), range)
            }
        },

        Expression::FunctionCall(function, arguments, range, _) => {
            let typed_function = type_check_expression(state, function)?;
            let (argument_types, return_type) = match typed_function.t() {
                Type::Function(argument_types, return_type) => (argument_types, return_type),
                _ => return type_error(&format!("trying to call an expression that is not a function"), range)
            };

            let mut typed_arguments = Vec::new();
            for argument in arguments {
                let typed = type_check_expression(state, argument)?;
                typed_arguments.push(typed);
            }

            if argument_types.len() != typed_arguments.len() {
                return type_error(&format!(
                    "trying to call a function that accepts {} arguments with {} arguments",
                    argument_types.len(),
                    typed_arguments.len()
                ), range);
            }

            let provided_types = typed_arguments.iter().map(Expression::t);
            for (expected, actual) in argument_types.iter().zip(provided_types) {
                if *expected != actual {
                    return type_error(&format!(
                        "expected an argument of type {}, but got an argument of type {}",
                        *expected,
                        actual
                    ), range);
                }
            }

            Ok(Expression::FunctionCall(Box::new(typed_function), typed_arguments, range.clone(), *return_type))
        },

        Expression::Index(base, index, range) => {
            let typed_base = type_check_expression(state, base)?;
            if typed_base.t() != Type::PDF {
                return type_error(&format!(
                    "can only extract pages of type PDF, but got type {}",
                    typed_base.t()
                ), range);
            }

            let typed_index = type_check_expression(state, index)?;
            if typed_index.t() != Type::Integer {
                return type_error(&format!(
                    "can only index with type Integer, but got type {}",
                    typed_index.t()
                ), range);
            }

            Ok(Expression::Index(Box::new(typed_base), Box::new(typed_index), range.clone()))
        },

        Expression::PDFConstructor(pages, range) => {
            let mut typed_pages = Vec::new();
            for page in pages {
                let typed_page = type_check_expression(state, page)?;
                typed_pages.push(typed_page);
            }

            for typed_page in typed_pages.iter() {
                if typed_page.t() != Type::Page {
                    return type_error(&format!(
                        "all the elements of a PDF must be of type Page, but got type {}",
                        typed_page.t()
                    ), typed_page.range());
                }
            }

            Ok(Expression::PDFConstructor(typed_pages, range.clone()))
        },

        Expression::BuiltIn(_, _) => unreachable!("built-ins are only introduced in type-checking")
    }
}

fn type_error<T>(message: &str, range: &Range<usize>) -> TypingResult<T> {
    Err(Error::new(String::from(message), range.clone()))
}
