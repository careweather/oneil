import { useAtomValue, useSetAtom } from "jotai"
import { useCallback, useMemo } from "react"
import ReactFlow, {
    type Viewport,
    Controls,
    Background,
    BackgroundVariant,
    Handle,
    Position,
} from "reactflow"
import "reactflow/dist/style.css"
import styled from "styled-components"
import { useTooltipTrigger } from "../components/Tooltip"
import { NodeContentGrid } from "../components/NodeContentGrid"
import type { RenderedNode, RenderedPoolEntry } from "../types/model"
import { graphZoomAtom, showNotesEnabledAtom } from "../store/atoms"
import { modelDisplayName } from "../utils/modelPath"
import { type MeasureItem, useMeasureContent } from "../components/MeasureContent"
import { NoteDisplay } from "../components/NoteDisplay"
import { DesignBadge } from "../components/DesignBadge"
import {
    type ModelNodeData,
    flattenNodes,
    buildElements,
    buildRefPoolElements,
    LEAF_MIN_W,
    REF_POOL_ID_PREFIX,
} from "../utils/graphLayout"
import { pathToNodeId } from "../utils/instancePath"

// ── Styled components ────────────────────────────────────────────────────────

const GraphContainer = styled.div`
    flex: 1;
    min-height: 0;
    position: relative;
`

const Card = styled.div`
    position: relative;
    border-radius: var(--radius-md);
    padding: var(--space-sm) var(--space-md);
    font-size: var(--font-size-sm);
    width: 100%;
    height: 100%;
    box-sizing: border-box;
`

const LeafCard = styled(Card)`
    background: var(--color-bg);
    border: 1px solid var(--color-border);
`

const GroupCard = styled(Card)`
    background: var(--color-bg-group);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
`

const Header = styled.div`
    margin-bottom: var(--space-xs);
`

const AliasTab = styled.div`
    position: absolute;
    top: -0.7em;
    left: 0.75em;
    padding: 0 0.4em;
    font-size: var(--font-size-xs);
    color: var(--color-fg-muted);
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    line-height: 1.3;
`

const HeaderTop = styled.div<{ $hasAlias?: boolean }>`
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    ${({ $hasAlias }) => $hasAlias && `margin-top: 0.5em;`}
`

const NodeName = styled.div`
    font-weight: var(--font-weight-bold);
`

const Badges = styled.div`
    display: flex;
    gap: var(--space-xs);
    flex-wrap: wrap;
    margin-left: auto;
`

const Note = styled.div`
    color: var(--color-fg-subtle);
    font-size: var(--font-size-sm);
    margin-top: var(--space-2xs);
`


// ── Header component ──────────────────────────────────────────────────────────

// ── Pure (atom-free) content components ──────────────────────────────────────
// These take all display options as props so they can be used both in the
// visible graph nodes and in the hidden measurement container without needing
// a Jotai store in scope.

/** Renders the header section of a model card (name + design badges + note). */
function NodeHeader({
    node,
    alias,
    showNotes,
    enableTooltip = false,
}: {
    node: RenderedNode
    alias: string | null
    showNotes: boolean
    enableTooltip?: boolean
}) {
    const name = modelDisplayName(node.model_path)
    const tooltipProps = useTooltipTrigger(enableTooltip && !showNotes ? node.note : undefined)
    return (
        <Header>
            {alias && <AliasTab>{alias}</AliasTab>}
            <HeaderTop $hasAlias={!!alias}>
                <NodeName
                    className={tooltipProps.className || undefined}
                    onMouseEnter={tooltipProps.onMouseEnter}
                    onMouseLeave={tooltipProps.onMouseLeave}
                >
                    {name}
                </NodeName>
                <Badges>
                    {node.applied_designs.map((d) => (
                        <DesignBadge key={d.design_name} design={d} />
                    ))}
                </Badges>
            </HeaderTop>
            {showNotes && node.note && (
                <Note>
                    <NoteDisplay text={node.note} parameters={node.parameters} />
                </Note>
            )}
        </Header>
    )
}

