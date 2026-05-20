//! Template loading and resolution for the runtime.

use indexmap::{IndexMap, IndexSet};
use oneil_frontend::{
    self as frontend, CompilationUnit, InstancedModel, ResolutionErrorCollection,
    build_unit_graph_for, collect_design_target_path, error::VariableResolutionError,
};
use oneil_shared::{
    load_result::LoadResult,
    paths::{DesignPath, ModelPath, PythonPath},
    symbols::{BuiltinFunctionName, BuiltinValueName, PyFunctionName, UnitBaseName, UnitPrefix},
};

use super::Runtime;
use crate::output::{self, ast, error::RuntimeErrors};

impl Runtime {
    /// Loads and lowers a model and all its dependencies into [`InstancedModel`] templates.
    ///
    /// Returns a reference to the lowered template so callers can inspect the
    /// file-static structure (parameters, references, types) for hover /
    /// definition / debug-IR flows.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeErrors`] if the model had parse or resolution errors.
    pub fn load_and_lower(
        &mut self,
        path: &ModelPath,
    ) -> (
        Option<output::reference::ModelTemplateReference<'_>>,
        RuntimeErrors,
    ) {
        self.load_and_lower_internal(path);

        let template_opt = self
            .unit_graph_cache
            .get(&CompilationUnit::Model(path.clone()))
            .map(|graph| {
                output::reference::ModelTemplateReference::new(
                    graph.root.as_ref(),
                    &self.unit_graph_cache,
                )
            });

        let include_indirect_errors = true;
        let errors = self.get_model_diagnostics(path, include_indirect_errors);

        (template_opt, errors)
    }

    pub(super) fn load_and_lower_internal(&mut self, path: &ModelPath) {
        let results = frontend::load_model(path, self);

        // Build local maps for the newly resolved files. Previously loaded
        // models are in `unit_graph_cache` and hit the cache during
        // `build_unit_graph_inner` without consulting these maps.
        let mut local_templates: IndexMap<
            ModelPath,
            LoadResult<InstancedModel, frontend::ResolutionErrorCollection>,
        > = IndexMap::new();
        let mut local_design_info: IndexMap<ModelPath, frontend::ModelDesignInfo> = IndexMap::new();

        for (model_path, result) in results {
            let (model, design_export, applied_designs, model_errors) = result.into_parts();

            local_design_info.insert(
                model_path.clone(),
                frontend::ModelDesignInfo {
                    applied_designs,
                    design_export,
                },
            );

            let load_result = if model_errors.is_empty() {
                LoadResult::success(model)
            } else {
                LoadResult::partial(model, model_errors)
            };
            local_templates.insert(model_path, load_result);
        }

        // Eagerly build and cache unit graphs for all models returned by
        // `load_model`. `load_model` uses a fresh `ResolutionContext` each
        // call and therefore always re-resolves the full transitive closure of
        // `path`, returning every dependency in `results`. Iterating all of
        // them (rather than just `path`) keeps the unit graph cache consistent
        // with the latest resolution pass. This is especially important for
        // design files: a design file's unit graph build does not recurse into
        // target models, so building only the design path's graph would leave
        // the target model's unit graph stale or absent.
        // Merge the newly resolved design info into the persistent map so it
        // is available to `apply_designs` at composition time.
        self.design_info.extend(local_design_info.clone());

        let model_paths: Vec<ModelPath> = local_templates.keys().cloned().collect();
        let mut cache = std::mem::take(&mut self.unit_graph_cache);
        for model_path in &model_paths {
            frontend::build_unit_graph(
                model_path,
                &mut cache,
                &mut Vec::new(),
                &local_templates,
                &local_design_info,
            );
            // For design files also build the `CompilationUnit::Design` cache
            // entry. `build_unit_graph` above only inserts
            // `CompilationUnit::Model`, but `compose`'s runtime-design loop
            // looks up by `CompilationUnit::Design` to find `design_export`.
            if let Ok(design_path) = DesignPath::try_from(model_path.clone()) {
                build_unit_graph_for(
                    &CompilationUnit::Design(design_path),
                    &mut cache,
                    &mut Vec::new(),
                    &local_templates,
                    &local_design_info,
                );
            }
        }
        self.unit_graph_cache = cache;
    }

