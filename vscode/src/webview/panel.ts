import * as vscode from "vscode"
import { LanguageClient } from "vscode-languageclient/node"
import { openPdfExternal } from "./pdfPanel"
import { getCacheDirPath } from "../pdf/cache"
import { readBibliography } from "../bibliography/locate"

/** The command name the LSP registers as an executeCommand handler. */
const INSTANCE_TREE_COMMAND = "oneil/instanceTree"

/**
 * Singleton panel instance; at most one rendered view is open at a time.
 */
let currentPanel: RenderedViewPanel | undefined

/**
 * Reloads the currently open rendered view, if any. No-op if no panel exists.
 */
export async function reloadRenderedView(): Promise<void> {
    if (currentPanel) {
        await currentPanel.refresh()
    }
}

/**
 * Opens (or reveals) the rendered view for the given URI, fetching instance
 * tree data from the language server and posting it to the webview.
 *
 * If a panel for a different file is already open it is replaced.
 */
export async function openRenderedView(
    uri: vscode.Uri,
    client: LanguageClient,
    context: vscode.ExtensionContext,
): Promise<void> {
    if (currentPanel) {
        // Webview already mounted — reveal and refresh immediately.
        currentPanel.reveal(uri)
        await currentPanel.refresh()
    } else {
        // New panel — refresh is triggered by the "ready" message from the
        // React app once it has mounted its message listener.
        currentPanel = new RenderedViewPanel(uri, client, context)
        currentPanel.onDispose(() => {
            currentPanel = undefined
        })
    }
}

// ── Panel class ───────────────────────────────────────────────────────────────

class RenderedViewPanel {
    private readonly panel: vscode.WebviewPanel
    private uri: vscode.Uri
    private readonly client: LanguageClient
    private readonly context: vscode.ExtensionContext
    private readonly disposables: vscode.Disposable[] = []
    private disposed = false

    constructor(
        uri: vscode.Uri,
        client: LanguageClient,
        context: vscode.ExtensionContext,
    ) {
        this.uri = uri
        this.client = client
        this.context = context

        const webviewDistUri = vscode.Uri.joinPath(context.extensionUri, "out", "model-renderer")
        const workspaceFolders = vscode.workspace.workspaceFolders
        const workspaceRoot = workspaceFolders?.[0]?.uri

        const localResourceRoots: vscode.Uri[] = [webviewDistUri]
        if (workspaceRoot) localResourceRoots.push(workspaceRoot)
        // Also allow loading resources from the directory containing the file
        // being rendered, so relative image paths like `![](./img/foo.png)`
        // resolve correctly even when the file isn't in the workspace root.
        const fileDir = vscode.Uri.joinPath(uri, "..")
        if (!workspaceRoot || !fileDir.path.startsWith(workspaceRoot.path)) {
            localResourceRoots.push(fileDir)
        }
        // Allow the PDF cache directory so cached PDFs can be rendered inline
        // by the react-pdf viewer in the webview.
        localResourceRoots.push(vscode.Uri.file(getCacheDirPath()))

        this.panel = vscode.window.createWebviewPanel(
            "oneilRenderedView",
            `Oneil: ${basename(uri)}`,
            vscode.ViewColumn.Beside,
            {
                enableScripts: true,
                localResourceRoots,
                // Keep the webview alive when it is not the active tab so that
                // switching back to it does not trigger a full reload (and
                // re-initialise the PDF.js worker from scratch).
                retainContextWhenHidden: true,
            },
        )

        this.panel.webview.html = appHtml(this.panel.webview, webviewDistUri)

        // Handle messages sent from the webview to the extension.
        this.panel.webview.onDidReceiveMessage(
            (message: unknown) => this.handleWebviewMessage(message),
            undefined,
            this.disposables,
        )

        // Refresh when the panel transitions from hidden to visible (e.g. the
        // user switches back to it after it was backgrounded, or VS Code
        // recreates the webview context after it was hidden).
        // We do NOT refresh when the panel merely gains focus while already
        // visible — that happens on every click inside the webview and would
        // cause a spurious round-trip to the language server.
        let wasVisible = this.panel.visible
        this.panel.onDidChangeViewState(
            ({ webviewPanel }) => {
                if (webviewPanel.visible && !wasVisible) {
                    void this.refresh()
                }
                wasVisible = webviewPanel.visible
            },
            undefined,
            this.disposables,
        )

        // Follow the active editor — when the user switches to a different
        // Oneil file the panel silently re-targets and refreshes.
        //
        // If the file opened in the same column as the rendered view (which
        // happens when the webview had focus), we move the editor to the
        // first column that doesn't contain the webview so the user can see
        // both side-by-side without having to drag tabs manually.
        vscode.window.onDidChangeActiveTextEditor(
            (editor) => {
                if (!editor || editor.document.languageId !== "oneil") { return }

                if (editor.document.uri.toString() !== this.uri.toString()) {
                    this.uri = editor.document.uri
                    this.panel.title = `Oneil: ${basename(this.uri)}`
                }
                void this.refresh()

                // If the file opened in the same column as the rendered view,
                // move it (not copy) to the adjacent group using a VS Code
                // command. This avoids the duplication that showTextDocument
                // causes by opening a second instance in the target column.
                const webviewColumn = this.panel.viewColumn
                if (webviewColumn !== undefined && editor.viewColumn === webviewColumn) {
                    void vscode.commands.executeCommand("workbench.action.moveEditorToPreviousGroup")
                }
            },
            undefined,
            this.disposables,
        )

        // Refresh whenever any Oneil file is saved — saving a dependency can
        // change the evaluated tree for the root model too.
        vscode.workspace.onDidSaveTextDocument(
            (doc) => {
                if (doc.languageId === "oneil") {
                    void this.refresh()
                }
            },
            undefined,
            this.disposables,
        )

        this.panel.onDidDispose(() => this.dispose(), undefined, this.disposables)
    }

