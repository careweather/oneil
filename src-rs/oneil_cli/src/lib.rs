#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! CLI for the Oneil programming language

#![expect(
    clippy::multiple_crate_versions,
    reason = "this isn't causing problems, and it's going to take time to fix"
)]

use std::{io::Write, sync::mpsc};

use anstream::{ColorChoice, eprintln, print, println};
use clap::Parser;
use indexmap::{IndexMap, IndexSet};
use notify::Watcher;
#[cfg(feature = "python")]
use oneil_runtime::output::PythonModule;
use oneil_runtime::{
    Runtime,
    output::{
        error::RuntimeErrors,
        tree::{DependencyTreeValue, ReferenceTreeValue, Tree},
    },
};
#[cfg(feature = "python")]
use oneil_shared::paths::PythonPath;
use oneil_shared::{
    paths::{ModelPath, SourcePath},
    symbols::ParameterName,
};

use crate::{
    command::{
        BuiltinsCommand, CheckArgs, CliCommand, Commands, CommonArgs, DevCommand, EvalArgs,
        IndependentArgs, IrIncludeSection, LspArgs, ModelResultIncludeSection, TestArgs, TreeArgs,
    },
    print_debug_ast::AstPrintConfig,
    print_debug_ir::IrPrintConfig,
    print_independents::IndependentPrintConfig,
    print_model_result::{ModelPrintConfig, TestPrintConfig},
    print_tree::TreePrintConfig,
    print_utils::PrintUtilsConfig,
};

mod command;
mod panic_handler;
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

#[cfg(feature = "python")]
mod load_python_venv;

/// Main entry point for the Oneil CLI application.
pub fn main() {
    panic_handler::register_panic_handler();

    let cli = CliCommand::parse();

    // handle common args
    let common_args = cli.get_common_args();
    apply_common_side_effects(common_args);

    match cli.get_command() {
        Commands::Eval(eval_args) => handle_eval_command(eval_args),
        Commands::Check(check_args) => handle_check_command(check_args),
        Commands::Test(test_args) => handle_test_command(test_args),
        Commands::Tree(tree_args) => handle_tree_command(tree_args),
        Commands::Builtins(builtins_args) => handle_builtins_command(builtins_args.get_command()),
        Commands::Independent(independent_args) => handle_independent_command(independent_args),
        Commands::Lsp(lsp_args) => handle_lsp_command(lsp_args),
        Commands::Dev(dev_command) => handle_dev_command(dev_command),
    }
}

fn handle_lsp_command(args: LspArgs) {
    let LspArgs { common } = args;
    // TODO: figure out how to handle common args for the LSP
    let _ = common;
    oneil_lsp::run();
}

fn handle_dev_command(command: DevCommand) {
    match command {
        DevCommand::PrintAst {
            files,
            debug: display_partial,
            common,
        } => handle_print_ast(&files, display_partial, common.dev_show_internal_errors),
        DevCommand::PrintIr {
            file,
            debug: display_partial,
            recursive,
            include,
            no_values,
            common,
        } => {
            let sections = ir_sections_from_include(include.as_deref());
            handle_print_ir(
                &file,
                display_partial,
                recursive,
                &sections,
                no_values,
                common.dev_show_internal_errors,
            );
        }
        DevCommand::PrintModelResult {
            file,
            debug: display_partial_results,
            recursive,
            include,
            no_values,
            common,
        } => {
            let sections = model_result_sections_from_include(include.as_deref());
            handle_print_model_result(
                &file,
                display_partial_results,
                recursive,
                &sections,
                no_values,
                common.dev_show_internal_errors,
            );
        }
        #[cfg(feature = "python")]
        DevCommand::PrintPythonImports { files, common } => {
            handle_print_python_imports(&files, common.dev_show_internal_errors);
        }
    }
}

fn apply_common_side_effects(common_args: &CommonArgs) {
    set_color_choice(common_args.no_colors);

    #[cfg(feature = "python")]
    load_python_venv::try_load_venv(common_args.venv_path.as_deref());
}

