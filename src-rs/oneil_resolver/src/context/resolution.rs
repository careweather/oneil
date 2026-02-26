use indexmap::{IndexMap, IndexSet};
use oneil_ir as ir;
use oneil_shared::span::Span;

use crate::error::{
    CircularDependencyError, ModelImportResolutionError, ParameterResolutionError,
    PythonImportResolutionError, ResolutionErrorCollection, VariableResolutionError,
};

use super::{AstLoadingFailedError, ExternalResolutionContext};

/// Result of resolving one or more models: resolved models and per-model errors.
pub struct ModelResolutionResult {
    /// Resolved model.
    model: ir::Model,
    /// Model resolution errors (including circular dependency errors).
    model_errors: ResolutionErrorCollection,
}

impl ModelResolutionResult {
    /// Creates an empty resolution result with an empty model and
    /// no resolution or circular dependency errors.
    #[must_use]
    pub fn new(model_path: ir::ModelPath) -> Self {
        let empty_model = ir::Model::new(
            model_path,
            IndexMap::new(),
            IndexMap::new(),
            IndexMap::new(),
            IndexMap::new(),
            IndexMap::new(),
        );

        Self {
            model: empty_model,
            model_errors: ResolutionErrorCollection::empty(),
        }
    }

    /// Returns a reference to the resolved model.
    #[must_use]
    pub const fn model(&self) -> &ir::Model {
        &self.model
    }

    /// Returns a mutable reference to the resolved model.
    pub const fn model_mut(&mut self) -> &mut ir::Model {
        &mut self.model
    }

    /// Returns a reference to the model resolution errors.
    #[must_use]
    pub const fn model_errors(&self) -> &ResolutionErrorCollection {
        &self.model_errors
    }

    /// Returns a mutable reference to the model resolution errors.
    pub const fn model_errors_mut(&mut self) -> &mut ResolutionErrorCollection {
        &mut self.model_errors
    }

    /// Breaks the result into its components.
    #[must_use]
    pub fn into_parts(self) -> (ir::Model, ResolutionErrorCollection) {
        (self.model, self.model_errors)
    }
}

/// In-memory context used while resolving one or more models.
pub struct ResolutionContext<'external, E: ExternalResolutionContext> {
    external_context: &'external mut E,
    /// Stack of active models. The last element is the current model.
    active_models: Vec<ir::ModelPath>,
    /// Set of models that have been visited.
    visited_models: IndexSet<ir::ModelPath>,
    /// Map of model results.
    model_results: IndexMap<ir::ModelPath, ModelResolutionResult>,
}

impl<'external, E: ExternalResolutionContext> ResolutionContext<'external, E> {
    /// Creates a new resolution context.
    #[must_use]
    pub fn new(external_context: &'external mut E) -> Self {
        Self {
            external_context,
            active_models: Vec::new(),
            visited_models: IndexSet::new(),
            model_results: IndexMap::new(),
        }
    }

    /// Consumes the context and returns the accumulated models and errors.
    #[must_use]
    pub fn into_result(self) -> IndexMap<ir::ModelPath, ModelResolutionResult> {
        self.model_results
    }

    /// Activates a model and initializes its result entry.
    pub fn push_active_model(&mut self, model_path: &ir::ModelPath) {
        self.active_models.push(model_path.clone());
        self.visited_models.insert(model_path.clone());
        self.model_results.insert(
            model_path.clone(),
            ModelResolutionResult::new(model_path.clone()),
        );
    }

    /// Deactivates the current model.
    ///
    /// # Panics
    ///
    /// Panics if the popped model does not match the given model path.
    pub fn pop_active_model(&mut self, model_path: &ir::ModelPath) {
        let popped = self
            .active_models
            .pop()
            .expect("attempted to pop from empty active models stack");

        assert_eq!(
            &popped, model_path,
            "popped model path does not match the given model path"
        );
    }

    /// Checks if the given model is active.
    #[must_use]
    pub fn is_model_active(&self, model_path: &ir::ModelPath) -> bool {
        self.active_models.contains(model_path)
    }

    /// Returns the stack of active models.
    #[must_use]
    pub fn active_models(&self) -> &[ir::ModelPath] {
        &self.active_models
    }

    /// Checks if the given model has been visited.
    #[must_use]
    pub fn has_visited_model(&self, model_path: &ir::ModelPath) -> bool {
        self.visited_models.contains(model_path)
    }

