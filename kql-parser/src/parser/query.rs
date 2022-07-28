use crate::ast::query::{Query, TabularOperator, SortOrder, NullsPosition};
use crate::ast::{self, ColumnDefinition, Sorting};

use crate::lexer::Token;
use crate::spans::{join_spans, span_precedes_span, M};

use crate::parser::{parse_term, ParseInput, ParserError};

use super::expression::parse_expression;

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
    input.assert_next(Token::Pipe, "Tabular operators begin with pipe")?;

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
        let hyphen = input.next_if(Token::Sub);

        if hyphen.is_some() {
            if let Ok(term) = parse_term(input) {
                if span_precedes_span(span.clone(), term.span.clone()) {
                    span = join_spans(span, term.span.clone());
                    name.push('-');
                    name.push_str(&term.value);
                    continue;
                }
            }
        }

        input.restore(checkpoint);
        break;
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
        _ => return Err(input.unexpected_token("Expected number literal for limit argument")),
    };
    Ok(TabularOperator::Limit {
        limit: M::new(amount, token.span.clone()),
    })
}

fn parse_project(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    let mut columns = Vec::new();

    if input.peek()?.value == Token::Pipe {
        return Ok(TabularOperator::Project { columns });
    }

    columns.push(parse_column_definition(input)?);

    while input.next_if(Token::Comma).is_some() {
        columns.push(parse_column_definition(input)?);
    }

    Ok(TabularOperator::Project { columns })
}

fn parse_column_definition(input: &mut ParseInput) -> Result<ColumnDefinition, ParserError> {
    let column = parse_term(input)?;
    let expr = if input.next_if(Token::Assign).is_some() {
        Some(parse_expression(input)?)
    } else {
        None
    };
    Ok(ColumnDefinition { column, expr })
}

fn parse_sort(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    let mut sortings = Vec::new();
    let first_term = parse_term(input)?;
    let by_kwd = match first_term.value.as_str() {
        "by" => first_term.span.clone(),
        _ => return Err(input.unexpected_token("Expected 'by' keyword"))
    };
    loop {
        let column = match parse_term(input) {
            Ok(column) => column,
            Err(_) => return Err(input.unexpected_token("Expected column name"))
        };

        let checkpoint = input.checkpoint();

        let order_term = parse_term(input)?;
        let order = match order_term.value.as_str() {
            "asc" => Some(M::new(SortOrder::Ascending, order_term.span.clone())),
            "desc" => Some(M::new( SortOrder::Descending, order_term.span.clone())),
            _ => {
                input.restore(checkpoint);
                None
            }
        };
        
        let checkpoint = input.checkpoint();
        
        let nulls_kwd = parse_term(input)?;
        let nulls = match nulls_kwd.value.as_str() {
            "nulls" => {
                let nulls_pos = parse_term(input)?;
                match nulls_pos.value.as_str() {
                    "first" => Some((nulls_kwd.span.clone(), M::new(NullsPosition::First, nulls_pos.span.clone()))),
                    "last" => Some((nulls_kwd.span.clone(), M::new(NullsPosition::Last, nulls_pos.span.clone()))),
                    _ => return Err(input.unexpected_token("Expected nulls position"))
                }
            }
            _ => {
                input.restore(checkpoint);
                None
            }
        };
    
        let sorting = Sorting { column, order, nulls };


        sortings.push(sorting);


        if input.next_if(Token::Comma).is_none() {
            break;
        }
    }

    Ok(TabularOperator::Sort { by_kwd, sortings })
}

fn parse_summarize(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    let mut result_columns: Vec<ColumnDefinition> = Vec::new();

    result_columns.push(parse_column_definition(input)?);

    while input.next_if(Token::Comma).is_some() {
        result_columns.push(parse_column_definition(input)?);
    }

    let next_term = parse_term(input)?;
    let by_kwd = match next_term.value.as_str() {
        "by" => next_term.span.clone(),
        _ => return Err(input.unexpected_token("Expected 'by' keyword"))
    };

    let mut grouping_columns: Vec<ColumnDefinition> = Vec::new();

    grouping_columns.push(parse_column_definition(input)?);
    while input.next_if(Token::Comma).is_some() {
        grouping_columns.push(parse_column_definition(input)?);
    }

    Ok(TabularOperator::Summarize { result_columns, by_kwd, grouping_columns })
}

fn parse_top(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    Err(input.unsupported_error("top operator"))
}

fn parse_where(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    let expr = parse_expression(input)?;
    Ok(TabularOperator::Where { expr })
}

#[cfg(test)]
mod tests {
    use miette::Report;

    use super::*;
    use crate::{parser::tests::make_input, spans::Span};
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_summarize_supports_groupings() {
        let cases = [
            ("NumTransactions=2, Total=foobar by Fruit, StartOfMonth", Span::from((0, 124))),
        ];
        for (source, span) in cases {
            let result = parse_summarize(&mut make_input(source));
            match result {
                Ok(_) => {},
                Err(error) => {
                    println!("{:?}", Report::new(error));
                    panic!();
                }
            }
        }
    }
}