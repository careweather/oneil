/**
 * Utilities for working with Oneil model file paths from the LSP.
 */

/**
 * Returns the display name for a model file path: the last path segment with
 * any `.on` or `.one` extension stripped.
 *
 * @example
 * ```ts
 * modelDisplayName("/path/to/vehicle.on")  // "vehicle"
 * modelDisplayName("/path/to/overlay.one") // "overlay"
 * modelDisplayName("engine")               // "engine"
 * ```
 */
export function modelDisplayName(modelPath: string): string {
    const segment = modelPath.split("/").at(-1) ?? modelPath
    return segment.replace(/\.(on|one)$/, "")
}
