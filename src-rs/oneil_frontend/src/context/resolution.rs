use indexmap::{IndexMap, IndexSet};
use oneil_ast as ast;
use oneil_ir as ir;
use oneil_shared::{
    InstancePath,
    load_result::LoadResult,
    paths::{ModelPath, PythonPath},
    span::Span,
    symbols::{
        BuiltinFunctionName, ParameterName, PyFunctionName, ReferenceName, SubmodelName, TestIndex,
        UnitBaseName, UnitPrefix,
    },
};

use crate::{
    error::{
        DesignResolutionError, ModelImportResolutionError, ParameterResolutionError,
        PythonImportResolutionError, ResolutionErrorCollection, VariableResolutionError,
    },
    instance::{
        ApplyDesign, InstancedModel,
        design::Design,
        imports::{AliasImport, ReferenceImport, SubmodelImport},
    },
};

use super::{AstLoadingFailedError, ExternalResolutionContext};

/// The maximum Levenshtein distance for a best match.
pub const MAX_BEST_MATCH_DISTANCE: usize = 2;

/// Result of resolving a single model file.
///
/// Holds the lowered [`InstancedModel`] template, design metadata that the
/// instance-graph build pass consumes, and any resolution errors. The design
/// metadata is kept separate from the model because it is only needed while
/// building the [`InstanceGraph`] and can be discarded afterwards.
#[derive(Debug)]
#[expect(
    clippy::partial_pub_fields,
    reason = "design_export and applied_designs are populated post-construction by the build pass; model and model_errors are accessed via accessor methods"
)]
pub struct ModelResolutionResult {
    /// Lowered model template (file-static, instance keys all `None`).
    model: InstancedModel,
    /// Resolved design content exported by this file, if it is a design file.
    pub design_export: Option<Design>,
    /// Declarative `apply X to ref` records from this model file.
    pub applied_designs: Vec<ApplyDesign>,
    /// Resolution errors.
    model_errors: ResolutionErrorCollection,
}

impl ModelResolutionResult {
    /// Creates an empty result with an empty model and no errors.
    #[must_use]
    pub fn new(model_path: ModelPath) -> Self {
        Self {
            model: InstancedModel::empty_for(model_path),
            design_export: None,
            applied_designs: Vec::new(),
            model_errors: ResolutionErrorCollection::empty(),
        }
    }

    /// Creates a result from a pre-loaded [`LoadResult`].
    ///
    /// Design metadata is not carried through `LoadResult`; callers that need
    /// it must populate `design_export` and `applied_designs` directly.
    #[must_use]
    pub fn from_load_result(
        model_path: ModelPath,
        result: &LoadResult<InstancedModel, ResolutionErrorCollection>,
    ) -> Self {
        match result {
            LoadResult::Success(model) => Self {
                model: model.clone(),
                design_export: None,
                applied_designs: Vec::new(),
                model_errors: ResolutionErrorCollection::empty(),
            },
            LoadResult::Partial(model, errors) => Self {
                model: model.clone(),
                design_export: None,
                applied_designs: Vec::new(),
                model_errors: errors.clone(),
            },
            LoadResult::Failure => Self::new(model_path),
        }
    }

    /// Creates a result from an already-extracted model and its errors.
    ///
    /// Design metadata (`design_export`, `applied_designs`) is not carried
    /// here; callers that need it must populate those fields separately.
    /// This constructor avoids the extra clone that [`Self::from_load_result`]
    /// requires when the caller already owns the data.
    #[must_use]
    pub const fn from_model_and_errors(
        model: InstancedModel,
        errors: ResolutionErrorCollection,
    ) -> Self {
        Self {
            model,
            design_export: None,
            applied_designs: Vec::new(),
            model_errors: errors,
        }
    }

    /// Returns the resolved model.
    #[must_use]
    pub const fn model(&self) -> &InstancedModel {
        &self.model
    }

    /// Returns a mutable reference to the model.
    pub const fn model_mut(&mut self) -> &mut InstancedModel {
        &mut self.model
    }

    /// Returns the resolution errors.
    #[must_use]
    pub const fn model_errors(&self) -> &ResolutionErrorCollection {
        &self.model_errors
    }

