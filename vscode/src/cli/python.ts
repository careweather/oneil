/**
 * Detect which supported CPython minor version is installed.
 *
 * Current release archives are one per minor version. The `oneil` loader finds
 * the install layout itself. The flavor helpers below remain for managed
 * binaries downloaded from older layout-specific archives.
 */

import { access } from "fs/promises"
import { execFile } from "child_process"
import { promisify } from "util"
import * as path from "path"

const execFileAsync = promisify(execFile)

export const PYTHON_FLAVORS = ["homebrew", "uv", "system"] as const

export type PythonFlavor = (typeof PYTHON_FLAVORS)[number]

/** Minor versions the Release workflow publishes, newest first. */
export const PYTHON_MINORS = ["3.14", "3.12"] as const

export type PythonMinor = (typeof PYTHON_MINORS)[number]

export type DetectedPython = {
    flavor: PythonFlavor
    /** Interpreter path when known (uv, Windows system). */
    python?: string
    /** Directory containing libpython / python312.dll, when the loader needs it. */
    libraryDir?: string
}

const HOMEBREW_PYTHON_312 =
    "/opt/homebrew/opt/python@3.12/Frameworks/Python.framework/Versions/3.12/Python"

const SYSTEM_FRAMEWORK_PYTHON_312 =
    "/Library/Frameworks/Python.framework/Versions/3.12/Python"

const LINUX_LIBPYTHON = [
    "/usr/lib/x86_64-linux-gnu/libpython3.12.so.1.0",
    "/usr/lib64/libpython3.12.so.1.0",
]

const LIBRARY_DIR_SCRIPT = `
import os, sys, sysconfig
if sys.platform == "win32":
    print(os.path.dirname(sys.executable))
else:
    print(sysconfig.get_config_var("LIBDIR") or "")
`

/**
 * True when `value` is a known release-binary Python flavor.
 */
export function isPythonFlavor(value: string | undefined): value is PythonFlavor {
    return value != null && (PYTHON_FLAVORS as readonly string[]).includes(value)
}

/**
 * True when `value` is a published Python minor version.
 */
export function isPythonMinor(value: string | undefined): value is PythonMinor {
    return value != null && (PYTHON_MINORS as readonly string[]).includes(value)
}

/**
 * Flavors published for this OS (Homebrew is macOS-only).
 */
export function flavorsForPlatform(platform: NodeJS.Platform = process.platform): PythonFlavor[] {
    if (platform === "darwin") {
        return ["homebrew", "uv", "system"]
    }
    return ["uv", "system"]
}

/**
 * Hint shown when neither Python 3.14 nor 3.12 is installed.
 */
export function missingPythonHint(platform: NodeJS.Platform = process.platform): string {
    if (platform === "darwin") {
        return "This Oneil CLI needs Python 3.12 or 3.14. Install Homebrew's python@3.14 or python@3.12, the official python.org installer, or `uv python install 3.14`. Then retry, or set oneil.serverPath to a local oneil."
    }
    if (platform === "win32") {
        return "This Oneil CLI needs Python 3.12 or 3.14. Install it from python.org or with `uv python install 3.14`, then retry, or set oneil.serverPath to a local oneil."
    }
    return "This Oneil CLI needs Python 3.12 or 3.14. Install python3.14 or python3.12 (and the matching libpython) from your package manager or with `uv python install 3.14`, then retry, or set oneil.serverPath to a local oneil."
}

/**
 * Newest installed minor version the Release workflow publishes.
 */
export async function detectPythonMinor(
    platform: NodeJS.Platform = process.platform,
): Promise<PythonMinor | undefined> {
    for (const minor of PYTHON_MINORS) {
        if (await pythonMinorInstalled(minor, platform)) {
            return minor
        }
    }
    return undefined
}

async function pythonMinorInstalled(minor: PythonMinor, platform: NodeJS.Platform): Promise<boolean> {
    if (await findCommandPython312(`python${minor}`)) {
        return true
    }
    if (platform === "win32" && (await findCommandPython312("py", [`-${minor}`]))) {
        return true
    }
    if (platform === "darwin") {
        const framework = `/Library/Frameworks/Python.framework/Versions/${minor}/Python`
        const homebrew = `/opt/homebrew/opt/python@${minor}/Frameworks/Python.framework/Versions/${minor}/Python`
        if ((await fileExists(framework)) || (await fileExists(homebrew))) {
            return true
        }
    }
    try {
        const { stdout } = await execFileAsync("uv", ["python", "find", minor], {
            timeout: 10_000,
            windowsHide: true,
        })
        return stdout.trim() !== ""
    } catch {
        return false
    }
}

/**
 * First supported 3.12 layout on this machine (Homebrew, then uv, then system).
 */
