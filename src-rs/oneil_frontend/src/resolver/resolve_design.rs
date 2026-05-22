//! Resolution of design surface declarations: `design <model>`,
//! `apply <file> to <ref>(.<ref>)*`, and design parameter assignments
//! (`id[.<ref>] = expr`).
//!
//! `apply` resolves sibling **`.one`** files; `design <model>` resolves the
//! target as a sibling **`.on`** model.

use std::ops::Deref;

use indexmap::IndexSet;
use oneil_ast as ast;
use oneil_ir as ir;
use oneil_shared::{
    InstancePath,
    labels::{ParameterLabel, SectionLabel},
    paths::{DesignPath, ModelPath},
    span::Span,
    symbols::{ParameterName, ReferenceName},
};

use crate::{
    ExternalResolutionContext, ResolutionContext,
    context::ModelResult,
    instance::{
        ApplyDesign,
        design::{Design, OverlayParameterValue},
    },
    resolver::{resolve_parameter, resolve_trace_level::resolve_trace_level},
};

/// Adds resolved tests to the design export, if one exists.
///
/// This must be called AFTER tests have been resolved so that they can be
/// collected from the active model and added to the design export. Tests
/// from design files are applied to the design's target model, not evaluated
/// in the design file's own scope.
pub fn add_tests_to_design_export<E: ExternalResolutionContext>(
    resolution_context: &mut ResolutionContext<'_, E>,
) {
    // Get tests from the active model (accessed via the active model's tests)
    let tests = resolution_context.active_model().tests().clone();
    if tests.is_empty() {
        return;
    }

    // Add tests to the design export if one exists
    resolution_context.add_tests_to_design_export(tests);
}

/// Loads sibling models referenced by `apply` declarations so their IR exists
/// before resolution.
pub fn preload_design_files<E: ExternalResolutionContext>(
    model_path: &ModelPath,
    model_ast: &ast::Model,
    resolution_context: &mut ResolutionContext<'_, E>,
) {
    if let Some(target_path) = collect_design_target_path(model_path, model_ast) {
        super::load_model(&target_path, resolution_context);
    }
    for path in collect_apply_design_paths(model_path, model_ast) {
        super::load_model(&path.to_model_path(), resolution_context);
    }
}

/// Renders a path for inclusion in user-facing messages.
fn display_model_path(path: &std::path::Path) -> String {
    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
        return name.to_string();
    }
    path.display().to_string()
}

/// Returns `true` when two [`ModelPath`]s point at the same on-disk file.
fn same_model_path(a: &ModelPath, b: &ModelPath) -> bool {
    if a == b {
        return true;
    }
    match (a.as_path().canonicalize(), b.as_path().canonicalize()) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => false,
    }
}

/// Returns the path to the design target model if there's a `design <target>` declaration.
#[must_use]
pub fn collect_design_target_path(
    model_path: &ModelPath,
    model_ast: &ast::Model,
) -> Option<ModelPath> {
    for item in collect_design_surface(model_ast) {
        if let DesignSurfaceItem::Target(node) = item {
            let relative_path = node.get_target_relative_path();
            return Some(model_path.get_sibling_model_path(relative_path));
        }
    }
    None
}

/// Collects unique paths referenced by every `apply` declaration (including nested ones).
#[must_use]
pub fn collect_apply_design_paths(
    model_path: &ModelPath,
    model_ast: &ast::Model,
) -> IndexSet<DesignPath> {
    let mut out = IndexSet::new();
    for ad in iter_apply_designs(model_ast) {
        let relative_path = ad.get_design_relative_path();
        out.insert(model_path.get_sibling_design_path(relative_path));
    }
    out
}

/// Section header context for a design parameter line declared inside a `section` block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignSectionContext {
    /// Section label from the design file.
    pub label: SectionLabel,
    /// Optional section note from the design file.
    pub note: Option<ir::Note>,
}

