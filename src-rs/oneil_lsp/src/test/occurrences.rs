//! Tests for occurrence collection across loaded models.

use oneil_shared::{
    paths::ModelPath,
    span::Span,
    symbols::{ParameterName, ReferenceName},
};

use crate::occurrences::{Occurrence, SearchTarget, collect_occurrences};

use super::util;

fn count_in_model(occurrences: &[Occurrence], model_path: &ModelPath) -> usize {
    occurrences
        .iter()
        .filter(|occurrence| &occurrence.model_path == model_path)
        .count()
}

fn has_span(occurrences: &[Occurrence], model_path: &ModelPath, span: &Span) -> bool {
    occurrences
        .iter()
        .any(|occurrence| &occurrence.model_path == model_path && occurrence.span == *span)
}

#[test]
fn collects_local_parameter_definition_and_references() {
    let mut runtime = util::test_runtime();
    let model_path = util::fixture_path("physics/basic.on");
    let mass_name_span = {
        let model = util::load_model(&mut runtime, "physics/basic.on");
        model
            .get_parameter(&ParameterName::from("m"))
            .expect("parameter m")
            .name_span()
            .clone()
    };

    let target = SearchTarget::Parameter {
        model_path: model_path.clone(),
        name: ParameterName::from("m"),
    };
    let occurrences = collect_occurrences(&target, &runtime);

    assert_eq!(
        occurrences.len(),
        3,
        "parameter m should have a definition and two references, got {}",
        occurrences.len()
    );
    assert_eq!(count_in_model(&occurrences, &model_path), occurrences.len());
    assert!(has_span(&occurrences, &model_path, &mass_name_span));
}

#[test]
fn collects_external_parameter_occurrences_across_files() {
    let mut runtime = util::test_runtime();
    let child_path = util::fixture_path("cross_file/child.on");
    let parent_path = util::fixture_path("cross_file/parent.on");
    util::load_model(&mut runtime, "cross_file/parent.on");

    let base_name_span = {
        let child = util::load_model(&mut runtime, "cross_file/child.on");
        child
            .get_parameter(&ParameterName::from("base"))
            .expect("parameter base")
            .name_span()
            .clone()
    };

    let target = SearchTarget::Parameter {
        model_path: child_path.clone(),
        name: ParameterName::from("base"),
    };
    let occurrences = collect_occurrences(&target, &runtime);

    assert_eq!(occurrences.len(), 2);
    assert_eq!(
        count_in_model(&occurrences, &child_path),
        1,
        "expected definition occurrence in child.on"
    );
    assert_eq!(
        count_in_model(&occurrences, &parent_path),
        1,
        "expected external reference occurrence in parent.on"
    );
    assert!(has_span(&occurrences, &child_path, &base_name_span));
}

#[test]
fn collects_import_alias_definition_and_references() {
    let mut runtime = util::test_runtime();
    let parent_path = util::fixture_path("cross_file/parent.on");
    let alias_span = {
        let model = util::load_model(&mut runtime, "cross_file/parent.on");
        let submodel_imports = model.submodel_models();
        submodel_imports
            .values()
            .next()
            .expect("submodel import")
            .alias_span()
            .expect("alias span for submodel import")
            .clone()
    };

    let target = SearchTarget::ImportAlias {
        model_path: parent_path.clone(),
        name: ReferenceName::from("c"),
    };
    let occurrences = collect_occurrences(&target, &runtime);

    assert_eq!(occurrences.len(), 2);
    assert_eq!(
        count_in_model(&occurrences, &parent_path),
        2,
        "expected definition occurrence in parent.on"
    );
    assert!(has_span(&occurrences, &parent_path, &alias_span));
}

#[test]
fn collects_design_parameter_addition_occurrence() {
    let mut runtime = util::test_runtime();
    let target_path = util::fixture_path("design/target.on");
    let design_path = util::fixture_path("design/augment.one");
    util::load_model(&mut runtime, "design/augment.one");

    let extra_name_span = {
        let design_export = util::load_design(&mut runtime, "design/augment.one")
            .design_export
            .expect("design export");
        design_export
            .parameter_additions()
            .find(|param| param.name().as_str() == "extra")
            .expect("added parameter extra")
            .name_span()
            .clone()
    };

    let target = SearchTarget::Parameter {
        model_path: target_path,
        name: ParameterName::from("extra"),
    };
    let occurrences = collect_occurrences(&target, &runtime);

    assert_eq!(occurrences.len(), 1);
    assert!(has_span(&occurrences, &design_path, &extra_name_span));
}
