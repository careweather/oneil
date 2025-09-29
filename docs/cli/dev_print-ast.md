# Print AST

The `dev print-ast` command prints the Abstract Syntax Tree (AST) of a Oneil source file. This is primarily a development tool used for debugging and understanding how the parser interprets Oneil code.

## Usage

```bash
oneil dev print-ast [<FILE>...]
```

### Arguments

- `FILE`: Path to the Oneil source file(s) to parse and print the AST for

## Examples

Print the AST of a simple Oneil file:
```bash
oneil dev print-ast examples/hello.oneil
```
