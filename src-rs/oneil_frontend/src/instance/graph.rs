//! Instance graph: tree of [`InstancedModel`] reachable from a root,
//! plus a reference pool of cross-file shared instances.
//!
//! ## Shape
//!
//! An [`InstanceGraph`] has two halves:
//!
//! * `root: Box<InstancedModel>` — the user-requested root of the
//!   compilation. The root owns its `submodel` children directly,
//!   each in turn owning theirs. Children are reached via the
//!   per-instance `submodels` map; `with`-extracted aliases live on
//!   the `aliases` map and resolve to a relative path within the same
//!   host's subtree.
//! * `reference_pool: IndexMap<ModelPath, Box<InstancedModel>>` —
//!   one self-rooted instance per unique `ModelPath` reached via a
//!   `reference` declaration anywhere in the tree. Eval looks up an
//!   external reference written `p.r` in source (parameter `p` accessed
//!   through reference `r`) by `host.references()[r].path`, then indexes
//!   the pool to get the actual instance.
//!
//! ## Two-pass build
//!
//! 1. **Per-unit build** ([`build_unit_graph_for`]) constructs and
//!    caches the *self-rooted* graph for a [`CompilationUnit`]. It
//!    recursively builds every reachable child unit through the cache
//!    and inlines each child's `root` subtree under the parent's
//!    submodel slot. References go to the pool (deduped by
//!    `ModelPath`). Cycle detection lives here. The unit's own
//!    `apply X to ref` declarations are overlaid before the graph is
//!    cached.
//! 2. **Design composition** ([`apply_designs`]) builds (or fetches)
//!    the root unit's cached graph and overlays runtime-supplied
//!    designs at the root anchor.
//!
//! Per-instance variable classification ([`classify_variables`]) is a
//! pre-validation step that the post-build validation pass invokes as
//! its first action. It walks the composed graph and rewrites raw
//! identifiers in expression IR as `Parameter` / `Builtin` /
//! `External` against each host's binding scope. Unresolved
//! identifiers stay as `Parameter` / `External`; the validation pass
//! that follows surfaces them as `UndefinedParameter` /
//! `UndefinedReference` diagnostics.

use indexmap::IndexMap;
use oneil_ir as ir;
use oneil_shared::{
    InstancePath, RelativePath,
    load_result::LoadResult,
    paths::{DesignPath, ModelPath},
    search::search,
    span::Span,
    symbols::{ParameterName, ReferenceName, TestIndex},
};

/// Returns the appropriate [`CompilationUnit`] for a submodel child path.
///
/// Submodels backed by a `.one` design file are routed through
/// [`CompilationUnit::Design`] so that the graph builder invokes
/// [`build_design_unit_graph`], which loads the target model declared inside
/// the design file and applies the design's contributions.  Plain `.on` model
/// files use [`CompilationUnit::Model`] as before.
fn child_compilation_unit(path: ModelPath) -> CompilationUnit {
    match DesignPath::try_from(path.clone()) {
        Ok(dp) => CompilationUnit::Design(dp),
        Err(()) => CompilationUnit::Model(path),
    }
}

use crate::context::MAX_BEST_MATCH_DISTANCE;
use crate::error::{DesignResolutionError, ResolutionErrorCollection};
use crate::instance::cycle_error::CompilationCycleError;

use super::{
    CompilationUnit, ContributionDiagnostic, InstancedModel,
    design::{ApplyDesign, Design, OverlayParameterValue},
    validation_error::InstanceValidationError,
};

// ── Public types ─────────────────────────────────────────────────────────────

/// The complete instance tree reachable from a root model, with all
/// design contributions applied.
#[derive(Debug, Clone)]
pub struct InstanceGraph {
    /// The user's root instance. Owns its `submodel` children directly.
    pub root: Box<InstancedModel>,
    /// Resolved design content for `CompilationUnit::Design` cache entries.
    /// Set by [`build_design_unit_graph`] and `None` for model units.
    pub design_export: Option<Design>,
    /// Cross-file shared instances reached via `reference` declarations
    /// anywhere in the tree, keyed by the referenced `ModelPath` (one
    /// entry per unique path).
    pub reference_pool: IndexMap<ModelPath, Box<InstancedModel>>,
    /// Cycle diagnostics observed anywhere in the build / compose pipeline.
    pub cycle_errors: Vec<CompilationCycleError>,
    /// Per-file resolver-time diagnostics for every file reachable
    /// through this graph.
    pub resolution_errors: IndexMap<ModelPath, ResolutionErrorCollection>,
    /// Contribution-time diagnostics produced by design overlays
    /// during this graph's build (overlay-target-missing,
    /// overlay-unit-mismatch).
    pub contribution_errors: Vec<ContributionDiagnostic>,
    /// Post-build validation errors pushed by
    /// `oneil_analysis::validate_instance_graph`.
    pub validation_errors: Vec<InstanceValidationError>,
}

impl InstanceGraph {
    /// Returns an empty graph rooted at `root_path`.
    #[must_use]
    pub fn empty(root_path: ModelPath) -> Self {
        Self {
            root: Box::new(InstancedModel::empty_for(root_path)),
            design_export: None,
            reference_pool: IndexMap::new(),
            cycle_errors: Vec::new(),
            resolution_errors: IndexMap::new(),
            contribution_errors: Vec::new(),
            validation_errors: Vec::new(),
        }
    }
}

/// Metadata stored alongside each model's template in the resolution results.
#[derive(Debug, Clone)]
pub struct ModelDesignInfo {
    /// `apply X to ref` declarations made by this model file.
    pub applied_designs: Vec<ApplyDesign>,
    /// Resolved design content exported by this file (for design files only).
    pub design_export: Option<Design>,
}

/// Lookup interface for builtin value names, used by [`classify_variables`]
/// (the pre-validation classification pass) to reclassify identifiers
/// against per-instance binding scopes.
pub trait BuiltinLookup {
    /// Returns `true` if `name` is a known builtin value.
    fn has_builtin_value(&self, name: &str) -> bool;
}

/// Per-compilation-unit cache for [`InstanceGraph`]s.
pub type UnitGraphCache = IndexMap<CompilationUnit, InstanceGraph>;

