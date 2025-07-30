//! Output formatting and printing functionality for the Oneil CLI
//!
//! This module provides the core printing capabilities for displaying AST, IR, and error
//! information in a user-friendly format. It supports both colored and plain text output,
//! debug mode for detailed internal representations, and hierarchical tree formatting.
//!
//! The module is organized into submodules:
//! - `ast`: AST-specific printing functionality
//! - `ir`: IR-specific printing functionality  
//! - `error`: Error message formatting and display
//! - `util`: Utility functions for color handling and formatting

mod ast;
mod error;
mod ir;
mod util;

pub use util::ColorChoice;

use std::io::{self, Write};

use oneil_ast::Model as AstModel;
use oneil_ir::model::ModelCollection as IrModelCollection;

use crate::convert_error::Error;

/// Main printer for formatting and displaying Oneil CLI output
///
/// The `Printer` struct provides a unified interface for printing various types of
/// Oneil data structures (AST, IR, errors) with configurable formatting options.
/// It supports colored output, debug mode, and writes to any type implementing `Write`.
///
/// # Examples
///
/// ```rust
/// use std::io::stdout;
/// use oneil::printer::Printer;
///
/// let mut writer = stdout();
/// let mut printer = Printer::new(true, false, &mut writer);
/// // Use printer to output formatted data
/// ```
pub struct Printer<'a, W1, W2>
where
    W1: Write,
    W2: Write,
{
    /// Color choice configuration for output formatting
    color_choice: ColorChoice,
    /// Whether to print in debug format (raw representation)
    print_debug: bool,
    /// The writer to output formatted data to
    writer: &'a mut W1,
    /// The writer to output error messages to
    error_writer: &'a mut W2,
}

impl<'a, W1, W2> Printer<'a, W1, W2>
where
    W1: Write,
    W2: Write,
{
    /// Creates a new printer with the specified configuration
    ///
    /// # Arguments
    ///
    /// * `use_colors` - Whether to enable colored output. When `true`, uses ANSI color codes
    ///                  for enhanced readability. When `false`, outputs plain text.
    /// * `print_debug` - Whether to print in debug format. When `true`, displays raw
    ///                   debug representations. When `false`, uses formatted tree structures.
    /// * `writer` - The writer to output formatted data to. Must implement `Write`.
    ///
    /// # Returns
    ///
    /// A new `Printer` instance configured with the specified options.
    pub fn new(
        use_colors: bool,
        print_debug: bool,
        writer: &'a mut W1,
        error_writer: &'a mut W2,
    ) -> Self {
        let color_choice = if use_colors {
            ColorChoice::EnableColors
        } else {
            ColorChoice::DisableColors
        };

        Self {
            color_choice,
            print_debug,
            writer,
            error_writer,
        }
    }

    /// Prints an Abstract Syntax Tree (AST) in the configured format
    ///
    /// Displays the AST either as a formatted hierarchical tree structure or as a
    /// raw debug representation, depending on the `print_debug` configuration.
    ///
    /// # Arguments
    ///
    /// * `ast` - The AST model to print
    ///
    /// # Returns
    ///
    /// Returns `io::Result<()>` indicating success or failure of the write operation.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the underlying writer fails.
    pub fn print_ast(&mut self, ast: &AstModel) -> io::Result<()> {
        if self.print_debug {
            writeln!(self.writer, "AST: {:?}", ast)?;
        } else {
            ast::print(ast, self.writer)?;
        }

        Ok(())
    }

    /// Prints an Intermediate Representation (IR) in the configured format
    ///
    /// Displays the IR either as a formatted hierarchical tree structure or as a
    /// raw debug representation, depending on the `print_debug` configuration.
    ///
    /// # Arguments
    ///
    /// * `ir` - The IR model collection to print
    ///
    /// # Returns
    ///
    /// Returns `io::Result<()>` indicating success or failure of the write operation.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the underlying writer fails.
    pub fn print_ir(&mut self, ir: &IrModelCollection) -> io::Result<()> {
        if self.print_debug {
            writeln!(self.writer, "IR: {:?}", ir)?;
        } else {
            ir::print(ir, self.writer)?;
        }

        Ok(())
    }

    /// Prints a single error with the configured formatting
    ///
    /// Displays an error message with source location information and optional
    /// color highlighting, depending on the color configuration.
    ///
    /// # Arguments
    ///
    /// * `error` - The error to print
    ///
    /// # Returns
    ///
    /// Returns `io::Result<()>` indicating success or failure of the write operation.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the underlying writer fails.
    pub fn print_error(&mut self, error: &Error) -> io::Result<()> {
        if self.print_debug {
            writeln!(self.error_writer, "Error: {:?}", error)?;
        } else {
            error::print(error, &self.color_choice, self.error_writer)?;
        }

        Ok(())
    }

    /// Prints multiple errors with the configured formatting
    ///
    /// Displays a sequence of error messages, each with source location information
    /// and optional color highlighting. Errors are separated by blank lines for
    /// better readability.
    ///
    /// # Arguments
    ///
    /// * `errors` - A slice of errors to print
    ///
    /// # Returns
    ///
    /// Returns `io::Result<()>` indicating success or failure of the write operation.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the underlying writer fails.
    pub fn print_errors(&mut self, errors: &[Error]) -> io::Result<()> {
        for error in errors {
            self.print_error(error)?;
            writeln!(self.error_writer)?;
        }

        Ok(())
    }

    /// Returns a mutable reference to the standard output writer
    ///
    /// Provides access to the underlying writer used for standard output,
    /// allowing direct writing operations when needed.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the writer of type `W1`
    pub fn writer(&mut self) -> &mut W1 {
        self.writer
    }

    // /// Returns a mutable reference to the error writer
    // ///
    // /// Provides access to the underlying writer used for error messages,
    // /// allowing direct writing operations when needed.
    // ///
    // /// # Returns
    // ///
    // /// Returns a mutable reference to the writer of type `W2`
    // pub fn error_writer(&mut self) -> &mut W2 {
    //     self.error_writer
    // }
}
