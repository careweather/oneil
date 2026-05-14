import React, { useCallback, useEffect, useRef, useState } from "react"

/** Pixel dimensions of a measured element. */
export interface ContentSize {
    width: number
    height: number
}

/** One item to render and measure. */
export interface MeasureItem {
    /** Stable ID — used to track which measurements have arrived. */
    id: string
    /** The React content to render at `constraintWidth` and measure. */
    element: React.ReactNode
}

/**
 * Renders each item in a hidden absolutely-positioned container at
 * `constraintWidth` pixels wide, then reports the natural height of each item
 * via `ResizeObserver`.
 *
 * **Important**: the returned `container` element *must* be rendered somewhere
 * inside the same component tree as the caller so that it inherits the correct
 * CSS context (font-family, font-size, custom properties, etc.).  A portal is
 * intentionally avoided for this reason.
 *
 * Behaviour:
 * - Returns `null` until every item has reported at least one measurement.
 * - Updates automatically whenever the dimensions of any item change (e.g.
 *   after a font-size change cascades into the hidden divs).
 * - Resets to `null` and re-measures whenever the set of item IDs changes
 *   (i.e. when the node tree changes).
 * - All measurements that arrive in the same event-loop tick are batched into
 *   a single `setState` call to avoid redundant re-renders.
 *
 * ```tsx
 * const items = useMemo(() => nodes.map(n => ({ id: n.id, element: <NodeContent node={n} /> })), [nodes])
 * const { sizes, container } = useMeasureContent(items, NODE_WIDTH)
 * return <div style={{ position: "relative" }}>{container}<Graph sizes={sizes} /></div>
 * ```
 */
export function useMeasureContent(
    items: readonly MeasureItem[],
    constraintWidth: number,
): {
    /** `null` while waiting for the first complete measurement pass. */
    sizes: Map<string, ContentSize> | null
    /**
     * Must be rendered inside the component tree (not in a portal).
     * Position the parent with `position: relative` so the hidden container
     * is clipped to the parent's visual bounds.
     */
    container: React.ReactElement
} {
    const [sizes, setSizes] = useState<Map<string, ContentSize> | null>(null)

    // Refs so ResizeObserver callbacks always see up-to-date data without
    // needing to be re-created on every render.
    const pendingRef = useRef(new Set<string>())
    const collectedRef = useRef(new Map<string, ContentSize>())
    const flushTimerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined)

    // Reset whenever the set of node IDs changes (tree structure changed).
    const itemKey = items.map((i) => i.id).join("\0")
    useEffect(() => {
        pendingRef.current = new Set(items.map((i) => i.id))
        collectedRef.current = new Map()
        setSizes(null)
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [itemKey])

    // Stable callback — safe to pass as a prop without triggering re-mounts.
    const onMeasured = useCallback((id: string, size: ContentSize) => {
        collectedRef.current.set(id, size)
        pendingRef.current.delete(id)
        // Batch all measurements that arrive in the same tick into one setState.
        clearTimeout(flushTimerRef.current)
        flushTimerRef.current = setTimeout(() => {
            setSizes(new Map(collectedRef.current))
        }, 0)
    }, [])

    const container = (
        <div
            aria-hidden="true"
            style={{
                position: "absolute",
                visibility: "hidden",
                pointerEvents: "none",
                // Stack slots starting at top-left; height grows freely downward.
                // The parent must be `position: relative` so this stays anchored.
                top: 0,
                left: 0,
                zIndex: -1,
            }}
        >
            {items.map((item) => (
                <MeasureSlot
                    key={item.id}
                    id={item.id}
                    width={constraintWidth}
                    onMeasured={onMeasured}
                >
                    {item.element}
                </MeasureSlot>
            ))}
        </div>
    )

    return { sizes, container }
}

// ── Internal ──────────────────────────────────────────────────────────────────

interface MeasureSlotProps {
    id: string
    width: number
    onMeasured: (id: string, size: ContentSize) => void
    children: React.ReactNode
}

/**
 * Renders `children` at intrinsic width (`max-content`) and calls `onMeasured`
 * with the resulting dimensions. Re-fires whenever the element's layout changes
 * via `ResizeObserver` (e.g. after a parent font-size change).
 *
 * The `minWidth` prop sets a floor so narrow content doesn't produce
 * unreasonably small measurements.
 */
function MeasureSlot({ id, width: minWidth, onMeasured, children }: MeasureSlotProps) {
    const ref = useRef<HTMLDivElement>(null)

    useEffect(() => {
        const el = ref.current
        if (!el) return

        const report = () => onMeasured(id, { width: el.offsetWidth, height: el.offsetHeight })

        const ro = new ResizeObserver(report)
        ro.observe(el)
        // Fire once immediately — ResizeObserver may not fire for the initial paint
        // before the observer is set up.
        report()
        return () => ro.disconnect()
    }, [id, minWidth, onMeasured])

    return (
        <div
            ref={ref}
            style={{
                width: "max-content",
                minWidth,
                boxSizing: "border-box",
            }}
        >
            {children}
        </div>
    )
}
