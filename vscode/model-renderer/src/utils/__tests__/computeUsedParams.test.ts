import { describe, expect, it } from "vitest"
import type { Expr, ParameterValue, RenderedChild, RenderedNode, RenderedParameter, Span } from "../../types/model"
import { computeUsedParams } from "../computeUsedParams"

/** Minimal span for AST fixtures; dependency extraction ignores span contents. */
const span: Span = {
    start: { offset: 0, line: 1, column: 1 },
    end: { offset: 0, line: 1, column: 1 },
}

function asParameterValue(e: Expr): ParameterValue {
    return { Simple: [e, null] }
}

function paramRef(name: string): Expr {
    return { Variable: { span, variable: { Parameter: { parameter_name: name, parameter_span: span } } } }
}

function extRef(alias: string, param: string): Expr {
    return {
        Variable: {
            span,
            variable: {
                External: {
                    reference_name: alias,
                    reference_span: span,
                    parameter_name: param,
                    parameter_span: span,
                },
            },
        },
    }
}

function binMul(a: Expr, b: Expr): ParameterValue {
    return asParameterValue({ BinaryOp: { span, op: "mul", left: a, right: b } })
}

function literalNum(n: number): Expr {
    return { Literal: { span, value: { Number: n } } }
}

function mkParam(
    name: string,
    expression: ParameterValue | null,
    print_level: RenderedParameter["print_level"] = "none",
): RenderedParameter {
    return {
        name,
        label: name,
        render_name: null,
        section: null,
        note: null,
        expression,
        value: { type: "number", value: 1, max: null },
        print_level,
        expr_span: { file: null, start: 0, end: 0 },
        design: null,
    }
}

function mkNode(instance_path: string[], parameters: RenderedParameter[], children: RenderedChild[] = []): RenderedNode {
    return {
        model_path: "models/test",
        instance_path,
        note: null,
        parameters,
        tests: [],
        sections: [],
        children,
        references: [],
        applied_designs: [],
    }
}

describe("computeUsedParams", () => {
    it("direct_submodel hides internal chain under a child but keeps parent-referenced params", () => {
        const engineChild: RenderedChild = {
            alias: "engine",
            node: mkNode(["engine"], [
                mkParam("thrust", binMul(paramRef("pressure"), paramRef("area"))),
                mkParam("pressure", asParameterValue(literalNum(2))),
                mkParam("area", asParameterValue(literalNum(3))),
            ]),
        }
        const root = mkNode([], [
            mkParam("total", asParameterValue(extRef("engine", "thrust")), "performance"),
        ], [engineChild])

        const emptyAliases = new Map<string, string>()

        const transitive = computeUsedParams(root, emptyAliases, { mode: "transitive", referencePool: [] })
        expect(transitive.usedParamKeys.has("engine/thrust")).toBe(true)
        expect(transitive.usedParamKeys.has("engine/pressure")).toBe(true)
        expect(transitive.usedParamKeys.has("engine/area")).toBe(true)

        const direct = computeUsedParams(root, emptyAliases, { mode: "direct_submodel", referencePool: [] })
        expect(direct.usedParamKeys.has("total")).toBe(true)
        expect(direct.usedParamKeys.has("engine/thrust")).toBe(true)
        expect(direct.usedParamKeys.has("engine/pressure")).toBe(false)
        expect(direct.usedParamKeys.has("engine/area")).toBe(false)
    })

    it("transitive mode matches full dependency closure under a child", () => {
        const engineChild: RenderedChild = {
            alias: "engine",
            node: mkNode(["engine"], [
                mkParam("thrust", binMul(paramRef("pressure"), paramRef("area"))),
                mkParam("pressure", asParameterValue(literalNum(2))),
                mkParam("area", asParameterValue(literalNum(3))),
            ]),
        }
        const root = mkNode([], [
            mkParam("total", asParameterValue(extRef("engine", "thrust")), "performance"),
        ], [engineChild])

        const emptyAliases = new Map<string, string>()
        const t = computeUsedParams(root, emptyAliases, { mode: "transitive", referencePool: [] })
        expect(t.usedParamKeys.has("engine/pressure")).toBe(true)
        expect(t.usedParamKeys.has("engine/area")).toBe(true)
    })

    it("direct_submodel keeps all root-level params in the transitive closure", () => {
        const root = mkNode([], [
            mkParam("out", asParameterValue({ BinaryOp: { span, op: "add", left: paramRef("a"), right: paramRef("b") } }), "performance"),
            mkParam("a", asParameterValue(literalNum(1))),
            mkParam("b", asParameterValue(literalNum(2))),
        ])

        const emptyAliases = new Map<string, string>()
        const direct = computeUsedParams(root, emptyAliases, { mode: "direct_submodel", referencePool: [] })
        expect(direct.usedParamKeys.has("out")).toBe(true)
        expect(direct.usedParamKeys.has("a")).toBe(true)
        expect(direct.usedParamKeys.has("b")).toBe(true)
    })

    it("direct_submodel includes root params even when not reachable from outputs", () => {
        const root = mkNode([], [
            mkParam("out", asParameterValue(literalNum(1)), "performance"),
            mkParam("orphan", asParameterValue(literalNum(99))),
        ])

        const direct = computeUsedParams(root, new Map(), { mode: "direct_submodel", referencePool: [] })
        expect(direct.usedParamKeys.has("out")).toBe(true)
        expect(direct.usedParamKeys.has("orphan")).toBe(true)
    })
})