/// One frame on the per-build cycle-detection stack.
#[derive(Debug, Clone)]
pub struct CycleStackFrame {
    /// The compilation unit being built at this frame.
    pub unit: CompilationUnit,
    /// Span of the reference declaration in the predecessor frame's
    /// file that brought [`Self::unit`] into the build.
    pub imported_at: Span,
}

// ── Build context ────────────────────────────────────────────────────────────

struct GraphCtx<'a> {
    templates: &'a IndexMap<ModelPath, LoadResult<InstancedModel, ResolutionErrorCollection>>,
    design_info: &'a IndexMap<ModelPath, ModelDesignInfo>,
}

impl<'a> GraphCtx<'a> {
    const fn new(
        templates: &'a IndexMap<ModelPath, LoadResult<InstancedModel, ResolutionErrorCollection>>,
        design_info: &'a IndexMap<ModelPath, ModelDesignInfo>,
    ) -> Self {
        Self {
            templates,
            design_info,
        }
    }

    fn template(&self, path: &ModelPath) -> Option<&'a InstancedModel> {
        self.templates.get(path)?.value()
    }

    fn resolution_errors(&self, path: &ModelPath) -> Option<&'a ResolutionErrorCollection> {
        self.templates.get(path)?.error()
    }
}

fn attach_file_resolution_errors(graph: &mut InstanceGraph, path: &ModelPath, ctx: &GraphCtx<'_>) {
    if let Some(errors) = ctx.resolution_errors(path)
        && !errors.is_empty()
    {
        graph.resolution_errors.insert(path.clone(), errors.clone());
    }
}

// ── Public entry points ──────────────────────────────────────────────────────

/// Convenience over [`build_unit_graph_for`] for the model variant.
pub fn build_unit_graph(
    unit_path: &ModelPath,
    cache: &mut UnitGraphCache,
    stack: &mut Vec<CycleStackFrame>,
    templates: &IndexMap<ModelPath, LoadResult<InstancedModel, ResolutionErrorCollection>>,
    design_info: &IndexMap<ModelPath, ModelDesignInfo>,
) -> InstanceGraph {
    build_unit_graph_for(
        &CompilationUnit::Model(unit_path.clone()),
        cache,
        stack,
        templates,
        design_info,
    )
}

/// Builds the self-rooted [`InstanceGraph`] for an arbitrary
/// [`CompilationUnit`].
pub fn build_unit_graph_for(
    unit: &CompilationUnit,
    cache: &mut UnitGraphCache,
    stack: &mut Vec<CycleStackFrame>,
    templates: &IndexMap<ModelPath, LoadResult<InstancedModel, ResolutionErrorCollection>>,
    design_info: &IndexMap<ModelPath, ModelDesignInfo>,
) -> InstanceGraph {
    let ctx = GraphCtx::new(templates, design_info);
    build_unit_graph_inner(unit, Span::synthetic(), cache, stack, &ctx)
}

/// Builds the user-facing composed graph for `root_path`, layering on
/// runtime-supplied designs at the root.
pub fn apply_designs(
    root_path: &ModelPath,
    runtime_designs: &[ApplyDesign],
    cache: &mut UnitGraphCache,
    templates: &IndexMap<ModelPath, LoadResult<InstancedModel, ResolutionErrorCollection>>,
    design_info: &IndexMap<ModelPath, ModelDesignInfo>,
) -> InstanceGraph {
    let ctx = GraphCtx::new(templates, design_info);
    let mut stack = Vec::new();
    let root_unit = CompilationUnit::Model(root_path.clone());
    let mut composed =
        build_unit_graph_inner(&root_unit, Span::synthetic(), cache, &mut stack, &ctx);

    for app in runtime_designs {
        let design_file = app.design_path.to_model_path();
        let applied_via = ir::DesignApplication {
            apply_span: app.span.clone(),
            applied_in: root_path.clone(),
            design_path: app.design_path.clone(),
        };
        let mut root = (*composed.root).clone();
        apply_design_recursive(
            &mut root,
            &mut composed.reference_pool,
            &mut composed.contribution_errors,
            &app.target,
            &design_file,
            Some(&applied_via),
            &ctx,
        );

        // When a design is applied at the root level (the common case when the
        // user opens or evaluates a `.one` file directly), propagate its model-
        // level note onto the composed root. This ensures the design's
        // introductory prose surfaces in the rendered view rather than being
        // silently discarded.
        if app.target.is_root()
            && let Some(note) = ctx
                .design_info
                .get(&design_file)
                .and_then(|info| info.design_export.as_ref())
                .and_then(|design| design.note.as_ref())
        {
            root.set_note(note.clone());
        }

        *composed.root = root;
    }

    composed
}

/// Backward-compatible single-shot graph build.
#[must_use]
pub fn build_instance_graph(
    root_path: &ModelPath,
    runtime_designs: &[ApplyDesign],
    templates: &IndexMap<ModelPath, LoadResult<InstancedModel, ResolutionErrorCollection>>,
    design_info: &IndexMap<ModelPath, ModelDesignInfo>,
) -> InstanceGraph {
    let mut cache = UnitGraphCache::new();
    apply_designs(
        root_path,
        runtime_designs,
        &mut cache,
        templates,
        design_info,
    )
}

// ── Per-unit build ───────────────────────────────────────────────────────────

fn build_unit_graph_inner(
    unit: &CompilationUnit,
    imported_at: Span,
    cache: &mut UnitGraphCache,
    stack: &mut Vec<CycleStackFrame>,
    ctx: &GraphCtx<'_>,
) -> InstanceGraph {
    if let Some(cached) = cache.get(unit) {
        return cached.clone();
    }
    if stack.iter().any(|f| &f.unit == unit) {
        return InstanceGraph::empty(unit.source_path());
    }

    stack.push(CycleStackFrame {
        unit: unit.clone(),
        imported_at,
    });
    let graph = build_unit_graph_uncached(unit, cache, stack, ctx);
    let popped = stack.pop();
    debug_assert_eq!(popped.as_ref().map(|f| &f.unit), Some(unit));

    cache.insert(unit.clone(), graph.clone());
    graph
}

