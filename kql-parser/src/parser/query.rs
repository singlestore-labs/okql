use crate::ast;
use crate::ast::query::{Query, TabularOperator};
use crate::lexer::Token;
use crate::spans::{join_spans, span_precedes_span, M};

use crate::parser::{parse_term, ParseInput, ParserError};

pub fn parse_query(input: &mut ParseInput) -> Result<Query, ParserError> {
    let table = parse_term(input)?;
    let operators = parse_operators(input)?;

    Ok(Query { table, operators })
}

fn parse_operators(
    input: &mut ParseInput,
) -> Result<Vec<(M<String>, TabularOperator)>, ParserError> {
    let mut operators = Vec::new();

    while !input.done() {
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
        _ => return Err(input.general_error("No tabular operator with this name")),
    };

    Ok((operator_name, operator))
}

fn parse_kebab_term(input: &mut ParseInput) -> Result<M<String>, ParserError> {
    let first_term = parse_term(input)?;
    let mut span = first_term.span.clone();
    let mut name = first_term.value;

    while !input.done() {
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
                break;
            }
        } else {
            break;
        }
    }

    Ok(M::new(name, span))
}

fn parse_distinct(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    Ok(TabularOperator::Distinct {
        columns: parse_columns(input)?,
    })
}

fn parse_columns(input: &mut ParseInput) -> Result<ast::query::Columns, ParserError> {
    let star_span = input.next_if(Token::Star);
    if let Some(span) = star_span {
        return Ok(ast::query::Columns::Wildcard(span));
    }

    let first_name = parse_term(input)?;
    let mut columns = vec![first_name];

    while input.next_if(Token::Comma).is_some() {
        columns.push(parse_term(input)?);
    }

    Ok(ast::query::Columns::Explicit(columns))
}

fn parse_extend(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    Err(input.unsupported_error("extend operator"))
}

fn parse_join(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    Err(input.unsupported_error("join operator"))
}

fn parse_limit(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    let token = input.next()?;
    let amount = match token.value {
        Token::IntLiteral(value) => value as i64,
        Token::LongLiteral(value) => value,
        _ => return Err(input.unexpected_token("Expected number literal for limit argument"))
    };
    Ok(TabularOperator::Limit { limit: M::new(amount, token.span.clone()) })
}

fn parse_project(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    Err(input.unsupported_error("project operator"))
}

fn parse_sort(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    let first_term = parse_term(input)?;
    let mut span = first_term.span.clone();
    let by_kwd = match first_term.value.as_str() {
        "by" => span,
        _ => return Err(input.unexpected_token("Expected 'by' keyword"))
    };
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
