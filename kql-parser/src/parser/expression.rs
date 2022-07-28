use crate::{
    ast::expression::{BinaryOp, Expression, Literal},
    lexer::Token,
    spans::{MBox, M},
};

use super::{parse_term, ParseInput, ParserError};

#[allow(dead_code)]
pub fn parse_expression(input: &mut ParseInput) -> Result<MBox<Expression>, ParserError> {
    pratt_parse(input, 0)
}

/// Pratt parsing of expressions based on
/// https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
fn pratt_parse(input: &mut ParseInput, min_bp: u8) -> Result<MBox<Expression>, ParserError> {
    let mut lhs = parse_leaf(input)?;
    loop {
        let checkpoint = input.checkpoint();
        if let Some(bin_op) = try_parse_bin_op(input) {
            let (l_bp, r_bp) = infix_binding_power(bin_op.value);

            if l_bp < min_bp {
                input.restore(checkpoint);
                break;
            }

            let rhs = pratt_parse(input, r_bp)?;
            let left = lhs.span.clone();
            let right = rhs.span.clone();
            let new_root = Expression::BinaryOp {
                left: lhs,
                op: bin_op,
                right: rhs,
            };
            lhs = MBox::new_range(new_root, left, right)
        } else {
            break;
        }
    }
    Ok(lhs)
}

fn parse_leaf(input: &mut ParseInput) -> Result<MBox<Expression>, ParserError> {
    let checkpoint = input.checkpoint();
    if let Ok(value) = parse_parenthetical(input) {
        return Ok(value);
    }
    input.restore(checkpoint);
    if let Ok(value) = parse_literal(input) {
        let span = value.span.clone();
        return Ok(MBox::new(Expression::Literal { value: value.value }, span));
    }
    input.restore(checkpoint);
    if let Ok(term) = parse_term(input) {
        let span = term.span.clone();
        if let Some(open_paren_sym) = input.next_if(Token::LParen) {
            let mut args = Vec::new();
            args.push(parse_expression(input)?);
            while input.next_if(Token::Comma).is_some() {
                args.push(parse_expression(input)?);
            }
            let close_paren_sym = input.assert_next(Token::RParen, "No closing parenthesis for function call")?;
            let end_span = close_paren_sym.clone();
            return Ok(MBox::new_range(Expression::FuncCall { name: term, open_paren_sym, args, close_paren_sym }, span, end_span))
        } else {
            return Ok(MBox::new(Expression::Identifier { name: term }, span));
        }
    }
    // advance so that error is generated on the correct token
    let _ = input.next();
    Err(input.unexpected_token("Parse Leaf"))
}

fn parse_parenthetical(input: &mut ParseInput) -> Result<MBox<Expression>, ParserError> {
    let _left = input.assert_next(Token::LParen, "Left parenthesis '('")?;
    let inner = parse_expression(input)?;
    let _right = input.assert_next(Token::RParen, "Right parenthesis ')'")?;
    Ok(inner)
}

fn parse_literal(input: &mut ParseInput) -> Result<M<Literal>, ParserError> {
    let next = input.next()?;
    let span = next.span.clone();
    let value = match next.value.clone() {
        // booleans
        Token::BoolLiteral(value) => Literal::Bool(Some(value)),
        Token::BoolNullLiteral => Literal::Bool(None),
        // ints
        Token::IntLiteral(value) => Literal::Int(Some(value)),
        Token::IntNullLiteral => Literal::Int(None),
        // longs
        Token::LongLiteral(value) => Literal::Long(Some(value)),
        Token::LongNullLiteral => Literal::Long(None),
        // reals
        Token::RealLiteral(value) => Literal::Real(Some(value)),
        Token::RealNullLiteral => Literal::Real(None),
        // strings
        Token::StringLiteral(value) => Literal::String(value),
        // errors
        _ => return Err(input.unexpected_token("Parse Literal")),
    };
    Ok(M::new(value, span))
}

fn try_parse_bin_op(input: &mut ParseInput) -> Option<M<BinaryOp>> {
    let next = input.peek().ok()?;
    let span = next.span.clone();
    let op = match &next.value {
        Token::LogicalOr => BinaryOp::LogicalOr,
        Token::LogicalAnd => BinaryOp::LogicalAnd,

        Token::EQ => BinaryOp::EQ,
        Token::NEQ => BinaryOp::NEQ,

        Token::LT => BinaryOp::LT,
        Token::LTE => BinaryOp::LTE,
        Token::GT => BinaryOp::GT,
        Token::GTE => BinaryOp::GTE,

        Token::Add => BinaryOp::Add,
        Token::Sub => BinaryOp::Sub,

        Token::Star => BinaryOp::Mul,
        Token::Div => BinaryOp::Div,
        Token::Mod => BinaryOp::Mod,

        _ => return None,
    };
    let _ = input.next();
    Some(M::new(op, span))
}

