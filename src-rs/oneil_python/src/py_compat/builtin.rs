//! Builtin functions and types for Python compatibility.

use oneil_builtins::{BuiltinFunction, BuiltinRef};
use oneil_output::{DisplayUnit, Value};
use oneil_shared::span::{SourceLocation, Span};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::unit::PyUnit;
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

/// Python submodule exposing Oneil builtin units (e.g. `m`, `kg`, `s`) as `PyUnit` objects.
///
/// For units that support SI prefixes, prefixed variants are also included
/// (e.g. `km`, `mm`, `kg`, `mg`).
#[pymodule]
pub fn units(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let builtins = BuiltinRef::new();

    let prefixes: Vec<_> = builtins.builtin_prefixes().collect();

    for (alias, unit) in builtins.builtin_units() {
        let python_alias = alias.replace('%', "percent").replace('$', "dollar");

        let py_unit = PyUnit::from(unit.clone());
        m.add(&python_alias, py_unit)?;

        if builtins.unit_supports_si_prefixes(alias) {
            for &(prefix, multiplier) in &prefixes {
                let prefixed_name = format!("{prefix}{alias}");
                let python_prefixed_name = format!("{prefix}{python_alias}");

                let prefixed_display = DisplayUnit::Unit {
                    name: prefixed_name.clone(),
                    exponent: 1.0,
                };

                let prefixed_unit = unit
                    .clone()
                    .mul_magnitude(multiplier)
                    .with_unit_display_expr(prefixed_display);

                let py_prefixed_unit = PyUnit::from(prefixed_unit);
                m.add(python_prefixed_name, py_prefixed_unit)?;
            }
        }
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
