//! Conversion between Oneil values and Python objects.

use oneil_output::{Interval, Value};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyNotImplemented, PyString};

/// Tries to convert a Python object to an [`Interval`]: accepts [`PyInterval`], or a
/// scalar (PyInt/PyFloat) as an interval with that value as both min and max.
fn py_any_to_py_interval(other: &Bound<'_, PyAny>) -> Option<Interval> {
    if let Ok(py_interval) = other.cast_exact::<PyInterval>() {
        return Some(py_interval.borrow().inner);
    }

    other.extract::<f64>().ok().map(Interval::from)
}

/// Python wrapper for Oneil’s [`Interval`].
///
/// An interval is a closed, connected set of numbers with a minimum and maximum.
#[pyclass(eq, ord, frozen, from_py_object)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct PyInterval {
    inner: Interval,
}

#[pymethods]
impl PyInterval {
    /// Creates a new interval with the given minimum and maximum.
    ///
    /// # Errors
    ///
    /// Raises `ValueError` if either bound is NaN or if min > max.
    #[new]
    fn new(min: f64, max: f64) -> PyResult<Self> {
        // TODO: verify
        if min.is_nan() || max.is_nan() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "min and max must not be NaN",
            ));
        }

        if min > max {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "min must be less than or equal to max",
            ));
        }

        Ok(Self {
            inner: Interval::new_unchecked(min, max),
        })
    }

    /// Returns an empty interval.
    #[staticmethod]
    const fn empty() -> Self {
        Self {
            inner: Interval::empty(),
        }
    }

    /// Returns the zero interval [0, 0].
    #[staticmethod]
    const fn zero() -> Self {
        Self {
            inner: Interval::zero(),
        }
    }

    /// Returns the minimum value of the interval.
    const fn min(&self) -> f64 {
        self.inner.min()
    }

    /// Returns the maximum value of the interval.
    const fn max(&self) -> f64 {
        self.inner.max()
    }

    /// Returns true if the interval is empty.
    const fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns true if the interval is valid (empty or min ≤ max).
    const fn is_valid(&self) -> bool {
        self.inner.is_valid()
    }

    /// Returns the intersection of this interval and the other.
    fn intersection(&self, other: &Bound<'_, Self>) -> Self {
        Self {
            inner: self.inner.intersection(other.borrow().inner),
        }
    }

    /// Returns the tightest interval that contains both this and the other.
    fn tightest_enclosing_interval(&self, other: &Bound<'_, Self>) -> Self {
        Self {
            inner: self.inner.tightest_enclosing_interval(other.borrow().inner),
        }
    }

    /// Returns true if this interval contains the other.
    fn contains(&self, other: &Bound<'_, Self>) -> bool {
        self.inner.contains(&other.borrow().inner)
    }

    /// Returns the square root of the interval.
    fn sqrt(&self) -> Self {
        Self {
            inner: self.inner.sqrt(),
        }
    }

    /// Returns the natural logarithm of the interval.
    fn ln(&self) -> Self {
        Self {
            inner: self.inner.ln(),
        }
    }

    /// Returns the base-10 logarithm of the interval.
    fn log10(&self) -> Self {
        Self {
            inner: self.inner.log10(),
        }
    }

    /// Returns the base-2 logarithm of the interval.
    fn log2(&self) -> Self {
        Self {
            inner: self.inner.log2(),
        }
    }

    /// Raises this interval to the power of the exponent interval.
    fn pow(&self, exponent: &Bound<'_, Self>) -> Self {
        Self {
            inner: self.inner.pow(exponent.borrow().inner),
        }
    }

    /// Escaped subtraction (min-min, max-max); not standard interval arithmetic.
    fn escaped_sub(&self, other: &Bound<'_, Self>) -> Self {
        Self {
            inner: self.inner.escaped_sub(other.borrow().inner),
        }
    }

    /// Escaped division (min/min, max/max); not standard interval arithmetic.
    fn escaped_div(&self, other: &Bound<'_, Self>) -> Self {
        Self {
            inner: self.inner.escaped_div(other.borrow().inner),
        }
    }

    fn __add__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let rhs = match py_any_to_py_interval(other) {
            Some(i) => i,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let result = Bound::new(
            py,
            Self {
                inner: self.inner + rhs,
            },
        )?;

        Ok(result.into_any())
    }

    fn __radd__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let lhs = match py_any_to_py_interval(other) {
            Some(i) => i,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let result = Bound::new(
            py,
            Self {
                inner: lhs + self.inner,
            },
        )?;

        Ok(result.into_any())
    }

    fn __sub__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let rhs = match py_any_to_py_interval(other) {
            Some(i) => i,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let result = Bound::new(
            py,
            Self {
                inner: self.inner - rhs,
            },
        )?;

        Ok(result.into_any())
    }

    fn __rsub__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let lhs = match py_any_to_py_interval(other) {
            Some(i) => i,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let result = Bound::new(
            py,
            Self {
                inner: lhs - self.inner,
            },
        )?;

        Ok(result.into_any())
    }

    fn __mul__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let rhs = match py_any_to_py_interval(other) {
            Some(i) => i,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let result = Bound::new(
            py,
            Self {
                inner: self.inner * rhs,
            },
        )?;

        Ok(result.into_any())
    }

    fn __rmul__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let lhs = match py_any_to_py_interval(other) {
            Some(i) => i,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let result = Bound::new(
            py,
            Self {
                inner: lhs * self.inner,
            },
        )?;

        Ok(result.into_any())
    }

    fn __truediv__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let rhs = match py_any_to_py_interval(other) {
            Some(i) => i,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let result = Bound::new(
            py,
            Self {
                inner: self.inner / rhs,
            },
        )?;

        Ok(result.into_any())
    }

    fn __rtruediv__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let lhs = match py_any_to_py_interval(other) {
            Some(i) => i,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let result = Bound::new(
            py,
            Self {
                inner: lhs / self.inner,
            },
        )?;

        Ok(result.into_any())
    }

    fn __mod__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        // if `other` is a scalar, use the specialized version of the modulo operation
        if let Ok(scalar) = other.extract::<f64>() {
            let result = Bound::new(
                py,
                Self {
                    inner: self.inner % scalar,
                },
            )?;

            return Ok(result.into_any());
        }

        let rhs = match py_any_to_py_interval(other) {
            Some(i) => i,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let result = Bound::new(
            py,
            Self {
                inner: self.inner % rhs,
            },
        )?;

        Ok(result.into_any())
    }

    fn __rmod__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let lhs = match py_any_to_py_interval(other) {
            Some(i) => i,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let result = Bound::new(
            py,
            Self {
                inner: lhs % self.inner,
            },
        )?;

        Ok(result.into_any())
    }

    fn __neg__(&self) -> Self {
        Self { inner: -self.inner }
    }

    const fn __pos__(&self) -> Self {
        Self { inner: self.inner }
    }

    fn __pow__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        modulus: Option<&Bound<'_, PyAny>>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if modulus.is_some() {
            return Ok(PyNotImplemented::get(py).to_owned().into_any());
        }

        let exponent = match py_any_to_py_interval(other) {
            Some(i) => i,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let result = Bound::new(
            py,
            Self {
                inner: self.inner.pow(exponent),
            },
        )?;

        Ok(result.into_any())
    }

    fn __rpow__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        modulus: Option<&Bound<'_, PyAny>>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if modulus.is_some() {
            return Ok(PyNotImplemented::get(py).to_owned().into_any());
        }

        let base = match py_any_to_py_interval(other) {
            Some(i) => i,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let result = Bound::new(
            py,
            Self {
                inner: base.pow(self.inner),
            },
        )?;

        Ok(result.into_any())
    }

    fn __repr__(&self) -> String {
        if self.inner.is_empty() {
            "Interval.empty()".to_string()
        } else {
            format!("Interval({}, {})", self.inner.min(), self.inner.max())
        }
    }
}

/// Converts an Oneil [`Value`] into a Python object.
///
/// - Boolean and string values are converted to the equivalent Python `bool` and `str`.
/// - Number and measured number conversions are not yet implemented.
pub fn value_to_py_any(value: &Value, py: Python<'_>) -> Py<PyAny> {
    match value {
        Value::Boolean(b) => PyBool::new(py, *b).to_owned().into_any().unbind(),
        Value::String(s) => PyString::new(py, s.as_str()).into_any().unbind(),
        Value::Number(_) => todo!("convert Oneil Number to Python"),
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
    if obj.is_instance_of::<pyo3::types::PyInt>() || obj.is_instance_of::<pyo3::types::PyFloat>() {
        todo!("convert Python number to Oneil Number");
    }
    Err(PyErr::new::<PyTypeError, _>(
        "expected bool, str, or number; conversion for number not yet implemented",
    ))
}
