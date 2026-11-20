# Oneil Architecture

The Rust code lives under `src/` as a single workspace. The **frontend** is what you use directly—the command-line app and editor support. The **backend** is what parses, resolves, and runs Oneil. The **`oneil_runtime`** crate ties the backend together: it loads files, keeps results in memory, and is what the CLI and language server call into.

**Data flow (simplified):** source text is parsed into an AST (**`oneil_ast`** / **`oneil_parser`**), per-file IR is produced and then composed into an `InstanceGraph` (**`oneil_ir`** / **`oneil_frontend`**), the composed graph is validated (**`oneil_analysis`**), and expressions are evaluated to values (**`oneil_eval`** / **`oneil_output`**). Standard units, prefixes, and builtins come from **`oneil_builtins`**. Optional Python interop is implemented in **`oneil_python`**. Cross-cutting types and helpers live in **`oneil_shared`**.

Further reading:

- [`compiler-pipeline.md`](compiler-pipeline.md) describes each phase: what it consumes and produces, what it validates, what is fixed afterwards, how caches are invalidated, and where a new check belongs.
- [`design-overlays.md`](design-overlays.md) is the implementation guide for designs, `apply`, and the two-pass instance graph. The motivating decision is [`../decisions/2026-04-24-two-pass-instance-graph.md`](../decisions/2026-04-24-two-pass-instance-graph.md).
- [`vscode-extension.md`](vscode-extension.md) describes the VS Code extension, its Rendered View webview, and how they talk to the language server.

The **`oneil`** crate is the entrypoint for external reuse. As a binary, it runs the Oneil CLI. As a library, it exports all other libraries with the `oneil_` prefix stripped.

The **`oneil_snapshot_tests`** crate is only for automated snapshot tests.

```mermaid
---
config:
    markdownAutoWrap: false
---

flowchart TD
    cli --> runtime
    lsp --> runtime
    runtime --> parser
    runtime --> resolver
    runtime --> evaluator
    runtime --> python
    runtime --> analysis
    runtime --> builtins

    subgraph frontend[Frontend]
        cli("`
            **CLI**
            _oneil_cli_
        `"):::frontend_element

        lsp("`
            **LSP**
            _oneil_lsp_
        `"):::frontend_element
    end

    class frontend frontend_container

    runtime("`
        **Runtime**
        _oneil_runtime_
    `"):::runtime

    subgraph backend[Backend]
        parser("`
            **Parser**
            _oneil_parser_
        `"):::backend_element

        resolver("`
            **Frontend**
            _oneil_frontend_
        `"):::backend_element

        evaluator("`
            **Evaluator**
            _oneil_eval_
        `"):::backend_element

        python("`
            **Python**
            _oneil_python_
        `"):::backend_element

        analysis("`
            **Analysis**
            _oneil_analysis_
        `"):::backend_element

        builtins("`
            **Builtins**
            _oneil_builtins_
        `"):::backend_element
    end

    class backend backend_container

    classDef frontend_element color:#000000,fill:#e3fae3,stroke:#249c2f
    classDef frontend_container color:#000000,fill:#f4fff4,stroke:#6ccb75
    classDef runtime color:#000000,fill:#fff3d9,stroke:#d46822
    classDef backend_element color:#000000,fill:#fbf0ff,stroke:#c443f8
    classDef backend_container color:#000000,fill:#fdf9ff,stroke:#e5a0ff
```

## What each crate is for

