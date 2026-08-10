# Coding Standards

## Error Handling

In general, follow the
[Midori Error Model](https://joeduffyblog.com/2016/02/07/the-error-model/).
There are *recoverable* and *unrecoverable* errors.

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
[`rustfmt.toml`](../rustfmt.toml), we use the default style for the `2024`
edition. This should be updated if the edition in [`Cargo.toml`](../Cargo.toml)
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
[`Cargo.toml`](../Cargo.toml) in the `[workspace.lints.*]` sections.

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

## Prefer direct field access over getters and setters

In Rust, structs that carry data should expose their fields as `pub` rather than
hiding them behind getter and setter methods. Direct field access is simpler and
avoids boilerplate that doesn't provide any value. For example,

```rs
pub struct DesignApplication {
    pub apply_span: Span,
    pub applied_in: ModelPath,
    pub design_path: DesignPath,
}

let span = &application.apply_span;
```

Use a method instead of a public field **only** when you need to:

- **maintain an invariant** (e.g., the setter validates the input)
- **return a computed or derived value** rather than a stored one
- **expose a borrowed view** of a private inner type, such as `as_str()` on a
  newtype around `String`

## Prefer Readable Code Over Terse Code

More people will read the code than write it, so optimize the code for reading.
Don't try and do everything on one line, and store intermediate values instead of
making large chains of function calls. For example,

```rs
let model = runtime.load_model(model_path);
let reference = model.references().get(reference_name);
let reference_path = reference.path();
```

It could also be improved by creating a helper function.

For example,

```rs
let reference_path = runtime.load_model(model_path).reference_path_of(reference_name);
```

In general, when there is a chain of function calls, ask yourself if some segment
of it can be made more generally useful throughout the codebase.

## Avoid Writing Declarative Macros

Do not write macros using `macro_rules!`. Declarative macros can reduce
boilerplate, but they come at the price of a syntax that's harder to read and
code that's harder to debug. It's also easy to introduce a "sublanguage" that
developers now have to learn.

## Use the type system

Because the type system is checked at compile time rather than at run time, the
type system can help you catch bugs earlier in the process if used correctly.
It can also encode invariants and rules about how a value should be used (or
*not* used).

### The newtype pattern

In Rust, a newtype is a struct with only one field. For example,

```rs
struct ParameterName(String);
struct PythonPath(PathBuf);
```

Use newtypes liberally. In Rust, they cost practically nothing in terms of
size or speed, and their use has some useful benefits.

Newtypes are a great way to help yourself and others use a value correctly.
For example, if we store both python paths and model paths as `PathBuf`s, we
might accidentally use a python path where we need a model path, or vice versa.

Using a newtype ensures that this kind of mistake can't happen.

```rs
let model_path = ModelPath::from("model.on");
let python_path = PythonPath::from("functions.on");
    
// ... later ...

// this produces the following compiler error
//
// error[E0308]: mismatched types
//   --> src/main.rs:22:28
//    |
// 22 |     let model = load_model(python_path);
//    |                 ---------- ^^^^^^^^^^^ expected `ModelPath`, found `PythonPath`
//    |                 |
//    |                 arguments to this function are incorrect
//    |
let model = load_model(python_path);
```

For more details about using newtypes, see
[Embrace the newtype pattern](https://www.lurklurk.org/effective-rust/newtype.html)
and
[Newtypes and contracts](https://research.texttotypes.com/newtypes-and-contracts/).

### Make invalid state unrepresentable

The type system can be used to verify at compile time that you can't reach an
invalid state. There are many ways that this can be accomplished. One simple
example of this is a reference that may or may not have an alias. It could be
represented in the following way.

```rs
struct Reference {
  name: ReferenceName,
  span: Span,
  alias_name: Option<AliasName>,
  alias_span: Option<Span>,
}
```

However, using this representation means that it is possible to have an alias
name without a span, or an alias span without a name. Neither of these states
make sense. To make them impossible, they can be merged into a single `Option`.

```rs
struct Reference {
  name: ReferenceName,
  span: Span,
  alias: Option<(AliasName, Span)>,
}
```

With this representation, it is impossible to have an alias name without a
span, and vice versa.

This also has the side effect of making code easier to write since you have to
enforce less invariants at run time.

## JSON Wire Types

When emitting JSON for external consumers (LSP webview, `oneil test --format
json`, etc.), put shared *leaf* DTOs in one place and reuse them:

- evaluated values → `oneil_output::EvaluatedValue`
- diagnostic severity → `oneil_shared::error::DiagnosticKind`

Envelope types (`RenderedTree`, `TestReport`, …) can differ per consumer.
Do not hand-copy leaf shapes between crates.

## Generated TypeScript Bindings

TypeScript types for those JSON wire formats are generated with `ts-rs` into
`packages/ts-interfaces` (`oneil-ts-interfaces`). Do not hand-edit
`packages/ts-interfaces/src/generated/`.

```sh
./scripts/generate-ts-interfaces.sh   # regenerate
./scripts/check-ts-interfaces.sh      # CI drift check
```

The pre-commit hook regenerates and stages these bindings whenever Rust files
are staged. CI still runs `./scripts/check-ts-interfaces.sh` as a backstop.

Opt into Rust-side derives with the `ts-bindings` Cargo feature. Opaque Serde
helpers (e.g. special floats) need an explicit `#[ts(type = "...")]` so the
generated types match the wire shape.

## Testing

### Test Kinds

Oneil's code can and should be tested in several different ways:

- unit tests
- snapshot tests
- property tests

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
let MyEnum::Variant1 { field1, field2 } = value else {
  panic!("Expected Variant1, got {value:?}");
};

assert_eq!(field1, expected_field1);
assert_eq!(field2, expected_field2);
```

### Snapshot Tests

Snapshot tests can be a great way to track the output of tests in a way that
is easy to observe visually. It allows a reviewer to quickly see what has
changed.

For snapshot testing in Rust, Oneil uses the `insta` crate. Snapshot tests are
found in [`oneil_snapshot_tests`](../src-rs/oneil_snapshot_tests/).

When writing a snapshot test, make the output easy to read. Don't use default
`Debug` output. For example,

```plaintext
5 | 10 :kg
```

is a lot easier to read than

```plaintext
MeasuredNumber(MeasuredNumber { normalized_value: NormalizedNumber(Interval(Interval { min: 5.0, max: 10.0 })), unit: Unit { dimension_map: DimensionMap({Mass: 1.0}), magnitude: 1.0, is_db: false, display_unit: Unit { name: "kg", exponent: 1.0 } } })
```

For further reading on snapshot testing, check out
[Testing can be fun, actually](https://giacomocavalieri.me/writing/testing-can-be-fun-actually)

### Property Tests

Property testing is a way to test that a given property holds. Property tests
differ from unit tests in that they use semi-random input rather than fixed
input.

For example, to test the associative property of addition, a unit test might
look like this:

```rust
#[test]
fn associativity_property() {
  let a = 1;
  let b = 2;
  assert_eq!(a + b, b + a);
}
```

where as a property test (using `cargo fuzz`) might look like this:

```rust
use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(arbitrary::Arbitrary)]
struct AssociativityInput(i32, i32);

fuzz_target!(|input: AssociativityInput| {
  let a = input.0;
  let b = input.1;
  assert_eq!(a + b, b + a);
})
```

The unit test is deterministic and only runs once, whereas the property test
is run on many semi-random inputs. Running on many inputs enables a property
test to discover inputs that break the property, if any exist. This can give
you confidence that a property holds.

For more details on fuzz testing and `cargo fuzz`, see the
[Rust Fuzz Book](https://rust-fuzz.github.io/book/cargo-fuzz.html).

> [!NOTE]
> Technically,
> [there is a difference](https://www.bjaress.com/posts/2021-07-03/fuzz-testing-vs-property-based-testing.html)
> between fuzz testing and property testing. For the purposes of documentation,
> though, we use them interchangeably.

### Test Coverage

**Testing doesn't have to have 100% coverage.** 100% coverage may be feasible
when a program is small and simple. However, the more complex a problem gets,
the harder it gets to cover every possible branch.

Instead of trying to cover every edge case, write several unit tests that test
the main functionality. This ensures that the expected use case works.

When you encounter a bug, write a test that proves that the bug exists. Fix the
bug, then rerun the test to prove that it no longer exists.
