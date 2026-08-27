import * as crypto from "crypto"
import * as path from "path"

/**
 * Returns a portable path to store in `references.bib` for a cached PDF.
 *
 * If the file lives inside `cacheDir`, only the filename is returned so the
 * entry is platform-agnostic. Otherwise the absolute path is returned as-is.
 */
export function portableCachePathFrom(absPath: string, cacheDir: string): string {
    if (absPath.startsWith(cacheDir + path.sep) || absPath.startsWith(cacheDir + "/")) {
        return path.basename(absPath)
    }
    return absPath
}

/**
 * Returns a safe, deterministic filename for a cached PDF.
 *
 * Format: `<sanitized-title>_<md5-of-url[0..7]>.pdf`
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

/** True when `bytes` starts with the PDF magic (`%PDF-`). */
export function isPdfBuffer(bytes: Uint8Array): boolean {
    return bytes.length >= 5
        && bytes[0] === 0x25 && bytes[1] === 0x50
        && bytes[2] === 0x44 && bytes[3] === 0x46
}

/**
 * Fetches `url` and returns the body only when it is a PDF.
 *
 * Follows HTTP redirects. Throws on network failure, a non-OK status, or a
 * response whose body does not start with `%PDF-` (typical of DOI landing pages).
 */
export async function fetchPdfBytes(url: string): Promise<Uint8Array> {
    let response: Response
    try {
        response = await fetch(url, { redirect: "follow" })
    } catch (err) {
        throw new Error(
            `Network error fetching PDF: ${err instanceof Error ? err.message : String(err)}`,
            { cause: err },
        )
    }

    if (!response.ok) {
        throw new Error(`HTTP ${response.status} ${response.statusText} — ${url}`)
    }

    let buffer: ArrayBuffer
    try {
        buffer = await response.arrayBuffer()
    } catch (err) {
        throw new Error(
            `Failed to read response body: ${err instanceof Error ? err.message : String(err)}`,
            { cause: err },
        )
    }

    const bytes = new Uint8Array(buffer)
    if (!isPdfBuffer(bytes)) {
        const ct = response.headers.get("content-type") ?? "unknown"
        throw new Error(
            `Not a PDF (content-type ${ct}). A DOI usually resolves to a publisher page — use a direct PDF url, or Open in Browser.`,
        )
    }
    return bytes
}

/**
 * Inserts or replaces the `file` field for `key` in raw BibTeX text.
 *
 * Uses a brace-depth counter to locate the entry, then splices in
 * `  file = {:<path>:PDF},` immediately before the closing `}`.
 */
export function updateBibText(text: string, key: string, filePath: string, sourceLabel = "references.bib"): string {
    const entryRe = new RegExp(`@\\w+\\{\\s*${escapeRegex(key)}\\s*,`, "i")
    const startMatch = entryRe.exec(text)
    if (!startMatch) {
        throw new Error(`Entry @${key} not found in ${sourceLabel}`)
    }

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

    const entryBody = text.slice(startMatch.index, entryEnd)
    const existingField = /^[ \t]*file\s*=\s*\{[^}]*\}/im.exec(entryBody)
    if (existingField) {
        const absStart = startMatch.index + existingField.index
        const absEnd = absStart + existingField[0].length
        return text.slice(0, absStart) + `file = {${fileValue}}` + text.slice(absEnd)
    }
    return text.slice(0, entryEnd) + fieldLine + text.slice(entryEnd)
}

function escapeRegex(s: string): string {
    return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")
}
