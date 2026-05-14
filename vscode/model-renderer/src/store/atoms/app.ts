/**
 * App-level state atoms: the LSP evaluation result, loading status, and the
 * primary accessors for the rendered tree and reference pool.
 */

import { atom } from "jotai"
import type { RenderedNode, RenderedPoolEntry, RenderedTree } from "../../types/model"

export type AppState =
    | { status: "loading" }
    | { status: "ready"; data: RenderedTree }
    | { status: "error"; message: string }

/** The current evaluation result from the language server. */
export const appStateAtom = atom<AppState>({ status: "loading" })

/**
 * Raw BibTeX content from workspace `.bib` file(s), as sent by the
 * extension alongside each tree refresh. `null` when no file was found.
 */
export const bibliographyRawAtom = atom<string | null>(null)

/**
 * The webview-accessible URI for the workspace root, as computed by the
 * extension via `webview.asWebviewUri(workspaceRoot)`. Used as the primary
 * base for resolving relative image paths in notes (e.g. `![](./img/foo.png)`).
 * `null` when no workspace folder is open or no URI has been received yet.
 */
export const workspaceUriAtom = atom<string | null>(null)

/**
 * The webview-accessible URI for the directory containing the currently
 * rendered file. Used as a fallback image base when a path doesn't resolve
 * relative to the workspace root.
 * `null` when no file URI has been received yet.
 */
export const fileBaseUriAtom = atom<string | null>(null)

/**
 * The webview-accessible URI for the user's PDF cache directory, sent by the
 * extension via `webview.asWebviewUri(cacheDirUri)`. Used to construct inline
 * PDF URLs for cached files stored as bare filenames in `references.bib`
 * (e.g. `pdfCacheUri + "/nasa-std-3001_abc123.pdf"`).
 * `null` until the first `instanceTree` message is received.
 */
export const pdfCacheUriAtom = atom<string | null>(null)

/** True while waiting for the LSP response. */
export const isLoadingAtom = atom((get) => get(appStateAtom).status === "loading")

/** The main instance tree root, or null when not yet loaded. */
export const instanceTreeAtom = atom<RenderedNode | null>((get) => {
    const s = get(appStateAtom)
    return s.status === "ready" ? s.data.root : null
})

/** The reference pool (imported models), or an empty array when not loaded. */
export const referencePoolAtom = atom<RenderedPoolEntry[]>((get) => {
    const s = get(appStateAtom)
    return s.status === "ready" ? s.data.reference_pool : []
})

// ── Full tree (unified main + refs) ───────────────────────────────────────────

/** A child node in the unified full tree view. */
export interface FullTreeChild {
    alias: string
    node: RenderedNode
    /** True for reference pool entries (for visual distinction). */
    isRef: boolean
}

/** The unified full tree with main-tree children + refs as siblings. */
export interface FullTree {
    root: RenderedNode | null
    /** Main-tree children + ref pool entries, treated as siblings. */
    allChildren: FullTreeChild[]
}

/**
 * Unified tree that merges main-tree children and reference pool entries.
 * Navigation uses paths where the first component determines the subtree:
 * if it's a ref pool alias → ref, else → main tree child.
 */
export const fullTreeAtom = atom<FullTree>((get) => {
    const root = get(instanceTreeAtom)
    const pool = get(referencePoolAtom)

    const mainChildren = (root?.children ?? []).map((c) => ({
        alias: c.alias,
        node: c.node,
        isRef: false,
    }))

    const refChildren = pool.map((e) => ({
        alias: e.alias,
        node: e.node,
        isRef: true,
    }))

    return {
        root,
        allChildren: [...mainChildren, ...refChildren],
    }
})

/** Set of reference pool aliases for quick tree-membership checks. */
export const refPoolAliasesAtom = atom<Set<string>>((get) => {
    const pool = get(referencePoolAtom)
    return new Set(pool.map((e) => e.alias))
})
