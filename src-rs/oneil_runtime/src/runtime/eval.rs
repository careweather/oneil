//! Model evaluation for the runtime.

use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use oneil_eval::{self as eval, IrLoadError};
use oneil_output::{Unit, Value};
use oneil_shared::{error::OneilError, load_result::LoadResult, span::Span};

use super::Runtime;
use crate::{
    error::RuntimeUnitConversionError,
    output::{self, error::RuntimeErrors, ir},
};

type EvalModelAndExpressionsResult<'runtime, 'expr> = (
    Option<(
        output::reference::ModelReference<'runtime>,
        IndexMap<&'expr str, Value>,
    )>,
    RuntimeErrors,
    Vec<OneilError>,
);

impl Runtime {
    /// Evaluates a model and returns the result.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeErrors`] (via [`get_model_errors`](super::Runtime::get_model_errors)) if the model could not be evaluated.
    pub fn eval_model(
        &mut self,
        path: impl AsRef<Path>,
    ) -> (Option<output::reference::ModelReference<'_>>, RuntimeErrors) {
        let path = path.as_ref();
        self.eval_model_internal(path);

        let model_opt = self
            .eval_cache
            .get_entry(path)
            .and_then(LoadResult::value)
            .map(|model| output::reference::ModelReference::new(model, &self.eval_cache));

        let include_indirect_errors = true;

        let errors = self.get_model_errors(path, include_indirect_errors);

        (model_opt, errors)
    }

    /// Evaluates a model and a list of expressions in the context of
    /// the given model and returns the result.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeErrors`] (via [`get_model_errors`](super::Runtime::get_model_errors)) if the model could not be evaluated.
    /// Returns [`OneilError`]s if the expressions could not be evaluated.
    pub fn eval_model_and_expressions<'runtime, 'expr>(
        &'runtime mut self,
        path: impl AsRef<Path>,
        expressions: &'expr [String],
    ) -> EvalModelAndExpressionsResult<'runtime, 'expr> {
        // evaluate the model and its dependencies
        self.eval_model_internal(&path);

        // evaluate the expressions
        let (expr_results, expr_errors) =
            self.eval_expressions_internal(expressions, path.as_ref());

        let model_opt = self
            .eval_cache
            .get_entry(path.as_ref())
            .and_then(LoadResult::value)
            .map(|model| output::reference::ModelReference::new(model, &self.eval_cache));

        let result = model_opt.map(|model| (model, expr_results));

        let include_indirect_errors = true;

        let model_errors = self.get_model_errors(path.as_ref(), include_indirect_errors);

        (result, model_errors, expr_errors)
    }

    pub(super) fn eval_model_internal(
        &mut self,
        path: impl AsRef<Path>,
    ) -> &LoadResult<output::Model, eval::EvalErrors> {
        // make sure the IR is loaded for the model and its dependencies
        // TODO: once caching works, evaluating the model should load the IR as it goes
        let _ir_results = self.load_ir(&path);

        // evaluate the model and its dependencies
        let eval_result = eval::eval_model(&path, self);

        for (model_path, maybe_partial) in eval_result {
            match maybe_partial.into_result() {
                Ok(model) => {
                    self.eval_cache
                        .insert(model_path, LoadResult::success(model));
                }
                Err(partial) => {
                    self.eval_cache.insert(
                        model_path,
                        LoadResult::partial(partial.partial_result, partial.error_collection),
                    );
                }
            }
        }

        self.eval_cache
            .get_entry(path.as_ref())
            .expect("eval_model populates cache for requested path and dependencies")
    }

