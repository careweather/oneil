/**
 * Pure layout-computation utilities for the model graph view.
 *
 * All functions operate only on `RenderedNode` data and numeric constants; they
 * have no React, ReactFlow, or Jotai dependencies so they can be unit-tested
 * in a plain Node environment.
 *
 * `ModelGraph.tsx` imports these functions and then wraps the resulting sizes
 * and positions into ReactFlow node objects.
 */
import type { Node } from "reactflow"
import type { RenderedNode, RenderedPoolEntry } from "../types/model"
import type { ContentSize } from "../components/MeasureContent"
import { pathToNodeId } from "./instancePath"

// ── Layout constants ──────────────────────────────────────────────────────────

/** Horizontal gap between sibling nodes inside a group. */
export const H_GAP = 12
/** Vertical gap between rows of submodels. */
export const V_GAP = 12
/** Padding inside a group node (sides and bottom). */
export const PADDING = 16
/** Maximum number of submodel columns in a group node. */
export const MAX_SUBMODEL_COLS = 2
/**
 * Fallback header height when no measurement is available yet.
 * Deliberately generous so the initial render is more likely to be too big
 * than too small, reducing the visual jump on the first measurement pass.
 */
export const HEADER_H_FALLBACK = 64
/**
 * Fallback per-parameter row height before measurement.
 * Also generous for the same reason.
 */
export const PARAM_ROW_H_FALLBACK = 36
/** Minimum node width — content is rendered at this width during measurement. */
export const LEAF_MIN_W = 340

// ── Types ─────────────────────────────────────────────────────────────────────

export interface Size {
    width: number
    height: number
}

export interface ModelNodeData {
    node: RenderedNode
    alias: string | null
}

export type FlowNode = Node<ModelNodeData>

// ── Helpers ───────────────────────────────────────────────────────────────────


/** Flattens a `RenderedNode` tree into a depth-first list. */
export function flattenNodes(node: RenderedNode): RenderedNode[] {
    return [node, ...node.children.flatMap((c) => flattenNodes(c.node))]
}

// ── Size computation ──────────────────────────────────────────────────────────

/**
 * Computes the pixel size required to render `node` and all its descendants
 * as nested boxes. `contentSizes` supplies accurately-measured content
 * dimensions; when absent the fallback constants are used.
 */
export function computeSize(
    node: RenderedNode,
    cache: Map<string, Size>,
    contentSizes: Map<string, ContentSize> | null,
): Size {
    const id = pathToNodeId(node.instance_path)
    const cached = cache.get(id)
    if (cached) return cached

    for (const child of node.children) computeSize(child.node, cache, contentSizes)

    const measured = contentSizes?.get(id)
    const contentH =
        measured?.height ??
        HEADER_H_FALLBACK + node.parameters.length * PARAM_ROW_H_FALLBACK
    const contentW = measured?.width ?? LEAF_MIN_W

    let size: Size
    if (node.children.length === 0) {
        size = { width: contentW, height: contentH + PADDING }
    } else {
        const childSizes = node.children.map((c) => cache.get(pathToNodeId(c.node.instance_path))!)
        const numCols = Math.min(childSizes.length, MAX_SUBMODEL_COLS)
        const numRows = Math.ceil(childSizes.length / MAX_SUBMODEL_COLS)

        const colWidths: number[] = []
        for (let col = 0; col < numCols; col++) {
            let maxW = 0
            for (let row = 0; row < numRows; row++) {
                const idx = row * MAX_SUBMODEL_COLS + col
                if (idx < childSizes.length) maxW = Math.max(maxW, childSizes[idx].width)
            }
            colWidths.push(maxW)
        }

        const rowHeights: number[] = []
        for (let row = 0; row < numRows; row++) {
            let maxH = 0
            for (let col = 0; col < numCols; col++) {
                const idx = row * MAX_SUBMODEL_COLS + col
                if (idx < childSizes.length) maxH = Math.max(maxH, childSizes[idx].height)
            }
            rowHeights.push(maxH)
        }

        const gridW = colWidths.reduce((s, w) => s + w, 0) + (numCols - 1) * H_GAP
        const gridH = rowHeights.reduce((s, h) => s + h, 0) + (numRows - 1) * V_GAP

        size = {
            width: Math.max(contentW, gridW + PADDING * 2),
            height: contentH + PADDING + gridH + PADDING,
        }
    }

    cache.set(id, size)
    return size
}

// ── Node placement ────────────────────────────────────────────────────────────

/**
 * Recursively builds the flat ReactFlow node list.
 * Parent nodes must be pushed before their children (ReactFlow requirement).
 * Child positions are relative to their parent's top-left corner.
 */
