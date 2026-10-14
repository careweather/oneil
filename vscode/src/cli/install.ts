/**
 * Download and install a single managed Oneil CLI binary under globalStorage.
 */

import * as fs from "fs/promises"
import * as path from "path"
import * as os from "os"
import { execFile } from "child_process"
import { promisify } from "util"
import type * as vscode from "vscode"

import { fetchCliReleaseByTag, releaseDownloadUrl, type GithubRelease } from "./github"
import { cliAssetCandidates, resolveCliPlatform, SUPPORTED_PLATFORMS_LABEL, type CliPlatform } from "./platforms"
import { detectPythonMinor, missingPythonHint, type PythonMinor } from "./python"
import { setInstalledPythonMinor } from "./state"
import { toReleaseTag } from "./version"

const execFileAsync = promisify(execFile)

/** Shown when neither Python 3.14 nor 3.12 is installed. */
export const PYTHON_312_HINT = missingPythonHint()

/**
 * True when `command --version` starts successfully.
 */
export async function cliBinaryRuns(
    command: string,
    env?: NodeJS.ProcessEnv,
): Promise<boolean> {
    try {
        await execFileAsync(command, ["--version"], {
            timeout: 10_000,
            windowsHide: true,
            env,
        })
        return true
    } catch {
        return false
    }
}

/**
 * Absolute path to the managed CLI binary for this host, or `undefined` if
 * the platform is unsupported.
 */
export function managedBinaryPath(context: vscode.ExtensionContext): string | undefined {
    const platform = resolveCliPlatform()
    if (!platform) {
        return undefined
    }
    return path.join(context.globalStorageUri.fsPath, "cli", platform.binaryName)
}

/**
 * True when a managed binary file exists on disk.
 */
export async function managedBinaryExists(context: vscode.ExtensionContext): Promise<boolean> {
    const binaryPath = managedBinaryPath(context)
    if (!binaryPath) {
        return false
    }
    try {
        await fs.access(binaryPath)
        return true
    } catch {
        return false
    }
}

/**
 * Downloads `release` into the fixed managed path (overwriting any previous binary).
 */
export async function installCliRelease(
    context: vscode.ExtensionContext,
    release: GithubRelease,
): Promise<string> {
    const platform = resolveCliPlatform()
    if (!platform) {
        throw new Error(
            `This platform is not supported for release binaries (${SUPPORTED_PLATFORMS_LABEL}).`,
        )
    }

    const storageRoot = path.join(context.globalStorageUri.fsPath, "cli")
    await fs.mkdir(storageRoot, { recursive: true })

    const tmpRoot = await fs.mkdtemp(path.join(os.tmpdir(), "oneil-cli-"))

    try {
        const { archivePath, assetName } = await downloadReleaseArchive(release, platform, tmpRoot)
        await extractArchive(archivePath, tmpRoot)

        const extracted = path.join(tmpRoot, platform.binaryName)
        try {
            await fs.access(extracted)
        } catch {
            throw new Error(`Archive ${assetName} did not contain ${platform.binaryName}`)
        }

        const dest = path.join(storageRoot, platform.binaryName)
        await fs.rm(dest, { force: true })
        await fs.copyFile(extracted, dest)
        await fs.chmod(dest, 0o755)

        const runnerName = platform.binaryName.endsWith(".exe") ? "oneil-runner.exe" : "oneil-runner"
        const extractedRunner = path.join(tmpRoot, runnerName)
        try {
            await fs.access(extractedRunner)
            const destRunner = path.join(storageRoot, runnerName)
            await fs.rm(destRunner, { force: true })
            await fs.copyFile(extractedRunner, destRunner)
            await fs.chmod(destRunner, 0o755)
        } catch {
            // Older archives contain only the `oneil` binary.
        }

        if (process.platform === "darwin") {
            try {
                await execFileAsync("xattr", ["-d", "com.apple.quarantine", dest])
            } catch {
                // Quarantine attribute may be absent.
            }
        }

        await setInstalledPythonMinor(context, release.python)
        if (!(await cliBinaryRuns(dest))) {
            throw new Error(missingPythonHint())
        }

        return dest
    } finally {
        await fs.rm(tmpRoot, { recursive: true, force: true })
    }
}

/**
 * Installs the release identified by `tagOrVersion` (with or without `v`).
 */
export async function installCliTag(
    context: vscode.ExtensionContext,
    tagOrVersion: string,
): Promise<string> {
    const platform = resolveCliPlatform()
    if (!platform) {
        throw new Error(
            `This platform is not supported for release binaries (${SUPPORTED_PLATFORMS_LABEL}).`,
        )
    }
    const python = await requireDetectedPython()
    const release = await fetchCliReleaseByTag(toReleaseTag(tagOrVersion), platform, python)
    return installCliRelease(context, release)
}

/**
 * Detects Python 3.14 or 3.12, or throws a user-facing hint.
 */
export async function requireDetectedPython(): Promise<PythonMinor> {
    const detected = await detectPythonMinor()
    if (!detected) {
        throw new Error(missingPythonHint())
    }
    return detected
}

async function downloadReleaseArchive(
    release: GithubRelease,
    platform: CliPlatform,
    destDir: string,
): Promise<{ archivePath: string; assetName: string }> {
    const names = cliAssetCandidates(platform, release.tag, release.python)
    let lastError: Error | undefined
    for (const assetName of names) {
        const url = releaseDownloadUrl(release.tag, assetName)
        const archivePath = path.join(destDir, assetName)
        try {
            await downloadFile(url, archivePath)
            return { archivePath, assetName }
        } catch (error) {
            lastError = error instanceof Error ? error : new Error(String(error))
            if (!lastError.message.includes("HTTP 404")) {
                throw lastError
            }
        }
    }
    throw lastError ?? new Error(`No CLI archive for ${release.tag}`)
}

async function downloadFile(url: string, dest: string): Promise<void> {
    const response = await fetch(url, {
        headers: { "User-Agent": "careweather-oneil-vscode" },
        redirect: "follow",
    })
    if (!response.ok) {
        throw new Error(`Download failed: HTTP ${response.status} for ${url}`)
    }
    const buffer = Buffer.from(await response.arrayBuffer())
    await fs.writeFile(dest, buffer)
}

async function extractArchive(archivePath: string, destDir: string): Promise<void> {
    if (archivePath.endsWith(".zip")) {
        if (process.platform === "win32") {
            await execFileAsync("powershell.exe", [
                "-NoProfile",
                "-Command",
                `Expand-Archive -LiteralPath '${archivePath.replace(/'/g, "''")}' -DestinationPath '${destDir.replace(/'/g, "''")}' -Force`,
            ])
            return
        }
        await execFileAsync("unzip", ["-o", archivePath, "-d", destDir])
        return
    }

    await execFileAsync("tar", ["-xzf", archivePath, "-C", destDir])
}
