# Oneil IR

Intermediate Representation (IR) for the Oneil programming language.

This crate defines the intermediate representation used in the Oneil compilation pipeline. The IR serves as a bridge between the AST and the final evaluation engine, providing a structured representation optimized for analysis and computation.

These structures share much in common with the AST structures. However, there are a few key differences:
- declarations are grouped by kind rather than ordered by appearance in the source file
- variables referencing imported models directly store the paths to those models
- parameter units are normalized into a list of units (TODO: should this be done in the evaluator instead? It seems like a more logical place for it)
