//! Traits for processing model components during traversal.
//!
//! This module defines the traits that must be implemented to process each component
//! (python imports, submodels, parameters, model tests, submodel tests) during model traversal.

use oneil_ir::{
    parameter::Parameter,
    reference::{Identifier, ModelPath, PythonPath},
    test::{ModelTest, SubmodelTest, TestIndex},
};

/// Trait for processing python imports during model traversal.
///
/// Implementations of this trait define how python imports should be processed
/// when encountered during model traversal. The processor can return either
/// successful output data or an error.
pub trait PythonImportProcess {
    /// The type of output data produced by successful processing.
    type Output;
    /// The type of error that can occur during processing.
    type Error;

    /// Processes a python import path.
    ///
    /// # Arguments
    ///
    /// * `import_path` - The path to the python module being imported
    ///
    /// # Returns
    ///
    /// Returns `Ok(output)` if processing succeeds, or `Err(error)` if it fails.
    fn process(&self, import_path: &PythonPath) -> Result<Self::Output, Self::Error>;
}

/// A default implementation for when we don't care about python imports
impl PythonImportProcess for () {
    type Output = ();
    type Error = ();

    fn process(&self, _import_path: &PythonPath) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

/// Trait for processing submodels during model traversal.
///
/// Implementations of this trait define how submodels should be processed
/// when encountered during model traversal. The processor can return either
/// successful output data or an error.
pub trait SubmodelProcess {
    /// The type of output data produced by successful processing.
    type Output;
    /// The type of error that can occur during processing.
    type Error;

    /// Processes a submodel.
    ///
    /// # Arguments
    ///
    /// * `submodel_id` - The identifier of the submodel
    /// * `submodel_path` - The path to the submodel
    ///
    /// # Returns
    ///
    /// Returns `Ok(output)` if processing succeeds, or `Err(error)` if it fails.
    fn process(
        &self,
        submodel_id: &Identifier,
        submodel_path: &ModelPath,
    ) -> Result<Self::Output, Self::Error>;
}

/// A default implementation for when we don't care about submodels
impl SubmodelProcess for () {
    type Output = ();
    type Error = ();

    fn process(
        &self,
        _submodel_id: &Identifier,
        _submodel_path: &ModelPath,
    ) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

/// Trait for processing parameters during model traversal.
///
/// Implementations of this trait define how parameters should be processed
/// when encountered during model traversal. The processor can return either
/// successful output data or an error.
pub trait ParameterProcess {
    /// The type of output data produced by successful processing.
    type Output;
    /// The type of error that can occur during processing.
    type Error;

    /// Processes a parameter.
    ///
    /// # Arguments
    ///
    /// * `parameter` - The parameter to process
    ///
    /// # Returns
    ///
    /// Returns `Ok(output)` if processing succeeds, or `Err(error)` if it fails.
    fn process(&self, parameter: &Parameter) -> Result<Self::Output, Self::Error>;
}

/// A default implementation for when we don't care about parameters
impl ParameterProcess for () {
    type Output = ();
    type Error = ();

    fn process(&self, _parameter: &Parameter) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

/// Trait for processing model tests during model traversal.
///
/// Implementations of this trait define how model tests should be processed
/// when encountered during model traversal. The processor can return either
/// successful output data or an error.
pub trait ModelTestProcess {
    /// The type of output data produced by successful processing.
    type Output;
    /// The type of error that can occur during processing.
    type Error;

    /// Processes a model test.
    ///
    /// # Arguments
    ///
    /// * `test_index` - The index of the test
    /// * `model_test` - The model test to process
    ///
    /// # Returns
    ///
    /// Returns `Ok(output)` if processing succeeds, or `Err(error)` if it fails.
    fn process(
        &self,
        test_index: &TestIndex,
        model_test: &ModelTest,
    ) -> Result<Self::Output, Self::Error>;
}

/// A default implementation for when we don't care about model tests
impl ModelTestProcess for () {
    type Output = ();
    type Error = ();

    fn process(
        &self,
        _test_index: &TestIndex,
        _model_test: &ModelTest,
    ) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

/// Trait for processing submodel tests during model traversal.
///
/// Implementations of this trait define how submodel tests should be processed
/// when encountered during model traversal. The processor can return either
/// successful output data or an error.
pub trait SubmodelTestProcess {
    /// The type of output data produced by successful processing.
    type Output;
    /// The type of error that can occur during processing.
    type Error;

    /// Processes a submodel test.
    ///
    /// # Arguments
    ///
    /// * `submodel_test` - The submodel test to process
    ///
    /// # Returns
    ///
    /// Returns `Ok(output)` if processing succeeds, or `Err(error)` if it fails.
    fn process(&self, submodel_test: &SubmodelTest) -> Result<Self::Output, Self::Error>;
}

/// A default implementation for when we don't care about submodel tests
impl SubmodelTestProcess for () {
    type Output = ();
    type Error = ();

    fn process(&self, _submodel_test: &SubmodelTest) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}
