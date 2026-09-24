import { describe, expect, it } from "vitest"
import {
    flavorsForPlatform,
    isPythonFlavor,
    isPythonMinor,
    missingPythonHint,
    pythonLaunchEnv,
} from "../python"

describe("python flavors", () => {
    it("publishes Homebrew only on macOS", () => {
        expect(flavorsForPlatform("darwin")).toEqual(["homebrew", "uv", "system"])
        expect(flavorsForPlatform("win32")).toEqual(["uv", "system"])
        expect(flavorsForPlatform("linux")).toEqual(["uv", "system"])
    })

    it("accepts known flavor names", () => {
        expect(isPythonFlavor("homebrew")).toBe(true)
        expect(isPythonFlavor("uv")).toBe(true)
        expect(isPythonFlavor("system")).toBe(true)
        expect(isPythonFlavor("pyenv")).toBe(false)
        expect(isPythonFlavor(undefined)).toBe(false)
        expect(isPythonMinor("3.12")).toBe(true)
        expect(isPythonMinor("3.14")).toBe(true)
        expect(isPythonMinor("3.13")).toBe(false)
    })

    it("mentions Python 3.12 and 3.14", () => {
        expect(missingPythonHint("darwin")).toContain("uv python install 3.14")
        expect(missingPythonHint("win32")).toContain("uv python install 3.14")
        expect(missingPythonHint("linux")).toContain("python3.14")
    })
})

describe("pythonLaunchEnv", () => {
    it("is omitted when the binary bakes an absolute path", () => {
        expect(pythonLaunchEnv({ flavor: "homebrew" })).toBeUndefined()
        expect(pythonLaunchEnv(undefined)).toBeUndefined()
    })

    it("prepends the uv library dir to the loader path", () => {
        const env = pythonLaunchEnv(
            { flavor: "uv", libraryDir: "/tmp/uv-lib" },
            { PATH: "/usr/bin", DYLD_LIBRARY_PATH: "/already", LD_LIBRARY_PATH: "/already" },
        )
        expect(env).toBeDefined()
        if (process.platform === "darwin") {
            expect(env?.DYLD_LIBRARY_PATH).toBe("/tmp/uv-lib:/already")
        } else if (process.platform === "linux") {
            expect(env?.LD_LIBRARY_PATH).toBe("/tmp/uv-lib:/already")
        } else if (process.platform === "win32") {
            expect(env?.PATH?.startsWith("/tmp/uv-lib;")).toBe(true)
        }
    })
})
