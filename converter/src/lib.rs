use kql_parser::{ast as kast, parse, Error as KqlError};
use miette::Report;
use sql_emitter::{ast as sast, emit};
use std::fmt::Write;

/// AST to AST conversion code
mod ktos;

pub fn kql_to_sql(source_name: String, kql: String) -> Result<String, String> {
    let kql_ast = match parse(source_name, kql) {
        Ok(ast) => ast,
        Err(KqlError::Lexer { errors }) => {
            let mut output = String::new();
            for error in errors {
                write!(output, "{:?}\n\n", Report::new(error)).unwrap();
            }
            return Err(output);
        }
        Err(KqlError::Parser { error }) => {
            return Err(format!("{:?}", Report::new(error)));
        }
    };

    let sql_ast = match convert(kql_ast) {
        Ok(result) => result,
        Err(_) => todo!(),
    };

    emit(&sql_ast)
}

pub enum ConverterError {}

pub fn convert(kquery: kast::Query) -> Result<sast::SelectStatement, ConverterError> {
    let mut state = MergeState::default();
    let mut head = sast::SelectStatement::simple(kquery.table.value);

    for (_, operator) in kquery.operators {
        (state, head) = apply_operator(state, head, operator)?;
    }

    Ok(head)
}

#[derive(Default, Debug)]
struct MergeState {
    columns: ColumnsState,
    limit: Option<i64>,
}

#[derive(PartialEq, Default, Debug)]
enum ColumnsState {
    #[default]
    Unmodified,
    Limited {
        columns: Vec<String>,
    },
    Modified {
        retained: Vec<String>,
        modified: Vec<String>,
    },
}

/// Takes a SELECT statement and either modifies it to include the provided operator
/// or creates a new SELECT statement wrapping the old one which does.
fn apply_operator(
    mut state: MergeState,
    mut head: sast::SelectStatement,
    operator: kast::TabularOperator,
) -> Result<(MergeState, sast::SelectStatement), ConverterError> {
    match operator {
        kast::TabularOperator::Count => todo!(),
        kast::TabularOperator::Distinct { columns } => todo!(),
        kast::TabularOperator::Extend { columns } => todo!(),
        kast::TabularOperator::Join {
            params,
            right_table,
            attributes,
        } => todo!(),
        kast::TabularOperator::Limit { limit } => todo!(),
        kast::TabularOperator::Project { columns } => {
            // Compute column state
            let (retained, modified): (Vec<_>, Vec<_>) =
                columns.iter().partition(|col| col.expr.is_none());
            let retained: Vec<String> = retained
                .into_iter()
                .map(|col| col.column.value.clone())
                .collect();
            let modified: Vec<String> = modified
                .into_iter()
                .map(|col| col.column.value.clone())
                .collect();
            let column_state = ColumnsState::Modified { retained, modified };
            // Compute SQL columns
            let new_columns: Vec<sast::SelectColumn> = columns
                .into_iter()
                .map(|col| ktos::column_definition(col))
                .collect::<Result<Vec<sast::SelectColumn>, ConverterError>>()?;

            if state.columns == ColumnsState::Unmodified {
                // Update head
                head.select.wildcard = false;
                head.select.columns.extend(new_columns);
                // Update state
                state.columns = column_state;
                //Result
                Ok((state, head))
            } else {
                let new_state = MergeState {
                    columns: column_state,
                    limit: None,
                };
                let new_head = sast::SelectStatement {
                    modifier: None,
                    select: sast::SelectList {
                        wildcard: false,
                        columns: new_columns,
                    },
                    from: sast::TableReference::InnerStatement {
                        value: Box::new(head),
                    },
                    where_: None,
                    order_by: None,
                };
                Ok((new_state, new_head))
            }
        }
        kast::TabularOperator::Sort {
            by_kwd,
            expr,
            order,
            nulls,
        } => todo!(),
        kast::TabularOperator::Summarize {
            result_columns,
            by_kwd,
            grouping_columns,
        } => todo!(),
        kast::TabularOperator::Top {
            limit,
            by_kwd,
            expr,
            order,
            nulls,
        } => todo!(),
        kast::TabularOperator::Where { expr } => todo!(),
    }
}
