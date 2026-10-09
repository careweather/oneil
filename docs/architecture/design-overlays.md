# Design Application: Implementation Guide

This document describes the implementation architecture for design files,
overlays, and parameter additions. It complements
[`../specs/designs-and-imports.md`](../specs/designs-and-imports.md) (the
user-facing spec) with developer-focused details.

> See also
> [`../decisions/2026-04-24-two-pass-instance-graph.md`](../decisions/2026-04-24-two-pass-instance-graph.md)
> for the decision record that motivates the two-pass design and the broader
> plan to make the instance graph the compiler's single structural
> representation.

## Pipeline overview

```
parse → resolve (per-file IR) → per-unit build (cached) → composition → validate → eval
```

The resolver produces a clean per-file IR (`Model`, `Design`, `ApplyDesign`).
It does **not** stamp out per-instance overlays, walk reference chains, or
cross-link variables.

The **instance graph** is built in two phases in `oneil_frontend::instance::graph`:

1. **Per-unit build** — `build_unit_graph(unit)` produces an `InstanceGraph`
   rooted at `(unit, InstancePath::root())` and caches it. Each compilation
   unit (`.on` model file or `.one` design file) has exactly one cached graph.
   All own-file design applies happen here, including recursive propagation:
   when design `D` is applied at target `T` and `D` itself contains
   `apply E to ref`, `E`'s contributions are automatically applied at `T.ref`.

2. **Composition** — `apply_designs(root_unit, runtime_designs)` clones the
   cached root-unit graph into a fresh `InstanceGraph` and applies any runtime
   designs (e.g. from `--design`). This is not cached and runs in time linear in
   instance count on each `eval_model` / `check_model` call.

`oneil_analysis::validate_instance_graph` then runs over the composed graph to
check for undefined parameters / references and parameter cycles introduced by
composition. Evaluation runs lazily over the validated composed graph.

## Data structures

### Per-file IR: `Design` and `ApplyDesign`

`Design` (in `oneil_ir::design_overlay`) is the resolved form of a `.one`
design file. It holds the optional target model path, parameter overrides and
additions at the top level, and scoped overrides for a single nested reference
declared directly in the design (e.g. `x.ref = value`). Nested `apply X to ref`
declarations within a design file are recorded as `ApplyDesign` records on the
model IR (not in `Design`) and are processed recursively by the graph builder.
A model's IR carries `applied_designs: Vec<ApplyDesign>`, consumed during the
per-unit build.

### Variables are unresolved at file time

Variables in IR expressions are one of three kinds (defined in
`oneil_ir::expr`): bare parameter references (a single name), external
`parameter.reference` accesses (the quantity first, the instance second), and
builtin identifiers. The resolver lowers bare identifiers to bare parameter
references and dotted `p.r` syntax to external references — without checking
whether the reference alias or parameter name exists. Both are deferred to
`validate_instance_graph`, which runs after the full composed graph (including
all design contributions) is available.

At eval time, external references are resolved dynamically: the evaluator looks
up the reference alias on the active instance's imports map to find the correct
child instance, then reads the parameter from that child.

### Instance graph: `InstanceGraph` and `InstancedModel`

An `InstanceGraph` (in `oneil_frontend::instance::graph`) has two storage
locations for model instances:

- **`root`** — the root model instance and its entire owned subtree.
- **`reference_pool`** — shared cross-file reference instances, keyed by
  canonical model path. Multiple importers of the same file share one pool
  entry.

It also carries four error buckets: `cycle_errors`, `resolution_errors`,
`contribution_errors`, and `validation_errors`. For design-keyed cache entries
it also holds the resolved `Design` ready for application.

Each `InstancedModel` has three disjoint maps for its named children (see
`instance/model.rs`):

- **`submodels`** — owned subtrees, one per `submodel foo as bar` declaration.
- **`references`** — cross-file pointer imports (`reference foo as r`); the
  actual instances live in the graph's `reference_pool`.
- **`aliases`** — local shortcuts into the owned subtree, created by
  extraction-list items (`submodel a as c [b as x]`).

Plus `parameters` (own IR parameters plus any design additions/overrides) and
`tests`.

All error buckets live on `InstanceGraph`, not on individual `InstancedModel`
nodes. `InstanceGraph` is structurally identical whether it is a cached
per-unit graph or a composed graph — the distinction is only a convention:
cached entries are self-rooted; a composed graph is rooted at whatever path
the user requested.

`EvalInstanceKey`s are materialised only at eval time by `EvalContext`, which
performs a single O(n) traversal of the `root` subtree and `reference_pool` to
build the key → slot map. This means no re-keying is needed when a child graph
is spliced under a parent: children are already at the correct relative position
in the tree.

### Compilation unit and cache

A `CompilationUnit` is either a `Model(ModelPath)` or a `Design(DesignPath)`
(defined in `oneil_frontend::instance::compilation_unit`). The `UnitGraphCache`
maps each `CompilationUnit` to its cached `InstanceGraph` and lives on
`Runtime` as `unit_graph_cache`. Cache invalidation:

