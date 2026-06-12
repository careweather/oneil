//! Intermediate Representation (IR) printing functionality for the Oneil CLI

use anstream::println;
use indexmap::IndexMap;
use oneil_frontend::{
    ApplyDesign, ModelDesignInfo, OverlayParameterValue, instance::design::Design,
};
use oneil_runtime::output::{
    ir,
    reference::{ModelTemplateReference, ReferenceImportReference, SubmodelImportReference},
};
use oneil_shared::{
    InstancePath,
    paths::PythonPath,
    symbols::{ParameterName, ReferenceName, TestIndex},
};

use crate::stylesheet::debug as dbg_style;

/// Which sections of the IR to include when printing.
#[derive(Clone, Debug, Default)]
pub enum IrSections {
    /// Show all sections.
    #[default]
    All,

    /// Show only the specified sections.
    Specified {
        python_imports: bool,
        submodels: bool,
        references: bool,
        parameters: bool,
        tests: bool,
    },
}

impl IrSections {
    /// Returns whether to show the Python imports section.
    #[must_use]
    pub const fn show_python_imports(&self) -> bool {
        match self {
            Self::All => true,
            Self::Specified { python_imports, .. } => *python_imports,
        }
    }

    /// Returns whether to show the submodels section.
    #[must_use]
    pub const fn show_submodels(&self) -> bool {
        match self {
            Self::All => true,
            Self::Specified { submodels, .. } => *submodels,
        }
    }

    /// Returns whether to show the references section.
    #[must_use]
    pub const fn show_references(&self) -> bool {
        match self {
            Self::All => true,
            Self::Specified { references, .. } => *references,
        }
    }

    /// Returns whether to show the parameters section.
    #[must_use]
    pub const fn show_parameters(&self) -> bool {
        match self {
            Self::All => true,
            Self::Specified { parameters, .. } => *parameters,
        }
    }

    /// Returns whether to show the tests section.
    #[must_use]
    pub const fn show_tests(&self) -> bool {
        match self {
            Self::All => true,
            Self::Specified { tests, .. } => *tests,
        }
    }
}

pub struct IrPrintConfig {
    pub recursive: bool,
    pub sections: IrSections,
    /// When false, parameter values/limits and test expressions are omitted.
    pub print_values: bool,
}

/// Prints the IR in a hierarchical tree format for debugging.
///
/// Collects all errors from the model IR by traversing the hierarchy. If there are errors, they
/// are printed. If there are no errors or `display_partial` is true, the IR is displayed.
/// When displaying, only the top-level model IR is shown unless `recursive` is true.
///
/// When `design_info_opt` is present, applied designs and resolved design-export content for the
/// entry file are printed alongside the lowered template IR.
pub fn print(
    ir_result: ModelTemplateReference<'_>,
    design_info_opt: Option<&ModelDesignInfo>,
    ir_print_config: &IrPrintConfig,
) {
    println!("{}", dbg_style::ROOT_HEADER.style("ModelCollection"));
    println!(
        "{} {}",
        dbg_style::TREE.style("└──"),
        dbg_style::SECTION.style("Models:")
    );
    let prefix = "└──";
    print_model(ir_result, design_info_opt, 1, prefix, ir_print_config);
}

