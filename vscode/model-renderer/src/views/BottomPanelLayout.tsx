/**
 * Bottom-panel layout: three horizontally-arranged columns
 * (Model Hierarchy | Equation | Bibliography). TOC and bibliography columns
 * have fixed pixel widths; the equation column takes the remaining space.
 */
import { useAtom, useAtomValue, useSetAtom } from "jotai"
import { useCallback } from "react"
import styled from "styled-components"
import { useResizeDrag } from "../utils/useResizeDrag"
import {
    citationGroupsAtom,
    detailPanelAtom,
    detailPanelBibWidthAtom,
    detailPanelBottomHeightAtom,
    detailPanelTocWidthAtom,
    instanceTreeAtom,
    referencePoolAtom,
} from "../store/atoms"
import { ModelTOC } from "../components/ModelTOC"
import { BibliographyPanel } from "../components/BibliographyPanel"
import { ParameterDetailsPanel } from "../components/ParameterDetailsPanel"

// ── Styled components ─────────────────────────────────────────────────────────

/** Drag handle on the top edge that resizes the panel's overall height. */
const PanelResizeHandle = styled.div`
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: var(--resize-handle-size);
    cursor: row-resize;
    z-index: var(--z-detail-resize-handle);
    &:hover { background: var(--color-fg-subtle); }
`

/** Flex-row body; fills the remaining height below the panel header. */
const Body = styled.div`
    overflow: hidden;
    flex: 1;
    display: flex;
    flex-direction: row;
    min-height: 0;
`

/** Base scrollable column. */
const Column = styled.div`
    overflow: auto;
    padding: var(--space-sm) var(--space-md);
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    min-width: 0;
    min-height: 0;
`

/** Fixed-width column; caller controls width via inline style. */
const FixedColumn = styled(Column)`
    flex-shrink: 0;
`

/** Non-scrolling column heading (mirrors the side layout's SectionHeader). */
const ColumnHeader = styled.div`
    flex-shrink: 0;
    padding: var(--space-xs) var(--space-md);
    border-bottom: 1px solid var(--color-border);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-bold);
    color: var(--color-fg-muted);
    text-transform: uppercase;
    letter-spacing: var(--letter-spacing-caps);
`

/** Vertical drag handle between adjacent columns. */
const ColumnDividerHandle = styled.div`
    width: var(--resize-handle-size);
    flex-shrink: 0;
    background: var(--color-border);
    cursor: col-resize;
    &:hover { background: var(--color-fg-subtle); }
`

// ── Component ─────────────────────────────────────────────────────────────────

const MIN_HEIGHT = 100
const MAX_HEIGHT = () => window.innerHeight * 0.8
const MIN_COL = 120

/**
 * Renders the bottom-panel layout. Reads and owns all bottom-specific resize state.
 */
export function BottomPanelLayout() {
    const panelState = useAtomValue(detailPanelAtom)
    const citationGroups = useAtomValue(citationGroupsAtom)
    const rootNode = useAtomValue(instanceTreeAtom)
    const referencePool = useAtomValue(referencePoolAtom)

    const setBottomHeight = useSetAtom(detailPanelBottomHeightAtom)
    const [tocWidth, setTocWidth] = useAtom(detailPanelTocWidthAtom)
    const [bibWidth, setBibWidth] = useAtom(detailPanelBibWidthAtom)

    const hasModelHierarchy = !!rootNode && (rootNode.children.length > 0 || referencePool.length > 0)
    const hasBib = citationGroups.length > 0

    const onPanelDrag = useCallback((delta: number) => {
        setBottomHeight(h => Math.round(Math.max(MIN_HEIGHT, Math.min(MAX_HEIGHT(), h - delta))))
    }, [setBottomHeight])

    const onTocDrag = useCallback((delta: number) => {
        setTocWidth(w => Math.round(Math.max(MIN_COL, w + delta)))
    }, [setTocWidth])

    const onBibDrag = useCallback((delta: number) => {
        setBibWidth(w => Math.round(Math.max(MIN_COL, w - delta)))
    }, [setBibWidth])

    const panelHandleMouseDown = useResizeDrag(onPanelDrag, "vertical")
    const tocHandleMouseDown = useResizeDrag(onTocDrag, "horizontal")
    const bibHandleMouseDown = useResizeDrag(onBibDrag, "horizontal")

    return (
        <>
            <PanelResizeHandle onMouseDown={panelHandleMouseDown} />
            <Body>
                {hasModelHierarchy && (
                    <>
                        <FixedColumn style={{ width: tocWidth }}>
                            <ColumnHeader>Model Hierarchy</ColumnHeader>
                            <ModelTOC />
                        </FixedColumn>
                        {(panelState || hasBib) && (
                            <ColumnDividerHandle onMouseDown={tocHandleMouseDown} />
                        )}
                    </>
                )}
                {panelState && (
                    <>
                        <ParameterDetailsPanel style={{ flex: 1 }} />
                        {hasBib && (
                            <ColumnDividerHandle onMouseDown={bibHandleMouseDown} />
                        )}
                    </>
                )}
                {hasBib && (
                    <FixedColumn style={{ width: bibWidth }}>
                        <BibliographyPanel />
                    </FixedColumn>
                )}
            </Body>
        </>
    )
}
