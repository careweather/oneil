import { useAtom, useAtomValue, useSetAtom } from "jotai"
import { useEffect, useCallback, useMemo, useRef } from "react"
import styled from "styled-components"
import { useTooltipTrigger } from "../components/Tooltip"
import { NodeContentGrid } from "../components/NodeContentGrid"
export { ValueDisplay } from "../components/ParameterRow"
import { NoteDisplay } from "../components/NoteDisplay"
import type { RenderedChild, RenderedNode, RenderedPoolEntry } from "../types/model"
import {
    detailPanelOpenAtom,
    focusedNodeAtom,
    focusedPathAtom,
    graphZoomAtom,
    isViewingRefAtom,
    showNotesEnabledAtom,
} from "../store/atoms"
import { modelDisplayName } from "../utils/modelPath"
import { DesignBadge } from "../components/DesignBadge"

// ── Instance tree styled components ──────────────────────────────────────────

const TreeContainer = styled.div`
    padding: var(--space-md) var(--space-lg);
    overflow: auto;
    flex: 1;
`

const ReferencePoolHeader = styled.h2`
    font-size: 1.0em;
    font-weight: var(--font-weight-bold);
    color: var(--color-fg-muted);
    margin: var(--space-lg) 0 var(--space-md) 0;
    padding-top: var(--space-md);
    border-top: 1px solid var(--color-border);
`


const ModelHeadingRow = styled.div`
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    flex-wrap: wrap;
    margin-bottom: var(--space-xs);
`

const ModelHeading = styled.h3`
    font-size: 1em;
    font-weight: var(--font-weight-bold);
    margin-bottom: 0;
`

const ModelNoteDiv = styled.div`
    color: var(--color-fg-muted);
    margin-bottom: var(--space-sm);
`

// ── Breadcrumb ────────────────────────────────────────────────────────────────

const BreadcrumbRow = styled.nav`
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--space-2xs);
    margin-bottom: var(--space-sm);
    font-size: var(--font-size-sm);
    min-height: 1.6em;
`

const BreadcrumbItem = styled.button`
    background: none;
    border: none;
    padding: var(--space-hairline) var(--space-xs);
    border-radius: var(--radius-sm);
    cursor: pointer;
    color: var(--color-fg-muted);
    font-size: inherit;

    &:hover {
        background: var(--color-hover-bg);
        color: var(--color-fg);
    }

    &:last-child {
        color: var(--color-fg);
        font-weight: var(--font-weight-bold);
        cursor: default;
        pointer-events: none;
    }
`

const BreadcrumbSep = styled.span`
    color: var(--color-fg-subtle);
    user-select: none;
`

// ── Submodel summary row ──────────────────────────────────────────────────────

const SubmodelsSection = styled.div`
    margin-top: var(--space-md);
`

const SubmodelsHeading = styled.div`
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-bold);
    color: var(--color-fg-muted);
    text-transform: uppercase;
    letter-spacing: var(--letter-spacing-caps);
    margin-bottom: var(--space-xs);
`

const SubmodelRowButton = styled.button`
    display: flex;
    align-items: baseline;
    gap: var(--space-sm);
    width: 100%;
    background: none;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: var(--space-xs) var(--space-sm);
    margin-bottom: var(--space-xs);
    cursor: pointer;
    text-align: left;
    font-size: var(--font-size-sm);

    &:hover {
        background: var(--color-hover-bg);
        border-color: var(--color-focus-border);
    }
`

const SubmodelAlias = styled.span`
    font-weight: var(--font-weight-bold);
    color: var(--color-fg);
    white-space: nowrap;
`

const SubmodelType = styled.span`
    color: var(--color-fg-muted);
    font-size: var(--font-size-xs);
    white-space: nowrap;
`

const SubmodelMeta = styled.span`
    color: var(--color-fg-subtle);
    font-size: var(--font-size-xs);
    margin-left: auto;
    white-space: nowrap;
    flex-shrink: 0;
`

const SubmodelDesigns = styled.span`
    display: inline-flex;
    gap: var(--space-xs);
    margin-left: var(--space-compact);
`

// ── InstanceTreeView ──────────────────────────────────────────────────────────

interface InstanceTreeViewProps {
    node: RenderedNode
    referencePool: RenderedPoolEntry[]
}

/**
 * Renders the evaluated model tree one level at a time.
 *
 * The currently focused model's parameters, tests and sections are shown in
 * full via `NodeContentGrid`. Each submodel appears as a compact summary row
 * that can be clicked to drill into it. A breadcrumb at the top allows
 * navigating back up the hierarchy.
 */