    /**
     * Reveals the panel. If the URI has changed the title is updated and data
     * is re-fetched on the next `refresh()` call.
     */
    reveal(uri: vscode.Uri): void {
        if (uri.toString() !== this.uri.toString()) {
            this.uri = uri
            this.panel.title = `Oneil: ${basename(uri)}`
        }
        this.panel.reveal(undefined, false)
    }

    /**
     * Fetches a fresh instance tree from the LSP and posts it to the webview.
     * Also reads bibliography files from the workspace (if present) and
     * includes them so the webview can format citations with author/year.
     */
    async refresh(): Promise<void> {
        if (this.disposed) return

        this.panel.webview.postMessage({ type: "loading" })

        try {
            const [tree, bibliography] = await Promise.all([
                this.client.sendRequest<unknown>(
                    "workspace/executeCommand",
                    {
                        command: INSTANCE_TREE_COMMAND,
                        arguments: [this.uri.toString()],
                    },
                ),
                readBibliography(this.uri),
            ])
            if (this.disposed) return
            const folders = vscode.workspace.workspaceFolders
            const workspaceUri = folders?.[0]?.uri
                ? this.panel.webview.asWebviewUri(folders[0].uri).toString()
                : null
            const fileDirUri = vscode.Uri.joinPath(this.uri, "..")
            const fileBaseUri = this.panel.webview.asWebviewUri(fileDirUri).toString()
            const pdfCacheUri = this.panel.webview.asWebviewUri(vscode.Uri.file(getCacheDirPath())).toString()
            this.panel.webview.postMessage({ type: "instanceTree", data: tree, bibliography, workspaceUri, fileBaseUri, pdfCacheUri, fileUri: this.uri.toString() })
        } catch (err) {
            if (this.disposed) return
            const message = err instanceof Error ? err.message : String(err)
            this.panel.webview.postMessage({ type: "error", message })
        }
    }

    /** Registers a callback for when the panel is disposed. */
    onDispose(cb: () => void): void {
        this.panel.onDidDispose(cb, undefined, this.disposables)
    }

    private handleWebviewMessage(message: unknown): void {
        if (typeof message !== "object" || message === null || !("type" in message)) return
        const msg = message as { type: string }

        if (msg.type === "ready") {
            this.refresh()
            return
        }
        if (msg.type === "reload") {
            void this.refresh()
            return
        }
        if (msg.type === "openPdf") {
            const m = msg as {
                type: "openPdf"
                pdfUrl: string | null
                cachePath: string | null
                page: number | null
                title: string
                citationKey: string
            }
            void openPdfExternal({
                pdfUrl: m.pdfUrl,
                cachePath: m.cachePath,
                page: m.page,
                title: m.title,
                citationKey: m.citationKey,
                sourceUri: this.uri,
            })
        }
    }

    private dispose(): void {
        this.disposed = true
        for (const d of this.disposables) {
            d.dispose()
        }
        this.disposables.length = 0
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/** Returns the filename portion of a URI (e.g. `satellite.on`). */
function basename(uri: vscode.Uri): string {
    return uri.path.split("/").at(-1) ?? uri.path
}

/**
 * Returns the HTML shell that loads the built React bundle from
 * `out/model-renderer/assets/index.js`.
 *
 * The Content-Security-Policy allows only scripts from the extension's own
 * `out/model-renderer` directory (via the nonce-less `'self'`-equivalent webview
 * source scheme that VS Code requires).
 */
function appHtml(webview: vscode.Webview, distUri: vscode.Uri): string {
    const scriptUri = webview.asWebviewUri(
        vscode.Uri.joinPath(distUri, "assets", "index.js"),
    )
    const styleUri = webview.asWebviewUri(
        vscode.Uri.joinPath(distUri, "assets", "index.css"),
    )
    // Pre-fetch the PDF.js worker module at HTML-parse time so it is already
    // in the browser cache when warmPdfWorker() calls new Worker() later.
    // This matches how tomoki1207/vscode-pdfviewer loads pdf.worker.js via a
    // <script> tag, giving Chromium the chance to fetch + compile the 1 MB
    // worker file in parallel with the main React bundle instead of waiting
    // for useEffect to fire.
    const pdfWorkerUri = webview.asWebviewUri(
        vscode.Uri.joinPath(distUri, "assets", "pdf.worker.min.mjs"),
    )
    const nonce = getNonce()

    return /* html */ `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <meta http-equiv="Content-Security-Policy"
        content="default-src 'none';
                 style-src ${webview.cspSource} 'unsafe-inline';
                 script-src 'nonce-${nonce}' ${webview.cspSource};
                 worker-src ${webview.cspSource} blob:;
                 connect-src ${webview.cspSource};
                 font-src ${webview.cspSource};
                 img-src ${webview.cspSource} https: data:;" />
  <title>Oneil Rendered View</title>
  <link rel="stylesheet" href="${styleUri}" />
  <link rel="modulepreload" href="${pdfWorkerUri}" />
</head>
<body>
  <div id="root"></div>
  <script type="module" nonce="${nonce}" src="${scriptUri}"></script>
</body>
</html>`
}

/** Generates a cryptographically random nonce for use in the CSP. */
function getNonce(): string {
    const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
    return Array.from({ length: 32 }, () => chars[Math.floor(Math.random() * chars.length)]).join("")
}
