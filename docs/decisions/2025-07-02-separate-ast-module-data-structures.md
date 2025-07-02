# Separate AST and Module Data Structures

## Status

Accepted

## Context

We have two distinct layers in the Oneil language implementation:
1. **Parser/AST layer** (`oneil_parser`, `oneil_ast`) - responsible for parsing source code and creating abstract syntax trees
2. **Module layer** (`oneil_module`) - responsible for representing loaded modules and their semantic information

The question arose whether to reuse AST data structures (like `Expr`, `Declaration`, etc.) in the module data structures, or to create separate, module-specific data structures.

## Decision

We will **not** reuse AST data structures in the module data structures. Instead, we will create separate, module-specific data structures that are independent of the AST representation.

This means:
- Module data structures will have their own `Expr`, `Parameter`, `Limit`, etc. types
- These types will be defined in the `oneil_module` crate and its dependencies
- The module layer will not depend on `oneil_ast` or `oneil_parser` crates
- When converting from AST to module representation, we will transform the data structures

## Consequences

### Positive Consequences
- **Decoupling**: The module layer is completely independent of the parser/AST layer
- **Flexibility**: Module data structures can evolve independently of AST structures
- **Clear separation of concerns**: Each layer has its own domain-specific data models
- **Easier testing**: Module logic can be tested without AST dependencies
- **Future-proofing**: Changes to AST structures won't affect module representation

### Negative Consequences
- **Code duplication**: Similar data structures exist in multiple places
- **Maintenance overhead**: Changes to language semantics may require updates in multiple places
- **Memory usage**: Slightly higher memory usage due to duplicate structures
- **Development time**: More initial work to create separate data structures

The benefits of decoupling outweigh the costs of duplication, as it provides a
cleaner architecture and better long-term maintainability. 