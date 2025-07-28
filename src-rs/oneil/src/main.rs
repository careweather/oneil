use std::io;

use clap::Parser;
use oneil_model_loader::FileLoader;

use crate::{
    command::{CliCommand, Commands, DevCommands},
    file_parser::LoadingError,
};

mod command;
mod convert_error;
mod file_parser;
mod printer;

fn main() -> io::Result<()> {
    let cli = CliCommand::parse();

    match cli.command {
        Commands::Dev { command } => match command {
            DevCommands::PrintAst {
                file,
                display_partial,
                print_debug,
                no_colors,
            } => {
                let use_colors = !no_colors;
                // TODO: have a separate out/err writer
                let mut writer = std::io::stdout();
                let mut printer = printer::Printer::new(use_colors, print_debug, &mut writer);

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

                        for error in errors {
                            printer.print_error(&error)?;
                        }

                        if display_partial {
                            printer.print_ast(&error_with_partial.partial_result)?;
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
                // TODO: have a separate out/err writer
                let mut writer = std::io::stdout();
                let mut printer = printer::Printer::new(use_colors, print_debug, &mut writer);

                let model_collection =
                    oneil_model_loader::load_model(file, &file_parser::FileLoader);
                match model_collection {
                    Ok(model_collection) => printer.print_ir(&model_collection)?,
                    Err((model_collection, error_map)) => {
                        let errors = convert_error::loader::convert_all(&error_map);

                        for error in errors {
                            printer.print_error(&error)?;
                        }

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
