//! Model evaluation for the runtime.

use std::path::PathBuf;

use indexmap::IndexMap;
use oneil_analysis::validate_instance_graph;
use oneil_eval as eval;
#[cfg(feature = "python")]
use oneil_eval::CallsiteInfo;
use oneil_frontend::{
    ApplyDesign, CompilationUnit, InstanceGraph, InstancedModel, ResolutionErrorCollection,
    apply_designs,
};
use oneil_output::{EvalError, Unit, Value};
use oneil_shared::{
    EvalInstanceKey,
    error::OneilDiagnostic,
    load_result::LoadResult,
    paths::{DesignPath, ModelPath},
    span::Span,
    symbols::{BuiltinFunctionName, BuiltinValueName, UnitBaseName, UnitPrefix},
};
#[cfg(feature = "python")]
use oneil_shared::{paths::PythonPath, symbols::PyFunctionName};

use super::{Runtime, RuntimeBuiltinLookup};
#[cfg(feature = "python")]
use crate::cache::PythonCallCache;
use crate::output::{self, error::RuntimeErrors};

type EvalModelAndExpressionsResult<'runtime, 'expr> = (
    Option<(
        output::reference::ModelReference<'runtime>,
        IndexMap<&'expr str, Value>,
    )>,
    RuntimeErrors,
    Vec<OneilDiagnostic>,
);

impl Runtime {
    /// Evaluates a model or design file.
    ///
    /// If `path` is a `.one` design file that declares `design <target>`, the
    /// target model is evaluated with the design applied. Otherwise the model
    /// at `path` is evaluated directly.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeErrors`] if the model or design file could not be evaluated.
    pub fn eval_model(
        &mut self,
        path: &ModelPath,
    ) -> (Option<output::reference::ModelReference<'_>>, RuntimeErrors) {
        let (eval_path, design_path) = self.resolve_design_redirect(path.clone());

        // Evaluate the model (with optional design) - populates caches
        self.eval_model_internal(&eval_path, design_path.as_ref());

        // Look up the model reference from cache
        let model_opt = self
            .eval_cache
            .get_entry(&eval_path)
            .and_then(LoadResult::value)
            .map(|model| output::reference::ModelReference::new(model, &self.eval_cache));

        let include_indirect_errors = true;
        let mut errors = self.get_model_diagnostics(&eval_path, include_indirect_errors);

        // Also include design file errors if present
        if let Some(design_path) = &design_path {
            let design_errors =
                self.get_model_diagnostics(&design_path.to_model_path(), include_indirect_errors);
            errors.extend(design_errors);
        }

        (model_opt, errors)
    }

    /// Evaluates a model or design file and a list of expressions in the context of
    /// the model.
    ///
    /// If `path` is a `.one` design file that declares `design <target>`, the
    /// target model is evaluated with the design applied. Otherwise the model
    /// at `path` is evaluated directly.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeErrors`] (via [`get_model_diagnostics`](super::Runtime::get_model_diagnostics)) if the model could not be evaluated.
    /// Returns [`OneilDiagnostic`]s if the expressions could not be evaluated.
    pub fn eval_model_and_expressions<'runtime, 'expr>(
        &'runtime mut self,
        path: &ModelPath,
        expressions: &'expr [String],
    ) -> EvalModelAndExpressionsResult<'runtime, 'expr> {
        let (eval_path, design_path) = self.resolve_design_redirect(path.clone());

        // Evaluate the model (with optional design) - populates caches
        self.eval_model_internal(&eval_path, design_path.as_ref());

        // Evaluate the expressions
        let (expr_results, expr_errors) = self.eval_expressions_internal(expressions, &eval_path);

        // Look up the model reference from cache
        let model_opt = self
            .eval_cache
            .get_entry(&eval_path)
            .and_then(LoadResult::value)
            .map(|model| output::reference::ModelReference::new(model, &self.eval_cache));

        let result = model_opt.map(|model| (model, expr_results));

        let include_indirect_errors = true;
        let mut model_errors = self.get_model_diagnostics(&eval_path, include_indirect_errors);

        // Also include design file errors if present
        if let Some(design_path) = &design_path {
            let design_errors =
                self.get_model_diagnostics(&design_path.to_model_path(), include_indirect_errors);
            model_errors.extend(design_errors);
        }

        (result, model_errors, expr_errors)
    }

