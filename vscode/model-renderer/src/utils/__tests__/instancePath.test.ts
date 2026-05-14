import { describe, expect, it } from "vitest"
import { isPathPrefix, pathKey, pathsEqual, pathToNodeId } from "../instancePath"

describe("pathKey", () => {
    it("returns empty string for root", () => {
        expect(pathKey([])).toBe("")
    })
    it("returns the segment for a single-element path", () => {
        expect(pathKey(["rover"])).toBe("rover")
    })
    it("joins segments with /", () => {
        expect(pathKey(["rover", "arm"])).toBe("rover/arm")
    })
})

describe("pathToNodeId", () => {
    it("returns __root__ for empty path", () => {
        expect(pathToNodeId([])).toBe("__root__")
    })
    it("returns slash-joined path otherwise", () => {
        expect(pathToNodeId(["rover", "arm"])).toBe("rover/arm")
    })
})

describe("pathsEqual", () => {
    it("returns true for identical paths", () => {
        expect(pathsEqual(["a", "b"], ["a", "b"])).toBe(true)
    })
    it("returns true for two empty paths", () => {
        expect(pathsEqual([], [])).toBe(true)
    })
    it("returns false when lengths differ", () => {
        expect(pathsEqual(["a"], ["a", "b"])).toBe(false)
    })
    it("returns false when segments differ", () => {
        expect(pathsEqual(["a", "b"], ["a", "c"])).toBe(false)
    })
})

describe("isPathPrefix", () => {
    it("root is a prefix of everything", () => {
        expect(isPathPrefix([], ["a", "b"])).toBe(true)
        expect(isPathPrefix([], [])).toBe(true)
    })
    it("path is a prefix of itself (non-strict)", () => {
        expect(isPathPrefix(["a", "b"], ["a", "b"])).toBe(true)
    })
    it("returns true for a proper prefix", () => {
        expect(isPathPrefix(["a"], ["a", "b"])).toBe(true)
    })
    it("returns false when the candidate is longer", () => {
        expect(isPathPrefix(["a", "b"], ["a"])).toBe(false)
    })
    it("returns false for a different branch", () => {
        expect(isPathPrefix(["a"], ["b"])).toBe(false)
    })
})
