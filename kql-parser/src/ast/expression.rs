use crate::spans::{M, MBox, Span};

/// Represents scalar, aggregate, and group expressions
#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Identifier {
        name: M<String>
    },
    FuncCall {
        name: M<String>,
        open_paren_sym: Span,
        args: Vec<MBox<Expression>>,
        close_paren_sym: Span
    },
    BinaryOp {
        left: MBox<Expression>,
        op: M<BinaryOp>,
        right: MBox<Expression>
    },
    Literal {
        value: Literal
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinaryOp {
    /// Add "+"
    Add,
    /// Subtract "-"
    Sub,
    /// Multipley "*"
    Mul,
    /// Divide "/"
    Div,
    /// Modulo "%"
    Mod,

    /// Logical And "and"
    LogicalAnd,
    /// Logical Or "or"
    LogicalOr,

    /// Less Than "<"
    LT,
    /// Greater Than ">"
    GT,
    /// Equal "=="
    EQ,
    /// Not Equal "!="
    NEQ,
    /// Less Than or Equal "<="
    LTE,
    /// Greater Than or Equal ">="
    GTE,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    /// A literal boolean value
    /// "null" value represented by `None`
    Bool(Option<bool>),
    /// A literal int value
    /// "null" value represented by `None`
    Int(Option<i32>),
    /// A literal long value
    /// "null" value represented by `None`
    Long(Option<i64>),
    /// A literal real number value
    /// "null" value represented by `None`
    Real(Option<f64>),
    /// A literal string value
    /// There is no "null" string value
    String(String)
}