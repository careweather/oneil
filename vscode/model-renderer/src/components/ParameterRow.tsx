/**
 * Shared parameter row rendering components used by both InstanceTree and ModelGraph views.
 * The only difference between views is how submodels/references are handled.
 */
import type React from "react"
import styled, { css } from "styled-components"
import type { ParameterValueAst, RenderedParameter, RenderedValue } from "../types/model"
import { fmtNum, isSimpleLiteral, mathName, paramExprOnlyToLatex } from "../utils/exprToLatex"
import katex from "katex"
import { useCallback, useMemo } from "react"
import { useAtomValue } from "jotai"
import { renderNameLookupAtom, type EffectiveParamLayout } from "../store/atoms"

export type { EffectiveParamLayout }

// ── Shared style fragments ────────────────────────────────────────────────────

/** Applied to name spans when the param has dependency links. */
const hasDepsStyles = css`
    cursor: pointer;
    text-decoration-line: underline;
    text-decoration-style: dotted;
    text-decoration-color: var(--color-fg-muted);
    text-underline-offset: var(--text-underline-offset);
    pointer-events: auto !important;

    &:hover {
        text-decoration-style: solid;
        text-decoration-color: var(--color-accent-blue);
    }
`

/** Applied to expression spans that open the detail panel on click. */
const exprClickableStyles = css`
    cursor: pointer;
    pointer-events: auto !important;
    border-radius: var(--radius-sm);
    padding: 0 var(--padding-inline-expr);
    border: 1px solid color-mix(in srgb, var(--color-accent-blue) 40%, transparent);
    transition: var(--transition-expr-hover);

    &:hover {
        background-color: color-mix(in srgb, var(--color-accent-blue) 15%, transparent);
        border-color: color-mix(in srgb, var(--color-accent-blue) 75%, transparent);
        text-decoration: underline;
        text-decoration-style: dotted;
        text-underline-offset: var(--text-underline-offset);
    }
`

// ── white-space: nowrap rationale ────────────────────────────────────────────
//
// The classic and new layouts use CSS grid with `auto` tracks for the name,
// expression, and value columns. A CSS `auto` track sizes to the *minimum*
// content size of its cells — for wrappable text that minimum collapses to a
// single character wide. `white-space: nowrap` makes min-content equal
// max-content, so each `auto` column sizes to the full rendered width instead.
//
// `nowrap` on KaTeX cells also prevents the browser from introducing line
// breaks inside a formula, and keeps "9.81 m/s²"-style values together.
//
// The "=" separator cells (ParamEq, ParamEqSecondary, ParamEqRendered) are
// single characters that can never be broken; their `nowrap` is redundant but
// kept for consistency.

// ── Classic layout cells ──────────────────────────────────────────────────────

/** Outer label+expression flex container (classic layout, col 1). */
export const ParamLabelExpr = styled.span`
    display: flex;
    align-items: baseline;
    min-width: 50px;
    padding: 0.1em 0;
`

const ParamLabelText = styled.span`
    color: var(--color-fg-muted);
    overflow-wrap: break-word;
    word-break: break-word;
    hyphens: auto;
    flex-shrink: 1;
    min-width: 0;
`

const ParamClassicSpacer = styled.span`
    flex: 1;
    min-width: var(--space-sm);
`

/** Expression text inside the label flex container (classic layout). */
export const ParamExprText = styled.span<{ $clickable?: boolean }>`
    color: var(--color-fg-muted);
    line-height: 1.4;
    flex-shrink: 0;
    text-align: right;

    ${({ $clickable }) => $clickable && exprClickableStyles}
`

const ParamSep = styled.span`
    color: var(--color-fg-muted);
`

/** Value container (classic layout, col 3). Holds "name = value" as one unit. */
const ParamValueClassic = styled.span`
    white-space: nowrap; /* auto col: keep "name = value" from splitting */
`

/** Parameter name symbol (shared: classic layout, inside ParamValueClassic). */
const ParamName = styled.span<{ $hasDeps?: boolean }>`
    font-weight: var(--font-weight-bold);
    color: var(--color-fg);

    ${({ $hasDeps }) => $hasDeps && hasDepsStyles}
`

// ── New layout cells ──────────────────────────────────────────────────────────

/** Label cell (new layout, col 1). */
export const ParamLabelNew = styled.span`
    color: var(--color-fg-muted);
    padding: 0.1em 0;
    overflow-wrap: break-word;
    word-break: break-word;
    hyphens: auto;
`