export function placeNodes(
    node: RenderedNode,
    alias: string | null,
    parentId: string | null,
    x: number,
    y: number,
    sizes: Map<string, Size>,
    contentSizes: Map<string, ContentSize> | null,
    idPrefix: string,
    out: FlowNode[],
): void {
    const baseId = pathToNodeId(node.instance_path)
    const id = idPrefix + baseId
    const size = sizes.get(baseId)!
    const isGroup = node.children.length > 0

    const flowNode: FlowNode = {
        id,
        position: { x, y },
        data: { node, alias },
        style: { width: size.width, height: size.height },
        type: isGroup ? "groupModel" : "leafModel",
        ...(parentId !== null ? { parentNode: parentId, extent: "parent" as const } : {}),
    }
    out.push(flowNode)

    if (isGroup) {
        const measuredContentH = contentSizes?.get(baseId)?.height
        const contentH =
            measuredContentH ??
            HEADER_H_FALLBACK + node.parameters.length * PARAM_ROW_H_FALLBACK

        const childSizes = node.children.map((c) => sizes.get(pathToNodeId(c.node.instance_path))!)
        const numCols = Math.min(childSizes.length, MAX_SUBMODEL_COLS)
        const numRows = Math.ceil(childSizes.length / MAX_SUBMODEL_COLS)

        const colWidths: number[] = []
        for (let col = 0; col < numCols; col++) {
            let maxW = 0
            for (let row = 0; row < numRows; row++) {
                const idx = row * MAX_SUBMODEL_COLS + col
                if (idx < childSizes.length) maxW = Math.max(maxW, childSizes[idx].width)
            }
            colWidths.push(maxW)
        }

        const rowHeights: number[] = []
        for (let row = 0; row < numRows; row++) {
            let maxH = 0
            for (let col = 0; col < numCols; col++) {
                const idx = row * MAX_SUBMODEL_COLS + col
                if (idx < childSizes.length) maxH = Math.max(maxH, childSizes[idx].height)
            }
            rowHeights.push(maxH)
        }

        let currentY = contentH + PADDING
        for (let row = 0; row < numRows; row++) {
            let currentX = PADDING
            for (let col = 0; col < numCols; col++) {
                const idx = row * MAX_SUBMODEL_COLS + col
                if (idx < node.children.length) {
                    const child = node.children[idx]
                    placeNodes(child.node, child.alias, id, currentX, currentY, sizes, contentSizes, idPrefix, out)
                }
                currentX += colWidths[col] + H_GAP
            }
            currentY += rowHeights[row] + V_GAP
        }
    }
}

/**
 * Builds the complete flat ReactFlow node list from the instance tree.
 * `contentSizes` is `null` on the very first render (before any measurements
 * have arrived) and the fallback constants are used instead.
 */
export function buildElements(
    root: RenderedNode,
    contentSizes: Map<string, ContentSize> | null,
    idPrefix = "",
): FlowNode[] {
    const sizes = new Map<string, Size>()
    computeSize(root, sizes, contentSizes)
    const nodes: FlowNode[] = []
    placeNodes(root, null, null, 0, 0, sizes, contentSizes, idPrefix, nodes)
    return nodes
}

// ── Reference pool layout ─────────────────────────────────────────────────────

/** Prefix for ref pool ReactFlow node IDs to distinguish from main tree nodes. */
export const REF_POOL_ID_PREFIX = "__refpool__"

/**
 * Builds the ReactFlow node list for all reference pool entries.
 * Each entry is placed to the right of the main tree, stacked vertically.
 */
export function buildRefPoolElements(
    referencePool: RenderedPoolEntry[],
    mainTreeWidth: number,
    contentSizes: Map<string, ContentSize> | null,
): FlowNode[] {
    const allRefNodes: FlowNode[] = []
    const refStartX = mainTreeWidth + PADDING * 3
    let currentY = 0

    for (const entry of referencePool) {
        const idPrefix = `${REF_POOL_ID_PREFIX}${entry.alias}/`
        const refContentSizes = new Map<string, ContentSize>()
        if (contentSizes) {
            for (const n of flattenNodes(entry.node)) {
                const key = `${idPrefix}${pathToNodeId(n.instance_path)}`
                const size = contentSizes.get(key)
                if (size) refContentSizes.set(pathToNodeId(n.instance_path), size)
            }
        }
        const entryNodes = buildElements(
            entry.node,
            refContentSizes,
            idPrefix,
        )
        const entryRoot = entryNodes.find((n) => n.parentId == null)
        if (entryRoot) {
            entryRoot.position = { x: refStartX, y: currentY }
            const entryHeight = (entryRoot.style?.height as number | undefined) ?? 0
            currentY += entryHeight + PADDING * 2
        }
        allRefNodes.push(...entryNodes)
    }

    return allRefNodes
}
