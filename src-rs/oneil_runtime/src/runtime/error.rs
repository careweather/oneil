//! Error reporting for models and parameters.

use indexmap::{IndexMap, IndexSet};
use oneil_frontend::{
    CompilationUnit, ContributionDiagnostic, DesignResolutionError, HostLocation, InstanceGraph,
    InstanceValidationError, InstanceValidationErrorKind, InstancedModel,
    ResolutionErrorCollection,
    error::{
        ModelImportResolutionError, ParameterResolutionError, PythonImportResolutionError,
        VariableResolutionError,
    },
};
use oneil_ir as ir;
use oneil_output::{EvalError, Model, ModelEvalErrors};
use oneil_shared::{
    EvalInstanceKey, InstancePath,
    error::{AsOneilDiagnostic, Context, DiagnosticKind, ErrorLocation, OneilDiagnostic},
    load_result::LoadResult,
    paths::{ModelPath, PythonPath, SourcePath},
    symbols::{ParameterName, ReferenceName, TestIndex},
};

use crate::cache::{AstCache, SourceCache};

use super::Runtime;
use crate::error::PythonImportError;
use crate::output::error::{ModelError, RuntimeErrors};

impl Runtime {
    /// Returns the file-time [`ResolutionErrorCollection`] recorded for
    /// `model_path`.
    ///
    /// Lookup order:
    ///
    /// 1. The most recently composed graph's
    ///    [`InstanceGraph::resolution_errors`]. Populated for every
    ///    file the composition reached.
    /// 2. The per-unit cached graph's `resolution_errors` map.
    ///    Populated for every file that has been built through
    ///    `build_unit_graph`. Because `load_and_lower_internal` now
    ///    eagerly builds unit graphs immediately after resolving, this
    ///    tier covers any file that has been loaded but not yet composed.
    fn resolution_errors_for(&self, model_path: &ModelPath) -> Option<&ResolutionErrorCollection> {
        if let Some(graph) = self.composed_graph.as_ref()
            && let Some(errors) = graph.resolution_errors.get(model_path)
        {
            return Some(errors);
        }
        if let Some(graph) = self
            .unit_graph_cache
            .get(&CompilationUnit::Model(model_path.clone()))
            && let Some(errors) = graph.resolution_errors.get(model_path)
        {
            return Some(errors);
        }
        None
    }

    /// Returns all diagnostics for the given model, as well as any referenced
    /// models that have issues (errors and evaluation warnings).
    ///
    /// Source or parsing failures are reported as a [`ModelError::FileError`].
    /// Resolution or evaluation failures are reported as [`ModelError::EvalErrors`].
    ///
    /// If `include_indirect_errors` is true, then errors from models that are referenced
    /// by the model are always included, regardless of whether they are referenced directly.
    ///
    /// For example, imagine there is an `x` in `model_a` that references `y` in `model_b`. Neither of
    /// these parameters have errors, but `model_b` has a parameter `z` that is used in a test, and both
    /// `z` and the test have errors. If `include_indirect_errors` is true, then the errors from `model_b`
    /// will be included in the errors for `model_a`. If `include_indirect_errors` is false, then the errors from `model_b`
    /// will not be included in the errors for `model_a` since there is no direct reference to `z` or to the test.
    #[must_use]
    pub(super) fn get_model_diagnostics(
        &self,
        model_path: &ModelPath,
        include_indirect_errors: bool,
    ) -> RuntimeErrors {
        let mut visited = IndexSet::new();
        let eval_instance_key = EvalInstanceKey::root(model_path.clone());
        self.get_model_diagnostics_inner(&eval_instance_key, include_indirect_errors, &mut visited)
    }

