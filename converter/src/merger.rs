use kql_parser::{
    ast as kast,
    spans::{MBox, Span, M},
};
use sql_emitter::ast as sast;

use std::sync::Arc;

use miette::NamedSource;

use crate::ConverterError;

#[derive(Debug)]
pub struct Merger {
    src: Arc<NamedSource>,
    columns: ColumnsState,
}

#[derive(PartialEq, Default, Debug)]
enum ColumnsState {
    #[default]
    Unmodified,
    Limited {
        columns: Vec<String>,
    },
    Modified {
        retained: Vec<String>,
        modified: Vec<String>,
    },
}

impl Merger {
    pub fn new(src: Arc<NamedSource>) -> Self {
        Merger {
            src,
            columns: ColumnsState::Unmodified,
        }
    }

    fn reset_columns(&mut self) {
        self.columns = ColumnsState::default();
    }

    fn non_condition_expression(&self, span: Span) -> ConverterError {
        ConverterError::ExpressionNotCondition {
            src: self.src.clone(),
            span,
        }
    }

    fn not_yet_implemented<T>(&self, span: Span, feature: &str) -> Result<T, ConverterError> {
        Err(ConverterError::NotImplemented {
            src: self.src.clone(),
            feature: feature.to_string(),
            span,
        })
    }

    /// Takes a SELECT statement and either modifies it to include the provided operator
    /// or creates a new SELECT statement wrapping the old one which does.
    pub fn merge_operator(
        mut self,
        mut head: sast::SelectStatement,
        name: M<String>,
        operator: kast::TabularOperator,
    ) -> Result<(Self, sast::SelectStatement), ConverterError> {
        match operator {
            kast::TabularOperator::Count => self.not_yet_implemented(name.span, "count operator"),

            kast::TabularOperator::Distinct { columns } => {
                self.not_yet_implemented(name.span, "distinct operator")
            }

            kast::TabularOperator::Extend { columns } => {
                self.not_yet_implemented(name.span, "extend operator")
            }

            kast::TabularOperator::Join {
                params,
                right_table,
                attributes,
            } => self.not_yet_implemented(name.span, "join operator"),

            kast::TabularOperator::Limit { limit } => {
                if head.limit.is_some() {
                    head = sast::SelectStatement::simple_wrapping(head);
                }
                head.limit = Some(limit.value);
                Ok((self, head))
            }

            kast::TabularOperator::Project { columns } => {
                // Compute column state
                let (retained, modified): (Vec<_>, Vec<_>) =
                    columns.iter().partition(|col| col.expr.is_none());
                let retained: Vec<String> = retained
                    .into_iter()
                    .map(|col| col.column.value.clone())
                    .collect();
                let modified: Vec<String> = modified
                    .into_iter()
                    .map(|col| col.column.value.clone())
                    .collect();
                let column_state = ColumnsState::Modified { retained, modified };
                // Compute SQL columns
                let new_columns: Vec<sast::SelectColumn> = columns
                    .into_iter()
                    .map(|col| self.to_select_column(col))
                    .collect::<Result<Vec<sast::SelectColumn>, ConverterError>>()?;

                if self.columns == ColumnsState::Unmodified {
                    // Update head
                    head.select.wildcard = false;
                    head.select.columns.extend(new_columns);
                    // Update state
                    self.columns = column_state;
                    //Result
                    Ok((self, head))
                } else {
                    self.columns = column_state;
                    let mut new_head = sast::SelectStatement::simple_wrapping(head);
                    new_head.select = sast::SelectList {
                        wildcard: false,
                        columns: new_columns,
                    };
                    Ok((self, new_head))
                }
            }
            kast::TabularOperator::Sort { by_kwd, sortings } => {
                self.not_yet_implemented(name.span, "sort operator")
            }

            kast::TabularOperator::Summarize {
                result_columns,
                by_kwd,
                grouping_columns,
            } => self.not_yet_implemented(name.span, "summarize operator"),

            kast::TabularOperator::Top {
                limit,
                by_kwd,
                expr,
                order,
                nulls,
            } => self.not_yet_implemented(name.span, "top operator"),

            kast::TabularOperator::Where { expr } => {
                let cond = self.to_search_condition(expr)?;
                let needs_wrapping = head.where_.is_some()
                    || if let ColumnsState::Modified { modified, .. } = &self.columns {
                        cond.depends_on_any(&modified)
                    } else {
                        false
                    };

                if needs_wrapping {
                    self.columns = ColumnsState::Unmodified;
                    head = sast::SelectStatement::simple_wrapping(head);
                }

                head.where_ = Some(cond);
                Ok((self, head))
            }
        }
    }