    /// Returns the target [`ModelPath`] declared by a `design <target>` line in
    /// `design_path`, or `None` if the file has no such declaration or could
    /// not be parsed.
    fn get_design_target(&mut self, design_path: &DesignPath) -> Option<ModelPath> {
        let model_path = design_path.to_model_path();
        let ast = self.load_ast_internal(&model_path);
        let ast = ast.value()?;
        collect_design_target_path(&model_path, ast)
    }

    /// Resolves a path that may be a design file to its target model and design.
    ///
    /// If `path` is a `.one` design file that declares `design <target>`, returns
    /// `(target_model_path, Some(design_path))` so the design will be applied.
    /// Otherwise returns `(path, None)` unchanged.
    ///
    /// This is the single source of truth for design file detection, used by
    /// all entry points (`eval_model`, `check_model`, etc.) to ensure consistent
    /// handling of design files across CLI, LSP, and other consumers.
    pub(super) fn resolve_design_redirect(
        &mut self,
        path: ModelPath,
    ) -> (ModelPath, Option<DesignPath>) {
        if let Ok(design_path) = DesignPath::try_from(path.clone())
            && let Some(target) = self.get_design_target(&design_path)
        {
            return (target, Some(design_path));
        }
        (path, None)
    }

    /// Resolves an expression as if it were in the context of the given model.
    pub(super) fn resolve_expr_in_model(
        &mut self,
        expr_ast: &ast::ExprNode,
        model_path: &ModelPath,
    ) -> Result<output::ir::Expr, Vec<VariableResolutionError>> {
        frontend::resolve_expr_in_model(expr_ast, model_path, self)
    }
}

impl frontend::ExternalResolutionContext for Runtime {
    fn has_builtin_value(&self, identifier: &ast::Identifier) -> bool {
        self.builtins.has_builtin_value(identifier.as_str())
    }

    fn has_builtin_function(&self, identifier: &ast::Identifier) -> bool {
        self.builtins.has_builtin_function(identifier.as_str())
    }

    fn get_builtin_value_names(&self) -> impl Iterator<Item = &BuiltinValueName> {
        self.builtins.builtin_values().map(|(name, _)| name)
    }

    fn get_builtin_function_names(&self) -> impl Iterator<Item = &BuiltinFunctionName> {
        self.builtins.builtin_functions().map(|(name, _)| name)
    }

    fn has_builtin_unit(&self, name: &str) -> bool {
        let name = UnitBaseName::from(name);
        self.builtins.get_unit(&name).is_some()
    }

    fn available_prefixes(&self) -> impl Iterator<Item = (&UnitPrefix, f64)> {
        self.builtins.builtin_prefixes()
    }

    fn unit_supports_si_prefixes(&self, name: &UnitBaseName) -> bool {
        self.builtins.unit_supports_si_prefixes(name)
    }

    fn lookup_unit(&self, name: &UnitBaseName) -> Option<&output::Unit> {
        self.builtins.get_unit(name)
    }

    fn load_ast(
        &mut self,
        path: &ModelPath,
    ) -> LoadResult<&ast::ModelNode, frontend::AstLoadingFailedError> {
        self.load_ast_internal(path)
            .as_ref()
            .map_err(|_e| frontend::AstLoadingFailedError)
    }

    fn load_python_import<'context>(
        &'context mut self,
        python_path: &PythonPath,
    ) -> Result<IndexSet<&'context PyFunctionName>, frontend::PythonImportLoadingFailedError> {
        self.load_python_import_internal(python_path)
            .as_ref()
            .ok()
            .map(|functions| functions.get_function_names().collect())
            .ok_or(frontend::PythonImportLoadingFailedError)
    }

    fn get_preloaded_models(
        &self,
    ) -> impl Iterator<Item = (ModelPath, InstancedModel, ResolutionErrorCollection)> {
        self.unit_graph_cache.iter().filter_map(|(unit, graph)| {
            let CompilationUnit::Model(path) = unit else {
                return None;
            };
            let model = (*graph.root).clone();
            let errors = graph
                .resolution_errors
                .get(path)
                .cloned()
                .unwrap_or_else(ResolutionErrorCollection::empty);
            Some((path.clone(), model, errors))
        })
    }
}
