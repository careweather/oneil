/**
 * Shared grid component that renders the full content of a model node:
 * tests, parameters, and named sections. Used by both tree and graph views.
 */
import React, { useCallback, useMemo } from "react"
import { useAtom, useAtomValue, useSetAtom } from "jotai"
import styled, { css } from "styled-components"
import { useTooltipTrigger } from "./Tooltip"
import {
    ParamRowContent,
    ParameterRowLi,
    ParamLabelExpr,
    ParamLabelNew,
    ParamExprText,
    ParamExprNew,
    ParamNoteLine,
} from "./ParameterRow"
import { TestRow } from "./TestRow"
import type { DepLabelEntry } from "./ParameterRow"
import type { ParameterValueAst, RenderedNode, RenderedParameter } from "../types/model"
import {
    aliasToModelPathAtom,
    buildDesignIndex,
    enabledParamLayoutAtom,
    focusedParamKeyAtom,
    hideUnusedEnabledAtom,
    highlightedDepsAtom,
    navigateDetailPanelAtom,
    paramLookupAtom,
    refPoolAliasesAtom,
    showDesignsAtom,
    showNotesEnabledAtom,
    showTraceAtom,
    usedParamSetsAtom,
    type EffectiveParamLayout,
} from "../store/atoms"
import { designColorVar } from "../utils/designColors"
import { extractDependencyKeys } from "../utils/extractDependencies"
import { paramKey } from "../utils/instancePath"
import { isParamVisible } from "../utils/paramVisibility"
import { NoteDisplay } from "./NoteDisplay"

// ── Grid-level item styled components ────────────────────────────────────────
//
// These are for the items that are NOT ParameterRow: test row wrappers and
// section headings/notes. ParameterRow-specific components live in ParameterRow.tsx.

/** `<li>` wrapper for test rows and other full-width items. */
const FullRowItem = styled.li`
    grid-column: 1 / -1;
    display: block;
`

/** Section heading `<li>`. */
const SectionHeadingItem = styled.li`
    grid-column: 1 / -1;
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-bold);
    color: var(--color-fg-muted);
    text-transform: uppercase;
    letter-spacing: var(--letter-spacing-caps);
    margin-top: var(--space-xs);
    padding-top: var(--space-xs);
    border-top: 1px solid var(--color-divider-subtle);
`

/** Section note `<li>`. */
const SectionNoteItem = styled.li`
    grid-column: 1 / -1;
    color: var(--color-fg-muted);
    font-size: var(--font-size-sm);
    margin-bottom: var(--space-xs);
`

/** Note wrapper for the graph view (passed as NoteWrapper to ParamRowContent). */
export const GraphParamNote = styled.span`
    grid-column: 1 / -1;
    padding-left: 1em;
    color: var(--color-fg-subtle);
    font-style: italic;
    font-size: var(--font-size-xs);
    line-height: 1.5;
`

/** Parameter grid used in the tree view. */
const ParameterList = styled.ul<{ $layout: EffectiveParamLayout }>`
    list-style: none;
    margin-bottom: var(--space-sm);
    display: grid;
    grid-template-columns: ${({ $layout }) =>
        $layout === "rendered"
            ? "1fr"
            : $layout === "new"
            ? "auto 1fr auto auto auto auto auto"
            : "1fr auto auto"};
    column-gap: var(--space-sm);
    row-gap: ${({ $layout }) => $layout === "rendered" ? "0" : "0.15em"};
    font-size: var(--font-size-sm);
    align-items: baseline;

    ${({ $layout }) => $layout === "rendered" && css`
        ${ParameterRowLi} {
            display: block;
            grid-column: 1 / -1;
            padding: 0.25em 0;
        }
    `}
`

/** Graph-view variant: smaller font; never uses the "rendered" layout. */
const GraphParameterList = styled(ParameterList)`
    font-size: var(--font-size-xs);

    ${ParamLabelExpr},
    ${ParamLabelNew} {
        font-size: var(--font-size-xs);
    }

    ${ParamExprText},
    ${ParamExprNew} {
        color: var(--color-fg-subtle);
        font-size: var(--font-size-sm);
    }
`


// ── NodeContentGrid ──────────────────────────────────────────────────────────

interface NodeContentGridProps {
    node: RenderedNode
    /** "graph" uses GraphParameterList + graph-param-note wrapper class */
    variant?: "tree" | "graph"
    enableTooltip?: boolean
}

