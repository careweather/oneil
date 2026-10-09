//! Units that plain parameter references inherit.
//!
//! A parameter whose value is only a reference to another parameter, such as
//! `P_l = P_t.r`, and that has no unit annotation takes the unit declared by
//! the parameter it references. The instance graph build writes that unit into
//! the reference's IR value, so design overlay checks, evaluation, and every
//! other consumer of the IR treat it like an explicit annotation.
//!
//! Only units with physical dimensions are inherited. A reference to a
//! dimensionless unit, such as `dB` or `%`, stays unannotated and evaluates to
//! a plain number, because `strip` and limits without units read a value in
//! its display unit.

use indexmap::IndexMap;
use oneil_ir as ir;
use oneil_shared::{
    paths::ModelPath,
    symbols::{ParameterName, ReferenceName},
};

use super::InstancedModel;

type ReferencePool = IndexMap<ModelPath, Box<InstancedModel>>;

/// Writes the inherited unit into every unannotated plain reference in
/// `root`'s subtree and in `pool`.
///
/// A scoped design overlay resolves its names from an ancestor of its host,
/// which this pass cannot reach, so its unit is written when the overlay is
/// applied, through [`fill_inherited_unit`].
pub(super) fn fill_inherited_units(root: &mut InstancedModel, pool: &mut ReferencePool) {
    let mut root_fills = Vec::new();
    collect_fills(root, &mut Vec::new(), pool, &mut root_fills);

    let pool_fills: Vec<(ModelPath, Vec<UnitFill>)> = pool
        .iter()
        .map(|(path, instance)| {
            let mut fills = Vec::new();
            collect_fills(instance, &mut Vec::new(), pool, &mut fills);
            (path.clone(), fills)
        })
        .collect();

    apply_fills(root, root_fills);
    for (path, fills) in pool_fills {
        let instance = pool
            .get_mut(&path)
            .expect("fills are collected from existing pool entries");
        apply_fills(instance, fills);
    }
}

/// Writes the unit that `value` inherits into it when `value` is an
/// unannotated plain reference, resolving the reference against `scope`.
pub(super) fn fill_inherited_unit(
    value: &mut ir::ParameterValue,
    scope: &InstancedModel,
    pool: &ReferencePool,
) {
    if let Some(unit) = inherited_unit(value, scope, pool) {
        set_missing_unit(value, unit);
    }
}

/// A unit to write into the parameter `parameter` of the instance reached by
/// following the submodel aliases in `path`.
struct UnitFill {
    path: Vec<ReferenceName>,
    parameter: ParameterName,
    unit: ir::CompositeUnit,
}

fn collect_fills(
    node: &InstancedModel,
    path: &mut Vec<ReferenceName>,
    pool: &ReferencePool,
    fills: &mut Vec<UnitFill>,
) {
    for (name, parameter) in node.parameters() {
        if has_scoped_anchor(parameter) {
            continue;
        }
        if let Some(unit) = inherited_unit(parameter.value(), node, pool) {
            fills.push(UnitFill {
                path: path.clone(),
                parameter: name.clone(),
                unit,
            });
        }
    }

    for (alias, submodel) in node.submodels() {
        path.push(alias.clone());
        collect_fills(&submodel.instance, path, pool, fills);
        path.pop();
    }
}

fn apply_fills(root: &mut InstancedModel, fills: Vec<UnitFill>) {
    for fill in fills {
        let node = fill.path.iter().fold(&mut *root, |node, alias| {
            node.submodels_mut()
                .get_mut(alias)
                .map(|submodel| submodel.instance.as_mut())
                .expect("fills are collected from existing submodels")
        });
        let parameter = node
            .parameters_mut()
            .get_mut(&fill.parameter)
            .expect("fills are collected from existing parameters");
        set_missing_unit(parameter.value_mut(), fill.unit);
    }
}

fn set_missing_unit(value: &mut ir::ParameterValue, unit: ir::CompositeUnit) {
    if let ir::ParameterValue::Simple(_, unit_slot) = value
        && unit_slot.is_none()
    {
        *unit_slot = Some(unit);
    }
}

