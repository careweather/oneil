//! Testing for Oneil model IR.
//!
//! This module provides the data structures for defining and managing tests
//! in Oneil models.

use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use crate::{
    debug_info::TraceLevel,
    expr::ExprWithSpan,
    reference::{Identifier, IdentifierWithSpan},
};

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
    inputs: HashSet<IdentifierWithSpan>,
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
    /// use oneil_ir::{test::Test, expr::{Expr, Literal}, debug_info::TraceLevel, reference::Identifier, span::WithSpan};
    /// use std::collections::HashSet;
    ///
    /// let mut inputs = HashSet::new();
    /// inputs.insert(WithSpan::test_new(Identifier::new("radius")));
    ///
    /// let test_expr = WithSpan::test_new(Expr::literal(Literal::number(78.54))); // Expected area for radius = 5
    /// let test = Test::new(TraceLevel::None, inputs, test_expr);
    /// ```
    pub fn new(
        trace_level: TraceLevel,
        inputs: HashSet<IdentifierWithSpan>,
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
    /// use oneil_ir::{test::Test, expr::{Expr, Literal}, debug_info::TraceLevel, reference::Identifier, span::WithSpan};
    /// use std::collections::HashSet;
    ///
    /// let mut inputs = HashSet::new();
    /// inputs.insert(WithSpan::test_new(Identifier::new("radius")));
    /// inputs.insert(WithSpan::test_new(Identifier::new("height")));
    ///
    /// let test = Test::new(TraceLevel::None, inputs, WithSpan::test_new(Expr::literal(Literal::number(0.0))));
    ///
    /// assert!(test.inputs().contains(&WithSpan::test_new(Identifier::new("radius"))));
    /// assert!(test.inputs().contains(&WithSpan::test_new(Identifier::new("height"))));
    /// ```
    pub fn inputs(&self) -> &HashSet<IdentifierWithSpan> {
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
    /// use std::collections::HashSet;
    ///
    /// let expected_area = WithSpan::test_new(Expr::literal(Literal::number(78.54)));
    /// let test = Test::new(TraceLevel::None, HashSet::new(), expected_area.clone());
    ///
    /// assert_eq!(test.test_expr(), &expected_area);
    /// ```
    pub fn test_expr(&self) -> &ExprWithSpan {
        &self.test_expr
    }
}

/// A test for validating the behavior of a model.
///
/// `ModelTest` represents testing done for a single model. It includes:
///
/// - **Model Name**: The identifier of the model being tested
/// - **Inputs**: Mapping of test input identifiers to their values
///
/// Model tests are used to ensure that individual models
/// work correctly with specific input parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelTest {
    model_name: Identifier,
    inputs: ModelTestInputs,
}

impl ModelTest {
    /// Creates a new model test with the specified properties.
    ///
    /// # Arguments
    ///
    /// * `model_name` - The identifier of the model to test
    /// * `inputs` - The input values for the model test
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{test::{ModelTest, ModelTestInputs}, expr::{Expr, Literal}, reference::Identifier, span::WithSpan};
    /// use std::collections::HashMap;
    ///
    /// let mut inputs = HashMap::new();
    /// inputs.insert(WithSpan::test_new(Identifier::new("radius")), WithSpan::test_new(Expr::literal(Literal::number(5.0))));
    ///
    /// let test = ModelTest::new(
    ///     Identifier::new("circle_area"),
    ///     ModelTestInputs::new(inputs)
    /// );
    /// ```
    pub fn new(model_name: Identifier, inputs: ModelTestInputs) -> Self {
        Self { model_name, inputs }
    }

    /// Returns the name of the model being tested.
    ///
    /// # Returns
    ///
    /// A reference to the model identifier.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{test::{ModelTest, ModelTestInputs}, reference::Identifier};
    /// use std::collections::HashMap;
    ///
    /// let test = ModelTest::new(
    ///     Identifier::new("my_model"),
    ///     ModelTestInputs::new(HashMap::new())
    /// );
    ///
    /// assert_eq!(test.model_name().as_str(), "my_model");
    /// ```
    pub fn model_name(&self) -> &Identifier {
        &self.model_name
    }

    /// Returns the input values for this model test.
    ///
    /// The inputs define the parameter values that should be used
    /// when testing the model.
    ///
    /// # Returns
    ///
    /// A reference to the model test inputs.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{test::{ModelTest, ModelTestInputs}, expr::{Expr, Literal}, reference::Identifier, span::WithSpan};
    /// use std::collections::HashMap;
    ///
    /// let mut inputs = HashMap::new();
    /// inputs.insert(WithSpan::test_new(Identifier::new("x")), WithSpan::test_new(Expr::literal(Literal::number(10.0))));
    /// inputs.insert(WithSpan::test_new(Identifier::new("y")), WithSpan::test_new(Expr::literal(Literal::number(20.0))));
    ///
    /// let test = ModelTest::new(
    ///     Identifier::new("calculator"),
    ///     ModelTestInputs::new(inputs)
    /// );
    ///
    /// let test_inputs = test.inputs();
    /// assert_eq!(test_inputs.len(), 2);
    /// assert!(test_inputs.contains_key(&WithSpan::test_new(Identifier::new("x"))));
    /// ```
    pub fn inputs(&self) -> &ModelTestInputs {
        &self.inputs
    }
}

/// Input values for a model test.
///
/// `ModelTestInputs` provides a mapping of parameter identifiers
/// to their test values. It implements `Deref` to provide direct
/// access to the underlying mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelTestInputs(HashMap<IdentifierWithSpan, ExprWithSpan>);

impl ModelTestInputs {
    /// Creates new model test inputs from a mapping of identifiers to expressions.
    ///
    /// # Arguments
    ///
    /// * `inputs` - Mapping of parameter identifiers to their test values
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{test::ModelTestInputs, expr::{Expr, Literal}, reference::Identifier, span::WithSpan};
    /// use std::collections::HashMap;
    ///
    /// let mut inputs = HashMap::new();
    /// inputs.insert(WithSpan::test_new(Identifier::new("width")), WithSpan::test_new(Expr::literal(Literal::number(10.0))));
    /// inputs.insert(WithSpan::test_new(Identifier::new("height")), WithSpan::test_new(Expr::literal(Literal::number(5.0))));
    ///
    /// let test_inputs = ModelTestInputs::new(inputs);
    /// ```
    pub fn new(inputs: HashMap<IdentifierWithSpan, ExprWithSpan>) -> Self {
        Self(inputs)
    }
}

impl Deref for ModelTestInputs {
    type Target = HashMap<IdentifierWithSpan, ExprWithSpan>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
