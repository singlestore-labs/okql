use std::sync::Arc;

use logos::Logos;

use miette::{Diagnostic, SourceSpan, NamedSource};
use thiserror::Error;


#[derive(Debug, PartialEq, Clone)]
pub struct TokenData {
    pub token: Token,
    pub span: SourceSpan
}


#[derive(Error, Debug, Diagnostic)]
#[error("The input did not match a token rule")]
#[diagnostic()]
pub struct LexerError {
    #[source_code]
    src: Arc<NamedSource>,
    #[label("This text was not recognized")]
    span: SourceSpan,
}

/// Returns a token vector or collection of errors
pub fn tokenize(src: Arc<NamedSource>, contents: String) -> Result<Vec<TokenData>, Vec<LexerError>> {
    let tokens: Vec<TokenData> = Token::lexer(&contents)
        .spanned()
        .map(|(token, span)| 
            TokenData {
                token,
                span: SourceSpan::from(span)
            }
        )
        .collect();

    let errors: Vec<LexerError> = tokens.iter()
        .filter(|token_data| token_data.token == Token::Error)
        .map(|token_data|
            LexerError {
                src: src.clone(),
                span: token_data.span.clone()
            }
        )
        .collect();

    if errors.is_empty() {
        Ok(tokens)
    } else {
        Err(errors)
    }
}

/// The Token type for the language.
#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    #[error]
    #[regex(r"[ \n\t\f]+", logos::skip)]
    #[regex(r"//[^\n]*", logos::skip)]
    Error,

    /// Term (e.g. summarize, OrderNumber, count)
    #[regex(r"[_a-zA-Z][_a-zA-Z0-9]*", |lex| String::from(lex.slice()))]
    Term(String),

    /// Terms that begin with "!" (e.g. !between)
    #[regex(r"![_a-zA-Z][_a-zA-Z0-9]*", |lex| String::from(&lex.slice()[1..]))]
    BangTerm(String),

    /// `bool` literal
    #[token("true", |_| true)]
    #[token("false", |_| false)]
    BoolLiteral(bool),

    #[token("bool(null)")]
    BoolNullLiteral,

    // TODO datetime: https://docs.microsoft.com/en-us/azure/data-explorer/kusto/query/scalar-data-types/datetime

    // TODO decimal: https://docs.microsoft.com/en-us/azure/data-explorer/kusto/query/scalar-data-types/decimal

    // TODO dynamic: https://docs.microsoft.com/en-us/azure/data-explorer/kusto/query/scalar-data-types/dynamic

    // Intentionally Omitting guid type

    /// `int` literal
    #[regex("int(-?[0-9]+)", |lex| parse_int_dec_literal(lex.slice(), 4, 1))]
    #[regex(r"int\(0x[0-9a-fA-F][0-9a-fA-F]*\)", |lex| parse_int_hex_literal(lex.slice(), 4, 1))]
    IntLiteral(i32),

    #[token("int(null)")]
    IntNullLiteral,

    /// `long` literal
    #[regex("-?[0-9]+", |lex| parse_long_dec_literal(lex.slice(), 0, 0))]
    #[regex(r"0x[0-9a-fA-F][0-9a-fA-F]*", |lex| parse_long_hex_literal(lex.slice(), 0, 0))]
    #[regex("long(-?[0-9]+)", |lex| parse_long_dec_literal(lex.slice(), 5, 1))]
    #[regex(r"long\(0x[0-9a-fA-F][0-9a-fA-F]*\)", |lex| parse_long_hex_literal(lex.slice(), 5, 1))]
    LongLiteral(i64),

    #[token("long(null)")]
    LongNullLiteral,

    /// `real` literal
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse())] // TODO: exp notation
    #[token("real(nan)", |_| f64::NAN)]
    #[token("real(+inf)", |_| f64::INFINITY)]
    #[token("real(-inf)", |_| f64::NEG_INFINITY)]
    RealLiteral(f64),

    #[token("real(null")]
    RealNullLiteral,

    /// `string` literal
    #[token("\"", |lex| parse_string_literal('"', lex))]
    #[token("\'", |lex| parse_string_literal('\'', lex))]
    #[token("@\"", |lex| parse_verbatim_string_literal('"', lex))]
    #[token("@\'", |lex| parse_verbatim_string_literal('\'', lex))]
    #[token("```", |lex| parse_multiline_string_literal(lex))]
    StringLiteral(String),

    // TODO timespan: https://docs.microsoft.com/en-us/azure/data-explorer/kusto/query/scalar-data-types/timespan

    // Symbols -----------------------------------------

    /// Pipe Symbol "|"
    #[token("|")]
    Pipe,

    /// Left Parenthesis Symbol "("
    #[token("(")]
    LParen,

    /// Right Parenthesis Symbol ")"
    #[token(")")]
    RParen,

    /// Left Brace Symbol "{"
    #[token("{")]
    LBrace,

    /// Right Brace Symbol "}"
    #[token("}")]
    RBrace,

    /// Left Bracket Symbol "["
    #[token("[")]
    LBracket,

    /// Right Bracket Symbol "]"
    #[token("]")]
    RBracket,

    /// The Comma Delimiter ","
    #[token(",")]
    Comma,

    /// The Period or Dot Operator "."
    #[token(".")]
    Dot,

    /// Assignment Operator "="
    #[token("=")]
    Assign,

    /// Addition Operator "+"
    #[token("+")]
    Add,

    /// Subtraction Operator "-"
    #[token("-")]
    Sub,

    /// Star Operator "*" (used for multiply and wildcard)
    #[token("*")]
    Star,

    /// Division Operator "/"
    #[token("/")]
    Div,

    /// Modulo Operator "%"
    #[token("%")]
    Mod,

    /// Logical And Operator
    #[token("and")]
    LogicalAnd,

    /// Logical Or Operator
    #[token("or")]
    LogicalOr,

    /// Less-than Operator "<"
    #[token("<")]
    LT,

    /// Less-than or Equal Operator "<="
    #[token("<=")]
    LTE,

    /// Greater-than Operator ">"
    #[token(">")]
    GT,

    /// Greater-than or Equal Operator ">="
    #[token(">=")]
    GTE,

    /// Equals Operator "=="
    #[token("==")]
    EQ,

    // Not Equals Operator "!="
    #[token("!=")]
    NEQ
}

