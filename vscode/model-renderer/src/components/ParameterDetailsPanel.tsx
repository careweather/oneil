/**
 * Equation detail section: shows the expression, note, direct dependencies, and
 * reverse-dependency ("used by") list for the currently selected parameter.
 *
 * Reads all navigation and content state from atoms directly so callers only
 * need to supply an optional `style` for flex sizing inside the layout.
 * Returns `null` when no parameter is selected.
 */
import React, { useCallback, useMemo } from "react"
import { useAtom, useAtomValue, useSetAtom } from "jotai"
import styled from "styled-components"
import {
    aliasToModelPathAtom,
    detailPanelAtom,
    detailPanelBackStackAtom,
    detailPanelForwardStackAtom,
    focusedNodeAtom,
    focusedParamKeyAtom,
    focusedPathAtom,
    fullParamLookupAtom,
    instanceTreeAtom,
    navigateDetailBackAtom,
    navigateDetailForwardAtom,
    navigateDetailPanelAtom,
    paramLookupAtom,
    refPoolAliasesAtom,
    reverseDepsAtom,
} from "../store/atoms"
import { ExprDisplay, NameDisplay, ValueDisplay, type DepLabelEntry } from "./ParameterRow"
import { NoteDisplay } from "./NoteDisplay"
import { extractDependencyKeys } from "../utils/extractDependencies"
import { isSimpleLiteral, mathNameWithRef } from "../utils/exprToLatex"
import { pathsEqual } from "../utils/instancePath"

// ── Styled components ─────────────────────────────────────────────────────────

/**
 * Outer flex-column container. The caller supplies `style` with the appropriate
 * `flex` value so both the side layout (flex-unit height) and the bottom layout
 * (flex: 1 remaining width) work without variant props.
 */
const DetailsContainer = styled.div`
    display: flex;
    flex-direction: column;
    overflow: hidden;
    flex-shrink: 1;
    min-width: 0;
    min-height: var(--detail-panel-section-min-height);
`

const Header = styled.div`
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--color-border);
`

const HeaderTitle = styled.div`
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-bold);
    color: var(--color-fg-muted);
    text-transform: uppercase;
    letter-spacing: var(--letter-spacing-caps);
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    min-width: 0;
    overflow: hidden;
`

const HeaderTitleName = styled.span`
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--color-fg);
    font-weight: var(--font-weight-bold);
    text-transform: none;
    letter-spacing: normal;
`

const HeaderActions = styled.div`
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    flex-shrink: 0;
`

const HeaderBtn = styled.button`
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-fg-muted);
    cursor: pointer;
    padding: var(--space-2xs) var(--space-compact);
    font-size: var(--font-size-xs);
    line-height: 1.4;

    &:hover:not(:disabled) {
        background: var(--color-hover-bg);
        color: var(--color-fg);
    }

    &:disabled {
        opacity: 0.35;
        cursor: default;
    }
`

const ScrollBody = styled.div`
    overflow: auto;
    flex: 1;
    padding: var(--space-sm) var(--space-md);
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    min-height: 0;
`

const ExprBlock = styled.div`
    margin-bottom: var(--space-sm);
    padding: var(--space-sm);
    background: var(--color-bg-group);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
    display: flex;
    align-items: baseline;
    gap: var(--space-sm);
    flex-wrap: wrap;
`

const DepsHeading = styled.div`
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-bold);
    color: var(--color-fg-muted);
    text-transform: uppercase;
    letter-spacing: var(--letter-spacing-caps);
    margin-bottom: var(--space-xs);
`

const DepList = styled.ul`
    list-style: none;
    margin: 0;
    padding: 0;
`

const DepRow = styled.li`
    display: grid;
    grid-template-columns: auto auto 1fr auto;
    column-gap: var(--space-sm);
    align-items: baseline;
    padding: var(--space-xs) var(--space-sm);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);

    &:hover {
        background: var(--color-hover-bg);
    }
`

const DepName = styled.span`
    font-weight: var(--font-weight-bold);
    white-space: nowrap;
`

const DepModelTag = styled.span`
    font-size: var(--font-size-xs);
    color: var(--color-fg-subtle);
    white-space: nowrap;
    background: var(--color-bg-group);
    border-radius: var(--radius-sm);
    padding: 0 var(--space-xs);
    line-height: 1.6;
    align-self: center;
`

const DepLabel = styled.span`
    color: var(--color-fg-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
`

const DepValue = styled.span`
    white-space: nowrap;
    color: var(--color-fg);
`

const EmptyMessage = styled.p`
    color: var(--color-fg-subtle);
    font-size: var(--font-size-sm);
    font-style: italic;
`

const ParamDescription = styled.div`
    font-size: var(--font-size-sm);
    color: var(--color-fg-muted);
    margin-bottom: var(--space-xs);
`

const ParamNote = styled.div`
    font-size: var(--font-size-sm);
    color: var(--color-fg-muted);
    padding: var(--space-xs) var(--space-sm);
    border-left: var(--border-accent-width) solid var(--color-border);
    margin-bottom: var(--space-sm);
`