// ── Measurement-only content component ───────────────────────────────────────

function MeasureNodeContent({ node }: { node: RenderedNode }) {
    const showNotes = useAtomValue(showNotesEnabledAtom)
    return (
        <Card>
            <NodeHeader node={node} alias={null} showNotes={showNotes} />
            <NodeContentGrid node={node} variant="graph" />
        </Card>
    )
}

// ── Custom reactflow node renderers ───────────────────────────────────────────

function LeafModelNode({ data }: { data: ModelNodeData }) {
    const showNotes = useAtomValue(showNotesEnabledAtom)
    return (
        <LeafCard>
            <Handle type="target" position={Position.Top} style={{ opacity: 0 }} />
            <NodeHeader node={data.node} alias={data.alias} showNotes={showNotes} enableTooltip />
            <NodeContentGrid node={data.node} variant="graph" enableTooltip />
            <Handle type="source" position={Position.Bottom} style={{ opacity: 0 }} />
        </LeafCard>
    )
}

function GroupModelNode({ data }: { data: ModelNodeData }) {
    const showNotes = useAtomValue(showNotesEnabledAtom)
    return (
        <GroupCard>
            <NodeHeader node={data.node} alias={data.alias} showNotes={showNotes} enableTooltip />
            <NodeContentGrid node={data.node} variant="graph" enableTooltip />
        </GroupCard>
    )
}

const nodeTypes = { leafModel: LeafModelNode, groupModel: GroupModelNode }

// ── Public component ──────────────────────────────────────────────────────────

interface ModelGraphViewProps {
    node: RenderedNode
    referencePool: RenderedPoolEntry[]
}

export function ModelGraphView({ node, referencePool }: ModelGraphViewProps) {
    const measureItems: MeasureItem[] = useMemo(() => {
        const mainNodes = flattenNodes(node).map((n) => ({
            id: pathToNodeId(n.instance_path),
            element: <MeasureNodeContent node={n} />,
        }))
        const refNodes = referencePool.flatMap((entry) =>
            flattenNodes(entry.node).map((n) => ({
                id: `${REF_POOL_ID_PREFIX}${entry.alias}/${pathToNodeId(n.instance_path)}`,
                element: <MeasureNodeContent node={n} />,
            })),
        )
        return [...mainNodes, ...refNodes]
    }, [node, referencePool])

    const { sizes: contentSizes, container: measureContainer } = useMeasureContent(
        measureItems,
        LEAF_MIN_W,
    )

    const mainNodes = useMemo(() => buildElements(node, contentSizes), [node, contentSizes])

    const refNodes = useMemo(() => {
        const rootNode = mainNodes.find((n) => n.parentId == null)
        const mainTreeWidth = (rootNode?.style?.width as number | undefined) ?? LEAF_MIN_W
        return buildRefPoolElements(referencePool, mainTreeWidth, contentSizes)
    }, [mainNodes, referencePool, contentSizes])

    const nodes = useMemo(() => [...mainNodes, ...refNodes], [mainNodes, refNodes])

    const setGraphZoom = useSetAtom(graphZoomAtom)
    const onMoveEnd = useCallback(
        (_event: MouseEvent | TouchEvent | null, viewport: Viewport) => {
            setGraphZoom(viewport.zoom)
        },
        [setGraphZoom],
    )
    const onInit = useCallback(
        (instance: { getViewport: () => Viewport }) => {
            setTimeout(() => setGraphZoom(instance.getViewport().zoom), 50)
        },
        [setGraphZoom],
    )

    return (
        <GraphContainer>
            {measureContainer}
            <ReactFlow
                nodes={nodes}
                edges={[]}
                nodeTypes={nodeTypes}
                fitView
                nodesDraggable={false}
                nodesConnectable={false}
                elementsSelectable={false}
                proOptions={{ hideAttribution: true }}
                onInit={onInit}
                onMoveEnd={onMoveEnd}
            >
                <Background variant={BackgroundVariant.Dots} />
                <Controls showInteractive={false} />
            </ReactFlow>
        </GraphContainer>
    )
}
