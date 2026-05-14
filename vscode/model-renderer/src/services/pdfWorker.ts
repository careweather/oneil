/**
 * Singleton that manages the PDF.js worker lifecycle.
 *
 * The worker is created lazily on the first call to `warmPdfWorker()`, which
 * should be called from a `useEffect` after the webview has fully mounted.
 * Creating the worker at module-load time fails in VS Code webviews because
 * the security context is not yet established when ES module initialisation
 * runs.
 *
 * Consumers (e.g. `PdfPane`) can read `getPdfWorker()` and subscribe to
 * readiness via `onPdfWorkerReady`.
 */

import { pdfjs } from "react-pdf"

pdfjs.GlobalWorkerOptions.workerSrc = new URL(
    "pdfjs-dist/build/pdf.worker.min.mjs",
    import.meta.url,
).toString()

let _worker: pdfjs.PDFWorker | null = null
let _ready = false
const _callbacks: Array<() => void> = []

/** Returns the worker instance once created, or null before `warmPdfWorker` is called. */
export function getPdfWorker(): pdfjs.PDFWorker | null {
    return _worker
}

/** True once the worker has fully initialised and is ready to parse documents. */
export function isPdfWorkerReady(): boolean {
    return _ready
}

/**
 * Register a callback that fires once the worker is ready.
 * If the worker is already ready, the callback fires synchronously.
 * Returns an unsubscribe function.
 */
export function onPdfWorkerReady(cb: () => void): () => void {
    if (_ready) { cb(); return () => {} }
    _callbacks.push(cb)
    return () => {
        const i = _callbacks.indexOf(cb)
        if (i >= 0) _callbacks.splice(i, 1)
    }
}

/**
 * Start warming the PDF.js worker.  Safe to call multiple times — only the
 * first call does any work.  Must be called from a React `useEffect` (i.e.
 * after the webview is mounted) rather than at module-load time.
 */
export function warmPdfWorker(): void {
    if (_worker) return
    _worker = new pdfjs.PDFWorker()
    _worker.promise
        .then(() => {
            _ready = true
            _callbacks.forEach(cb => cb())
            _callbacks.length = 0
        })
        .catch(() => {
            // Worker failed to initialise; react-pdf will surface the error
            // through its own onLoadError callback when a Document is opened.
            _worker = null
        })
}
