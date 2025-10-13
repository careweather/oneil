# Contributing to Oneil

Thank you for your interest in contributing to the Oneil programming language!
This document provides guidelines and instructions for contributing to the
project.

*This document is a work in progress. If you have any suggestions for improvement, feel free to open a pull request!*

## Development Setup

1. Clone the repository
2. Install Rust toolchain (latest stable version)
3. Run `cargo build` to verify your setup

For development, you can use these Cargo commands:

- Run tests:
  ```sh
  cargo test
  ```

- Check for compilation errors without producing an executable:
  ```sh
  cargo check
  ```

- Format code:
  ```sh
  cargo fmt
  ```

- Run linter:
  ```sh
  cargo clippy
  ```

You can also run the following developer commands built into Oneil:
- Print the AST that is constructed from an Oneil file:
  ```sh
  cargo run -- dev print-ast path/to/model.on
  ```

- Print the IR that is constructed from an Oneil model
  ```sh
  cargo run -- dev print-ir path/to/model.on
  ```

In addition, you will want to install the
[`rust-analyzer`](https://open-vsx.org/extension/rust-lang/rust-analyzer)
VS Code extension in order to help you develop in Rust.

If you are using `rust-analyzer` in VS Code, ensure that you are using the
`clippy` linter by [updating your
settings](https://users.rust-lang.org/t/how-to-use-clippy-in-vs-code-with-rust-analyzer/41881)

## Test Oneil Files

Test Oneil files are found in [the `test` directory](./test). These files are mainly used for manual testing and experimentation and are not automatically tested.

## System Architecture

The architecture of the system is described in [`docs/architecture/README.md`](docs/architecture/README.md). The code itself is found in [`src-rs/`](src-rs/).


## Coding Standards

Code should follow the principles laid out in
[`docs/principles.md`](docs/principles.md).


## Resources

- [Crafting Interpreters](https://craftinginterpreters.com/) - If you've never
  worked on a programming language before, this is a great resource for
  understanding how to build a programming language!
