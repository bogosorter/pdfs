use pest::{Parser, iterators::Pair};
use pest_derive::Parser;
use pest::error::InputLocation;
use crate::{
    language::ast::*,
    utils::error::Error
};


#[derive(Parser)]
#[grammar = "language/grammar.pest"]
struct PestParser;

pub fn parse<'a>(text: &str) -> Result<UntypedProgram, Error> {
    let mut parsed = match PestParser::parse(Rule::program, text) {
        Ok(parsed) => parsed,
        Err(error) => {
            let message = error.variant.message().to_string();
            let range = match error.location {
                InputLocation::Pos(p) => p..p + 1,
                InputLocation::Span((s, e)) => s..e,
            };

            return Err(Error::new(message, range));
        }
    };

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
    let range = statement.as_span().start()..statement.as_span().end();
    match statement.as_rule() {
        Rule::assignment => {
            let mut children = statement.into_inner();
            let identifier = children.next().unwrap().as_str();
            let expression = parse_expression(children.next().unwrap());

            Statement::Assignment(String::from(identifier), expression, range)
        },
        Rule::expression_statement => {
            let content = parse_expression(statement.into_inner().next().unwrap());
            Statement::ExpressionStatement(content, range)
        },
        _ => unreachable!()
    }
}

fn parse_expression(expression: Pair<'_, Rule>) -> Expression<()> {
    let range = expression.as_span().start()..expression.as_span().end();
    let mut children = expression.into_inner();

    let term = children.next().unwrap();
    let mut result = parse_term(term);

    for concatenation in children {
        let concatenee = parse_term(concatenation);
        let function = Expression::Variable(String::from(">>"), range.clone(), ());
        result = Expression::FunctionCall(Box::new(function), vec![result, concatenee], range.clone(), ());
    }

    result
}

fn parse_term(term: Pair<'_, Rule>) -> Expression<()> {
    let range = term.as_span().start()..term.as_span().end();
    let mut children = term.into_inner();

    let atom = children.next().unwrap().into_inner().next().unwrap();
    let mut result = parse_atom(atom);

    for modifier in children {
        match modifier.as_rule() {
            Rule::call => {
                let arguments = modifier.into_inner().map(parse_expression).collect();
                result = Expression::FunctionCall(Box::new(result), arguments, range.clone(), ());
            },
            Rule::index => {
                let index = parse_expression(modifier.into_inner().next().unwrap());
                result = Expression::Index(Box::new(result), Box::new(index), range.clone());
            },
            Rule::range => {
                let mut start = None;
                let mut end = None;

                for child in modifier.into_inner() {
                    match child.as_rule() {
                        Rule::range_start => {
                            start = Some(Box::new(parse_expression(child)));
                        },
                        Rule::range_end => {
                            end = Some(Box::new(parse_expression(child)));
                        },
                        _ => unreachable!()
                    }
                }

                result = Expression::Range(Box::new(result), start, end, range.clone());
            },
            _ => unreachable!()
        }
    }

    result
}

fn parse_atom(atom: Pair<'_, Rule>) -> Expression<()> {
    let range = atom.as_span().start()..atom.as_span().end();
    match atom.as_rule() {
        Rule::integer_literal => {
            let content = atom.as_str();
            Expression::IntegerLiteral(content.parse().unwrap(), range)
        },
        Rule::string_literal => {
            let content = atom.as_str();
            let trimmed = &content[1..content.len() - 1];
            Expression::StringLiteral(String::from(trimmed), range)
        },
        Rule::identifier => {
            let content = atom.as_str();
            Expression::Variable(String::from(content), range, ())
        },
        Rule::array => {
            let elements = atom.into_inner().into_iter().map(parse_expression).collect();
            Expression::PDFConstructor(elements, range.clone())
        },
        _ => unreachable!()
    }
}