fn build_unit_graph_uncached(
    unit: &CompilationUnit,
    cache: &mut UnitGraphCache,
    stack: &mut Vec<CycleStackFrame>,
    ctx: &GraphCtx<'_>,
) -> InstanceGraph {
    match unit {
        CompilationUnit::Model(model_path) => build_model_unit_graph(model_path, cache, stack, ctx),
        CompilationUnit::Design(design_path) => {
            build_design_unit_graph(design_path, cache, stack, ctx)
        }
    }
}

fn build_model_unit_graph(
    model_path: &ModelPath,
    cache: &mut UnitGraphCache,
    stack: &mut Vec<CycleStackFrame>,
    ctx: &GraphCtx<'_>,
) -> InstanceGraph {
    let Some(template) = ctx.template(model_path) else {
        let mut graph = InstanceGraph::empty(model_path.clone());
        attach_file_resolution_errors(&mut graph, model_path, ctx);
        return graph;
    };

    let mut graph = InstanceGraph::empty(model_path.clone());

    // Construct the root subtree by recursively building children.
    let mut root = build_instance_subtree(template, cache, stack, ctx, &mut graph);

    // Apply this unit's own `apply X to ref` declarations, recursing into any
    // nested applies declared within each applied design file.
    if let Some(info) = ctx.design_info.get(model_path) {
        let applies: Vec<ApplyDesign> = info.applied_designs.clone();
        for app in &applies {
            let applied_via = ir::DesignApplication {
                apply_span: app.span.clone(),
                applied_in: model_path.clone(),
                design_path: app.design_path.clone(),
            };
            let design_file = app.design_path.to_model_path();
            apply_design_recursive(
                &mut root,
                &mut graph.reference_pool,
                &mut graph.contribution_errors,
                &app.target,
                &design_file,
                Some(&applied_via),
                ctx,
            );
        }
    }

    graph.root = Box::new(root);
    attach_file_resolution_errors(&mut graph, model_path, ctx);

    graph
}

fn build_design_unit_graph(
    design_path: &DesignPath,
    cache: &mut UnitGraphCache,
    stack: &mut Vec<CycleStackFrame>,
    ctx: &GraphCtx<'_>,
) -> InstanceGraph {
    let design_model_path = design_path.to_model_path();

    let empty_with_errors = |ctx: &GraphCtx<'_>| {
        let mut graph = InstanceGraph::empty(design_model_path.clone());
        attach_file_resolution_errors(&mut graph, &design_model_path, ctx);
        graph
    };

    let Some(info) = ctx.design_info.get(&design_model_path) else {
        return empty_with_errors(ctx);
    };
    let Some(design) = info.design_export.as_ref() else {
        return empty_with_errors(ctx);
    };
    let Some(target_path) = design.target_model.clone() else {
        return empty_with_errors(ctx);
    };

    let target_unit = CompilationUnit::Model(target_path);
    if let Some(error) = cycle_error_for_revisit(&target_unit, Span::synthetic(), stack) {
        let mut graph = empty_with_errors(ctx);
        graph.cycle_errors.push(error);
        return graph;
    }
    let mut graph = build_unit_graph_inner(&target_unit, Span::synthetic(), cache, stack, ctx);
    let mut root = (*graph.root).clone();

    // Overlay the design's own contributions at the target root, then
    // recurse into any nested `apply X to ref` declarations within the design.
    apply_design_recursive(
        &mut root,
        &mut graph.reference_pool,
        &mut graph.contribution_errors,
        &InstancePath::root(),
        &design_model_path,
        None,
        ctx,
    );

    // If the design file carries a model-level note, apply it to the composed
    // root so the rendered view shows the design's introductory prose rather
    // than the target model's own note (which is absent for most base models).
    if let Some(note) = &design.note {
        root.set_note(note.clone());
    }

    graph.root = Box::new(root);
    attach_file_resolution_errors(&mut graph, &design_model_path, ctx);
    graph.design_export = Some(design.clone());

    graph
}

/// Builds the subtree rooted at `template`, recursively populating
/// child submodels (replacing stub instances with built subtrees) and
/// merging child reference-pool entries into `graph.reference_pool`.
fn build_instance_subtree(
    template: &InstancedModel,
    cache: &mut UnitGraphCache,
    stack: &mut Vec<CycleStackFrame>,
    ctx: &GraphCtx<'_>,
    graph: &mut InstanceGraph,
) -> InstancedModel {
    let mut node = template.clone();

    // Recurse into owned submodel children.
    let aliases: Vec<ReferenceName> = node.submodels().keys().cloned().collect();
    for alias in aliases {
        let (child_path, name_span) = {
            let sub = node
                .get_submodel(&alias)
                .expect("alias just enumerated from submodels");
            (sub.instance.path().clone(), sub.name_span.clone())
        };
        // Design-file submodels (path ends in .one) are routed through
        // CompilationUnit::Design so build_design_unit_graph applies the
        // design's overrides on top of the target model.
        let child_unit = child_compilation_unit(child_path);
        if let Some(error) = cycle_error_for_revisit(&child_unit, name_span.clone(), stack) {
            graph.cycle_errors.push(error);
            continue;
        }
        let mut child_graph = build_unit_graph_inner(&child_unit, name_span, cache, stack, ctx);
        // Inline child's root subtree under the submodel slot.
        let child_root = std::mem::replace(
            &mut child_graph.root,
            Box::new(InstancedModel::empty_for(child_unit.source_path())),
        );
        if let Some(sub) = node.submodels_mut().get_mut(&alias) {
            sub.instance = child_root;
        }
        merge_child_graph(graph, child_graph);
    }

    // Build references — each unique ModelPath becomes a pool entry.
    let ref_paths: Vec<(ModelPath, Span)> = node
        .references()
        .values()
        .map(|r| (r.path.clone(), r.name_span.clone()))
        .collect();
    for (path, name_span) in ref_paths {
        if graph.reference_pool.contains_key(&path) {
            continue;
        }
        let child_unit = CompilationUnit::Model(path.clone());
        if let Some(error) = cycle_error_for_revisit(&child_unit, name_span.clone(), stack) {
            graph.cycle_errors.push(error);
            continue;
        }
        let mut child_graph = build_unit_graph_inner(&child_unit, name_span, cache, stack, ctx);
        let child_root = std::mem::replace(
            &mut child_graph.root,
            Box::new(InstancedModel::empty_for(path.clone())),
        );
        graph.reference_pool.insert(path, child_root);
        merge_child_graph(graph, child_graph);
    }

    node
}

