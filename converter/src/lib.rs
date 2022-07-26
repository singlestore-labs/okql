use kql_parser::{ast as kast, parse, Error as KqlError};
use miette::Report;
use sql_emitter::{ast as sast, emit};
use std::fmt::Write;

/// AST to AST conversion code
mod direct;
mod merge;

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
    let mut head = sast::SelectStatement::simple(kquery.table.value);

    for (_, operator) in kquery.operators {
        match merge::attempt_merge(&mut head, &operator)? {
            merge::MergeOutcome::Combined => continue,
            merge::MergeOutcome::CannotCombine => {
                head = sast::SelectStatement::simple_wrapping(head);
                if merge::attempt_merge(&mut head, &operator)? == merge::MergeOutcome::CannotCombine
                {
                    panic!("Could not combine with new query")
                }
            }
        }
    }

    Ok(head)
}