    /// Checks a model or design file without evaluating its parameters.
    ///
    /// If `path` is a `.one` design file that declares `design <target>`, the
    /// target model is checked with the design applied. Otherwise the model
    /// at `path` is checked directly.
    ///
    /// This is the cheap diagnostic-only entry point used by IDE / CLI
    /// surfaces that want to surface parse, IR, composition, and
    /// post-build validation errors without paying for the lazy eval
    /// pass (or the panicking failure modes it has on broken IR).
    ///
    /// Goes through the same compose path as [`Self::eval_model`] (loads IR,
    /// applies any design at the root, splices through the per-unit graph
    /// cache, runs `validate_instance_graph`), stashes the composed graph on
    /// `self` so [`Self::get_model_diagnostics`] can pull per-instance
    /// diagnostics off it, and returns:
    ///
    /// - The unique model paths visited in the composed graph (so
    ///   callers can clear stale diagnostics on every file the
    ///   composition touched, mirroring `eval_model`'s
    ///   `result.all_model_paths()`).
    /// - The aggregated `RuntimeErrors` for the queried path,
    ///   including indirect / chain-walked errors and (if a design
    ///   file was involved) the design file's own errors.
    ///
    /// Eval-time errors (numeric overflow, piecewise miss, runtime
    /// cycles via Python) are *not* surfaced here — those still
    /// require a full [`Self::eval_model`] run.
    pub fn check_model(&mut self, path: &ModelPath) -> (Vec<ModelPath>, RuntimeErrors) {
        let (check_path, design_path) = self.resolve_design_redirect(path.clone());

        self.check_model_internal(&check_path, design_path.as_ref());

        let mut visited_paths: Vec<ModelPath> = self
            .composed_graph
            .as_ref()
            .map(|graph| {
                let mut seen: indexmap::IndexSet<ModelPath> = indexmap::IndexSet::new();
                collect_visited_paths(graph.root.as_ref(), &mut seen);
                for instance in graph.reference_pool.values() {
                    collect_visited_paths(instance.as_ref(), &mut seen);
                }
                seen.into_iter().collect()
            })
            .unwrap_or_default();

        // Include the design file path in visited paths so its diagnostics get cleared
        if let Some(design_path) = &design_path {
            let design_model_path = design_path.to_model_path();
            if !visited_paths.contains(&design_model_path) {
                visited_paths.push(design_model_path);
            }
        }

        let include_indirect_errors = true;
        let mut errors = self.get_model_diagnostics(&check_path, include_indirect_errors);

        // Include design file errors - design files are their own model-path
        // keyed bucket in the IR / source caches, so their diagnostics aren't
        // reachable through the root model's recursive walk.
        if let Some(design_path) = &design_path {
            let design_errors =
                self.get_model_diagnostics(&design_path.to_model_path(), include_indirect_errors);
            errors.extend(design_errors);
        }

        (visited_paths, errors)
    }

    /// Internal "compose without eval" that mirrors the prefix of
    /// [`Self::eval_model_internal`]: load IR (root + optional
    /// design), compose through the unit-graph cache with the
    /// runtime-supplied design at the root, run the post-build
    /// validation pass, and stash the composed graph on `self` so
    /// error collection can read off it. Skips the lazy
    /// `eval::eval_model_from_graph` step and the `eval_cache`
    /// writes that produce eval-time errors.
    fn check_model_internal(&mut self, path: &ModelPath, design_path: Option<&DesignPath>) {
        self.load_and_lower_internal(path);

        let runtime_designs = self.build_runtime_designs(design_path);
        let mut graph = self.compose_root_graph(path, &runtime_designs);
        validate_instance_graph(&mut graph, &RuntimeBuiltinLookup { runtime: self });

        self.composed_graph = Some(graph);
    }

    /// Loads and lowers the optional design file (populating its unit graph
    /// in `unit_graph_cache`) and builds the singleton runtime [`ApplyDesign`]
    /// vector targeting the root. Returns an empty vector when no design path
    /// is supplied.
    fn build_runtime_designs(&mut self, design_path: Option<&DesignPath>) -> Vec<ApplyDesign> {
        let Some(design_path) = design_path else {
            return Vec::new();
        };

        let design_model_path = design_path.to_model_path();
        self.load_and_lower_internal(&design_model_path);

        vec![ApplyDesign {
            design_path: design_path.clone(),
            target: oneil_shared::InstancePath::root(),
            span: oneil_shared::span::Span::synthetic(),
        }]
    }

