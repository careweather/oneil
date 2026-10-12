//! Units that plain parameter references inherit.
//!
//! A parameter whose value is only a reference to another parameter, such as
//! `P_l = P_t.r`, and that has no unit annotation takes the unit of the
//! parameter it references. Composition writes that unit into the reference's
//! IR value after every design is applied, so validation, evaluation, and the
//! rendered view read it like an explicit annotation, and a design that
//! changes the referenced parameter's unit changes the inherited unit too. The
//! design overlay unit check runs while designs are still being applied, so it
//! reads inherited units through [`value_unit`] without writing them.
//!
//! Only units with physical dimensions are inherited. A reference to a
//! dimensionless unit, such as `dB` or `%`, stays unannotated and evaluates to
//! a plain number, because `strip` and limits without units read a value in
//! its display unit.

use indexmap::IndexMap;
use oneil_ir as ir;
use oneil_shared::{
    InstancePath,
    paths::ModelPath,
    symbols::{ParameterName, ReferenceName},
};

use super::{
    InstancedModel,
    graph::{AbsolutePath, host_at_in_view},
};

type ReferencePool = IndexMap<ModelPath, Box<InstancedModel>>;

/// The instances that references resolve in: the root model's tree and the
/// reference pool.
#[derive(Clone, Copy)]
pub(super) struct GraphView<'a> {
    pub root: &'a InstancedModel,
    pub pool: &'a ReferencePool,
}

/// The names that a parameter value resolves against: an instance and where
/// it lives, plus the parameters that a design being applied to it adds.
#[derive(Clone)]
pub(super) struct Scope<'a> {
    pub instance: &'a InstancedModel,
    pub path: AbsolutePath,
    pub additions: Option<&'a IndexMap<ParameterName, ir::Parameter>>,
}

impl<'a> Scope<'a> {
    /// The scope of `instance`, which lives at `path`, with no pending design
    /// additions.
    pub(super) const fn at(instance: &'a InstancedModel, path: AbsolutePath) -> Self {
        Self {
            instance,
            path,
            additions: None,
        }
    }

    fn parameter(&self, name: &ParameterName) -> Option<(&'a ParameterName, &'a ir::Parameter)> {
        self.additions
            .and_then(|additions| additions.get_key_value(name))
            .or_else(|| self.instance.parameters().get_key_value(name))
    }
}

/// Returns the unit of `value`: its annotation, or else the unit it inherits
/// as a plain reference resolved in `scope`.
pub(super) fn value_unit<'a>(
    value: &'a ir::ParameterValue,
    scope: &Scope<'a>,
    graph: GraphView<'a>,
) -> Option<&'a ir::CompositeUnit> {
    annotated_unit(value).or_else(|| inherited_unit(value, scope.clone(), graph))
}

/// Returns the scope that `parameter`, which lives in `host`, resolves its
/// value in: the anchor that a scoped design overlay recorded, or else `host`.
///
/// Returns `None` when the anchor lies above the root of `host`'s tree.
pub(super) fn value_scope<'a>(
    parameter: &ir::Parameter,
    host: &Scope<'a>,
    graph: GraphView<'a>,
) -> Option<Scope<'a>> {
    let anchor_path = parameter
        .design_provenance()
        .map(|provenance| &provenance.anchor_path)
        .filter(|anchor_path| !anchor_path.is_self());
    let Some(anchor_path) = anchor_path else {
        return Some(host.clone());
    };
    let path = host.path.resolve(anchor_path)?;
    let anchor = host_at_in_view(graph.root, graph.pool, &path)?;
    Some(Scope::at(anchor, path))
}

/// Writes the inherited unit into every unannotated plain reference in
/// `root`'s subtree and in `pool`.
pub(super) fn fill_inherited_units(root: &mut InstancedModel, pool: &mut ReferencePool) {
    let (root_fills, pool_fills) = {
        let graph = GraphView {
            root: &*root,
            pool: &*pool,
        };
        let root_fills = tree_fills(graph.root, AbsolutePath::Root(InstancePath::root()), graph);
        let pool_fills: Vec<(ModelPath, Vec<UnitFill>)> = graph
            .pool
            .iter()
            .map(|(model_path, instance)| {
                let path = AbsolutePath::Pool(model_path.clone(), InstancePath::root());
                (model_path.clone(), tree_fills(instance, path, graph))
            })
            .collect();
        (root_fills, pool_fills)
    };

    apply_fills(root, root_fills);
    for (model_path, fills) in pool_fills {
        let instance = pool
            .get_mut(&model_path)
            .expect("fills are collected from existing pool entries");
        apply_fills(instance, fills);
    }
}

/// A unit to write into the parameter `parameter` of the instance at `path`
/// within one tree of the graph.
struct UnitFill {
    path: InstancePath,
    parameter: ParameterName,
    unit: ir::CompositeUnit,
}

/// Returns the fills for the tree whose root `root` lives at `path`.
fn tree_fills<'a>(
    root: &'a InstancedModel,
    path: AbsolutePath,
    graph: GraphView<'a>,
) -> Vec<UnitFill> {
    let mut fills = Vec::new();
    collect_fills(root, path, graph, &mut fills);
    fills
}

/// Collects the fills for `node`, which lives at `path`, and its submodels.
fn collect_fills<'a>(
    node: &'a InstancedModel,
    path: AbsolutePath,
    graph: GraphView<'a>,
    fills: &mut Vec<UnitFill>,
) {
    let host = Scope::at(node, path);
    for (name, parameter) in node.parameters() {
        let unit = value_scope(parameter, &host, graph)
            .and_then(|scope| inherited_unit(parameter.value(), scope, graph));
        if let Some(unit) = unit {
            fills.push(UnitFill {
                path: host.path.host_path(),
                parameter: name.clone(),
                unit: unit.clone(),
            });
        }
    }

    for (alias, submodel) in node.submodels() {
        collect_fills(
            &submodel.instance,
            host.path.append(alias.clone()),
            graph,
            fills,
        );
    }
}

