//! Printing of builtin units, functions, values, and prefixes for the `builtins` CLI command.

use oneil_runtime::{
    Runtime,
    output::value::Value,
};

use crate::{print_utils, stylesheet};

pub fn search_builtins_units(runtime: &Runtime, unit_name: &str) {
    let search_result = runtime
        .builtin_units_docs()
        .find(|(name, aliases)| *name == unit_name || aliases.contains(&unit_name));

    if let Some((name, aliases)) = search_result {
        print_builtin_unit(name, &aliases);
    } else {
        let msg = format!("No builtin unit found for \"{unit_name}\"");
        let msg = stylesheet::BUILTIN_NOT_FOUND.style(msg);
        println!("{msg}");
    }
}

pub fn search_builtins_functions(runtime: &Runtime, function_name: &str) {
    let search_result = runtime
        .builtin_functions_docs()
        .find(|(name, _)| *name == function_name);

    if let Some((name, (args, description))) = search_result {
        print_builtin_function(name, args, description);
    } else {
        let msg = format!("No builtin function found for \"{function_name}\"");
        let msg = stylesheet::BUILTIN_NOT_FOUND.style(msg);
        println!("{msg}");
    }
}

pub fn search_builtins_values(runtime: &Runtime, value_name: &str) {
    let search_result = runtime
        .builtin_values_docs()
        .find(|(name, _)| *name == value_name);

    if let Some((name, (description, value))) = search_result {
        print_builtin_value(name, description, &value);
    } else {
        let msg = format!("No builtin value found for \"{value_name}\"");
        let msg = stylesheet::BUILTIN_NOT_FOUND.style(msg);
        println!("{msg}");
    }
}

pub fn search_builtins_prefixes(runtime: &Runtime, prefix_name: &str) {
    let search_result = runtime
        .builtin_prefixes_docs()
        .find(|(name, _)| *name == prefix_name);

    if let Some((name, (description, value))) = search_result {
        print_builtin_prefix(name, description, value);
    } else {
        let msg = format!("No builtin prefix found for \"{prefix_name}\"");
        let msg = stylesheet::BUILTIN_NOT_FOUND.style(msg);
        println!("{msg}");
    }
}

pub fn print_builtins_all(runtime: &Runtime) {
    print_builtins_values(runtime);
    println!();
    print_builtins_prefixes(runtime);
    println!();
    print_builtins_units(runtime);
    println!();
    print_builtins_functions(runtime);
}

pub fn print_builtins_units(runtime: &Runtime) {
    let header = stylesheet::BUILTIN_SECTION_HEADER.style("Builtin Units:");
    println!("{header}");
    println!();

    for (name, aliases) in runtime.builtin_units_docs() {
        print_builtin_unit(name, &aliases);
    }
}

fn print_builtin_unit(name: &str, aliases: &[&str]) {
    let styled_name = stylesheet::BUILTIN_NAME.style(name);
    let aliases_str = aliases.join(", ");
    let styled_aliases = stylesheet::BUILTIN_ALIASES.style(aliases_str);
    println!("  {styled_name}: {styled_aliases}");
}

pub fn print_builtins_functions(runtime: &Runtime) {
    let header = stylesheet::BUILTIN_SECTION_HEADER.style("Builtin Functions:");
    println!("{header}");
    println!();

    for (name, (args, description)) in runtime.builtin_functions_docs() {
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

pub fn print_builtins_values(runtime: &Runtime) {
    let header = stylesheet::BUILTIN_SECTION_HEADER.style("Builtin Values:");
    println!("{header}");
    println!();

    for (name, (description, value)) in runtime.builtin_values_docs() {
        print_builtin_value(name, description, &value);
    }
}

fn print_builtin_value(name: &str, description: &str, value: &Value) {
    let styled_name = stylesheet::BUILTIN_NAME.style(name);
    print!("  {styled_name} = ");
    print_utils::print_value(value);
    println!();
    let styled_description = stylesheet::BUILTIN_DESCRIPTION.style(description);
    println!("    {styled_description}");
    println!();
}

pub fn print_builtins_prefixes(runtime: &Runtime) {
    let header = stylesheet::BUILTIN_SECTION_HEADER.style("Builtin Prefixes:");
    println!("{header}");
    println!();

    for (name, (description, value)) in runtime.builtin_prefixes_docs() {
        print_builtin_prefix(name, description, value);
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
