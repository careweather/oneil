//! Testing for Oneil modules.
//!
//! This module provides the data structures for defining and managing tests
//! in Oneil modules.

use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use crate::{debug_info::TraceLevel, expr::Expr, reference::Identifier};

/// An index for identifying model tests.
///
/// `TestIndex` provides a unique identifier for model tests within a module.
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
    /// use oneil_module::test::TestIndex;
    ///
    /// let index = TestIndex::new(0);
    /// ```
    pub fn new(index: usize) -> Self {
        Self(index)
    }
}

/// A test within a module.
///
/// `ModelTest` represents a test that validates the output of a module. It
/// includes:
///
/// - **Trace Level**: How much debugging information to output during test execution
/// - **Inputs**: Set of parameter identifiers that serve as test inputs
/// - **Test Expression**: The expression that defines the expected behavior
///
/// Model tests are used to ensure that the module produces correct
/// results given specific input values.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelTest {
    trace_level: TraceLevel,
    inputs: HashSet<Identifier>,
    test_expr: Expr,
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
    /// use oneil_module::{test::ModelTest, expr::{Expr, Literal}, debug_info::TraceLevel, reference::Identifier};
    /// use std::collections::HashSet;
    ///
    /// let mut inputs = HashSet::new();
    /// inputs.insert(Identifier::new("radius"));
    ///
    /// let test_expr = Expr::literal(Literal::number(78.54)); // Expected area for radius = 5
    /// let test = ModelTest::new(TraceLevel::None, inputs, test_expr);
    /// ```
    pub fn new(trace_level: TraceLevel, inputs: HashSet<Identifier>, test_expr: Expr) -> Self {
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
    /// use oneil_module::{test::ModelTest, expr::{Expr, Literal}, debug_info::TraceLevel, reference::Identifier};
    /// use std::collections::HashSet;
    ///
    /// let mut inputs = HashSet::new();
    /// inputs.insert(Identifier::new("radius"));
    /// inputs.insert(Identifier::new("height"));
    ///
    /// let test = ModelTest::new(TraceLevel::None, inputs, Expr::literal(Literal::number(0.0)));
    ///
    /// assert!(test.inputs().contains(&Identifier::new("radius")));
    /// assert!(test.inputs().contains(&Identifier::new("height")));
    /// ```
    pub fn inputs(&self) -> &HashSet<Identifier> {
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
    /// use oneil_module::{test::ModelTest, expr::{Expr, Literal}, debug_info::TraceLevel};
    /// use std::collections::HashSet;
    ///
    /// let expected_area = Expr::literal(Literal::number(78.54));
    /// let test = ModelTest::new(TraceLevel::None, HashSet::new(), expected_area.clone());
    ///
    /// assert_eq!(test.test_expr(), &expected_area);
    /// ```
    pub fn test_expr(&self) -> &Expr {
        &self.test_expr
    }
}

/// A test for validating the behavior of a specific submodel.
///
/// `SubmodelTest` represents testing done for a single submodel within a
/// module. It includes:
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
    /// use oneil_module::{test::{SubmodelTest, SubmodelTestInputs}, expr::{Expr, Literal}, reference::Identifier};
    /// use std::collections::HashMap;
    ///
    /// let mut inputs = HashMap::new();
    /// inputs.insert(Identifier::new("radius"), Expr::literal(Literal::number(5.0)));
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
    /// use oneil_module::{test::{SubmodelTest, SubmodelTestInputs}, reference::Identifier};
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
    /// use oneil_module::{test::{SubmodelTest, SubmodelTestInputs}, expr::{Expr, Literal}, reference::Identifier};
    /// use std::collections::HashMap;
    ///
    /// let mut inputs = HashMap::new();
    /// inputs.insert(Identifier::new("x"), Expr::literal(Literal::number(10.0)));
    /// inputs.insert(Identifier::new("y"), Expr::literal(Literal::number(20.0)));
    ///
    /// let test = SubmodelTest::new(
    ///     Identifier::new("calculator"),
    ///     SubmodelTestInputs::new(inputs)
    /// );
    ///
    /// let test_inputs = test.inputs();
    /// assert_eq!(test_inputs.len(), 2);
    /// assert!(test_inputs.contains_key(&Identifier::new("x")));
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
pub struct SubmodelTestInputs(HashMap<Identifier, Expr>);

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
    /// use oneil_module::{test::SubmodelTestInputs, expr::{Expr, Literal}, reference::Identifier};
    /// use std::collections::HashMap;
    ///
    /// let mut inputs = HashMap::new();
    /// inputs.insert(Identifier::new("width"), Expr::literal(Literal::number(10.0)));
    /// inputs.insert(Identifier::new("height"), Expr::literal(Literal::number(5.0)));
    ///
    /// let test_inputs = SubmodelTestInputs::new(inputs);
    /// ```
    pub fn new(inputs: HashMap<Identifier, Expr>) -> Self {
        Self(inputs)
    }
}

impl Deref for SubmodelTestInputs {
    type Target = HashMap<Identifier, Expr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