    /// Returns a reference to the current active model.
    ///
    /// # Panics
    ///
    /// Panics if there is no active model or if the active model is not in the model map.
    fn active_model(&self) -> &ir::Model {
        let path = self.active_models.last().expect("no active model");
        self.model_results
            .get(path)
            .expect("active model not in model results map")
            .model()
    }

    /// Returns a mutable reference to the current active model.
    ///
    /// # Panics
    ///
    /// Panics if there is no active model or if the active model is not in the model map.
    fn active_model_mut(&mut self) -> &mut ir::Model {
        let path = self.active_models.last().expect("no active model");
        self.model_results
            .get_mut(path)
            .expect("active model not in model results map")
            .model_mut()
    }

    /// Returns a mutable reference to the resolution errors for the current active model.
    ///
    /// # Panics
    ///
    /// Panics if there is no active model or if the active model has no error entry.
    fn active_model_errors_mut(&mut self) -> &mut ResolutionErrorCollection {
        let path = self.active_models.last().expect("no active model");
        self.model_results
            .get_mut(path)
            .expect("active model not in model results map")
            .model_errors_mut()
    }

    /// Returns whether the given model path has any resolution errors.
    fn model_has_errors(&self, model_path: &ir::ModelPath) -> bool {
        self.model_results
            .get(model_path)
            .is_some_and(|r| !r.model_errors().is_empty())
    }

    /// Checks if the given identifier refers to a builtin value.
    #[must_use]
    pub fn has_builtin_value(&self, identifier: &ir::Identifier) -> bool {
        self.external_context.has_builtin_value(identifier)
    }

    /// Checks if the given identifier refers to a builtin function.
    #[must_use]
    pub fn has_builtin_function(&self, identifier: &ir::Identifier) -> bool {
        self.external_context.has_builtin_function(identifier)
    }

    /// Checks if the given name refers to a builtin unit.
    #[must_use]
    pub fn has_builtin_unit(&self, name: &str) -> bool {
        self.external_context.has_builtin_unit(name)
    }

    /// Returns the available unit prefixes.
    pub fn available_prefixes(&self) -> impl Iterator<Item = (&str, f64)> {
        self.external_context.available_prefixes()
    }

    /// Returns whether the given unit name supports SI prefixes.
    #[must_use]
    pub fn unit_supports_si_prefixes(&self, name: &str) -> bool {
        self.external_context.unit_supports_si_prefixes(name)
    }

    /// Loads the AST for a model.
    pub fn load_ast(
        &mut self,
        path: &ir::ModelPath,
    ) -> oneil_shared::load_result::LoadResult<&oneil_ast::ModelNode, AstLoadingFailedError> {
        self.external_context.load_ast(path)
    }

    /// Loads a Python import and records either the import or an error.
    pub fn load_python_import_to_active_model(
        &mut self,
        python_path: &ir::PythonPath,
        python_path_span: Span,
    ) {
        let load_result = self.external_context.load_python_import(python_path);

        if let Ok(functions) = load_result {
            let functions = functions.into_iter().map(String::from).collect();
            let import = ir::PythonImport::new(python_path.clone(), python_path_span, functions);
            self.active_model_mut()
                .add_python_import(python_path.clone(), import);
        } else {
            let error = PythonImportResolutionError::failed_validation(
                python_path_span,
                python_path.clone(),
            );

            self.add_python_import_error_to_active_model(python_path.clone(), error);
        }
    }

    /// Adds a Python import error to the active model.
    pub fn add_python_import_error_to_active_model(
        &mut self,
        python_path: ir::PythonPath,
        error: PythonImportResolutionError,
    ) {
        self.active_model_errors_mut()
            .add_import_error(python_path, error);
    }

    /// Gets a Python import from the active model.
    #[must_use]
    pub fn get_python_import_from_active_model(
        &self,
        python_path: &ir::PythonPath,
    ) -> Option<&ir::PythonImport> {
        self.active_model().get_python_imports().get(python_path)
    }

    /// Returns the Python paths in the active model that export the given function name.
    #[must_use]
    pub fn lookup_imported_function(&self, function_name: &str) -> Vec<ir::PythonPath> {
        self.active_model()
            .get_python_imports()
            .iter()
            .filter(|(_path, import)| import.functions().contains(function_name))
            .map(|(path, _)| path.clone())
            .collect()
    }

