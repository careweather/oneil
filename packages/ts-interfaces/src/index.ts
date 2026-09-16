/**
 * Generated TypeScript bindings for Oneil JSON wire formats.
 *
 * Do not edit `./generated/` by hand — run `./scripts/generate-ts-interfaces.sh`
 * from the repo root. CI fails if regeneration would change committed output
 * (`./scripts/check-ts-interfaces.sh`).
 *
 * See `docs/CODING_STANDARDS.md` (Generated TypeScript Bindings).
 */

export type { AppliedDesign } from "./generated/AppliedDesign.js"
export type { BinaryOp } from "./generated/BinaryOp.js"
export type { BuiltinFunctionName } from "./generated/BuiltinFunctionName.js"
export type { BuiltinValueName } from "./generated/BuiltinValueName.js"
export type { ComparisonOp } from "./generated/ComparisonOp.js"
export type { CompositeUnit } from "./generated/CompositeUnit.js"
export type { DesignMark } from "./generated/DesignMark.js"
export type { DiagnosticKind } from "./generated/DiagnosticKind.js"
export type { EvaluatedValue } from "./generated/EvaluatedValue.js"
export type { Expr } from "./generated/Expr.js"
export type { ExprSpan } from "./generated/ExprSpan.js"
export type { FunctionName } from "./generated/FunctionName.js"
export type { Literal } from "./generated/Literal.js"
export type { ModelTestReport } from "./generated/ModelTestReport.js"
export type { ParameterName } from "./generated/ParameterName.js"
export type { ParameterValue } from "./generated/ParameterValue.js"
export type { PiecewiseExpr } from "./generated/PiecewiseExpr.js"
export type { PyFunctionName } from "./generated/PyFunctionName.js"
export type { PythonPath } from "./generated/PythonPath.js"
export type { ReferenceName } from "./generated/ReferenceName.js"
export type { RenderedChild } from "./generated/RenderedChild.js"
export type { RenderedNode } from "./generated/RenderedNode.js"
export type { RenderedParameter } from "./generated/RenderedParameter.js"
export type { RenderedPoolEntry } from "./generated/RenderedPoolEntry.js"
export type { RenderedReference } from "./generated/RenderedReference.js"
export type { RenderedSection } from "./generated/RenderedSection.js"
export type { RenderedSectionItem } from "./generated/RenderedSectionItem.js"
export type { RenderedTest } from "./generated/RenderedTest.js"
export type { RenderedTree } from "./generated/RenderedTree.js"
export type { ReportDiagnostic } from "./generated/ReportDiagnostic.js"
export type { SourceLocation } from "./generated/SourceLocation.js"
export type { Span } from "./generated/Span.js"
export type { TestDependency } from "./generated/TestDependency.js"
export type { TestOutcome } from "./generated/TestOutcome.js"
export type { TestReport } from "./generated/TestReport.js"
export type { TestReportEntry } from "./generated/TestReportEntry.js"
export type { UnaryOp } from "./generated/UnaryOp.js"
export type { Variable } from "./generated/Variable.js"

/** Wire shape for a float under `oneil_shared::serde::f64`. */
export type FloatValue = number | { float_special: "NAN" | "INFINITY" | "NEGATIVE_INFINITY" }
