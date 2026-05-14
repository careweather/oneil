/**
 * Derived data atoms: flat parameter lookup, alias-to-model-path map, and the
 * "used" parameter/child sets for graph-view filtering.
 *
 * Bibliography and citation atoms live in `bibliography.ts`.
 * Also exports pure helper functions used alongside these atoms.
 */

import { atom } from "jotai"
import type { ParameterValueAst, RenderedNode, RenderedPoolEntry, RenderedValue } from "../../types/model"
import { buildAliasToModelPath, extractDependencyKeys } from "../../utils/extractDependencies"
import { paramKey } from "../../utils/instancePath"
import { computeUsedParams, GRAPH_USED_PARAMS_MODE, type UsedParamSets } from "../../utils/computeUsedParams"
import { instanceTreeAtom, referencePoolAtom, refPoolAliasesAtom } from "./app"
import { viewModeAtom } from "./display"

// ── Alias map ─────────────────────────────────────────────────────────────────

/** Alias → model_path mapping derived from the instance tree. */
export const aliasToModelPathAtom = atom((get) => {
    const root = get(instanceTreeAtom)
    return root ? buildAliasToModelPath(root) : new Map<string, string>()
})

// ── Parameter lookup ──────────────────────────────────────────────────────────

/** Shape of each entry in the flat parameter lookup map. */
export interface ParamLookupEntry {
    name: string
    renderName: string | null
    label: string
    value: RenderedValue
}

/** Flat map: `paramKey → { name, label, value }` for the whole tree + ref pool. */
export const paramLookupAtom = atom((get) => {
    const root = get(instanceTreeAtom)
    const pool = get(referencePoolAtom)
    return root ? buildParamLookup(root, pool) : new Map<string, ParamLookupEntry>()
})

// ── Full parameter lookup ─────────────────────────────────────────────────────

/** Full parameter data needed to build a `DetailPanelState` for any parameter key. */
export interface FullParamLookupEntry {
    name: string
    renderName: string | null
    label: string
    note: string | null
    value: RenderedValue
    expression: ParameterValueAst | null
    instancePath: string[]
}

/**
 * Builds a flat map of `paramKey → FullParamLookupEntry` by walking the entire
 * main tree depth-first, then each reference-pool entry.
 *
 * Uses unified key format: `instance_path` from the backend already contains
 * the ref alias as the first segment for ref pool entries.
 */
function buildFullParamLookup(
    root: RenderedNode,
    referencePool: RenderedPoolEntry[],
): Map<string, FullParamLookupEntry> {
    const map = new Map<string, FullParamLookupEntry>()

    function walkNode(node: RenderedNode): void {
        for (const p of node.parameters) {
            map.set(paramKey(node.instance_path, p.name), {
                name: p.name,
                renderName: p.render_name,
                label: p.label,
                note: p.note,
                value: p.value,
                expression: p.expression as ParameterValueAst | null,
                instancePath: node.instance_path,
            })
        }
        for (const child of node.children) walkNode(child.node)
    }

    walkNode(root)

    // Ref pool entries: instance_path already includes ref alias as first segment
    for (const entry of referencePool) {
        walkNode(entry.node)
    }

    return map
}

/**
 * Full flat map: `paramKey → FullParamLookupEntry` for the whole tree + ref pool.
 * Used by the detail panel to build a `DetailPanelState` when navigating to a
 * dependency's detail view.
 */
export const fullParamLookupAtom = atom((get) => {
    const root = get(instanceTreeAtom)
    const pool = get(referencePoolAtom)
    return root ? buildFullParamLookup(root, pool) : new Map<string, FullParamLookupEntry>()
})

// ── Reverse dependency lookup ─────────────────────────────────────────────────

/**
 * Derived atom that maps each parameter key to the set of parameter keys that
 * directly depend on it. Built by walking every parameter's expression across
 * the full tree and reference pool.
 *
 * Used by the detail panel to show "Used by" (reverse dependencies).
 */
