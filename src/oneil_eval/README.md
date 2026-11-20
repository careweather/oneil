# Oneil Evaluator

This crate evaluates a validated, composed `InstanceGraph` into `oneil_output` values. It checks every type and unit rule that depends on values, and it acts as the backstop for rules not yet checked earlier (see [`docs/decisions/2026-11-20-check-as-early-as-possible.md`](../../docs/decisions/2026-11-20-check-as-early-as-possible.md)).

The crate does not parse, resolve, or compose models. Its place in the pipeline is described in [`docs/architecture/compiler-pipeline.md`](../../docs/architecture/compiler-pipeline.md).

## Entry points

- `eval_model_from_graph` evaluates every instance reachable from the graph's root and reference pool, returning one `MaybePartialResult<oneil_output::Model, ModelEvalErrors>` per `EvalInstanceKey`.
- `eval_expr_in_model` evaluates a single resolved expression in the scope of an already-evaluated model (used by `oneil eval --expr`).

Everything outside the graph comes through the `ExternalEvaluationContext` trait, which `oneil_runtime::Runtime` implements: builtin values, functions, units, and prefixes (from `oneil_builtins`), Python function calls (through `oneil_python` and the Python call cache), and models evaluated earlier.

## Evaluation strategy

Evaluation is lazy and memoized. `EvalContext::from_graph` walks the graph once, assigns each instance its `EvalInstanceKey`, and seeds one slot per parameter:

- `Pending` holds the IR parameter until it is first needed.
- `InProgress` marks a parameter being computed. Re-entering it reports a dependency cycle.
- `Done` holds the result, successful or not.

Parameters are forced on demand as expressions reference them, so evaluation order follows dependencies rather than source order. After every model is forced, errors in referenced instances are propagated to the instances that depend on them, and tests are evaluated like parameters.

Variables are resolved against the live context. Bare names look up the current instance's parameters. `p.r` references go through the current instance's references to the target instance. Overlay right-hand sides are evaluated in the scope recorded by their `DesignProvenance` (see [`docs/architecture/design-overlays.md`](../../docs/architecture/design-overlays.md#variable-lookup)).

## Errors

Evaluation errors are `oneil_output::EvalError` values. They cover type mismatches, unit mismatches, limit violations, piecewise expressions with no matching branch, numeric domain errors, cycles, and Python exceptions. Errors are collected per parameter and per test, so one failure does not stop the rest of the model from evaluating.

Value types, unit handling, float comparison, and interval-arithmetic fuzzing are documented in [`oneil_output`](../oneil_output/README.md).
