
pub struct SelectStatement {
    pub modifier: Option<Modifier>,
    pub select: SelectList,
    pub from: TableReference,
    pub where_: Option<SearchCondition>,
    pub order_by: Option<OrderByClause>
}

pub enum Modifier {
    All,
    Distinct
}

pub enum SelectList {
    Explicit {
        fields: Vec<String>
    },
    Wildcard
}

pub enum TableReference {
    TableName {
        name: String
    },
    InnerStatement {
        value: Box<SelectStatement>
    }
}

pub enum SearchCondition {
    BoolExpr {
        left: Box<SearchCondition>,
        op: BoolOperator,
        right: Box<SearchCondition>
    },
    ComparisonExpr {
        left: Box<ValueExpression>,
        op: ComparisonOperator,
        right: Box<ValueExpression>
    }
}

pub enum BoolOperator {
    AND, OR
}

pub enum ComparisonOperator {
    LT, GT, LTE, GTE, EQ, NEQ
}

pub enum ValueExpression {
    FuncCall {
        name: String,
        args: Vec<ValueExpression>
    },
    ArithmeticExpr {
        left: Box<ValueExpression>,
        op: ArithmeticOperator,
        right: Box<ValueExpression>
    }
}

pub enum ArithmeticOperator {
    Add, Sub, Mul, Div
}

pub struct OrderByClause {
    pub specs: Vec<SortSpecification>
}

pub struct SortSpecification {
    pub column_name: String,
    pub order: SortOrder
}

pub enum SortOrder {
    Ascending,
    Descending
}