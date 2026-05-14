import { describe, expect, it } from "vitest"
import type { ExprAst, ParameterValueAst, RenderedChild, RenderedNode, RenderedParameter } from "../../types/model"
import { computeUsedParams } from "../computeUsedParams"

const span = {}

function exprAst(e: ExprAst): ParameterValueAst {
    return { Simple: [e, null] }
}

function paramRef(name: string): ExprAst {
    return { Variable: { span, variable: { Parameter: { parameter_name: name, parameter_span: span } } } }
}

function extRef(alias: string, param: string): ExprAst {
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

function binMul(a: ExprAst, b: ExprAst): ParameterValueAst {
    return exprAst({ BinaryOp: { span, op: "mul", left: a, right: b } })
}

function literalNum(n: number): ExprAst {
    return { Literal: { span, value: { Number: n } } }
}

function mkParam(
    name: string,
    expression: ParameterValueAst | null,
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
                mkParam("pressure", exprAst(literalNum(2))),
                mkParam("area", exprAst(literalNum(3))),
            ]),
        }
        const root = mkNode([], [
            mkParam("total", exprAst(extRef("engine", "thrust")), "performance"),
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
                mkParam("pressure", exprAst(literalNum(2))),
                mkParam("area", exprAst(literalNum(3))),
            ]),
        }
        const root = mkNode([], [
            mkParam("total", exprAst(extRef("engine", "thrust")), "performance"),
        ], [engineChild])

        const emptyAliases = new Map<string, string>()
        const t = computeUsedParams(root, emptyAliases, { mode: "transitive", referencePool: [] })
        expect(t.usedParamKeys.has("engine/pressure")).toBe(true)
        expect(t.usedParamKeys.has("engine/area")).toBe(true)
    })

    it("direct_submodel keeps all root-level params in the transitive closure", () => {
        const root = mkNode([], [
            mkParam("out", exprAst({ BinaryOp: { span, op: "add", left: paramRef("a"), right: paramRef("b") } }), "performance"),
            mkParam("a", exprAst(literalNum(1))),
            mkParam("b", exprAst(literalNum(2))),
        ])

        const emptyAliases = new Map<string, string>()
        const direct = computeUsedParams(root, emptyAliases, { mode: "direct_submodel", referencePool: [] })
        expect(direct.usedParamKeys.has("out")).toBe(true)
        expect(direct.usedParamKeys.has("a")).toBe(true)
        expect(direct.usedParamKeys.has("b")).toBe(true)
    })

    it("direct_submodel includes root params even when not reachable from outputs", () => {
        const root = mkNode([], [
            mkParam("out", exprAst(literalNum(1)), "performance"),
            mkParam("orphan", exprAst(literalNum(99))),
        ])

        const direct = computeUsedParams(root, new Map(), { mode: "direct_submodel", referencePool: [] })
        expect(direct.usedParamKeys.has("out")).toBe(true)
        expect(direct.usedParamKeys.has("orphan")).toBe(true)
    })
})
