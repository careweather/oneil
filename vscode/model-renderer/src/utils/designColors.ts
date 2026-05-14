/**
 * Utility helpers for the design overlay color system.
 *
 * The palette is defined as `--design-color-N` CSS custom properties in
 * `index.css`. Components read them at render time so they automatically
 * respect any theme overrides.
 */

const PALETTE_SIZE = 8

/**
 * Returns the CSS `var(--design-color-N)` string for a given `color_index`.
 * Wraps around if the index exceeds the palette size.
 *
 * @example
 * ```ts
 * designColorVar(0) // "var(--design-color-0)"
 * designColorVar(9) // "var(--design-color-1)"
 * ```
 */
export function designColorVar(colorIndex: number): string {
    return `var(--design-color-${colorIndex % PALETTE_SIZE})`
}

