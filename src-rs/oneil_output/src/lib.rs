//! Output and value types for the Oneil programming language.
//!
//! This crate provides the data structures for evaluated model output and
//! for runtime values (numbers, units, intervals, etc.).

mod dependency;
pub mod error;
mod evaluated_value;
mod interval;
mod model;
mod number_kinds;
mod type_;
mod unit;
pub mod util;
mod value;

pub use dependency::{BuiltinDependency, DependencySet, ExternalDependency, ParameterDependency};
pub use error::{
    BinaryEvalError, EvalError, EvalWarning, ExpectedArgumentCount, ExpectedType, ModelEvalErrors,
    UnaryEvalError, UnitConversionError,
};
pub use evaluated_value::EvaluatedValue;
pub use interval::Interval;
pub use model::{DebugInfo, Model, Parameter, PrintLevel, Test, TestResult};
pub use number_kinds::{MeasuredNumber, Number};
pub use type_::{NumberType, ValueType};
pub use unit::{Dimension, DimensionMap, DisplayUnit, Unit};
pub use value::Value;
