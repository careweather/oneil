//! Resolution functionality for Oneil module dependencies.
//!
//! This module provides functionality for resolving various dependencies within Oneil
//! modules, including submodels, parameters, and tests. It handles the complex task
//! of connecting references to their actual definitions across multiple modules.
//!
//! # Resolution Types
//!
//! - **Submodel resolution**: Resolves `use model` declarations to their actual module paths
//! - **Parameter resolution**: Resolves parameter references to their actual parameter definitions
//! - **Test resolution**: Resolves test references to their actual test definitions
//!
//! # Resolution Process
//!
//! The resolution process involves several phases:
//!
//! 1. **Submodel resolution**: First, all `use model` declarations are resolved to their
//!    corresponding module paths
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
//! The module uses `InfoMap` types to pass information about available modules, submodels,
//! and parameters to the resolution functions. These maps track both successful resolutions
//! and items that have errors, allowing the resolution functions to make informed decisions
//! about error handling.

use oneil_ir::{
    module::Module,
    parameter::Parameter,
    reference::{Identifier, ModulePath},
};

use crate::util::info::InfoMap;

mod expr;
mod parameter;
mod submodel;
mod test;
mod trace_level;
mod unit;
mod variable;

pub use parameter::resolve_parameters;
pub use submodel::resolve_submodels_and_tests;
pub use test::resolve_model_tests;
pub use test::resolve_submodel_tests;

/// Type alias for parameter information maps used during resolution.
///
/// This type represents a map from parameter identifiers to their resolved parameter
/// definitions, along with information about which parameters have errors.
pub type ParameterInfo<'a> = InfoMap<&'a Identifier, &'a Parameter>;

/// Type alias for submodel information maps used during resolution.
///
/// This type represents a map from submodel identifiers to their resolved module paths,
/// along with information about which submodels have errors.
pub type SubmodelInfo<'a> = InfoMap<&'a Identifier, &'a ModulePath>;

/// Type alias for module information maps used during resolution.
///
/// This type represents a map from module paths to their loaded modules,
/// along with information about which modules have errors.
pub type ModuleInfo<'a> = InfoMap<&'a ModulePath, &'a Module>;
