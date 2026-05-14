/**
 * NoteDisplay: renders an Oneil note string as rich markdown.
 *
 * Uses `marked` with custom extensions for the Oneil-specific inline syntax,
 * so every extension is parsed by the same lexer pass that handles headings,
 * lists, tables, blockquotes, and fenced code — no fragile pre/post-processing
 * or slot tokens required.
 *
 * Custom extensions (in priority order):
 *   - `mathBlock`    — `\begin{equation}…\end{equation}` and `$$…$$`
 *   - `mathInline`   — `$…$`
 *   - `citation`     — `[@key]`, `[@k1; @k2]`, `[+@k]`, `[-@k]`
 *   - `placeholder`  — `{{param:value}}` / `{{param:equation}}`
 *
 * Standard marked renderers are overridden to:
 *   - Resolve workspace-relative image paths.
 *   - Open external links in a new tab.
 */
import { useMemo, useRef, useEffect } from "react"
import {
    Marked,
    type MarkedExtension,
    type TokenizerAndRendererExtension,
    type Tokens,
} from "marked"
import katex from "katex"
import { useAtomValue, useSetAtom } from "jotai"
import styled from "styled-components"
import type { RenderedParameter, RenderedValue } from "../types/model"
import { fmtNum, mathName, paramExprOnlyToLatex } from "../utils/exprToLatex"
import {
    fileBaseUriAtom,
    focusedPdfAtom,
    parsedBibliographyAtom,
    pdfCacheUriAtom,
    workspaceUriAtom,
} from "../store/atoms"
import type { ParsedCitation } from "../store/atoms/bibliography"
import { getVsCodeApi } from "../vscode"

// ── Styled wrapper ────────────────────────────────────────────────────────────

/**
 * Scoped CSS wrapper for all marked-generated HTML.
 * Headings, lists, blockquotes, code, tables, and inline special elements
 * are all styled here without touching global selectors.
 */
const NoteWrapper = styled.div`
    color: var(--color-fg-subtle);
    font-style: italic;
    font-size: var(--font-size-sm);
    line-height: 1.6;

    /* ── Block structure ──────────────────────────────────── */
    p { margin: 0.25em 0; }
    p + p { margin-top: 0.5em; }

    h1, h2, h3, h4, h5, h6 {
        font-style: normal;
        font-weight: var(--font-weight-bold);
        color: var(--color-fg);
        margin: 0.6em 0 0.2em;
        line-height: 1.3;
    }
    h1 { font-size: 1.15em; }
    h2 { font-size: 1.05em; }
    h3, h4, h5, h6 { font-size: 1em; }

    ul, ol {
        font-style: italic;
        padding-left: 1.4em;
        margin: 0.25em 0;
    }
    li { margin: 0.1em 0; }
    li > ul, li > ol { margin: 0; }

    blockquote {
        border-left: var(--border-blockquote-width) solid var(--color-border);
        margin: 0.4em 0;
        padding: 0 0.6em;
        color: var(--color-fg-muted);
    }

    pre, code {
        font-family: var(--font-mono);
        font-style: normal;
        font-size: 0.9em;
        background: var(--color-bg-subtle);
        border-radius: var(--radius-sm);
    }
    code { padding: 0.1em 0.3em; }
    pre  { padding: 0.5em 0.7em; overflow-x: auto; margin: 0.4em 0; }
    pre code { background: none; padding: 0; }

    hr {
        border: none;
        border-top: 1px solid var(--color-border);
        margin: 0.6em 0;
    }

    /* ── Tables ───────────────────────────────────────────── */
    table {
        border-collapse: collapse;
        font-style: normal;
        font-size: inherit;
        margin: 0.4em 0;
        width: max-content;
        max-width: 100%;
    }
    th, td {
        border: 1px solid var(--color-border);
        padding: 0.2em 0.55em;
        text-align: left;
        vertical-align: top;
    }
    th {
        background: var(--color-bg-subtle);
        font-weight: var(--font-weight-bold);
        color: var(--color-fg-muted);
    }
    tr:nth-child(even) td {
        background: var(--color-bg-subtle-muted);
    }

    /* ── Inline special elements ──────────────────────────── */
    a.note-cite {
        font-style: normal;
        font-size: 0.85em;
        color: var(--vscode-textLink-foreground, var(--color-fg-muted));
        text-decoration: none;
        cursor: pointer;
        white-space: nowrap;
        &:hover {
            text-decoration: underline;
            color: var(--vscode-textLink-activeForeground, var(--color-fg));
        }
    }
    .note-ph-value {
        font-weight: var(--font-weight-bold);
        color: var(--color-fg);
        font-style: normal;
    }
    .note-ph-error {
        color: var(--color-error);
        font-family: var(--font-mono);
        font-size: 0.9em;
        opacity: 0.85;
        font-style: normal;
    }

    /* ── Images ───────────────────────────────────────────── */
    img {
        display: block;
        max-width: 100%;
        height: auto;
        margin: var(--space-sm) 0;
        border-radius: var(--radius-sm);
    }
    /* Inline images (mid-sentence) stay inline */
    p img { display: inline; margin: 0 0.2em; vertical-align: middle; }
`