/** 1fr spacer between label and name (new layout, col 2). */
const ParamSpacer = styled.span``

/** Name symbol (new layout, col 3, right-aligned). */
const ParamNameNew = styled.span<{ $hasDeps?: boolean }>`
    font-weight: var(--font-weight-bold);
    color: var(--color-fg);
    white-space: nowrap; /* auto col: size to full KaTeX symbol width */
    text-align: right;

    ${({ $hasDeps }) => $hasDeps && hasDepsStyles}
`

/** Primary "=" (new layout, col 4). */
const ParamEq = styled.span`
    color: var(--color-fg-muted);
    white-space: nowrap; /* redundant (single char), kept for consistency */
`

/** Secondary "=" between expression and value (new layout, col 6). */
const ParamEqSecondary = styled.span`
    color: var(--color-fg-muted);
    white-space: nowrap; /* redundant (single char), kept for consistency */
`

/** Expression cell (new layout, col 5). */
export const ParamExprNew = styled.span<{ $clickable?: boolean }>`
    color: var(--color-fg-muted);
    line-height: 1.4;
    white-space: nowrap; /* auto col: size to full KaTeX expression width */

    ${({ $clickable }) => $clickable && exprClickableStyles}
`

/** Value cell (new layout, col 7). */
const ParamValueNew = styled.span`
    white-space: nowrap; /* auto col: keep number+unit together */
`

/** Inline note row spanning all columns (new/classic layouts). */
export const ParamNoteLine = styled.span`
    grid-column: 1 / -1;
    padding-left: 2em;
    text-align: left;
`

// ── Rendered layout cells ─────────────────────────────────────────────────────

/** Label row (rendered layout). */
const ParamLabelRendered = styled.div`
    color: var(--color-fg-muted);
    padding: 0.1em 0;
    overflow-wrap: break-word;
    word-break: break-word;
    hyphens: auto;
`

/** Centered name = [expr =] value line (rendered layout). */
const ParamCenterLine = styled.div`
    display: flex;
    justify-content: center;
    align-items: baseline;
    gap: 0.35em;
    flex-wrap: wrap;
    color: var(--color-fg-subtle);
    font-style: italic;
    font-size: var(--font-size-sm);
    line-height: 1.6;
`

/** Name symbol (rendered layout). */
const ParamNameRendered = styled.span<{ $hasDeps?: boolean }>`
    color: var(--color-fg);
    white-space: nowrap; /* treat KaTeX output as a single flex-wrap item */

    ${({ $hasDeps }) => $hasDeps && hasDepsStyles}
`

/** "=" separator (rendered layout). */
const ParamEqRendered = styled.span`
    color: var(--color-fg-muted);
    white-space: nowrap; /* redundant (single char), kept for consistency */
    font-style: normal;
`

/** Expression (rendered layout). */
const ParamExprRendered = styled.span<{ $clickable?: boolean }>`
    white-space: nowrap; /* treat KaTeX output as a single flex-wrap item */

    ${({ $clickable }) => $clickable && exprClickableStyles}
`

/** Evaluated value (rendered layout). */
const ParamValueRendered = styled.span`
    white-space: nowrap; /* keep number+unit together as a single flex-wrap item */
    color: var(--color-fg);
    font-style: normal;
`

/** Inline note (rendered layout). */
const ParamNoteRendered = styled.div`
    padding: 0;
    text-align: left;
`

// ── ParameterRowLi ────────────────────────────────────────────────────────────

/**
 * The `<li>` wrapper for a parameter row. Uses `display: contents` so its
 * children participate directly in the parent CSS grid. State props drive
 * highlighted, performance, and focus-flash styles on the child cells.
 */
