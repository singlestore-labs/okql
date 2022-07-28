use kql_parser::ast as kast;
use sql_emitter::ast as sast;

use crate::ConverterError;

pub fn column_definition(
    column: kast::ColumnDefinition,
) -> Result<sast::SelectColumn, ConverterError> {
    let name = column.column.value;

    let def = match column.expr {
        Some(k_expr) => sast::SelectColumn {
            value: expression(*k_expr.value)?,
            alias: Some(name),
        },
        None => sast::SelectColumn {
            value: Box::new(sast::ValueExpression::Column { name }),
            alias: None,
        },
    };

    Ok(def)
}

pub fn expression(expr: kast::Expression) -> Result<Box<sast::ValueExpression>, ConverterError> {
    let value = match expr {
        kast::Expression::Identifier { name } => sast::ValueExpression::Column { name: name.value },
        kast::Expression::FuncCall {
            name,
            open_paren_sym,
            args,
            close_paren_sym,
        } => {
            let name = name.value;
            let result: Result<Vec<Box<sast::ValueExpression>>, ConverterError> = args.into_iter().map(|arg| expression(*arg.value)).collect();
            sast::ValueExpression::FuncCall { name, args: result? }
        },
        kast::Expression::BinaryOp { left, op, right } => {
            let left = expression(*left.value)?;
            let right = expression(*right.value)?;
            match op.value {
                kast::BinaryOp::Add => sast::ValueExpression::ArithmeticExpr {
                    left, op: sast::ArithmeticOperator::Add, right
                },
                kast::BinaryOp::Sub => sast::ValueExpression::ArithmeticExpr {
                    left, op: sast::ArithmeticOperator::Sub, right
                },
                kast::BinaryOp::Mul => sast::ValueExpression::ArithmeticExpr {
                    left, op: sast::ArithmeticOperator::Mul, right
                },
                kast::BinaryOp::Div => sast::ValueExpression::ArithmeticExpr {
                    left, op: sast::ArithmeticOperator::Div, right
                },
                kast::BinaryOp::Mod => todo!(),
                kast::BinaryOp::LogicalAnd => todo!(),
                kast::BinaryOp::LogicalOr => todo!(),
                kast::BinaryOp::LT => todo!(),
                kast::BinaryOp::GT => todo!(),
                kast::BinaryOp::EQ => todo!(),
                kast::BinaryOp::NEQ => todo!(),
                kast::BinaryOp::LTE => todo!(),
                kast::BinaryOp::GTE => todo!(),
            }
        },
        kast::Expression::Literal { value } => {
            let literal = match value {
                kast::Literal::Bool(Some(v)) => sast::Literal::Bool(v),
                kast::Literal::Int(Some(v)) => sast::Literal::Integer(v as i64),
                kast::Literal::Long(Some(v)) => sast::Literal::Integer(v),
                kast::Literal::Real(Some(v)) => sast::Literal::Real(v),
                kast::Literal::String(v) => sast::Literal::String(v),
                _ => todo!()
            };
            sast::ValueExpression::Literal { value: literal }
        },
    };

    Ok(Box::new(value))
}
