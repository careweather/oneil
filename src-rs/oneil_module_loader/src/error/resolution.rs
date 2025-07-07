use std::collections::HashMap;

use oneil_module::{
    reference::{Identifier, ModulePath, PythonPath},
    test::TestIndex,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ResolutionErrors {
    import_errors: HashMap<PythonPath, ImportResolutionError>,
    submodel_resolution_errors: HashMap<Identifier, SubmodelResolutionError>,
    parameter_resolution_errors: HashMap<Identifier, Vec<ParameterResolutionError>>,
    model_test_resolution_errors: HashMap<TestIndex, Vec<ModelTestResolutionError>>,
    submodel_test_resolution_errors: HashMap<Identifier, Vec<SubmodelTestInputResolutionError>>,
}

impl ResolutionErrors {
    pub fn new(
        import_errors: HashMap<PythonPath, ImportResolutionError>,
        submodel_resolution_errors: HashMap<Identifier, SubmodelResolutionError>,
        parameter_resolution_errors: HashMap<Identifier, Vec<ParameterResolutionError>>,
        model_test_resolution_errors: HashMap<TestIndex, Vec<ModelTestResolutionError>>,
        submodel_test_resolution_errors: HashMap<Identifier, Vec<SubmodelTestInputResolutionError>>,
    ) -> Self {
        Self {
            import_errors,
            submodel_resolution_errors,
            parameter_resolution_errors,
            model_test_resolution_errors,
            submodel_test_resolution_errors,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.import_errors.is_empty()
            && self.submodel_resolution_errors.is_empty()
            && self.parameter_resolution_errors.is_empty()
            && self.model_test_resolution_errors.is_empty()
            && self.submodel_test_resolution_errors.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportResolutionError;

impl ImportResolutionError {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubmodelResolutionError {
    ModuleHasError(ModulePath),
    UndefinedSubmodel(Option<ModulePath>, Identifier),
}

impl SubmodelResolutionError {
    pub fn module_has_error(module_path: ModulePath) -> Self {
        Self::ModuleHasError(module_path)
    }

    pub fn undefined_submodel(identifier: Identifier) -> Self {
        Self::UndefinedSubmodel(None, identifier)
    }

    pub fn undefined_submodel_in_submodel(
        parent_module_path: ModulePath,
        identifier: Identifier,
    ) -> Self {
        Self::UndefinedSubmodel(Some(parent_module_path), identifier)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterResolutionError {
    CircularDependency(Vec<Identifier>),
    VariableResolution(VariableResolutionError),
}

impl ParameterResolutionError {
    pub fn circular_dependency(circular_dependency: Vec<Identifier>) -> Self {
        Self::CircularDependency(circular_dependency)
    }

    pub fn variable_resolution(error: VariableResolutionError) -> Self {
        Self::VariableResolution(error)
    }
}

impl From<VariableResolutionError> for ParameterResolutionError {
    fn from(error: VariableResolutionError) -> Self {
        Self::variable_resolution(error)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelTestResolutionError(VariableResolutionError);

impl ModelTestResolutionError {
    pub fn new(error: VariableResolutionError) -> Self {
        Self(error)
    }
}

impl From<VariableResolutionError> for ModelTestResolutionError {
    fn from(error: VariableResolutionError) -> Self {
        Self::new(error)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubmodelTestInputResolutionError {
    VariableResolution(VariableResolutionError),
}

impl SubmodelTestInputResolutionError {
    pub fn variable_resolution(error: VariableResolutionError) -> Self {
        Self::VariableResolution(error)
    }
}

impl From<VariableResolutionError> for SubmodelTestInputResolutionError {
    fn from(error: VariableResolutionError) -> Self {
        Self::variable_resolution(error)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableResolutionError {
    ModuleHasError(ModulePath),
    ParameterHasError(Identifier),
    SubmodelHasError(Identifier),
    UndefinedParameter(Option<ModulePath>, Identifier),
    UndefinedSubmodel(Option<ModulePath>, Identifier),
}

impl VariableResolutionError {
    pub fn module_has_error(module_path: ModulePath) -> Self {
        Self::ModuleHasError(module_path)
    }

    pub fn parameter_has_error(identifier: Identifier) -> Self {
        Self::ParameterHasError(identifier)
    }

    pub fn submodel_has_error(identifier: Identifier) -> Self {
        Self::SubmodelHasError(identifier)
    }

    pub fn undefined_parameter(identifier: Identifier) -> Self {
        Self::UndefinedParameter(None, identifier)
    }

    pub fn undefined_parameter_in_submodel(
        submodel_path: ModulePath,
        identifier: Identifier,
    ) -> Self {
        Self::UndefinedParameter(Some(submodel_path), identifier)
    }

    pub fn undefined_submodel(identifier: Identifier) -> Self {
        Self::UndefinedSubmodel(None, identifier)
    }

    pub fn undefined_submodel_in_submodel(
        parent_module_path: ModulePath,
        identifier: Identifier,
    ) -> Self {
        Self::UndefinedSubmodel(Some(parent_module_path), identifier)
    }
}