/// Parses a string according to the JSON string format in ECMA-404.
fn parse_string_literal<'src>(init: char, lex: &mut logos::Lexer<'src, Token>) -> Option<String> {
    let mut c_iter = lex.remainder().chars();
    let mut buf = String::new();

    while let Some(c) = c_iter.next() {
        // End the parse when you encounter another quote
        if c == init {
            lex.bump(1);
            return Some(buf);
        }

        if c == '\n' || c == '\r' {
            return None;
        }

        // If slash, then parse an escaped character
        if c == '\\' {
            lex.bump(1);
            if let Some((c_esc, c_len)) = parse_escaped_char(&mut c_iter) {
                lex.bump(c_len);
                buf.push(c_esc);
            }
        } else {
            lex.bump(c.len_utf8());
            buf.push(c);
        }
    }

    None
}

/// Parses an escaped character.
/// Takes in an iterator which starts after the beginning slash.
/// If successful, returns the produced char and the length of input consumed.
fn parse_escaped_char<'src>(lex: &mut std::str::Chars<'src>) -> Option<(char, usize)> {
    let res = match lex.next()? {
        '\"' => ('\"', 1),
        '\'' => ('\'', 1),
        '\\' => ('\\', 1),
        'n'  => ('\n', 1),
        'r'  => ('\r', 1),
        't'  => ('\t', 1),
        _ => return None,
    };

    Some(res)
}

fn parse_verbatim_string_literal<'src>(init: char, lex: &mut logos::Lexer<'src, Token>) -> Option<String> {
    let mut c_iter = lex.remainder().chars();
    let mut buf = String::new();

    while let Some(c) = c_iter.next() {
        // End the parse when you encounter another quote
        if c == init {
            lex.bump(1);
            return Some(buf);
        }

        if c == '\n' || c == '\r' {
            return None;
        }

        lex.bump(c.len_utf8());
        buf.push(c);
    }

    // Did not find end sequence
    None
}

