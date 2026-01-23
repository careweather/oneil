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
use oneil_eval::{
    EvalContext,
    builtin::{BuiltinFunction, std as oneil_std},
    output::{
        dependency::{DependencyTreeValue, ReferenceTreeValue},
        eval_result::EvalResult,
        tree::Tree,
    },
    value::Value,
};
use oneil_ir as ir;
use oneil_model_resolver::FileLoader;
use oneil_runner::{
    builtins::Builtins,
    file_parser::{self, LoadingError},
};

use crate::{
    command::{
        BuiltinsCommand, CliCommand, Commands, DevCommand, EvalArgs, IndependentArgs, TestArgs,
        TreeArgs,
    },
    print_independents::IndependentPrintConfig,
    print_model_result::{ModelPrintConfig, TestPrintConfig},
    print_tree::TreePrintConfig,
};

mod command;
mod convert_error;
mod print_ast;
mod print_error;
mod print_independents;
mod print_ir;
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
            oneil_lsp::run();
        }
        Commands::Dev { command } => handle_dev_command(command),
        Commands::Eval(args) => handle_eval_command(args),
        Commands::Test(args) => handle_test_command(args),
        Commands::Tree(args) => handle_tree_command(args),
        Commands::Builtins { command } => handle_builtins_command(command),
        Commands::Independent(args) => handle_independent_command(args),
    }
}

fn handle_dev_command(command: DevCommand) {
    match command {
        DevCommand::PrintAst {
            files,
            display_partial,
            print_debug,
        } => handle_print_ast(&files, display_partial, print_debug),
        DevCommand::PrintIr {
            file,
            display_partial,
            print_debug,
        } => handle_print_ir(&file, display_partial, print_debug),
        DevCommand::PrintModelResult {
            file,
            display_partial,
        } => {
            handle_print_model_result(&file, display_partial);
        }
    }
}

