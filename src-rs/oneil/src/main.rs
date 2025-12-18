#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! CLI for the Oneil programming language

use std::path::Path;

use anstream::{ColorChoice, eprintln, println};
use clap::Parser;
use oneil_eval::builtin::std as oneil_std;
use oneil_model_resolver::FileLoader;
use oneil_runner::{
    builtins::Builtins,
    file_parser::{self, LoadingError},
};

use crate::command::{CliCommand, Commands, DevCommand};

mod command;
mod convert_error;
mod print_ast;
mod print_error;
mod print_ir;
mod stylesheet;

/// Main entry point for the Oneil CLI application
fn main() {
    let cli = CliCommand::parse();

    match cli.command {
        Commands::Lsp {} => {
            oneil_lsp::run();
        }
        Commands::Dev { command } => handle_dev_command(command),
        Commands::Eval {
            file,
            print_debug,
            no_colors,
        } => handle_eval_command(&file, print_debug, no_colors),
    }
}

fn handle_dev_command(command: DevCommand) {
    match command {
        DevCommand::PrintAst {
            files,
            display_partial,
            print_debug,
            no_colors,
        } => {
            set_color_choice(no_colors);

            let is_multiple_files = files.len() > 1;
            for file in files {
                if is_multiple_files {
                    println!("===== {} =====", file.display());
                }

                let ast = file_parser::FileLoader.parse_ast(&file);
                match ast {
                    Ok(ast) => print_ast::print(&ast, print_debug),
                    Err(LoadingError::InvalidFile(error)) => {
                        let error = convert_error::file::convert(&file, &error);
                        print_error::print(&error, print_debug);
                    }
                    Err(LoadingError::Parser(error_with_partial)) => {
                        let errors =
                            convert_error::parser::convert_all(&file, &error_with_partial.errors);

                        for error in errors {
                            print_error::print(&error, print_debug);
                            eprintln!();
                        }

                        if display_partial {
                            print_ast::print(&error_with_partial.partial_result, print_debug);
                        }
                    }
                }
            }
        }
        DevCommand::PrintIr {
            file,
            display_partial,
            print_debug,
            no_colors,
        } => {
            set_color_choice(no_colors);

            let builtin_variables = Builtins::new(
                oneil_std::builtin_values(),
                oneil_std::builtin_functions(),
                oneil_std::builtin_units(),
                oneil_std::builtin_prefixes(),
            );

            let model_collection = oneil_model_resolver::load_model(
                file,
                &builtin_variables,
                &file_parser::FileLoader,
            );
            match model_collection {
                Ok(model_collection) => print_ir::print(&model_collection, print_debug),
                Err(error) => {
                    let (model_collection, error_map) = *error;
                    let errors = convert_error::loader::convert_map(&error_map);
                    for error in errors {
                        print_error::print(&error, print_debug);
                        eprintln!();
                    }

                    if display_partial {
                        print_ir::print(&model_collection, print_debug);
                    }
                }
            }
        }
    }
}

fn handle_eval_command(file: &Path, print_debug: bool, no_colors: bool) {
    set_color_choice(no_colors);

    let builtins = Builtins::new(
        oneil_std::builtin_values(),
        oneil_std::builtin_functions(),
        oneil_std::builtin_units(),
        oneil_std::builtin_prefixes(),
    );

    let model_collection =
        oneil_model_resolver::load_model(&file, &builtins, &file_parser::FileLoader);
    let model_collection = match model_collection {
        Ok(model_collection) => model_collection,
        Err(error) => {
            let (_model_collection, error_map) = *error;
            let errors = convert_error::loader::convert_map(&error_map);
            for error in errors {
                print_error::print(&error, print_debug);
                eprintln!();
            }
            return;
        }
    };

    let eval_context = oneil_eval::eval_model_collection(&model_collection, builtins.builtin_map);

    let model = eval_context.get_model_result(&file);
    println!("{model:?}");
}

fn set_color_choice(no_colors: bool) {
    let color_choice = if no_colors {
        ColorChoice::Never
    } else {
        ColorChoice::Auto
    };
    ColorChoice::write_global(color_choice);
}
