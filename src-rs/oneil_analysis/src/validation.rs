//! Post-build validation over a composed [`InstanceGraph`].
//!
//! Runs once the graph is fully built (all designs applied, every
//! unit's subtree inlined) and:
//!
//! 0. Reclassifies bare names against per-instance binding scopes via
//!    [`classify_variables`]: `Parameter("x")` becomes `Builtin("x")`
//!    when `x` is not a parameter on the host but is a known builtin,
//!    and `Builtin("x")` becomes `Parameter("x")` when a design has
//!    added `x` to the host (shadowing the builtin). This is folded
//!    into validation so the existence checks below see the canonical
//!    classification, and so a future shadowing warning can live next
//!    to the rest of the diagnostics it would join.
//! 1. Undefined bare names: `Parameter("x")` where `x` is neither in
//!    the host's parameter scope nor a builtin.
//! 2. Undefined reference names: `External { reference_name: r,
//!    parameter_name: p }` (source spelling `p.r`) where `r` is not a
//!    reference / alias on the host instance.
//! 3. Undefined reference parameters: same `External` shape where `r`
//!    resolves but `p` doesn't exist on the target instance.
//! 4. Parameter dependency cycles: SCC over `(host, parameter)` nodes
//!    where edges follow `Variable::Parameter` / `Variable::External`
//!    against each host's binding scope.
//!
//! Errors land on `graph.validation_errors`. The bucket is cleared at
//! the start of every pass so repeated calls are idempotent. The
//! classification step is itself idempotent — running it twice yields
//! the same IR.

use indexmap::{IndexMap, IndexSet};
use oneil_frontend::{
    BuiltinLookup, CycleMember, HostLocation, InstanceGraph, InstanceValidationError,
    InstanceValidationErrorKind, InstancedModel, classify_variables,
};
use oneil_ir as ir;
use oneil_shared::{
    InstancePath,
    paths::ModelPath,
    search::{SearchResult, search},
    span::Span,
    symbols::{ParameterName, ReferenceName},
};
use std::collections::HashMap;

/// Validates `graph` and pushes diagnostics onto
/// [`InstanceGraph::validation_errors`].
///
/// Before running the existence and cycle checks this also reclassifies
/// every bare-name variable against each instance's binding scope (see
/// [`classify_variables`]); design additions and builtin shadowing are
/// resolved at this point. The existence checks then see canonical IR.
///
/// The validation bucket is cleared first so repeated calls are
/// idempotent — and the classification mutation is idempotent too.
pub fn validate_instance_graph(graph: &mut InstanceGraph, builtins: &dyn BuiltinLookup) {
    classify_variables(graph, builtins);
    graph.validation_errors.clear();
    let mut collected: Vec<InstanceValidationError> = Vec::new();

    // Build a set of model paths that have file-time resolution errors.
    // `UndefinedReferenceParameter` is suppressed for targets in this set to
    // avoid spurious secondary errors when the target file itself is broken.
    let models_with_resolution_errors: IndexSet<ModelPath> =
        graph.resolution_errors.keys().cloned().collect();

    // Validate the root subtree (and any pool entries as separate sub-graphs).
    walk_subtree(
        &graph.root,
        &InstancePath::root(),
        &[],
        &graph.reference_pool,
        &models_with_resolution_errors,
        &mut collected,
    );
    for instance in graph.reference_pool.values() {
        // Pool entries get validated as their own self-rooted subtrees;
        // when a host references them (`p.r` in source), the existence
        // check on the host catches missing parameters.
        walk_subtree(
            instance,
            &InstancePath::root(),
            &[],
            &graph.reference_pool,
            &models_with_resolution_errors,
            &mut collected,
        );
    }

    // Cross-instance SCC over the parameter dependency graph.
    let dep = ParamDepGraph::build(graph);
    collect_cycle_errors(&dep, &mut collected);

    graph.validation_errors.extend(collected);
}

/// Pre-order traversal visiting every owned descendant via `submodels`.
/// Pool entries reached via `references` are *not* recursed into here;
/// they are walked separately as their own subtrees.
///
/// `ancestors` is the chain of parent instances from the root down to (but not
/// including) `node`. It is used to resolve `DesignProvenance::anchor_path` for
/// overlay parameters so that the overlay RHS is validated in the anchor's scope.
fn walk_subtree<'a>(
    node: &'a InstancedModel,
    host_path: &InstancePath,
    ancestors: &[&'a InstancedModel],
    pool: &IndexMap<ModelPath, Box<InstancedModel>>,
    models_with_resolution_errors: &IndexSet<ModelPath>,
    errors: &mut Vec<InstanceValidationError>,
) {
    validate_instance(
        node,
        host_path,
        ancestors,
        pool,
        models_with_resolution_errors,
        errors,
    );
    for (alias, sub) in node.submodels() {
        let child_path = host_path.clone().child(alias.clone());
        let mut child_ancestors = ancestors.to_vec();
        child_ancestors.push(node);
        walk_subtree(
            sub.instance.as_ref(),
            &child_path,
            &child_ancestors,
            pool,
            models_with_resolution_errors,
            errors,
        );
    }
}