fn parse_multiline_string_literal<'src>(lex: &mut logos::Lexer<'src, Token>) -> Option<String> {
    let mut c_iter = lex.remainder().chars();
    let mut buf = String::new();

    let mut backtick_count = 0;
    while let Some(c) = c_iter.next() {
        // End the parse when you encounter another quote
        if c == '`' {
            backtick_count += 1;
            lex.bump(1);

            if backtick_count == 3 {
                return Some(buf);
            } else {
                continue;
            }
        } else {
            for _ in 0..backtick_count {
                buf.push('`');
            }
            backtick_count = 0;
        }

        if c == '\n' || c == '\r' {
            return None;
        }

        lex.bump(c.len_utf8());
        buf.push(c);
    }

    // Did not find end sequence
    None
}

fn parse_int_dec_literal(s: &str, trim_front: usize, trim_back: usize) -> Option<i32> {
    let s = &s[trim_front.. s.len()-trim_back];

    if s.starts_with("-") {
        (&s[1..]).parse::<i32>().ok().map(|n| -n)
    } else {
        s.parse().ok()
    }
}

fn parse_long_dec_literal(s: &str, trim_front: usize, trim_back: usize) -> Option<i64> {
    let s = &s[trim_front.. s.len()-trim_back];

    if s.starts_with("-") {
        (&s[1..]).parse::<i64>().ok().map(|n| -n)
    } else {
        s.parse().ok()
    }
}

fn parse_int_hex_literal(s: &str, trim_front: usize, trim_back: usize) -> Option<i32> {
    let s = &s[trim_front.. s.len()-trim_back];

    i32::from_str_radix(&s[2..], 16).ok()
}

fn parse_long_hex_literal(s: &str, trim_front: usize, trim_back: usize) -> Option<i64> {
    let s = &s[trim_front.. s.len()-trim_back];

    i64::from_str_radix(&s[2..], 16).ok()
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use super::*;

    #[test]
    fn tokenize_fn_declaration() {
        let contents: String = "StormEvents | take 5 | extend Duration = EndTime - StartTime".into();
        let src = Arc::new(NamedSource::new(String::from("test"), contents.clone()));
        let output = vec![
            ( SourceSpan::from(0 ..11), Token::Term(String::from("StormEvents")) ),
            ( SourceSpan::from(12..13), Token::Pipe ),
            ( SourceSpan::from(14..18), Token::Term(String::from("take")) ),
            ( SourceSpan::from(19..20), Token::LongLiteral(5) ),
            ( SourceSpan::from(21..22), Token::Pipe ),
            ( SourceSpan::from(23..29), Token::Term(String::from("extend")) ),
            ( SourceSpan::from(30..38), Token::Term(String::from("Duration")) ),
            ( SourceSpan::from(39..40), Token::Assign ),
            ( SourceSpan::from(41..48), Token::Term(String::from("EndTime")) ),
            ( SourceSpan::from(49..50), Token::Sub ),
            ( SourceSpan::from(51..60), Token::Term(String::from("StartTime")) ),
        ].into_iter().map(to_token_data).collect::<Vec<TokenData>>();

        match tokenize(src, contents) {
            Ok(tokens) => assert_eq!(output, tokens),
            Err(_) => panic!("Should not have failed")
        }
    }

    // #[test]
    // fn tokenize_let() {
    //     let contents: String = r#"let a = "asdf\"";"#.into();
    //     let src = Arc::new(NamedSource::new(String::from("test"), contents.clone()));
    //     let ident_a = Token::Identifier(String::from("a"));
    //     let string_asdf = Token::StringLiteral(String::from(r#"asdf""#));
    //     let output = vec![
    //         ( Token::Let,       SourceSpan::from(0..3) ),
    //         ( ident_a,          SourceSpan::from(4..5) ),
    //         ( Token::Assign,    SourceSpan::from(6..7) ),
    //         ( string_asdf,      SourceSpan::from(8..16) ),
    //         ( Token::Semicolon, SourceSpan::from(16..17) )
    //     ].into_iter().map(to_token_data).collect::<Vec<TokenData>>();

    //     match tokenize(src, contents) {
    //         Ok(tokens) => assert_eq!(output, tokens),
    //         Err(_) => panic!("Should not have failed")
    //     }
    // }

    fn to_token_data(d: (SourceSpan, Token)) -> TokenData {
        TokenData {
            token: d.1,
            span: d.0
        }
    }
}