/// Handles the `dev print-python-imports` command.
#[cfg(feature = "python")]
fn handle_print_python_imports(files: &[PythonPath], show_internal_errors: bool) {
    let mut runtime = Runtime::new();

    let mut imports: IndexMap<PythonPath, PythonModule> = IndexMap::new();
    let mut errors = Vec::new();

    for file in files {
        let import_result = runtime.load_python_import(file);

        match import_result {
            Ok(module) => {
                imports.insert(file.clone(), module.clone());
            }

            Err(runtime_errors) => {
                let runtime_errors = runtime_errors.to_vec().into_iter().cloned();
                errors.extend(runtime_errors);
            }
        }
    }

    let print_result = print_error::print_all(errors, show_internal_errors);
    if print_result.saw_error_diagnostic() {
        return;
    }

    let is_multiple_files = imports.len() > 1;
    for (file, module) in imports {
        if is_multiple_files {
            println!("===== {} =====", file.as_path().display());
        }

        let doc_string = module.get_docs();

        if let Some(doc_string) = doc_string {
            let styled_doc_string = stylesheet::PYTHON_MODULE_DOC_STRING.style(doc_string);
            println!("{styled_doc_string}");
            println!();
        }

        let header = stylesheet::PYTHON_MODULE_SECTION_HEADER.style("Functions:");
        println!("{header}");

        let functions = module.get_function_names().collect::<Vec<_>>();

        if functions.is_empty() {
            let message = format!("  (no functions found in `{}`)", file.as_path().display());
            let styled_message = stylesheet::NO_PYTHON_FUNCTIONS_FOUND_MESSAGE.style(message);
            println!("{styled_message}");
        } else {
            for function in functions {
                let styled_function =
                    stylesheet::PYTHON_MODULE_SECTION_ITEM.style(function.as_str());
                println!("- {styled_function}");
            }
        }

        let imports = module.get_imports();
        if !imports.is_empty() {
            println!();

            let header = stylesheet::PYTHON_MODULE_SECTION_HEADER.style("Imports:");
            println!("{header}");

            for import in imports {
                let styled_import = stylesheet::PYTHON_MODULE_SECTION_ITEM.style(import.display());
                println!("- {styled_import}");
            }
        }
    }
}

