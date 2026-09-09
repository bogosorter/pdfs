use crate::ast::*;
use crate::error::Error;

use std::collections::HashMap;
use std::ops::Range;

type State = HashMap<String, Type>;
type TypingResult<T> = Result<T, Error>;

pub fn type_check(program: &UntypedProgram) -> TypingResult<TypedProgram> {
    let mut state = initial_state();

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
        Expression::StringLiteral(s, range) => Ok(Expression::StringLiteral(s.clone(), range.clone())),

        Expression::Variable(name, range, _) => {
            if let Some(t) = state.get(name) {
                Ok(Expression::Variable(name.clone(), range.clone(), t.clone()))
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
        }
    }
}

// Initializes an environment with the built-in function
fn initial_state() -> State {
    let mut state = HashMap::new();
    state.insert(String::from("read"), Type::Function(vec![Type::String], Box::new(Type::PDF)));
    state.insert(String::from("write"), Type::Function(vec![Type::String, Type::PDF], Box::new(Type::Unit)));
    state
}

fn type_error(message: &str, range: &Range<usize>) -> TypingResult<Expression<Type>> {
    Err(Error::new(String::from(message), range.clone()))
}
