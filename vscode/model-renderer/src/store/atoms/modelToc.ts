/**
 * Derived TOC data for the model hierarchy panel: flattens main and reference
 * trees into depth-first row lists.
 */
import { atom } from "jotai"
import type { AppliedDesign, RenderedNode } from "../../types/model"
import { modelDisplayName } from "../../utils/modelPath"
import { instanceTreeAtom, referencePoolAtom } from "./app"

/** One row in the flat table-of-contents list. */
export interface TocEntry {
    /** Alias path segments from subtree root (empty = root of that subtree). */
    path: string[]
    /** Primary label for the row. */
    label: string
    /** Model type name (secondary annotation). */
    modelName: string
    /** Nesting level for indentation. */
    depth: number
    /** Designs applied to this node. */
    designs: AppliedDesign[]
}

/**
 * Flattens a `RenderedNode` tree into {@link TocEntry} rows in depth-first order.
 *
 * @param rootLabel - When set, used as the root row’s label (reference pool alias).
 *
 * @example
 * ```ts
 * const root: RenderedNode = { ... }
 * const rows = collectTocEntries(root, [], 0)
 * // rows[0] is the root; deeper nodes follow in tree order
 * ```
 */
export function collectTocEntries(
    node: RenderedNode,
    path: string[],
    depth: number,
    rootLabel?: string,
): TocEntry[] {
    const isRoot = path.length === 0
    const entries: TocEntry[] = [
        {
            path,
            label: rootLabel ?? (isRoot ? modelDisplayName(node.model_path) : path[path.length - 1]),
            modelName: modelDisplayName(node.model_path),
            depth,
            designs: node.applied_designs,
        },
    ]
    for (const child of node.children) {
        entries.push(...collectTocEntries(child.node, [...path, child.alias], depth + 1))
    }
    return entries
}

/** One reference pool import, with its subtree flattened for the TOC. */
export interface ReferenceTocSection {
    alias: string
    entries: TocEntry[]
}

/**
 * Main instance tree and reference-import TOC rows derived from the current model.
 * `null` when there is no instance root yet.
 */
export const modelTocAtom = atom((get) => {
    const root = get(instanceTreeAtom)
    if (!root) return null
    const pool = get(referencePoolAtom)
    const mainEntries = collectTocEntries(root, [], 0)
    const referenceSections: ReferenceTocSection[] = pool.map((e) => ({
        alias: e.alias,
        entries: collectTocEntries(e.node, [], 0, e.alias),
    }))
    return { mainEntries, referenceSections }
})
