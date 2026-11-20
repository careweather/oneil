# oneil_analysis

Semantic analysis for the Oneil programming language.

This crate runs two kinds of analysis, at two points in the pipeline described in [`docs/architecture/compiler-pipeline.md`](../../docs/architecture/compiler-pipeline.md).

## Validation of the composed graph

`validate_instance_graph` runs after composition and before evaluation. It:

- reclassifies bare names as parameters or builtins against each instance's final scope, after every design has been applied;
- reports undefined bare names, undefined reference names, and parameters missing on a referenced instance (`p.r`);
- reports parameter dependency cycles across instances.

Diagnostics are written to `InstanceGraph::validation_errors`. The pass clears that bucket first, so running it again gives the same result. After this pass the graph's structure is final, and the evaluator only reads it.

## Analysis of evaluated models

These functions work on evaluated models through `ExternalAnalysisContext`, which `oneil_runtime::Runtime` implements:

- `get_dependency_tree` and `get_reference_tree` build the trees printed by `oneil tree`;
- `get_independents` finds the independent parameters printed by `oneil independent`.