/// A single design-related declaration in source order.
#[derive(Debug, Clone)]
pub enum DesignSurfaceItem<'a> {
    Target(&'a ast::DesignTargetNode),
    Apply(&'a ast::ApplyDesignNode),
    Parameter {
        node: &'a ast::DesignParameterNode,
        section: Option<DesignSectionContext>,
    },
}

/// Walks all declarations (top-level and within sections) producing a
/// [`DesignSurfaceItem`] for each design-related entry.
pub fn collect_design_surface(model_ast: &ast::Model) -> Vec<DesignSurfaceItem<'_>> {
    let mut items = Vec::new();

    for decl in model_ast.decls() {
        push_design_decl(decl, None, &mut items);
    }
    for section in model_ast.sections() {
        let ctx = DesignSectionContext {
            label: section.header().label().deref().clone(),
            note: section.note().map(|n| ir::Note::new(n.value().to_string())),
        };
        for decl in section.decls() {
            push_design_decl(decl, Some(ctx.clone()), &mut items);
        }
    }

    items
}

fn push_design_decl<'a>(
    decl: &'a ast::DeclNode,
    section: Option<DesignSectionContext>,
    items: &mut Vec<DesignSurfaceItem<'a>>,
) {
    match &**decl {
        ast::Decl::DesignTarget(n) => items.push(DesignSurfaceItem::Target(n)),
        ast::Decl::ApplyDesign(n) => items.push(DesignSurfaceItem::Apply(n)),
        ast::Decl::DesignParameter(n) => {
            items.push(DesignSurfaceItem::Parameter { node: n, section });
        }
        ast::Decl::Import(_)
        | ast::Decl::Submodel(_)
        | ast::Decl::Parameter(_)
        | ast::Decl::Test(_) => {}
    }
}

/// Returns the span of the parameter definition on `model_path`, or `fallback` when absent.
fn span_of_parameter_on_model<E: ExternalResolutionContext>(
    ctx: &ResolutionContext<'_, E>,
    model_path: &ModelPath,
    param: &ParameterName,
    fallback: Span,
) -> Span {
    match ctx.lookup_model(model_path) {
        ModelResult::Found(m) => m
            .get_parameter(param)
            .map_or(fallback, |p| p.span().clone()),
        ModelResult::HasError | ModelResult::NotFound => fallback,
    }
}

/// Returns the resolved limits from a design parameter line, when present.
///
/// Returns `Ok(None)` when the line has no limits. Returns `Err(())` after
/// recording resolution errors.
fn resolve_optional_design_limits<E: ExternalResolutionContext>(
    p: &ast::DesignParameterNode,
    name: &ParameterName,
    resolution_context: &mut ResolutionContext<'_, E>,
) -> Result<Option<ir::Limits>, ()> {
    let Some(limits_node) = p.limits() else {
        return Ok(None);
    };
    match resolve_parameter::resolve_limits(Some(limits_node), resolution_context) {
        Ok(limits) => Ok(Some(limits)),
        Err(errs) => {
            for e in errs {
                resolution_context.add_parameter_error_to_active_model(name.clone(), e);
            }
            Err(())
        }
    }
}

fn handle_design_parameter_addition<E: ExternalResolutionContext>(
    p: &ast::DesignParameterNode,
    value: ir::ParameterValue,
    section_placement: Option<(SectionLabel, Option<ir::Note>)>,
    running: &mut Design,
    resolution_context: &mut ResolutionContext<'_, E>,
) {
    let name = ParameterName::from(p.ident().as_str());
    let design_span = p.ident().span();
    let section_label = section_placement.as_ref().map(|(l, _)| l.clone());
    let limits = match resolve_optional_design_limits(p, &name, resolution_context) {
        Ok(Some(limits)) => limits,
        Ok(None) => ir::Limits::default(),
        Err(()) => return,
    };
    let dependencies = resolve_parameter::get_parameter_dependencies(&value, &limits);
    let label = p.label().map_or_else(
        || ParameterLabel::from(p.ident().as_str()),
        |l| ParameterLabel::from(l.as_str()),
    );
    let render_name = p.render_name().map(|r| r.deref().clone());
    let is_performance = p.performance_marker().is_some();
    let trace_level = resolve_trace_level(p.trace_level());
    let note = p.note().map(|n| ir::Note::new(n.value().to_string()));
    let local_param = ir::Parameter::new(
        dependencies,
        name.clone(),
        design_span.clone(),
        design_span.clone(),
        label,
        render_name,
        section_label,
        value,
        limits,
        is_performance,
        trace_level,
        note,
    );
    if let Some(placement) = section_placement {
        running
            .parameter_section_placements
            .insert(name.clone(), placement);
    }
    running.parameter_additions.insert(name, local_param);
}

