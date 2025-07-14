#![allow(missing_docs)]
// AST nodes should be self-documenting

//! Abstract Syntax Tree (AST) definitions for the Oneil language.
//!
//! This module contains the core data structures that represent Oneil programs
//! in memory after parsing.

pub mod atom;
pub mod debug_info;
pub mod declaration;
pub mod expression;
pub mod model;
pub mod node;
pub mod note;
pub mod parameter;
pub mod span;
pub mod test;
pub mod unit;

pub use declaration::Decl;
pub use expression::Expr;
pub use model::Model;
pub use note::Note;
pub use parameter::Parameter;
pub use span::Span;
pub use test::Test;
pub use unit::UnitExpr;
