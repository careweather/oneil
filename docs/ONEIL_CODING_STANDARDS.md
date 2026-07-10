# Oneil Coding Standards

## Modeling Principles

### Show Your Work

Do not use magic numbers. Always show your work or your sources. Clarify your
assumptions.

### Subdivide Into Hierarchical Subsystems

Subdivide models into logical hierarchical subsystems. You should typically
align these subsystems with a specific hardware component if it stands by
itself. If a functionality is filled collaboratively by multiple subsystems, it
should be modeled in a top-level system model.

### Only Model What Is Required

Only model what is required to calculate performance metrics. Don't include
superfluous modeling. Think carefully about all of the considerations that
affect the performance metrics.

### Model From the Bottom Up

Specify the design inputs and calculate the performance output, not the other
way around. Independent parameters (those that are assigned a value instead of
an equation) should generally be design parameters that the engineer has more
direct control over.

### One Source of Truth

Do not duplicate parameters. There should be one source of truth for each
physical property or relationship. If this is not possible for some reason, use
comments to make clear that this is a duplicate parameter.

## Parameters

### Parameter Names

Parameter names should use sentence case.

### IDs

Parameter IDs should be as simple as possible. Prefer short subscripts and don't
use multiple subscripts (`v_wmx` instead of `v_wind_max`).

IDs are used to produce typeset equations. The shorter the name the better. For
example, battery voltage should use `V_b` instead of `V_batt`.

In typesetting, imported submodels are given as a superscript. If the battery
voltage appears in the battery submodel, then it should have no subscript at
all, just `V`.

### Notes and Sources

Be very clear in the note that follows the parameter. Provide a description of
how you derived the equation or obtained a value. Provide sources where
relevant, either URLs or journal references. But do not repeat yourself. For
example, if the parameter name is "Flux capacitor power consumption", don't say
in the note "This is the power consumption of the flux capacitor"; instead say,
"taken from the Doc's own Delorean handbook, page 13."

Write notes in LaTeX. If you give a URL in a note, use `\href`. Escape special
LaTeX characters like `%` and `&`.

If multiple parameters would give the same URL as a source, consider including
that source in the introductory note and referencing it in the parameter notes.
For example, if this is an off-the-shelf electronic component, the introductory
note would give the source for the datasheet and the parameter notes could just
say something like, "given on page # of the datasheet."

### Don't Repeat Yourself

For Oneil, name, ID, math, units, and sources/notes all have their own place:

- Don't put units in the name, ID, or note.
- Don't re-state the name in the note.
- Don't re-state the math in the note, unless you derive it in more detail
  there.

## Units

Oneil treats units as built-in types. You don't need to specify units anywhere
else. Do not specify units as a subscript to the ID, as part of the name, or in
the note. Do not convert units manually. Doing so will result in duplicate
conversion errors.

Oneil should handle all units that the user might specify. Always specify units
as cited in the source. For example, if the length of an object is given as 18
inches, use:

```oneil
Length: L = 18 :in
```

not:

```oneil
Length: L = 18*.254 :m
```

## Model Structure

### Structure Around Hardware

It's generally better to structure your submodels around actual hardware, at
least the lowest-level models, because then you can have a model file that's
tied to the specifications and properties of one component.

For example, if you have a `solar.on` file which represents a solar power
system, it could import a `SM500K12L.on`, which represents a specific solar cell
component that can be purchased off the shelf. If a Oneil file refers
specifically to an off-the-shelf component, it is preferable to name the file
after the component model number.

### Constants

If a parameter is a fact that is generally true regardless of the component or
design, include it in a `constants.on` file and import it. For example, the
speed of light should go in `constants.on`.

## Limits

Use limits as a sanity check for real world values. For example, if calculating an
efficiency, only values in the range `(0, 1)` are valid.

## Tests

Use tests to model relationships between parameters. For example, let's say you
are designing a smartphone. You specify the battery capacity, `C_b`, and the
model calculates the corresponding battery volume, `V_b`. You could use a
relational test to make sure the battery volume is not larger than the total
smartphone volume, `V`:

```oneil
test : V_b < V
```

## Interval Arithmetic

Oneil supports built-in interval arithmetic. Never make separate minimum and
maximum parameters when you can make one parameter and specify the minimum and
maximum edge cases.
