use clap::Parser;
use oneil_module_loader::FileLoader;

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
            DevCommands::PrintModules { file } => {
                let module = oneil_module_loader::load_module(file, &file_parser::FileLoader);
                println!("{:#?}", module);
            }
        },
    }
}
