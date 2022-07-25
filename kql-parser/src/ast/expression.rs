use super::{M, MBox, Span};

/// Represents scalar, aggregate, and group expressions
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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