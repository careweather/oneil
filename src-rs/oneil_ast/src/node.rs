use std::{fmt::Debug, ops::Deref};

use crate::{Span, span::SpanLike};

#[derive(Debug, Clone, PartialEq)]
pub struct Node<T>
where
    T: Debug + Clone + PartialEq,
{
    span: Span,
    value: T,
}

impl<T> Node<T>
where
    T: Debug + Clone + PartialEq,
{
    pub fn new(spanlike: impl SpanLike, value: T) -> Self {
        let span = Span::from(&spanlike);
        Self { span, value }
    }

    pub fn node_span(&self) -> &Span {
        &self.span
    }

    pub fn node_value(&self) -> &T {
        &self.value
    }
}

impl<T> SpanLike for Node<T>
where
    T: Debug + Clone + PartialEq,
{
    fn get_start(&self) -> usize {
        self.span.start()
    }

    fn get_end(&self) -> usize {
        self.span.end()
    }

    fn get_whitespace_end(&self) -> usize {
        self.span.whitespace_end()
    }
}

impl<T> From<T> for Node<T>
where
    T: Debug + Clone + PartialEq + SpanLike,
{
    fn from(value: T) -> Self {
        let span = Span::from(&value);
        Self::new(span, value)
    }
}

// this allows us to treat node as the value it contains
impl<T> Deref for Node<T>
where
    T: Debug + Clone + PartialEq,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