export function InstanceTreeView({ node, referencePool }: InstanceTreeViewProps) {
    const setGraphZoom = useSetAtom(graphZoomAtom)
    const [focusedPath, setFocusedPath] = useAtom(focusedPathAtom)
    const focusedNode = useAtomValue(focusedNodeAtom)
    const isViewingRef = useAtomValue(isViewingRefAtom)


    useEffect(() => { setGraphZoom(1) }, [setGraphZoom])

    const containerRef = useRef<HTMLDivElement>(null)

    // Scroll to the top whenever the focused model changes.
    useEffect(() => {
        if (containerRef.current) {
            containerRef.current.scrollTop = 0
        }
    }, [focusedPath])

    const handleNavigate = useCallback(
        (path: string[]) => { setFocusedPath(path) },
        [setFocusedPath],
    )

    const handleBackToRoot = useCallback(
        () => { setFocusedPath([]) },
        [setFocusedPath],
    )

    // Fall back to root if the focused path no longer resolves (after a reload).
    const currentNode = focusedNode ?? node

    // All hooks must be called unconditionally before any early return.
    const usedRefAliases = useMemo(
        () => new Set((currentNode.references ?? []).map((r) => r.alias)),
        [currentNode.references],
    )
    const visibleRefPool = useMemo(
        () => referencePool.filter((entry) => usedRefAliases.has(entry.alias)),
        [referencePool, usedRefAliases],
    )

    // ── Reference-pool focus ──────────────────────────────────────────────────
    if (isViewingRef && focusedPath.length > 0) {
        const refAlias = focusedPath[0]
        const refSubpath = focusedPath.slice(1)
        return (
            <TreeContainer ref={containerRef}>
                <RefPoolBreadcrumb
                    rootNode={node}
                    alias={refAlias}
                    subpath={refSubpath}
                    onBackToRoot={handleBackToRoot}
                    onNavigateSubpath={(subpath) => setFocusedPath([refAlias, ...subpath])}
                />
                <FocusedModelNode
                    node={currentNode}
                    focusedPath={focusedPath}
                    onNavigate={handleNavigate}
                />
            </TreeContainer>
        )
    }

    // ── Main tree focus ───────────────────────────────────────────────────────
    return (
        <TreeContainer ref={containerRef}>
            <Breadcrumb root={node} path={focusedPath} onNavigate={handleNavigate} />
            <FocusedModelNode
                node={currentNode}
                focusedPath={focusedPath}
                onNavigate={handleNavigate}
            />
            {focusedPath.length === 0 && visibleRefPool.length > 0 && (
                <>
                    <ReferencePoolHeader>Reference Imports</ReferencePoolHeader>
                    <SubmodelsSection>
                        {visibleRefPool.map((entry) => (
                            <RefPoolSummaryRow
                                key={entry.alias}
                                entry={entry}
                                onClick={() => setFocusedPath([entry.alias])}
                            />
                        ))}
                    </SubmodelsSection>
                </>
            )}
        </TreeContainer>
    )
}

// ── Breadcrumb ────────────────────────────────────────────────────────────────

interface BreadcrumbProps {
    root: RenderedNode
    path: string[]
    onNavigate: (path: string[]) => void
}

/**
 * Displays the navigation path from the root to the currently focused model.
 * Each segment is clickable to jump to that ancestor. Hidden when at root.
 */
function Breadcrumb({ root, path, onNavigate }: BreadcrumbProps) {
    if (path.length === 0) return null

    return (
        <BreadcrumbRow aria-label="Model breadcrumb">
            <BreadcrumbItem onClick={() => onNavigate([])}>
                {modelDisplayName(root.model_path)}
            </BreadcrumbItem>
            {path.map((alias, i) => (
                <span key={alias} style={{ display: "contents" }}>
                    <BreadcrumbSep aria-hidden>›</BreadcrumbSep>
                    <BreadcrumbItem onClick={() => onNavigate(path.slice(0, i + 1))}>
                        {alias}
                    </BreadcrumbItem>
                </span>
            ))}
        </BreadcrumbRow>
    )
}

// ── RefPoolBreadcrumb ─────────────────────────────────────────────────────────

interface RefPoolBreadcrumbProps {
    rootNode: RenderedNode
    alias: string
    /** Alias path from the ref root to the currently viewed submodel (empty at root). */
    subpath: string[]
    onBackToRoot: () => void
    /** Navigate to a subpath within the focused reference (empty → ref root). */
    onNavigateSubpath: (path: string[]) => void
}

/**
 * Breadcrumb shown when a reference-pool entry is focused.
 * Displays "Root › alias [› sub › ...]" where every ancestor segment is
 * clickable; the last (current) segment is styled as active by CSS `:last-child`.
 */
function RefPoolBreadcrumb({ rootNode, alias, subpath, onBackToRoot, onNavigateSubpath }: RefPoolBreadcrumbProps) {
    return (
        <BreadcrumbRow aria-label="Reference import breadcrumb">
            <BreadcrumbItem onClick={onBackToRoot}>
                {modelDisplayName(rootNode.model_path)}
            </BreadcrumbItem>
            <BreadcrumbSep aria-hidden>›</BreadcrumbSep>
            <BreadcrumbItem onClick={() => onNavigateSubpath([])}>
                {alias}
            </BreadcrumbItem>
            {subpath.map((seg, i) => (
                <span key={seg} style={{ display: "contents" }}>
                    <BreadcrumbSep aria-hidden>›</BreadcrumbSep>
                    <BreadcrumbItem onClick={() => onNavigateSubpath(subpath.slice(0, i + 1))}>
                        {seg}
                    </BreadcrumbItem>
                </span>
            ))}
        </BreadcrumbRow>
    )
}

