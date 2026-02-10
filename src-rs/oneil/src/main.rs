#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! CLI for the Oneil programming language

#![expect(
    clippy::multiple_crate_versions,
    reason = "this isn't causing problems, and it's going to take time to fix"
)]

use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::mpsc,
};

use anstream::{ColorChoice, eprintln, print, println};
use clap::Parser;
use indexmap::{IndexMap, IndexSet};
use notify::Watcher;
use oneil_runtime::{
    Runtime,
    output::{
        Tree,
        dependency::{DependencyTreeValue, ReferenceTreeValue},
        error::TreeError,
    },
};

use crate::{
    command::{
        BuiltinsCommand, CliCommand, Commands, DevCommand, EvalArgs, IndependentArgs, TestArgs,
        TreeArgs,
    },
    print_debug_ast::AstPrintConfig,
    print_debug_ir::IrPrintConfig,
    print_independents::IndependentPrintConfig,
    print_model_result::{ModelPrintConfig, TestPrintConfig},
    print_tree::TreePrintConfig,
};

mod command;
mod print_builtins;
mod print_debug_ast;
mod print_debug_ir;
mod print_debug_model_result;
mod print_error;
mod print_independents;
mod print_model_result;
mod print_tree;
mod print_utils;
mod stylesheet;

/// Main entry point for the Oneil CLI application
fn main() {
    let cli = CliCommand::parse();

    set_color_choice(cli.no_colors);

    match cli.command {
        Commands::Lsp {} => {
            // TODO: uncomment this when we're ready to deal with
            //       the LSP
            //oneil_lsp::run();
        }
        Commands::Dev { command } => handle_dev_command(command, cli.dev_show_internal_errors),
        Commands::Eval(args) => handle_eval_command(args, cli.dev_show_internal_errors),
        Commands::Test(args) => handle_test_command(args, cli.dev_show_internal_errors),
        Commands::Tree(args) => handle_tree_command(args, cli.dev_show_internal_errors),
        Commands::Builtins { command } => handle_builtins_command(command),
        Commands::Independent(args) => {
            handle_independent_command(args, cli.dev_show_internal_errors)
        }
    }
}

fn handle_dev_command(command: DevCommand, show_internal_errors: bool) {
    match command {
        DevCommand::PrintAst {
            files,
            display_partial,
        } => handle_print_ast(&files, display_partial, show_internal_errors),
        DevCommand::PrintIr {
            file,
            display_partial,
            recursive,
        } => handle_print_ir(&file, display_partial, recursive, show_internal_errors),
        DevCommand::PrintModelResult {
            file,
            display_partial,
            recursive,
        } => handle_print_model_result(&file, display_partial, recursive, show_internal_errors),
    }
}

/// Handles the `dev print-ast` command.
fn handle_print_ast(files: &[PathBuf], display_partial: bool, show_internal_errors: bool) {
    let ast_print_config = AstPrintConfig {
        display_partial,
        show_internal_errors,
    };

    let mut runtime = Runtime::new();

    let is_multiple_files = files.len() > 1;
    for file in files {
        if is_multiple_files {
            println!("===== {} =====", file.display());
        }

        let ast_result = runtime.load_ast(file);

        print_debug_ast::print(ast_result, &ast_print_config);
    }
}

/// Handles the `dev print-ir` command.
fn handle_print_ir(
    file: &Path,
    display_partial: bool,
    recursive: bool,
    show_internal_errors: bool,
) {
    let ir_print_config = IrPrintConfig {
        display_partial,
        recursive,
        show_internal_errors,
    };

    let mut runtime = Runtime::new();

    let ir_result = runtime.load_ir(file);

    print_debug_ir::print(ir_result, &ir_print_config);
}

/// Handles the `dev print-model-result` command.
fn handle_print_model_result(
    file: &Path,
    display_partial: bool,
    recursive: bool,
    show_internal_errors: bool,
) {
    let config = print_debug_model_result::DebugModelResultPrintConfig {
        display_partial,
        recursive,
        show_internal_errors,
    };

    let mut runtime = Runtime::new();
    let eval_result = runtime.eval_model(file);

    print_debug_model_result::print(eval_result, &config);
}

