pub mod query;
pub mod expression;

use std::sync::Arc;

use crate::spans::{M, MBox, Span};
use crate::lexer::Token;
use crate::ast::query::Query;

use miette::{Diagnostic, NamedSource};
use thiserror::Error;

use self::query::parse_query;

#[derive(Error, Debug, Diagnostic)]
pub enum ParserError{
    #[diagnostic()]
    #[error("End of input reached")]
    EndOfInput,
    #[diagnostic()]
    #[error("Failed to parse")]
    Simple {
        #[source_code]
        src: Arc<NamedSource>,
        #[label("Unable to parse this code")]
        span: Span,
    },
    #[diagnostic()]
    #[error("Failed to parse: {message}")]
    General {
        #[source_code]
        src: Arc<NamedSource>,
        #[label("Unable to parse this code")]
        span: Span,
        message: String
    },
    #[diagnostic()]
    #[error("Unexpected token {token:?} with description '{description}'")]
    UnexpectedToken {
        #[source_code]
        src: Arc<NamedSource>,
        #[label("Here")]
        span: Span,
        description: String,
        token: Token
    },
    #[diagnostic()]
    #[error("Feature {feature} not supported yet")]
    NotYetSupported {
        #[source_code]
        src: Arc<NamedSource>,
        #[label("Unable to parse this code")]
        span: Span,
        feature: String
    }
}


pub fn parse(src: Arc<NamedSource>, tokens: Vec<M<Token>>) -> Result<Query, ParserError> {
    let mut parse_input = ParseInput::new(src, tokens);
    parse_query(&mut parse_input)
}


#[derive(Debug, Clone)]
pub struct ParseInput {
    src: Arc<NamedSource>, 
    tokens: Vec<M<Token>>,
    index: usize
}

#[derive(Debug, Clone, Copy)]
pub struct Checkpoint {
    index: usize
}

impl ParseInput {
    pub fn new(src: Arc<NamedSource>, tokens: Vec<M<Token>>) -> Self {
        ParseInput {
            src,
            tokens,
            index: 0
        }
    }

    pub fn general_error(&self, message: &str) -> ParserError {
        let data = &self.tokens[self.index-1];
        ParserError::General {
            src: self.src.clone(),
            span: data.span.clone(),
            message: message.to_string(),
        }
    }

    pub fn unsupported_error(&self, feature: &str) -> ParserError {
        let data = &self.tokens[self.index-1];
        ParserError::NotYetSupported {
            src: self.src.clone(),
            span: data.span.clone(),
            feature: feature.to_string(),
        }
    }

    pub fn unexpected_token(&self, description: &str) -> ParserError {
        let data = &self.tokens[self.index-1];
        ParserError::UnexpectedToken {
            src: self.src.clone(),
            span: data.span.clone(),
            description: description.to_string(),
            token: data.value.clone()
        }
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint { index: self.index }
    }

    pub fn restore(&mut self, checkpoint: Checkpoint) {
        self.index = checkpoint.index
    }

    pub fn get_source(&self) -> Arc<NamedSource> {
        self.src.clone()
    }

    pub fn done(&self) -> bool {
        self.index >= self.tokens.len()
    }

    pub fn peek(&mut self) -> Result<&M<Token>, ParserError> {
        self.tokens.get(self.index).ok_or(ParserError::EndOfInput)
    }

    pub fn next(&mut self) -> Result<&M<Token>, ParserError> {
        let result = self.tokens.get(self.index);
        self.index += 1;
        result.ok_or(ParserError::EndOfInput)
    }

    pub fn assert_next(&mut self, token: Token, description: &str) -> Result<Span, ParserError> {
        let next = self.next()?;
        if next.value == token {
            Ok(next.span.clone())
        } else {
            Err(self.unexpected_token(description))
        }
    }

    pub fn next_if(&mut self, token: Token) -> Option<Span> {
        {
            let next = self.peek().ok()?;
            if next.value != token {
                return None;
            }
        }
        Some(self.next().ok()?.span.clone())
    }

    pub fn has(&self, num: usize) -> bool {
        self.index + num <= self.tokens.len()
    }

    pub fn slice_next(&mut self, num: usize) -> Result<&[M<Token>], ParserError> {
        if self.has(num) {
            let result = &self.tokens[self.index..self.index+num];
            self.index += num;
            Ok(result)
        } else {
            Err(ParserError::EndOfInput)
        }
    }
}

fn parse_term(input: &mut ParseInput) -> Result<M<String>, ParserError> {
    let token = input.next()?;
    match token.value.clone() {
        Token::Term(s) => Ok(M::new(s, token.span.clone())),
        _ => Err(input.unexpected_token("Term expected"))
    }
}

#[cfg(test)]
mod tests {
    
    use std::sync::Arc;
    use miette::NamedSource;

    use crate::{
        lexer::tokenize,
        spans::Span,
        parser::ParseInput
    };

    pub fn make_input(source: &str) -> ParseInput {
        let src = Arc::new(NamedSource::new("test", source.to_string()));
        let tokens = tokenize(src.clone(), source.to_string()).unwrap();
        ParseInput::new(src, tokens)
    }
}