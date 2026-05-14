import { createStore, Provider, useAtom, useAtomValue, useSetAtom } from "jotai"
import { useEffect } from "react"
import styled from "styled-components"
import { DetailPanel, useDetailPanelLayout } from "./views/DetailPanel"
import { PdfPanel } from "./views/PdfPanel"
import { TooltipProvider } from "./components/Tooltip"
import { initMessageService } from "./services/messageService"
import { warmPdfWorker } from "./services/pdfWorker"
import {
    appStateAtom,
    detailPanelAtom,
    detailPanelOpenAtom,
    focusedPdfAtom,
    FONT_SCALE_MAX,
    FONT_SCALE_MIN,
    FONT_SCALE_STEP,
    fontScaleAtom,
    paramLayoutAtom,
    showDesignsAtom,
    showTraceAtom,
    viewModeAtom,
    type DetailPanelPosition,
    type ViewMode,
} from "./store/atoms"
import type { RenderedNode } from "./types/model"
import { getVsCodeApi } from "./vscode"
import { InstanceTreeView } from "./views/InstanceTree"
import { ModelGraphView } from "./views/ModelGraph"

// ── App styled components ────────────────────────────────────────────────────

const AppShell = styled.div`
    display: flex;
    flex-direction: column;
    height: 100%;
`

const ContentArea = styled.div<{ $panelPosition: DetailPanelPosition; $panelOpen: boolean }>`
    display: flex;
    flex-direction: ${({ $panelPosition, $panelOpen }) =>
        $panelOpen && $panelPosition === "bottom" ? "column" : "row"};
    flex: 1;
    min-height: 0;
    overflow: hidden;
`

/**
 * Always-row flex container that holds the main view and the PDF panel
 * side-by-side.  When the detail panel is docked to the bottom, this row
 * sits above it inside the column-direction `ContentArea`.
 */
const ContentRow = styled.div`
    display: flex;
    flex-direction: row;
    flex: 1;
    min-height: 0;
    min-width: 0;
    overflow: hidden;
`

const ViewContainer = styled.div`
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
`

const StatusMessage = styled.p<{ $error?: boolean }>`
    padding: var(--space-lg);
    color: ${({ $error }) => $error ? "var(--color-error)" : "var(--color-fg-muted)"};
`

const ToolbarWrapper = styled.div`
    display: flex;
    gap: var(--space-xs);
    padding: var(--space-xs) var(--space-sm);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
`

const ToolbarBtn = styled.button<{ $active?: boolean }>`
    background: transparent;
    border: 1px solid ${({ $active }) => $active ? "var(--color-focus-border)" : "transparent"};
    border-radius: var(--radius-sm);
    color: var(--color-fg);
    cursor: pointer;
    font-size: inherit;
    padding: var(--space-2xs) calc(var(--space-md) - var(--space-2xs));

    &:hover {
        background: var(--color-hover-bg);
    }

    &:disabled {
        opacity: 0.35;
        cursor: default;
    }
`

const ToolbarToggle = styled.label`
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    font-size: inherit;
    cursor: pointer;
    padding: var(--space-2xs) var(--space-compact);
    user-select: none;
    color: var(--color-fg-muted);

    input[type="checkbox"] {
        cursor: pointer;
        accent-color: var(--color-focus-border);
    }
`

const FontSizeControls = styled.div`
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    margin-left: auto;
`

const FontLabel = styled.span`
    font-size: var(--font-size-xs);
    color: var(--color-fg-muted);
    min-width: 3em;
    text-align: center;
    user-select: none;
`

/**
 * Single Jotai store — analogous to Riverpod's ProviderScope.
 * Created at module load so atom values survive React StrictMode remounts.
 */
const store = createStore()

/**
 * Attach the VS Code message listener and send "ready" before React renders,
 * so no messages can arrive before the listener is active.
 */
initMessageService(store)

export function App() {
    return (
        <Provider store={store}>
            <TooltipProvider>
                <AppContent />
            </TooltipProvider>
        </Provider>
    )
}

