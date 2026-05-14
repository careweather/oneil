import { describe, expect, it } from "vitest"
import { modelDisplayName } from "../modelPath"

describe("modelDisplayName", () => {
    it("strips .on and .one extensions from the final path segment", () => {
        expect(modelDisplayName("/path/to/vehicle.on")).toBe("vehicle")
        expect(modelDisplayName("/path/to/overlay.one")).toBe("overlay")
        expect(modelDisplayName("engine")).toBe("engine")
    })
})
