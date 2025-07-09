//! # Oneil Module
//!
//! This crate provides the core data structures and abstractions for representing
//! modules in the Oneil programming language. It defines the fundamental building
//! blocks for modeling systems, including parameters, expressions, tests, and
//! module organization.
//!
//! ## Overview
//!
//! The `oneil_module` crate is responsible for:
//!
//! - **Module Structure**: Defining how Oneil modules are organized and composed
//! - **Parameter Management**: Handling parameter definitions, dependencies, and values
//! - **Expression System**: Providing a rich expression language for calculations
//! - **Testing Framework**: Supporting model and submodel testing capabilities
//! - **Unit System**: Managing physical units and dimensional analysis
//! - **Reference System**: Handling identifiers, module paths, and Python imports
//!
//! ## Key Components
//!
//! ### Modules
//! - [`module::Module`] - Represents a single Oneil module with parameters, tests, and submodels
//! - [`module::ModuleCollection`] - Manages collections of modules and their relationships
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
//! use oneil_module::{
//!     module::Module, parameter::{Parameter, ParameterCollection, ParameterValue, Limits},
//!     expr::{Expr, Literal}, reference::Identifier, unit::CompositeUnit,
//!     debug_info::TraceLevel
//! };
//! use std::collections::{HashMap, HashSet};
//!
//! // Create a simple parameter
//! let param_expr = Expr::literal(Literal::number(42.0));
//! let param_value = ParameterValue::simple(param_expr, None);
//! let param = Parameter::new(
//!     HashSet::new(),
//!     Identifier::new("my_param"),
//!     param_value,
//!     Limits::default(),
//!     false,
//!     TraceLevel::None,
//! );
//!
//! // Create a module
//! let mut params = HashMap::new();
//! params.insert(Identifier::new("my_param"), param);
//! let module = Module::new(
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
pub mod module;
pub mod parameter;
pub mod reference;
pub mod test;
pub mod unit;