/** Routes to the correct view based on current app state. */
function AppContent() {
    const state = useAtomValue(appStateAtom)
    const [view, setView] = useAtom(viewModeAtom)
    const fontScale = useAtomValue(fontScaleAtom)
    const { position: panelPosition, isOpen: panelOpen } = useDetailPanelLayout()

    // Start warming the PDF.js worker as soon as the app mounts.
    // Must be in a useEffect (not at module level) so the VS Code webview
    // security context is fully established before Worker() is called.
    useEffect(() => { warmPdfWorker() }, [])

    // Auto-open the detail panel whenever an equation is selected.
    const panelState = useAtomValue(detailPanelAtom)
    const setPanelOpen = useSetAtom(detailPanelOpenAtom)
    useEffect(() => {
        if (panelState !== null) setPanelOpen(true)
    }, [panelState, setPanelOpen])

    // The PDF panel is always shown when focusedPdfAtom is non-null (it
    // renders inside ContentRow, separate from the detail panel).
    const focusedPdf = useAtomValue(focusedPdfAtom)

    const hasDesigns =
        state.status === "ready" && treeHasDesigns(state.data.root)

    return (
        <AppShell style={{ fontSize: `calc(var(--vscode-font-size, 13px) * ${fontScale})` }}>
            <Toolbar view={view} onViewChange={setView} hasDesigns={hasDesigns} />
            <ContentArea $panelPosition={panelPosition} $panelOpen={panelOpen}>
                {/* ContentRow keeps ViewContainer and PdfPanel always in a row,
                    even when the detail panel is docked to the bottom. */}
                <ContentRow>
                    <ViewContainer>
                        {state.status === "loading" && <StatusMessage>Loading…</StatusMessage>}
                        {state.status === "error" && <StatusMessage $error>Error: {state.message}</StatusMessage>}
                        {state.status === "ready" && view === "tree" && (
                            <InstanceTreeView node={state.data.root} referencePool={state.data.reference_pool} />
                        )}
                        {state.status === "ready" && view === "graph" && (
                            <ModelGraphView node={state.data.root} referencePool={state.data.reference_pool} />
                        )}
                    </ViewContainer>
                    {focusedPdf && <PdfPanel />}
                </ContentRow>
                {panelOpen && <DetailPanel />}
            </ContentArea>
        </AppShell>
    )
}

/** Recursively checks whether any node in the tree has applied designs. */
function treeHasDesigns(node: RenderedNode): boolean {
    if (node.applied_designs.length > 0) return true
    return node.children.some((c) => treeHasDesigns(c.node))
}

interface ToolbarProps {
    view: ViewMode
    onViewChange: (v: ViewMode) => void
    hasDesigns: boolean
}

/** Tab bar with view switcher and display toggles. */
function Toolbar({ view, onViewChange, hasDesigns }: ToolbarProps) {
    const [showDesigns, setShowDesigns] = useAtom(showDesignsAtom)
    const [showTrace, setShowTrace] = useAtom(showTraceAtom)
    const [fontScale, setFontScale] = useAtom(fontScaleAtom)
    const [layout, setLayout] = useAtom(paramLayoutAtom)
    const [panelOpen, setPanelOpen] = useAtom(detailPanelOpenAtom)

    const decreaseFontSize = () =>
        setFontScale((s) => Math.max(FONT_SCALE_MIN, Math.round((s - FONT_SCALE_STEP) * 10) / 10))
    const increaseFontSize = () =>
        setFontScale((s) => Math.min(FONT_SCALE_MAX, Math.round((s + FONT_SCALE_STEP) * 10) / 10))

    return (
        <ToolbarWrapper>
            <ToolbarBtn $active={view === "tree"} onClick={() => onViewChange("tree")}>
                Rendered
            </ToolbarBtn>
            <ToolbarBtn $active={view === "graph"} onClick={() => onViewChange("graph")}>
                Graph
            </ToolbarBtn>
            {hasDesigns && (
                <ToolbarToggle title="Highlight design additions and overrides">
                    <input
                        type="checkbox"
                        checked={showDesigns}
                        onChange={(e) => setShowDesigns(e.target.checked)}
                    />
                    Show Design Changes
                </ToolbarToggle>
            )}
            <ToolbarToggle title="Show trace/debug parameters">
                <input
                    type="checkbox"
                    checked={showTrace}
                    onChange={(e) => setShowTrace(e.target.checked)}
                />
                Show Trace Parameters
            </ToolbarToggle>
            {view === "graph" && (
                <ToolbarBtn
                    onClick={() => setLayout((l) => (l === "new" ? "classic" : "new"))}
                    title={layout === "new" ? "Switch to classic layout" : "Switch to new layout"}
                >
                    {layout === "new" ? "name = expr" : "expr: name"}
                </ToolbarBtn>
            )}
            <ToolbarBtn
                $active={panelOpen}
                onClick={() => setPanelOpen((o) => !o)}
                title={panelOpen ? "Close details panel" : "Open details panel"}
            >
                Details
            </ToolbarBtn>
            <FontSizeControls title="Adjust font size">
                <ToolbarBtn
                    onClick={decreaseFontSize}
                    disabled={fontScale <= FONT_SCALE_MIN}
                    aria-label="Decrease font size"
                >
                    A−
                </ToolbarBtn>
                <FontLabel>{Math.round(fontScale * 100)}%</FontLabel>
                <ToolbarBtn
                    onClick={increaseFontSize}
                    disabled={fontScale >= FONT_SCALE_MAX}
                    aria-label="Increase font size"
                >
                    A+
                </ToolbarBtn>
            </FontSizeControls>
            {import.meta.env.DEV && (
                <ToolbarBtn
                    onClick={() => getVsCodeApi().postMessage({ type: "reload" })}
                    title="Reload rendered view"
                >
                    ↺
                </ToolbarBtn>
            )}
        </ToolbarWrapper>
    )
}