export const ParameterRowLi = styled.li<{
    $highlighted?: boolean
    $performance?: boolean
    $focused?: boolean
}>`
    display: contents;

    ${({ $highlighted }) =>
        $highlighted &&
        css`
            & > ${ParamLabelExpr},
            & > ${ParamLabelNew},
            & > ${ParamNameNew},
            & > ${ParamValueClassic},
            & > ${ParamValueNew},
            & > ${ParamLabelRendered},
            & > ${ParamCenterLine} {
                background-color: color-mix(in srgb, var(--color-accent-blue) 20%, transparent);
                border-radius: var(--radius-sm);
            }
        `}

    ${({ $performance }) =>
        $performance &&
        css`
            & > ${ParamLabelExpr},
            & > ${ParamLabelNew},
            & > ${ParamLabelRendered} {
                color: var(--color-accent-yellow);
            }

            // & > ${ParamLabelExpr}::before,
            // & > ${ParamLabelNew}::before,
            // & > ${ParamLabelRendered}::before {
            //     content: "★ ";
            //     color: var(--color-accent-yellow);
            // }

            & > ${ParamName},
            & > ${ParamNameNew},
            & > ${ParamValueClassic},
            & > ${ParamValueNew},
            & > ${ParamValueClassic} ${ParamName},
            & > ${ParamCenterLine} ${ParamNameRendered},
            & > ${ParamCenterLine} ${ParamValueRendered} {
                color: var(--color-accent-yellow);
            }
        `}

    ${({ $focused }) =>
        $focused &&
        css`
            animation: focus-flash 1.5s ease-out;

            & > ${ParamLabelExpr},
            & > ${ParamLabelNew},
            & > ${ParamLabelRendered},
            & > ${ParamCenterLine} {
                animation: focus-flash 1.5s ease-out;
                border-radius: var(--radius-sm);
            }
        `}
`

// ── ValueDisplay ──────────────────────────────────────────────────────────────

/**
 * Formats a number to at most 4 significant figures, trimming trailing zeros
 * so that `9.810` → `"9.81"` and `5.97e24` → `"5.97e+24"`.
 */

/** Renders a `RenderedValue` union as a human-readable string. */
export function ValueDisplay({ value }: { value: RenderedValue }) {
    switch (value.type) {
        case "boolean":
            return <>{String(value.value)}</>
        case "string":
            return <>&ldquo;{value.value}&rdquo;</>
        case "number":
            return value.max !== null
                ? <>[{fmtNum(value.value)}, {fmtNum(value.max)}]</>
                : <>{fmtNum(value.value)}</>
        case "measured_number": {
            const unit = value.unit === "1" ? null : value.unit
            return value.max !== null
                ? <>[{fmtNum(value.value)}, {fmtNum(value.max)}]{unit && <> {unit}</>}</>
                : <>{fmtNum(value.value)}{unit && <> {unit}</>}</>
        }
    }
}

// ── ExprDisplay ───────────────────────────────────────────────────────────────

/**
 * Renders a `ParameterValueAst` using KaTeX in MathML mode.
 *
 * MathML output lets the browser's native math layout engine handle subscript
 * and superscript positioning, which avoids the vlist/inline-table alignment
 * issues that KaTeX HTML output suffers inside flex containers.  Chromium
 * (used by VS Code webviews) has full MathML support.
 *
 * When `instancePath` is provided, parameter variable references inside the
 * expression are rendered using their render-name (if defined) rather than
 * the auto-derived `mathName`.
 */
export function ExprDisplay({ expr, instancePath }: { expr: ParameterValueAst; instancePath?: string[] }) {
    const globalLookup = useAtomValue(renderNameLookupAtom)
    const html = useMemo(() => {
        const lookup = instancePath
            ? (paramName: string, refAlias?: string) => globalLookup(instancePath, paramName, refAlias)
            : undefined
        const latex = paramExprOnlyToLatex(expr, lookup)
        if (!latex) return null
        try {
            return katex.renderToString(latex, {
                output: "mathml",
                throwOnError: false,
                strict: false,
            })
        } catch {
            return null
        }
    }, [expr, globalLookup, instancePath])

    if (!html) return null
    return <span dangerouslySetInnerHTML={{ __html: html }} />
}

// ── DepLabelEntry ─────────────────────────────────────────────────────────────

export interface DepLabelEntry {
    /** The unique parameter key used for lookup in the full param lookup table. */
    key: string
    name: string
    /** Optional LaTeX render-name; used instead of auto-deriving from `name` when set. */
    renderName: string | null
    label: string
    value: RenderedValue
}

// ── NameDisplay ───────────────────────────────────────────────────────────────

/**
 * Renders a parameter symbol as a KaTeX math expression.
 *
 * When `renderName` is provided it is used as raw LaTeX directly (e.g. `\hat{v}`
 * from the `{...}` render-name block in source).  Otherwise the identifier string
 * is converted to LaTeX automatically via `mathName`.
 */
