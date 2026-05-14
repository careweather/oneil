/**
 * Bibliography panel — shows all citations used in the currently focused model.
 *
 * Each citation is formatted with author/year from `references.bib`.
 * Each usage is a clickable link that scrolls to the parameter row or note
 * that contains the citation.
 */
import { useAtomValue, useSetAtom } from "jotai"
import { useCallback } from "react"
import styled from "styled-components"
import {
    citationGroupsAtom,
    focusedParamKeyAtom,
    focusedPdfAtom,
    parsedBibliographyAtom,
    pdfCacheUriAtom,
    workspaceUriAtom,
    type CitationGroup,
    type CitationUsageLocation,
    type ParsedCitation,
} from "../store/atoms"
import { getVsCodeApi } from "../vscode"

// ── Styled components ─────────────────────────────────────────────────────────

const BibContainer = styled.div`
    margin-bottom: var(--space-md);
`

const BibHeading = styled.div`
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-bold);
    color: var(--color-fg-muted);
    text-transform: uppercase;
    letter-spacing: var(--letter-spacing-caps);
    margin-bottom: var(--space-xs);
`

const BibEntry = styled.div`
    margin-bottom: var(--space-sm);
    border-left: var(--border-accent-width) solid var(--color-border);
    padding-left: var(--space-sm);
`

const BibKey = styled.span`
    font-size: var(--font-size-xs);
    font-family: var(--font-mono);
    color: var(--color-fg-subtle);
    margin-right: 0.4em;
`

const BibTitle = styled.div`
    font-size: var(--font-size-sm);
    color: var(--color-fg);
    font-weight: var(--font-weight-bold);
    line-height: 1.3;
`

const BibTitleClickable = styled(BibTitle)`
    cursor: pointer;
    display: flex;
    align-items: baseline;
    gap: var(--space-2xs);
    &:hover { color: var(--color-focus-border); }
`

const PdfBadge = styled.span`
    font-size: var(--font-size-2xs, 0.65em);
    font-weight: normal;
    font-family: var(--font-mono);
    color: var(--color-fg-subtle);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-xs);
    padding: 0 0.3em;
    flex-shrink: 0;
    text-transform: uppercase;
    letter-spacing: 0.04em;
`

const BibMeta = styled.div`
    font-size: var(--font-size-xs);
    color: var(--color-fg-muted);
    margin-top: var(--space-hairline);
`

const BibUnknown = styled.div`
    font-size: var(--font-size-sm);
    color: var(--color-fg-subtle);
    font-style: italic;
`

const BibUsageList = styled.ul`
    list-style: none;
    margin: var(--space-xs) 0 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-2xs);
`

const BibUsageItem = styled.li`
    font-size: var(--font-size-xs);
    color: var(--color-fg-muted);
    padding: var(--space-2xs) var(--space-xs);
    border-radius: var(--radius-sm);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.3em;

    &:hover {
        background: var(--color-hover-bg);
        color: var(--color-fg);
    }
`

const BibUsageIcon = styled.span`
    opacity: 0.6;
    flex-shrink: 0;
`

const BibNoBibMessage = styled.p`
    font-size: var(--font-size-xs);
    color: var(--color-fg-subtle);
    font-style: italic;
    margin: 0;
`

/** Scroll to a note-like element and briefly highlight it. */
function flashNoteElement(el: Element): void {
    el.scrollIntoView({ behavior: "smooth", block: "center" })
    el.classList.add("model-note-flash")
    setTimeout(() => el.classList.remove("model-note-flash"), 1500)
}

/**
 * Handles a click on a citation usage. Exhaustive over {@link CitationUsageLocation}
 * so new location types must be handled here.
 */
function jumpToCitationUsage(
    loc: CitationUsageLocation,
    setFocusedParam: (key: string | null) => void,
): void {
    switch (loc.type) {
        case "param-note": {
            const el = document.querySelector(`[data-param-key="${loc.paramKey}"]`)
            if (el) {
                el.scrollIntoView({ behavior: "smooth", block: "center" })
                setFocusedParam(loc.paramKey)
                setTimeout(() => setFocusedParam(null), 1500)
            }
            break
        }
        case "section-note": {
            const el = document.querySelector(`[data-section-note="${CSS.escape(loc.sectionLabel)}"]`)
            if (el) flashNoteElement(el)
            break
        }
        case "model-note": {
            const el = document.querySelector("[data-model-note]")
            if (el) flashNoteElement(el)
            break
        }
        default: {
            const _exhaustive: never = loc
            void _exhaustive
        }
    }
}

// ── BibliographyPanel ─────────────────────────────────────────────────────────