fn iter_apply_designs(model_ast: &ast::Model) -> Vec<&ast::ApplyDesignNode> {
    let mut out = Vec::new();
    for item in collect_design_surface(model_ast) {
        if let DesignSurfaceItem::Apply(n) = item {
            push_apply_recursive(n, &mut out);
        }
    }
    out
}

fn push_apply_recursive<'a>(
    node: &'a ast::ApplyDesignNode,
    out: &mut Vec<&'a ast::ApplyDesignNode>,
) {
    out.push(node);
    for nested in node.nested_applies() {
        push_apply_recursive(nested, out);
    }
}

/// Registers a design-local parameter as a scratch entry so it is visible to
/// variable lookups while the target model is active.
fn register_design_local_scratch<E: ExternalResolutionContext>(
    model_path: &ModelPath,
    name: ParameterName,
    resolution_context: &mut ResolutionContext<'_, E>,
) {
    let scratch_span = Span::synthetic();

    let scratch = ir::Parameter::new(
        ir::Dependencies::new(),
        name.clone(),
        scratch_span.clone(),
        scratch_span.clone(),
        ParameterLabel::from(name.as_str()),
        None,
        None,
        ir::ParameterValue::simple(
            ir::Expr::literal(scratch_span, ir::Literal::Number(0.0)),
            None,
        ),
        ir::Limits::default(),
        false,
        ir::TraceLevel::None,
        None,
    );
    resolution_context.register_design_local_parameter(model_path.clone(), name, scratch);
}

/// Resolves the design surface for the active model.
///
/// Processes four stages in order:
///
/// 1. Scan for the design target and register scratch entries for design-local
///    parameters so they can cross-reference each other during resolution.
/// 2. Dispatch each surface item to the appropriate handler, accumulating a
///    running [`Design`].
/// 3. Store the resulting design export on the active result.
/// 4. Record an [`ApplyDesign`] for every `apply <file> to <path>` declaration.
pub fn resolve_design_surface<E: ExternalResolutionContext>(
    model_path: &ModelPath,
    model_ast: &ast::Model,
    resolution_context: &mut ResolutionContext<'_, E>,
) {
    let surface = collect_design_surface(model_ast);

    let (mut explicit_target, design_local_param_names) =
        scan_design_locals(model_path, &surface, resolution_context);

    let mut running = Design::new();

    for item in &surface {
        match item {
            DesignSurfaceItem::Target(node) => {
                handle_design_target(node, model_path, &mut explicit_target, &mut running);
            }
            DesignSurfaceItem::Parameter { node: p, section } => handle_design_parameter(
                p,
                section.clone(),
                explicit_target.as_ref(),
                &design_local_param_names,
                &mut running,
                resolution_context,
            ),
            DesignSurfaceItem::Apply(_) => {}
        }
    }

    let exported_target = explicit_target.clone();
    store_design_export(&surface, explicit_target, running, resolution_context);
    record_applied_designs(
        &surface,
        model_path,
        exported_target.as_ref(),
        resolution_context,
    );
}

