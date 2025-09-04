//! # Oneil Model
//!
//! This crate provides the core data structures and abstractions for representing
//! models in the Oneil programming language. It defines the fundamental building
//! blocks for modeling systems, including parameters, expressions, tests, and
//! model organization.
//!
//! "IR" stands for "Intermediate Representation".
//!
//! This crate follows a functional programming approach with immutable data structures.
//! All major types implement `Clone`, `Debug`, and `PartialEq` for easy manipulation
//! and testing.

// TODO: get rid of this after prototyping
#![allow(missing_docs)]

mod debug_info;
mod expr;
mod model;
mod model_import;
mod parameter;
mod reference;
mod span;
mod test;
mod unit;

pub use debug_info::TraceLevel;
pub use expr::{
    BinaryOp, ComparisonOp, Expr, ExprWithSpan, FunctionName, Literal, UnaryOp, Variable,
};
pub use model::{Model, ModelCollection};
pub use model_import::{
    ReferenceImport, ReferenceMap, ReferenceName, ReferenceNameWithSpan, SubmodelImport,
    SubmodelMap, SubmodelName, SubmodelNameWithSpan,
};
pub use parameter::{Limits, Parameter, ParameterCollection, ParameterValue, PiecewiseExpr};
pub use reference::{Identifier, IdentifierWithSpan, ModelPath, PythonPath};
pub use span::{IrSpan, WithSpan};
pub use test::{Test, TestIndex};
pub use unit::{CompositeUnit, Unit};
