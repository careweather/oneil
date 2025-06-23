use oneil_module::{ModulePath, PythonPath};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleLoaderError<E> {
    ParseError(E),
    CyclicalDependency(Vec<ModulePath>),
    ResolutionError(ResolutionError),
}

impl<E> ModuleLoaderError<E> {
    pub fn parse_error(e: E) -> Self {
        Self::ParseError(e)
    }

    pub fn cyclical_dependency(deps: Vec<ModulePath>) -> Self {
        Self::CyclicalDependency(deps)
    }

    pub fn resolution_error(e: ResolutionError) -> Self {
        Self::ResolutionError(e)
    }

    pub fn resolution_error_from_python_path(path: PythonPath) -> Self {
        Self::ResolutionError(ResolutionError::python_file_not_found(path))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolutionError {
    PythonFileNotFound(PythonPath),
}

impl ResolutionError {
    pub fn python_file_not_found(path: PythonPath) -> Self {
        Self::PythonFileNotFound(path)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleErrorCollection<E> {
    module_errors: HashMap<ModulePath, ModuleLoaderError<E>>,
}

impl<E> ModuleErrorCollection<E> {
    pub fn new() -> Self {
        Self {
            module_errors: HashMap::new(),
        }
    }

    pub fn add_error(&mut self, module_path: &ModulePath, error: ModuleLoaderError<E>) {
        self.module_errors.insert(module_path.clone(), error);
    }

    pub fn has_error_for(&self, module_path: &ModulePath) -> bool {
        self.module_errors.contains_key(module_path)
    }

    pub fn is_empty(&self) -> bool {
        self.module_errors.is_empty()
    }
}
