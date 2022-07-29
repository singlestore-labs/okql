use crate::ast::query::{NullsPosition, Query, SortOrder, TabularOperator};
use crate::ast::{self, ColumnDefinition, Sorting, JoinParams, JoinKind, JoinAttribute};

use crate::lexer::Token;
use crate::spans::{join_spans, span_precedes_span, M};

use crate::parser::{parse_term, ParseInput, ParserError};

use super::expression::parse_expression;
use super::parse_dollar_term;

pub fn parse_query(input: &mut ParseInput) -> Result<Query, ParserError> {
    let table = parse_term(input)?;
    let operators = parse_operators(input)?;

    Ok(Query { table, operators })
}

fn parse_operators(
    input: &mut ParseInput,
) -> Result<Vec<(M<String>, TabularOperator)>, ParserError> {
    let mut operators = Vec::new();

    while input.next_if(Token::Pipe).is_some() {
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

fn parse_join_kind(input: &mut ParseInput) -> Result<JoinKind, ParserError> {
    let term = parse_term(input)?;
    match term.value.as_str() {
        "innerunique" => Ok(JoinKind::InnerUnique),
        "inner" => Ok(JoinKind::Inner),
        "leftouter" => Ok(JoinKind::LeftOuter),
        "rightouter" => Ok(JoinKind::RightOuter),
        "fullouter" => Ok(JoinKind::FullOuter),
        "leftanti" => Ok(JoinKind::LeftAnti),
        "rightanti" => Ok(JoinKind::RightAnti),
        "leftsemi" => Ok(JoinKind::LeftSemi),
        "rightsemi" => Ok(JoinKind::RightSemi),
        _ => return Err(input.unexpected_token("Expected join parameter or '(table_name)'")),
    }
}

fn parse_join(input: &mut ParseInput) -> Result<TabularOperator, ParserError> {
    let checkpoint = input.checkpoint();

    let kind = if input.next_if(Token::LParen).is_some() {
        input.restore(checkpoint);
        None
    } else {
        Some(parse_join_kind(input)?)
    };
        
    let params = JoinParams { kind };

    let lparen = input.next()?;
    if lparen.value != Token::LParen {
        return Err(input.unexpected_token("Expected left paranthesis before table name"));
    };

    let table_query = parse_query(input)?;
    let right_table = Box::new(table_query);

    let rparen = input.next()?;
    if rparen.value != Token::RParen {
        return Err(input.unexpected_token("Expected right paranthesis after table name"));
    };

    let mut attributes = Vec::new();

    loop {
        let checkpoint = input.checkpoint();
        let token = input.next()?;
        let attribute = match token.value.clone() {
            Token::Term(s) => {
                let name = M::new(s, token.span.clone());
                JoinAttribute::Matching{ name }
            },
            Token::DollarTerm(s) => {
                input.restore(checkpoint);
                let dollar_term = parse_dollar_term(input)?;
                if dollar_term.value.as_str() != "left" {
                    return Err(input.unexpected_token("'left' keyword expected"));
                }

                let span = dollar_term.span.clone();
                let dot = input.next()?;
                if dot.value != Token::Dot {
                    return Err(input.unexpected_token("Dot expected"));
                };
                let left_kwd = span;
                let left_name = parse_term(input)?;

                let eq = input.next()?;
                if eq.value != Token::EQ {
                    return Err(input.unexpected_token("'==' expected"));
                };

                let dollar_term = parse_dollar_term(input)?;
                if dollar_term.value.as_str() != "right" {
                    return Err(input.unexpected_token("'right' keyword expected"));
                }
                let right_kwd = dollar_term.span.clone();

                let dot = input.next()?;
                if dot.value != Token::Dot {
                    return Err(input.unexpected_token("Dot expected"));
                };

                let right_name = parse_term(input)?;
                JoinAttribute::NonMatching { left_kwd, left_name, right_kwd, right_name }
            },
            _ => return Err(input.unexpected_token("Term expected")),
        };

        attributes.push(attribute);

        if input.next_if(Token::Comma).is_none() {
            break;
        }
    };

    Ok(TabularOperator::Join { params, right_table, attributes })
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
        _ => return Err(input.unexpected_token("Expected 'by' keyword")),
    };
    loop {
        let column = match parse_term(input) {
            Ok(column) => column,
            Err(_) => return Err(input.unexpected_token("Expected column name")),
        };

        let checkpoint = input.checkpoint();

        let order_term = parse_term(input)?;
        let order = match order_term.value.as_str() {
            "asc" => Some(M::new(SortOrder::Ascending, order_term.span.clone())),
            "desc" => Some(M::new(SortOrder::Descending, order_term.span.clone())),
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
                    "first" => Some((
                        nulls_kwd.span.clone(),
                        M::new(NullsPosition::First, nulls_pos.span.clone()),
                    )),
                    "last" => Some((
                        nulls_kwd.span.clone(),
                        M::new(NullsPosition::Last, nulls_pos.span.clone()),
                    )),
                    _ => return Err(input.unexpected_token("Expected nulls position")),
                }
            }
            _ => {
                input.restore(checkpoint);
                None
            }
        };

        let sorting = Sorting {
            column,
            order,
            nulls,
        };

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
        _ => return Err(input.unexpected_token("Expected 'by' keyword")),
    };

    let mut grouping_columns: Vec<ColumnDefinition> = Vec::new();

    grouping_columns.push(parse_column_definition(input)?);
    while input.next_if(Token::Comma).is_some() {
        grouping_columns.push(parse_column_definition(input)?);
    }

    Ok(TabularOperator::Summarize {
        result_columns,
        by_kwd,
        grouping_columns,
    })
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
    use crate::parser::tests::make_input;

    #[test]
    fn parse_summarize_supports_groupings() {
        let source = "NumTransactions=2, Total=foobar by Fruit, StartOfMonth";
        let result = parse_summarize(&mut make_input(source));
        match result {
            Ok(_) => {}
            Err(error) => {
                println!("{:?}", Report::new(error));
                panic!();
            }
        }
    }

    #[test]
    fn parse_join_supports_attributes() {
        let source = "(Table2) on CommonColumn, $left.Col1 == $right.Col2";
        let result = parse_join(&mut make_input(source));
        match result {
            Ok(_) => {}
            Err(error) => {
                println!("{:?}", Report::new(error));
                panic!();
            }
        }
    }

    #[test]
    fn parse_join_supports_kind() {
        let source = "rightouter (Table) on $left.Col1 == $right.Col2";
        let result = parse_join(&mut make_input(source));
        match result {
            Ok(_) => {}
            Err(error) => {
                println!("{:?}", Report::new(error));
                panic!();
            }
        }
    }
}