fn validate_instance(
    instance: &InstancedModel,
    host_path: &InstancePath,
    ancestors: &[&InstancedModel],
    pool: &IndexMap<ModelPath, Box<InstancedModel>>,
    models_with_resolution_errors: &IndexSet<ModelPath>,
    errors: &mut Vec<InstanceValidationError>,
) {
    for (param_name, parameter) in instance.parameters() {
        let location = HostLocation::Parameter(param_name.clone());
        // For overlay parameters, validate the RHS in the anchor's scope
        // (the design's target) rather than the host's scope.
        let provenance = parameter.design_provenance();
        let anchor = provenance
            .and_then(|prov| resolve_anchor_for_validation(instance, ancestors, &prov.anchor_path));
        let design_info =
            provenance.map(|prov| (prov.design_path.clone(), prov.assignment_span.clone()));
        validate_value(
            parameter.value(),
            instance,
            anchor.unwrap_or(instance),
            host_path,
            &location,
            design_info.as_ref(),
            pool,
            models_with_resolution_errors,
            errors,
        );
    }
    for (test_index, test) in instance.tests() {
        let location = HostLocation::Test(*test_index);
        // Use design provenance to attribute errors to the design file
        // where the test was defined (if it came from a design).
        let provenance = test.design_provenance();
        let design_info =
            provenance.map(|prov| (prov.design_path.clone(), prov.assignment_span.clone()));
        validate_expr(
            test.expr(),
            instance,
            instance,
            host_path,
            &location,
            design_info.as_ref(),
            pool,
            models_with_resolution_errors,
            errors,
        );
    }
}

/// Resolves a `RelativePath` from the given `host` instance by walking the
/// `ancestors` chain upward and then descending through submodels.
///
/// Returns `None` if the path cannot be resolved. The most common cause is
/// that `path.up` exceeds the depth of the current traversal: an
/// `anchor_path` is recorded relative to the depth at which the design was
/// applied, but validation also walks pool entries as self-rooted subtrees
/// (`ancestors` empty). In that case the caller falls back to `host` itself,
/// which validates the overlay RHS against the pool entry's own scope — a
/// conservative but safe choice.
fn resolve_anchor_for_validation<'a>(
    host: &'a InstancedModel,
    ancestors: &[&'a InstancedModel],
    path: &oneil_shared::RelativePath,
) -> Option<&'a InstancedModel> {
    if path.is_self() {
        return Some(host);
    }
    let chain_len = ancestors.len();
    if path.up > chain_len {
        return None;
    }
    // Walk `up` steps to the appropriate ancestor.
    let mut node: &InstancedModel = ancestors[chain_len - path.up];
    // Then descend through `down` segments.
    for seg in &path.down {
        node = node.submodels().get(seg)?.instance.as_ref();
    }
    Some(node)
}

