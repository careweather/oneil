/**
 * Standalone PDF viewer panel.
 *
 * Rendered alongside the main view (not inside the equations/bibliography
 * detail panel), so it is always visible when a PDF is focused regardless
 * of whether the detail panel is open.
 *
 * The panel docks to the right of the main `ViewContainer` with a
 * drag-resizable left edge, mirroring `DetailPanel`'s side layout.
 */

import { useAtom, useAtomValue } from "jotai"
import { useCallback } from "react"
import styled from "styled-components"
import { focusedPdfAtom, pdfPanelWidthAtom } from "../store/atoms"
import { PdfPane } from "../components/PdfPane"
import { useResizeDrag } from "../utils/useResizeDrag"

// ── Styled components ─────────────────────────────────────────────────────────

const PanelContainer = styled.div`
    display: flex;
    flex-direction: column;
    background: var(--color-bg-sidebar);
    border-left: 1px solid var(--color-border);
    min-width: var(--detail-panel-side-min-width);
    max-width: 80vw;
    overflow: hidden;
    flex-shrink: 0;
    position: relative;
    font-size: var(--font-size-sm);
`

/** Drag handle on the left edge for resizing the panel width. */
const ResizeHandle = styled.div`
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: var(--resize-handle-size);
    cursor: col-resize;
    z-index: var(--z-detail-resize-handle);
    &:hover { background: var(--color-fg-subtle); }
`

// ── Component ─────────────────────────────────────────────────────────────────

const MIN_WIDTH = 200
const MAX_WIDTH = () => window.innerWidth * 0.8

/**
 * Renders the PDF panel when a PDF is focused.
 * Returns `null` and takes no layout space when `focusedPdfAtom` is null.
 *
 * ```tsx
 * <PdfPanel />
 * ```
 */
export function PdfPanel() {
    const focusedPdf = useAtomValue(focusedPdfAtom)
    const [width, setWidth] = useAtom(pdfPanelWidthAtom)

    const onDrag = useCallback((delta: number) => {
        setWidth(w => Math.round(Math.max(MIN_WIDTH, Math.min(MAX_WIDTH(), w - delta))))
    }, [setWidth])

    const handleMouseDown = useResizeDrag(onDrag, "horizontal")

    if (!focusedPdf) return null

    return (
        <PanelContainer style={{ width }}>
            <ResizeHandle onMouseDown={handleMouseDown} />
            <PdfPane />
        </PanelContainer>
    )
}
