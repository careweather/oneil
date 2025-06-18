# Contributing to Oneil

Thank you for your interest in contributing to the Oneil programming language! This document provides guidelines and instructions for contributing to the project.

## Documentation

The project's documentation is organized in the `docs/` directory. Please refer to [docs/README.md](docs/README.md) for an overview of the documentation structure, which includes:

- Architecture documentation
- CLI tool documentation
- Design decisions
- Language specifications

## Development Setup

1. Clone the repository
2. Install Rust toolchain (latest stable version)
3. Run `cargo build` to verify your setup

## Contribution Workflow

1. Create a new branch for your changes
2. Make your changes
3. Add or update tests as needed
4. Update relevant documentation
5. Submit a pull request

## Documentation Guidelines

### Architecture Documentation
- Document new components in `docs/architecture/`
- Follow existing patterns for code organization
- Include diagrams where helpful

### CLI Documentation
- Document new commands in `docs/cli/`
- Follow the established command documentation structure
- Include usage examples and error cases

### Design Decisions
- Document significant decisions in `docs/decisions/`
- Use the provided template
- Include rationale and alternatives considered

### Language Specifications
- Update specifications in `docs/specs/`
- Use formal notation where appropriate
- Include examples and edge cases

## Code Style

- Follow Rust's standard formatting guidelines
- Use `cargo fmt` to format your code
- Run `cargo clippy` to check for common issues

## Testing

- Write unit tests for new functionality
- Ensure all tests pass before submitting
- Add integration tests for significant features

## Pull Request Process

1. Update documentation as needed
2. Ensure all tests pass
3. Update the changelog if necessary
4. Request review from maintainers

## Questions and Discussion

If you have questions or want to discuss potential changes:
- Open an issue for discussion
- Join our community channels
- Reach out to maintainers

## License

By contributing to Oneil, you agree that your contributions will be licensed under the project's license. 