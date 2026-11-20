# Two-Pass Instance Graph: Per-Unit Cache + Composition

## Status

**Accepted and implemented.** `oneil_frontend::instance::graph` implements the
architecture described here. See [Implementation](#implementation) for current
state and [Planned follow-ups](#planned-follow-ups) for open items.

## Context

`InstanceGraph` is the single structural representation between the file-static
resolver pass and evaluation. Three tensions in a naive single-pass walk
motivate the two-pass design:

1. **The IDE doesn't know the root.** LSP open-file flows want diagnostics for
   whatever file the user is editing, before any user-supplied root is chosen.
   A single-pass walk only produces diagnostics in the context of a specific
   root, so opening `child.on` requires picking a synthetic root or re-walking
   speculatively.
2. **Caching is lopsided without per-unit graphs.** A model-keyed template
   cache stripped of per-instance design effects requires runtime side tables to
   re-attach per-eval diagnostics to the instances they came from. Side tables
   are model-keyed, diagnostics are instance-keyed, and dedup against file-time
   errors grows messier with each new diagnostic class.
3. **File-local vs composition errors aren't separated.** "This file doesn't
   compile on its own" and "this file doesn't compose with the parent that
   applied this design" deserve distinct user-facing stories. A single-pass
   design surfaces them through the same bucket, which makes per-file LSP
   rendering hard and hides the distinction from the user.

The fix is one more layer of structure: cache an `InstanceGraph` per
compilation unit (file or design), with the convention that the unit is its own
root, and assemble user-rooted graphs on demand by recursively splicing cached
subgraphs and overlaying parent-supplied designs.

## Decision

The pipeline has two passes before evaluation:

```text
Per-unit build  →  Composition  →  Validate  →  Eval
   (cached)       (per eval call)
```

`build_unit_graph(unit)` produces an `InstanceGraph` rooted at
the compilation unit. Result is cached by `CompilationUnit`.

`apply_designs(root_unit, runtime_designs)` produces a user-rooted
`InstanceGraph` by cloning the cached root unit graph and applying runtime
designs as a final overlay pass.

### `InstanceGraph` shape

```rust
struct InstanceGraph {
    /// The root model instance and its entire owned subtree.
    root: Box<InstancedModel>,
    /// Shared cross-file references, keyed by their canonical model path.
    /// References are not owned by any one parent; multiple importers of the
    /// same file point into the same pool entry.
    reference_pool: IndexMap<ModelPath, Box<InstancedModel>>,
    /// Compilation-cycle errors detected during the per-unit build.
    cycle_errors: Vec<CompilationCycleError>,
    /// File-time resolver errors, keyed by model path.
    resolution_errors: IndexMap<ModelPath, ResolutionErrorCollection>,
    /// Design contribution diagnostics (overlay-target-missing, unit mismatch).
    contribution_errors: Vec<ContributionDiagnostic>,
    /// Post-composition validation errors (undefined params/references, cycles).
    validation_errors: Vec<InstanceValidationError>,
    /// For design-keyed cache entries: the resolved `Design` ready for application.
    design_export: Option<Design>,
}
```

### `InstancedModel` — the three-map structure

Each model instance in the graph or pool is an `InstancedModel`:

```rust
struct InstancedModel {
    /// Owned child subtrees, one per `submodel foo as bar` declaration.
    submodels: IndexMap<ReferenceName, SubmodelImport>,
    /// Cross-file pointer imports (`reference foo as r`), whose instances
    /// live in the containing graph's `reference_pool`.
    references: IndexMap<ReferenceName, ReferenceImport>,
    /// Local aliases for subpaths within this instance's subtree, created
    /// by extraction-list items in `submodel a as c [b as x]` syntax.
    aliases: IndexMap<ReferenceName, AliasImport>,
    /// Parameters (own and design-overlaid) with their IR expressions.
    parameters: IndexMap<ParameterName, ir::Parameter>,
    // …path, tests, etc.
}
```

The extraction-list syntax (`submodel a as c [b as x, b.sub as y]`) creates
`AliasImport` entries, each holding a relative `InstancePath` descending from
the host into the owned subtree — no new instance is created, no copy is made.
Dotted paths are allowed in extraction items (`b.sub as y`) but not in the
main model name (`a` must be a plain identifier).

### Parameter access syntax

Oneil uses a subscript-style parameter access, **not** OOP-style:

```
value.reference      # ✓ parameter `value` accessed via `reference`
reference.value      # ✗ OOP-style (never correct in Oneil source)
```

This keeps the quantity — the thing you actually care about — at the front.
`Variable::External` in IR stores `{ reference_name, parameter_name, .. }`
where the source spelling is `parameter_name.reference_name`.

### `build_unit_graph(unit)`

Walks the unit's AST and produces an `InstanceGraph` rooted at the unit.
For each child the unit pulls in (submodel, reference, extraction-alias,
design's target):

1. Recursively obtain that child's cached graph via `build_unit_graph(child)`.
2. Determine the contributions the unit supplies to that child (own-`apply`
   statements landing on this anchor).
3. Insert the child's root or pool entries directly into the unit's graph at the
   appropriate position in the tree (for submodels) or pool (for references).
4. Apply the contributions in-place.

Children are fully composed under the unit's local applies *before* the unit
links its own expressions. So when `parent.on` writes `g.planet` after
`apply augment to planet`, the `planet` instance already has `g` from `augment`
and the lookup resolves cleanly file-locally.

`.one` design files are also units. `build_unit_graph(design)` produces "the
design's target with this design applied at the root" — concretely, the target's
unit graph with the design's contributions overlaid. One cache abstraction, two
source kinds.

#### Cycle detection

Per-build stack of `(CompilationUnit, imported_at: Span)` values
(`CycleStackFrame`). Entering a unit pushes; finishing pops. Encountering a
unit already on the stack is an error, attributed to the cycle *target* — the
file the cycle closes back onto — at that file's outgoing-reference span
(carried along the stack as the predecessor frame's `imported_at`). The
recorded `CompilationCycleError` carries the full cycle chain in the message
body. This catches both file/file mutual references and file/design mutual
references under one rule.

Each file separately reaches its own root build through the cache (LSP
open-file flows, CLI invocations against a specific file), so any one detection
only needs to attribute the cycle to its target — the cache round-trip covers
the other participants.

### `apply_designs(root_unit, runtime_designs)`

Clones the cached `root_unit` graph into a fresh `InstanceGraph` and applies
`runtime_designs` as contributions at the root. No-op when `runtime_designs` is
empty, in which case the cached root unit graph *is* the composed graph (modulo
a cheap clone).

The composed graph is **not cached**. Composition is clone + overlay over
already-cached structures and runs in time linear in instance count. If
profiling later shows pain, add a memo keyed by `(root_unit, runtime_design_set)`
on top — the per-unit cache stays the source of truth.

### `DesignProvenance` and binding scope

When a design overlays a parameter, the overlay carries a `DesignProvenance`:

```rust
struct DesignProvenance {
    /// Span of the assignment in the design file.
    assignment_span: Span,
    /// Path *relative* to the overlay host, identifying the lexical scope
    /// in which the RHS should be evaluated. Kept relative so that a
    /// cached unit graph can be embedded under any parent without rebasing.
    anchor_path: RelativePath,
    /// `apply X to Y` statement that produced this overlay, when one
    /// exists. `None` only for the synthetic CLI design-as-root path.
    applied_via: Option<DesignApplication>,
}
```

`anchor_path` is a `RelativePath { up: usize, down: Vec<ReferenceName> }` —
the number of steps to walk up the parent chain followed by descending into
the named children. Using a relative path means the cached unit graph can be
spliced under any parent without rebasing absolute keys.

At eval time, `force_parameter` resolves `anchor_path` against the current
model's `parent_key` chain and temporarily pushes the anchor's
`EvalInstanceKey` onto the eval scope, so the RHS is evaluated in the design's
original lexical scope.

### Error partitioning

Two distinct buckets, with distinct lifetimes:

- **File-local** (cached on each unit's `InstanceGraph`):
  - parse / lower
  - own-`apply` target existence
  - own-`apply` overlay-target-missing (message + best-match suggestion +
    design assignment span)
  - dimensional grammar
- **Composition** (computed per `eval_model` / `check_model` call):
  - runtime-overlay-target-missing (same diagnostic, both callsites)
  - runtime-overlay unit mismatch
  - composition-introduced cycles
- **Post-composition validation** (run once after composition via
  `validate_instance_graph`):
  - reclassification of bare names against each instance's final scope
  - undefined parameters (`Variable::Parameter` not in binding scope)
  - undefined cross-instance references (`p.r` where `r` not in any import map)
  - undefined cross-instance reference parameters (`p.r` where `p` not in `r`'s
    model); locally-applied designs are already visible here
  - SCC over parameter dependencies, both file-local and cross-instance

All error buckets live on the graph itself, not on individual `InstancedModel`
nodes. This lets `get_model_diagnostics` read a single structure rather than
traversing the tree.

### No runtime side tables

`Runtime` holds no `*_errors` fields. Instead it holds
`composed_graph: Option<InstanceGraph>` populated by `eval_model_internal` and
`check_model_internal`. `get_model_diagnostics` reads the four error buckets
(`cycle_errors`, `resolution_errors`, `contribution_errors`,
`validation_errors`) straight off the composed graph, falling back through
`unit_graph_cache` for paths not yet reached by composition.

### Cache shape

```rust
enum CompilationUnit {
    Model(ModelPath),
    Design(DesignPath),
}

type UnitGraphCache = IndexMap<CompilationUnit, InstanceGraph>;
```

One map, one source of structural truth per unit. Because cached graphs are self-rooted, the cache supports per-unit invalidation:

- Edit `foo.on` → invalidate `CompilationUnit::Model(foo)` and the units that depend on it.
- Edit `bar.one` → invalidate `CompilationUnit::Design(bar)` and the units that depend on it.

The runtime currently clears the whole cache on any source change; see [Per-unit invalidation](#per-unit-invalidation). Anyone composing with an affected unit re-composes on the next `eval_model` / `check_model` call (composition is cheap).

### Settled semantics

1. **Cross-instance `p.r` against design-added params (own-file).** A file
   applying a design to one of its own children fully instantiates that child
   *and* applies the design before the parent links its own expressions.
   Patterns like `apply augment to planet; g.planet` resolve file-locally
   because `planet`'s post-augment scope is what the linker sees. The pattern
   *not* allowed is "this file references a name only added by an *external*
   runtime design": that's a validation-time error, because the cached unit
   graph has nothing to bind to until the composed graph is available.
2. **Mutual references between files / designs.** Per-build stack;
   revisit-while-on-stack is an error. Neither file in a mutual pair can
   complete its own self-rooted build, so the cycle is reported.
3. **`.one` design file caching.** A design is a compilation unit like a model
   file. Its cached `InstanceGraph` is "the design's target model with this
   design applied at root." Same cache, same primitive, no separate
   `design_info` table at the runtime layer.
4. **Design-provenance relative paths.** `DesignProvenance::anchor_path` is
   always relative to the overlay host (`RelativePath { up, down }`). This
   ensures that when a cached unit graph is used as a submodel under any parent,
   no rebase of design provenances is needed.
5. **Composition cache.** Not cached. Only per-unit graphs are cached. Revisit
   on profiling data.
6. **`references` vs `submodels` vs `aliases`.** A `reference` import declares
   a shared cross-file instance (lives in `reference_pool`); multiple models
   that `reference` the same file observe the *same* instance — overlay on
   one is visible from all aliases. A `submodel` import declares an *owned*
   child (lives in the parent's subtree as a `Box<InstancedModel>`); each
   occurrence is an independent instance.    An `alias` (from an extraction-list item like `submodel a as c [b as x]`)
   names an existing path within the subtree via a relative `InstancePath` —
   no new instance is created.

## Implementation

`src/oneil_frontend/src/instance/graph.rs` implements the full architecture
described above. Key pieces:

- **Entry points.** `build_unit_graph(unit, &mut UnitGraphCache, &mut stack, …)`
  and `apply_designs(root, runtime_designs, &mut UnitGraphCache, …)`. The legacy
  `build_instance_graph` is a thin convenience wrapper around `apply_designs`
  that creates an ephemeral cache; retained for external test use.

- **Tree assembly.** `merge_child_graph(parent, child_graph, anchor)` inserts the
  child's root into the parent's subtree at `anchor` and merges the child's
  `reference_pool` entries (deduplicating by path). Cycle errors and
  contribution errors are deduplicated during merges so each error appears at
  most once regardless of how many parents share the same child.

- **`UnitGraphCache`.** Lives on `Runtime` as `unit_graph_cache`; cleared
  alongside the eval cache in `Runtime::clear_non_source_caches`.

- **`composed_graph`.** `Runtime` holds `composed_graph: Option<InstanceGraph>`
  populated by each `eval_model_internal` / `check_model_internal`.
  `get_model_diagnostics` reads the four error buckets directly off it.

- **`CompilationCycleError`.** First-class cycle errors with
  `cycle: Vec<CompilationUnit>` and `(unit, imported_at)` stack frames,
  attributed to the cycle target at that file's outgoing-reference span. Lives
  in `oneil_frontend::instance::cycle_error`; re-exported from the crate root.
  `merge_child_graph` propagates cycle errors upward with deduplication so the
  composed graph carries every cycle observed anywhere in the dependency tree.

- **Design unit build.** `build_design_unit_graph` caches a graph rooted at
  the design's *target model* under a `Design(d)` cache entry. Two overlay
  passes run after the target's unit graph is spliced in: the design's own
  resolved contributions (parameter overrides and additions), then the design
  file's own `apply X to ref` declarations.

- **`DesignProvenance` and apply attribution.** `oneil_ir::DesignProvenance`
  carries `assignment_span` + `anchor_path: RelativePath` +
  `applied_via: Option<DesignApplication>`. The build pass writes the
  originating apply (the `apply X to Y` statement that brought this
  contribution in) when one exists; `None` is reserved for the synthetic
  CLI design-as-root path. `get_model_diagnostics` uses
  `surface_contribution_diagnostic` to render the precise diagnostic at the
  design file and a generic "applied design produced invalid contributions"
  diagnostic at the apply site. The same primitive (`surface_apply_hop`)
  covers both `ContributionDiagnostic` (overlay-target-missing +
  unit-mismatch) and validation errors against design-overlaid parameters.

  Multi-hop chains across cache boundaries are deliberately not modelled
  here. A child unit's contributions are baked into its cached graph with
  their original 1-hop `applied_via` and reused as-is when the unit is
  included as a submodel. Cross-cache propagation is handled separately
  by the "submodel `<alias>` has errors" notification emitted at the
  parent's import declaration in `emit_submodel_import_notifications`.

- **Resolution errors on graph.** `InstanceGraph::resolution_errors:
  IndexMap<ModelPath, ResolutionErrorCollection>` carries file-time resolver
  diagnostics alongside the graph. `merge_child_graph` merges child entries in.
  `get_model_diagnostics` prefers the composed graph, falls back to the per-unit
  cached graph, then `ir_cache` for paths not yet reached by composition.

- **`validate_instance_graph`.** Runs cross-instance `p.r` existence and
  cross-instance SCC (Tarjan-style) once per root composition in
  `oneil_analysis`, pushing errors into `InstanceGraph::validation_errors`.
  Traverses both `root` subtree and `reference_pool`. No per-unit caching —
  any external overlay can shift target param sets, RHS expressions, or
  retargeted refs. Uses `HostId { Tree(InstancePath) | Pool(ModelPath) }` to
  uniformly identify each host across the two storage locations.

- **`check_model` / `oneil check`.** Diagnostic-only entry point that runs the
  compose + validate pass without the eval pass. Used by LSP
  `Backend::publish_diagnostics_for_model_path` for `did_open` / `did_save`
  feedback, and by `oneil check <file>` (alias `c`, with `--design` / `-d`
  mirroring `oneil eval --design`). Returns the unique model paths visited in
  the composed graph as the clear-set. Exits 1 on any diagnostic.

- **Module layout.** `oneil_frontend::error` is exclusively the resolver-time
  error bucket. `CompilationCycleError` lives in
  `oneil_frontend::instance::cycle_error`. `oneil_frontend::resolver`'s module
  header states its responsibility as file-static lowering only (cross-file
  cycle detection is delegated to the instance pass).

- **Snapshot coverage.** End-to-end snapshot tests in `oneil_snapshot_tests`
  cover the key scenarios: design application, reference augmentation,
  compilation cycles, overlay-target-missing, parameter cycles, unit mismatch,
  and multi-segment apply targets.

## Planned follow-ups

### Composed-graph caching

`apply_designs` recomposes from scratch on every `eval_model` / `check_model`
call. Profiling will dictate whether to introduce a composed-graph memo keyed
by `(root, runtime_designs)`.

### Per-unit invalidation

`Runtime::clear_non_source_caches` clears the entire `UnitGraphCache`, design info, eval cache, and composed graph whenever any source changes. Invalidating only the changed unit and its dependents is not implemented yet.

### Analysis split — per-unit vs composition check map

For reference, the settled assignment of checks to passes:

| Check | Where it runs |
|---|---|
| Bare-name classification and existence | `validate_instance_graph` over the composed graph |
| Reference-name existence | `validate_instance_graph` over the composed graph |
| Unit compatibility (overlay vs target) | Inline in `apply_contribution_here`, both per-unit splice and runtime-design |
| Overlay-target-missing | Same |
| `p.r` existence (cross-instance) | `validate_instance_graph` over the composed graph |
| Cross-instance SCC over param deps | `validate_instance_graph` over the composed graph |
| Dimensional grammar of expressions | Per-unit build; cached via the unit graph |
| Limit dim propagation | Per-eval (value-dependent) |

The clean rule: whole-graph analysis checks run **once per root composition**,
on the post-splice, post-all-applies graph. Editor-time file-local diagnostics
for an opened file `foo.on` fall out naturally because the LSP builds `foo.on`
as its own root — a degenerate composition with no parent overlays.

## Error inventory

| Origin | Bucket | Notes |
|---|---|---|
| Parser/lexer errors | File-local (AST cache) | Per file, surfaced once |
| AST file missing/unreadable | File-local (AST cache) | |
| Python import file/symbol missing | File-local (per-unit build) | Per consuming unit |
| File / unit cycle (mutual references) | `InstanceGraph::cycle_errors` | Per-build stack of `(unit, imported_at)` frames; one `CompilationCycleError` per detection, attributed to the cycle target (the file the cycle closes back onto) at that file's outgoing-reference span. Other participants surface the same cycle from their own perspective when *they* are reached as a build root through the cache. |
| Unit grammar parse errors | File-local (per-unit build) | Lowering-time |
| Duplicate parameter / test in same file | File-local (per-unit build) | Within one AST |
| `Variable::Parameter { p }` where `p` ∉ binding scope | `InstanceGraph::validation_errors` | Bare-identifier existence; detected post-composition |
| `Variable::External { r }` where `r` ∉ binding scope | `InstanceGraph::validation_errors` | Deferred to post-composition validation in `validate_instance_graph` |
| `Variable::External { r, p }` where `p` ∉ `r`'s parameters | `InstanceGraph::validation_errors` | Detected post-composition; locally-applied designs already visible at link time |
| Apply target reference not found | File-local (per-unit build) | `apply_errors` against the apply span |
| Apply path component ambiguous / multi-segment | File-local (per-unit build) | Walked segment-by-segment |
| Design `design <target>` doesn't match resolved model | `InstanceGraph::contribution_errors` | At every depth |
| Overlay unit dimensionally incompatible with target | `InstanceGraph::contribution_errors` | File-local for own-applies, composition for runtime designs |
| Overlay overrides nonexistent target parameter | `InstanceGraph::contribution_errors` | Same diagnostic, both callsites |
| SCC on parameter dependencies (file-local and cross-instance) | `InstanceGraph::validation_errors` | Post-composition only |
| Composition cycle introduced by runtime designs | `InstanceGraph::cycle_errors` | Only emerges with runtime overlays |
| Best-match suggestions for typo'd identifiers | File-local or `contribution_errors` | Same machinery in both buckets |
| Test expression has unresolved variable | File-local (per-unit build) | Tests lower like parameters |
| Trace-level references | File-local (per-unit build) | Lowered per instance |
| Limit unit dimensionally incompatible | File-local (per-unit build) | Done over the file-local dependency graph |
| Numeric overflow / domain errors | Eval | Value-dependent |
| Piecewise: no condition matched | Eval | Value-dependent |
| Parameter outside declared limits | Eval | Value-dependent |
| Python function exception | Eval | Value-dependent |
| Runtime cycle via piecewise/python | Eval | `ParamSlot::InProgress` backstop |

## Rejected alternatives

- **Keep the single-pass walk, fix the side tables in place.** The side tables
  are a symptom of the structural mismatch between `ModelPath`-keyed caches and
  `EvalInstanceKey`-keyed diagnostics. Adding more dedup machinery scales
  linearly with the number of diagnostic classes; the per-unit cache scales
  constantly.
- **Cache composed graphs by `(root, runtime_designs)`.** Defers the question
  of when "design set" equality is cheap (it's a set of arbitrary `ApplyDesign`
  records), and pessimizes the common case (no runtime designs) where
  composition is identity over the cached root unit graph. Revisit on profiling.
- **Type-level distinction between `SelfRootedGraph` and `ComposedGraph`.**
  They're structurally identical, callers that consume either don't care which
  one they got, and the type-level split forces conversions everywhere
  (re-keying is mechanical, not semantic). The "self-rooted" property is just a
  convention on cached entries; downstream code reads `InstanceGraph`.
- **Flat `instances: IndexMap<EvalInstanceKey, InstancedModel>` map.** Requires
  re-keying (patching instance path prefixes) every time a child graph is
  spliced under a new parent — O(n) work that also requires rewriting all
  `Variable::External::instance_key` fields inside expressions. The tree
  eliminates re-keying entirely: children are already at the right relative
  position in the tree, and `EvalInstanceKey`s are only materialized at eval
  time via a single O(n) traversal.
- **Absolute `anchor_key: EvalInstanceKey` in `DesignProvenance`.** Would
  require rebasing all design provenances when a cached unit graph is embedded
  under a new parent. `RelativePath` avoids this: the anchor is always expressed
  relative to the overlay host, valid regardless of where in a larger graph the
  host is embedded.

## Consequences

What becomes easier:

- **LSP open-file flows have a natural diagnostics target.** No synthetic root
  needed: the cached unit graph for the opened file carries every file-local
  diagnostic.
- **Cache invalidation can be per-file.** Self-rooted unit graphs let an edit to `foo.on` invalidate only `foo` and its dependents, with downstream composers rebuilding composition cheaply on the next eval. The runtime does not do this yet (see [Per-unit invalidation](#per-unit-invalidation)).
- **Diagnostic shape is uniform across the two callsites.** Local-apply and
  runtime-overlay errors share the same emit path through `merge_child_graph` +
  `apply_contribution_here`, so message / span / best-match logic doesn't fork.
- **Adding new file-local checks is a per-unit-build concern.** New
  composition-only checks (e.g. eventual runtime-design conflict detection)
  layer onto `apply_designs` without disturbing the cache.
- **No re-keying on splice.** The tree structure means children land at the
  correct relative position without patching instance path prefixes or rewriting
  expression fields.

What becomes harder / requires care:

- **Tree clone on composition.** `apply_designs` clones the root unit graph
  before applying runtime overlays. Acceptable at our scales; revisit on
  profiling.
- **Mutual references are a hard error.** Files that share circular dependencies
  surface cycles through the per-build stack.
- **Composition errors live on a transient graph.** `get_model_diagnostics` has to
  materialize the composed graph on demand, since composition errors don't
  survive on the cached unit graphs. The cost is bounded by the per-unit cache
  and the linearity of composition.