/// Merges the *graph-level* state of a freshly-built child unit graph
/// into `parent`: pool entries (deduped), cycle / resolution /
/// contribution diagnostics. The child's `root` is consumed by the
/// caller (inlined as a submodel or stored in the pool); the rest of
/// the child graph is rolled into the parent here.
///
/// Cycle and contribution errors are deduplicated: a cycle with the same
/// chain and source path, or a contribution at the same host path with
/// the same design file, is only recorded once.
fn merge_child_graph(parent: &mut InstanceGraph, child: InstanceGraph) {
    let InstanceGraph {
        root: _,
        reference_pool,
        cycle_errors,
        resolution_errors,
        design_export: _,
        contribution_errors,
        validation_errors: _,
    } = child;

    for (path, instance) in reference_pool {
        parent.reference_pool.entry(path).or_insert(instance);
    }

    // Deduplicate cycle errors by (cycle_chain, source_path).
    for err in cycle_errors {
        #[expect(
            clippy::maybe_infinite_iter,
            reason = "cycle_errors is a finite Vec; slice == is O(n), not infinite"
        )]
        let dominated = parent.cycle_errors.iter().any(|existing| {
            existing.cycle() == err.cycle() && existing.source_path() == err.source_path()
        });
        if !dominated {
            parent.cycle_errors.push(err);
        }
    }

    for (path, errors) in resolution_errors {
        parent.resolution_errors.entry(path).or_insert(errors);
    }

    // Deduplicate contribution errors by (host_path, design_file, error message).
    for err in contribution_errors {
        let dominated = parent.contribution_errors.iter().any(|existing| {
            existing.host_path == err.host_path
                && existing.design_file == err.design_file
                && existing.error.message() == err.error.message()
        });
        if !dominated {
            parent.contribution_errors.push(err);
        }
    }
}

fn cycle_error_for_revisit(
    child_unit: &CompilationUnit,
    back_edge_span: Span,
    stack: &[CycleStackFrame],
) -> Option<CompilationCycleError> {
    let stack_idx = stack.iter().position(|f| &f.unit == child_unit)?;
    let mut cycle: Vec<CompilationUnit> =
        stack[stack_idx..].iter().map(|f| f.unit.clone()).collect();
    cycle.push(child_unit.clone());
    let target_span = stack
        .get(stack_idx + 1)
        .map_or(back_edge_span, |f| f.imported_at.clone());
    let target_path = stack[stack_idx].unit.source_path();
    Some(CompilationCycleError::new(cycle, target_path, target_span))
}

// ── Design-contribution application ──────────────────────────────────────────

/// Applies `design`'s contributions at `target` (relative to `root`)
/// in place. The `target` may resolve to a node inside the root
/// subtree or to (a sub-path of) a pool entry.
#[expect(clippy::too_many_arguments, reason = "single internal helper")]
fn apply_design_at_target_in_place(
    root: &mut InstancedModel,
    pool: &mut IndexMap<ModelPath, Box<InstancedModel>>,
    contribution_errors: &mut Vec<ContributionDiagnostic>,
    target: &InstancePath,
    design: &Design,
    design_file: &ModelPath,
    applied_via: Option<ir::DesignApplication>,
    ctx: &GraphCtx<'_>,
) {
    let segments: Vec<ReferenceName> = target.segments().to_vec();
    let Some(host_loc) = resolve_host_location(root, pool, &segments) else {
        // Apply target couldn't be resolved against the live tree;
        // surface as a contribution diagnostic on the (synthetic)
        // host-path equivalent of the apply target.
        let span = applied_via
            .as_ref()
            .map_or(Span::synthetic(), |a| a.apply_span.clone());
        contribution_errors.push(ContributionDiagnostic::new(
            target.clone(),
            DesignResolutionError::new(
                format!(
                    "design target `{}` could not be resolved on the host model",
                    render_path(target)
                ),
                span,
            ),
            design_file.clone(),
            applied_via,
        ));
        return;
    };

    apply_design_at_host(
        root,
        pool,
        contribution_errors,
        &host_loc,
        design,
        design_file,
        applied_via.as_ref(),
        ctx,
    );
}

/// Applies the contributions of the design file at `design_file` to the
/// instance tree at `effective_target`, then recursively processes any
/// `apply X to ref` declarations that the design file itself contains —
/// applying each nested design at the correspondingly prefixed target.
///
/// This ensures that when a model applies `outer.one` to target `T`, any
/// designs that `outer.one` in turn applies to its own sub-references also
/// land in the consuming model's tree.
fn apply_design_recursive(
    root: &mut InstancedModel,
    pool: &mut IndexMap<ModelPath, Box<InstancedModel>>,
    contribution_errors: &mut Vec<ContributionDiagnostic>,
    effective_target: &InstancePath,
    design_file: &ModelPath,
    applied_via: Option<&ir::DesignApplication>,
    ctx: &GraphCtx<'_>,
) {
    let Some(d_info) = ctx.design_info.get(design_file) else {
        return;
    };

    if let Some(design) = d_info.design_export.as_ref() {
        apply_design_at_target_in_place(
            root,
            pool,
            contribution_errors,
            effective_target,
            design,
            design_file,
            applied_via.cloned(),
            ctx,
        );
    }

    // Walk nested `apply X to ref` declarations inside this design file and
    // apply them at the correspondingly prefixed targets. We clone to avoid
    // holding the immutable borrow on `ctx` while mutating `root`/`pool`.
    let nested_applies: Vec<ApplyDesign> = d_info.applied_designs.clone();
    for app in &nested_applies {
        let nested_target = effective_target.join(&app.target);
        let nested_design_file = app.design_path.to_model_path();
        let nested_applied_via = ir::DesignApplication {
            apply_span: app.span.clone(),
            applied_in: design_file.clone(),
            design_path: app.design_path.clone(),
        };
        apply_design_recursive(
            root,
            pool,
            contribution_errors,
            &nested_target,
            &nested_design_file,
            Some(&nested_applied_via),
            ctx,
        );
    }
}

