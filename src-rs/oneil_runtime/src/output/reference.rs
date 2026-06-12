//! References into evaluation results and error contexts.
//!
//! Provides [`ModelReference`] for navigating evaluated models,
//! [`EvalErrorReference`] for inspecting evaluation failures,
//! [`ModelTemplateReference`] for navigating lowered template models, and
//! [`ResolutionErrorReference`] for inspecting resolution failures.

use crate::output;
use indexmap::{IndexMap, IndexSet};
use oneil_frontend::{
    CompilationUnit, InstancedModel,
    instance::graph::UnitGraphCache,
    instance::imports::{AliasImport, ReferenceImport, SubmodelImport},
};
use oneil_ir as ir;
use oneil_shared::{
    paths::{ModelPath, PythonPath},
    span::Span,
    symbols::{ParameterName, ReferenceName, SubmodelName, TestIndex},
};

use crate::cache::EvalCache;

/// A reference to an evaluated model within a model hierarchy.
///
/// This stores a reference to a model and a reference to the
/// entire model collection.
#[derive(Debug, Clone, Copy)]
pub struct ModelReference<'runtime> {
    model: &'runtime output::Model,
    eval_cache: &'runtime EvalCache,
}

impl<'runtime> ModelReference<'runtime> {
    /// Creates a new `ModelReference` for the given model and evaluation cache.
    #[must_use]
    pub const fn new(model: &'runtime output::Model, eval_cache: &'runtime EvalCache) -> Self {
        Self { model, eval_cache }
    }

    /// Returns the file path of this model.
    #[must_use]
    pub const fn path(&self) -> &'runtime ModelPath {
        &self.model.path
    }

    /// Returns the set of aliases declared as submodels on this model.
    ///
    /// Each alias is also a key in [`Self::references`]. The set is provided
    /// so consumers can preserve the submodel-vs-reference distinction
    /// declared in source.
    #[must_use]
    pub const fn submodels(&self) -> &'runtime IndexSet<ReferenceName> {
        &self.model.submodels
    }

    /// Returns a map of reference names to their model references or evaluation errors.
    ///
    /// # Panics
    ///
    /// Panics if any reference has not been visited and
    /// added to the model collection. This should never be
    /// the case as long as creating the `EvalResult`
    /// resolves successfully.
    #[must_use]
    pub fn references(&self) -> IndexMap<&'runtime ReferenceName, Self> {
        self.model
            .references
            .iter()
            .filter_map(|(reference_name, child_key)| {
                let entry = self
                    .eval_cache
                    .get_entry_instance(child_key)
                    .expect("reference should be in cache");

                let model = entry.value()?;

                let result = Self {
                    model,
                    eval_cache: self.eval_cache,
                };

                Some((reference_name, result))
            })
            .collect()
    }

    /// Returns a map of parameter names to their evaluated parameter data.
    #[must_use]
    pub fn parameters(&self) -> IndexMap<&'runtime str, &'runtime output::Parameter> {
        self.model
            .parameters
            .iter()
            .map(|(name, parameter)| (name.as_str(), parameter))
            .collect()
    }

    /// Returns the list of evaluated test results for this model.
    #[must_use]
    pub const fn tests(&self) -> &'runtime IndexMap<TestIndex, output::Test> {
        &self.model.tests
    }

    /// Returns the list of model paths that were successfully evaluated.
    #[must_use]
    pub fn all_model_paths(&self) -> Vec<ModelPath> {
        let mut paths = Vec::new();
        self.all_model_paths_internal(&mut paths);
        paths
    }

    fn all_model_paths_internal(&self, paths: &mut Vec<ModelPath>) {
        paths.push(self.model.path.clone());

        for reference_model in self.references().values() {
            reference_model.all_model_paths_internal(paths);
        }
    }
}

/// A reference to a lowered template model within a model hierarchy.
///
/// Wraps an [`InstancedModel`] template from the unit graph cache together with
/// a handle to the cache so child references can be followed transitively.
#[derive(Debug, Clone, Copy)]
pub struct ModelTemplateReference<'runtime> {
    model: &'runtime InstancedModel,
    unit_graph_cache: &'runtime UnitGraphCache,
}

