//! Python wrapper for Oneil’s [`MeasuredNumber`].

use oneil_output::{BinaryEvalError, MeasuredNumber, Number, Unit};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyFloat, PyTuple};

use super::interval::PyInterval;
use super::unit::PyUnit;

/// Python wrapper for Oneil’s [`MeasuredNumber`].
///
/// A measured number is a number (scalar or interval) with a unit.
#[pyclass(name = "OneilMeasuredNumber", from_py_object)]
#[derive(Debug, Clone)]
pub struct PyMeasuredNumber {
    inner: MeasuredNumber,
}

#[pymethods]
impl PyMeasuredNumber {
    /// Creates a measured number from a value (float or Interval) and a unit.
    #[new]
    fn new(value: &Bound<'_, PyAny>, unit: &Bound<'_, PyUnit>) -> PyResult<Self> {
        let number = py_any_to_number(value)?;
        let unit: Unit = Unit::from(&*unit.borrow());

        let inner = MeasuredNumber::from_number_and_unit(number, unit);
        Ok(Self { inner })
    }

    /// Returns the unit of the measured number.
    fn unit<'py>(&self, py: Python<'py>) -> Bound<'py, PyUnit> {
        Bound::new(py, PyUnit::from(self.inner.unit().clone()))
            .expect("PyUnit construction should not fail")
    }

    /// Returns a tuple of (number, unit). The number is a float or Interval in this measured number’s unit.
    fn as_number_and_unit<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let (number, unit) = self.inner.clone().into_number_and_unit();

        let number_py = number_to_py_any(&number, py);
        let unit_py = Bound::new(py, PyUnit::from(unit))
            .expect("PyUnit construction should not fail")
            .into_any();

        PyTuple::new(py, [number_py, unit_py])
    }

    /// Returns a copy with the given unit. Raises [`ValueError`] if the new unit is not dimensionally equivalent.
    fn with_unit(&self, unit: &Bound<'_, PyUnit>) -> PyResult<Self> {
        let unit: Unit = Unit::from(&*unit.borrow());

        if !self.inner.unit().dimensionally_eq(&unit) {
            return Err(PyErr::new::<PyValueError, _>(
                "units are not dimensionally equivalent",
            ));
        }

        Ok(Self {
            inner: self.inner.clone().with_unit(unit),
        })
    }

    /// Negates the measured number.
    fn neg(&self) -> Self {
        Self {
            inner: self.inner.clone().checked_neg(),
        }
    }

    /// Adds two measured numbers. Raises if units do not match.
    fn checked_add(&self, other: &Bound<'_, Self>) -> PyResult<Self> {
        self.inner
            .clone()
            .checked_add(&other.borrow().inner)
            .map(|inner| Self { inner })
            .map_err(binary_eval_error_to_py_err)
    }

    /// Subtracts two measured numbers. Raises if units do not match.
    fn checked_sub(&self, other: &Bound<'_, Self>) -> PyResult<Self> {
        self.inner
            .clone()
            .checked_sub(&other.borrow().inner)
            .map(|inner| Self { inner })
            .map_err(binary_eval_error_to_py_err)
    }

    /// Multiplies two measured numbers.
    fn checked_mul(&self, other: &Bound<'_, Self>) -> PyResult<Self> {
        self.inner
            .clone()
            .checked_mul(other.borrow().inner.clone())
            .map(|inner| Self { inner })
            .map_err(binary_eval_error_to_py_err)
    }

    /// Divides two measured numbers. Raises if units do not match.
    fn checked_div(&self, other: &Bound<'_, Self>) -> PyResult<Self> {
        self.inner
            .clone()
            .checked_div(other.borrow().inner.clone())
            .map(|inner| Self { inner })
            .map_err(binary_eval_error_to_py_err)
    }

    /// Remainder of two measured numbers. Raises if units do not match.
    fn checked_rem(&self, other: &Bound<'_, Self>) -> PyResult<Self> {
        self.inner
            .clone()
            .checked_rem(&other.borrow().inner)
            .map(|inner| Self { inner })
            .map_err(binary_eval_error_to_py_err)
    }

    /// Square root.
    fn sqrt(&self) -> Self {
        Self {
            inner: self.inner.clone().sqrt(),
        }
    }

    /// Natural logarithm.
    fn ln(&self) -> Self {
        Self {
            inner: self.inner.clone().ln(),
        }
    }

    /// Base-10 logarithm.
    fn log10(&self) -> Self {
        Self {
            inner: self.inner.clone().log10(),
        }
    }

    /// Base-2 logarithm.
    fn log2(&self) -> Self {
        Self {
            inner: self.inner.clone().log2(),
        }
    }

    fn __repr__(&self) -> String {
        let (number, unit) = self.inner.clone().into_number_and_unit();
        format!("MeasuredNumber({:?}, {})", number, unit.display_unit)
    }
}

impl From<MeasuredNumber> for PyMeasuredNumber {
    fn from(m: MeasuredNumber) -> Self {
        Self { inner: m }
    }
}

impl From<PyMeasuredNumber> for MeasuredNumber {
    fn from(p: PyMeasuredNumber) -> Self {
        p.inner
    }
}

impl From<&PyMeasuredNumber> for MeasuredNumber {
    fn from(p: &PyMeasuredNumber) -> Self {
        p.inner.clone()
    }
}

fn binary_eval_error_to_py_err(err: BinaryEvalError) -> PyErr {
    let msg = match &err {
        BinaryEvalError::UnitMismatch { lhs_unit, rhs_unit } => {
            format!("unit mismatch: {lhs_unit} vs {rhs_unit}")
        }

        BinaryEvalError::TypeMismatch { lhs_type, rhs_type } => {
            format!("type mismatch: {lhs_type:?} vs {rhs_type:?}")
        }

        BinaryEvalError::InvalidLhsType {
            expected_type,
            lhs_type,
        } => {
            format!("invalid left-hand side type: expected {expected_type:?}, got {lhs_type:?}")
        }

        BinaryEvalError::InvalidRhsType {
            expected_type,
            rhs_type,
        } => {
            format!("invalid right-hand side type: expected {expected_type:?}, got {rhs_type:?}")
        }

        BinaryEvalError::ExponentHasUnits { exponent_unit } => {
            format!("exponent has units: {exponent_unit}")
        }

        BinaryEvalError::ExponentIsInterval { .. } => {
            "exponent cannot be an interval when base has a unit".to_string()
        }
    };

    PyErr::new::<PyValueError, _>(msg)
}

/// Converts a Python object to a [`Number`]: float -> Scalar, PyInterval -> Interval.
fn py_any_to_number(obj: &Bound<'_, PyAny>) -> PyResult<Number> {
    if let Ok(f) = obj.extract::<f64>() {
        return Ok(Number::Scalar(f));
    }

    if let Ok(py_interval) = obj.extract::<PyInterval>() {
        return Ok(Number::Interval(py_interval.into()));
    }

    Err(PyErr::new::<PyValueError, _>(
        "value must be a float or an Interval",
    ))
}

/// Converts a [`Number`] to a Python object.
fn number_to_py_any<'py>(number: &Number, py: Python<'py>) -> Bound<'py, PyAny> {
    match number {
        Number::Scalar(f) => PyFloat::new(py, *f).into_any(),
        Number::Interval(interval) => Bound::new(py, PyInterval::from(*interval))
            .expect("PyInterval construction should not fail")
            .into_any(),
    }
}
