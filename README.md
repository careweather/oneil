# The Oneil Design Language

<!-- markdownlint-disable-next-line no-inline-html -->
<img alt="The Oneil Logo" src="docs/icons/oneil-logo.svg" align="right" width="25%">

Oneil is a design specification language for rapid, comprehensive system modeling.

Traditional approaches to system engineering are too cumbersome for non-system engineers who don't have all day. Oneil makes it easy for everyone to contribute to the central source of system knowledge. With Oneil everyone can think like a system engineer and understand how their design impacts the whole.

Oneil enables specification of a system *model*, which is a collection of *parameters*, or attributes of the system. The model can be used to evaluate any corresponding *design* (which is a collection of value assignments for the parameters of the model).

## Features

Oneil makes it easier than ever to build, debug, explore, and version-control models and designs of complex systems.

* Fully-updated design with every modification (no more passing results back and forth)
* Seamless background unit handling (say goodbye to conversions).
* Single source of truth for equations (united documentation and code).
* Automatic calculation of extreme range of possibilities.
* Built-in tests and reality checks.
* Command-line interface for evaluating models and designs:
  * Dependency trees for at-a-glance calculation tracing.
* Python extensibility.
* Vim highlighting.
* Caching and automatic change reports.
* VSCode highlighting and linting.
* (coming soon) Automatic documentation:
  * Model derivations.
  * Design test reports.
  * Parametric figures.
* (coming soon) Side-by-side design comparisons.

## Requirements

Oneil has only been tested on Linux. Instructions for Oneil assume you are on Linux.

## Quickstart

To run the Rust version of Oneil locally:

