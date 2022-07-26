use std::fmt::Write;
use kql_parser::{ast::query::Query as KQLQuery, parse, Error as KqlError};
use sql_emitter::{ast::SelectStatement as SQLQuery, emit};
use miette::Report;

pub fn kql_to_sql(source_name: String, kql: String) -> Result<String, String> {
    let kql_ast = match parse(source_name, kql) {
        Ok(ast) => ast,
        Err(KqlError::Lexer { errors }) => {
            let mut output = String::new();
            for error in errors {
                write!(output, "{:?}\n\n", Report::new(error)).unwrap();
            }
            return Err(output);
        },
        Err(KqlError::Parser { error }) => {
            return Err(format!("{:?}", Report::new(error)));
        }
    };

    let sql_ast = convert(kql_ast);

    emit(&sql_ast)
}

pub fn convert(kql_ast: KQLQuery) -> SQLQuery {
    todo!()
}

