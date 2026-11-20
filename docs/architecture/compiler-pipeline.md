# Compiler Pipeline

This document describes the phases that turn Oneil source into evaluated values: what each phase consumes and produces, what it validates, and what is fixed once it has run. It describes contracts between crates rather than their internals. Internals belong in each crate's `README.md` and rustdoc.

For the overlay-specific details of the instance graph (designs, `apply`, contributions, provenance), see [`design-overlays.md`](design-overlays.md). For the reasoning behind the two-pass graph build, see the [two-pass instance graph decision](../decisions/2026-04-24-two-pass-instance-graph.md).

## Overview

`oneil_runtime::Runtime` drives every phase and owns every cache. The CLI and the language server call the runtime, never the phase crates directly.

```mermaid
flowchart LR
    src["Source text"] --> parse["Parse"]
    parse --> resolve["Resolve"]
    resolve --> unit["Per-unit build"]
    unit --> compose["Composition"]
    compose --> validate["Validation"]
    validate --> evaluate["Evaluation"]
    evaluate --> report["Reporting"]
```

| Phase | Crate | Produces | Cached in |
| --- | --- | --- | --- |
| Source load | `oneil_runtime` | source text | `SourceCache` (by `SourcePath`) |
| Parse | `oneil_parser` | `oneil_ast::ModelNode` | `AstCache` (by `ModelPath`) |
| Resolve | `oneil_frontend` (resolver) | `InstancedModel` template and design metadata per file | `Runtime` design info (by `ModelPath`) |
| Per-unit build | `oneil_frontend` (instance) | self-rooted `InstanceGraph` | `UnitGraphCache` (by `CompilationUnit`) |
| Composition | `oneil_frontend` (instance) | composed `InstanceGraph` | most recent graph only, on `Runtime` |
| Validation | `oneil_analysis` | diagnostics on the composed graph | (part of the composed graph) |
| Evaluation | `oneil_eval` | `oneil_output::Model` values | `EvalCache` (by `EvalInstanceKey`) |
| Reporting | `oneil_runtime`, `oneil_analysis`, `oneil_cli`, `oneil_lsp` | `RuntimeErrors`, `OneilDiagnostic`, trees, rendered output | none |

Every phase is error-tolerant: it keeps going past errors, records diagnostics, and hands a partial result (`LoadResult` / `MaybePartialResult`) to the next phase. One phase gate is deliberate: evaluation does not run when the composed graph has validation, cycle, or contribution errors, because those mean the IR is structurally broken.

## Phases

### Source load

