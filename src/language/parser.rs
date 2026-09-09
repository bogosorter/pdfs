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

    for call in children {
        let arguments = call.into_inner().map(parse_expression).collect();
        result = Expression::FunctionCall(Box::new(result), arguments, range.clone(), ());
    }

    result
}

fn parse_atom(atom: Pair<'_, Rule>) -> Expression<()> {
    let range = atom.as_span().start()..atom.as_span().end();
    match atom.as_rule() {
        Rule::string_literal => {
            let content = atom.as_str();
            let trimmed = &content[1..content.len() - 1];
            Expression::StringLiteral(String::from(trimmed), range)
        },
        Rule::identifier => {
            let content = atom.as_str();
            Expression::Variable(String::from(content), range, ())
        }
        _ => unreachable!()
    }
}
