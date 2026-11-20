# Documentation

This directory contains the various parts of documentation associated with the Oneil programming language.

- [Architecture](architecture/README.md) - How the Rust code and the editor tooling are organized
  - [Compiler pipeline](architecture/compiler-pipeline.md) - The contract of each compiler phase: inputs, outputs, validations, what is fixed afterwards, caches, and where new checks belong
  - [Design overlays](architecture/design-overlays.md) - Implementation guide for design files, overlays, and parameter augmentation
  - [VS Code extension](architecture/vscode-extension.md) - The extension, the language server contract, and the Rendered View
- [Coding standards](CODING_STANDARDS.md) - Standards for Rust code in this repository
- [Oneil coding standards](ONEIL_CODING_STANDARDS.md) - Standards for writing Oneil models
- [Decisions](decisions/README.md) - A record of design decisions that have been made related to the Oneil programming language
- [Specs](specs/) - Specifications that define how the Oneil programming language functions on an implementation-agnostic level
  - [Grammar (EBNF)](specs/grammar.ebnf)
  - [Designs, overlays, and imports](specs/designs-and-imports.md)
- [User guide](guide/src/SUMMARY.md) - The mdBook source for [oneil-lang.org](https://oneil-lang.org), including CLI usage
- [Migration guide](migration_guide.md) - Converting models written for the Python implementation of Oneil to the Rust implementation
- [Research](research/) - Background research behind implementation choices, such as interval arithmetic
