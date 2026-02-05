//! Error handling for the Oneil model loader.
//!
//! This module defines error types that can occur during the resolution phase of
//! model loading. Resolution errors occur when references cannot be resolved to
//! their actual definitions, such as when a submodel reference points to a
//! non-existent model or when a parameter reference cannot be found.
//!
//! # Error Categories
//!
//! - **Circular dependency errors**: Errors that occur when models depend on each other in a cycle
//! - **Import errors**: Errors that occur during Python import validation
//! - **Submodel resolution errors**: Errors that occur when resolving `use model` declarations
//! - **Parameter resolution errors**: Errors that occur when resolving parameter references
//! - **Test resolution errors**: Errors that occur when resolving test references
//! - **Variable resolution errors**: Errors that occur when resolving variable references within expressions

mod circular_dependency;
mod errors;
mod import;
mod parameter;
mod submodel;
mod util;
mod variable;

pub use circular_dependency::CircularDependencyError;
pub use errors::ResolutionErrorCollection;
pub use import::PythonImportResolutionError;
pub use parameter::ParameterResolutionError;
pub use submodel::ModelImportResolutionError;
pub use util::{combine_error_list, combine_errors, convert_errors, split_ok_and_errors};
pub use variable::VariableResolutionError;
