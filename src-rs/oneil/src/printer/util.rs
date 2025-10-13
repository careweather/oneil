//! Utility functions for output formatting and color handling

use colored::Colorize;

/// Color choice configuration for output formatting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorChoice {
    /// Enable colored output using ANSI color codes
    EnableColors,
    /// Disable colored output, use plain text only
    DisableColors,
}

impl ColorChoice {
    /// Applies bold formatting to text
    pub fn bold(self, text: &str) -> String {
        match self {
            Self::EnableColors => text.bold().to_string(),
            Self::DisableColors => text.to_string(),
        }
    }

    /// Applies bold red formatting to text
    pub fn bold_red(self, text: &str) -> String {
        match self {
            Self::EnableColors => text.bold().red().to_string(),
            Self::DisableColors => text.to_string(),
        }
    }

    /// Applies bold blue formatting to text
    pub fn bold_blue(self, text: &str) -> String {
        match self {
            Self::EnableColors => text.bold().blue().to_string(),
            Self::DisableColors => text.to_string(),
        }
    }
}