/// Prints a single model with its components.
///
/// When `recursive` is true and `model_ref` is `Some`, also prints nested submodels and references.
/// Design metadata from `design_info_opt` is printed only for the entry model (when `Some`).
fn print_model(
    model_ref: ModelTemplateReference<'_>,
    design_info_opt: Option<&ModelDesignInfo>,
    indent: usize,
    prefix: &str,
    config: &IrPrintConfig,
) {
    println!(
        "{}  {} {} \"{}\"",
        "    ".repeat(indent),
        dbg_style::TREE.style(prefix),
        dbg_style::LABEL.style("Model:"),
        dbg_style::IDENTIFIER.style(model_ref.path().as_path().display())
    );

    let sections = &config.sections;
    let mut section_list: Vec<(&str, usize, SectionTag)> = Vec::new();

    if sections.show_python_imports() {
        let python_imports = model_ref.python_imports();
        let count = python_imports.len();
        if count > 0 {
            section_list.push(("Python imports", count, SectionTag::PythonImports));
        }
    }

    if sections.show_submodels() {
        let submodels = model_ref.submodel_models();
        if !submodels.is_empty() {
            section_list.push(("Submodels", submodels.len(), SectionTag::Submodels));
        }
    }

    if sections.show_parameters() {
        let parameters = model_ref.parameters();
        if !parameters.is_empty() {
            section_list.push(("Parameters", parameters.len(), SectionTag::Parameters));
        }
    }

    if sections.show_references() {
        let references = model_ref.reference_models();
        if !references.is_empty() {
            section_list.push(("References", references.len(), SectionTag::References));
        }
    }

    if sections.show_tests() {
        let tests = model_ref.tests();
        if !tests.is_empty() {
            section_list.push(("Tests", tests.len(), SectionTag::Tests));
        }
    }

    if let Some(design_info) = design_info_opt {
        push_design_sections(design_info, config, &mut section_list);
    }

    for (i, (section_name, count, tag)) in section_list.iter().enumerate() {
        let is_last = i == section_list.len() - 1;
        let section_prefix = if is_last { "└──" } else { "├──" };
        println!(
            "{}    {} {} {}:",
            "    ".repeat(indent),
            dbg_style::TREE.style(section_prefix),
            dbg_style::SECTION.style(section_name),
            dbg_style::COUNT.style(format!("({count})"))
        );

        match tag {
            SectionTag::PythonImports => {
                print_python_imports(model_ref.python_imports(), indent + 2);
            }
            SectionTag::Submodels => print_submodels(&model_ref.submodel_models(), indent + 2),
            SectionTag::Parameters => print_parameters(&model_ref.parameters(), indent + 2, config),
            SectionTag::References => print_references(&model_ref.reference_models(), indent + 2),
            SectionTag::Tests => print_tests(model_ref.tests(), indent + 2, config),
            SectionTag::AppliedDesigns => {
                if let Some(design_info) = design_info_opt {
                    print_applied_designs(&design_info.applied_designs, indent + 2);
                }
            }
            SectionTag::DesignExport => {
                if let Some(design_info) = design_info_opt
                    && let Some(design) = design_info.design_export.as_ref()
                {
                    print_design_export(design, indent + 2, config);
                }
            }
        }
    }

    if config.recursive {
        // Get all the references that have valid IR
        let refs_to_print: Vec<_> = model_ref
            .reference_models()
            .values()
            .filter_map(ReferenceImportReference::model)
            .collect();

        // Print the references
        for (i, nested_ref) in refs_to_print.iter().enumerate() {
            let is_last = i == refs_to_print.len() - 1;
            let nested_prefix = if is_last { "└──" } else { "├──" };
            print_model(*nested_ref, None, indent + 1, nested_prefix, config);
        }
    }
}

enum SectionTag {
    PythonImports,
    Submodels,
    Parameters,
    References,
    Tests,
    AppliedDesigns,
    DesignExport,
}

/// Appends design-related sections to `section_list` when the entry file carries design metadata.
fn push_design_sections(
    design_info: &ModelDesignInfo,
    config: &IrPrintConfig,
    section_list: &mut Vec<(&str, usize, SectionTag)>,
) {
    if !design_info.applied_designs.is_empty() {
        section_list.push((
            "Applied designs",
            design_info.applied_designs.len(),
            SectionTag::AppliedDesigns,
        ));
    }

    let Some(design) = design_info.design_export.as_ref() else {
        return;
    };

    let mut export_count = 0_usize;
    if design.target_model().is_some() {
        export_count += 1;
    }
    if config.sections.show_parameters() {
        export_count += design.parameter_overrides().count();
        export_count += design.scoped_parameter_overrides().count();
        export_count += design.parameter_additions().count();
    }
    if config.sections.show_tests() {
        export_count += design.test_additions().count();
    }

    if export_count > 0 {
        section_list.push(("Design export", export_count, SectionTag::DesignExport));
    }
}

/// Prints `apply <design> to <path>` declarations from the entry file.
fn print_applied_designs(applied_designs: &[ApplyDesign], indent: usize) {
    for (i, apply) in applied_designs.iter().enumerate() {
        let is_last = i == applied_designs.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        let target = format_instance_path(&apply.target);
        println!(
            "{}    {} {} \"{}\"{}",
            "    ".repeat(indent),
            dbg_style::TREE.style(prefix),
            dbg_style::LABEL.style("ApplyDesign:"),
            dbg_style::IDENTIFIER.style(apply.design_path.as_path().display()),
            dbg_style::LITERAL.style(format!(" to {target}"))
        );
    }
}

