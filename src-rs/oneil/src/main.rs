#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! CLI for the Oneil programming language

#![expect(
    clippy::multiple_crate_versions,
    reason = "this isn't causing problems, and it's going to take time to fix"
)]

use std::{
    collections::HashSet,
    io::Write,
    path::{Path, PathBuf},
    sync::mpsc,
};

use anstream::{ColorChoice, eprintln, print, println};
use clap::Parser;
use notify::Watcher;
use oneil_eval::builtin::{BuiltinFunction, std as oneil_std};
use oneil_ir as ir;
use oneil_model_resolver::FileLoader;
use oneil_runner::{
    builtins::Builtins,
    file_parser::{self, LoadingError},
};

use crate::{
    command::{CliCommand, Commands, DevCommand, EvalArgs},
    print_model_result::ModelPrintConfig,
};

mod command;
mod convert_error;
mod print_ast;
mod print_error;
mod print_ir;
mod print_model_result;
mod stylesheet;

/// Main entry point for the Oneil CLI application
fn main() {
    let cli = CliCommand::parse();

    match cli.command {
        Commands::Lsp {} => {
            oneil_lsp::run();
        }
        Commands::Dev { command } => handle_dev_command(command),
        Commands::Eval(args) => handle_eval_command(args),
    }
}

fn handle_dev_command(command: DevCommand) {
    match command {
        DevCommand::PrintAst {
            files,
            display_partial,
            print_debug,
            no_colors,
        } => handle_print_ast(&files, display_partial, print_debug, no_colors),
        DevCommand::PrintIr {
            file,
            display_partial,
            print_debug,
            no_colors,
        } => handle_print_ir(&file, display_partial, print_debug, no_colors),
        DevCommand::PrintModelResult { file, no_colors } => {
            handle_print_model_result(&file, no_colors);
        }
    }
}