// ── Helpers ───────────────────────────────────────────────────────────────────

/**
 * Extracts the immediate parent model alias from a parameter key.
 * - `"thrust"` → `null` (root param, no context)
 * - `"engine/thrust"` → `"engine"`
 * - `"stage1/engine/thrust"` → `"engine"` (immediate parent)
 * - `"sensor/brightness"` → `"sensor"` (ref alias)
 */
function modelAliasFromKey(key: string): string | null {
    const segments = key.split("/")
    return segments.length > 1 ? segments[segments.length - 2] : null
}

/**
 * Builds the LaTeX string for a dep's name, including the model alias as a
 * subscript tag when the dep belongs to a submodel or reference pool entry.
 * Mirrors the rendering used by `ExprDisplay` for `Variable::External`.
 */
function depNameLatex(dep: DepLabelEntry): string | null {
    const alias = modelAliasFromKey(dep.key)
    if (!alias) return null
    if (dep.renderName) {
        const safeAlias = alias.replace(/_/g, "\\_")
        return `${dep.renderName}_{\\text{[${safeAlias}]}}`
    }
    return mathNameWithRef(dep.name, alias)
}

// ── Component ─────────────────────────────────────────────────────────────────

interface ParameterDetailsPanelProps {
    /**
     * Passed through to the outer container so the parent layout can control
     * flex sizing without needing a variant prop. Typically `{ flex: 1 }` for
     * the bottom layout and `{ flex: n }` for the resizable side layout.
     */
    style?: React.CSSProperties
}

/**
 * Self-contained equation detail section. Owns all navigation callbacks and
 * reads its state directly from atoms.
 */