/// Applies `design` to the resolved host location (the apply target
/// instance), then descends into its `scoped_*` overlays.
#[expect(clippy::too_many_arguments, reason = "single internal helper")]
fn apply_design_at_host(
    root: &mut InstancedModel,
    pool: &mut IndexMap<ModelPath, Box<InstancedModel>>,
    contribution_errors: &mut Vec<ContributionDiagnostic>,
    host_loc: &HostLocation,
    design: &Design,
    design_file: &ModelPath,
    applied_via: Option<&ir::DesignApplication>,
    ctx: &GraphCtx<'_>,
) {
    {
        let host = host_at_mut(root, pool, host_loc).expect("host_loc just resolved");
        apply_overlay_at_host(
            host,
            &design.parameter_overrides,
            &design.parameter_additions,
            &host_loc.absolute_path.host_path(),
            &RelativePath::self_path(),
            design_file,
            applied_via,
            contribution_errors,
            ctx,
        );

        // Apply test additions from the design.
        // Tests are evaluated in the target's scope (the host instance).
        // We offset the test indices to avoid collisions with existing tests.
        let test_offset = host.tests().len();
        for (index, test) in &design.test_additions {
            let new_index = TestIndex::new(test_offset + index.into_usize());
            // Attach design provenance so validation errors are attributed to the design file.
            let provenance = ir::DesignProvenance {
                design_path: design_file.clone(),
                is_addition: true,
                assignment_span: test.span().clone(),
                anchor_path: RelativePath::self_path(),
                applied_via: applied_via.cloned(),
            };
            let test_with_provenance = test.clone().with_design_provenance(provenance);
            host.add_test(new_index, test_with_provenance);
        }
    }

    // Apply scoped contributions: each scoped path descends from the host.
    for (scope_path, overrides) in &design.scoped_overrides {
        apply_scoped_overlay(
            root,
            pool,
            contribution_errors,
            host_loc,
            scope_path,
            overrides,
            &IndexMap::new(),
            design_file,
            applied_via,
            ctx,
        );
    }
}

#[expect(clippy::too_many_arguments, reason = "single internal helper")]
fn apply_scoped_overlay(
    root: &mut InstancedModel,
    pool: &mut IndexMap<ModelPath, Box<InstancedModel>>,
    contribution_errors: &mut Vec<ContributionDiagnostic>,
    anchor_loc: &HostLocation,
    scope_path: &InstancePath,
    overrides: &IndexMap<ParameterName, OverlayParameterValue>,
    additions: &IndexMap<ParameterName, ir::Parameter>,
    design_file: &ModelPath,
    applied_via: Option<&ir::DesignApplication>,
    ctx: &GraphCtx<'_>,
) {
    // The scope_path is relative to the anchor (the apply target).
    let scope_segments: Vec<ReferenceName> = scope_path.segments().to_vec();

    // Resolve from the anchor into the deeper host, following submodels,
    // aliases, and references (cross-pool jumps).
    let Some((host_loc, anchor_relative)) =
        resolve_scoped_host_location(root, pool, anchor_loc, &scope_segments)
    else {
        // Scoped path didn't resolve. Surface a diagnostic.
        let span = applied_via
            .filter(|a| a.apply_span.start() != a.apply_span.end())
            .map_or_else(Span::synthetic, |a| a.apply_span.clone());
        let anchor_base = match &anchor_loc.absolute_path {
            AbsolutePath::Root(p) | AbsolutePath::Pool(_, p) => p.clone(),
        };
        let host_path_for_diag = extend_path(&anchor_base, &scope_segments);
        contribution_errors.push(ContributionDiagnostic::new(
            host_path_for_diag,
            DesignResolutionError::new(
                format!(
                    "scoped design target `{}` could not be resolved",
                    render_path(scope_path)
                ),
                span,
            ),
            design_file.clone(),
            applied_via.cloned(),
        ));
        return;
    };

    let host = host_at_mut(root, pool, &host_loc).expect("scoped host must exist");
    apply_overlay_at_host(
        host,
        overrides,
        additions,
        &host_loc.absolute_path.host_path(),
        &anchor_relative,
        design_file,
        applied_via,
        contribution_errors,
        ctx,
    );
}

/// Applies one host's worth of design contributions in place,
/// recording errors for missing / unit-mismatched overlay targets.
#[expect(clippy::too_many_arguments, reason = "single internal helper")]
fn apply_overlay_at_host(
    host: &mut InstancedModel,
    overrides: &IndexMap<ParameterName, OverlayParameterValue>,
    additions: &IndexMap<ParameterName, ir::Parameter>,
    host_path: &InstancePath,
    anchor_path: &RelativePath,
    design_file: &ModelPath,
    applied_via: Option<&ir::DesignApplication>,
    contribution_errors: &mut Vec<ContributionDiagnostic>,
    _ctx: &GraphCtx<'_>,
) {
    // Insert additions first so subsequent override RHS expressions can
    // reference them.
    for (name, parameter) in additions {
        let provenance = ir::DesignProvenance {
            design_path: design_file.clone(),
            is_addition: true,
            assignment_span: parameter.span().clone(),
            anchor_path: anchor_path.clone(),
            applied_via: applied_via.cloned(),
        };
        host.add_parameter(
            name.clone(),
            parameter.clone().with_design_provenance(provenance),
        );
    }

    // Apply overrides.
    for (name, overlay) in overrides {
        let Some(parameter) = host.parameters_mut().get_mut(name) else {
            let best_match = best_match_parameter(host.parameters(), name);
            let suggestion = best_match
                .as_deref()
                .map(|m| format!(" (did you mean `{m}`?)"))
                .unwrap_or_default();
            let message = format!(
                "design overlay targets parameter `{}`, which is not defined on the target model{suggestion}",
                name.as_str(),
            );
            contribution_errors.push(ContributionDiagnostic::new(
                host_path.clone(),
                DesignResolutionError::new(message, overlay.design_span.clone()),
                design_file.clone(),
                applied_via.cloned(),
            ));
            continue;
        };
        if let Some(err) = check_unit_compatibility(parameter, name, overlay) {
            contribution_errors.push(ContributionDiagnostic::new(
                host_path.clone(),
                err,
                design_file.clone(),
                applied_via.cloned(),
            ));
            continue;
        }
        let provenance = ir::DesignProvenance {
            design_path: design_file.clone(),
            is_addition: false,
            assignment_span: overlay.design_span.clone(),
            anchor_path: anchor_path.clone(),
            applied_via: applied_via.cloned(),
        };
        *parameter = parameter.clone().with_design_provenance(provenance);
        *parameter.value_mut() = overlay.value.clone();
        // Propagate design-supplied metadata overrides so the rendered view
        // shows the design's context rather than the base model's boilerplate.
        if let Some(note) = &overlay.note {
            parameter.set_note(note.clone());
        }
        if let Some(label) = &overlay.label {
            parameter.set_label(label.clone());
        }
        if let Some(render_name) = &overlay.render_name {
            parameter.set_render_name(render_name.clone());
        }
    }
}

