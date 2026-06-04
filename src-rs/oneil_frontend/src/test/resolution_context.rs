//! Builder for [`ResolutionContext`] in tests.
//!
//! Because [`ResolutionContext`] holds a mutable reference to an
//! [`ExternalResolutionContext`], it cannot be constructed and returned.
//! This builder stores the desired configuration and provides a
//! [`build`](ResolutionContextBuilder::build) method to create the context
//! and populate it.

use std::path::PathBuf;

use oneil_ir as ir;
use oneil_shared::{
    paths::{ModelPath, PythonPath},
    symbols::{ParameterName, ReferenceName},
};

use crate::{
    ResolutionContext,
    error::{ModelImportResolutionError, ParameterResolutionError, VariableResolutionError},
    instance::InstancedModel,
    test::external_context::TestExternalContext,
};

use super::unimportant_span;

/// Builder for a test [`ResolutionContext`].
///
/// Configures one active model with optional parameters, references
/// (and their target models), parameter errors, reference resolution errors,
/// and model paths to mark as having errors. Use [`build`](Self::build) to
/// create the context.
///
/// For model-import tests, use [`with_models`](Self::with_models) to
/// pre-register models (e.g. so `lookup_model` finds them) without adding
/// references from the active model.
#[derive(Debug, Default)]
pub struct ResolutionContextBuilder<'external> {
    active_model_path: Option<ModelPath>,
    /// Models to pre-register so that `lookup_model` finds them.
    models: Vec<(ModelPath, InstancedModel)>,
    parameters: Vec<ir::Parameter>,
    references: Vec<(ReferenceName, ModelPath, InstancedModel)>,
    parameter_error_names: Vec<ParameterName>,
    reference_errors: Vec<(ReferenceName, ModelImportResolutionError)>,
    model_error_paths: Vec<ModelPath>,
    python_import_paths: Vec<PathBuf>,
    external_context: Option<&'external mut TestExternalContext>,
}

impl<'external> ResolutionContextBuilder<'external> {
    /// Creates a new empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the single active model path.
    ///
    /// Exactly one active model is pushed; this is required before
    /// parameters, references, or errors are meaningful.
    #[must_use]
    pub fn with_active_model(mut self, path: ModelPath) -> Self {
        self.active_model_path = Some(path);
        self
    }

    /// Pre-registers models so that `lookup_model` finds them.
    ///
    /// Does not add any references from the active model.
    #[must_use]
    pub fn with_models(
        mut self,
        models: impl IntoIterator<Item = (ModelPath, InstancedModel)>,
    ) -> Self {
        self.models.extend(models);
        self
    }

    /// Adds parameters to the active model.
    #[must_use]
    pub fn with_parameters(mut self, parameters: impl IntoIterator<Item = ir::Parameter>) -> Self {
        self.parameters.extend(parameters);
        self
    }

    /// Marks the given parameter names as having resolution errors on the active model.
    #[must_use]
    pub fn with_parameter_errors(mut self, names: impl IntoIterator<Item = ParameterName>) -> Self {
        self.parameter_error_names.extend(names);
        self
    }

    /// Marks the given model paths as having errors.
    #[must_use]
    pub fn with_model_errors(mut self, paths: impl IntoIterator<Item = ModelPath>) -> Self {
        self.model_error_paths.extend(paths);
        self
    }

    /// Sets the external context.
    #[must_use]
    pub fn with_external_context(mut self, external: &'external mut TestExternalContext) -> Self {
        self.external_context = Some(external);
        self
    }

    /// Adds Python import paths to load into the active model during build.
    ///
    /// For each path, [`load_python_import_to_active_model`](ResolutionContext::load_python_import_to_active_model)
    /// is called, so the external context must have the path registered (e.g. via
    /// [`with_python_imports_ok`](TestExternalContext::with_python_imports_ok) and
    /// [`with_python_import_functions`](TestExternalContext::with_python_import_functions)).
    #[must_use]
    pub fn with_python_import_paths(
        mut self,
        paths: impl IntoIterator<Item = impl AsRef<std::path::Path>>,
    ) -> Self {
        self.python_import_paths
            .extend(paths.into_iter().map(|p| p.as_ref().to_path_buf()));
        self
    }

