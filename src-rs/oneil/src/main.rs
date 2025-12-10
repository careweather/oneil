#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! CLI for the Oneil programming language

use std::{
    io::{self, Write},
    path::PathBuf,
};

use clap::Parser;
use oneil_eval::builtin::std as oneil_std;
use oneil_model_resolver::FileLoader;

use crate::{
    builtins::Builtins,
    command::{CliCommand, Commands, DevCommand},
    file_parser::LoadingError,
};

mod builtins;
mod command;
mod convert_error;
mod file_parser;
mod printer;

/// Main entry point for the Oneil CLI application
fn main() -> io::Result<()> {
    let cli = CliCommand::parse();

    match cli.command {
        Commands::Dev { command } => handle_dev_command(command),
        Commands::Eval {
            file,
            print_debug,
            no_colors,
        } => handle_eval_command(file, print_debug, no_colors),
    }
}

fn handle_dev_command(command: DevCommand) -> io::Result<()> {
    match command {
        DevCommand::PrintAst {
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
                        let errors =
                            convert_error::parser::convert_all(&file, &error_with_partial.errors);
                        printer.print_errors(&errors)?;

                        if display_partial {
                            printer.print_ast(&error_with_partial.partial_result)?;
                        }
                    }
                }
            }

            Ok(())
        }
        DevCommand::PrintIr {
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

            let builtin_variables = Builtins::new(
                oneil_std::BUILTIN_VALUES,
                oneil_std::BUILTIN_FUNCTIONS,
                oneil_std::builtin_units(),
                oneil_std::BUILTIN_PREFIXES,
            );

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
    }
}

fn handle_eval_command(file: PathBuf, print_debug: bool, no_colors: bool) -> io::Result<()> {
    let use_colors = !no_colors;

    let mut stdout_writer = std::io::stdout();
    let mut stderr_writer = std::io::stderr();
    let mut printer = printer::Printer::new(
        use_colors,
        print_debug,
        &mut stdout_writer,
        &mut stderr_writer,
    );

    let builtins = Builtins::new(
        oneil_std::BUILTIN_VALUES,
        oneil_std::BUILTIN_FUNCTIONS,
        oneil_std::builtin_units(),
        oneil_std::BUILTIN_PREFIXES,
    );

    let model_collection =
        oneil_model_resolver::load_model(file, &builtins, &file_parser::FileLoader);
    let model_collection = match model_collection {
        Ok(model_collection) => model_collection,
        Err(error) => {
            let (_model_collection, error_map) = *error;
            let errors = convert_error::loader::convert_map(&error_map);
            printer.print_errors(&errors)?;
            return Ok(());
        }
    };

    let eval_result = oneil_eval::eval_model_collection(&model_collection, &builtins);

    println!("{eval_result:?}");

    Ok(())
}