/// Where in the graph a host instance lives.
#[derive(Debug, Clone)]
struct HostLocation {
    absolute_path: AbsolutePath,
}

#[derive(Debug, Clone)]
enum AbsolutePath {
    /// Node at the given path in `graph.root`'s subtree.
    Root(InstancePath),
    /// Node at the given sub-path of pool entry `ModelPath`.
    Pool(ModelPath, InstancePath),
}

impl AbsolutePath {
    fn host_path(&self) -> InstancePath {
        match self {
            Self::Root(p) | Self::Pool(_, p) => p.clone(),
        }
    }

    fn append(&self, seg: ReferenceName) -> Self {
        match self {
            Self::Root(p) => Self::Root(p.child(seg)),
            Self::Pool(pp, p) => Self::Pool(pp.clone(), p.child(seg)),
        }
    }
}

/// Resolves an `InstancePath` (relative to the graph root) into an
/// absolute [`HostLocation`], following aliases and crossing
/// reference jumps as needed.
fn resolve_host_location(
    root: &InstancedModel,
    pool: &IndexMap<ModelPath, Box<InstancedModel>>,
    segments: &[ReferenceName],
) -> Option<HostLocation> {
    let mut absolute = AbsolutePath::Root(InstancePath::root());
    let mut remaining: Vec<ReferenceName> = segments.to_vec();
    remaining.reverse(); // pop from end as a stack

    while let Some(seg) = remaining.pop() {
        let host = host_at_in_view(root, pool, &absolute)?;
        if host.get_submodel(&seg).is_some() {
            absolute = absolute.append(seg);
            continue;
        }
        if let Some(alias) = host.get_alias(&seg) {
            // Expand: prepend alias_path's segments to remaining, then continue.
            let mut expansion: Vec<ReferenceName> = alias.alias_path.segments().to_vec();
            expansion.reverse();
            remaining.extend(expansion);
            continue;
        }
        if let Some(reference) = host.get_reference(&seg) {
            absolute = AbsolutePath::Pool(reference.path.clone(), InstancePath::root());
            continue;
        }
        return None;
    }

    Some(HostLocation {
        absolute_path: absolute,
    })
}

fn host_at_in_view<'a>(
    root: &'a InstancedModel,
    pool: &'a IndexMap<ModelPath, Box<InstancedModel>>,
    absolute: &AbsolutePath,
) -> Option<&'a InstancedModel> {
    match absolute {
        AbsolutePath::Root(p) => navigate_in_subtree(root, p.segments()),
        AbsolutePath::Pool(pp, p) => navigate_in_subtree(pool.get(pp)?.as_ref(), p.segments()),
    }
}

fn host_at_mut<'a>(
    root: &'a mut InstancedModel,
    pool: &'a mut IndexMap<ModelPath, Box<InstancedModel>>,
    loc: &HostLocation,
) -> Option<&'a mut InstancedModel> {
    match &loc.absolute_path {
        AbsolutePath::Root(p) => navigate_mut_in_subtree(root, p.segments()),
        AbsolutePath::Pool(pp, p) => {
            let entry = pool.get_mut(pp)?;
            navigate_mut_in_subtree(entry.as_mut(), p.segments())
        }
    }
}

/// Resolves `scope_segments` relative to `anchor_loc`, following submodels,
/// aliases, and references (cross-pool jumps). Returns the resolved
/// [`HostLocation`] and a [`RelativePath`] for design provenance.
///
/// The provenance `RelativePath` counts submodel hops descended within the
/// final tree segment; it resets to zero whenever a reference boundary is
/// crossed (since the new pool entry has its own independent coordinate
/// system).
///
/// Returns `None` if any segment cannot be resolved.
fn resolve_scoped_host_location(
    root: &InstancedModel,
    pool: &IndexMap<ModelPath, Box<InstancedModel>>,
    anchor_loc: &HostLocation,
    scope_segments: &[ReferenceName],
) -> Option<(HostLocation, RelativePath)> {
    let mut absolute = anchor_loc.absolute_path.clone();
    // Number of submodel hops since the last reference cross (or from start).
    let mut submodel_depth: usize = 0;
    let mut remaining: Vec<ReferenceName> = scope_segments.to_vec();
    remaining.reverse(); // use as a stack: pop() yields segments left-to-right

    while let Some(seg) = remaining.pop() {
        let host = host_at_in_view(root, pool, &absolute)?;
        if host.get_submodel(&seg).is_some() {
            absolute = absolute.append(seg);
            submodel_depth += 1;
            continue;
        }
        if let Some(alias) = host.get_alias(&seg) {
            // Expand the alias into its path segments and push them back so
            // each hop is processed individually (and counted separately).
            let mut expansion: Vec<ReferenceName> = alias.alias_path.segments().to_vec();
            expansion.reverse();
            remaining.extend(expansion);
            continue;
        }
        if let Some(reference) = host.get_reference(&seg) {
            // Jump into the reference pool. The new pool-root is a separate
            // coordinate system, so the submodel depth resets.
            absolute = AbsolutePath::Pool(reference.path.clone(), InstancePath::root());
            submodel_depth = 0;
            continue;
        }
        return None;
    }

    let anchor_relative = RelativePath {
        up: submodel_depth,
        down: Vec::new(),
    };
    Some((
        HostLocation {
            absolute_path: absolute,
        },
        anchor_relative,
    ))
}

