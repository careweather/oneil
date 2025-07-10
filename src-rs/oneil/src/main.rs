use clap::Parser;
use oneil_model_loader::FileLoader;

use crate::command::{CliCommand, Commands, DevCommands};

mod command;
mod file_parser;

fn main() {
    let cli = CliCommand::parse();

    match cli.command {
        Commands::Dev { command } => match command {
            DevCommands::PrintAst { file } => {
                let ast = file_parser::FileLoader.parse_ast(file);
                println!("{:#?}", ast);
            }
            DevCommands::PrintIr { file } => {
                let model = oneil_model_loader::load_model(file, &file_parser::FileLoader);
                println!("{:#?}", model);
            }
        },
    }
}