    /// Internal evaluation that populates caches without returning references.
    fn eval_model_internal(&mut self, path: &ModelPath, design_path: Option<&DesignPath>) {
        self.load_and_lower_internal(path);

        let runtime_designs = self.build_runtime_designs(design_path);

        // Build the instance graph through the persistent unit-graph cache.
        // `compose` pulls (or builds) the root unit's self-rooted graph,
        // clones it, and overlays runtime-supplied designs at the root.
        // We then run the post-walk validation pass over the composed graph
        // before evaluation; the pass pushes per-instance diagnostics into
        // each [`oneil_frontend::InstancedModel`]'s `validation_errors`
        // bucket. Walk-link / apply / overlay-target-missing errors are
        // already on the per-instance buckets, and cycle errors live on
        // the graph itself, so all error sources are now reachable through
        // `self.composed_graph` once we stash it below.
        let mut graph = self.compose_root_graph(path, &runtime_designs);
        validate_instance_graph(&mut graph, &RuntimeBuiltinLookup { runtime: self });

        // Skip evaluation if there are validation errors. Validation errors
        // indicate broken IR (e.g., undefined references) that would cause
        // panics during evaluation. The errors are already collected on the
        // graph and will be reported to the user.
        let has_blocking_errors = !graph.validation_errors.is_empty()
            || !graph.cycle_errors.is_empty()
            || !graph.contribution_errors.is_empty();

        if !has_blocking_errors {
            // make sure the replacement cache is empty
            #[cfg(feature = "python")]
            self.python_call_replacement_cache.clear();

            // Evaluate the model and its dependencies
            let eval_result = eval::eval_model_from_graph(&graph, self);

            #[cfg(feature = "python")]
            {
                // save the updated call cache
                // TODO: handle errors from saving the replacement cache
                self.python_call_replacement_cache
                    .save_all()
                    .expect("should be able to save the replacement cache");

                // merge the replacement cache into the call cache
                let replacement_cache = std::mem::replace(
                    &mut self.python_call_replacement_cache,
                    PythonCallCache::new(self.cache_dir.clone()),
                );
                self.python_call_cache.merge(replacement_cache);
            }

            for (instance_key, maybe_partial) in eval_result {
                match maybe_partial.into_result() {
                    Ok(model) => {
                        self.eval_cache
                            .insert(instance_key.clone(), LoadResult::success(model));
                    }
                    Err(partial) => {
                        self.eval_cache.insert(
                            instance_key,
                            LoadResult::partial(partial.partial_result, partial.error_collection),
                        );
                    }
                }
            }
        }

        // Hand the composed graph to the runtime so `get_model_diagnostics`
        // can pull diagnostics directly off the per-instance buckets.
        self.composed_graph = Some(graph);
    }

    /// Evaluates a list of expressions in the context of
    /// the given model and returns the results.
    fn eval_expressions_internal<'expr>(
        &mut self,
        expressions: &'expr [String],
        model_path: &ModelPath,
    ) -> (IndexMap<&'expr str, Value>, Vec<OneilDiagnostic>) {
        let mut results = IndexMap::new();
        let mut errors = Vec::new();

        for (index, expression) in expressions.iter().enumerate() {
            // a pseudo path for the expression, to be used for error reporting
            // this is not a real path, but it is a unique path for the expression
            let pseudo_path = format!("/oneil-eval/expr-{index}");
            let pseudo_path = PathBuf::from(pseudo_path);

            let expr_ast = match Self::parse_expression(expression) {
                Ok(expr_ast) => expr_ast,
                Err(error) => {
                    let oneil_error =
                        OneilDiagnostic::from_error_with_source(&error, pseudo_path, expression);

                    errors.push(oneil_error);

                    continue;
                }
            };

            let expr_ir = match self.resolve_expr_in_model(&expr_ast, model_path) {
                Ok(expr_ir) => expr_ir,
                Err(resolution_errors) => {
                    let oneil_errors = resolution_errors.into_iter().map(|error| {
                        OneilDiagnostic::from_error_with_source(
                            &error,
                            pseudo_path.clone(),
                            expression,
                        )
                    });

                    errors.extend(oneil_errors);

                    continue;
                }
            };

            let eval_result = match self.eval_expr_in_model(&expr_ir, model_path) {
                Ok(eval_result) => eval_result,
                Err(eval_errors) => {
                    let oneil_errors = eval_errors.into_iter().map(|error| {
                        OneilDiagnostic::from_error_with_source(
                            &error,
                            pseudo_path.clone(),
                            expression,
                        )
                    });

                    errors.extend(oneil_errors);

                    continue;
                }
            };

            results.insert(expression.as_str(), eval_result);
        }

        (results, errors)
    }

    /// Evaluates an expression as if it were in the context
    /// of the given model.
    fn eval_expr_in_model(
        &mut self,
        expr_ir: &output::ir::Expr,
        model_path: &ModelPath,
    ) -> Result<Value, Vec<EvalError>> {
        eval::eval_expr_in_model(expr_ir, model_path, self)
    }
}

