use oneil_module::reference::ModulePath;

pub mod collection;
pub mod resolution;
pub mod util;

pub use resolution::{
    ModelTestResolutionError, ParameterResolutionError, ResolutionErrors, SubmodelResolutionError,
    SubmodelTestInputResolutionError, VariableResolutionError,
};
pub use util::{combine_error_list, combine_errors, convert_errors, split_ok_and_errors};

#[derive(Debug, Clone, PartialEq)]
pub enum LoadError<Ps> {
    ModuleError(ModuleError),
    ResolutionErrors(resolution::ResolutionErrors),
    ParseError(Ps),
}

impl<Ps> LoadError<Ps> {
    pub fn module_circular_dependency(circular_dependency: Vec<ModulePath>) -> Self {
        Self::ModuleError(ModuleError::CircularDependency(circular_dependency))
    }

    pub fn parse_error(parse_error: Ps) -> Self {
        Self::ParseError(parse_error)
    }

    pub fn resolution_errors(resolution_errors: resolution::ResolutionErrors) -> Self {
        Self::ResolutionErrors(resolution_errors)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleError {
    CircularDependency(Vec<ModulePath>),
}
