//! Output formatting and printing functionality for the Oneil CLI

mod ast;
mod error;
mod ir;
mod util;

pub use util::ColorChoice;

use std::io::{self, Write};

use oneil_ast::Model as AstModel;
use oneil_ir::ModelCollection as IrModelCollection;
use oneil_shared::error::OneilError;

/// Main printer for formatting and displaying Oneil CLI output
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
    pub const fn new(
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
    pub fn print_ast(&mut self, ast: &AstModel) -> io::Result<()> {
        if self.print_debug {
            #[expect(clippy::use_debug, reason = "a debug output is expected")]
            writeln!(self.writer, "AST: {ast:?}")?;
        } else {
            ast::print(ast, self.writer)?;
        }

        Ok(())
    }

    /// Prints an Intermediate Representation (IR) in the configured format
    pub fn print_ir(&mut self, ir: &IrModelCollection) -> io::Result<()> {
        if self.print_debug {
            #[expect(clippy::use_debug, reason = "a debug output is expected")]
            writeln!(self.writer, "IR: {ir:?}")?;
        } else {
            ir::print(ir, self.writer)?;
        }

        Ok(())
    }

    /// Prints a single error with the configured formatting
    pub fn print_error(&mut self, error: &OneilError) -> io::Result<()> {
        if self.print_debug {
            #[expect(clippy::use_debug, reason = "a debug output is expected")]
            writeln!(self.error_writer, "Error: {error:?}")?;
        } else {
            error::print(error, self.color_choice, self.error_writer)?;
        }

        Ok(())
    }

    /// Prints multiple errors with the configured formatting
    pub fn print_errors(&mut self, errors: &[OneilError]) -> io::Result<()> {
        for error in errors {
            self.print_error(error)?;
            writeln!(self.error_writer)?;
        }

        Ok(())
    }

    /// Returns a mutable reference to the standard output writer
    pub const fn writer(&mut self) -> &mut W1 {
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