    /// Recursive worker for [`get_model_diagnostics`].
    ///
    /// `visited` carries the set of paths already queried in this
    /// call tree. It guards against infinite recursion when chain
    /// extension (`extend_models_with_chain_files`) cross-links files
    /// that mutually reference one another's contribution diagnostics
    /// — every file participating in a chain otherwise re-discovers
    /// the chain and re-recurses indefinitely.
    ///
    /// TODO: AST and IR errors are duplicated if the same model is
    ///       referenced multiple times (eg. `use model_a as a1` and
    ///       `use model_a as a2`). This is because `visited` tracks
    ///       `EvalInstanceKey`s, but `AST` and `IR` errors are keyed by
    ///       `ModelPath`.
    #[expect(
        clippy::too_many_lines,
        reason = "cohesive error-aggregation logic; splitting would obscure the overall flow"
    )]
    fn get_model_diagnostics_inner(
        &self,
        eval_instance_key: &EvalInstanceKey,
        include_indirect_errors: bool,
        visited: &mut IndexSet<EvalInstanceKey>,
    ) -> RuntimeErrors {
        let model_path = &eval_instance_key.model_path;
        if !visited.insert(eval_instance_key.clone()) {
            return RuntimeErrors::default();
        }

        let path_buf = model_path.clone().into_path_buf();

        // Handle source errors
        //
        // If the source failed to load, then there can be no
        // other errors, so we return early
        let Some(source_entry) = self.source_cache.get_entry(&SourcePath::from(model_path)) else {
            return RuntimeErrors::default();
        };

        let source = match source_entry {
            Ok(source) => source,
            Err(source_err) => {
                let mut errors = RuntimeErrors::default();

                errors.add_model_error(
                    model_path.clone(),
                    ModelError::FileError(vec![OneilDiagnostic::from_error(source_err, path_buf)]),
                );

                return errors;
            }
        };

        // Get the AST errors, if any
        let Some(ast_entry) = self.ast_cache.get_entry(model_path) else {
            return RuntimeErrors::default();
        };

        let ast_errors = match ast_entry {
            LoadResult::Failure => return RuntimeErrors::default(),
            LoadResult::Partial(_, parser_errors) => {
                let errors: Vec<OneilDiagnostic> = parser_errors
                    .iter()
                    .map(|e| OneilDiagnostic::from_error_with_source(e, path_buf.clone(), source))
                    .collect();

                Some(errors)
            }
            LoadResult::Success(_) => None,
        };

        let ir_error_collection = self.resolution_errors_for(model_path);
        let ir_errors = ir_error_collection
            .map(|errors| collect_ir_errors(errors, model_path, source, include_indirect_errors));

        let raw_validation_errors: Option<Vec<InstanceValidationError>> = self
            .composed_graph
            .as_ref()
            .map(|graph| collect_validation_errors_from_graph(graph, model_path));
        let cycle_param_names: IndexSet<ParameterName> = raw_validation_errors
            .as_deref()
            .map(parameter_cycle_names)
            .unwrap_or_default();
        let validation_errors = raw_validation_errors.as_deref().map(|errors| {
            collect_validation_errors(
                errors,
                model_path,
                source,
                ir_error_collection,
                &self.source_cache,
            )
        });

        // get the eval errors, if any. Eval-time `CircularParameterEvaluation`
        // is suppressed for parameters that the graph-time SCC pass already
        // flagged on the composed graph: the runtime's `ParamSlot::InProgress`
        // backstop is defense-in-depth; with SCC catching the cycle we don't
        // want to surface both diagnostics for the same parameter.
        let eval_entry = self.eval_cache.get_entry_instance(eval_instance_key);
        let eval_errors = eval_entry.and_then(|entry| entry.error()).map(|errors| {
            collect_eval_errors(
                errors,
                model_path,
                source,
                include_indirect_errors,
                &cycle_param_names,
            )
        });

        // get the evaluation warnings, if any
        let eval_model = eval_entry.and_then(|entry| entry.value());
        let eval_warning_diagnostics =
            eval_model.map(|model| extract_eval_warning_diagnostics(model, model_path, source));

        let merged = merge_ir_eval_diagnostics(ir_errors, eval_errors, eval_warning_diagnostics);

        let MergedErrors {
            mut models_with_errors,
            python_imports_with_errors,
            model_import_errors,
            python_import_errors,
            mut parameter_errors,
            mut test_errors,
            mut design_resolution_errors,
        } = merged;

        if let Some(graph) = self.composed_graph.as_ref() {
            collect_graph_contribution_errors(
                graph,
                model_path,
                &path_buf,
                source,
                &mut design_resolution_errors,
                &mut models_with_errors,
            );
            // Emit a generic "submodel/reference has errors" notification at
            // the import site in the root model so the user sees a squiggle
            // there in the language server and knows to navigate to the child
            // file for the detailed diagnostics. Also adds any submodel /
            // reference paths that have parse errors to `models_with_errors`
            // so the recursion below collects their diagnostics.
            if model_path == graph.root.path() {
                emit_submodel_import_notifications(
                    graph,
                    &graph.root,
                    &path_buf,
                    source,
                    &self.ast_cache,
                    &mut design_resolution_errors,
                    &mut models_with_errors,
                );
            }
        }

        // Merge validation errors into the per-parameter / per-test maps.
        if let Some(validation) = validation_errors {
            for (name, errs) in validation.parameter_errors {
                parameter_errors.entry(name).or_default().extend(errs);
            }
            for (idx, errs) in validation.test_errors {
                test_errors.entry(idx).or_default().extend(errs);
            }
        }

        // add the errors for models that are referenced
        //
        // this includes both models that have errors and models
        // that were successfully resolved, since models that
        // were successfully resolved may have evaluation warnings

        let mut errors = RuntimeErrors::new();

        let model_successful_references = self
            .eval_cache
            .get_entry(model_path)
            .as_ref()
            .and_then(|entry| entry.value())
            .map(|model| model.references.values())
            .into_iter()
            .flatten();

        let model_references = models_with_errors
            .iter()
            .chain(model_successful_references)
            .collect::<IndexSet<_>>();

        for eval_instance_key in model_references {
            let model_errors = self.get_model_diagnostics_inner(
                eval_instance_key,
                include_indirect_errors,
                visited,
            );
            errors.extend(model_errors);
        }

        // add the errors for Python imports that are referenced
        for python_import_path in python_imports_with_errors {
            let python_import_errors = self.get_python_import_errors(&python_import_path);
            errors.extend(python_import_errors);
        }

        if let Some(ast_errors) = ast_errors {
            // if there are AST errors, add them as a file error
            errors.add_model_error(model_path.clone(), ModelError::FileError(ast_errors));
        } else if !model_import_errors.is_empty()
            || !python_import_errors.is_empty()
            || !parameter_errors.is_empty()
            || !test_errors.is_empty()
            || !design_resolution_errors.is_empty()
        {
            // if there are other errors, add them as a model error
            errors.add_model_error(
                model_path.clone(),
                ModelError::EvalErrors {
                    model_import_errors: Box::new(model_import_errors),
                    python_import_errors: Box::new(python_import_errors),
                    parameter_errors: Box::new(parameter_errors),
                    test_errors: Box::new(test_errors),
                    design_resolution_errors: Box::new(design_resolution_errors),
                },
            );
        }

        errors
    }

    /// Returns errors for the given Python import path.
    ///
    /// If the source failed to load or the Python module failed to load (e.g. file not found or load error),
    /// returns a [`RuntimeErrors`] with [`ModelError::FileError`] entries for each.
    #[must_use]
    pub(super) fn get_python_import_errors(
        &self,
        python_import_path: &PythonPath,
    ) -> RuntimeErrors {
        let path_buf = python_import_path.clone().into_path_buf();
        let mut errors = RuntimeErrors::new();

        if let Some(Err(source_err)) = self.source_cache.get_entry(&python_import_path.into()) {
            errors.add_python_import_error(
                python_import_path.clone(),
                OneilDiagnostic::from_error(source_err, path_buf.clone()),
            );
        }

        if let Some(Err(load_err)) = self.python_import_cache.get_entry(python_import_path)
            && let PythonImportError::LoadFailed(load_err) = load_err
        {
            errors.add_python_import_error(
                python_import_path.clone(),
                OneilDiagnostic::from_error(load_err, path_buf),
            );
        }

        errors
    }
}

/// Result of collecting errors from IR resolution.
#[expect(
    clippy::struct_field_names,
    reason = "removing 'errors' might be confusing"
)]
#[derive(Debug)]
struct IrErrorsResult {
    /// Model paths that have errors (for recursive collection).
    models_with_errors: IndexSet<ModelPath>,
    /// Python import paths that have errors (for recursive collection).
    python_imports_with_errors: IndexSet<PythonPath>,
    /// Model import resolution errors by reference name.
    model_import_errors: IndexMap<ReferenceName, OneilDiagnostic>,
    /// Python import resolution errors by path.
    python_import_errors: IndexMap<PythonPath, OneilDiagnostic>,
    /// Parameter resolution errors by parameter name.
    parameter_errors: IndexMap<ParameterName, Vec<OneilDiagnostic>>,
    /// Test resolution errors.
    test_errors: IndexMap<TestIndex, Vec<OneilDiagnostic>>,
    /// Design / `apply` resolution messages.
    design_resolution_errors: Vec<OneilDiagnostic>,
}