fn apply_fills(root: &mut InstancedModel, fills: Vec<UnitFill>) {
    for fill in fills {
        let node = fill.path.segments().iter().fold(&mut *root, |node, alias| {
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

/// Returns the unit written on `value`, if any.
pub(super) const fn annotated_unit(value: &ir::ParameterValue) -> Option<&ir::CompositeUnit> {
    match value {
        ir::ParameterValue::Simple(_, unit) | ir::ParameterValue::Piecewise(_, unit) => {
            unit.as_ref()
        }
    }
}

/// Returns the unit that `value` inherits when it is an unannotated plain
/// reference whose target, possibly through further plain references, has a
/// unit with physical dimensions.
///
/// The chain continues through a scoped design overlay in the anchor that the
/// overlay recorded.
fn inherited_unit<'a>(
    value: &'a ir::ParameterValue,
    scope: Scope<'a>,
    graph: GraphView<'a>,
) -> Option<&'a ir::CompositeUnit> {
    let mut value = value;
    let mut scope = scope;
    // A parameter is identified by its instance and its name, because every
    // instance of a model has parameters with the same names.
    let mut visited: Vec<(&InstancedModel, &ParameterName)> = Vec::new();

    loop {
        let (target_scope, target_name, target) = reference_target(value, &scope, graph.pool)?;

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

        value = target.value();
        scope = value_scope(target, &target_scope, graph)?;
    }
}

/// Returns the scope, name, and parameter that `value` refers to when it is
/// an unannotated plain reference resolved in `scope`.
fn reference_target<'a>(
    value: &'a ir::ParameterValue,
    scope: &Scope<'a>,
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
            Some((scope.clone(), name, parameter))
        }
        // A parameter that a design adds shadows the builtin with its name,
        // and variable classification turns the builtin into a reference to it.
        ir::Variable::Builtin { ident, .. } => {
            let (name, parameter) = scope.parameter(&ParameterName::from(ident.as_str()))?;
            Some((scope.clone(), name, parameter))
        }
        ir::Variable::External {
            reference_name,
            parameter_name,
            ..
        } => {
            let target_scope = referenced_scope(scope, reference_name, pool)?;
            let (name, parameter) = target_scope
                .instance
                .parameters()
                .get_key_value(parameter_name)?;
            Some((target_scope, name, parameter))
        }
    }
}

/// Returns the scope of the instance that `reference_name` names on the
/// instance of `host`, through a `reference`, a `submodel`, or a `with` alias.
fn referenced_scope<'a>(
    host: &Scope<'a>,
    reference_name: &ReferenceName,
    pool: &'a ReferencePool,
) -> Option<Scope<'a>> {
    let submodel_scope = |parent: &Scope<'a>, name: &ReferenceName| {
        let submodel = parent.instance.submodels().get(name)?;
        Some(Scope::at(
            &submodel.instance,
            parent.path.append(name.clone()),
        ))
    };
    let direct = |name: &ReferenceName| {
        let Some(reference) = host.instance.references().get(name) else {
            return submodel_scope(host, name);
        };
        let instance = pool.get(&reference.path)?;
        let path = AbsolutePath::Pool(reference.path.clone(), InstancePath::root());
        Some(Scope::at(instance, path))
    };

    if let Some(scope) = direct(reference_name) {
        return Some(scope);
    }

    // An alias path starts at a submodel or reference of the host and then
    // descends through submodels.
    let alias = host.instance.aliases().get(reference_name)?;
    let (first, rest) = alias.alias_path.segments().split_first()?;
    rest.iter().try_fold(direct(first)?, |scope, segment| {
        submodel_scope(&scope, segment)
    })
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
        InstancePath, RelativePath,
        span::Span,
        symbols::{ParameterName, ReferenceName, SubmodelName},
    };

    use super::{GraphView, ReferencePool, Scope, fill_inherited_units, value_unit};
    use crate::{
        instance::{InstancedModel, ReferenceImport, SubmodelImport, graph::AbsolutePath},
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

    /// A root with `L = 2 :km` and `R_c = R.c`, where a scoped design overlay
    /// anchored at the root sets the child's `R` to `L`.
    fn scoped_overlay_model() -> InstancedModel {
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
        root
    }

    #[test]
    fn scoped_overlay_and_reference_to_it_inherit_units() {
        let root = fill(scoped_overlay_model());

        let child = &root.submodels()[&ReferenceName::from("c")].instance;
        assert_eq!(unit_of(child, "R"), Some(&kilometers()));
        assert_eq!(unit_of(&root, "R_c"), Some(&kilometers()));
    }

    #[test]
    fn reference_through_unfilled_scoped_overlay_has_unit() {
        let root = scoped_overlay_model();
        let pool = ReferencePool::new();
        let graph = GraphView {
            root: &root,
            pool: &pool,
        };
        let scope = Scope::at(&root, AbsolutePath::Root(InstancePath::root()));
        let reference = root
            .get_parameter(&ParameterName::from("R_c"))
            .expect("parameter should exist");

        let unit = value_unit(reference.value(), &scope, graph);

        assert_eq!(unit, Some(&kilometers()));
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
            path: AbsolutePath::Root(InstancePath::root()),
            additions: Some(&additions),
        };
        let pool = ReferencePool::new();
        let graph = GraphView {
            root: &host,
            pool: &pool,
        };

        let unit = value_unit(&override_value, &scope, graph);

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
