//! Error collection and management for module loading.
//!
//! This module provides data structures for collecting and managing errors that occur
//! during the module loading process. It supports collecting errors from multiple
//! modules and different error types, allowing for comprehensive error reporting.
//!
//! # Key Types
//!
//! - `ModuleErrorMap`: Collects errors for multiple modules, including parse errors,
//!   resolution errors, circular dependency errors, and Python import errors
//! - `ParameterErrorMap`: Collects parameter resolution errors for a single module
//!
//! # Error Separation
//!
//! The module separates different types of errors to allow for different handling
//! strategies:
//!
//! - **Module errors**: Parse and resolution errors for specific modules
//! - **Circular dependency errors**: Detected circular dependencies (stored separately
//!   because they're discovered before module resolution)
//! - **Import errors**: Python import validation errors
//! - **Parameter errors**: Parameter resolution errors within a module

use std::collections::{HashMap, HashSet};

use oneil_ir::reference::{Identifier, ModulePath, PythonPath};

use crate::error::{CircularDependencyError, LoadError, ParameterResolutionError};

// note that circular dependency errors are stored seperately from module errors
// since circular dependencies are discovered before the module is resolved, and
// returning them back up the loading stack would require a lot of extra work

/// A collection of errors that occurred during module loading.
///
/// This struct maintains separate collections for different types of errors that can
/// occur during the module loading process. It provides methods for adding errors
/// and querying which modules have errors.
///
/// # Error Types
///
/// - **Module errors**: Parse and resolution errors for specific modules
/// - **Circular dependency errors**: Detected circular dependencies (stored separately
///   because they're discovered before module resolution)
/// - **Import errors**: Python import validation errors
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleErrorMap<Ps, Py> {
    module_errors: HashMap<ModulePath, LoadError<Ps>>,
    circular_dependency_errors: HashMap<ModulePath, Vec<CircularDependencyError>>,
    import_errors: HashMap<PythonPath, Py>,
}

impl<Ps, Py> ModuleErrorMap<Ps, Py> {
    /// Creates a new empty error map.
    ///
    /// # Returns
    ///
    /// A new `ModuleErrorMap` with no errors.
    pub fn new() -> Self {
        Self {
            module_errors: HashMap::new(),
            circular_dependency_errors: HashMap::new(),
            import_errors: HashMap::new(),
        }
    }

    /// Adds a module error for the specified module.
    ///
    /// # Arguments
    ///
    /// * `module_path` - The path of the module that has an error
    /// * `error` - The error that occurred for this module
    ///
    /// # Panics
    ///
    /// Panics if a module error already exists for the given module path.
    /// This ensures that each module can only have one error recorded.
    pub fn add_module_error(&mut self, module_path: ModulePath, error: LoadError<Ps>) {
        assert!(!self.module_errors.contains_key(&module_path));
        self.module_errors.insert(module_path, error);
    }

    /// Adds a circular dependency error for the specified module.
    ///
    /// # Arguments
    ///
    /// * `module_path` - The path of the module that has a circular dependency
    /// * `circular_dependency` - The circular dependency error
    ///
    /// Multiple circular dependency errors can be added for the same module,
    /// as a module might be involved in multiple circular dependency cycles.
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

    /// Adds a parse error for the specified module.
    ///
    /// This is a convenience method that wraps the parse error in a `LoadError::ParseError`.
    ///
    /// # Arguments
    ///
    /// * `module_path` - The path of the module that has a parse error
    /// * `error` - The parse error that occurred
    ///
    /// # Panics
    ///
    /// Panics if a module error already exists for the given module path.
    pub fn add_parse_error(&mut self, module_path: ModulePath, error: Ps) {
        self.add_module_error(module_path, LoadError::ParseError(error));
    }

    /// Adds a Python import error for the specified import.
    ///
    /// # Arguments
    ///
    /// * `python_path` - The Python path that failed to import
    /// * `error` - The import error that occurred
    ///
    /// # Panics
    ///
    /// Panics if an import error already exists for the given Python path.
    pub fn add_import_error(&mut self, python_path: PythonPath, error: Py) {
        assert!(!self.import_errors.contains_key(&python_path));
        self.import_errors.insert(python_path, error);
    }

    /// Returns a set of all module paths that have errors.
    ///
    /// This includes modules with parse/resolution errors and modules with circular
    /// dependency errors.
    ///
    /// # Returns
    ///
    /// A set of module paths that have any type of error.
    pub fn get_modules_with_errors(&self) -> HashSet<&ModulePath> {
        self.module_errors
            .keys()
            .chain(self.circular_dependency_errors.keys())
            .collect()
    }

    /// Returns whether there are any errors in this error map.
    ///
    /// This checks for all types of errors - module errors, circular dependency errors,
    /// and Python import errors.
    ///
    /// # Returns
    ///
    /// `true` if there are no errors of any type, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.module_errors.is_empty()
            && self.circular_dependency_errors.is_empty()
            && self.import_errors.is_empty()
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

/// A collection of parameter resolution errors for a single module.
///
/// This struct collects parameter resolution errors that occur during the resolution
/// phase of module loading. It allows for tracking which parameters have errors and
/// what those errors are.
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
    /// Creates a new empty parameter error map.
    ///
    /// # Returns
    ///
    /// A new `ParameterErrorMap` with no errors.
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    /// Adds a parameter resolution error for the specified parameter.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the parameter that has an error
    /// * `error` - The parameter resolution error that occurred
    ///
    /// Multiple errors can be added for the same parameter, as a parameter might
    /// have multiple resolution issues.
    pub fn add_error(&mut self, identifier: Identifier, error: ParameterResolutionError) {
        self.errors.entry(identifier).or_insert(vec![]).push(error);
    }

    /// Returns whether there are any parameter errors.
    ///
    /// # Returns
    ///
    /// `true` if there are no parameter errors, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns a set of all parameter identifiers that have errors.
    ///
    /// # Returns
    ///
    /// A set of parameter identifiers that have resolution errors.
    pub fn get_parameters_with_errors(&self) -> HashSet<&Identifier> {
        self.errors.keys().collect()
    }
}

impl From<ParameterErrorMap> for HashMap<Identifier, Vec<ParameterResolutionError>> {
    fn from(error: ParameterErrorMap) -> Self {
        error.errors
    }
}
