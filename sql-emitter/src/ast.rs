use std::fmt;

pub struct SelectStatement {
    pub modifier: Option<Modifier>,
    pub select: SelectList,
    pub from: TableReference,
    pub where_: Option<SearchCondition>,
    pub order_by: Option<OrderByClause>,
}

impl SelectStatement {
    pub fn simple(table: String) -> Self {
        SelectStatement {
            modifier: None,
            select: SelectList {
                wildcard: true,
                columns: vec![],
            },
            from: TableReference::TableName { name: table },
            where_: None,
            order_by: None,
        }
    }

    pub fn simple_wrapping(other: Self) -> Self {
        SelectStatement {
            modifier: None,
            select: SelectList {
                wildcard: true,
                columns: vec![],
            },
            from: TableReference::InnerStatement {
                value: Box::new(other),
            },
            where_: None,
            order_by: None,
        }
    }
}

pub enum Modifier {
    All,
    Distinct,
}

pub struct SelectList {
    pub wildcard: bool,
    pub columns: Vec<SelectColumn>,
}

pub struct SelectColumn {
    pub value: Box<ValueExpression>,
    pub alias: Option<String>,
}

pub enum TableReference {
    TableName { name: String },
    InnerStatement { value: Box<SelectStatement> },
}

pub enum SearchCondition {
    BoolExpr {
        left: Box<SearchCondition>,
        op: BoolOperator,
        right: Box<SearchCondition>,
    },
    ComparisonExpr {
        left: Box<ValueExpression>,
        op: ComparisonOperator,
        right: Box<ValueExpression>,
    },
}

pub enum BoolOperator {
    AND,
    OR,
}

pub enum ComparisonOperator {
    LT,
    GT,
    LTE,
    GTE,
    EQ,
    NEQ,
}

pub enum ValueExpression {
    Column {
        name: String,
    },
    FuncCall {
        name: String,
        args: Vec<Box<ValueExpression>>,
    },
    ArithmeticExpr {
        left: Box<ValueExpression>,
        op: ArithmeticOperator,
        right: Box<ValueExpression>,
    },
    Literal {
        value: Literal,
    },
}

impl ValueExpression {
    fn depends_on_any(&self, columns: &Vec<String>) -> bool {
        match self {
            ValueExpression::Column { name } => columns.contains(name),
            ValueExpression::FuncCall { name, args } => {
                args.iter().any(|arg| arg.depends_on_any(columns))
            }
            ValueExpression::ArithmeticExpr { left, op, right } => {
                left.depends_on_any(columns) || right.depends_on_any(columns)
            }
            ValueExpression::Literal { value } => false,
        }
    }
}

pub enum ArithmeticOperator {
    Add,
    Sub,
    Mul,
    Div,
}

impl fmt::Display for ArithmeticOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArithmeticOperator::Add => write!(f, "+"),
            ArithmeticOperator::Sub => write!(f, "-"),
            ArithmeticOperator::Mul => write!(f, "*"),
            ArithmeticOperator::Div => write!(f, "/"),
        }
    }
}

pub struct OrderByClause {
    pub specs: Vec<SortSpecification>,
}

pub struct SortSpecification {
    pub column_name: String,
    pub order: SortOrder,
}

pub enum SortOrder {
    Ascending,
    Descending,
}

pub enum Literal {
    Bool(bool),
    Integer(i64),
    Real(f64),
    String(String),
}
