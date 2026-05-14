/**
 * Converts a serialized `ir::ParameterValue` AST into a LaTeX string for
 * KaTeX rendering.
 *
 * All types are driven from the actual Rust serde output — see comments in
 * `types/model.ts` for the key serialization rules.
 *
 * Math-name helpers (`mathName`, `mathNameWithRef`) live in `./mathName`.
 */

import type {
    BinaryOpAst,
    ComparisonOpAst,
    ExprAst,
    FunctionNameAst,
    LiteralAst,
    ParameterValueAst,
    PiecewiseExprAst,
    UnaryOpAst,
    VariableAst,
} from "../types/model"
import { mathName, mathNameWithRef } from "./mathName"
export { mathName, mathNameWithRef } from "./mathName"

function escapeIdent(s: string): string {
    return s.replace(/[#$%&_{}\\^~]/g, (c) => `\\${c}`)
}

// ── Render-name lookup ────────────────────────────────────────────────────────

/**
 * Callback that resolves an optional render-name for a parameter variable.
 *
 * - For same-model (`Parameter`) variables, `refAlias` is `undefined`.
 * - For cross-model (`External`) variables, `refAlias` is the reference alias.
 *
 * Return `null` / `undefined` to fall back to the auto-derived `mathName`.
 */
export type RenderNameLookup = (paramName: string, refAlias?: string) => string | null | undefined

// Module-level slot, populated only during a synchronous render call.
// Safe because JavaScript is single-threaded and all internal render helpers
// are synchronous — the finally block always restores it to undefined.
let _paramLatexNameLookup: RenderNameLookup | undefined

// ── Precedence levels (higher binds tighter) ─────────────────────────────────

const PREC = {
    addSub:  2,
    mul:     3,
    unary:   5,
    pow:     6,
    atom:    7,
} as const

type Prec = number

// ── Public entry points ───────────────────────────────────────────────────────

/**
 * Converts a single `ExprAst` to a LaTeX string.
 *
 * Use this for test expressions, which are bare `Expr` nodes rather than
 * `ParameterValue` wrappers.
 *
 * @example
 * ```ts
 * exprAstToLatex({ ComparisonOp: { span: 0, op: "less_than", left: ..., right: ..., rest_chained: [] } })
 * // → "mass < 100"
 * ```
 */
export function exprAstToLatex(ast: ExprAst): string {
    return exprToLatex(ast, 0)
}

/**
 * Converts a `ParameterValueAst` to a LaTeX string.
 *
 * For `Simple` values renders `expr\,\mathrm{unit}` (unit optional).
 * For `Piecewise` values renders a `\begin{cases}…\end{cases}` block.
 *
 * @example
 * ```ts
 * paramValueToLatex({ Simple: [{ Literal: { span: 0, value: { Number: 9.81 } } }, "m/s^2"] })
 * // → "9.81\\,\\mathrm{m/s^2}"
 * ```
 */
/**
 * Core conversion: renders `ast` to LaTeX, optionally appending the unit
 * suffix. Assumes `_paramLatexNameLookup` is already set by the caller.
 */
function renderParameterAst(ast: ParameterValueAst, withUnit: boolean): string {
    if ("Simple" in ast) {
        const [expr, unit] = ast.Simple
        // Pass 0 as the parent precedence so the top-level expression is never
        // wrapped in extra parentheses — surrounding context requires none.
        const exprLatex = exprToLatex(expr, 0)
        return withUnit && unit != null ? `${exprLatex}\\,\\mathrm{${escapeUnit(unit)}}` : exprLatex
    }
    const [pieces, unit] = ast.Piecewise
    const body = piecewiseToLatex(pieces)
    return withUnit && unit != null ? `${body}\\,\\mathrm{${escapeUnit(unit)}}` : body
}

export function paramValueToLatex(ast: ParameterValueAst, lookup?: RenderNameLookup): string {
    _paramLatexNameLookup = lookup
    try {
        return renderParameterAst(ast, true)
    } finally {
        _paramLatexNameLookup = undefined
    }
}

/**
 * Like `paramValueToLatex` but **omits the unit suffix** from the rendered
 * expression.  Use this when the numeric value (which already carries its
 * unit string) is displayed right next to the equation — there is no need to
 * repeat the unit inside the LaTeX.
 */
export function paramExprOnlyToLatex(ast: ParameterValueAst, lookup?: RenderNameLookup): string {
    _paramLatexNameLookup = lookup
    try {
        return renderParameterAst(ast, false)
    } finally {
        _paramLatexNameLookup = undefined
    }
}

/**
 * Returns `true` when the expression is a bare literal — rendering it as
 * an equation would add no information over the already-displayed value.
 *
 * @example
 * ```ts
 * isSimpleLiteral({ Simple: [{ Literal: { span: 0, value: { Number: 9.81 } } }, null] })
 * // → true
 * ```
 */
export function isSimpleLiteral(ast: ParameterValueAst): boolean {
    if ("Piecewise" in ast) return false
    const [expr] = ast.Simple
    return "Literal" in expr
}

// ── Piecewise ─────────────────────────────────────────────────────────────────

function piecewiseToLatex(pieces: PiecewiseExprAst[]): string {
    const rows = pieces
        .map((p) => `${exprToLatex(p.expr, PREC.atom)} & \\text{if } ${exprToLatex(p.if_expr, PREC.atom)}`)
        .join(" \\\\ ")
    return `\\begin{cases} ${rows} \\end{cases}`
}

// ── Expression ────────────────────────────────────────────────────────────────

function exprToLatex(expr: ExprAst, parentPrec: Prec): string {
    if ("Literal" in expr) {
        return literalToLatex(expr.Literal.value)
    }

    if ("Variable" in expr) {
        return variableToLatex(expr.Variable.variable)
    }

    if ("BinaryOp" in expr) {
        const { op, left, right } = expr.BinaryOp
        return binaryOpToLatex(op, left, right, parentPrec)
    }

    if ("UnaryOp" in expr) {
        const { op, expr: operand } = expr.UnaryOp
        return unaryOpToLatex(op, operand, parentPrec)
    }

    if ("ComparisonOp" in expr) {
        const { op, left, right, rest_chained } = expr.ComparisonOp
        // Build chain: a < b < c
        let latex = `${exprToLatex(left, PREC.atom)} ${comparisonSym(op)} ${exprToLatex(right, PREC.atom)}`
        for (const [chainOp, chainExpr] of rest_chained) {
            latex += ` ${comparisonSym(chainOp)} ${exprToLatex(chainExpr, PREC.atom)}`
        }
        return maybeParen(latex, 1, parentPrec)
    }

    if ("FunctionCall" in expr) {
        const { name, args } = expr.FunctionCall
        return functionCallToLatex(name, args)
    }

    if ("UnitCast" in expr) {
        // Render as: expr\,[\mathrm{unit}]
        const inner = exprToLatex(expr.UnitCast.expr, PREC.atom)
        return `${inner}\\,\\left[\\mathrm{${escapeUnit(expr.UnitCast.unit)}}\\right]`
    }

    return "?"
}

// ── Literals ──────────────────────────────────────────────────────────────────

function literalToLatex(lit: LiteralAst): string {
    if ("Number" in lit) return formatNumber(lit.Number)
    if ("String" in lit) return `\\text{${escapeText(lit.String)}}`
    return `\\text{${lit.Boolean}}`
}

// ── Fraction exponent ─────────────────────────────────────────────────────────

/**
 * If `expr` is a numeric literal that can be expressed as a simple fraction
 * (denominator up to 12), returns the LaTeX fraction string. Otherwise `null`.
 */
function fractionExponent(expr: ExprAst): string | null {
    if (!("Literal" in expr)) return null
    const lit = expr.Literal.value
    if (!("Number" in lit)) return null
    const n = lit.Number
    if (Number.isInteger(n)) return null

    const MAX_DENOM = 12
    for (let d = 2; d <= MAX_DENOM; d++) {
        const num = n * d
        if (Math.abs(num - Math.round(num)) < 1e-9) {
            const intNum = Math.round(num)
            if (intNum === 0) return null
            return `${intNum}/${d}`
        }
    }
    return null
}

// ── Binary operators ──────────────────────────────────────────────────────────

function binaryOpToLatex(op: BinaryOpAst, left: ExprAst, right: ExprAst, parentPrec: Prec): string {
    switch (op) {
        case "add": {
            const inner = `${exprToLatex(left, PREC.addSub)} + ${exprToLatex(right, PREC.addSub)}`
            return maybeParen(inner, PREC.addSub, parentPrec)
        }
        case "sub":
        case "escaped_sub": {
            const inner = `${exprToLatex(left, PREC.addSub)} - ${exprToLatex(right, PREC.addSub + 1)}`
            return maybeParen(inner, PREC.addSub, parentPrec)
        }
        case "mul": {
            const inner = `${exprToLatex(left, PREC.mul)} \\cdot ${exprToLatex(right, PREC.mul)}`
            return maybeParen(inner, PREC.mul, parentPrec)
        }
        case "div":
        case "escaped_div": {
            return `\\frac{${exprToLatex(left, PREC.atom)}}{${exprToLatex(right, PREC.atom)}}`
        }
        case "mod": {
            const inner = `${exprToLatex(left, PREC.mul)} \\bmod ${exprToLatex(right, PREC.mul)}`
            return maybeParen(inner, PREC.mul, parentPrec)
        }
        case "pow": {
            const expLatex = fractionExponent(right) ?? exprToLatex(right, PREC.atom)
            return `{${exprToLatex(left, PREC.pow)}}^{${expLatex}}`
        }
        case "and": {
            const inner = `${exprToLatex(left, PREC.addSub)} \\land ${exprToLatex(right, PREC.addSub)}`
            return maybeParen(inner, PREC.addSub, parentPrec)
        }
        case "or": {
            const inner = `${exprToLatex(left, PREC.addSub)} \\lor ${exprToLatex(right, PREC.addSub)}`
            return maybeParen(inner, PREC.addSub, parentPrec)
        }
        case "min_max": {
            // a | b → \min(a, b) or \max(a, b) — we don't know which so use the source notation
            return `\\left(${exprToLatex(left, PREC.atom)} \\mid ${exprToLatex(right, PREC.atom)}\\right)`
        }
    }
}

// ── Unary operators ───────────────────────────────────────────────────────────

function unaryOpToLatex(op: UnaryOpAst, operand: ExprAst, parentPrec: Prec): string {
    switch (op) {
        case "neg": {
            const inner = `-${exprToLatex(operand, PREC.unary)}`
            return maybeParen(inner, PREC.unary, parentPrec)
        }
        case "not": {
            const inner = `\\lnot ${exprToLatex(operand, PREC.unary)}`
            return maybeParen(inner, PREC.unary, parentPrec)
        }
    }
}

// ── Variables ─────────────────────────────────────────────────────────────────

function variableToLatex(v: VariableAst): string {
    if ("Parameter" in v) {
        const renderName = _paramLatexNameLookup?.(v.Parameter.parameter_name)
        return renderName ?? mathName(v.Parameter.parameter_name)
    }
    if ("Builtin" in v) return mathName(v.Builtin.ident)
    // External: keep the [refAlias] context tag even when a render-name is present,
    // so the reader can tell which submodel the variable belongs to.
    const { parameter_name, reference_name } = v.External
    const renderName = _paramLatexNameLookup?.(parameter_name, reference_name)
    if (renderName != null) {
        const safeAlias = reference_name.replace(/_/g, "\\_")
        return `${renderName}_{\\text{[${safeAlias}]}}`
    }
    return mathNameWithRef(parameter_name, reference_name)
}

// ── Function calls ────────────────────────────────────────────────────────────

function functionCallToLatex(name: FunctionNameAst, args: ExprAst[]): string {
    const argsLatex = args.map((a) => exprToLatex(a, PREC.atom))

    if ("Builtin" in name) {
        const builtinName = name.Builtin[0]

        // Special-case functions that have non-standard LaTeX syntax
        switch (builtinName) {
            case "sqrt":
                return `\\sqrt{${argsLatex.join(", ")}}`
            case "abs":
                return `\\left|${argsLatex.join(", ")}\\right|`
            case "floor":
                return `\\left\\lfloor ${argsLatex.join(", ")} \\right\\rfloor`
            case "ceil":
                return `\\left\\lceil ${argsLatex.join(", ")} \\right\\rceil`
        }

        // Standard operator-style functions: \sin, \cos, etc.
        const operatorMap: Record<string, string> = {
            sin:  "\\sin",
            cos:  "\\cos",
            tan:  "\\tan",
            asin: "\\arcsin",
            acos: "\\arccos",
            atan: "\\arctan",
            ln:   "\\ln",
            log:  "\\log",
            exp:  "\\exp",
            min:  "\\min",
            max:  "\\max",
        }
        const op = operatorMap[builtinName]
        if (op) {
            return `${op}\\left(${argsLatex.join(", ")}\\right)`
        }

        // Unknown builtin — render as upright text
        return `\\mathrm{${escapeIdent(builtinName)}}\\left(${argsLatex.join(", ")}\\right)`
    }

    // Imported Python function
    return `\\mathrm{${escapeIdent(name.Imported.name)}}\\left(${argsLatex.join(", ")}\\right)`
}

// ── Comparison operators ──────────────────────────────────────────────────────

function comparisonSym(op: ComparisonOpAst): string {
    switch (op) {
        case "eq":             return "="
        case "not_eq":         return "\\neq"
        case "less_than":      return "<"
        case "less_than_eq":   return "\\leq"
        case "greater_than":   return ">"
        case "greater_than_eq": return "\\geq"
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function maybeParen(inner: string, ownPrec: Prec, parentPrec: Prec): string {
    return ownPrec < parentPrec ? `\\left(${inner}\\right)` : inner
}

function formatNumber(n: number): string {
    if (!isFinite(n)) return n > 0 ? "\\infty" : "-\\infty"
    if (Number.isInteger(n) && Math.abs(n) < 1e9) return String(n)
    const abs = Math.abs(n)
    if (abs !== 0 && (abs >= 1e6 || abs < 1e-3)) {
        const exp = Math.floor(Math.log10(abs))
        const mantissa = n / Math.pow(10, exp)
        const mantissaStr = trimTrailingZeros(mantissa.toPrecision(4))
        if (mantissaStr === "1")  return `10^{${exp}}`
        if (mantissaStr === "-1") return `-10^{${exp}}`
        return `${mantissaStr} \\times 10^{${exp}}`
    }
    return trimTrailingZeros(n.toPrecision(6))
}

function trimTrailingZeros(s: string): string {
    return s.includes(".") ? s.replace(/\.?0+$/, "") : s
}

/**
 * Formats a finite number to 4 significant figures, stripping trailing zeros
 * and a bare decimal point. Passes through non-finite values (Inf, NaN) as-is.
 *
 * ```ts
 * fmtNum(1.5000) // "1.5"
 * fmtNum(1234.5) // "1235"
 * fmtNum(1.23e-10) // "1.23e-10"
 * ```
 */
export function fmtNum(n: number | string): string {
    const num = typeof n === "number" ? n : parseFloat(n)
    if (!isFinite(num)) return String(n)
    const s = num.toPrecision(4)
    return s.includes("e")
        ? s.replace(/(\.\d*?)0+(e)/, "$1$2").replace(/\.(e)/, "$1")
        : s.replace(/(\.\d*?)0+$/, "$1").replace(/\.$/, "")
}

function escapeText(s: string): string {
    return s.replace(/[#$%&_{}\\^~]/g, (c) => `\\${c}`)
}

function escapeUnit(unit: string): string {
    return unit.replace(/[#$%&{}\\~]/g, (c) => `\\${c}`)
}
