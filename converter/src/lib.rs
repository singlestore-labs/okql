#![allow(dead_code)]
#![allow(unused_variables)]

use kql_parser::{ast as kast, parse, Error as KqlError, spans::Span};
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
        Err(_) => todo!(),
    };

    emit(&sql_ast)
}

pub fn convert(src: Arc<NamedSource>, query: kast::Query) -> Result<sast::SelectStatement, ConverterError> {
    let mut merger = merger::Merger::new(src);
    let mut head = sast::SelectStatement::simple(query.table.value);

    for (_, operator) in query.operators {
        (merger, head) = merger.merge_operator(head, operator)?;
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
}