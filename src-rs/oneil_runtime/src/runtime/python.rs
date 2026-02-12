//! Python import and evaluation for the runtime (when the `python` feature is enabled).

use oneil_eval::EvalError;
use oneil_shared::span::Span;

use super::Runtime;
use crate::output::{self, ir};

impl Runtime {
    /// Evaluates a Python function by path and identifier.
    pub(super) fn evaluate_python_function(
        &self,
        python_path: &ir::PythonPath,
        identifier: &ir::Identifier,
        identifier_span: Span,
        args: Vec<(output::Value, Span)>,
    ) -> Option<Result<output::Value, Box<EvalError>>> {
        let python_import = self
            .python_import_cache
            .get_entry(python_path.as_ref())?
            .as_ref()
            .expect("should not be trying to evaluate a Python function if the import failed");

        let function = python_import.get(identifier.as_str())?;

        let eval_result = oneil_python::evaluate_python_function(
            function,
            identifier.as_str(),
            identifier_span,
            args,
        );

        Some(eval_result.map_err(|e| {
            Box::new(EvalError::PythonEvalError {
                function_name: e.function_name,
                identifier_span: e.identifier_span,
                message: e.message,
            })
        }))
    }
}
