# Oneil Coding Standards

## Modeling Principles

### Show Your Work

Do not use magic numbers. Always show your work or your sources. Clarify your
assumptions.

### Subdivide Into Hierarchical Subsystems

Subdivide models into logical hierarchical subsystems. You should typically
align these subsystems with a specific hardware component if it stands by
itself. If a functionality is filled collaboratively by multiple subsystems, it
should be modeled in a system model.

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
physical property or relationship.

## Parameters

### Parameter Names

Parameter names should use sentence case.

### IDs

Parameter IDs should be as simple as possible. Prefer short subscripts and don't
use multiple subscripts (`v_wmx` instead of `v_wind_max`).

IDs are used to produce typeset equations. The shorter the name the better. For
example, battery voltage should use `V_b` instead of `V_batt`.

In typesetting, imported submodels are given as `model` beneath the parameter.
If the battery voltage appears in the battery submodel, then it should have no
subscript at all, just `V`.

### Notes and Sources

Be very clear in the note that follows the parameter. Provide a description of
how you derived the equation or obtained a value. Provide sources where
relevant, either URLs or journal references. But do not repeat yourself. For
example, if the parameter name is "Flux capacitor power consumption", don't say
in the note "This is the power consumption of the flux capacitor"; instead say,
"taken from the Doc's own Delorean handbook, page 13."

Write notes in markdown. Use markdown for headings, emphasis, bold, and tables.
Use LaTeX only for inline equations (e.g., `$E = mc^2$`). For URLs, use
standard markdown links (`[text](url)`).

If multiple parameters would give the same URL as a source, consider including
that source in the introductory note and referencing it in the parameter notes.
For example, if this is an off-the-shelf electronic component, the introductory
note would give the source for the datasheet and the parameter notes could just
say something like, "given on page # of the datasheet." Alternatively, use a
citation.

### Citations

Oneil uses Pandoc-style citations. Citations can reference a specific page.
The following bracketed styles are supported:

| Syntax             | Use case                         |
|--------------------|----------------------------------|
| `[@key]`           | Standard citation                |
| `[+@key]`          | Long form - all authors listed   |
| `[-@key]`          | Suppress author - year only      |
| `[!@key]`          | Author name only, no parentheses |
| `[@key, p.42]`     | Citation with page reference     |
| `[@key, pp.42-45]` | Citation with page range         |
| `[@key, 42]`       | Bare integer as page number      |
| `[@k1; @k2]`       | Multiple citations               |

Unbracketed citations are rendered inline:

| Syntax  | Use case                             |
|---------|--------------------------------------|
| `@key`  | Narrative: "Author (year)"           |
| `+@key` | Narrative long: "All Authors (year)" |
| `-@key` | Year only                            |
| `!@key` | Author name only                     |

### Don't Repeat Yourself

For Oneil, name, ID, math, units, and sources/notes all have their own place:

- Don't put units in the name, ID, or note.
- Don't re-state the name in the note.
- Don't re-state the math in the note, unless you derive it in more detail
  there.

Instead of restating equations or values, use interpolation placeholders in
notes. The rendered view will display these as the live LaTeX equation or
computed value:

- `{{R:equation}}` - renders the equation for parameter `R`
- `{{R:value}}` - renders the computed value for parameter `R`

An exception to the "don't repeat yourself" rule is **summary tables**, where
interpolation is commonly used to collect results from multiple parameters in
one place. Interpolation should generally not be used in the note for the
equation's own parameter.

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
system, it could import a `solar_cell.on` which models the essential aspects of
a solar cell (efficiency, area, degradation, etc.). A specific off-the-shelf
component like the SM500K12L would then be represented as a design file
(`SM500K12L.one`) that assigns values to `solar_cell.on`'s parameters. This
separation is especially useful when a component might be swapped out later.

If an Oneil file refers specifically to an off-the-shelf component and is not
expected to be interchangeable, it is acceptable to name the file after the
component model number directly.

### Constants

If a parameter is a fact that is generally true regardless of the component or
design, include it in a `constants.on` file and import it. For example, the
speed of light should go in `constants.on`. `constants.on` should always be
imported as a reference, not as a submodel.

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

As a best practice, every performance parameter should have at least one test
referencing it.

## Interval Arithmetic

Oneil supports built-in interval arithmetic. Never make separate minimum and
maximum parameters when you can make one parameter and specify the minimum and
maximum edge cases.