/**
 * Renders the full parameter grid for a model node: unsectioned tests,
 * unsectioned params, and named sections with their items.
 * Reads display state from atoms directly.
 */
export function NodeContentGrid({ node, variant = "tree", enableTooltip = false }: NodeContentGridProps) {
    const showTrace = useAtomValue(showTraceAtom)
    const showNotes = useAtomValue(showNotesEnabledAtom)
    const hideUnused = useAtomValue(hideUnusedEnabledAtom)
    const usedSets = useAtomValue(usedParamSetsAtom)
    const layout = useAtomValue(enabledParamLayoutAtom)

    const designIndex = useMemo(() => buildDesignIndex(node.applied_designs), [node.applied_designs])

    const paramByName = useMemo(
        () => new Map(node.parameters.map((p) => [p.name, p])),
        [node.parameters],
    )

    const sections = node.sections ?? []
    const tests = node.tests ?? []
    const ip = node.instance_path

    const sectionedParamNames = useMemo(
        () => new Set(sections.flatMap((s) => s.items.flatMap((i) => i.type === "parameter" ? [i.name] : []))),
        [sections],
    )
    const sectionedTestIndices = useMemo(
        () => new Set(sections.flatMap((s) => s.items.flatMap((i) => i.type === "test" ? [i.index] : []))),
        [sections],
    )

    const visible = useCallback((name: string, printLevel: string): boolean => {
        return isParamVisible(name, printLevel, ip, showTrace, hideUnused, usedSets.usedParamKeys)
    }, [showTrace, hideUnused, usedSets, ip])

    const unsectionedParams = useMemo(
        () => (node.parameters ?? []).filter((p) => !sectionedParamNames.has(p.name) && visible(p.name, p.print_level)),
        [node.parameters, sectionedParamNames, visible],
    )
    const unsectionedTests = useMemo(
        () => tests.map((t, i) => ({ test: t, index: i })).filter(({ index }) => !sectionedTestIndices.has(index)),
        [tests, sectionedTestIndices],
    )

    const hasContent = unsectionedParams.length > 0 || unsectionedTests.length > 0 || sections.length > 0
    if (!hasContent) return null

    const Grid = variant === "graph" ? GraphParameterList : ParameterList
    const NoteWrapper = variant === "graph" ? GraphParamNote : ParamNoteLine

    return (
        <Grid $layout={layout}>
            {unsectionedTests.map(({ test, index }) => (
                <FullRowItem key={`test-${index}`}>
                    <TestRow test={test} index={index} instancePath={ip} enableTooltip={enableTooltip} parameters={node.parameters} />
                </FullRowItem>
            ))}
            {unsectionedParams.map((p) => (
                <ParameterRow
                    key={p.name}
                    param={p}
                    designIndex={designIndex}
                    instancePath={ip}
                    layout={layout}
                    enableTooltip={enableTooltip}
                    NoteWrapper={NoteWrapper}
                    parameters={node.parameters}
                />
            ))}
            {sections.map((sec) => {
                const visibleItems = sec.items.filter((item) => {
                    if (item.type === "test") return true
                    return visible(item.name, paramByName.get(item.name)?.print_level ?? "none")
                })
                if (visibleItems.length === 0) return null
                return (
                    <React.Fragment key={sec.label}>
                        <SectionHeadingItem>{sec.label}</SectionHeadingItem>
                        {showNotes && sec.note && (
                            <SectionNoteItem data-section-note={sec.label}>
                                <NoteDisplay text={sec.note} parameters={node.parameters} />
                            </SectionNoteItem>
                        )}
                        {visibleItems.map((item) => {
                            if (item.type === "test") {
                                const test = tests[item.index]
                                if (!test) return null
                                return (
                                    <FullRowItem key={`test-${item.index}`}>
                                        <TestRow test={test} index={item.index} instancePath={ip} enableTooltip={enableTooltip} parameters={node.parameters} />
                                    </FullRowItem>
                                )
                            }
                            const param = paramByName.get(item.name)
                            if (!param) return null
                            return (
                                <ParameterRow
                                    key={item.name}
                                    param={param}
                                    designIndex={designIndex}
                                    instancePath={ip}
                                    layout={layout}
                                    enableTooltip={enableTooltip}
                                    NoteWrapper={NoteWrapper}
                                    parameters={node.parameters}
                                />
                            )
                        })}
                    </React.Fragment>
                )
            })}
        </Grid>
    )
}

// ── ParameterRow ──────────────────────────────────────────────────────────────

