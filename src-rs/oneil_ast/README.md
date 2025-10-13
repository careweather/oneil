# Oneil AST

Abstract Syntax Tree (AST) data structures for the Oneil programming language.

This library is divided up into modules internally for organization. Externally, all data structures are exported at the same level.


## Spans

In order to keep track of where an AST node is found in the source code, the [`Span`](src/span.rs) data structure is used. It includes information about the start offset within the source, as well as the length of the node text and the trailing whitespace.

To make a `Span` to an AST data structure, use the [`Node<T>`](src/node.rs) data structure.
