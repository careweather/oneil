//! Builder types for constructing module and parameter collections.
//!
//! This module provides builder types that facilitate the construction of module and
//! parameter collections while collecting errors that occur during the building process.
//! The builders allow for incremental construction and error collection, making it
//! easier to handle partial failures gracefully.
//!
//! # Key Types
//!
//! - `ModuleCollectionBuilder`: Builds module collections while collecting loading errors
//! - `ParameterCollectionBuilder`: Builds parameter collections while collecting resolution errors
//!
//! # Error Handling
//!
//! Both builder types collect errors during the building process and provide methods
//! to query which items have errors. When converting to the final collection type,
//! the builders return either the successful collection or a tuple containing the
//! partial collection and the collected errors.

use std::collections::{HashMap, HashSet};

use oneil_ir::{
    module::{Module, ModuleCollection},
    parameter::{Parameter, ParameterCollection},
    reference::{Identifier, ModulePath, PythonPath},
};

use crate::error::{
    CircularDependencyError, LoadError, ParameterResolutionError,
    collection::{ModuleErrorMap, ParameterErrorMap},
};

/// A builder for constructing module collections while collecting loading errors.
///
/// This builder facilitates the incremental construction of module collections
/// while collecting various types of errors that can occur during the loading
/// process. It tracks visited modules to prevent duplicate loading and provides
/// methods for adding modules and different types of errors.
///
/// # Error Types
///
/// - **Module errors**: Parse and resolution errors for specific modules
/// - **Circular dependency errors**: Detected circular dependencies
/// - **Import errors**: Python import validation errors
///
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleCollectionBuilder<Ps, Py> {
    initial_modules: HashSet<ModulePath>,
    modules: HashMap<ModulePath, Module>,
    visited_modules: HashSet<ModulePath>,
    errors: ModuleErrorMap<Ps, Py>,
}

impl<Ps, Py> ModuleCollectionBuilder<Ps, Py> {
    /// Creates a new module collection builder.
    ///
    /// # Arguments
    ///
    /// * `initial_modules` - The set of initial module paths that should be loaded
    ///
    /// # Returns
    ///
    /// A new `ModuleCollectionBuilder` with the specified initial modules.
    pub fn new(initial_modules: HashSet<ModulePath>) -> Self {
        Self {
            initial_modules,
            modules: HashMap::new(),
            visited_modules: HashSet::new(),
            errors: ModuleErrorMap::new(),
        }
    }

    /// Checks if a module has already been visited during loading.
    ///
    /// This method is used to prevent loading the same module multiple times,
    /// which is important for both performance and circular dependency detection.
    ///
    /// # Arguments
    ///
    /// * `module_path` - The path of the module to check
    ///
    /// # Returns
    ///
    /// Returns `true` if the module has been visited, `false` otherwise.
    pub fn module_has_been_visited(&self, module_path: &ModulePath) -> bool {
        self.visited_modules.contains(module_path)
    }

    /// Marks a module as visited during loading.
    ///
    /// This method should be called when a module is about to be processed to
    /// prevent it from being loaded again if it's referenced by other modules.
    ///
    /// # Arguments
    ///
    /// * `module_path` - The path of the module to mark as visited
    pub fn mark_module_as_visited(&mut self, module_path: &ModulePath) {
        self.visited_modules.insert(module_path.clone());
    }

    /// Returns a reference to the map of loaded modules.
    ///
    /// # Returns
    ///
    /// A reference to the map of module paths to loaded modules.
    pub fn get_modules(&self) -> &HashMap<ModulePath, Module> {
        &self.modules
    }

    /// Returns a set of module paths that have errors.
    ///
    /// This includes modules with parse/resolution errors and modules with circular
    /// dependency errors.
    ///
    /// # Returns
    ///
    /// A set of module paths that have any type of error.
    pub fn get_modules_with_errors(&self) -> HashSet<&ModulePath> {
        self.errors.get_modules_with_errors()
    }

    /// Adds a module error for the specified module.
    ///
    /// # Arguments
    ///
    /// * `module_path` - The path of the module that has an error
    /// * `error` - The loading error that occurred
    pub fn add_module_error(&mut self, module_path: ModulePath, error: LoadError<Ps>) {
        self.errors.add_module_error(module_path, error);
    }

    /// Adds a circular dependency error for the specified module.
    ///
    /// # Arguments
    ///
    /// * `module_path` - The path of the module that has a circular dependency
    /// * `circular_dependency` - The circular dependency path
    pub fn add_circular_dependency_error(
        &mut self,
        module_path: ModulePath,
        circular_dependency: Vec<ModulePath>,
    ) {
        self.errors.add_circular_dependency_error(
            module_path,
            CircularDependencyError::new(circular_dependency),
        );
    }

    /// Adds a Python import error for the specified import.
    ///
    /// # Arguments
    ///
    /// * `python_path` - The Python path that failed to import
    /// * `error` - The import error that occurred
    pub fn add_import_error(&mut self, python_path: PythonPath, error: Py) {
        self.errors.add_import_error(python_path, error);
    }

