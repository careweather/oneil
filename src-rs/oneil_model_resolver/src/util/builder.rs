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

use oneil_ir as ir;

use crate::error::{
    CircularDependencyError, LoadError, ModelImportResolutionError, ParameterResolutionError,
    collection::ModelErrorMap,
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
    initial_models: HashSet<ir::ModelPath>,
    models: HashMap<ir::ModelPath, ir::Model>,
    visited_models: HashSet<ir::ModelPath>,
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
    pub fn new(initial_models: HashSet<ir::ModelPath>) -> Self {
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
    pub fn model_has_been_visited(&self, model_path: &ir::ModelPath) -> bool {
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
    pub fn mark_model_as_visited(&mut self, model_path: &ir::ModelPath) {
        self.visited_models.insert(model_path.clone());
    }

    /// Returns a reference to the map of loaded models.
    ///
    /// # Returns
    ///
    /// A reference to the map of model paths to loaded models.
    #[must_use]
    pub const fn get_models(&self) -> &HashMap<ir::ModelPath, ir::Model> {
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
    #[must_use]
    pub fn get_models_with_errors(&self) -> HashSet<&ir::ModelPath> {
        self.errors.get_models_with_errors()
    }

    /// Adds a model error for the specified model.
    ///
    /// # Arguments
    ///
    /// * `model_path` - The path of the model that has an error
    /// * `error` - The loading error that occurred
    pub fn add_model_error(&mut self, model_path: ir::ModelPath, error: LoadError<Ps>) {
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
        model_path: ir::ModelPath,
        circular_dependency: Vec<ir::ModelPath>,
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
    pub fn add_import_error(&mut self, python_path: ir::PythonPath, error: Py) {
        self.errors.add_import_error(python_path, error);
    }

    /// Adds a successfully loaded model to the collection.
    ///
    /// # Arguments
    ///
    /// * `model_path` - The path of the model
    /// * `model` - The loaded model
    pub fn add_model(&mut self, model_path: ir::ModelPath, model: ir::Model) {
        self.models.insert(model_path, model);
    }

    #[cfg(test)]
    pub fn get_imports_with_errors(&self) -> HashSet<&ir::PythonPath> {
        self.errors.get_imports_with_errors()
    }

    #[cfg(test)]
    pub const fn get_model_errors(&self) -> &HashMap<ir::ModelPath, LoadError<Ps>> {
        self.errors.get_model_errors()
    }

    #[cfg(test)]
    pub const fn get_circular_dependency_errors(
        &self,
    ) -> &HashMap<ir::ModelPath, Vec<CircularDependencyError>> {
        self.errors.get_circular_dependency_errors()
    }
}

impl<Ps, Py> TryInto<ir::ModelCollection> for ModelCollectionBuilder<Ps, Py> {
    type Error = (ir::ModelCollection, ModelErrorMap<Ps, Py>);

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
    fn try_into(self) -> Result<ir::ModelCollection, (ir::ModelCollection, ModelErrorMap<Ps, Py>)> {
        let model_collection = ir::ModelCollection::new(self.initial_models, self.models);
        if self.errors.is_empty() {
            Ok(model_collection)
        } else {
            Err((model_collection, self.errors))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelImportsBuilder {
    submodels: HashMap<ir::SubmodelName, ir::SubmodelImport>,
    submodel_resolution_errors: HashMap<ir::SubmodelName, ModelImportResolutionError>,
    references: HashMap<ir::ReferenceName, ir::ReferenceImport>,
    reference_resolution_errors: HashMap<ir::ReferenceName, ModelImportResolutionError>,
}

impl ModelImportsBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            submodels: HashMap::new(),
            submodel_resolution_errors: HashMap::new(),
            references: HashMap::new(),
            reference_resolution_errors: HashMap::new(),
        }
    }

    #[must_use]
    pub fn get_submodel(&self, submodel_name: &ir::SubmodelName) -> Option<&ir::SubmodelImport> {
        self.submodels.get(submodel_name)
    }

    #[must_use]
    pub fn get_reference(
        &self,
        reference_name: &ir::ReferenceName,
    ) -> Option<&ir::ReferenceImport> {
        self.references.get(reference_name)
    }

    pub fn add_submodel(
        &mut self,
        submodel_name: ir::SubmodelNameWithSpan,
        submodel_path: ir::ModelPath,
    ) {
        let submodel_ident = submodel_name.value().clone();
        let submodel_import = ir::SubmodelImport::new(submodel_name, submodel_path);
        self.submodels.insert(submodel_ident, submodel_import);
    }

    pub fn add_reference(
        &mut self,
        reference_name: ir::ReferenceNameWithSpan,
        reference_path: ir::ModelPath,
    ) {
        let reference_ident = reference_name.value().clone();
        let reference_import = ir::ReferenceImport::new(reference_name, reference_path);
        self.references.insert(reference_ident, reference_import);
    }

    pub fn add_submodel_resolution_error(
        &mut self,
        submodel_name: ir::SubmodelName,
        error: ModelImportResolutionError,
    ) {
        self.submodel_resolution_errors.insert(submodel_name, error);
    }

    pub fn add_reference_resolution_error(
        &mut self,
        reference_name: ir::ReferenceName,
        error: ModelImportResolutionError,
    ) {
        self.reference_resolution_errors
            .insert(reference_name, error);
    }

    pub fn into_submodels_and_references_and_resolution_errors(
        self,
    ) -> (
        ir::SubmodelMap,
        ir::ReferenceMap,
        HashMap<ir::SubmodelName, ModelImportResolutionError>,
        HashMap<ir::ReferenceName, ModelImportResolutionError>,
    ) {
        (
            ir::SubmodelMap::new(self.submodels),
            ir::ReferenceMap::new(self.references),
            self.submodel_resolution_errors,
            self.reference_resolution_errors,
        )
    }
}

pub struct ParameterBuilder {
    parameters: HashMap<ir::Identifier, ir::Parameter>,
    parameter_errors: HashMap<ir::Identifier, Vec<ParameterResolutionError>>,
    visited: HashSet<ir::Identifier>,
}

impl ParameterBuilder {
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
            parameter_errors: HashMap::new(),
            visited: HashSet::new(),
        }
    }

    pub fn add_parameter(&mut self, identifier: ir::Identifier, parameter: ir::Parameter) {
        self.parameters.insert(identifier, parameter);
    }

    pub const fn get_parameters(&self) -> &HashMap<ir::Identifier, ir::Parameter> {
        &self.parameters
    }

    pub fn add_parameter_error(
        &mut self,
        identifier: ir::Identifier,
        error: ParameterResolutionError,
    ) {
        self.parameter_errors
            .entry(identifier)
            .or_default()
            .push(error);
    }

    pub const fn get_parameter_errors(
        &self,
    ) -> &HashMap<ir::Identifier, Vec<ParameterResolutionError>> {
        &self.parameter_errors
    }

    pub fn mark_as_visited(&mut self, identifier: ir::Identifier) {
        self.visited.insert(identifier);
    }

    pub fn has_visited(&self, identifier: &ir::Identifier) -> bool {
        self.visited.contains(identifier)
    }

    pub fn into_parameter_collection_and_errors(
        self,
    ) -> (
        ir::ParameterCollection,
        HashMap<ir::Identifier, Vec<ParameterResolutionError>>,
    ) {
        (
            ir::ParameterCollection::new(self.parameters),
            self.parameter_errors,
        )
    }
}