fn handle_eval_command(args: EvalArgs, show_internal_errors: bool) {
    let EvalArgs {
        file,
        params: variables,
        print_mode,
        debug: print_debug_info,
        watch,
        recursive,
        partial: display_partial_results,
        no_header,
        no_test_report,
        no_parameters,
    } = args;

    let model_print_config = ModelPrintConfig {
        print_mode,
        print_debug_info,
        variables,
        recursive,
        display_partial_results,
        no_header,
        no_test_report,
        no_parameters,
        show_internal_errors,
    };

    if watch {
        watch_model(&file, &model_print_config);
    } else {
        let mut runtime = Runtime::new();

        let eval_result = runtime.eval_model(&file);

        print_model_result::print_eval_result(eval_result, &model_print_config);
    }
}

fn watch_model(file: &Path, model_print_config: &ModelPrintConfig) {
    let mut runtime = Runtime::new();

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

    let mut watch_paths = IndexSet::new();

    clear_screen();

    let eval_result = runtime.eval_model(file);
    print_model_result::print_eval_result(eval_result, model_print_config);

    let new_watch_paths = runtime.get_watch_paths();
    let (add_paths, remove_paths) = find_watch_paths_difference(&watch_paths, &new_watch_paths);

    update_watcher(&mut watcher, &add_paths, &remove_paths);
    watch_paths = new_watch_paths;

    for event in rx {
        match event {
            Ok(event) => match event.kind {
                notify::EventKind::Modify(_) => {
                    clear_screen();

                    let eval_result = runtime.eval_model(file);
                    print_model_result::print_eval_result(eval_result, model_print_config);

                    let new_watch_paths = runtime.get_watch_paths();
                    let (add_paths, remove_paths) =
                        find_watch_paths_difference(&watch_paths, &new_watch_paths);

                    update_watcher(&mut watcher, &add_paths, &remove_paths);
                    watch_paths = new_watch_paths;
                }
                notify::EventKind::Any
                | notify::EventKind::Access(_)
                | notify::EventKind::Create(_)
                | notify::EventKind::Remove(_)
                | notify::EventKind::Other => { /* do nothing */ }
            },
            Err(error) => {
                let error_msg = stylesheet::ERROR_COLOR.bold().style("watcher error:");
                eprintln!("{error_msg} - {error}");
            }
        }
    }
}

fn find_watch_paths_difference<'a>(
    old_paths: &'a IndexSet<PathBuf>,
    new_paths: &'a IndexSet<PathBuf>,
) -> (IndexSet<&'a PathBuf>, IndexSet<&'a PathBuf>) {
    let add_paths = new_paths.difference(old_paths).collect();
    let remove_paths = old_paths.difference(new_paths).collect();
    (add_paths, remove_paths)
}

