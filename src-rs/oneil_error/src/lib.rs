#![warn(missing_docs)]

//! # Oneil Error
//!
//! A unified error handling system for the Oneil programming language.
//!
//! This crate provides a standardized way to represent, format, and display errors
//! throughout the Oneil compiler and toolchain. It includes:
//!
//! - **Unified Error Structure**: The `OneilError` type that combines error messages,
//!   source locations, and contextual information
//! - **Source Location Tracking**: Precise line and column information for error reporting
//! - **Rich Context System**: Support for notes, help text, and other contextual information
//! - **Trait-based Interface**: The `AsOneilError` trait for converting any error type
//!   into the unified format
//!
//! ## Usage
//!
//! ```rust
//! use oneil_error::{OneilError, AsOneilError, Context};
//! use std::path::PathBuf;
//!
//! // Define an error type that implements AsOneilError
//! struct MyError {
//!     message: String,
//!     offset: usize,
//! }
//!
//! impl AsOneilError for MyError {
//!     fn message(&self) -> String {
//!         self.message.clone()
//!     }
//!     
//!     fn error_location(&self, source: &str) -> Option<oneil_error::ErrorLocation> {
//!         if self.offset < source.len() {
//!             Some(oneil_error::ErrorLocation::from_source_and_offset(source, self.offset))
//!         } else {
//!             None
//!         }
//!     }
//!     
//!     fn context(&self) -> Vec<Context> {
//!         vec![Context::Help("Try checking your syntax".to_string())]
//!     }
//! }
//!
//! // Convert to OneilError
//! let my_error = MyError {
//!     message: "Unexpected token".to_string(),
//!     offset: 5,
//! };
//!
//! let source = "let x = 42;";
//! let path = PathBuf::from("example.on");
//! let oneil_error = OneilError::from_error_with_source(&my_error, path, source);
//! ```

mod context;
mod location;
mod traits;

use std::path::PathBuf;

pub use context::Context;
pub use location::ErrorLocation;
pub use traits::AsOneilError;

/// Unified error representation for Oneil
///
/// This struct represents errors in a format suitable for display to users.
/// It includes the file path where the error occurred, a human-readable message,
/// and optional source location information for precise error reporting.
#[derive(Debug, Clone, PartialEq)]
pub struct OneilError {
    /// The path to the file where the error occurred
    path: PathBuf,
    /// Human-readable error message
    message: String,
    /// Optional source location information for precise error reporting
    location: Option<ErrorLocation>,
    /// Optional context information
    context: Vec<Context>,
    /// Optional context information with source location
    context_with_source: Vec<(Context, ErrorLocation)>,
}

impl OneilError {
    /// Creates a new `OneilError` from an error that implements `AsOneilError`
    ///
    /// This constructor creates an error without source location information.
    /// Use `from_error_with_source` if you need precise line and column information.
    ///
    /// # Arguments
    ///
    /// * `error` - The error that implements `AsOneilError`
    /// * `path` - The path to the file where the error occurred
    ///
    /// # Returns
    ///
    /// Returns a new `OneilError` with the error message and context, but no source location.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oneil_error::{OneilError, AsOneilError, Context};
    /// use std::path::PathBuf;
    ///
    /// struct SimpleError(String);
    ///
    /// impl AsOneilError for SimpleError {
    ///     fn message(&self) -> String {
    ///         self.0.clone()
    ///     }
    /// }
    ///
    /// let error = SimpleError("Something went wrong".to_string());
    /// let path = PathBuf::from("example.on");
    /// let oneil_error = OneilError::from_error(&error, path);
    /// ```
    pub fn from_error(error: &impl AsOneilError, path: PathBuf) -> Self {
        let message = error.message();
        let location = None;
        let context = error.context();
        let context_with_source = vec![];

        Self {
            path,
            message,
            location,
            context,
            context_with_source,
        }
    }