- Edit `foo.on` → invalidate `CompilationUnit::Model(foo)` only.
- Edit `bar.one` → invalidate `CompilationUnit::Design(bar)` only.

Composition is cheap (linear in instance count); downstream composers rebuild
on the next `eval_model` / `check_model` call.

## Resolution phase

Resolution produces per-file IR without composing models together. For
design files, `resolve_design_surface` runs in two logical steps:

1. **Scan** — find the `design <target>` declaration and collect
   unscoped parameter names. Names that don't already exist on the
   target are registered in a temporary scratch table on
   `ResolutionContext` so design-local params can cross-reference each
   other and the target's own parameters.
2. **Dispatch** — for each surface item:
   - `design <target>` — sets `Design::target_model`.
   - `id = expr` — if `id` exists on the target: `Design::parameter_overrides`; otherwise: `Design::parameter_additions`.
   - `id.ref = expr` — `Design::scoped_overrides[ref][id]`.
   - `apply X to ref(.ref)*` — recorded as an `ApplyDesign` on the consuming model.

Variable resolution emits an external reference node carrying the reference alias
and parameter name without checking whether the alias exists in the active
model's imports or whether the parameter exists on the target model — design
additions might supply either. Both are deferred to `validate_instance_graph`,
which runs after the full composed graph (including all design contributions) is
available.

## Per-unit build

`build_unit_graph(unit)` walks the unit's model tree and produces an
`InstanceGraph` rooted at `(unit, InstancePath::root())`. The result is cached
by `CompilationUnit`.

For each child the unit pulls in:

1. Recursively obtain the child's cached graph via `build_unit_graph(child)`.
2. Determine the contributions the unit supplies to that child (own-`apply`
   statements landing on this anchor).
3. Call `merge_child_graph(parent, child_graph, anchor)` to insert the child's
   `root` into the parent's subtree at `anchor` and merge `reference_pool`
   entries (deduplicating by path). No re-keying of `EvalInstanceKey`s is
   needed — the tree structure means children are already at the correct
   relative position.
4. Apply the contributions in-place via `apply_contribution_here`.

Children are fully composed under the unit's own applies *before* the unit
links its own expressions. So when `parent.on` writes `doubled.c` after
`apply augment to c`, the `c` instance in `parent.on`'s cached graph already
has `doubled` from `augment`, and the lookup resolves file-locally.

`.one` design files are also compilation units. `build_design_unit_graph`
caches a graph rooted at the design's *target model* under a `Design(d)` cache
entry. Two overlay passes run after the target's unit graph is merged in: the
design's own resolved contributions (overrides and additions), then the design
file's own `apply X to ref` declarations.

### Inherited units

A parameter whose value is only a reference to another parameter, such as
`P_l = P_t.r`, and that has no unit annotation inherits the referenced
parameter's unit when that unit has physical dimensions. Every per-unit build
and every composition ends with `fill_inherited_units`
(`oneil_frontend::instance::reference_units`), which writes the inherited unit
into the reference's `ParameterValue`. Overlay values get their inherited unit
from the design's anchor scope before the overlay unit check, so the check
compares inherited units on either side. Validation, evaluation, and the
rendered view then read the unit as if the source had annotated it.

### Cycle detection

A per-build stack of `(CompilationUnit, imported_at: Span)` values tracks
active units. Encountering a unit already on the stack is an error, attributed
to the cycle target — the file the cycle closes back onto — at that file's
outgoing-reference span. The recorded `CompilationCycleError` (in
`oneil_frontend::instance::cycle_error`) carries the full cycle chain.
`merge_child_graph` propagates cycle errors upward so the composed graph carries
every cycle observed anywhere in the dependency tree.

## Composition

`apply_designs(root_unit, runtime_designs)` clones the cached root-unit graph
into a fresh `InstanceGraph` and applies `runtime_designs` as contributions via
the same `merge_child_graph` + `apply_contribution_here` pair. No-op when
`runtime_designs` is empty, in which case the cached root unit graph *is* the
composed graph (modulo a cheap clone).

The composed graph is **not cached**. Composition runs in time linear in
instance count. If profiling later shows pain, add a memo keyed by
`(root_unit, runtime_design_set)` on top — the per-unit cache stays the source
of truth.

The CLI's `--design path.one` injects a single synthetic contribution into
`apply_designs`'s `runtime_designs` slot. There is no separate code path for
"with design" vs. "without design" — both flow through the same primitive.

## Error partitioning

Errors fall into two distinct buckets with distinct lifetimes:

