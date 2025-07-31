//! Error handling for the Oneil model loader.
//!
//! This module provides error types and utilities for handling various error conditions
//! that can occur during model loading, including:
//!
//! - Parse errors from AST parsing
//! - Resolution errors for submodels, parameters, and tests
//! - Circular dependency errors
//! - Python import validation errors
//!
//! # Error Types
//!
//! - `LoadError`: Represents errors that occur during model loading
//! - `CircularDependencyError`: Represents circular dependency detection
//! - Various resolution error types for different resolution phases
//!
//! # Error Collection
//!
//! The module provides utilities for collecting and managing errors across multiple
//! models and resolution phases, allowing for comprehensive error reporting.

use oneil_ir::reference::ModelPath;

pub mod collection;
pub mod resolution;
pub mod util;

pub use resolution::{
    ImportResolutionError, ModelTestResolutionError, ParameterResolutionError, ResolutionErrors,
    SubmodelResolutionError, SubmodelTestInputResolutionError, VariableResolutionError,
};
pub use util::{combine_error_list, combine_errors, convert_errors, split_ok_and_errors};

/// Represents errors that can occur during model loading.
///
/// This enum encapsulates all possible error types that can occur during the model
/// loading process. It distinguishes between parse errors (which occur during AST
/// parsing) and resolution errors (which occur during dependency resolution).
#[derive(Debug, Clone, PartialEq)]
pub enum LoadError<Ps> {
    /// Error that occurred during AST parsing of a model file.
    ParseError(Ps),
    /// Errors that occurred during dependency resolution.
    ResolutionErrors(resolution::ResolutionErrors),
}

impl<Ps> LoadError<Ps> {
    /// Creates a new parse error.
    ///
    /// # Arguments
    ///
    /// * `parse_error` - The parse error that occurred
    ///
    /// # Returns
    ///
    /// A `LoadError::ParseError` variant containing the parse error.
    pub fn parse_error(parse_error: Ps) -> Self {
        Self::ParseError(parse_error)
    }

    /// Creates a new resolution error.
    ///
    /// # Arguments
    ///
    /// * `resolution_errors` - The resolution errors that occurred
    ///
    /// # Returns
    ///
    /// A `LoadError::ResolutionErrors` variant containing the resolution errors.
    pub fn resolution_errors(resolution_errors: resolution::ResolutionErrors) -> Self {
        Self::ResolutionErrors(resolution_errors)
    }
}

/// Represents a circular dependency detected during model loading.
///
/// A circular dependency occurs when model A depends on model B, which depends on
/// model C, which depends back on model A (or any other cycle). This error contains
/// the complete cycle of model paths that form the circular dependency.
#[derive(Debug, Clone, PartialEq)]
pub struct CircularDependencyError(Vec<ModelPath>);

impl CircularDependencyError {
    /// Creates a new circular dependency error.
    ///
    /// # Arguments
    ///
    /// * `circular_dependency` - A vector of model paths that form the circular dependency.
    ///   The vector should contain the complete cycle, with the first and last elements
    ///   being the same model if the cycle is complete.
    ///
    /// # Returns
    ///
    /// A new `CircularDependencyError` containing the circular dependency path.
    pub fn new(circular_dependency: Vec<ModelPath>) -> Self {
        Self(circular_dependency)
    }

    /// Returns the circular dependency path.
    ///
    /// # Returns
    ///
    /// A vector of model paths that form the circular dependency.
    pub fn circular_dependency(&self) -> &[ModelPath] {
        &self.0
    }
}
