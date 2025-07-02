use std::collections::{HashMap, HashSet};

use oneil_module::reference::{Identifier, ModulePath};

use crate::error::{LoadError, ResolutionError, ResolutionErrorSource};

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleErrorMap<Ps, Py> {
    errors: HashMap<ModulePath, Vec<LoadError<Ps, Py>>>,
}

impl<Ps, Py> ModuleErrorMap<Ps, Py> {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    pub fn add_error(&mut self, module_path: ModulePath, error: LoadError<Ps, Py>) {
        self.errors.entry(module_path).or_insert(vec![]).push(error);
    }

    pub fn get_modules_with_errors(&self) -> HashSet<&ModulePath> {
        self.errors.keys().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParameterErrorMap {
    errors: HashMap<Identifier, ResolutionErrorSource>,
}

impl ParameterErrorMap {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    pub fn add_error(&mut self, identifier: Identifier, error: ResolutionErrorSource) {
        self.errors.insert(identifier, error);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

impl From<ParameterErrorMap> for Vec<ResolutionError> {
    fn from(error: ParameterErrorMap) -> Self {
        error
            .errors
            .into_iter()
            .map(|(ident, error)| ResolutionError::new(ident, error))
            .collect()
    }
}
