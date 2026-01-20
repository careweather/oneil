import * as vscode from "vscode"
import { LanguageClient, LanguageClientOptions, ServerOptions } from "vscode-languageclient/node"

export async function activate(context: vscode.ExtensionContext) {
    const serverOptions: ServerOptions = {
        command: process.env.ONEIL_PATH ?? "oneil",
        args: ["lsp"],
    }

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: "file", language: "oneil" }],
    }

    const client = new LanguageClient(
        "oneil-language-server",
        "Oneil Language Server",
        serverOptions,
        clientOptions,
    )

    await client.start()

    console.log("Oneil extension is now active!")
}

export function deactivate() { }