export function ParameterDetailsPanel({ style }: ParameterDetailsPanelProps) {
    const panelState = useAtomValue(detailPanelAtom)
    const backStack = useAtomValue(detailPanelBackStackAtom)
    const forwardStack = useAtomValue(detailPanelForwardStackAtom)
    const fullParamLookup = useAtomValue(fullParamLookupAtom)
    const paramLookup = useAtomValue(paramLookupAtom)
    const aliasToModelPath = useAtomValue(aliasToModelPathAtom)
    const refPoolAliases = useAtomValue(refPoolAliasesAtom)
    const reverseDeps = useAtomValue(reverseDepsAtom)
    const focusedNode = useAtomValue(focusedNodeAtom)
    const rootNode = useAtomValue(instanceTreeAtom)

    const setFocusedParam = useSetAtom(focusedParamKeyAtom)
    const navigateDetailPanel = useSetAtom(navigateDetailPanelAtom)
    const navigateBack = useSetAtom(navigateDetailBackAtom)
    const navigateForward = useSetAtom(navigateDetailForwardAtom)

    const [focusedPath, setFocusedPath] = useAtom(focusedPathAtom)

    const contextParams = (focusedNode ?? rootNode)?.parameters

    /** Parameters that directly reference the currently focused parameter. */
    const dependents: DepLabelEntry[] = useMemo(() => {
        if (!panelState) return []
        const depKeys = reverseDeps.get(panelState.key)
        if (!depKeys) return []
        return [...depKeys].flatMap((k) => {
            const entry = paramLookup.get(k)
            return entry ? [{ key: k, ...entry }] : []
        })
    }, [panelState, reverseDeps, paramLookup])

    const closeEquation = useCallback(() => navigateDetailPanel(null), [navigateDetailPanel])

    const scrollAndFlash = useCallback(
        (key: string) => {
            const el = document.querySelector(`[data-param-key="${key}"]`)
            if (el) {
                el.scrollIntoView({ behavior: "smooth", block: "center" })
                setFocusedParam(key)
                setTimeout(() => setFocusedParam(null), 1500)
            }
        },
        [setFocusedParam],
    )

    /**
     * Navigates the instance tree to the model that owns the given parameter,
     * then scrolls to and flashes it. When the destination is already the
     * currently focused model the scroll is immediate; otherwise it is deferred
     * by one frame so React has time to re-render the new model's parameters
     * into the DOM before the querySelector runs.
     */
    const navigateToParam = useCallback(
        (key: string, instancePath: string[]) => {
            const alreadyThere = pathsEqual(focusedPath, instancePath)
            if (!alreadyThere) {
                setFocusedPath(instancePath)
                setTimeout(() => scrollAndFlash(key), 50)
            } else {
                scrollAndFlash(key)
            }
        },
        [focusedPath, setFocusedPath, scrollAndFlash],
    )

    const handleBack = useCallback(() => {
        if (backStack.length === 0) return
        const target = backStack[backStack.length - 1]
        navigateBack()
        navigateToParam(target.key, target.instancePath)
    }, [backStack, navigateBack, navigateToParam])

    const handleForward = useCallback(() => {
        if (forwardStack.length === 0) return
        const target = forwardStack[forwardStack.length - 1]
        navigateForward()
        navigateToParam(target.key, target.instancePath)
    }, [forwardStack, navigateForward, navigateToParam])

    const handleDepClick = useCallback(
        (dep: DepLabelEntry) => {
            const fullEntry = fullParamLookup.get(dep.key)
            if (!fullEntry) return

            navigateToParam(dep.key, fullEntry.instancePath)

            const depKeys = extractDependencyKeys(fullEntry.expression, fullEntry.instancePath, aliasToModelPath, refPoolAliases)
            const depDeps: DepLabelEntry[] = [...depKeys].flatMap((k) => {
                const entry = paramLookup.get(k)
                return entry ? [{ key: k, ...entry }] : []
            })
            navigateDetailPanel({
                key: dep.key,
                paramName: fullEntry.name,
                paramRenderName: fullEntry.renderName,
                paramLabel: fullEntry.label,
                note: fullEntry.note,
                expression: fullEntry.expression,
                value: fullEntry.value,
                deps: depDeps,
                instancePath: fullEntry.instancePath,
            })
        },
        [navigateToParam, fullParamLookup, paramLookup, aliasToModelPath, refPoolAliases, navigateDetailPanel],
    )

    if (!panelState) return null

    return (
        <DetailsContainer style={style}>
            <Header>
                <HeaderTitle>
                    Equation —&nbsp;
                    <HeaderTitleName>
                        <NameDisplay name={panelState.paramName} renderName={panelState.paramRenderName} />
                    </HeaderTitleName>
                </HeaderTitle>
                <HeaderActions>
                    <HeaderBtn
                        onClick={handleBack}
                        disabled={backStack.length === 0}
                        title={backStack.length > 0
                            ? `Back to ${backStack[backStack.length - 1].paramLabel || backStack[backStack.length - 1].paramName}`
                            : "No history"}
                    >
                        ←
                    </HeaderBtn>
                    <HeaderBtn
                        onClick={handleForward}
                        disabled={forwardStack.length === 0}
                        title={forwardStack.length > 0
                            ? `Forward to ${forwardStack[forwardStack.length - 1].paramLabel || forwardStack[forwardStack.length - 1].paramName}`
                            : "No forward history"}
                    >
                        →
                    </HeaderBtn>
                    <HeaderBtn onClick={closeEquation} title="Deselect parameter">
                        ✕
                    </HeaderBtn>
                </HeaderActions>
            </Header>
            <ScrollBody>
                {panelState.paramLabel && (
                    <ParamDescription>{panelState.paramLabel}</ParamDescription>
                )}
                <ExprBlock>
                    <NameDisplay name={panelState.paramName} renderName={panelState.paramRenderName} />
                    <span>=</span>
                    {panelState.expression && !isSimpleLiteral(panelState.expression) && (
                        <>
                            <ExprDisplay expr={panelState.expression} instancePath={panelState.instancePath} />
                            <span>=</span>
                        </>
                    )}
                    <ValueDisplay value={panelState.value} />
                </ExprBlock>
                {panelState.note && (
                    <ParamNote>
                        <NoteDisplay text={panelState.note} parameters={contextParams} />
                    </ParamNote>
                )}
                {panelState.deps.length > 0 ? (
                    <>
                        <DepsHeading>Dependencies ({panelState.deps.length})</DepsHeading>
                        <DepList>
                            {panelState.deps.map((dep) => {
                                const modelAlias = modelAliasFromKey(dep.key)
                                return (
                                    <DepRow
                                        key={dep.key}
                                        onClick={() => handleDepClick(dep)}
                                        title={`Jump to ${dep.label}`}
                                    >
                                        <DepName>
                                            <NameDisplay name={dep.name} renderName={depNameLatex(dep) ?? dep.renderName} />
                                        </DepName>
                                        {modelAlias ? <DepModelTag>{modelAlias}</DepModelTag> : <span />}
                                        <DepLabel>{dep.label}</DepLabel>
                                        <DepValue><ValueDisplay value={dep.value} /></DepValue>
                                    </DepRow>
                                )
                            })}
                        </DepList>
                    </>
                ) : (
                    <EmptyMessage>No dependencies for this expression.</EmptyMessage>
                )}
                {dependents.length > 0 && (
                    <>
                        <DepsHeading>Used by ({dependents.length})</DepsHeading>
                        <DepList>
                            {dependents.map((dep) => {
                                const modelAlias = modelAliasFromKey(dep.key)
                                return (
                                    <DepRow
                                        key={dep.key}
                                        onClick={() => handleDepClick(dep)}
                                        title={`Jump to ${dep.label}`}
                                    >
                                        <DepName>
                                            <NameDisplay name={dep.name} renderName={depNameLatex(dep) ?? dep.renderName} />
                                        </DepName>
                                        {modelAlias ? <DepModelTag>{modelAlias}</DepModelTag> : <span />}
                                        <DepLabel>{dep.label}</DepLabel>
                                        <DepValue><ValueDisplay value={dep.value} /></DepValue>
                                    </DepRow>
                                )
                            })}
                        </DepList>
                    </>
                )}
            </ScrollBody>
        </DetailsContainer>
    )
}