/// Pushes a synthetic "applied design `<basename>` produced invalid
/// contributions" diagnostic into `out` when `applied_via` is the
/// `apply` statement that lives in `model_path`, attached to that
/// statement's span.
///
/// Returns silently when there is no `applied_via` (synthetic CLI
/// design-as-root has no apply to attribute to), when the apply lives
/// in a different file, or when its span is empty.
///
/// Used by both `surface_contribution_diagnostic` (for
/// `ContributionDiagnostic`s) and
/// `surface_design_caused_parameter_errors` (for design-overridden /
/// -added parameters that fail post-composition validation) so the
/// rendered per-apply diagnostic stays identical across error classes.
fn surface_apply_hop(
    applied_via: Option<&ir::DesignApplication>,
    model_path: &ModelPath,
    path_buf: &std::path::Path,
    source: &str,
    out: &mut Vec<OneilDiagnostic>,
) {
    let Some(hop) = applied_via else {
        return;
    };
    if &hop.applied_in != model_path {
        return;
    }
    if hop.apply_span.clone().start() == hop.apply_span.clone().end() {
        return;
    }
    let design_basename = hop
        .design_path
        .as_path()
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("<design>");
    let message = format!("applied design `{design_basename}` produced invalid contributions");
    let synthetic = DesignResolutionError::new(message, hop.apply_span.clone());
    out.push(OneilDiagnostic::from_error_with_source(
        &synthetic,
        path_buf.to_path_buf(),
        source,
    ));
}

/// Adds every apply-relevant file other than `model_path` to
/// `models_with_errors` so the recursive `get_model_diagnostics` pass
/// picks them up.
///
/// `extra_files` is an optional list of extra paths to include
/// (e.g. the host instance's path for parameter-attached errors,
/// or the design file for `ContributionDiagnostic`s). The
/// `applied_via` hop's `applied_in` is added unless it equals
/// `model_path` or its span is empty.
fn extend_models_with_apply(
    applied_via: Option<&ir::DesignApplication>,
    extra_files: &[&ModelPath],
    models_with_errors: &mut IndexSet<EvalInstanceKey>,
    model_path: &ModelPath,
) {
    for path in extra_files {
        if *path != model_path {
            models_with_errors.insert(EvalInstanceKey::root((*path).clone()));
        }
    }
    if let Some(hop) = applied_via
        && hop.apply_span.clone().start() != hop.apply_span.clone().end()
        && &hop.applied_in != model_path
    {
        models_with_errors.insert(EvalInstanceKey::root(hop.applied_in.clone()));
    }
}

/// Adds every other file involved in a contribution diagnostic to
/// `models_with_errors` so the recursive `get_model_diagnostics` pass
/// picks them up.
///
/// Without this, only the file currently being queried would render
/// its perspective of a contribution diagnostic — the design file's
/// assignment-side error and the apply file would be invisible at
/// the CLI when a model with own-applies fails. With it, each
/// participating file's `get_model_diagnostics` runs and surfaces its own
/// view of the same diagnostic.
fn extend_models_with_chain_files(
    diag: &ContributionDiagnostic,
    models_with_errors: &mut IndexSet<EvalInstanceKey>,
    model_path: &ModelPath,
) {
    extend_models_with_apply(
        diag.applied_via.as_ref(),
        &[&diag.design_file],
        models_with_errors,
        model_path,
    );
}

/// Surfaces a contribution-time diagnostic against `model_path`.
///
/// Pushes one entry into `out` for the design file when
/// `model_path == diag.design_file` (rendered with the diagnostic's
/// own assignment span and message), and one entry for the apply
/// statement when `model_path` matches the file containing it
/// (rendered with a generic "applied design `<file>` produced
/// invalid contributions" message at the apply statement's span).
fn surface_contribution_diagnostic(
    diag: &ContributionDiagnostic,
    model_path: &ModelPath,
    path_buf: &std::path::Path,
    source: &str,
    out: &mut Vec<OneilDiagnostic>,
) {
    if &diag.design_file == model_path {
        out.push(OneilDiagnostic::from_error_with_source(
            &diag.error,
            path_buf.to_path_buf(),
            source,
        ));
    }
    surface_apply_hop(diag.applied_via.as_ref(), model_path, path_buf, source, out);
}

/// Walks every parameter on `graph` whose `DesignProvenance.applied_via`
/// is set and which has a parameter-keyed `validation_errors` entry on
/// the host instance, and pushes one generic "applied design
/// `<basename>` produced invalid contributions" diagnostic into `out`
/// when the originating apply lives in `model_path`. Also adds the
/// parameter's host instance's file plus the apply's file (when other
/// than `model_path`) to `models_with_errors` so the recursive
/// `get_model_diagnostics` pass picks them up.
///
/// The precise error continues to surface where it always did — at
/// the host instance's path via the existing `validation_errors`
/// collector. This walk only adds the apply-site fan-out so that a
/// model file applying a faulty design gets its `apply X to Y` line
/// marked, even though the actual failure lives in the design.
fn surface_design_caused_parameter_errors(
    graph: &InstanceGraph,
    model_path: &ModelPath,
    path_buf: &std::path::Path,
    source: &str,
    out: &mut Vec<OneilDiagnostic>,
    models_with_errors: &mut IndexSet<EvalInstanceKey>,
) {
    // Build a lookup set of (host_path, parameter_name) pairs that
    // have validation errors, to detect design-applied parameters
    // that became invalid after contribution.
    let validation_errored: std::collections::HashSet<(
        &oneil_shared::InstancePath,
        &ParameterName,
    )> = graph
        .validation_errors
        .iter()
        .filter_map(|e| {
            if let HostLocation::Parameter(param_name) = &e.host_location {
                Some((&e.host_path, param_name))
            } else {
                None
            }
        })
        .collect();

    // Dedupe per-apply emissions so two failing parameters sharing
    // one apply only produce one generic diagnostic at the apply site.
    let mut emitted_apply_offsets: std::collections::HashSet<(usize, usize)> =
        std::collections::HashSet::new();

    let mut instances: Vec<(InstancePath, &InstancedModel)> = Vec::new();
    collect_tree_instances(graph.root.as_ref(), &InstancePath::root(), &mut instances);
    for instance in graph.reference_pool.values() {
        collect_tree_instances(instance.as_ref(), &InstancePath::root(), &mut instances);
    }

    for (host_path, instance) in &instances {
        for (param_name, parameter) in instance.parameters() {
            let Some(provenance) = parameter.design_provenance() else {
                continue;
            };
            let Some(applied_via) = provenance.applied_via.as_ref() else {
                continue;
            };
            if !validation_errored.contains(&(host_path, param_name)) {
                continue;
            }

            if &applied_via.applied_in == model_path
                && applied_via.apply_span.start() != applied_via.apply_span.end()
            {
                let span_key = (
                    applied_via.apply_span.start().offset,
                    applied_via.apply_span.end().offset,
                );
                if emitted_apply_offsets.insert(span_key) {
                    surface_apply_hop(Some(applied_via), model_path, path_buf, source, out);
                }
            }
            extend_models_with_apply(
                Some(applied_via),
                &[instance.path()],
                models_with_errors,
                model_path,
            );
        }
    }
}

