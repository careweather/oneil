# Rendered View JSON Schema

## Status

Accepted

## Context

We want a VS Code rendered view that shows the evaluated instance tree of an Oneil model: parameter values, original expressions, documentation notes, applied designs, tests, and hierarchy (submodels and reference imports). The UI needs data from two layers: the evaluated output (`oneil_output::Model`) and the IR (`oneil_ir::Parameter`, `ir::Note`), which carries the original expressions and prose.

The client is a React application (**model renderer**) bundled into the extension under `vscode/out/model-renderer/` and loaded by `vscode/src/webview/panel.ts`. State (navigation, detail panel, graph layout, etc.) lives in the renderer; the extension’s job is to host the webview, push JSON + ancillary payloads, and react to `ready` / `reload`.

## Decision

### LSP payload

A custom LSP command `oneil/instanceTree` returns a JSON payload describing the rooted instance tree. The payload is built in `oneil_lsp` by combining the evaluated `output::Model` with the IR parameter definitions accessed via the runtime’s template reference. Each node carries its identity, evaluated values, original expressions, notes, child submodel nodes, and reference cross-links.

Serialization uses `serde` as a direct dependency of `oneil_shared`, `oneil_output`, and `oneil_ir`. The schema is defined by Rust types via `serde` derives and mirrored manually as TypeScript types in `vscode/model-renderer/src/types/model.ts`.

The top-level shape matches `RenderedTree`: a `root` node plus a `reference_pool` of fully rendered imported models.

### Extension ↔ webview transport

Beyond `RenderedTree`, each successful push includes:

- `bibliography`: raw `references.bib` text or `null`
- `workspaceUri`: webview URI for the workspace root (for resolving images in notes), or `null`
- `fileUri`: string URI of the `.one` document being rendered

The renderer uses `fileUri` to tell **file switches** from **same-document refreshes**: only when `fileUri` changes does it clear equation selection and reset panel-open state, so focus-driven reloads do not wipe UI preferences.

Messages are typed in `vscode/model-renderer/src/types/messages.ts` and kept in sync with `vscode/src/webview/panel.ts`. The webview posts `ready` (after attaching its listener) and may post `reload` (dev) to request another fetch.

The extension avoids redundant LSP work when the panel merely gains focus while already visible; it refreshes when the panel transitions from hidden to visible, when the active Oneil editor changes, when an Oneil document is saved, and on `ready` / explicit reload.

### Bidirectional product API

Sweep parameter overrides and similar features remain deferred; the API will be decided when needed.

## Consequences

- `serde` is a direct (unconditional) dependency of `oneil_shared`, `oneil_output`, and `oneil_ir`; `serde_json` is added to `oneil_lsp`.
- The LSP server runs a full `eval_model` pass for the rendered view (more expensive than the `check_model` path used for diagnostics); this is acceptable because it is user-initiated.
- TypeScript types must stay aligned with Rust manually (no codegen yet).
- The renderer is a separate npm package (`vscode/model-renderer/`), built with `npm run build:webview` from `vscode/` into `out/model-renderer/`.
- **Graph view — unused-parameter filter** is purely client-side over the same JSON. Two modes exist in `computeUsedParams` (`GRAPH_USED_PARAMS_MODE` in `vscode/model-renderer/src/utils/computeUsedParams.ts`): **`transitive`** (hide params not reachable from root performance outputs / root tests along dependency edges) and **`direct_submodel`** (same reachability seed, but non-root params are shown only if referenced from **outside** their instance subtree — typically parent→child externals). The UI currently hardcodes **`direct_submodel`** until a setting exists.
- Future: parameter sweep inputs and diff at the JSON level extend this schema.

## Future Work: Interactive Variable Hovering in Equations

To enable hover tooltips on variable references within rendered KaTeX equations:

1. **Modify `exprToLatex.ts`** (model renderer) to generate custom KaTeX macros like `\href{#param:var_name}{\mathrm{var}}` or define a custom command that wraps variables in spans with data attributes.

2. **Post-process rendered output** after KaTeX renders: use `querySelectorAll('[data-param]')` to attach event listeners or event delegation on the equation container.

3. **Context mapping** — pass available parameters to the expression renderer so each variable resolves as local parameter, external reference (`model.param`), or builtin.

4. **Tooltip content** — show label, value, unit, and optionally the note; for externals, include source model path.
