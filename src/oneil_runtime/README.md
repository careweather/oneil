# Oneil Runtime

The runtime drives the Oneil compiler pipeline and owns every cache. `oneil_cli` and `oneil_lsp` depend on `Runtime` rather than calling the parser, frontend, analysis, and evaluator crates directly.

The phases, their validations, and what is fixed after each one are described in [`docs/architecture/compiler-pipeline.md`](../../docs/architecture/compiler-pipeline.md). This README covers how `Runtime` exposes them.

## Entry points

Each entry point runs the pipeline up to a particular phase, reusing whatever is already cached:

- `load_source`: reads a file into the source cache. If the content has changed, it invalidates the derived caches.
- `load_ast`: parses a model into an AST.
- `load_and_lower`: resolves a model and its dependencies and builds their per-unit instance graphs. Language-server navigation (hover, definition, references, rename) uses this.
- `check_model`: composes the root graph with any design and validates it, without evaluating. `oneil check` and language-server diagnostics use this.
- `eval_model` and `eval_model_and_expressions`: compose, validate, and evaluate. Evaluation is skipped when the composed graph has validation, cycle, or contribution errors.
- `get_dependency_tree`, `get_reference_tree`, and `get_independents`: evaluate, then run the `oneil_analysis` tree and independent-parameter analyses.

Entry points that take a `.one` design file redirect to the design's target model and apply the design at the root, so every consumer handles design files the same way.

Each entry point returns `RuntimeErrors`. These collect diagnostics from the AST cache, the composed graph's error buckets (falling back to cached unit graphs), the eval cache, and Python, including errors reached indirectly through references.

## Caches

`Runtime` holds:

- `SourceCache`, keyed by `SourcePath`;
- `AstCache`, keyed by `ModelPath`;
- per-file design metadata, keyed by `ModelPath`;
- `UnitGraphCache`, with one self-rooted `InstanceGraph` per `CompilationUnit`;
- the most recent composed graph, kept for diagnostics and the Rendered View;
- `EvalCache`, keyed by `EvalInstanceKey`;
- `PythonImportCache`, keyed by `PythonPath`;
- `PythonCallCache`, which persists Python call results under `__oncache__/` according to `CacheReadPolicy` and `CacheWritePolicy`.

Changing a source file clears the AST for that file, the whole unit graph cache, the design metadata, the eval cache, and the composed graph. It also drops the changed Python module and the modules that import it.

## External contexts

`Runtime` implements the context traits through which the phase crates reach the outside world: `oneil_frontend::ExternalResolutionContext` (loading ASTs and Python imports, and looking up builtins and units), `oneil_eval::ExternalEvaluationContext` (builtins, units, Python calls, models evaluated earlier), and `oneil_analysis::ExternalAnalysisContext` (evaluated models for tree analyses). This keeps the phase crates free of file system and Python state.
