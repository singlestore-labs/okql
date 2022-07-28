use std::sync::Arc;

use lexer::LexerError;
use miette::NamedSource;
use parser::ParserError;

/// KQL Abstract Syntax Tree
pub mod ast;
/// KQL Tokenizer/Lexer
pub mod lexer;
/// KQL Parser
pub mod parser;
/// Miette Span Utilities
pub mod spans;

pub enum Error {
    Lexer { errors: Vec<LexerError> },
    Parser { error: ParserError },
}

pub fn parse<SN: ToString, SC: ToString>(
    source_name: SN,
    source_code: SC,
) -> Result<ast::query::Query, Error> {
    let source_name = source_name.to_string();
    let source_code = source_code.to_string();
    let src = Arc::new(NamedSource::new(source_name, source_code.clone()));

    let tokens = match lexer::tokenize(src.clone(), source_code) {
        Ok(token_data) => token_data,
        Err(errors) => {
            return Err(Error::Lexer { errors });
        }
    };

    let ast = match parser::parse(src.clone(), tokens) {
        Ok(query) => query,
        Err(error) => {
            return Err(Error::Parser { error });
        }
    };

    Ok(ast)
}