**File-local** (cached on each unit's `InstanceGraph`):
- parse / lower
- duplicate parameter / reference names in the same file
- model import resolution failures (bad path, missing file)
- own-`apply` target existence
- own-`apply` overlay-target-missing (message + best-match suggestion)
- dimensional grammar
- design `design <target>` doesn't match resolved model

**Post-composition validation** (run once via `validate_instance_graph`):
- bare-name existence (`Variable::Parameter` not in binding scope)
- reference-name existence (`p.r` where `r` not in any import map)
- reference parameter existence (`p.r` where `p` not in `r`'s model)
- cross-instance SCC over parameter dependencies

**Composition** (computed per `eval_model` / `check_model` call):
- runtime-overlay-target-missing (same diagnostic shape as file-local)
- runtime-overlay unit mismatch
- composition-introduced cycles

All error buckets live on `InstanceGraph` directly
(`cycle_errors`, `resolution_errors`, `contribution_errors`,
`validation_errors`), never on individual `InstancedModel` nodes. On overlay,
newly-introduced errors are recorded on the *composed* graph; the cached unit
graph is never mutated.

`Runtime` holds no `*_errors` fields. Instead it holds
`composed_graph: Option<InstanceGraph>` populated by each
`eval_model_internal` / `check_model_internal`. `get_model_errors` reads the
four error buckets directly off the composed graph, falling back through
`unit_graph_cache` for paths not yet reached by composition.

## Evaluation phase

Evaluation uses a **lazy memoized** strategy. `EvalContext::from_graph` seeds
one `ParamSlot` per parameter per instance from the (already applied + linked)
composed graph. Parameters are forced on demand; cycles surface on re-entrance
via `ParamSlot::InProgress`.

Design-application failures (e.g. overlay-target-missing) are recorded on the
composed graph and surfaced through the same parameter-error machinery as any
other evaluation failure.

### Memo table: `ParamSlot`

| State                   | Meaning                                              |
|-------------------------|------------------------------------------------------|
| `Pending(ir::Parameter)`| Not yet evaluated                                    |
| `InProgress`            | Currently evaluating; re-entry means a cycle         |
| `Done(Result<…>)`       | Evaluation complete                                  |

### Variable lookup

`eval_expr` resolves variables against the live `EvalContext`. Bare parameter
references look up the name in the current instance's parameter table. External
`parameter.reference` accesses first resolve the reference alias to the correct
child instance via the active instance's `references` map, then look up the
parameter on that child instance.

Overlay-RHS variables are evaluated in the `anchor_path` scope set by
`DesignProvenance`. At eval time, `force_parameter` resolves `anchor_path`
(a `RelativePath { up, down }`) against the current instance's parent chain
and temporarily pushes the anchor's key, so the RHS is evaluated in the
design's original lexical scope with no per-call overhead in the common case.

### Key types and entry points

| Item                          | Location                           | Purpose                                          |
|-------------------------------|------------------------------------|--------------------------------------------------|
| `build_unit_graph`            | `instance/graph.rs`                | Build + cache per-unit `InstanceGraph`           |
| `apply_designs`               | `instance/graph.rs`                | Clone cached graph + apply runtime designs       |
| `apply_design_recursive`      | `instance/graph.rs`                | Apply a design and walk its nested applies       |
| `merge_child_graph`           | `instance/graph.rs`                | Insert child subtree into parent; merge pool     |
| `validate_instance_graph`     | `oneil_analysis`                   | Cross-instance `p.r` existence + SCC             |
| `InstancedModel`              | `instance/model.rs`                | Per-instance model view (three-map structure)    |
| `CompilationCycleError`       | `instance/cycle_error.rs`          | First-class cycle errors with full chain         |
| `EvalContext::from_graph`     | `oneil_eval/context.rs`            | Build key→slot map; seed memo table              |
| `eval_model_from_graph`       | `oneil_eval/eval_model.rs`         | Drive evaluation over the composed graph         |
| `force_parameter`             | `oneil_eval/context.rs`            | Lazy memo-table core                             |

## Design-augmented references (`new_param.ref`)

When a model applies a design to a reference (`apply D to b`) and `D`
adds new parameters, those parameters become accessible via
`new_param.b` (subscript-style: parameter first, reference second).

| Phase           | What happens                                                                                      |
|-----------------|---------------------------------------------------------------------------------------------------|
| Resolution      | An external reference for `new_param.b` is emitted with no existence check against `b`'s standard model. |
| Per-unit build  | Parameter additions from `D` are folded into `b`'s instance parameters during `apply_design_recursive`. |
| Validation      | `validate_instance_graph` walks the composed graph. If `D` was applied, `b`'s parameters contain `new_param` — no error. If not, an undefined-reference-parameter error is emitted. |
| Evaluation      | The evaluator resolves `b` to the correct child instance dynamically and finds the addition like any other parameter. |

Cross-instance `x.r` against design-added params is visible file-locally
when the consuming model is the one applying the design: the child's post-apply
scope is what the graph build sees. A parameter that only exists because of an
*external* runtime design surfaces as a validation error from the cached unit
graph's perspective; it clears once the runtime design is applied in
`apply_designs`.
