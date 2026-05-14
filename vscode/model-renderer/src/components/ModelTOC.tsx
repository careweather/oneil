/**
 * Model hierarchy table of contents for the detail panel.
 *
 * Renders the full model tree as a navigable list where each entry can be
 * clicked to focus that model in the tree view. The currently focused model
 * is visually highlighted. Reference pool models are listed in a separate
 * section below the main tree; those entries focus the matching reference view.
 */
import { useAtom, useAtomValue } from "jotai"
import styled from "styled-components"
import {
    focusedPathAtom,
    modelTocAtom,
} from "../store/atoms"
import { pathKey, pathsEqual } from "../utils/instancePath"
import { AppliedDesign } from "../types/model"
import { DesignBadge } from "./DesignBadge"

// ── Styled components ─────────────────────────────────────────────────────────

const TOCContainer = styled.div`
    margin-bottom: var(--space-md);
`

const TOCList = styled.ul`
    list-style: none;
    margin: 0;
    padding: 0;
`

const TOCItem = styled.li<{ $depth: number; $active: boolean }>`
    padding: var(--toc-row-pad-y) var(--space-xs) var(--toc-row-pad-y)
        ${({ $depth }) => `calc(var(--toc-indent-base) + ${$depth} * var(--toc-indent-step))`};
    cursor: pointer;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
    color: ${({ $active }) => ($active ? "var(--color-fg)" : "var(--color-fg-muted)")};
    font-weight: ${({ $active }) => ($active ? "var(--font-weight-bold)" : "normal")};
    background: ${({ $active }) => ($active ? "var(--color-hover-bg)" : "transparent")};
    display: flex;
    align-items: baseline;
    gap: var(--space-xs);

    &:hover {
        background: var(--color-hover-bg);
        color: var(--color-fg);
    }
`

const TOCLabel = styled.span`
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
`

const TOCModelName = styled.span`
    color: var(--color-fg-subtle);
    font-size: var(--font-size-xs);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
`

const TOCIndentGuide = styled.span<{ $depth: number }>`
    display: ${({ $depth }) => ($depth > 0 ? "inline" : "none")};
    color: var(--color-border);
    flex-shrink: 0;
`

const TOCDesigns = styled.span`
    display: inline-flex;
    gap: var(--space-xs);
    align-items: baseline;
    margin-left: var(--space-xs);
`

const RefSectionDivider = styled.div`
    font-size: var(--font-size-xs);
    color: var(--color-fg-subtle);
    margin: var(--space-sm) 0 var(--space-xs) 0;
    padding-top: var(--space-xs);
    border-top: 1px solid var(--color-border);
`

// ── TOCRow component ──────────────────────────────────────────────────────────

interface TOCRowProps {
    entry: { depth: number; label: string; modelName: string; designs: AppliedDesign[] }
    isActive: boolean
    onClick: () => void
    itemKey: string
}

/** A single row in the TOC list, shared by both main-tree and reference sections. */
function TOCRow({ entry, isActive, onClick, itemKey }: TOCRowProps) {
    const isRoot = entry.depth === 0
    const title = `${entry.label} — ${entry.modelName}${entry.designs.length > 0 ? ` [${entry.designs.map((d) => d.design_name).join(", ")}]` : ""}`
    return (
        <TOCItem
            key={itemKey}
            $depth={entry.depth}
            $active={isActive}
            onClick={onClick}
            title={title}
        >
            {entry.depth > 0 && (
                <TOCIndentGuide $depth={entry.depth} aria-hidden>
                    {"·".repeat(entry.depth)}
                </TOCIndentGuide>
            )}
            <TOCLabel>{entry.label}</TOCLabel>
            {entry.designs.length > 0 && (
                <TOCDesigns>
                    {entry.designs.map((d) => (
                        <DesignBadge key={d.design_name} design={d} />
                    ))}
                </TOCDesigns>
            )}
            {!isRoot && <TOCModelName>({entry.modelName})</TOCModelName>}
        </TOCItem>
    )
}

// ── ModelTOC component ────────────────────────────────────────────────────────

/**
 * Renders a clickable table of contents for the full model hierarchy.
 *
 * Main-tree rows focus that model in the instance tree. Reference imports are
 * listed in a separate section; each row focuses that pool entry (and nested
 * path, when applicable) so the tree view switches to the focused reference.
 */
export function ModelTOC() {
    const toc = useAtomValue(modelTocAtom)
    const [focusedPath, setFocusedPath] = useAtom(focusedPathAtom)

    if (!toc) return null

    const { mainEntries, referenceSections } = toc

    // No hierarchy to navigate: just a single root with no children or refs.
    if (mainEntries.length <= 1 && referenceSections.length === 0) return null

    return (
        <TOCContainer>
            <TOCList>
                {mainEntries.map((entry) => {
                    const entryKey = pathKey(entry.path)
                    const isActive = pathsEqual(focusedPath, entry.path)
                    return (
                        <TOCRow
                            key={entryKey || "__root__"}
                            itemKey={entryKey || "__root__"}
                            entry={entry}
                            isActive={isActive}
                            onClick={() => setFocusedPath(entry.path)}
                        />
                    )
                })}
            </TOCList>
            {referenceSections.length > 0 && (
                <>
                    <RefSectionDivider>Reference Imports</RefSectionDivider>
                    <TOCList>
                        {referenceSections.flatMap((section) =>
                            section.entries.map((entry) => {
                                // Ref entries use unified paths: [alias, ...subpath]
                                const fullPath = [section.alias, ...entry.path]
                                const entryKey = `__toc_ref__${section.alias}/${pathKey(entry.path)}`
                                const isActive = pathsEqual(focusedPath, fullPath)
                                return (
                                    <TOCRow
                                        key={entryKey}
                                        itemKey={entryKey}
                                        entry={entry}
                                        isActive={isActive}
                                        onClick={() => setFocusedPath(fullPath)}
                                    />
                                )
                            })
                        )}
                    </TOCList>
                </>
            )}
        </TOCContainer>
    )
}
