use std::collections::{HashMap, HashSet};

use oneil_module::reference::{Identifier, ModulePath, PythonPath};

use crate::error::{LoadError, ParameterResolutionError};

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleErrorMap<Ps, Py> {
    errors: HashMap<ModulePath, LoadError<Ps>>,
    import_errors: HashMap<PythonPath, Py>,
}

impl<Ps, Py> ModuleErrorMap<Ps, Py> {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
            import_errors: HashMap::new(),
        }
    }

    pub fn add_error(&mut self, module_path: ModulePath, error: LoadError<Ps>) {
        self.errors.insert(module_path, error);
    }

    pub fn add_import_error(&mut self, python_path: PythonPath, error: Py) {
        self.import_errors.insert(python_path, error);
    }

    pub fn get_modules_with_errors(&self) -> HashSet<&ModulePath> {
        self.errors.keys().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    #[cfg(test)]
    pub fn get_imports_with_errors(&self) -> HashSet<&PythonPath> {
        self.import_errors.keys().collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParameterErrorMap {
    errors: HashMap<Identifier, Vec<ParameterResolutionError>>,
}

impl ParameterErrorMap {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    pub fn add_error(&mut self, identifier: Identifier, error: ParameterResolutionError) {
        self.errors.entry(identifier).or_insert(vec![]).push(error);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn get_parameters_with_errors(&self) -> HashSet<&Identifier> {
        self.errors.keys().collect()
    }
}

impl From<ParameterErrorMap> for HashMap<Identifier, Vec<ParameterResolutionError>> {
    fn from(error: ParameterErrorMap) -> Self {
        error.errors
    }
}
