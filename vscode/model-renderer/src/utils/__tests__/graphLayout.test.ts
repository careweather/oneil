import { describe, it, expect } from "vitest"
import { pathToNodeId } from "../instancePath"
import {
    flattenNodes,
    computeSize,
    buildElements,
    LEAF_MIN_W,
    HEADER_H_FALLBACK,
    PARAM_ROW_H_FALLBACK,
    PADDING,
    H_GAP,
} from "../graphLayout"
import type { RenderedNode } from "../../types/model"

// ── Fixtures ──────────────────────────────────────────────────────────────────

function makeNode(
    name: string,
    instancePath: string[],
    numParams = 0,
    children: { alias: string; node: RenderedNode }[] = [],
): RenderedNode {
    return {
        model_path: `models/${name}.on`,
        instance_path: instancePath,
        parameters: Array.from({ length: numParams }, (_, i) => ({
            name: `p${i}`,
            label: `Param ${i}`,
            render_name: null,
            section: null,
            note: null,
            print_level: "none" as const,
            value: { type: "number" as const, value: i, max: null },
            expression: null,
            expr_span: { file: null, start: 0, end: 0 },
            design: null,
        })),
        tests: [],
        sections: [],
        children,
        applied_designs: [],
        note: null,
        references: [],
    }
}

// ── pathToNodeId ──────────────────────────────────────────────────────────────

describe("pathToNodeId", () => {
    it("returns __root__ for empty instance_path", () => {
        expect(pathToNodeId([])).toBe("__root__")
    })

    it("joins path segments with /", () => {
        expect(pathToNodeId(["stage1", "engine"])).toBe("stage1/engine")
    })
})

// ── flattenNodes ──────────────────────────────────────────────────────────────

describe("flattenNodes", () => {
    it("returns single node for a leaf", () => {
        const node = makeNode("leaf", [])
        expect(flattenNodes(node)).toHaveLength(1)
    })

    it("includes all descendants depth-first", () => {
        const child1 = makeNode("c1", ["c1"])
        const child2 = makeNode("c2", ["c2"])
        const root = makeNode("root", [], 0, [
            { alias: "c1", node: child1 },
            { alias: "c2", node: child2 },
        ])
        const flat = flattenNodes(root)
        expect(flat).toHaveLength(3)
        expect(flat[0]).toBe(root)
        expect(flat[1]).toBe(child1)
        expect(flat[2]).toBe(child2)
    })
})

// ── computeSize ───────────────────────────────────────────────────────────────

describe("computeSize", () => {
    it("leaf node uses fallback height based on param count", () => {
        const node = makeNode("leaf", [], 3)
        const cache = new Map()
        const size = computeSize(node, cache, null)
        const expectedH = HEADER_H_FALLBACK + 3 * PARAM_ROW_H_FALLBACK + PADDING
        expect(size.height).toBe(expectedH)
        expect(size.width).toBe(LEAF_MIN_W)
    })

    it("leaf node uses measured height when available", () => {
        const node = makeNode("leaf", [], 0)
        const id = pathToNodeId(node.instance_path)
        const cache = new Map()
        const sizes = new Map([[id, { width: 400, height: 80 }]])
        const size = computeSize(node, cache, sizes)
        expect(size.height).toBe(80 + PADDING)
        expect(size.width).toBe(400)
    })

    it("caches result on second call", () => {
        const node = makeNode("leaf", [], 2)
        const cache = new Map()
        const s1 = computeSize(node, cache, null)
        const s2 = computeSize(node, cache, null)
        expect(s1).toBe(s2)
    })

    it("group node incorporates child sizes", () => {
        const child = makeNode("child", ["child"], 1)
        const root = makeNode("root", [], 0, [{ alias: "child", node: child }])
        const cache = new Map()
        const size = computeSize(root, cache, null)
        const childSize = cache.get("child")!
        expect(size.width).toBeGreaterThanOrEqual(childSize.width + PADDING * 2)
        expect(size.height).toBeGreaterThan(childSize.height)
    })
})

// ── buildElements ─────────────────────────────────────────────────────────────

describe("buildElements", () => {
    it("single leaf → one ReactFlow node at origin", () => {
        const root = makeNode("root", [])
        const nodes = buildElements(root, null)
        expect(nodes).toHaveLength(1)
        expect(nodes[0].position).toEqual({ x: 0, y: 0 })
        expect(nodes[0].type).toBe("leafModel")
    })

    it("group node has type groupModel", () => {
        const child = makeNode("child", ["child"])
        const root = makeNode("root", [], 0, [{ alias: "child", node: child }])
        const nodes = buildElements(root, null)
        const rootNode = nodes.find((n) => n.id === "__root__")
        expect(rootNode?.type).toBe("groupModel")
    })

    it("child node is nested inside parent (parentNode set)", () => {
        const child = makeNode("child", ["child"])
        const root = makeNode("root", [], 0, [{ alias: "child", node: child }])
        const nodes = buildElements(root, null)
        const childNode = nodes.find((n) => n.id === "child")
        expect(childNode?.parentNode).toBe("__root__")
    })

    it("two children are placed side-by-side (x differs)", () => {
        const c1 = makeNode("c1", ["c1"])
        const c2 = makeNode("c2", ["c2"])
        const root = makeNode("root", [], 0, [
            { alias: "c1", node: c1 },
            { alias: "c2", node: c2 },
        ])
        const nodes = buildElements(root, null)
        const n1 = nodes.find((n) => n.id === "c1")!
        const n2 = nodes.find((n) => n.id === "c2")!
        expect(n2.position.x).toBeGreaterThan(n1.position.x)
        expect(n2.position.y).toBe(n1.position.y)
    })

    it("third child wraps to new row (y differs from first two)", () => {
        const children = Array.from({ length: 3 }, (_, i) => ({
            alias: `c${i}`,
            node: makeNode(`c${i}`, [`c${i}`]),
        }))
        const root = makeNode("root", [], 0, children)
        const nodes = buildElements(root, null)
        const n0 = nodes.find((n) => n.id === "c0")!
        const n2 = nodes.find((n) => n.id === "c2")!
        expect(n2.position.y).toBeGreaterThan(n0.position.y)
    })

    it("idPrefix is prepended to all node ids", () => {
        const root = makeNode("root", [])
        const nodes = buildElements(root, null, "__refpool__alias/")
        expect(nodes[0].id).toBe("__refpool__alias/__root__")
    })

    it("two children are spaced by H_GAP", () => {
        const c1 = makeNode("c1", ["c1"])
        const c2 = makeNode("c2", ["c2"])
        const root = makeNode("root", [], 0, [
            { alias: "c1", node: c1 },
            { alias: "c2", node: c2 },
        ])
        const nodes = buildElements(root, null)
        const n1 = nodes.find((n) => n.id === "c1")!
        const n2 = nodes.find((n) => n.id === "c2")!
        const c1Width = (n1.style?.width as number) ?? 0
        expect(n2.position.x).toBe(n1.position.x + c1Width + H_GAP)
    })
})