/// Recursively walks an `InstancedModel` subtree rooted at `prefix`, collecting
/// `(InstancePath, &InstancedModel)` pairs for every node in the tree.
fn collect_tree_instances<'g>(
    node: &'g InstancedModel,
    prefix: &InstancePath,
    out: &mut Vec<(InstancePath, &'g InstancedModel)>,
) {
    out.push((prefix.clone(), node));
    for (name, sub) in node.submodels() {
        let child = prefix.child(name.clone());
        collect_tree_instances(sub.instance.as_ref(), &child, out);
    }
}

/// Walks all graph-level contribution diagnostics plus compilation-cycle
/// diagnostics, surfacing the ones that point at `model_path` and recording
/// chain files for the recursive `models_with_errors` walk.
fn collect_graph_contribution_errors(
    graph: &InstanceGraph,
    model_path: &ModelPath,
    path_buf: &std::path::Path,
    source: &str,
    design_resolution_errors: &mut Vec<OneilDiagnostic>,
    models_with_errors: &mut IndexSet<EvalInstanceKey>,
) {
    for diag in &graph.contribution_errors {
        surface_contribution_diagnostic(
            diag,
            model_path,
            path_buf,
            source,
            design_resolution_errors,
        );
        extend_models_with_chain_files(diag, models_with_errors, model_path);
    }

    for error in &graph.cycle_errors {
        if error.source_path() == model_path {
            design_resolution_errors.push(OneilDiagnostic::from_error_with_source(
                error,
                path_buf.to_path_buf(),
                source,
            ));
        }
    }

    surface_design_caused_parameter_errors(
        graph,
        model_path,
        path_buf,
        source,
        design_resolution_errors,
        models_with_errors,
    );
}

/// Emits a generic "submodel `<alias>` has errors" or "reference `<alias>` has errors"
/// diagnostic at the import declaration site for each direct submodel or reference of
/// `model` that has errors **independently** — i.e. before the parent incorporates it.
///
/// "Independent errors" means either:
/// - The model appears as `hop.applied_in` in at least one `ContributionDiagnostic` (its
///   own `apply X to Y` statement failed), or
/// - The model's AST entry in `ast_cache` is [`LoadResult::Partial`] (it has syntax errors).
///
/// This deliberately excludes cases where a child model merely appears as a *target* of a
/// validation error emitted by the parent (e.g. `UndefinedReferenceParameter` on a parameter
/// that does not exist on an otherwise-healthy reference model).
///
/// When a submodel or reference has parse errors, its path is also added to
/// `models_with_errors` so the recursive `get_model_diagnostics` pass collects those errors.
///
/// Only called for the root of a composed graph (the model directly queried by the user),
/// so the import spans always belong to the current `source`.
fn emit_submodel_import_notifications(
    graph: &InstanceGraph,
    model: &InstancedModel,
    path_buf: &std::path::Path,
    source: &str,
    ast_cache: &AstCache,
    out: &mut Vec<OneilDiagnostic>,
    models_with_errors: &mut IndexSet<EvalInstanceKey>,
) {
    // Build the set of model paths that independently own at least one failing
    // apply statement — either a contribution-time error (unit mismatch, missing
    // target) or a design-caused validation error (parameter cycle introduced by
    // an `apply` in that model).
    let mut independently_errored: IndexSet<&ModelPath> = graph
        .contribution_errors
        .iter()
        .filter_map(|d| d.applied_via.as_ref().map(|a| &a.applied_in))
        .collect();

    // Also include models whose `apply` caused validation errors (e.g. cycles).
    let validation_errored: std::collections::HashSet<(&InstancePath, &ParameterName)> = graph
        .validation_errors
        .iter()
        .filter_map(|e| {
            if let HostLocation::Parameter(param_name) = &e.host_location {
                Some((&e.host_path, param_name))
            } else {
                None
            }
        })
        .collect();
    let mut instances: Vec<(InstancePath, &InstancedModel)> = Vec::new();
    collect_tree_instances(graph.root.as_ref(), &InstancePath::root(), &mut instances);
    for instance in graph.reference_pool.values() {
        collect_tree_instances(instance.as_ref(), &InstancePath::root(), &mut instances);
    }
    for (host_path, instance) in &instances {
        for (param_name, parameter) in instance.parameters() {
            let Some(provenance) = parameter.design_provenance() else {
                continue;
            };
            if !validation_errored.contains(&(host_path, param_name)) {
                continue;
            }
            if let Some(applied_via) = provenance.applied_via.as_ref() {
                independently_errored.insert(&applied_via.applied_in);
            }
        }
    }

    for (ref_name, submodel) in model.submodels() {
        let path = submodel.instance.path();
        let has_parse_errors = ast_cache
            .get_entry(path)
            .is_some_and(LoadResult::is_partial);

        if independently_errored.contains(path) || has_parse_errors {
            let message = format!("submodel `{}` has errors", ref_name.as_str());
            let synthetic = DesignResolutionError::new(message, submodel.name_span.clone());
            out.push(OneilDiagnostic::from_error_with_source(
                &synthetic,
                path_buf.to_path_buf(),
                source,
            ));
        }

        // Ensure parse-only errors are collected by the recursion pass.
        if has_parse_errors {
            models_with_errors.insert(EvalInstanceKey::root(path.clone()));
        }
    }
    for (ref_name, ref_import) in model.references() {
        let path = &ref_import.path;
        let has_parse_errors = ast_cache
            .get_entry(path)
            .is_some_and(LoadResult::is_partial);

        if independently_errored.contains(path) || has_parse_errors {
            let message = format!("reference `{}` has errors", ref_name.as_str());
            let synthetic = DesignResolutionError::new(message, ref_import.name_span.clone());
            out.push(OneilDiagnostic::from_error_with_source(
                &synthetic,
                path_buf.to_path_buf(),
                source,
            ));
        }

        // Ensure parse-only errors are collected by the recursion pass.
        if has_parse_errors {
            models_with_errors.insert(EvalInstanceKey::root(path.clone()));
        }
    }
}