fn navigate_in_subtree<'a>(
    node: &'a InstancedModel,
    segments: &[ReferenceName],
) -> Option<&'a InstancedModel> {
    let mut current = node;
    for seg in segments {
        if let Some(sub) = current.submodels().get(seg) {
            current = sub.instance.as_ref();
        } else if let Some(alias) = current.aliases().get(seg) {
            // Follow the alias path through the submodel tree.
            let mut resolved = current;
            for alias_seg in alias.alias_path.segments() {
                resolved = resolved.submodels().get(alias_seg)?.instance.as_ref();
            }
            current = resolved;
        } else {
            return None;
        }
    }
    Some(current)
}

fn navigate_mut_in_subtree<'a>(
    node: &'a mut InstancedModel,
    segments: &[ReferenceName],
) -> Option<&'a mut InstancedModel> {
    let mut current = node;
    for seg in segments {
        if current.aliases().contains_key(seg) {
            // For aliases, collect the path first then navigate mutably.
            let alias_segs: Vec<ReferenceName> =
                current.aliases()[seg].alias_path.segments().to_vec();
            for alias_seg in &alias_segs {
                let sub = current.submodels_mut().get_mut(alias_seg)?;
                current = sub.instance.as_mut();
            }
        } else {
            let sub = current.submodels_mut().get_mut(seg)?;
            current = sub.instance.as_mut();
        }
    }
    Some(current)
}

fn extend_path(base: &InstancePath, segs: &[ReferenceName]) -> InstancePath {
    segs.iter()
        .fold(base.clone(), |acc, seg| acc.child(seg.clone()))
}

fn render_path(p: &InstancePath) -> String {
    p.segments()
        .iter()
        .map(ReferenceName::as_str)
        .collect::<Vec<_>>()
        .join(".")
}

// ── Apply-time helpers ───────────────────────────────────────────────────────

fn check_unit_compatibility(
    target_param: &ir::Parameter,
    parameter_name: &ParameterName,
    overlay: &OverlayParameterValue,
) -> Option<DesignResolutionError> {
    let target_unit = match target_param.value() {
        ir::ParameterValue::Simple(_, u) | ir::ParameterValue::Piecewise(_, u) => u.as_ref()?,
    };
    let overlay_unit = match &overlay.value {
        ir::ParameterValue::Simple(_, u) | ir::ParameterValue::Piecewise(_, u) => u.as_ref()?,
    };

    if target_unit.dimension() == overlay_unit.dimension() {
        return None;
    }

    Some(DesignResolutionError::new(
        format!(
            "unit mismatch: parameter `{}` declared as `{}` but design applies value with unit `{}`",
            parameter_name.as_str(),
            target_unit.display_unit().to_resolved_display(),
            overlay_unit.display_unit().to_resolved_display(),
        ),
        overlay.design_span.clone(),
    ))
}

fn best_match_parameter(
    parameters: &IndexMap<ParameterName, ir::Parameter>,
    query: &ParameterName,
) -> Option<String> {
    let names: Vec<&str> = parameters.keys().map(ParameterName::as_str).collect();
    search(query.as_str(), &names)
        .and_then(|r| r.some_if_within_distance(MAX_BEST_MATCH_DISTANCE))
        .map(String::from)
}

// ── Variable classification (pre-validation) ─────────────────────────────────

/// Walks every `InstancedModel` in `graph` (root subtree + reference
/// pool) and reclassifies each variable in every parameter / test
/// expression against that instance's binding scope.
///
/// Reclassification is pure name-against-scope work — no diagnostics
/// are produced. It happens after the graph has been fully composed
/// (designs applied, submodel subtrees inlined) so that names added
/// or shadowed by design contributions resolve correctly:
///
/// * a `Parameter("x")` whose host has no `x` parameter but where `x`
///   is a known builtin becomes `Builtin("x")`, and
/// * a `Builtin("x")` whose host has gained an `x` parameter (e.g. a
///   design added `pi`) becomes `Parameter("x")`.
///
/// The result is a graph where every parameter/test expression is in
/// the canonical form expected by [`oneil_analysis::validate_instance_graph`]
/// and the evaluator. Validation calls this as its first step.
pub fn classify_variables(graph: &mut InstanceGraph, builtins: &dyn BuiltinLookup) {
    classify_subtree(&mut graph.root, builtins);
    for instance in graph.reference_pool.values_mut() {
        classify_subtree(instance, builtins);
    }
}

fn classify_subtree(node: &mut InstancedModel, builtins: &dyn BuiltinLookup) {
    classify_instance(node, builtins);
    let aliases: Vec<ReferenceName> = node.submodels().keys().cloned().collect();
    for alias in aliases {
        if let Some(sub) = node.submodels_mut().get_mut(&alias) {
            classify_subtree(sub.instance.as_mut(), builtins);
        }
    }
}

fn classify_instance(node: &mut InstancedModel, builtins: &dyn BuiltinLookup) {
    let scope_params: indexmap::IndexSet<ParameterName> =
        node.parameters().keys().cloned().collect();
    let scope = ClassifyScope {
        parameters: &scope_params,
        builtins,
    };

    for (_, parameter) in node.parameters_mut() {
        if parameter.design_provenance().is_some() {
            // Overlay RHS was already classified at design-resolve time
            // against the design's anchor scope; reclassifying here would
            // use the host's scope, which is wrong for overlay parameters.
            continue;
        }
        classify_value(parameter.value_mut(), &scope);
    }
    for (_, test) in node.tests_mut() {
        test.expr_mut().walk_variables_mut(&mut |variable| {
            classify_variable(variable, &scope);
        });
    }
}

/// Per-instance binding scope used by classification.
///
/// Variables no longer pin instance keys, so the scope only needs the
/// host's local parameter names plus a builtin-name lookup. Reference
/// names live on the host and are checked against by the validation
/// pass directly, not here.
struct ClassifyScope<'a> {
    parameters: &'a indexmap::IndexSet<ParameterName>,
    builtins: &'a dyn BuiltinLookup,
}

fn classify_value(value: &mut ir::ParameterValue, scope: &ClassifyScope<'_>) {
    let walk = |expr: &mut ir::Expr| {
        expr.walk_variables_mut(&mut |variable| classify_variable(variable, scope));
    };
    match value {
        ir::ParameterValue::Simple(expr, _) => walk(expr),
        ir::ParameterValue::Piecewise(pieces, _) => {
            for piece in pieces {
                walk(piece.expr_mut());
                walk(piece.if_expr_mut());
            }
        }
    }
}