impl<'runtime> ModelTemplateReference<'runtime> {
    /// Creates a new `ModelTemplateReference` for the given model and unit graph cache.
    #[must_use]
    pub const fn new(
        model: &'runtime InstancedModel,
        unit_graph_cache: &'runtime UnitGraphCache,
    ) -> Self {
        Self {
            model,
            unit_graph_cache,
        }
    }

    /// Returns the path of this model.
    #[must_use]
    pub const fn path(&self) -> &'runtime ModelPath {
        self.model.path()
    }

    /// Returns the optional model-level documentation note.
    #[must_use]
    pub const fn note(&self) -> Option<&'runtime ir::Note> {
        self.model.note()
    }

    /// Returns a map of submodel aliases (= reference names) to their
    /// `SubmodelImport`s.
    ///
    /// If you need the model reference itself, use `submodel_models` instead.
    #[must_use]
    pub fn submodel_imports(&self) -> IndexMap<&'runtime ReferenceName, &'runtime SubmodelImport> {
        self.model.submodels().iter().collect()
    }

    /// Returns a map of submodel aliases (= reference names) to their
    /// template model references.
    #[must_use]
    pub fn submodel_models(
        &self,
    ) -> IndexMap<&'runtime ReferenceName, SubmodelImportReference<'runtime>> {
        self.model
            .submodels()
            .iter()
            .map(|(alias, submodel_import)| {
                (
                    alias,
                    SubmodelImportReference::new(alias, submodel_import, self.unit_graph_cache),
                )
            })
            .collect()
    }

    /// Returns a map of reference names to their `ReferenceImport`s.
    ///
    /// If you need the model reference itself, use `reference_models` instead.
    #[must_use]
    pub fn reference_imports(
        &self,
    ) -> IndexMap<&'runtime ReferenceName, &'runtime ReferenceImport> {
        self.model.references().iter().collect()
    }

    /// Returns a map of extracted alias names to their `AliasImport`s.
    #[must_use]
    pub fn alias_imports(&self) -> IndexMap<&'runtime ReferenceName, &'runtime AliasImport> {
        self.model.aliases().iter().collect()
    }

    /// Returns a map of reference names to their template model references.
    #[must_use]
    pub fn reference_models(
        &self,
    ) -> IndexMap<&'runtime ReferenceName, ReferenceImportReference<'runtime>> {
        self.model
            .references()
            .iter()
            .map(|(name, reference_import)| {
                (
                    name,
                    ReferenceImportReference::new(name, reference_import, self.unit_graph_cache),
                )
            })
            .collect()
    }

    /// Returns a map of parameter names to their parameter data.
    #[must_use]
    pub fn parameters(&self) -> IndexMap<&'runtime ParameterName, &'runtime ir::Parameter> {
        self.model.parameters().iter().collect()
    }

    /// Returns a parameter by its name.
    #[must_use]
    pub fn get_parameter(&self, name: &ParameterName) -> Option<&'runtime ir::Parameter> {
        self.model.parameters().get(name)
    }

    /// Returns the list of tests for this model.
    #[must_use]
    pub const fn tests(&self) -> &'runtime IndexMap<TestIndex, ir::Test> {
        self.model.tests()
    }

    /// Returns the Python imports for this model.
    #[must_use]
    pub const fn python_imports(&self) -> &'runtime IndexMap<PythonPath, ir::PythonImport> {
        self.model.python_imports()
    }
}

/// A reference to a submodel import within a model.
#[derive(Debug, Clone, Copy)]
pub struct SubmodelImportReference<'runtime> {
    alias: &'runtime ReferenceName,
    submodel_import: &'runtime SubmodelImport,
    unit_graph_cache: &'runtime UnitGraphCache,
}