// ── FocusedModelNode ──────────────────────────────────────────────────────────

interface FocusedModelNodeProps {
    node: RenderedNode
    focusedPath: string[]
    onNavigate: (path: string[]) => void
}

/**
 * Renders a single model node at full detail (heading, note, parameters,
 * sections, tests) and lists its submodels as compact navigable summary rows.
 */
function FocusedModelNode({ node, focusedPath, onNavigate }: FocusedModelNodeProps) {
    const modelName = modelDisplayName(node.model_path)
    const showNotes = useAtomValue(showNotesEnabledAtom)
    const tooltipProps = useTooltipTrigger(!showNotes ? node.note : undefined)
    const setPanelOpen = useSetAtom(detailPanelOpenAtom)

    return (
        <section>
            <ModelHeadingRow>
                <ModelHeading
                    className={tooltipProps.className || undefined}
                    onMouseEnter={tooltipProps.onMouseEnter}
                    onMouseLeave={tooltipProps.onMouseLeave}
                >
                    {modelName}
                </ModelHeading>
                {node.applied_designs.map((d) => (
                    <DesignBadge key={d.design_name} design={d} />
                ))}
            </ModelHeadingRow>
            {showNotes && node.note && (
                <ModelNoteDiv data-model-note="true">
                    <NoteDisplay text={node.note} parameters={node.parameters} />
                </ModelNoteDiv>
            )}
            <NodeContentGrid node={node} variant="tree" />
            {node.children.length > 0 && (
                <SubmodelsSection>
                    <SubmodelsHeading>Submodels</SubmodelsHeading>
                    {node.children.map((child) => (
                        <SubmodelSummaryRow
                            key={child.alias}
                            child={child}
                            onClick={() => {
                                onNavigate([...focusedPath, child.alias])
                                setPanelOpen(true)
                            }}
                        />
                    ))}
                </SubmodelsSection>
            )}
        </section>
    )
}

// ── SubmodelSummaryRow ────────────────────────────────────────────────────────

interface SubmodelSummaryRowProps {
    child: RenderedChild
    onClick: () => void
}

/**
 * A compact single-line representation of a submodel that navigates into it
 * when clicked. Shows the alias, model type, and counts of parameters and
 * nested submodels.
 */
function SubmodelSummaryRow({ child, onClick }: SubmodelSummaryRowProps) {
    const modelName = modelDisplayName(child.node.model_path)
    const paramCount = child.node.parameters.length
    const childCount = child.node.children.length
    const designs = child.node.applied_designs

    const metaParts: string[] = []
    if (paramCount > 0) metaParts.push(`${paramCount} param${paramCount !== 1 ? "s" : ""}`)
    if (childCount > 0) metaParts.push(`${childCount} submodel${childCount !== 1 ? "s" : ""}`)

    return (
        <SubmodelRowButton onClick={onClick} title={`Navigate into ${child.alias}`}>
            <SubmodelAlias>{child.alias}</SubmodelAlias>
            <SubmodelType>({modelName})</SubmodelType>
            {designs.length > 0 && (
                <SubmodelDesigns>
                    {designs.map((d) => (
                        <DesignBadge key={d.design_name} design={d} />
                    ))}
                </SubmodelDesigns>
            )}
            {metaParts.length > 0 && (
                <SubmodelMeta>{metaParts.join(", ")}</SubmodelMeta>
            )}
        </SubmodelRowButton>
    )
}

// ── RefPoolSummaryRow ─────────────────────────────────────────────────────────

interface RefPoolSummaryRowProps {
    entry: RenderedPoolEntry
    onClick: () => void
}

/**
 * A compact single-line representation of a reference import that navigates
 * into the focused reference view when clicked. Mirrors `SubmodelSummaryRow`.
 */
function RefPoolSummaryRow({ entry, onClick }: RefPoolSummaryRowProps) {
    const modelName = modelDisplayName(entry.node.model_path)
    const paramCount = entry.node.parameters.length
    const childCount = entry.node.children.length
    const designs = entry.node.applied_designs

    const metaParts: string[] = []
    if (paramCount > 0) metaParts.push(`${paramCount} param${paramCount !== 1 ? "s" : ""}`)
    if (childCount > 0) metaParts.push(`${childCount} submodel${childCount !== 1 ? "s" : ""}`)

    return (
        <SubmodelRowButton onClick={onClick} title={`Navigate into ${entry.alias}`}>
            <SubmodelAlias>{entry.alias}</SubmodelAlias>
            <SubmodelType>({modelName})</SubmodelType>
            {designs.length > 0 && (
                <SubmodelDesigns>
                    {designs.map((d) => (
                        <DesignBadge key={d.design_name} design={d} />
                    ))}
                </SubmodelDesigns>
            )}
            {metaParts.length > 0 && (
                <SubmodelMeta>{metaParts.join(", ")}</SubmodelMeta>
            )}
        </SubmodelRowButton>
    )
}

