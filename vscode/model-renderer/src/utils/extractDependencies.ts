/**
 * Extracts variable dependencies from an expression AST and provides
 * utilities for creating consistent parameter keys.
 *
 * Key format (unified `/` separator — see `instancePath.ts`):
 * - All params: `"rover/arm/thrust"` (or just `"thrust"` at root)
 * - Ref pool params: same format, e.g. `"sensor/optics/fov"` where `sensor`
 *   is the ref alias (the backend includes it as the first path segment)
 *
 * For dependencies extracted from expressions:
 * - `Variable::Parameter` → same instance_path as the containing param
 * - `Variable::External` → resolved via alias → model_path mapping
 */

import type { ExprAst, ParameterValueAst, PiecewiseExprAst, RenderedNode } from "../types/model"
import { paramKey } from "./instancePath"

// Re-export for convenience
export { paramKey }

/**
 * Extracts all dependency keys from a bare test expression (`ExprAst`).
 *
 * Identical semantics to `extractDependencyKeys` but for test expressions,
 * which are raw `ExprAst` nodes rather than `ParameterValueAst` wrappers.
 */
export function extractDepsFromExpr(
    expr: ExprAst | null,
    instancePath: string[],
    aliasToModelPath: Map<string, string>,
    refPoolAliases?: Set<string>,
): Set<string> {
    if (!expr) return new Set()
    return extractDependencyKeys({ Simple: [expr, null] }, instancePath, aliasToModelPath, refPoolAliases)
}

/**
 * Builds a mapping from reference alias → model_path by walking the tree.
 */
export function buildAliasToModelPath(node: RenderedNode): Map<string, string> {
    const map = new Map<string, string>()

    function walk(n: RenderedNode) {
        for (const ref of n.references) {
            map.set(ref.alias, ref.model_path)
        }
        for (const child of n.children) {
            walk(child.node)
        }
    }

    walk(node)
    return map
}

/**
 * Extracts all dependency keys from a parameter's expression.
 *
 * @param expr - The parameter's expression AST
 * @param instancePath - The instance_path of the node containing this parameter
 * @param aliasToModelPath - Mapping from cross-file reference alias to model_path
 *        (does NOT include submodel aliases)
 * @param refPoolAliases - Optional set of ref pool aliases (if not provided, uses aliasToModelPath keys)
 * @returns Set of parameter keys that this expression depends on
 */
export function extractDependencyKeys(
    expr: ParameterValueAst | null,
    instancePath: string[],
    aliasToModelPath: Map<string, string>,
    refPoolAliases?: Set<string>,
): Set<string> {
    const keys = new Set<string>()
    if (!expr) return keys

    const isRefAlias = refPoolAliases
        ? (alias: string) => refPoolAliases.has(alias)
        : (alias: string) => aliasToModelPath.has(alias)

    function walkExpr(e: ExprAst): void {
        if ("Variable" in e) {
            const v = e.Variable.variable
            if ("Parameter" in v) {
                // Local parameter - same instance_path
                keys.add(paramKey(instancePath, v.Parameter.parameter_name))
            } else if ("External" in v) {
                // External reference - could be submodel or cross-file reference
                const { reference_name, parameter_name } = v.External
                if (isRefAlias(reference_name)) {
                    // Cross-file reference (in reference pool) — alias is first path segment
                    keys.add(paramKey([reference_name], parameter_name))
                } else {
                    // Submodel reference - extend instance path with child alias
                    keys.add(paramKey([...instancePath, reference_name], parameter_name))
                }
            }
            // Builtins are ignored
        } else if ("BinaryOp" in e) {
            walkExpr(e.BinaryOp.left)
            walkExpr(e.BinaryOp.right)
        } else if ("UnaryOp" in e) {
            walkExpr(e.UnaryOp.expr)
        } else if ("ComparisonOp" in e) {
            walkExpr(e.ComparisonOp.left)
            walkExpr(e.ComparisonOp.right)
            for (const [, rest] of e.ComparisonOp.rest_chained) {
                walkExpr(rest)
            }
        } else if ("FunctionCall" in e) {
            for (const arg of e.FunctionCall.args) {
                walkExpr(arg)
            }
        } else if ("UnitCast" in e) {
            walkExpr(e.UnitCast.expr)
        }
        // Literals have no dependencies
    }

    function walkPiecewise(pw: PiecewiseExprAst): void {
        walkExpr(pw.expr)
        walkExpr(pw.if_expr)
    }

    if ("Simple" in expr) {
        walkExpr(expr.Simple[0])
    } else if ("Piecewise" in expr) {
        for (const pw of expr.Piecewise[0]) {
            walkPiecewise(pw)
        }
    }

    return keys
}