| Crate | What it does |
| --- | --- |
| **`oneil`** | Umbrella crate for downstream Rust projects: re-exports the other libraries under short names (for example `oneil::parser` instead of `oneil_parser`). The workspace default binary is the Oneil CLI. With optional features it can also build the Python extension as a shared library. |
| **`oneil_shared`** | Lowest-level shared types and helpers every crate can depend on without pulling in the rest of the compiler—things like source locations (spans), identifiers, and small utilities so the same concepts appear everywhere. |
| **`oneil_ast`** | Defines the abstract syntax tree: the data structures that represent Oneil source after lexing and parsing. The parser builds these; the resolver reads them. No execution logic—just the shape of the program as written. |
| **`oneil_ir`** | Defines the intermediate representation: a resolved, name-bound form of the program that the evaluator and analysis passes share. It sits between the raw syntax tree and running code. |
| **`oneil_output`** | Defines runtime values (numbers with units, aggregates, etc.) and how results are formatted or reported. The evaluator produces `oneil_output` values; the CLI and tools display them. |
| **`oneil_parser`** | Reads Oneil source text and produces an AST. Implemented with parser combinators (`nom`). Depends on `oneil_ast` and `oneil_shared` only—no resolution or evaluation. |
| **`oneil_frontend`** | Resolves per-file IR from the AST and runs the per-unit build pass that produces a cached `InstanceGraph` per compilation unit. Composition (cloning a cached root graph and overlaying runtime designs) also lives here. See `oneil_frontend/README.md` for crate-internal layout. |
| **`oneil_eval`** | Evaluates IR to values: the core interpreter for expressions and the dynamic semantics. Takes resolved IR and `oneil_output` types; does not parse or resolve by itself. |
| **`oneil_builtins`** | Supplies the standard environment: builtin functions, physical units, SI prefixes, and related values. Built on the evaluator and value types; the runtime installs this as the default builtins. |
| **`oneil_python`** | Bridges Oneil and an installed CPython: converting values and calling into Python from evaluated Oneil code. Used from the runtime path that supports Python interop, not from the parser alone. |
| **`oneil_py_call_cache`** | Persists Python call results to disk under `__oncache__/`, keyed by the import's content hash, the function, and its inputs, so unchanged Python calls are not re-run. The runtime's read and write cache policies control it. |
| **`oneil_analysis`** | Validation of the composed instance graph (name existence, variable classification, parameter cycles), plus analyses over evaluated models: dependency and reference trees and independent parameters. |
| **`oneil_runtime`** | The `Runtime` type and glue: caches for sources, ASTs, per-unit `InstanceGraph`s (`UnitGraphCache`), evaluated models, and Python; loads files; wires parser → frontend → analysis → eval; collects diagnostics. Both **`oneil_cli`** and **`oneil_lsp`** depend on this, not on each backend crate separately. |
| **`oneil_cli`** | The command-line interface: `eval`, `check`, `test`, `tree`, `independent`, `builtins`, and `lsp` (which starts **`oneil_lsp`**), with file watching and hidden `dev` commands for printing the AST and IR. |
| **`oneil_lsp`** | Language Server Protocol server for editors: diagnostics, hover, definition, references, rename, highlights, and the `oneil/instanceTree` command behind the Rendered View, implemented on top of **`oneil_runtime`**. |
| **`oneil_snapshot_tests`** | Integration tests that run the runtime against fixtures and compare output to saved snapshots. Keeps behavior stable across refactors; not shipped to end users. |

## External libraries

Each choice below is the default for its concern. Prefer it over adding a new dependency that does the same job.

| Concern | Library | Used in |
| --- | --- | --- |
| Parsing | `nom` with `nom_locate` (combinators in a recursive-descent style; no separate lexer) | `oneil_parser` |
| Ordered maps and sets | `indexmap` | most crates |
| Serialization of wire types | `serde`, `serde_json` | `oneil_shared`, `oneil_ir`, `oneil_output`, `oneil_lsp`, `oneil_cli`, `oneil_py_call_cache` |
| TypeScript bindings for wire types | `ts-rs` (behind the `ts-bindings` feature; output in `packages/ts-interfaces`) | `oneil_shared`, `oneil_ir`, `oneil_output`, `oneil_lsp`, `oneil_cli` |
| Python interop | `pyo3` (stable `abi3-py310` ABI) | `oneil_python`, `oneil` (optional Python extension module) |
| Hashing Python sources | `xxhash-rust` | `oneil_python` |
| Command-line parsing | `clap` (derive) | `oneil_cli` |
| File watching | `notify` | `oneil_cli` |
| Terminal colour | `owo-colors`, `anstream` | `oneil_cli`, `oneil_analysis` |
| Language server | `tower-lsp-server` on `tokio` | `oneil_lsp` |
| Snapshot tests | `insta` | `oneil_snapshot_tests` |
| Fuzzing | `cargo fuzz` with `arbitrary` | `oneil_output/fuzz` |
