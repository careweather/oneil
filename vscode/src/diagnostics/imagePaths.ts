/**
 * Image and PDF path diagnostics for Oneil files.
 *
 * Scans every open `.one` / `.on` file for image and PDF references embedded
 * in notes, and reports a warning diagnostic for each path that cannot be
 * found on disk. Resolution follows the same two-step logic used at render
 * time: workspace root first, then the directory of the file itself.
 *
 * Note syntax supported:
 *   - Single-line:  `~ <text>`
 *   - Multi-line:   `~~~\n<text>\n~~~`
 *
 * Within a note, the standard Markdown image syntax `![alt](path)` is scanned.
 * PDF references follow the same syntax (e.g. `![Figure 1](./datasheets/sensor.pdf)`).
 */

import * as vscode from "vscode"
import * as path from "path"

// ── Regex helpers ─────────────────────────────────────────────────────────────

/** Captures everything inside a multi-line `~~~…~~~` note block. */
const MULTILINE_NOTE_RE = /~~~([\s\S]*?)~~~/g

/** Captures the text after a single-line `~` note prefix. */
const SINGLELINE_NOTE_RE = /^[ \t]*~(?!~~)[ \t](.*)$/gm

/** Captures `![alt](path)` inside a note.  Group 2 is the path/URL. */
const IMAGE_REF_RE = /!\[[^\]]*\]\(([^)]+)\)/g

/** Returns true for URLs that should not be validated on disk. */
function isRemoteOrDataUrl(src: string): boolean {
    return /^(https?:|data:|ftp:)/i.test(src)
}

// ── Path resolution ───────────────────────────────────────────────────────────

/**
 * Resolves a raw src string to a filesystem path to check for existence.
 * Returns `null` when the path should not be validated (remote URL, absolute
 * path that already exists, etc.).
 *
 * Returns an array of candidate absolute paths to try in order.
 */
