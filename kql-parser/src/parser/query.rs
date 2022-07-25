
use crate::spans::{M, MBox, Span, span_precedes_span, join_spans};
use crate::lexer::Token;
use crate::ast;
use crate::ast::query::{Query, TabularOperator};

use crate::parser::{
    ParserError, ParseInput,
    // statements::parse_block,
    // types::parse_valtype,
    // expressions::parse_expression
};

pub fn parse_query(input: &mut ParseInput) -> Result<Query, ParserError> {
    let table = parse_term(input)?;
    let operators = parse_operators(input)?;
    
    Ok(Query {
        table,
        operators
    })
}

fn parse_term(input: &mut ParseInput) -> Result<M<String>, ParserError> {
    let token = input.next()?;
    match token.value.clone() {
        Token::Term(s) => Ok(M::new(s, token.span.clone())),
        _ => Err(input.unexpected_token("Term expected"))
    }
}

fn parse_operators(input: &mut ParseInput) -> Result<Vec<(M<String>, TabularOperator)>, ParserError> {
    let mut operators = Vec::new();

    while input.has(1) {
        operators.push(parse_operator(input)?);
    }

    Ok(operators)
}

fn parse_operator(input: &mut ParseInput) -> Result<(M<String>, TabularOperator), ParserError> {
    let operator_name = parse_kebab_term(input)?;

    let operator = match operator_name.value.as_str() {
        "count" => TabularOperator::Count,
        "distinct" => parse_distinct(input)?,
        "extend" => parse_extend(input)?,
        "join" => parse_join(input)?,
        "limit" | "take" => parse_limit(input)?,
        "project" => parse_project(input)?,
        "sort" | "order" => parse_sort(input)?,
        "summarize" => parse_summarize(input)?,
        "top" => parse_top(input)?,
        "where" => parse_where(input)?,
        _ => return Err(input.general_error("No tabular operator with this name"))
    };

    Ok((operator_name, operator))
}

fn parse_kebab_term(input: &mut ParseInput) -> Result<M<String>, ParserError> {
    let first_term = parse_term(input)?;
    let mut span = first_term.span.clone();
    let mut name = first_term.value;

    while input.has(1) {
        let checkpoint = input.checkpoint();
        let hyphen = input.next_if(Token::Div);
        if hyphen.is_some() {
            let term = parse_term(input)?;
            if span_precedes_span(span.clone(), term.span.clone()) {
                span = join_spans(span, term.span.clone());
                name.push('-');
                name.push_str(&term.value);
            } else {
                input.restore(checkpoint);
                break
            }
        } else {
            break
        }
    }

    Ok(M::new(name, span))
}

fn parse_distinct(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    Err(input.unsupported_error("distinct operator"))
}

fn parse_columns(input: &mut ParseInput) -> Result<ast::query::Columns, ParserError> {
    Err(input.unsupported_error("columns operator"))
}

fn parse_extend(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    Err(input.unsupported_error("extend operator"))
}

fn parse_join(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    Err(input.unsupported_error("join operator"))
}

fn parse_limit(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    Err(input.unsupported_error("limit operator"))
}

fn parse_project(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    Err(input.unsupported_error("project operator"))
}

fn parse_sort(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    Err(input.unsupported_error("sort operator"))
}

fn parse_summarize(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    Err(input.unsupported_error("summarize operator"))
}

fn parse_top(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    Err(input.unsupported_error("top operator"))
}

fn parse_where(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    Err(input.unsupported_error("where operator"))
}