export function NameDisplay({ name, renderName }: { name: string; renderName?: string | null }) {
    const latex = renderName ?? mathName(name)
    const html = useMemo(() => {
        try {
            return katex.renderToString(latex, {
                output: "mathml",
                throwOnError: false,
                strict: false,
            })
        } catch {
            return null
        }
    }, [latex])

    if (!html) return <>{name}</>
    return <span dangerouslySetInnerHTML={{ __html: html }} />
}

// ── Shared row content props ──────────────────────────────────────────────────

export interface ParamRowContentProps {
    /** Layout format */
    layout: EffectiveParamLayout
    /** Parameter name (identifier) */
    name: string
    /** Optional raw LaTeX render-name; overrides auto-derived symbol when set */
    renderName?: string | null
    /** Human-readable label */
    label: string
    /** Optional expression AST */
    expression: ParameterValueAst | null
    /** Evaluated value */
    value: RenderedValue
    /** Optional note content */
    note: string | null
    /** Whether to show notes inline */
    showNotes: boolean
    /** Whether this param has dependencies (for hover highlighting) */
    hasDeps: boolean
    /** Handlers for name hover (dependency highlighting) */
    onNameMouseEnter: () => void
    onNameMouseLeave: () => void
    /** Style for design attribution (border-left etc) */
    labelStyle: React.CSSProperties
    /** Tooltip props for label/row */
    tooltipProps: {
        className?: string
        onMouseEnter?: React.MouseEventHandler<Element>
        onMouseLeave?: () => void
    }
    /** Note display component */
    NoteDisplay: React.ComponentType<{ text: string; parameters?: RenderedParameter[] }>
    /** Parameters array from the parent node, forwarded to NoteDisplay for placeholder resolution. */
    parameters?: RenderedParameter[]
    /**
     * Styled-component (or any element type) used to wrap inline notes in
     * new/classic layouts. Defaults to `ParamNoteLine`. Pass `GraphParamNote`
     * from `NodeContentGrid` when rendering in the graph view.
     */
    NoteWrapper?: React.ElementType
    /** Direct dependencies to show in the detail panel on click. */
    exprDeps?: DepLabelEntry[]
    /** Called when the expression is clicked to open the detail panel. */
    onExprClick?: () => void
    /**
     * Called when the parameter label or name symbol is clicked.
     * Opens the detail panel even for literal parameters with no expression.
     */
    onRowClick?: () => void
    /** Instance path of the node; forwarded to ExprDisplay for render-name lookup. */
    instancePath?: string[]
}

/**
 * Renders the inner content of a parameter row.
 * Works with both classic and new layouts.
 */
