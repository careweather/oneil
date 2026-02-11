//! Evaluation of Python functions from Oneil.

use oneil_output::Value;
use oneil_shared::span::Span;
use pyo3::Python;

use crate::py_value::{py_any_to_value, value_to_py_any};
use crate::PythonFunction;

/// Error from calling a Python function (argument conversion, call, or result conversion failed).
#[derive(Debug)]
pub struct PythonCallError {
    /// Span of the function identifier in the Oneil source.
    pub identifier_span: Span,
    /// Error message from Python or from conversion.
    pub message: String,
}

/// Evaluates a Python function with the given Oneil values as positional arguments.
///
/// Converts each argument with [`value_to_py_any`], calls the function, then converts
/// the return value with [`py_any_to_value`]. Returns [`PythonCallError`] on conversion
/// or Python exception.
pub fn evaluate_python_function(
    function: &PythonFunction,
    _identifier: &str,
    identifier_span: Span,
    args: Vec<(Value, Span)>,
) -> Result<Value, PythonCallError> {
    let to_call_err = |e: pyo3::PyErr| PythonCallError {
        identifier_span,
        message: e.to_string(),
    };

    Python::attach(|py| {
        let py_args: Vec<_> = args
            .into_iter()
            .map(|(value, _span)| value_to_py_any(value, py))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| to_call_err(e))?;

        let result = function.call(py, &py_args).map_err(|e| to_call_err(e))?;
        py_any_to_value(&result).map_err(|e| to_call_err(e))
    })
}
