# Oneil Parser

The parser for the Oneil programming language.

This parser is a recursive descent parser, and it uses parsing combinators from
the [`nom`](https://crates.io/crates/nom) parsing crate. In addition, it uses
[`nom_locate`](https://crates.io/crates/nom_locate) to provide information about
span locations.


## Grammar

This parser parses the Oneil grammar, as defined in
[/docs/specs/grammar.ebnf](/docs/specs/grammar.ebnf). **The grammar should be
treated as the source of truth.** Therefore, any updates to the language syntax
**should be made in the grammar first**, then to the parser. Any inconsistency
between the grammar and the code **should be treated as a bug in the parser.**


## `nom`

This parser uses **parser combinators** from the `nom` library. If you are not
already familiar with the library, read [the
documentation](https://docs.rs/nom/8.0.0/nom/), especially the following
sections:
  - [Parser combinators](https://docs.rs/nom/8.0.0/nom/#parser-combinators)
  - [Making new parsers with function
    combinators](https://docs.rs/nom/8.0.0/nom/#making-new-parsers-with-function-combinators)
  - [Combining parser](https://docs.rs/nom/8.0.0/nom/#combining-parsers)
    - **NOTE:** we generally try to stick with the *imperative style* that they
      mention in this section in order to make the parser easier to read. We've
      tried using complex parsers and, while it's definitely possible to use, it's
      very difficult to read, especially when you come back a month later to
      modify the code.

For further (optional) reading, you can also read `nom`'s [*Making a new parser
from
scratch*](https://github.com/rust-bakery/nom/blob/main/doc/making_a_new_parser_from_scratch.md).


### Don't write complex combinators

As mentioned before, try to avoid complex combinators. It is generally more
readable to use the imperative style than it is to use deeply nested
combinators. The control flow is more clear, and parser outputs are assigned
when the parser is called.

For example, prefer

```rust,ignore
let ref_keyword = ref_.convert_errors().map(|ref_token| ModelKind::Reference);
let use_keyword = use_.convert_errors().map(|use_token| ModelKind::Submodel);

let (rest, model_kind) = alt((ref_keyword, use_keyword)).parse(input)?;

let (rest, directory_path) = opt(directory_path).parse(input)?;

let (rest, model_info) = model_info.parse(input)?;

// ... rest of parsing ...

// ... construct AST ...
```

over

```rust,ignore
(
  alt((
    ref_.convert_errors().map(|ref_token| ModelKind::Reference),
    use_.convert_errors().map(|use_token| ModelKind::Submodel),
  )),
  opt(directory_path),
  model_info,
  // ... rest of parsing ...
).map(|(model_kind, directory_path, /* ... */)| {
  // ... construct AST ...
}).parse(input)
```


## Use `<parser>.parse(input)`, even with parser functions

Prefer `<parser>.parse(input)` for *every* parser, including parser functions.
This helps keep the code consistent and makes it clear whenever a parser is
being used.

> Functions that match the type `fn <parser>(input: InputSpan<'_>) -> Result<'_,
> <output>, <error>>` have the [`nom::Parser`
> trait](https://docs.rs/nom/8.0.0/nom/trait.Parser.html) implemented
> automatically. This implementation treats `<parser>.parse(input)` as the
> equivalent of `<parser>(input)`.


## Error Handling

In `nom`, there are two kinds of parsing errors: `Err::Error` and `Err::Failure`.

> There is also an `Err::Incomplete` variant, but we never use it.

`nom`'s [error managment
article](https://github.com/rust-bakery/nom/blob/main/doc/error_management.md)
describes the error variants as follows:

> - `Error` is a normal parser error. If a child parser of the `alt` combinator
>   returns `Error`, it will try another child parser
> 
> - `Failure` is an error from which we cannot recover: The `alt` combinator will
>   not try other branches if a child parser returns Failure. If we know we were
>   in the right branch (example: we found a correct prefix character but input
>   after that was wrong), we can transform a `Err::Error` into a `Err::Failure`
>   with the `cut()` combinator

If we were trying to parse an alphabetic character ( regex `'[a-z]'`), examples
of `Error` parse errors would be inputs that don't match the expected pattern at
all.

```py
abc
1.23
```

Examples of `Failure` parse errors would be inputs that match at the beginning,
but fail to match the full pattern.

```py
'
'a
'a"
```

For more information on this, see `nom`'s [error managment
article](https://github.com/rust-bakery/nom/blob/main/doc/error_management.md),
as well as [this
article](https://research.texttotypes.com/error-handling-unmatched-incomplete/),
which explains the approach in more detail.


### Error Handling Trait

In order to remove the boilerplate of having to match on an error to convert it,
we've designed an [`ErrorHandlingParser`](src/error/parser_trait.rs#L17) trait
that builds on the
[`nom::Parser`](https://docs.rs/nom/8.0.0/nom/trait.Parser.html). Anything that
implements `nom::Parser` also implements `ErrorHandlingParser`.

The `ErrorHandlingParser` trait adds three methods to parsers: `convert_errors`,
`convert_error_to`, and `or_fail_with`.

[`convert_errors`](src/error/parser_trait.rs#L99) is a convenience function that
automatically converts errors of type `E` to type `E2`. `E` must implement
`Into<E2>`. This function is mainly used when you are using a token parser,
which produces a `TokenError`, inside of an AST parser, which expects a
`ParserError`.

[`convert_error_to`](src/error/parser_trait.rs#L40) is similar to
`convert_errors`. However, it lets you define how you want the `Err::Error`
variant to be converted, rather than using the default `Into` conversion. This
is useful to make an error more detailed.

[`or_fail_with`](src/error/parser_trait.rs#L77) converts an `Err::Error` to an
`Err::Failure`. In addition, it allows you to define how the conversion should
be done.


## Adding hints and notes to an error

Hints can be added to errors for common mistakes. Notes can also be added in
order to give more detail to an error. This is done in
[`error::context::from_source`](src/error/context.rs#L8), which constructs a
list of notes and help messages from the reason, the offset, and the source.

Look at the functions listed in `from_source`, such as
`parameter_missing_label`, for implementation examples.

Consider each function in the list as a *rule*. If the rule doesn't match, it
returns an empty `Vec`. Otherwise, the rule returns one or more notes or hints.

Note that each rule is always run, regardless of the error reason. If you want
the rule to only match with certain reasons, you will need to do that filtering
within the rule.


## Parser functions return nodes

On a success, a parser function returns some kind of `Node<T>` that generally encompasses
all of the input that was parsed by that parser. There should be a `FooNode`
type alias for each `Foo` that is part of the AST.


## Parser functions should only use token functions and functions that modify/combine parsers

Parser functions should only use token functions and parser combinators that
combine or modify other parsers (such as `alt` or `opt`). Nom combinators from
`nom::bits`, `nom::bytes`, `nom::character`, and `nom::number` should only be used in
token functions.

## Tokens

Token parsing is done in the [`token`](src/token/) module. All token parsers
return a `Token` on success. On failure, they return a `TokenError`.

When writing a token parser, it is generally best to use the `token` wrapper
found in the `util` module: `token(<parser>, <error>).parse(input)`. On a
success, `token` will capture the parsed span and convert it into a token,
regardless of what `<parser>` itself returns. On an error, it will return
`<error>`.
