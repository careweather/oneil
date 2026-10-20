/**
 * Resolve which Oneil CLI binary the extension should spawn.
 *
 * Precedence: `oneil.serverPath` → managed globalStorage binary → `ONEIL_PATH` → `oneil` on PATH.
 */

import { access } from "fs/promises"
import { constants as fsConstants } from "fs"
import { execFile } from "child_process"
import { promisify } from "util"
import * as path from "path"
import * as vscode from "vscode"

import { cliBinaryRuns, managedBinaryExists, managedBinaryPath } from "./install"
import { launchEnvForFlavor, missingPythonHint, type PythonFlavor } from "./python"
import { getInstalledPythonFlavor } from "./state"
import { parseVersionOutput } from "./version"

const execFileAsync = promisify(execFile)

export type ResolvedCli = {
    /** Absolute path or command name to spawn. */
    command: string
    /** Where the command came from. */
    source: "serverPath" | "managed" | "env" | "path"
    /** Extra env for the managed uv (and some system) flavors. */
    env?: NodeJS.ProcessEnv
    /** Python layout of the managed binary, when known. */
    flavor?: PythonFlavor
}

/**
 * Resolves the CLI command according to extension precedence rules.
 */
export async function resolveCli(context: vscode.ExtensionContext): Promise<ResolvedCli | undefined> {
    const config = vscode.workspace.getConfiguration("oneil")
    const serverPath = config.get<string | null>("serverPath", null)
    if (serverPath && serverPath.trim() !== "") {
        return { command: serverPath.trim(), source: "serverPath" }
    }

    if (await managedBinaryExists(context)) {
        const command = managedBinaryPath(context)
        const flavor = getInstalledPythonFlavor(context)
        const legacy = command != null && !(await runnerBeside(command))
        const env = legacy && flavor ? await launchEnvForFlavor(flavor) : undefined
        if (command && (await cliBinaryRuns(command, env))) {
            return { command, source: "managed", env, flavor }
        }
        if (command) {
            void vscode.window.showWarningMessage(`Oneil: ${missingPythonHint()}`)
        }
    }

    const envPath = process.env.ONEIL_PATH
    if (envPath && envPath.trim() !== "") {
        return { command: envPath.trim(), source: "env" }
    }

    if (await commandExistsOnPath("oneil")) {
        return { command: "oneil", source: "path" }
    }

    return undefined
}

/**
 * Runs `<command> --version` and returns the normalized version id.
 */
export async function readCliVersion(
    command: string,
    env?: NodeJS.ProcessEnv,
): Promise<string | undefined> {
    try {
        const { stdout } = await execFileAsync(command, ["--version"], {
            timeout: 10_000,
            windowsHide: true,
            env,
        })
        return parseVersionOutput(stdout)
    } catch {
        return undefined
    }
}

/**
 * True when a current archive's `oneil-runner` sits next to the loader.
 */
async function runnerBeside(command: string): Promise<boolean> {
    const runnerName = command.endsWith(".exe") ? "oneil-runner.exe" : "oneil-runner"
    try {
        await access(path.join(path.dirname(command), runnerName))
        return true
    } catch {
        return false
    }
}

async function commandExistsOnPath(command: string): Promise<boolean> {
    if (command.includes("/") || command.includes("\\")) {
        try {
            await access(command, fsConstants.X_OK)
            return true
        } catch {
            try {
                await access(command, fsConstants.F_OK)
                return true
            } catch {
                return false
            }
        }
    }

    try {
        await execFileAsync(command, ["--version"], { timeout: 5_000, windowsHide: true })
        return true
    } catch {
        return false
    }
}
