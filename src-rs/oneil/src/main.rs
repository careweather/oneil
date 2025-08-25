//! Oneil CLI - Command-line interface for the Oneil programming language
//!
//! This module provides the main entry point for the Oneil CLI tool, which offers
//! development utilities for parsing, analyzing, and debugging Oneil source files.

use std::io::{self, Write};

use clap::Parser;
use oneil_model_loader::FileLoader;

use crate::{
    builtins::Builtins,
    command::{CliCommand, Commands, DevCommands},
    file_parser::LoadingError,
};

mod builtins;
mod command;
mod convert_error;
mod file_parser;
mod printer;

/// Main entry point for the Oneil CLI application
///
/// Parses command-line arguments and executes the appropriate command based on
/// the user's input. Handles both AST and IR printing operations with error
/// reporting and partial result display capabilities.
///
/// # Returns
///
/// Returns `io::Result<()>` indicating success or failure of printing to the
/// console. All errors are properly formatted and displayed to the user.
fn main() -> io::Result<()> {
    let cli = CliCommand::parse();

    match cli.command {
        Commands::Dev { command } => match command {
            DevCommands::PrintAst {
                files,
                display_partial,
                print_debug,
                no_colors,
            } => {
                let use_colors = !no_colors;

                let mut stdout_writer = std::io::stdout();
                let mut stderr_writer = std::io::stderr();
                let mut printer = printer::Printer::new(
                    use_colors,
                    print_debug,
                    &mut stdout_writer,
                    &mut stderr_writer,
                );

                let is_multiple_files = files.len() > 1;
                for file in files {
                    if is_multiple_files {
                        writeln!(printer.writer(), "===== {} =====", file.display())?;
                    }

                    let ast = file_parser::FileLoader.parse_ast(&file);
                    match ast {
                        Ok(ast) => printer.print_ast(&ast)?,
                        Err(LoadingError::InvalidFile(error)) => {
                            let error = convert_error::file::convert(&file, &error);
                            printer.print_error(&error)?;
                        }
                        Err(LoadingError::Parser(error_with_partial)) => {
                            let errors = convert_error::parser::convert_all(
                                &file,
                                &error_with_partial.errors,
                            );
                            printer.print_errors(&errors)?;

                            if display_partial {
                                printer.print_ast(&error_with_partial.partial_result)?;
                            }
                        }
                    }
                }

                Ok(())
            }
            DevCommands::PrintIr {
                file,
                display_partial,
                print_debug,
                no_colors,
            } => {
                let use_colors = !no_colors;

                let mut stdout_writer = std::io::stdout();
                let mut stderr_writer = std::io::stderr();
                let mut printer = printer::Printer::new(
                    use_colors,
                    print_debug,
                    &mut stdout_writer,
                    &mut stderr_writer,
                );

                let builtin_variables = Builtins::new();

                let model_collection = oneil_model_loader::load_model(
                    file,
                    &builtin_variables,
                    &file_parser::FileLoader,
                );
                match model_collection {
                    Ok(model_collection) => printer.print_ir(&model_collection)?,
                    Err((model_collection, error_map)) => {
                        let errors = convert_error::loader::convert_map(&error_map);
                        printer.print_errors(&errors)?;

                        if display_partial {
                            printer.print_ir(&model_collection)?;
                        }
                    }
                }

                Ok(())
            }
        },
    }
}
