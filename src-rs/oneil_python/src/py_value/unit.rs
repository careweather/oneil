//! Python wrapper for Oneil’s [`Unit`].

use oneil_output::Unit;
use pyo3::prelude::*;

/// Python wrapper for Oneil’s [`Unit`].
///
/// A unit has dimensions, a magnitude (e.g. 1000 for km), an optional decibel flag,
/// and display information. Units are not compared for equality by default; use
/// `dimensionally_eq` or `numerically_eq` as appropriate.
#[pyclass(name = "OneilUnit", from_py_object)]
#[derive(Debug, Clone)]
pub struct PyUnit {
    inner: Unit,
}

#[pymethods]
impl PyUnit {
    /// Creates the unitless unit.
    #[staticmethod]
    fn unitless() -> Self {
        Self {
            inner: Unit::unitless(),
        }
    }

    /// Returns true if the unit is unitless.
    fn is_unitless(&self) -> bool {
        self.inner.is_unitless()
    }

    /// Returns the magnitude of the unit (e.g. 1000 for km).
    fn magnitude(&self) -> f64 {
        self.inner.magnitude
    }

    /// Returns true if the unit is a decibel unit.
    fn is_db(&self) -> bool {
        self.inner.is_db
    }

    /// Returns a human-readable display string for the unit.
    fn display_string(&self) -> String {
        format!("{}", self.inner.display_unit)
    }

    /// Returns a copy of this unit with the decibel flag set.
    fn with_is_db_as(&self, is_db: bool) -> Self {
        Self {
            inner: self.inner.clone().with_is_db_as(is_db),
        }
    }

    /// Returns a copy of this unit with magnitude multiplied by the given factor.
    fn mul_magnitude(&self, magnitude: f64) -> Self {
        Self {
            inner: self.inner.clone().mul_magnitude(magnitude),
        }
    }

    /// Returns this unit raised to the given exponent.
    fn pow(&self, exponent: f64) -> Self {
        Self {
            inner: self.inner.clone().pow(exponent),
        }
    }

    /// Returns true if this unit has the same dimensions as the other.
    fn dimensionally_eq(&self, other: &Bound<'_, Self>) -> bool {
        self.inner.dimensionally_eq(&other.borrow().inner)
    }

    /// Returns true if this unit is numerically equal to the other (dimensions, magnitude, and is_db).
    fn numerically_eq(&self, other: &Bound<'_, Self>) -> bool {
        self.inner.numerically_eq(&other.borrow().inner)
    }

    fn __mul__(&self, other: &Bound<'_, Self>) -> Self {
        Self {
            inner: self.inner.clone() * other.borrow().inner.clone(),
        }
    }

    fn __rmul__(&self, other: &Bound<'_, Self>) -> Self {
        Self {
            inner: other.borrow().inner.clone() * self.inner.clone(),
        }
    }

    fn __truediv__(&self, other: &Bound<'_, Self>) -> Self {
        Self {
            inner: self.inner.clone() / other.borrow().inner.clone(),
        }
    }

    fn __rtruediv__(&self, other: &Bound<'_, Self>) -> Self {
        Self {
            inner: other.borrow().inner.clone() / self.inner.clone(),
        }
    }

    fn __repr__(&self) -> String {
        format!("Unit({})", self.inner.display_unit)
    }
}

impl From<Unit> for PyUnit {
    fn from(unit: Unit) -> Self {
        Self { inner: unit }
    }
}

impl From<PyUnit> for Unit {
    fn from(py_unit: PyUnit) -> Self {
        py_unit.inner
    }
}

impl From<&PyUnit> for Unit {
    fn from(py_unit: &PyUnit) -> Self {
        py_unit.inner.clone()
    }
}
