/**
 * Side-panel layout: three vertically-stacked, independently-scrollable
 * sections (Model Hierarchy / Equation / Bibliography). Sections resize by
 * redistributing flex between neighbors via drag handles.
 */
import { useAtom, useAtomValue, useSetAtom } from "jotai"
import { useCallback, useRef } from "react"
import styled from "styled-components"
import { useResizeDrag } from "../utils/useResizeDrag"
import {
    citationGroupsAtom,
    detailPanelAtom,
    detailPanelSideBibFlexAtom,
    detailPanelSideEqFlexAtom,
    detailPanelSideTocFlexAtom,
    detailPanelSideWidthAtom,
    instanceTreeAtom,
    referencePoolAtom,
} from "../store/atoms"
import { ModelTOC } from "../components/ModelTOC"
import { BibliographyPanel } from "../components/BibliographyPanel"
import { ParameterDetailsPanel } from "../components/ParameterDetailsPanel"

// ── Styled components ─────────────────────────────────────────────────────────

/** Drag handle on the left edge that resizes the panel's overall width. */
const PanelResizeHandle = styled.div`
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: var(--resize-handle-size);
    cursor: col-resize;
    z-index: var(--z-detail-resize-handle);
    &:hover { background: var(--color-fg-subtle); }
`

/** Flex-column body; fills the remaining height below the panel header. */
const Body = styled.div`
    overflow: hidden;
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
`

/**
 * One independently-scrollable section. Each section receives its flex
 * proportion via an inline `style={{ flex }}` so adjacent drag handles can
 * redistribute flex while keeping the total constant.
 */
const Section = styled.div`
    display: flex;
    flex-direction: column;
    overflow: hidden;
    flex-shrink: 1;
    min-height: var(--detail-panel-section-min-height);
`

/** Non-scrolling title bar pinned at the top of a section. */
const SectionHeader = styled.div`
    flex-shrink: 0;
    padding: var(--space-xs) var(--space-md);
    border-bottom: 1px solid var(--color-border);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-bold);
    color: var(--color-fg-muted);
    text-transform: uppercase;
    letter-spacing: var(--letter-spacing-caps);
`

/** Scrollable content area inside a section. */
const SectionBody = styled.div`
    overflow: auto;
    flex: 1;
    padding: var(--space-sm) var(--space-md);
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    min-height: 0;
`

/** Horizontal drag handle between adjacent sections. */
const RowDividerHandle = styled.div`
    height: var(--resize-handle-size);
    flex-shrink: 0;
    background: var(--color-border);
    cursor: row-resize;
    &:hover { background: var(--color-fg-subtle); }
`

// ── Component ─────────────────────────────────────────────────────────────────

const MIN_WIDTH = 160
const MAX_WIDTH = () => window.innerWidth * 0.8

/**
 * Renders the side-panel layout. Reads and owns all side-specific resize state.
 */
export function SidePanelLayout() {
    const panelState = useAtomValue(detailPanelAtom)
    const citationGroups = useAtomValue(citationGroupsAtom)
    const rootNode = useAtomValue(instanceTreeAtom)
    const referencePool = useAtomValue(referencePoolAtom)

    const setSideWidth = useSetAtom(detailPanelSideWidthAtom)
    const [sideTocFlex, setSideTocFlex] = useAtom(detailPanelSideTocFlexAtom)
    const [sideEqFlex, setSideEqFlex] = useAtom(detailPanelSideEqFlexAtom)
    const [sideBibFlex, setSideBibFlex] = useAtom(detailPanelSideBibFlexAtom)
    const bodyRef = useRef<HTMLDivElement>(null)

    const hasModelHierarchy = !!rootNode && (rootNode.children.length > 0 || referencePool.length > 0)
    const hasBib = citationGroups.length > 0

    const onPanelDrag = useCallback((delta: number) => {
        setSideWidth(w => Math.round(Math.max(MIN_WIDTH, Math.min(MAX_WIDTH(), w - delta))))
    }, [setSideWidth])

    // Convert a pixel drag delta to a flex-unit change, redistributing flex
    // between the two adjacent sections so their combined proportion is preserved.
    const onTocRowDrag = useCallback((delta: number) => {
        const height = bodyRef.current?.clientHeight ?? 600
        const totalFlex = sideTocFlex + (panelState ? sideEqFlex : 0) + (hasBib ? sideBibFlex : 0)
        const change = delta * totalFlex / height
        setSideTocFlex(f => Math.max(0.05, f + change))
        if (panelState) {
            setSideEqFlex(f => Math.max(0.05, f - change))
        } else {
            setSideBibFlex(f => Math.max(0.05, f - change))
        }
    }, [panelState, hasBib, sideTocFlex, sideEqFlex, sideBibFlex,
        setSideTocFlex, setSideEqFlex, setSideBibFlex])

    const onBibRowDrag = useCallback((delta: number) => {
        const height = bodyRef.current?.clientHeight ?? 600
        const totalFlex = sideTocFlex + (panelState ? sideEqFlex : 0) + (hasBib ? sideBibFlex : 0)
        const change = delta * totalFlex / height
        setSideEqFlex(f => Math.max(0.05, f + change))
        setSideBibFlex(f => Math.max(0.05, f - change))
    }, [panelState, hasBib, sideTocFlex, sideEqFlex, sideBibFlex,
        setSideEqFlex, setSideBibFlex])

    const panelHandleMouseDown = useResizeDrag(onPanelDrag, "horizontal")
    const tocRowHandleMouseDown = useResizeDrag(onTocRowDrag, "vertical")
    const bibRowHandleMouseDown = useResizeDrag(onBibRowDrag, "vertical")

    return (
        <>
            <PanelResizeHandle onMouseDown={panelHandleMouseDown} />
            <Body ref={bodyRef}>
                {hasModelHierarchy && (
                    <>
                        <Section style={{ flex: sideTocFlex }}>
                            <SectionHeader>Model Hierarchy</SectionHeader>
                            <SectionBody><ModelTOC /></SectionBody>
                        </Section>
                        {(panelState || hasBib) && (
                            <RowDividerHandle onMouseDown={tocRowHandleMouseDown} />
                        )}
                    </>
                )}
                {panelState && (
                    <>
                        <ParameterDetailsPanel style={{ flex: sideEqFlex }} />
                        {hasBib && (
                            <RowDividerHandle onMouseDown={bibRowHandleMouseDown} />
                        )}
                    </>
                )}
                {hasBib && (
                    <Section style={{ flex: sideBibFlex }}>
                        <SectionBody><BibliographyPanel /></SectionBody>
                    </Section>
                )}
            </Body>
        </>
    )
}