// ── Utility helpers ───────────────────────────────────────────────────────────

function esc(s: string): string {
    return s
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
}


function formatValue(value: RenderedValue): string {
    switch (value.type) {
        case "boolean":  return String(value.value)
        case "string":   return `"${value.value}"`
        case "number":
            return value.max !== null
                ? `[${fmtNum(value.value)}, ${fmtNum(value.max)}]`
                : fmtNum(value.value)
        case "measured_number": {
            const unit = value.unit === "1" ? "" : ` ${value.unit}`
            return value.max !== null
                ? `[${fmtNum(value.value)}, ${fmtNum(value.max)}]${unit}`
                : `${fmtNum(value.value)}${unit}`
        }
    }
}

type Modifier = "" | "+" | "-" | "!"

/**
 * Resolves the best external URL for a citation entry.
 *
 * Priority:
 * 1. `url` field — a direct link supplied in the BibTeX entry.
 * 2. DOI — converted to `https://doi.org/<doi>`.
 * 3. Google Scholar title search — `https://scholar.google.com/scholar?q=<title>`
 *    (falls back to the citation key if no title is available).
 */
function citationUrl(entry: ParsedCitation | undefined, key: string): string {
    if (!entry) return `https://scholar.google.com/scholar?q=${encodeURIComponent(key)}`
    if (entry.url) return entry.url
    if (entry.doi) return `https://doi.org/${entry.doi}`
    const query = entry.title || key
    return `https://scholar.google.com/scholar?q=${encodeURIComponent(query)}`
}

/**
 * Builds the HTML attributes for a citation link.
 *
 * When the entry has a local PDF cache path (or a DOI/URL that could be a
 * PDF), we include `data-pdf-*` attributes so the delegated click handler
 * in `NoteDisplay` can intercept the click and send an `openPdf` message to
 * the extension rather than following the `href`.
 *
 * @param pageOverride - Inline page locator from the citation syntax
 *   (e.g. `[@key, p.42]`).  Takes precedence over `entry.pdfPage`.
 */
function citationLinkAttrs(
    entry: ParsedCitation | undefined,
    key: string,
    titleOverride?: string,
    pageOverride?: number,
): string {
    const href = citationUrl(entry, key)
    const titleAttr = titleOverride ?? key
    const base = `class="note-cite" title="${esc(titleAttr)}" target="_blank" rel="noreferrer"`

    if (entry?.pdfCachePath || entry?.pdfPage !== undefined || pageOverride !== undefined || entry?.url?.endsWith(".pdf") || entry?.doi) {
        const cachePath = entry?.pdfCachePath ?? ""
        const pdfUrl = entry?.url ?? (entry?.doi ? `https://doi.org/${entry.doi}` : "")
        // Inline locator takes priority over the BibTeX pdfpage default.
        const page = pageOverride ?? entry?.pdfPage ?? 1
        const label = entry?.title || key
        // Use href="#" — the click is fully handled by the delegated JS handler.
        // An external href here would be intercepted by VS Code's link handler
        // and open the system browser *before* e.preventDefault() fires.
        return (
            `class="note-cite" title="${esc(titleAttr)}" href="#" ` +
            `data-pdf="1" ` +
            `data-pdf-key="${esc(key)}" ` +
            `data-pdf-cache="${esc(cachePath)}" ` +
            `data-pdf-url="${esc(pdfUrl)}" ` +
            `data-pdf-page="${page}" ` +
            `data-pdf-title="${esc(label)}"`
        )
    }

    return `${base} href="${esc(href)}"`
}

