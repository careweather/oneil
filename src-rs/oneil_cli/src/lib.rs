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
        error::RuntimeErrors,
        tree::{DependencyTreeValue, ReferenceTreeValue, Tree},
    },
};

use crate::{
    command::{
        BuiltinsCommand, CliCommand, Commands, DevCommand, EvalArgs, IndependentArgs,
        IrIncludeSection, ModelResultIncludeSection, TestArgs, TreeArgs,
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

/// Main entry point for the Oneil CLI application.
pub fn main() {
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
            handle_independent_command(args, cli.dev_show_internal_errors);
        }
    }
}

fn handle_dev_command(command: DevCommand, show_internal_errors: bool) {
    match command {
        DevCommand::PrintAst {
            files,
            partial: display_partial,
        } => handle_print_ast(&files, display_partial, show_internal_errors),
        DevCommand::PrintIr {
            file,
            partial: display_partial,
            recursive,
            include,
            no_values,
        } => {
            let sections = ir_sections_from_include(include.as_deref());
            handle_print_ir(
                &file,
                display_partial,
                recursive,
                &sections,
                no_values,
                show_internal_errors,
            );
        }
        DevCommand::PrintModelResult {
            file,
            partial: display_partial,
            recursive,
            include,
            no_values,
        } => {
            let sections = model_result_sections_from_include(include.as_deref());
            handle_print_model_result(
                &file,
                display_partial,
                recursive,
                &sections,
                no_values,
                show_internal_errors,
            );
        }
        #[cfg(feature = "python")]
        DevCommand::PrintPythonImports { files } => {
            handle_print_python_imports(&files, show_internal_errors);
        }
    }
}

/// Handles the `dev print-python-imports` command.
#[cfg(feature = "python")]
fn handle_print_python_imports(files: &[PathBuf], show_internal_errors: bool) {
    let mut runtime = Runtime::new();

    let mut imports = IndexMap::new();
    let mut errors = Vec::new();

    for file in files {
        let import_result = runtime.load_python_import(file);

        match import_result {
            Ok(import) => {
                let functions: IndexSet<String> = import.into_iter().map(str::to_string).collect();
                imports.insert(file.clone(), functions);
            }

            Err(runtime_errors) => {
                let runtime_errors = runtime_errors.to_vec().into_iter().cloned();
                errors.extend(runtime_errors);
            }
        }
    }

    for error in errors {
        print_error::print(&error, show_internal_errors);
    }

    let is_multiple_files = imports.len() > 1;
    for (file, import) in imports {
        if is_multiple_files {
            println!("===== {} =====", file.display());
        }

        if import.is_empty() {
            let message = format!("no functions found in `{}`", file.display());
            let styled_message = stylesheet::NO_PYTHON_FUNCTIONS_FOUND_MESSAGE.style(message);
            println!("{styled_message}");
            continue;
        }

        for s in import {
            println!("{s}");
        }
    }
}

/// Handles the `dev print-ast` command.
fn handle_print_ast(files: &[PathBuf], display_partial: bool, show_internal_errors: bool) {
    let ast_print_config = AstPrintConfig {};

    let mut runtime = Runtime::new();

    let mut asts = IndexMap::new();
    let mut errors = RuntimeErrors::new();

    for file in files {
        let (ast_opt, runtime_errors) = runtime.load_ast(file);

        if let Some(ast) = ast_opt {
            asts.insert(file.clone(), ast.clone());
        }

        errors.extend(runtime_errors);
    }

    for error in errors.to_vec() {
        print_error::print(error, show_internal_errors);
    }

    if !errors.is_empty() && !display_partial {
        return;
    }

    let is_multiple_files = files.len() > 1;

    for (file, ast) in asts {
        if is_multiple_files {
            println!("===== {} =====", file.display());
        }

        print_debug_ast::print(&ast, &ast_print_config);
    }
}

/// Builds `IrSections` from `--include` list: if None or empty, show all sections;
/// otherwise only the listed sections.
fn ir_sections_from_include(include: Option<&[IrIncludeSection]>) -> print_debug_ir::IrSections {
    let list = match include {
        None | Some([]) => return print_debug_ir::IrSections::All,
        Some(sections) => sections,
    };

    let mut python_imports = false;
    let mut submodels = false;
    let mut references = false;
    let mut parameters = false;
    let mut tests = false;

    for &section in list {
        match section {
            IrIncludeSection::PythonImports => python_imports = true,
            IrIncludeSection::Submodels => submodels = true,
            IrIncludeSection::References => references = true,
            IrIncludeSection::Parameters => parameters = true,
            IrIncludeSection::Tests => tests = true,
        }
    }

    print_debug_ir::IrSections::Specified {
        python_imports,
        submodels,
        references,
        parameters,
        tests,
    }
}

/// Handles the `dev print-ir` command.
#[expect(
    clippy::fn_params_excessive_bools,
    reason = "this is just passing in all the arguments from the CLI"
)]
fn handle_print_ir(
    file: &Path,
    display_partial: bool,
    recursive: bool,
    sections: &print_debug_ir::IrSections,
    no_values: bool,
    show_internal_errors: bool,
) {
    let ir_print_config = IrPrintConfig {
        recursive,
        sections: sections.clone(),
        print_values: !no_values,
    };

    let mut runtime = Runtime::new();

    let (ir_result, errors) = runtime.load_ir(file);

    for error in errors.to_vec() {
        print_error::print(error, show_internal_errors);
    }

    if !errors.is_empty() && !display_partial {
        return;
    }

    if let Some(ir_result) = ir_result {
        print_debug_ir::print(ir_result, &ir_print_config);
    }
}

