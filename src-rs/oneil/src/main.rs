use clap::Parser;
use oneil_model_loader::FileLoader;

use crate::{
    command::{CliCommand, Commands, DevCommands},
    file_parser::LoadingError,
};

mod command;
mod file_parser;
mod printer;

fn main() {
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
                    Ok(ast) => printer.print_ast(&ast),
                    Err(LoadingError::InvalidFile(error)) => {
                        printer.print_file_error(&file, &error)
                    }
                    Err(LoadingError::Parser(error_with_partial)) => {
                        printer.print_parser_error(&file, &error_with_partial.errors);
                        if display_partial {
                            printer.print_ast(&error_with_partial.partial_result);
                        }
                    }
                }
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
                    Ok(model_collection) => printer.print_ir(&model_collection),
                    Err((model_collection, error_map)) => {
                        printer.print_loader_error(&error_map);
                        if display_partial {
                            printer.print_ir(&model_collection);
                        }
                    }
                }
            }
        },
    }
}
