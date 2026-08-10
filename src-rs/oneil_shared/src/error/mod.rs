//! Diagnostics for the Oneil programming language

mod context;
mod location;
mod traits;

use std::path::PathBuf;

use serde::Serialize;

pub use context::Context;
pub use location::ErrorLocation;
pub use traits::AsOneilDiagnostic;

/// Classification of a [`OneilDiagnostic`].
///
/// Derives `Serialize` (rather than requiring JSON-emitting consumers to
/// define their own tagged copy) so that all of Oneil's JSON-facing output
/// formats — the LSP's rendered-view diagnostics and `oneil test --format
/// json`'s CI report alike — share one canonical wire representation
/// (`"error"` / `"warning"`) instead of hand-maintained duplicates that could
/// drift out of sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "ts-bindings", derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticKind {
    /// A fatal or blocking issue.
    Error,
    /// A non-fatal issue the user should be aware of.
    Warning,
}

/// Unified diagnostic representation for Oneil
///
/// This struct represents diagnostics in a format suitable for display to users.
/// It includes the file path where the diagnostic occurred, a human-readable message,
/// and optional source location information for precise reporting.
// TODO: refactor this to use Span/SourceLocation instead of ErrorLocation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneilDiagnostic {
    /// How this diagnostic should be interpreted (e.g. error vs. future severities).
    kind: DiagnosticKind,
    /// The path to the file where the diagnostic occurred
    path: PathBuf,
    /// Human-readable message
    message: String,
    /// Optional source location information for precise error reporting
    location: Option<ErrorLocation>,
    /// Optional context information
    context: Vec<Context>,
    /// Optional context information with source location
    context_with_source: Vec<(Context, ErrorLocation)>,
    /// Whether this diagnostic represents an internal diagnostic.
    is_internal_diagnostic: bool,
}

impl OneilDiagnostic {
    /// Creates a new `OneilDiagnostic` from a value that implements [`AsOneilDiagnostic`].
    /// The kind is taken from [`AsOneilDiagnostic::kind`].
    ///
    /// This constructor creates a diagnostic without source location information.
    /// Use `from_error_with_source` if you need precise line and column information.
    ///
    /// # Arguments
    ///
    /// * `error` - The value that implements [`AsOneilDiagnostic`]
    /// * `path` - The path to the file where the error occurred
    ///
    /// # Returns
    ///
    /// Returns a new `OneilDiagnostic` with the error message and context, but no source location.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oneil_shared::error::{DiagnosticKind, OneilDiagnostic, AsOneilDiagnostic, Context};
    /// use std::path::PathBuf;
    ///
    /// struct SimpleError(String);
    ///
    /// impl AsOneilDiagnostic for SimpleError {
    ///     fn kind(&self) -> DiagnosticKind {
    ///         DiagnosticKind::Error
    ///     }
    ///
    ///     fn message(&self) -> String {
    ///         self.0.clone()
    ///     }
    /// }
    ///
    /// let error = SimpleError("Something went wrong".to_string());
    /// let path = PathBuf::from("example.on");
    /// let diagnostic = OneilDiagnostic::from_error(&error, path);
    /// ```
    pub fn from_error(error: &impl AsOneilDiagnostic, path: PathBuf) -> Self {
        let kind = error.kind();
        let message = error.message();
        let location = None;
        let context = error.context();
        let context_with_source = vec![];
        let is_internal_diagnostic = error.is_internal_diagnostic();

        Self {
            kind,
            path,
            message,
            location,
            context,
            context_with_source,
            is_internal_diagnostic,
        }
    }

    /// Creates a new `OneilDiagnostic` with source code for location tracking.
    /// The kind is taken from [`AsOneilDiagnostic::kind`].
    ///
    /// This constructor creates a diagnostic with full source location information,
    /// including line and column numbers. The source code is used to calculate
    /// precise positions for better reporting.
    ///
    /// # Arguments
    ///
    /// * `error` - The value that implements [`AsOneilDiagnostic`]
    /// * `path` - The path to the file where the error occurred
    /// * `source` - The complete source code content for location calculation
    ///
    /// # Returns
    ///
    /// Returns a new `OneilDiagnostic` with message, context, and source location information.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oneil_shared::error::{DiagnosticKind, OneilDiagnostic, AsOneilDiagnostic, ErrorLocation};
    /// use std::path::PathBuf;
    ///
    /// struct PositionalError {
    ///     message: String,
    ///     offset: usize,
    /// }
    ///
    /// impl AsOneilDiagnostic for PositionalError {
    ///     fn kind(&self) -> DiagnosticKind {
    ///         DiagnosticKind::Error
    ///     }
    ///
    ///     fn message(&self) -> String {
    ///         self.message.clone()
    ///     }
    ///
    ///     fn diagnostic_location(&self, source: &str) -> Option<ErrorLocation> {
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
    /// let diagnostic = OneilDiagnostic::from_error_with_source(&error, path, source);
    /// ```
    pub fn from_error_with_source(
        error: &impl AsOneilDiagnostic,
        path: PathBuf,
        source: &str,
    ) -> Self {
        let kind = error.kind();
        let message = error.message();
        let location = error.diagnostic_location(source);

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

        let is_internal_diagnostic = error.is_internal_diagnostic();

        Self {
            kind,
            path,
            message,
            location,
            context,
            context_with_source,
            is_internal_diagnostic,
        }
    }