/// Scans the design surface for the target declaration and design-local
/// parameter names, registering scratch entries for them.
///
/// Returns `(explicit_target, design_local_param_names)`.
fn scan_design_locals<E: ExternalResolutionContext>(
    model_path: &ModelPath,
    surface: &[DesignSurfaceItem<'_>],
    resolution_context: &mut ResolutionContext<'_, E>,
) -> (Option<ModelPath>, IndexSet<ParameterName>) {
    let mut explicit_target: Option<ModelPath> = None;
    let mut design_param_names: IndexSet<ParameterName> = IndexSet::new();
    for item in surface {
        match item {
            DesignSurfaceItem::Target(node) => {
                let relative_path = node.get_target_relative_path();
                explicit_target = Some(model_path.get_sibling_model_path(relative_path));
            }
            DesignSurfaceItem::Parameter { node: p, .. } if p.instance_path().is_none() => {
                design_param_names.insert(ParameterName::from(p.ident().as_str()));
            }
            DesignSurfaceItem::Parameter { .. } | DesignSurfaceItem::Apply(_) => {}
        }
    }

    let mut design_local_param_names: IndexSet<ParameterName> = IndexSet::new();
    if let Some(tgt) = &explicit_target {
        for name in &design_param_names {
            let exists = matches!(
                resolution_context.lookup_model(tgt),
                ModelResult::Found(m) if m.get_parameter(name).is_some()
            );
            if !exists {
                design_local_param_names.insert(name.clone());
                register_design_local_scratch(tgt, name.clone(), resolution_context);
            }
        }
    }

    (explicit_target, design_local_param_names)
}

/// Records the design's `design <model>` target.
///
/// The parser enforces a single `design` declaration per file (see
/// `oneil_parser::declaration`) and rejects `design` headers in `.on`
/// files outright, so we don't need to guard against duplicates here.
fn handle_design_target(
    node: &ast::DesignTargetNode,
    model_path: &ModelPath,
    explicit_target: &mut Option<ModelPath>,
    running: &mut Design,
) {
    let relative_path = node.get_target_relative_path();
    let p = model_path.get_sibling_model_path(relative_path);
    *explicit_target = Some(p.clone());
    running.target_model = Some(p);
}

fn handle_design_parameter<E: ExternalResolutionContext>(
    p: &ast::DesignParameterNode,
    section: Option<DesignSectionContext>,
    explicit_target: Option<&ModelPath>,
    design_local_param_names: &IndexSet<ParameterName>,
    running: &mut Design,
    resolution_context: &mut ResolutionContext<'_, E>,
) {
    let Some(tgt) = explicit_target.cloned() else {
        resolution_context.add_design_resolution_error_to_active_model(
            "design parameter line requires a preceding `design <model>` declaration",
            p.ident().span().clone(),
        );
        return;
    };
    let name = ParameterName::from(p.ident().as_str());

    let instance_path = p
        .instance_path()
        .map(|seg| InstancePath::root().child(ReferenceName::new(seg.as_str().to_string())));

    // Resolve the RHS in the design target's scope.
    let active = resolution_context
        .active_models()
        .last()
        .expect("active model");
    let pushed = active != &tgt;
    if pushed {
        resolution_context.push_active_model(&tgt);
    }
    let resolved_value =
        resolve_parameter::resolve_parameter_value(p.value().deref(), resolution_context);
    if pushed {
        resolution_context.pop_active_model(&tgt);
    }

    let value = match resolved_value {
        Ok(v) => v,
        Err(errs) => {
            for e in errs {
                resolution_context.add_parameter_error_to_active_model(name.clone(), e);
            }
            return;
        }
    };

    let is_local_param = instance_path.is_none() && design_local_param_names.contains(&name);
    let section_placement = section.map(|ctx| (ctx.label, ctx.note));

    if is_local_param {
        handle_design_parameter_addition(p, value, section_placement, running, resolution_context);
        return;
    }

    let design_span = p.ident().span();

    let original_model_span =
        span_of_parameter_on_model(resolution_context, &tgt, &name, design_span.clone());
    let Ok(limits_override) = resolve_optional_design_limits(p, &name, resolution_context) else {
        return;
    };

    let overlay_value = OverlayParameterValue {
        value,
        design_span: design_span.clone(),
        original_model_span,
        note: p.note().map(|n| ir::Note::new(n.value().to_string())),
        label: p.label().map(|l| ParameterLabel::from(l.as_str())),
        render_name: p.render_name().map(|r| r.deref().clone()),
        limits_override,
        section: section_placement,
    };
    match instance_path {
        Some(ip) => {
            running
                .scoped_overrides
                .entry(ip)
                .or_default()
                .insert(name, overlay_value);
        }
        None => {
            running.parameter_overrides.insert(name, overlay_value);
        }
    }
}

/// Stores the final [`Design`] on the active result when the surface produced
/// design content; otherwise stores nothing.
fn store_design_export<E: ExternalResolutionContext>(
    surface: &[DesignSurfaceItem<'_>],
    explicit_target: Option<ModelPath>,
    mut running: Design,
    resolution_context: &mut ResolutionContext<'_, E>,
) {
    let has_design_content = explicit_target.is_some()
        || surface
            .iter()
            .any(|i| matches!(i, DesignSurfaceItem::Parameter { .. }));

    if has_design_content {
        running.target_model = explicit_target;
        resolution_context.set_active_model_design_export(running);
    }
}

/// Records a declarative [`ApplyDesign`] for every `apply <file> to <path>`
/// declaration. Validates that the target path is resolvable and that the
/// design's declared target matches the model the path resolves to.
fn record_applied_designs<E: ExternalResolutionContext>(
    surface: &[DesignSurfaceItem<'_>],
    model_path: &ModelPath,
    explicit_target: Option<&ModelPath>,
    resolution_context: &mut ResolutionContext<'_, E>,
) {
    let consuming_model = explicit_target
        .cloned()
        .unwrap_or_else(|| model_path.clone());

    for item in surface {
        let DesignSurfaceItem::Apply(node) = item else {
            continue;
        };
        record_apply_recursive(
            node,
            &InstancePath::root(),
            model_path,
            &consuming_model,
            resolution_context,
        );
    }
}

/// Resolves and records a single `apply <file> to <target>` declaration,
/// recursing into any nested applies in the `[ … ]` block.
///
/// `consuming_model` is the model whose named children are searched when
/// resolving the *first* segment of `node.target()`.  For top-level applies
/// this is the design file's declared target (or the file itself for `.on`
/// files); for nested applies it is the model that the outer target segment
/// resolves to, so each level validates against the correct scope.
fn record_apply_recursive<E: ExternalResolutionContext>(
    node: &ast::ApplyDesignNode,
    outer_target: &InstancePath,
    model_path: &ModelPath,
    consuming_model: &ModelPath,
    resolution_context: &mut ResolutionContext<'_, E>,
) {
    let segments = node.target();
    if segments.is_empty() {
        return;
    }

    // Walk every segment, validating each one against the model that the
    // preceding segment resolved to.  This lets `apply X to a.b.c` work
    // correctly even when `a`, `b`, and `c` live in different files.
    let mut target_path = outer_target.clone();
    let mut current_model = consuming_model.clone();
    let mut last_resolved = ReferenceName::new(String::new());

    for (i, seg) in segments.iter().enumerate() {
        let resolved = match resolve_segment_in(&current_model, seg.as_str(), resolution_context) {
            Ok(name) => name,
            Err(err) => {
                resolution_context
                    .add_design_resolution_error_to_active_model(err, seg.span().clone());
                return;
            }
        };
        target_path = target_path.child(resolved.clone());
        last_resolved = resolved.clone();

        // For every segment except the last, advance `current_model` to the
        // model that `resolved` points to so the next segment is validated in
        // the correct scope.
        if i + 1 < segments.len() {
            if let Some(next) =
                lookup_referenced_model_path(&current_model, &resolved, resolution_context)
            {
                current_model = next;
            } else {
                resolution_context.add_design_resolution_error_to_active_model(
                    format!(
                        "cannot follow apply target through `{}`: \
                         its model path could not be determined",
                        seg.as_str(),
                    ),
                    seg.span().clone(),
                );
                return;
            }
        }
    }

    // `current_model` is now the model containing the last segment.
    // Check that the design's declared target matches the model the last
    // segment resolves to (if the design declares a target at all).
    let relative_path = node.get_design_relative_path();
    let dpath = model_path.get_sibling_design_path(relative_path);
    let dpath_as_model = dpath.to_model_path();

    let final_model =
        lookup_referenced_model_path(&current_model, &last_resolved, resolution_context);

    if let Some(ref final_model_path) = final_model {
        let design_target = resolution_context
            .get_design_export(&dpath_as_model)
            .and_then(|d| d.target_model.clone());

        if let Some(design_target) = design_target.as_ref()
            && !same_model_path(design_target, final_model_path)
        {
            let target_display = segments
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(".");
            resolution_context.add_design_resolution_error_to_active_model(
                format!(
                    "`apply {design} to {target}`: design `{design}` targets \
                     `{design_target}`, but `{target}` resolves to `{ref_target}`. \
                     Use a design whose `design <model>` matches `{ref_target}`.",
                    target = target_display,
                    design = display_model_path(dpath.as_path()),
                    design_target = display_model_path(design_target.as_path()),
                    ref_target = display_model_path(final_model_path.as_path()),
                ),
                node.span().clone(),
            );
            return;
        }
    }

    resolution_context.add_applied_design_to_active_model(ApplyDesign {
        design_path: dpath,
        target: target_path.clone(),
        span: node.span().clone(),
    });

    // Nested applies (`apply X to a [ apply Y to b ]`) are validated against
    // the model that the final segment of the outer target resolves to, not
    // the top-level `consuming_model`.  Fall back to `current_model` (the
    // parent of the last segment) if the final model path is unavailable.
    let nested_consuming_model = final_model.unwrap_or(current_model);
    for nested in node.nested_applies() {
        record_apply_recursive(
            nested,
            &target_path,
            model_path,
            &nested_consuming_model,
            resolution_context,
        );
    }
}

fn resolve_segment_in<E: ExternalResolutionContext>(
    model_path: &ModelPath,
    segment: &str,
    resolution_context: &ResolutionContext<'_, E>,
) -> Result<ReferenceName, String> {
    let model = match resolution_context.lookup_model(model_path) {
        ModelResult::Found(m) => m,
        ModelResult::HasError | ModelResult::NotFound => {
            return Err(format!(
                "cannot resolve `{}` because target model `{}` failed to load",
                segment,
                display_model_path(model_path.as_path()),
            ));
        }
    };

    let candidate = ReferenceName::new(segment.to_string());
    if model.references().contains_key(&candidate)
        || model.submodels().contains_key(&candidate)
        || model.aliases().contains_key(&candidate)
    {
        return Ok(candidate);
    }

    // Fallback: match a `reference` whose target file's stem is `segment`,
    // for users who wrote the model file name instead of the alias.
    let matches: Vec<&ReferenceName> = model
        .references()
        .iter()
        .filter(|(_, import)| {
            import.path.as_path().file_stem().and_then(|s| s.to_str()) == Some(segment)
        })
        .map(|(name, _)| name)
        .collect();

    match matches.as_slice() {
        [] => Err(format!(
            "no reference, submodel, or alias named `{segment}` on `{model}`",
            segment = segment,
            model = display_model_path(model_path.as_path()),
        )),
        [single] => Ok((*single).clone()),
        _ => Err(format!(
            "segment `{segment}` is ambiguous on `{model}`: matches references {refs}",
            segment = segment,
            model = display_model_path(model_path.as_path()),
            refs = matches
                .iter()
                .map(|r| format!("`{}`", r.as_str()))
                .collect::<Vec<_>>()
                .join(", "),
        )),
    }
}

fn lookup_referenced_model_path<E: ExternalResolutionContext>(
    consuming_model: &ModelPath,
    rn: &ReferenceName,
    resolution_context: &ResolutionContext<'_, E>,
) -> Option<ModelPath> {
    let model = match resolution_context.lookup_model(consuming_model) {
        ModelResult::Found(m) => m,
        ModelResult::HasError | ModelResult::NotFound => return None,
    };
    if let Some(r) = model.references().get(rn) {
        return Some(r.path.clone());
    }
    if let Some(s) = model.submodels().get(rn) {
        return Some(s.instance.path().clone());
    }
    // Aliases resolve to a path by walking submodels along their alias_path.
    if let Some(alias) = model.aliases().get(rn) {
        let mut node = model;
        for seg in alias.alias_path.segments() {
            node = node.submodels().get(seg)?.instance.as_ref();
        }
        return Some(node.path().clone());
    }
    None
}
