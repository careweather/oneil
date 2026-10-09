//! Units that plain parameter references inherit.
//!
//! A parameter whose value is only a reference to another parameter, such as
//! `P_l = P_t.r`, and that has no unit annotation takes the unit of the
//! parameter it references. Composition writes that unit into the reference's
//! IR value after every design is applied, so validation, evaluation, and the
//! rendered view read it like an explicit annotation, and a design that
//! changes the referenced parameter's unit changes the inherited unit too. The
//! design overlay unit check runs while designs are still being applied, so it
//! reads inherited units through [`parameter_unit`] and [`value_unit`] without
//! writing them.
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

/// The names that a parameter value resolves against: an instance, plus the
/// parameters that a design being applied to it adds.
#[derive(Clone, Copy)]
pub(super) struct Scope<'a> {
    pub instance: &'a InstancedModel,
    pub additions: Option<&'a IndexMap<ParameterName, ir::Parameter>>,
}

impl<'a> Scope<'a> {
    /// The scope of `instance` with no pending design additions.
    pub(super) const fn of(instance: &'a InstancedModel) -> Self {
        Self {
            instance,
            additions: None,
        }
    }

    fn parameter(self, name: &ParameterName) -> Option<(&'a ParameterName, &'a ir::Parameter)> {
        self.additions
            .and_then(|additions| additions.get_key_value(name))
            .or_else(|| self.instance.parameters().get_key_value(name))
    }
}

/// Returns the unit of `parameter`, which lives on the instance of `scope`:
/// its annotation, or else the unit it inherits as a plain reference.
///
/// A scoped design overlay resolves its names from an anchor above its host,
/// so only its annotation counts.
pub(super) fn parameter_unit<'a>(
    parameter: &'a ir::Parameter,
    scope: Scope<'a>,
    pool: &'a ReferencePool,
) -> Option<&'a ir::CompositeUnit> {
    if has_scoped_anchor(parameter) {
        return annotated_unit(parameter.value());
    }
    value_unit(parameter.value(), scope, pool)
}

/// Returns the unit of `value`: its annotation, or else the unit it inherits
/// as a plain reference resolved in `scope`.
pub(super) fn value_unit<'a>(
    value: &'a ir::ParameterValue,
    scope: Scope<'a>,
    pool: &'a ReferencePool,
) -> Option<&'a ir::CompositeUnit> {
    annotated_unit(value).or_else(|| inherited_unit(value, scope, pool))
}

/// Writes the inherited unit into every unannotated plain reference in
/// `root`'s subtree and in `pool`.
///
/// A chain of references stops at a scoped design overlay that has no unit
/// yet, because its anchor is only known from its host. The pass repeats until
/// it writes nothing, so a reference to such an overlay gets its unit in the
/// round after the overlay does.
pub(super) fn fill_inherited_units(root: &mut InstancedModel, pool: &mut ReferencePool) {
    loop {
        let root_fills = subtree_fills(root, pool);
        let pool_fills: Vec<(ModelPath, Vec<UnitFill>)> = pool
            .iter()
            .map(|(path, instance)| (path.clone(), subtree_fills(instance, pool)))
            .filter(|(_, fills)| !fills.is_empty())
            .collect();
        if root_fills.is_empty() && pool_fills.is_empty() {
            return;
        }

        apply_fills(root, root_fills);
        for (path, fills) in pool_fills {
            let instance = pool
                .get_mut(&path)
                .expect("fills are collected from existing pool entries");
            apply_fills(instance, fills);
        }
    }
}

/// A unit to write into the parameter `parameter` of the instance reached by
/// following the submodel aliases in `path`.
struct UnitFill {
    path: Vec<ReferenceName>,
    parameter: ParameterName,
    unit: ir::CompositeUnit,
}

fn subtree_fills(root: &InstancedModel, pool: &ReferencePool) -> Vec<UnitFill> {
    let mut fills = Vec::new();
    collect_fills(root, &mut Vec::new(), &mut Vec::new(), pool, &mut fills);
    fills
}

/// Collects the fills for `node` and its submodels. `ancestors` runs from the
/// root of the walk down to `node`'s parent, and `path` holds the submodel
/// aliases from the root of the walk to `node`.
fn collect_fills<'a>(
    node: &'a InstancedModel,
    ancestors: &mut Vec<&'a InstancedModel>,
    path: &mut Vec<ReferenceName>,
    pool: &ReferencePool,
    fills: &mut Vec<UnitFill>,
) {
    for (name, parameter) in node.parameters() {
        let Some(scope) = value_scope(parameter, node, ancestors) else {
            continue;
        };
        if let Some(unit) = inherited_unit(parameter.value(), Scope::of(scope), pool) {
            fills.push(UnitFill {
                path: path.clone(),
                parameter: name.clone(),
                unit: unit.clone(),
            });
        }
    }

    ancestors.push(node);
    for (alias, submodel) in node.submodels() {
        path.push(alias.clone());
        collect_fills(&submodel.instance, ancestors, path, pool, fills);
        path.pop();
    }
    ancestors.pop();
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
        if let ir::ParameterValue::Simple(_, unit_slot) = parameter.value_mut() {
            *unit_slot = Some(fill.unit);
        }
    }
}

/// Returns the instance that `parameter`'s value resolves its names in: the
/// design anchor for a scoped design overlay, or else `host`.
///
/// `ancestors` runs from the root of the walk down to `host`'s parent, so an
/// anchor above the root of the walk cannot be resolved.
fn value_scope<'a>(
    parameter: &ir::Parameter,
    host: &'a InstancedModel,
    ancestors: &[&'a InstancedModel],
) -> Option<&'a InstancedModel> {
    let Some(provenance) = parameter.design_provenance() else {
        return Some(host);
    };
    let anchor_path = &provenance.anchor_path;
    let base = match anchor_path.up {
        0 => host,
        up => *ancestors.get(ancestors.len().checked_sub(up)?)?,
    };
    anchor_path.down.iter().try_fold(base, |node, segment| {
        node.submodels()
            .get(segment)
            .map(|submodel| submodel.instance.as_ref())
    })
}

