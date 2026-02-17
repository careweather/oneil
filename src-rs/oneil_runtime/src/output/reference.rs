//! References into evaluation results and error contexts.
//!
//! Provides [`ModelReference`] for navigating evaluated models,
//! [`EvalErrorReference`] for inspecting evaluation failures,
//! [`ModelIrReference`] for navigating resolved IR models, and
//! [`ResolutionErrorReference`] for inspecting resolution failures.

use std::path::Path;

use crate::output;
use indexmap::IndexMap;
use oneil_ir as ir;
use oneil_shared::span::Span;

use crate::cache::{EvalCache, IrCache};

/// A reference to an evaluated model within a model hierarchy.
///
/// This stores a reference to a model and a reference to the
/// entire model collection.
#[derive(Debug, Clone, Copy)]
pub struct ModelReference<'result> {
    model: &'result output::Model,
    eval_cache: &'result EvalCache,
}

impl<'result> ModelReference<'result> {
    /// Creates a new `ModelReference` for the given model and evaluation cache.
    #[must_use]
    pub const fn new(model: &'result output::Model, eval_cache: &'result EvalCache) -> Self {
        Self { model, eval_cache }
    }

    /// Returns the file path of this model.
    #[must_use]
    pub fn path(&self) -> &'result Path {
        self.model.path.as_path()
    }

    /// Returns a map of submodel names to their model references or evaluation errors.
    ///
    /// # Panics
    ///
    /// Panics if any submodel has not been visited and
    /// added to the model collection. This should never be
    /// the case as long as creating the `EvalResult`
    /// resolves successfully.
    #[must_use]
    pub fn submodels(&self) -> IndexMap<&'result str, &'result str> {
        self.model
            .submodels
            .iter()
            .map(|(name, reference_name)| (name.as_str(), reference_name.as_str()))
            .collect()
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
    pub fn references(&self) -> IndexMap<&'result str, Self> {
        self.model
            .references
            .iter()
            .filter_map(|(name, path)| {
                let entry = self
                    .eval_cache
                    .get_entry(path)
                    .expect("reference should be in cache");

                let model = entry.value()?;

                let result = Self {
                    model,
                    eval_cache: self.eval_cache,
                };

                Some((name.as_str(), result))
            })
            .collect()
    }

    /// Returns a map of parameter names to their evaluated parameter data.
    #[must_use]
    pub fn parameters(&self) -> IndexMap<&'result str, &'result output::Parameter> {
        self.model
            .parameters
            .iter()
            .map(|(name, parameter)| (name.as_str(), parameter))
            .collect()
    }

    /// Returns the list of evaluated test results for this model.
    #[must_use]
    pub fn tests(&self) -> Vec<&'result output::Test> {
        self.model.tests.iter().collect()
    }
}

/// A reference to a resolved IR model within a model hierarchy.
///
/// This stores a reference to an IR model, the path it was loaded from,
/// and a reference to the IR cache.
#[derive(Debug, Clone, Copy)]
pub struct ModelIrReference<'result> {
    model: &'result ir::Model,
    ir_cache: &'result IrCache,
}

impl<'result> ModelIrReference<'result> {
    /// Creates a new `ModelIrReference` for the given model, IR cache, and path.
    #[must_use]
    pub const fn new(model: &'result ir::Model, ir_cache: &'result IrCache) -> Self {
        Self { model, ir_cache }
    }

    /// Returns the path of this model.
    #[must_use]
    pub const fn path(&self) -> &'result ir::ModelPath {
        self.model.get_path()
    }

    /// Returns a map of submodel names to their IR model references or resolution errors.
    ///
    /// # Panics
    ///
    /// Panics if any submodel's reference has not been visited and
    /// added to the IR cache.
    #[must_use]
    pub fn submodels(
        &self,
    ) -> IndexMap<&'result ir::SubmodelName, SubmodelImportReference<'result>> {
        self.model
            .get_submodels()
            .iter()
            .map(|(name, submodel_import)| {
                (
                    name,
                    SubmodelImportReference::new(
                        submodel_import,
                        self.ir_cache,
                        self.model.get_references(),
                    ),
                )
            })
            .collect()
    }

    /// Returns a map of reference names to their IR model references or resolution errors.
    ///
    /// # Panics
    ///
    /// Panics if any reference has not been visited and
    /// added to the IR cache.
    #[must_use]
    pub fn references(
        &self,
    ) -> IndexMap<&'result ir::ReferenceName, ReferenceImportReference<'result>> {
        self.model
            .get_references()
            .iter()
            .map(|(name, reference_import)| {
                (
                    name,
                    ReferenceImportReference::new(reference_import, self.ir_cache),
                )
            })
            .collect()
    }

    /// Returns a map of parameter names to their parameter data.
    #[must_use]
    pub fn parameters(&self) -> IndexMap<&'result ir::ParameterName, &'result ir::Parameter> {
        self.model.get_parameters().iter().collect()
    }

    /// Returns a parameter by its name.
    #[must_use]
    pub fn get_parameter(&self, name: &ir::ParameterName) -> Option<&'result ir::Parameter> {
        self.model.get_parameters().get(name)
    }

    /// Returns the list of tests for this model.
    #[must_use]
    pub fn tests(&self) -> Vec<&'result ir::Test> {
        self.model.get_tests().values().collect()
    }
}