fn infix_binding_power(op: BinaryOp) -> (u8, u8) {
    match op {
        BinaryOp::LogicalOr => (10, 1),
        BinaryOp::LogicalAnd => (20, 21),

        BinaryOp::EQ | BinaryOp::NEQ => (30, 31),

        BinaryOp::LT | BinaryOp::LTE | BinaryOp::GT | BinaryOp::GTE => (40, 41),

        BinaryOp::Add | BinaryOp::Sub => (50, 51),

        BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => (60, 61),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parser::tests::make_input, spans::Span};
    use pretty_assertions::assert_eq;

    #[test]
    fn parsing_supports_ints() {
        let cases = [
            ("int(0)", Some(0), Span::from((0, 6))),
            ("int(234)", Some(234), Span::from((0, 8))),
            ("int(null)", None, Span::from((0, 9))),
        ];
        for (source, value, span) in cases {
            let parsed_literal = M::new(Literal::Int(value), span.clone());
            let parsed_expression = MBox::new(
                Expression::Literal {
                    value: Literal::Int(value),
                },
                span.clone(),
            );
            assert_eq!(
                parse_literal(&mut make_input(source)).unwrap(),
                parsed_literal
            );
            assert_eq!(
                parse_leaf(&mut make_input(source)).unwrap(),
                parsed_expression
            );
            assert_eq!(
                parse_expression(&mut make_input(source)).unwrap(),
                parsed_expression
            );
        }
    }

    #[test]
    fn parsing_supports_idents() {
        let cases = [
            ("foo", Span::from((0, 3))),
            ("foobar", Span::from((0, 6))),
            ("asdf", Span::from((0, 4))),
            ("asdf2", Span::from((0, 5))),
        ];
        for (source, span) in cases {
            let parsed_expression = MBox::new(
                Expression::Identifier {
                    name: M::new(source.to_string(), span.clone()),
                },
                span.clone(),
            );
            assert_eq!(
                parse_leaf(&mut make_input(source)).unwrap(),
                parsed_expression
            );
            assert_eq!(
                parse_expression(&mut make_input(source)).unwrap(),
                parsed_expression
            );
        }
    }

    #[test]
    fn parsing_supports_parenthesized_idents() {
        // parenthesized, raw, raw-span
        let cases = [
            ("(foo)", "foo", Span::from((1, 3))),
            ("(foobar)", "foobar", Span::from((1, 6))),
            ("(asdf)", "asdf", Span::from((1, 4))),
            ("(asdf2)", "asdf2", Span::from((1, 5))),
        ];
        for (source, ident, span) in cases {
            let parsed_expression = MBox::new(
                Expression::Identifier {
                    name: M::new(ident.to_string(), span.clone()),
                },
                span.clone(),
            );
            assert_eq!(
                parse_parenthetical(&mut make_input(source)).unwrap(),
                parsed_expression
            );
            assert_eq!(
                parse_leaf(&mut make_input(source)).unwrap(),
                parsed_expression
            );
            assert_eq!(
                parse_expression(&mut make_input(source)).unwrap(),
                parsed_expression
            );
        }
    }

    macro_rules! lit {
        ($val:expr => ($span_l:expr, $span_r:expr)) => {
            MBox::new(
                Expression::Literal {
                    value: Literal::Long($val),
                },
                Span::from(($span_l, $span_r)),
            )
        };
    }

    macro_rules! op {
        ($op_val:expr => ($span_l:expr, $span_r:expr)) => {
            M::new($op_val, Span::from(($span_l, $span_r)))
        };
    }

    macro_rules! bin {
        (($left:expr, $op:expr, $right:expr) => ($span_l:expr, $span_r:expr)) => {
            MBox::new(
                Expression::BinaryOp {
                    left: $left,
                    op: $op,
                    right: $right,
                },
                Span::from(($span_l, $span_r)),
            )
        };
    }

    #[test]
    fn parse_expression_respects_precedence() {
        let source0 = "0 + 1 * 2";
        let expected0 = bin! (
            (
                lit!(Some(0) => (0, 1)),
                op!(BinaryOp::Add => (2, 1)),
                bin!((
                    lit!(Some(1) => (4, 1)),
                    op!(BinaryOp::Mul => (6, 1)),
                    lit!(Some(2) => (8, 1))
                ) => (4, 5))
            ) => (0, 9)
        );

        let source1 = "0 * 1 + 2";
        let expected1 = bin! (
            (
                bin!((
                    lit!(Some(0) => (0, 1)),
                    op!(BinaryOp::Mul => (2, 1)),
                    lit!(Some(1) => (4, 1))
                ) => (0, 5)),
                op!(BinaryOp::Add => (6, 1)),
                lit!(Some(2) => (8, 1))
            ) => (0, 9)
        );

        let cases = [(source0, expected0), (source1, expected1)];

        for (source, expected) in cases {
            assert_eq!(parse_expression(&mut make_input(source)).unwrap(), expected);
        }
    }

    #[test]
    fn parse_expression_respects_associativity() {
        let source0 = "0 + 1 + 2";
        let expected0 = bin! (
            (
                bin!((
                    lit!(Some(0) => (0, 1)),
                    op!(BinaryOp::Add => (2, 1)),
                    lit!(Some(1) => (4, 1))
                ) => (0, 5)),
                op!(BinaryOp::Add => (6, 1)),
                lit!(Some(2) => (8, 1))
            ) => (0, 9)
        );

        let source1 = "0 * 1 * 2";
        let expected1 = bin! (
            (
                bin!((
                    lit!(Some(0) => (0, 1)),
                    op!(BinaryOp::Mul => (2, 1)),
                    lit!(Some(1) => (4, 1))
                ) => (0, 5)),
                op!(BinaryOp::Mul => (6, 1)),
                lit!(Some(2) => (8, 1))
            ) => (0, 9)
        );

        let cases = [(source0, expected0), (source1, expected1)];

        for (source, expected) in cases {
            assert_eq!(parse_expression(&mut make_input(source)).unwrap(), expected);
        }
    }
}