    /// Returns a mutable reference to the resolution errors.
    pub const fn model_errors_mut(&mut self) -> &mut ResolutionErrorCollection {
        &mut self.model_errors
    }

    /// Breaks the result into its components: model, design export, applied
    /// designs, and errors.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        InstancedModel,
        Option<Design>,
        Vec<ApplyDesign>,
        ResolutionErrorCollection,
    ) {
        (
            self.model,
            self.design_export,
            self.applied_designs,
            self.model_errors,
        )
    }
}

/// In-memory context used while resolving one or more models.
#[derive(Debug)]
pub struct ResolutionContext<'external, E: ExternalResolutionContext> {
    external_context: &'external mut E,
    /// Stack of active models (last element is the current model).
    active_models: Vec<ModelPath>,
    /// Set of models that have been visited.
    visited_models: IndexSet<ModelPath>,
    /// Paths for which `load_ast` was called by `load_model` and returned
    /// failure (file not found / parse error).  Distinct from `visited_models`
    /// because the test helper pre-registers models without ever calling
    /// `load_ast`, so `visited_models` alone cannot differentiate "tried and
    /// failed" from "injected directly in a test".
    failed_ast_loads: IndexSet<ModelPath>,
    /// Per-model resolution results.
    model_results: IndexMap<ModelPath, ModelResolutionResult>,
    /// Design-local parameters visible during resolution but not persisted in
    /// the model's IR.
    ///
    /// When a design file declares new parameters (e.g. `design target; x = 1;
    /// y = 2 * x`), those names must be visible to variable lookups while the
    /// target model is active, but should not be added to the target model's
    /// shared template. Keyed by `(target model path, parameter name)`.
    design_local_scratch: IndexMap<(ModelPath, ParameterName), ir::Parameter>,
}

impl<'external, E: ExternalResolutionContext> ResolutionContext<'external, E> {
    /// Creates a new resolution context.
    #[must_use]
    pub fn new(external_context: &'external mut E) -> Self {
        Self {
            external_context,
            active_models: Vec::new(),
            visited_models: IndexSet::new(),
            failed_ast_loads: IndexSet::new(),
            model_results: IndexMap::new(),
            design_local_scratch: IndexMap::new(),
        }
    }

    /// Creates a resolution context pre-seeded with already-loaded models.
    #[must_use]
    pub fn with_preloaded_models(external_context: &'external mut E) -> Self {
        let model_results = external_context
            .get_preloaded_models()
            .map(|(path, model, errors)| {
                (
                    path,
                    ModelResolutionResult::from_model_and_errors(model, errors),
                )
            })
            .collect();

        Self {
            external_context,
            active_models: Vec::new(),
            visited_models: IndexSet::new(),
            failed_ast_loads: IndexSet::new(),
            model_results,
            design_local_scratch: IndexMap::new(),
        }
    }

    /// Consumes the context and returns the accumulated results.
    #[must_use]
    pub fn into_result(self) -> IndexMap<ModelPath, ModelResolutionResult> {
        self.model_results
    }

    /// Activates a model and initializes its result entry.
    pub fn push_active_model(&mut self, model_path: &ModelPath) {
        self.active_models.push(model_path.clone());
        self.visited_models.insert(model_path.clone());
        self.model_results
            .entry(model_path.clone())
            .or_insert_with(|| ModelResolutionResult::new(model_path.clone()));
    }

    /// Deactivates the current model.
    ///
    /// # Panics
    ///
    /// Panics if the popped model does not match `model_path`.
    pub fn pop_active_model(&mut self, model_path: &ModelPath) {
        let popped = self
            .active_models
            .pop()
            .expect("attempted to pop from empty active models stack");

        assert_eq!(
            &popped, model_path,
            "popped model path does not match the given model path"
        );
    }

    /// Returns `true` if the given model is currently active (on the stack).
    #[must_use]
    pub fn is_model_active(&self, model_path: &ModelPath) -> bool {
        self.active_models.contains(model_path)
    }

    /// Returns the stack of active models.
    #[must_use]
    pub fn active_models(&self) -> &[ModelPath] {
        &self.active_models
    }

    /// Returns `true` if the given model has been visited.
    #[must_use]
    pub fn has_visited_model(&self, model_path: &ModelPath) -> bool {
        self.visited_models.contains(model_path)
    }

