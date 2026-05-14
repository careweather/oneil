/**
 * Inline PDF viewer rendered inside the detail panel.
 *
 * Uses `react-pdf` (backed by PDF.js) to render the PDF entirely within the
 * VS Code webview — no external browser required.  The worker is loaded from
 * the extension's `out/model-renderer/assets/` directory, which is always
 * included in the webview's `localResourceRoots`.
 *
 * The viewer opens when `focusedPdfAtom` is non-null and provides:
 *  - Page navigation (prev/next buttons + keyboard left/right arrows).
 *  - A close button that returns to the previous panel view.
 *  - Width-responsive rendering: the page fills the available panel width.
 */

import { useAtomValue, useSetAtom } from "jotai"
import { useCallback, useEffect, useRef, useState } from "react"
import { Document, Page } from "react-pdf"
import styled from "styled-components"
import { focusedPdfAtom } from "../store/atoms"
import { getPdfWorker, isPdfWorkerReady, onPdfWorkerReady } from "../services/pdfWorker"

// ── Styled components ─────────────────────────────────────────────────────────

const Container = styled.div`
    display: flex;
    flex-direction: column;
    overflow: hidden;
    flex: 1;
    min-height: 0;
`

const Header = styled.div`
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xs) var(--space-md);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-bold);
    color: var(--color-fg-muted);
    text-transform: uppercase;
    letter-spacing: var(--letter-spacing-caps);
    min-width: 0;
    position: relative;
    overflow: hidden;
`

const Title = styled.span`
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
`

const NavGroup = styled.div`
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    flex-shrink: 0;
`

const NavButton = styled.button`
    all: unset;
    cursor: pointer;
    padding: 0 var(--space-xs);
    color: var(--color-fg-muted);
    font-size: var(--font-size-sm);
    line-height: 1;
    border-radius: var(--radius-xs);
    &:hover { color: var(--color-fg); background: var(--color-bg-hover, rgba(127,127,127,.15)); }
    &:disabled { opacity: 0.35; cursor: default; }
`

const PageLabel = styled.span`
    font-size: var(--font-size-xs);
    color: var(--color-fg-muted);
    min-width: 4ch;
    text-align: center;
`

const CloseButton = styled.button`
    all: unset;
    cursor: pointer;
    padding: 0 var(--space-xs);
    color: var(--color-fg-muted);
    font-size: var(--font-size-sm);
    border-radius: var(--radius-xs);
    flex-shrink: 0;
    &:hover { color: var(--color-fg); background: var(--color-bg-hover, rgba(127,127,127,.15)); }
`

const ScrollArea = styled.div`
    flex: 1;
    overflow: auto;
    min-height: 0;
    display: flex;
    justify-content: center;
    padding: var(--space-sm) 0;
`

const StatusMsg = styled.div`
    padding: var(--space-md);
    color: var(--color-fg-muted);
    font-size: var(--font-size-sm);
`

const ProgressBar = styled.div<{ $pct: number }>`
    position: absolute;
    bottom: 0;
    left: 0;
    height: 2px;
    width: ${({ $pct }) => $pct}%;
    background: var(--color-focus-border);
    transition: width 0.1s linear;
`

// ── Component ─────────────────────────────────────────────────────────────────

// Pixel density cap: never render above 2× even on high-DPI displays.
// Rendering at 3× or 4× produces very large canvases with no perceptible
// quality gain at datasheet-reading zoom levels.
const DEVICE_PIXEL_RATIO = Math.min(typeof window !== "undefined" ? window.devicePixelRatio : 1, 2)

/** Props forwarded from the layout — allows the parent to control flex sizing. */
interface PdfPaneProps {
    style?: React.CSSProperties
}

/**
 * Renders the focused PDF in a scrollable panel section.
 * Returns `null` when no PDF is focused.
 *
 * ```tsx
 * <PdfPane style={{ flex: sideEqFlex }} />
 * ```
 */
