use indexmap::{IndexMap, IndexSet};
use oneil_ast as ast;
use oneil_ir as ir;
use oneil_shared::span::Span;

use crate::error::{
    CircularDependencyError, ModelImportResolutionError, ParameterResolutionError,
    PythonImportResolutionError, ResolutionErrorCollection, VariableResolutionError,
};

/// Error indicating that loading/parsing a model's AST failed.
pub struct AstLoadingFailedError;

/// Error indicating that loading a Python import failed.
pub struct PythonImportLoadingFailedError;

/// Result of resolving one or more models: resolved models and per-model errors.
pub struct ModelResolutionResult {
    /// Resolved model.
    model: ir::Model,
    /// Model resolution errors (including circular dependency errors).
    model_errors: ResolutionErrorCollection,
    /// Whether the AST for the model has been loaded.
    ast_loaded: bool,
    /// Failed Python imports.
    failed_python_imports: IndexSet<ir::PythonPath>,
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
            ast_loaded: true,
            failed_python_imports: IndexSet::new(),
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

    /// Returns whether the AST for the model has been loaded.
    #[must_use]
    pub const fn ast_loaded(&self) -> bool {
        self.ast_loaded
    }

    /// Returns a mutable reference to the AST loaded flag.
    pub const fn ast_loaded_mut(&mut self) -> &mut bool {
        &mut self.ast_loaded
    }

    /// Returns a reference to the failed Python imports.
    #[must_use]
    pub const fn failed_python_imports(&self) -> &IndexSet<ir::PythonPath> {
        &self.failed_python_imports
    }

    /// Returns a mutable reference to the failed Python imports.
    pub const fn failed_python_imports_mut(&mut self) -> &mut IndexSet<ir::PythonPath> {
        &mut self.failed_python_imports
    }

    /// Breaks the result into its components.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        ir::Model,
        ResolutionErrorCollection,
        bool,
        IndexSet<ir::PythonPath>,
    ) {
        (
            self.model,
            self.model_errors,
            self.ast_loaded,
            self.failed_python_imports,
        )
    }
}

/// Context provided by the environment for resolving models (builtins, AST loading, Python imports).
pub trait ExternalResolutionContext {
    /// Checks if the given identifier refers to a builtin value.
    fn has_builtin_value(&self, identifier: &ir::Identifier) -> bool;

    /// Checks if the given identifier refers to a builtin function.
    fn has_builtin_function(&self, identifier: &ir::Identifier) -> bool;

    /// Loads the AST for a model.
    ///
    /// # Errors
    ///
    /// Returns `Err(AstLoadingFailedError)` when the model file cannot be read or parsed.
    fn load_ast(&mut self, path: &ir::ModelPath) -> Result<&ast::Model, AstLoadingFailedError>;

    /// Loads a Python import.
    ///
    /// # Errors
    ///
    /// Returns `Err(PythonImportLoadingFailedError)` when the Python import cannot be loaded or
    /// validated.
    fn load_python_import(
        &mut self,
        python_path: &ir::PythonPath,
    ) -> Result<IndexSet<String>, PythonImportLoadingFailedError>;
}

pub struct ResolutionContext<'external, E: ExternalResolutionContext> {
    external_context: &'external mut E,
    /// Stack of active models. The last element is the current model.
    active_models: Vec<ir::ModelPath>,
    /// Set of models that have been visited.
    visited_models: IndexSet<ir::ModelPath>,
    /// Map of model results.
    model_results: IndexMap<ir::ModelPath, ModelResolutionResult>,
}

/// A trait for providing resolution context to the model resolver.
///
/// This is used to provide the model resolver information about the outside world.
impl<'external, E: ExternalResolutionContext> ResolutionContext<'external, E> {
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

    // ===== MODEL LOADING =====

    /// Activates a model.
    ///
    /// This has the responsibility of:
    /// - Adding the model to the active models stack
    /// - Marking the model as visited
    /// - Initializing the model IR in the model map (empty, to be populated later)
    ///
    /// This assumes stack-like behavior, where the most recently
    /// activated model is the current model.
    pub fn push_active_model(&mut self, model_path: &ir::ModelPath) {
        self.active_models.push(model_path.clone());
        self.visited_models.insert(model_path.clone());
        self.model_results.insert(
            model_path.clone(),
            ModelResolutionResult::new(model_path.clone()),
        );
    }

    /// Deactivates a model.
    ///
    /// This assumes stack-like behavior, where the most recently
    /// activated model is the current model.
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

    /// Marks the AST for a model as not loaded.
    pub fn mark_ast_not_loaded(&mut self, model_path: &ir::ModelPath) {
        self.model_results
            .get_mut(model_path)
            .expect("model not found")
            .ast_loaded = false;
    }