#[expect(
    clippy::too_many_arguments,
    reason = "private validation helper; all args are needed"
)]
fn validate_value(
    value: &ir::ParameterValue,
    // Instance that owns `value` — used for reference/submodel lookups in `Variable::External`.
    host_instance: &InstancedModel,
    // Instance in whose scope `Variable::Parameter` names are resolved. For
    // non-overlay parameters this is the same as `host_instance`; for overlay
    // parameters it is the design anchor.
    param_scope: &InstancedModel,
    host_path: &InstancePath,
    location: &HostLocation,
    // When `value` was contributed by a design, the design path and
    // assignment span for attribution of any errors found within `value`.
    design_info: Option<&(ModelPath, Span)>,
    pool: &IndexMap<ModelPath, Box<InstancedModel>>,
    models_with_resolution_errors: &IndexSet<ModelPath>,
    errors: &mut Vec<InstanceValidationError>,
) {
    match value {
        ir::ParameterValue::Simple(expr, _) => {
            validate_expr(
                expr,
                host_instance,
                param_scope,
                host_path,
                location,
                design_info,
                pool,
                models_with_resolution_errors,
                errors,
            );
        }
        ir::ParameterValue::Piecewise(piecewise, _) => {
            for piece in piecewise {
                validate_expr(
                    piece.expr(),
                    host_instance,
                    param_scope,
                    host_path,
                    location,
                    design_info,
                    pool,
                    models_with_resolution_errors,
                    errors,
                );
                validate_expr(
                    piece.if_expr(),
                    host_instance,
                    param_scope,
                    host_path,
                    location,
                    design_info,
                    pool,
                    models_with_resolution_errors,
                    errors,
                );
            }
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "private validation helper; all args are needed"
)]
fn validate_expr(
    expr: &ir::Expr,
    // Instance used for `Variable::External` reference resolution.
    instance: &InstancedModel,
    // Scope used for `Variable::Parameter` name resolution. For overlay
    // parameters this is the anchor; for regular parameters it is `instance`.
    param_scope: &InstancedModel,
    host_path: &InstancePath,
    location: &HostLocation,
    // When the enclosing parameter was contributed by a design, the design
    // path and assignment span so errors in this expression are attributed
    // to the design file instead of the host model.
    design_info: Option<&(ModelPath, Span)>,
    pool: &IndexMap<ModelPath, Box<InstancedModel>>,
    // Model paths that have file-time resolution errors; `UndefinedReferenceParameter`
    // is suppressed for these targets to avoid spurious secondary diagnostics.
    models_with_resolution_errors: &IndexSet<ModelPath>,
    errors: &mut Vec<InstanceValidationError>,
) {
    expr.walk_variables(&mut |variable| match variable {
        ir::Variable::Parameter {
            parameter_name,
            parameter_span,
        } => {
            if param_scope.parameters().contains_key(parameter_name) {
                return;
            }
            let best_match = best_match_parameter(param_scope.parameters(), parameter_name);
            errors.push(InstanceValidationError {
                host_path: host_path.clone(),
                host_location: location.clone(),
                kind: InstanceValidationErrorKind::UndefinedParameter {
                    parameter_name: parameter_name.clone(),
                    parameter_span: parameter_span.clone(),
                    best_match,
                    design_info: design_info.cloned(),
                },
            });
        }
        ir::Variable::External {
            reference_name,
            reference_span,
            parameter_name,
            parameter_span,
            ..
        } => {
            // Resolve the reference name. It can appear in any of the three maps:
            //  • `references` — cross-file reference declarations
            //  • `submodels`  — owned child subtrees
            //  • `aliases`    — extraction-list aliases (follow alias_path)
            #[expect(
                clippy::option_if_let_else,
                reason = "multi-arm chain; map_or_else would reduce clarity"
            )]
            let target_model = if let Some(r) = instance.references().get(reference_name) {
                Some(r.path.clone())
            } else if let Some(sub) = instance.submodels().get(reference_name) {
                Some(sub.instance.path().clone())
            } else if let Some(alias) = instance.aliases().get(reference_name) {
                // Alias path descends through submodels (or starts with a reference pool entry).
                resolve_alias_model(instance, alias, pool)
            } else {
                None
            };

            let Some(target_path) = target_model else {
                let best_match = best_match_reference_all(instance, reference_name);
                errors.push(InstanceValidationError {
                    host_path: host_path.clone(),
                    host_location: location.clone(),
                    kind: InstanceValidationErrorKind::UndefinedReference {
                        reference_name: reference_name.clone(),
                        reference_span: reference_span.clone(),
                        best_match,
                        design_info: design_info.cloned(),
                    },
                });
                return;
            };

            // Find the target instance. Submodels are in the tree (walk host_path +
            // the alias); cross-file references are in the pool.
            let target_opt = if instance.submodels().contains_key(reference_name) {
                // Owned submodel — navigate into the current node's submodels.
                instance
                    .submodels()
                    .get(reference_name)
                    .map(|sub| sub.instance.as_ref())
            } else if let Some(alias) = instance.aliases().get(reference_name) {
                resolve_alias_instance(instance, alias, pool)
            } else {
                pool.get(&target_path).map(std::convert::AsRef::as_ref)
            };

            let Some(target_instance) = target_opt else {
                // The only way to reach this branch is for a cross-file
                // `reference` whose pool entry was never inserted because
                // the reference's build traversal detected a compilation
                // cycle and executed `continue` instead of calling
                // `reference_pool.insert` (see `build_instance_subtree`).
                // A `CompilationCycleError` has already been recorded for
                // that reference, so emitting a secondary
                // `UndefinedReferenceParameter` here would be misleading.
                return;
            };
            if target_instance.parameters().contains_key(parameter_name) {
                return;
            }
            // Suppress `UndefinedReferenceParameter` when the target file has its own
            // resolution errors: those errors will already be reported on the target and
            // reporting a secondary error here would be misleading / redundant.
            if models_with_resolution_errors.contains(&target_path) {
                return;
            }
            let best_match = best_match_parameter(target_instance.parameters(), parameter_name);
            errors.push(InstanceValidationError {
                host_path: host_path.clone(),
                host_location: location.clone(),
                kind: InstanceValidationErrorKind::UndefinedReferenceParameter {
                    reference_name: reference_name.clone(),
                    reference_span: reference_span.clone(),
                    parameter_name: parameter_name.clone(),
                    parameter_span: parameter_span.clone(),
                    target_model: target_path,
                    best_match,
                    design_info: design_info.cloned(),
                },
            });
        }
        ir::Variable::Builtin { .. } => {}
    });
}

/// Resolves an alias to the model path of its target, by walking the
/// `alias_path` starting from `host`.
///
/// The first segment may refer to a direct submodel **or** to a cross-file
/// `reference` (pool entry).  Subsequent segments always descend through
/// `submodels` of the reached instance.
fn resolve_alias_model(
    host: &InstancedModel,
    alias: &oneil_frontend::AliasImport,
    pool: &IndexMap<ModelPath, Box<InstancedModel>>,
) -> Option<ModelPath> {
    let segs = alias.alias_path.segments();
    let mut iter = segs.iter();
    let first = iter.next()?;

    let mut node: &InstancedModel = if let Some(sub) = host.submodels().get(first) {
        sub.instance.as_ref()
    } else if let Some(r) = host.references().get(first) {
        pool.get(&r.path).map(std::convert::AsRef::as_ref)?
    } else {
        return None;
    };

    for seg in iter {
        node = node.submodels().get(seg)?.instance.as_ref();
    }
    Some(node.path().clone())
}