export function PdfPane({ style }: PdfPaneProps) {
    const focusedPdf = useAtomValue(focusedPdfAtom)
    const setFocusedPdf = useSetAtom(focusedPdfAtom)

    const [numPages, setNumPages] = useState<number | null>(null)
    const [currentPage, setCurrentPage] = useState(1)
    const [loadError, setLoadError] = useState<string | null>(null)
    const [loadProgress, setLoadProgress] = useState<number | null>(null)
    const [workerReady, setWorkerReady] = useState(isPdfWorkerReady)

    // Container ref used for responsive width measurement.
    const scrollRef = useRef<HTMLDivElement>(null)
    const [paneWidth, setPaneWidth] = useState(400)
    const resizeTimer = useRef<ReturnType<typeof setTimeout> | null>(null)

    // Subscribe to worker readiness so the status message updates live.
    useEffect(() => onPdfWorkerReady(() => setWorkerReady(true)), [])

    // Reset state when the focused PDF changes.
    useEffect(() => {
        if (focusedPdf) {
            setCurrentPage(focusedPdf.page)
            setNumPages(null)
            setLoadError(null)
            setLoadProgress(null)
        }
    }, [focusedPdf])

    // Measure the scroll area width so the PDF page fills it.
    // Debounced at 60 ms to avoid re-rendering on every pixel of a resize drag.
    useEffect(() => {
        const el = scrollRef.current
        if (!el) return
        const obs = new ResizeObserver(([entry]) => {
            if (resizeTimer.current) clearTimeout(resizeTimer.current)
            resizeTimer.current = setTimeout(() => {
                setPaneWidth(Math.floor(entry.contentRect.width) - 16)
            }, 60)
        })
        obs.observe(el)
        return () => { obs.disconnect(); if (resizeTimer.current) clearTimeout(resizeTimer.current) }
    }, [])

    // Keyboard left/right navigation while focused.
    useEffect(() => {
        const handler = (e: KeyboardEvent) => {
            if (!focusedPdf) return
            if (e.key === "ArrowLeft") setCurrentPage(p => Math.max(1, p - 1))
            if (e.key === "ArrowRight") setCurrentPage(p => Math.min(numPages ?? p, p + 1))
        }
        window.addEventListener("keydown", handler)
        return () => window.removeEventListener("keydown", handler)
    }, [focusedPdf, numPages])

    const handleLoadSuccess = useCallback(({ numPages }: { numPages: number }) => {
        setNumPages(numPages)
        setLoadError(null)
        setLoadProgress(100)
    }, [])

    const handleLoadProgress = useCallback(({ loaded, total }: { loaded: number; total: number }) => {
        if (total > 0) setLoadProgress(Math.round(loaded / total * 100))
    }, [])

    const handleLoadError = useCallback((err: Error) => {
        setLoadError(err.message)
        setLoadProgress(null)
    }, [])

    const close = useCallback(() => setFocusedPdf(null), [setFocusedPdf])
    const prev = useCallback(() => setCurrentPage(p => Math.max(1, p - 1)), [])
    const next = useCallback(() => setCurrentPage(p => Math.min(numPages ?? p, p + 1)), [numPages])

    if (!focusedPdf) return null

    return (
        <Container style={style}>
            <Header>
                <Title title={focusedPdf.title}>{focusedPdf.title}</Title>
                <NavGroup>
                    <NavButton onClick={prev} disabled={currentPage <= 1} aria-label="Previous page">‹</NavButton>
                    <PageLabel>{currentPage}{numPages != null ? ` / ${numPages}` : ""}</PageLabel>
                    <NavButton onClick={next} disabled={numPages != null && currentPage >= numPages} aria-label="Next page">›</NavButton>
                </NavGroup>
                <CloseButton onClick={close} aria-label="Close PDF viewer" title="Close">✕</CloseButton>
                {loadProgress != null && loadProgress < 100 && (
                    <ProgressBar $pct={loadProgress} />
                )}
            </Header>
            <ScrollArea ref={scrollRef}>
                {loadError ? (
                    <StatusMsg>Failed to load PDF: {loadError}</StatusMsg>
                ) : (
                    <Document
                        file={focusedPdf.url}
                        options={{ worker: getPdfWorker() ?? undefined }}
                        onLoadSuccess={handleLoadSuccess}
                        onLoadError={handleLoadError}
                        onLoadProgress={handleLoadProgress}
                        loading={
                            <StatusMsg>
                                {workerReady
                                    ? loadProgress != null
                                        ? `Loading PDF… ${loadProgress}%`
                                        : "Loading PDF…"
                                    : "Initializing PDF engine…"}
                            </StatusMsg>
                        }
                        error={<StatusMsg>Could not load PDF.</StatusMsg>}
                    >
                        <Page
                            pageNumber={currentPage}
                            width={paneWidth}
                            renderTextLayer={false}
                            renderAnnotationLayer={false}
                            devicePixelRatio={DEVICE_PIXEL_RATIO}
                        />
                    </Document>
                )}
            </ScrollArea>
        </Container>
    )
}
