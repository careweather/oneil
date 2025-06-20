use oneil_module::ModulePath;
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolutionError {}

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
}
