//! Conversion between Oneil values and Python objects.

use oneil_output::{Number, Value};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyFloat, PyString};

use crate::py_value::PyInterval;

/// Converts an Oneil [`Value`] into a Python object.
///
/// - Boolean and string values are converted to the equivalent Python `bool` and `str`.
/// - [`Number::Scalar`] becomes a Python `float`; [`Number::Interval`] becomes a [`PyInterval`].
/// - Measured number conversion is not yet implemented.
pub fn value_to_py_any(value: &Value, py: Python<'_>) -> Py<PyAny> {
    match value {
        Value::Boolean(b) => PyBool::new(py, *b).to_owned().into_any().unbind(),
        Value::String(s) => PyString::new(py, s.as_str()).into_any().unbind(),
        Value::Number(number) => match number {
            Number::Scalar(value) => PyFloat::new(py, *value).into_any().unbind(),
            Number::Interval(interval) => Bound::new(py, PyInterval::from(*interval))
                .expect("PyInterval construction should not fail")
                .into_any()
                .unbind(),
        },

        Value::MeasuredNumber(_) => todo!("convert Oneil MeasuredNumber to Python"),
    }
}

/// Converts a Python object into an Oneil [`Value`].
///
/// - Python `bool` and `str` are converted to the equivalent Oneil values.
/// - Number and measured number conversions are not yet implemented.
/// - Returns a type error if the object is not a supported type.
pub fn py_any_to_value(obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    if let Ok(py_bool) = obj.cast_exact::<PyBool>() {
        return Ok(Value::Boolean(py_bool.is_true()));
    }

    if let Ok(py_str) = obj.cast_exact::<PyString>() {
        return Ok(Value::String(py_str.to_string()));
    }

    if let Ok(float) = obj.extract::<f64>() {
        return Ok(Value::Number(Number::Scalar(float)));
    }

    if let Ok(interval) = obj.extract::<PyInterval>() {
        return Ok(Value::Number(Number::Interval(interval.into())));
    }

    Err(PyErr::new::<PyTypeError, _>(
        "expected bool, str, or number; conversion for number not yet implemented",
    ))
}
