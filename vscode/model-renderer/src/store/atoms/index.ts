/**
 * Barrel re-export for all Jotai atoms and related types.
 *
 * Import from `"../store/atoms"` (or `"./store/atoms"`) anywhere in the app.
 *
 * Slices:
 *   app          — AppState, appStateAtom, isLoadingAtom, instanceTreeAtom,
 *                  referencePoolAtom, bibliographyRawAtom, workspaceUriAtom,
 *                  fileBaseUriAtom, pdfCacheUriAtom, fullTreeAtom, refPoolAliasesAtom,
 *                  FullTree, FullTreeChild
 *   display      — viewModeAtom, showDesignsAtom, showTraceAtom, paramLayoutAtom,
 *                  enabledParamLayoutAtom, fontScaleAtom, showNotesEnabledAtom,
 *                  hideUnusedEnabledAtom, graphZoomAtom, detailPanelOpenAtom,
 *                  detailPanelPositionAtom, detailPanelSideWidthAtom,
 *                  detailPanelBottomHeightAtom, detailPanelTocWidthAtom,
 *                  detailPanelBibWidthAtom, detailPanelSideTocFlexAtom,
 *                  detailPanelSideEqFlexAtom, detailPanelSideBibFlexAtom,
 *                  pdfPanelWidthAtom
 *   data         — aliasToModelPathAtom, paramLookupAtom, fullParamLookupAtom,
 *                  reverseDepsAtom, renderNameLookupAtom, usedParamSetsAtom,
 *                  buildParamLookup, buildDesignIndex, FullParamLookupEntry
 *   bibliography — parsedBibliographyAtom, citationGroupsAtom,
 *                  extractCitationKeys, ParsedCitation, CitationUsage,
 *                  CitationGroup, CitationUsageLocation
 *   navigation   — focusedPathAtom, focusedNodeAtom, isViewingRefAtom,
 *                  DetailPanelState, detailPanelAtom,
 *                  detailPanelBackStackAtom, detailPanelForwardStackAtom,
 *                  navigateDetailPanelAtom, navigateDetailBackAtom,
 *                  navigateDetailForwardAtom, focusedParamKeyAtom
 *   interaction  — hoveredParamAtom, highlightedDepsAtom
 *   modelToc     — modelTocAtom, collectTocEntries, TocEntry, ReferenceTocSection
 *   pdf          — focusedPdfAtom, FocusedPdf
 */

export * from "./app"
export * from "./display"
export * from "./data"
export * from "./bibliography"
export * from "./navigation"
export * from "./interaction"
export * from "./modelToc"
export * from "./pdf"