export async function detectPython312(
    platform: NodeJS.Platform = process.platform,
): Promise<DetectedPython | undefined> {
    for (const flavor of flavorsForPlatform(platform)) {
        const found = await resolvePythonFlavor(flavor)
        if (found) {
            return found
        }
    }
    return undefined
}

/**
 * Resolves a specific flavor if that layout is installed.
 */
export async function resolvePythonFlavor(flavor: PythonFlavor): Promise<DetectedPython | undefined> {
    switch (flavor) {
        case "homebrew":
            return (await fileExists(HOMEBREW_PYTHON_312)) ? { flavor: "homebrew" } : undefined
        case "uv":
            return detectUvPython312()
        case "system":
            return detectSystemPython312()
    }
}

/**
 * Process env for spawning a CLI built for `detected`.
 *
 * Returns `undefined` when the binary's load command is an absolute path
 * (Homebrew / macOS system) and no search-path change is required.
 */
export function pythonLaunchEnv(
    detected: DetectedPython | undefined,
    baseEnv: NodeJS.ProcessEnv = process.env,
): NodeJS.ProcessEnv | undefined {
    if (!detected?.libraryDir) {
        return undefined
    }
    const env = { ...baseEnv }
    if (process.platform === "darwin") {
        env.DYLD_LIBRARY_PATH = prependPath(detected.libraryDir, env.DYLD_LIBRARY_PATH)
    } else if (process.platform === "linux") {
        env.LD_LIBRARY_PATH = prependPath(detected.libraryDir, env.LD_LIBRARY_PATH)
    } else if (process.platform === "win32") {
        env.PATH = prependPath(detected.libraryDir, env.PATH)
    }
    return env
}

/**
 * Launch env for a previously installed flavor (re-resolves uv/system paths).
 */
export async function launchEnvForFlavor(flavor: PythonFlavor): Promise<NodeJS.ProcessEnv | undefined> {
    return pythonLaunchEnv(await resolvePythonFlavor(flavor))
}

async function detectUvPython312(): Promise<DetectedPython | undefined> {
    try {
        const { stdout } = await execFileAsync("uv", ["python", "find", "3.12"], {
            timeout: 10_000,
            windowsHide: true,
        })
        const python = stdout.trim()
        if (!python) {
            return undefined
        }
        const libraryDir = await pythonLibraryDir(python)
        return { flavor: "uv", python, libraryDir }
    } catch {
        return undefined
    }
}

async function detectSystemPython312(): Promise<DetectedPython | undefined> {
    if (process.platform === "darwin") {
        return (await fileExists(SYSTEM_FRAMEWORK_PYTHON_312)) ? { flavor: "system" } : undefined
    }
    if (process.platform === "linux") {
        for (const lib of LINUX_LIBPYTHON) {
            if (await fileExists(lib)) {
                return { flavor: "system", libraryDir: path.dirname(lib) }
            }
        }
        const python = await findCommandPython312("python3.12")
        if (python) {
            const libraryDir = await pythonLibraryDir(python)
            return { flavor: "system", python, libraryDir }
        }
        return undefined
    }
    if (process.platform === "win32") {
        const python = await findWindowsPython312()
        if (!python) {
            return undefined
        }
        return { flavor: "system", python, libraryDir: path.dirname(python) }
    }
    return undefined
}

async function findWindowsPython312(): Promise<string | undefined> {
    return (
        (await findCommandPython312("py", ["-3.12"])) ??
        (await findCommandPython312("python3.12"))
    )
}

async function findCommandPython312(
    command: string,
    prefixArgs: string[] = [],
): Promise<string | undefined> {
    try {
        const { stdout } = await execFileAsync(
            command,
            [...prefixArgs, "-c", "import sys; print(sys.executable)"],
            { timeout: 8_000, windowsHide: true },
        )
        const python = stdout.trim()
        return python === "" ? undefined : python
    } catch {
        return undefined
    }
}

async function pythonLibraryDir(python: string): Promise<string | undefined> {
    try {
        const { stdout } = await execFileAsync(python, ["-c", LIBRARY_DIR_SCRIPT], {
            timeout: 8_000,
            windowsHide: true,
        })
        const dir = stdout.trim()
        return dir === "" ? undefined : dir
    } catch {
        return undefined
    }
}

async function fileExists(filePath: string): Promise<boolean> {
    try {
        await access(filePath)
        return true
    } catch {
        return false
    }
}

function prependPath(dir: string, existing: string | undefined): string {
    if (!existing || existing === "") {
        return dir
    }
    const sep = process.platform === "win32" ? ";" : ":"
    const parts = existing.split(sep).filter((part) => part !== "" && part !== dir)
    return [dir, ...parts].join(sep)
}