/// Collects resolution errors from IR into structured error data and model/python path sets.
///
/// See [`Runtime::get_model_diagnostics`] for more details on the `include_indirect_errors` parameter.
fn collect_ir_errors(
    errors: &ResolutionErrorCollection,
    path: &ModelPath,
    source: &str,
    include_indirect_errors: bool,
) -> IrErrorsResult {
    let path_buf = path.clone().into_path_buf();

    // collect model import errors
    let mut model_import_errors = IndexMap::new();
    let mut models_with_errors = IndexSet::new();

    if include_indirect_errors {
        for (ref_name, (_submodel_name, ref_error)) in errors.get_model_import_resolution_errors() {
            if let Some(model_path) = get_model_path_from_model_import_error(ref_error) {
                models_with_errors.insert(model_path);
            }

            let error =
                OneilDiagnostic::from_error_with_source(ref_error, path_buf.clone(), source);
            model_import_errors.insert(ref_name.clone(), error);
        }
    }

    // collect Python import errors
    let mut python_import_errors = IndexMap::new();
    let mut python_imports_with_errors = IndexSet::new();
    for (python_path, err) in errors.get_python_import_resolution_errors() {
        if let Some(python_path) = get_python_path_from_python_import_error(err) {
            python_imports_with_errors.insert(python_path);
        }

        let error = OneilDiagnostic::from_error_with_source(err, path_buf.clone(), source);
        python_import_errors.insert(python_path.clone(), error);
    }

    let has_python_import_errors = !python_import_errors.is_empty();

    // collect parameter errors
    let mut parameter_errors = IndexMap::new();
    for (param_name, param_errs) in errors.get_parameter_resolution_errors() {
        let oneil_errors: Vec<OneilDiagnostic> = param_errs
            .iter()
            .filter(|e| !(has_python_import_errors && is_undefined_function_error(e)))
            .map(|e| OneilDiagnostic::from_error_with_source(e, path_buf.clone(), source))
            .collect();
        parameter_errors.insert(param_name.clone(), oneil_errors);
    }

    // collect test errors
    let mut test_errors = IndexMap::new();
    for (test_index, test_errs) in errors.get_test_resolution_errors() {
        let oneil_errors: Vec<OneilDiagnostic> = test_errs
            .iter()
            .map(|e| OneilDiagnostic::from_error_with_source(e, path_buf.clone(), source))
            .collect();
        test_errors.insert(*test_index, oneil_errors);
    }

    let design_resolution_errors: Vec<OneilDiagnostic> = errors
        .get_design_resolution_errors()
        .iter()
        .map(|e| OneilDiagnostic::from_error_with_source(e, path_buf.clone(), source))
        .collect();

    IrErrorsResult {
        models_with_errors,
        python_imports_with_errors,
        model_import_errors,
        python_import_errors,
        parameter_errors,
        test_errors,
        design_resolution_errors,
    }
}

const fn is_undefined_function_error(error: &ParameterResolutionError) -> bool {
    matches!(
        error,
        ParameterResolutionError::VariableResolution(
            VariableResolutionError::UndefinedFunction { .. }
        )
    )
}

/// Result of collecting errors from evaluation.
#[expect(
    clippy::struct_field_names,
    reason = "removing 'errors' might be confusing"
)]
#[derive(Debug)]
struct EvalErrorsResult {
    /// Model paths that have errors (for recursive collection).
    models_with_errors: IndexSet<EvalInstanceKey>,
    /// Parameter evaluation errors by parameter name.
    parameter_errors: IndexMap<ParameterName, Vec<OneilDiagnostic>>,
    /// Test evaluation errors.
    test_errors: IndexMap<TestIndex, Vec<OneilDiagnostic>>,
}

/// Collects evaluation errors into structured error data and model path set.
///
/// See [`Runtime::get_model_diagnostics`] for more details on the `include_indirect_errors` parameter.
fn collect_eval_errors(
    errors: &ModelEvalErrors,
    path: &ModelPath,
    source: &str,
    include_indirect_errors: bool,
    cycle_param_names: &IndexSet<ParameterName>,
) -> EvalErrorsResult {
    let path_buf = path.clone().into_path_buf();

    let mut models_with_errors = IndexSet::new();

    if include_indirect_errors {
        for reference_key in &errors.references {
            models_with_errors.insert(reference_key.clone());
        }
    }

    let mut parameter_errors = IndexMap::new();
    for (name, param_errs) in &errors.parameters {
        let suppress_cycle = cycle_param_names.contains(name);

        let models_with_errors_in_param: IndexSet<_> = param_errs
            .iter()
            .filter_map(|error| {
                if let EvalError::ParameterHasError {
                    parameter_instance_key,
                    ..
                } = error
                {
                    Some(parameter_instance_key.clone())
                } else {
                    None
                }
            })
            .collect();
        models_with_errors.extend(models_with_errors_in_param);

        let oneil_errors: Vec<OneilDiagnostic> = param_errs
            .iter()
            .filter(|e| {
                !(suppress_cycle && matches!(e, EvalError::CircularParameterEvaluation { .. }))
            })
            .map(|e| OneilDiagnostic::from_error_with_source(e, path_buf.clone(), source))
            .collect();
        if oneil_errors.is_empty() {
            continue;
        }
        parameter_errors.insert(name.clone(), oneil_errors);
    }

    let mut test_errors = IndexMap::new();
    for (test_index, test_errs) in &errors.tests {
        let mut test_errors_in_test = Vec::new();

        for test_err in test_errs {
            if let EvalError::ParameterHasError {
                parameter_instance_key,
                ..
            } = test_err
            {
                models_with_errors.insert(parameter_instance_key.clone());
            }

            let error = OneilDiagnostic::from_error_with_source(test_err, path_buf.clone(), source);
            test_errors_in_test.push(error);
        }

        test_errors.insert(*test_index, test_errors_in_test);
    }

    EvalErrorsResult {
        models_with_errors,
        parameter_errors,
        test_errors,
    }
}

/// Diagnostics produced from [`Model`] evaluation warnings before they are merged with errors.
#[derive(Debug, Default)]
struct EvalWarningDiagnostics {
    parameter_warnings: IndexMap<ParameterName, Vec<OneilDiagnostic>>,
    test_warnings: IndexMap<TestIndex, Vec<OneilDiagnostic>>,
}

/// Builds [`EvalWarningDiagnostics`] from an evaluated model's parameter and test warning lists.
fn extract_eval_warning_diagnostics(
    model: &Model,
    path: &ModelPath,
    source: &str,
) -> EvalWarningDiagnostics {
    let path_buf = path.clone().into_path_buf();
    let mut out = EvalWarningDiagnostics::default();

    for (name, parameter) in &model.parameters {
        if parameter.warnings.is_empty() {
            continue;
        }

        let diags: Vec<OneilDiagnostic> = parameter
            .warnings
            .iter()
            .map(|w| OneilDiagnostic::from_error_with_source(w, path_buf.clone(), source))
            .collect();

        out.parameter_warnings.insert(name.clone(), diags);
    }

    for (test_index, test) in &model.tests {
        if test.warnings.is_empty() {
            continue;
        }

        let diags: Vec<OneilDiagnostic> = test
            .warnings
            .iter()
            .map(|w| OneilDiagnostic::from_error_with_source(w, path_buf.clone(), source))
            .collect();

        out.test_warnings.insert(*test_index, diags);
    }

    out
}

