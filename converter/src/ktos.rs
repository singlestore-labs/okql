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
        } => todo!(),
        kast::Expression::BinaryOp { left, op, right } => todo!(),
        kast::Expression::Literal { value } => todo!(),
    };

    Ok(Box::new(value))
}
