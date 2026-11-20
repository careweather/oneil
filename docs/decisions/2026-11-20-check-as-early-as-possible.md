# Check As Early As Possible

## Status

Accepted. Supersedes the direction of [Seperate Typechecking Step](2025-12-05-seperate-typechecking-step.md); its conclusion that some checks need runtime values still holds.

## Context

The 2025-12-05 decision kept all expression type and unit checking inside evaluation, because some rules need runtime values: the result unit of `a ^ b` depends on the value of `b`, and Python functions receive and return values whose types are only known at runtime.

Keeping *every* type rule in evaluation has costs. A branch that is never evaluated (for example an unmatched piecewise case) is never checked. Errors that could be reported from the source alone wait for a full evaluation, so `oneil check` and editor diagnostics miss them.

## Decision

Each check runs in the earliest compiler phase that has the information it needs (see [`../architecture/compiler-pipeline.md`](../architecture/compiler-pipeline.md#where-a-new-check-or-transformation-belongs)). For type and unit rules:

- rules that can be decided from declarations, literals, and resolved units (for example the dimensions of a parameter's declared unit, or adding quantities whose dimensions are known to differ) belong before evaluation, in resolution or validation;
- rules that depend on values (exponents, Python calls and return values, limits, numeric domains, piecewise selection) stay in evaluation.

There is no requirement for a single separate type-checking pass. Early checks are added to the phase, and the traversal within that phase, where their inputs become available.

## Consequences

- `oneil check` and editor diagnostics report more errors without evaluating the model.
- Evaluation keeps its checks as the backstop, so a rule checked early may also be checked again in evaluation until the early check covers every case.
- Static rules need dimension information before evaluation, which makes resolved units and builtin signatures part of the contract between resolution and validation.