/// Returns the unit that `value` inherits when it is an unannotated plain
/// reference whose target, possibly through further plain references, declares
/// a unit with physical dimensions.
fn inherited_unit(
    value: &ir::ParameterValue,
    scope: &InstancedModel,
    pool: &ReferencePool,
) -> Option<ir::CompositeUnit> {
    let mut value = value;
    let mut scope = scope;
    let mut visited: Vec<(&InstancedModel, &ParameterName)> = Vec::new();

    loop {
        let (target_scope, target_name) = reference_target(value, scope, pool)?;

        // References that form a cycle have no unit to inherit.
        let is_revisit = visited
            .iter()
            .any(|(model, name)| std::ptr::eq(*model, target_scope) && *name == target_name);
        if is_revisit {
            return None;
        }
        visited.push((target_scope, target_name));

        let target = target_scope.parameters().get(target_name)?;
        if let ir::ParameterValue::Simple(_, Some(unit))
        | ir::ParameterValue::Piecewise(_, Some(unit)) = target.value()
        {
            return (!unit.dimension().is_dimensionless()).then(|| unit.clone());
        }
        if has_scoped_anchor(target) {
            return None;
        }

        value = target.value();
        scope = target_scope;
    }
}

/// Returns the instance and parameter name that `value` refers to when it is
/// an unannotated plain reference resolved against `scope`.
fn reference_target<'a>(
    value: &'a ir::ParameterValue,
    scope: &'a InstancedModel,
    pool: &'a ReferencePool,
) -> Option<(&'a InstancedModel, &'a ParameterName)> {
    let ir::ParameterValue::Simple(expr, None) = value else {
        return None;
    };
    let ir::Expr::Variable { variable, .. } = expr.as_ref() else {
        return None;
    };

    match variable {
        ir::Variable::Parameter { parameter_name, .. } => Some((scope, parameter_name)),
        ir::Variable::External {
            reference_name,
            parameter_name,
            ..
        } => Some((
            referenced_instance(scope, reference_name, pool)?,
            parameter_name,
        )),
        ir::Variable::Builtin { ident, .. } => scope
            .parameters()
            .get_key_value(&ParameterName::from(ident.as_str()))
            .map(|(name, _)| (scope, name)),
    }
}

/// Returns the instance that `reference_name` names on `host`, through a
/// `reference`, a `submodel`, or a `with` alias.
fn referenced_instance<'a>(
    host: &'a InstancedModel,
    reference_name: &ReferenceName,
    pool: &'a ReferencePool,
) -> Option<&'a InstancedModel> {
    let direct = |name: &ReferenceName| {
        if let Some(reference) = host.references().get(name) {
            return pool.get(&reference.path).map(AsRef::as_ref);
        }
        host.submodels()
            .get(name)
            .map(|submodel| submodel.instance.as_ref())
    };

    if let Some(instance) = direct(reference_name) {
        return Some(instance);
    }

    // An alias path starts at a submodel or reference of the host and then
    // descends through submodels.
    let alias = host.aliases().get(reference_name)?;
    let (first, rest) = alias.alias_path.segments().split_first()?;
    rest.iter().try_fold(direct(first)?, |node, segment| {
        node.submodels()
            .get(segment)
            .map(|submodel| submodel.instance.as_ref())
    })
}

