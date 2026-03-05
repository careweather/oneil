//! Python wrapper for Oneil’s [`MeasuredNumber`].

use oneil_output::{BinaryEvalError, MeasuredNumber, Number, Unit};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyNotImplemented;
use pyo3::types::{PyFloat, PyTuple};

use super::interval::PyInterval;
use super::unit::PyUnit;

/// Python wrapper for Oneil’s [`MeasuredNumber`].
///
/// A measured number is a number (scalar or interval) with a unit.
#[pyclass(name = "MeasuredNumber", module = "oneil", frozen, from_py_object)]
#[derive(Debug, Clone)]
pub struct PyMeasuredNumber {
    inner: MeasuredNumber,
}

impl PyMeasuredNumber {
    fn verify_unit_is_dimensionally_equivalent(&self, unit: &Unit) -> PyResult<()> {
        if !self.inner.unit().dimensionally_eq(unit) {
            return Err(PyErr::new::<PyValueError, _>(
                "units are not dimensionally equivalent",
            ));
        }

        Ok(())
    }

    fn verify_unit_is_dimensionless(&self) -> PyResult<()> {
        if !self.inner.unit().is_dimensionless() {
            return Err(PyErr::new::<PyValueError, _>("units are not dimensionless"));
        }

        Ok(())
    }
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
    #[expect(clippy::wrong_self_convention, reason = "this is for Python, not Rust")]
    fn into_number_and_unit<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let (number, unit) = self.inner.clone().into_number_and_unit();

        let number_py = number_to_py_any(&number, py);
        let unit_py = Bound::new(py, PyUnit::from(unit))
            .expect("PyUnit construction should not fail")
            .into_any();

