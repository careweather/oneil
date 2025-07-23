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
            } => {
                let ast = file_parser::FileLoader.parse_ast(&file);
                match ast {
                    Ok(ast) => printer::ast::print(&ast, false),
                    Err(LoadingError::InvalidFile(error)) => {
                        printer::error::file::print(&file, &error)
                    }
                    Err(LoadingError::Parser(error_with_partial)) => {
                        printer::error::parser::print(&file, error_with_partial.errors);
                        if display_partial {
                            printer::ast::print(&error_with_partial.partial_result, false);
                        }
                    }
                }
            }
            DevCommands::PrintIr {
                file,
                display_partial,
            } => {
                let model = oneil_model_loader::load_model(file, &file_parser::FileLoader);
                match model {
                    Ok(model) => printer::ir::print(&model),
                    Err((model, error_map)) => {
                        printer::error::loader::print(error_map);
                        if display_partial {
                            printer::ir::print(&model);
                        }
                    }
                }
            }
        },
    }
}
