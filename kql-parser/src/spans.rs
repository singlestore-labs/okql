use miette::SourceSpan;

pub type Span = SourceSpan;

/// The metadata wrapper type
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct M<T> {
    pub span: Span,
    pub value: T
}

impl<T> M<T> {
    pub fn new(value: T, span: Span) -> Self {
        M { span, value }
    }

    pub fn new_range(value: T, left: Span, right: Span) -> Self {
        M {
            span: join_spans(left, right),
            value
        }
    }
}

/// The boxed metadata wrapper type
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct MBox<T> {
    pub span: Span,
    pub value: Box<T>
}

impl<T> MBox<T> {
    pub fn new(value: T, span: Span) -> Self {
        MBox { span, value: Box::new(value) }
    }

    pub fn new_range(value: T, left: Span, right: Span) -> Self {
        MBox {
            span: join_spans(left, right),
            value: Box::new(value)
        }
    }
}

pub fn span_precedes_span(left: Span, right: Span) -> bool {
    left.offset() + left.len() == right.offset()
}

pub fn join_spans(left: Span, right: Span) -> Span {
    let left_most = left.offset();
    let right_most = right.offset() + right.len();
    let len = right_most - left_most;
    Span::from((left_most, len))
}