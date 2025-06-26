# The Oneil Design Language

Oneil is a design specification language for rapid, comprehensive system modeling.

Traditional approaches to system engineering are too cumbersome for non-system engineers who don't have all day. Oneil makes it easy for everyone to contribute to the central source of system knowledge. With Oneil everyone can think like a system engineer and understand how their design impacts the whole.

Oneil enables specification of a system "model", which is a collection of "parameters", or attributes of the system. The model can be used to evaluate any corresponding design (which is a collection of value assignments for the parameters of the model).

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
* (coming soon) Automatic documentation:
  * Model derivations.
  * Design test reports.
  * Parametric figures.
* (coming soon) Side-by-side design comparisons.
* (future) Caching and automatic change reports.
* (future) VSCode highlighting and linting.

## Requirements

Oneil has only been tested on Linux. Instructions for Oneil assume you are on Linux.

## Python Quickstart

Clone Oneil and install it.

``` { .sh }
git clone git@github.com/careweather/oneil.git
oneil/install.sh
```

<!-- Add this back in when repo is public and test.>
<!-- Install Oneil using pip (add @\<version-number\> if you need a specific version):

``` { .sh }
pip install git+ssh://git@github.com/careweather/oneil.git
``` -->

Once installed, Oneil can be run from the command line in the directory where your design is. This will open the oneil command line interface, which needs a model before it can accept commands:

``` { .sh }
$ cd path/to/directory/containing/your/model
$ oneil your-model.on
Oneil <Version>
----------------------------------------
Loading model your-model.on
----------------------------------------
Model: your-model
Design: default
Parameters: <count>
Tests: <count> (PASS/FAIL)
----------------------------------------
(your-model) >>>
```

To see all the results of the model:

``` { Oneil CLI }
>>> all
<param 1>: <min>|<max> <unit>
<param 2>: <min>|<max> <unit>
...
<param n>: <min>|<max> <unit>
```
### Development

If you are developing Oneil, you will want to install Oneil in "editable" mode. To do this, use the `-e` flag.

```sh
oneil/install.sh -e
```

This will allow you to modify Oneil's python code and run it with `oneil` without having to re-run `oneil/install.sh` for every change.

## Rust Quickstart

To run the Rust version of Oneil locally:

