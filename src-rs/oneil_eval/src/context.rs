use indexmap::{IndexMap, IndexSet};

use oneil_frontend::{InstanceGraph, InstancedModel};
use oneil_ir as ir;
use oneil_output::{self as output, EvalError};
use oneil_shared::{
    EvalInstanceKey,
    load_result::LoadResult,
    partial::MaybePartialResult,
    paths::{ModelPath, PythonPath},
    span::Span,
    symbols::{
        BuiltinFunctionName, BuiltinValueName, ParameterName, PyFunctionName, ReferenceName,
        TestIndex, UnitBaseName, UnitPrefix,
    },
};

use crate::eval_parameter;

/// Error indicating that an IR model could not be loaded.
#[derive(Debug, Clone, Copy)]
pub struct IrLoadError;

/// Context provided by the runtime for resolving builtins and units during evaluation.
pub trait ExternalEvaluationContext {
    /// Returns the value of a builtin variable by identifier, if it exists.
    fn lookup_builtin_variable(&self, name: &BuiltinValueName) -> Option<&output::Value>;

    /// Evaluates a builtin function by identifier with the given arguments, if it exists.
    ///
    /// If the function does not exist, returns `None`.
    ///
    /// # Errors
    ///
    /// Returns an error if there was an error evaluating the builtin function.
    fn evaluate_builtin_function(
        &self,
        name: &BuiltinFunctionName,
        name_span: Span,
        args: Vec<(output::Value, Span)>,
    ) -> Option<Result<output::Value, Vec<EvalError>>>;

    /// Evaluates an imported function by identifier with the given arguments, if it exists.
    ///
    /// If the function does not exist, returns `None`.
    ///
    /// # Errors
    ///
    /// Returns an error if there was an error evaluating the imported function.
    #[cfg(feature = "python")]
    fn evaluate_imported_function(
        &mut self,
        python_path: &PythonPath,
        identifier: &PyFunctionName,
        function_call_span: Span,
        args: Vec<(output::Value, Span)>,
        callsite_info: &CallsiteInfo,
    ) -> Option<Result<output::Value, Box<EvalError>>>;

    /// Returns a unit by name if it is defined in the builtin context.
    fn lookup_unit(&self, name: &UnitBaseName) -> Option<&output::Unit>;

    /// Returns a prefix by name if it is defined in the builtin context.
    fn lookup_prefix(&self, name: &UnitPrefix) -> Option<f64>;

    /// Returns pre-loaded evaluated models (each distinct import instance).
    fn get_preloaded_models(
        &self,
    ) -> impl Iterator<
        Item = (
            EvalInstanceKey,
            &LoadResult<output::Model, output::ModelEvalErrors>,
        ),
    >;
}

/// State of a parameter in the lazy evaluation memo table.
///
/// Parameters are registered as [`ParamSlot::Pending`] during model setup (carrying the IR
/// needed to evaluate them later), transition to [`ParamSlot::InProgress`] while they are being
/// evaluated (so recursive lookups can detect cycles), and settle to [`ParamSlot::Done`] once
/// evaluation completes (either with a value or with errors).
#[derive(Debug, Clone)]
#[expect(
    clippy::large_enum_variant,
    reason = "Done carries output::Parameter (680 bytes); boxing adds indirection on the hot Done path"
)]
enum ParamSlot {
    /// Not yet evaluated. Carries the IR so the evaluator can compute it on demand.
    Pending(Box<ir::Parameter>),
    /// Currently being evaluated. Re-entering this slot indicates a dependency cycle.
    InProgress,
    /// Evaluation completed (successfully or with errors).
    Done(Result<output::Parameter, Vec<EvalError>>),
}

/// Represents a model in progress of being evaluated.
#[derive(Debug, Clone)]
struct ModelInProgress {
    parameters: IndexMap<ParameterName, ParamSlot>,
    /// Aliases of submodel imports on this instance (= subset of `references`
    /// keys). Carried through so the output model can preserve the
    /// submodel/reference distinction declared in source.
    submodels: IndexSet<ReferenceName>,
    references: IndexMap<ReferenceName, EvalInstanceKey>,
    references_with_errors: IndexSet<EvalInstanceKey>,
    tests: IndexMap<TestIndex, Result<output::Test, Vec<EvalError>>>,
    /// Key of the direct parent in the instance tree, if any. Used to resolve
    /// `DesignProvenance::anchor_path` at eval time: walking `up` steps follows
    /// the parent chain.
    parent_key: Option<EvalInstanceKey>,
}

