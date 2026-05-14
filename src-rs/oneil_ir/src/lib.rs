#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! Intermediate Representation (IR) for the Oneil programming language.
//!
//! This crate provides the expression-level IR types used throughout the compiler
//! pipeline: expressions, parameters (with their dependencies and units), tests,
//! Python imports, and notes.
//!
//! The model-level types (`InstancedModel`, `InstanceGraph`, and design-related
//! structs) live in `oneil_frontend::instance`.

mod debug_info;
mod expr;
mod note;
mod parameter;
mod python_import;
mod section;
mod test;
mod unit;

pub use debug_info::TraceLevel;
pub use expr::{
    BinaryOp, ComparisonOp, Expr, ExprVisitor, FunctionName, Literal, UnaryOp, Variable,
};
pub use note::Note;
pub use parameter::{
    Dependencies, DesignApplication, DesignProvenance, Limits, Parameter, ParameterValue,
    PiecewiseExpr,
};
pub use python_import::PythonImport;
pub use section::{Section, SectionItem};
pub use test::Test;
pub use unit::{
    CompositeUnit, DisplayCompositeUnit, DisplayUnit, Unit, UnitInfo, compute_dimension_map,
};
