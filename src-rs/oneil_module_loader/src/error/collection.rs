use std::collections::{HashMap, HashSet};

use oneil_module::reference::{Identifier, ModulePath, PythonPath};

use crate::error::{CircularDependencyError, LoadError, ParameterResolutionError};

// note that circular dependency errors are stored seperately from module errors
// since circular dependencies are discovered before the module is resolved, and
// returning them back up the loading stack would require a lot of extra work
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleErrorMap<Ps, Py> {
    module_errors: HashMap<ModulePath, LoadError<Ps>>,
    circular_dependency_errors: HashMap<ModulePath, Vec<CircularDependencyError>>,
    import_errors: HashMap<PythonPath, Py>,
}

impl<Ps, Py> ModuleErrorMap<Ps, Py> {
    pub fn new() -> Self {
        Self {
            module_errors: HashMap::new(),
            circular_dependency_errors: HashMap::new(),
            import_errors: HashMap::new(),
        }
    }

    pub fn add_module_error(&mut self, module_path: ModulePath, error: LoadError<Ps>) {
        assert!(!self.module_errors.contains_key(&module_path));
        self.module_errors.insert(module_path, error);
    }

    pub fn add_circular_dependency_error(
        &mut self,
        module_path: ModulePath,
        circular_dependency: CircularDependencyError,
    ) {
        self.circular_dependency_errors
            .entry(module_path)
            .or_insert(vec![])
            .push(circular_dependency);
    }

    pub fn add_parse_error(&mut self, module_path: ModulePath, error: Ps) {
        self.add_module_error(module_path, LoadError::ParseError(error));
    }

    pub fn add_import_error(&mut self, python_path: PythonPath, error: Py) {
        assert!(!self.import_errors.contains_key(&python_path));
        self.import_errors.insert(python_path, error);
    }

    pub fn get_modules_with_errors(&self) -> HashSet<&ModulePath> {
        self.module_errors
            .keys()
            .chain(self.circular_dependency_errors.keys())
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.module_errors.is_empty()
    }

    #[cfg(test)]
    pub fn get_imports_with_errors(&self) -> HashSet<&PythonPath> {
        self.import_errors.keys().collect()
    }

    #[cfg(test)]
    pub fn get_module_errors(&self) -> &HashMap<ModulePath, LoadError<Ps>> {
        &self.module_errors
    }

    #[cfg(test)]
    pub fn get_circular_dependency_errors(
        &self,
    ) -> &HashMap<ModulePath, Vec<CircularDependencyError>> {
        &self.circular_dependency_errors
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParameterErrorMap {
    errors: HashMap<Identifier, Vec<ParameterResolutionError>>,
}

impl<Ps, Py> From<ModuleErrorMap<Ps, Py>>
    for (
        HashMap<ModulePath, (Option<LoadError<Ps>>, Option<Vec<CircularDependencyError>>)>,
        HashMap<PythonPath, Py>,
    )
{
    fn from(mut error: ModuleErrorMap<Ps, Py>) -> Self {
        let mut all_module_errors = HashMap::new();

        // remove module errors and corresponding circular dependency errors, if
        // any exists
        error
            .module_errors
            .into_iter()
            .for_each(|(module_path, load_error)| {
                let circular_dependency_errors =
                    error.circular_dependency_errors.remove(&module_path);

                all_module_errors
                    .insert(module_path, (Some(load_error), circular_dependency_errors));
            });

        // remove any remaining circular dependency errors
        error.circular_dependency_errors.into_iter().for_each(
            |(module_path, circular_dependency_errors)| {
                all_module_errors.insert(module_path, (None, Some(circular_dependency_errors)));
            },
        );

        (all_module_errors, error.import_errors)
    }
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