    /// Evaluates a list of expressions in the context of
    /// the given model and returns the results.
    fn eval_expressions_internal<'expr>(
        &mut self,
        expressions: &'expr [String],
        file: &Path,
    ) -> (IndexMap<&'expr str, Value>, Vec<OneilError>) {
        let mut results = IndexMap::new();
        let mut errors = Vec::new();

        for (index, full_expression) in expressions.iter().enumerate() {
            let ExprRequest {
                expression,
                target_unit,
            } = parse_expr_request(full_expression);

            // a pseudo path for the expression, to be used for error reporting
            // this is not a real path, but it is a unique path for the expression
            let pseudo_path = format!("/oneil-eval/expr-{index}");
            let pseudo_path = PathBuf::from(pseudo_path);

            let expr_ast = match Self::parse_expression(expression) {
                Ok(expr_ast) => expr_ast,
                Err(error) => {
                    let oneil_error =
                        OneilError::from_error_with_source(&error, pseudo_path, full_expression);

                    errors.push(oneil_error);

                    continue;
                }
            };

            let expr_ir = match self.resolve_expr_in_model(&expr_ast, file) {
                Ok(expr_ir) => expr_ir,
                Err(resolution_errors) => {
                    let oneil_errors = resolution_errors.into_iter().map(|error| {
                        OneilError::from_error_with_source(
                            &error,
                            pseudo_path.clone(),
                            full_expression,
                        )
                    });

                    errors.extend(oneil_errors);

                    continue;
                }
            };

            let eval_result = match self.eval_expr_in_model(&expr_ir, file) {
                Ok(eval_result) => eval_result,
                Err(eval_errors) => {
                    let oneil_errors = eval_errors.into_iter().map(|error| {
                        OneilError::from_error_with_source(
                            &error,
                            pseudo_path.clone(),
                            full_expression,
                        )
                    });

                    errors.extend(oneil_errors);

                    continue;
                }
            };

            let Some(target_unit) = target_unit else {
                // no target unit, so we just use the result as is
                results.insert(full_expression.as_str(), eval_result);
                continue;
            };

            // parse the target unit
            let unit_ast = match Self::parse_unit(target_unit) {
                Ok(unit_ast) => unit_ast,
                Err(error) => {
                    let oneil_error = OneilError::from_error_with_source(
                        &error,
                        pseudo_path.clone(),
                        target_unit,
                    );
                    errors.push(oneil_error);

                    continue;
                }
            };

            // resolve the target unit
            let target_unit_ir = match self.resolve_unit(&unit_ast) {
                Ok(target_unit_ir) => target_unit_ir,
                Err(resolution_errors) => {
                    let oneil_errors = resolution_errors.into_iter().map(|error| {
                        OneilError::from_error_with_source(&error, pseudo_path.clone(), target_unit)
                    });
                    errors.extend(oneil_errors);

                    continue;
                }
            };

            // evaluate the target unit
            let target_unit_result = self.eval_unit(&target_unit_ir);

            // convert the result to the target unit
            let eval_result = match eval_result.with_unit(target_unit_result) {
                Ok(eval_result) => eval_result,
                Err(error) => {
                    // TODO: because the unit is parsed seperately from the expression,
                    //       the unit_ast.span() is not the correct span for the error.
                    //
                    //       Either we need to update how we're parsing the unit, or we
                    //       need to have a way to adjust the unit span.
                    let conversion_error =
                        RuntimeUnitConversionError::new(error, expr_ast.span(), unit_ast.span());

                    let oneil_error = OneilError::from_error_with_source(
                        &conversion_error,
                        pseudo_path.clone(),
                        expression,
                    );

                    errors.push(oneil_error);
                    continue;
                }
            };

            results.insert(expression, eval_result);
        }

        (results, errors)
    }

    /// Evaluates an expression as if it were in the context
    /// of the given model.
    fn eval_expr_in_model(
        &mut self,
        expr_ir: &output::ir::Expr,
        file: &Path,
    ) -> Result<Value, Vec<eval::EvalError>> {
        eval::eval_expr_in_model(expr_ir, file, self)
    }

    /// Evaluates a composite unit and returns the resulting sized unit.
    fn eval_unit(&mut self, unit_ir: &output::ir::CompositeUnit) -> Unit {
        eval::eval_unit(unit_ir, self)
    }
}

/// Parsed representation of an `--expr` argument.
#[derive(Debug, Clone, Copy)]
struct ExprRequest<'expr> {
    expression: &'expr str,
    target_unit: Option<&'expr str>,
}

/// Parses an expression request that can optionally end with `: <unit>`.
fn parse_expr_request(expression: &str) -> ExprRequest<'_> {
    let maybe_split = expression.rsplit_once(':').and_then(|(expression, unit)| {
        let expression = expression.trim_end();
        let unit = unit.trim();

        (!expression.is_empty() && !unit.is_empty()).then_some((expression, unit))
    });

    match maybe_split {
        Some((expression, target_unit)) => ExprRequest {
            expression,
            target_unit: Some(target_unit),
        },
        None => ExprRequest {
            expression,
            target_unit: None,
        },
    }
}

impl eval::ExternalEvaluationContext for Runtime {
    fn lookup_ir(&self, path: impl AsRef<Path>) -> Option<LoadResult<&ir::Model, IrLoadError>> {
        let entry = self.ir_cache.get_entry(path.as_ref())?;
        let result = entry.as_ref().map_err(|_error| eval::IrLoadError);

        Some(result)
    }

    fn lookup_builtin_variable(&self, identifier: &oneil_ir::Identifier) -> Option<&Value> {
        self.builtins.get_value(identifier.as_str())
    }

    fn evaluate_builtin_function(
        &self,
        identifier: &oneil_ir::Identifier,
        identifier_span: Span,
        args: Vec<(Value, Span)>,
    ) -> Option<Result<Value, Vec<eval::EvalError>>> {
        let function = self.builtins.get_function(identifier.as_str())?;
        Some(function(identifier_span, args))
    }

    #[cfg(feature = "python")]
    fn evaluate_imported_function(
        &self,
        python_path: &ir::PythonPath,
        identifier: &ir::Identifier,
        function_call_span: Span,
        args: Vec<(output::Value, Span)>,
    ) -> Option<Result<output::Value, Box<eval::EvalError>>> {
        self.evaluate_python_function(python_path, identifier, function_call_span, args)
    }

    fn lookup_unit(&self, name: &str) -> Option<&Unit> {
        self.builtins.get_unit(name)
    }

    fn lookup_prefix(&self, name: &str) -> Option<f64> {
        self.builtins.get_prefix(name)
    }

    fn get_preloaded_models(
        &self,
    ) -> impl Iterator<Item = (PathBuf, &LoadResult<output::Model, eval::EvalErrors>)> {
        self.eval_cache
            .iter()
            .map(|(path, result)| (path.clone(), result))
    }
}
