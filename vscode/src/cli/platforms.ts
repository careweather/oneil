/**
 * Platform → GitHub release archive mapping for the Oneil CLI.
 * Mirrors `actions/install-oneil/install.sh`.
 */

import type { PythonMinor } from "./python"

export type CliPlatform = {
    /** Rust target triple used in release asset names. */
    triple: string
    /** Archive extension published by the Release workflow. */
    archiveExt: "tar.gz" | "zip"
    /** Binary name inside the archive. */
    binaryName: string
}

/**
 * Returns the release platform for this host, or `undefined` if unsupported.
 */
export function resolveCliPlatform(
    platform: NodeJS.Platform = process.platform,
    arch: string = process.arch,
): CliPlatform | undefined {
    if (platform === "linux" && arch === "x64") {
        return {
            triple: "x86_64-unknown-linux-gnu",
            archiveExt: "tar.gz",
            binaryName: "oneil",
        }
    }
    if (platform === "win32" && arch === "x64") {
        return {
            triple: "x86_64-pc-windows-msvc",
            archiveExt: "zip",
            binaryName: "oneil.exe",
        }
    }
    if (platform === "darwin" && arch === "arm64") {
        return {
            triple: "aarch64-apple-darwin",
            archiveExt: "tar.gz",
            binaryName: "oneil",
        }
    }
    return undefined
}

/**
 * Versioned archive name: `oneil-{tag}-{triple}-py3.12.{ext}`.
 */
export function cliArchiveName(platform: CliPlatform, tag: string, python: PythonMinor): string {
    return `oneil-${tag}-${platform.triple}-py${python}.${platform.archiveExt}`
}

/**
 * Pre-flavor archive name used by older 1.x releases.
 */
export function cliUnflavoredArchiveName(platform: CliPlatform, tag: string): string {
    return `oneil-${tag}-${platform.triple}.${platform.archiveExt}`
}

/**
 * Asset names to try for a tag.
 *
 * Current releases use `py3.12` / `py3.14`. Python 3.12 also tries the older
 * layout flavors and the unflavored 1.x name.
 */
export function cliAssetCandidates(
    platform: CliPlatform,
    tag: string,
    python: PythonMinor,
): string[] {
    const names = [cliArchiveName(platform, tag, python)]
    if (python === "3.12") {
        const flavors = platform.triple.includes("apple")
            ? ["homebrew", "uv", "system"]
            : ["uv", "system"]
        for (const flavor of flavors) {
            names.push(`oneil-${tag}-${platform.triple}-${flavor}.${platform.archiveExt}`)
        }
        names.push(cliUnflavoredArchiveName(platform, tag))
    }
    return names
}

/** Human-readable list of platforms that publish CLI archives. */
export const SUPPORTED_PLATFORMS_LABEL =
    "Linux x86_64, Windows x86_64, and Apple Silicon macOS"
