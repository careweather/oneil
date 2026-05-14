/**
 * Hook that drives a CSS-based resize drag along one axis.
 *
 * Returns a `mousedown` handler that, while the pointer is held, calls
 * `onDelta` with the pixel delta on every `mousemove`, then restores cursor
 * and text-selection styles on `mouseup`.
 *
 * Intentionally free of any component-specific logic so it can be reused by
 * any resizable panel or splitter.
 */
import type React from "react"
import { useCallback, useEffect, useRef } from "react"

/**
 * @param onDelta - Called on each mouse-move with the pixel delta along the
 *   given axis. Positive means right/down, negative means left/up.
 * @param direction - Axis to track: `"horizontal"` for left/right resize,
 *   `"vertical"` for top/bottom resize.
 * @returns A stable `mousedown` handler to attach to the drag handle element.
 */
export function useResizeDrag(
    onDelta: (delta: number) => void,
    direction: "horizontal" | "vertical",
): (e: React.MouseEvent) => void {
    // Keep a stable ref so the mousemove closure always sees the latest callback.
    const onDeltaRef = useRef(onDelta)
    useEffect(() => { onDeltaRef.current = onDelta }, [onDelta])

    return useCallback((e: React.MouseEvent) => {
        e.preventDefault()
        let last = direction === "horizontal" ? e.clientX : e.clientY
        const cursor = direction === "horizontal" ? "col-resize" : "row-resize"
        document.body.style.cursor = cursor
        document.body.style.userSelect = "none"

        const onMove = (ev: MouseEvent) => {
            const pos = direction === "horizontal" ? ev.clientX : ev.clientY
            onDeltaRef.current(pos - last)
            last = pos
        }
        const onUp = () => {
            document.body.style.cursor = ""
            document.body.style.userSelect = ""
            document.removeEventListener("mousemove", onMove)
            document.removeEventListener("mouseup", onUp)
        }
        document.addEventListener("mousemove", onMove)
        document.addEventListener("mouseup", onUp)
    }, [direction])
}