/// Prints resolved design-export content from a `.one` design file.
fn print_design_export(design: &Design, indent: usize, config: &IrPrintConfig) {
    let sections = &config.sections;
    let mut items: Vec<DesignExportItem<'_>> = Vec::new();

    if let Some((target, _)) = design.target_model() {
        items.push(DesignExportItem::Target(target));
    }

    if sections.show_parameters() {
        for (name, overlay) in design.parameter_overrides() {
            items.push(DesignExportItem::ParameterOverride { name, overlay });
        }
        for (path, name, overlay) in design.scoped_parameter_overrides() {
            items.push(DesignExportItem::ScopedOverride {
                path,
                name,
                overlay,
            });
        }
        for parameter in design.parameter_additions() {
            items.push(DesignExportItem::ParameterAddition(parameter));
        }
    }

    if sections.show_tests() {
        for test in design.test_additions() {
            items.push(DesignExportItem::TestAddition(test));
        }
    }

    for (i, item) in items.iter().enumerate() {
        let is_last = i == items.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        match item {
            DesignExportItem::Target(path) => {
                println!(
                    "{}    {} {} \"{}\"",
                    "    ".repeat(indent),
                    dbg_style::TREE.style(prefix),
                    dbg_style::LABEL.style("Design target:"),
                    dbg_style::IDENTIFIER.style(path.as_path().display())
                );
            }
            DesignExportItem::ParameterOverride { name, overlay } => {
                print_overlay_parameter(name, None, overlay, indent, prefix, config);
            }
            DesignExportItem::ScopedOverride {
                path,
                name,
                overlay,
            } => {
                print_overlay_parameter(name, Some(path), overlay, indent, prefix, config);
            }
            DesignExportItem::ParameterAddition(parameter) => {
                print_parameter(parameter.name(), parameter, indent, prefix, config);
            }
            DesignExportItem::TestAddition(test) => {
                println!(
                    "{}    {} {}:",
                    "    ".repeat(indent),
                    dbg_style::TREE.style(prefix),
                    dbg_style::LABEL.style("Test addition")
                );
                print_test(test, indent + 1, config);
            }
        }
    }
}

