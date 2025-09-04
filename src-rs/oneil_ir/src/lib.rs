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

pub mod debug_info;
pub mod expr;
pub mod model;
pub mod model_import;
pub mod parameter;
pub mod reference;
pub mod span;
pub mod test;
pub mod unit;