    /// Creates a new `OneilDiagnostic` with optional source code for location tracking.
    /// The kind is taken from [`AsOneilDiagnostic::kind`].
    ///
    /// This constructor is a convenience method that chooses between `from_error`
    /// and `from_error_with_source` based on whether source code is available.
    /// If source code is provided, it will include location information; otherwise,
    /// it will create a diagnostic without location details.
    ///
    /// # Arguments
    ///
    /// * `error` - The value that implements [`AsOneilDiagnostic`]
    /// * `path` - The path to the file where the error occurred
    /// * `source` - Optional source code content for location calculation
    ///
    /// # Returns
    ///
    /// Returns a new `OneilDiagnostic`. If source code is provided, it will include
    /// location information; otherwise, it will not.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oneil_shared::error::{DiagnosticKind, OneilDiagnostic, AsOneilDiagnostic};
    /// use std::path::PathBuf;
    ///
    /// struct MyError(String);
    ///
    /// impl AsOneilDiagnostic for MyError {
    ///     fn kind(&self) -> DiagnosticKind {
    ///         DiagnosticKind::Error
    ///     }
    ///
    ///     fn message(&self) -> String {
    ///         self.0.clone()
    ///     }
    /// }
    ///
    /// let error = MyError("Something went wrong".to_string());
    /// let path = PathBuf::from("example.on");
    ///
    /// // With source code
    /// let diagnostic = OneilDiagnostic::from_error_with_optional_source(
    ///     &error,
    ///     path.clone(),
    ///     Some("let x = 42;")
    /// );
    ///
    /// // Without source code
    /// let diagnostic = OneilDiagnostic::from_error_with_optional_source(
    ///     &error,
    ///     path,
    ///     None
    /// );
    /// ```
    pub fn from_error_with_optional_source(
        error: &impl AsOneilDiagnostic,
        path: PathBuf,
        source: Option<&str>,
    ) -> Self {
        match source {
            Some(source) => Self::from_error_with_source(error, path, source),
            None => Self::from_error(error, path),
        }
    }

    /// Returns how this diagnostic is classified.
    #[must_use]
    pub const fn kind(&self) -> DiagnosticKind {
        self.kind
    }

    /// Returns the path to the file where the diagnostic occurred
    ///
    /// # Returns
    ///
    /// Returns a reference to the `PathBuf` containing the file path.
    #[must_use]
    pub const fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Returns the human-readable message
    ///
    /// # Returns
    ///
    /// Returns a reference to the message string.
    #[must_use]
    pub const fn message(&self) -> &str {
        self.message.as_str()
    }

    /// Returns the optional source location information
    ///
    /// # Returns
    ///
    /// Returns an optional reference to the `ErrorLocation` if available.
    #[must_use]
    pub const fn location(&self) -> Option<&ErrorLocation> {
        self.location.as_ref()
    }

    /// Returns the optional context information
    ///
    /// # Returns
    ///
    /// Returns a reference to the context information.
    #[must_use]
    pub const fn context(&self) -> &[Context] {
        self.context.as_slice()
    }

    /// Returns the optional context information with source location
    ///
    /// # Returns
    ///
    /// Returns a reference to the context information with source location.
    #[must_use]
    pub const fn context_with_source(&self) -> &[(Context, ErrorLocation)] {
        self.context_with_source.as_slice()
    }

    /// Creates an `OneilDiagnostic` from explicit parts.
    ///
    /// Prefer `from_error` / `from_error_with_source` when possible; use this
    /// constructor only when the primary path and location need to differ from
    /// the host model (e.g. a design-file cycle attribution).
    #[must_use]
    pub const fn from_parts(
        kind: DiagnosticKind,
        message: String,
        path: PathBuf,
        location: Option<ErrorLocation>,
        context: Vec<Context>,
        context_with_source: Vec<(Context, ErrorLocation)>,
    ) -> Self {
        Self {
            kind,
            path,
            message,
            location,
            context,
            context_with_source,
            is_internal_diagnostic: false,
        }
    }

    /// Returns whether this diagnostic represents an internal diagnostic.
    ///
    /// Internal diagnostics are not important for the user to see,
    /// such as diagnostics that are caused by other errors. In that case, it's most
    /// useful to see only the first error.
    #[must_use]
    pub const fn is_internal_diagnostic(&self) -> bool {
        self.is_internal_diagnostic
    }
}
