/**
 * Typed wrapper around the `acquireVsCodeApi()` global injected by VS Code
 * into webview pages. Call `getVsCodeApi()` once and reuse the result.
 */

interface VsCodeApi {
    postMessage(message: unknown): void
    getState(): unknown
    setState(state: unknown): void
}

declare function acquireVsCodeApi(): VsCodeApi

let _api: VsCodeApi | undefined

/** Returns the VS Code API singleton, acquiring it on first call. */
export function getVsCodeApi(): VsCodeApi {
    _api ??= acquireVsCodeApi()
    return _api
}
