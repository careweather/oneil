/**
 * Interaction atoms: transient hover state driving dependency highlighting in
 * the graph view.
 *
 * Detail panel state and navigation history live in `navigation.ts`.
 * Graph zoom and panel layout preferences live in `display.ts`.
 */

import { atom } from "jotai"

// ── Dependency highlighting ───────────────────────────────────────────────────

/**
 * Currently hovered parameter identity.
 * Present when hovering a name cell; used as a source for dependency lookups.
 */
export const hoveredParamAtom = atom<{ modelPath: string; paramName: string } | null>(null)

/**
 * Set of parameter keys currently highlighted as dependencies of the hovered
 * parameter. Uses unified key format `paramKey` (e.g. `"engine/thrust"`,
 * `"thrust"` at root, or `"sensor/brightness"` for ref pool params).
 */
export const highlightedDepsAtom = atom<Set<string>>(new Set<string>())
