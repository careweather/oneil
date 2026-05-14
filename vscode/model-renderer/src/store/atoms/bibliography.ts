/**
 * Bibliography atoms: parses raw BibTeX into structured citation data, then
 * derives citation-usage groups for the currently focused model.
 *
 * Separated from `data.ts` so the `citation-js` dependency is isolated here
 * and not bundled into any code path that doesn't need it.  Parsing is deferred
 * until a non-null BibTeX payload arrives so workspaces without a
 * `references.bib` file never load citation-js.
 */
import { atom } from "jotai"
import { instanceTreeAtom } from "./app"
import { focusedNodeAtom } from "./navigation"
import { paramKey } from "../../utils/instancePath"

/** Shared empty bibliography — reused so absent-bib workspaces stay stable. */
export const EMPTY_BIBLIOGRAPHY: ReadonlyMap<string, ParsedCitation> = new Map()

// ── Parsed citation ───────────────────────────────────────────────────────────

/** A single parsed bibliography entry extracted from BibTeX. */
export interface ParsedCitation {
    /** The BibTeX citation key (e.g. `"Knuth:TeX"`). */
    key: string
    /** Short author display: `"Smith"`, `"Smith & Jones"`, or `"Smith et al."` */
    authorDisplay: string
    /** Full author list: `"Smith"`, `"Smith & Jones"`, or `"Smith, Jones & Brown"`. */
    authorDisplayFull: string
    /** Publication year as a string, e.g. `"1984"`. */
    year: string
    /** Full title of the work. */
    title: string
    /** Direct URL to the work, if present in the BibTeX `url` field. */
    url: string | undefined
    /** DOI identifier (without `https://doi.org/` prefix), if present. */
    doi: string | undefined
    /**
     * Local path to a cached PDF of the work, parsed from the BibTeX `file`
     * field (standard Zotero / JabRef format: `{Description:path.pdf:PDF}`
     * or just `{path.pdf}`).  Relative paths are resolved against the
     * workspace root or the directory of the file being rendered at display
     * time.
     */
    pdfCachePath: string | undefined
    /**
     * Suggested page number within the PDF where the relevant content
     * begins, taken from the custom `pdfpage` BibTeX field.
     */
    pdfPage: number | undefined
}

/** CSL-JSON author record as returned by citation-js. */
interface CslAuthor {
    family?: string
    given?: string
    literal?: string
}

/** CSL-JSON date as returned by citation-js. */
interface CslDate {
    "date-parts"?: [[number, ...number[]]]
}

/** Formats an author list into a short display string (truncates to "et al." after 2). */
function formatAuthors(authors: CslAuthor[] | undefined): string {
    if (!authors || authors.length === 0) return ""
    const names = authors.map((a) => a.family ?? a.literal ?? a.given ?? "")
    if (names.length === 1) return names[0]
    if (names.length === 2) return `${names[0]} & ${names[1]}`
    return `${names[0]} et al.`
}

/** Formats an author list listing every name (no "et al." truncation). */
function formatAuthorsAll(authors: CslAuthor[] | undefined): string {
    if (!authors || authors.length === 0) return ""
    const names = authors.map((a) => a.family ?? a.literal ?? a.given ?? "")
    if (names.length === 1) return names[0]
    return `${names.slice(0, -1).join(", ")} & ${names[names.length - 1]}`
}

/**
 * Parses the BibTeX `file` field value into a filesystem path.
 *
 * Handles three common formats used by Zotero, JabRef, and similar tools:
 *   - `Description:path/to/file.pdf:PDF`  → `path/to/file.pdf`
 *   - `:path/to/file.pdf:PDF`             → `path/to/file.pdf`
 *   - `path/to/file.pdf`                  → `path/to/file.pdf`
 */
function parseFileField(raw: string): string | undefined {
    const trimmed = raw.trim()
    if (!trimmed) return undefined
    // JabRef / Zotero colon-delimited: `description:path:type`
    const parts = trimmed.split(":")
    if (parts.length >= 2) {
        // Handle Windows absolute paths like `C:\Users\…` (single-letter drive).
        const isWindowsPath = parts[0].length === 1 && /[A-Za-z]/.test(parts[0])
        if (!isWindowsPath) {
            const pathPart = parts[1].trim()
            if (pathPart) return pathPart
        }
    }
    return trimmed
}