/// A reference to a submodel import within a model.
#[derive(Debug, Clone, Copy)]
pub struct SubmodelImportReference<'result> {
    submodel_import: &'result ir::SubmodelImport,
    ir_cache: &'result IrCache,
    references: &'result IndexMap<ir::ReferenceName, ir::ReferenceImport>,
}

impl<'result> SubmodelImportReference<'result> {
    /// Creates a new `SubmodelImportReference` for the given submodel import and IR cache.
    #[must_use]
    pub const fn new(
        submodel_import: &'result ir::SubmodelImport,
        ir_cache: &'result IrCache,
        references: &'result IndexMap<ir::ReferenceName, ir::ReferenceImport>,
    ) -> Self {
        Self {
            submodel_import,
            ir_cache,
            references,
        }
    }

    /// Returns the name of the submodel.
    #[must_use]
    pub const fn name(&self) -> &'result ir::SubmodelName {
        self.submodel_import.name()
    }

    /// Returns the span of the name of the submodel.
    #[must_use]
    pub const fn name_span(&self) -> &'result Span {
        self.submodel_import.name_span()
    }

    /// Returns the reference name of the submodel.
    #[must_use]
    pub const fn reference_name(&self) -> &'result ir::ReferenceName {
        self.submodel_import.reference_name()
    }

    /// Returns the reference import of the submodel.
    #[expect(
        clippy::missing_panics_doc,
        reason = "the panic only happens if an internal invariant is violated"
    )]
    #[must_use]
    pub fn reference_import(&self) -> ReferenceImportReference<'result> {
        let reference_name = self.submodel_import.reference_name();
        let reference_import = self
            .references
            .get(reference_name)
            .expect("reference should be found");

        ReferenceImportReference::new(reference_import, self.ir_cache)
    }
}

/// A reference to a reference import within a model.
#[derive(Debug, Clone, Copy)]
pub struct ReferenceImportReference<'result> {
    reference_import: &'result ir::ReferenceImport,
    ir_cache: &'result IrCache,
}

impl<'result> ReferenceImportReference<'result> {
    /// Creates a new `ReferenceImportReference` for the given reference import and IR cache.
    #[must_use]
    pub const fn new(
        reference_import: &'result ir::ReferenceImport,
        ir_cache: &'result IrCache,
    ) -> Self {
        Self {
            reference_import,
            ir_cache,
        }
    }

    /// Returns the name of the reference.
    #[must_use]
    pub const fn name(&self) -> &'result ir::ReferenceName {
        self.reference_import.name()
    }

    /// Returns the span of the name of the reference.
    #[must_use]
    pub const fn name_span(&self) -> &'result Span {
        self.reference_import.name_span()
    }

    /// Returns the path of the reference.
    #[must_use]
    pub const fn path(&self) -> &'result ir::ModelPath {
        self.reference_import.path()
    }

    /// Returns the model that this reference imports. If the referenced model
    /// failed to resolve, returns `None`.
    #[expect(
        clippy::missing_panics_doc,
        reason = "the panic only happens if an internal invariant is violated"
    )]
    #[must_use]
    pub fn model(&self) -> Option<ModelIrReference<'result>> {
        let entry = self
            .ir_cache
            .get_entry(self.reference_import.path().as_ref())
            .expect("reference should be in cache");

        let ir = entry.value()?;

        Some(ModelIrReference::new(ir, self.ir_cache))
    }
}
