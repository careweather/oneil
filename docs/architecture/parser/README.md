# Parser Key Concepts

## Parser functions return nodes

On a success, a parser function returns some kind of `Node<T>` that generally encompasses
all of the input that was parsed by that parser. There should be a `FooNode`
type alias for each `Foo` that is part of the AST.


## Token functions return tokens

On a success, a token function returns a `Token`.


## Parser functions should only use token functions and functions that modify/combine parsers

Parser functions should only use token functions and parser combinators that
combine or modify other parsers (such as `alt` or `opt`). Nom combinators from
`nom::bits`, `nom::bytes`, `nom::character`, and `nom::number` should only be used in
token functions.


## Functions should fail when they have evidence that a parser should match

If we have evidence that a parser *should* match, (such as a `test` keyword
indicating that a test should be parsed), failure to find the rest of the
expected parts should set the error with `map_error` then use `cut` to convert
the error from an error into a failure.

On the other hand, if we don't have proof that a parser should match, then
errors should not be `cut`.

Theoretical example:

```rs
// a theoretical example that parses a negative number (ex. `-5`)
let (rest, neg_op_token) = neg_op.convert_errors().parse(input)?;

// at this point, we have the `-` operator, so we know that the next token
// must be a number. if it isn't, we have reached an unrecoverable failure
let (rest, number_token) = cut(
    number.map_err(ParserError::negative_number_missing_number(neg_op_token))
  ).parse(input)?;

// ... rest of processing ...
```
