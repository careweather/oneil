//! Testing for Oneil model IR.
//!
//! This module provides the data structures for defining and managing tests
//! in Oneil models.

use std::collections::HashMap;

use crate::{debug_info::TraceLevel, expr::ExprWithSpan, reference::Identifier, span::Span};

/// An index for identifying tests.
///
/// `TestIndex` provides a unique identifier for tests within a model.
/// It wraps a `usize` value and provides a type-safe way to reference tests.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TestIndex(usize);

impl TestIndex {
    /// Creates a new test index from a numeric value.
    ///
    /// # Arguments
    ///
    /// * `index` - The numeric index value
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::test::TestIndex;
    ///
    /// let index = TestIndex::new(0);
    /// ```
    pub fn new(index: usize) -> Self {
        Self(index)
    }
}

/// A test within a model.
///
/// `Test` represents a test that validates the output of a model. It
/// includes:
///
/// - **Trace Level**: How much debugging information to output during test execution
/// - **Inputs**: Set of parameter identifiers that serve as test inputs
/// - **Test Expression**: The expression that defines the expected behavior
///
/// Tests are used to ensure that the model produces correct
/// results given specific input values.
#[derive(Debug, Clone, PartialEq)]
pub struct Test {
    trace_level: TraceLevel,
    inputs: HashMap<Identifier, Span>,
    test_expr: ExprWithSpan,
}

impl Test {
    /// Creates a new test with the specified properties.
    ///
    /// # Arguments
    ///
    /// * `trace_level` - The trace level for debugging output
    /// * `inputs` - Set of parameter identifiers that serve as test inputs
    /// * `test_expr` - The expression that defines the expected test behavior
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{test::Test, expr::{Expr, Literal}, debug_info::TraceLevel, reference::Identifier, span::{Span, WithSpan}};
    /// use std::collections::HashMap;
    ///
    /// let mut inputs = HashMap::new();
    /// inputs.insert(Identifier::new("radius"), Span::new(0, 0));
    ///
    /// let test_expr = WithSpan::test_new(Expr::literal(Literal::number(78.54))); // Expected area for radius = 5
    /// let test = Test::new(TraceLevel::None, inputs, test_expr);
    /// ```
    pub fn new(
        trace_level: TraceLevel,
        inputs: HashMap<Identifier, Span>,
        test_expr: ExprWithSpan,
    ) -> Self {
        Self {
            trace_level,
            inputs,
            test_expr,
        }
    }

    /// Returns the trace level for this test.
    ///
    /// The trace level determines how much debugging information
    /// is output during test execution.
    ///
    /// # Returns
    ///
    /// A reference to the trace level for this test.
    pub fn trace_level(&self) -> &TraceLevel {
        &self.trace_level
    }

    /// Returns the set of input parameters for this test.
    ///
    /// Input parameters are the identifiers of parameters that
    /// should be set to specific values when running this test.
    ///
    /// # Returns
    ///
    /// A reference to the set of input parameter identifiers.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{test::Test, expr::{Expr, Literal}, debug_info::TraceLevel, reference::Identifier, span::{Span, WithSpan}};
    /// use std::collections::HashMap;
    ///
    /// let mut inputs = HashMap::new();
    /// inputs.insert(Identifier::new("radius"), Span::new(0, 0));
    /// inputs.insert(Identifier::new("height"), Span::new(0, 0));
    ///
    /// let test = Test::new(TraceLevel::None, inputs, WithSpan::test_new(Expr::literal(Literal::number(0.0))));
    ///
    /// assert!(test.inputs().contains_key(&Identifier::new("radius")));
    /// assert!(test.inputs().contains_key(&Identifier::new("height")));
    /// ```
    pub fn inputs(&self) -> &HashMap<Identifier, Span> {
        &self.inputs
    }

    /// Returns the test expression that defines the expected behavior.
    ///
    /// The test expression is evaluated during test execution and
    /// compared against the actual model output to determine if
    /// the test passes or fails.
    ///
    /// # Returns
    ///
    /// A reference to the test expression.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{test::Test, expr::{Expr, Literal}, debug_info::TraceLevel, span::WithSpan};
    /// use std::collections::HashMap;
    ///
    /// let expected_area = WithSpan::test_new(Expr::literal(Literal::number(78.54)));
    /// let test = Test::new(TraceLevel::None, HashMap::new(), expected_area.clone());
    ///
    /// assert_eq!(test.test_expr(), &expected_area);
    /// ```
    pub fn test_expr(&self) -> &ExprWithSpan {
        &self.test_expr
    }
}
