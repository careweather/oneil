//! Resolution functionality for Oneil model dependencies.
//!
//! This module provides functionality for resolving various dependencies within Oneil
//! models, including submodels, parameters, and tests. It handles the complex task
//! of connecting references to their actual definitions across multiple models.
//!
//! # Resolution Types
//!
//! - **Submodel resolution**: Resolves `use model` declarations to their actual model paths
//! - **Parameter resolution**: Resolves parameter references to their actual parameter definitions
//! - **Test resolution**: Resolves test references to their actual test definitions
//!
//! # Resolution Process
//!
//! The resolution process involves several phases:
//!
//! 1. **Submodel resolution**: First, all `use model` declarations are resolved to their
//!    corresponding model paths
//! 2. **Parameter resolution**: Parameters are resolved using the resolved submodel information
//! 3. **Test resolution**: Tests are resolved using both parameter and submodel information
//!
//! # Error Handling
//!
//! Each resolution phase can produce errors when references cannot be resolved. These
//! errors are collected and returned along with the successfully resolved items, allowing
//! for partial resolution and comprehensive error reporting.
//!
//! # Info Maps
//!
//! The model uses `InfoMap` types to pass information about available models, submodels,
//! and parameters to the resolution functions. These maps track both successful resolutions
//! and items that have errors, allowing the resolution functions to make informed decisions
//! about error handling.

mod expr;
mod parameter;
mod submodel;
mod test;
mod trace_level;
mod unit;
mod variable;

pub use parameter::resolve_parameters;
pub use submodel::resolve_submodels;
pub use test::resolve_tests;

// TODO: in all resolver tests, seperate out tests that test spans and tests that test values