/**
 * Formats a bracketed citation group: `(Author, year)`, `(year)`, or `Author`.
 *
 * An optional inline `page` locator (from `[@key, p.42]` syntax) is appended
 * in Pandoc style — but **only for single-key citations**.  Page locators are
 * meaningless on multi-key groups because the number appears at the end of the
 * bracket yet would have to be attributed to an arbitrary key.
 *
 * | modifier | output                        |
 * |----------|-------------------------------|
 * | `""`     | `(Author, year)`              |
 * | `"+"`    | `(All Authors, year)`         |
 * | `"-"`    | `(year)`                      |
 * | `"!"`    | `Author` (no parens, no page) |
 */
function formatCiteGroup(
    keys: string[],
    modifier: Modifier,
    bib: Map<string, ParsedCitation>,
    page?: number,
): string {
    if (modifier === "!") {
        const parts = keys.map((key) => {
            const entry = bib.get(key)
            return entry?.authorDisplayFull || key
        })
        return parts.join("; ")
    }
    const parts = keys.map((key) => {
        const entry = bib.get(key)
        if (!entry) return key
        if (modifier === "-") return entry.year || key
        const author = (modifier === "+" ? entry.authorDisplayFull : entry.authorDisplay) || key
        return entry.year ? `${author}, ${entry.year}` : author
    })
    const joined = parts.join("; ")
    // Only append the page locator for single-key citations; for groups the
    // locator position is ambiguous so we silently ignore it.
    const pageSuffix = page !== undefined && keys.length === 1 ? `, p.\u2009${page}` : ""
    return `(${joined}${pageSuffix})`
}

/**
 * Formats a textual (unbracketed) citation: `Author (year)`, `year`, or `Author`.
 *
 * | modifier | output                 |
 * |----------|------------------------|
 * | `""`     | `Author (year)`        |
 * | `"+"`    | `All Authors (year)`   |
 * | `"-"`    | `year`                 |
 * | `"!"`    | `Author`               |
 */
function formatCiteTextual(
    key: string,
    modifier: Modifier,
    bib: Map<string, ParsedCitation>,
): string {
    const entry = bib.get(key)
    if (!entry) return `@${key}`
    if (modifier === "-") return entry.year || key
    const author = (modifier === "+" || modifier === "!") ? entry.authorDisplayFull || key : entry.authorDisplay || key
    if (modifier === "!") return author
    return entry.year ? `${author} (${entry.year})` : author
}

/**
 * Resolves a relative image `src` to a webview-accessible URI.
 *
 * Resolution order:
 * 1. Absolute / external URLs (`https:`, `data:`) — returned as-is.
 * 2. Paths starting with `/` — prefixed with the workspace root URI.
 * 3. Relative paths (`./` or bare) — first tried against the workspace root
 *    (so `./images/foo.png` works from any model inside the workspace), then
 *    against the directory of the file being rendered as a fallback (useful
 *    when models live outside the workspace root or the image is co-located).
 *
 * Both bases are returned separated by `" "` as an `<img srcset>`-style
 * hint is not applicable here; instead we embed both candidates and let
 * the browser try each — we achieve this by returning the workspace attempt
 * first, and the file-dir attempt is a second `<img>` attribute via onerror.
 * However, since we can only set one `src`, we just pick the most specific:
 * if the path starts with `./` or a bare name, the file-dir base is tried
 * first (more specific), then workspace root.
 */
