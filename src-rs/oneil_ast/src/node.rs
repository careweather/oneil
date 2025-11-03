//! AST node wrapper with source location information

use std::{fmt::Debug, ops::Deref};

use oneil_shared::span::Span;

/// A wrapper around AST elements that includes source location information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node<T> {
    value: Box<T>,
    span: Span,
    whitespace_span: Span,
}

impl<T> Node<T> {
    /// Creates a new node with the given span and value
    #[must_use]
    pub fn new(value: T, span: Span, whitespace_span: Span) -> Self {
        let value = Box::new(value);
        Self {
            value,
            span,
            whitespace_span,
        }
    }

    /// Returns a reference to the node's span information
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns a reference to the node's whitespace span information
    #[must_use]
    pub const fn whitespace_span(&self) -> Span {
        self.whitespace_span
    }

    /// Consumes the node and returns its value
    #[must_use]
    pub fn take_value(self) -> T {
        *self.value
    }

    /// Wraps the node in a new node with the same span and whitespace span
    #[must_use]
    pub fn wrap<U>(self, wrapper: impl FnOnce(Self) -> U) -> Node<U> {
        let span = self.span;
        let whitespace_span = self.whitespace_span;
        let value = wrapper(self);

        Node::new(value, span, whitespace_span)
    }
}

impl<T> Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