/**
 * Extracts raw field values directly from BibTeX source text.
 *
 * citation-js silently drops non-CSL fields such as `file` and `pdfpage`
 * during its BibTeX → CSL-JSON conversion.  This function scans the raw text
 * so those fields are always available.
 *
 * Returns a map from citation key → `{ fieldName: rawValue }`.  Only entries
 * that contain at least one of the requested fields appear in the map.
 * Values are the inner content of the outermost `{…}` wrapper (braces
 * stripped); `"value"` quoted fields are supported too.
 */
function extractRawBibFields(
    raw: string,
    fieldNames: string[],
): Map<string, Record<string, string>> {
    const result = new Map<string, Record<string, string>>()

    // Locate the start of every @TYPE{KEY, ... body ...} entry.
    const entryRe = /@\w+\s*\{\s*([^,\s{]+)\s*,/g
    const entries: Array<{ key: string; bodyStart: number; entryStart: number }> = []
    let m: RegExpExecArray | null
    while ((m = entryRe.exec(raw)) !== null) {
        entries.push({ key: m[1], bodyStart: m.index + m[0].length, entryStart: m.index })
    }

    for (let i = 0; i < entries.length; i++) {
        const { key, bodyStart } = entries[i]
        // Body ends where the next entry starts (or at end of string).
        const bodyEnd = i + 1 < entries.length ? entries[i + 1].entryStart : raw.length
        const body = raw.slice(bodyStart, bodyEnd)

        const fields: Record<string, string> = {}
        for (const name of fieldNames) {
            // Match:  fieldname = {value}   or   fieldname = "value"
            // The value must not contain unbalanced braces; for file/pdfpage
            // this is always satisfied.
            const re = new RegExp(`\\b${name}\\s*=\\s*(?:\\{([^{}]*)\\}|"([^"]*)")`, "i")
            const fm = re.exec(body)
            if (fm) fields[name] = (fm[1] ?? fm[2] ?? "").trim()
        }

        if (Object.keys(fields).length > 0) result.set(key, fields)
    }

    return result
}

/** Parses raw BibTeX text into a lookup map keyed by citation key. */
function parseBibTeXWithCite(
    raw: string,
    Cite: new (data: string) => { data: Record<string, unknown>[] },
): Map<string, ParsedCitation> {
    const cleaned = raw.replace(/^%.*$/gm, "")

    // citation-js gives us author objects + LaTeX unescaping. It silently
    // drops non-CSL fields (file, pdfpage), so we extract those ourselves.
    const cite = new Cite(cleaned)
    const rawFields = extractRawBibFields(raw, ["file", "pdfpage"])

    const result = new Map<string, ParsedCitation>()
    for (const entry of cite.data as Record<string, unknown>[]) {
        const key = entry["id"] as string
        const year =
            ((entry["issued"] as CslDate | undefined)?.["date-parts"]?.[0]?.[0] ?? "").toString()
        const authorDisplay = formatAuthors(entry["author"] as CslAuthor[] | undefined)
        const authorDisplayFull = formatAuthorsAll(entry["author"] as CslAuthor[] | undefined)
        const title = (entry["title"] as string | undefined) ?? ""
        const url = (entry["URL"] as string | undefined) || undefined
        const doi = (entry["DOI"] as string | undefined) || undefined

        // Use our regex extractor for fields citation-js drops.
        const extra = rawFields.get(key)
        const rawFile = extra?.["file"]
        const pdfCachePath = rawFile ? parseFileField(rawFile) : undefined
        const rawPage = extra?.["pdfpage"]
        const pdfPage = rawPage !== undefined ? parseInt(rawPage, 10) || undefined : undefined

        result.set(key, { key, authorDisplay, authorDisplayFull, year, title, url, doi, pdfCachePath, pdfPage })
    }
    return result
}

/**
 * Parses raw BibTeX, dynamically loading citation-js only when needed.
 * Returns an empty map on parse failure.
 */
export async function loadParsedBibliography(raw: string): Promise<Map<string, ParsedCitation>> {
    try {
        const [{ default: Cite }] = await Promise.all([
            import("citation-js"),
            import("@citation-js/plugin-bibtex"),
        ])
        return parseBibTeXWithCite(raw, Cite)
    } catch (err) {
        console.error("[oneil-webview] parseBibTeX error:", err)
        return new Map()
    }
}

/**
 * Parsed citation data derived from the raw BibTeX atom.
 * Starts empty and is populated asynchronously when a bibliography file exists.
 */
export const parsedBibliographyAtom = atom<Map<string, ParsedCitation>>(
    new Map(EMPTY_BIBLIOGRAPHY),
)

// ── Citation usages ───────────────────────────────────────────────────────────

/** Where in a model a citation was found. */
export type CitationUsageLocation =
    | { type: "model-note" }
    | { type: "section-note"; sectionLabel: string }
    | { type: "param-note"; paramName: string; paramLabel: string; paramKey: string }

/** A single occurrence of a citation key inside the current focused model. */
export interface CitationUsage {
    location: CitationUsageLocation
    /** Human-readable label shown in the bibliography panel. */
    displayLabel: string
}

/** All usages of a single citation key in the current focused model. */
export interface CitationGroup {
    key: string
    entry: ParsedCitation | undefined
    usages: CitationUsage[]
}

/** Bracketed citations: `[@key]`, `[@k1; @k2]`, `[+@key]`, `[-@key]`, `[!@key]`. */
const CITE_BRACKETED_RE = /\[([+\-!]?@[^\]]+)\]/g

/** Unbracketed textual citations: `@key`, `+@key`, `-@key`, `!@key`. */
const CITE_TEXTUAL_RE = /(?<![[\w])([+\-!]?)@([A-Za-z0-9_:.-]+)/g

/**
 * Strips a trailing inline page locator from a citation inner string so it is
 * not included when extracting citation keys.
 *
 * Handles: `, p.42`, `, p. 42`, `, pp.42`, `, pp.42-45`, `, 42`
 */
function stripPageLocator(s: string): string {
    return s.replace(/,\s*(?:pp?\.\s*)?\d+(?:-\d+)?\s*$/, "")
}

/** Extracts all citation keys from a note string. */
export function extractCitationKeys(text: string): string[] {
    const keys: string[] = []
    // Strip bracketed citations while collecting their keys, so the textual
    // regex doesn't double-match the @ inside `[@key]`.
    const remainder = text.replace(CITE_BRACKETED_RE, (_, inner: string) => {
        // Remove any trailing page locator before splitting on `;`.
        const cleaned = stripPageLocator(inner)
        for (const part of cleaned.split(";")) {
            const key = part.trim().replace(/^[+\-!]?@/, "").trim()
            if (key) keys.push(key)
        }
        return ""
    })
    for (const m of remainder.matchAll(CITE_TEXTUAL_RE)) {
        if (m[2]) keys.push(m[2])
    }
    return keys
}

/**
 * All citation keys used in the currently focused model node, grouped by key
 * and annotated with where they appear (model note, parameter note, etc.).
 * Only covers direct content of the focused node, not its children.
 */
export const citationGroupsAtom = atom<CitationGroup[]>((get) => {
    const node = get(focusedNodeAtom) ?? get(instanceTreeAtom)
    const bib = get(parsedBibliographyAtom)
    if (!node) return []

    const groups = new Map<string, CitationGroup>()

    const addUsage = (key: string, usage: CitationUsage) => {
        let group = groups.get(key)
        if (!group) {
            group = { key, entry: bib.get(key), usages: [] }
            groups.set(key, group)
        }
        group.usages.push(usage)
    }

    if (node.note) {
        for (const key of extractCitationKeys(node.note)) {
            addUsage(key, {
                location: { type: "model-note" },
                displayLabel: "Model note",
            })
        }
    }

    for (const param of node.parameters) {
        if (!param.note) continue
        for (const key of extractCitationKeys(param.note)) {
            addUsage(key, {
                location: {
                    type: "param-note",
                    paramName: param.name,
                    paramLabel: param.label,
                    paramKey: paramKey(node.instance_path, param.name),
                },
                displayLabel: param.label || param.name,
            })
        }
    }

    for (const section of node.sections) {
        if (!section.note) continue
        for (const key of extractCitationKeys(section.note)) {
            addUsage(key, {
                location: { type: "section-note", sectionLabel: section.label },
                displayLabel: `Section: ${section.label}`,
            })
        }
    }

    return [...groups.values()]
})
