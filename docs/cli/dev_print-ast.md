# Dev Print AST

The `dev print-ast` command prints the Abstract Syntax Tree (AST) of a Oneil source file. This is primarily a development tool used for debugging and understanding how the parser interprets Oneil code.

## Usage

```bash
oneil dev print-ast <FILE>
```

### Arguments

- `FILE`: Path to the Oneil source file to parse and print the AST for

## Examples

Print the AST of a simple Oneil file:
```bash
oneil dev print-ast examples/hello.oneil
```

## Output

The command outputs a pretty-printed representation of the AST in a debug format. The output shows the hierarchical structure of the parsed code, including:
- Tokens and their values
- Node types and their relationships
- Source locations and spans

## Error Handling

The command will display an error message if:
- The specified file cannot be opened
- The file content cannot be read
- The parser encounters invalid syntax

## Use Cases

This command is particularly useful for:
- Debugging parser issues
- Understanding how Oneil code is structured internally
- Verifying the correctness of syntax
- Learning about the language's AST structure

## Implementation Details

The command uses the following components:
- `parser::model::parse_complete` for parsing the input
- `Span` for tracking source locations
- `Config::default()` for parser configuration

## Related Commands

- Other development commands in the `dev` subcommand group 