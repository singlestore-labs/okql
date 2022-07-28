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

pub fn parse(src: Arc<NamedSource>, kql_code: String) -> Result<ast::query::Query, Error> {
    let tokens = match lexer::tokenize(src.clone(), kql_code) {
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
