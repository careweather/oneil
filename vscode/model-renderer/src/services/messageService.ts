/**
 * Message service — bridges VS Code postMessage ↔ Jotai store.
 *
 * Called once at module load time (before React renders) so the message
 * listener is attached and the "ready" signal is sent as early as possible.
 * This avoids the timing gap where a message arrives before useEffect fires.
 */

import { createStore } from "jotai"
import {
    appStateAtom,
    bibliographyRawAtom,
    detailPanelOpenAtom,
    fileBaseUriAtom,
    navigateDetailPanelAtom,
    parsedBibliographyAtom,
    pdfCacheUriAtom,
    workspaceUriAtom,
    EMPTY_BIBLIOGRAPHY,
    loadParsedBibliography,
} from "../store/atoms"
import { getVsCodeApi } from "../vscode"
import type { ExtensionMessage } from "../types/messages"

export type Store = ReturnType<typeof createStore>

/**
 * Attaches the window message listener and posts "ready" to the extension.
 * Must be called with the same store instance passed to Jotai's `<Provider>`.
 */
export function initMessageService(store: Store): void {
    let currentFileUri: string | null = null

    window.addEventListener("message", (event: MessageEvent<ExtensionMessage>) => {
        const msg = event.data
        if (msg.type === "loading") {
            store.set(appStateAtom, { status: "loading" })
        } else if (msg.type === "instanceTree") {
            const fileChanged = msg.fileUri !== currentFileUri
            currentFileUri = msg.fileUri
            // On a file switch, clear the selected equation and close the panel
            // so no stale selection from the previous file is shown.
            // On a same-file reload (e.g. the webview regaining focus), leave
            // the panel state untouched — it is a persisted user preference.
            if (fileChanged) {
                store.set(navigateDetailPanelAtom, null)
                store.set(detailPanelOpenAtom, false)
            }
            store.set(bibliographyRawAtom, msg.bibliography ?? null)
            store.set(parsedBibliographyAtom, new Map(EMPTY_BIBLIOGRAPHY))
            store.set(workspaceUriAtom, msg.workspaceUri)
            store.set(fileBaseUriAtom, msg.fileBaseUri)
            store.set(pdfCacheUriAtom, msg.pdfCacheUri)
            store.set(appStateAtom, { status: "ready", data: msg.data })

            const rawBib = msg.bibliography
            if (rawBib) {
                void loadParsedBibliography(rawBib).then((bib) => {
                    store.set(parsedBibliographyAtom, bib)
                })
            }
        } else if (msg.type === "error") {
            store.set(appStateAtom, { status: "error", message: msg.message })
        }
    })

    // Tell the extension the webview is ready to receive data.
    getVsCodeApi().postMessage({ type: "ready" })
}
