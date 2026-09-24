/**
 * GitHub helpers for careweather/oneil CLI archives.
 *
 * Do not call `GET /repos/.../releases` (the collection or a single release).
 * Those payloads embed every asset and 504 now that each tag ships several
 * CLI archives plus wheels. List git tags, then download archives directly
 * from `github.com/releases/download/...`.
 */

import {
    isManagedReleaseVersion,
    isPrereleaseVersion,
    MIN_MANAGED_CLI_VERSION,
    pickLatestManagedRelease,
    toReleaseTag,
} from "./version"
import { cliArchiveName, type CliPlatform } from "./platforms"
import type { PythonMinor } from "./python"

const REPO = "careweather/oneil"
const API = `https://api.github.com/repos/${REPO}`
const FETCH_TIMEOUT_MS = 20_000
const MAX_ATTEMPTS = 3
const TRANSIENT_STATUS = new Set([408, 429, 502, 503, 504])

export type GithubRelease = {
    tag: string
    name: string
    prerelease: boolean
    /** Browser download URL for the CLI archive on this platform. */
    assetUrl: string
    assetName: string
    /** Python minor version used to select the asset. */
    python: PythonMinor
}

type GithubApiTag = {
    name: string
}

/**
 * Fetches the CLI release the installer should treat as latest for `platform`.
 *
 * Does not use GitHub’s `/releases/latest` — that endpoint follows
 * `make_latest` on the Release workflow, which historically marked 1.0.0
 * betas as latest. Instead we list git tags and pick the newest stable 1.x,
 * or the newest 1.x prerelease if no stable exists yet.
 */
export async function fetchLatestCliRelease(
    platform: CliPlatform,
    python: PythonMinor,
): Promise<GithubRelease> {
    const tags = await listManagedTags(30)
    const picked = pickLatestManagedRelease(tags.map((tag) => ({ tag })))
    if (!picked) {
        throw new Error(
            `No ${MIN_MANAGED_CLI_VERSION}+ CLI archive was found for ${platform.triple} (Python ${python})`,
        )
    }
    return cliReleaseFromTag(picked.tag, platform, python)
}

/**
 * Lists recent 1.x tags as CLI releases (newest first). Asset URLs are
 * constructed from the naming convention — no GitHub Releases JSON.
 */
export async function listCliReleases(
    platform: CliPlatform,
    python: PythonMinor,
    limit = 30,
): Promise<GithubRelease[]> {
    const tags = await listManagedTags(limit)
    return tags.map((tag) => cliReleaseFromTag(tag, platform, python)).slice(0, limit)
}

/**
 * Resolves a specific tag’s CLI asset for `platform`.
 */
export async function fetchCliReleaseByTag(
    tag: string,
    platform: CliPlatform,
    python: PythonMinor,
): Promise<GithubRelease> {
    const releaseTag = toReleaseTag(tag)
    if (!isManagedReleaseVersion(releaseTag)) {
        throw new Error(unsupportedReleaseMessage(releaseTag))
    }
    return cliReleaseFromTag(releaseTag, platform, python)
}

/**
 * Builds a release record from the published archive naming convention.
 */
export function cliReleaseFromTag(
    tag: string,
    platform: CliPlatform,
    python: PythonMinor,
): GithubRelease {
    const releaseTag = toReleaseTag(tag)
    const assetName = cliArchiveName(platform, releaseTag, python)
    return {
        tag: releaseTag,
        name: releaseTag,
        prerelease: isPrereleaseVersion(releaseTag),
        assetUrl: releaseDownloadUrl(releaseTag, assetName),
        assetName,
        python,
    }
}

/**
 * Direct download URL for a known tag/asset.
 */
export function releaseDownloadUrl(tag: string, archiveName: string): string {
    return `https://github.com/${REPO}/releases/download/${toReleaseTag(tag)}/${archiveName}`
}

/**
 * User-facing message for a GitHub HTTP status.
 */
export function githubHttpError(status: number): string {
    if (status === 502 || status === 503 || status === 504) {
        return "GitHub timed out. Try again in a moment."
    }
    if (status === 403 || status === 429) {
        return "GitHub rate-limited the request. Try again in a few minutes."
    }
    return `GitHub request failed: HTTP ${status}`
}

function unsupportedReleaseMessage(tag: string): string {
    return `Release ${tag} is a 0.x build (before ${MIN_MANAGED_CLI_VERSION}) and is not supported by the extension-managed installer.`
}

async function listManagedTags(limit: number): Promise<string[]> {
    const tags = await getJson<GithubApiTag[]>(
        `${API}/tags?per_page=${Math.min(Math.max(limit, 1), 100)}`,
    )
    return tags.map((item) => item.name).filter((name) => isManagedReleaseVersion(name))
}

async function getJson<T>(url: string): Promise<T> {
    let lastError: Error | undefined
    for (let attempt = 0; attempt < MAX_ATTEMPTS; attempt++) {
        try {
            const response = await fetch(url, {
                headers: {
                    Accept: "application/vnd.github+json",
                    "User-Agent": "careweather-oneil-vscode",
                    "X-GitHub-Api-Version": "2022-11-28",
                },
                signal: AbortSignal.timeout(FETCH_TIMEOUT_MS),
            })
            if (response.ok) {
                return (await response.json()) as T
            }
            lastError = new Error(githubHttpError(response.status))
            if (!TRANSIENT_STATUS.has(response.status) || attempt === MAX_ATTEMPTS - 1) {
                throw lastError
            }
        } catch (error) {
            lastError = error instanceof Error ? error : new Error(String(error))
            if (lastError.name === "TimeoutError" || lastError.name === "AbortError") {
                lastError = new Error("GitHub timed out. Try again in a moment.")
            }
            const statusMatch = /HTTP (\d+)/.exec(lastError.message)
            const status = statusMatch ? Number(statusMatch[1]) : undefined
            const retryable =
                lastError.message.includes("timed out")
                || lastError.name === "TimeoutError"
                || lastError.name === "AbortError"
                || (status != null && TRANSIENT_STATUS.has(status))
            if (!retryable || attempt === MAX_ATTEMPTS - 1) {
                throw lastError
            }
        }
        await delay(400 * 2 ** attempt)
    }
    throw lastError ?? new Error("GitHub request failed")
}

function delay(ms: number): Promise<void> {
    return new Promise((resolve) => {
        setTimeout(resolve, ms)
    })
}
