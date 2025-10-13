# Print IR

The `dev print-ir` command prints the Intermediate Representation (IR) of an Oneil model and its dependencies. This is primarily a development tool used for debugging and understanding how the model resolver interprets Oneil code.

## Usage

```bash
oneil dev print-ir <FILE>
```

### Arguments

- `FILE`: Path to the Oneil source file to resolve

## Examples

Print the AST of a simple Oneil file:
```bash
oneil dev print-ir examples/hello.oneil
```
