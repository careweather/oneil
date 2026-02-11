//! Conversion between Oneil values and Python objects.

use oneil_output::{Number, Value};
use pyo3::exceptions::PyTypeError;
use pyo3::types::{PyBool, PyString};
use pyo3::{IntoPyObjectExt, prelude::*};

use super::interval::PyInterval;
use super::measured_number::PyMeasuredNumber;

/// Converts an Oneil [`Value`] into a Python object.
///
/// - Boolean and string values are converted to the equivalent Python `bool` and `str`.
/// - [`Number::Scalar`] becomes a Python `float`; [`Number::Interval`] becomes a [`PyInterval`].
/// - [`Value::MeasuredNumber`] becomes a [`PyMeasuredNumber`].
pub fn value_to_py_any(value: Value, py: Python<'_>) -> Bound<'_, PyAny> {
    match value {
        Value::Boolean(b) => b
            .into_bound_py_any(py)
            .expect("boolean conversion should not fail"),

        Value::String(s) => s
            .into_bound_py_any(py)
            .expect("string conversion should not fail"),

        Value::Number(number) => match number {
            Number::Scalar(value) => value
                .into_bound_py_any(py)
                .expect("scalar conversion should not fail"),

            Number::Interval(interval) => PyInterval::from(interval)
                .into_bound_py_any(py)
                .expect("interval conversion should not fail"),
        },

        Value::MeasuredNumber(m) => PyMeasuredNumber::from(m)
            .into_bound_py_any(py)
            .expect("measured number conversion should not fail"),
    }
}

/// Converts a Python object into an Oneil [`Value`].
///
/// - Python `bool` and `str` are converted to the equivalent Oneil values.
/// - Python `float` becomes [`Number::Scalar`]; [`PyInterval`] becomes [`Number::Interval`].
/// - [`PyMeasuredNumber`] becomes [`Value::MeasuredNumber`].
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

    if let Ok(py_mn) = obj.extract::<PyMeasuredNumber>() {
        return Ok(Value::MeasuredNumber(py_mn.into()));
    }

    Err(PyErr::new::<PyTypeError, _>(
        "expected bool, str, float, Interval, or MeasuredNumber",
    ))
}
