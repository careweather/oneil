/**
 * Computes which parameters are "used" for graph view when hiding unused rows.
 *
 * **Transitive mode** — A parameter is used if it is reachable by BFS from
 * root-level performance outputs and root tests, following expression
 * dependencies across the whole tree (and reference-pool roots).
 *
 * **Direct-submodel mode** — Same reachability seed, but parameters whose
 * instance path is non-empty are kept only if some expression **outside** their
 * instance subtree references them directly (typical parent → child external
 * refs). Every parameter on the root model (`instance_path.length === 0`) is
 * always shown regardless of reachability.
 */

import type { ParameterValueAst, RenderedNode, RenderedPoolEntry } from "../types/model"
import {
    extractDependencyKeys,
    extractDepsFromExpr,
} from "./extractDependencies"
import { isPathPrefix, paramKey } from "./instancePath"

export interface UsedParamSets {
    /** Keys of every parameter shown when "hide unused" is on in graph view. */
    usedParamKeys: Set<string>
    /**
     * Slash-joined instance path strings of every submodel that contains at least
     * one used parameter anywhere in its subtree.
     * e.g. `"rover_a"`, `"sol"`, `"sol/earth"`.
     */
    usedChildPaths: Set<string>
}

/** How graph view decides which non-root parameters stay visible. */
export type GraphUsedParamsMode = "transitive" | "direct_submodel"

/**
 * Toggle graph-view filtering here until a UI preference exists.
 * - `"transitive"` — every parameter on the dependency chain under a submodel.
 * - `"direct_submodel"` — only parameters referenced from outside that submodel's instance subtree.
 */
export const GRAPH_USED_PARAMS_MODE: GraphUsedParamsMode = "direct_submodel"

type ParamEntry = { expression: ParameterValueAst | null; instancePath: string[] }

/**
 * Dependency `depKey` is directly referenced from outside its owner's instance
 * subtree iff the referencing expression sits at `exprInstancePath` such that
 * the owner path is **not** a prefix of `exprInstancePath`.
 */
function noteDirectExternalDeps(
    depKeys: Iterable<string>,
    exprInstancePath: string[],
    paramIndex: Map<string, ParamEntry>,
    into: Set<string>,
): void {
    for (const depKey of depKeys) {
        const owner = paramIndex.get(depKey)
        if (!owner) continue
        if (!isPathPrefix(owner.instancePath, exprInstancePath)) {
            into.add(depKey)
        }
    }
}

function collectDirectExternalRefs(
    paramIndex: Map<string, ParamEntry>,
    root: RenderedNode,
    referencePool: RenderedPoolEntry[],
    aliasToModelPath: Map<string, string>,
): Set<string> {
    const direct = new Set<string>()

    function walk(node: RenderedNode): void {
        const ip = node.instance_path
        for (const p of node.parameters) {
            noteDirectExternalDeps(
                extractDependencyKeys(p.expression as ParameterValueAst | null, ip, aliasToModelPath),
                ip,
                paramIndex,
                direct,
            )
        }
        for (const test of node.tests ?? []) {
            noteDirectExternalDeps(
                extractDepsFromExpr(test.expression, ip, aliasToModelPath),
                ip,
                paramIndex,
                direct,
            )
        }
        for (const child of node.children) {
            walk(child.node)
        }
    }

    walk(root)
    for (const entry of referencePool) {
        walk(entry.node)
    }

    return direct
}

function buildParamIndex(root: RenderedNode, referencePool: RenderedPoolEntry[]): Map<string, ParamEntry> {
    const paramIndex = new Map<string, ParamEntry>()

    function indexNode(node: RenderedNode): void {
        const ip = node.instance_path
        for (const p of node.parameters) {
            paramIndex.set(paramKey(ip, p.name), {
                expression: p.expression as ParameterValueAst | null,
                instancePath: ip,
            })
        }
        for (const child of node.children) {
            indexNode(child.node)
        }
    }

    indexNode(root)

    // Ref pool entries: instance_path already includes ref alias as first segment
    for (const entry of referencePool) {
        indexNode(entry.node)
    }

    return paramIndex
}

function deriveUsedChildPaths(usedParamKeys: Set<string>): Set<string> {
    const usedChildPaths = new Set<string>()
    for (const key of usedParamKeys) {
        const parts = key.split("/")
        // Include all ancestor paths (excluding the root)
        for (let i = 1; i < parts.length; i++) {
            usedChildPaths.add(parts.slice(0, i).join("/"))
        }
    }
    return usedChildPaths
}

/**
 * Applies {@link GRAPH_USED_PARAMS_MODE}: non-root params need a direct external
 * reference; root-model params are always included.
 */
function filterDirectSubmodelParams(
    transitiveUsed: Set<string>,
    directExternal: Set<string>,
    paramIndex: Map<string, ParamEntry>,
): Set<string> {
    const filtered = new Set<string>()

    for (const [key, entry] of paramIndex) {
        if (entry.instancePath.length === 0) {
            filtered.add(key)
        }
    }

    for (const key of transitiveUsed) {
        const entry = paramIndex.get(key)
        if (!entry) {
            filtered.add(key)
            continue
        }
        if (entry.instancePath.length === 0 || directExternal.has(key)) {
            filtered.add(key)
        }
    }

    return filtered
}

export interface ComputeUsedParamsOptions {
    /** Defaults to `"transitive"` when omitted. */
    mode?: GraphUsedParamsMode
    referencePool?: RenderedPoolEntry[]
}

/**
 * Computes used parameter and submodel sets for the given root node.
 *
 * @param root              The top-level `RenderedNode` of the instance tree.
 * @param aliasToModelPath  Mapping from reference alias → model_path (cross-file
 *                          references only — built by `buildAliasToModelPath`).
 * @param options           Optional mode switch and reference pool for indexing / scanning.
 */
export function computeUsedParams(
    root: RenderedNode,
    aliasToModelPath: Map<string, string>,
    options?: ComputeUsedParamsOptions,
): UsedParamSets {
    const mode = options?.mode ?? "transitive"
    const referencePool = options?.referencePool ?? []

    const paramIndex = buildParamIndex(root, referencePool)

    const used = new Set<string>()
    const queue: string[] = []

    const enqueue = (key: string): void => {
        if (!used.has(key)) {
            used.add(key)
            queue.push(key)
        }
    }

    for (const p of root.parameters) {
        if (p.print_level === "performance") {
            enqueue(paramKey([], p.name))
        }
    }

    // Seed from root test expression dependencies.
    for (const test of root.tests ?? []) {
        for (const dep of extractDepsFromExpr(test.expression, [], aliasToModelPath)) {
            enqueue(dep)
        }
    }

    // ── Step 3: BFS over expression dependencies ─────────────────────────────
    // Use index-based iteration to avoid O(n) shift() calls
    let head = 0
    while (head < queue.length) {
        const key = queue[head++]
        const entry = paramIndex.get(key)
        if (!entry) continue
        for (const dep of extractDependencyKeys(entry.expression, entry.instancePath, aliasToModelPath)) {
            enqueue(dep)
        }
    }

    let usedParamKeys: Set<string>
    if (mode === "direct_submodel") {
        const directExternal = collectDirectExternalRefs(paramIndex, root, referencePool, aliasToModelPath)
        usedParamKeys = filterDirectSubmodelParams(used, directExternal, paramIndex)
    } else {
        usedParamKeys = used
    }

    return {
        usedParamKeys,
        usedChildPaths: deriveUsedChildPaths(usedParamKeys),
    }
}
