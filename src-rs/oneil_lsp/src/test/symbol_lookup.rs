//! Tests for symbol lookup at byte offsets.

use oneil_runtime::{Runtime, output::ir};
use oneil_shared::symbols::ParameterName;

use crate::symbol_lookup::{ModelImportName, SymbolAtPosition, find_symbol_at_offset};

use super::util;

/// Loads a fixture and finds the symbol at the given byte offset.
///
/// # Panics
///
/// Panics if no symbol is found at the offset.
#[track_caller]
fn symbol_at(runtime: &mut Runtime, fixture: &str, offset: usize) -> SymbolAtPosition {
    let (model, design_info) = util::load_model_and_design(runtime, fixture);
    find_symbol_at_offset(model, design_info.as_ref(), offset).expect("symbol at offset")
}

#[test]
fn finds_parameter_definition_at_name_span() {
    let mut runtime = util::test_runtime();
    let model = util::load_model(&mut runtime, "physics/basic.on");
    let param = model
        .get_parameter(&ParameterName::from("m"))
        .expect("parameter m");
    let offset = param.name_span().start().offset;

    let symbol = symbol_at(&mut runtime, "physics/basic.on", offset);

    let SymbolAtPosition::ParameterDefinition { name, .. } = symbol else {
        panic!("expected ParameterDefinition, got {symbol:?}");
    };

    assert_eq!(name.as_str(), "m");
}

#[test]
fn finds_parameter_reference_in_expression() {
    let mut runtime = util::test_runtime();
    let model = util::load_model(&mut runtime, "physics/basic.on");
    let force = model
        .get_parameter(&ParameterName::from("f"))
        .expect("parameter f");

    let offset = util::variable_offset_in_parameter_expr(force.value(), |variable| {
        matches!(
            variable,
            ir::Variable::Parameter {
                parameter_name,
                ..
            } if parameter_name.as_str() == "m"
        )
    });
    let symbol = symbol_at(&mut runtime, "physics/basic.on", offset);

    let SymbolAtPosition::ParameterReference { name, .. } = symbol else {
        panic!("expected ParameterReference, got {symbol:?}");
    };

    assert_eq!(name.as_str(), "m");
}

#[test]
fn finds_external_parameter_reference() {
    let mut runtime = util::test_runtime();
    let model = util::load_model(&mut runtime, "cross_file/parent.on");
    let x_param = model
        .get_parameter(&ParameterName::from("x"))
        .expect("parameter x");

    let offset = util::variable_offset_in_parameter_expr(x_param.value(), |variable| {
        matches!(
            variable,
            ir::Variable::External {
                reference_name,
                parameter_name,
                ..
            } if reference_name.as_str() == "c" && parameter_name.as_str() == "base"
        )
    });
    let symbol = symbol_at(&mut runtime, "cross_file/parent.on", offset);

    let SymbolAtPosition::ExternalParameterReference {
        reference_name,
        parameter_name,
        ..
    } = symbol
    else {
        panic!("expected ExternalParameterReference, got {symbol:?}");
    };

    assert_eq!(reference_name.as_str(), "c");
    assert_eq!(parameter_name.as_str(), "base");
}

#[test]
fn finds_builtin_value_reference() {
    let mut runtime = util::test_runtime();
    let model = util::load_model(&mut runtime, "physics/basic.on");
    let factor = model
        .get_parameter(&ParameterName::from("k"))
        .expect("parameter k");

    let offset = util::variable_offset_in_parameter_expr(factor.value(), |variable| {
        matches!(
            variable,
            ir::Variable::Builtin { ident, .. }
                if ident.as_str() == "pi"
        )
    });
    let symbol = symbol_at(&mut runtime, "physics/basic.on", offset);

    let SymbolAtPosition::BuiltinValueReference { name, .. } = symbol else {
        panic!("expected BuiltinValueReference, got {symbol:?}");
    };

    assert_eq!(name.as_str(), "pi");
}

#[test]
fn finds_model_import_definition_at_submodel_name_span() {
    let mut runtime = util::test_runtime();
    let model = util::load_model(&mut runtime, "cross_file/parent.on");
    let submodel_imports = model.submodel_models();
    let submodel = submodel_imports.values().next().expect("submodel import");
    let offset = submodel.name_span().start().offset;

    let symbol = symbol_at(&mut runtime, "cross_file/parent.on", offset);

    let SymbolAtPosition::ModelImportDefinition { name, path, .. } = symbol else {
        panic!("expected ModelImportDefinition, got {symbol:?}");
    };

    let ModelImportName::Submodel(submodel_name) = name else {
        panic!("expected Submodel import name, got {name:?}");
    };

    assert_eq!(submodel_name.as_str(), "child");
    assert_eq!(path, util::fixture_path("cross_file/child.on"));
}

