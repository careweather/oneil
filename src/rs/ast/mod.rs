#![allow(missing_docs)]
// AST nodes should be self-documenting

//! Abstract Syntax Tree (AST) definitions for the Oneil language.
//!
//! This module contains the core data structures that represent Oneil programs
//! in memory after parsing.

mod declaration;
mod expression;
mod literal;
mod model;
mod note;
mod parameter;
mod unit;

pub use declaration::*;
pub use expression::*;
pub use literal::*;
pub use model::*;
pub use note::*;
pub use parameter::*;
