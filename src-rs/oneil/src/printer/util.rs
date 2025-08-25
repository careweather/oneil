//! Utility functions for output formatting and color handling
//!
//! This module provides utility functionality for the printer system, primarily
//! focused on color handling and text formatting. It abstracts the colored output
//! functionality to support both colored and plain text modes.

use colored::Colorize;

/// Color choice configuration for output formatting
///
/// This enum controls whether colored output is enabled or disabled.
/// When colors are enabled, ANSI color codes are used to enhance readability.
/// When disabled, plain text is output for better compatibility with terminals
/// that don't support colors or for redirecting output to files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorChoice {
    /// Enable colored output using ANSI color codes
    EnableColors,
    /// Disable colored output, use plain text only
    DisableColors,
}

impl ColorChoice {
    /// Applies bold formatting to text
    ///
    /// When colors are enabled, applies ANSI bold formatting. When disabled,
    /// returns the text unchanged.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to format
    ///
    /// # Returns
    ///
    /// Returns the formatted text as a `String`. When colors are enabled,
    /// the text will be bold. When disabled, the text will be unchanged.
    pub fn bold(self, text: &str) -> String {
        match self {
            Self::EnableColors => text.bold().to_string(),
            Self::DisableColors => text.to_string(),
        }
    }

    /// Applies bold red formatting to text
    ///
    /// When colors are enabled, applies ANSI bold red formatting. When disabled,
    /// returns the text unchanged. This is typically used for error messages.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to format
    ///
    /// # Returns
    ///
    /// Returns the formatted text as a `String`. When colors are enabled,
    /// the text will be bold and red. When disabled, the text will be unchanged.
    pub fn bold_red(self, text: &str) -> String {
        match self {
            Self::EnableColors => text.bold().red().to_string(),
            Self::DisableColors => text.to_string(),
        }
    }

    /// Applies bold blue formatting to text
    ///
    /// When colors are enabled, applies ANSI bold blue formatting. When disabled,
    /// returns the text unchanged. This is typically used for informational
    /// messages and file paths.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to format
    ///
    /// # Returns
    ///
    /// Returns the formatted text as a `String`. When colors are enabled,
    /// the text will be bold and blue. When disabled, the text will be unchanged.
    pub fn bold_blue(self, text: &str) -> String {
        match self {
            Self::EnableColors => text.bold().blue().to_string(),
            Self::DisableColors => text.to_string(),
        }
    }
}
