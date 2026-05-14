/**
 * Design badge component — renders a single design overlay attribution.
 *
 * Used wherever applied designs are listed: the model hierarchy TOC,
 * the instance tree, and the graph view card headers.
 */
import styled from "styled-components"
import { designColorVar } from "../utils/designColors"
import type { AppliedDesign } from "../types/model"

const DesignBadgeSpan = styled.span`
    font-size: var(--font-size-xs);
    font-weight: normal;
    padding: var(--space-hairline) var(--space-compact);
    border-radius: var(--radius-sm);
    border: 1px solid currentColor;
    opacity: 0.85;
`

/**
 * Renders the name of an applied design in a coloured pill badge.
 *
 * @example
 * ```tsx
 * {node.applied_designs.map((d) => <DesignBadge key={d.design_name} design={d} />)}
 * ```
 */
export function DesignBadge({ design }: { design: AppliedDesign }) {
    return (
        <DesignBadgeSpan
            style={{
                color: designColorVar(design.color_index),
                borderColor: designColorVar(design.color_index),
            }}
        >
            {design.design_name}
        </DesignBadgeSpan>
    )
}