/**
 * Renders the bibliography panel, listing all citations used in the focused
 * model with clickable usage links that scroll to the citing note.
 *
 * Returns `null` when there are no citations to show.
 */
export function BibliographyPanel() {
    const groups = useAtomValue(citationGroupsAtom)
    const bib = useAtomValue(parsedBibliographyAtom)
    const setFocusedParam = useSetAtom(focusedParamKeyAtom)
    const setFocusedPdf = useSetAtom(focusedPdfAtom)
    const pdfCacheUri = useAtomValue(pdfCacheUriAtom)
    const workspaceUri = useAtomValue(workspaceUriAtom)

    const handleUsageClick = useCallback(
        (group: CitationGroup, usageIdx: number) => {
            jumpToCitationUsage(group.usages[usageIdx].location, setFocusedParam)
        },
        [setFocusedParam],
    )

    /** Opens a PDF for the given entry, inline when a webview URL can be
     *  constructed from the cached path, otherwise falling back to the extension. */
    const handlePdfOpen = useCallback(
        (entry: ParsedCitation) => {
            const { pdfCachePath, pdfPage, url, doi, key, title } = entry
            const page = pdfPage ?? 1
            const pdfUrl = url ?? (doi ? `https://doi.org/${doi}` : null)

            let webviewUrl: string | null = null
            if (pdfCachePath) {
                const isBare = !pdfCachePath.startsWith("/") && !pdfCachePath.startsWith("~") &&
                    !pdfCachePath.startsWith("./") && !pdfCachePath.startsWith("../")
                if (isBare && pdfCacheUri) {
                    webviewUrl = `${pdfCacheUri.replace(/\/$/, "")}/${pdfCachePath}`
                } else if ((pdfCachePath.startsWith("./") || pdfCachePath.startsWith("../")) && workspaceUri) {
                    webviewUrl = `${workspaceUri.replace(/\/$/, "")}/${pdfCachePath.replace(/^\.\//, "")}`
                }
            }

            if (webviewUrl) {
                setFocusedPdf({ url: webviewUrl, page, title: title || key })
            } else {
                getVsCodeApi().postMessage({
                    type: "openPdf",
                    pdfUrl: pdfUrl ?? null,
                    cachePath: pdfCachePath ?? null,
                    page,
                    title: title || key,
                    citationKey: key,
                })
            }
        },
        [pdfCacheUri, workspaceUri, setFocusedPdf],
    )

    if (groups.length === 0) return null

    return (
        <BibContainer>
            <BibHeading>Bibliography ({groups.length})</BibHeading>
            {groups.map((group) => {
                const entry = group.entry
                const hasPdf = !!(entry?.pdfCachePath || entry?.url?.endsWith(".pdf") || entry?.doi)
                return (
                    <BibEntry key={group.key}>
                        {entry ? (
                            <>
                                {hasPdf ? (
                                    <BibTitleClickable
                                        onClick={() => handlePdfOpen(entry)}
                                        title="Open PDF"
                                    >
                                        {entry.title || group.key}
                                        <PdfBadge>PDF</PdfBadge>
                                    </BibTitleClickable>
                                ) : (
                                    <BibTitle>{entry.title || group.key}</BibTitle>
                                )}
                                <BibMeta>
                                    {[entry.authorDisplay, entry.year].filter(Boolean).join(", ")}
                                    {" "}
                                    <BibKey>[{group.key}]</BibKey>
                                </BibMeta>
                            </>
                        ) : (
                            <BibUnknown>
                                <BibKey>[{group.key}]</BibKey>
                                {bib.size === 0
                                    ? "no bibliography loaded"
                                    : "key not found in bibliography"}
                            </BibUnknown>
                        )}
                        {group.usages.length > 0 && (
                            <BibUsageList>
                                {group.usages.map((usage, ui) => (
                                    <BibUsageItem
                                        key={ui}
                                        onClick={() => handleUsageClick(group, ui)}
                                        title={
                                            usage.location.type === "param-note"
                                                ? `Jump to parameter: ${usage.displayLabel}`
                                                : "Jump to note"
                                        }
                                    >
                                        <BibUsageIcon>↗</BibUsageIcon>
                                        {usage.displayLabel}
                                    </BibUsageItem>
                                ))}
                            </BibUsageList>
                        )}
                    </BibEntry>
                )
            })}
            {bib.size === 0 && groups.length > 0 && (
                <BibNoBibMessage>
                    Add a <code>.bib</code> file (e.g. <code>references.bib</code>) to the workspace to show full references.
                </BibNoBibMessage>
            )}
        </BibContainer>
    )
}
