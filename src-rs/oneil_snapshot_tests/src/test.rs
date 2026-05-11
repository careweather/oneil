//! Integration snapshot tests for Oneil evaluation output and errors.
//!
//! Each test runs the full pipeline (parse → resolve → eval) on a fixture
//! and compares the formatted output against a stored snapshot.
//!
//! Tests are grouped by feature category with prefixes (e.g., `basic_`, `overlay_`)
//! so snapshot files sort together.

use std::path::PathBuf;

use crate::util::run_model_and_format;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Evaluates a single fixture model or design file and returns formatted output
/// for snapshotting. If a design file is provided, the runtime automatically
/// detects it and applies it to the declared target model.
fn run_fixture(name: &str) -> String {
    run_model_and_format(&fixture_path(name), Some(manifest_dir().as_path()))
}

// =============================================================================
// Basic Tests
// =============================================================================

#[test]
fn basic_model() {
    insta::assert_snapshot!(run_fixture("basic/basic.on"));
}

#[test]
fn basic_syntax_error() {
    insta::assert_snapshot!(run_fixture("basic/syntax_error.on"));
}

#[test]
fn submodel_syntax_error() {
    // When a submodel has a syntax error, the parent model should report a
    // "submodel X has errors" diagnostic at the import site and the submodel's
    // own parse errors should also be collected.
    insta::assert_snapshot!(run_fixture("submodel_syntax_error/parent.on"));
}

#[test]
fn basic_failing_test() {
    insta::assert_snapshot!(run_fixture("basic/failing_test.on"));
}

#[test]
fn basic_undefined_reference_parameter() {
    // A cross-file reference whose parameter access names a parameter that
    // does not exist on the referenced model.  This exercises the
    // validation-pass `UndefinedReferenceParameter` check (formerly surfaced
    // at resolver time in the old `load_model_with_submodel_with_error` test).
    insta::assert_snapshot!(run_fixture("ref_undefined_param/consumer.on"));
}

// =============================================================================
// Design Overlay Tests
// =============================================================================

#[test]
fn overlay_shared_ref() {
    // `reference` makes `planet` a shared instance: an overlay on it is
    // observed by both reads of the same alias.
    insta::assert_snapshot!(run_fixture("design_overlay/shared_ref.on"));
}

#[test]
fn overlay_two_instances() {
    // `submodel` stamps two independent instances: an overlay applied `to
    // planet_a` must not affect planet_b.
    insta::assert_snapshot!(run_fixture("design_overlay/two_instances.on"));
}

#[test]
fn overlay_submodel_with_apply_explicit() {
    // Explicit two-line form: `submodel M as A` + `apply D to A`.
    // Planet A gets Mars gravity override; planet B keeps Earth gravity.
    insta::assert_snapshot!(run_fixture("design_overlay/submodel_with_apply.on"));
}

#[test]
fn submodel_neither_on_nor_one_found() {
    // When neither `nonexistent.on` nor `nonexistent.one` exists, the resolver
    // should emit a clear "no model or design file found" error pointing at
    // both paths that were tried.
    insta::assert_snapshot!(run_fixture("design_overlay/missing_model/parent.on"));
}

#[test]
fn overlay_wrong_target_for_ref() {
    // A design whose declared target model doesn't match the model that `r`
    // resolves to should produce a clear error instead of silently doing nothing.
    insta::assert_snapshot!(run_fixture("design_overlay/wrong_target/parent.on"));
}

#[test]
fn overlay_nested_parameter() {
    // `param.instance = value` syntax overrides a parameter on a nested
    // reference instance. Running the design file automatically applies it
    // to its declared target model.
    insta::assert_snapshot!(run_fixture(
        "design_overlay/nested_param/nested_param_design.one"
    ));
}

#[test]
fn overlay_scoped_reference_override() {
    // `param.ref_alias = value` in a design overrides `param` inside the
    // shared reference instance (not just a submodel hop). This verifies
    // that scoped overrides work across reference boundaries, not only
    // submodel ones.
    insta::assert_snapshot!(run_fixture(
        "design_overlay/scoped_ref_override/far_distance.one"
    ));
}

// =============================================================================
// Submodel Extraction Tests
// =============================================================================

#[test]
fn extract_overlay_propagation() {
    // An `[inner]` extraction aliases the extracted submodel onto the same
    // instance as the deeper child it was lifted from.  An overlay setting
    // `value.inner = 99` on the parent must therefore land on that single
    // shared instance so every path that reaches `inner` — direct
    // (`value.inner`), indirect via mid (`value.mid.inner`), and mid's own
    // computed result — all observe the overridden value.
    insta::assert_snapshot!(run_fixture("with_overlay_propagation/overlay.one"));
}

