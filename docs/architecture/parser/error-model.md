# Error Handling Model

There are two types of errors when tokenizing: "not found" errors and
incomplete errors.


## "Not found" errors

**Summary:**
- related to `nom::Err::Error`
- Example: parsing `foo` as a number

"Not found" errors are errors that occur when a token is not found. For
example, if we are trying to parse the input `foo` and we expect a number,
this is a "not found" error.

"Not found" errors are handled by the `token` function for tokens, which will
return an associated error kind.

These errors are represented by `nom::Err::Error`, since it is possible that
another parser may succeed on this input. In the case of `foo`, the identifier
parser would succeed.


## Incomplete errors

Incomplete errors are errors that occur when a token is clearly started, but
the parser is unable to complete the token. For example, if we are trying to
parse the input `"foo` and we expect a string, we start to parse the string,
but we reach the end of the input without finding a closing quote. This is an
incomplete error.

Incomplete errors are indicated by the use of `cut`, which converts a
`nom::Err::Error` to a `nom::Err::Failure`. Incomplete errors should be
converted to the correct error kind *when it is "cut"*, since other error
handlers should only be handling `nom::Err::Error`, not `nom::Err::Failure`