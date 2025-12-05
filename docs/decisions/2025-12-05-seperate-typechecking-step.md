# Seperate Typechecking Step

## Status

Accepted

## Context

At the present time, "type checking", including units checking, happens as
values are evaluated. However, this means that in order to type check a branch,
the branch has to be evaluated. This can be a bit of a slow down.

Should type checking be its own seperate step, or should we keep it integrated
with evaluation.

## Decision

After [attempting to implement type checking as a seperate step](https://github.com/careweather/oneil/commit/a37b8c975b8b2b177b69c48dac72bc5b37494a6e),
we have decided to keep type checking integrated with evaluation. There are parts
of typechecking that require runtime information.

Specifically, the
**exponent operator** (`a ^ b`) requires us to know what the exponent is
in order to determine what the resulting unit is.

In addition,
**evaluating imported** (`foo(a, b, ...)`) requires type info at
runtime so that the function can use it if it desires. And we don't
know the type of the value returned by the function, so that needs
to be checked at runtime as well.

## Consequences

Evaluation will be slower any time we need to check the types of branches
that won't be evaluated (such as piecewise functions).