enum DesignExportItem<'design> {
    Target(&'design oneil_shared::paths::ModelPath),
    ParameterOverride {
        name: &'design ParameterName,
        overlay: &'design OverlayParameterValue,
    },
    ScopedOverride {
        path: &'design InstancePath,
        name: &'design ParameterName,
        overlay: &'design OverlayParameterValue,
    },
    ParameterAddition(&'design ir::Parameter),
    TestAddition(&'design ir::Test),
}

/// Formats an instance path as dot-separated reference segments.
fn format_instance_path(path: &InstancePath) -> String {
    path.segments()
        .iter()
        .map(oneil_shared::symbols::ReferenceName::as_str)
        .collect::<Vec<_>>()
        .join(".")
}

/// Prints a design parameter override or scoped override.
fn print_overlay_parameter(
    parameter_name: &ParameterName,
    instance_path: Option<&InstancePath>,
    overlay: &OverlayParameterValue,
    indent: usize,
    prefix: &str,
    config: &IrPrintConfig,
) {
    let scoped_prefix = instance_path
        .map(format_instance_path)
        .map_or_else(String::new, |path| format!("{path}."));
    let label = if instance_path.is_some() {
        "Scoped override:"
    } else {
        "Parameter override:"
    };

    println!(
        "{}    {} {} \"{}{}\"",
        "    ".repeat(indent),
        dbg_style::TREE.style(prefix),
        dbg_style::LABEL.style(label),
        dbg_style::IDENTIFIER.style(scoped_prefix),
        dbg_style::IDENTIFIER.style(parameter_name.as_str())
    );

    if !config.print_values {
        return;
    }

    let value_indent = indent + 1;
    println!(
        "{}    {} {}",
        "    ".repeat(value_indent),
        dbg_style::TREE.style("├──"),
        dbg_style::DETAIL.style("Value:")
    );
    print_parameter_value(&overlay.value, value_indent + 1);

    if let Some(limits) = overlay.limits_override.as_ref() {
        println!(
            "{}    {} {}",
            "    ".repeat(value_indent),
            dbg_style::TREE.style("└──"),
            dbg_style::DETAIL.style("Limits:")
        );
        print_limits(limits, value_indent + 1);
    }
}

/// Prints Python imports, showing the path and function names for each import.
fn print_python_imports(imports: &IndexMap<PythonPath, ir::PythonImport>, indent: usize) {
    for (i, (python_path, import)) in imports.iter().enumerate() {
        let is_last_import = i == imports.len() - 1;

        let prefix = if is_last_import {
            "└──"
        } else {
            "├──"
        };

        let functions = import.functions();
        let count = functions.len();

        println!(
            "{}    {} {} \"{}\" {}",
            "    ".repeat(indent),
            dbg_style::TREE.style(prefix),
            dbg_style::LABEL.style("Python import:"),
            dbg_style::IDENTIFIER.style(python_path.as_path().display()),
            dbg_style::COUNT.style(format!(
                "({} function{})",
                count,
                if count == 1 { "" } else { "s" }
            ))
        );

        let func_indent = indent + 1;

        for (j, name) in functions.iter().enumerate() {
            let is_last_func = j == functions.len() - 1;

            let func_prefix = if is_last_func {
                "└──"
            } else {
                "├──"
            };

            println!(
                "{}    {} {} \"{}\"",
                "    ".repeat(func_indent),
                dbg_style::TREE.style(func_prefix),
                dbg_style::DETAIL.style("function:"),
                dbg_style::IDENTIFIER.style(name.as_str())
            );
        }
    }
}

/// Prints submodels, showing the source-level model name behind each alias.
///
/// The map is keyed by alias (= reference name); the source-level model name
/// (e.g., the `foo` in `submodel foo as bar`) is taken from `SubmodelImport.name`.
fn print_submodels(
    submodels: &IndexMap<&ReferenceName, SubmodelImportReference<'_>>,
    indent: usize,
) {
    for (i, (alias, submodel)) in submodels.iter().enumerate() {
        let is_last = i == submodels.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        println!(
            "{}    {} {} \"{}\" -> \"{}\"",
            "    ".repeat(indent),
            dbg_style::TREE.style(prefix),
            dbg_style::LABEL.style("Submodel:"),
            dbg_style::IDENTIFIER.style(alias.as_str()),
            dbg_style::IDENTIFIER.style(submodel.name().as_str())
        );
    }
}

/// Prints submodels
fn print_references(
    references: &IndexMap<&ReferenceName, ReferenceImportReference<'_>>,
    indent: usize,
) {
    for (i, (identifier, reference)) in references.iter().enumerate() {
        let path = reference.path();

        let is_last = i == references.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        println!(
            "{}    {} {} \"{}\" -> \"{}\"",
            "    ".repeat(indent),
            dbg_style::TREE.style(prefix),
            dbg_style::LABEL.style("Reference:"),
            dbg_style::IDENTIFIER.style(identifier.as_str()),
            dbg_style::IDENTIFIER.style(path.as_path().display())
        );
    }
}

/// Prints parameters
fn print_parameters(
    parameters: &IndexMap<&ParameterName, &ir::Parameter>,
    indent: usize,
    config: &IrPrintConfig,
) {
    for (i, (parameter_name, parameter)) in parameters.iter().enumerate() {
        let is_last = i == parameters.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        print_parameter(parameter_name, parameter, indent, prefix, config);
    }
}

/// Prints a single parameter
fn print_parameter(
    parameter_name: &ParameterName,
    parameter: &ir::Parameter,
    indent: usize,
    prefix: &str,
    config: &IrPrintConfig,
) {
    println!(
        "{}    {} {} \"{}\"",
        "    ".repeat(indent),
        dbg_style::TREE.style(prefix),
        dbg_style::LABEL.style("Parameter:"),
        dbg_style::IDENTIFIER.style(parameter_name.as_str())
    );

    let indent = indent + 1;

    // Print dependencies
    let dependencies = parameter.dependencies();
    print_dependencies(indent, dependencies);

    if config.print_values {
        // Print value
        println!(
            "{}    {} {}",
            "    ".repeat(indent),
            dbg_style::TREE.style("├──"),
            dbg_style::DETAIL.style("Value:")
        );
        print_parameter_value(parameter.value(), indent + 1);

        // Print limits
        println!(
            "{}    {} {}",
            "    ".repeat(indent),
            dbg_style::TREE.style("├──"),
            dbg_style::DETAIL.style("Limits:")
        );
        print_limits(parameter.limits(), indent + 1);
    }

    // Print metadata
    if parameter.is_performance() {
        println!(
            "{}    {} {}",
            "    ".repeat(indent),
            dbg_style::TREE.style("├──"),
            dbg_style::DETAIL.style("Performance: true")
        );
    }

    println!(
        "{}    {} {} {:?}",
        "    ".repeat(indent),
        dbg_style::TREE.style("└──"),
        dbg_style::DETAIL.style("Trace Level:"),
        parameter.trace_level()
    );
}

fn print_dependencies(indent: usize, dependencies: &ir::Dependencies) {
    let builtin_deps = dependencies.builtin();
    let parameter_deps = dependencies.parameter();
    let external_deps = dependencies.external();

    let total_deps = builtin_deps.len() + parameter_deps.len() + external_deps.len();
    if total_deps > 0 {
        println!(
            "{}    {} {}",
            "    ".repeat(indent),
            dbg_style::TREE.style("├──"),
            dbg_style::DETAIL.style("Dependencies:")
        );
        let mut dep_index = 0;

        // Print builtin dependencies
        for ident in builtin_deps.keys() {
            dep_index += 1;
            let is_last = dep_index == total_deps;
            let dep_prefix = if is_last { "└──" } else { "├──" };
            println!(
                "{}        {} {} \"{}\"",
                "    ".repeat(indent),
                dbg_style::TREE.style(dep_prefix),
                dbg_style::DETAIL.style("Builtin:"),
                dbg_style::IDENTIFIER.style(ident.as_str())
            );
        }

        // Print parameter dependencies
        for param_name in parameter_deps.keys() {
            dep_index += 1;
            let is_last = dep_index == total_deps;
            let dep_prefix = if is_last { "└──" } else { "├──" };
            println!(
                "{}        {} {} \"{}\"",
                "    ".repeat(indent),
                dbg_style::TREE.style(dep_prefix),
                dbg_style::DETAIL.style("Parameter:"),
                dbg_style::IDENTIFIER.style(param_name.as_str())
            );
        }

        // Print external dependencies
        for (ref_name, param_name) in external_deps.keys() {
            dep_index += 1;
            let is_last = dep_index == total_deps;
            let dep_prefix = if is_last { "└──" } else { "├──" };
            println!(
                "{}        {} {} \"{}\".\"{}\"",
                "    ".repeat(indent),
                dbg_style::TREE.style(dep_prefix),
                dbg_style::DETAIL.style("External:"),
                dbg_style::IDENTIFIER.style(param_name.as_str()),
                dbg_style::IDENTIFIER.style(ref_name.as_str()),
            );
        }
    }
}

/// Prints a parameter value
fn print_parameter_value(value: &ir::ParameterValue, indent: usize) {
    match value {
        ir::ParameterValue::Simple(expr, unit) => {
            println!(
                "{}    {} {} {}",
                "    ".repeat(indent),
                dbg_style::TREE.style("├──"),
                dbg_style::DETAIL.style("Type:"),
                dbg_style::LITERAL.style("Simple")
            );
            println!(
                "{}    {} {}",
                "    ".repeat(indent),
                dbg_style::TREE.style("├──"),
                dbg_style::DETAIL.style("Expression:")
            );
            print_expression(expr, indent + 1);
            if let Some(unit) = unit {
                println!(
                    "{}    {} {}",
                    "    ".repeat(indent),
                    dbg_style::TREE.style("└──"),
                    dbg_style::DETAIL.style("Unit:")
                );
                print_unit(unit, indent + 1);
            }
        }
        ir::ParameterValue::Piecewise(exprs, unit) => {
            println!(
                "{}    {} {} {}",
                "    ".repeat(indent),
                dbg_style::TREE.style("├──"),
                dbg_style::DETAIL.style("Type:"),
                dbg_style::LITERAL.style("Piecewise")
            );
            for (i, piecewise_expr) in exprs.iter().enumerate() {
                let is_last = i == exprs.len() - 1 && unit.is_none();
                let prefix = if is_last { "└──" } else { "├──" };
                println!(
                    "{}    {} {} {}:",
                    "    ".repeat(indent),
                    dbg_style::TREE.style(prefix),
                    dbg_style::DETAIL.style("Piece"),
                    dbg_style::COUNT.style(i + 1)
                );
                println!(
                    "{}        {} {}",
                    "    ".repeat(indent),
                    dbg_style::TREE.style("├──"),
                    dbg_style::DETAIL.style("Expression:")
                );
                print_expression(piecewise_expr.expr(), indent + 2);
                println!(
                    "{}        {} {}",
                    "    ".repeat(indent),
                    dbg_style::TREE.style("└──"),
                    dbg_style::DETAIL.style("Condition:")
                );
                print_expression(piecewise_expr.if_expr(), indent + 2);
            }
            if let Some(unit) = unit {
                println!(
                    "{}    {} {}",
                    "    ".repeat(indent),
                    dbg_style::TREE.style("└──"),
                    dbg_style::DETAIL.style("Unit:")
                );
                print_unit(unit, indent + 2);
            }
        }
    }
}

/// Prints limits
fn print_limits(limits: &ir::Limits, indent: usize) {
    match limits {
        ir::Limits::Default => {
            println!(
                "{}    {} {} {}",
                "    ".repeat(indent),
                dbg_style::TREE.style("└──"),
                dbg_style::DETAIL.style("Type:"),
                dbg_style::LITERAL.style("Default")
            );
        }
        ir::Limits::Continuous {
            min,
            max,
            limit_expr_span: _,
        } => {
            println!(
                "{}    {} {} {}",
                "    ".repeat(indent),
                dbg_style::TREE.style("├──"),
                dbg_style::DETAIL.style("Type:"),
                dbg_style::LITERAL.style("Continuous")
            );
            println!(
                "{}    {} {}",
                "    ".repeat(indent),
                dbg_style::TREE.style("├──"),
                dbg_style::DETAIL.style("Min:")
            );
            print_expression(min, indent + 1);
            println!(
                "{}    {} {}",
                "    ".repeat(indent),
                dbg_style::TREE.style("└──"),
                dbg_style::DETAIL.style("Max:")
            );
            print_expression(max, indent + 1);
        }
        ir::Limits::Discrete {
            values,
            limit_expr_span: _,
        } => {
            println!(
                "{}    {} {} {}",
                "    ".repeat(indent),
                dbg_style::TREE.style("├──"),
                dbg_style::DETAIL.style("Type:"),
                dbg_style::LITERAL.style("Discrete")
            );
            for (i, value) in values.iter().enumerate() {
                let is_last = i == values.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };
                println!(
                    "{}    {} {} {}:",
                    "    ".repeat(indent),
                    dbg_style::TREE.style(prefix),
                    dbg_style::DETAIL.style("Value"),
                    dbg_style::COUNT.style(i + 1)
                );
                print_expression(value, indent + 1);
            }
        }
    }
}

/// Prints an expression
fn print_expression(expr: &ir::Expr, indent: usize) {
    match expr {
        ir::Expr::BinaryOp {
            span: _,
            op,
            left,
            right,
        } => {
            println!("{}    ├── BinaryOp: {:?}", "    ".repeat(indent), op);
            print_expression(left, indent + 1);
            print_expression(right, indent + 1);
        }
        ir::Expr::Fallback {
            span: _,
            left,
            right,
        } => {
            println!("{}    ├── Fallback", "    ".repeat(indent));
            print_expression(left, indent + 1);
            print_expression(right, indent + 1);
        }
        ir::Expr::UnaryOp { span: _, op, expr } => {
            println!("{}    ├── UnaryOp: {:?}", "    ".repeat(indent), op);
            print_expression(expr, indent + 1);
        }
        ir::Expr::FunctionCall {
            span: _,
            name_span: _,
            name,
            args,
        } => {
            match name {
                ir::FunctionName::Builtin(name, _) => {
                    println!(
                        "{}    ├── FunctionCall (builtin): \"{}\"",
                        "    ".repeat(indent),
                        name.as_str()
                    );
                }
                ir::FunctionName::Imported {
                    name, python_path, ..
                } => {
                    println!(
                        "{}    ├── FunctionCall (imported): \"{}\" from \"{}\"",
                        "    ".repeat(indent),
                        name.as_str(),
                        python_path.as_path().display()
                    );
                }
            }

            for (i, arg) in args.iter().enumerate() {
                let is_last = i == args.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };
                println!("{}        {}Arg {}:", "    ".repeat(indent), prefix, i + 1);
                print_expression(arg, indent + 2);
            }
        }
        ir::Expr::Variable { span: _, variable } => {
            print_variable(variable, indent);
        }
        ir::Expr::Literal { span: _, value } => {
            print_literal(value, indent);
        }
        ir::Expr::ComparisonOp {
            span: _,
            op,
            left,
            right,
            rest_chained,
        } => {
            println!("{}    ├── ComparisonOp: {:?}", "    ".repeat(indent), op);
            print_expression(left, indent + 1);
            print_expression(right, indent + 1);
            for (i, (op, expr)) in rest_chained.iter().enumerate() {
                let is_last = i == rest_chained.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };
                println!(
                    "{}        {}Chained: {:?}",
                    "    ".repeat(indent),
                    prefix,
                    op
                );
                print_expression(expr, indent + 2);
            }
        }
        ir::Expr::UnitCast { expr, unit, .. } => {
            println!("{}    ├── UnitCast", "    ".repeat(indent));
            print_expression(expr, indent + 1);
            print_unit(unit, indent + 1);
        }
    }
}

