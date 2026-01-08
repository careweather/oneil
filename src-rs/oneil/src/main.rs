#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! CLI for the Oneil programming language

use std::io::{self, Write};

use clap::Parser;
use oneil_model_resolver::FileLoader;

use crate::command::{CliCommand, Commands, DevCommands};
use oneil_runner::{builtins::Builtins, file_parser::{self, LoadingError}};

mod command;
mod convert_error;
mod printer;

/// Main entry point for the Oneil CLI application
fn main() -> io::Result<()> {
    let cli = CliCommand::parse();

    match cli.command {
        Commands::Lsp {} => {
            oneil_lsp::run();
            Ok(())
        }
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

                let model_collection = oneil_model_resolver::load_model(
                    file,
                    &builtin_variables,
                    &file_parser::FileLoader,
                );
                match model_collection {
                    Ok(model_collection) => printer.print_ir(&model_collection)?,
                    Err(error) => {
                        let (model_collection, error_map) = *error;
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