1. [Install Rust and Cargo](https://www.rust-lang.org/tools/install) if you haven't already.

2. Clone the repository and navigate to the Rust project directory:
```sh
git clone git@github.com/careweather/oneil.git
cd oneil/rust
```

<!-- TODO: not ready yet
3. Build and run the project:
```sh
cargo run -- path/to/your/model.on
```
-->

<!-- TODO: when ready, add instructions for installation -->

### Development

For development, you can use these Cargo commands:

- Run tests:
  ```sh
  cargo test
  ```

- Check for compilation errors without producing an executable:
  ```sh
  cargo check
  ```

- Format code:
  ```sh
  cargo fmt
  ```

- Run linter:
  ```sh
  cargo clippy
  ```

You can also run the following developer commands built into Oneil:
- Print the AST that is constructed from an Oneil file:
  ```sh
  cargo run dev print-ast path/to/model.on
  ```

In addition, you will want to install the
[rust-analyzer](https://marketplace.cursorapi.com/items?itemName=rust-lang.rust-analyzer)
VS Code extension in order to help you develop in Rust.

### Model Syntax Updates for Rust Version

The Rust version of Oneil has two syntax updates from the Python version.

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

2. Discrete parameter string values are now enclosed in quotes. For example:
   
   ```oneil
   # Before
   X[foo, bar, baz]: x = foo
   Y: y = { 1 if x == foo
          { 2 if x == bar
          { 3 if x == baz

   # After
   X["foo", "bar", "baz"]: x = "foo"
   Y: y = { 1 if x == "foo"
          { 2 if x == "bar"
          { 3 if x == "baz"
   ```

   This change does not have an automatic fix (yet, at least), and must be done
   by hand.

## Manual Install

Alternatively, if you've cloned Oneil, you can install it using pip. You will need the following packages to run Oneil:

* Numpy
* Beautiful Table
* Pytexit

Install using the following commands:

``` { .sh }
pip install path/to/oneil
pip install numpy beautifultable pytexit 
```

### Toolchain

Oneil supports syntax highlighting in vim. Oneil is already designed for readability, but syntax highlighting makes it even better.

If you installed using `install.sh`, then vim should be installed and configured.

Otherwise, create a `~/.vim` directory with subdirectories `syntax` and `ftdetect` if they don't exist yet. From this directory create soft links to the files in the `vim` directory of the Oneil repository.

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

## Parameters

The oneil language supports definition of a collection of "parameters", with *independent* parameters that have specified values and *dependent* parameters that are functions of other parameters. The syntax for defining a parameter in Oneil is:

``` { .on }
Preamble: Body
```

Or expressed in more detail:

``` { .on }
Name (Limits): ID = Assignment :Units
```

Detailed syntax rules for each of these parts is described in the following sections.

## Preamble Syntax

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
Space domain [EarthOrbit, interplanetary, interstellar]: ...
Dimensions [1, 2, 3]: ...
```

You can mark a parameter as a "performance" parameter by prepending it with a `$`. Performance parameters are included in summaries of the model.

``` { .on }
$ Artificial gravity: ...
...
```

## Body Syntax

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

### Assignment

The parameter assignment can either be a value (independent) or an equation (dependent).

Value assignments can specify a single value or a minimum and maximum value separated by a pipe. These values use numbers, math constants, or in the case of discrete values, they can also use a set of [word characters](https://stackoverflow.com/questions/2998519/net-regex-what-is-the-word-character-w).

``` { .on }
Window count: n_w = 20
Communications amplifier efficiency (0, 1): eta_c = 0.5|0.7
Space domain [earth_orbital, interplanetary, interstellar]: D_s = interstellar
```

Equation assignments define a parameter as a function of other parameters using parameter IDs (e.g. `"m*x + b"` where `m`, `x`, and `b` are parameter IDs).

``` { .on }
Cylinder radius: r = D/2 : ...
Artificial gravity: g_a = r*omega**2 : ...
```

Use a pointer (`=>`) if the equation for a parameter simply sets it equal to another parameter:

``` { .on }
Radar amplifier efficiency (0, 1): eta_r => eta_c
```

Alternate equations for the minimum and maximum case can be given, separated by a pipe.

``` { .on }
Power consumption: P_c = eta_c*P_q | eta_c*P_a
```

Again, if one of these equations is just another parameter, a pointer must be used:

``` { .on }
Population: P => N_c | N_c + N_r
```

Note that pointers cannot be used with discrete variables.

For more details on valid equations, see [here](#extrema-math).

### Units

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
Artificial gravity: g_a = r*omega**2 :m/s^2
Temperature: T = temperature(D) :K
```

You can review supported units using the [CLI units command](#unit-help). If a unit isn't supported, you can specify it in terms of base units: `kg`, `m`, `s`, `K`, `A`, `b`, `$`.

Oneil currently supports `dB` as a nonlinear display unit. When any unit is specified with prefix `dB`, Oneil internally converts the parameter to the corresponding linear value, performs all calculations in linear terms, and reconverts the value to dB for display. This means that equations that contain parameters with dB units should use linear math. For example, when calculating the signal to noise ratio by hand, you might subtract the noise (dB) from the signal (dB), but in oneil, you divide the signal by the noise:

``` { .on }
Noise power: P_n = -100 :dBmW
Signal power: P_s = -90 :dBmW
Signal-to-noise ratio: S_N = P_s/P_n
```

While [limits](#preamble-syntax) are typically specified in the parameter's units, limits only support linear values. Parameters with dB units should typically not specify a limit (other than the default 0-inf) since negative linear values would lead to imaginary dB values.

> [!IMPORTANT]
> Oneil handles nearly all unit conversion in the background, but there is a [major exception with frequencies (Hz) and angular frequencies (rad/s)](#something-funny-is-happening-with-angular-frequencies-and-frequencies).

## Extrema Math

In the backend, Oneil uses parametric extrema math to calculate the extremes of the range of possibilities for a given calculation, as defined in Chapter 3 of [Concepts for Rapid-refresh, Global Ocean Surface Wind Measurement Evaluated Using Full-system Parametric Extrema Modeling](https://scholarsarchive.byu.edu/cgi/viewcontent.cgi?article=10166&context=etd). Expressions are limited to the following operators and functions: `+`, `-`, `\*`, `/`, `^`, `==,` `!=`, `<=`, `>=`, `%`, `()`, `min()`, `max()`, `sin()`, `cos()`, `tan()`, `asin()`, `acos()`, `atan()`, `sqrt()`, `ln()`, `log()`, `log10()`, `floor()`, `ceiling()`, `extent()`, `range()`, `abs()`, `sign()`, `mid()`, `strip()` (removes units in calculation), and `mnmx()` (an extreme function which gets the extremes of the inputs).

The `min()` and `max()` functions can be used to compare parameters or it can be used on a single Parameter to access the minimum or maximum value of the Parameter's value range.

Extrema math yields substantially different results for subtraction and division. If the extreme cases are incompatible with a given parameter, you can specify standard math using the `--` and `//` operators.

### Piecewise Equations

Piecewise equations can be used for parameter assignments.

``` { .on }
Orbital gravity: g_o = {G*m_E/h**2 if D_s == 'earth_orbital' :km/s
                       {G*m_S/h**2 if D_s == 'interplanetary'
                       {G*m_G/h**2 if D_s == 'interstellar'
```

(`m_E`, `m_S`, and `m_G` are the masses of the Earth, Sun, and galactic center)

The conditions for piecewise equations are pythonic, so pythonic comparison operators are used and discrete values that use strings should be given in single quotes. Conditions are evaluated in order, and the first equation corresponding to a true condition is calculated to obtain the value for the parameter.

### Breakout Functions

For functions not supported by the above equation formats, you can define a python function and link it.

The breakout functions are stored in a separate python file, which must be imported in the Oneil file.

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

Multiplying a python function by another parameter is not currently supported. Specify the factors as separate parameters and use another parameter to multiply them.

## Submodels

A model can use parameters from a submodel:

``` { .on }
use submodel as s
```

The word after use gives the submodel which should match the name of an oneil file with ".on" as the file extension. The symbol after as is used with a "." after parameters from that model to show where they come from. For example:

``` { . on }
use cylinder as c

Length of day: t_day = omega.c/2*pi :day
```

To use a parameter, it's submodel has to be specified directly. For example, if cylinder uses submodel life_support, specifying cylinder does not give access to life_support. Life_support and any of it's submodels must also be specified if parameters from them are needed. Submodel symbols should be as short as possible for readability.

``` { .on }
use cylinder as c
from cylinder use life_support as ls
from cylinder.life_support use oxygen_tank as o
```

## Designs

A design consists of the values assigned to independent parameters in a model. Oneil model files include a default design, but Oneil makes it easy to overwrite that default with alternative designs. Design files use the same syntax of model files, but only require the body instead of the whole line (no preamble required). Designs let you change a subset of the independent parameters from the default design. For example,

``` { .on }
m = 1e6 :kg
D = 0.5 :km
omega = 1 :deg/min
case = clockwise
L => L.d
```

To use a design, see [the command line interface `design` command](#design). A design parameter overwrites the value of the model parameter while keeping the original metadata. If you want your design to alter a submodel parameter, you'll need to make sure the corresponding model uses that submodel.

## Tests

Models can also specify tests to verify model reasonability and accuracy. Tests use math expressions with comparison operators (`==`, `>`, `<`, `>=`, `<=`, `!=`) to return True or False. Tests can't include unit specifications, so any values with units must be specified separately and used in the test equation. This turns out to be a useful limitation for preventing magic numbers.

``` { .on }
Earth gravity: g_E = 9.81 :m/s^2

test : g_E*0.9 <= g_a <= g_E*1.1

    The artificial gravity should be within 10% of Earth's gravity.
```

Say you have a submodel that's only valid in certain larger contexts. You can specify a test in that submodel that requires an input from a parent model to pass:

``` { .on }
test {delta_g} : g_E - delta_g <= g_a <= g_E + delta_g
```

In this case, the test is specifying that the submodel needs to be given a value named `delta_g` for verification. In the parent model, these parameters can be passed to the submodel in the `use` statement:

``` { .on }
use cylinder(delta_g=delta_ghuman) as c
```

## Notes and Comments

Oneil defines "notes" and "comments" differently. Notes are comments that you want to show up in reports explaining and justifying the model or design. Comments are "notes to self" that don't show up in any reports. When the model is exported to a report, notes are included, but comments are not.

Oneil recognizes notes as any line that is not blank and begins with whitespace (four spaces or a tab, for example). When a note is found, Oneil will tie it to the most recently-defined parameter or test (above the note in the file). If none are found, Oneil will tie the note to the model itself. On export, notes are processed as LaTeX.

Oneil recognizes any line starting with `#` as a comment.

In the following example, "O'neill cylinder for..." is a note tied to the model while `cylinder radius` has no note and `standard Earth gravity` has "From \href..." as its note. "#TODO..." is ignored as a comment.

``` { .on }
    O'neill cylinder for supporting long-term human habitation in deep space.

#TODO: refactor this as a function of the diameter
Cylinder diameter: d = 0.5 :km

Standard Earth gravity: g_E = 9.81 m/s^2

    From \href{https://en.wikipedia.org/wiki/Gravity_of_Earth}{wikipedia}.
```

## Using the Command line interface

See the [quickstart](#quickstart) for how to start the command line interface (CLI) and load a model. Once a model has been loaded in the CLI, the model can be explored and evaluated using an expression or the CLI functions described in the following sections.

### Queries and Expressions

The CLI can be used for to query any parameter in the model, evaluate an expression consisting of numbers and parameters, and convert a parameter other units. For example:

``` { Oneil CLI }
(cylinder) >>> r
250 km
(cylinder) >>>  (r/10)*omega**2
2.795 g
(cylinder) >>> (r/30)*omega**2 < R_E
True
(cylinder) >>> r:m
250000 m
```

The following CLI commands are reserved. If you use one of them as an ID in your model, you won't be able to query that ID, because Oneil will prefer the command.

### Tree

Print a tree of the parameters:

``` { .on }
>>> tree [parameter 1] [parameter 2] ... [parameter n]
<parameter 1>: <result>
<equation>
    <equation arg>: <result>
    ...
```

For example:

``` { .on }
(cylinder) >>> tree g_a
g_a: 27.95 g
=r*omega**2
    omega: 60.0 °/s
    r: 250.0 m
    =D/2
        D: 500.0 m
```

### All

Print all parameters and their results, for example:

``` { Oneil CLI }
(cylinder) >>> all
m: 1000000.0 kg -- Mass
D: 500.0 m -- Diameter
r: 250.0 m -- Radius
omega: 60.0 °/s -- Rotation rate
g_a: 27.95 g -- Artificial gravity
```

### Dependents

Print all parameters dependent on the given parameter, for example:

``` { Oneil CLI }
(cylinder) >>> dependents omega
['g_a', 't_day']
```

### Summarize

Summarize the design:

``` { Oneil CLI }
(cylinder) >>> summarize
--------------------------------------------------------------------------------
Model: cylinder
Design: default
Parameters: 5 (4 independent, 1 dependent, 0 constants)
Tests: 1 (PASS/FAIL)
--------------------------------------------------------------------------------
g_a: 27.95 g
```

### Test

Run tests on the model and any added designs:

``` { Oneil CLI }
>>> test
Test (cylinder): g_E*0.9 <= g_a <= g_E*1.1
    Result: fail
    g_E: 1.0 g
    g_a: 27.95 g
    g_E: 1.0 g
```

### Export to a report (not maintained)

> [!CAUTION]
> This feature has not been maintained, but will be resored soon.

Export a model (if no args) or a list of parameters (if args) to a typeset pdf:

``` { Oneil CLI }
>>> export <param 1> <param 2> ... <param n>
```

This saves the pdf as `export.pdf` if you have [installed LaTeX locally](https://www.latex-project.org/get/).

If there are issues with the PDF, you can review the `export.tex` file. In VSCode the [LaTeX Workshop](https://marketplace.visualstudio.com/items?itemName=James-Yu.latex-workshop) extension is helpful. Also, VS Code doesn't handle LaTeX errors well. So if you run into lots of issues, use a more established TeX IDE, like [TexMaker](https://www.xm1math.net/texmaker/).

The current implementation uses biblatex for references.

### Design

Write a design onto the model:

``` { Oneil CLI }
(your-model) >>> design design-name [variation-name]
----------------------------------------
Model: your-model
Design: variation-of-variation
Parameters: <count>
Tests: <count> (PASS/FAIL)
----------------------------------------
(variation-name@design-name@your-model) >>>
```

The compiler writes the first design to the model, overwriting any overlapping model defaults. If there are further designs, it writes the each subsequent design, overwriting any overlapping defaults or values from the preceding ones. See below for more details on designs.

### Load

Load a new model:

``` { Oneil CLI }
>>> load model-name
```

### Unit Help

See all units supported by Oneil:

``` { Oneil CLI }
>>> units
```

### Quit

Exit the CLI:

``` { Oneil CLI }
>>> quit
```

or

``` { .on }
>>> quit()
```

### Error Handling in the CLI

Ideally, if there's a problem with your Oneil code or Python extensions, the Oneil compiler will catch it and tell you. In that case, you can try debugging by prepending a parameter with `*`, but debugging is limited and requires some understanding of how Oneil handles parameters in the background.

If you instead see an error missed by Oneil and raised by Python, it's likely an error with the Oneil compiler which is still in development. If that happens, please post the issue in GitHub. The compiler doesn't yet support step by step debugging, so you'll have to use [VSCode for this](#oneil-has-a-bug) for now.

## Known Issues and Limitations

* Scientific notation is supported in value assignments, but not limits. It should be supported in expressions, but this hasn't been tested.
* The Vim syntax highlighter gets *really* slow if you try to paste large amounts of LaTeX in. For now, make sure to paste large blocks of LaTeX using a different text editor or temporarily remove the ".on" file extension while you do.
* The Vim syntax highlighter breaks for the rest of the file after a LaTeX syntax error in a note. As a result, the rest of the file will be highlighted as a note.

And many more. These will be ported to GitHub issues for planning and visibility in coming months. If you find an issue that isn't listed in GitHub, please post it.

## Troubleshooting

### Something funny is happening with angular frequencies and frequencies

The funny thing about Hz and rad/s is that `1 Hz != 1 rad/s` even though `1 Hz = 1/s` and `1 rad/s = 1/s`. You can [thank the International System of Units for this madness](https://iopscience.iop.org/article/10.1088/1681-7575/ac0240). To escape this, Oneil doesn't recognize the SI definition of Hz. If you specify Hz as a unit, Oneil will internally convert it to rad/s by multiplying by 2 pi. If you want to use a frequency in an equation that expects Hz, you need to make sure the equation converts your frequency (rad/s) to Hz. For example, instead of `c=lambda*f` for the speed of light, you would use `c=lambda*f/(2*pi)`.

> As a side note, some people have suggested that this problem is solved if you use `cycles` as a base unit and let `Hz = 1 cycle/s`, but this quickly becomes messy as cycles will get propagated throughout your model where you don't want it. It's much cleaner to convert rad/s to Hz in equations that expect it.

### Oneil has a bug

You can report bugs using the issues section on Github. If you want to try and fix a bug yourself, here are some things that will help:

To edit Oneil, you'll need to clone it and install it with the editable flag (-e), which ensures the pip install tracks your local changes.

``` { .sh }
git clone git@github.com/careweather/oneil.git

pip install -e oneil
```

To debug Oneil, you can call the CLI from a python file:

``` { .py }
import oneil

oneil.main()
```

You can also give the file off the bat:

``` { .py }
oneil.main(["", "your-model.on"])
```

### TexMaker works, but VS Code doesn't

Try closing all VS Code files and closing VS Code to clear its mystery cache.

## Contributing

If you're interested in contributing, email [us](mailto:oneil@careweather.com).

## About

The initial methodology that inspired Oneil was proposed in Chapter 3 of [Concepts for Rapid-refresh, Global Ocean Surface Wind Measurement Evaluated Using Full-system Parametric Extrema Modeling](https://scholarsarchive.byu.edu/cgi/viewcontent.cgi?article=10166&context=etd), by M. Patrick Walton. For that work, the methodology was painfully implemented in a Google sheet. The conclusion provided ideas and inspiration for early versions of Oneil.

Oneil was developed at Care Weather Technologies, Inc. to support design of the Veery scatterometer. Veery is designed to perform as well as $100M heritage scatterometers at orders of magnitude less cost. This dramatic improvement is facilitated in part by Oneil's streamlined systems engineering capabilities.

Oneil is named after American physicist and space activist [Gerard K. O'Neill](https://en.wikipedia.org/wiki/Gerard_K._O%27Neill) who proposed the gargantuan space settlements known as [O'Neill cylinders](https://en.wikipedia.org/wiki/O%27Neill_cylinder). We built Oneil to meet our own needs, but we hope it stitches together the many domains required to make O'Neill cylinders and move humanity up the [Kardashev scale](https://en.wikipedia.org/wiki/Kardashev_scale).