    /// Adds a reference to the active model.
    pub fn add_reference_to_active_model(
        &mut self,
        reference_name: ir::ReferenceName,
        reference_name_span: Span,
        reference_path: ir::ModelPath,
    ) {
        let import =
            ir::ReferenceImport::new(reference_name.clone(), reference_name_span, reference_path);
        self.active_model_mut()
            .add_reference(reference_name, import);
    }

    /// Adds a submodel to the active model.
    pub fn add_submodel_to_active_model(
        &mut self,
        submodel_name: ir::SubmodelName,
        submodel_name_span: Span,
        reference_name: ir::ReferenceName,
    ) {
        let import =
            ir::SubmodelImport::new(submodel_name.clone(), submodel_name_span, reference_name);

        self.active_model_mut().add_submodel(submodel_name, import);
    }

    /// Gets a reference from the active model.
    #[must_use]
    pub fn get_reference_from_active_model(
        &self,
        reference_name: &ir::ReferenceName,
    ) -> Option<&ir::ReferenceImport> {
        self.active_model().get_references().get(reference_name)
    }

    /// Gets a submodel from the active model.
    #[must_use]
    pub fn get_submodel_from_active_model(
        &self,
        submodel_name: &ir::SubmodelName,
    ) -> Option<&ir::SubmodelImport> {
        self.active_model().get_submodels().get(submodel_name)
    }

    /// Looks up a model by path.
    #[must_use]
    pub fn lookup_model(&self, model_path: &ir::ModelPath) -> ModelResult<'_> {
        if self.model_has_errors(model_path) {
            return ModelResult::HasError;
        }

        self.model_results
            .get(model_path)
            .map_or(ModelResult::NotFound, |r| ModelResult::Found(r.model()))
    }

    /// Looks up the path to a reference in the active model.
    #[must_use]
    pub fn lookup_reference_path_in_active_model(
        &self,
        reference_name: &ir::ReferenceName,
    ) -> ReferencePathResult<'_, '_> {
        let active_path = self.active_models.last().expect("no active model");
        let errors = self
            .model_results
            .get(active_path)
            .map(ModelResolutionResult::model_errors);

        if let Some(errors) = errors
            && errors
                .get_model_import_resolution_errors()
                .contains_key(reference_name)
        {
            return ReferencePathResult::ReferenceHasResolutionError;
        }

        let model = self.active_model();
        let Some(reference_import) = model.get_references().get(reference_name) else {
            return ReferencePathResult::ReferenceNotFound;
        };
        let reference_path = reference_import.path();

        if self.model_has_errors(reference_path) {
            return ReferencePathResult::ModelHasResolutionError(reference_path);
        }

        let Some(target_model) = self
            .model_results
            .get(reference_path)
            .map(ModelResolutionResult::model)
        else {
            return ReferencePathResult::ModelNotFound(reference_path);
        };

        ReferencePathResult::Found(target_model, reference_path)
    }

    /// Adds a circular dependency error to the active model.
    pub fn add_circular_dependency_error_to_active_model(
        &mut self,
        circular_dependency: CircularDependencyError,
    ) {
        self.active_model_errors_mut()
            .add_circular_dependency_error(circular_dependency);
    }

    /// Adds a reference resolution error to the active model.
    pub fn add_model_import_resolution_error_to_active_model(
        &mut self,
        reference_name: ir::ReferenceName,
        submodel_name: Option<ir::SubmodelName>,
        error: ModelImportResolutionError,
    ) {
        self.active_model_errors_mut()
            .add_model_import_resolution_error(reference_name, submodel_name, error);
    }

    /// Adds a parameter to the active model.
    pub fn add_parameter_to_active_model(
        &mut self,
        parameter_name: ir::ParameterName,
        parameter: ir::Parameter,
    ) {
        self.active_model_mut()
            .add_parameter(parameter_name, parameter);
    }

    /// Looks up a parameter in the active model.
    #[must_use]
    pub fn lookup_parameter_in_active_model(
        &self,
        parameter_name: &ir::ParameterName,
    ) -> ParameterResult<'_> {
        let active_path = self.active_models.last().expect("no active model");
        if let Some(errors) = self
            .model_results
            .get(active_path)
            .map(ModelResolutionResult::model_errors)
            && errors
                .get_parameter_resolution_errors()
                .contains_key(parameter_name)
        {
            return ParameterResult::HasError;
        }

        self.active_model()
            .get_parameters()
            .get(parameter_name)
            .map_or(ParameterResult::NotFound, ParameterResult::Found)
    }

    /// Adds a parameter error to the active model.
    pub fn add_parameter_error_to_active_model(
        &mut self,
        parameter_name: ir::ParameterName,
        error: ParameterResolutionError,
    ) {
        self.active_model_errors_mut()
            .add_parameter_error(parameter_name, error);
    }

    /// Adds a test to the active model.
    pub fn add_test_to_active_model(&mut self, test_index: ir::TestIndex, test: ir::Test) {
        self.active_model_mut().add_test(test_index, test);
    }

    /// Adds a test error to the active model.
    pub fn add_test_error_to_active_model(
        &mut self,
        test_index: ir::TestIndex,
        error: VariableResolutionError,
    ) {
        self.active_model_errors_mut()
            .add_test_error(test_index, error);
    }
}

