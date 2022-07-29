#![allow(dead_code)]
#![allow(unused_variables)]

use kql_parser::{ast as kast, parse, spans::Span, Error as KqlError};
use miette::Report;
use sql_emitter::{ast as sast, emit};
use std::{fmt::Write, sync::Arc};

use miette::{Diagnostic, NamedSource};
use thiserror::Error;

/// AST to AST conversion code
mod merger;

pub fn kql_to_sql(source_name: String, kql: String) -> Result<String, String> {
    let src = Arc::new(NamedSource::new(source_name, kql.clone()));

    let kql_ast = match parse(src.clone(), kql) {
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

    let sql_ast = match convert(src, kql_ast) {
        Ok(result) => result,
        Err(error) => return Err(format!("{:?}", Report::new(error))),
    };

    emit(&sql_ast)
}

pub fn convert(
    src: Arc<NamedSource>,
    query: kast::Query,
) -> Result<sast::SelectStatement, ConverterError> {
    let mut merger = merger::Merger::new(src);
    let mut head = sast::SelectStatement::simple(query.table.value);

    for (name, operator) in query.operators {
        (merger, head) = merger.merge_operator(head, name, operator)?;
    }

    Ok(head)
}

#[derive(Error, Debug, Diagnostic)]
pub enum ConverterError {
    #[diagnostic()]
    #[error("Expression cannot be interpreted as a condition")]
    ExpressionNotCondition {
        #[source_code]
        src: Arc<NamedSource>,
        #[label("Non-condition expression")]
        span: Span,
    },
    #[diagnostic()]
    #[error("{feature} not yet implemented")]
    NotImplemented {
        #[source_code]
        src: Arc<NamedSource>,
        #[label("Here")]
        span: Span,
        feature: String,
    },
}
