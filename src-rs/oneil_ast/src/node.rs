//! AST node wrapper with source location information
//!
//! This module provides the `Node<T>` wrapper that combines AST elements with
//! source location information (spans) for error reporting and debugging.

use std::{fmt::Debug, ops::Deref};

use crate::{Span, span::SpanLike};

/// A wrapper around AST elements that includes source location information
///
/// Every AST element is wrapped in a `Node<T>` to provide source location
/// information for error reporting, debugging, and other source-aware operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node<T> {
    span: Span,
    value: Box<T>,
}

impl<T> Node<T> {
    /// Creates a new node with the given span and value
    pub fn new(spanlike: &impl SpanLike, value: T) -> Self {
        let span = Span::from(spanlike);
        let value = Box::new(value);
        Self { span, value }
    }

    /// Returns a reference to the node's span information
    #[must_use]
    pub const fn node_span(&self) -> Span {
        self.span
    }

    /// Returns a reference to the node's value
    #[must_use]
    pub const fn node_value(&self) -> &T {
        &self.value
    }

    /// Consumes the node and returns its value
    #[must_use]
    pub fn take_value(self) -> T {
        *self.value
    }
}

impl<T> SpanLike for Node<T>
where
    T: Debug + Clone + PartialEq,
{
    fn get_start(&self) -> usize {
        self.span.get_start()
    }

    fn get_length(&self) -> usize {
        self.span.get_length()
    }

    fn get_whitespace_length(&self) -> usize {
        self.span.get_whitespace_length()
    }
}

impl<T> SpanLike for &Node<T>
where
    T: Debug + Clone + PartialEq,
{
    fn get_start(&self) -> usize {
        self.span.get_start()
    }

    fn get_length(&self) -> usize {
        self.span.get_length()
    }

    fn get_whitespace_length(&self) -> usize {
        self.span.get_whitespace_length()
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