export function ParamRowContent({
    layout,
    name,
    renderName,
    label,
    expression,
    value,
    note,
    showNotes,
    hasDeps,
    onNameMouseEnter,
    onNameMouseLeave,
    labelStyle,
    tooltipProps,
    NoteDisplay,
    parameters,
    NoteWrapper = ParamNoteLine,
    onExprClick,
    onRowClick,
    instancePath,
}: ParamRowContentProps) {
    const showExpr = expression != null && !isSimpleLiteral(expression)
    const exprClickable = showExpr && onExprClick != null

    const handleExprClick = useCallback(
        (e: React.MouseEvent) => {
            e.stopPropagation()
            onExprClick?.()
        },
        [onExprClick],
    )

    if (layout === "rendered") {
        // Rendered (tree) layout: label on top, centered name = [expr =] value, note below.
        // The li has display:block so we return a vertical stack of divs.
        return (
            <>
                <ParamLabelRendered
                    className={tooltipProps.className}
                    style={{ ...labelStyle, ...(onRowClick ? { cursor: "pointer" } : {}) }}
                    onMouseEnter={tooltipProps.onMouseEnter}
                    onMouseLeave={tooltipProps.onMouseLeave}
                    onClick={onRowClick}
                >
                    {label}
                </ParamLabelRendered>
                <ParamCenterLine>
                    <ParamNameRendered
                        $hasDeps={hasDeps}
                        style={onRowClick && !hasDeps ? { cursor: "pointer" } : undefined}
                        onMouseEnter={onNameMouseEnter}
                        onMouseLeave={onNameMouseLeave}
                        onClick={onRowClick}
                    >
                        <NameDisplay name={name} renderName={renderName} />
                    </ParamNameRendered>
                    <ParamEqRendered>=</ParamEqRendered>
                    {showExpr && (
                        <>
                            <ParamExprRendered
                                $clickable={exprClickable}
                                onClick={exprClickable ? handleExprClick : undefined}
                            >
                                <ExprDisplay expr={expression} instancePath={instancePath} />
                            </ParamExprRendered>
                            <ParamEqRendered>=</ParamEqRendered>
                        </>
                    )}
                    <ParamValueRendered>
                        <ValueDisplay value={value} />
                    </ParamValueRendered>
                </ParamCenterLine>
                {showNotes && note && (
                    <ParamNoteRendered>
                        <NoteDisplay text={note} parameters={parameters} />
                    </ParamNoteRendered>
                )}
            </>
        )
    }

    if (layout === "new") {
        // 7 grid columns: label | spacer(1fr) | name | = | expr | = | value
        // The spacer pushes name+equation block right; auto cols size to widest cell.
        return (
            <>
                {/* col 1: label */}
                <ParamLabelNew
                    className={tooltipProps.className}
                    style={{ ...labelStyle, ...(onRowClick ? { cursor: "pointer" } : {}) }}
                    onMouseEnter={tooltipProps.onMouseEnter}
                    onMouseLeave={tooltipProps.onMouseLeave}
                    onClick={onRowClick}
                >
                    {label}
                </ParamLabelNew>
                {/* col 2: 1fr spacer */}
                <ParamSpacer />
                {/* col 3: name (right-aligned) */}
                <ParamNameNew
                    $hasDeps={hasDeps}
                    style={onRowClick && !hasDeps ? { cursor: "pointer" } : undefined}
                    onMouseEnter={onNameMouseEnter}
                    onMouseLeave={onNameMouseLeave}
                    onClick={onRowClick}
                >
                    <NameDisplay name={name} renderName={renderName} />
                </ParamNameNew>
                {/* col 4: primary = */}
                <ParamEq>=</ParamEq>
                {/* col 5: expression (left-aligned, empty when simple literal) */}
                <ParamExprNew
                    $clickable={exprClickable}
                    onClick={exprClickable ? handleExprClick : undefined}
                >
                    {showExpr && <ExprDisplay expr={expression} instancePath={instancePath} />}
                </ParamExprNew>
                {/* col 6: secondary = (empty when no expression, keeping column present) */}
                <ParamEqSecondary>{showExpr ? "=" : ""}</ParamEqSecondary>
                {/* col 7: value */}
                <ParamValueNew>
                    <ValueDisplay value={value} />
                </ParamValueNew>
                {/* note: grid-column 1/-1, appears on its own row */}
                {showNotes && note && (
                    <NoteWrapper>
                        <NoteDisplay text={note} parameters={parameters} />
                    </NoteWrapper>
                )}
            </>
        )
    }

    // Classic layout: [label ··· expression] : name = value
    // col 1 (1fr): flex row — label | spacer | expr; col 2: :; col 3: name = value
    return (
        <>
            <ParamLabelExpr
                className={tooltipProps.className}
                style={{ ...labelStyle, ...(onRowClick ? { cursor: "pointer" } : {}) }}
                onMouseEnter={tooltipProps.onMouseEnter}
                onMouseLeave={tooltipProps.onMouseLeave}
                onClick={onRowClick}
            >
                <ParamLabelText>{label}</ParamLabelText>
                <ParamClassicSpacer />
                {showExpr && (
                    <ParamExprText
                        $clickable={exprClickable}
                        onClick={exprClickable ? handleExprClick : undefined}
                    >
                        <ExprDisplay expr={expression} instancePath={instancePath} />
                    </ParamExprText>
                )}
            </ParamLabelExpr>
            <ParamSep>:</ParamSep>
            <ParamValueClassic>
                <ParamName
                    $hasDeps={hasDeps}
                    style={onRowClick && !hasDeps ? { cursor: "pointer" } : undefined}
                    onMouseEnter={onNameMouseEnter}
                    onMouseLeave={onNameMouseLeave}
                    onClick={onRowClick}
                >
                    <NameDisplay name={name} renderName={renderName} />
                </ParamName>
                {" = "}
                <ValueDisplay value={value} />
            </ParamValueClassic>
            {showNotes && note && (
                <NoteWrapper>
                    <NoteDisplay text={note} parameters={parameters} />
                </NoteWrapper>
            )}
        </>
    )
}