    /// Records that `load_ast` was called for `path` and returned a failure.
    ///
    /// Called by [`load_model`](crate::resolver::load_model) when the
    /// underlying file cannot be read or parsed.  Use
    /// [`ast_load_failed`](Self::ast_load_failed) to query the flag later.
    pub fn record_failed_ast_load(&mut self, path: ModelPath) {
        self.failed_ast_loads.insert(path);
    }

    /// Returns `true` when [`record_failed_ast_load`](Self::record_failed_ast_load)
    /// was previously called for `path`.
    ///
    /// Returns `false` for models that were pre-registered directly (e.g. via
    /// the test helper) without ever going through the file-loading path.
    #[must_use]
    pub fn ast_load_failed(&self, path: &ModelPath) -> bool {
        self.failed_ast_loads.contains(path)
    }

    pub(crate) fn active_model(&self) -> &InstancedModel {
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
    pub(crate) fn active_model_mut(&mut self) -> &mut InstancedModel {
        let path = self.active_models.last().expect("no active model");
        self.model_results
            .get_mut(path)
            .expect("active model not in model results map")
            .model_mut()
    }

    /// Records a design resolution error on the active model.
    pub(crate) fn add_design_resolution_error_to_active_model(
        &mut self,
        message: impl Into<String>,
        span: Span,
    ) {
        self.active_model_errors_mut()
            .add_design_resolution_error(DesignResolutionError::new(message, span));
    }

    fn active_model_errors_mut(&mut self) -> &mut ResolutionErrorCollection {
        let path = self.active_models.last().expect("no active model");
        self.model_results
            .get_mut(path)
            .expect("active model not in model results map")
            .model_errors_mut()
    }

    fn model_has_errors(&self, model_path: &ModelPath) -> bool {
        self.model_results
            .get(model_path)
            .is_some_and(|r| !r.model_errors().is_empty())
    }

    /// Returns `true` if the given identifier refers to a builtin value.
    #[must_use]
    pub fn has_builtin_value(&self, identifier: &ast::Identifier) -> bool {
        self.external_context.has_builtin_value(identifier)
    }

    /// Returns `true` if the given identifier refers to a builtin function.
    #[must_use]
    pub fn has_builtin_function(&self, identifier: &ast::Identifier) -> bool {
        self.external_context.has_builtin_function(identifier)
    }

    /// Returns all builtin function names for fuzzy matching (stable iteration order).
    pub fn get_builtin_functions(&self) -> impl Iterator<Item = &BuiltinFunctionName> {
        self.external_context.get_builtin_function_names()
    }

    /// Returns every Python function name imported into the active model (across all imports).
    pub fn get_active_model_imported_functions(&self) -> impl Iterator<Item = &PyFunctionName> {
        self.active_model()
            .python_imports()
            .values()
            .flat_map(ir::PythonImport::functions)
    }

    /// Returns `true` if the given name refers to a builtin unit.
    #[must_use]
    pub fn has_builtin_unit(&self, name: &str) -> bool {
        self.external_context.has_builtin_unit(name)
    }

    /// Returns the available unit prefixes.
    pub fn available_prefixes(&self) -> impl Iterator<Item = (&UnitPrefix, f64)> {
        self.external_context.available_prefixes()
    }

    /// Returns `true` if the given unit name supports SI prefixes.
    #[must_use]
    pub fn unit_supports_si_prefixes(&self, name: &UnitBaseName) -> bool {
        self.external_context.unit_supports_si_prefixes(name)
    }

    /// Returns the resolved unit for the given builtin base name, if any.
    #[must_use]
    pub fn lookup_unit(&self, name: &UnitBaseName) -> Option<&oneil_output::Unit> {
        self.external_context.lookup_unit(name)
    }

    /// Loads the AST for a model.
    pub fn load_ast(
        &mut self,
        path: &ModelPath,
    ) -> oneil_shared::load_result::LoadResult<&oneil_ast::ModelNode, AstLoadingFailedError> {
        self.external_context.load_ast(path)
    }

    /// Loads a Python import and records either the import or an error on the active model.
    pub fn load_python_import_to_active_model(
        &mut self,
        python_path: &PythonPath,
        python_path_span: Span,
    ) {
        let load_result = self.external_context.load_python_import(python_path);

        if let Ok(functions) = load_result {
            let functions = functions.into_iter().cloned().collect();
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
        python_path: PythonPath,
        error: PythonImportResolutionError,
    ) {
        self.active_model_errors_mut()
            .add_import_error(python_path, error);
    }

    /// Gets a Python import from the active model.
    #[must_use]
    pub fn get_python_import_from_active_model(
        &self,
        python_path: &PythonPath,
    ) -> Option<&ir::PythonImport> {
        self.active_model().python_imports().get(python_path)
    }

    /// Returns the Python paths in the active model that export the given function name.
    #[must_use]
    pub fn lookup_imported_function(&self, function_name: &PyFunctionName) -> Vec<PythonPath> {
        self.active_model()
            .python_imports()
            .iter()
            .filter(|(_path, import)| import.functions().contains(function_name))
            .map(|(path, _)| path.clone())
            .collect()
    }

    /// Adds a reference to the active model.
    pub fn add_reference_to_active_model(
        &mut self,
        reference_name: ReferenceName,
        reference_name_span: Span,
        alias: Option<ReferenceName>,
        alias_span: Option<Span>,
        reference_path: ModelPath,
    ) {
        let import = ReferenceImport::new(
            reference_name.clone(),
            reference_name_span,
            alias,
            alias_span,
            reference_path,
        );
        self.active_model_mut()
            .add_reference(reference_name, import);
    }

    /// Adds a direct submodel to the active model.
    ///
    /// `alias` is the alias (`bar` in `submodel foo as bar`), which is the map
    /// key. `source_name` is the source-level model name (`foo`).
    /// `child_path` is the resolved path of the submodel's source file.
    pub fn add_submodel_to_active_model(
        &mut self,
        alias: ReferenceName,
        source_name: SubmodelName,
        source_name_span: Span,
        explicit_alias: Option<ReferenceName>,
        explicit_alias_span: Option<Span>,
        child_path: ModelPath,
    ) {
        let import = SubmodelImport::stub(
            source_name,
            source_name_span,
            explicit_alias,
            explicit_alias_span,
            child_path,
        );
        self.active_model_mut().add_submodel(alias, import);
    }

    /// Adds a `with`-extracted alias to the active model.
    ///
    /// `alias` is the map key. `alias_segments` is the chain of
    /// reference-name segments (relative to the host) that the alias
    /// resolves to.
    pub fn add_extracted_alias_to_active_model(
        &mut self,
        alias: ReferenceName,
        source_name: SubmodelName,
        source_name_span: Span,
        explicit_alias: Option<ReferenceName>,
        explicit_alias_span: Option<Span>,
        alias_segments: Vec<ReferenceName>,
    ) {
        let alias_path = alias_segments
            .into_iter()
            .fold(InstancePath::root(), |acc, seg| acc.child(seg));
        let import = AliasImport::new(
            source_name,
            source_name_span,
            explicit_alias,
            explicit_alias_span,
            alias_path,
        );
        self.active_model_mut().add_alias(alias, import);
    }

    /// Gets a submodel from the active model by its alias.
    #[must_use]
    pub fn get_submodel_from_active_model(&self, alias: &ReferenceName) -> Option<&SubmodelImport> {
        self.active_model().submodels().get(alias)
    }

    /// Returns the source span of the existing named child (reference,
    /// submodel, or extracted alias) bound to `name` on the active model,
    /// if any.
    ///
    /// Used by import resolution to detect alias collisions: the three child
    /// maps share a single name space because `parameter.alias` lookups walk
    /// any of them.
    #[must_use]
    pub fn get_named_child_span_in_active_model(&self, name: &ReferenceName) -> Option<Span> {
        let model = self.active_model();
        if let Some(r) = model.references().get(name) {
            return Some(r.name_span.clone());
        }
        if let Some(s) = model.submodels().get(name) {
            return Some(s.name_span.clone());
        }
        if let Some(a) = model.aliases().get(name) {
            return Some(a.name_span.clone());
        }
        None
    }

    /// Returns the resolved references for the active model.
    #[cfg(test)]
    #[must_use]
    pub fn get_active_model_references(&self) -> &IndexMap<ReferenceName, ReferenceImport> {
        self.active_model().references()
    }

    /// Returns the resolved parameters for the active model.
    ///
    /// Only used by tests now: the file-time resolver no
    /// longer scans the active model's parameter set for bare-name
    /// existence checks (the walk does that against the binding scope).
    #[cfg(test)]
    #[must_use]
    pub fn get_active_model_parameters(&self) -> &IndexMap<ParameterName, ir::Parameter> {
        self.active_model().parameters()
    }

    /// Looks up a model by path.
    #[must_use]
    pub fn lookup_model(&self, model_path: &ModelPath) -> ModelResult<'_> {
        if self.model_has_errors(model_path) {
            return ModelResult::HasError;
        }
        self.model_results
            .get(model_path)
            .map_or(ModelResult::NotFound, |r| ModelResult::Found(r.model()))
    }

    /// Adds a reference resolution error to the active model.
    pub fn add_model_import_resolution_error_to_active_model(
        &mut self,
        reference_name: ReferenceName,
        submodel_name: Option<SubmodelName>,
        error: ModelImportResolutionError,
    ) {
        self.active_model_errors_mut()
            .add_model_import_resolution_error(reference_name, submodel_name, error);
    }

    /// Adds a parameter to the active model.
    pub fn add_parameter_to_active_model(
        &mut self,
        parameter_name: ParameterName,
        parameter: ir::Parameter,
    ) {
        self.active_model_mut()
            .add_parameter(parameter_name, parameter);
    }

    /// Registers a design-local parameter scoped to `target_model_path`.
    ///
    /// The parameter is visible to variable lookups while the target model is
    /// active, but is not added to the target model's template. Design-local
    /// parameters end up in the design's `parameter_additions`, not on the
    /// target itself.
    pub fn register_design_local_parameter(
        &mut self,
        target_model_path: ModelPath,
        parameter_name: ParameterName,
        parameter: ir::Parameter,
    ) {
        self.design_local_scratch
            .insert((target_model_path, parameter_name), parameter);
    }

    /// Looks up a parameter in the active model.
    #[must_use]
    pub fn lookup_parameter_in_active_model(
        &self,
        parameter_name: &ParameterName,
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

        if let Some(parameter) = self.active_model().parameters().get(parameter_name) {
            return ParameterResult::Found(parameter);
        }

        if let Some(parameter) = self
            .design_local_scratch
            .get(&(active_path.clone(), parameter_name.clone()))
        {
            return ParameterResult::Found(parameter);
        }

        ParameterResult::NotFound
    }

    /// Adds a parameter resolution error to the active model.
    pub fn add_parameter_error_to_active_model(
        &mut self,
        parameter_name: ParameterName,
        error: ParameterResolutionError,
    ) {
        self.active_model_errors_mut()
            .add_parameter_error(parameter_name, error);
    }

    /// Adds a test to the active model.
    pub fn add_test_to_active_model(&mut self, test_index: TestIndex, test: ir::Test) {
        self.active_model_mut().add_test(test_index, test);
    }

    /// Adds a test resolution error to the active model.
    pub fn add_test_error_to_active_model(
        &mut self,
        test_index: TestIndex,
        error: VariableResolutionError,
    ) {
        self.active_model_errors_mut()
            .add_test_error(test_index, error);
    }

    /// Sets the model-level note on the active model when `note` is `Some`.
    pub fn set_active_model_note(&mut self, note: Option<ir::Note>) {
        if let Some(note) = note {
            self.active_model_mut().set_note(note);
        }
    }

    /// Sets the section metadata on the active model.
    ///
    /// Called by the resolver after parameters and tests have been resolved so
    /// that `TestIndex` values are already finalized.
    pub fn set_active_model_sections(
        &mut self,
        sections: indexmap::IndexMap<oneil_shared::labels::SectionLabel, ir::Section>,
    ) {
        self.active_model_mut().set_sections(sections);
    }

    /// Sets the design export on the active model's resolution result.
    pub(crate) fn set_active_model_design_export(&mut self, design: Design) {
        let path = self.active_models.last().expect("no active model");
        if let Some(result) = self.model_results.get_mut(path) {
            result.design_export = Some(design);
        }
    }

    /// Records an `apply X to ref` declaration on the active model's result.
    pub(crate) fn add_applied_design_to_active_model(&mut self, application: ApplyDesign) {
        let path = self.active_models.last().expect("no active model");
        if let Some(result) = self.model_results.get_mut(path) {
            result.applied_designs.push(application);
        }
    }

    /// Returns the design export for a given model path, if one was recorded.
    #[must_use]
    pub(crate) fn get_design_export(&self, model_path: &ModelPath) -> Option<&Design> {
        self.model_results.get(model_path)?.design_export.as_ref()
    }

    /// Adds tests to the design export on the active model's resolution result.
    ///
    /// Copies the model-level note onto the active model's design export so it
    /// can be applied to the composed target node at instance time.
    ///
    /// Must be called after `set_active_model_note` and after the design export
    /// has been stored (i.e. after `resolve_design_surface`).
    pub(crate) fn add_note_to_design_export(&mut self, note: Option<ir::Note>) {
        let Some(note) = note else { return };
        let path = self.active_models.last().expect("no active model");
        if let Some(result) = self.model_results.get_mut(path)
            && let Some(design) = result.design_export.as_mut()
        {
            design.note = Some(note);
        }
    }

    /// Tests from design files are added to the design export so they can be
    /// applied to the target model at instance time. This must be called after
    /// tests have been resolved.
    pub(crate) fn add_tests_to_design_export(&mut self, tests: IndexMap<TestIndex, ir::Test>) {
        let path = self.active_models.last().expect("no active model");
        if let Some(result) = self.model_results.get_mut(path)
            && let Some(design) = result.design_export.as_mut()
        {
            design.test_additions = tests;
        }
    }
}

/// Test-only accessors.
#[cfg(test)]
impl<E: ExternalResolutionContext> ResolutionContext<'_, E> {
    fn active_model_errors(&self) -> &ResolutionErrorCollection {
        let path = self.active_models.last().expect("no active model");
        self.model_results
            .get(path)
            .expect("active model not in model errors map")
            .model_errors()
    }

