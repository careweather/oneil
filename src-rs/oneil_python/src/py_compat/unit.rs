//! Python wrapper for Oneil’s [`Unit`].

use indexmap::IndexMap;
use oneil_output::{Dimension, DimensionMap, DisplayUnit, Unit};
use pyo3::prelude::*;
use pyo3::types::PyDict;

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
    /// Creates a unit from keyword arguments.
    ///
    /// - `dimensions`: optional dict mapping dimension keys to exponents (str -> float).
    ///   Valid keys: `"kg"`, `"m"`, `"s"`, `"K"`, `"A"`, `"b"`, `"$"`, `"mol"`, `"cd"`.
    /// - `magnitude`: optional magnitude (default 1.0).
    /// - `is_db`: optional decibel flag (default false).
    /// - `display_unit`: required display name (used as a single unit with exponent 1).
    #[new]
    #[pyo3(signature = (*, dimensions=None, magnitude=None, is_db=None, display_unit))]
    fn new(
        dimensions: Option<&Bound<'_, PyDict>>,
        magnitude: Option<f64>,
        is_db: Option<bool>,
        display_unit: String,
    ) -> PyResult<Self> {
        let dimension_map = match dimensions {
            Some(d) => dimension_map_from_dict(d)?,
            None => DimensionMap::unitless(),
        };
        let magnitude = magnitude.unwrap_or(1.0);
        let is_db = is_db.unwrap_or(false);
        let display_unit = DisplayUnit::Unit {
            name: display_unit,
            exponent: 1.0,
        };

        let inner = Unit {
            dimension_map,
            magnitude,
            is_db,
            display_unit,
        };

        Ok(Self { inner })
    }

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

    /// Returns the dimensions as a dict mapping dimension keys to exponents (str -> float).
    ///
    /// Keys are the same as for the constructor: `"kg"`, `"m"`, `"s"`, `"K"`, `"A"`, `"b"`, `"$"`, `"mol"`, `"cd"`.
    fn get_dimensions<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);

        for (dim, exp) in self.inner.dimension_map.as_map() {
            dict.set_item(dimension_to_key(*dim), *exp)?;
        }

        Ok(dict)
    }

    /// Returns the magnitude of the unit (e.g. 1000 for km).
    #[getter]
    const fn magnitude(&self) -> f64 {
        self.inner.magnitude
    }

    /// Returns true if the unit is a decibel unit.
    #[getter]
    const fn is_db(&self) -> bool {
        self.inner.is_db
    }

    /// Returns a human-readable display string for the unit.
    #[getter]
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

    /// Returns true if this unit's dimensions match the given dimension map (dict mapping dimension keys to exponents).
    fn dimensions_match(&self, dimensions: &Bound<'_, PyDict>) -> PyResult<bool> {
        let dimension_map = dimension_map_from_dict(dimensions)?;
        Ok(self.inner.dimensions_match(&dimension_map))
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

/// Maps a Python dimension key (dict key) to a [`Dimension`]. Returns `None` for invalid keys.
fn dimension_from_key(key: &str) -> Option<Dimension> {
    match key {
        "kg" => Some(Dimension::Mass),
        "m" => Some(Dimension::Distance),
        "s" => Some(Dimension::Time),
        "K" => Some(Dimension::Temperature),
        "A" => Some(Dimension::Current),
        "b" => Some(Dimension::Information),
        "$" => Some(Dimension::Currency),
        "mol" => Some(Dimension::Substance),
        "cd" => Some(Dimension::LuminousIntensity),
        _ => None,
    }
}

/// Maps a [`Dimension`] to the Python dimension key (dict key).
const fn dimension_to_key(dim: Dimension) -> &'static str {
    match dim {
        Dimension::Mass => "kg",
        Dimension::Distance => "m",
        Dimension::Time => "s",
        Dimension::Temperature => "K",
        Dimension::Current => "A",
        Dimension::Information => "b",
        Dimension::Currency => "$",
        Dimension::Substance => "mol",
        Dimension::LuminousIntensity => "cd",
    }
}

/// Builds a [`DimensionMap`] from a Python dict (string -> float). Invalid keys return an error.
fn dimension_map_from_dict(dict: &Bound<'_, PyDict>) -> PyResult<DimensionMap> {
    let mut map = IndexMap::new();

    for (key, value) in dict.iter() {
        let key_str: String = key.extract()?;

        let dim = dimension_from_key(&key_str).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!("invalid dimension key: {key_str:?}"))
        })?;
        let exponent: f64 = value.extract()?;

        map.insert(dim, exponent);
    }

    Ok(DimensionMap::new(map))
}
