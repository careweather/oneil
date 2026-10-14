import { describe, expect, it } from "vitest"
import {
    cliArchiveName,
    cliAssetCandidates,
    cliUnflavoredArchiveName,
    resolveCliPlatform,
} from "../platforms"

describe("resolveCliPlatform archive names", () => {
    it("matches flavored GitHub Release asset names", () => {
        const linux = resolveCliPlatform("linux", "x64")
        const win = resolveCliPlatform("win32", "x64")
        const mac = resolveCliPlatform("darwin", "arm64")

        expect(linux && cliArchiveName(linux, "v1.0.0-beta.4", "3.12")).toBe(
            "oneil-v1.0.0-beta.4-x86_64-unknown-linux-gnu-py3.12.tar.gz",
        )
        expect(win && cliArchiveName(win, "v1.0.0-beta.4", "3.14")).toBe(
            "oneil-v1.0.0-beta.4-x86_64-pc-windows-msvc-py3.14.zip",
        )
        expect(mac && cliArchiveName(mac, "v1.0.0-beta.4", "3.12")).toBe(
            "oneil-v1.0.0-beta.4-aarch64-apple-darwin-py3.12.tar.gz",
        )
        expect(linux?.binaryName).toBe("oneil")
        expect(win?.binaryName).toBe("oneil.exe")
        expect(mac?.binaryName).toBe("oneil")
    })

    it("falls back to unflavored names from older 1.x releases", () => {
        const linux = resolveCliPlatform("linux", "x64")
        expect(linux && cliUnflavoredArchiveName(linux, "v1.0.0")).toBe(
            "oneil-v1.0.0-x86_64-unknown-linux-gnu.tar.gz",
        )
        expect(linux && cliAssetCandidates(linux, "v1.0.0", "3.12")).toEqual([
            "oneil-v1.0.0-x86_64-unknown-linux-gnu-py3.12.tar.gz",
            "oneil-v1.0.0-x86_64-unknown-linux-gnu-uv.tar.gz",
            "oneil-v1.0.0-x86_64-unknown-linux-gnu-system.tar.gz",
            "oneil-v1.0.0-x86_64-unknown-linux-gnu.tar.gz",
        ])
    })
})
