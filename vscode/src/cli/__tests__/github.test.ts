import { afterEach, describe, expect, it, vi } from "vitest"
import { fetchLatestCliRelease, githubHttpError, listCliReleases } from "../github"
import { resolveCliPlatform } from "../platforms"

describe("githubHttpError", () => {
    it("maps gateway timeouts to a retry hint", () => {
        expect(githubHttpError(504)).toBe("GitHub timed out. Try again in a moment.")
        expect(githubHttpError(502)).toBe("GitHub timed out. Try again in a moment.")
        expect(githubHttpError(403)).toContain("rate-limited")
        expect(githubHttpError(404)).toBe("GitHub request failed: HTTP 404")
    })
})

describe("fetchLatestCliRelease", () => {
    afterEach(() => {
        vi.unstubAllGlobals()
        vi.useRealTimers()
    })

    it("lists git tags and constructs the download URL without GET /releases", async () => {
        const platform = resolveCliPlatform("darwin", "arm64")
        expect(platform).toBeDefined()
        if (platform == null) {
            return
        }

        const urls: string[] = []
        vi.stubGlobal(
            "fetch",
            vi.fn(async (input: RequestInfo | URL) => {
                const url = String(input)
                urls.push(url)
                if (url.includes("/tags?")) {
                    return jsonResponse([{ name: "v1.0.0-beta.5" }, { name: "v0.16.1" }])
                }
                return jsonResponse({}, 404)
            }),
        )

        const release = await fetchLatestCliRelease(platform, "3.14")
        expect(release.tag).toBe("v1.0.0-beta.5")
        expect(release.assetName).toBe(
            "oneil-v1.0.0-beta.5-aarch64-apple-darwin-py3.14.tar.gz",
        )
        expect(release.assetUrl).toContain(
            "/releases/download/v1.0.0-beta.5/oneil-v1.0.0-beta.5-aarch64-apple-darwin-py3.14.tar.gz",
        )
        expect(urls.every((url) => !url.includes("/releases"))).toBe(true)
        expect(urls.some((url) => url.includes("/tags?"))).toBe(true)
    })

    it("retries a 504 on /tags and then succeeds", async () => {
        vi.useFakeTimers()
        const platform = resolveCliPlatform("linux", "x64")
        expect(platform).toBeDefined()
        if (platform == null) {
            return
        }

        let tagsCalls = 0
        vi.stubGlobal(
            "fetch",
            vi.fn(async (input: RequestInfo | URL) => {
                const url = String(input)
                if (url.includes("/tags?")) {
                    tagsCalls += 1
                    if (tagsCalls === 1) {
                        return jsonResponse({}, 504)
                    }
                    return jsonResponse([{ name: "v1.0.0-beta.5" }])
                }
                return jsonResponse({}, 404)
            }),
        )

        const pending = fetchLatestCliRelease(platform, "3.12")
        await vi.runAllTimersAsync()
        const release = await pending
        expect(release.assetName).toContain("-py3.12.tar.gz")
        expect(tagsCalls).toBe(2)
    })
})

describe("listCliReleases", () => {
    afterEach(() => {
        vi.unstubAllGlobals()
    })

    it("does not fetch release JSON for each tag", async () => {
        const platform = resolveCliPlatform("win32", "x64")
        expect(platform).toBeDefined()
        if (platform == null) {
            return
        }

        const urls: string[] = []
        vi.stubGlobal(
            "fetch",
            vi.fn(async (input: RequestInfo | URL) => {
                const url = String(input)
                urls.push(url)
                return jsonResponse([
                    { name: "v1.0.0-beta.5" },
                    { name: "v1.0.0-beta.4" },
                    { name: "v0.16.1" },
                ])
            }),
        )

        const releases = await listCliReleases(platform, "3.12")
        expect(releases.map((item) => item.tag)).toEqual(["v1.0.0-beta.5", "v1.0.0-beta.4"])
        expect(urls).toHaveLength(1)
        expect(urls[0]).toContain("/tags?")
    })
})

function jsonResponse(body: unknown, status = 200): Response {
    return new Response(JSON.stringify(body), {
        status,
        headers: { "Content-Type": "application/json" },
    })
}