/// Walks an alias path from `host` and returns the target `InstancedModel`,
/// if reachable.
///
/// The first segment may refer to a direct submodel **or** to a cross-file
/// `reference` (pool entry).  Subsequent segments always descend through
/// `submodels` of the reached instance.
fn resolve_alias_instance<'a>(
    host: &'a InstancedModel,
    alias: &oneil_frontend::AliasImport,
    pool: &'a IndexMap<ModelPath, Box<InstancedModel>>,
) -> Option<&'a InstancedModel> {
    let segs = alias.alias_path.segments();
    let mut iter = segs.iter();
    let first = iter.next()?;

    let mut node: &InstancedModel = if let Some(sub) = host.submodels().get(first) {
        sub.instance.as_ref()
    } else if let Some(r) = host.references().get(first) {
        pool.get(&r.path).map(std::convert::AsRef::as_ref)?
    } else {
        return None;
    };

    for seg in iter {
        node = node.submodels().get(seg)?.instance.as_ref();
    }
    Some(node)
}

/// Fuzzy-matches `reference_name` against all three import maps (references,
/// submodels, aliases) and returns the closest name, if any.
fn best_match_reference_all(
    instance: &InstancedModel,
    reference_name: &ReferenceName,
) -> Option<ReferenceName> {
    use oneil_shared::search::search;
    let candidates: Vec<&str> = instance
        .references()
        .keys()
        .chain(instance.submodels().keys())
        .chain(instance.aliases().keys())
        .map(ReferenceName::as_str)
        .collect();
    search(reference_name.as_str(), &candidates)
        .and_then(|r| r.some_if_within_distance(2))
        .map(|s| ReferenceName::new(s.to_string()))
}

// ── Cross-instance SCC pass ─────────────────────────────────────────────────
//
// This section implements a cross-instance parameter-dependency cycle check
// using Tarjan's strongly-connected-components (SCC) algorithm (1972).
//
// # Why SCC and not DFS coloring?
//
// Cycle detection via DFS coloring (white/grey/black) is simpler but only
// works on a single traversal from one root. Here the graph has two
// structurally distinct regions:
//
//  • The *tree* — owned `submodel` instances rooted at the evaluation root,
//    each uniquely identified by their `InstancePath`.
//  • The *pool* — shared `reference` instances, each identified by their
//    `ModelPath` (they have one copy regardless of how many times they are
//    imported).
//
// Parameters in the tree can depend on pool parameters and vice-versa.
// SCC handles this naturally: we build one flat integer-indexed graph
// covering both regions and run Tarjan once over the whole thing.
//
// # Algorithm overview (Tarjan 1972)
//
// Each node gets an integer `index` (DFS discovery order) and a `lowlink`
// (the smallest index reachable from the node's subtree via back-edges).
// A node `v` is the *root* of an SCC when `lowlink[v] == index[v]`:
// nothing in `v`'s subtree can reach a node discovered before `v`.
// At that point everything on the auxiliary stack down to `v` belongs to
// the same SCC and is popped off together.
//
// The implementation here is *iterative* (explicit `work` stack) rather
// than recursive, to avoid stack-overflows on large models.
//
// # From SCC to cycle path
//
// Tarjan returns the *set* of nodes in each SCC, but not the ordered path.
// `find_cycle_path` / `dfs_back_to_start` do a second DFS within the SCC
// to extract one concrete `a → b → c → a` path for the error message.
// Each SCC member then gets its own `ParameterCycle` error with the same
// path rotated to start at that member (`rotate_cycle_to_start_at`).

/// Identifies a host instance for the dependency graph: either a node
/// in the root subtree (by `InstancePath`) or a pool entry (by
/// `ModelPath`).
///
/// The two variants cannot be unified into `EvalInstanceKey` here because
/// the SCC pass needs to build a flat integer-indexed adjacency list
/// upfront and look up nodes by key. Pool entries are addressed by their
/// on-disk path; tree nodes by their position in the submodel hierarchy.
/// Both can appear as edge endpoints, so edges crossing the boundary
/// (tree → pool, pool → tree) are correctly represented.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum HostId {
    Tree(InstancePath),
    Pool(ModelPath),
}

impl HostId {
    fn to_path(&self) -> InstancePath {
        match self {
            Self::Tree(p) => p.clone(),
            // Pool entries are validated as self-rooted subtrees, so their
            // logical host path is always the root.
            Self::Pool(_) => InstancePath::root(),
        }
    }
}

/// Design info attached to a node when the parameter's value was set by a design.
type DesignInfo = Option<(ModelPath, Span)>;

struct ParamDepGraph {
    /// One entry per `(host, parameter)` pair across all instances.
    ///
    /// Fields: `(host_id, parameter_name, name_span, model_path, design_info)`
    ///
    /// `design_info` is `Some((design_path, assignment_span))` when the
    /// parameter was last written by a design override or addition — used
    /// to attribute cycle errors to the design file rather than the host.
    nodes: Vec<(HostId, ParameterName, Span, ModelPath, DesignInfo)>,
    /// Adjacency list: `adj[i]` is the list of node indices that node `i`
    /// directly depends on (i.e. `i`'s parameter expression references those
    /// parameters). An edge `i → j` means "evaluating `i` requires `j`".
    adj: Vec<Vec<usize>>,
}

