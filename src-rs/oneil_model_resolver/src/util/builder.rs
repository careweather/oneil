//! Builder types for constructing model and parameter collections.

use std::collections::{HashMap, HashSet};

use oneil_ir as ir;
use oneil_shared::span::Span;

use crate::{
    error::{
        CircularDependencyError, LoadError, ModelImportResolutionError, ParameterResolutionError,
        collection::ModelErrorMap,
    },
    util::{ReferenceMap, ReferenceResolutionErrors, SubmodelMap, SubmodelResolutionErrors},
};

/// A builder for constructing model collections while collecting loading errors.
///
/// This builder facilitates the incremental construction of model collections
/// while collecting various types of errors that can occur during the loading
/// process. It tracks visited models to prevent duplicate loading and provides
/// methods for adding models and different types of errors.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelCollectionBuilder<Ps, Py> {
    initial_models: HashSet<ir::ModelPath>,
    models: HashMap<ir::ModelPath, ir::Model>,
    visited_models: HashSet<ir::ModelPath>,
    errors: ModelErrorMap<Ps, Py>,
}

impl<Ps, Py> ModelCollectionBuilder<Ps, Py> {
    /// Creates a new model collection builder.
    pub fn new(initial_models: HashSet<ir::ModelPath>) -> Self {
        Self {
            initial_models,
            models: HashMap::new(),
            visited_models: HashSet::new(),
            errors: ModelErrorMap::new(),
        }
    }

    /// Checks if a model has already been visited during loading.
    pub fn model_has_been_visited(&self, model_path: &ir::ModelPath) -> bool {
        self.visited_models.contains(model_path)
    }

    /// Marks a model as visited during loading.
    pub fn mark_model_as_visited(&mut self, model_path: &ir::ModelPath) {
        self.visited_models.insert(model_path.clone());
    }

    /// Returns a reference to the map of loaded models.
    #[must_use]
    pub const fn get_models(&self) -> &HashMap<ir::ModelPath, ir::Model> {
        &self.models
    }

    /// Returns a set of model paths that have errors.
    ///
    /// This includes models with parse/resolution errors and models with circular
    /// dependency errors.
    #[must_use]
    pub fn get_models_with_errors(&self) -> HashSet<&ir::ModelPath> {
        self.errors.get_models_with_errors()
    }

    /// Adds a model error for the specified model.
    pub fn add_model_error(&mut self, model_path: ir::ModelPath, error: LoadError<Ps>) {
        self.errors.add_model_error(model_path, error);
    }

    /// Adds a circular dependency error for the specified model.
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
    pub fn add_import_error(&mut self, python_path: ir::PythonPath, error: Py) {
        self.errors.add_import_error(python_path, error);
    }

    /// Adds a successfully loaded model to the collection.
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
        submodel_name: ir::SubmodelName,
        submodel_name_span: Span,
        submodel_path: ir::ModelPath,
    ) {
        let submodel_ident = submodel_name.clone();
        let submodel_import =
            ir::SubmodelImport::new(submodel_name, submodel_name_span, submodel_path);
        self.submodels.insert(submodel_ident, submodel_import);
    }

    pub fn add_reference(
        &mut self,
        reference_name: ir::ReferenceName,
        reference_name_span: Span,
        reference_path: ir::ModelPath,
    ) {
        let reference_ident = reference_name.clone();
        let reference_import =
            ir::ReferenceImport::new(reference_name, reference_name_span, reference_path);
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
        SubmodelMap,
        ReferenceMap,
        SubmodelResolutionErrors,
        ReferenceResolutionErrors,
    ) {
        (
            self.submodels,
            self.references,
            self.submodel_resolution_errors,
            self.reference_resolution_errors,
        )
    }
}

pub struct ParameterBuilder {
    parameters: HashMap<ir::ParameterName, ir::Parameter>,
    parameter_errors: HashMap<ir::ParameterName, Vec<ParameterResolutionError>>,
    visited: HashSet<ir::ParameterName>,
}

impl ParameterBuilder {
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
            parameter_errors: HashMap::new(),
            visited: HashSet::new(),
        }
    }

    pub fn add_parameter(&mut self, parameter_name: ir::ParameterName, parameter: ir::Parameter) {
        self.parameters.insert(parameter_name, parameter);
    }

    pub const fn get_parameters(&self) -> &HashMap<ir::ParameterName, ir::Parameter> {
        &self.parameters
    }

    pub fn add_parameter_error(
        &mut self,
        parameter_name: ir::ParameterName,
        error: ParameterResolutionError,
    ) {
        self.parameter_errors
            .entry(parameter_name)
            .or_default()
            .push(error);
    }

    pub const fn get_parameter_errors(
        &self,
    ) -> &HashMap<ir::ParameterName, Vec<ParameterResolutionError>> {
        &self.parameter_errors
    }

    pub fn mark_as_visited(&mut self, parameter_name: ir::ParameterName) {
        self.visited.insert(parameter_name);
    }

    pub fn has_visited(&self, parameter_name: &ir::ParameterName) -> bool {
        self.visited.contains(parameter_name)
    }

    pub fn into_parameter_collection_and_errors(
        self,
    ) -> (
        HashMap<ir::ParameterName, ir::Parameter>,
        HashMap<ir::ParameterName, Vec<ParameterResolutionError>>,
    ) {
        (self.parameters, self.parameter_errors)
    }
}
