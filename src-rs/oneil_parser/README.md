# Oneil Parser

A robust and efficient parser for the Oneil programming language built in Rust.

## Overview

The Oneil parser converts Oneil source code into an Abstract Syntax Tree (AST) representation. It's designed with performance, reliability, and maintainability in mind, using a recursive descent parsing approach with the [nom] parsing library.

For more details, see the crate-level documentation in [`src/lib.rs`](src/lib.rs)

## Contributing

When contributing to the parser:

1. Follow the functional programming style
2. Add comprehensive tests for new features
3. Add syntax updates to the grammar found in [`/docs/specs/grammar.ebnf`](/docs/specs/grammar.ebnf)
3. Update documentation for any API changes
4. Ensure error messages are clear and helpful
5. Maintain backward compatibility when possible

## License

This project is licensed under the same terms as the main Oneil project. 