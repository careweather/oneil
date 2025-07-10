# Separate AST and IR Data Structures

## Status

Accepted

## Context

We have two distinct layers in the Oneil language implementation:
1. **Parser/AST layer** (`oneil_parser`, `oneil_ast`) - responsible for parsing source code and creating abstract syntax trees
2. **IR layer** (`oneil_ir`) - responsible for representing loaded models and their semantic information

The question arose whether to reuse AST data structures (like `Expr`, `Declaration`, etc.) in the IR data structures, or to create separate, IR-specific data structures.

## Decision

We will **not** reuse AST data structures in the IR data structures. Instead, we will create separate, IR-specific data structures that are independent of the AST representation.

This means:
- IR data structures will have their own `Expr`, `Parameter`, `Limit`, etc. types
- These types will be defined in the `oneil_ir` crate and its dependencies
- The IR layer will not depend on `oneil_ast` or `oneil_parser` crates
- When converting from AST to IR representation, we will transform the data structures

## Consequences

### Positive Consequences
- **Decoupling**: The IR layer is completely independent of the parser/AST layer
- **Flexibility**: IR data structures can evolve independently of AST structures
- **Clear separation of concerns**: Each layer has its own domain-specific data models
- **Easier testing**: IR logic can be tested without AST dependencies
- **Future-proofing**: Changes to AST structures won't affect IR representation

### Negative Consequences
- **Code duplication**: Similar data structures exist in multiple places
- **Maintenance overhead**: Changes to language semantics may require updates in multiple places
- **Memory usage**: Slightly higher memory usage due to duplicate structures
- **Development time**: More initial work to create separate data structures

The benefits of decoupling outweigh the costs of duplication, as it provides a
cleaner architecture and better long-term maintainability. 