function resolveImageSrc(
    src: string,
    workspaceUri: string | null,
    fileBaseUri: string | null,
): string {
    if (/^(https?:|data:)/i.test(src)) return src
    if (src.startsWith("/")) {
        return workspaceUri ? `${workspaceUri.replace(/\/$/, "")}/${src.replace(/^\//, "")}` : src
    }
    // Relative path — prefer workspace root (consistent with how models are
    // structured), fall back to file directory via onerror on the element.
    const normalized = src.replace(/^\.\//, "")
    if (workspaceUri) return `${workspaceUri.replace(/\/$/, "")}/${normalized}`
    if (fileBaseUri) return `${fileBaseUri.replace(/\/$/, "")}/${normalized}`
    return src
}

function renderKatex(src: string, display: boolean): string {
    try {
        return katex.renderToString(src, { output: "mathml", displayMode: display, throwOnError: false })
    } catch {
        return `<code>${esc(src)}</code>`
    }
}

/**
 * Substitutes `{{param:equation}}` and `{{param:value}}` placeholders inside
 * a raw LaTeX string (i.e. already inside a `$…$` or `$$…$$` block).
 *
 * Unlike the outside-math `placeholderExt`, this emits raw LaTeX fragments
 * rather than KaTeX HTML so that KaTeX can process the whole expression as a
 * single unit:
 *
 *   - `equation` → `\mathrm{name} = <expr latex>`
 *   - `value`    → formatted number with LaTeX unit (e.g. `7906\,\mathrm{m/s^2}`)
 *
 * Unrecognised parameter names are left as-is so KaTeX can surface them as
 * undefined-command errors rather than silently swallowing them.
 */
function substitutePlaceholders(src: string, parameters: RenderedParameter[] | undefined): string {
    if (!parameters || !src.includes("{{")) return src
    return src.replace(/\{\{(\w+):(equation|value)\}\}/g, (match, paramName: string, mode: string) => {
        const param = parameters.find((p) => p.name === paramName)
        if (!param) return match
        if (mode === "value") return formatValueLatex(param.value)
        // equation mode — full "name = expr" as LaTeX
        if (param.expression) {
            try {
                return `${mathName(param.name)} = ${paramExprOnlyToLatex(param.expression)}`
            } catch {
                return match
            }
        }
        return match
    })
}

/**
 * Formats a `RenderedValue` as a LaTeX-safe string for use inside a math
 * block.  Units are wrapped in `\mathrm{}` so they render upright.
 */
function formatValueLatex(value: RenderedValue): string {
    const num = (v: number) => fmtNum(v)
    switch (value.type) {
        case "boolean": return String(value.value)
        case "string":  return `\\text{${value.value}}`
        case "number":
            return value.max !== null
                ? `[${num(value.value)},\\,${num(value.max)}]`
                : num(value.value)
        case "measured_number": {
            const unit = value.unit === "1" ? "" : `\\,\\mathrm{${escLatexUnit(value.unit)}}`
            return value.max !== null
                ? `[${num(value.value)},\\,${num(value.max)}]${unit}`
                : `${num(value.value)}${unit}`
        }
    }
}

/**
 * Escapes a raw unit string for use inside `\mathrm{…}`.
 * Converts `^` exponents and `/` fraction separators to LaTeX equivalents
 * so that e.g. `m/s^2` becomes `m/s^{2}` (valid inside \mathrm).
 */
function escLatexUnit(unit: string): string {
    return unit.replace(/\^(-?\d+)/g, "^{$1}")
}

// ── Extension factories ───────────────────────────────────────────────────────

/**
 * Block math: `$$…$$` and `\begin{equation}…\end{equation}`.
 *
 * `{{param:equation}}` / `{{param:value}}` placeholders inside the block are
 * substituted before KaTeX renders the source so they integrate cleanly with
 * surrounding LaTeX rather than being injected as opaque HTML spans.
 */
function mathBlockExt(parameters: RenderedParameter[] | undefined): TokenizerAndRendererExtension {
    return {
        name: "mathBlock",
        level: "block",
        start(src) {
            const a = src.indexOf("$$")
            const b = src.indexOf("\\begin{equation}")
            if (a === -1) return b
            if (b === -1) return a
            return Math.min(a, b)
        },
        tokenizer(src) {
            // Content: any char that isn't $ or \, OR a backslash-escaped char
            // (e.g. \$ inside display math).
            const dbl = src.match(/^\$\$((?:[^$\\]|\\.)+)\$\$\s*(?:\n|$)/)
            if (dbl) return { type: "mathBlock", raw: dbl[0], src: dbl[1].trim() }
            const env = src.match(/^\\begin\{equation\}([\s\S]*?)\\end\{equation\}\s*(?:\n|$)/)
            if (env) {
                const body = env[1].replace(/\\label\{[^}]*\}/g, "").trim()
                return { type: "mathBlock", raw: env[0], src: body }
            }
            return undefined
        },
        renderer(token) {
            const src = substitutePlaceholders((token as unknown as { src: string }).src, parameters)
            return renderKatex(src, true)
        },
    }
}