/// Returns whether `parameter` is a design overlay whose names resolve from an
/// ancestor of its host instead of from the host itself.
fn has_scoped_anchor(parameter: &ir::Parameter) -> bool {
    parameter
        .design_provenance()
        .is_some_and(|provenance| !provenance.anchor_path.is_self())
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use oneil_ir::{
        self as ir,
        test_helpers::{
            expr::{binary, builtin_var, external_var, lit_number, param_var},
            parameter::build_parameter_from_expr,
        },
    };
    use oneil_output::{Dimension, DimensionMap};
    use oneil_shared::{
        span::Span,
        symbols::{ParameterName, ReferenceName, SubmodelName},
    };

    use super::{ReferencePool, fill_inherited_units};
    use crate::{
        instance::{InstancedModel, ReferenceImport, SubmodelImport},
        test::test_model_path,
    };

    fn unit(name: &str, dimension: DimensionMap) -> ir::CompositeUnit {
        ir::CompositeUnit::new(
            Vec::new(),
            ir::DisplayCompositeUnit::BaseUnit(ir::DisplayUnit::new(name.to_string(), 1.0)),
            Span::synthetic(),
            dimension,
        )
    }

    fn distance() -> DimensionMap {
        std::iter::once((Dimension::Distance, 1.0)).collect()
    }

    fn kilometers() -> ir::CompositeUnit {
        unit("km", distance())
    }

    fn parameter(name: &str, expr: ir::Expr, unit: Option<ir::CompositeUnit>) -> ir::Parameter {
        build_parameter_from_expr(name, expr, unit, ir::Limits::default())
    }

    fn model(name: &str, parameters: impl IntoIterator<Item = ir::Parameter>) -> InstancedModel {
        let mut model = InstancedModel::empty_for(test_model_path(name));
        for parameter in parameters {
            model.add_parameter(parameter.name().clone(), parameter);
        }
        model
    }

    fn unit_of<'a>(model: &'a InstancedModel, name: &str) -> Option<&'a ir::CompositeUnit> {
        let parameter = model
            .get_parameter(&ParameterName::from(name))
            .expect("parameter should exist");
        match parameter.value() {
            ir::ParameterValue::Simple(_, unit) | ir::ParameterValue::Piecewise(_, unit) => {
                unit.as_ref()
            }
        }
    }

    fn fill(mut root: InstancedModel) -> InstancedModel {
        fill_inherited_units(&mut root, &mut ReferencePool::new());
        root
    }

    #[test]
    fn local_reference_inherits_unit() {
        let root = fill(model(
            "root",
            [
                parameter("L", lit_number(2.0), Some(kilometers())),
                parameter("L_r", param_var("L"), None),
            ],
        ));

        assert_eq!(unit_of(&root, "L_r"), Some(&kilometers()));
    }

    #[test]
    fn submodel_reference_inherits_unit() {
        let child = model(
            "child",
            [parameter("R", lit_number(3.0), Some(kilometers()))],
        );
        let mut root = model("root", [parameter("R_c", external_var("R", "c"), None)]);
        root.add_submodel(
            ReferenceName::from("c"),
            SubmodelImport {
                name: SubmodelName::new("child".to_string()),
                name_span: Span::synthetic(),
                alias: Some(ReferenceName::from("c")),
                alias_span: None,
                instance: Box::new(child),
            },
        );

        let root = fill(root);

        assert_eq!(unit_of(&root, "R_c"), Some(&kilometers()));
    }

    #[test]
    fn pooled_reference_and_pool_entry_inherit_units() {
        let child_path = test_model_path("child");
        let child = model(
            "child",
            [
                parameter("R", lit_number(3.0), Some(kilometers())),
                parameter("R_l", param_var("R"), None),
            ],
        );
        let mut pool: ReferencePool = IndexMap::new();
        pool.insert(child_path.clone(), Box::new(child));

        let mut root = model("root", [parameter("R_r", external_var("R", "r"), None)]);
        root.add_reference(
            ReferenceName::from("r"),
            ReferenceImport::new(
                ReferenceName::from("child"),
                Span::synthetic(),
                Some(ReferenceName::from("r")),
                None,
                child_path.clone(),
            ),
        );

        fill_inherited_units(&mut root, &mut pool);

        assert_eq!(unit_of(&root, "R_r"), Some(&kilometers()));
        assert_eq!(unit_of(&pool[&child_path], "R_l"), Some(&kilometers()));
    }

    #[test]
    fn reference_chain_inherits_unit() {
        let root = fill(model(
            "root",
            [
                parameter("a", param_var("b"), None),
                parameter("b", param_var("c"), None),
                parameter("c", lit_number(1.0), Some(kilometers())),
            ],
        ));

        assert_eq!(unit_of(&root, "a"), Some(&kilometers()));
        assert_eq!(unit_of(&root, "b"), Some(&kilometers()));
    }

    #[test]
    fn dimensionless_reference_stays_unannotated() {
        let decibels = unit("dB", DimensionMap::dimensionless());
        let root = fill(model(
            "root",
            [
                parameter("G", lit_number(18.0), Some(decibels)),
                parameter("G_r", param_var("G"), None),
            ],
        ));

        assert_eq!(unit_of(&root, "G_r"), None);
    }

    #[test]
    fn calculation_stays_unannotated() {
        let root = fill(model(
            "root",
            [
                parameter("L", lit_number(2.0), Some(kilometers())),
                parameter(
                    "L_2",
                    binary(ir::BinaryOp::Mul, lit_number(2.0), param_var("L")),
                    None,
                ),
            ],
        ));

        assert_eq!(unit_of(&root, "L_2"), None);
    }

    #[test]
    fn builtin_shadowed_by_parameter_inherits_unit() {
        let root = fill(model(
            "root",
            [
                parameter("pi", lit_number(3.0), Some(kilometers())),
                parameter("L", builtin_var("pi"), None),
            ],
        ));

        assert_eq!(unit_of(&root, "L"), Some(&kilometers()));
    }

    #[test]
    fn annotated_reference_keeps_its_unit() {
        let meters = unit("m", distance());
        let root = fill(model(
            "root",
            [
                parameter("L", lit_number(2.0), Some(kilometers())),
                parameter("L_m", param_var("L"), Some(meters.clone())),
            ],
        ));

        assert_eq!(unit_of(&root, "L_m"), Some(&meters));
    }

    #[test]
    fn reference_cycle_stays_unannotated() {
        let root = fill(model(
            "root",
            [
                parameter("a", param_var("b"), None),
                parameter("b", param_var("a"), None),
            ],
        ));

        assert_eq!(unit_of(&root, "a"), None);
        assert_eq!(unit_of(&root, "b"), None);
    }
}
