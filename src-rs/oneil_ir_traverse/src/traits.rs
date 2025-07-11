use oneil_ir::{
    parameter::Parameter,
    reference::{Identifier, ModelPath, PythonPath},
    test::{ModelTest, SubmodelTest, TestIndex},
};

pub trait PythonImportProcess {
    type Output;
    type Error;

    fn process(&self, input: PythonPath) -> Result<Self::Output, Self::Error>;
}

/// A default implementation for when we don't care about python imports
impl PythonImportProcess for () {
    type Output = ();
    type Error = ();

    fn process(&self, _input: PythonPath) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

pub trait SubmodelProcess {
    type Output;
    type Error;

    fn process(&self, input: (Identifier, ModelPath)) -> Result<Self::Output, Self::Error>;
}

/// A default implementation for when we don't care about submodels
impl SubmodelProcess for () {
    type Output = ();
    type Error = ();

    fn process(&self, _input: (Identifier, ModelPath)) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

pub trait ParameterProcess {
    type Output;
    type Error;

    fn process(&self, input: (Identifier, Parameter)) -> Result<Self::Output, Self::Error>;
}

/// A default implementation for when we don't care about parameters
impl ParameterProcess for () {
    type Output = ();
    type Error = ();

    fn process(&self, _input: (Identifier, Parameter)) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

pub trait ModelTestProcess {
    type Output;
    type Error;

    fn process(&self, input: (TestIndex, ModelTest)) -> Result<Self::Output, Self::Error>;
}

/// A default implementation for when we don't care about model tests
impl ModelTestProcess for () {
    type Output = ();
    type Error = ();

    fn process(&self, _input: (TestIndex, ModelTest)) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

pub trait SubmodelTestProcess {
    type Output;
    type Error;

    fn process(&self, input: SubmodelTest) -> Result<Self::Output, Self::Error>;
}

/// A default implementation for when we don't care about submodel tests
impl SubmodelTestProcess for () {
    type Output = ();
    type Error = ();

    fn process(&self, _input: SubmodelTest) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}
