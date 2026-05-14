/**
 * Utilities for working with instance / alias paths.
 *
 * An **instance path** is an ordered `string[]` of alias segments that locates
 * a model node inside a hierarchy:
 *   - `[]`              → root node (main tree)
 *   - `["rover"]`       → the child aliased as "rover"
 *   - `["rover", "arm"]` → the grandchild aliased as "arm" inside "rover"
 *   - `["sensor"]`      → a ref pool root (if "sensor" is a ref alias)
 *   - `["sensor", "optics"]` → a submodel inside a ref pool entry
 *
 * These paths appear as `RenderedNode.instance_path` from the LSP and are used
 * for the unified `focusedPathAtom` and TOC entries. Tree membership is
 * determined from the first path segment: if it's in `refPoolAliases`, the
 * node is in a reference; otherwise it's in the main tree.
 *
 * ## Unified key format
 *
 * A single `/` separator is used everywhere: path keys, node IDs, and parameter
 * dependency keys. Alias segments and parameter names are Oneil identifiers
 * (`[A-Za-z_][A-Za-z0-9_]*`) so they cannot contain `/`, making this
 * unambiguous.
 *
 * | Purpose                          | Example                    | Helper            |
 * |----------------------------------|----------------------------|-------------------|
 * | React keys, navigation equality  | `"rover/arm"`, `""`        | `pathKey`         |
 * | ReactFlow node IDs (non-empty)   | `"rover/arm"`, `"__root__"`| `pathToNodeId`    |
 * | All param keys                   | `"rover/arm/thrust"`       | `paramKey`        |
 */

// ── Serialisation ─────────────────────────────────────────────────────────────

const SEP = "/"

/**
 * Returns a `/`-separated string key for a path, suitable for React `key`
 * props and navigation equality checks. Returns `""` for the root path.
 *
 * @example
 * ```ts
 * pathKey([])              // ""
 * pathKey(["rover"])       // "rover"
 * pathKey(["rover","arm"]) // "rover/arm"
 * ```
 */
export function pathKey(path: string[]): string {
    return path.join(SEP)
}

/**
 * Returns a non-empty node-ID string for use in ReactFlow and layout caches.
 * Falls back to `"__root__"` for the empty path.
 *
 * @example
 * ```ts
 * pathToNodeId([])              // "__root__"
 * pathToNodeId(["rover"])       // "rover"
 * pathToNodeId(["rover","arm"]) // "rover/arm"
 * ```
 */
export function pathToNodeId(path: string[]): string {
    return path.length > 0 ? pathKey(path) : "__root__"
}

// ── Parameter keys ────────────────────────────────────────────────────────────

/**
 * Creates a unique key for a parameter given its instance path and name.
 * This is the unified key format for both main tree and reference pool params.
 *
 * @example
 * ```ts
 * paramKey([], "thrust")           // "thrust"
 * paramKey(["engine"], "thrust")   // "engine/thrust"
 * paramKey(["sensor", "optics"], "fov") // "sensor/optics/fov"
 * ```
 */
export function paramKey(instancePath: string[], paramName: string): string {
    const prefix = pathKey(instancePath)
    return prefix ? `${prefix}/${paramName}` : paramName
}

// ── Comparison ────────────────────────────────────────────────────────────────

/**
 * Returns `true` when two paths contain identical segments in the same order.
 *
 * @example
 * ```ts
 * pathsEqual(["a", "b"], ["a", "b"]) // true
 * pathsEqual(["a"], ["a", "b"])      // false
 * ```
 */
export function pathsEqual(a: string[], b: string[]): boolean {
    return a.length === b.length && a.every((v, i) => v === b[i])
}

/**
 * Returns `true` when `ancestor` is a (non-strict) prefix of `path` — i.e.
 * `path` lies within the instance subtree rooted at `ancestor`.
 *
 * @example
 * ```ts
 * isPathPrefix([], ["a", "b"])       // true  (root is prefix of everything)
 * isPathPrefix(["a"], ["a", "b"])    // true
 * isPathPrefix(["a", "b"], ["a"])    // false
 * isPathPrefix(["a"], ["b"])         // false
 * ```
 */
export function isPathPrefix(ancestor: string[], path: string[]): boolean {
    return ancestor.length <= path.length
        && ancestor.every((segment, i) => path[i] === segment)
}