1. [Install Rust and Cargo](https://www.rust-lang.org/tools/install) if you haven't already.

2. Clone the repository and navigate to the Rust project directory:

   ```sh
   git clone git@github.com/careweather/oneil.git
   cd oneil
   ```

3. Build and run the project:

   ```sh
   cargo run -- path/to/your/model.on
   ```

See [Installation](docs/installation.md) for installing the `oneil` binary from a release or from source.

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for more details on how to work on Oneil code.

### Model Syntax Updates for Rust Version

**If you were using the Python version of Oneil, you may need to update your models.**

The Rust version of Oneil has syntax updates from the Python version.

1. **Instead of being delimited by indentation, notes are delimited with
   tildes.** For example:

   ```oneil
   # Before
       This is a single line note

       This is
       a multi-line
       note.

   # After
   ~ This is a single line note

   ~~~
   This is
   a multi-line
   note.
   ~~~
   ```

   This will need to be updated in any old model code. A script has been written
   in `tools/convert_notes.py` that can do this automatically. It can be used as
   follows:

   ```sh
   python3 tools/convert_notes.py <FILE1> [FILE2 ...]
   ```

   Each file that is passed in will be overwritten with the updated code
   in-place. In addition, a backup file with the extension `.bak` will be
   created with the old code. This file can be deleted once it has been verified
   that the conversion worked correctly.

2. **Discrete parameter string values are now enclosed in quotes.** For example:

   ```oneil
   # Before
   X[foo, bar, baz]: x = foo
   Y: y = { 1 if x == foo
          { 2 if x == bar
          { 3 if x == baz

   # After
   X['foo', 'bar', 'baz']: x = 'foo'
   Y: y = { 1 if x == 'foo'
          { 2 if x == 'bar'
          { 3 if x == 'baz'
   ```

   This change does not have an automatic fix (yet, at least), and must be done
   by hand.

3. **Pointer parameters are no longer necessary.** For example:

   ```oneil
   # Before
   X: x = 1
   Y: y => x

   # After
   X: x = 1
   Y: y = x
   ```

### Toolchain

Oneil has the several tools available for working with Oneil code.

#### Vim Support

Oneil supports syntax highlighting in vim. Oneil is already designed for readability, but syntax highlighting makes it even better.

*This highlighting has not been updated since the updates to the syntax have been released, so it might have some problems highlighting*.

Create a `~/.vim` directory with subdirectories `syntax` and `ftdetect` if they don't exist yet. From this directory create soft links to the files in the `vim` directory of the Oneil repository.

``` { .sh }
mkdir ~/.vim
cd ~/.vim
mkdir syntax
mkdir ftdetect
cd ~/.vim/syntax
ln -s path/to/oneil/vim/syntax/oneil.vim
cd ../ftdetect
ln -s path/to/oneil/vim/ftdetect/oneil.vim
```

If you don't have a `~\.vim` directory, you can just symlink the directory itself.

``` { .sh }
ln -s $CAREWEATHER/oneil/vim ~/.vim
```

#### VS Code Support

Oneil has an [extension](https://marketplace.visualstudio.com/items?itemName=careweather.oneil) available in VS Code. This extension is also available in VS Code forks such as Cursor.

## Syntax

Here is a brief overview of the syntax of Oneil.

### Parameters

The oneil language supports definition of a collection of "parameters", with *independent* parameters that have specified values and *dependent* parameters that are functions of other parameters. The syntax for defining a parameter in Oneil is:

``` { .on }
Preamble: Body
```

Or expressed in more detail:

``` { .on }
Name (Limits): ID = Assignment :Units
```

Detailed syntax rules for each of these parts is described in the following sections.

### Preamble Syntax

At a minimum, the preamble must contain the name of the parameter. The name must be plain text with no special characters except apostrophes. For example:

``` { .on }
Mass: ...
Angular position: ...
Boltzmann's constant: ...
```

The preamble may also specify limits on the parameter. Oneil checks all parameters to ensure their assigned or calculated values are within their limits. Limits can be specified using ints, floats, or math constants as follows:

``` { .on }
Mass (0, 10e8): ...
Angular position (-pi/2, pi/2): ...
Total heat flux (-inf, inf): ...
```

If no limits are specified, Oneil assumes the allowable domain is 0 to infinity (non-negative real numbers). Limits are given in the same units as the parameter.

Use limits to ensure fundamental physical properties are true (distances can't be negative, for example). To check that parameter values are reasonable with respect to other parameters, use a test (see below).

You can specify discrete limits using brackets. These can be words or numbers:

``` { .on }
Space domain ['EarthOrbit', 'interplanetary', 'interstellar']: ...
Dimensions [1, 2, 3]: ...
```

You can mark a parameter as a "performance" parameter by prepending it with a `$`. Performance parameters are included in summaries of the model.

``` { .on }
$ Artificial gravity: ...
...
```

### Body Syntax

The first element of the body is the ID. It follows the first colon and comes before the equals sign. The ID is a short alternative to the name used for readable equations. It's the key used for a parameter in the model namespace and must be unique within a model file.

``` { .on }
Cylinder diameter: D = ...
Rotation rate: omega = ...
Crew count: N_c = ...
Resident count: N_r = ...
Orbital altitude: h = ...
Quiescent power: P_q = ...
Active power: P_a = ...
```

Oneil names and IDs overcome the classic naming conflict in mathematical computing: long variable names make equations unreadable while short names make variables unidentifiable. Oneil makes it possible specify equations in short form while keeping parameter meaning clear.

#### Assignment

The parameter assignment can either be a value (independent) or an equation (dependent).

Value assignments can specify a single value or a minimum and maximum value separated by a pipe. These values use numbers, math constants, or discrete values, which are strings wrapped in single quotes (`'`).

``` { .on }
Window count: n_w = 20
Communications amplifier efficiency (0, 1): eta_c = 0.5|0.7
Space domain ['earth_orbital', 'interplanetary', 'interstellar']: D_s = 'interstellar'
```

Equation assignments define a parameter as a function of other parameters using parameter IDs (e.g. `m*x + b` where `m`, `x`, and `b` are parameter IDs).

``` { .on }
Cylinder radius: r = D/2 : ...
Artificial gravity: g_a = r*omega^2 : ...
```

Alternate equations for the minimum and maximum case can be given, separated by a pipe.

``` { .on }
Power consumption: P_c = eta_c*P_q | eta_c*P_a
```

#### Units

Units are specified after a second colon with the "^" operator for exponents and a "/" preceeding *each* unit in the denominator. Units must be specified if the parameter has units, but can be left off for unitless parameters.

``` { .on }
Mass (0, 100000000): m = 1e6 :kg
Cylinder diameter: D = 0.5 :km
Angular position: theta_p = pi/2
Window count: n_w = 20
Rotation rate: omega = 1 :deg/min
Amplifier efficiency (0, 1): eta = 0.5|0.7
Boltzmann's constant: C_b = 1.380649e-23 :m^2*kg/s^2/K
Cylinder radius: r = D/2 :km
Artificial gravity: g_a = r*omega^2 :m/s^2
Temperature: T = temperature(D) :K
```

You can review supported units using the [CLI builtin units command](#builtins). If a unit isn't supported, you can specify it in terms of base units: `kg`, `m`, `s`, `K`, `A`, `b`, `$`.

Oneil supports `dB` as a nonlinear display unit. When any unit is specified with prefix `dB`, Oneil internally converts the parameter to the corresponding linear value, performs all calculations in linear terms, and reconverts the value to dB for display. This means that equations that contain parameters with dB units should use linear math. For example, when calculating the signal to noise ratio by hand, you might subtract the noise (dB) from the signal (dB), but in oneil, you divide the signal by the noise:

``` { .on }
Noise power: P_n = -100 :dBmW
Signal power: P_s = -90 :dBmW
Signal-to-noise ratio: S_N = P_s/P_n
```

While [limits](#preamble-syntax) are typically specified in the parameter's units, limits only support linear values. Parameters with dB units should typically not specify a limit (other than the default 0-inf) since negative linear values would lead to imaginary dB values.

> [!IMPORTANT]
> Oneil handles nearly all unit conversion in the background, but there is a [major exception with frequencies (Hz) and angular frequencies (rad/s)](#something-funny-is-happening-with-angular-frequencies-and-frequencies).

### Arithmetic

In Oneil, all number values are 64-bit floating-point values. Thus, `1`, `-0.2`,
`3.0e14`, and `-inf` are all valid values. Regular arithmetic operations are available,
including:

* `a + b` - addition
* `a - b` - subtraction
* `a \* b` - multiplication
* `a / b` - division
* `a % b` - modulo
* `a ^ b` - exponent
* `(a)` - grouping

In addition, numbers can be compared with comparison operators:

* `a == b` - equals
* `a != b` - not equal
* `a < b` - less than
* `a <= b` - less than or equal
* `a > b` - greater than
* `a >= b` - greater than or equal

In addition, builtin functions are provided, as described later.

### Interval Arithmetic

In addition to standard "scalar" values, Oneil supports "interval" values.
Interval values are a values with a *minimum* and a *maximum* value, and
can be created using the bar operator, `|`.

```oneil
# an interval from 0 to 5
My interval: i = 0 | 5
```

Intervals can also be combined with the bar operator. This creates
the smallest interval that covers both intervals. In other words,
it creates an interval with the lesser minimum and the greater
maximum.

```oneil
Interval 1: i1 = 0 | 2
Interval 2: i2 = 4 | 6
Combined: c = i1 | i2
# => min(0, 4) | max(2, 6)
# => 0 | 6
```

#### Arithmetic Operators

The same operators that are defined for scalar values are also
defined for interval values: `+`, `-`, `\*`, `/`, `%`, and `^`.

However, interval arithmetic behaves slightly differently than one
might inspect, since intervals represent a *range* of values,
rather than an individual value.

One example of this is that when evaluating subtraction, one might initially
expect to subtract the min from the min and the max and the max:
`i1 - i2 == min(i1) - min(i2) | max(i1) - max(i2)`. However, this produces
incorrect results. For example,

```oneil
X: x = 10 | 15
Y: y = 0 | 5
Z: z = x - y
# => (10 | 15) - (0 | 5)
# => 10 - 0 | 15 - 5
# => 10 | 10
```

Instead, subtraction is implemented as `min(i1) - max(i2) | max(i1) - min(i2)`.

```oneil
X: x = 10 | 15
Y: y = 0 | 5
Z: z = x - y
# => (10 | 15) - (0 | 5)
# => 10 - 5 | 15 - 0
# => 5 | 15
```

All arithmetic operators produce arithmetically correct results. For more details,
on their implementation, refer to the
[paper review](docs/research/2025-11-13-interval-arithmetic-paper-review.md) or
the implementation code.

<!-- TODO: figure out the best way to make these details accessible -->

#### Escaping the interval arithmetic implementation

Oneil's implementation of interval arithmetic intends to be arithmetically correct.
That is to say, if you were to replace every interval in an expression with a value
within that interval and then evaluated the expression, the resulting value would
be contained within the interval produced by evaluating the initial expression.
This is known as the
[inclusion property](docs/research/2025-11-13-interval-arithmetic-paper-review.md#inclusion-property).

However, the arithmetic may *overapproximate* an interval. For example, we would
expect `a - a` to always be equal to `0`, no matter what `a` is. Therefore, if
`a` is an interval, we would expect `a - a` to produce an interval with `0` as
both the minimum and maximum value, `0 | 0`.

If we take `a` as `0 | 1`, however, `a - a` would produce the interval `-1 | 1`.
While this answer is technically correct (`0 | 0` is contained within `-1 | 1`),
it isn't as precise as we would expect.

This problem is known as the
[dependency problem](https://en.wikipedia.org/wiki/Interval_arithmetic#Dependency_problem).

If more precision is needed (such as in geometry, where relationships such as `a - a = 0`
are important), you can "escape" interval arithmetic using `min(i)` and
`max(i)` functions, which get the minimum and maximum values of an interval. This allows
users to operate on scalar values until they are ready to return to interval arithmetic
using the bar operator. For example, instead of `a - a`, a user could use
`min(a) - min(a) | max(a) - max(a)` in order to get a more precise result.

To simplify this escape, Oneil provides the `--` and `//` operators,
which behave as follows:

| Operator | Equivalent To                        |
|----------|--------------------------------------|
| `a -- b` | `min(a) - min(b) \| max(a) - max(b)` |
| `a // b` | `min(a) / min(b) \| max(a) / max(b)` |

#### Comparison

Intervals can also be compared with each other using the comparison operators,
which are implemented as defined below.

| Operator | Equivalent To                           | Description                                                           |
|----------|-----------------------------------------|-----------------------------------------------------------------------|
| `a == b` | `min(a) == min(b) and max(a) == max(b)` | The min and the max are the same                                      |
| `a != b` | `min(a) != min(b) or max(a) != max(b)`  | The min or the max is not the same                                    |
| `a < b`  | `max(a) < min(b)`                       | The max value of `a` is less than the min value of `b`                |
| `a <= b` | `max(a) <= min(b)`                      | The max value of `a` is less than or equal to the min value of `b`    |
| `a > b`  | `min(a) > max(b)`                       | The min value of `a` is greater than the max value of `b`             |
| `a >= b` | `min(a) >= max(b)`                      | The min value of `a` is greater than or equal to the max value of `b` |

### Builtin Functions

Oneil has the following builtin functions.

| Function      | Description                                                                                                      |
|---------------|------------------------------------------------------------------------------------------------------------------|
| `min(a)`      | If `a` is an interval, return the minimum value of the interval. Otherwise, return the value of `a`              |
| `min(a, ...)` | Find the minimum value of the given values. If a value is an interval, the minimum value of the interval is used |
| `max(a)`      | If `a` is an interval, return the maximum value of the interval. Otherwise, return the value of `a`              |
| `max(a, ...)` | Find the maximum value of the given values. If a value is an interval, the maximum value of the interval is used |
| `mid(a, b)`   | Find the midpoint between the                                                                                    |
| `range(a)`    | Return the width of an interval (max−min)                                                                        |
| `sqrt(a)`     | Calculate the square root                                                                                        |
| `sin(a)`      | Calculate the sine                                                                                               |
| `cos(a)`      | Calculate the cosine                                                                                             |
| `tan(a)`      | Calculate the tangent                                                                                            |
| `asin(a)`     | Calculate the arcsine                                                                                            |
| `acos(a)`     | Calculate the arccosine                                                                                          |
| `atan(a)`     | Calculate the arctangent                                                                                         |
| `ln(a)`       | Natural logarithm                                                                                                |
| `log(a)`      | Base 10 logarithm                                                                                                |
| `log10(a)`    | Base 10 logarithm (alias for `log(a)`)                                                                           |
| `floor(a)`    | Round down to nearest integer                                                                                    |
| `ceiling(a)`  | Round up to nearest integer                                                                                      |
| `abs(a)`      | Absolute value                                                                                                   |
| `sign(a)`     | Sign of value (−1, 0, 1)                                                                                         |
| `strip(a)`    | Remove units from calculation                                                                                    |
| `mnmx(...)`   | Gets the minimum and maximum of the list of values                                                               |

### Piecewise Equations

Piecewise equations can be used for parameter assignments.

``` { .on }
Orbital gravity: g_o = {G*m_E/h^2 if D_s == 'earth_orbital' :km/s
                       {G*m_S/h^2 if D_s == 'interplanetary'
                       {G*m_G/h^2 if D_s == 'interstellar'
```

(`m_E`, `m_S`, and `m_G` are the masses of the Earth, Sun, and galactic center)

Conditions are evaluated in order, and the first equation corresponding to a true condition is calculated to obtain the value for the parameter.

### Python Functions

For functions not supported by the above equation formats, you can define a python function and link it.

The Python functions are stored in a separate python file, which must be imported in the Oneil file.

``` { .on }
import <name of functions file>
```

That file should simply define functions matching the name used in the parameter:

``` { .py }
import numpy as np

def temperature(transit_mode):
    ...
```

In the Oneil file, give the python function on the right hand of the equation, including other parameters as inputs:

``` { .on }
Temperature: T = temperature(D) :K
```

#### Fallback Calculations

Python functions may have dependencies that aren't always available. You can specify a fallback calculation using the `?` operator. If the Python function fails (e.g., missing dependencies, runtime errors), Oneil will use the fallback and warn the user:

```oneil
Temperature: T = expensive_simulation(D) ? D * 0.5 + 273 :K
```

In this example, if `expensive_simulation` fails, Oneil will calculate `D * 0.5 + 273` instead and display a warning that the Python function should be run for greater accuracy.

This is particularly useful for:

* Sharing models with users who may not have all Python dependencies installed
* Providing quick approximations during iterative design
* Graceful degradation when simulations fail

### References and Submodels

One of the purposes of Oneil's models is to be able to represent **collections of systems and subsystems**. To this end, Oneil provides two different ways to import a model.

The first way to import a model is as a **reference**. When a model is imported as a reference, all of the *reference model parameters* are made available through the *reference alias*. The *reference alias* is either the alias provided or, if there isn't one, the name of the model.

```oneil
# === constants.on ===
Gravity of Earth: g = 9.8 :m/s^2


# === my_model.on ===
Mass of box: m_b = 5 :kg

# reference with alias
reference constants as c
Weight of box: w_b = m_b * g.c :N

# reference without alias
reference constants
Weight of box: w_b = m_b * g.constants :N
```

The second way to import a model is as a **submodel**. Like with a reference, all of the *submodel parameters* are available through the *submodel alias*. In addition to this, the model is also exported as a *submodel* of the current model. This means that the imported model can be referenced as `model.submodel`.

```oneil
# === radar.on ===
Radar cost: cost = 1000 :$


# === solar_panel.on ===
Solar panel cost: cost = 500 :$


# === satellite.on ===
submodel radar
submodel solar_panel as solar

Satellite cost: cost = cost.radar + cost.solar :$


# === product.on ===
# Import satellite as a whole; cost.satellite.radar, cost.satellite.solar, etc.
submodel satellite

# ... or extract submodels directly to parent scope with an extraction list:
submodel satellite [radar, solar_panel as solar]
# Now accessible as cost.radar, cost.solar
```

Note that in the case of a submodel, *the submodel and reference name may be different*. If an alias is provided, it will be used as the reference name, but not as the submodel name. The submodel name will always be the name of the model.

### Designs

A design consists of the values assigned to independent parameters in a model. Oneil model files include a default design, but Oneil makes it easy to overwrite that default with alternative designs. Design files use the same syntax of model files, but only require the body instead of the whole line (no preamble required). Designs let you change a subset of the independent parameters from the default design. For example,

``` { .on }
m = 1e6 :kg
D = 0.5 :km
omega = 1 :deg/min
case = clockwise
L = L.d
```

To use a design, see the command line interface `design` command. A design parameter overwrites the value of the model parameter while keeping the original metadata. If you want your design to alter a submodel parameter, you'll need to make sure the corresponding model uses that submodel.

### Tests

Models can also specify tests to verify model reasonability and accuracy. Tests use math expressions with comparison operators (`==`, `>`, `<`, `>=`, `<=`, `!=`) to return True or False. Tests can't include unit specifications, so any values with units must be specified separately and used in the test equation. This turns out to be a useful limitation for preventing magic numbers.

``` { .on }
Earth gravity: g_E = 9.81 :m/s^2

test : g_E*0.9 <= g_a <= g_E*1.1

    ~ The artificial gravity should be within 10% of Earth's gravity.
```

### Notes and Comments

Oneil defines "notes" and "comments" differently. Notes are comments that you want to show up in reports explaining and justifying the model or design. Comments are "notes to self" that don't show up in any reports. When the model is exported to a report, notes are included, but comments are not.

Oneil recognizes notes as any line that begins with a `~` or any lines that are enclosed by `~~~` on their own line . When a note is found, Oneil will tie it to the most recently-defined parameter or test (above the note in the file). If none are found, Oneil will tie the note to the model itself. On export, notes are processed as LaTeX.

Oneil recognizes any line starting with `#` as a comment.

In the following example, "O'neill cylinder for..." is a note tied to the model while `cylinder radius` has no note and `standard Earth gravity` has "From \href..." as its note. "#TODO..." is ignored as a comment.

``` { .on }
    ~ O'neill cylinder for supporting long-term human habitation in deep space.

#TODO: refactor this as a function of the diameter
Cylinder diameter: d = 0.5 :km

Standard Earth gravity: g_E = 9.81 m/s^2
    ~~~
    From \href{https://en.wikipedia.org/wiki/Gravity_of_Earth}{wikipedia}.

    For more information, see \href{https://example.com/info}{this page}.
    ~~~
```

## Using the CLI

The authoritative reference for flags, defaults, and examples is the built-in help: run `oneil --help`. For a subcommand, run `oneil <command> --help` (for example `oneil eval --help`).

### Invocation

You can use the CLI in two ways:

1. **`oneil [OPTIONS] <FILE>`** — Evaluate an Oneil model. If you do not pass a subcommand, the CLI parses arguments the same as **`oneil eval`**, so `oneil model.on` and `oneil eval model.on` are equivalent.

2. **`oneil <COMMAND> ...`** — Run a named command (`eval`, `test`, `tree`, and so on).

### Commands

These are the commands listed by `oneil --help`:

| Command       | Alias | Purpose                                                                                                          |
|---------------|-------|------------------------------------------------------------------------------------------------------------------|
| `eval`        | `e`   | Evaluate a model and print results.                                                                              |
| `test`        | `t`   | Run tests in a model.                                                                                            |
| `tree`        | —     | Print the dependency or reference tree for one or more parameters.                                               |
| `builtins`    | —     | Print language builtins; see `oneil builtins --help` for subcommands (`all`, `unit`, `func`, `value`, `prefix`). |
| `independent` | —     | Print independent parameters in a model.                                                                         |
| `lsp`         | —     | Run the language server.                                                                                         |
| `help`        | —     | Print help for the program or subcommands.                                                                       |

### Options for evaluation (`eval` and default `<FILE>`)

The evaluation path accepts the options shown in `oneil --help` / `oneil eval --help`. In short:

* **`-p` / `--params`** — Comma-separated parameters to print. Use dots for submodels (for example `a.sub2.sub1` is parameter `a` inside nested submodels). When set, the default print mode for “which parameters to show” is replaced by this explicit list.

* **`-P` / `--print`** — When `--params` is not used, choose what to print: `trace` (trace `*`, debug `**`, and performance `$` markers), `perf` (`$` only), or `all`. Default is `trace`.

* **`-x` / `--expr`** — Evaluate an expression in the model’s context; repeat the flag for multiple expressions.

* **`-r` / `--recursive`** — Include submodels, not only the top model.

* **`-w` / `--watch`** — Watch files and re-evaluate when they change.

* **`-D` / `--debug`** — After errors, still show partial results.

* **`--no-header`**, **`--no-test-report`**, **`--no-parameters`** — Suppress parts of the output. **`--no-parameters`** overrides `--params` and print mode.

* **`--sig-figs`** — Significant figures for printed numbers (default 4).

* **`--no-colors`** — Turn off ANSI colors (useful for logs or terminals without color).

* **`--venv-path`** — Python virtual environment to use when Python integration is enabled; if unset and `VIRTUAL_ENV` is unset, the CLI will discover `venv` or `.venv` by searching upward from the current directory.

### `test` (`t`)

**Usage:** `oneil test [OPTIONS] <FILE>`

Runs tests defined in the model at `<FILE>`.

* **`-r` / `--recursive`** — Include test results from submodels, not only the top model.
* **`-D` / `--debug`** — After errors, still show partial test output.
* **`--with-header`** — Print the results header (model path and test summary) before the test results.
* **`--format <text|json>`** — Output format. `text` (default) prints human-readable, colorized output. `json` prints a single machine-readable JSON object to stdout (diagnostics plus per-test results, including dependency values for failures) intended for CI tooling — see `oneil_cli::json_test_report` for the schema.
* **`--sig-figs`**, **`--no-colors`**, **`--venv-path`** — Same role as for `eval` (see `oneil test --help`).

`oneil test` exits with status 1 if there were any error diagnostics or any test failed (in either format), so it can be used directly as a CI gate without additional tooling — e.g. `oneil test model.on && echo "tests passed"`.

### `tree`

**Usage:** `oneil tree [OPTIONS] <FILE> <PARAM>...`

Prints a tree for each named parameter. `<PARAM>...` is one or more parameter names.

* **`-u` / `--up`** — Tree of parameters that *reference* the given parameters (mutually exclusive with `--down`).
* **`-d` / `--down`** — Tree of *dependencies* of the given parameters. If neither `--up` nor `--down` is set, behavior matches `--down`.
* **`-r` / `--recursive`** — Include submodel values in the tree, not only the top model.
* **`--depth <DEPTH>`** — Limit tree depth (default is full depth).
* **`-D` / `--debug`** — After errors, still show partial trees.
* **`--sig-figs`**, **`--no-colors`**, **`--venv-path`** — Same as other commands (see `oneil tree --help`).

### `independent`

**Usage:** `oneil independent [OPTIONS] <FILE>`

Lists parameters that are independent (assigned directly rather than by equation) in the model at `<FILE>`.

* **`-r` / `--recursive`** — Include independents from submodels as well as the top model.
* **`-D` / `--debug`** — After errors, still show partial results.
* **`--sig-figs`**, **`--no-colors`**, **`--venv-path`** — See `oneil independent --help`.

### `builtins`

**Usage:** `oneil builtins [OPTIONS]` or `oneil builtins <COMMAND> ...`

Without a subcommand, run `oneil builtins --help` for the command list. Subcommands:

| Subcommand | Arguments             | Purpose                                         |
|------------|-----------------------|-------------------------------------------------|
| `all`      | —                     | Print all builtins.                             |
| `unit`     | optional `[UNIT]`     | List units, or search for a specific unit name. |
| `func`     | optional `[FUNCTION]` | List builtin functions, or search for one.      |
| `value`    | optional `[VALUE]`    | List builtin values, or search for one.         |
| `prefix`   | optional `[PREFIX]`   | List unit prefixes, or search for one.          |

Each subcommand accepts **`--sig-figs`**, **`--no-colors`**, and **`--venv-path`** where applicable; see `oneil builtins <COMMAND> --help`.

### Examples

```sh
oneil model.on
oneil eval model.on -P all
oneil eval model.on -p r,g_a -x "r / 10 * omega^2"
oneil test model.on
oneil tree model.on g_a
oneil builtins unit
```

For building and running from the repository, see the [quickstart](#quickstart) (`cargo run -- path/to/your/model.on`).

## CI Integration for Model Repos

Repos that define Oneil models (e.g. `.on` files checked into their own repo)
can use [`careweather/oneil/actions/model-test-report`](actions/model-test-report/README.md),
a GitHub Action that runs `oneil test --format json` and, when comparing a
PR's head against its base, reports regressions and fixes instead of just a
raw pass/fail count. See the action's README for inputs, outputs, and
example workflows.

## Using Oneil with AI

Oneil can be used effectively with AI to model and design systems. For an example of an AI ruleset, see [Appendix B in the guide](docs/guide/src/b-using-ai.md).

## Known Issues and Limitations

* The Vim syntax highlighter gets *really* slow if you try to paste large amounts of LaTeX in. For now, make sure to paste large blocks of LaTeX using a different text editor or temporarily remove the ".on" file extension while you do.
* The Vim syntax highlighter breaks for the rest of the file after a LaTeX syntax error in a note. As a result, the rest of the file will be highlighted as a note.

And many more. These will be ported to GitHub issues for planning and visibility in coming months. If you find an issue that isn't listed in GitHub, please post it.

## Troubleshooting

### Something funny is happening with angular frequencies and frequencies

The funny thing about Hz and rad/s is that `1 Hz != 1 rad/s` even though `1 Hz = 1/s` and `1 rad/s = 1/s`. You can [thank the International System of Units for this madness](https://iopscience.iop.org/article/10.1088/1681-7575/ac0240). To escape this, Oneil doesn't recognize the SI definition of Hz. If you specify Hz as a unit, Oneil will internally convert it to rad/s by multiplying by 2 pi. If you want to use a frequency in an equation that expects Hz, you need to make sure the equation converts your frequency (rad/s) to Hz. For example, instead of `c=lambda*f` for the speed of light, you would use `c=lambda*f/(2*pi)`.

> As a side note, some people have suggested that this problem is solved if you use `cycles` as a base unit and let `Hz = 1 cycle/s`, but this quickly becomes messy as cycles will get propagated throughout your model where you don't want it. It's much cleaner to convert rad/s to Hz in equations that expect it.

### Oneil has a bug

You can report bugs using the issues section on Github. If you want to try and fix a bug yourself, see [`CONTRIBUTING.md`](CONTRIBUTING.md) for help.

### TexMaker works, but VS Code doesn't

Try closing all VS Code files and closing VS Code to clear its mystery cache.

## Contributing

If you've found a bug or would like to request a feature, feel free to [submit an issue](https://github.com/careweather/oneil/issues)!

If you would like to contribute code, read [`CONTRIBUTING.md`](CONTRIBUTING.md), then feel free to [submit a pull request](https://github.com/careweather/oneil/pulls)!

## About

The initial methodology that inspired Oneil was proposed in Chapter 3 of [Concepts for Rapid-refresh, Global Ocean Surface Wind Measurement Evaluated Using Full-system Parametric Extrema Modeling](https://scholarsarchive.byu.edu/cgi/viewcontent.cgi?article=10166&context=etd), by M. Patrick Walton. For that work, the methodology was painfully implemented in a Google sheet. The conclusion provided ideas and inspiration for early versions of Oneil.

Oneil was developed at Care Weather Technologies, Inc. to support design of the Veery scatterometer. Veery is designed to perform as well as $100M heritage scatterometers at orders of magnitude less cost. This dramatic improvement is facilitated in part by Oneil's streamlined systems engineering capabilities.

Oneil is named after American physicist and space activist [Gerard K. O'Neill](https://en.wikipedia.org/wiki/Gerard_K._O%27Neill) who proposed the gargantuan space settlements known as [O'Neill cylinders](https://en.wikipedia.org/wiki/O%27Neill_cylinder). We built Oneil to meet our own needs, but we hope it stitches together the many domains required to make O'Neill cylinders and move humanity up the [Kardashev scale](https://en.wikipedia.org/wiki/Kardashev_scale).