/// Result of merging IR resolution errors, evaluation errors, and evaluation warnings.
#[expect(
    clippy::struct_field_names,
    reason = "removing 'errors' might be confusing"
)]
#[derive(Debug)]
struct MergedErrors {
    /// Model paths that have errors (for recursive collection).
    pub models_with_errors: IndexSet<EvalInstanceKey>,
    /// Python import paths that have errors (for recursive collection).
    pub python_imports_with_errors: IndexSet<PythonPath>,
    /// Model import errors by reference name.
    pub model_import_errors: IndexMap<ReferenceName, OneilDiagnostic>,
    /// Python import errors by path.
    pub python_import_errors: IndexMap<PythonPath, OneilDiagnostic>,
    /// Parameter diagnostics (resolution and evaluation errors, plus evaluation warnings).
    pub parameter_errors: IndexMap<ParameterName, Vec<OneilDiagnostic>>,
    /// Test diagnostics (resolution and evaluation errors, plus evaluation warnings).
    pub test_errors: IndexMap<TestIndex, Vec<OneilDiagnostic>>,
    /// Design / `apply` resolution errors.
    pub design_resolution_errors: Vec<OneilDiagnostic>,
}

/// Merges IR resolution errors, evaluation errors, and evaluation warnings.
///
/// When both are present, model paths are unioned. Parameter diagnostics are built with eval then IR
/// in the iterator so IR overwrites eval for duplicate keys; test diagnostics use IR then eval so
/// eval overwrites IR for duplicate keys.
fn merge_ir_eval_diagnostics(
    ir_errors: Option<IrErrorsResult>,
    eval_errors: Option<EvalErrorsResult>,
    eval_warning_diagnostics: Option<EvalWarningDiagnostics>,
) -> MergedErrors {
    let mut merged = match (ir_errors, eval_errors) {
        (Some(ir), Some(eval)) => MergedErrors {
            models_with_errors: ir
                .models_with_errors
                .into_iter()
                .map(EvalInstanceKey::root)
                .chain(eval.models_with_errors)
                .collect(),
            python_imports_with_errors: ir.python_imports_with_errors,
            model_import_errors: ir.model_import_errors,
            python_import_errors: ir.python_import_errors,
            // note that in the case of the same parameter/test having errors in both IR and eval,
            // the IR errors are preferred because `ir` comes later in the chain
            parameter_errors: eval
                .parameter_errors
                .into_iter()
                .chain(ir.parameter_errors)
                .collect(),
            test_errors: ir.test_errors.into_iter().chain(eval.test_errors).collect(),
            design_resolution_errors: ir.design_resolution_errors,
        },

        (Some(ir), None) => MergedErrors {
            models_with_errors: ir
                .models_with_errors
                .into_iter()
                .map(EvalInstanceKey::root)
                .collect(),
            python_imports_with_errors: ir.python_imports_with_errors,
            model_import_errors: ir.model_import_errors,
            python_import_errors: ir.python_import_errors,
            parameter_errors: ir.parameter_errors,
            test_errors: ir.test_errors,
            design_resolution_errors: ir.design_resolution_errors,
        },

        (None, Some(eval)) => MergedErrors {
            models_with_errors: eval.models_with_errors,
            python_imports_with_errors: IndexSet::new(),
            model_import_errors: IndexMap::new(),
            python_import_errors: IndexMap::new(),
            parameter_errors: eval.parameter_errors,
            test_errors: eval.test_errors,
            design_resolution_errors: Vec::new(),
        },

        (None, None) => MergedErrors {
            models_with_errors: IndexSet::new(),
            python_imports_with_errors: IndexSet::new(),
            model_import_errors: IndexMap::new(),
            python_import_errors: IndexMap::new(),
            parameter_errors: IndexMap::new(),
            test_errors: IndexMap::new(),
            design_resolution_errors: Vec::new(),
        },
    };

    // add the warnings to the merged errors if they are present
    let Some(warnings) = eval_warning_diagnostics else {
        return merged;
    };

    for (name, parameter) in &warnings.parameter_warnings {
        merged
            .parameter_errors
            .entry(name.clone())
            .or_default()
            .extend(parameter.clone());
    }

    for (test_index, test) in &warnings.test_warnings {
        merged
            .test_errors
            .entry(*test_index)
            .or_default()
            .extend(test.clone());
    }

    merged
}

/// Returns the model path from a model import error when available.
fn get_model_path_from_model_import_error(err: &ModelImportResolutionError) -> Option<ModelPath> {
    match err {
        ModelImportResolutionError::ModelHasError { model_path, .. } => Some(model_path.clone()),

        ModelImportResolutionError::UndefinedSubmodel {
            parent_model_path, ..
        } => Some(parent_model_path.clone()),

        ModelImportResolutionError::ParentModelHasError { .. }
        | ModelImportResolutionError::DuplicateSubmodel { .. }
        | ModelImportResolutionError::DuplicateReference { .. }
        | ModelImportResolutionError::ModelOrDesignNotFound { .. } => None,
    }
}

/// Returns the Python path from a Python import error when available.
fn get_python_path_from_python_import_error(
    err: &PythonImportResolutionError,
) -> Option<PythonPath> {
    match err {
        PythonImportResolutionError::FailedValidation { python_path, .. } => {
            Some(python_path.clone())
        }

        PythonImportResolutionError::DuplicateImport { .. } => None,
    }
}

/// Filters graph-level `validation_errors` to those whose host instance's
/// model path matches `model_path`.
fn collect_validation_errors_from_graph(
    graph: &InstanceGraph,
    model_path: &ModelPath,
) -> Vec<InstanceValidationError> {
    graph
        .validation_errors
        .iter()
        .filter(|e| host_path_model(graph, &e.host_path) == Some(model_path))
        .cloned()
        .collect()
}

/// Returns the [`ModelPath`] of the host node identified by `host_path` in
/// `graph`, walking through submodels and aliases as needed.
fn host_path_model<'g>(
    graph: &'g InstanceGraph,
    host_path: &oneil_shared::InstancePath,
) -> Option<&'g ModelPath> {
    let mut node: &InstancedModel = graph.root.as_ref();
    for seg in host_path.segments() {
        if let Some(submodel) = node.submodels().get(seg) {
            node = submodel.instance.as_ref();
            continue;
        }
        if let Some(alias) = node.aliases().get(seg) {
            // Aliases re-target into the root subtree relative to root.
            return resolve_instance_path_in_root(graph, &alias.alias_path)
                .map(InstancedModel::path);
        }
        if let Some(reference) = node.references().get(seg) {
            return Some(graph.reference_pool.get(&reference.path)?.path());
        }
        return None;
    }
    Some(node.path())
}

/// Walks `path` from the graph root (no aliases, no references) to find an
/// `InstancedModel`, used by [`host_path_model`] for alias resolution.
fn resolve_instance_path_in_root<'g>(
    graph: &'g InstanceGraph,
    path: &oneil_shared::InstancePath,
) -> Option<&'g InstancedModel> {
    let mut node: &InstancedModel = graph.root.as_ref();
    for seg in path.segments() {
        node = node.submodels().get(seg)?.instance.as_ref();
    }
    Some(node)
}

