use kql_parser::ast as kast;
use sql_emitter::ast as sast;

use crate::ConverterError;

fn ktos_column_definition(
    column: &kast::ColumnDefinition,
) -> Result<sast::SelectColumn, ConverterError> {
    let name = &column.column.value;

    let value = match &column.expr {
        Some(k_expr) => ktos_expression(&k_expr.value)?,
        None => Box::new(sast::ValueExpression::Column { name: name.clone() }),
    };
    Ok(sast::SelectColumn {
        value,
        alias: Some(name.clone()),
    })
}

fn ktos_expression(expr: &kast::Expression) -> Result<Box<sast::ValueExpression>, ConverterError> {
    let value = match expr {
        kast::Expression::Identifier { name } => sast::ValueExpression::Column {
            name: name.value.clone(),
        },
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