    /// Checks if the given model is active.
    ///
    /// A model is active if it is anywhere in the active models stack.
    pub fn is_model_active(&self, model_path: &ir::ModelPath) -> bool {
        self.active_models.contains(model_path)
    }

    /// Returns the stack of active models.
    ///
    /// The last model in the stack is the current active model.
    pub fn active_models(&self) -> &[ir::ModelPath] {
        &self.active_models
    }

    /// Checks if the given model has been visited.
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

    /// Returns a mutable reference to the failed Python imports for the current active model.
    ///
    /// # Panics
    ///
    /// Panics if there is no active model or if the active model has no failed Python imports.
    fn active_model_failed_python_imports_mut(&mut self) -> &mut IndexSet<ir::PythonPath> {
        let path = self.active_models.last().expect("no active model");
        self.model_results
            .get_mut(path)
            .expect("active model not in model results map")
            .failed_python_imports_mut()
    }

    /// Returns whether the given model path has any resolution errors.
    fn model_has_errors(&self, model_path: &ir::ModelPath) -> bool {
        self.model_results
            .get(model_path)
            .is_some_and(|r| !r.model_errors().is_empty())
    }

    // ===== BUILTINS =====

    /// Checks if the given identifier refers to a builtin value.
    pub fn has_builtin_value(&self, identifier: &ir::Identifier) -> bool {
        self.external_context.has_builtin_value(identifier)
    }

    /// Checks if the given identifier refers to a builtin function.
    pub fn has_builtin_function(&self, identifier: &ir::Identifier) -> bool {
        self.external_context.has_builtin_function(identifier)
    }

    // ===== AST LOADING =====

    /// Loads the AST for a model.
    ///
    /// Returns `Ok(ast)` if parsing succeeds, or `Err(())` if parsing fails.
    pub fn load_ast(&mut self, path: &ir::ModelPath) -> Result<ast::Model, AstLoadingFailedError> {
        self.external_context.load_ast(path).cloned()
    }

    // ===== PYTHON IMPORT LOADING =====

    /// Loads a Python import.
    ///
    /// This has the responsibility of:
    /// - Loading the Python import
    /// - Adding the Python import to the active model
    /// - Adding a Python import error if the import fails to load
    pub fn load_python_import_to_active_model(
        &mut self,
        python_path: &ir::PythonPath,
        python_path_span: Span,
    ) {
        let load_result = self.external_context.load_python_import(python_path);

        if let Ok(functions) = load_result {
            let import = ir::PythonImport::new(python_path.clone(), python_path_span, functions);
            self.active_model_mut()
                .add_python_import(python_path.clone(), import);
        } else {
            let error = PythonImportResolutionError::failed_validation(
                python_path_span,
                python_path.clone(),
            );

            self.mark_python_import_not_loaded(python_path);
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
    pub fn get_python_import_from_active_model(
        &self,
        python_path: &ir::PythonPath,
    ) -> Option<&ir::PythonImport> {
        self.active_model().get_python_imports().get(python_path)
    }

    /// Marks a Python import as not loaded.
    pub fn mark_python_import_not_loaded(&mut self, python_path: &ir::PythonPath) {
        self.active_model_failed_python_imports_mut()
            .insert(python_path.clone());
    }

    /// Returns the Python paths in the active model that export the given function name.
    ///
    /// Only imports that were successfully loaded and that include `function_name` in
    /// their function set are returned.
    #[must_use]
    pub fn lookup_imported_function(&self, function_name: &str) -> Vec<ir::PythonPath> {
        self.active_model()
            .get_python_imports()
            .iter()
            .filter(|(_path, import)| import.functions().contains(function_name))
            .map(|(path, _)| path.clone())
            .collect()
    }

    // ===== MODEL IMPORT LOADING =====

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
    ///
    /// A submodel always refers to a reference.
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
    pub fn get_reference_from_active_model(
        &self,
        reference_name: &ir::ReferenceName,
    ) -> Option<&ir::ReferenceImport> {
        self.active_model().get_references().get(reference_name)
    }

    /// Gets a submodel from the active model.
    pub fn get_submodel_from_active_model(
        &self,
        submodel_name: &ir::SubmodelName,
    ) -> Option<&ir::SubmodelImport> {
        self.active_model().get_submodels().get(submodel_name)
    }

    /// Looks up a model by path.
    pub fn lookup_model(&self, model_path: &ir::ModelPath) -> ModelResult<'_> {
        if self.model_has_errors(model_path) {
            return ModelResult::HasError;
        }

        self.model_results
            .get(model_path)
            .map_or(ModelResult::NotFound, |r| ModelResult::Found(r.model()))
    }

    /// Looks up the path to a reference in the active model.
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

    // ===== PARAMETER LOADING =====

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

    // ===== TEST LOADING =====

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