export const reverseDepsAtom = atom((get) => {
    const fullLookup = get(fullParamLookupAtom)
    const aliasToModelPath = get(aliasToModelPathAtom)
    const refPoolAliases = get(refPoolAliasesAtom)
    const reverseMap = new Map<string, Set<string>>()

    for (const [key, entry] of fullLookup) {
        if (!entry.expression) continue
        const depKeys = extractDependencyKeys(entry.expression, entry.instancePath, aliasToModelPath, refPoolAliases)
        for (const depKey of depKeys) {
            let set = reverseMap.get(depKey)
            if (!set) {
                set = new Set()
                reverseMap.set(depKey, set)
            }
            set.add(key)
        }
    }

    return reverseMap
})

// ── Render-name lookup ────────────────────────────────────────────────────────

/**
 * Derived atom that returns a function for resolving a parameter's render-name
 * given its instance path, parameter name, and optional reference alias.
 *
 * - No `refAlias` → same-instance parameter key via `paramKey(path, name)`.
 * - `refAlias` present in refPoolAliases → ref-pool key via `paramKey([alias], name)`.
 * - `refAlias` absent from set → submodel child key via `paramKey([...path, alias], name)`.
 *
 * Returns `null` when no render-name is recorded (caller falls back to `mathName`).
 */
export const renderNameLookupAtom = atom((get) => {
    const lookup = get(paramLookupAtom)
    const refPoolAliases = get(refPoolAliasesAtom)
    return (instancePath: string[], paramName: string, refAlias?: string): string | null => {
        if (refAlias !== undefined) {
            const key = refPoolAliases.has(refAlias)
                ? paramKey([refAlias], paramName)
                : paramKey([...instancePath, refAlias], paramName)
            return lookup.get(key)?.renderName ?? null
        }
        return lookup.get(paramKey(instancePath, paramName))?.renderName ?? null
    }
})

// ── Used-parameter sets ───────────────────────────────────────────────────────

const EMPTY_USED: UsedParamSets = {
    usedParamKeys: new Set(),
    usedChildPaths: new Set(),
}

/**
 * Which params and child paths are "reachable" from performance-level params
 * and test expressions. Only computed in graph view where unused items are
 * hidden; returns empty sets in tree view.
 */
export const usedParamSetsAtom = atom((get) => {
    const root = get(instanceTreeAtom)
    const view = get(viewModeAtom)
    const aliasMap = get(aliasToModelPathAtom)
    const referencePool = get(referencePoolAtom)
    if (!root || view !== "graph") return EMPTY_USED
    return computeUsedParams(root, aliasMap, {
        mode: GRAPH_USED_PARAMS_MODE,
        referencePool,
    })
})

// ── Helper functions ──────────────────────────────────────────────────────────

/**
 * Builds a flat map of `paramKey → { name, label, value }` by walking the
 * entire main tree depth-first and then each reference-pool entry.
 *
 * Uses unified key format: `instance_path` from the backend already contains
 * the ref alias as the first segment for ref pool entries.
 */
export function buildParamLookup(
    root: RenderedNode,
    referencePool: RenderedPoolEntry[],
): Map<string, ParamLookupEntry> {
    const map = new Map<string, ParamLookupEntry>()

    function walkNode(node: RenderedNode): void {
        for (const p of node.parameters) {
            map.set(paramKey(node.instance_path, p.name), {
                name: p.name,
                renderName: p.render_name,
                label: p.label,
                value: p.value,
            })
        }
        for (const child of node.children) walkNode(child.node)
    }

    walkNode(root)

    // Ref pool entries: instance_path already includes ref alias as first segment
    for (const entry of referencePool) {
        walkNode(entry.node)
    }

    return map
}

/** Builds a `design_name → color_index` map from a node's `applied_designs` list. */
export function buildDesignIndex(
    designs: { design_name: string; color_index: number }[],
): Map<string, number> {
    return new Map(designs.map((d) => [d.design_name, d.color_index]))
}