    /// Adds a successfully loaded module to the collection.
    ///
    /// # Arguments
    ///
    /// * `module_path` - The path of the module
    /// * `module` - The loaded module
    pub fn add_module(&mut self, module_path: ModulePath, module: Module) {
        self.modules.insert(module_path, module);
    }

    #[cfg(test)]
    pub fn get_imports_with_errors(&self) -> HashSet<&PythonPath> {
        self.errors.get_imports_with_errors()
    }

    #[cfg(test)]
    pub fn get_module_errors(&self) -> &HashMap<ModulePath, LoadError<Ps>> {
        self.errors.get_module_errors()
    }

    #[cfg(test)]
    pub fn get_circular_dependency_errors(
        &self,
    ) -> &HashMap<ModulePath, Vec<CircularDependencyError>> {
        self.errors.get_circular_dependency_errors()
    }
}

impl<Ps, Py> TryInto<ModuleCollection> for ModuleCollectionBuilder<Ps, Py> {
    type Error = (ModuleCollection, ModuleErrorMap<Ps, Py>);

    /// Attempts to convert the builder into a module collection.
    ///
    /// If there are no errors, returns `Ok(ModuleCollection)`. If there are errors,
    /// returns `Err((ModuleCollection, ModuleErrorMap))` where the collection contains
    /// all successfully loaded modules and the error map contains all collected errors.
    ///
    /// # Returns
    ///
    /// Returns `Ok(collection)` if no errors occurred, or `Err((partial_collection, errors))`
    /// if there were errors during loading.
    fn try_into(self) -> Result<ModuleCollection, (ModuleCollection, ModuleErrorMap<Ps, Py>)> {
        let module_collection = ModuleCollection::new(self.initial_modules, self.modules);
        if self.errors.is_empty() {
            Ok(module_collection)
        } else {
            Err((module_collection, self.errors))
        }
    }
}

/// A builder for constructing parameter collections while collecting resolution errors.
///
/// This builder facilitates the incremental construction of parameter collections
/// while collecting parameter resolution errors. It provides methods for adding
/// parameters and errors, and can convert to a final `ParameterCollection` or return
/// a partial collection with errors.
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterCollectionBuilder {
    parameters: HashMap<Identifier, Parameter>,
    errors: ParameterErrorMap,
}

impl ParameterCollectionBuilder {
    /// Creates a new parameter collection builder.
    ///
    /// # Returns
    ///
    /// A new `ParameterCollectionBuilder` with no parameters or errors.
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
            errors: ParameterErrorMap::new(),
        }
    }

    /// Adds a parameter to the collection.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the parameter
    /// * `parameter` - The parameter to add
    pub fn add_parameter(&mut self, identifier: Identifier, parameter: Parameter) {
        self.parameters.insert(identifier, parameter);
    }

    /// Adds a parameter resolution error for the specified parameter.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the parameter that has an error
    /// * `error` - The parameter resolution error that occurred
    pub fn add_error(&mut self, identifier: Identifier, error: ParameterResolutionError) {
        self.errors.add_error(identifier, error);
    }

    /// Adds multiple parameter resolution errors for the specified parameter.
    ///
    /// This is a convenience method for adding multiple errors for the same parameter.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the parameter that has errors
    /// * `errors` - An iterator of parameter resolution errors
    pub fn add_error_list<I>(&mut self, identifier: Identifier, errors: I)
    where
        I: IntoIterator<Item = ParameterResolutionError>,
    {
        for error in errors {
            self.add_error(identifier.clone(), error);
        }
    }

    /// Returns a reference to the map of defined parameters.
    ///
    /// # Returns
    ///
    /// A reference to the map of parameter identifiers to parameters.
    pub fn get_defined_parameters(&self) -> &HashMap<Identifier, Parameter> {
        &self.parameters
    }

    /// Returns a set of parameter identifiers that have errors.
    ///
    /// # Returns
    ///
    /// A set of parameter identifiers that have resolution errors.
    pub fn get_parameters_with_errors(&self) -> HashSet<&Identifier> {
        self.errors.get_parameters_with_errors()
    }
}

impl TryInto<ParameterCollection> for ParameterCollectionBuilder {
    type Error = (
        ParameterCollection,
        HashMap<Identifier, Vec<ParameterResolutionError>>,
    );

    /// Attempts to convert the builder into a parameter collection.
    ///
    /// If there are no errors, returns `Ok(ParameterCollection)`. If there are errors,
    /// returns `Err((ParameterCollection, HashMap))` where the collection contains
    /// all successfully resolved parameters and the hash map contains all collected errors.
    ///
    /// # Returns
    ///
    /// Returns `Ok(collection)` if no errors occurred, or `Err((partial_collection, errors))`
    /// if there were errors during resolution.
    fn try_into(
        self,
    ) -> Result<
        ParameterCollection,
        (
            ParameterCollection,
            HashMap<Identifier, Vec<ParameterResolutionError>>,
        ),
    > {
        if self.errors.is_empty() {
            Ok(ParameterCollection::new(self.parameters))
        } else {
            Err((
                ParameterCollection::new(self.parameters),
                self.errors.into(),
            ))
        }
    }
}