#[test]
fn finds_model_import_alias_at_submodel_alias_span() {
    let mut runtime = util::test_runtime();
    let model = util::load_model(&mut runtime, "cross_file/parent.on");
    let submodel_imports = model.submodel_models();
    let submodel = submodel_imports.values().next().expect("submodel import");
    let offset = submodel.alias_span().expect("alias span").start().offset;

    let symbol = symbol_at(&mut runtime, "cross_file/parent.on", offset);

    let SymbolAtPosition::ModelImportAlias { alias, .. } = symbol else {
        panic!("expected ModelImportAlias, got {symbol:?}");
    };

    assert_eq!(alias.as_str(), "c");
}

#[test]
fn finds_model_import_reference_in_expression() {
    let mut runtime = util::test_runtime();
    let model = util::load_model(&mut runtime, "cross_file/parent.on");
    let x_param = model
        .get_parameter(&ParameterName::from("x"))
        .expect("parameter x");

    let offset = util::import_reference_offset_in_parameter_expr(x_param.value(), |variable| {
        matches!(
            variable,
            ir::Variable::External {
                reference_name,
                parameter_name,
                ..
            } if reference_name.as_str() == "c" && parameter_name.as_str() == "base"
        )
    });
    let symbol = symbol_at(&mut runtime, "cross_file/parent.on", offset);

    let SymbolAtPosition::ModelImportReference { reference_name, .. } = symbol else {
        panic!("expected ModelImportReference, got {symbol:?}");
    };

    assert_eq!(reference_name.as_str(), "c");
}

#[test]
fn finds_python_import_at_import_path_span() {
    let mut runtime = util::test_runtime();
    let (offset, expected_path) = {
        let model = util::load_model(&mut runtime, "python/square_area.on");
        let python_imports = model.python_imports();
        let (path, python_import) = python_imports.iter().next().expect("python import");
        (
            python_import.import_path_span().start().offset,
            path.clone(),
        )
    };

    let symbol = symbol_at(&mut runtime, "python/square_area.on", offset);

    let SymbolAtPosition::PythonImport {
        path: symbol_path, ..
    } = symbol
    else {
        panic!("expected PythonImport, got {symbol:?}");
    };

    assert_eq!(
        expected_path,
        util::python_fixture_path("python/py_helpers.py")
    );
    assert_eq!(
        symbol_path,
        util::python_fixture_path("python/py_helpers.py")
    );
}

#[test]
fn finds_python_function_reference_in_expression() {
    let mut runtime = util::test_runtime();
    let offset = {
        let model = util::load_model(&mut runtime, "python/square_area.on");
        let area = model
            .get_parameter(&ParameterName::from("A"))
            .expect("parameter A");

        util::function_call_offset_in_parameter_expr(area.value(), |name| {
            matches!(
                name,
                ir::FunctionName::Imported {
                    name: function_name,
                    ..
                } if function_name.as_str() == "square_area"
            )
        })
    };
    let symbol = symbol_at(&mut runtime, "python/square_area.on", offset);

    let SymbolAtPosition::PythonFunctionReference {
        python_path, name, ..
    } = symbol
    else {
        panic!("expected PythonFunctionReference, got {symbol:?}");
    };

    assert_eq!(
        python_path,
        util::python_fixture_path("python/py_helpers.py")
    );
    assert_eq!(name.as_str(), "square_area");
}

#[test]
fn finds_builtin_function_reference_in_expression() {
    let mut runtime = util::test_runtime();
    let model = util::load_model(&mut runtime, "physics/basic.on");
    let magnitude = model
        .get_parameter(&ParameterName::from("mag"))
        .expect("parameter mag");

    let offset = util::function_call_offset_in_parameter_expr(magnitude.value(), |name| {
        matches!(
            name,
            ir::FunctionName::Builtin(function_name, ..) if function_name.as_str() == "abs"
        )
    });
    let symbol = symbol_at(&mut runtime, "physics/basic.on", offset);

    let SymbolAtPosition::BuiltinFunctionReference { name, .. } = symbol else {
        panic!("expected BuiltinFunctionReference, got {symbol:?}");
    };

    assert_eq!(name.as_str(), "abs");
}

#[test]
fn finds_model_import_definition_at_reference_name_span() {
    let mut runtime = util::test_runtime();
    let model = util::load_model(&mut runtime, "cross_file/reference_parent.on");
    let reference_imports = model.reference_models();
    let reference = reference_imports.values().next().expect("reference import");
    let offset = reference.name_span().start().offset;

    let symbol = symbol_at(&mut runtime, "cross_file/reference_parent.on", offset);

    let SymbolAtPosition::ModelImportDefinition { name, path, .. } = symbol else {
        panic!("expected ModelImportDefinition, got {symbol:?}");
    };

    let ModelImportName::Reference(reference_name) = name else {
        panic!("expected Reference import name, got {name:?}");
    };

    assert_eq!(reference_name.as_str(), "child");
    assert_eq!(path, util::fixture_path("cross_file/child.on"));
}