/// Test-only accessors for inspecting the active model's resolved tests and errors.
#[cfg(test)]
impl<E: ExternalResolutionContext> ResolutionContext<'_, E> {
    /// Returns a mutable reference to the current active model.
    ///
    /// # Panics
    ///
    /// Panics if there is no active model or if the active model is not in the model map.
    #[cfg(test)]
    fn active_model_errors(&self) -> &ResolutionErrorCollection {
        let path = self.active_models.last().expect("no active model");
        self.model_results
            .get(path)
            .expect("active model not in model errors map")
            .model_errors()
    }

    /// Returns the resolved tests for the active model.
    #[must_use]
    pub fn get_active_model_tests(&self) -> &IndexMap<ir::TestIndex, ir::Test> {
        self.active_model().get_tests()
    }

    /// Returns the test resolution errors for the active model, if any.
    #[must_use]
    pub fn get_active_model_test_errors(
        &self,
    ) -> &IndexMap<ir::TestIndex, Vec<VariableResolutionError>> {
        self.active_model_errors().get_test_resolution_errors()
    }

    /// Returns the resolved Python imports for the active model.
    #[must_use]
    pub fn get_active_model_python_imports(&self) -> &IndexMap<ir::PythonPath, ir::PythonImport> {
        self.active_model().get_python_imports()
    }

    /// Returns the Python import resolution errors for the active model.
    #[must_use]
    pub fn get_active_model_python_import_errors(
        &self,
    ) -> &IndexMap<ir::PythonPath, PythonImportResolutionError> {
        self.active_model_errors()
            .get_python_import_resolution_errors()
    }

    /// Returns the resolved parameters for the active model.
    #[must_use]
    pub fn get_active_model_parameters(&self) -> &IndexMap<ir::ParameterName, ir::Parameter> {
        self.active_model().get_parameters()
    }

    /// Returns the parameter resolution errors for the active model.
    #[must_use]
    pub fn get_active_model_parameter_errors(
        &self,
    ) -> &IndexMap<ir::ParameterName, Vec<ParameterResolutionError>> {
        self.active_model_errors().get_parameter_resolution_errors()
    }

    /// Returns the resolved references for the active model.
    #[must_use]
    pub fn get_active_model_references(&self) -> &IndexMap<ir::ReferenceName, ir::ReferenceImport> {
        self.active_model().get_references()
    }

    /// Returns the resolved submodels for the active model.
    #[must_use]
    pub fn get_active_model_submodels(&self) -> &IndexMap<ir::SubmodelName, ir::SubmodelImport> {
        self.active_model().get_submodels()
    }

    /// Returns the reference resolution errors for the active model.
    #[must_use]
    pub fn get_active_model_model_import_errors(
        &self,
    ) -> &IndexMap<ir::ReferenceName, (Option<ir::SubmodelName>, ModelImportResolutionError)> {
        self.active_model_errors()
            .get_model_import_resolution_errors()
    }
}

#[derive(Debug)]
pub enum ModelResult<'model> {
    Found(&'model ir::Model),
    HasError,
    NotFound,
}

#[derive(Debug)]
pub enum ReferencePathResult<'model, 'reference> {
    Found(&'model ir::Model, &'reference ir::ModelPath),
    ReferenceHasResolutionError,
    ReferenceNotFound,
    ModelHasResolutionError(&'reference ir::ModelPath),
    ModelNotFound(&'reference ir::ModelPath),
}

#[derive(Debug)]
pub enum ParameterResult<'parameter> {
    Found(&'parameter ir::Parameter),
    HasError,
    NotFound,
}
