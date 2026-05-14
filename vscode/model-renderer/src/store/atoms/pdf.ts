/**
 * PDF-viewer state atoms.
 *
 * Tracks which (if any) PDF is currently displayed inline inside the
 * detail panel, and at which page.  Components set `focusedPdfAtom` to
 * open the in-panel viewer; setting it to `null` closes the viewer and
 * returns focus to whichever panel was previously visible.
 */

import { atom } from "jotai"

/** Describes the PDF currently shown in the inline viewer. */
export interface FocusedPdf {
    /**
     * Webview-accessible URL to the PDF file.  May be a `vscode-webview-resource:`
     * URI (for local/cached files) or a remote `https:` URL as a last resort.
     */
    url: string
    /** 1-based page to display when the viewer first opens. */
    page: number
    /** Human-readable title shown in the panel header. */
    title: string
}

/**
 * The PDF currently open in the inline detail-panel viewer.
 * `null` means no PDF is focused and the default equations panel is shown.
 */
export const focusedPdfAtom = atom<FocusedPdf | null>(null)