function resolveCandidates(src: string, documentUri: vscode.Uri): vscode.Uri[] {
    if (isRemoteOrDataUrl(src)) return []

    const normalized = src.replace(/^\.\//, "")
    const candidates: vscode.Uri[] = []

    // Workspace-root-relative
    const folders = vscode.workspace.workspaceFolders
    if (folders && folders.length > 0) {
        candidates.push(vscode.Uri.joinPath(folders[0].uri, normalized))
    }

    // File-directory-relative
    const fileDir = vscode.Uri.joinPath(documentUri, "..")
    candidates.push(vscode.Uri.joinPath(fileDir, normalized))

    // Absolute path (starts with `/`)
    if (path.isAbsolute(src)) {
        candidates.push(vscode.Uri.file(src))
    }

    return candidates
}

/** Returns true when at least one candidate exists on disk. */
async function anyExists(candidates: vscode.Uri[]): Promise<boolean> {
    for (const uri of candidates) {
        try {
            await vscode.workspace.fs.stat(uri)
            return true
        } catch {
            // not found — try next
        }
    }
    return false
}

// ── Note extraction with positions ───────────────────────────────────────────

interface ImageRef {
    /** The raw path/URL string from the `![…](here)` token. */
    src: string
    /** Character offset of the `(` that opens the path in the document. */
    srcStart: number
    /** Character offset of the `)` that closes the path in the document. */
    srcEnd: number
}

/**
 * Finds all image references inside note blocks in `text`, returning each
 * with the absolute character offsets into the original document so that
 * VS Code diagnostics can be placed on the right range.
 */
function extractImageRefs(text: string): ImageRef[] {
    const refs: ImageRef[] = []

    // --- Multi-line notes ---
    for (const noteMatch of text.matchAll(MULTILINE_NOTE_RE)) {
        const noteBody = noteMatch[1]
        const noteBodyStart = noteMatch.index! + 3 // skip leading `~~~`
        for (const imgMatch of noteBody.matchAll(IMAGE_REF_RE)) {
            // imgMatch[0] = full `![alt](src)`, imgMatch[1] = src
            const parenOpen = imgMatch.index! + imgMatch[0].indexOf("(")
            refs.push({
                src: imgMatch[1],
                srcStart: noteBodyStart + parenOpen + 1,
                srcEnd: noteBodyStart + parenOpen + 1 + imgMatch[1].length,
            })
        }
    }

    // --- Single-line notes ---
    // Strip out multi-line note ranges first so `~` inside a `~~~` block
    // is not double-counted.
    const withoutMultiline = text.replace(MULTILINE_NOTE_RE, (m) => " ".repeat(m.length))
    for (const noteMatch of withoutMultiline.matchAll(SINGLELINE_NOTE_RE)) {
        const lineText = noteMatch[1]
        // noteMatch.index is start of the line (including leading whitespace + `~`)
        const prefixLen = noteMatch[0].length - lineText.length
        const lineBodyStart = noteMatch.index! + prefixLen
        for (const imgMatch of lineText.matchAll(IMAGE_REF_RE)) {
            const parenOpen = imgMatch.index! + imgMatch[0].indexOf("(")
            refs.push({
                src: imgMatch[1],
                srcStart: lineBodyStart + parenOpen + 1,
                srcEnd: lineBodyStart + parenOpen + 1 + imgMatch[1].length,
            })
        }
    }

    return refs
}

/** Converts a flat character offset in `text` to a `vscode.Position`. */
function offsetToPosition(text: string, offset: number): vscode.Position {
    const before = text.slice(0, offset)
    const lines = before.split("\n")
    const line = lines.length - 1
    const character = lines[lines.length - 1].length
    return new vscode.Position(line, character)
}

// ── Diagnostic computation ────────────────────────────────────────────────────

/**
 * Computes diagnostics for a single Oneil document.
 * Returns a (possibly empty) array of warnings for missing image/PDF paths.
 */
async function computeDiagnostics(document: vscode.TextDocument): Promise<vscode.Diagnostic[]> {
    const text = document.getText()
    const refs = extractImageRefs(text)
    const diagnostics: vscode.Diagnostic[] = []

    for (const ref of refs) {
        if (isRemoteOrDataUrl(ref.src)) continue

        const candidates = resolveCandidates(ref.src, document.uri)
        if (candidates.length === 0) continue

        const found = await anyExists(candidates)
        if (!found) {
            const start = offsetToPosition(text, ref.srcStart)
            const end = offsetToPosition(text, ref.srcEnd)
            const range = new vscode.Range(start, end)
            const diag = new vscode.Diagnostic(
                range,
                `Image/PDF not found: "${ref.src}". Checked relative to workspace root and file directory.`,
                vscode.DiagnosticSeverity.Warning,
            )
            diag.source = "oneil"
            diag.code = "missing-asset"
            diagnostics.push(diag)
        }
    }

    return diagnostics
}

// ── Registration ──────────────────────────────────────────────────────────────

/**
 * Registers the image/PDF path diagnostic provider and attaches it to the
 * extension context so it is disposed when the extension is deactivated.
 *
 * Diagnostics are (re-)computed on:
 *   - Document open
 *   - Document save
 *   - Document content change (debounced via a short delay)
 */
export function registerImagePathDiagnostics(context: vscode.ExtensionContext): void {
    const collection = vscode.languages.createDiagnosticCollection("oneil-assets")
    context.subscriptions.push(collection)

    const ONEIL_SELECTOR: vscode.DocumentFilter[] = [
        { language: "oneil", scheme: "file" },
    ]

    async function refresh(document: vscode.TextDocument): Promise<void> {
        if (!vscode.languages.match(ONEIL_SELECTOR, document)) return
        const diags = await computeDiagnostics(document)
        collection.set(document.uri, diags)
    }

    // Debounce on-change to avoid hammering disk on every keystroke.
    const pending = new Map<string, ReturnType<typeof setTimeout>>()
    function debounce(document: vscode.TextDocument): void {
        const key = document.uri.toString()
        const existing = pending.get(key)
        if (existing !== undefined) clearTimeout(existing)
        pending.set(key, setTimeout(() => {
            pending.delete(key)
            void refresh(document)
        }, 800))
    }

    context.subscriptions.push(
        vscode.workspace.onDidOpenTextDocument((doc) => void refresh(doc)),
        vscode.workspace.onDidSaveTextDocument((doc) => void refresh(doc)),
        vscode.workspace.onDidChangeTextDocument((e) => debounce(e.document)),
        vscode.workspace.onDidCloseTextDocument((doc) => collection.delete(doc.uri)),
    )

    // Seed diagnostics for any already-open Oneil documents.
    for (const doc of vscode.workspace.textDocuments) {
        void refresh(doc)
    }
}