/**
 * Inline math: `$…$` (does not match `$$`).
 *
 * Same placeholder substitution as `mathBlockExt` — applied before KaTeX.
 */
function mathInlineExt(parameters: RenderedParameter[] | undefined): TokenizerAndRendererExtension {
    return {
        name: "mathInline",
        level: "inline",
        start(src) {
            // Skip `$$` (block math) and `\$` (escaped dollar, not a delimiter).
            let i = src.indexOf("$")
            while (i !== -1) {
                if (src[i + 1] === "$") { i = src.indexOf("$", i + 2); continue }
                if (i > 0 && src[i - 1] === "\\") { i = src.indexOf("$", i + 1); continue }
                break
            }
            return i
        },
        tokenizer(src) {
            // Must not match `$$` prefix.
            if (src.startsWith("$$")) return undefined
            // Content: non-$, non-newline chars, or backslash-escaped chars (e.g. \$).
            const m = src.match(/^\$((?:[^$\n\\]|\\.)+)\$(?!\$)/)
            if (m) return { type: "mathInline", raw: m[0], src: m[1] }
            return undefined
        },
        renderer(token) {
            const src = substitutePlaceholders((token as unknown as { src: string }).src, parameters)
            return renderKatex(src, false)
        },
    }
}

/** Matches a trailing page locator inside a bracketed citation. */
const PAGE_LOCATOR_RE = /,\s*(?:pp?\.\s*)?(\d+)(?:-\d+)?\s*$/

/**
 * Bracketed citations with optional inline page locator.
 *
 * Supported syntax:
 *   `[@key]`           — standard citation
 *   `[@key, p.42]`     — open PDF at page 42
 *   `[@key, pp.42]`    — `pp.` variant (plural pages prefix)
 *   `[@key, pp.42-45]` — page range; viewer opens at 42
 *   `[@key, 42]`       — bare integer treated as page number
 *   `[@k1; @k2, p.42]` — multi-key group; page applies to the whole group
 *   `[+@key, p.42]`    — modifier + page
 *
 * Registered before marked's link tokenizer so `[@…]` is never mistaken for
 * a link missing its URL.
 */
function citationExt(bib: Map<string, ParsedCitation>): TokenizerAndRendererExtension {
    return {
        name: "citation",
        level: "inline",
        start(src) { return src.search(/\[[+\-!]?@/) },
        tokenizer(src) {
            const m = src.match(/^\[([+\-!]?@[^\]]+)\]/)
            if (!m) return undefined

            // Extract modifier from the very start of the inner string.
            const modMatch = m[1].match(/^([+\-!])@/)
            const modifier = (modMatch?.[1] ?? "") as Modifier

            // Separate the page locator (if any) from the keys string.
            const pageMatch = PAGE_LOCATOR_RE.exec(m[1])
            const page: number | undefined = pageMatch
                ? parseInt(pageMatch[1], 10)
                : undefined
            const keysStr = pageMatch ? m[1].slice(0, pageMatch.index) : m[1]

            const keys = keysStr
                .split(";")
                .map((s) => s.trim().replace(/^[+\-!]?@/, "").trim())
                .filter(Boolean)

            return { type: "citation", raw: m[0], keys, modifier, page }
        },
        renderer(token) {
            const t = token as unknown as { keys: string[]; modifier: Modifier; page?: number }
            const isSingle = t.keys.length === 1
            // Page locator only applies to single-key citations; ignore it for
            // groups so we don't misroute a page number to the wrong key's PDF.
            const effectivePage = isSingle ? t.page : undefined
            const text = formatCiteGroup(t.keys, t.modifier, bib, effectivePage)
            const firstEntry = bib.get(t.keys[0])
            const attrs = citationLinkAttrs(firstEntry, t.keys[0], t.keys.join("; "), effectivePage)
            return `<a ${attrs}>${esc(text)}</a>`
        },
    }
}

/**
 * Textual (unbracketed) citations: `@key`, `+@key`, `-@key`, `!@key`.
 * Renders as narrative prose: "Author (year)", "year", or "Author".
 * Must be registered after `citationExt` so bracketed `[@key]` is consumed first.
 */