function ParameterRow({
    param,
    designIndex,
    instancePath,
    layout,
    enableTooltip,
    NoteWrapper,
    parameters,
}: {
    param: RenderedParameter
    designIndex: Map<string, number>
    instancePath: string[]
    layout: EffectiveParamLayout
    enableTooltip: boolean
    NoteWrapper?: React.ElementType
    parameters?: RenderedParameter[]
}) {
    const showDesigns = useAtomValue(showDesignsAtom)
    const showNotes = useAtomValue(showNotesEnabledAtom)
    const [highlightedDeps, setHighlightedDeps] = useAtom(highlightedDepsAtom)
    const aliasToModelPath = useAtomValue(aliasToModelPathAtom)
    const refPoolAliases = useAtomValue(refPoolAliasesAtom)
    const paramLookup = useAtomValue(paramLookupAtom)
    const navigateDetailPanel = useSetAtom(navigateDetailPanelAtom)
    const setFocusedParam = useSetAtom(focusedParamKeyAtom)
    const focusedParamKeyValue = useAtomValue(focusedParamKeyAtom)
    const tooltipProps = useTooltipTrigger(enableTooltip && !showNotes ? param.note : undefined)
    const mark = param.design
    const colorIdx = mark != null ? (designIndex.get(mark.design_name) ?? 0) : null

    const depKeys = useMemo(
        () => extractDependencyKeys(param.expression as ParameterValueAst | null, instancePath, aliasToModelPath, refPoolAliases),
        [param.expression, instancePath, aliasToModelPath, refPoolAliases],
    )

    const exprDeps: DepLabelEntry[] = useMemo(
        () => [...depKeys].flatMap((key) => {
            const entry = paramLookup.get(key)
            return entry ? [{ key, ...entry }] : []
        }),
        [depKeys, paramLookup],
    )

    const myKey = useMemo(
        () => paramKey(instancePath, param.name),
        [instancePath, param.name],
    )

    const isHighlighted = highlightedDeps.has(myKey)
    const isFocused = focusedParamKeyValue === myKey

    const onNameMouseEnter = useCallback(() => {
        if (depKeys.size > 0) setHighlightedDeps(depKeys)
    }, [depKeys, setHighlightedDeps])

    const onNameMouseLeave = useCallback(() => {
        setHighlightedDeps(new Set<string>())
    }, [setHighlightedDeps])

    const expression = param.expression as ParameterValueAst | null

    const onParamClick = useCallback(() => {
        navigateDetailPanel({
            key: myKey,
            paramName: param.name,
            paramRenderName: param.render_name,
            paramLabel: param.label,
            note: param.note,
            expression,
            value: param.value,
            deps: exprDeps,
            instancePath,
        })
        setFocusedParam(myKey)
        setTimeout(() => setFocusedParam(null), 1500)
    }, [expression, param.name, param.render_name, param.label, param.value, exprDeps, instancePath, navigateDetailPanel, setFocusedParam, myKey])

    const labelStyle: React.CSSProperties =
        showDesigns && mark != null && colorIdx != null
            ? {
                  borderLeftColor: designColorVar(colorIdx),
                  borderLeftWidth: "var(--design-border-width)",
                  borderLeftStyle: "solid",
                  paddingLeft: "var(--space-compact)",
                  ...(mark.is_addition
                      ? { backgroundColor: `color-mix(in srgb, ${designColorVar(colorIdx)} var(--design-color-tint-alpha), transparent)` }
                      : {}),
              }
            : {}

    const isPerformance = param.print_level === "performance"

    return (
        <ParameterRowLi
            $highlighted={isHighlighted}
            $performance={isPerformance}
            $focused={isFocused}
            data-param-key={myKey}
            data-param-name={param.name}
        >
            <ParamRowContent
                layout={layout}
                name={param.name}
                renderName={param.render_name}
                label={param.label}
                expression={expression}
                value={param.value}
                note={param.note}
                showNotes={showNotes}
                hasDeps={depKeys.size > 0}
                onNameMouseEnter={onNameMouseEnter}
                onNameMouseLeave={onNameMouseLeave}
                labelStyle={labelStyle}
                tooltipProps={tooltipProps}
                onExprClick={onParamClick}
                onRowClick={onParamClick}
                NoteDisplay={NoteDisplay}
                parameters={parameters}
                NoteWrapper={NoteWrapper}
                instancePath={instancePath}
            />
        </ParameterRowLi>
    )
}

