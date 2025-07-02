use oneil_module::reference::{Identifier, ModulePath};

pub mod collection;

#[derive(Debug, Clone, PartialEq)]
pub enum LoadError<Ps, Py> {
    ModuleError(ModuleError),
    ResolutionError(ResolutionError),
    ParseError(Ps),
    PythonError(Py),
}

impl<Ps, Py> LoadError<Ps, Py> {
    pub fn module_circular_dependency(circular_dependency: Vec<ModulePath>) -> Self {
        Self::ModuleError(ModuleError::CircularDependency(circular_dependency))
    }

    pub fn parse_error(parse_error: Ps) -> Self {
        Self::ParseError(parse_error)
    }

    pub fn python_error(python_error: Py) -> Self {
        Self::PythonError(python_error)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleError {
    CircularDependency(Vec<ModulePath>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolutionError {
    pub identifier: Identifier,
    pub error: ResolutionErrorSource,
}

impl ResolutionError {
    pub fn new(identifier: Identifier, error: ResolutionErrorSource) -> Self {
        Self { identifier, error }
    }
}

impl<Ps, Py> From<ResolutionError> for LoadError<Ps, Py> {
    fn from(error: ResolutionError) -> Self {
        Self::ResolutionError(error)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolutionErrorSource {
    SubmodelHadError(ModulePath),
}

impl ResolutionErrorSource {
    pub fn submodel_had_error(submodel_path: ModulePath) -> Self {
        Self::SubmodelHadError(submodel_path)
    }
}
