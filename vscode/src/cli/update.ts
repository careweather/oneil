/**
 * Update prompts and install/select-version command flows for the managed CLI.
 */

import * as vscode from "vscode"

import { fetchLatestCliRelease, listCliReleases, type GithubRelease } from "./github"
import { installCliRelease, requireDetectedPython } from "./install"
import {
    getSkippedVersion,
    markUpdateChecked,
    setSkippedVersion,
    shouldAutoCheckUpdates,
} from "./state"
import { readCliVersion, resolveCli, type ResolvedCli } from "./resolve"
import { resolveCliPlatform, SUPPORTED_PLATFORMS_LABEL } from "./platforms"
import { isNewerVersion, normalizeVersionId, versionsEqual } from "./version"

export type RestartLanguageServer = (context: vscode.ExtensionContext) => Promise<void>

/**
 * On activate: offer install if missing. Returns the resolved CLI if any.
 * Callers should start the LSP, then optionally run a background update check.
 */
export async function runActivateCliFlow(
    context: vscode.ExtensionContext,
    restart: RestartLanguageServer,
): Promise<ResolvedCli | undefined> {
    let resolved = await resolveCli(context)

    if (!resolved) {
        const choice = await vscode.window.showInformationMessage(
            "Oneil CLI was not found. Download the latest release from GitHub?",
            "Install",
            "Not now",
        )
        if (choice === "Install") {
            await installLatestWithProgress(context, restart)
            resolved = await resolveCli(context)
        }
        return resolved
    }

    return resolved
}

/**
 * True when a throttled background update check should run.
 * Skipped versions are handled inside `checkForUpdates`.
 */
export function shouldRunBackgroundUpdateCheck(
    context: vscode.ExtensionContext,
    resolved: ResolvedCli | undefined,
): boolean {
    if (!resolved || resolved.source === "serverPath") {
        return false
    }
    const autoCheck = vscode.workspace
        .getConfiguration("oneil")
        .get<boolean>("cli.autoCheckUpdates", true)
    if (!autoCheck) {
        return false
    }
    return shouldAutoCheckUpdates(context)
}

/**
 * Manual or throttled update check against GitHub latest.
 */
export async function checkForUpdates(
    context: vscode.ExtensionContext,
    restart: RestartLanguageServer,
    options: { silentIfCurrent?: boolean } = {},
): Promise<void> {
    const platform = resolveCliPlatform()
    if (!platform) {
        void vscode.window.showErrorMessage(
            `Oneil: release binaries are not published for this platform (${SUPPORTED_PLATFORMS_LABEL}).`,
        )
        return
    }

    const serverPath = vscode.workspace.getConfiguration("oneil").get<string | null>("serverPath", null)
    if (serverPath && serverPath.trim() !== "") {
        void vscode.window.showInformationMessage(
            "Oneil: `oneil.serverPath` is set, so managed CLI updates are disabled. Clear the setting to use extension-managed installs.",
        )
        return
    }

    await markUpdateChecked(context)

    let latest: GithubRelease
    try {
        const python = await requireDetectedPython()
        latest = await fetchLatestCliRelease(platform, python)
    } catch (error) {
        void vscode.window.showErrorMessage(
            `Oneil: could not check for updates (${error instanceof Error ? error.message : String(error)})`,
        )
        return
    }

    const resolved = await resolveCli(context)
    if (!resolved) {
        const choice = await vscode.window.showInformationMessage(
            `Oneil CLI is not installed. Install ${latest.tag}?`,
            "Install",
            "Cancel",
        )
        if (choice === "Install") {
            await installReleaseWithProgress(context, latest, restart)
        }
        return
    }

    const current = await readCliVersion(resolved.command, resolved.env)
    const latestId = normalizeVersionId(latest.tag)
    if (latestId == null) {
        void vscode.window.showErrorMessage(
            `Oneil: could not parse latest release tag as semver (${latest.tag}).`,
        )
        return
    }

    if (current && versionsEqual(current, latestId)) {
        if (!options.silentIfCurrent) {
            void vscode.window.showInformationMessage(`Oneil is up to date (${current}).`)
        }
        return
    }

    if (current && !isNewerVersion(latestId, current)) {
        if (!options.silentIfCurrent) {
            void vscode.window.showInformationMessage(
                `Oneil ${current} is newer than the latest stable release (${latest.tag}).`,
            )
        }
        return
    }

    const skipped = getSkippedVersion(context)
    if (skipped && versionsEqual(skipped, latestId)) {
        if (!options.silentIfCurrent) {
            void vscode.window.showInformationMessage(
                `Oneil ${latest.tag} is available but was skipped. Use “Oneil: Install or Update CLI” to install it.`,
            )
        }
        return
    }

    const currentLabel = current ?? "unknown"
    const choice = await vscode.window.showInformationMessage(
        `Oneil ${latest.tag} is available (current: ${currentLabel}). Update?`,
        "Update",
        "Later",
        "Skip this version",
    )

    if (choice === "Update") {
        await installReleaseWithProgress(context, latest, restart)
    } else if (choice === "Skip this version") {
        await setSkippedVersion(context, latestId)
    }
}

