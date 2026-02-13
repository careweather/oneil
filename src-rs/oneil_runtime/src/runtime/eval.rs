//! Model evaluation for the runtime.

use std::path::Path;

use oneil_eval::{self as eval, IrLoadError};
use oneil_output::{Unit, Value};
use oneil_shared::{load_result::LoadResult, span::Span};

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
        identifier_span: Span,
        args: Vec<(output::Value, Span)>,
    ) -> Option<Result<output::Value, Box<eval::EvalError>>> {
        self.evaluate_python_function(python_path, identifier, identifier_span, args)
    }

    fn lookup_unit(&self, name: &str) -> Option<&Unit> {
        self.builtins.get_unit(name)
    }

    fn available_prefixes(&self) -> impl Iterator<Item = (&str, f64)> {
        self.builtins.builtin_prefixes()
    }
}