/// Pure name classification:
///
/// * `Parameter("x")` becomes `Builtin("x")` if `x` is not a parameter
///   in scope but matches a builtin.
/// * `Builtin("x")` becomes `Parameter("x")` if `x` shadows a builtin
///   with a parameter (e.g. a design adds `pi`).
/// * `External("r", "p")` is left as-is — reference / parameter
///   existence is handled by the validation pass (`UndefinedReference`
///   / `UndefinedReferenceParameter`).
/// * `Parameter("x")` where `x` is neither a parameter nor a builtin
///   stays `Parameter`; validation surfaces it as `UndefinedParameter`.
fn classify_variable(variable: &mut ir::Variable, scope: &ClassifyScope<'_>) {
    use oneil_shared::symbols::BuiltinValueName;

    match variable {
        ir::Variable::Parameter {
            parameter_name,
            parameter_span,
        } => {
            if scope.parameters.contains(parameter_name) {
                // Already in correct form.
            } else if scope.builtins.has_builtin_value(parameter_name.as_str()) {
                let ident = BuiltinValueName::new(parameter_name.as_str().to_string());
                *variable = ir::Variable::Builtin {
                    ident,
                    ident_span: parameter_span.clone(),
                };
            }
            // Else: leave as `Parameter`; validation surfaces UndefinedParameter.
        }
        ir::Variable::Builtin { ident, ident_span } => {
            let name = ParameterName::from(ident.as_str());
            if scope.parameters.contains(&name) {
                *variable = ir::Variable::Parameter {
                    parameter_name: name,
                    parameter_span: ident_span.clone(),
                };
            }
        }
        ir::Variable::External { .. } => {}
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use indexmap::IndexSet;
    use oneil_ir as ir;
    use oneil_shared::{
        span::Span,
        symbols::{BuiltinValueName, ParameterName, ReferenceName},
    };

    use super::{BuiltinLookup, ClassifyScope, classify_variable};

    struct StubBuiltins {
        names: Vec<String>,
    }

    impl StubBuiltins {
        fn new(names: &[&str]) -> Self {
            Self {
                names: names.iter().map(|s| (*s).to_string()).collect(),
            }
        }
    }

    impl BuiltinLookup for StubBuiltins {
        fn has_builtin_value(&self, name: &str) -> bool {
            self.names.iter().any(|n| n == name)
        }
    }

    #[test]
    fn parameter_in_scope_stays_parameter() {
        let name = ParameterName::from("x");
        let mut params = IndexSet::new();
        params.insert(name.clone());
        let builtins = StubBuiltins::new(&[]);
        let scope = ClassifyScope {
            parameters: &params,
            builtins: &builtins,
        };

        let mut var = ir::Variable::parameter(name.clone(), Span::synthetic());
        classify_variable(&mut var, &scope);

        match var {
            ir::Variable::Parameter { parameter_name, .. } => {
                assert_eq!(parameter_name, name);
            }
            other @ (ir::Variable::Builtin { .. } | ir::Variable::External { .. }) => {
                panic!("expected Parameter, got {other:?}");
            }
        }
    }

    #[test]
    fn parameter_not_in_scope_but_matching_builtin_becomes_builtin() {
        let params = IndexSet::new();
        let builtins = StubBuiltins::new(&["pi"]);
        let scope = ClassifyScope {
            parameters: &params,
            builtins: &builtins,
        };

        let mut var = ir::Variable::parameter(ParameterName::from("pi"), Span::synthetic());
        classify_variable(&mut var, &scope);

        match var {
            ir::Variable::Builtin { ident, .. } => assert_eq!(ident.as_str(), "pi"),
            other @ (ir::Variable::Parameter { .. } | ir::Variable::External { .. }) => {
                panic!("expected Builtin reclassification, got {other:?}");
            }
        }
    }

    #[test]
    fn builtin_shadowed_by_parameter_becomes_parameter() {
        let mut params = IndexSet::new();
        params.insert(ParameterName::from("pi"));
        let builtins = StubBuiltins::new(&["pi"]);
        let scope = ClassifyScope {
            parameters: &params,
            builtins: &builtins,
        };

        let mut var =
            ir::Variable::builtin(BuiltinValueName::new("pi".to_string()), Span::synthetic());
        classify_variable(&mut var, &scope);

        match var {
            ir::Variable::Parameter { parameter_name, .. } => {
                assert_eq!(parameter_name.as_str(), "pi");
            }
            other @ (ir::Variable::Builtin { .. } | ir::Variable::External { .. }) => {
                panic!("expected Parameter reclassification, got {other:?}");
            }
        }
    }

    #[test]
    fn unknown_parameter_stays_unresolved() {
        let params = IndexSet::new();
        let builtins = StubBuiltins::new(&[]);
        let scope = ClassifyScope {
            parameters: &params,
            builtins: &builtins,
        };

        let mut var = ir::Variable::parameter(ParameterName::from("ghost"), Span::synthetic());
        classify_variable(&mut var, &scope);

        match var {
            ir::Variable::Parameter { parameter_name, .. } => {
                assert_eq!(parameter_name.as_str(), "ghost");
            }
            other @ (ir::Variable::Builtin { .. } | ir::Variable::External { .. }) => {
                panic!("expected Parameter, got {other:?}");
            }
        }
    }

    #[test]
    fn external_is_unchanged() {
        let params = IndexSet::new();
        let builtins = StubBuiltins::new(&[]);
        let scope = ClassifyScope {
            parameters: &params,
            builtins: &builtins,
        };

        let mut var = ir::Variable::external(
            ReferenceName::new("r".to_string()),
            Span::synthetic(),
            ParameterName::from("p"),
            Span::synthetic(),
        );
        classify_variable(&mut var, &scope);

        match var {
            ir::Variable::External {
                reference_name,
                parameter_name,
                ..
            } => {
                assert_eq!(reference_name.as_str(), "r");
                assert_eq!(parameter_name.as_str(), "p");
            }
            other @ (ir::Variable::Builtin { .. } | ir::Variable::Parameter { .. }) => {
                panic!("expected External, got {other:?}");
            }
        }
    }
}
