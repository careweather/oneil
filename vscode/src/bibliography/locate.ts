import * as vscode from "vscode"

const BIB_EXCLUDE = "**/node_modules/**"

function addUnique(out: vscode.Uri[], seen: Set<string>, uri: vscode.Uri): void {
    const key = uri.toString()
    if (!seen.has(key)) {
        seen.add(key)
        out.push(uri)
    }
}

/** Returns `true` when `uri` is a `references.bib` file. */
function isReferencesBib(uri: vscode.Uri): boolean {
    return uri.path.endsWith("/references.bib")
}

/**
 * Collects bibliography file URIs for a document, most-preferred first.
 *
 * Search order:
 *  1. `references.bib` in the document's directory
 *  2. Other `.bib` files in the document's directory (alphabetical)
 *  3. `references.bib` at each workspace folder root
 *  4. Remaining `.bib` files anywhere in the workspace (shallower paths first)
 */
export async function findBibliographyUris(documentUri: vscode.Uri): Promise<vscode.Uri[]> {
    const seen = new Set<string>()
    const result: vscode.Uri[] = []
    const docDir = vscode.Uri.joinPath(documentUri, "..")

    addUnique(result, seen, vscode.Uri.joinPath(docDir, "references.bib"))

    try {
        const entries = await vscode.workspace.fs.readDirectory(docDir)
        for (const [name, type] of entries.sort(([a], [b]) => a.localeCompare(b))) {
            if (type === vscode.FileType.File && name.endsWith(".bib") && name !== "references.bib") {
                addUnique(result, seen, vscode.Uri.joinPath(docDir, name))
            }
        }
    } catch {
        // The document directory may be outside the workspace or unreadable.
    }

    for (const folder of vscode.workspace.workspaceFolders ?? []) {
        addUnique(result, seen, vscode.Uri.joinPath(folder.uri, "references.bib"))
    }

    const found = await vscode.workspace.findFiles("**/*.bib", BIB_EXCLUDE)
    const sorted = [...found].sort((a, b) => {
        const depth = (p: string) => p.split("/").length
        const delta = depth(a.path) - depth(b.path)
        return delta !== 0 ? delta : a.path.localeCompare(b.path)
    })
    for (const uri of sorted) {
        addUnique(result, seen, uri)
    }

    return result
}

/**
 * Reads and concatenates all bibliography files found for a document.
 * Returns `null` when none exist or all are empty.
 */
export async function readBibliography(documentUri: vscode.Uri): Promise<string | null> {
    const uris = await findBibliographyUris(documentUri)
    const parts: string[] = []

    for (const uri of uris) {
        try {
            const bytes = await vscode.workspace.fs.readFile(uri)
            const text = Buffer.from(bytes).toString("utf-8").trim()
            if (text) parts.push(text)
        } catch {
            // Unreadable or removed since discovery — skip.
        }
    }

    return parts.length > 0 ? parts.join("\n\n") : null
}

/**
 * Returns the preferred bibliography file for updates (e.g. cached PDF paths).
 *
 * Prefers `references.bib` beside the source file, then any other
 * `references.bib`, then the first discovered `.bib` file.
 */
export async function findPrimaryBibUri(sourceUri: vscode.Uri): Promise<vscode.Uri | null> {
    const uris = await findBibliographyUris(sourceUri)
    if (uris.length === 0) return null

    const docDir = vscode.Uri.joinPath(sourceUri, "..").toString()

    for (const uri of uris) {
        if (isReferencesBib(uri) && vscode.Uri.joinPath(uri, "..").toString() === docDir) {
            return uri
        }
    }

    for (const uri of uris) {
        if (isReferencesBib(uri)) return uri
    }

    return uris[0]
}
