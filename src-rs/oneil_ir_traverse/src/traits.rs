use oneil_ir::{
    parameter::Parameter,
    reference::{Identifier, ModelPath, PythonPath},
    test::{ModelTest, SubmodelTest, TestIndex},
};

// X is generic because we want to be able to use the () implementation with any
// extra context
pub trait PythonImportProcess<X> {
    type Output;
    type Error;

    fn process(
        &self,
        import_path: &PythonPath,
        extra_context: &X,
    ) -> Result<Self::Output, Self::Error>;
}

/// A default implementation for when we don't care about python imports
impl<X> PythonImportProcess<X> for () {
    type Output = ();
    type Error = ();

    fn process(
        &self,
        _import_path: &PythonPath,
        _extra_context: &X,
    ) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

pub trait SubmodelProcess<X> {
    type Output;
    type Error;

    fn process(
        &self,
        submodel_id: &Identifier,
        submodel_path: &ModelPath,
        extra_context: &X,
    ) -> Result<Self::Output, Self::Error>;
}

/// A default implementation for when we don't care about submodels
impl<X> SubmodelProcess<X> for () {
    type Output = ();
    type Error = ();

    fn process(
        &self,
        _submodel_id: &Identifier,
        _submodel_path: &ModelPath,
        _extra_context: &X,
    ) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

pub trait ParameterProcess<X> {
    type Output;
    type Error;

    fn process(
        &self,
        parameter: &Parameter,
        extra_context: &X,
    ) -> Result<Self::Output, Self::Error>;
}

/// A default implementation for when we don't care about parameters
impl<X> ParameterProcess<X> for () {
    type Output = ();
    type Error = ();

    fn process(
        &self,
        _parameter: &Parameter,
        _extra_context: &X,
    ) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

pub trait ModelTestProcess<X> {
    type Output;
    type Error;

    fn process(
        &self,
        test_index: &TestIndex,
        model_test: &ModelTest,
        extra_context: &X,
    ) -> Result<Self::Output, Self::Error>;
}

/// A default implementation for when we don't care about model tests
impl<X> ModelTestProcess<X> for () {
    type Output = ();
    type Error = ();

    fn process(
        &self,
        _test_index: &TestIndex,
        _model_test: &ModelTest,
        _extra_context: &X,
    ) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

pub trait SubmodelTestProcess<X> {
    type Output;
    type Error;

    fn process(&self, input: &SubmodelTest, extra_context: &X)
    -> Result<Self::Output, Self::Error>;
}

/// A default implementation for when we don't care about submodel tests
impl<X> SubmodelTestProcess<X> for () {
    type Output = ();
    type Error = ();

    fn process(
        &self,
        _input: &SubmodelTest,
        _extra_context: &X,
    ) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}