/**
 * Downloads the latest GitHub release into the managed path and restarts the LSP.
 */
export async function installLatestWithProgress(
    context: vscode.ExtensionContext,
    restart: RestartLanguageServer,
): Promise<void> {
    const platform = resolveCliPlatform()
    if (!platform) {
        void vscode.window.showErrorMessage(
            `Oneil: release binaries are not published for this platform (${SUPPORTED_PLATFORMS_LABEL}).`,
        )
        return
    }

    try {
        const python = await requireDetectedPython()
        const latest = await fetchLatestCliRelease(platform, python)
        await installReleaseWithProgress(context, latest, restart)
    } catch (error) {
        void vscode.window.showErrorMessage(
            `Oneil: install failed (${error instanceof Error ? error.message : String(error)})`,
        )
    }
}

/**
 * QuickPick recent GitHub releases; selecting one re-downloads that tag (overwrite).
 */
export async function selectCliVersion(
    context: vscode.ExtensionContext,
    restart: RestartLanguageServer,
): Promise<void> {
    const platform = resolveCliPlatform()
    if (!platform) {
        void vscode.window.showErrorMessage(
            `Oneil: release binaries are not published for this platform (${SUPPORTED_PLATFORMS_LABEL}).`,
        )
        return
    }

    const serverPath = vscode.workspace.getConfiguration("oneil").get<string | null>("serverPath", null)
    if (serverPath && serverPath.trim() !== "") {
        void vscode.window.showInformationMessage(
            "Oneil: clear `oneil.serverPath` before selecting a managed CLI version.",
        )
        return
    }

    let releases: GithubRelease[]
    try {
        const python = await requireDetectedPython()
        releases = await listCliReleases(platform, python)
    } catch (error) {
        void vscode.window.showErrorMessage(
            `Oneil: could not list releases (${error instanceof Error ? error.message : String(error)})`,
        )
        return
    }

    if (releases.length === 0) {
        void vscode.window.showWarningMessage("Oneil: no GitHub releases with CLI assets were found.")
        return
    }

    const resolved = await resolveCli(context)
    const current = resolved ? await readCliVersion(resolved.command, resolved.env) : undefined

    const items: vscode.QuickPickItem[] = releases.map((release) => {
        const id = normalizeVersionId(release.tag)
        const isCurrent = current != null && id != null && versionsEqual(current, id)
        return {
            label: release.tag,
            description: [
                isCurrent ? "current" : undefined,
                release.prerelease ? "prerelease" : undefined,
                release.name !== release.tag ? release.name : undefined,
            ]
                .filter(Boolean)
                .join(" · "),
        }
    })

    const picked = await vscode.window.showQuickPick(items, {
        title: "Select Oneil CLI release",
        placeHolder: "Choosing a release downloads it into the extension (overwrites the managed binary)",
    })
    if (!picked) {
        return
    }

    const release = releases.find((r) => r.tag === picked.label)
    if (!release) {
        return
    }

    await installReleaseWithProgress(context, release, restart)
}

async function installReleaseWithProgress(
    context: vscode.ExtensionContext,
    release: GithubRelease,
    restart: RestartLanguageServer,
): Promise<void> {
    try {
        await vscode.window.withProgress(
            {
                location: vscode.ProgressLocation.Notification,
                title: `Installing Oneil ${release.tag}…`,
            },
            async () => {
                await installCliRelease(context, release)
                await setSkippedVersion(context, undefined)
                await restart(context)
            },
        )
        void vscode.window.showInformationMessage(`Oneil ${release.tag} installed.`)
    } catch (error) {
        void vscode.window.showErrorMessage(
            `Oneil: install failed (${error instanceof Error ? error.message : String(error)})`,
        )
    }
}
