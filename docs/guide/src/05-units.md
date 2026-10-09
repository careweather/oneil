# Units

One of Oneil's defining features is its unit-based type system. Oneil tracks
units, disallows invalid operations between different physical properties, and
automatically converts between differing units of the same physical property.

For example, Oneil will throw an error if you try to add a time and a distance
or compare a mass and a temperature. But it will automatically convert a length
in meters and a length in feet to a common base before adding them together.

This simplifies expressions to focus on relationships between physical
properties while preventing unit conversion errors that might
[crash your spacecraft](https://en.wikipedia.org/wiki/Mars_Climate_Orbiter).

## Units, dimensions, and magnitude

Before we get into how units work in Oneil, we're going to take a quick detour
to delve into _what_ makes units compatible. Why can you add meters and
kilometers, but not meters and kilograms? Why is a Joule equivalent to a
Watt-second?

The answer is _dimensions_. A dimension can be defined as _an aspect of
something that can be measured_. That definition is hard to understand on its
own, though, so lets consider the dimension of _time_ as an example.

### A trip to the store (and more)

When measuring how long a car takes to get from your house to the store, it
doesn't matter whether you measure in seconds, minutes, or even millennia. They
all are measurements of the dimension of _time_.

You can also measure how long it takes for you to get from the store to work,
and you can measure that in any unit of _time_ as well. Then, you could add
the values together because they both measure the dimension of _time_.

However, it wouldn't make sense to add the mass of your car to the measured travel time,
since travel time is measured in the dimension of _time_, while car mass is measured in
the dimension of _mass_.

### Supported dimensions

Oneil supports the following dimensions, listed here with their associated base
unit.

- mass: `kilogram`
- distance: `meter`
- time: `second`
- temperature: `Kelvin`
- current: `ampere`
- information: `bit`
- currency: `$` (USD)
- substance: `mole`
- luminous intensity: `candela`

_Base units_ convey a single dimension, like the unit `kilometer` with its
dimension `distance`. Derived units convey 0 to many dimensions, like the unit
`degree` which is dimensionless or the unit `Joule` with its dimensions of of
`mass`, `distance^2`, and `time^-2`. Dimensionless units are discussed in more
detail [later in this chapter](#dimensionless-units).

Each unit has 0 or more dimensions associated with it. The kilometer is defined
as having a dimension of `distance`, while a `Joule` would have the dimensions
of `mass`, `distance^2`, and `time^-2`.

There are also dimensionless units such `%`. These are discussed
[later in this chapter](#dimensionless-units).

If you haven't quite wrapped your head around dimensions yet, don't worry. You
don't need to fully understand it to use Oneil.

### Magnitudes

So if `kilometers` and `millimeters` are the same _dimensions_, then what makes
them different? The difference is in the _magnitudes_.

A magnitude is the relative size of a unit compared to the base unit. Relative
to the base unit of `meters`, `kilometers` has a magnitude of `1000` since
`1 km == 1000 m`. Meanwhile, `millimeters` has a magnitude of `0.001` because
`1 mm == 0.001 m`.

Oneil tracks magnitudes and performs automatic conversions to handle units with
different magnitudes. So when Oneil sees `1 m + 1 km`, it knows that it needs
to convert `1 km` to `1000 m` before adding. The result would therefore end up
being `1001 m`.

This automatic conversion also applies to units such as `feet` and `meters`, as
well as more complex units like `ft*lb/s^2` and `Newtons`, which could save your
climate orbiter from
[a devastating crash](https://en.wikipedia.org/wiki/Mars_Climate_Orbiter).

## Assigning units

Now that we've reviewed the motivation behind tracking units, let's get into
the practical application. For parameters with a literal value, units can be
assigned with the `:<unit>` syntax:

```oneil
# velocity.on
Distance: d = 100 :meters
Travel time: t = 20 :seconds
```

This will assign `d` to a value of `100 :meters` and `t` to a value of
`20 :seconds`. These values are now _measured numbers_, or numbers with units.

> [!NOTE]
> There are often multiple synonyms for a given unit. For example, the above
> model could also be written as
>
> ```oneil
> Distance: d = 100 :m
> Travel time: t = 20 :s
> ```
>
> To see a list of all builtin units and their synonyms, run
> `oneil builtins unit`. Also, if you would like to search for a given unit, run
> `oneil builtins unit <unit>`.

## Annotating expected units

For calculated parameters, the `:<unit>` syntax declares the _expected_ units of
a calculation, which Oneil checks.

For example, using `d` and `t` from the previous section, we can then define
velocity as

```oneil
# velocity.on (continued)
$ Velocity: v = d/t :m/s
```

defining a parameter with a measured value with the units `m/s`.

Running the model with `oneil eval velocity.on` produces

```oneil-eval-output
v = 5 :m/s  # Velocity
```

If we wanted to, we could just as easily define velocity in kilometers per hour.

```oneil
$ Velocity: v = d/t :km/hr
#                    ^^^^^ `km/hr` instead of `m/s`
```

```bash
oneil eval velocity.on
```

```oneil-eval-output
v = 18 :km/hr  # Velocity
```

Note that we did not have to do any conversions. Oneil handles that for us.
However, if we try to use incorrect units, Oneil will produce an error.

```oneil
$ Velocity: v = d/t :kg/hr
#                    ^^ `kg` instead of `km`
```

```bash
oneil eval velocity.on
```

```text
error: calculated unit does not match expected unit
 --> velocity.on:3:17
  | 
3 | $ Velocity: v = d/t :kg/hr
  |               ^--
  = note: calculated unit is `meters/seconds` but expected unit is `kg/hr`
```

In addition, Oneil requires units on any parameters whose calculations are
expected to produce a measured value. If we leave out the unit, we get an error.

Likewise, the calculation for a unitless parameter should not have a measured
result. In that case, we do leave the units out (see
[dimensionless values](#dimensionless-units)).

```oneil
$ Velocity: v = d/t
#                   ^ No unit
```

```bash
oneil eval velocity.on
```

```text
error: parameter is missing a unit
 --> velocity.on:3:17
  | 
3 | $ Velocity: v = d/t
  |                 ^--
  = note: parameter value has unit `meters/seconds`
  = help: add a unit annotation `:meters/seconds` to the parameter
```

A parameter whose value is only a reference to another parameter is the
exception. The referenced parameter already declares its unit, so the reference
inherits that unit and how it is displayed.

```oneil
Trip distance: d_t = d
```

```oneil-eval-output
d_t = 100 :m  # Trip distance
```

This also applies to a parameter from another model, such as `P_l = P_t.r`, and
to a design override that is only a reference. A design override has to keep
the dimensions of the parameter it replaces, including a unit that the
parameter inherits.

A reference to a dimensionless parameter, such as a gain in `dB` or an
efficiency in `%`, becomes a plain number, like any other dimensionless value
without a unit annotation. To keep the referenced unit, add the annotation, such
as `G_r = G :dB`.

## Composing units in a unit expression

A **unit expression** is built from one or more units separated by `*` or `/`.
Each unit can be raised to a numeric power with `^`, such as `s^2`.

Unit expressions can also use the literal `1` as a dimensionless unit. This is
used in rates such as `1/s`.

> [!WARNING]
> Multiplication and division operate left to right. So `J/kg*K` is treated as
> `(J/kg)*K` rather than `J/(kg*K)`.
>
> To express `J/(kg*K)`, explicit parentheses are required.

## Unit casting

Imagine that you have a test that takes time to start up before it runs. The full time
of the test is 5 minutes and start-up time is 10 seconds. To calculate what the actual
run time of the test is, you might write the following model.

```oneil
# testing.on
Full time: t_full = 5 :min
$ Run time: t_run = t_full - 10 :min
```

However, this will produce an error.

```bash
oneil eval testing.on
```

<!-- TODO: make this error more user friendly -->

```text
error: expected scalar with unit `min` but found scalar
 --> testing.on:2:30
  | 
4 | Run time: t_r = t_f - 10 :min
  |                       ^-
```

In other words, Oneil can't determine whether `10` is supposed to be 10 seconds,
10 minutes, or 10 hours.

The first recommended solution is to create another parameter to hold this "magic
number". You can then define a unit on that parameter.

```oneil
# testing.on
Full time: t_full = 5 :min
Startup time: t_start = 10 :s
$ Run time: t_run = t_full - t_start :min
```

```bash
oneil eval testing.on
```

```oneil-eval-output
t_run = 4.833 :min  # Run time
```

However, there are some situations where you may just want to label a unitless
number with a unit.

To do so, you can use _unit casting_. Unit casting takes the form of
`(<expression> : <unit>)`. This allows a unitless value to be assigned a unit.

Using this, the model could be rewritten as

```oneil
# testing.on
Full time: t_full = 5 :min
$ Run time: t_run = t_full - (10:s) :min
```

```bash
oneil eval testing.on
```

```oneil-eval-output
t_run = 4.833 :min  # Run time
```

## Arithmetic and comparison operators

Arithmetic and comparison operator rules and behavior are defined by the
following table. The unit of a given value `x` is indicated by `x_unit`.

| Operation                                     | Input Rules                                         | Unit Output                      |
|-----------------------------------------------|-----------------------------------------------------|----------------------------------|
| `a + b`, `a - b`, `a % b`                     | `a_unit` and `b_unit` must have the same dimensions | `a_unit`                         |
| `a * b`                                       | None                                                | `a_unit * b_unit`                |
| `a / b`                                       | None                                                | `a_unit / b_unit`                |
| `a ^ b`                                       | `b` cannot have any dimensions                      | `a_unit ^ b`                     |
| comparison (`<`, `>`, `<=`, `>=`, `==`, `!=`) | `a_unit` and `b_unit` must have the same dimensions | N/A (produces `true` or `false`) |

### Examples

> [!NOTE]
> `empty.on` is just an empty model, since we don't reference any model
> parameters.

```bash
# addition, subtraction, modulo
oneil eval empty.on \
  -x "(1000:m) + (1:km)" \
  -x "(1:km) + (1000:m)" \
  -x "(5:min) - (30:s)" \
  -x "(80:s) % (1:min)"
```

```oneil-eval-output
(1000:m) + (1:km) = 2e3 :m
(1:km) + (1000:m) = 2 :km
(5:min) - (30:s) = 4.5 :min
(80:s) % (1:min) = 20 :s
```

```bash
# multiplication, division, exponentiation
oneil eval empty.on \
  -x "(1:m) * (1:s)" \
  -x "(1:m) * (1:m)" \
  -x "(1:m) * 1" \
  -x "(1:m) / (1:s)" \
  -x "(1:m) / 1" \
  -x "(1:m)^2"
```

```oneil-eval-output
(1:m) * (1:s) = 1 :m*s
(1:m) * (1:m) = 1 :m*m
(1:m) * 1 = 1 :m
(1:m) / (1:s) = 1 :m/s
(1:m) / 1 = 1 :m
(1:m)^2 = 1 :m^2
```

```bash
# comparison
oneil eval empty.on \
  -x "(1:kg) < (2000:g)" \
  -x "(1:kg) > (1:g)" \
  -x "(1:kg) <= (1000:g)" \
  -x "(1:kg) >= (900:g)" \
  -x "(1:kg) == (1000:g)" \
  -x "(1:kg) != (1:g)"
```

```oneil-eval-output
(1:kg) < (2000:g) = true
(1:kg) > (1:g) = true
(1:kg) <= (1000:g) = true
(1:kg) >= (900:g) = true
(1:kg) == (1000:g) = true
(1:kg) != (1:g) = true
```

## `strip`

In the case that you would like to treat a measured value as unitless, Oneil
provides the `strip` function. The strip function removes any units from a
value.

```oneil
# adc.on
ADC bit resolution: S_adc = 10 :b
$ ADC step count: n_adc = 2^(strip(S_adc)-1)
```

```bash
oneil eval adc.on
```

```oneil-eval-output
n_adc = 512  # ADC step count
```

The places where this should be used are rare and should be treated cautiously
since `strip` effectively disables unit checking.

In addition, it is important to realize that `strip` strips the unit that is
_currently associated with a value_.

```oneil
# length.on
Length in meters: l_m = 1000 :m
Length in kilometers: l_km = 1 :km
```

```bash
oneil eval length.on \
  -x "strip(l_m)" \
  -x "strip(l_km)"
```

```oneil-eval-output
strip(l_m) = 1e3
strip(l_km) = 1
```

For this reason, when using `strip`, it is recommended that you first cast
the value to the unit that you expect it to be.

```bash
oneil eval length.on \
  -x "strip((l_m :m))" \
  -x "strip((l_km :m))"
```

```oneil-eval-output
strip((l_m :m)) = 1e3
strip((l_km :m)) = 1e3
```

## Non-linear units

On top of linear units, Oneil supports _decibel_ (**dB**) units. You form a
decibel unit by prefixing `dB` directly to a built-in unit name, for example
`dBmW` (decibels relative to one milliwatt) or `dBV`. The bare name `dB`
(with no following unit) is also valid; it behaves as a dimensionless logarithmic
unit.

Support for other non-linear units is on the roadmap.

When any unit is specified with prefix `dB`, Oneil internally converts the
parameter to the corresponding linear value, performs all calculations in linear
terms, and reconverts the value to `dB` for display. This means that equations
that contain parameters with `dB` units should use linear math. For example,
when calculating the signal to noise ratio by hand, you might subtract the noise
(`dB`) from the signal (`dB`), but in Oneil, you divide the signal by the noise:

```oneil
# power.on
Noise power: P_n = -100 :dBmW
Signal power: P_s = -90 :dBmW
$ Signal-to-noise ratio: S_N = P_s/P_n
```

```bash
oneil eval power.on
```

```oneil-eval-output
S_N = 10  # Signal-to-noise ratio
```

## Dimensionless units

There are some units that don't have any dimensions, such as `%` or `ppm` (parts
per million). These units are referred to as _dimensionless units_, and values
with dimensionless units are referred to as _dimensionless values_.

### Unitless equivalence

Dimensionless values can be treated as if they have no unit. The following
demonstrates this with the `%` unit.

> [!NOTE]
> `empty.on` is just an empty model, since we don't reference any model
> parameters.

```bash
# `100%` is treated as equal to `1`
oneil eval empty.on \
  -x "(100:%) == 1" \
```

```oneil-eval-output
(100:%) == 1 = true
```

```bash
# the `1` is equal to `100%`, not `1%`
oneil eval empty.on \
  -x "(100:%) + 1"
```

```oneil-eval-output
(100:%) + 1 = 200 :%
```

### Angular Units

The lack of distinction between dimensionless values and unitless values is
especially important when it comes to units involving _radians_. The
International System of Units treats radians as dimensionless, and Oneil has
opted to follow this convention. Therefore, all angular units (such as
`radians`, `degrees`, and `revolutions`) are specified in radians. Therefore,
when adding a unitless number to an angular value, the unitless number is
treated as if it is specified in `radians`.

```bash
oneil eval empty.on \
  -x "(1:rad) == 1" \
  -x "(360:deg) == 2*pi" \
  -x "(1:rad) + 1" \
  -x "(360:deg) + 2*pi"
```

```oneil-eval-output
(1:rad) == 1 = true
(360:deg) == 2*pi = true
(1:rad) + 1 = 2
(360:deg) + 2*pi = 720 :deg
```

## `Hz` and `rad/s`

There is one place where Oneil's automatic conversions might cause confusion.
That is with the `Hz` unit. In order to solve the problem described
[in this article](https://iopscience.iop.org/article/10.1088/1681-7575/ac0240)
and make `Hz` compatible with `rad/s`, Oneil defines `Hz` as

```text
1 Hz == 1 cycle/s == 2*pi rad/s.
```

Note that both `cycles` and `radians` are both dimensionless values, but
`1 cycle == 2*pi radians`.

```oneil
# freq.on
Frequency: f = 1 :Hz
```

```bash
oneil eval freq.on \
  -x "f" \
  -x "(f :cycle/s)" \
  -x "(f :rad/s)" \
```

```oneil-eval-output
f = 1 :Hz
(f :cycle/s) = 1 :cycle/s
(f :rad/s) = 6.283 :rad/s
```

By default, Oneil treats dimensionless values as if they are in `radians`.
Because of this, anytime you would like dimensionless values to be in `cycles`,
you need to manually convert from `radians` to `cycles` by dividing by `2*pi`.

```oneil
# freq2.on
Frequency: f = 5 :GHz
Speed of light: c = 299792458 :m/s

$ Wavelength: lambda = c/(f/2*pi) :cm
#                          ^^^^^ Need to divide by 2*pi to convert radians to cycles
```

```bash
oneil eval freq2.on
```

```oneil-eval-output
lambda = 0.6075 :cm  # Wavelength
```