/// Returns the set of parameter names with graph-time
/// [`InstanceValidationErrorKind::ParameterCycle`] diagnostics in
/// `errors`. Used to suppress the eval-time
/// `CircularParameterEvaluation` backstop for the same parameters.
fn parameter_cycle_names(errors: &[InstanceValidationError]) -> IndexSet<ParameterName> {
    errors
        .iter()
        .filter_map(|e| match &e.kind {
            InstanceValidationErrorKind::ParameterCycle { parameter_name, .. } => {
                Some(parameter_name.clone())
            }
            InstanceValidationErrorKind::UndefinedParameter { .. }
            | InstanceValidationErrorKind::UndefinedReference { .. }
            | InstanceValidationErrorKind::UndefinedReferenceParameter { .. } => None,
        })
        .collect()
}

/// Collected post-walk validation diagnostics, bucketed by parameter / test the
/// same way as the IR and eval error collectors so they can be merged in.
#[derive(Debug, Default)]
struct ValidationErrorsResult {
    parameter_errors: IndexMap<ParameterName, Vec<OneilDiagnostic>>,
    test_errors: IndexMap<TestIndex, Vec<OneilDiagnostic>>,
}

/// Builds an [`OneilDiagnostic`] for a single validation error.
///
/// When a variant carries `design_info: Some(...)` the primary location is
/// flipped to point into the design file so the user sees the exact line that
/// introduced the problem.  A plain-text note records the host model file for
/// context.  All other errors use the standard host-model path/source.
fn build_validation_oneil_error(
    error: &InstanceValidationError,
    host_path: &std::path::Path,
    host_source: &str,
    source_cache: &SourceCache,
) -> OneilDiagnostic {
    // Extract design_info from whichever variant carries it.
    let design_info = match &error.kind {
        InstanceValidationErrorKind::ParameterCycle { design_info, .. }
        | InstanceValidationErrorKind::UndefinedParameter { design_info, .. }
        | InstanceValidationErrorKind::UndefinedReference { design_info, .. }
        | InstanceValidationErrorKind::UndefinedReferenceParameter { design_info, .. } => {
            design_info.as_ref()
        }
    };

    if let Some((design_path, assignment_span)) = design_info {
        let design_source_path = design_path.into();
        if let Some(Ok(_design_source)) = source_cache.get_entry(&design_source_path) {
            let design_path_buf = design_path.clone().into_path_buf();
            // For `ParameterCycle` the primary span is in the host model, so we
            // use `assignment_span` (the design's `param = value` line).
            // For `Undefined*` the variable spans are already in design-file space.
            let span_in_design = match &error.kind {
                InstanceValidationErrorKind::ParameterCycle { .. } => assignment_span.clone(),
                InstanceValidationErrorKind::UndefinedParameter { .. }
                | InstanceValidationErrorKind::UndefinedReference { .. }
                | InstanceValidationErrorKind::UndefinedReferenceParameter { .. } => {
                    error.primary_span()
                }
            };
            let design_location = ErrorLocation::from_span(&span_in_design);

            let model_name = host_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("(model)");

            // For cycle errors the note says which parameter was overridden;
            // for undefined-* errors just name the target model.
            let note_text =
                if let InstanceValidationErrorKind::ParameterCycle { parameter_name, .. } =
                    &error.kind
                {
                    let host_span = error.primary_span();
                    let line = host_span.start().line;
                    let col = host_span.start().column;
                    format!(
                        "overrides `{}` in `{model_name}` at line {line}:{col}",
                        parameter_name.as_str()
                    )
                } else {
                    format!("in design for `{model_name}`")
                };

            // Preserve any help/note context from the error itself (e.g. "did you mean").
            let extra_context = error.context();

            let mut context = vec![Context::Note(note_text)];
            context.extend(extra_context);

            return OneilDiagnostic::from_parts(
                DiagnosticKind::Error,
                error.message(),
                design_path_buf,
                Some(design_location),
                context,
                vec![],
            );
        }
    }

    // Default: attribute to the host model file.
    OneilDiagnostic::from_error_with_source(error, host_path.to_path_buf(), host_source)
}

/// Converts validation errors for `model_path` into [`OneilDiagnostic`]s, deduping
/// against any matching file-time `UndefinedParameter` already in
/// `ir_errors` (same host parameter / test, same parameter span). Without this
/// dedup the user would see duplicate "parameter `p` is not defined" messages
/// during the transition: the file-time resolver still emits them too.
fn collect_validation_errors(
    errors: &[InstanceValidationError],
    model_path: &ModelPath,
    source: &str,
    ir_errors: Option<&ResolutionErrorCollection>,
    source_cache: &SourceCache,
) -> ValidationErrorsResult {
    let path_buf = model_path.clone().into_path_buf();
    let mut result = ValidationErrorsResult::default();

    for error in errors {
        if validation_error_is_duplicate(ir_errors, error) {
            continue;
        }

        let oneil_error =
            build_validation_oneil_error(error, path_buf.as_path(), source, source_cache);

        match &error.host_location {
            HostLocation::Parameter(host_param) => {
                result
                    .parameter_errors
                    .entry(host_param.clone())
                    .or_default()
                    .push(oneil_error);
            }
            HostLocation::Test(test_index) => {
                result
                    .test_errors
                    .entry(*test_index)
                    .or_default()
                    .push(oneil_error);
            }
        }
    }

    result
}

/// True when the file-time resolver already emitted an equivalent diagnostic
/// for the same host site, parameter name, and source span.
///
/// Only `UndefinedParameter` (bare-name) errors have any remaining file-time
/// counterpart; reference and external-parameter errors are fully deferred to
/// the post-build validation pass and therefore never need deduplication.
fn validation_error_is_duplicate(
    ir_errors: Option<&ResolutionErrorCollection>,
    error: &InstanceValidationError,
) -> bool {
    let Some(ir_errors) = ir_errors else {
        return false;
    };

    let host_var_errors = host_variable_errors(ir_errors, &error.host_location);

    match &error.kind {
        InstanceValidationErrorKind::UndefinedParameter {
            parameter_name,
            parameter_span,
            ..
        } => host_var_errors.iter().any(|e| {
            matches!(
                e,
                VariableResolutionError::UndefinedParameter {
                    parameter_name: pname,
                    reference_span,
                    ..
                } if pname == parameter_name && *reference_span == *parameter_span,
            )
        }),
        // `UndefinedReference`, `UndefinedReferenceParameter`, and `ParameterCycle`
        // are all fully deferred to the post-build validation pass; there is no
        // file-time counterpart to deduplicate against.
        InstanceValidationErrorKind::UndefinedReference { .. }
        | InstanceValidationErrorKind::UndefinedReferenceParameter { .. }
        | InstanceValidationErrorKind::ParameterCycle { .. } => false,
    }
}

