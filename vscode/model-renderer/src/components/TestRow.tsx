/**
 * TestRow — renders a single model test (assertion) row.
 *
 * Displays a pass/fail icon, the expression, and an optional note.
 * In the "rendered" layout the note appears as a full block below the
 * expression; in other layouts it appears inline or as a tooltip.
 */
import { useAtomValue, useSetAtom } from "jotai"
import { useCallback, useMemo } from "react"
import katex from "katex"
import styled from "styled-components"
import { useTooltipTrigger } from "./Tooltip"
import { NoteDisplay } from "./NoteDisplay"
import type { ExprAst, RenderedParameter, RenderedTest } from "../types/model"
import {
    aliasToModelPathAtom,
    highlightedDepsAtom,
    showNotesEnabledAtom,
} from "../store/atoms"
import { extractDepsFromExpr } from "../utils/extractDependencies"
import { exprAstToLatex } from "../utils/exprToLatex"

// ── Styled components ─────────────────────────────────────────────────────────

const TestRowWrapper = styled.div<{ $passed: boolean }>`
    display: flex;
    align-items: center;
    gap: 0.4em;
    cursor: default;
    border-radius: var(--radius-sm);
    padding: 0 var(--space-2xs);
    font-size: var(--font-size-sm);

    &:hover {
        background: var(--color-list-hover);
    }
`

const TestIcon = styled.span<{ $passed: boolean }>`
    font-weight: var(--font-weight-bold);
    line-height: 1;
    color: ${({ $passed }) =>
        $passed ? "var(--color-testing-passed)" : "var(--color-testing-failed)"};
`

const TestExpr = styled.span<{ $passed: boolean }>`
    color: ${({ $passed }) =>
        $passed
            ? "var(--color-fg-muted)"
            : "var(--color-testing-failed)"};
`

/** Note row below the test expression, shown when notes are enabled. */
const TestNoteBlock = styled.div`
    padding-left: 1.4em;
    color: var(--color-fg-muted);
    font-style: italic;
    font-size: var(--font-size-xs);
    line-height: 1.5;
    padding-bottom: 0.15em;
`

// ── TestExprDisplay ───────────────────────────────────────────────────────────

function TestExprDisplay({ expr }: { expr: ExprAst }) {
    try {
        const latex = exprAstToLatex(expr)
        const html = katex.renderToString(latex, {
            output: "mathml",
            throwOnError: false,
        })
        return <span dangerouslySetInnerHTML={{ __html: html }} />
    } catch {
        return null
    }
}

// ── TestRow ───────────────────────────────────────────────────────────────────

/**
 * Renders a single test row with pass/fail icon, expression, and optional note.
 */
export function TestRow({
    test,
    index: _index,
    instancePath,
    enableTooltip,
    parameters,
}: {
    test: RenderedTest
    index: number
    instancePath: string[]
    enableTooltip: boolean
    parameters?: RenderedParameter[]
}) {
    const aliasToModelPath = useAtomValue(aliasToModelPathAtom)
    const setHighlightedDeps = useSetAtom(highlightedDepsAtom)
    const showNotes = useAtomValue(showNotesEnabledAtom)
    const showTooltip = enableTooltip && !showNotes // If notes are shown inline, the tooltip is redundant
    const tooltipProps = useTooltipTrigger(showTooltip ? (test.note ?? undefined) : undefined)

    const depKeys = useMemo(
        () => extractDepsFromExpr(test.expression, instancePath, aliasToModelPath),
        [test.expression, instancePath, aliasToModelPath],
    )

    const onMouseEnter = useCallback(() => {
        if (depKeys.size > 0) setHighlightedDeps(depKeys)
    }, [depKeys, setHighlightedDeps])

    const onMouseLeave = useCallback(() => {
        setHighlightedDeps(new Set<string>())
    }, [setHighlightedDeps])

    return (
        <>
            <TestRowWrapper
                $passed={test.passed}
                onMouseEnter={onMouseEnter}
                onMouseLeave={onMouseLeave}
            >
                <TestIcon $passed={test.passed} aria-label={test.passed ? "passed" : "failed"}>
                    {test.passed ? "✓" : "✗"}
                </TestIcon>
                {test.expression != null && (
                    <TestExpr
                        $passed={test.passed}
                        className={tooltipProps.className || undefined}
                        onMouseEnter={tooltipProps.onMouseEnter}
                        onMouseLeave={tooltipProps.onMouseLeave}
                    >
                        <TestExprDisplay expr={test.expression} />
                    </TestExpr>
                )}
            </TestRowWrapper>
            {showNotes && test.note && (
                <TestNoteBlock>
                    <NoteDisplay text={test.note} parameters={parameters} />
                </TestNoteBlock>
            )}
        </>
    )
}
