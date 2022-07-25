use std::sync::Arc;

use miette::{NamedSource, Report};

/// Miette Span Utilities
mod spans;
/// KQL Tokenizer/Lexer
mod lexer;
/// KQL Abstract Syntax Tree
mod ast;
/// KQL Parser
mod parser;

pub fn parse<SN: ToString, SC: ToString>(
    source_name: SN,
    source_code: SC
) -> Option<ast::query::Query> {
    let source_name = source_name.to_string();
    let source_code = source_code.to_string();
    let src = Arc::new(NamedSource::new(source_name, source_code.clone()));

    let tokens = match lexer::tokenize(src.clone(), source_code) {
        Ok(token_data) => token_data,
        Err(errors) => {
            for error in errors {
                println!("{:?}", Report::new(error));
            }
            return None;
        }
    };

    unimplemented!()
}