    /// Creates a new `OneilError` from an error with source code for location tracking
    ///
    /// This constructor creates an error with full source location information,
    /// including line and column numbers. The source code is used to calculate
    /// precise error positions for better error reporting.
    ///
    /// # Arguments
    ///
    /// * `error` - The error that implements `AsOneilError`
    /// * `path` - The path to the file where the error occurred
    /// * `source` - The complete source code content for location calculation
    ///
    /// # Returns
    ///
    /// Returns a new `OneilError` with error message, context, and source location information.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oneil_error::{OneilError, AsOneilError, ErrorLocation};
    /// use std::path::PathBuf;
    ///
    /// struct PositionalError {
    ///     message: String,
    ///     offset: usize,
    /// }
    ///
    /// impl AsOneilError for PositionalError {
    ///     fn message(&self) -> String {
    ///         self.message.clone()
    ///     }
    ///
    ///     fn error_location(&self, source: &str) -> Option<ErrorLocation> {
    ///         Some(ErrorLocation::from_source_and_offset(source, self.offset))
    ///     }
    /// }
    ///
    /// let error = PositionalError {
    ///     message: "Unexpected token".to_string(),
    ///     offset: 5,
    /// };
    /// let path = PathBuf::from("example.on");
    /// let source = "let x = 42;";
    /// let oneil_error = OneilError::from_error_with_source(&error, path, source);
    /// ```
    pub fn from_error_with_source(error: &impl AsOneilError, path: PathBuf, source: &str) -> Self {
        let message = error.message();
        let location = error.error_location(source);

        let mut context = error.context();
        let mut context_with_source = vec![];

        for (context_item, location) in error.context_with_source(source) {
            match location {
                Some(location) => {
                    context_with_source.push((context_item, location));
                }
                None => {
                    context.push(context_item);
                }
            }
        }

        Self {
            path,
            message,
            location,
            context,
            context_with_source,
        }
    }

    /// Creates a new `OneilError` with optional source code for location tracking
    ///
    /// This constructor is a convenience method that chooses between `from_error`
    /// and `from_error_with_source` based on whether source code is available.
    /// If source code is provided, it will include location information; otherwise,
    /// it will create an error without location details.
    ///
    /// # Arguments
    ///
    /// * `error` - The error that implements `AsOneilError`
    /// * `path` - The path to the file where the error occurred
    /// * `source` - Optional source code content for location calculation
    ///
    /// # Returns
    ///
    /// Returns a new `OneilError`. If source code is provided, it will include
    /// location information; otherwise, it will not.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oneil_error::{OneilError, AsOneilError};
    /// use std::path::PathBuf;
    ///
    /// struct MyError(String);
    ///
    /// impl AsOneilError for MyError {
    ///     fn message(&self) -> String {
    ///         self.0.clone()
    ///     }
    /// }
    ///
    /// let error = MyError("Something went wrong".to_string());
    /// let path = PathBuf::from("example.on");
    ///
    /// // With source code
    /// let oneil_error = OneilError::from_error_with_optional_source(
    ///     &error,
    ///     path.clone(),
    ///     Some("let x = 42;")
    /// );
    ///
    /// // Without source code
    /// let oneil_error = OneilError::from_error_with_optional_source(
    ///     &error,
    ///     path,
    ///     None
    /// );
    /// ```
    pub fn from_error_with_optional_source(
        error: &impl AsOneilError,
        path: PathBuf,
        source: Option<&str>,
    ) -> Self {
        match source {
            Some(source) => Self::from_error_with_source(error, path, source),
            None => Self::from_error(error, path),
        }
    }

    /// Returns the path to the file where the error occurred
    ///
    /// # Returns
    ///
    /// Returns a reference to the `PathBuf` containing the file path.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Returns the human-readable error message
    ///
    /// # Returns
    ///
    /// Returns a reference to the error message string.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the optional source location information
    ///
    /// # Returns
    ///
    /// Returns an optional reference to the `ErrorLocation` if available.
    pub fn location(&self) -> Option<&ErrorLocation> {
        self.location.as_ref()
    }

    /// Returns the optional context information
    ///
    /// # Returns
    ///
    /// Returns a reference to the context information.
    pub fn context(&self) -> &[Context] {
        &self.context
    }

    /// Returns the optional context information with source location
    ///
    /// # Returns
    ///
    /// Returns a reference to the context information with source location.
    pub fn context_with_source(&self) -> &[(Context, ErrorLocation)] {
        &self.context_with_source
    }
}