#[test]
fn extract_with_partial_overlay() {
    // Parent extracts `inner` (gravity) from an intermediate via `[inner]`.
    // A Mars-gravity override applied through the extracted alias changes
    // gravity-dependent parameters (`result`, `g_ref`) while constants
    // defined inside the intermediate (`thrust_out`, `mass_out`) stay fixed,
    // confirming that the overlay reaches only what it targets.
    insta::assert_snapshot!(run_fixture("with_clause/mars_override.one"));
}

// =============================================================================
// Design Augmentation Tests
// =============================================================================

#[test]
fn design_local_augmentation() {
    // A design file adds new parameters that don't exist on the target model
    // and overrides an existing one.  New parameters can cross-reference each
    // other and the target's own parameters.
    insta::assert_snapshot!(run_fixture("design_local/augment.one"));
}

#[test]
fn design_local_augmentation_flagged() {
    // A design file adds new parameters with `$` (output) and `*` (trace)
    // flags.  The flags should be accepted by the parser and threaded through
    // the resolver without affecting the evaluated values.
    insta::assert_snapshot!(run_fixture("design_local/augment_flagged.one"));
}

#[test]
fn design_local_augmentation_labeled() {
    // A design file uses the full `Label: id = value` syntax for both overrides
    // and new parameter additions.  Labels are preserved through the resolver and
    // the evaluated values must match the shorthand-equivalent.
    insta::assert_snapshot!(run_fixture("design_local/augment_labeled.one"));
}

#[test]
fn design_augmented_reference_params() {
    // A model applies a design to one of its own references (`apply augment to
    // c`).  Parameters added by the design become accessible on the parent via
    // `added_param.c` syntax, and the parent's own expressions can read them.
    insta::assert_snapshot!(run_fixture("augmented_refs/parent.on"));
}

#[test]
fn design_augmented_override() {
    // A design overrides a parameter that was itself added by an earlier
    // inline `apply`.  Scoped override syntax (`doubled.c = 100`) must reach
    // the design-augmented parameter on the child instance.
    insta::assert_snapshot!(run_fixture("augmented_refs/override_augmented.one"));
}

#[test]
fn design_sibling_designs() {
    // Two sibling submodel imports of the same model each receive a *different*
    // CLI design.  Each `submodel` import is an independent instance so the
    // designs must not bleed across instances.
    insta::assert_snapshot!(run_fixture("sibling_designs/parent.on"));
}

#[test]
fn design_deep_apply_additions() {
    // A design addition propagates through multiple model hops: `mid` applies
    // `augment` to `leaf`, adding a parameter that `mid` then reads via
    // `Variable::External`.  The parent reads the forwarded value without
    // knowing about the augmentation directly.
    insta::assert_snapshot!(run_fixture("deep_apply_additions/parent.on"));
}

#[test]
fn design_test_scope() {
    // A design file that only adds a test (no parameter overrides/additions).
    // The test expression must be evaluated in the target model's scope, not
    // the design file's scope, so that it can reference the target's parameters.
    insta::assert_snapshot!(run_fixture("design_test_scope/test_only.one"));
}

#[test]
fn design_applies_design_model_level() {
    // A model (`parent`) applies `outer.one` to its `h` submodel. `outer.one`
    // itself applies `inner.one` to its `l` reference, adding `squared = base
    // * base`. `outer.one` also adds `result = squared.l` to `h`. The parent
    // reads `result.h` to verify that inner's addition propagated through the
    // design-applies-design chain all the way to the root model's output.
    insta::assert_snapshot!(run_fixture("design_applies_design/parent.on"));
}

#[test]
fn design_applies_design_runtime() {
    // Same scenario as `design_applies_design_model_level` but the outer
    // design is run directly rather than declared in a model file. Tests
    // the `apply_designs` → `apply_design_recursive` path.
    insta::assert_snapshot!(run_fixture("design_applies_design/outer.one"));
}

#[test]
fn design_multi_segment_apply_target() {
    // `parent` applies a design to `m.l` — a two-segment target where `m` is
    // a direct submodel of parent and `l` is a submodel of `mid`.  The
    // resolver must thread through the intermediate model to validate both
    // segments, and the instancing pass must land the override on the correct
    // deep instance.  `result = y.m = x.l` should evaluate to 10.0.
    insta::assert_snapshot!(run_fixture("multi_seg_apply/parent.on"));
}