/// Returns the file-time `VariableResolutionError`s emitted for the given
/// host site (parameter or test), unwrapping
/// [`ParameterResolutionError::VariableResolution`] for parameter sites.
fn host_variable_errors<'e>(
    ir_errors: &'e ResolutionErrorCollection,
    host_location: &HostLocation,
) -> Vec<&'e VariableResolutionError> {
    match host_location {
        HostLocation::Parameter(host_param) => ir_errors
            .get_parameter_resolution_errors()
            .get(host_param)
            .map(|errs| {
                errs.iter()
                    .filter_map(|e| match e {
                        ParameterResolutionError::VariableResolution(v) => Some(v),
                        ParameterResolutionError::UnitResolution(_)
                        | ParameterResolutionError::DuplicateParameter { .. } => None,
                    })
                    .collect()
            })
            .unwrap_or_default(),
        HostLocation::Test(test_index) => ir_errors
            .get_test_resolution_errors()
            .get(test_index)
            .map(|errs| errs.iter().collect())
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod validation_error_tests {
    use std::path::PathBuf;

    use oneil_analysis::{HostLocation, InstanceValidationError, InstanceValidationErrorKind};
    use oneil_frontend::{
        ResolutionErrorCollection,
        error::{ParameterResolutionError, VariableResolutionError},
    };
    use oneil_shared::{
        InstancePath,
        paths::ModelPath,
        span::{SourceLocation, Span},
        symbols::{ParameterName, ReferenceName, TestIndex},
    };

    use super::collect_validation_errors;
    use crate::cache::SourceCache;

    fn span(start: usize, end: usize) -> Span {
        use std::path::Path;
        use std::sync::Arc;
        // Source needs to be long enough for the end offset.
        let source: Arc<str> = Arc::from(" ".repeat(end + 1).as_str());
        let path: Arc<Path> = Arc::from(Path::new("test.on"));
        Span::new(
            SourceLocation {
                offset: start,
                line: 1,
                column: start + 1,
            },
            SourceLocation {
                offset: end,
                line: 1,
                column: end + 1,
            },
            path,
            source,
        )
    }

    fn model_path(name: &str) -> ModelPath {
        ModelPath::from_path_with_ext(&PathBuf::from(format!("{name}.on")))
    }

    fn validation_error(
        _host_model: ModelPath,
        host_location: HostLocation,
        target_model: ModelPath,
        reference: &str,
        parameter: &str,
        parameter_span: Span,
    ) -> InstanceValidationError {
        InstanceValidationError {
            host_path: InstancePath::root(),
            host_location,
            kind: InstanceValidationErrorKind::UndefinedReferenceParameter {
                reference_name: ReferenceName::new(reference.to_string()),
                reference_span: span(0, 1),
                parameter_name: ParameterName::from(parameter),
                parameter_span,
                target_model,
                best_match: None,
                design_info: None,
            },
        }
    }

    #[test]
    fn undefined_parameter_is_suppressed_when_ir_already_has_matching_error() {
        // `UndefinedParameter` (bare-name) is the one case where the post-build
        // validation error is deduplicated against a matching file-time resolver error,
        // so the user doesn't see the same diagnostic twice (from two different submodels).
        let host_param = ParameterName::from("host_param");
        let parameter_name = ParameterName::from("x");
        let parameter_span = span(10, 15);

        let mut ir = ResolutionErrorCollection::empty();
        ir.add_parameter_error(
            host_param.clone(),
            ParameterResolutionError::VariableResolution(
                VariableResolutionError::undefined_parameter(
                    parameter_name.clone(),
                    parameter_span.clone(),
                    None,
                ),
            ),
        );

        let validation_err = InstanceValidationError {
            host_path: InstancePath::root(),
            host_location: HostLocation::Parameter(host_param),
            kind: InstanceValidationErrorKind::UndefinedParameter {
                parameter_name,
                parameter_span,
                best_match: None,
                design_info: None,
            },
        };

        let source = "host_param = x";
        let cache = SourceCache::new();
        let result = collect_validation_errors(
            &[validation_err],
            &model_path("foo"),
            source,
            Some(&ir),
            &cache,
        );

        assert!(
            result.parameter_errors.is_empty(),
            "duplicate should be suppressed"
        );
    }

    #[test]
    fn collected_errors_bucket_into_parameter_and_test_maps() {
        let host_model = model_path("car");
        let target_model = model_path("engine");
        let host_param = ParameterName::from("speed");
        let test_index = TestIndex::new(0);

        let errors = vec![
            validation_error(
                host_model.clone(),
                HostLocation::Parameter(host_param.clone()),
                target_model.clone(),
                "r",
                "ghost",
                span(10, 15),
            ),
            validation_error(
                host_model.clone(),
                HostLocation::Test(test_index),
                target_model,
                "r",
                "phantom",
                span(20, 27),
            ),
        ];

        let source = "speed = r.ghost\ntest a: r.phantom > 0";
        let cache = SourceCache::new();
        let result = collect_validation_errors(&errors, &host_model, source, None, &cache);

        assert_eq!(result.parameter_errors.len(), 1);
        assert_eq!(result.parameter_errors[&host_param].len(), 1);
        assert_eq!(result.test_errors.len(), 1);
        assert_eq!(result.test_errors[&test_index].len(), 1);
    }

    #[test]
    fn collected_errors_not_skipped_for_ref_param_errors() {
        // `UndefinedReferenceParameter` is never dedup'd against IR errors
        // (no file-time counterpart), so validation errors are always collected
        // even when the IR has a same-named bare `UndefinedParameter` error.
        let host_model = model_path("car");
        let target_model = model_path("engine");
        let host_param = ParameterName::from("speed");
        let parameter_name = ParameterName::from("ghost");
        let parameter_span = span(10, 15);

        let mut ir = ResolutionErrorCollection::empty();
        ir.add_parameter_error(
            host_param.clone(),
            ParameterResolutionError::VariableResolution(
                VariableResolutionError::undefined_parameter(
                    parameter_name.clone(),
                    parameter_span.clone(),
                    None,
                ),
            ),
        );

        let errors = vec![validation_error(
            host_model.clone(),
            HostLocation::Parameter(host_param.clone()),
            target_model,
            "r",
            parameter_name.as_str(),
            parameter_span,
        )];

        let source = "speed = r.ghost\nmore source padding here";
        let cache = SourceCache::new();
        let result = collect_validation_errors(&errors, &host_model, source, Some(&ir), &cache);

        // The validation error is NOT skipped — it is always reported.
        assert_eq!(result.parameter_errors.len(), 1);
        assert_eq!(result.parameter_errors[&host_param].len(), 1);
    }
}
