//! Builder types for constructing model and parameter collections.
//!
//! This module provides builder types that facilitate the construction of model and
//! parameter collections while collecting errors that occur during the building process.
//! The builders allow for incremental construction and error collection, making it
//! easier to handle partial failures gracefully.
//!
//! # Key Types
//!
//! - `ModelCollectionBuilder`: Builds model collections while collecting loading errors
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
    model::{Model, ModelCollection},
    parameter::{Parameter, ParameterCollection},
    reference::{Identifier, ModelPath, PythonPath},
};

use crate::error::{
    CircularDependencyError, LoadError, ParameterResolutionError,
    collection::{ModelErrorMap, ParameterErrorMap},
};

/// A builder for constructing model collections while collecting loading errors.
///
/// This builder facilitates the incremental construction of model collections
/// while collecting various types of errors that can occur during the loading
/// process. It tracks visited models to prevent duplicate loading and provides
/// methods for adding models and different types of errors.
///
/// # Error Types
///
/// - **Model errors**: Parse and resolution errors for specific models
/// - **Circular dependency errors**: Detected circular dependencies
/// - **Import errors**: Python import validation errors
///
#[derive(Debug, Clone, PartialEq)]
pub struct ModelCollectionBuilder<Ps, Py> {
    initial_models: HashSet<ModelPath>,
    models: HashMap<ModelPath, Model>,
    visited_models: HashSet<ModelPath>,
    errors: ModelErrorMap<Ps, Py>,
}

impl<Ps, Py> ModelCollectionBuilder<Ps, Py> {
    /// Creates a new model collection builder.
    ///
    /// # Arguments
    ///
    /// * `initial_models` - The set of initial model paths that should be loaded
    ///
    /// # Returns
    ///
    /// A new `ModelCollectionBuilder` with the specified initial models.
    pub fn new(initial_models: HashSet<ModelPath>) -> Self {
        Self {
            initial_models,
            models: HashMap::new(),
            visited_models: HashSet::new(),
            errors: ModelErrorMap::new(),
        }
    }

    /// Checks if a model has already been visited during loading.
    ///
    /// This method is used to prevent loading the same model multiple times,
    /// which is important for both performance and circular dependency detection.
    ///
    /// # Arguments
    ///
    /// * `model_path` - The path of the model to check
    ///
    /// # Returns
    ///
    /// Returns `true` if the model has been visited, `false` otherwise.
    pub fn model_has_been_visited(&self, model_path: &ModelPath) -> bool {
        self.visited_models.contains(model_path)
    }

    /// Marks a model as visited during loading.
    ///
    /// This method should be called when a model is about to be processed to
    /// prevent it from being loaded again if it's referenced by other models.
    ///
    /// # Arguments
    ///
    /// * `model_path` - The path of the model to mark as visited
    pub fn mark_model_as_visited(&mut self, model_path: &ModelPath) {
        self.visited_models.insert(model_path.clone());
    }

    /// Returns a reference to the map of loaded models.
    ///
    /// # Returns
    ///
    /// A reference to the map of model paths to loaded models.
    pub fn get_models(&self) -> &HashMap<ModelPath, Model> {
        &self.models
    }

    /// Returns a set of model paths that have errors.
    ///
    /// This includes models with parse/resolution errors and models with circular
    /// dependency errors.
    ///
    /// # Returns
    ///
    /// A set of model paths that have any type of error.
    pub fn get_models_with_errors(&self) -> HashSet<&ModelPath> {
        self.errors.get_models_with_errors()
    }

    /// Adds a model error for the specified model.
    ///
    /// # Arguments
    ///
    /// * `model_path` - The path of the model that has an error
    /// * `error` - The loading error that occurred
    pub fn add_model_error(&mut self, model_path: ModelPath, error: LoadError<Ps>) {
        self.errors.add_model_error(model_path, error);
    }

    /// Adds a circular dependency error for the specified model.
    ///
    /// # Arguments
    ///
    /// * `model_path` - The path of the model that has a circular dependency
    /// * `circular_dependency` - The circular dependency path
    pub fn add_circular_dependency_error(
        &mut self,
        model_path: ModelPath,
        circular_dependency: Vec<ModelPath>,
    ) {
        self.errors.add_circular_dependency_error(
            model_path,
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

    /// Adds a successfully loaded model to the collection.
    ///
    /// # Arguments
    ///
    /// * `model_path` - The path of the model
    /// * `model` - The loaded model
    pub fn add_model(&mut self, model_path: ModelPath, model: Model) {
        self.models.insert(model_path, model);
    }

    #[cfg(test)]
    pub fn get_imports_with_errors(&self) -> HashSet<&PythonPath> {
        self.errors.get_imports_with_errors()
    }

    #[cfg(test)]
    pub fn get_model_errors(&self) -> &HashMap<ModelPath, LoadError<Ps>> {
        self.errors.get_model_errors()
    }

    #[cfg(test)]
    pub fn get_circular_dependency_errors(
        &self,
    ) -> &HashMap<ModelPath, Vec<CircularDependencyError>> {
        self.errors.get_circular_dependency_errors()
    }
}

impl<Ps, Py> TryInto<ModelCollection> for ModelCollectionBuilder<Ps, Py> {
    type Error = (ModelCollection, ModelErrorMap<Ps, Py>);

    /// Attempts to convert the builder into a model collection.
    ///
    /// If there are no errors, returns `Ok(ModelCollection)`. If there are errors,
    /// returns `Err((ModelCollection, ModelErrorMap))` where the collection contains
    /// all successfully loaded models and the error map contains all collected errors.
    ///
    /// # Returns
    ///
    /// Returns `Ok(collection)` if no errors occurred, or `Err((partial_collection, errors))`
    /// if there were errors during loading.
    fn try_into(self) -> Result<ModelCollection, (ModelCollection, ModelErrorMap<Ps, Py>)> {
        let model_collection = ModelCollection::new(self.initial_models, self.models);
        if self.errors.is_empty() {
            Ok(model_collection)
        } else {
            Err((model_collection, self.errors))
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
    pub fn add_error_list<I>(&mut self, identifier: &Identifier, errors: I)
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