impl ParamDepGraph {
    /// Builds the dependency graph in two phases:
    ///
    /// **Phase 1 — node collection**: assigns a stable integer index to every
    /// `(host, parameter)` pair. Tree nodes are collected recursively via
    /// `collect_nodes_subtree`; pool nodes are added in a flat loop (pool
    /// entries do not have sub-entries of their own in the graph — their
    /// submodels are either also in the pool or are part of the root tree).
    ///
    /// **Phase 2 — edge collection**: for each node, walks the parameter's
    /// value expression and adds an edge to every `(host, parameter)` it
    /// depends on. Cross-boundary edges (tree → pool, pool → tree) are
    /// resolved by looking up the target in `node_index`.
    fn build(graph: &InstanceGraph) -> Self {
        let mut nodes: Vec<(HostId, ParameterName, Span, ModelPath, DesignInfo)> = Vec::new();
        let mut node_index: HashMap<(HostId, ParameterName), usize> = HashMap::new();

        // Phase 1a: tree nodes — recurse through the root's owned submodel hierarchy.
        collect_nodes_subtree(
            &graph.root,
            &mut |host_id, name, span, model, design_info| {
                node_index.insert((host_id.clone(), name.clone()), nodes.len());
                nodes.push((host_id, name, span, model, design_info));
            },
            &HostId::Tree(InstancePath::root()),
        );
        // Phase 1b: pool nodes — flat list; pool entries are shared, so each
        // on-disk path appears at most once regardless of how many tree nodes
        // import it.
        for (path, instance) in &graph.reference_pool {
            let host_id = HostId::Pool(path.clone());
            for (name, parameter) in instance.parameters() {
                node_index.insert((host_id.clone(), name.clone()), nodes.len());
                let design_info = parameter
                    .design_provenance()
                    .map(|p| (p.design_path.clone(), p.assignment_span.clone()));
                nodes.push((
                    host_id.clone(),
                    name.clone(),
                    parameter.name_span().clone(),
                    instance.path().clone(),
                    design_info,
                ));
            }
        }

        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
        // Phase 2a: edges for tree nodes.
        collect_edges_subtree(
            &graph.root,
            &HostId::Tree(InstancePath::root()),
            &graph.reference_pool,
            &node_index,
            &mut adj,
        );
        // Phase 2b: edges for pool nodes.
        for (path, instance) in &graph.reference_pool {
            let host_id = HostId::Pool(path.clone());
            collect_edges_for_instance(
                instance,
                &host_id,
                &graph.reference_pool,
                &node_index,
                &mut adj,
            );
        }

        Self { nodes, adj }
    }

    /// Runs Tarjan's SCC algorithm over `self` and returns the list of
    /// strongly-connected components as index vectors.
    ///
    /// # Algorithm (Tarjan 1972)
    ///
    /// Each node is assigned a unique `index` (DFS discovery order) and a
    /// `lowlink` value — the smallest `index` reachable from the node's
    /// subtree via tree or back edges. A node `v` is the *root* of an SCC
    /// exactly when `lowlink[v] == index[v]`: nothing below `v` in the DFS
    /// tree can escape to an ancestor of `v`. When that condition triggers,
    /// all nodes on the auxiliary `stack` down to and including `v` form one
    /// SCC and are popped together.
    ///
    /// The outer `for v in 0..n` loop handles disconnected graphs: nodes
    /// already visited (non-None `indices[v]`) are skipped.
    ///
    /// # Iterative implementation
    ///
    /// The recursive formulation would overflow the OS stack on deep models.
    /// The `work` stack simulates the call stack: each entry is
    /// `(node, next_edge_idx)`. On each iteration we either:
    ///  • **Advance**: pop `edge_idx`, push the unvisited neighbour onto
    ///    `work` (the recursive call), or update `lowlink` for an
    ///    already-on-stack neighbour (the back-edge case).
    ///  • **Return**: when all edges of `node` are exhausted (`edge_idx >=
    ///    adj[node].len()`), pop `node` from `work`, check if it's an SCC
    ///    root, and propagate its `lowlink` to its parent.
    ///
    /// # Returns
    ///
    /// Every node appears in exactly one component. Trivial singleton SCCs
    /// (nodes not part of any cycle) are included; callers must filter them
    /// (see `collect_cycle_errors`).
    fn tarjan_scc(&self) -> Vec<Vec<usize>> {
        let n = self.nodes.len();
        let mut index_counter: usize = 0;
        // `indices[v]` is None until v is first visited, then set to its DFS index.
        let mut indices: Vec<Option<usize>> = vec![None; n];
        // `lowlinks[v]` = min index reachable from v's subtree (init = index[v]).
        let mut lowlinks: Vec<usize> = vec![0; n];
        // `on_stack[v]` tracks membership on the auxiliary SCC-building stack.
        // Distinct from `work`: a node leaves `on_stack` only when its SCC is finalized.
        let mut on_stack: Vec<bool> = vec![false; n];
        let mut stack: Vec<usize> = Vec::new();
        let mut sccs: Vec<Vec<usize>> = Vec::new();
        // Explicit DFS work stack: `(node, index of next edge to explore)`.
        let mut work: Vec<(usize, usize)> = Vec::new();

        for v in 0..n {
            if indices[v].is_some() {
                continue; // already visited in a prior DFS root
            }
            // Seed DFS from v.
            indices[v] = Some(index_counter);
            lowlinks[v] = index_counter;
            index_counter += 1;
            stack.push(v);
            on_stack[v] = true;
            work.push((v, 0));

            while let Some(&(node, edge_idx)) = work.last() {
                if edge_idx < self.adj[node].len() {
                    // There are still outgoing edges to explore from `node`.
                    let w = self.adj[node][edge_idx];
                    // Advance the edge cursor before potentially pushing `w`
                    // so when we return to `node` we look at the next edge.
                    if let Some(top) = work.last_mut() {
                        top.1 += 1;
                    }
                    match indices[w] {
                        None => {
                            // Tree edge: `w` is unvisited — recurse into it.
                            indices[w] = Some(index_counter);
                            lowlinks[w] = index_counter;
                            index_counter += 1;
                            stack.push(w);
                            on_stack[w] = true;
                            work.push((w, 0));
                        }
                        Some(w_index) if on_stack[w] => {
                            // Back edge to an ancestor still on the stack:
                            // `w` is reachable from `node`, so update lowlink.
                            lowlinks[node] = lowlinks[node].min(w_index);
                        }
                        Some(_) => {
                            // Cross/forward edge to a node already assigned to
                            // a completed SCC; no lowlink update needed.
                        }
                    }
                } else {
                    // All edges from `node` are exhausted — "return" from this frame.
                    work.pop();
                    if Some(lowlinks[node]) == indices[node] {
                        // `node` is the root of an SCC: pop the stack down to it.
                        let mut component = Vec::new();
                        loop {
                            let w = stack.pop().expect("stack non-empty during SCC pop");
                            on_stack[w] = false;
                            component.push(w);
                            if w == node {
                                break;
                            }
                        }
                        sccs.push(component);
                    }
                    // Propagate lowlink upward to the caller frame.
                    if let Some(&(parent, _)) = work.last() {
                        lowlinks[parent] = lowlinks[parent].min(lowlinks[node]);
                    }
                }
            }
        }

        sccs
    }

