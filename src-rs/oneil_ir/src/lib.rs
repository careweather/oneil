#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! Intermediate Representation (IR) for the Oneil programming language

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