function citationTextualExt(bib: Map<string, ParsedCitation>): TokenizerAndRendererExtension {
    return {
        name: "citationTextual",
        level: "inline",
        start(src) { return src.search(/[+\-!]?@[A-Za-z0-9]/) },
        tokenizer(src) {
            const m = src.match(/^([+\-!]?)@([A-Za-z0-9_:.-]+)/)
            if (!m) return undefined
            const modifier = m[1] as Modifier
            const key = m[2]
            return { type: "citationTextual", raw: m[0], key, modifier }
        },
        renderer(token) {
            const t = token as unknown as { key: string; modifier: Modifier }
            const text = formatCiteTextual(t.key, t.modifier, bib)
            const attrs = citationLinkAttrs(bib.get(t.key), t.key)
            return `<a ${attrs}>${esc(text)}</a>`
        },
    }
}

/** Parameter placeholder: `{{param:value}}` / `{{param:equation}}` */
function placeholderExt(parameters: RenderedParameter[] | undefined): TokenizerAndRendererExtension {
    return {
        name: "placeholder",
        level: "inline",
        start(src) { return src.indexOf("{{") },
        tokenizer(src) {
            const m = src.match(/^\{\{(\w+):(equation|value)\}\}/)
            if (m) return { type: "placeholder", raw: m[0], paramName: m[1], mode: m[2] }
            return undefined
        },
        renderer(token) {
            const t = token as unknown as { paramName: string; mode: string }
            const param = parameters?.find((p) => p.name === t.paramName)
            if (!param) {
                return `<span class="note-ph-error">{{${esc(t.paramName)}:${esc(t.mode)}}}</span>`
            }
            if (t.mode === "value") {
                return `<span class="note-ph-value">${esc(formatValue(param.value))}</span>`
            }
            // equation mode
            if (param.expression) {
                try {
                    const latex = `${mathName(param.name)} = ${paramExprOnlyToLatex(param.expression)}`
                    return renderKatex(latex, false)
                } catch {
                    return `<code>${esc(t.paramName)}</code>`
                }
            }
            return `<code>${esc(t.paramName)}</code>`
        },
    }
}

// ── Renderer overrides ────────────────────────────────────────────────────────

