use oneil_ir::{
    parameter::Parameter,
    reference::{Identifier, ModelPath, PythonPath},
    test::{ModelTest, SubmodelTest, TestIndex},
};

// X is generic because we want to be able to use the () implementation with any
// extra context
pub trait PythonImportProcess {
    type Output;
    type Error;

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

pub trait SubmodelProcess {
    type Output;
    type Error;

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

pub trait ParameterProcess {
    type Output;
    type Error;

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

pub trait ModelTestProcess {
    type Output;
    type Error;

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

pub trait SubmodelTestProcess {
    type Output;
    type Error;

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