/// Handles the `dev print-ast` command.
fn handle_print_ast(files: &[PathBuf], display_partial: bool, print_debug: bool, no_colors: bool) {
    set_color_choice(no_colors);

    let is_multiple_files = files.len() > 1;
    for file in files {
        if is_multiple_files {
            println!("===== {} =====", file.display());
        }

        let ast = file_parser::FileLoader.parse_ast(file);
        match ast {
            Ok(ast) => print_ast::print(&ast, print_debug),
            Err(LoadingError::InvalidFile(error)) => {
                let error = convert_error::file::convert(file, &error);
                print_error::print(&error, print_debug);
            }
            Err(LoadingError::Parser(error_with_partial)) => {
                let errors = convert_error::parser::convert_all(file, &error_with_partial.errors);

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

/// Handles the `dev print-ir` command.
fn handle_print_ir(file: &Path, display_partial: bool, print_debug: bool, no_colors: bool) {
    set_color_choice(no_colors);

    let builtin_variables = create_builtins();

    let model_collection =
        oneil_model_resolver::load_model(file, &builtin_variables, &file_parser::FileLoader);
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

/// Handles the `dev print-model-result` command.
fn handle_print_model_result(file: &Path, no_colors: bool) {
    set_color_choice(no_colors);

    let builtins = create_builtins();

    let Some(model_collection) = load_model_collection(file, &builtins, false) else {
        return;
    };

    let eval_context = oneil_eval::eval_model_collection(&model_collection, builtins.builtin_map);

    let model_result = eval_context.get_model_result(file);

    match model_result {
        Ok(model_result) => {
            println!("{:?}", model_result);
        }
        Err(errors) => {
            for error in errors {
                let error = convert_error::eval::convert(&error);
                print_error::print(&error, false);
                eprintln!();
            }
        }
    }
}

/// Creates a new `Builtins` instance with standard library builtins.
fn create_builtins() -> Builtins<oneil_std::StdBuiltinFunction> {
    Builtins::new(
        oneil_std::builtin_values(),
        oneil_std::builtin_functions(),
        oneil_std::builtin_units(),
        oneil_std::builtin_prefixes(),
    )
}

/// Loads a model collection, printing errors if loading fails.
///
/// Returns `Some(model_collection)` if loading succeeds, otherwise prints errors and returns `None`.
fn load_model_collection<F: BuiltinFunction>(
    file: &Path,
    builtins: &Builtins<F>,
    print_debug: bool,
) -> Option<Box<ir::ModelCollection>> {
    let model_collection =
        oneil_model_resolver::load_model(file, builtins, &file_parser::FileLoader);
    match model_collection {
        Ok(model_collection) => Some(model_collection),
        Err(error) => {
            let (_model_collection, error_map) = *error;
            let errors = convert_error::loader::convert_map(&error_map);
            for error in errors {
                print_error::print(&error, print_debug);
                eprintln!();
            }
            None
        }
    }
}

fn handle_eval_command(args: EvalArgs) {
    let EvalArgs {
        file,
        params: variables,
        print_mode,
        debug: print_debug_info,
        watch,
        top_only,
        no_header,
        no_test_report,
        no_parameters,
        no_colors,
    } = args;

    set_color_choice(no_colors);

    let model_print_config = ModelPrintConfig {
        print_mode,
        print_debug_info,
        variables,
        top_model_only: top_only,
        no_header,
        no_test_report,
        no_parameters,
    };

    let builtins = Builtins::new(
        oneil_std::builtin_values(),
        oneil_std::builtin_functions(),
        oneil_std::builtin_units(),
        oneil_std::builtin_prefixes(),
    );

    if watch {
        watch_model(&file, &builtins, model_print_config);
    } else {
        let _watch_paths = eval_model(&file, &builtins, model_print_config);
    }
}

fn watch_model<F: BuiltinFunction + Clone>(
    file: &Path,
    builtins: &Builtins<F>,
    model_print_config: ModelPrintConfig,
) {
    let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();

    let mut watcher = match notify::recommended_watcher(tx) {
        Ok(watcher) => watcher,
        Err(error) => {
            // we don't expect this to happen often, so we can just print the error and return
            let error_msg = stylesheet::ERROR_COLOR
                .bold()
                .style("error: failed to create watcher");
            eprintln!("{error_msg} - {error}");
            return;
        }
    };

    clear_screen();
    let watch_paths = eval_model(file, builtins, model_print_config);

    update_watch_paths(&mut watcher, &watch_paths, &HashSet::new());

    for event in rx {
        match event {
            Ok(event) => {
                clear_screen();
                println!("{:?}", event);
            }
            Err(error) => println!("error: {error}"),
        }
    }
}

fn eval_model<F: BuiltinFunction + Clone>(
    file: &Path,
    builtins: &Builtins<F>,
    model_print_config: ModelPrintConfig,
) -> HashSet<PathBuf> {
    let model_collection =
        oneil_model_resolver::load_model(file, builtins, &file_parser::FileLoader);
    let model_collection = match model_collection {
        Ok(model_collection) => model_collection,
        Err(error) => {
            let (model_collection, error_map) = *error;

            let errors = convert_error::loader::convert_map(&error_map);
            for error in errors {
                print_error::print(&error, false);
                eprintln!();
            }

            return watch_paths_from_model_collection(&model_collection);
        }
    };

    // TODO: remove this clone?
    let eval_context =
        oneil_eval::eval_model_collection(&model_collection, builtins.builtin_map.clone());
    let model_result = eval_context.get_model_result(file);

    match model_result {
        Ok(model_result) => {
            print_model_result::print(&model_result, model_print_config);
        }
        Err(errors) => {
            for error in errors {
                let error = convert_error::eval::convert(&error);
                print_error::print(&error, false);
                eprintln!();
            }
        }
    }

    watch_paths_from_model_collection(&model_collection)
}

fn watch_paths_from_model_collection(model_collection: &ir::ModelCollection) -> HashSet<PathBuf> {
    let model_paths = model_collection
        .get_models()
        .keys()
        .map(|path| path.as_ref().to_path_buf());

    let python_imports = model_collection
        .get_python_imports()
        .into_iter()
        .map(|import| import.import_path().as_ref().to_path_buf());

    model_paths.chain(python_imports).collect()
}

fn update_watch_paths(
    watcher: &mut notify::RecommendedWatcher,
    add_paths: &HashSet<PathBuf>,
    remove_paths: &HashSet<PathBuf>,
) {
    let mut watcher_paths_mut = watcher.paths_mut();

    for path in add_paths {
        let result = watcher_paths_mut.add(path, notify::RecursiveMode::NonRecursive);
        if let Err(error) = result {
            let error_msg = format!("error: failed to add path {} to watcher", path.display());
            let error_msg = stylesheet::ERROR_COLOR.bold().style(error_msg);
            eprintln!("{error_msg} - {error}");
        }
    }

    for path in remove_paths {
        let result = watcher_paths_mut.remove(path);
        if let Err(error) = result {
            let error_msg = format!(
                "error: failed to remove path {} from watcher",
                path.display()
            );
            let error_msg = stylesheet::ERROR_COLOR.bold().style(error_msg);
            eprintln!("{error_msg} - {error}");
        }
    }

    let commit_result = watcher_paths_mut.commit();
    if let Err(error) = commit_result {
        let error_msg = stylesheet::ERROR_COLOR
            .bold()
            .style("error: failed to commit watcher paths");
        eprintln!("{error_msg} - {error}");
    }
}

fn set_color_choice(no_colors: bool) {
    let color_choice = if no_colors {
        ColorChoice::Never
    } else {
        ColorChoice::Auto
    };
    ColorChoice::write_global(color_choice);
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    std::io::stdout().flush().expect("failed to flush stdout");
}
