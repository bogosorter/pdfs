use crate::ast::Statement::{Assignment, ExpressionStatement};
use crate::ast::*;
use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub struct ParserError(pest::error::Error<Rule>);

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
struct PestParser;

pub fn parse(text: &str) -> Result<UntypedProgram, ParserError> {
    let mut parsed = PestParser::parse(Rule::program, text).map_err(ParserError)?;
    let rule = parsed.next().unwrap();

    let mut statements = Vec::new();
    for line in rule.into_inner() {
        match line.as_rule() {
            Rule::statement => {
                let pair = line.into_inner().next().unwrap();
                statements.push(parse_statement(pair));
            },
            Rule::EOI => break,
            _ => unreachable!()
        }
    }

    Ok(Program(statements))
}

fn parse_statement(statement: Pair<'_, Rule>) -> Statement<()> {
    match statement.as_rule() {
        Rule::assignment => {
            let mut children = statement.into_inner();
            let identifier = children.next().unwrap().as_str();
            let expression = parse_expression(children.next().unwrap());
            Assignment(String::from(identifier), expression)
        },
        Rule::expression_statement => {
            let content = parse_expression(statement.into_inner().next().unwrap());
            ExpressionStatement(content)
        },
        _ => unreachable!()
    }
}

fn parse_expression(expression: Pair<'_, Rule>) -> Expression<()> {
    let mut children = expression.into_inner();
    let mut atom = children.next().unwrap().into_inner();
    let mut result = parse_atom(atom.next().unwrap());

    for call in atom {
        let arguments = call.into_inner().map(parse_expression).collect();
        result = Expression::FunctionCall(Box::new(result), arguments, ());
    }

    result
}

fn parse_atom(atom: Pair<'_, Rule>) -> Expression<()> {
    match atom.as_rule() {
        Rule::string_literal => {
            let content = atom.into_inner().next().unwrap().as_str();
            Expression::StringLiteral(String::from(content))
        },
        Rule::identifier => {
            let content = atom.as_str();
            Expression::Variable(String::from(content), ())
        }
        _ => unreachable!()
    }
}


impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ParserError {}
