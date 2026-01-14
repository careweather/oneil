//! Oneil values and their related types
//!
//! For a more detailed overview of how values
//! are represented, see the `README.md` for
//! this crate.

mod error;
mod interval;
mod number;
mod type_;
mod unit;
pub mod util;
mod value_impl;

pub use self::interval::Interval;
pub use self::number::{MeasuredNumber, Number};
pub use self::type_::{NumberType, ValueType};
pub use self::unit::{Dimension, DimensionMap, DisplayUnit, Unit};
pub use self::value_impl::Value;
