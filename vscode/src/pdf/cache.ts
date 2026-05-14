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
import * as crypto from "crypto"
import { findPrimaryBibUri } from "../bibliography/locate"

/**
 * Returns a portable path to store in `references.bib` for a cached PDF.
 *
 * If the file lives inside the user's cache directory, only the filename is
 * returned so the entry is platform-agnostic (the extension resolves it back
 * to the cache dir at open time).  Otherwise the absolute path is returned
 * as-is so it can still be resolved directly.
 */
export function portableCachePath(absPath: string): string {
    const cacheDir = getCacheDirPath()
    if (absPath.startsWith(cacheDir + path.sep) || absPath.startsWith(cacheDir + "/")) {
        return path.basename(absPath)
    }
    return absPath
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

// ── Filename derivation ───────────────────────────────────────────────────────

/**
 * Returns a safe, deterministic filename for a cached PDF.
 *
 * Format: `<sanitized-title>_<md5-of-url[0..7]>.pdf`
 *
 * The URL hash makes the filename unique per source even when two citations
 * share the same title.  The human-readable prefix makes the cache directory
 * easy to browse.
 */
export function cacheFilename(url: string, title: string): string {
    const hash = crypto.createHash("md5").update(url).digest("hex").slice(0, 8)
    const safe = (title || "pdf")
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, "-")
        .replace(/^-+|-+$/g, "")
        .slice(0, 48)
    return `${safe}_${hash}.pdf`
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

            // Ensure the cache directory exists.
            const dirUri = vscode.Uri.file(getCacheDirPath())
            await vscode.workspace.fs.createDirectory(dirUri)

            // Fetch with redirect following (fetch() follows by default).
            let response: Response
            try {
                response = await fetch(url, { redirect: "follow" })
            } catch (err) {
                throw new Error(`Network error fetching PDF: ${err instanceof Error ? err.message : String(err)}`)
            }

            if (!response.ok) {
                throw new Error(`HTTP ${response.status} ${response.statusText} — ${url}`)
            }

            progress.report({ message: "Saving to cache…" })

            let buffer: ArrayBuffer
            try {
                buffer = await response.arrayBuffer()
            } catch (err) {
                throw new Error(`Failed to read response body: ${err instanceof Error ? err.message : String(err)}`)
            }

            try {
                await vscode.workspace.fs.writeFile(destUri, new Uint8Array(buffer))
            } catch (err) {
                // Best-effort cleanup of any partial file.
                try { await vscode.workspace.fs.delete(destUri) } catch { /* ignore */ }
                throw new Error(`Failed to write cache file: ${err instanceof Error ? err.message : String(err)}`)
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
 *
 * Uses a brace-depth counter to locate the entry boundaries robustly, then
 * splices in `  file = {:<path>:PDF},` immediately before the closing `}`.
 */
async function updateBibFile(
    bibUri: vscode.Uri,
    key: string,
    filePath: string,
): Promise<void> {
    const bytes = await vscode.workspace.fs.readFile(bibUri)
    const text = Buffer.from(bytes).toString("utf-8")

    // Find the entry start.
    const entryRe = new RegExp(`@\\w+\\{\\s*${escapeRegex(key)}\\s*,`, "i")
    const startMatch = entryRe.exec(text)
    if (!startMatch) {
        throw new Error(`Entry @${key} not found in ${bibUri.fsPath}`)
    }

    // Walk forward from the entry start tracking brace depth to find the
    // closing `}` of the entry.
    let depth = 0
    let entryEnd = -1
    for (let i = startMatch.index; i < text.length; i++) {
        if (text[i] === "{") depth++
        else if (text[i] === "}") {
            depth--
            if (depth === 0) {
                entryEnd = i
                break
            }
        }
    }
    if (entryEnd === -1) {
        throw new Error(`Could not find closing brace for @${key}`)
    }

    const fileValue = `:${filePath}:PDF`
    const fieldLine = `  file = {${fileValue}},\n`

    // If a `file` field already exists inside the entry, replace it.
    const entryBody = text.slice(startMatch.index, entryEnd)
    const existingField = /^[ \t]*file\s*=\s*\{[^}]*\}/im.exec(entryBody)
    let updated: string
    if (existingField) {
        const absStart = startMatch.index + existingField.index
        const absEnd = absStart + existingField[0].length
        updated =
            text.slice(0, absStart) +
            `file = {${fileValue}}` +
            text.slice(absEnd)
    } else {
        // Insert before the closing brace of the entry.
        updated = text.slice(0, entryEnd) + fieldLine + text.slice(entryEnd)
    }

    await vscode.workspace.fs.writeFile(bibUri, Buffer.from(updated, "utf-8"))
}

function escapeRegex(s: string): string {
    return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")
}
