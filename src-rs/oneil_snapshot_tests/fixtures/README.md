# Test Fixtures

This directory contains fixture files for the snapshot tests defined in
[`../src/test.rs`](../src/test.rs). Each test runs the full pipeline (parse →
resolve → per-unit build → composition → validate → eval) on a fixture and
diffs the formatted output against a stored snapshot under `../src/snapshots/`.

Fixtures use realistic engineering values where practical so failures are
easy to read by inspection.

## Shared models

Two planetary-gravity models live at the top level and are referenced from
several fixture directories with a `../gravity_*` relative path:

- `gravity_earth.on` — `g = 9.81 :m/s^2`
- `gravity_mars.on`  — `g = 3.72 :m/s^2`

## Test categories

### `basic/` — sanity (4 tests)

Smallest possible models, used to pin down regressions in the core pipeline.

- `basic.on` — independent + dependent params, one passing test
- `syntax_error.on` — parse error
- `failing_test.on` — assertion that fails at eval
- `ref_undefined_param/consumer.on` — reference whose `param.ref` access
  names a parameter not present on the target model
  (`basic_undefined_reference_parameter`)

### `design_overlay/` — overlay semantics (4 tests)

- `shared_ref.on` + `mars_override.one` — `reference` makes one shared
  instance; an overlay applied via the shared alias is seen everywhere
- `two_instances.on` + `mars_override.one` — two `submodel` imports of the
  same model are independent; an overlay on one must not leak to the other
- `wrong_target/` — design's `design <model>` declared target does not match
  the model the reference resolves to (error)
- `nested_param/` — `param.alias = value` shorthand in a design file
  overrides a parameter on a nested instance

### `with_clause/` — extraction-list overlays (2 tests)

- `with_extract_through_mid.on` + `mars_override.one` —
  parent extracts `inner` (gravity) out of an intermediate via
  `submodel mid_with_matching as propulsion [inner]`. A Mars-gravity overlay
  applied through the extracted alias changes gravity-dependent parameters
  while constants inside the intermediate stay fixed
- `with_overlay_propagation/` — `[inner]` extraction aliases the extracted
  submodel onto the same instance as the deeper child it was lifted from;
  an overlay on the extracted alias is visible from every path that reaches
  it (direct, indirect via `mid`, and `mid`'s own derived parameters)

### `augmented_refs/` — design augmentations on references (2 tests)

A design adds a new parameter on a referenced model that the parent then
consumes via `param.ref`. Verifies that augmentation propagation works
through both unmodified and overridden augmentations.

### `design_local/` — design augmentations on the design's own target (1 test)

A design adds new parameters on its target model and overrides an existing
one. New parameters can cross-reference each other and the target's own
parameters.

### `sibling_designs/` — multiple distinct overlays (1 test)

Two `submodel` imports of the same model, each receiving a different
CLI design. Pins down that the per-import instances stay independent.

### `deep_apply_additions/` — apply propagation through hops (1 test)

`mid` applies a design adding a parameter on `leaf`; `parent` reads the
forwarded value via `Variable::External` without knowing about the
augmentation directly.

### `overlay_anchor_scope/` — design RHS evaluation scope (2 tests)

Scoped overlay RHS expressions evaluate in the design's *target* scope
(the anchor), not the reference's instance scope. The transitive variant
exercises the case where the RHS reads a parent-local parameter that
itself depends on an external reference (lazy-evaluation regression).

### `parameter_cycle/` — pure file-static cycle (1 test)

`a = b; b = a` in a single file. The post-build SCC pass in
`oneil_analysis::validate_instance_graph` catches it without any design
composition involved.

### `cycle_via_overlay/` — composition-introduced cycle (1 test)

An overlay retargets `x` to read `y` while `y` already reads `x`. The
SCC pass detects the cycle on the composed graph; the eval-time
`InProgress` backstop is suppressed so each member produces one
diagnostic, not duplicates.

### `compilation_cycle/` — file-level cycle (1 test)

`a.on` submodels `b`; `b.on` submodels `a` back. The per-unit build
detects the back-edge and attributes the error to the cycle target.

### `unit_mismatch_overlay/` — local apply unit error (1 test)

An override whose unit is incompatible with the host parameter is
rejected by the apply pass. A single diagnostic is emitted against the
design assignment span; the host parameter retains its pre-overlay value.

### `overlay_target_missing/` — local apply name error (1 test)

An override targeting a parameter that doesn't exist on the host emits a
single diagnostic against the design with a best-match suggestion.

### `chain_apply_unit_mismatch/` — chain attribution: unit error (1 test)

Two-hop `parent → mid → leaf` where `mid` owns the failing apply.
Verifies that the full unit-mismatch error attaches to `bad.one` and
that `mid.on` also surfaces a generic "applied design produced invalid
contributions" diagnostic at its own `apply` span. `parent.on` stays
silent (the apply is one hop deeper).

### `chain_apply_validation_cycle/` — chain attribution: cycle (1 test)

Same shape, but the failing apply introduces a parameter cycle on
`leaf`. Verifies that `ParameterCycle` errors land on `leaf.on` and a
generic contribution diagnostic at `mid.on`'s `apply` span. The root
also surfaces a generic "submodel `m` has errors" notification at the
import declaration so the LSP can squiggle there.

### `analysis/` — dependency tree, reference tree, independents (6 tests)

Plain-text snapshots of the analysis CLI surfaces (tree topology and
independents), driven through the runtime APIs rather than ANSI-styled
stdout:

- `basic/basic.on` — dependency tree of `f`, reference tree of `m`,
  top-level independents
- `analysis/force_with_ref.on` — dependency tree with an external
  `g.planet` edge; reference tree of local `m`; recursive independents
  across the referenced planet

## Test count summary (42 tests)

| Category                       | Tests |
|--------------------------------|------:|
| Basic                          | 4     |
| Design overlay                 | 4     |
| Extraction (`with_clause/`)    | 2     |
| Augmented references           | 2     |
| Local design augmentation      | 1     |
| Sibling designs                | 1     |
| Deep apply additions           | 1     |
| Anchor scope                   | 2     |
| Cycles (file, overlay, build)  | 3     |
| Apply errors (local + chain)   | 4     |
| Analysis (trees / independents)| 6     |
| Other (designs, python, …)     | 12    |
| **Total**                      | **42**|