function rendererOverrides(
    workspaceUri: string | null,
    fileBaseUri: string | null,
): MarkedExtension["renderer"] {
    return {
        image(token: Tokens.Image): string {
            const src = resolveImageSrc(token.href, workspaceUri, fileBaseUri)
            const title = token.title ? ` title="${esc(token.title)}"` : ""
            // When workspace-root resolution is used, add an onerror fallback
            // that retries with the file-local base URI.
            let onerror = ""
            if (
                fileBaseUri &&
                workspaceUri &&
                src.startsWith(workspaceUri) &&
                !/^(https?:|data:)/i.test(token.href) &&
                !token.href.startsWith("/")
            ) {
                const normalized = token.href.replace(/^\.\//, "")
                const fallback = `${fileBaseUri.replace(/\/$/, "")}/${normalized}`
                onerror = ` onerror="if(this.src!=='${esc(fallback)}')this.src='${esc(fallback)}'"`
            }
            return `<img src="${esc(src)}" alt="${esc(token.text)}"${title}${onerror}>`
        },
        link(token: Tokens.Link): string {
            // Render inner tokens via the default pipeline; fall back to text.
            const inner = this.parser?.parseInline(token.tokens ?? []) ?? esc(token.text)
            const href = token.href ?? ""
            if (/^https?:\/\//i.test(href)) {
                return `<a href="${esc(href)}" target="_blank" rel="noreferrer">${inner}</a>`
            }
            return `<a href="${esc(href)}">${inner}</a>`
        },
    }
}

// ── Core rendering function ───────────────────────────────────────────────────

/**
 * Renders an Oneil note string to an HTML string using a `marked` instance
 * configured with all custom extensions and renderer overrides.
 *
 * Returns a stable reference when all inputs are referentially equal (they never mutate internally after being built), so it
 * is safe to wrap in `useMemo`.
 */
function renderNoteToHtml(
    text: string,
    parameters: RenderedParameter[] | undefined,
    bib: Map<string, ParsedCitation>,
    workspaceUri: string | null,
    fileBaseUri: string | null,
): string {
    try {
        const instance = new Marked({
            gfm: true, // github-flavored markdown
            breaks: false,
            extensions: [
                mathBlockExt(parameters),
                mathInlineExt(parameters),
                citationExt(bib),
                citationTextualExt(bib),
                placeholderExt(parameters),
            ],
            renderer: rendererOverrides(workspaceUri, fileBaseUri),
        })

        return instance.parse(text) as string
    } catch (err) {
        console.error("[oneil-webview] note render error:", err)
        return esc(text)
    }
}

// ── NoteDisplay ───────────────────────────────────────────────────────────────

/**
 * Renders an Oneil note string as rich markdown with math, citations, and
 * parameter placeholders.
 *
 * @param text       - Raw note string from the model.
 * @param parameters - Node parameters for `{{param:value}}` / `{{param:equation}}`.
 *
 * @example
 * ```tsx
 * <NoteDisplay text={node.note} parameters={node.parameters} />
 * ```
 */
export function NoteDisplay({ text, parameters }: { text: string; parameters?: RenderedParameter[] }) {
    const bib = useAtomValue(parsedBibliographyAtom)
    const workspaceUri = useAtomValue(workspaceUriAtom)
    const fileBaseUri = useAtomValue(fileBaseUriAtom)
    const pdfCacheUri = useAtomValue(pdfCacheUriAtom)
    const setFocusedPdf = useSetAtom(focusedPdfAtom)
    const wrapperRef = useRef<HTMLDivElement>(null)

    const html = useMemo(
        () => renderNoteToHtml(text, parameters, bib, workspaceUri, fileBaseUri),
        [text, parameters, bib, workspaceUri, fileBaseUri],
    )

    // Delegated click handler: intercept PDF citation links.
    //
    // Resolution priority:
    //  1. Bare filename in references.bib → construct `pdfCacheUri/<filename>`
    //     and open inline with react-pdf (sets focusedPdfAtom).
    //  2. Workspace-relative path (starts with ./) → construct from workspaceUri
    //     and open inline.
    //  3. Anything else (absolute path, ~, no local path) → fall back to the
    //     extension so it can resolve / download / open externally.
    useEffect(() => {
        const el = wrapperRef.current
        if (!el) return
        const handler = (e: MouseEvent) => {
            const anchor = (e.target as HTMLElement).closest("a[data-pdf]")
            if (!anchor) return
            e.preventDefault()

            const cachePath = anchor.getAttribute("data-pdf-cache") || null
            const pdfUrl = anchor.getAttribute("data-pdf-url") || null
            const pageAttr = anchor.getAttribute("data-pdf-page")
            const page = pageAttr ? parseInt(pageAttr, 10) : 1
            const title = anchor.getAttribute("data-pdf-title") || anchor.getAttribute("title") || ""
            const citationKey = anchor.getAttribute("data-pdf-key") || ""

            // Try to resolve to a webview-accessible URL for inline rendering.
            let webviewUrl: string | null = null
            if (cachePath) {
                const isBare = !cachePath.startsWith("/") && !cachePath.startsWith("~") &&
                    !cachePath.startsWith("./") && !cachePath.startsWith("../")
                if (isBare && pdfCacheUri) {
                    // Bare filename stored in references.bib — look up in cache dir.
                    webviewUrl = `${pdfCacheUri.replace(/\/$/, "")}/${cachePath}`
                } else if ((cachePath.startsWith("./") || cachePath.startsWith("../")) && workspaceUri) {
                    // Workspace-relative path.
                    webviewUrl = `${workspaceUri.replace(/\/$/, "")}/${cachePath.replace(/^\.\//, "")}`
                }
            }

            if (webviewUrl) {
                setFocusedPdf({ url: webviewUrl, page, title })
            } else {
                // Fall back to extension: it will resolve absolute / ~ paths,
                // check the cache directory, offer downloads, and open externally.
                getVsCodeApi().postMessage({ type: "openPdf", pdfUrl, cachePath, page, title, citationKey })
            }
        }
        el.addEventListener("click", handler)
        return () => el.removeEventListener("click", handler)
    // Re-attach when cache/workspace URIs arrive (they start as null).
    }, [pdfCacheUri, workspaceUri, setFocusedPdf])

    return <NoteWrapper ref={wrapperRef} dangerouslySetInnerHTML={{ __html: html }} />
}