    /// Returns the resolved tests for the active model.
    #[must_use]
    pub fn get_active_model_tests(&self) -> &IndexMap<TestIndex, ir::Test> {
        self.active_model().tests()
    }

    /// Returns the test resolution errors for the active model.
    #[must_use]
    pub fn get_active_model_test_errors(
        &self,
    ) -> &IndexMap<TestIndex, Vec<VariableResolutionError>> {
        self.active_model_errors().get_test_resolution_errors()
    }

    /// Returns the resolved Python imports for the active model.
    #[must_use]
    pub fn get_active_model_python_imports(&self) -> &IndexMap<PythonPath, ir::PythonImport> {
        self.active_model().python_imports()
    }

    /// Returns the Python import resolution errors for the active model.
    #[must_use]
    pub fn get_active_model_python_import_errors(
        &self,
    ) -> &IndexMap<PythonPath, PythonImportResolutionError> {
        self.active_model_errors()
            .get_python_import_resolution_errors()
    }

    /// Returns the parameter resolution errors for the active model.
    #[must_use]
    pub fn get_active_model_parameter_errors(
        &self,
    ) -> &IndexMap<ParameterName, Vec<ParameterResolutionError>> {
        self.active_model_errors().get_parameter_resolution_errors()
    }

    /// Returns the resolved submodels for the active model, keyed by alias.
    #[must_use]
    pub fn get_active_model_submodels(&self) -> &IndexMap<ReferenceName, SubmodelImport> {
        self.active_model().submodels()
    }

    /// Returns the `with`-extracted aliases for the active model, keyed by alias.
    #[must_use]
    pub fn get_active_model_aliases(&self) -> &IndexMap<ReferenceName, AliasImport> {
        self.active_model().aliases()
    }

    /// Returns the reference resolution errors for the active model.
    #[must_use]
    pub fn get_active_model_model_import_errors(
        &self,
    ) -> &IndexMap<ReferenceName, (Option<SubmodelName>, ModelImportResolutionError)> {
        self.active_model_errors()
            .get_model_import_resolution_errors()
    }
}

#[derive(Debug)]
pub enum ModelResult<'model> {
    Found(&'model InstancedModel),
    HasError,
    NotFound,
}

#[derive(Debug)]
pub enum ParameterResult<'parameter> {
    Found(&'parameter ir::Parameter),
    HasError,
    NotFound,
}
