# Oneil Architecture

The Oneil programming language has many different parts of it, but the core of
Oneil follows the following architecture for evaluation of a model.

1. [Tokenization and Parsing](#tokenizerparser)
2. Module Resolution
3. Type and Unit Checking
4. Evaluation

This document gives a high level overview of each step, as well as technical
details that are needed in order to contribute.

> If you are new to programming language design and would like to understand
> more, [*Crafting Interpreters*](https://craftinginterpreters.com/) is a great
> resource. It takes you through the process of building a programming
> language, and it explains standard programming language patterns along the
> way. The book is free to read online (just scroll down to the "Web" section).

## Module organization

```
lib
|- ast: types for the abstract syntax tree
|- parser: parsing code
   |- token: tokenization code
   |  |- error: token error handling
   |  |- util: utility functions for tokenization
   |- config: configuration of the parser
   |- error: parser error handling
   |- util: utility functions and types for parsing
```


## Tokenizer/Parser

### Overview 

The goal of a tokenizer and parser is to take *unstructured plain text* and to
convert it into a *structured representation* of the underlying code. This
structured representation is known as an abstract syntax tree (AST). As a simple
example, consider the following expression:

```
3 + 4 * x
```

A tokenizer could convert this to the following tokens.

```
(NUMBER "3") (PLUS) (NUMBER "4") (STAR) (IDENTIFIER "x")
```

A parser could then produce the following AST:

```
     MULTIPLY
    /       \
  ADD      VARIABLE
 /   \       |
3     4     "x"
```

### Combining tokenizer and parsing

Because the language is context-sensitive in one case, Oneil does not completely
seperate tokenization and parsing, although they are still treated as individual
concepts.

> The context-sensitive piece of grammar is the "label" token, which can
> include spaces and tabs, while in all other cases, spaces and tabs can be
> ignored.

### `nom` and `nom_locate` library

For both these tasks, we use the [`nom`](https://docs.rs/nom/8.0.0/nom/)
library, which provides us with [parser
combinators](https://docs.rs/nom/8.0.0/nom/#parser-combinators) that we can use
to build our parser.

In order to better understand the parser and tokenizer, it is recommended to
read the ["Parser
combinators"](https://docs.rs/nom/8.0.0/nom/#parser-combinators), ["Making new
parsers with function
combinators"](https://docs.rs/nom/8.0.0/nom/#making-new-parsers-with-function-combinators),
and ["Combining parser"](https://docs.rs/nom/8.0.0/nom/#combining-parsers)
sections in the `nom` documentation.

In addition to `nom`, we also use the `nom_locate` library, which provides us
with the `LocatedSpan` struct. The `LocatedSpan` struct allows us to track the
line and the offset of the contained string. In addition, it allows us to pass
configuration details around in the form of the `crate::parser::config::Config`
struct.

### Tokenization

Code for tokenization is found in the `crate::parser::token` module. 

Note that all tokens take the form of

```rs
use crate::parser::util::{Result, Span, Token};

pub fn my_token(input: Span) -> Result<Token, TokenError> {
    // ... parsing code ...
}
```

Notice that it:
- takes a `Span` as input
- returns a `Result`, with `Token` as the output and `TokenError` as the error kind

For most tokens, this is done using the `crate::parser::token::util::token`
combinator. (The only place it is not used is in the note token parsers)

The `token` combinator accepts any parser and returns the `Token` that it
parses. A `Token` stores data about the lexeme that it parsed and the whitespace
that followed.  In addition, the `token` combinator accepts a `TokenErrorKind`
that it will attach as an error if it fails to parse the token. For example, a
simple string token parser could look like the following:

```rs
use crate::parser::util::{Result, Span, Token};
use crate::parser::token::util::token;

pub fn my_token(input: Span) -> Result<Token, TokenError> {
    token(
        |input| {
            let (rest, _) = tag("\"").parse(input)?;
            let (rest, _) = take_while(|c: char| c != '"').parse(rest)?;
            let (_, _) = tag("\"").parse(rest)?;
        },
        TokenErrorKind::String(StringError::ExpectString),
    )
}
```

### Parsing

Code for parsers are found in the `crate::parser` module. This code implements
[the Oneil grammar](/docs/specs/grammar.ebnf).

Each of the following modules are parsers:
- `declaration`
- `expression`
- `model`
- `note`
- `parameter`
- `test`
- `unit`

These modules correspond to nodes of the AST, defined in `crate::ast`. Each
module exports two functions, `parse` and `parse_complete`. Both of these functions have the signature

```rs
pub fn parse(input: Span) -> Result<AST_NODE, ParserError> {
    // ...
}
```

where `AST_NODE` is the AST node that corresponds to the parser.

The difference between `parse` and `parse_complete` is that `parse_complete`
will fail if it doesn't parse the full string, while `parse` will accept part of
a string. For example,

```rs
use crate::parser::util::Span;
use crate::parser::expression;

let span = Span::new_extra("1 + 2 <some more non-expression stuff>", Config::default());

let result = expression::parse(span);
assert!(result.is_ok())

let result = expression::parse_complete(span);
assert!(result.is_err())
```

Typically, `parse` is used to build other parsers, while `parse_complete` is
used when the string should *only* contain that item.

Note that because of the way that the `model` module attempts to recover when it
hits an error and return a partial result alongside errors encountered, `model`
only has a `parse_complete` version.

Parsers should also avoid using combinators that recognize strings directly,
such as `tag` or `char`. Instead, parsers should rely on `token` parsers.

### Error Handling

For details on how error handling is done in the parser and tokenizer, see
[parser/error-model.md](parser/error-model.md).
