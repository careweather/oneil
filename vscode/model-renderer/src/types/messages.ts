/**
 * Messages the VS Code extension posts into the webview, and messages the
 * webview posts back to the extension.
 *
 * Keep in sync with the TypeScript in `vscode/src/webview/panel.ts`.
 */

import type { RenderedTree } from "./model"

// ── Extension → webview ───────────────────────────────────────────────────────

export type ExtensionMessage =
    | { type: "loading" }
    | {
        type: "instanceTree"
        data: RenderedTree
        /**
         * Raw BibTeX content from workspace `.bib` file(s), or
         * `null` when no such file exists. Sent alongside every tree refresh
         * so citations in notes can be formatted with author/year.
         */
        bibliography: string | null
        /**
         * Webview-accessible URI for the workspace root folder, converted by
         * the extension via `webview.asWebviewUri(workspaceRoot)`. Used to
         * resolve relative image paths in notes (e.g. `![](./img/foo.png)`).
         * `null` when no workspace folder is open.
         */
        workspaceUri: string | null
        /**
         * Webview-accessible URI for the directory containing the file being
         * rendered. Used as a fallback base for resolving relative image paths
         * when the image is not found relative to the workspace root.
         * `null` when the file URI is not available.
         */
        fileBaseUri: string | null
        /**
         * Webview-accessible URI for the user's PDF cache directory, converted
         * by the extension via `webview.asWebviewUri(cacheDirUri)`. The webview
         * constructs inline PDF URLs as `pdfCacheUri + "/" + bareFilename`.
         */
        pdfCacheUri: string | null
        /**
         * The URI of the `.one` file being rendered. Used by the webview to
         * detect file switches (as opposed to same-file reloads) so it can
         * reset UI state only when the file actually changes.
         */
        fileUri: string
      }
    | { type: "error"; message: string }

// ── Webview → extension ───────────────────────────────────────────────────────

export type WebviewMessage =
    /** Sent once on mount so the extension knows the webview is ready. */
    | { type: "ready" }
    /** Requests the extension to re-fetch and push a fresh instance tree. */
    | { type: "reload" }
    /**
     * Requests the extension to open a PDF in the dedicated PDF viewer panel,
     * optionally at a specific page.  The extension resolves `cachePath`
     * against the workspace root and source-file directory; if no local file
     * is found it falls back to opening `pdfUrl` in the system browser.
     */
    | {
        type: "openPdf"
        /** Remote URL of the PDF (e.g. a datasheet on a manufacturer's site). */
        pdfUrl: string | null
        /** Workspace-relative or absolute path to a cached local copy. */
        cachePath: string | null
        /** 1-based page to open to. */
        page: number | null
        /** Human-readable title shown in the panel tab. */
        title: string
        /**
         * BibTeX citation key for the entry.  Passed back to the extension so
         * it can offer to update `references.bib` with the cached path after
         * a successful download.
         */
        citationKey: string
      }