#[test]
fn design_nested_apply_syntax() {
    // Verifies nested apply bracket syntax: `apply X to m [ Y to l ]`.
    // Reuses the multi_seg_apply fixtures; semantics are equivalent to
    // separate `apply X to m` + `apply Y to m.l` declarations.
    // The inner `l` must resolve against mid's model, not parent's.
    insta::assert_snapshot!(run_fixture("multi_seg_apply/parent_nested.on"));
}

// =============================================================================
// Overlay Anchor Scope Tests
// =============================================================================

#[test]
fn overlay_anchor_scope() {
    // Scoped overlay RHS expressions are evaluated in the design's *target*
    // scope (the anchor), not in the ref's instance scope.  A parameter that
    // lives on the design's target and not on the child ref must still resolve
    // correctly in the override's RHS.
    insta::assert_snapshot!(run_fixture("overlay_anchor_scope/anchor_scope.one"));
}

#[test]
fn overlay_anchor_scope_transitive() {
    // An overlay RHS references a parent-local parameter whose own RHS
    // depends on an external reference.  Lazy evaluation must force the
    // dependency on demand when the overlay scope is set up — the
    // anchor-scope push must not assume the parent's parameter is already
    // evaluated.
    insta::assert_snapshot!(run_fixture(
        "overlay_anchor_scope/anchor_scope_transitive.one"
    ));
}

// =============================================================================
// Cycle Tests
// =============================================================================

#[test]
fn parameter_cycle_file_static() {
    // A mutual dependency within a single file (`a = b`, `b = a`) is caught by
    // the composed-graph SCC pass without any design contributions involved.
    insta::assert_snapshot!(run_fixture("parameter_cycle/cycle.on"));
}

#[test]
fn overlay_introduces_cycle() {
    // Applying an overlay that retargets `x` to read `y` (while `y` already
    // reads `x`) creates a cycle on the composed graph.  The cross-instance SCC
    // pass in `oneil_analysis::validate_instance_graph` catches it and emits one
    // `ParameterCycle` per member; the eval-time `InProgress` backstop is
    // suppressed so the user sees one diagnostic per member, not duplicates.
    insta::assert_snapshot!(run_fixture("cycle_via_overlay/cycle.one"));
}

#[test]
fn compilation_cycle() {
    // `a.on` submodels `b`; `b.on` submodels `a` back.  The per-unit build
    // detects the back-edge while building `a`'s unit graph and attributes the
    // error to the cycle *target* (`a.on`) at its own outgoing-reference span.
    // The rendered message includes the full unit chain.
    insta::assert_snapshot!(run_fixture("compilation_cycle/a.on"));
}

// =============================================================================
// Design Error Attribution Tests
// =============================================================================

#[test]
fn overlay_unit_mismatch() {
    // A unit-incompatible override is rejected by the apply pass.  The host
    // parameter retains its pre-overlay value; a single diagnostic is emitted
    // against the design file's assignment span, not the host model.
    insta::assert_snapshot!(run_fixture("unit_mismatch_overlay/mismatch.one"));
}

#[test]
fn overlay_target_missing() {
    // An override targeting a parameter that doesn't exist on the host emits a
    // single diagnostic against the design file with a best-match suggestion.
    insta::assert_snapshot!(run_fixture("overlay_target_missing/typo.one"));
}

#[test]
fn chain_apply_unit_mismatch() {
    // Two-hop apply chain: `parent` → `mid` (owns `apply bad to l`) → `leaf`.
    // The `bad` design assigns a `:s` value to `leaf.lng` (declared `:m`),
    // which the apply pass rejects.  Chain provenance surfaces the diagnostic
    // against every file on the chain: the full error against `bad.one` and a
    // generic "applied design produced invalid contributions" diagnostic at
    // `mid.on`'s `apply` span.  `parent.on` is silent (the apply is one hop
    // deeper).
    insta::assert_snapshot!(run_fixture("chain_apply_unit_mismatch/parent.on"));
}

#[test]
fn chain_apply_validation_cycle() {
    // Two-hop apply chain: `parent` → `mid` (owns `apply cycle to l`) → `leaf`.
    // The `cycle` design turns the leaf's `x = 1; y = 2 * x` pair into an SCC.
    // Chain provenance fans out: `ParameterCycle` errors against `leaf.on` and
    // a generic contribution diagnostic at `mid.on`'s `apply` span.
    insta::assert_snapshot!(run_fixture("chain_apply_validation_cycle/parent.on"));
}

// =============================================================================
// Python integration
// =============================================================================

/// Snapshot for evaluating a model that imports a sibling `.py` module and calls a function.
#[test]
fn python_square_area() {
    insta::assert_snapshot!(run_fixture("python/python_square_area.on"));
}