    /// Extracts one concrete ordered cycle path from an SCC member set.
    ///
    /// `tarjan_scc` tells us *which* nodes are in a cycle but not the order.
    /// For a useful error message we want `a → b → c → a`, not just `{a,b,c}`.
    ///
    /// Strategy: DFS from `scc[0]` restricted to nodes in the SCC, looking
    /// for a path that returns to the start (`dfs_back_to_start`). Because
    /// every node in a non-trivial SCC can reach every other node by
    /// definition, the DFS is guaranteed to find such a path.
    ///
    /// For the self-loop case (`scc.len() == 1`) the single-element vec is
    /// returned immediately — there is no multi-node path to find.
    ///
    /// Note: this finds *a* cycle through `scc[0]`, not necessarily the
    /// *shortest* cycle. The full SCC may contain several interleaved cycles;
    /// the DFS picks whichever it encounters first. That is fine for error
    /// reporting: showing one concrete cycle is enough to diagnose the problem.
    fn find_cycle_path(&self, scc: &[usize]) -> Vec<usize> {
        if scc.is_empty() {
            return Vec::new();
        }
        if scc.len() == 1 {
            // Self-loop: the single node IS the cycle.
            return scc.to_vec();
        }
        let scc_set: std::collections::HashSet<usize> = scc.iter().copied().collect();
        let start = scc[0];
        let mut path: Vec<usize> = vec![start];
        let mut visited: std::collections::HashSet<usize> = std::collections::HashSet::new();
        visited.insert(start);
        if dfs_back_to_start(start, start, &self.adj, &scc_set, &mut path, &mut visited) {
            path.pop(); // the DFS pushes `start` again when it closes the loop
            return path;
        }
        // Unreachable for a valid Tarjan SCC with ≥ 2 members (every node
        // in the SCC can reach every other node by definition). Falling
        // back to the raw SCC slice avoids a panic and still produces a
        // diagnostic, albeit without a clean cycle ordering.
        scc.to_vec()
    }
}

fn collect_cycle_errors(dep: &ParamDepGraph, errors: &mut Vec<InstanceValidationError>) {
    let sccs = dep.tarjan_scc();
    for scc in sccs {
        // Tarjan's algorithm returns every node as a trivial singleton SCC
        // even when it has no cycle. Only report an error when the SCC
        // contains more than one node (mutual cycle) or the single node has
        // a self-edge (direct self-reference: `x = f(x)`).
        let is_self_loop = scc.len() == 1 && dep.adj[scc[0]].contains(&scc[0]);
        if scc.len() < 2 && !is_self_loop {
            continue;
        }
        let cycle_path = dep.find_cycle_path(&scc);
        for &member_idx in &scc {
            let (host_id, parameter_name, parameter_span, _, design_info) = &dep.nodes[member_idx];
            let rotated = rotate_cycle_to_start_at(&cycle_path, member_idx, dep);
            errors.push(InstanceValidationError {
                host_path: host_id.to_path(),
                host_location: HostLocation::Parameter(parameter_name.clone()),
                kind: InstanceValidationErrorKind::ParameterCycle {
                    parameter_name: parameter_name.clone(),
                    parameter_span: parameter_span.clone(),
                    design_info: design_info.clone(),
                    cycle: rotated,
                },
            });
        }
    }
}