fn update_watcher(
    watcher: &mut notify::RecommendedWatcher,
    add_paths: &IndexSet<&PathBuf>,
    remove_paths: &IndexSet<&PathBuf>,
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

fn handle_test_command(args: TestArgs, show_internal_errors: bool) {
    let TestArgs {
        file,
        recursive,
        partial: display_partial_results,
        no_header,
        no_test_report,
    } = args;

    let test_print_config = TestPrintConfig {
        no_header,
        no_test_report,
        recursive,
        display_partial_results,
        show_internal_errors,
    };

    let mut runtime = Runtime::new();

    let eval_result = runtime.eval_model(&file);

    print_model_result::print_test_results(eval_result, &test_print_config);
}

fn handle_tree_command(args: TreeArgs, show_internal_errors: bool) {
    enum TreeResults {
        ReferenceTrees(Vec<(String, Option<Tree<ReferenceTreeValue>>)>),
        DependencyTrees(Vec<(String, Option<Tree<DependencyTreeValue>>)>),
    }

    let TreeArgs {
        file,
        params,
        list_refs,
        recursive,
        depth,
        partial,
    } = args;

    let tree_print_config = TreePrintConfig { recursive, depth };

    let mut runtime = Runtime::new();

    let (trees, errors) = if list_refs {
        let mut trees = Vec::new();
        let mut errors = IndexMap::<PathBuf, TreeError>::new();

        for param in params {
            let (reference_tree, tree_errors) = runtime.get_reference_tree(&file, &param);

            trees.push((param, reference_tree));
            for (path, error) in tree_errors {
                errors.entry(path).or_default().insert_all(error);
            }
        }

        (TreeResults::ReferenceTrees(trees), errors)
    } else {
        let mut trees = Vec::new();
        let mut errors = IndexMap::<PathBuf, TreeError>::new();

        for param in params {
            let (dependency_tree, tree_errors) = runtime.get_dependency_tree(&file, &param);

            trees.push((param, dependency_tree));
            for (path, error) in tree_errors {
                errors.entry(path).or_default().insert_all(error);
            }
        }

        (TreeResults::DependencyTrees(trees), errors)
    };

    let errors = errors
        .into_values()
        .flat_map(|errors| errors.to_vec())
        .collect::<Vec<_>>();

    for error in &errors {
        print_error::print(error, show_internal_errors);
        eprintln!();
    }

    if !errors.is_empty() && !partial {
        return;
    }

    let mut file_cache = std::collections::HashMap::new();

    match trees {
        TreeResults::ReferenceTrees(trees) => {
            for (param, tree) in trees {
                match tree {
                    Some(reference_tree) => {
                        print_tree::print_reference_tree(
                            &file,
                            &reference_tree,
                            &tree_print_config,
                            &mut file_cache,
                        );
                    }
                    None => {
                        print_param_not_found(&param);
                    }
                }
            }
        }
        TreeResults::DependencyTrees(trees) => {
            for (param, tree) in trees {
                match tree {
                    Some(dependency_tree) => {
                        print_tree::print_dependency_tree(
                            &file,
                            &dependency_tree,
                            &tree_print_config,
                            &mut file_cache,
                        );
                    }
                    None => {
                        print_param_not_found(&param);
                    }
                }
            }
        }
    }
}

fn print_param_not_found(param: &str) {
    let error_label = stylesheet::ERROR_COLOR.bold().style("error:");
    eprintln!("{error_label} parameter \"{param}\" not found in model");
}

fn handle_builtins_command(command: Option<BuiltinsCommand>) {
    let runtime = Runtime::new();
    match command {
        None | Some(BuiltinsCommand::All) => print_builtins::print_builtins_all(&runtime),
        Some(BuiltinsCommand::Units {
            unit_name: Some(unit_name),
        }) => print_builtins::search_builtins_units(&runtime, &unit_name),
        Some(BuiltinsCommand::Units { unit_name: None }) => {
            print_builtins::print_builtins_units(&runtime);
        }
        Some(BuiltinsCommand::Functions {
            function_name: Some(function_name),
        }) => print_builtins::search_builtins_functions(&runtime, &function_name),
        Some(BuiltinsCommand::Functions {
            function_name: None,
        }) => print_builtins::print_builtins_functions(&runtime),
        Some(BuiltinsCommand::Values {
            value_name: Some(value_name),
        }) => print_builtins::search_builtins_values(&runtime, &value_name),
        Some(BuiltinsCommand::Values { value_name: None }) => {
            print_builtins::print_builtins_values(&runtime);
        }
        Some(BuiltinsCommand::Prefixes {
            prefix_name: Some(prefix_name),
        }) => print_builtins::search_builtins_prefixes(&runtime, &prefix_name),
        Some(BuiltinsCommand::Prefixes { prefix_name: None }) => {
            print_builtins::print_builtins_prefixes(&runtime);
        }
    }
}

fn handle_independent_command(args: IndependentArgs, show_internal_errors: bool) {
    let IndependentArgs {
        file,
        recursive,
        values: print_values,
        partial: display_partial_results,
    } = args;

    let independent_print_config = IndependentPrintConfig {
        print_values,
        recursive,
        display_partial_results,
        show_internal_errors,
    };

    let mut runtime = Runtime::new();
    let model_result = runtime.eval_model(&file);

    print_independents::print(model_result, &independent_print_config);
}
