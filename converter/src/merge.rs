use kql_parser::ast as kast;
use sql_emitter::ast as sast;

use crate::ConverterError;

#[derive(PartialEq)]
pub enum MergeOutcome {
    Combined,
    CannotCombine,
}

pub fn attempt_merge(
    head: &mut sast::SelectStatement,
    operator: &kast::TabularOperator,
) -> Result<MergeOutcome, ConverterError> {
    match operator {
        kast::TabularOperator::Count => todo!(),
        kast::TabularOperator::Distinct { columns } => todo!(),
        kast::TabularOperator::Extend { columns } => todo!(),
        kast::TabularOperator::Join {
            params,
            right_table,
            attributes,
        } => todo!(),
        kast::TabularOperator::Limit { limit } => todo!(),
        kast::TabularOperator::Project { columns } => match &head.select {
            sast::SelectList::Explicit { fields } => Ok(MergeOutcome::CannotCombine),
            sast::SelectList::Wildcard => {
                let fields: Result<Vec<sast::SelectColumn>, ConverterError> =
                    columns.iter().map(ktos_column_definition).collect();
                let fields = fields?;
                head.select = sast::SelectList::Explicit { fields };
                Ok(MergeOutcome::Combined)
            }
        },
        kast::TabularOperator::Sort {
            by_kwd,
            expr,
            order,
            nulls,
        } => todo!(),
        kast::TabularOperator::Summarize {
            result_columns,
            by_kwd,
            grouping_columns,
        } => todo!(),
        kast::TabularOperator::Top {
            limit,
            by_kwd,
            expr,
            order,
            nulls,
        } => todo!(),
        kast::TabularOperator::Where { expr } => todo!(),
    }
}
