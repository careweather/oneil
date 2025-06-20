pub mod command;

use clap::Parser;
use oneil_parser::{Config, Span};
use std::{fs::File, io::Read};

use crate::command::{CliCommand, Commands, DevCommands};

fn main() {
    let cli = CliCommand::parse();

    match cli.command {
        Commands::Dev { command } => match command {
            DevCommands::PrintAst { file } => {
                let mut file_content = String::new();
                File::open(&file)
                    .expect("Unable to open file")
                    .read_to_string(&mut file_content)
                    .expect("Unable to read file");

                let input = Span::new_extra(&file_content, Config::default());
                // Assuming there's a function to parse the file content into an AST
                let ast = oneil_parser::model::parse_complete(input);

                match ast {
                    Ok((_rest, ast)) => println!("{:#?}", ast),
                    Err(e) => eprintln!("Error printing AST: {:?}", e),
                }
            }
            DevCommands::PrintModules { file } => todo!(),
        },
    }
}