/// Prints a variable
fn print_variable(var: &ir::Variable, indent: usize) {
    match var {
        ir::Variable::Builtin { ident, .. } => {
            println!(
                "{}    ├── Builtin Variable: \"{}\"",
                "    ".repeat(indent),
                ident.as_str()
            );
        }
        ir::Variable::Parameter { parameter_name, .. } => {
            println!(
                "{}    ├── Parameter Variable: \"{}\"",
                "    ".repeat(indent),
                parameter_name.as_str()
            );
        }
        ir::Variable::External {
            reference_name,
            parameter_name,
            ..
        } => {
            println!(
                "{}    ├── External Variable: \"{}.{}\"",
                "    ".repeat(indent),
                parameter_name.as_str(),
                reference_name.as_str(),
            );
        }
    }
}

/// Prints a literal
fn print_literal(lit: &ir::Literal, indent: usize) {
    match lit {
        ir::Literal::Number(n) => {
            println!("{}    ├── Literal: {}", "    ".repeat(indent), n);
        }
        ir::Literal::String(s) => {
            println!("{}    ├── Literal: \"{}\"", "    ".repeat(indent), s);
        }
        ir::Literal::Boolean(b) => {
            println!("{}    ├── Literal: {}", "    ".repeat(indent), b);
        }
    }
}