#[test]
fn finds_design_target_in_design_file() {
    let mut runtime = util::test_runtime();
    let offset = {
        let design_info = util::load_design(&mut runtime, "design/augment.one");
        let design = design_info.design_export.expect("design export");
        design
            .target_model()
            .expect("design target")
            .1
            .start()
            .offset
    };

    let symbol = symbol_at(&mut runtime, "design/augment.one", offset);

    let SymbolAtPosition::DesignTarget { path, .. } = symbol else {
        panic!("expected DesignTarget, got {symbol:?}");
    };

    assert_eq!(path, util::fixture_path("design/target.on"));
}

#[test]
fn finds_design_parameter_addition_in_design_file() {
    let mut runtime = util::test_runtime();
    let offset = {
        let design_info = util::load_design(&mut runtime, "design/augment.one");
        let design = design_info.design_export.expect("design export");
        design
            .parameter_additions()
            .find(|param| param.name().as_str() == "extra")
            .expect("added parameter extra")
            .name_span()
            .start()
            .offset
    };

    let symbol = symbol_at(&mut runtime, "design/augment.one", offset);

    let SymbolAtPosition::DesignParameterAddition { name, .. } = symbol else {
        panic!("expected DesignParameterAddition, got {symbol:?}");
    };

    assert_eq!(name.as_str(), "extra");
}

#[test]
fn finds_design_parameter_override_in_design_file() {
    let mut runtime = util::test_runtime();
    let offset = {
        let design_info = util::load_design(&mut runtime, "design/override.one");
        let design = design_info.design_export.expect("design export");
        design
            .parameter_overrides()
            .find(|(name, _)| name.as_str() == "base")
            .expect("override for base")
            .1
            .design_span
            .start()
            .offset
    };

    let symbol = symbol_at(&mut runtime, "design/override.one", offset);

    let SymbolAtPosition::DesignParameterOverride {
        name,
        instance_path,
        ..
    } = symbol
    else {
        panic!("expected DesignParameterOverride, got {symbol:?}");
    };

    assert_eq!(name.as_str(), "base");
    assert!(instance_path.is_none());
}

#[test]
fn finds_design_parameter_override_instance_path_in_design_file() {
    let mut runtime = util::test_runtime();
    let offset = {
        let design_info = util::load_design(&mut runtime, "design_scoped/far.one");
        let design = design_info.design_export.expect("design export");
        design
            .scoped_parameter_overrides()
            .find(|(_path, name, _)| name.as_str() == "d")
            .expect("scoped override for d")
            .2
            .instance_path_span
            .as_ref()
            .expect("instance path span")
            .start()
            .offset
    };

    let symbol = symbol_at(&mut runtime, "design_scoped/far.one", offset);

    let SymbolAtPosition::DesignParameterOverrideInstancePath { instance_path, .. } = symbol else {
        panic!("expected DesignParameterOverrideInstancePath, got {symbol:?}");
    };

    let segments: Vec<_> = instance_path
        .segments()
        .iter()
        .map(oneil_shared::symbols::ReferenceName::as_str)
        .collect();
    assert_eq!(segments, ["c"]);
}

#[test]
fn finds_apply_design_path_in_apply_declaration() {
    let mut runtime = util::test_runtime();
    let offset = {
        let design_info = util::load_design(&mut runtime, "design/apply_parent.on");
        design_info
            .applied_designs
            .first()
            .expect("applied design")
            .design_path_span
            .start()
            .offset
    };

    let symbol = symbol_at(&mut runtime, "design/apply_parent.on", offset);

    let SymbolAtPosition::ApplyDesignPath { path, .. } = symbol else {
        panic!("expected ApplyDesignPath, got {symbol:?}");
    };

    assert_eq!(
        path,
        util::design_fixture_path("design/override.one").to_model_path()
    );
}

#[test]
fn finds_apply_target_reference_in_apply_declaration() {
    let mut runtime = util::test_runtime();
    let offset = {
        let design_info = util::load_design(&mut runtime, "design_apply_multi/parent.on");
        design_info
            .applied_designs
            .first()
            .expect("applied design")
            .target_segments
            .iter()
            .find(|(reference_name, _)| reference_name.as_str() == "l")
            .expect("target segment l")
            .1
            .start()
            .offset
    };

    let symbol = symbol_at(&mut runtime, "design_apply_multi/parent.on", offset);

    let SymbolAtPosition::ApplyTargetReference { reference_name, .. } = symbol else {
        panic!("expected ApplyTargetReference, got {symbol:?}");
    };

    assert_eq!(reference_name.as_str(), "l");
}