/// Rotates `cycle_path` so that `start_idx` is first, then converts each
/// node index to a `CycleMember` for inclusion in the error variant.
///
/// Every member of an SCC gets its own `ParameterCycle` error (so the IDE
/// can highlight each offending parameter individually), but each error
/// also carries the full cycle description. By rotating the shared path to
/// start at the parameter being reported, the description reads naturally:
/// "parameter `x` depends on `y`, which depends on `x`".
///
/// If `start_idx` is not in `cycle_path` (shouldn't happen for a well-formed
/// SCC result), `unwrap_or(0)` makes it start at position 0 rather than panic.
fn rotate_cycle_to_start_at(
    cycle_path: &[usize],
    start_idx: usize,
    dep: &ParamDepGraph,
) -> Vec<CycleMember> {
    let start_pos = cycle_path.iter().position(|&i| i == start_idx).unwrap_or(0);
    let mut rotated = Vec::with_capacity(cycle_path.len());
    for offset in 0..cycle_path.len() {
        let idx = cycle_path[(start_pos + offset) % cycle_path.len()];
        let (host_id, name, _, model, _) = &dep.nodes[idx];
        rotated.push(CycleMember {
            host_path: host_id.to_path(),
            model: model.clone(),
            parameter_name: name.clone(),
        });
    }
    rotated
}

/// DFS helper for `find_cycle_path`.
///
/// Walks from `node` through edges in `adj`, staying within `scc_set`, until
/// it finds an edge back to `target`. Builds the path in `path` as it
/// descends and prunes (backtracks) on dead ends.
///
/// The `path.len() >= 2` guard on the `next == target` check prevents
/// immediately returning on the very first edge out of the start node before
/// exploring any intermediate nodes — we need at least one intermediate step
/// to form a real cycle (or a self-loop is handled before calling this).
///
/// `visited` prevents re-entering nodes, keeping the DFS acyclic within this
/// search (we're looking for *one* path, not enumerating all cycles).
fn dfs_back_to_start(
    node: usize,
    target: usize,
    adj: &[Vec<usize>],
    scc_set: &std::collections::HashSet<usize>,
    path: &mut Vec<usize>,
    visited: &mut std::collections::HashSet<usize>,
) -> bool {
    for &next in &adj[node] {
        if !scc_set.contains(&next) {
            continue; // stay within the SCC
        }
        if next == target && path.len() >= 2 {
            // Found the back-edge to `target`; push it so the caller can pop
            // it off to get the clean open path `[start, ..., node]`.
            path.push(next);
            return true;
        }
        if visited.contains(&next) {
            continue; // already on the current path or a dead end
        }
        visited.insert(next);
        path.push(next);
        if dfs_back_to_start(next, target, adj, scc_set, path, visited) {
            return true;
        }
        // Backtrack.
        path.pop();
        visited.remove(&next);
    }
    false
}

/// Registers one graph node per parameter of `node`, then recurses into
/// owned submodels if `host_id` is a `Tree` variant.
///
/// Pool entries (`HostId::Pool`) are *not* recursed into here — their
/// parameters are registered directly by the flat loop in `ParamDepGraph::build`.
/// Pool entries are indexed under `HostId::Pool(model_path)`, not as tree
/// children, so recursing here would try to register them under
/// `HostId::Tree(root.child(alias))`, producing duplicate or mismatched keys.
fn collect_nodes_subtree<F: FnMut(HostId, ParameterName, Span, ModelPath, DesignInfo)>(
    node: &InstancedModel,
    f: &mut F,
    host_id: &HostId,
) {
    for (name, parameter) in node.parameters() {
        let design_info = parameter
            .design_provenance()
            .map(|p| (p.design_path.clone(), p.assignment_span.clone()));
        f(
            host_id.clone(),
            name.clone(),
            parameter.name_span().clone(),
            node.path().clone(),
            design_info,
        );
    }
    if let HostId::Tree(host_path) = host_id {
        for (alias, sub) in node.submodels() {
            let child_path = host_path.clone().child(alias.clone());
            collect_nodes_subtree(sub.instance.as_ref(), f, &HostId::Tree(child_path));
        }
    }
}

fn collect_edges_subtree(
    node: &InstancedModel,
    host_id: &HostId,
    pool: &IndexMap<ModelPath, Box<InstancedModel>>,
    node_index: &HashMap<(HostId, ParameterName), usize>,
    adj: &mut [Vec<usize>],
) {
    collect_edges_for_instance(node, host_id, pool, node_index, adj);
    if let HostId::Tree(host_path) = host_id {
        for (alias, sub) in node.submodels() {
            let child_path = host_path.clone().child(alias.clone());
            collect_edges_subtree(
                sub.instance.as_ref(),
                &HostId::Tree(child_path),
                pool,
                node_index,
                adj,
            );
        }
    }
}

fn collect_edges_for_instance(
    instance: &InstancedModel,
    host_id: &HostId,
    pool: &IndexMap<ModelPath, Box<InstancedModel>>,
    node_index: &HashMap<(HostId, ParameterName), usize>,
    adj: &mut [Vec<usize>],
) {
    for (name, parameter) in instance.parameters() {
        let host_idx_key = (host_id.clone(), name.clone());
        let Some(&host_idx) = node_index.get(&host_idx_key) else {
            // Every parameter present during node collection should be in
            // the index, so this branch is unreachable under normal
            // conditions. Skip defensively rather than panicking.
            continue;
        };
        collect_param_edges(
            parameter,
            instance,
            host_id,
            pool,
            node_index,
            &mut adj[host_idx],
        );
    }
}

