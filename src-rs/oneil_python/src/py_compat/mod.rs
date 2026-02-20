//! Python compatibility data types and conversion functions.

mod builtin;
mod interval;
mod measured_number;
mod unit;
mod value_convert;

pub use value_convert::{py_any_to_value, value_to_py_any};

use pyo3::prelude::*;

#[pymodule(name = "oneil")]
pub mod oneil_python_module {
    #[pymodule_export]
    pub use super::builtin::values;

    #[pymodule_export]
    pub use super::builtin::units;

    #[pymodule_export]
    pub use super::builtin::functions;

    #[pymodule_export]
    pub use super::interval::PyInterval;

    #[pymodule_export]
    pub use super::measured_number::PyMeasuredNumber;

    #[pymodule_export]
    pub use super::unit::PyUnit;
}