impl Runtime {
    /// Builds a `templates` map from the unit graph cache for passing to
    /// [`compose`]. Because all reachable units are eagerly cached by
    /// [`super::ir::Runtime::load_and_lower_internal`], this map is only
    /// consulted on cache misses (which never occur at runtime); it is
    /// constructed here to satisfy [`compose`]'s type signature.
    fn templates_from_unit_graph_cache(
        &self,
    ) -> IndexMap<ModelPath, LoadResult<InstancedModel, ResolutionErrorCollection>> {
        self.unit_graph_cache
            .iter()
            .filter_map(|(unit, graph)| {
                let CompilationUnit::Model(path) = unit else {
                    return None;
                };
                let template = (*graph.root).clone();
                let errors = graph.resolution_errors.get(path).cloned();
                let result = match errors {
                    Some(e) => LoadResult::partial(template, e),
                    None => LoadResult::success(template),
                };
                Some((path.clone(), result))
            })
            .collect()
    }

    /// Composes the user-facing instance graph for `root_path` with the
    /// supplied runtime designs, going through the persistent `unit_graph_cache`.
    fn compose_root_graph(
        &mut self,
        root_path: &ModelPath,
        runtime_designs: &[ApplyDesign],
    ) -> InstanceGraph {
        // Build templates from the unit graph cache: all reachable units are
        // pre-cached so `compose` will never miss, but the type requires it.
        let templates = self.templates_from_unit_graph_cache();
        // Move out of `self` to satisfy the borrow checker: the `compose`
        // call needs a mutable borrow of the cache while other borrows of
        // `self` are active.
        let design_info = std::mem::take(&mut self.design_info);
        let mut cache = std::mem::take(&mut self.unit_graph_cache);
        let graph = apply_designs(
            root_path,
            runtime_designs,
            &mut cache,
            &templates,
            &design_info,
        );
        self.unit_graph_cache = cache;
        self.design_info = design_info;
        graph
    }
}

fn collect_visited_paths(node: &InstancedModel, seen: &mut indexmap::IndexSet<ModelPath>) {
    seen.insert(node.path().clone());
    for (_, sub) in node.submodels() {
        collect_visited_paths(sub.instance.as_ref(), seen);
    }
}

impl eval::ExternalEvaluationContext for Runtime {
    fn lookup_builtin_variable(&self, name: &BuiltinValueName) -> Option<&Value> {
        self.builtins.get_value(name)
    }

    fn evaluate_builtin_function(
        &self,
        name: &BuiltinFunctionName,
        name_span: Span,
        args: Vec<(Value, Span)>,
    ) -> Option<Result<Value, Vec<EvalError>>> {
        let builtin = self.builtins.get_function(name)?;
        Some(builtin.call(name_span, args))
    }

    #[cfg(feature = "python")]
    fn evaluate_imported_function(
        &mut self,
        python_path: &PythonPath,
        identifier: &PyFunctionName,
        function_call_span: Span,
        args: Vec<(output::Value, Span)>,
        callsite_info: &CallsiteInfo,
    ) -> Option<Result<output::Value, Box<EvalError>>> {
        self.evaluate_python_function(
            python_path,
            identifier,
            function_call_span,
            args,
            callsite_info,
        )
    }

    fn lookup_unit(&self, name: &UnitBaseName) -> Option<&Unit> {
        self.builtins.get_unit(name)
    }

    fn lookup_prefix(&self, name: &UnitPrefix) -> Option<f64> {
        self.builtins.get_prefix(name)
    }

    fn get_preloaded_models(
        &self,
    ) -> impl Iterator<
        Item = (
            EvalInstanceKey,
            &LoadResult<output::Model, output::ModelEvalErrors>,
        ),
    > {
        self.eval_cache
            .iter()
            .map(|(key, result)| (key.clone(), result))
    }
}
