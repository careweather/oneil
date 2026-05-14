/**
 * PDF open helper for Oneil citation links.
 *
 * Resolves the best available copy of a PDF and opens it in the system
 * browser (or the OS default PDF viewer).  Local files are opened as
 * `file://` URIs with a `#page=N` fragment; remote URLs get the same
 * fragment appended before being handed to `vscode.env.openExternal`.
 *
 * Resolution order for a local file:
 *   1. BibTeX `file` field path — absolute, relative (`./`), or cache-relative
 *      (bare filename looked up in the user's PDF cache directory).
 *   2. User-level PDF cache (`~/.local/oneil/resources/`), keyed by URL hash.
 *   3. Download from `pdfUrl` (subject to `offlineOnly` / `autoDownload`
 *      settings), then open.
 *   4. Open `pdfUrl` directly in the browser as a last resort.
 */

import * as vscode from "vscode"
import * as path from "path"
import * as os from "os"
import {
    findCached,
    downloadAndCache,
    offerDownload,
    offerBibUpdate,
    isOfflineMode,
    isAutoDownload,
    getCacheDirPath,
} from "../pdf/cache"

/**
 * Opens a citation PDF in the system browser or OS default PDF viewer.
 *
 * @param params.pdfUrl      - Remote URL of the PDF.
 * @param params.cachePath   - BibTeX `file` field path (absolute, relative, or
 *                             cache-relative bare filename).
 * @param params.page        - 1-based page to jump to.
 * @param params.title       - Human-readable label for notifications.
 * @param params.citationKey - BibTeX key; used when offering to update
 *                             `references.bib` after a download.
 * @param params.sourceUri   - The `.one` file being rendered; used to resolve
 *                             relative cache paths.
 */
export async function openPdfExternal(
    params: {
        pdfUrl: string | null
        cachePath: string | null
        page: number | null
        title: string
        citationKey: string
        sourceUri: vscode.Uri
    },
): Promise<void> {
    const page = params.page ?? 1

    // 1. Try the BibTeX-specified cache path.
    let localUri = params.cachePath
        ? await resolveLocalPdf(params.cachePath, params.sourceUri)
        : null

    // 2. Try the user-level PDF cache (keyed by URL hash).
    if (!localUri && params.pdfUrl) {
        localUri = await findCached(params.pdfUrl, params.title)
    }

    // 3. No local copy — decide whether to download.
    if (!localUri && params.pdfUrl) {
        if (isOfflineMode()) {
            void vscode.window.showWarningMessage(
                `Oneil (offline): "${params.title || params.pdfUrl}" is not cached. ` +
                `Disable offline mode to download.`,
            )
            return
        }

        if (isAutoDownload()) {
            try {
                localUri = await downloadAndCache(params.pdfUrl, params.title)
                if (params.citationKey) {
                    void offerBibUpdate(params.citationKey, localUri, params.sourceUri)
                }
            } catch {
                // Fall through to direct URL.
            }
        } else {
            const result = await offerDownload(params.pdfUrl, params.title)
            if (result === "cached") {
                localUri = await findCached(params.pdfUrl, params.title)
                if (localUri && params.citationKey) {
                    void offerBibUpdate(params.citationKey, localUri, params.sourceUri)
                }
            } else if (result === "browser") {
                localUri = null // open URL directly below
            } else {
                return // cancelled
            }
        }
    }

    if (!localUri && !params.pdfUrl) {
        void vscode.window.showWarningMessage("Oneil: no PDF path or URL provided for this citation.")
        return
    }

    // Open in system browser / OS default PDF viewer.
    const target = localUri
        ? vscode.Uri.parse(`${localUri.toString()}#page=${page}`)
        : vscode.Uri.parse(`${params.pdfUrl!}#page=${page}`)

    void vscode.env.openExternal(target)
}

// ── Path resolution ───────────────────────────────────────────────────────────

/**
 * Resolves a raw BibTeX `file` field path to a `vscode.Uri`.
 *
 * Resolution order:
 *   1. Absolute path (after `~` expansion).
 *   2. Relative path (`./…` / `../…`) — workspace root then file directory.
 *   3. Bare / cache-relative — cache directory first, then workspace root,
 *      then file directory.
 */
async function resolveLocalPdf(
    cachePath: string,
    sourceUri: vscode.Uri,
): Promise<vscode.Uri | null> {
    const candidates: vscode.Uri[] = []

    const expanded = cachePath.startsWith("~/") || cachePath === "~"
        ? os.homedir() + cachePath.slice(1)
        : cachePath

    const isAbsolute = path.isAbsolute(expanded)
    const isRelative = expanded.startsWith("./") || expanded.startsWith("../")

    if (isAbsolute) {
        candidates.push(vscode.Uri.file(expanded))
    } else if (isRelative) {
        const normalized = expanded.replace(/^\.\//, "")
        for (const folder of vscode.workspace.workspaceFolders ?? []) {
            candidates.push(vscode.Uri.joinPath(folder.uri, normalized))
        }
        candidates.push(vscode.Uri.joinPath(sourceUri, "..", normalized))
    } else {
        // Cache-relative bare filename.
        candidates.push(vscode.Uri.file(path.join(getCacheDirPath(), expanded)))
        for (const folder of vscode.workspace.workspaceFolders ?? []) {
            candidates.push(vscode.Uri.joinPath(folder.uri, expanded))
        }
        candidates.push(vscode.Uri.joinPath(sourceUri, "..", expanded))
    }

    for (const uri of candidates) {
        try {
            await vscode.workspace.fs.stat(uri)
            return uri
        } catch { /* not found */ }
    }

    return null
}
