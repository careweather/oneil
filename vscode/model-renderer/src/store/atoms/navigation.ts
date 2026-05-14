/**
 * Navigation atoms: the unified focused path for tree drill-down, and the
 * detail panel's current focus with its full back/forward history.
 *
 * The focused path is a simple `string[]` that works for both main tree and
 * reference pool. The first segment determines tree membership: if it's in
 * `refPoolAliasesAtom` → ref tree, else → main tree.
 */

import { atom } from "jotai"
import type { RenderedNode, ParameterValueAst, RenderedValue } from "../../types/model"
import type { DepLabelEntry } from "../../components/ParameterRow"
import { fullTreeAtom, refPoolAliasesAtom } from "./app"

// ── Unified navigation ────────────────────────────────────────────────────────

/**
 * The focused path in the unified full tree. `[]` = main root.
 * Works for both main tree and ref pool — the first segment determines which.
 */
export const focusedPathAtom = atom<string[]>([])

/**
 * Resolves a node from a starting node by walking a path of alias segments.
 * Returns null if any segment doesn't resolve.
 */
function resolveNodeByPath(start: RenderedNode, path: string[]): RenderedNode | null {
    let node = start
    for (const alias of path) {
        const child = node.children.find((c) => c.alias === alias)
        if (!child) return null
        node = child.node
    }
    return node
}

/**
 * The `RenderedNode` at the focused path, or `null` if the path does not
 * resolve. Works for both main tree (path starts with main child alias or is
 * empty) and ref pool (path starts with ref pool alias).
 */
export const focusedNodeAtom = atom<RenderedNode | null>((get) => {
    const path = get(focusedPathAtom)
    const fullTree = get(fullTreeAtom)

    if (path.length === 0) return fullTree.root

    const child = fullTree.allChildren.find((c) => c.alias === path[0])
    if (!child) return null

    return resolveNodeByPath(child.node, path.slice(1))
})

/**
 * True if the focused path is in the reference pool (for visual styling).
 * Derived from the first path segment being in refPoolAliases.
 */
export const isViewingRefAtom = atom((get) => {
    const path = get(focusedPathAtom)
    const refAliases = get(refPoolAliasesAtom)
    return path.length > 0 && refAliases.has(path[0])
})

// ── Detail panel navigation ───────────────────────────────────────────────────

/** Everything needed to display a parameter's equation and its dependencies. */
export interface DetailPanelState {
    /** Unique lookup key for this parameter (used for reverse-dependency queries). */
    key: string
    paramName: string
    /** Optional LaTeX render-name; used instead of auto-deriving from `paramName` when set. */
    paramRenderName: string | null
    paramLabel: string
    note: string | null
    /** Expression AST, or null for literal/simple-value parameters with no formula. */
    expression: ParameterValueAst | null
    value: RenderedValue
    deps: DepLabelEntry[]
    instancePath: string[]
}

/** The equation currently shown in the detail panel, or null when none is selected. */
export const detailPanelAtom = atom<DetailPanelState | null>(null)

/**
 * History stack of previously visited detail panel states (oldest → newest).
 * Populated when navigating forward to a new state; cleared when closing.
 */
export const detailPanelBackStackAtom = atom<DetailPanelState[]>([])

/**
 * Forward stack of states that can be re-visited after pressing Back.
 * Populated when navigating backward; cleared when a new state is pushed.
 */
export const detailPanelForwardStackAtom = atom<DetailPanelState[]>([])

/**
 * Write-only atom for navigating the detail panel with history tracking.
 * Pass a `DetailPanelState` to open/navigate, or `null` to close and clear history.
 *
 * - When navigating to a new state: pushes the current state onto the back stack
 *   and clears the forward stack.
 * - When closing (`null`): clears both stacks.
 */
export const navigateDetailPanelAtom = atom(
    null,
    (get, set, newState: DetailPanelState | null) => {
        if (newState === null) {
            set(detailPanelAtom, null)
            set(detailPanelBackStackAtom, [])
            set(detailPanelForwardStackAtom, [])
        } else {
            const current = get(detailPanelAtom)
            if (current !== null) {
                set(detailPanelBackStackAtom, [...get(detailPanelBackStackAtom), current])
            }
            set(detailPanelForwardStackAtom, [])
            set(detailPanelAtom, newState)
        }
    },
)

/**
 * Write-only atom that navigates back one step in the detail panel history.
 * Moves the current state onto the forward stack and restores the previous one.
 * No-op when the back stack is empty.
 */
export const navigateDetailBackAtom = atom(null, (get, set) => {
    const backStack = get(detailPanelBackStackAtom)
    if (backStack.length === 0) return
    const current = get(detailPanelAtom)
    const prev = backStack[backStack.length - 1]
    set(detailPanelBackStackAtom, backStack.slice(0, -1))
    if (current !== null) {
        set(detailPanelForwardStackAtom, [...get(detailPanelForwardStackAtom), current])
    }
    set(detailPanelAtom, prev)
})

/**
 * Write-only atom that navigates forward one step in the detail panel history.
 * Moves the current state onto the back stack and restores the next one.
 * No-op when the forward stack is empty.
 */
export const navigateDetailForwardAtom = atom(null, (get, set) => {
    const forwardStack = get(detailPanelForwardStackAtom)
    if (forwardStack.length === 0) return
    const current = get(detailPanelAtom)
    const next = forwardStack[forwardStack.length - 1]
    set(detailPanelForwardStackAtom, forwardStack.slice(0, -1))
    if (current !== null) {
        set(detailPanelBackStackAtom, [...get(detailPanelBackStackAtom), current])
    }
    set(detailPanelAtom, next)
})

/**
 * Key of the parameter to temporarily flash after a "jump to" action.
 * Set to the key just before scrolling; components clear it after the
 * animation completes.
 */
export const focusedParamKeyAtom = atom<string | null>(null)