/// Handles the `dev print-ast` command.
fn handle_print_ast(files: &[ModelPath], display_partial: bool, show_internal_errors: bool) {
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

    let print_result = print_error::print_all(errors.to_vec(), show_internal_errors);
    if print_result.saw_error_diagnostic() && !display_partial {
        return;
    }

    let is_multiple_files = files.len() > 1;

    for (file, ast) in asts {
        if is_multiple_files {
            println!("===== {} =====", file.as_path().display());
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
    file: &ModelPath,
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

    let (ir_result, errors) = runtime.load_and_lower(file);

    let print_result = print_error::print_all(errors.to_vec(), show_internal_errors);
    if print_result.saw_error_diagnostic() && !display_partial {
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
    file: &ModelPath,
    display_partial_results: bool,
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

    let print_result = print_error::print_all(errors.to_vec(), show_internal_errors);
    if print_result.saw_error_diagnostic() && !display_partial_results {
        return;
    }

    if let Some(model_ref) = model_opt {
        print_debug_model_result::print(model_ref, &config);
    }
}

fn handle_eval_command(args: EvalArgs) {
    let EvalArgs {
        file,
        params: variables,
        print: print_mode,
        debug: display_partial_results,
        watch,
        expr: eval_expressions,
        recursive,
        with_header,
        with_test_report,
        common,
    } = args;

    let file = file.expect("file should be provided since it is required");

    let print_utils_config = PrintUtilsConfig {
        sig_figs: common.sig_figs,
    };

    // When running a design file directly (e.g., `oneil high_dv.one`),
    // we want the hint to say "Run `oneil test high_dv.one`", not the target model.
    // Check if the file is a design file based on extension.
    let hint_path = file.is_design_file().then(|| file.as_path().to_path_buf());

    let model_print_config = ModelPrintConfig {
        print_mode,
        print_debug_info: display_partial_results,
        variables,
        recursive,
        with_header,
        with_test_report,
        print_utils_config,
        hint_path,
    };

    if watch {
        watch_model(
            &file,
            &eval_expressions,
            common.dev_show_internal_errors,
            display_partial_results,
            &model_print_config,
        );
        return;
    }

    let mut runtime = Runtime::new();
    eval_and_print_model(
        &file,
        &eval_expressions,
        common.dev_show_internal_errors,
        display_partial_results,
        &model_print_config,
        &mut runtime,
    );
}

fn eval_and_print_model(
    file: &ModelPath,
    eval_expressions: &[String],
    show_internal_errors: bool,
    display_partial_results: bool,
    model_print_config: &ModelPrintConfig,
    runtime: &mut Runtime,
) {
    let (result, model_errors, expr_errors) =
        runtime.eval_model_and_expressions(file, eval_expressions);

    let model_print_result = print_error::print_all(model_errors.to_vec(), show_internal_errors);
    if model_print_result.saw_error_diagnostic() && !display_partial_results {
        return;
    }

    let expr_print_result = print_error::print_all(expr_errors, show_internal_errors);
    if expr_print_result.saw_error_diagnostic() && !display_partial_results {
        return;
    }

    if let Some((model_ref, expr_results)) = result {
        print_model_result::print_eval_result(model_ref, &expr_results, model_print_config);
    }
}

fn watch_model(
    file: &ModelPath,
    eval_expressions: &[String],
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

    let mut watch_paths: IndexSet<SourcePath> = IndexSet::new();

    clear_screen();

    eval_and_print_model(
        file,
        eval_expressions,
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
                        eval_expressions,
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
    old_paths: &'a IndexSet<SourcePath>,
    new_paths: &'a IndexSet<SourcePath>,
) -> (IndexSet<&'a SourcePath>, IndexSet<&'a SourcePath>) {
    let add_paths = new_paths.difference(old_paths).collect();
    let remove_paths = old_paths.difference(new_paths).collect();
    (add_paths, remove_paths)
}

fn update_watcher(
    watcher: &mut notify::RecommendedWatcher,
    add_paths: &IndexSet<&SourcePath>,
    remove_paths: &IndexSet<&SourcePath>,
) {
    let mut watcher_paths_mut = watcher.paths_mut();

    for path in add_paths {
        let result = watcher_paths_mut.add(path.as_path(), notify::RecursiveMode::NonRecursive);
        if let Err(error) = result {
            let error_msg = format!(
                "error: failed to add path {} to watcher",
                path.as_path().display()
            );
            let error_msg = stylesheet::ERROR_COLOR.bold().style(error_msg);
            eprintln!("{error_msg} - {error}");
        }
    }

    for path in remove_paths {
        let result = watcher_paths_mut.remove(path.as_path());
        if let Err(error) = result {
            let error_msg = format!(
                "error: failed to remove path {} from watcher",
                path.as_path().display()
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

fn handle_test_command(args: TestArgs) {
    let TestArgs {
        file,
        recursive,
        debug: display_partial_results,
        with_header,
        common,
    } = args;

    let print_utils_config = PrintUtilsConfig {
        sig_figs: common.sig_figs,
    };

    let test_print_config = TestPrintConfig {
        with_header,
        recursive,
        print_utils_config,
    };

    let mut runtime = Runtime::new();
    let (model_opt, errors) = runtime.eval_model(&file);

    let print_result = print_error::print_all(errors.to_vec(), common.dev_show_internal_errors);
    if print_result.saw_error_diagnostic() && !display_partial_results {
        return;
    }

    if let Some(model_ref) = model_opt {
        print_model_result::print_test_results(model_ref, &test_print_config);
    }
}

fn handle_tree_command(args: TreeArgs) {
    enum TreeResults {
        ReferenceTrees(Vec<(ParameterName, Option<Tree<ReferenceTreeValue>>)>),
        DependencyTrees(Vec<(ParameterName, Option<Tree<DependencyTreeValue>>)>),
    }

    let TreeArgs {
        file,
        params,
        up,
        down: _, // down is ignored since it is the default behavior anyway
        recursive,
        depth,
        debug: display_partial_results,
        common,
    } = args;

    let print_utils_config = PrintUtilsConfig {
        sig_figs: common.sig_figs,
    };

    let tree_print_config = TreePrintConfig {
        recursive,
        depth,
        print_utils_config,
    };

    let mut runtime = Runtime::new();

    let (trees, errors) = if up {
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

    let print_result = print_error::print_all(errors_vec, common.dev_show_internal_errors);
    if print_result.saw_error_diagnostic() && !display_partial_results {
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

fn print_param_not_found(param: &ParameterName) {
    let error_label = stylesheet::ERROR_COLOR.bold().style("error:");
    let param_name = param.as_str();
    eprintln!("{error_label} parameter \"{param_name}\" not found in model");
}

fn handle_builtins_command(command: BuiltinsCommand) {
    let runtime = Runtime::new();
    match command {
        BuiltinsCommand::All { common } => {
            let print_utils_config = PrintUtilsConfig {
                sig_figs: common.sig_figs,
            };
            print_builtins::print_builtins_all(&runtime, print_utils_config);
        }
        BuiltinsCommand::Units {
            unit_name: Some(unit_name),
            common: _,
        } => print_builtins::search_builtins_units(&runtime, &unit_name),
        BuiltinsCommand::Units {
            unit_name: None,
            common: _,
        } => {
            print_builtins::print_builtins_units(&runtime);
        }
        BuiltinsCommand::Functions {
            function_name: Some(function_name),
            common: _,
        } => print_builtins::search_builtins_functions(&runtime, &function_name),
        BuiltinsCommand::Functions {
            function_name: None,
            common: _,
        } => print_builtins::print_builtins_functions(&runtime),
        BuiltinsCommand::Values {
            value_name: Some(value_name),
            common,
        } => {
            let print_utils_config = PrintUtilsConfig {
                sig_figs: common.sig_figs,
            };
            print_builtins::search_builtins_values(&runtime, &value_name, print_utils_config);
        }
        BuiltinsCommand::Values {
            value_name: None,
            common,
        } => {
            let print_utils_config = PrintUtilsConfig {
                sig_figs: common.sig_figs,
            };
            print_builtins::print_builtins_values(&runtime, print_utils_config);
        }
        BuiltinsCommand::Prefixes {
            prefix_name: Some(prefix_name),
            common: _,
        } => print_builtins::search_builtins_prefixes(&runtime, &prefix_name),
        BuiltinsCommand::Prefixes {
            prefix_name: None,
            common: _,
        } => {
            print_builtins::print_builtins_prefixes(&runtime);
        }
    }
}

/// Handles the `oneil check` subcommand.
///
/// Mirrors the LSP's open-file diagnostic flow: composes the instance
/// graph through the per-unit cache (no eval) and prints whatever
/// `RuntimeErrors` come back. Exits with status 1 when there are
/// diagnostics so the command is scriptable in CI.
#[expect(
    clippy::exit,
    reason = "scriptable diagnostic surface for CI; exit 1 on any diagnostic so wrappers can short-circuit without parsing stderr"
)]
fn handle_check_command(args: CheckArgs) {
    let CheckArgs { file, common } = args;

    let mut runtime = Runtime::new();
    let (_visited_paths, errors) = runtime.check_model(&file);

    for error in errors.to_vec() {
        print_error::print(error, common.dev_show_internal_errors);
    }

    if !errors.is_empty() {
        std::process::exit(1);
    }
}

fn handle_independent_command(args: IndependentArgs) {
    let IndependentArgs {
        file,
        recursive,
        debug: display_partial_results,
        common,
    } = args;

    let print_utils_config = PrintUtilsConfig {
        sig_figs: common.sig_figs,
    };

    let independent_print_config = IndependentPrintConfig {
        recursive,
        print_utils_config,
    };

    let mut runtime = Runtime::new();
    let (independents, errors) = runtime.get_independents(&file);

    let print_result = print_error::print_all(errors.to_vec(), common.dev_show_internal_errors);
    if print_result.saw_error_diagnostic() && !display_partial_results {
        return;
    }

    print_independents::print(&file, &independents, &independent_print_config);
}