    fn to_select_column(
        &self,
        column: kast::ColumnDefinition,
    ) -> Result<sast::SelectColumn, ConverterError> {
        let name = column.column.value;

        let def = match column.expr {
            Some(k_expr) => sast::SelectColumn {
                value: self.to_value_expression(k_expr)?,
                alias: Some(name),
            },
            None => sast::SelectColumn {
                value: Box::new(sast::ValueExpression::Column { name }),
                alias: None,
            },
        };

        Ok(def)
    }

    pub fn to_value_expression(
        &self,
        expr: MBox<kast::Expression>,
    ) -> Result<Box<sast::ValueExpression>, ConverterError> {
        let value = match *expr.value {
            kast::Expression::Identifier { name } => {
                sast::ValueExpression::Column { name: name.value }
            }
            kast::Expression::FuncCall {
                name,
                open_paren_sym,
                args,
                close_paren_sym,
            } => {
                let name = name.value;
                let result: Result<Vec<Box<sast::ValueExpression>>, ConverterError> = args
                    .into_iter()
                    .map(|arg| self.to_value_expression(arg))
                    .collect();
                sast::ValueExpression::FuncCall {
                    name,
                    args: result?,
                }
            }
            kast::Expression::BinaryOp { left, op, right } => {
                let left = self.to_value_expression(left)?;
                let right = self.to_value_expression(right)?;
                match op.value {
                    kast::BinaryOp::Add
                    | kast::BinaryOp::Sub
                    | kast::BinaryOp::Mul
                    | kast::BinaryOp::Div => sast::ValueExpression::ArithmeticExpr {
                        left,
                        op: match op.value {
                            kast::BinaryOp::Add => sast::ArithmeticOperator::Add,
                            kast::BinaryOp::Sub => sast::ArithmeticOperator::Sub,
                            kast::BinaryOp::Mul => sast::ArithmeticOperator::Mul,
                            kast::BinaryOp::Div => sast::ArithmeticOperator::Div,
                            _ => unreachable!(),
                        },
                        right,
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
            }
            kast::Expression::Literal { value } => {
                let literal = match value {
                    kast::Literal::Bool(Some(v)) => sast::Literal::Bool(v),
                    kast::Literal::Int(Some(v)) => sast::Literal::Integer(v as i64),
                    kast::Literal::Long(Some(v)) => sast::Literal::Integer(v),
                    kast::Literal::Real(Some(v)) => sast::Literal::Real(v),
                    kast::Literal::String(v) => sast::Literal::String(v),
                    _ => todo!(),
                };
                sast::ValueExpression::Literal { value: literal }
            }
        };

        Ok(Box::new(value))
    }

    fn to_search_condition(
        &self,
        expr: MBox<kast::Expression>,
    ) -> Result<Box<sast::SearchCondition>, ConverterError> {
        let cond: sast::SearchCondition = match *expr.value {
            kast::Expression::Identifier { .. }
            | kast::Expression::FuncCall { .. }
            | kast::Expression::Literal { .. } => {
                return Err(self.non_condition_expression(expr.span))
            }
            kast::Expression::BinaryOp { left, op, right } => match op.value {
                kast::BinaryOp::Add
                | kast::BinaryOp::Sub
                | kast::BinaryOp::Mul
                | kast::BinaryOp::Div
                | kast::BinaryOp::Mod => return Err(self.non_condition_expression(expr.span)),

                kast::BinaryOp::LogicalAnd | kast::BinaryOp::LogicalOr => {
                    sast::SearchCondition::BoolExpr {
                        left: self.to_search_condition(left)?,
                        op: match op.value {
                            kast::BinaryOp::LogicalAnd => sast::BoolOperator::AND,
                            kast::BinaryOp::LogicalOr => sast::BoolOperator::OR,
                            _ => unreachable!(),
                        },
                        right: self.to_search_condition(right)?,
                    }
                }

                kast::BinaryOp::LT
                | kast::BinaryOp::GT
                | kast::BinaryOp::EQ
                | kast::BinaryOp::NEQ
                | kast::BinaryOp::LTE
                | kast::BinaryOp::GTE => sast::SearchCondition::ComparisonExpr {
                    left: self.to_value_expression(left)?,
                    op: match op.value {
                        kast::BinaryOp::LT => sast::ComparisonOperator::LT,
                        kast::BinaryOp::GT => sast::ComparisonOperator::GT,
                        kast::BinaryOp::EQ => sast::ComparisonOperator::EQ,
                        kast::BinaryOp::NEQ => sast::ComparisonOperator::NEQ,
                        kast::BinaryOp::LTE => sast::ComparisonOperator::LTE,
                        kast::BinaryOp::GTE => sast::ComparisonOperator::GTE,
                        _ => unreachable!(),
                    },
                    right: self.to_value_expression(right)?,
                },
            },
        };
        Ok(Box::new(cond))
    }
}
