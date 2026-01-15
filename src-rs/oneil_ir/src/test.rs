//! Testing for Oneil model IR.

use oneil_shared::span::Span;

use crate::{debug_info::TraceLevel, expr::Expr};

/// An index for identifying tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TestIndex(usize);

impl TestIndex {
    /// Creates a new test index from a numeric value.
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }
}

/// A test within a model.
#[derive(Debug, Clone, PartialEq)]
pub struct Test {
    span: Span,
    trace_level: TraceLevel,
    expr: Expr,
}

impl Test {
    /// Creates a new test with the specified properties.
    #[must_use]
    pub const fn new(span: Span, trace_level: TraceLevel, expr: Expr) -> Self {
        Self {
            span,
            trace_level,
            expr,
        }
    }

    /// Returns the span of the entire test definition.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the trace level for this test.
    #[must_use]
    pub const fn trace_level(&self) -> TraceLevel {
        self.trace_level
    }

    /// Returns the test expression that defines the expected behavior.
    #[must_use]
    pub const fn expr(&self) -> &Expr {
        &self.expr
    }
}