const fn annotated_unit(value: &ir::ParameterValue) -> Option<&ir::CompositeUnit> {
    match value {
        ir::ParameterValue::Simple(_, unit) | ir::ParameterValue::Piecewise(_, unit) => {
            unit.as_ref()
        }
    }
}

/// Returns the unit that `value` inherits when it is an unannotated plain
/// reference whose target, possibly through further plain references, has a
/// unit with physical dimensions.
fn inherited_unit<'a>(
    value: &'a ir::ParameterValue,
    scope: Scope<'a>,
    pool: &'a ReferencePool,
) -> Option<&'a ir::CompositeUnit> {
    let mut value = value;
    let mut scope = scope;
    // A parameter is identified by its instance and its name, because every
    // instance of a model has parameters with the same names.
    let mut visited: Vec<(&InstancedModel, &ParameterName)> = Vec::new();

    loop {
        let (target_scope, target_name, target) = reference_target(value, scope, pool)?;

        // References that form a cycle have no unit to inherit.
        let is_revisit = visited.iter().any(|(instance, name)| {
            std::ptr::eq(*instance, target_scope.instance) && *name == target_name
        });
        if is_revisit {
            return None;
        }
        visited.push((target_scope.instance, target_name));

        if let Some(unit) = annotated_unit(target.value()) {
            return (!unit.dimension().is_dimensionless()).then_some(unit);
        }
        if has_scoped_anchor(target) {
            return None;
        }

        value = target.value();
        scope = target_scope;
    }
}

/// Returns the scope, name, and parameter that `value` refers to when it is
/// an unannotated plain reference resolved in `scope`.
fn reference_target<'a>(
    value: &'a ir::ParameterValue,
    scope: Scope<'a>,
    pool: &'a ReferencePool,
) -> Option<(Scope<'a>, &'a ParameterName, &'a ir::Parameter)> {
    let ir::ParameterValue::Simple(expr, None) = value else {
        return None;
    };
    let ir::Expr::Variable { variable, .. } = expr.as_ref() else {
        return None;
    };

    match variable {
        ir::Variable::Parameter { parameter_name, .. } => {
            let (name, parameter) = scope.parameter(parameter_name)?;
            Some((scope, name, parameter))
        }
        // A parameter that a design adds shadows the builtin with its name,
        // and variable classification turns the builtin into a reference to it.
        ir::Variable::Builtin { ident, .. } => {
            let (name, parameter) = scope.parameter(&ParameterName::from(ident.as_str()))?;
            Some((scope, name, parameter))
        }
        ir::Variable::External {
            reference_name,
            parameter_name,
            ..
        } => {
            let instance = referenced_instance(scope.instance, reference_name, pool)?;
            let (name, parameter) = instance.parameters().get_key_value(parameter_name)?;
            Some((Scope::of(instance), name, parameter))
        }
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
        RelativePath,
        span::Span,
        symbols::{ParameterName, ReferenceName, SubmodelName},
    };

    use super::{ReferencePool, Scope, fill_inherited_units, value_unit};
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

    fn add_child(parent: &mut InstancedModel, alias: &str, child: InstancedModel) {
        parent.add_submodel(
            ReferenceName::from(alias),
            SubmodelImport {
                name: SubmodelName::new(alias.to_string()),
                name_span: Span::synthetic(),
                alias: Some(ReferenceName::from(alias)),
                alias_span: None,
                instance: Box::new(child),
            },
        );
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
        add_child(&mut root, "c", child);

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
    fn scoped_overlay_and_reference_to_it_inherit_units() {
        let provenance = ir::DesignProvenance {
            design_path: test_model_path("design"),
            is_addition: false,
            assignment_span: Span::synthetic(),
            anchor_path: RelativePath {
                up: 1,
                down: Vec::new(),
            },
            applied_via: None,
        };
        let overlay = parameter("R", param_var("L"), None).with_design_provenance(provenance);
        let mut root = model(
            "root",
            [
                parameter("L", lit_number(2.0), Some(kilometers())),
                parameter("R_c", external_var("R", "c"), None),
            ],
        );
        add_child(&mut root, "c", model("child", [overlay]));

        let root = fill(root);

        let child = &root.submodels()[&ReferenceName::from("c")].instance;
        assert_eq!(unit_of(child, "R"), Some(&kilometers()));
        assert_eq!(unit_of(&root, "R_c"), Some(&kilometers()));
    }

    #[test]
    fn reference_to_shadowed_builtin_inherits_unit() {
        let root = fill(model(
            "root",
            [
                parameter("pi", lit_number(3.0), Some(kilometers())),
                parameter("t", builtin_var("pi"), None),
            ],
        ));

        assert_eq!(unit_of(&root, "t"), Some(&kilometers()));
    }

    #[test]
    fn reference_to_design_addition_has_unit() {
        let host = model("root", []);
        let additions: IndexMap<ParameterName, ir::Parameter> = std::iter::once((
            ParameterName::from("d"),
            parameter("d", lit_number(5.0), Some(kilometers())),
        ))
        .collect();
        let override_value = ir::ParameterValue::Simple(Box::new(param_var("d")), None);
        let scope = Scope {
            instance: &host,
            additions: Some(&additions),
        };
        let pool = ReferencePool::new();

        let unit = value_unit(&override_value, scope, &pool);

        assert_eq!(unit, Some(&kilometers()));
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
