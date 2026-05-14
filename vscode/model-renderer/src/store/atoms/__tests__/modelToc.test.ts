import { describe, expect, it } from "vitest"
import { collectTocEntries } from "../modelToc"
import type { RenderedNode } from "../../../types/model"

function node(partial: Partial<RenderedNode> & Pick<RenderedNode, "model_path">): RenderedNode {
    return {
        instance_path: [],
        note: null,
        parameters: [],
        tests: [],
        sections: [],
        children: [],
        references: [],
        applied_designs: [],
        ...partial,
    }
}

describe("collectTocEntries", () => {
    it("lists root then children depth-first and uses model display name at root", () => {
        const tree = node({
            model_path: "models/foo.on",
            children: [{ alias: "a", node: node({ model_path: "child.on" }) }],
        })
        const rows = collectTocEntries(tree, [], 0)
        expect(rows.map((r) => r.label)).toEqual(["foo", "a"])
        expect(rows.map((r) => r.path)).toEqual([[], ["a"]])
        expect(rows.map((r) => r.depth)).toEqual([0, 1])
    })

    it("uses rootLabel for the root row when provided", () => {
        const tree = node({
            model_path: "models/foo.on",
            children: [{ alias: "a", node: node({ model_path: "child.on" }) }],
        })
        const rows = collectTocEntries(tree, [], 0, "refAlias")
        expect(rows[0].label).toBe("refAlias")
        expect(rows[1].label).toBe("a")
    })
})
