# Principles

## Error Handling

In general, we follow the [Midori Error
Model](https://joeduffyblog.com/2016/02/07/the-error-model/). There are
*recoverable* and *unrecoverable* errors.

### Recoverable Errors

*Recoverable errors* are errors that can be anticipated and *should be handled*.
This includes things like being unable to parse a file or find a model. Note
that one way of handling recoverable errors is to display the error to the user.

In general, use `Result<T, E>` to represent recoverable errors.

### Unrecoverable Errors

*Unrecoverable errors* indicate bugs in the code. This is generally caused by
an invalid state within the program. As often as possible, the type system
should make invalid states representable. However, this isn't always feasible.

As an example of this, model loading stores its results in a `HashMap`. Later,
when submodel resolution occurs, we assume that the submodel either exists in
the `HashMap` or has a corresponding error. If neither of those is true, the
program is in an invalid state.

In situations like these, we would rather fail loudly as soon as we know about
the invalid state rather than fail silently and learn about the invalid state
much later in the program.

To handle unrecoverable errors:
- use `Result::expect` and `Option::expect` to unwrap `Result`s/`Option`s that
  should succeed
- use `assert!(<condition>, <message>)` if you want to ensure that a condition
  (such as a function invariant) holds
  - If an assertion is expensive, use `debug_assert!(...)` instead. In all other
    cases, prefer `assert!(...)`.
  - If you have an `assert!(...)` followed by a `<value>.expect`, consider if
    you can remove the assertion and just use the call to `expect` to enforce
    the same invariant
- use `unreachable!(<message>)` to indicate a path that should never be taken
- use `panic!(<message>)` if none of the other use cases apply
  - This should rarely be used. The other options give a more clear reason for
    failure, and they generally cover most unrecoverable bugs.

Make sure that you *include an informative error message* no matter which option
you decide to use. Note that the messages for the macros can be formatted in the
same way as `println!`.


## Mark TODOs And Unimplemented Features

When implementing a feature, you may think of future improvements that could be
made to the code. In addition, you often don't have time to handle every path or
edge case. 

If the code works as is but could be improved in the future, use a `// TODO`
comment. This makes these tasks easier to find.

When you are developing, mark unhandled paths and edge cases with the
`todo!(<message>)` macro. This ensures that those paths fail when you encounter
them.

However, if you try to merge a pull request with `todo!`s in the code, it will
fail when linted. If you don't intend to resolve the `todo!`s in the pull request,
change the `todo!` macro to `unimplemented!`.

> Note that the `todo!` and `unimplemented!` macros returns a type that coerces
> to any other type. So the following code is valid.
> 
> ```rust
> let my_number =
>   if some_condition {
>     42
>   } else {
>     todo!("handle when `some_condition` is not true")
>   };
> ```


## Use Tools

There are lots of tools to improve the developer experience and code quality. We
use the following tools.

### `cargo fmt`

Using `cargo fmt` allows us to keep the code style consistent. As defined in
[`rustfmt.toml`](./rustfmt.toml), we use the default style for the `2024`
edition. This should be updated if the edition in [`Cargo.toml`](./Cargo.toml)
is updated.

If you are running VS Code, set `"editor.formatOnSave"` to `true` in your
settings in order to have `cargo fmt` run whenever you save a file.

### `cargo test`

Run `cargo test` frequently to test any changes made to the code. If some crates
have compile errors or you just want to test a single crate, use `cargo test -p
<crate>`.

For more information, see [the section on testing](#testing).

### `cargo clippy`

Use `cargo clippy` to lint your code. This helps to catch potential errors and
to use a consistent style of coding. `clippy` lints are defined in
[`Cargo.toml`](./Cargo.toml) in the `[workspace.lints.*]` sections.

If you are using `rust-analyzer` in VS Code, ensure that you are using the
`clippy` linter by [updating your
settings](https://users.rust-lang.org/t/how-to-use-clippy-in-vs-code-with-rust-analyzer/41881)

The lints can be strict sometimes, and there are some cases where the lints are
not useful. In this case, you may insert `#[expect(clippy::<lint>, reason =
"<message>")]` with an included reason for *why* it should be disabled. Try to
keep the scope of the exception as limited as possible.

> If you feel that a lint is unhelpful or even harmful, feel free to [open an
> issue!](https://github.com/careweather/oneil/issues) The purpose of the lints
> are to help ensure a consistent code style and to make developers aware when
> they should be using a certain pattern. However, it shouldn't be at the cost
> of readability, and it shouldn't be causing major problems for developers.
>
> In your issue, please include reasoning behind why a lint should be disabled.
> You could also include examples of how it is harmful.


## Prefer Flat Code

Nested code is harder to reason about, since you have to remember what
information each level of nesting introduces. Whenever possible, keep the
nesting down to a minimum. 

One way to do this is to use `let ... else` and `if ... let` combined with early
returns to handle `Options`.

```rust
fn foo(hash_map: HashMap<u32, u32>) -> u32 {
  let maybe_value = hash_map.get(0);
  let Some(value) = maybe_value else {
    return 0;
  };

  // ... do other things with `value` ...
}
```

```rust
fn bar(key: u32) -> u32 {
  let maybe_duplicate = find_duplicate_of(key)
  if let Some(duplicate) = maybe_duplicate {
    return 0;
  }

  // ... assume key is not a duplicate ...
}
```


## Avoid Writing Declarative Macros

Do not write macros using `macro_rules!`. Declarative macros can reduce
boilerplate, but they come at the price of a syntax that's harder to read and
code that's harder to debug. It's also easy to introduce a "sublanguage" that
developers now have to learn.

### `assert_...!` Macros

The only exception to this rule is `assert_...!` macros in tests. Because tests
often have assertion steps in common, these can be combined in a macro. This
ensures that no necessary assertions are forgotten, and it can make it a lot
more clear what a grouping of assertions does.

The reason for this exception is that if we were to combine the assertions into
an `assert_...` function, then anytime an assertion in that function failed, it
would point to code inside the function, rather than where the function was
called. Using a macro means that the location of the assertion in the test is
displayed, rather than the code inside the macro.

Note, however, that these macros are written in a specific style.

1. Assert macros should be written in a way that makes them look equivalent to a
   function call. Arguments are passed in as comma seperated expressions, with
   an optional trailing comma at the end.

2. There is only one macro branch, so it's always clear which code the macro is
   using.

3. Macro "arguments" are immediately assigned to variables with explicit types
   in order to ensure that each argument has the expected type.

4. Assertions inside the macro have an explicit message in order to make it
   quickly clear which assertion failed.

```rust
macro_rules! assert_var_is_builtin {
  ($variable:expr, $expected_ident:expr $(,)?) => {
      let variable: ir::ExprWithSpan = $variable;
      let expected_ident: &str = $expected_ident;

      let ir::Expr::Variable(ir::Variable::Builtin(actual_ident)) = variable.value() else {
          panic!("expected builtin variable, got {variable:?}");
      };

      assert_eq!(
          actual_ident.as_str(),
          expected_ident,
          "actual ident does not match expected ident"
      );
  };
}
```

## Testing

### 3-Step Unit Tests

Unit tests should test a single function. A single unit test should test that
**one given input** produces **one expected output**. When writing a unit test,
it should follow three steps: **prepare**, **run**, then **assert**.

First, the test should **prepare** any inputs needed to run the function. This
could include constructing the string that needs to be parsed, building the set
of models that have previously been resolved, or making the IR that needs to be
evaluated.

Next, the test should **run** the function. Pass in the inputs and store the
result.

Finally, the test should **assert** things about the result. Use liberally
`assert!`, `assert_eq!`, `panic!`, `Option::expect`, `Result::expect`,
`Result::expect_err`, and anything else that panics.

Also, when you expect a certain variant of an enum, use `let ... else` to unwrap
it. Generally avoid using `match` since there's usually only one expected path,
and `let ... else` keeps the failure close to the unwrapping.

```rust
// PREFER THIS
let MyEnum::Variant1 { field1, field2 } = value else {
  panic!("Expected Variant1, got {value:?}");
};

assert_eq!(field1, expected_field1);
assert_eq!(field2, expected_field2);

// OVER THIS
match value {
  MyEnum::Variant1 { field1, field2 } => {
    assert_eq!(field1, expected_field1);
    assert_eq!(field2, expected_field2);
  }
  _ => panic!("Expected Variant1, got {value:?}");
}
```

### Test Coverage

**Testing doesn't have to have 100% coverage.** 100% coverage may be feasible
when a program is small and simple. However, the more complex a problem gets,
the harder it gets to cover every possible branch.

Instead of trying to cover every edge case, write a unit test or two that test
the main functionality. This ensures that the expected use case works.

When you encounter a bug, write a test that proves that the bug exists. Fix the
bug, then rerun the test to prove that it no longer exists.