impl ModelInProgress {
    /// Creates a new empty model.
    pub fn new() -> Self {
        Self {
            parameters: IndexMap::new(),
            submodels: IndexSet::new(),
            references: IndexMap::new(),
            references_with_errors: IndexSet::new(),
            tests: IndexMap::new(),
            parent_key: None,
        }
    }
}

impl Default for ModelInProgress {
    fn default() -> Self {
        Self::new()
    }
}

/// Recursively seeds [`ModelInProgress`] entries for `instance` and
/// every owned submodel descendant. Each entry's `references` map
/// collapses the three disjoint child maps on [`InstancedModel`]
/// (`references`, `submodels`, `aliases`) into a single
/// name -> [`EvalInstanceKey`] lookup the evaluator uses for
/// `parameter.alias` resolution.
fn seed_subtree(
    instance: &InstancedModel,
    key: &EvalInstanceKey,
    parent_key: Option<&EvalInstanceKey>,
    models: &mut IndexMap<EvalInstanceKey, ModelInProgress>,
) {
    let parameters: IndexMap<ParameterName, ParamSlot> = instance
        .parameters()
        .iter()
        .map(|(name, param)| (name.clone(), ParamSlot::Pending(Box::new(param.clone()))))
        .collect();

    // The three import maps on `InstancedModel` are disjoint: each named child
    // lives in exactly one of `references`, `submodels`, or `aliases`. Eval
    // collapses them into a single name -> `EvalInstanceKey` lookup table.
    let mut references: IndexMap<ReferenceName, EvalInstanceKey> = IndexMap::new();

    // Cross-file references — pool entries (root-keyed by their ModelPath).
    for (name, import) in instance.references() {
        references.insert(name.clone(), EvalInstanceKey::root(import.path.clone()));
    }

    // Owned submodels — child key directly under this instance.
    for (alias, sub) in instance.submodels() {
        let child_key = EvalInstanceKey {
            model_path: sub.instance.path().clone(),
            instance_path: key.instance_path.clone().child(alias.clone()),
        };
        references.insert(alias.clone(), child_key);
    }

    // Extraction-list aliases — resolve `alias_path` from this key.
    for (name, alias) in instance.aliases() {
        let mut p = key.instance_path.clone();
        let mut model_path = instance.path().clone();
        let mut node: &InstancedModel = instance;
        for seg in alias.alias_path.segments() {
            p = p.child(seg.clone());
            if let Some(sub) = node.submodels().get(seg) {
                model_path = sub.instance.path().clone();
                node = sub.instance.as_ref();
            } else {
                break;
            }
        }
        references.insert(
            name.clone(),
            EvalInstanceKey {
                model_path,
                instance_path: p,
            },
        );
    }

    let submodels: IndexSet<ReferenceName> = instance.submodels().keys().cloned().collect();

    models.insert(
        key.clone(),
        ModelInProgress {
            parameters,
            submodels,
            references,
            references_with_errors: IndexSet::new(),
            tests: IndexMap::new(),
            parent_key: parent_key.cloned(),
        },
    );

    for (alias, sub) in instance.submodels() {
        let child_key = EvalInstanceKey {
            model_path: sub.instance.path().clone(),
            instance_path: key.instance_path.clone().child(alias.clone()),
        };
        seed_subtree(sub.instance.as_ref(), &child_key, Some(key), models);
    }
}

/// Internal helper for `force_parameter`. Represents the outcome of inspecting (and
/// conditionally taking ownership of) the memo slot for a parameter.
enum SlotPeek {
    /// The parameter has already been evaluated (successfully or with errors).
    Done(Result<output::Value, Vec<EvalError>>),
    /// The parameter is currently being evaluated; re-entry signals a cycle.
    Cycle,
    /// The parameter was pending; we took ownership of its IR to evaluate it.
    /// Boxed to keep the enum size small.
    TakenForEval(Box<ir::Parameter>),
}

