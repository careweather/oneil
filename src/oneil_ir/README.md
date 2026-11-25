# Oneil IR

Intermediate Representation (IR) for the Oneil programming language.

This crate defines the expression-level IR shared by the resolver, validation, the evaluator, and tooling: expressions and variables, parameters (values, limits, units, dependencies, and design provenance), tests, Python imports, notes, and sections. The model-level types that hold this IR (`InstancedModel`, `InstanceGraph`, `Design`, `ApplyDesign`) live in `oneil_frontend::instance`.

The resolver in `oneil_frontend` produces IR from the AST. These structures share much with the AST, with a few key differences:

- declarations are grouped by kind rather than ordered by appearance in the source file;
- names are lowered to `Variable` kinds (bare parameter, `p.r` external reference, or builtin) without checking that they exist, because designs applied later can add parameters (`oneil_analysis::validate_instance_graph` checks existence and reclassifies bare names after composition);
- parameter units are resolved against the builtin unit and prefix tables into composite units;
- parameters that a design overrides or adds carry a `DesignProvenance` recording where the overlay came from, so its expression is evaluated in the design's scope.

With the `ts-bindings` feature, the serializable IR types are exported to TypeScript through `ts-rs` for the VS Code Rendered View.

See [`docs/architecture/compiler-pipeline.md`](../../docs/architecture/compiler-pipeline.md) for where IR is produced, when it may change, and when it is fixed.
