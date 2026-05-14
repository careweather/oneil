/**
 * Detail panel wrapper: owns the outer container sizing, the shared header
 * (position toggle + close), and routes to the appropriate layout component.
 */
import { useAtom, useAtomValue, useSetAtom } from "jotai"
import { useCallback } from "react"
import styled from "styled-components"
import {
    detailPanelBottomHeightAtom,
    detailPanelOpenAtom,
    detailPanelPositionAtom,
    detailPanelSideWidthAtom,
    type DetailPanelPosition,
} from "../store/atoms"
import { SidePanelLayout } from "./SidePanelLayout"
import { BottomPanelLayout } from "./BottomPanelLayout"

// ── Styled components ─────────────────────────────────────────────────────────

const PanelContainer = styled.div<{ $position: DetailPanelPosition }>`
    display: flex;
    flex-direction: column;
    background: var(--color-bg-sidebar);
    border-left: ${({ $position }) => $position === "side" ? "1px solid var(--color-border)" : "none"};
    border-top: ${({ $position }) => $position === "bottom" ? "1px solid var(--color-border)" : "none"};
    ${({ $position }) => $position === "side"
        ? "min-width: var(--detail-panel-side-min-width); max-width: 80vw;"
        : "min-height: var(--detail-panel-bottom-min-height); max-height: 80vh;"}
    overflow: hidden;
    flex-shrink: 0;
    position: relative;
    font-size: var(--font-size-sm);
`

const PanelHeader = styled.div`
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
`

const PanelTitle = styled.div`
    font-weight: var(--font-weight-bold);
    font-size: var(--font-size-sm);
    color: var(--color-fg-muted);
`

const PanelActions = styled.div`
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    flex-shrink: 0;
`

export const PanelBtn = styled.button<{ $active?: boolean }>`
    background: ${({ $active }) => $active ? "var(--color-hover-bg)" : "transparent"};
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-fg-muted);
    cursor: pointer;
    padding: var(--space-2xs) var(--space-compact);
    font-size: var(--font-size-xs);
    line-height: 1.4;

    &:hover:not(:disabled) {
        background: var(--color-hover-bg);
        color: var(--color-fg);
    }

    &:disabled {
        opacity: 0.35;
        cursor: default;
    }
`

// ── Component ─────────────────────────────────────────────────────────────────

/**
 * Renders the detail panel. The shared header owns position toggle and close;
 * layout-specific resize logic and content live in `SidePanelLayout` and
 * `BottomPanelLayout`.
 */
export function DetailPanel() {
    const [position, setPosition] = useAtom(detailPanelPositionAtom)
    const sideWidth = useAtomValue(detailPanelSideWidthAtom)
    const bottomHeight = useAtomValue(detailPanelBottomHeightAtom)
    const setPanelOpen = useSetAtom(detailPanelOpenAtom)

    const closePanel = useCallback(() => setPanelOpen(false), [setPanelOpen])
    const togglePosition = useCallback(
        () => setPosition(p => p === "side" ? "bottom" : "side"),
        [setPosition],
    )

    return (
        <PanelContainer
            $position={position}
            style={position === "side" ? { width: sideWidth } : { height: bottomHeight }}
        >
            <PanelHeader>
                <PanelTitle>Details</PanelTitle>
                <PanelActions>
                    <PanelBtn
                        onClick={togglePosition}
                        title={position === "side" ? "Move to bottom" : "Move to side"}
                    >
                        {position === "side" ? "⇊" : "⇉"}
                    </PanelBtn>
                    <PanelBtn onClick={closePanel} title="Close panel">
                        ✕
                    </PanelBtn>
                </PanelActions>
            </PanelHeader>

            {position === "side" ? <SidePanelLayout /> : <BottomPanelLayout />}
        </PanelContainer>
    )
}

/**
 * Hook that returns the current panel position and whether the panel is open.
 * The `ContentArea` in `App.tsx` uses `isOpen` to decide the flex direction
 * for the bottom-panel layout.
 *
 * Panel open/closed state is independent of whether an equation is selected:
 * the ✕ in the equation header deselects the equation without closing the
 * panel, while the close button in the panel header hides the panel entirely.
 */
export function useDetailPanelLayout(): { position: DetailPanelPosition; isOpen: boolean } {
    const position = useAtomValue(detailPanelPositionAtom)
    const isOpen = useAtomValue(detailPanelOpenAtom)
    return { position, isOpen }
}
