# Decision record template by Michael Nygard

# Discrete Value Syntax

## Status

Accepted


## Context

In v0.13 of the Oneil programming language, discrete limits have the following
format:

```oneil
Main color[red, green, blue]: color = red
```

However, this can cause problems with using variable names, such as in the
following example:

```oneil
My color[red, green, blue]: my_color = red

Main color[red, green, blue]: color = my_color
```

In the context of the above code, the current implementation of Oneil doesn't
know how to distinguish between a variable (`my_color`) and a discrete value
(`red`), so anytime the right hand of an assignment is a single identifier,
Oneil treats it as a discrete value.

Therefore, the above code would produce an error, since `my_color` is not within
the discrete limit of `[red, green, blue]`.

In order to solve this problem, Oneil introduced the "pointer" (`=>`) syntax:

```oneil
My color[red, green, blue]: my_color = red

Main color[red, green, blue]: color => my_color
```

The pointer syntax indicates that any single-identifier expressions that follow
should be interpreted as a variable, rather than as a discrete value.

However, this distinction is confusing for a user that isn't familiar with the
internals of the language. Therefore, we would like to replace this with a new
solution.

In addition, if we remove the pointer syntax for this use case, we may be able
to apply it in a different way later.


## Options

### Semantic Analysis

In the new implementation, we could determine which identifiers should be
treated as variables and which identifiers should be treated as strings through
the use of semantic analysis. In other words, the language would be able to
figure out the following using context:

```oneil
My color[red, green, blue]: my_color = red

Main color[red, green, blue]: color = my_color
```

#### Pros

1. No new syntax would be added.

#### Cons

1. It would add a layer of complexity to the language since an extra pass would
   be required for the semantic analysis.

2. Users might still be confused about what determines when an identifier is a
   discrete variable or a variable.

3. If a discrete variable shares a name with a variable, there's no clear way to
   distinguish which one a user intends to use.

### Quote-delimited

Discrete values are quote-delimited.

```oneil
My color["red", "green", "blue"]: my_color = "red"

Main color["red", "green", "blue"]: color = my_color
```

#### Pros

1. It's easily distinguishable from a variable.

2. It can be decided at the parsing step whether it's a variable or a discrete
   value.

3. There's no possible conflict between a variable name and a discrete value.

4. For any users that have programming experience in other languages, such as
   Python, it maps to their previous experience with strings.

#### Cons

1. The syntax is more verbose.


## Decision

We will use quote-delimited syntax (`"red"`).


## Consequences

1. We don't have to use semantic analysis to determine what is a discrete value and what is a variable.

2. Discrete values become a little bit longer to write.
