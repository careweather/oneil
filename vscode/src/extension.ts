import * as vscode from "vscode"
import { LanguageClient, LanguageClientOptions, ServerOptions } from "vscode-languageclient/node"
import { openRenderedView, reloadRenderedView } from "./webview/panel"
import { registerImagePathDiagnostics } from "./diagnostics/imagePaths"
import { toggleOfflineMode, isOfflineMode, getCacheDirPath } from "./pdf/cache"

let client: LanguageClient | undefined

export async function activate(context: vscode.ExtensionContext) {
    registerImagePathDiagnostics(context)

    client?.info("starting language server")
    await restartLanguageServer(context)
    client?.info("language server started")

    // ── Commands ───────────────────────────────────────────────────────────────

    context.subscriptions.push(
        vscode.commands.registerCommand("oneil.restartLanguageServer", () =>
            restartLanguageServer(context),
        ),
        vscode.commands.registerCommand("oneil.openRenderedView", async () => {
            const editor = vscode.window.activeTextEditor
            if (!editor) {
                void vscode.window.showWarningMessage(
                    "Oneil: open a Oneil file before opening the rendered view.",
                )
                return
            }
            if (editor.document.languageId !== "oneil") {
                void vscode.window.showWarningMessage(
                    "Oneil: the active file is not a Oneil file (.on or .one).",
                )
                return
            }
            if (!client) {
                void vscode.window.showWarningMessage(
                    "Oneil: language server is not running.",
                )
                return
            }
            await openRenderedView(editor.document.uri, client, context)
        }),
        vscode.commands.registerCommand("oneil.reloadRenderedView", () =>
            reloadRenderedView(),
        ),
        vscode.commands.registerCommand("oneil.pdf.toggleOfflineMode", async () => {
            await toggleOfflineMode()
            updateStatusBar(statusBar)
        }),
    )

    // ── PDF offline-mode status bar item ───────────────────────────────────────

    const statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 90)
    statusBar.command = "oneil.pdf.toggleOfflineMode"
    statusBar.tooltip = new vscode.MarkdownString(
        "**Oneil PDF mode**\n\nClick to toggle between online (download) and offline (cache only) mode.\n\n" +
        `Cache directory: \`${getCacheDirPath()}\``,
    )
    statusBar.tooltip.isTrusted = true
    updateStatusBar(statusBar)
    statusBar.show()
    context.subscriptions.push(statusBar)

    // Keep the status bar in sync when the setting changes externally.
    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration((e) => {
            if (e.affectsConfiguration("oneil.pdf.offlineOnly")) {
                updateStatusBar(statusBar)
            }
        }),
    )

    client?.info("extension is now active!")
}

/** Refreshes the status bar label to reflect the current offline/online mode. */
function updateStatusBar(item: vscode.StatusBarItem): void {
    if (isOfflineMode()) {
        item.text = "$(database) Oneil PDFs: Offline"
        item.backgroundColor = new vscode.ThemeColor("statusBarItem.warningBackground")
    } else {
        item.text = "$(cloud) Oneil PDFs: Online"
        item.backgroundColor = undefined
    }
}

export function deactivate(): Thenable<void> | undefined {
    return client?.stop()
}

/**
 * Builds server and client options from the current Oneil configuration.
 */
function buildOptions(): { serverOptions: ServerOptions; clientOptions: LanguageClientOptions } {
    const config = vscode.workspace.getConfiguration("oneil")
    const configuredPath = config.get<string | null>("serverPath", null)
    const command = configuredPath ?? process.env.ONEIL_PATH ?? "oneil"

    return {
        serverOptions: { command, args: ["lsp"] },
        clientOptions: {
            documentSelector: [
                { scheme: "file", language: "oneil" },
                { scheme: "file", language: "python" },
            ],
        },
    }
}

/**
 * Restarts the Oneil language server. Uses the current configuration (e.g. serverPath).
 */
async function restartLanguageServer(context: vscode.ExtensionContext): Promise<void> {
    if (client == null) {
        const { serverOptions, clientOptions } = buildOptions()

        const newClient = new LanguageClient(
            "oneil-language-server",
            "Oneil Language Server",
            serverOptions,
            clientOptions,
        )
        await newClient.start()

        client = newClient
        client.info("language server initialized")
    } else {
        client.info("restarting language server")
        await client.restart()
        client.info("language server restarted")
    }
}