/// Information about the callsite of the function being evaluated.
///
/// This is used for caching Python function calls.
#[derive(Debug, Default, Clone)]
pub enum CallsiteInfo {
    /// A parameter in a model.
    Parameter(EvalInstanceKey, ParameterName),
    /// A test in a model.
    Test(EvalInstanceKey, TestIndex),
    /// Other callsite (e.g. an expression in the REPL).
    #[default]
    Other,
}

/// Evaluation context that tracks per-instance memo state for lazy parameter evaluation.
///
/// The context owns the parameter memo table for every instance the evaluator may touch:
/// declared parameters start as [`ParamSlot::Pending`] and transition to [`ParamSlot::Done`]
/// (or back through [`ParamSlot::InProgress`] for cycle detection) as evaluation proceeds.
/// Reference wiring, submodel aliases, and overlay bindings are seeded once from the
/// [`InstanceGraph`] at construction time; the graph itself is not retained.
#[derive(Debug)]
pub struct EvalContext<'external, E: ExternalEvaluationContext> {
    models: IndexMap<EvalInstanceKey, ModelInProgress>,
    /// Stack of nested evaluation scopes. `.last()` is always the model whose
    /// parameters/overlays are currently being evaluated.
    ///
    /// Pushed by [`Self::force_parameter`] (to evaluate a parameter in its own model's
    /// scope) and by the overlay-anchor bracket in
    /// [`crate::eval_parameter::eval_parameter`] (to evaluate an overlay's RHS in the
    /// design's lexical scope).
    eval_scope: Vec<EvalInstanceKey>,
    external_context: &'external mut E,

    /// Context for the callsite of any function evaluated in the context.
    callsite_context: Vec<CallsiteInfo>,

    /// Warnings for the parameter or test expression currently being evaluated.
    expression_eval_warnings: Vec<output::EvalWarning>,
}

