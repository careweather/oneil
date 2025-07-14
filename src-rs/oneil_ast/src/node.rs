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
    pub fn new(span: Span, value: T) -> Self {
        Self { span, value }
    }

    pub fn get_span(&self) -> &Span {
        &self.span
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