- **Input:** a `SourcePath` (`.on` model, `.one` design, or `.py` import).
- **Output:** file text in `SourceCache`, with a content hash.
- **Validates:** that the file exists and can be read (`SourceError`).
- **Fixed after:** the text is the single source of truth until it is reloaded. Reloading text that has changed invalidates the derived caches (see [Caches and invalidation](#caches-and-invalidation)).

Source of truth: `oneil_runtime::Runtime::load_source`.

### Parse

- **Input:** source text.
- **Output:** `oneil_ast::ModelNode`. Parsing may produce a partial AST alongside errors.
- **Validates:** syntax only, including the structural rules of design headers. The parser does no name, import, or unit lookups (`oneil_parser::error::ParserError`).
- **Fixed after:** the AST is immutable for a given source version. Later phases read it and never rewrite it.

The parser is built from `nom` combinators (with `nom_locate` for spans) in a recursive-descent style; there is no separate lexer. The grammar is specified in [`../specs/grammar.ebnf`](../specs/grammar.ebnf), and parser changes must keep that grammar up to date.

Source of truth: `oneil_parser::parse_model`.

### Resolve (per-file lowering)

- **Input:** the AST for a file and, through `ExternalResolutionContext`, the ASTs of its imports, its Python imports, and the builtin tables.
- **Output:** one result per file in the dependency closure. Each result holds:
  - an `InstancedModel` *template*, whose expressions and parameters are `oneil_ir` types;
  - the design metadata (`ModelDesignInfo`): the file's `Design` export, if it is a design file, and its `ApplyDesign` declarations;
  - a `ResolutionErrorCollection`.
- **Validates:** everything that only needs a single file plus the files it names:
  - model and Python import paths, and that Python symbols exist;
  - duplicate parameter, reference, and submodel names;
  - unit expressions against the builtin unit and prefix tables;
  - the design surface (`design <target>` matching the resolved target, and the shape of `apply` targets);
  - lowering of test and parameter expressions.
- **Deliberately not validated:** whether a variable names a real parameter or reference. Designs can add parameters, so existence is checked during validation, after composition.
- **Fixed after:** a template is immutable until its file is resolved again. Graph building clones templates and never edits them in place.

Source of truth: `oneil_frontend::load_model` and the `oneil_frontend::resolver` module.

### Per-unit build

- **Input:** templates and design metadata for a compilation unit and its dependencies.
- **Output:** one self-rooted `InstanceGraph` per `CompilationUnit`, where a unit is either a `.on` model or a `.one` design. The graph includes the unit's own `apply` declarations and every transitive dependency spliced into its tree, but no designs supplied at runtime.
- **Validates:**
  - import cycles between compilation units (`CompilationCycleError`);
  - the contributions from the unit's own `apply` declarations: overlay target missing, unit mismatch (`ContributionDiagnostic`).
- **Fixed after:** a cached unit graph is never mutated. Composition and every later phase work on a clone.

Each graph carries four error buckets (`resolution_errors`, `cycle_errors`, `contribution_errors`, `validation_errors`). Individual `InstancedModel` nodes hold no errors.

Source of truth: `oneil_frontend::build_unit_graph` and `oneil_frontend::instance::graph`.

### Composition

- **Input:** the cached graph for the root unit and the runtime designs (for example `oneil eval --design x.one`, or a `.one` file opened directly, which redirects to its target model).
- **Output:** a fresh, composed `InstanceGraph` rooted at the requested model.
- **Validates:** the same contribution checks as the per-unit build, for runtime designs; any errors are recorded on the composed graph.
- **Fixed after:** composition is the only place runtime designs are applied. After it, no phase adds, removes, or overrides parameters or instances.

Composition is not cached. It runs on every `check_model` / `eval_model` call, and its cost grows linearly with the number of instances.

Source of truth: `oneil_frontend::apply_designs`.

### Validation

- **Input:** the composed `InstanceGraph` and a builtin lookup.
- **Output:** the same graph, with variables reclassified and `validation_errors` filled in.
- **Validates:**
  - reclassification of bare names as parameters or builtins against each instance's final scope, which handles design additions and builtin shadowing;
  - undefined bare names;
  - undefined reference names, and parameters missing on a referenced instance (`p.r`);
  - parameter dependency cycles across instances.
- **Fixed after:** the composed graph's structure and variable classification are final. Evaluation only reads the graph. Validation clears its own bucket first, so running it again gives the same result.

Source of truth: `oneil_analysis::validate_instance_graph`.

### Evaluation

- **Input:** the validated composed graph and an `ExternalEvaluationContext` (builtins, units, Python calls, models evaluated earlier).
- **Output:** one `oneil_output::Model` per instance (parameter values, test results), stored in `EvalCache`, possibly partial with `ModelEvalErrors`.
- **Validates:** everything that needs values:
  - unit and dimension compatibility of operations;
  - type rules (booleans, strings, and unitless or measured numbers, each scalar or interval);
  - limits, piecewise branches with no match, and numeric domain errors;
  - evaluation-time cycles;
  - Python exceptions;
  - test assertions.
- **Fixed after:** values in `EvalCache` are final for the current sources.

Today, expression type and unit checking happens here. Before evaluation, the only type-like checks are resolving declared units and the unit check on design overrides. Checks belong in the earliest phase that has their information ([check as early as possible](../decisions/2026-11-20-check-as-early-as-possible.md)). So rules that can be decided without values should move earlier as they are implemented, while value-dependent rules stay here: exponents in `^`, Python arguments and return values, limits, and piecewise selection. Evaluation is lazy and memoized per parameter: a parameter is computed when it is first needed, and cycles are detected when evaluation re-enters a parameter that is still in progress.

Python calls go through `oneil_python` (pyo3). Their results can be persisted to disk by `oneil_py_call_cache`.

Source of truth: `oneil_eval::eval_model_from_graph`.

### Reporting

- **Diagnostics:** `Runtime::get_model_diagnostics` collects errors from the AST cache, the composed graph's buckets (falling back to the unit graph cache), the eval cache, and Python. It follows chains of errors through references, and every error converts to the shared `oneil_shared::error::OneilDiagnostic` envelope.
- **Analysis over values:** dependency and reference trees and independent parameters (`oneil_analysis::get_dependency_tree`, `get_reference_tree`, `get_independents`) run after evaluation.
- **Output:** value formatting lives in `oneil_output`. Terminal printing lives in `oneil_cli`, and the editor payloads live in `oneil_lsp`.

## Which entry points stop where

| Entry point | Last phase run |
| --- | --- |
| `Runtime::load_ast`, `oneil dev print-ast` | Parse |
| `Runtime::load_and_lower`, `oneil dev print-ir`, LSP hover, definition, references, rename, highlight | Per-unit build |
| `Runtime::check_model`, `oneil check`, LSP diagnostics on open and save, LSP workspace discovery | Validation |
| `Runtime::eval_model`, `oneil eval`, `oneil test`, LSP `oneil/instanceTree` (Rendered View) | Evaluation |
| `oneil tree`, `oneil independent` | Evaluation, then analysis |

The language server refreshes diagnostics when a file is opened and when it is saved. It does not refresh them while the user types.

## Caches and invalidation

| Cache | Key | Holds |
| --- | --- | --- |
| `SourceCache` | `SourcePath` | text and content hash |
| `AstCache` | `ModelPath` | AST and parse errors |
| design info | `ModelPath` | `ModelDesignInfo` per file |
| `UnitGraphCache` | `CompilationUnit` | self-rooted `InstanceGraph` |
| composed graph | (single slot) | the most recent composed graph, kept for diagnostics and the Rendered View |
| `EvalCache` | `EvalInstanceKey` | evaluated models |
| `PythonImportCache` | `PythonPath` | loaded Python modules and their local imports |
| `PythonCallCache` | Python path, function, inputs, and import hash | call results, persisted under `__oncache__/` and governed by `CacheReadPolicy` / `CacheWritePolicy` |

Invalidation is currently coarse. When `load_source` sees changed content, it removes that file's AST and clears the whole unit graph cache, the design info, the eval cache, and the composed graph. It also drops the changed Python module and every module that imports it. Per-unit invalidation is possible because unit graphs are self-rooted, but it is not implemented yet.

## Where a new check or transformation belongs

The same rules apply to checks (which emit diagnostics) and to transformations (which rewrite IR, such as reclassifying bare names). Put each one in the earliest phase that has the information it needs:

- **Parse:** the check depends only on the text of one file.
- **Resolve:** the check needs one file plus the files and Python modules it imports, but not the final set of parameters on an instance.
- **Per-unit build or composition:** the check concerns how units are combined (cycles, overlay targets, contributions). Record it in the graph's `cycle_errors` or `contribution_errors`.
- **Validation:** the check or transformation needs the final scope of each instance after every design has been applied (existence, classification, structural cycles). Record diagnostics in `validation_errors`.
- **Evaluation:** the check needs computed values or runtime types. Return an `EvalError`.

Within a phase, fold the work into the traversal that phase already does. A check or transformation that only needs the node being visited (and data an earlier phase has fixed) runs in the same walk as the other per-node work, even if it must run after another step on that same node; for example, classify a node's variables and then check it in one visit. Add a separate, sequential pass only when the work needs a result that covers the whole graph first, such as a dependency graph that has to be complete before cycles can be found. When adding such a pass, say in its rustdoc which whole-graph result it waits for.

Checks and transformations must not mutate data from an earlier phase. In particular, don't edit ASTs, templates, or cached unit graphs. A transformation rewrites only the data its own phase produced (for example, validation rewrites the composed graph), and it must give the same result when run again. Add new diagnostic types to the owning crate's error module, and implement conversion to `OneilDiagnostic` so the CLI and the language server display them without extra work.

## Testing by phase

- **Parser and resolver:** unit tests next to the code.
- **Whole pipeline:** snapshot tests in `oneil_snapshot_tests` (run through `Runtime`, compared with `insta`).
- **Interval arithmetic in `oneil_output`:** `cargo fuzz` targets driven by `arbitrary` (run on pull requests by `rust-pr.yml`).
- **Language server:** tests under `oneil_lsp/src/test`.
