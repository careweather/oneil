import { useAtomValue } from "jotai"
import { createContext, useCallback, useContext, useEffect, useLayoutEffect, useRef, useState, type ReactNode } from "react"
import { createPortal } from "react-dom"
import styled from "styled-components"
import { fontScaleAtom, graphZoomAtom } from "../store/atoms"
import { NoteDisplay } from "./NoteDisplay"

// ── Tooltip styled components ────────────────────────────────────────────────

const TooltipPopup = styled.div`
    z-index: var(--z-tooltip);
    padding: var(--space-sm) var(--space-md);
    max-width: var(--tooltip-max-width);
    font-style: normal;
    font-weight: normal;
    line-height: 1.5;
    color: var(--color-fg);
    background: var(--vscode-editorHoverWidget-background, var(--color-bg));
    border: 1px solid var(--vscode-editorHoverWidget-border, var(--color-border));
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-tooltip);
    pointer-events: none;
    transition: none !important;

    * {
        transition: none !important;
    }
`

interface TooltipState {
    content: ReactNode | null
    x: number
    y: number
}

interface TooltipContextValue {
    show: (content: ReactNode, x: number, y: number) => void
    hide: () => void
}

const TooltipContext = createContext<TooltipContextValue | null>(null)

/**
 * Provides tooltip functionality to the component tree.
 * Renders the tooltip via portal at document body level so it appears
 * above all other content including ReactFlow nodes.
 * Font size scales with both the user's fontScale preference and the
 * current ReactFlow zoom level.
 */
export function TooltipProvider({ children }: { children: ReactNode }) {
    const [state, setState] = useState<TooltipState>({ content: null, x: 0, y: 0 })
    // Track the final computed left position; null means not yet computed
    const [computedLeft, setComputedLeft] = useState<number | null>(null)
    const fontScale = useAtomValue(fontScaleAtom)
    const graphZoom = useAtomValue(graphZoomAtom)

    const show = useCallback((content: ReactNode, x: number, y: number) => {
        setComputedLeft(null) // Reset for new tooltip
        setState({ content, x, y })
    }, [])

    const hide = useCallback(() => {
        setState((s) => ({ ...s, content: null }))
        setComputedLeft(null)
    }, [])

    // Combine font scale and graph zoom; clamp zoom contribution to reasonable bounds
    const effectiveZoom = Math.max(0.5, Math.min(2, graphZoom))
    const tooltipFontSize = fontScale * 0.85 * effectiveZoom

    const popupRef = useRef<HTMLDivElement>(null)

    // Global safety net: if the tooltip is visible but the cursor wanders off
    // every .has-tooltip element (rapid movement, scroll, element re-render,
    // or cursor leaving the browser), force-hide immediately.
    // Only active while a tooltip is actually showing.
    useEffect(() => {
        if (state.content == null) return
        const handleMove = (e: MouseEvent) => {
            if (!(e.target as Element | null)?.closest(".has-tooltip")) hide()
        }
        document.addEventListener("mousemove", handleMove)
        document.addEventListener("mouseleave", hide)
        return () => {
            document.removeEventListener("mousemove", handleMove)
            document.removeEventListener("mouseleave", hide)
        }
    }, [state.content, hide])

    // Compute clamped position once after initial render, before paint
    useLayoutEffect(() => {
        if (computedLeft !== null) return // Already positioned
        const el = popupRef.current
        if (!el) return
        const { width } = el.getBoundingClientRect()
        const clamped = Math.max(8, Math.min(state.x, window.innerWidth - width - 8))
        setComputedLeft(clamped)
    }, [state.content, state.x, computedLeft])

    // Determine positioning: off-screen while measuring, final position once computed
    const leftPos = computedLeft ?? -9999
    const isVisible = computedLeft !== null

    return (
        <TooltipContext.Provider value={{ show, hide }}>
            {children}
            {state.content != null && createPortal(
                <TooltipPopup
                    ref={popupRef}
                    style={{
                        position: "fixed",
                        left: leftPos,
                        top: state.y,
                        visibility: isVisible ? "visible" : "hidden",
                        fontSize: `calc(var(--vscode-font-size, 13px) * ${tooltipFontSize})`,
                    }}
                >
                    {state.content}
                </TooltipPopup>,
                document.body,
            )}
        </TooltipContext.Provider>
    )
}

/**
 * Hook to access tooltip show/hide functions.
 */
export function useTooltip() {
    const ctx = useContext(TooltipContext)
    if (!ctx) throw new Error("useTooltip must be used within TooltipProvider")
    return ctx
}

/**
 * Makes an element show a note-string tooltip on hover.
 * The content is wrapped in `<NoteDisplay>` so LaTeX/markdown renders correctly.
 */
export function useTooltipTrigger(content: string | null | undefined) {
    const tooltip = useTooltip()

    const node: ReactNode | null = content ? <NoteDisplay text={content} /> : null

    const onMouseEnter = useCallback(
        (e: React.MouseEvent) => {
            if (!node) return
            const rect = e.currentTarget.getBoundingClientRect()
            tooltip.show(node, rect.left, rect.bottom + 4)
        },
        // eslint-disable-next-line react-hooks/exhaustive-deps
        [content, tooltip],
    )

    const onMouseLeave = useCallback(() => {
        tooltip.hide()
    }, [tooltip])

    if (!content) return {}

    return {
        onMouseEnter,
        onMouseLeave,
        className: "has-tooltip",
    }
}

/**
 * Like `useTooltipTrigger` but accepts arbitrary ReactNode content.
 * Use when you need richer tooltip content than a plain note string.
 */
export function useReactTooltipTrigger(content: ReactNode | null | undefined) {
    const tooltip = useTooltip()

    const onMouseEnter = useCallback(
        (e: React.MouseEvent) => {
            if (content == null) return
            const rect = e.currentTarget.getBoundingClientRect()
            tooltip.show(content, rect.left, rect.bottom + 4)
        },
        // eslint-disable-next-line react-hooks/exhaustive-deps
        [content, tooltip],
    )

    const onMouseLeave = useCallback(() => {
        tooltip.hide()
    }, [tooltip])

    if (content == null) return {}

    return {
        onMouseEnter,
        onMouseLeave,
        className: "has-tooltip",
    }
}