/// Builds `ModelResultSections` from `--include` list: if None or empty, show all sections;
/// otherwise only the listed sections.
fn model_result_sections_from_include(
    include: Option<&[ModelResultIncludeSection]>,
) -> print_debug_model_result::ModelResultSections {
    let list = match include {
        None | Some([]) => return print_debug_model_result::ModelResultSections::All,
        Some(s) => s,
    };

    let mut submodels = false;
    let mut references = false;
    let mut parameters = false;
    let mut tests = false;

    for &section in list {
        match section {
            ModelResultIncludeSection::Submodels => submodels = true,
            ModelResultIncludeSection::References => references = true,
            ModelResultIncludeSection::Parameters => parameters = true,
            ModelResultIncludeSection::Tests => tests = true,
        }
    }

    print_debug_model_result::ModelResultSections::Specified {
        submodels,
        references,
        parameters,
        tests,
    }
}

/// Handles the `dev print-model-result` command.
#[expect(
    clippy::fn_params_excessive_bools,
    reason = "this is just passing in all the arguments from the CLI"
)]
fn handle_print_model_result(
    file: &Path,
    display_partial: bool,
    recursive: bool,
    sections: &print_debug_model_result::ModelResultSections,
    no_values: bool,
    show_internal_errors: bool,
) {
    let config = print_debug_model_result::DebugModelResultPrintConfig {
        recursive,
        sections: sections.clone(),
        print_values: !no_values,
    };

    let mut runtime = Runtime::new();
    let (model_opt, errors) = runtime.eval_model(file);

    for error in errors.to_vec() {
        print_error::print(error, show_internal_errors);
    }

    if !errors.is_empty() && !display_partial {
        return;
    }

    if let Some(model_ref) = model_opt {
        print_debug_model_result::print(model_ref, &config);
    }
}

fn handle_eval_command(args: EvalArgs, show_internal_errors: bool) {
    let EvalArgs {
        file,
        params: variables,
        print_mode,
        debug: print_debug_info,
        watch,
        exec: exec_expressions,
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
        no_header,
        no_test_report,
        no_parameters,
    };

    if watch {
        watch_model(
            &file,
            &exec_expressions,
            show_internal_errors,
            display_partial_results,
            &model_print_config,
        );
    } else {
        let mut runtime = Runtime::new();

        eval_and_print_model(
            &file,
            &exec_expressions,
            show_internal_errors,
            display_partial_results,
            &model_print_config,
            &mut runtime,
        );
    }
}

fn eval_and_print_model(
    file: &Path,
    exec_expressions: &[String],
    show_internal_errors: bool,
    display_partial_results: bool,
    model_print_config: &ModelPrintConfig,
    runtime: &mut Runtime,
) {
    let (result, model_errors, expr_errors) =
        runtime.eval_model_and_expressions(file, exec_expressions);

    for error in model_errors.to_vec() {
        print_error::print(error, show_internal_errors);
    }

    if !model_errors.is_empty() && !display_partial_results {
        return;
    }

    for error in expr_errors {
        print_error::print(&error, show_internal_errors);
    }

    if let Some((model_ref, exec_results)) = result {
        print_model_result::print_eval_result(model_ref, &exec_results, model_print_config);
    }
}

fn watch_model(
    file: &Path,
    exec_expressions: &[String],
    show_internal_errors: bool,
    display_partial_results: bool,
    model_print_config: &ModelPrintConfig,
) {
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

    eval_and_print_model(
        file,
        exec_expressions,
        show_internal_errors,
        display_partial_results,
        model_print_config,
        &mut runtime,
    );

    let new_watch_paths = runtime.get_watch_paths();
    let (add_paths, remove_paths) = find_watch_paths_difference(&watch_paths, &new_watch_paths);

    update_watcher(&mut watcher, &add_paths, &remove_paths);
    watch_paths = new_watch_paths;

    for event in rx {
        match event {
            Ok(event) => match event.kind {
                notify::EventKind::Modify(_) => {
                    clear_screen();

                    eval_and_print_model(
                        file,
                        exec_expressions,
                        show_internal_errors,
                        display_partial_results,
                        model_print_config,
                        &mut runtime,
                    );

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
    let (model_opt, errors) = runtime.eval_model(&file);

    for error in errors.to_vec() {
        print_error::print(error, show_internal_errors);
    }

    if !errors.is_empty() && !display_partial_results {
        return;
    }

    if let Some(model_ref) = model_opt {
        print_model_result::print_test_results(model_ref, &test_print_config);
    }
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
        partial: display_partial_results,
    } = args;

    let tree_print_config = TreePrintConfig { recursive, depth };

    let mut runtime = Runtime::new();

    let (trees, errors) = if list_refs {
        let mut trees = Vec::new();
        let mut errors = RuntimeErrors::new();

        for param in params {
            let (reference_tree, tree_errors) = runtime.get_reference_tree(&file, &param);

            trees.push((param, reference_tree));
            errors.extend(tree_errors);
        }

        (TreeResults::ReferenceTrees(trees), errors)
    } else {
        let mut trees = Vec::new();
        let mut errors = RuntimeErrors::new();

        for param in params {
            let (dependency_tree, tree_errors) = runtime.get_dependency_tree(&file, &param);

            trees.push((param, dependency_tree));
            errors.extend(tree_errors);
        }

        (TreeResults::DependencyTrees(trees), errors)
    };

    let errors_vec = errors.to_vec();

    for error in &errors_vec {
        print_error::print(error, show_internal_errors);
        eprintln!();
    }

    if !errors_vec.is_empty() && !display_partial_results {
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
    };

    let mut runtime = Runtime::new();
    let (independents, errors) = runtime.get_independents(&file);

    for error in errors.to_vec() {
        print_error::print(error, show_internal_errors);
    }

    if !errors.is_empty() && !display_partial_results {
        return;
    }

    print_independents::print(&file, &independents, &independent_print_config);
}
