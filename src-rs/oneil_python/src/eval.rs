//! Evaluation of Python functions from Oneil.

use oneil_output::Value;
use oneil_shared::span::Span;
use pyo3::Python;
use pyo3::types::PyTracebackMethods;

use crate::error::PythonEvalError;
use crate::function::PythonFunction;
use crate::py_compat::{py_any_to_value, value_to_py_any};

/// Evaluates a Python function with the given Oneil values as positional arguments.
///
/// Converts each argument with [`value_to_py_any`], calls the function, then converts
/// the return value with [`py_any_to_value`]. Returns [`PythonEvalError`] on conversion
/// or Python exception.
pub fn evaluate_python_function(
    function: &PythonFunction,
    args: Vec<(Value, Span)>,
) -> Result<Value, PythonEvalError> {
    Python::attach(|py| {
        let py_args: Vec<_> = args
            .into_iter()
            .map(|(value, _span)| value_to_py_any(value, py))
            .collect::<Vec<_>>();

        let result = function
            .call(py, &py_args)
            .map_err(|e| PythonEvalError::PyErr {
                message: e.to_string(),
                traceback: e.traceback(py).and_then(|tb| tb.format().ok()),
            })?;

        py_any_to_value(&result)
            .map_err(|e| PythonEvalError::InvalidReturnValue { value_repr: e.0 })
    })
}
