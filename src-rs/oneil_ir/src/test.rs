//! Testing for Oneil model IR.

use crate::{debug_info::TraceLevel, expr::ExprWithSpan};

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
    trace_level: TraceLevel,
    test_expr: ExprWithSpan,
}

impl Test {
    /// Creates a new test with the specified properties.
    #[must_use]
    pub const fn new(trace_level: TraceLevel, test_expr: ExprWithSpan) -> Self {
        Self {
            trace_level,
            test_expr,
        }
    }

    /// Returns the trace level for this test.
    #[must_use]
    pub const fn trace_level(&self) -> TraceLevel {
        self.trace_level
    }

    /// Returns the test expression that defines the expected behavior.
    #[must_use]
    pub const fn test_expr(&self) -> &ExprWithSpan {
        &self.test_expr
    }
}
