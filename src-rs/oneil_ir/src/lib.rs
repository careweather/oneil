//! # Oneil Model
//!
//! This crate provides the core data structures and abstractions for representing
//! models in the Oneil programming language. It defines the fundamental building
//! blocks for modeling systems, including parameters, expressions, tests, and
//! model organization.
//!
//! "IR" stands for "Intermediate Representation".
//!
//! ## Overview
//!
//! The `oneil_ir` crate is responsible for:
//!
//! - **Model Structure**: Defining how Oneil models are organized and composed
//! - **Parameter Management**: Handling parameter definitions, dependencies, and values
//! - **Expression System**: Providing a rich expression language for calculations
//! - **Testing Framework**: Supporting model and submodel testing capabilities
//! - **Unit System**: Managing physical units and dimensional analysis
//! - **Reference System**: Handling identifiers, model paths, and Python imports
//!
//! ## Key Components
//!
//! ### Models
//! - [`model::Model`] - Represents a single Oneil model with parameters, tests, and submodels
//! - [`model::ModelCollection`] - Manages collections of models and their relationships
//!
//! ### Parameters
//! - [`parameter::Parameter`] - Individual parameter definitions with dependencies and constraints
//! - [`parameter::ParameterCollection`] - Container for managing multiple parameters
//! - [`parameter::ParameterValue`] - Values can be simple expressions or piecewise functions
//!
//! ### Expressions
//! - [`expr::Expr`] - Abstract syntax tree for mathematical and logical expressions
//! - [`expr::BinaryOp`] and [`expr::UnaryOp`] - Mathematical and logical operators
//! - [`expr::FunctionName`] - Built-in and imported function references
//! - [`expr::Variable`] - Local, parameter, and external variable references
//!
//! ### Testing
//! - [`test::ModelTest`] - Tests for entire models
//! - [`test::SubmodelTest`] - Tests for individual submodels
//!
//! ### Units
//! - [`unit::CompositeUnit`] - Complex units composed of multiple base units
//! - [`unit::Unit`] - Individual units with names and exponents
//!
//! ## Usage Example
//!
//! ```rust
//! use oneil_ir::{
//!     model::Model, parameter::{Parameter, ParameterCollection, ParameterValue, Limits},
//!     expr::{Expr, Literal}, reference::Identifier, unit::CompositeUnit,
//!     debug_info::TraceLevel
//! };
//! use std::collections::{HashMap, HashSet};
//!
//! // Create a simple parameter
//! use oneil_ir::span::WithSpan;
//! let param_expr = WithSpan::test_new(Expr::literal(Literal::number(42.0)));
//! let param_value = ParameterValue::simple(param_expr, None);
//! let param = Parameter::new(
//!     HashSet::new(),
//!     WithSpan::test_new(Identifier::new("my_param")),
//!     param_value,
//!     Limits::default(),
//!     false,
//!     TraceLevel::None,
//! );
//!
//! // Create a model
//! let mut params = HashMap::new();
//! params.insert(Identifier::new("my_param"), param);
//! let model = Model::new(
//!     HashSet::new(), // no Python imports
//!     HashMap::new(),  // no submodels
//!     ParameterCollection::new(params),
//!     HashMap::new(),  // no model tests
//!     Vec::new(),      // no submodel tests
//! );
//! ```
//!
//! ## Architecture
//!
//! This crate follows a functional programming approach with immutable data structures.
//! All major types implement `Clone`, `Debug`, and `PartialEq` for easy manipulation
//! and testing. The design emphasizes:
//!
//! - **Immutability**: Data structures are immutable by default
//! - **Composability**: Complex structures are built from simple, composable parts
//! - **Type Safety**: Strong typing prevents invalid operations
//! - **Extensibility**: The expression system can be easily extended with new operators and functions

#![warn(missing_docs)]

pub mod debug_info;
pub mod expr;
pub mod model;
pub mod parameter;
pub mod reference;
pub mod span;
pub mod test;
pub mod unit;
