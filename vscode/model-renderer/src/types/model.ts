/**
 * TypeScript mirror of the Rust types in `oneil_lsp::custom_requests`.
 *
 * Keep in sync with the Rust structs manually — no code generation yet.
 */

/** Top-level response containing the main tree plus any referenced models. */
export interface RenderedTree {
    /** The primary model's rendered tree. */
    root: RenderedNode
    /** Fully rendered trees for models referenced via `ref` (not `sub`). */
    reference_pool: RenderedPoolEntry[]
}

/** An entry in the reference pool: a fully rendered model that was referenced. */
export interface RenderedPoolEntry {
    /** Alias under which this model was first referenced in the main tree. */
    alias: string
    /** The fully rendered subtree. */
    node: RenderedNode
}

export interface RenderedNode {
    model_path: string
    instance_path: string[]
    note: string | null
    parameters: RenderedParameter[]
    /** Evaluated test results in source declaration order. */
    tests: RenderedTest[]
    /**
     * Named sections in source order. Unsectioned parameters/tests have
     * `section === null` on their own records and do not appear in any section's `items`.
     */
    sections: RenderedSection[]
    children: RenderedChild[]
    references: RenderedReference[]
    /** Design files that contributed at least one parameter to this node. */
    applied_designs: AppliedDesign[]
}

/** A named section with an optional note and ordered parameter/test item refs. */
export interface RenderedSection {
    label: string
    note: string | null
    items: RenderedSectionItem[]
}

/** A reference to a parameter or test within a section. */
export type RenderedSectionItem =
    | { type: "parameter"; name: string }
    | { type: "test"; index: number }

/** One evaluated test result. */
export interface RenderedTest {
    passed: boolean
    /** Serialized `ir::Expr` AST for the test expression; render with KaTeX. `null` when IR unavailable. */
    expression: ExprAst | null
    /** Byte offsets of the test expression in the source file. */
    expr_span: ExprSpan
    /** Optional documentation note for this test. */
    note: string | null
}

/** A design file that contributed parameters to a node. */
export interface AppliedDesign {
    /** Short file-stem name of the design (e.g. `"overlay"`). */
    design_name: string
    /** Stable index into the design color palette (0-based). */
    color_index: number
}

export interface RenderedChild {
    alias: string
    node: RenderedNode
}

export interface RenderedReference {
    alias: string
    model_path: string
}

export interface RenderedParameter {
    name: string
    label: string
    /** Raw LaTeX render-name (e.g. `\hat{v}`). When present, rendered directly instead of deriving from `name`. */
    render_name: string | null
    section: string | null
    note: string | null
    /** Serialized `ir::ParameterValue` AST — render to display form in the UI. */
    expression: ParameterValueAst | null
    value: RenderedValue
    print_level: "none" | "trace" | "performance"
    expr_span: ExprSpan
    /** Present when a design contributed this parameter. */
    design: DesignMark | null
}

/** Records that a design contributed this parameter. */
export interface DesignMark {
    /** Short file-stem name of the design. */
    design_name: string
    /**
     * `true` if the design *added* this parameter (not in the base model);
     * `false` if it *overrode* an existing parameter's value.
     */
    is_addition: boolean
}

export interface ExprSpan {
    /** Absolute path of the source file. null for synthetic/fallback spans. */
    file: string | null
    start: number
    end: number
}

// ── Value types ───────────────────────────────────────────────────────────────

export type RenderedValue =
    | { type: "boolean"; value: boolean }
    | { type: "string"; value: string }
    | { type: "number"; value: number; max: number | null }
    | { type: "measured_number"; value: number; max: number | null; unit: string }

// ── IR expression AST ─────────────────────────────────────────────────────────
// Mirrors oneil_ir::ParameterValue / Expr. Keep in sync with the Rust types.
//
// Key serialization notes:
//  - Enum variant tags serialize as PascalCase (Rust default).
//  - Struct fields within variants are snake_case (Rust default).
//  - `BinaryOp`, `ComparisonOp`, `UnaryOp` *values* use `serde(rename_all =
//    "snake_case")`, so e.g. `BinaryOp::Add` → `"add"`.
//  - `FunctionName::Builtin` is a tuple variant → serializes as `[name, span]`.

export type ParameterValueAst =
    | { Simple: [ExprAst, CompositeUnitAst | null] }
    | { Piecewise: [PiecewiseExprAst[], CompositeUnitAst | null] }

export interface PiecewiseExprAst {
    expr: ExprAst
    if_expr: ExprAst
}

export type ExprAst =
    | { Literal:      { span: unknown; value: LiteralAst } }
    | { Variable:     { span: unknown; variable: VariableAst } }
    | { BinaryOp:     { span: unknown; op: BinaryOpAst; left: ExprAst; right: ExprAst } }
    | { UnaryOp:      { span: unknown; op: UnaryOpAst; expr: ExprAst } }
    | { ComparisonOp: { span: unknown; op: ComparisonOpAst; left: ExprAst; right: ExprAst; rest_chained: [ComparisonOpAst, ExprAst][] } }
    | { FunctionCall: { span: unknown; name_span: unknown; name: FunctionNameAst; args: ExprAst[] } }
    | { UnitCast:     { span: unknown; expr: ExprAst; unit: CompositeUnitAst } }

export type LiteralAst =
    | { Number: number }
    | { String: string }
    | { Boolean: boolean }

export type VariableAst =
    | { Parameter: { parameter_name: string; parameter_span: unknown } }
    | { External:  { reference_name: string; reference_span: unknown; parameter_name: string; parameter_span: unknown } }
    | { Builtin:   { ident: string; ident_span: unknown } }

/** `serde(rename_all = "snake_case")` */
export type BinaryOpAst =
    | "add" | "sub" | "escaped_sub"
    | "mul" | "div" | "escaped_div"
    | "mod" | "pow"
    | "and" | "or" | "min_max"

/** `serde(rename_all = "snake_case")` */
export type UnaryOpAst = "neg" | "not"

/** `serde(rename_all = "snake_case")` */
export type ComparisonOpAst =
    | "less_than" | "less_than_eq"
    | "greater_than" | "greater_than_eq"
    | "eq" | "not_eq"

export type FunctionNameAst =
    /** Tuple variant: `[BuiltinFunctionName_string, Span]` */
    | { Builtin: [string, unknown] }
    /** Struct variant with `name` field */
    | { Imported: { python_path: unknown; name: string; name_span: unknown } }

/** A composite unit serialized to its display string by Rust (e.g. `"kg"`, `"m/s^2"`). */
export type CompositeUnitAst = string
