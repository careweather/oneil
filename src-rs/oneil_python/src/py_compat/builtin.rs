//! Builtin functions and types for Python compatibility.

use oneil_builtins::{BuiltinFunction, BuiltinRef};
use oneil_output::Value;
use oneil_shared::span::{SourceLocation, Span};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::value_convert::{ValueReprError, py_any_to_value, value_to_py_any};

/// Python submodule exposing Oneil builtin values (e.g. `pi`, `e`) as Python objects.
#[pymodule]
pub fn values(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let builtins = BuiltinRef::new();

    for (name, value) in builtins.builtin_values() {
        m.add(name, value_to_py_any(value.clone(), m.py()))?;
    }

    Ok(())
}

/// Python submodule exposing Oneil builtin functions (e.g. `min`, `max`, `sqrt`) as callables.
#[pymodule]
pub fn functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<BuiltinFunctionWrapper>()?;

    let builtins = BuiltinRef::new();
    for (name, function) in builtins.builtin_functions() {
        let wrapper = BuiltinFunctionWrapper {
            function: function.clone(),
        };

        let py_wrapper = Py::new(m.py(), wrapper)?;

        m.add(name, py_wrapper)?;
    }

    Ok(())
}

/// Returns a dummy span for Python-originated calls (no source location).
///
/// This is used when only the message of the resulting error is needed.
const fn dummy_span() -> Span {
    let loc = SourceLocation {
        offset: 0,
        line: 1,
        column: 1,
    };

    Span::empty(loc)
}

/// Wrapper that exposes a single Oneil builtin function as a Python callable.
#[pyclass]
pub struct BuiltinFunctionWrapper {
    function: BuiltinFunction,
}

#[pymethods]
impl BuiltinFunctionWrapper {
    /// Calls the builtin function with the given positional arguments.
    #[pyo3(signature = (*args))]
    fn __call__<'py>(
        &self,
        py: Python<'py>,
        args: &Bound<'_, PyTuple>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let function = self.function.function;

        let oneil_args: Vec<(Value, Span)> = args
            .iter()
            .map(|obj| py_any_to_value(&obj).map(|v| (v, dummy_span())))
            .collect::<Result<Vec<_>, ValueReprError>>()
            .map_err(|v| pyo3::exceptions::PyTypeError::new_err(v.to_string()))?;

        let call_span = dummy_span();

        match function(call_span, oneil_args) {
            Ok(value) => Ok(value_to_py_any(value, py)),
            Err(errors) => {
                let message = errors
                    .first()
                    .expect("evaluation error should have at least one error")
                    .to_string();

                Err(pyo3::exceptions::PyValueError::new_err(message))
            }
        }
    }
}
