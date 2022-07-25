use super::{M, MBox, Span, expression::Expression};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Query {
    /// The base table value to start with.
    pub table: M<String>,
    /// The tabular operators to apply to it.
    pub operators: Vec<(M<String>, TabularOperator)>
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum TabularOperator {
    Count,
    Distinct {
        /// Columns to ensure distinctness of
        columns: Columns
    },
    Extend {
        /// New columns to define
        columns: Vec<(M<String>, MBox<Expression>)>
    },
    Join {
        /// The join parameters
        params: JoinParams,
        /// The other table to join
        right_table: Box<Query>,
        /// Attributes to join tables on
        attributes: Vec<JoinAttribute>
    },
    Limit {
        /// The maximum number of records
        limit: M<i64>
    },
    Project {
        /// Columns to retain and new columns to define
        columns: Vec<ColumnDefinition>
    },
    Sort {
        /// Span for by keyword
        by_kwd: Span,
        /// The expression to sort on
        expr: MBox<Expression>,
        /// Ascending or descending
        order: Option<M<SortOrder>>,
        // Span of `nulls` keyword, value for `first`|`last`
        nulls: Option<(Span, M<NullsPosition>)>
    },
    Summarize {
        result_columns: Vec<ColumnDefinition>,
        by_kwd: Span,
        grouping_columns: Vec<ColumnDefinition>
    },
    Top { 
        /// The maximum number of records
        limit: M<i64>,
        /// Span for by keyword
        by_kwd: Span,
        /// The expression to sort on
        expr: MBox<Expression>,
        /// Ascending or descending
        order: Option<M<SortOrder>>,
        // Span of `nulls` keyword, value for `first`|`last`
        nulls: Option<(Span, M<NullsPosition>)>
    },
    Where {
        /// The expression to filter on
        expr: MBox<Expression>
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Columns {
    Explicit(Vec<M<String>>),
    Wildcard
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct JoinParams {
    kind: Option<JoinKind>
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum JoinKind {
    /// Inner join with left side deduplication
    /// Name "innerunique"
    InnerUnique,
    /// Standard inner join
    /// Name "inner"
    Inner,
    /// Left outer join
    /// Name "leftouter"
    LeftOuter,
    /// Right outer join
    /// Name "rightouter"
    RightOuter,
    /// Full outer join
    /// Name "fullouter"
    FullOuter,
    /// Left anti join
    /// Name "leftanti" or "anti" or "leftantisemi"
    LeftAnti,
    /// Right anti join
    /// Name "rightanti" or "rightantisemi"
    RightAnti,
    /// Left semi join
    /// Name "leftsemi"
    LeftSemi,
    /// Right semi join
    /// Name "rightsemi"
    RightSemi
}

impl JoinKind {
    /// See https://docs.microsoft.com/en-us/azure/data-explorer/kusto/query/joinoperator?pivots=azuredataexplorer#returns
    pub fn return_columns(&self) -> JoinReturnColumns {
        match self {
            JoinKind::LeftAnti
            | JoinKind::LeftSemi => JoinReturnColumns::Left,

            JoinKind::RightAnti
            | JoinKind::RightSemi => JoinReturnColumns::Right,

            JoinKind::InnerUnique
            | JoinKind::Inner
            | JoinKind::LeftOuter
            | JoinKind::RightOuter
            | JoinKind::FullOuter => JoinReturnColumns::Both,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum JoinReturnColumns {
    Left, Right, Both
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum JoinAttribute {
    Matching {
        name: M<String>
    },
    NonMatching {
        left_kwd: Span,
        left_name: M<String>,
        right_kwd: Span,
        right_name: M<String>
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ColumnDefinition {
    column: M<String>,
    expr: Option<MBox<Expression>>
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum SortOrder {
    Ascending,
    Descending
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum NullsPosition {
    First,
    Last
}