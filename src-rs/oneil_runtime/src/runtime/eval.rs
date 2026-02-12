//! Model evaluation for the runtime.

use std::path::Path;

use oneil_eval::{self as eval, EvalError, IrLoadError};
use oneil_output::{Unit, Value};
use oneil_shared::error::OneilError;
use oneil_shared::span::Span;

use super::Runtime;
use crate::output::{self, ir};

impl Runtime {
    /// Evaluates a model and returns the result.
    ///
    /// # Errors
    ///
    /// Returns a [`EvalErrorReference`](output::reference::EvalErrorReference) if the model could not be evaluated.
    #[expect(
        clippy::missing_panics_doc,
        reason = "the panic only happens if an internal invariant is violated"
    )]
    pub fn eval_model(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<
        output::reference::ModelReference<'_>,
        output::reference::EvalErrorReference<'_>,
    > {
        // make sure the IR is loaded for the model and its dependencies
        // TODO: once caching works, evaluating the model should load the IR as it goes
        let _ir_results = self.load_ir(&path);

        // evaluate the model and its dependencies
        let eval_result = eval::eval_model(&path, self);

        for (model_path, result) in eval_result {
            let source = self.source_cache.get(&model_path).unwrap_or("");

            match result {
                Ok(model) => {
                    self.eval_cache.insert_ok(model_path, model);
                }
                Err(eval_errors) if eval_errors.had_resolution_errors => {
                    let resolution_errors = self
                        .ir_cache
                        .get_error(&model_path)
                        .expect("should have resolution errors")
                        .clone();

                    self.eval_cache.insert_err(
                        model_path,
                        output::error::EvalError::Resolution(resolution_errors),
                    );
                }
                Err(eval_errors) => {
                    let parameter_errors = eval_errors
                        .parameters
                        .into_iter()
                        .map(|(name, errs)| {
                            (
                                name,
                                errs.into_iter()
                                    .map(|e| {
                                        OneilError::from_error_with_source(
                                            &e,
                                            model_path.clone(),
                                            source,
                                        )
                                    })
                                    .collect(),
                            )
                        })
                        .collect();

                    let test_errors = eval_errors
                        .tests
                        .into_iter()
                        .map(|e| {
                            OneilError::from_error_with_source(&e, model_path.clone(), source)
                        })
                        .collect();

                    self.eval_cache.insert_err(
                        model_path,
                        output::error::EvalError::EvalErrors {
                            partial_result: Box::new(eval_errors.partial_result),
                            parameter_errors,
                            test_errors,
                        },
                    );
                }
            }
        }

        let model = self
            .eval_cache
            .get_entry(path.as_ref())
            .expect("eval_model populates cache for requested path and dependencies");

        match model {
            Ok(model) => {
                let model_ref = output::reference::ModelReference::new(model, &self.eval_cache);
                Ok(model_ref)
            }
            Err(err) => {
                let err_ref =
                    output::reference::EvalErrorReference::new(err, &self.eval_cache);
                Err(err_ref)
            }
        }
    }
}

impl eval::ExternalEvaluationContext for Runtime {
    fn lookup_ir(&self, path: impl AsRef<Path>) -> Option<Result<&oneil_ir::Model, IrLoadError>> {
        let result = self.ir_cache.get_entry(path.as_ref())?;
        match result {
            Ok(ir) => Some(Ok(ir)),
            Err(_error) => Some(Err(IrLoadError)),
        }
    }

    fn lookup_builtin_variable(&self, identifier: &oneil_ir::Identifier) -> Option<&Value> {
        self.builtins.get_value(identifier.as_str())
    }

    fn evaluate_builtin_function(
        &self,
        identifier: &oneil_ir::Identifier,
        identifier_span: Span,
        args: Vec<(Value, Span)>,
    ) -> Option<Result<Value, Vec<EvalError>>> {
        let function = self.builtins.get_function(identifier.as_str())?;
        Some(function(identifier_span, args))
    }

    #[cfg(feature = "python")]
    fn evaluate_imported_function(
        &self,
        python_path: &ir::PythonPath,
        identifier: &ir::Identifier,
        identifier_span: Span,
        args: Vec<(output::Value, Span)>,
    ) -> Option<Result<output::Value, Box<EvalError>>> {
        self.evaluate_python_function(python_path, identifier, identifier_span, args)
    }

    fn lookup_unit(&self, name: &str) -> Option<&Unit> {
        self.builtins.get_unit(name)
    }

    fn available_prefixes(&self) -> impl Iterator<Item = (&str, f64)> {
        self.builtins.builtin_prefixes()
    }
}
