/**
 * Display-preference atoms: view mode, visibility toggles, font scale,
 * parameter layout, graph viewport, detail panel layout, and derived
 * "effective" values consumed by renderers.
 */

import { atom } from "jotai"
import { atomWithStorage } from "jotai/utils"

// ── View mode ─────────────────────────────────────────────────────────────────

export type ViewMode = "tree" | "graph"

/** Which top-level view is active. Drives per-view effective defaults. */
export const viewModeAtom = atom<ViewMode>("tree")

// ── Visibility toggles ────────────────────────────────────────────────────────

/**
 * Whether design additions/overrides are visually highlighted.
 * When `false`, all parameters render identically regardless of provenance.
 */
export const showDesignsAtom = atom(true)

/**
 * Whether trace/debug parameters are shown.
 * When `false`, parameters with `print_level === "trace"` are hidden.
 */
export const showTraceAtom = atom(false)

// ── Parameter layout ──────────────────────────────────────────────────────────

/**
 * User-selectable layout for graph-view parameter rows.
 * - "classic": `[label ... expression] : name = value`
 * - "new":     `label | name = [expression =] value` (with aligned `=`)
 */
export type ParamLayout = "classic" | "new"

/**
 * Layout used by rendering components.
 * - "rendered": tree-view style — label above, `name = [expr =] value` below
 * - "classic" / "new": graph-view toggle options
 */
export type EffectiveParamLayout = "classic" | "new" | "rendered"

/** Persisted user preference for graph-view parameter layout. */
export const paramLayoutAtom = atomWithStorage<ParamLayout>("oneil.paramLayout", "new")

/**
 * The enabled layout for the current view.
 * Tree view always uses "rendered"; graph view uses the persisted toggle.
 * Derived so renderers never need to inspect `viewMode` directly.
 */
export const enabledParamLayoutAtom = atom<EffectiveParamLayout>((get) => {
    return get(viewModeAtom) === "tree" ? "rendered" : get(paramLayoutAtom)
})

// ── Font scale ────────────────────────────────────────────────────────────────

export const FONT_SCALE_MIN = 0.8
export const FONT_SCALE_MAX = 3.0
export const FONT_SCALE_STEP = 0.1

/**
 * User-controlled font scale multiplier applied on top of VS Code's base font
 * size. Defaults to 1.5× so nested KaTeX subscripts remain comfortably
 * readable. Persisted to localStorage.
 */
export const fontScaleAtom = atomWithStorage("oneil.fontScale", 1.5)

// ── View-driven effective values ──────────────────────────────────────────────

/** Notes are shown inline in tree view; hidden (tooltip-only) in graph view. */
export const showNotesEnabledAtom = atom((get) => get(viewModeAtom) === "tree")

/** Unused params are hidden in graph view; all shown in tree view. */
export const hideUnusedEnabledAtom = atom((get) => get(viewModeAtom) === "graph")

// ── Graph viewport ────────────────────────────────────────────────────────────

/** Current ReactFlow viewport zoom level (1 = 100%). Used to scale tooltips. */
export const graphZoomAtom = atom(1)

// ── Detail panel layout ───────────────────────────────────────────────────────

/** Position of the detail panel relative to the main view. */
export type DetailPanelPosition = "side" | "bottom"

/**
 * Whether the detail panel is currently visible.
 *
 * Closing the panel (the ✕ in the panel header) sets this to `false`.
 * Navigating to an equation automatically sets it back to `true`.
 * Persisted so the panel remembers its open/closed state across reloads.
 */
export const detailPanelOpenAtom = atomWithStorage("oneil.detailPanelOpen", true)

/** Persisted dock position of the detail panel. */
export const detailPanelPositionAtom = atomWithStorage<DetailPanelPosition>(
    "oneil.detailPanelPosition",
    "side",
)

/** Persisted pixel width of the panel when docked to the side. */
export const detailPanelSideWidthAtom = atomWithStorage<number>("oneil.detailPanelSideWidth", 340)
/** Persisted pixel height of the panel when docked to the bottom. */
export const detailPanelBottomHeightAtom = atomWithStorage<number>("oneil.detailPanelBottomHeight", 260)
/** Persisted pixel width of the TOC column in bottom mode. */
export const detailPanelTocWidthAtom = atomWithStorage<number>("oneil.detailPanelTocWidth", 200)
/** Persisted pixel width of the bibliography column in bottom mode. */
export const detailPanelBibWidthAtom = atomWithStorage<number>("oneil.detailPanelBibWidth", 220)
/**
 * Flex-proportion atoms for the three side-panel sections (TOC, equation, bibliography).
 * All default to 1 so the visible sections share space equally on first load.
 * Drag handles redistribute flex between adjacent sections.
 */
export const detailPanelSideTocFlexAtom = atomWithStorage<number>("oneil.detailPanelSideTocFlex", 1)
export const detailPanelSideEqFlexAtom = atomWithStorage<number>("oneil.detailPanelSideEqFlex", 1)
export const detailPanelSideBibFlexAtom = atomWithStorage<number>("oneil.detailPanelSideBibFlex", 1)

// ── PDF panel layout ──────────────────────────────────────────────────────────

/**
 * Persisted pixel width of the inline PDF panel.
 * The panel always docks to the right of the main view (left of the details
 * panel), independent of the details panel position.
 */
export const pdfPanelWidthAtom = atomWithStorage<number>("oneil.pdfPanelWidth", 420)