        PyTuple::new(py, [number_py, unit_py])
    }

    /// Converts this measured number to a number (float or Interval) in the given unit.
    #[expect(clippy::wrong_self_convention, reason = "this is for Python, not Rust")]
    fn into_number_using_unit<'py>(
        &self,
        unit: &Bound<'_, PyUnit>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let unit: Unit = Unit::from(&*unit.borrow());

        self.verify_unit_is_dimensionally_equivalent(&unit)?;

        let number = self.inner.clone().into_number_using_unit(&unit);
        Ok(number_to_py_any(&number, py))
    }

    /// Converts this measured number to a unitless number (float or Interval).
    ///
    /// This will fail if the value's unit is not dimensionless.
    #[expect(clippy::wrong_self_convention, reason = "this is for Python, not Rust")]
    fn into_unitless_number<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.verify_unit_is_dimensionless()?;

        let number = self.inner.clone().into_number_using_unit(&Unit::one());
        Ok(number_to_py_any(&number, py))
    }

    /// Returns a copy with the given unit. Raises [`ValueError`] if the new unit is not dimensionally equivalent.
    fn with_unit(&self, unit: &Bound<'_, PyUnit>) -> PyResult<Self> {
        let unit: Unit = Unit::from(&*unit.borrow());

        self.verify_unit_is_dimensionally_equivalent(&unit)?;

        Ok(Self {
            inner: self.inner.clone().with_unit(unit),
        })
    }

    fn __neg__(&self) -> Self {
        Self {
            inner: self.inner.clone().checked_neg(),
        }
    }

    fn __pos__(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }

    fn __add__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let rhs = match py_any_to_measured_number_with_unit(other, self.inner.unit()) {
            Some(m) => m,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let inner = self
            .inner
            .clone()
            .checked_add(&rhs)
            .map_err(binary_eval_error_to_py_err)?;

        Bound::new(py, Self { inner }).map(|b| b.into_any())
    }

    fn __radd__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let lhs = match py_any_to_measured_number_with_unit(other, self.inner.unit()) {
            Some(m) => m,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let inner = lhs
            .checked_add(&self.inner)
            .map_err(binary_eval_error_to_py_err)?;

        Bound::new(py, Self { inner }).map(|b| b.into_any())
    }

    fn __sub__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let rhs = match py_any_to_measured_number_with_unit(other, self.inner.unit()) {
            Some(m) => m,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let inner = self
            .inner
            .clone()
            .checked_sub(&rhs)
            .map_err(binary_eval_error_to_py_err)?;

        Bound::new(py, Self { inner }).map(|b| b.into_any())
    }

    fn __rsub__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let lhs = match py_any_to_measured_number_with_unit(other, self.inner.unit()) {
            Some(m) => m,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let inner = lhs
            .checked_sub(&self.inner)
            .map_err(binary_eval_error_to_py_err)?;

        Bound::new(py, Self { inner }).map(|b| b.into_any())
    }

    fn __mul__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let rhs = match py_any_to_measured_number(other) {
            Some(m) => m,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let inner = self
            .inner
            .clone()
            .checked_mul(rhs)
            .map_err(binary_eval_error_to_py_err)?;

        Bound::new(py, Self { inner }).map(|b| b.into_any())
    }

    fn __rmul__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let lhs = match py_any_to_measured_number(other) {
            Some(m) => m,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let inner = lhs
            .checked_mul(self.inner.clone())
            .map_err(binary_eval_error_to_py_err)?;

        Bound::new(py, Self { inner }).map(|b| b.into_any())
    }

    fn __truediv__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let rhs = match py_any_to_measured_number(other) {
            Some(m) => m,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let inner = self
            .inner
            .clone()
            .checked_div(rhs)
            .map_err(binary_eval_error_to_py_err)?;

        Bound::new(py, Self { inner }).map(|b| b.into_any())
    }

    fn __rtruediv__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let lhs = match py_any_to_measured_number(other) {
            Some(m) => m,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let inner = lhs
            .checked_div(self.inner.clone())
            .map_err(binary_eval_error_to_py_err)?;

        Bound::new(py, Self { inner }).map(|b| b.into_any())
    }

    fn __mod__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let rhs = match py_any_to_measured_number_with_unit(other, self.inner.unit()) {
            Some(m) => m,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let inner = self
            .inner
            .clone()
            .checked_rem(&rhs)
            .map_err(binary_eval_error_to_py_err)?;

        Bound::new(py, Self { inner }).map(|b| b.into_any())
    }

    fn __rmod__<'py>(
        &self,
        other: &Bound<'_, PyAny>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let lhs = match py_any_to_measured_number_with_unit(other, self.inner.unit()) {
            Some(m) => m,
            None => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let inner = lhs
            .checked_rem(&self.inner)
            .map_err(binary_eval_error_to_py_err)?;

        Bound::new(py, Self { inner }).map(|b| b.into_any())
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

        let exponent_number = match py_any_to_number(other) {
            Ok(n) => n,
            Err(_) => return Ok(PyNotImplemented::get(py).to_owned().into_any()),
        };

        let inner = self
            .inner
            .clone()
            .checked_pow(&exponent_number)
            .map_err(binary_eval_error_to_py_err)?;

        Bound::new(py, Self { inner }).map(|b| b.into_any())
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let rhs = match py_any_to_measured_number_with_unit(other, self.inner.unit()) {
            Some(m) => m,
            None => return Ok(false),
        };

        self.inner
            .checked_eq(&rhs)
            .map_err(binary_eval_error_to_py_err)
    }

    fn __ne__(&self, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let rhs = match py_any_to_measured_number_with_unit(other, self.inner.unit()) {
            Some(m) => m,
            None => return Ok(true),
        };

        self.inner
            .checked_eq(&rhs)
            .map(|eq| !eq)
            .map_err(binary_eval_error_to_py_err)
    }

    fn __lt__(&self, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let rhs = match py_any_to_measured_number_with_unit(other, self.inner.unit()) {
            Some(m) => m,
            None => return Ok(false),
        };

        self.inner
            .checked_lt(&rhs)
            .map_err(binary_eval_error_to_py_err)
    }

    fn __le__(&self, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let rhs = match py_any_to_measured_number_with_unit(other, self.inner.unit()) {
            Some(m) => m,
            None => return Ok(false),
        };

        self.inner
            .checked_lte(&rhs)
            .map_err(binary_eval_error_to_py_err)
    }

    fn __gt__(&self, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let rhs = match py_any_to_measured_number_with_unit(other, self.inner.unit()) {
            Some(m) => m,
            None => return Ok(false),
        };

        self.inner
            .checked_gt(&rhs)
            .map_err(binary_eval_error_to_py_err)
    }

    fn __ge__(&self, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let rhs = match py_any_to_measured_number_with_unit(other, self.inner.unit()) {
            Some(m) => m,
            None => return Ok(false),
        };

        self.inner
            .checked_gte(&rhs)
            .map_err(binary_eval_error_to_py_err)
    }

    /// Escaped subtraction (min-min, max-max). Raises if units do not match.
    fn escaped_sub(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let rhs = py_any_to_measured_number_with_unit(other, self.inner.unit())
            .ok_or_else(|| PyErr::new::<PyValueError, _>("expected MeasuredNumber"))?;

        self.inner
            .clone()
            .checked_escaped_sub(&rhs)
            .map(|inner| Self { inner })
            .map_err(binary_eval_error_to_py_err)
    }

    /// Escaped division (min/min, max/max). Raises if units do not match.
    fn escaped_div(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let rhs = py_any_to_measured_number(other)
            .ok_or_else(|| PyErr::new::<PyValueError, _>("expected MeasuredNumber"))?;

        self.inner
            .clone()
            .checked_escaped_div(rhs)
            .map(|inner| Self { inner })
            .map_err(binary_eval_error_to_py_err)
    }

    /// Returns the tightest enclosing interval of this and the other measured number. Raises if units do not match.
    fn min_max(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let rhs = py_any_to_measured_number_with_unit(other, self.inner.unit())
            .ok_or_else(|| PyErr::new::<PyValueError, _>("expected MeasuredNumber"))?;

        self.inner
            .clone()
            .checked_min_max(&rhs)
            .map(|inner| Self { inner })
            .map_err(binary_eval_error_to_py_err)
    }

    /// Returns the minimum value of the measured number (as a scalar measured number).
    fn min(&self) -> Self {
        Self {
            inner: self.inner.min(),
        }
    }

    /// Returns the maximum value of the measured number (as a scalar measured number).
    fn max(&self) -> Self {
        Self {
            inner: self.inner.max(),
        }
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

    /// Absolute value.
    fn abs(&self) -> Self {
        Self {
            inner: self.inner.clone().abs(),
        }
    }

    /// Rounds down to the nearest integer (in this number's unit).
    fn floor(&self) -> Self {
        Self {
            inner: self.inner.clone().floor(),
        }
    }

    /// Rounds up to the nearest integer (in this number's unit).
    fn ceiling(&self) -> Self {
        Self {
            inner: self.inner.clone().ceiling(),
        }
    }

    /// Returns the tightest enclosing measured number of this and the given number (float or Interval) in the same unit.
    fn min_max_number(&self, rhs: &Bound<'_, PyAny>) -> PyResult<Self> {
        let number = py_any_to_number(rhs)?;
        Ok(Self {
            inner: self.inner.clone().min_max_number(number),
        })
    }

    fn __repr__(&self) -> String {
        let (number, unit) = self.inner.clone().into_number_and_unit();

        let number_repr = match number {
            Number::Scalar(f) => f.to_string(),
            Number::Interval(interval) => {
                format!("Interval({}, {})", interval.min(), interval.max())
            }
        };

        format!("MeasuredNumber({number_repr}, {})", unit.display_unit)
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

/// Tries to convert a Python object to a [`MeasuredNumber`]: accepts [`PyMeasuredNumber`], or a
/// float/interval which is interpreted as a measured value in the given `unit`.
///
/// Used by binary operators to accept `PyAny` and return `NotImplemented` when conversion fails.
fn py_any_to_measured_number_with_unit(
    other: &Bound<'_, PyAny>,
    unit: &Unit,
) -> Option<MeasuredNumber> {
    if let Ok(py_mn) = other.extract::<PyMeasuredNumber>() {
        return Some(MeasuredNumber::from(py_mn));
    }

    py_any_to_number(other)
        .ok()
        .map(|number| MeasuredNumber::from_number_and_unit(number, unit.clone()))
}

/// Tries to convert a Python object to a [`MeasuredNumber`]: accepts [`PyMeasuredNumber`], or a
/// float/interval which is interpreted as a measured value with unit `1`.
///
/// Used by binary operators to accept `PyAny` and return `NotImplemented` when conversion fails.
fn py_any_to_measured_number(other: &Bound<'_, PyAny>) -> Option<MeasuredNumber> {
    if let Ok(py_mn) = other.extract::<PyMeasuredNumber>() {
        return Some(MeasuredNumber::from(py_mn));
    }

    py_any_to_number(other)
        .ok()
        .map(|number| MeasuredNumber::from_number_and_unit(number, Unit::one()))
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