/// Prints a unit
fn print_unit(unit: &ir::CompositeUnit, indent: usize) {
    println!("{}    └── Unit: {:?}", "    ".repeat(indent), unit);
}

/// Prints tests
fn print_tests(tests: &IndexMap<TestIndex, ir::Test>, indent: usize, config: &IrPrintConfig) {
    for (test_index, test) in tests {
        let test_index = test_index.into_usize();
        let is_last = test_index == tests.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        println!(
            "{}    {} {} {}:",
            "    ".repeat(indent),
            dbg_style::TREE.style(prefix),
            dbg_style::LABEL.style("Test"),
            dbg_style::COUNT.style(test_index + 1)
        );
        print_test(test, indent + 1, config);
    }
}

/// Prints a single test
fn print_test(test: &ir::Test, indent: usize, config: &IrPrintConfig) {
    if config.print_values {
        println!(
            "{}    {} {} {:?}",
            "    ".repeat(indent),
            dbg_style::TREE.style("├──"),
            dbg_style::DETAIL.style("Trace Level:"),
            test.trace_level()
        );
        println!(
            "{}    {} {}",
            "    ".repeat(indent),
            dbg_style::TREE.style("└──"),
            dbg_style::DETAIL.style("Test Expression:")
        );
        print_expression(test.expr(), indent + 1);
    } else {
        println!(
            "{}    {} {} {:?}",
            "    ".repeat(indent),
            dbg_style::TREE.style("└──"),
            dbg_style::DETAIL.style("Trace Level:"),
            test.trace_level()
        );
    }
}
