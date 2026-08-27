/**
 * PDF cache management for Oneil citation PDFs.
 *
 * Provides:
 *  - A standard per-user cache directory (`~/.local/oneil/resources/` by default,
 *    overridable via the `oneil.pdf.cacheDir` setting).
 *  - Deterministic filenames derived from the PDF URL so repeated opens never
 *    re-download.
 *  - A download helper with VS Code progress notification and redirect following.
 *  - An offer flow that asks the user whether to download before caching.
 *  - A helper to update `references.bib` with the new `file` field so the
 *    cached path is remembered across workspaces.
 *  - Getters for the `offlineOnly` and `autoDownload` settings.
 */

import * as vscode from "vscode"
import * as os from "os"
import * as path from "path"
import { findPrimaryBibUri } from "../bibliography/locate"
import {
    cacheFilename,
    fetchPdfBytes,
    portableCachePathFrom,
    updateBibText,
} from "./cacheLogic"

export { cacheFilename } from "./cacheLogic"

/**
 * Returns a portable path to store in `references.bib` for a cached PDF.
 *
 * If the file lives inside the user's cache directory, only the filename is
 * returned so the entry is platform-agnostic (the extension resolves it back
 * to the cache dir at open time).  Otherwise the absolute path is returned
 * as-is so it can still be resolved directly.
 */
export function portableCachePath(absPath: string): string {
    return portableCachePathFrom(absPath, getCacheDirPath())
}

// ── Settings helpers ──────────────────────────────────────────────────────────

/** Returns the resolved path to the user-level PDF cache directory. */
export function getCacheDirPath(): string {
    const raw = vscode.workspace.getConfiguration("oneil.pdf").get<string>("cacheDir", "")
    if (raw) return raw.replace(/^~/, os.homedir())
    return path.join(os.homedir(), ".local", "oneil", "resources")
}

/** True when the extension should only use locally cached PDFs. */
export function isOfflineMode(): boolean {
    return vscode.workspace.getConfiguration("oneil.pdf").get<boolean>("offlineOnly", false)
}

/** True when PDFs should be downloaded automatically without prompting. */
export function isAutoDownload(): boolean {
    return vscode.workspace.getConfiguration("oneil.pdf").get<boolean>("autoDownload", false)
}

/** Flips the `oneil.pdf.offlineOnly` setting globally. */
export async function toggleOfflineMode(): Promise<void> {
    const config = vscode.workspace.getConfiguration("oneil.pdf")
    await config.update("offlineOnly", !isOfflineMode(), vscode.ConfigurationTarget.Global)
}

/** Returns the `vscode.Uri` of the expected cache file for a given URL. */
export function cacheUri(url: string, title: string): vscode.Uri {
    return vscode.Uri.file(path.join(getCacheDirPath(), cacheFilename(url, title)))
}

// ── Cache lookup ──────────────────────────────────────────────────────────────

/**
 * Returns the cached `vscode.Uri` for the given URL if the file exists on
 * disk, or `null` when it has not been downloaded yet.
 */
export async function findCached(url: string, title: string): Promise<vscode.Uri | null> {
    const uri = cacheUri(url, title)
    try {
        await vscode.workspace.fs.stat(uri)
        return uri
    } catch {
        return null
    }
}

// ── Downloading ───────────────────────────────────────────────────────────────

/**
 * Downloads a PDF from `url` into the cache directory, showing a VS Code
 * progress notification.  Follows up to 5 HTTP redirects.
 *
 * Returns the `vscode.Uri` of the saved file.
 * Throws when the download fails (the partially-written file is removed).
 */
export async function downloadAndCache(url: string, title: string): Promise<vscode.Uri> {
    const destUri = cacheUri(url, title)

    return vscode.window.withProgress(
        {
            location: vscode.ProgressLocation.Notification,
            title: `Downloading PDF: ${title || url}`,
            cancellable: false,
        },
        async (progress) => {
            progress.report({ message: "Connecting…" })

            const dirUri = vscode.Uri.file(getCacheDirPath())
            await vscode.workspace.fs.createDirectory(dirUri)

            const bytes = await fetchPdfBytes(url)

            progress.report({ message: "Saving to cache…" })

            try {
                await vscode.workspace.fs.writeFile(destUri, bytes)
            } catch (err) {
                try { await vscode.workspace.fs.delete(destUri) } catch { /* ignore */ }
                throw new Error(
                    `Failed to write cache file: ${err instanceof Error ? err.message : String(err)}`,
                    { cause: err },
                )
            }

            return destUri
        },
    )
}

// ── Offer flow ────────────────────────────────────────────────────────────────

/**
 * Asks the user whether to download and cache a PDF.
 *
 * Returns:
 *  - `"cached"` — the file was downloaded successfully.
 *  - `"browser"` — the user chose to open in the system browser instead.
 *  - `"cancelled"` — the user dismissed the prompt.
 */
export async function offerDownload(
    url: string,
    title: string,
): Promise<"cached" | "browser" | "cancelled"> {
    const label = title || url
    const choice = await vscode.window.showInformationMessage(
        `"${label}" is not in the local PDF cache.`,
        "Download & Cache",
        "Open in Browser",
    )

    if (choice === "Download & Cache") {
        try {
            await downloadAndCache(url, title)
            return "cached"
        } catch (err) {
            void vscode.window.showErrorMessage(
                `Failed to download PDF: ${err instanceof Error ? err.message : String(err)}`,
            )
            return "cancelled"
        }
    }

    if (choice === "Open in Browser") return "browser"
    return "cancelled"
}

// ── BibTeX update ─────────────────────────────────────────────────────────────

/**
 * After caching a PDF, offers to write the local path back into `references.bib`
 * so the `file` field is populated for future opens.
 *
 * Searches for the bib file in the workspace (same strategy as `readWorkspaceBib`),
 * finds the entry by citation key, and inserts / replaces the `file` field.
 */
export async function offerBibUpdate(
    citationKey: string,
    cachedUri: vscode.Uri,
    sourceUri: vscode.Uri,
): Promise<void> {
    const choice = await vscode.window.showInformationMessage(
        `PDF cached. Update references.bib with the local path?`,
        "Update references.bib",
        "No thanks",
    )
    if (choice !== "Update references.bib") return

    const bibUri = await findPrimaryBibUri(sourceUri)
    if (!bibUri) {
        void vscode.window.showWarningMessage("Oneil: could not find references.bib to update.")
        return
    }

    try {
        await updateBibFile(bibUri, citationKey, portableCachePath(cachedUri.fsPath))
        void vscode.window.showInformationMessage(`references.bib updated for @${citationKey}.`)
    } catch (err) {
        void vscode.window.showErrorMessage(
            `Failed to update references.bib: ${err instanceof Error ? err.message : String(err)}`,
        )
    }
}

/**
 * Inserts or replaces the `file` field for a given citation key in the bib file.
 */
async function updateBibFile(
    bibUri: vscode.Uri,
    key: string,
    filePath: string,
): Promise<void> {
    const bytes = await vscode.workspace.fs.readFile(bibUri)
    const text = Buffer.from(bytes).toString("utf-8")
    const updated = updateBibText(text, key, filePath, bibUri.fsPath)
    await vscode.workspace.fs.writeFile(bibUri, Buffer.from(updated, "utf-8"))
}