fn collect_param_edges(
    parameter: &ir::Parameter,
    instance: &InstancedModel,
    host_id: &HostId,
    pool: &IndexMap<ModelPath, Box<InstancedModel>>,
    node_index: &HashMap<(HostId, ParameterName), usize>,
    out: &mut Vec<usize>,
) {
    let mut visit = |variable: &ir::Variable| match variable {
        ir::Variable::Parameter { parameter_name, .. } => {
            if instance.parameters().contains_key(parameter_name)
                && let Some(&idx) = node_index.get(&(host_id.clone(), parameter_name.clone()))
            {
                out.push(idx);
            }
        }
        ir::Variable::External {
            reference_name,
            parameter_name,
            ..
        } => {
            // Resolve reference_name across all three maps.
            if let Some(_sub) = instance.submodels().get(reference_name) {
                // Submodel: the dependency target is the submodel node
                // directly beneath `host_id`.  Pool entries are validated
                // as self-rooted sub-graphs (ancestors empty, host_path =
                // root), so their submodels are indexed as
                // `Tree(root.child(alias))` — the same convention used
                // when `collect_nodes_subtree` recurses into them.
                let target_id = match host_id {
                    HostId::Tree(p) => HostId::Tree(p.clone().child(reference_name.clone())),
                    HostId::Pool(_) => {
                        HostId::Tree(InstancePath::root().child(reference_name.clone()))
                    }
                };
                if let Some(&idx) = node_index.get(&(target_id, parameter_name.clone())) {
                    out.push(idx);
                }
            } else if let Some(alias) = instance.aliases().get(reference_name) {
                // Alias: the first segment may be a submodel (tree) or a reference
                // (pool entry). Walk accordingly to resolve the dependency target.
                let segs = alias.alias_path.segments();
                let mut seg_iter = segs.iter();
                if let Some(first) = seg_iter.next() {
                    if instance.submodels().contains_key(first) {
                        // First segment is a submodel — fold remaining segments into
                        // a tree path from the host's current position.
                        let base_path = match host_id {
                            HostId::Tree(p) => p.clone().child(first.clone()),
                            HostId::Pool(_) => InstancePath::root().child(first.clone()),
                        };
                        let target_path =
                            seg_iter.fold(base_path, |acc, seg| acc.child(seg.clone()));
                        let target_id = HostId::Tree(target_path);
                        if let Some(&idx) = node_index.get(&(target_id, parameter_name.clone())) {
                            out.push(idx);
                        }
                    } else if let Some(r) = instance.references().get(first) {
                        // First segment is a reference — the rest descend through the
                        // pool entry's submodels. We address the final instance by its
                        // ModelPath in the pool.
                        let mut target_path = r.path.clone();
                        let mut found = true;
                        for seg in seg_iter {
                            if let Some(pool_entry) = pool.get(&target_path) {
                                if let Some(sub) = pool_entry.submodels().get(seg) {
                                    target_path = sub.instance.path().clone();
                                } else {
                                    found = false;
                                    break;
                                }
                            } else {
                                found = false;
                                break;
                            }
                        }
                        if found {
                            let target_id = HostId::Pool(target_path);
                            if let Some(&idx) = node_index.get(&(target_id, parameter_name.clone()))
                            {
                                out.push(idx);
                            }
                        }
                    }
                }
            } else if let Some(target_path) = instance
                .references()
                .get(reference_name)
                .map(|r| r.path.clone())
            {
                let target_id = HostId::Pool(target_path.clone());
                if pool.contains_key(&target_path)
                    && let Some(&idx) = node_index.get(&(target_id, parameter_name.clone()))
                {
                    out.push(idx);
                }
            }
        }
        ir::Variable::Builtin { .. } => {}
    };
    walk_parameter_value(parameter.value(), &mut visit);
}

fn walk_parameter_value<F: FnMut(&ir::Variable)>(value: &ir::ParameterValue, f: &mut F) {
    match value {
        ir::ParameterValue::Simple(expr, _) => expr.walk_variables(f),
        ir::ParameterValue::Piecewise(piecewise, _) => {
            for piece in piecewise {
                piece.expr().walk_variables(f);
                piece.if_expr().walk_variables(f);
            }
        }
    }
}

// ── Existence-check helpers ─────────────────────────────────────────────────

fn best_match_parameter(
    parameters: &IndexMap<ParameterName, ir::Parameter>,
    query: &ParameterName,
) -> Option<ParameterName> {
    let candidates: Vec<&str> = parameters.keys().map(ParameterName::as_str).collect();
    best_match(&candidates, query.as_str()).map(ParameterName::from)
}

fn best_match<'c>(candidates: &[&'c str], query: &str) -> Option<&'c str> {
    let result = search(query, candidates)?;
    let max_distance = (query.len() / 3).max(1);
    match result {
        SearchResult::Exact(s) => Some(s),
        SearchResult::Fuzzy { result, distance } if distance <= max_distance => Some(result),
        SearchResult::Fuzzy { .. } => None,
    }
}
