mod interval;
mod number;
mod type_;
mod unit;
pub mod util;
mod value_impl;

use crate::EvalError;

pub use self::interval::Interval;
pub use self::number::{MeasuredNumber, Number};
pub use self::type_::{NumberType, ValueType};
pub use self::unit::{Dimension, SizedUnit, Unit};
pub use self::value_impl::Value;
