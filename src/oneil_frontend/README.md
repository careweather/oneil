# Oneil Frontend

The Oneil Frontend turns parsed Oneil source into an `InstanceGraph` —
the single structural representation passed to evaluation and analysis.
It runs in two passes:

1. **Resolve (per file)** — `oneil_frontend::resolver` walks the AST of a
   single model or design file and produces per-file IR
   (`ir::Model`, `ir::Design`). Names inside expressions are lowered to
   `Variable::Parameter` / `Variable::External` *without* existence
   checks: a parameter or reference name might be introduced by a
   design overlay that hasn't been applied yet.

2. **Per-unit build (cached)** — `oneil_frontend::instance::graph`
   walks the resolved IR for a compilation unit (`.on` model or `.one`
   design) and produces a self-rooted `InstanceGraph`. All own-file
   `apply` statements land on their target instances during this pass,
   and each unannotated plain parameter reference gets the unit of the
   parameter it references. Results are cached by `CompilationUnit`.

A subsequent **composition** step (also in `instance::graph`) clones a
cached unit graph and overlays any runtime designs to produce the graph
that evaluation runs against. Existence checks for variables and
parameter cycle detection happen *after* composition in
`oneil_analysis::validate_instance_graph`, when every contribution that
could introduce a name is in scope.

See [`docs/decisions/2026-04-24-two-pass-instance-graph.md`](../../docs/decisions/2026-04-24-two-pass-instance-graph.md)
for the architecture decision record and
[`docs/architecture/design-overlays.md`](../../docs/architecture/design-overlays.md)
for the developer-facing implementation guide.

## Crate layout

- `src/resolver/` — file-static resolution producing `ir::Model` /
  `ir::Design`. Handles bare-name lowering, unit normalisation, and
  recording each `apply` declaration.
- `src/instance/` — `InstanceGraph`, `InstancedModel`, the import value
  types (`SubmodelImport`, `ReferenceImport`, `AliasImport`),
  per-unit build (`build_unit_graph`), composition (`apply_designs`),
  and the error types scoped to the build pass (`CycleError`,
  `ContributionDiagnostic`, `InstanceValidationError`).
- `src/error/` — file-time error variants emitted by the resolver
  (`DesignResolutionError`, `VariableResolutionError`, etc.).
- `src/context/` — resolution context shared between resolver
  sub-passes (parameter scope, reference table, external registry).

## References, submodels, and aliases

Oneil offers three import forms with three distinct semantics. The
current frontend models them as three separate maps on `InstancedModel`,
so each form has a single representation throughout the pipeline.

```oneil
# === gravity_earth.on ===
Gravity of Earth: g = 9.81 :m/s^2


# === my_model.on ===
Mass of box: m_b = 5 :kg

# Reference: cross-file pointer to a single shared instance.
# Multiple importers see the same instance; an overlay on one is
# visible to every alias.
reference gravity_earth as planet
Weight of box: w_b = m_b * g.planet :N
```

```oneil
# === satellite.on ===
# Submodel: owned child. Two `submodel` imports of the same file are
# two independent instances; designs on one don't leak to the other.
submodel radar
submodel solar_panel as solar

Satellite cost: cost = cost.radar + cost.solar :$


# === product.on ===
# Importing satellite pulls in its full subtree. cost.satellite.radar
# / cost.satellite.solar are reachable through the dotted parameter
# access path.
submodel satellite

# An extraction list (`[ … ]`) creates *aliases* for paths inside the
# imported subtree, surfacing them at the parent's scope without
# creating a new instance.
submodel satellite [radar, solar_panel as solar]
# Now reachable as cost.radar, cost.solar
```

The submodel's source name (`satellite`) is *not* the map key on the
parent. The map key is the alias (`solar` in `submodel solar_panel as
solar`); two `submodel foo as a` and `submodel foo as b` lines produce
two independent map entries. Extraction-list aliases never produce a
new instance — they record a `RelativePath` into the existing subtree.