/// Handles the `dev print-ast` command.
fn handle_print_ast(files: &[PathBuf], display_partial: bool, print_debug: bool) {
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
fn handle_print_ir(file: &Path, display_partial: bool, print_debug: bool) {
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
fn handle_print_model_result(file: &Path, display_partial: bool) {
    let builtins = create_builtins();

    let Some(model_collection) = load_model_collection(file, &builtins, false) else {
        return;
    };

    let eval_context = oneil_eval::eval_model_collection(&model_collection, builtins.builtin_map);

    let model_result = eval_context
        .get_model_result(file)
        .expect("model should be evaluated");

    let errors = model_result.get_errors();

    for error in &errors {
        let error = convert_error::eval::convert(error);

        if let Some(error) = error {
            print_error::print(&error, false);
            eprintln!();
        }
    }

    if errors.is_empty() || display_partial {
        println!("{:?}", model_result);
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
    };

    let builtins = create_builtins();

    if watch {
        watch_model(&file, &builtins, &model_print_config);
    } else {
        let (eval_context, _watch_paths) = eval_model(&file, &builtins);

        if let Some(eval_context) = eval_context {
            let model_result = eval_context
                .get_model_result(&file)
                .expect("model should be evaluated");

            print_model_result(&model_result, &model_print_config);
        }
    }
}

fn watch_model<F: BuiltinFunction + Clone>(
    file: &Path,
    builtins: &Builtins<F>,
    model_print_config: &ModelPrintConfig,
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
    let mut watch_paths = HashSet::new();

    let (eval_context, new_watch_paths) = eval_model(file, builtins);

    if let Some(eval_context) = eval_context {
        let model_result = eval_context
            .get_model_result(file)
            .expect("model should be evaluated");

        print_model_result(&model_result, model_print_config);
    }

    let (add_paths, remove_paths) = find_watch_paths_difference(&watch_paths, &new_watch_paths);

    update_watcher(&mut watcher, &add_paths, &remove_paths);
    watch_paths = new_watch_paths;

    for event in rx {
        match event {
            Ok(event) => match event.kind {
                notify::EventKind::Modify(_) => {
                    clear_screen();

                    let (eval_context, new_watch_paths) = eval_model(file, builtins);

                    if let Some(eval_context) = eval_context {
                        let model_result = eval_context
                            .get_model_result(file)
                            .expect("model should be evaluated");

                        print_model_result(&model_result, model_print_config);
                    }

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

fn eval_model<F: BuiltinFunction + Clone>(
    file: &Path,
    builtins: &Builtins<F>,
) -> (Option<EvalContext<F>>, HashSet<PathBuf>) {
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

            let model_watch_paths = watch_paths_from_model_collection(&model_collection);
            let error_model_watch_paths = error_map.get_watch_paths();

            let watch_paths = model_watch_paths
                .into_iter()
                .chain(error_model_watch_paths)
                .collect();

            return (None, watch_paths);
        }
    };

    // TODO: remove this clone?
    let eval_context =
        oneil_eval::eval_model_collection(&model_collection, builtins.builtin_map.clone());

    let watch_paths = watch_paths_from_model_collection(&model_collection);

    (Some(eval_context), watch_paths)
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

fn print_model_result(model_result: &EvalResult, model_print_config: &ModelPrintConfig) {
    let errors = model_result.get_errors();

    for error in &errors {
        let error = convert_error::eval::convert(error);

        if let Some(error) = error {
            print_error::print(&error, false);
            eprintln!();
        }
    }

    if errors.is_empty() || model_print_config.display_partial_results {
        print_model_result::print_eval_result(model_result, model_print_config);
    }
}

fn find_watch_paths_difference<'a>(
    old_paths: &'a HashSet<PathBuf>,
    new_paths: &'a HashSet<PathBuf>,
) -> (HashSet<&'a PathBuf>, HashSet<&'a PathBuf>) {
    let add_paths = new_paths.difference(old_paths).collect();
    let remove_paths = old_paths.difference(new_paths).collect();
    (add_paths, remove_paths)
}

fn update_watcher(
    watcher: &mut notify::RecommendedWatcher,
    add_paths: &HashSet<&PathBuf>,
    remove_paths: &HashSet<&PathBuf>,
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

fn handle_test_command(args: TestArgs) {
    let TestArgs {
        file,
        recursive,
        no_header,
        no_test_report,
    } = args;

    let test_print_config = TestPrintConfig {
        no_header,
        no_test_report,
        recursive,
    };

    let builtins = create_builtins();
    let (eval_context, _watch_paths) = eval_model(&file, &builtins);

    if let Some(eval_context) = eval_context {
        let model_result = eval_context
            .get_model_result(&file)
            .expect("model should be evaluated");

        print_model_result::print_test_results(&model_result, &test_print_config);
    }
}

fn handle_tree_command(args: TreeArgs) {
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

    let builtins = create_builtins();
    let (eval_context, _watch_paths) = eval_model(&file, &builtins);

    let Some(eval_context) = eval_context else {
        return;
    };

    let (trees, errors) = if list_refs {
        let mut trees = Vec::new();
        let mut errors = Vec::new();

        for param in params {
            let (reference_tree, tree_errors) = eval_context.get_reference_tree(&file, &param);

            trees.push((param, reference_tree));
            errors.extend(tree_errors);
        }

        (TreeResults::ReferenceTrees(trees), errors)
    } else {
        let mut trees = Vec::new();
        let mut errors = Vec::new();

        for param in params {
            let (dependency_tree, tree_errors) = eval_context.get_dependency_tree(&file, &param);

            trees.push((param, dependency_tree));
            errors.extend(tree_errors);
        }

        (TreeResults::DependencyTrees(trees), errors)
    };

    for error in &errors {
        let error = convert_error::eval::convert(error);

        if let Some(error) = error {
            print_error::print(&error, false);
            eprintln!();
        }
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
    match command {
        None | Some(BuiltinsCommand::All) => print_builtins_all(),
        Some(BuiltinsCommand::Units {
            unit_name: Some(unit_name),
        }) => search_builtins_units(&unit_name),
        Some(BuiltinsCommand::Units { unit_name: None }) => print_builtins_units(),
        Some(BuiltinsCommand::Functions {
            function_name: Some(function_name),
        }) => search_builtins_functions(&function_name),
        Some(BuiltinsCommand::Functions {
            function_name: None,
        }) => print_builtins_functions(),
        Some(BuiltinsCommand::Values {
            value_name: Some(value_name),
        }) => search_builtins_values(&value_name),
        Some(BuiltinsCommand::Values { value_name: None }) => print_builtins_values(),
        Some(BuiltinsCommand::Prefixes {
            prefix_name: Some(prefix_name),
        }) => search_builtins_prefixes(&prefix_name),
        Some(BuiltinsCommand::Prefixes { prefix_name: None }) => print_builtins_prefixes(),
    }
}

fn search_builtins_units(unit_name: &str) {
    let docs = oneil_std::builtin_units_docs();
    let search_result = docs
        .into_iter()
        .find(|(name, aliases)| *name == unit_name || aliases.contains(&unit_name));

    if let Some((name, aliases)) = search_result {
        print_builtin_unit(name, &aliases);
    } else {
        let msg = format!("No builtin unit found for \"{unit_name}\"");
        let msg = stylesheet::BUILTIN_NOT_FOUND.style(msg);
        println!("{msg}");
    }
}

fn search_builtins_functions(function_name: &str) {
    let docs = oneil_std::builtin_functions_docs();
    let search_result = docs.into_iter().find(|(name, _)| *name == function_name);

    if let Some((name, (args, description))) = search_result {
        print_builtin_function(name, args, description);
    } else {
        let msg = format!("No builtin function found for \"{function_name}\"");
        let msg = stylesheet::BUILTIN_NOT_FOUND.style(msg);
        println!("{msg}");
    }
}

fn search_builtins_values(value_name: &str) {
    let docs = oneil_std::builtin_values_docs();
    let search_result = docs.into_iter().find(|(name, _)| *name == value_name);

    if let Some((name, (description, value))) = search_result {
        print_builtin_value(&name, &description, &value);
    } else {
        let msg = format!("No builtin value found for \"{value_name}\"");
        let msg = stylesheet::BUILTIN_NOT_FOUND.style(msg);
        println!("{msg}");
    }
}

fn search_builtins_prefixes(prefix_name: &str) {
    let docs = oneil_std::builtin_prefixes_docs();
    let search_result = docs.into_iter().find(|(name, _)| *name == prefix_name);

    if let Some((name, (description, value))) = search_result {
        print_builtin_prefix(&name, &description, value);
    } else {
        let msg = format!("No builtin prefix found for \"{prefix_name}\"");
        let msg = stylesheet::BUILTIN_NOT_FOUND.style(msg);
        println!("{msg}");
    }
}

fn print_builtins_all() {
    print_builtins_values();
    println!();
    print_builtins_prefixes();
    println!();
    print_builtins_units();
    println!();
    print_builtins_functions();
}

fn print_builtins_units() {
    let header = stylesheet::BUILTIN_SECTION_HEADER.style("Builtin Units:");
    println!("{header}");
    println!();

    let docs = oneil_std::builtin_units_docs();
    for (name, aliases) in docs {
        print_builtin_unit(name, &aliases);
    }
}

fn print_builtin_unit(name: &str, aliases: &[&str]) {
    let styled_name = stylesheet::BUILTIN_NAME.style(name);
    let aliases_str = aliases.join(", ");
    let styled_aliases = stylesheet::BUILTIN_ALIASES.style(aliases_str);
    println!("  {styled_name}: {styled_aliases}");
}

fn print_builtins_functions() {
    let header = stylesheet::BUILTIN_SECTION_HEADER.style("Builtin Functions:");
    println!("{header}");
    println!();

    let docs = oneil_std::builtin_functions_docs();
    for (name, (args, description)) in docs {
        print_builtin_function(name, args, description);
    }
}

fn print_builtin_function(name: &str, args: &[&str], description: &str) {
    let styled_name = stylesheet::BUILTIN_NAME.style(name);
    let args_str = args.join(", ");
    let styled_args = stylesheet::BUILTIN_FUNCTION_ARGS.style(args_str);
    let description = description.replace('\n', "\n    ");
    let styled_description = stylesheet::BUILTIN_DESCRIPTION.style(description);

    println!("  {styled_name}({styled_args})");
    println!();
    println!("    {styled_description}");
    println!();
}

fn print_builtins_values() {
    let header = stylesheet::BUILTIN_SECTION_HEADER.style("Builtin Values:");
    println!("{header}");
    println!();

    let docs = oneil_std::builtin_values_docs();
    for (name, (description, value)) in docs {
        print_builtin_value(&name, &description, &value);
    }
}

fn print_builtin_value(name: &str, description: &str, value: &Value) {
    let styled_name = stylesheet::BUILTIN_NAME.style(name);
    print!("  {styled_name} = ");
    crate::print_utils::print_value(value);
    println!();
    let styled_description = stylesheet::BUILTIN_DESCRIPTION.style(description);
    println!("    {styled_description}");
    println!();
}

fn print_builtins_prefixes() {
    let header = stylesheet::BUILTIN_SECTION_HEADER.style("Builtin Prefixes:");
    println!("{header}");
    println!();

    let docs = oneil_std::builtin_prefixes_docs();
    for (name, (description, value)) in docs {
        print_builtin_prefix(&name, &description, value);
    }
}

fn print_builtin_prefix(name: &str, description: &str, value: f64) {
    let styled_name = stylesheet::BUILTIN_NAME.style(name);
    let description = format!("({description})");
    let padded_description = format!("{description: <8}");
    let styled_description = stylesheet::BUILTIN_DESCRIPTION.style(padded_description);
    let styled_value = stylesheet::BUILTIN_VALUE.style(format!("{value:e}"));
    println!("  {styled_name} {styled_description} = {styled_value}");
}

fn handle_independent_command(args: IndependentArgs) {
    let IndependentArgs {
        file,
        recursive,
        values: print_values,
    } = args;

    let independent_print_config = IndependentPrintConfig {
        print_values,
        recursive,
    };

    let builtins = create_builtins();
    let (eval_context, _watch_paths) = eval_model(&file, &builtins);

    if let Some(eval_context) = eval_context {
        let model_result = eval_context
            .get_model_result(&file)
            .expect("model should be evaluated");

        print_independents::print(&model_result, &independent_print_config);
    }
}
