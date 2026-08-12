# Appendix A: Python API (oneil) — deprecated

> **Deprecated.** Oneil 1.0 ships the **Rust CLI** as the primary interface.
> This Python library (`import oneil`) is kept for compatibility. New work
> should use the `oneil` command-line tool; see
> [Installation](./02-installation.md).

The Oneil Rust implementation exposes a Python library built from
`oneil_python::py_compat`. It provides Oneil’s builtin values, units, and
functions, plus Python classes for **Interval**, **MeasuredNumber**, and
**Unit**, so you can use Oneil’s number and unit semantics from Python.

## Installation

The Python package is built with [maturin](https://www.maturin.rs/) and the
`python-lib` feature. From the repository root:

```sh
pip install -e .
```

Or build a wheel:

```sh
maturin build --release -f
pip install target/wheels/oneil-*.whl
```

Version tags may also attach pre-built wheels on the
[GitHub Releases](https://github.com/careweather/oneil/releases) page (marked
deprecated in the release notes).

Requires **Python 3.10+**.

## Module layout

The package is named `oneil`. It exposes:

| Submodule / class      | Description                                                                             |
|------------------------|-----------------------------------------------------------------------------------------|
| `oneil.values`         | Builtin constants (e.g. `pi`, `e`) as Python objects                                    |
| `oneil.functions`      | Builtin functions (e.g. `min`, `max`, `sqrt`, `sin`) as callables                       |
| `oneil.units`          | Builtin units (e.g. `m`, `kg`, `s`) as `Unit` instances, including SI-prefixed variants |
| `oneil.Interval`       | Class for closed numeric intervals                                                      |
| `oneil.MeasuredNumber` | Class for a number (scalar or interval) with a unit                                     |
| `oneil.Unit`           | Class for dimensional units                                                             |

---

## `oneil.values`

Constants matching Oneil’s builtin values. Each is converted to a Python object (float, or other value type as in Oneil).

- **`pi`** — π
- **`e`** — Euler’s number

Example:

```python
import oneil

# for now, Oneil doesn't support direct imports like the following,
# but it may in the future
#from oneil.values import pi as on_pi, e as on_e

on_pi = oneil.values.pi
on_e = oneil.values.e

print(on_pi, on_e)
```

---

## `oneil.functions`

Oneil’s builtin functions as callables. They accept the same conceptual argument types as in Oneil: Python `float`, `oneil.Interval`, and `oneil.MeasuredNumber`. Arguments are converted to Oneil values; on type or unit mismatch they raise `TypeError` or `ValueError`.

Supported functions:

- **`min`**, **`max`** — minimum/maximum of numbers (scalars or intervals).
- **`sin`**, **`cos`**, **`tan`** — trig (radians).
- **`asin`**, **`acos`**, **`atan`** — inverse trig (result in radians).
- **`sqrt`** — square root.
- **`ln`**, **`log2`**, **`log10`** — natural, base-2, and base-10 logarithms.
- **`floor`**, **`ceiling`** — round down/up to nearest integer.
- **`range`** — with one argument (an interval): max − min; with two arguments: their difference.
- **`abs`**, **`sign`** — absolute value and sign.
- **`mid`** — with one argument (an interval): midpoint; with two arguments: midpoint between them.
- **`strip`** — strip units from a measured number, returning the numeric value.
- **`mnmx`** — return both the minimum and maximum of the given values.

Call with positional arguments:

```python
import oneil

# for now, Oneil doesn't support direct imports like the following,
# but it may in the future
#from oneil.functions import sqrt as on_sqrt, min as on_min

on_sqrt = oneil.functions.sqrt
on_min = oneil.functions.min

print(on_sqrt(2.0))
print(on_min(1.0, 2.0, 3.0))
```

---

## `oneil.units`

Builtin units as `oneil.Unit` instances. Names match Oneil’s unit aliases, with two substitutions for valid Python identifiers:

- `%` → **`percent`**
- `$` → **`dollar`**

Units that support SI prefixes (e.g. `m`, `g`, `s`) also have prefixed names (e.g. `km`, `mm`, `kg`, `mg`, `ms`).

Examples:

```python
import oneil
from oneil.units import m, kg, seconds, dollar, percent
# prefixed
from oneil.units import km, mm, kg, mg
```

---

## `oneil.Interval`

Closed interval of real numbers with a minimum and maximum. Wraps Oneil’s interval type; supports arithmetic and comparison with other `Interval` instances or with Python scalars (a scalar is treated as a point interval).

### `oneil.Interval` constructor

- **`Interval(min, max)`** — `min` and `max` must not be NaN, and `min` ≤ `max`; otherwise `ValueError` is raised.

### `oneil.Interval` class methods

- **`Interval.empty()`** — empty interval.
- **`Interval.zero()`** — [0, 0].

### `oneil.Interval` instance methods and properties

- **`min`**, **`max`** — bounds (read-only).
- **`is_empty()`**, **`is_valid()`** — emptiness and validity checks.
- **`intersection(other)`** — intersection with another `Interval`.
- **`tightest_enclosing_interval(other)`** — smallest interval containing both.
- **`contains(other)`** — whether this interval contains the other.

Arithmetic and comparison: `+`, `-`, `*`, `/`, `%`, `**`, unary `+`/`-`, and `==`, `!=`, `<`, `<=`, `>`, `>=`. The other operand may be an `Interval` or a scalar (`float`).

Math methods (return new `Interval`): **`sqrt`**, **`ln`**, **`log10`**, **`log2`**, **`abs`**, **`sign`**, **`sin`**, **`cos`**, **`tan`**, **`asin`**, **`acos`**, **`atan`**, **`floor`**, **`ceiling`**, **`pow(exponent)`** (exponent is an `Interval`).

Specialized (interval) operations:

- **`escaped_sub(other)`** — subtract using (min−min, max−max).
- **`escaped_div(other)`** — divide using (min/min, max/max).

---

## `oneil.MeasuredNumber`

A number (scalar or interval) with a unit. Wraps Oneil’s measured number type; arithmetic and comparison enforce dimensional consistency where required.

### `oneil.MeasuredNumber` constructor

- **`MeasuredNumber(value, unit)`** — `value` is a `float`, an `Interval`, or a `MeasuredNumber`; `unit` is a `Unit`. Builds a measured number from that value and unit.

### `oneil.MeasuredNumber` instance methods

- **`unit()`** — returns the `Unit` of this measured number.
- **`into_number_and_unit()`** — returns a tuple `(number, unit)` where `number` is the numeric part (float or `Interval`) in this object’s unit.
- **`into_number_using_unit(unit)`** — converts to a number (float or `Interval`) in the given `Unit`; raises if dimensions don’t match.
- **`into_unitless_number()`** — same as converting to a dimensionless unit; raises if not dimensionless.
- **`with_unit(unit)`** — returns a copy with the given unit; raises if not dimensionally equivalent.

Arithmetic: `+`, `-`, `*`, `/`, `%`, `**` with other `MeasuredNumber` or, when the measured number is effectively unitless, with plain numbers. Unit mismatches raise `ValueError`.

Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=` (with same conversion rules as arithmetic).

Math (return `MeasuredNumber`): **`sqrt`**, **`ln`**, **`log10`**, **`log2`**, **`abs`**, **`floor`**, **`ceiling`**.

Other:

- **`min()`**, **`max()`** — minimum/maximum as measured numbers.
- **`min_max(other)`** — tightest enclosing measured number of this and `other`.
- **`escaped_sub(other)`**, **`escaped_div(other)`** — escaped subtraction and division (units must match).

---

## `oneil.Unit`

Represents a dimensional unit (dimensions, magnitude, optional decibel flag, and display info).

### `oneil.Unit` constructor

- **`Unit(*, dimensions=None, magnitude=None, is_db=None, display_unit)`**  
  - **`dimensions`** — optional dict mapping dimension keys to exponents (e.g. `{"m": 1, "s": -1}`). Valid keys: `"kg"`, `"m"`, `"s"`, `"K"`, `"A"`, `"b"`, `"$"`, `"mol"`, `"cd"`.
  - **`magnitude`** — optional scale (default 1.0).
  - **`is_db`** — optional decibel flag (default `False`).
  - **`display_unit`** — required string used as the display name (single unit, exponent 1).

### `oneil.Unit` class methods

- **`Unit.one()`** — dimensionless unit 1.

### `oneil.Unit` properties and methods

- **`magnitude`**, **`is_db`**, **`display_string`** — magnitude, decibel flag, and display string.
- **`get_dimensions()`** — dict of dimension key → exponent.
- **`is_dimensionless()`** — whether the unit is dimensionless.
- **`dimensionally_eq(other)`** — same dimensions as another `Unit`.
- **`dimensions_match(dimensions)`** — dimensions match the given dict.
- **`numerically_eq(other)`** — same dimensions, magnitude, and `is_db`.

Arithmetic: `*`, `/`, `**` (exponent as float) with other `Unit` instances.

- **`with_is_db_as(is_db)`** — copy with decibel flag set.
- **`mul_magnitude(factor)`** — copy with magnitude multiplied.
- **`pow(exponent)`** — unit raised to a power.

---

## Value conversion (Python ↔ Oneil)

From Python into Oneil’s value type:

- **`bool`** → boolean  
- **`str`** → string  
- **`float`** → scalar number  
- **`oneil.Interval`** → interval number  
- **`oneil.MeasuredNumber`** → measured number  

From Oneil to Python:

- Boolean → **`bool`**  
- String → **`str`**  
- Scalar number → **`float`**  
- Interval number → **`oneil.Interval`**  
- Measured number → **`oneil.MeasuredNumber`**  

The builtin functions in `oneil.functions` use this mapping for their arguments and return values. Passing an unsupported type raises `TypeError` with a message that includes the received type.
