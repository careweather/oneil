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

/// An index for identifying model tests.
///
/// `TestIndex` provides a unique identifier for model tests within a model.
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
/// `ModelTest` represents a test that validates the output of a model. It
/// includes:
///
/// - **Trace Level**: How much debugging information to output during test execution
/// - **Inputs**: Set of parameter identifiers that serve as test inputs
/// - **Test Expression**: The expression that defines the expected behavior
///
/// Model tests are used to ensure that the model produces correct
/// results given specific input values.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelTest {
    trace_level: TraceLevel,
    inputs: HashSet<IdentifierWithSpan>,
    test_expr: ExprWithSpan,
}

impl ModelTest {
    /// Creates a new model test with the specified properties.
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
    /// use oneil_ir::{test::ModelTest, expr::{Expr, Literal}, debug_info::TraceLevel, reference::Identifier, span::WithSpan};
    /// use std::collections::HashSet;
    ///
    /// let mut inputs = HashSet::new();
    /// inputs.insert(WithSpan::test_new(Identifier::new("radius")));
    ///
    /// let test_expr = WithSpan::test_new(Expr::literal(Literal::number(78.54))); // Expected area for radius = 5
    /// let test = ModelTest::new(TraceLevel::None, inputs, test_expr);
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
    /// use oneil_ir::{test::ModelTest, expr::{Expr, Literal}, debug_info::TraceLevel, reference::Identifier, span::WithSpan};
    /// use std::collections::HashSet;
    ///
    /// let mut inputs = HashSet::new();
    /// inputs.insert(WithSpan::test_new(Identifier::new("radius")));
    /// inputs.insert(WithSpan::test_new(Identifier::new("height")));
    ///
    /// let test = ModelTest::new(TraceLevel::None, inputs, WithSpan::test_new(Expr::literal(Literal::number(0.0))));
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
    /// use oneil_ir::{test::ModelTest, expr::{Expr, Literal}, debug_info::TraceLevel, span::WithSpan};
    /// use std::collections::HashSet;
    ///
    /// let expected_area = WithSpan::test_new(Expr::literal(Literal::number(78.54)));
    /// let test = ModelTest::new(TraceLevel::None, HashSet::new(), expected_area.clone());
    ///
    /// assert_eq!(test.test_expr(), &expected_area);
    /// ```
    pub fn test_expr(&self) -> &ExprWithSpan {
        &self.test_expr
    }
}

/// A test for validating the behavior of a specific submodel.
///
/// `SubmodelTest` represents testing done for a single submodel within a
/// model. It includes:
///
/// - **Submodel Name**: The identifier of the submodel being tested
/// - **Inputs**: Mapping of test input identifiers to their values
///
/// Submodel tests are used to ensure that individual submodels
/// work correctly with specific input parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct SubmodelTest {
    submodel_name: Identifier,
    inputs: SubmodelTestInputs,
}

impl SubmodelTest {
    /// Creates a new submodel test with the specified properties.
    ///
    /// # Arguments
    ///
    /// * `submodel_name` - The identifier of the submodel to test
    /// * `inputs` - The input values for the submodel test
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{test::{SubmodelTest, SubmodelTestInputs}, expr::{Expr, Literal}, reference::Identifier, span::WithSpan};
    /// use std::collections::HashMap;
    ///
    /// let mut inputs = HashMap::new();
    /// inputs.insert(WithSpan::test_new(Identifier::new("radius")), WithSpan::test_new(Expr::literal(Literal::number(5.0))));
    ///
    /// let test = SubmodelTest::new(
    ///     Identifier::new("circle_area"),
    ///     SubmodelTestInputs::new(inputs)
    /// );
    /// ```
    pub fn new(submodel_name: Identifier, inputs: SubmodelTestInputs) -> Self {
        Self {
            submodel_name,
            inputs,
        }
    }

    /// Returns the name of the submodel being tested.
    ///
    /// # Returns
    ///
    /// A reference to the submodel identifier.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{test::{SubmodelTest, SubmodelTestInputs}, reference::Identifier};
    /// use std::collections::HashMap;
    ///
    /// let test = SubmodelTest::new(
    ///     Identifier::new("my_submodel"),
    ///     SubmodelTestInputs::new(HashMap::new())
    /// );
    ///
    /// assert_eq!(test.submodel_name().value(), "my_submodel");
    /// ```
    pub fn submodel_name(&self) -> &Identifier {
        &self.submodel_name
    }

    /// Returns the input values for this submodel test.
    ///
    /// The inputs define the parameter values that should be used
    /// when testing the submodel.
    ///
    /// # Returns
    ///
    /// A reference to the submodel test inputs.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{test::{SubmodelTest, SubmodelTestInputs}, expr::{Expr, Literal}, reference::Identifier, span::WithSpan};
    /// use std::collections::HashMap;
    ///
    /// let mut inputs = HashMap::new();
    /// inputs.insert(WithSpan::test_new(Identifier::new("x")), WithSpan::test_new(Expr::literal(Literal::number(10.0))));
    /// inputs.insert(WithSpan::test_new(Identifier::new("y")), WithSpan::test_new(Expr::literal(Literal::number(20.0))));
    ///
    /// let test = SubmodelTest::new(
    ///     Identifier::new("calculator"),
    ///     SubmodelTestInputs::new(inputs)
    /// );
    ///
    /// let test_inputs = test.inputs();
    /// assert_eq!(test_inputs.len(), 2);
    /// assert!(test_inputs.contains_key(&WithSpan::test_new(Identifier::new("x"))));
    /// ```
    pub fn inputs(&self) -> &SubmodelTestInputs {
        &self.inputs
    }
}

/// Input values for a submodel test.
///
/// `SubmodelTestInputs` provides a mapping of parameter identifiers
/// to their test values. It implements `Deref` to provide direct
/// access to the underlying mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct SubmodelTestInputs(HashMap<IdentifierWithSpan, ExprWithSpan>);

impl SubmodelTestInputs {
    /// Creates new submodel test inputs from a mapping of identifiers to expressions.
    ///
    /// # Arguments
    ///
    /// * `inputs` - Mapping of parameter identifiers to their test values
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{test::SubmodelTestInputs, expr::{Expr, Literal}, reference::Identifier, span::WithSpan};
    /// use std::collections::HashMap;
    ///
    /// let mut inputs = HashMap::new();
    /// inputs.insert(WithSpan::test_new(Identifier::new("width")), WithSpan::test_new(Expr::literal(Literal::number(10.0))));
    /// inputs.insert(WithSpan::test_new(Identifier::new("height")), WithSpan::test_new(Expr::literal(Literal::number(5.0))));
    ///
    /// let test_inputs = SubmodelTestInputs::new(inputs);
    /// ```
    pub fn new(inputs: HashMap<IdentifierWithSpan, ExprWithSpan>) -> Self {
        Self(inputs)
    }
}

impl Deref for SubmodelTestInputs {
    type Target = HashMap<IdentifierWithSpan, ExprWithSpan>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