impl<'runtime> SubmodelImportReference<'runtime> {
    /// Creates a new `SubmodelImportReference` for the given submodel import and unit graph cache.
    #[must_use]
    pub const fn new(
        alias: &'runtime ReferenceName,
        submodel_import: &'runtime SubmodelImport,
        unit_graph_cache: &'runtime UnitGraphCache,
    ) -> Self {
        Self {
            alias,
            submodel_import,
            unit_graph_cache,
        }
    }

    /// Returns the source-level name of the submodel
    /// (`foo` in `submodel foo as bar`).
    #[must_use]
    pub const fn name(&self) -> &'runtime SubmodelName {
        &self.submodel_import.name
    }

    /// Returns the span of the source-level name of the submodel.
    #[must_use]
    pub const fn name_span(&self) -> &'runtime Span {
        &self.submodel_import.name_span
    }

    /// Returns the explicit `as` alias, if the declaration includes one.
    #[must_use]
    pub const fn alias(&self) -> Option<&'runtime ReferenceName> {
        self.submodel_import.alias.as_ref()
    }

    /// Returns the span of the explicit `as` alias, if the declaration includes one.
    #[must_use]
    pub const fn alias_span(&self) -> Option<&'runtime Span> {
        self.submodel_import.alias_span.as_ref()
    }

    /// Returns the alias under which the submodel is bound in the parent
    /// (`bar` in `submodel foo as bar`).
    #[must_use]
    pub const fn reference_name(&self) -> &'runtime ReferenceName {
        self.alias
    }

    /// Returns the file path of the submodel.
    #[must_use]
    pub fn path(&self) -> &'runtime ModelPath {
        self.submodel_import.instance.path()
    }

    /// Returns a [`ModelTemplateReference`] navigating the submodel's
    /// embedded subtree.
    #[must_use]
    pub fn model(&self) -> ModelTemplateReference<'runtime> {
        ModelTemplateReference::new(
            self.submodel_import.instance.as_ref(),
            self.unit_graph_cache,
        )
    }
}

/// A reference to a reference import within a model.
#[derive(Debug, Clone, Copy)]
pub struct ReferenceImportReference<'runtime> {
    name: &'runtime ReferenceName,
    reference_import: &'runtime ReferenceImport,
    unit_graph_cache: &'runtime UnitGraphCache,
}

impl<'runtime> ReferenceImportReference<'runtime> {
    /// Creates a new `ReferenceImportReference` for the given reference import and unit graph cache.
    #[must_use]
    pub const fn new(
        name: &'runtime ReferenceName,
        reference_import: &'runtime ReferenceImport,
        unit_graph_cache: &'runtime UnitGraphCache,
    ) -> Self {
        Self {
            name,
            reference_import,
            unit_graph_cache,
        }
    }

    /// Returns the alias under which the referenced model is bound.
    #[must_use]
    pub const fn name(&self) -> &'runtime ReferenceName {
        self.name
    }

    /// Returns the span of the name of the reference.
    #[must_use]
    pub const fn name_span(&self) -> &'runtime Span {
        &self.reference_import.name_span
    }

    /// Returns the explicit `as` alias, if the declaration includes one.
    #[must_use]
    pub const fn alias(&self) -> Option<&'runtime ReferenceName> {
        self.reference_import.alias.as_ref()
    }

    /// Returns the span of the explicit `as` alias, if the declaration includes one.
    #[must_use]
    pub const fn alias_span(&self) -> Option<&'runtime Span> {
        self.reference_import.alias_span.as_ref()
    }

    /// Returns the path of the reference.
    #[must_use]
    pub const fn path(&self) -> &'runtime ModelPath {
        &self.reference_import.path
    }

    /// Returns the model that this reference imports. If the referenced model
    /// failed to resolve or has not been loaded, returns `None`.
    #[must_use]
    pub fn model(&self) -> Option<ModelTemplateReference<'runtime>> {
        let path = &self.reference_import.path;
        let graph = self
            .unit_graph_cache
            .get(&CompilationUnit::Model(path.clone()))?;
        Some(ModelTemplateReference::new(
            graph.root.as_ref(),
            self.unit_graph_cache,
        ))
    }
}