    #[expect(
        clippy::too_many_lines,
        reason = "logic is naturally split up into sections"
    )]
    pub fn build(self) -> ResolutionContext<'external, TestExternalContext> {
        let active_path = self
            .active_model_path
            .as_ref()
            .expect("ResolutionContextBuilder: cannot build without an active model");

        let external = self
            .external_context
            .expect("ResolutionContextBuilder: cannot build without an external context");

        let mut ctx = ResolutionContext::new(external);

        for (path, model) in &self.models {
            ctx.push_active_model(path);
            for (name, p) in model.parameters() {
                ctx.add_parameter_to_active_model(name.clone(), p.clone());
            }
            for (name, import) in model.references() {
                ctx.add_reference_to_active_model(
                    name.clone(),
                    import.name_span.clone(),
                    import.alias.clone(),
                    import.alias_span.clone(),
                    import.path.clone(),
                );
            }
            for (alias, import) in model.submodels() {
                ctx.add_submodel_to_active_model(
                    alias.clone(),
                    import.name.clone(),
                    import.name_span.clone(),
                    import.alias.clone(),
                    import.alias_span.clone(),
                    import.instance.path().clone(),
                );
            }
            ctx.pop_active_model(path);
        }

        ctx.push_active_model(active_path);

        for param in &self.parameters {
            let name = param.name().clone();
            ctx.add_parameter_to_active_model(name, param.clone());
        }

        for (ref_name, ref_path, ref_model) in &self.references {
            ctx.push_active_model(ref_path);
            for (name, p) in ref_model.parameters() {
                ctx.add_parameter_to_active_model(name.clone(), p.clone());
            }
            for (name, import) in ref_model.references() {
                ctx.add_reference_to_active_model(
                    name.clone(),
                    import.name_span.clone(),
                    import.alias.clone(),
                    import.alias_span.clone(),
                    import.path.clone(),
                );
            }
            for (alias, import) in ref_model.submodels() {
                ctx.add_submodel_to_active_model(
                    alias.clone(),
                    import.name.clone(),
                    import.name_span.clone(),
                    import.alias.clone(),
                    import.alias_span.clone(),
                    import.instance.path().clone(),
                );
            }
            ctx.pop_active_model(ref_path);
            ctx.add_reference_to_active_model(
                ref_name.clone(),
                unimportant_span(),
                None,
                None,
                ref_path.clone(),
            );
        }

        for name in &self.parameter_error_names {
            let span = unimportant_span();
            let err = ParameterResolutionError::variable_resolution(
                VariableResolutionError::undefined_parameter(name.clone(), span, None),
            );
            ctx.add_parameter_error_to_active_model(name.clone(), err);
        }

        for (ref_name, err) in &self.reference_errors {
            ctx.add_model_import_resolution_error_to_active_model(
                ref_name.clone(),
                None,
                err.clone(),
            );
        }

        for path in &self.model_error_paths {
            ctx.push_active_model(path);
            let dummy_name = ParameterName::from("__error__");
            let span = unimportant_span();
            let err = ParameterResolutionError::variable_resolution(
                VariableResolutionError::undefined_parameter(dummy_name.clone(), span, None),
            );
            ctx.add_parameter_error_to_active_model(dummy_name, err);
            ctx.pop_active_model(path);
        }

        for path in &self.python_import_paths {
            let path_str = path.to_string_lossy().to_string();
            let python_path = PythonPath::from_str_no_ext(&path_str);
            ctx.load_python_import_to_active_model(&python_path, unimportant_span());
        }

        ctx
    }
}