impl<'external, E: ExternalEvaluationContext> EvalContext<'external, E> {
    /// Creates a new empty evaluation context.
    ///
    /// Used by unit tests that exercise the expression / unit evaluators directly
    /// without standing up an [`InstanceGraph`]. Production code paths (model
    /// evaluation) always go through [`Self::from_graph`], which seeds references,
    /// submodels, overlays, and pending parameters from the graph.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn new(external_context: &'external mut E) -> Self {
        Self {
            models: IndexMap::new(),
            eval_scope: Vec::new(),
            external_context,
            expression_eval_warnings: Vec::new(),
            callsite_context: Vec::new(),
        }
    }

    /// Seeds an evaluation context from a fully-built [`InstanceGraph`].
    ///
    /// The graph is a tree: starting at `graph.root`, every owned
    /// `submodel` child is recursively flattened into a
    /// [`ModelInProgress`] entry keyed by an `EvalInstanceKey`
    /// reflecting its position in the tree. Pool entries reached via
    /// `reference` declarations are flattened separately and keyed by
    /// their own `ModelPath` at the root instance path.
    ///
    /// `references` maps on `ModelInProgress` resolve a reference name
    /// to either an owned-child key (for `with`-extracted aliases) or
    /// to the pool entry's root key.
    #[must_use]
    pub fn from_graph(graph: &InstanceGraph, external_context: &'external mut E) -> Self {
        let mut models: IndexMap<EvalInstanceKey, ModelInProgress> = IndexMap::new();
        let root_key = EvalInstanceKey::root(graph.root.path().clone());
        seed_subtree(graph.root.as_ref(), &root_key, None, &mut models);
        for (path, instance) in &graph.reference_pool {
            let pool_key = EvalInstanceKey::root(path.clone());
            seed_subtree(instance.as_ref(), &pool_key, None, &mut models);
        }
        Self {
            models,
            eval_scope: Vec::new(),
            external_context,
            callsite_context: Vec::new(),
            expression_eval_warnings: Vec::new(),
        }
    }

    /// Creates a new evaluation context with the given pre-loaded models.
    ///
    /// Used by expression-level evaluation against already-evaluated models (e.g. the
    /// runtime's `eval_expr_in_model` entry point). No overlay seeding happens — the
    /// preloaded models already contain final values.
    #[must_use]
    pub fn with_preloaded_models(external_context: &'external mut E) -> Self {
        let models: IndexMap<EvalInstanceKey, ModelInProgress> = external_context
            .get_preloaded_models()
            .map(|(key, result)| {
                let model = match result {
                    LoadResult::Success(model) => ModelInProgress {
                        parameters: model
                            .parameters
                            .iter()
                            .map(|(name, parameter)| {
                                (name.clone(), ParamSlot::Done(Ok(parameter.clone())))
                            })
                            .collect(),
                        submodels: model.submodels.clone(),
                        references: model.references.clone(),
                        references_with_errors: IndexSet::new(),
                        tests: model
                            .tests
                            .iter()
                            .map(|(index, test)| (*index, Ok(test.clone())))
                            .collect(),
                        parent_key: None,
                    },

                    LoadResult::Partial(model, errors) => ModelInProgress {
                        parameters: model
                            .parameters
                            .iter()
                            .map(|(name, parameter)| {
                                (name.clone(), ParamSlot::Done(Ok(parameter.clone())))
                            })
                            .chain(errors.parameters.iter().map(|(name, errs)| {
                                (name.clone(), ParamSlot::Done(Err(errs.clone())))
                            }))
                            .collect(),

                        submodels: model.submodels.clone(),
                        references: model.references.clone(),
                        references_with_errors: errors.references.clone(),
                        tests: model
                            .tests
                            .iter()
                            .map(|(index, test)| (*index, Ok(test.clone())))
                            .chain(
                                errors
                                    .tests
                                    .iter()
                                    .map(|(index, errs)| (*index, Err(errs.clone()))),
                            )
                            .collect(),
                        parent_key: None,
                    },

                    LoadResult::Failure => ModelInProgress::new(),
                };

                (key, model)
            })
            .collect();

        Self {
            models,
            eval_scope: Vec::new(),
            external_context,
            callsite_context: Vec::new(),
            expression_eval_warnings: Vec::new(),
        }
    }

    /// Consumes the context and returns the accumulated models and errors.
    ///
    /// Each entry maps an [`EvalInstanceKey`] to a [`MaybePartialResult`]: either a full
    /// success with the evaluated [`output::Model`], or a partial result (the model) and
    /// any [`ModelEvalErrors`] that occurred during evaluation (e.g. from parameters
    /// or tests that failed).
    #[must_use]
    pub fn into_result(
        self,
    ) -> IndexMap<EvalInstanceKey, MaybePartialResult<output::Model, output::ModelEvalErrors>> {
        let mut result = IndexMap::new();

        // for each model, collect the parameters and tests, and any errors
        for (key, model) in self.models {
            // collect the parameters and any errors
            let mut parameters = IndexMap::new();
            let mut parameter_errors = IndexMap::new();
            for (name, slot) in model.parameters {
                match slot {
                    ParamSlot::Done(Ok(param)) => {
                        parameters.insert(name, param);
                    }
                    ParamSlot::Done(Err(errs)) => {
                        parameter_errors.insert(name, errs);
                    }
                    // Pending or InProgress indicates the setup loop skipped this parameter or
                    // left it mid-force. Either is an evaluator bug — every registered parameter
                    // should have been forced to Done before `into_result` is called.
                    ParamSlot::Pending(_) | ParamSlot::InProgress => {
                        panic!(
                            "parameter `{}` was never evaluated before into_result was called",
                            name.as_str()
                        );
                    }
                }
            }

            // collect the tests and any errors
            let mut tests = IndexMap::new();
            let mut test_errors = IndexMap::new();
            for (index, test) in model.tests {
                match test {
                    Ok(test) => {
                        tests.insert(index, test);
                    }

                    Err(errs) => {
                        test_errors.insert(index, errs);
                    }
                }
            }

            // create the output model
            let output_model = output::Model {
                path: key.model_path.clone(),
                instance_path: key.instance_path.clone(),
                submodels: model.submodels,
                references: model.references,
                parameters,
                tests,
            };

            if parameter_errors.is_empty()
                && test_errors.is_empty()
                && model.references_with_errors.is_empty()
            {
                result.insert(key, MaybePartialResult::ok(output_model));
            } else {
                result.insert(
                    key,
                    MaybePartialResult::err(
                        output_model,
                        output::ModelEvalErrors {
                            parameters: parameter_errors,
                            tests: test_errors,
                            references: model.references_with_errors,
                        },
                    ),
                );
            }
        }

        result
    }

    /// Looks up the given builtin variable and returns the corresponding value.
    ///
    /// # Panics
    ///
    /// Panics if the builtin value is not defined. This should never be the case.
    /// If it is, then there is a bug either in the model resolver when it resolves builtin variables
    /// or in the builtin map when it defines the builtin values.
    #[must_use]
    pub fn lookup_builtin_variable(&self, name: &BuiltinValueName) -> output::Value {
        self.external_context
            .lookup_builtin_variable(name)
            .expect("builtin value should be defined (checked during resolution)")
            .clone()
    }

    /// Looks up a parameter value in the current model, forcing evaluation on demand if
    /// necessary.
    ///
    /// If the parameter hasn't been evaluated yet, it is evaluated lazily in the current
    /// model's scope. Cycles are detected via the in-progress sentinel in the memo table
    /// and surface as [`EvalError::CircularParameterEvaluation`].
    ///
    /// # Panics
    ///
    /// Panics if no current model is set.
    pub fn lookup_parameter_value(
        &mut self,
        parameter_name: &ParameterName,
        variable_span: Span,
    ) -> Result<output::Value, Vec<EvalError>> {
        let current_key = self
            .eval_scope
            .last()
            .expect("current model should be set when looking up a parameter")
            .clone();

        self.force_parameter(&current_key, parameter_name, variable_span, &current_key)
    }

    /// Returns the model path of the instance reached via `reference_name` from the active model.
    ///
    /// Used when building `ExternalDependency` output where the model path is needed but
    /// is no longer stored in `Variable::External`.
    #[must_use]
    pub fn lookup_external_model_path(&self, reference_name: &ReferenceName) -> Option<ModelPath> {
        let current = self.eval_scope.last()?;
        let model = self.models.get(current)?;
        let key = model.references.get(reference_name)?;
        Some(key.model_path.clone())
    }

    /// Looks up a parameter on the instance reached via `reference_name` from the active model,
    /// forcing evaluation on demand if necessary.
    ///
    /// Uses the stored instance key from when the reference was added, which correctly
    /// handles both shared refs (using root instance path) and unique use imports
    /// (using nested instance path).
    pub fn lookup_external_parameter_value(
        &mut self,
        reference_name: &ReferenceName,
        parameter_name: &ParameterName,
        variable_span: Span,
    ) -> Result<output::Value, Vec<EvalError>> {
        let current = self
            .eval_scope
            .last()
            .expect("current model should be set for external lookup")
            .clone();
        let model = self
            .models
            .get(&current)
            .expect("current model should be created when set");
        let key = model
            .references
            .get(reference_name)
            .expect("reference should be added before lookup")
            .clone();
        self.force_parameter(&key, parameter_name, variable_span, &current)
    }

    /// Forces lazy evaluation of a parameter, returning its value.
    ///
    /// The memo table at `key` tracks each parameter's state. On a miss (`Pending`), the IR
    /// is taken out, the slot is transitioned to `InProgress` to detect cycles, and
    /// [`eval_parameter::eval_parameter`] is invoked with this context. The result is
    /// memoized as `Done`. Re-entry on an `InProgress` slot surfaces as
    /// [`EvalError::CircularParameterEvaluation`].
    fn force_parameter(
        &mut self,
        parameter_instance_key: &EvalInstanceKey,
        parameter_name: &ParameterName,
        looked_up_from_span: Span,
        looked_up_from_instance_key: &EvalInstanceKey,
    ) -> Result<output::Value, Vec<EvalError>> {
        // Inspect and potentially take ownership of the pending IR from the memo slot.
        let peek = {
            let Some(slot) = self
                .models
                .get_mut(parameter_instance_key)
                .expect("model should be created when set")
                .parameters
                .get_mut(parameter_name)
            else {
                // Parameter was not seeded — validation already flagged this as
                // `UndefinedReferenceParameter`.  Return a graceful error instead of
                // panicking so the composed graph error bucket surfaces it cleanly.
                return Err(vec![EvalError::ParameterHasError {
                    parameter_instance_key: parameter_instance_key.clone(),
                    parameter_name: parameter_name.clone(),
                    looked_up_from_instance_key: looked_up_from_instance_key.clone(),
                    looked_up_from_span,
                }]);
            };

            match slot {
                ParamSlot::Done(Ok(param)) => SlotPeek::Done(Ok(param.value.clone())),
                ParamSlot::Done(Err(_)) => {
                    SlotPeek::Done(Err(vec![EvalError::ParameterHasError {
                        parameter_instance_key: parameter_instance_key.clone(),
                        parameter_name: parameter_name.clone(),
                        looked_up_from_instance_key: looked_up_from_instance_key.clone(),
                        looked_up_from_span: looked_up_from_span.clone(),
                    }]))
                }
                ParamSlot::InProgress => SlotPeek::Cycle,
                ParamSlot::Pending(_) => {
                    let ParamSlot::Pending(param) = std::mem::replace(slot, ParamSlot::InProgress)
                    else {
                        unreachable!("just matched Pending");
                    };
                    SlotPeek::TakenForEval(param)
                }
            }
        };

        match peek {
            SlotPeek::Done(r) => r,
            SlotPeek::Cycle => Err(vec![EvalError::CircularParameterEvaluation {
                parameter_instance_key: parameter_instance_key.clone(),
                parameter_name: parameter_name.clone(),
                looked_up_from_instance_key: looked_up_from_instance_key.clone(),
                looked_up_from_span,
            }]),
            SlotPeek::TakenForEval(param) => {
                // Evaluate in `key`'s scope so overlay anchor-detection and external lookups use
                // this model as the innermost active scope.
                self.push_active_model(parameter_instance_key.clone());

                // If this parameter carries a design provenance, push the anchor scope so that
                // `Variable::Parameter` references in the overlay RHS resolve against the design
                // target instead of the host instance.
                let anchor_key = param.design_provenance().and_then(|prov| {
                    if prov.anchor_path.is_self() {
                        None
                    } else {
                        self.resolve_anchor_key(parameter_instance_key, &prov.anchor_path)
                    }
                });
                if let Some(ref ak) = anchor_key {
                    self.push_active_model(ak.clone());
                }

                let eval_result =
                    eval_parameter::eval_parameter(parameter_name.clone(), &param, self);

                // pop the callsite context the parameter evaluation
                let _callsite_info = self.pop_callsite_context();

                if let Some(ref ak) = anchor_key {
                    self.pop_active_model(ak);
                }
                self.pop_active_model(parameter_instance_key);

                let done_slot: Result<output::Parameter, Vec<EvalError>> = match eval_result {
                    Ok(epr) => Ok(eval_parameter::build_output_parameter(
                        epr.value,
                        epr.expr_span,
                        epr.warnings,
                        param.as_ref(),
                        self,
                    )),
                    Err(errs) => Err(errs),
                };

                // Memoize the result.
                self.models
                    .get_mut(parameter_instance_key)
                    .expect("model still exists")
                    .parameters
                    .insert(parameter_name.clone(), ParamSlot::Done(done_slot.clone()));

                match done_slot {
                    Ok(p) => Ok(p.value),
                    Err(_) => Err(vec![EvalError::ParameterHasError {
                        parameter_instance_key: parameter_instance_key.clone(),
                        parameter_name: parameter_name.clone(),
                        looked_up_from_instance_key: looked_up_from_instance_key.clone(),
                        looked_up_from_span,
                    }]),
                }
            }
        }
    }

    /// Forces all still-pending parameters on the model identified by `key`.
    ///
    /// Used by Pass 2 to drive evaluation of every registered parameter. `force_parameter`
    /// itself pushes/pops `key` as the evaluation scope, so this function does not need to
    /// be called from inside any active-model bracket.
    pub(crate) fn force_all_pending_on(&mut self, key: &EvalInstanceKey) {
        // Take a snapshot of pending names; the loop mutates state below.
        let pending_names: Vec<(ParameterName, Span)> = self
            .models
            .get(key)
            .expect("model should be registered before forcing")
            .parameters
            .iter()
            .filter_map(|(name, slot)| match slot {
                ParamSlot::Pending(param) => Some((name.clone(), param.name_span().clone())),
                ParamSlot::InProgress | ParamSlot::Done(_) => None,
            })
            .collect();

        for (name, span) in pending_names {
            // Errors are already memoized by force_parameter; we ignore the return value.
            let _ = self.force_parameter(key, &name, span, key);
        }
    }

    /// Evaluates a builtin function with the given arguments.
    ///
    /// # Panics
    ///
    /// Panics if the builtin function is not defined. This should never be the case.
    pub fn evaluate_builtin_function(
        &self,
        name: &BuiltinFunctionName,
        name_span: Span,
        args: Vec<(output::Value, Span)>,
    ) -> Result<output::Value, Vec<EvalError>> {
        self.external_context
            .evaluate_builtin_function(name, name_span, args)
            .expect("builtin function should be defined (checked during resolution)")
    }

    /// Evaluates an imported function with the given arguments.
    pub fn evaluate_imported_function(
        &mut self,
        python_path: &PythonPath,
        name: &PyFunctionName,
        function_call_span: Span,
        args: Vec<(output::Value, Span)>,
    ) -> Result<output::Value, Box<EvalError>> {
        #[cfg(feature = "python")]
        {
            let callsite_info = self.callsite_context.last().cloned().unwrap_or_default();
            self.external_context
                .evaluate_imported_function(
                    python_path,
                    name,
                    function_call_span,
                    args,
                    &callsite_info,
                )
                .expect("imported function should be defined (checked during resolution)")
        }

        #[cfg(not(feature = "python"))]
        {
            let _ = (self, python_path, name, args);
            Err(Box::new(EvalError::PythonNotEnabled {
                relevant_span: function_call_span,
            }))
        }
    }

    /// Looks up a unit by name.
    #[must_use]
    pub fn lookup_unit(&self, name: &UnitBaseName) -> Option<output::Unit> {
        self.external_context.lookup_unit(name).cloned()
    }

    /// Looks up a prefix by name.
    #[must_use]
    pub fn lookup_prefix(&self, name: &UnitPrefix) -> Option<f64> {
        self.external_context.lookup_prefix(name)
    }

    /// Clears warnings for a new parameter or test expression evaluation.
    ///
    /// Call at the start of evaluating each top-level parameter or test expression.
    pub fn begin_parameter_evaluation(&mut self, parameter_name: ParameterName) {
        self.expression_eval_warnings.clear();

        let current_key = self
            .eval_scope
            .last()
            .expect("current model should be set when beginning a parameter evaluation");

        self.callsite_context
            .push(CallsiteInfo::Parameter(current_key.clone(), parameter_name));
    }

    /// Clears warnings for a new test expression evaluation.
    ///
    /// Call at the start of evaluating each top-level test expression.
    pub fn begin_test_evaluation(&mut self, test_index: TestIndex) {
        self.expression_eval_warnings.clear();

        let current_key = self
            .eval_scope
            .last()
            .expect("current model should be set when beginning a test evaluation");

        self.callsite_context
            .push(CallsiteInfo::Test(current_key.clone(), test_index));
    }

    /// Takes warnings collected while evaluating the current expression.
    ///
    /// Typically called after successful evaluation to attach them to the evaluated
    /// `oneil_output::Parameter` or `oneil_output::Test`.
    #[must_use]
    pub fn take_expression_warnings(&mut self) -> Vec<output::EvalWarning> {
        std::mem::take(&mut self.expression_eval_warnings)
    }

    /// Records an evaluation warning for the expression currently being evaluated.
    pub fn push_eval_warning(&mut self, warning: output::EvalWarning) {
        self.expression_eval_warnings.push(warning);
    }

    /// Resolves a `RelativePath` relative to `host_key`, returning the anchor's
    /// `EvalInstanceKey` if all walk/descent steps can be resolved in the models map.
    ///
    /// Each `up` step follows `parent_key` on the current model. Each `down` step
    /// follows the named entry in the current model's `references` map.
    fn resolve_anchor_key(
        &self,
        host_key: &EvalInstanceKey,
        path: &oneil_shared::RelativePath,
    ) -> Option<EvalInstanceKey> {
        let mut current = host_key.clone();
        for _ in 0..path.up {
            let parent = self.models.get(&current)?.parent_key.clone()?;
            current = parent;
        }
        for seg in &path.down {
            let next = self.models.get(&current)?.references.get(seg)?.clone();
            current = next;
        }
        Some(current)
    }

    /// Pushes `key` as the innermost active evaluation scope.
    ///
    /// Creates an empty [`ModelInProgress`] entry if `key` hasn't been seeded yet.
    /// In the normal pipeline every key is seeded by [`Self::from_graph`], so this
    /// fallback only fires for tests and expression-level entry points that call
    /// [`Self::new`] directly.
    pub fn push_active_model(&mut self, key: EvalInstanceKey) {
        self.models.entry(key.clone()).or_default();
        self.eval_scope.push(key);
    }

    /// Clears the active model.
    pub fn pop_active_model(&mut self, expected: &EvalInstanceKey) {
        assert_eq!(self.eval_scope.last(), Some(expected));

        self.eval_scope.pop();
    }

    /// Returns a snapshot of all registered model instance keys, in insertion order.
    #[must_use]
    #[expect(dead_code, reason = "debugging/testing helper")]
    pub fn model_keys_snapshot(&self) -> Vec<EvalInstanceKey> {
        self.models.keys().cloned().collect()
    }

    /// Returns every `(parent, child)` pair implied by the `references` entries on each
    /// instance, as a snapshot that doesn't borrow `self`.
    #[must_use]
    pub fn reference_pairs_snapshot(&self) -> Vec<(EvalInstanceKey, EvalInstanceKey)> {
        let mut out = Vec::new();
        for (parent_key, model) in &self.models {
            for child_key in model.references.values() {
                out.push((parent_key.clone(), child_key.clone()));
            }
        }
        out
    }

    /// Adds a parameter evaluation result to the current model.
    ///
    /// Inserts the result directly as a `Done` slot, skipping lazy evaluation. Used by tests
    /// that want to pre-populate parameter values without going through the lazy evaluator.
    ///
    /// # Panics
    ///
    /// Panics if no current model is set or if the current model was not created.
    #[cfg(test)]
    pub fn add_parameter_result(
        &mut self,
        parameter_name: ParameterName,
        result: Result<output::Parameter, Vec<EvalError>>,
    ) {
        let Some(current_key) = self.eval_scope.last() else {
            panic!("current model should be set when adding a parameter result");
        };

        // add the parameter result to the current model
        let model = self
            .models
            .get_mut(current_key)
            .expect("current model should be created when set");

        model
            .parameters
            .insert(parameter_name, ParamSlot::Done(result));
    }

    /// Returns whether the evaluated instance at `key` has any evaluation errors.
    #[must_use]
    pub fn reference_has_errors(&self, key: &EvalInstanceKey) -> bool {
        let Some(model) = self.models.get(key) else {
            return false;
        };
        let has_parameter_errors = model
            .parameters
            .values()
            .any(|slot| matches!(slot, ParamSlot::Done(Err(_))));
        let has_test_errors = model.tests.iter().any(|(_, result)| result.is_err());
        let has_reference_errors = !model.references_with_errors.is_empty();

        has_parameter_errors || has_test_errors || has_reference_errors
    }

    /// Records that `child_key` has errors on the parent model identified by `parent_key`.
    ///
    /// # Panics
    ///
    /// Panics if the parent model has not been registered.
    pub fn add_reference_error_to(
        &mut self,
        parent_key: &EvalInstanceKey,
        child_key: &EvalInstanceKey,
    ) {
        let model = self
            .models
            .get_mut(parent_key)
            .expect("parent model should be registered when adding a reference error");

        model.references_with_errors.insert(child_key.clone());
    }

    /// Adds a test evaluation result to the model identified by `key`.
    ///
    /// # Panics
    ///
    /// Panics if the model has not been registered.
    pub(crate) fn add_test_result(
        &mut self,
        key: &EvalInstanceKey,
        test_index: TestIndex,
        test_result: Result<output::Test, Vec<EvalError>>,
    ) {
        let current_model = self
            .eval_scope
            .last()
            .expect("current model should be set when adding a test result");

        // assert that the callsite context is the expected test index
        let Some(CallsiteInfo::Test(expected_model_path, expected_test_index)) =
            self.callsite_context.pop()
        else {
            panic!("callsite context should be a test when adding a test result");
        };
        assert_eq!(&expected_model_path, current_model);
        assert_eq!(expected_test_index, test_index);

        // add the test result to the current model
        let model = self
            .models
            .get_mut(key)
            .expect("model should be registered when adding a test result");

        model.tests.insert(test_index, test_result);
    }

    /// Pops the last callsite context.
    ///
    /// # Panics
    ///
    /// Panics if the callsite context is empty.
    fn pop_callsite_context(&mut self) -> CallsiteInfo {
        self.callsite_context
            .pop()
            .expect("callsite context should not be empty")
    }
}
