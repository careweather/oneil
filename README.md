# The Oneil Design Language

Oneil is a design specification language for rapid, comprehensive system modeling.

Traditional approaches to systems engineering are cumbersome and inaccessible to other engineers. Oneil enables everyone to think like a system engineer by contributing to one central source of system knowledge and understanding how their design decisions impact the whole.

Oneil enables specification of a system `Model`, which is a collection of `Parameters`. The Model can be used to evaluate any corresponding design (collection of value assignments for the parameters of the model).

Oneil is pre-release, under construction. See [known issues](#known-issues-and-limitations) and [troubleshooting](#troubleshooting) for context on its limitations.

## Quickstart

Clone oneil and install it using pip.

``` { .sh }
git clone git@github.com/careweather/oneil.git

pip install -e oneil
```
You will need the following packages to run Oneil: 
* Numpy
* Beautiful Table
* Py2tex
* Pytexit

These can be installed using the following command: 

```
pip install numpy beautifultable py2tex pytexit 
```

<!-- Install Oneil using pip (add @\<version-number\> if you need a specific version):

``` { .sh }
pip install git+ssh://git@github.com/careweather/oneil.git
``` -->

Once installed, Oneil can be run from the command line in the directory where your design is. This will open the oneil post-interpreter, which needs a model before it can accept commands:

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
Tests: <count>
----------------------------------------
(your-model) >>>
```

To see all the results of the model:

``` { Oneil interpreter }
>>> all
<param 1>: <min>|<max> <unit>
<param 2>: <min>|<max> <unit>
...
<param n>: <min>|<max> <unit>
```

## Toolchain

Oneil supports syntax highlighting in vim. While Oneil is already designed for readability, the difference with syntax highlighting is night and day. To set this up on Linux, create a `~/.vim` directory with subdirectories `syntax` and `ftdetect` if they doesn't exist yet. From this directory create soft links to the files in the `vim` directory of the Oneil repository.

Oneil supports syntax highlighting in vim. While Oneil is already designed for readability, the difference with syntax highlighting is night and day. To set this up on Linux, simlink the files in oneil/vim/ftdetect and oneil/vim/syntax to the corresponding folders in your `~/.vim` directory.

``` { .sh }
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

The oneil language supports definition of a collection of `Parameters`, starting with *independent* parameters that have specified values, and progressing through *dependent* parameters that are specified functions of other parameters. The syntax for defining a parameter in Oneil is:

``` { .on }
Preamble: ID = Assignment :Units
```

Detailed syntax rules for each of these parts is described in the following sections.

### Preamble Syntax

At a minimum, the preamble must contain the name of the parameter. The name must be plain text with no special characters except apostrophes. For example:

``` { .on }
Mass: ...
Angular position: ...
Boltzmann's constant: ...
```

The preamble may also specify the allowable range of the parameter. Oneil checks all parameters to ensure their assigned or calculated values are within this range. Ranges can be specified using ints, floats, or math constants as follows:

``` { .on }
Mass (0, 10e8): ...
Angular position (-pi/2, pi/2): ...
```

Range values are given in base units, *not specified units*. Use a [test](#tests) to check against a specified unit. If no range is specified, Oneil assumes the allowable range is is 0 to infinity (non-negative numbers). Ranges are intended to capture the maximum range of physical possibility. They should be as broad as possible. To check that values are within a reasonable range, use a test (see below).

If the allowable values of the parameter are discrete, they must be specified using brackets. This is used for specifying different modes of operation. These can be word characters or numbers:

``` { .on }
Space domain [EarthOrbit, interplanetary, interstellar]: ...
```

You can mark a parameter as a "performance" parameter by prepending it with a `$`. Performance parameters are included in summaries of the model.

``` { .on }
$ Artificial gravity: ...
...
```

### ID

The ID follows the first colon and comes before the equals sign. It is the variable key used in the model namespace. Within a model, IDs must be unique.

``` { .on }
Cylinder diameter: D = ...
Rotation rate: omega = ...
Boltzmann's constant: C_b = ...
Crew count: N_c = ...
Resident count: N_r = ...
Orbital altitude: h = ...
```

Use short IDs where possible. They'll make it easier to read equations when you export to PDF.

### Assignment

The parameter assignment can either be a value (independent) or an equation (dependent).

Value assignments can specify a single value or a minimum and maximum value separated by a pipe. These values use numbers, math constants, or in the case of discrete options, they can also use a set of word characters.

``` { .on }
Window count: n_w = 20
Communications amplifier efficiency (0, 1): eta_c = 0.5|0.7
Space domain [EarthOrbit, interplanetary, interstellar]: D = interstellar
```

Parameters can also be set to equal another parameters.

``` { .on }
Radar amplifier efficiency (0, 1): eta_r = eta_c
```

Equation assignments define a parameter as a function of other parameters. This is typically done using a python expression with other parameter IDs as variables (e.g. `"m*x + b"` where `m`, `x`, and `b` are parameter IDs).

``` { .on }
Cylinder radius: r = D/2 : ...
Artificial gravity: g_a = r*omega**2 : ...
```

Alternate equations for the minimum and maximum case can be given, separated by a pipe.

``` { .on }
Population: P = N_c | N_c + N_r
```

For more details on valid equations, see [here](#extrema-math).

### Units

Units are specified after a second colon using their [SI symbol](https://en.wikipedia.org/wiki/International_System_of_Units#Units_and_prefixes) with the "^" operator for exponents and a "/" preceeding *each* unit in the denominator. Units must be specified if the parameter has units, but can be left off for unitless parameters.

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

Most [SI units](https://en.wikipedia.org/wiki/International_System_of_Units) are supported. Reference oneil/units/\_\_init\_\_.py for supported units. If a unit isn't supported, you can specify it in terms of base units: `kg`, `m`, `s`, `K`, `A`, `b`, `$`.

## Extrema Math

Oneil uses parametric extrema math as defined in Chapter 3 of [Concepts for Rapid-refresh, Global Ocean Surface Wind Measurement Evaluated Using Full-system Parametric Extrema Modeling](https://scholarsarchive.byu.edu/cgi/viewcontent.cgi?article=10166&context=etd). Expressions are limited to the following operators and functions: `+`, `-`, `\*`, `/`, `\*\*`, `<`, `>`, `==,` `!=`, `<=`, `>=`, `min()`, `max()`, `sin()`, `cos()`, `tan()`, `asin()`, `acos()`, `atan()`, `sqrt()`, `log()`, `log10`, and `mnmx()` (an extreme function which gets the extremes of the inputs). `|` and `&` are also available for parameters with boolean values.

The `min` and `max` functions can be used on a single Parameter to access the minimum or maximum value of the Parameter's value range.

Extrema math yields substantially different results for subtraction and division. You can specify standard math for these using the `--` and `//` operators.

### Piecewise Equations

Piecewise equations can be used for parameter assignments.

``` { .on }
Orbital gravity: g_o = {G*m_E/h**2 if D == 'EarthOrbit' :km/s
                       {G*m_S/h**2 if D == 'interplanetary'
                       {G*m_G/h**2 if D == 'interstellar'
```

(m_E, m_S, and m_G are the masses of the Earth, Sun, and galactic center)

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

In the Oneil file, give only the python function on the right hand of the equation, including other parameters as inputs:

``` { .on }
Temperature: T = temperature(D) :K
```

## Submodels

A model can use parameters from a submodel by specifying the submodel as

``` { .on }
use submodel as s
```

The word after use gives the submodel which should match the name of an oneil file with ".on" as a file extension. The symbol after as is used with a "." after parameters from that model to show where they come from. For example:

``` { . on }
use cylinder as c

Length of day: t_day = omega.c/2*pi :day
```

To use a parameter, it's submodel has to be specified directly. For example, if cylinder uses submodel life_support, specifying cylinder does not give access to life_support. Life_support and any of it's submodels must also be specified if parameters from them are needed.

``` { .on }
use cylinder as c
from cylinder use life_support as l
from cylinder.life_support use oxygen_tank as o
```

Short import as symbols are ideal, because they make complex equations more readable.

## Designs

While models specify values for independent parameters that make up the default design, you may want to specify multiple other variations of the design. Design files use the same syntax of model files, but they typically do not require full parameter specifications, instead they simply define the values of a subset of the independent parameters that deviate from the default design. For example,

``` { .on }
m = 1e6 :kg
D = 0.5 :km
omega = 1 :deg/min
```

These can be used to overwrite the default values on the model and evaluate the variation's performance. See [the interpreter's `design` command](#design). The only difference between a value as above and a full parameter is that the value doesn't fully overwrite the metadata of the parameter in the original model.

## Tests

Models and designs can also specify tests to verify that the parameters of a model properly calculate what they are intended to. Tests are python expressions that use comparison operators to return True or False. Tests can't include unit specifications, so any values with units must be specified separately.

``` { .on }
Earth gravity: g_E = 9.81 :m/s^2

test : g_E*0.9 <= g_a <= g_E*1.1

    The artificial gravity should be within 10% of Earth's gravity.
```

If a submodel test needs a parameter from a model that includes it, this can be specified by giving an ID in brackets before the colon:

``` { .on }
test {delta_g} : g_E - delta_g <= g_a <= g_E + delta_g
```

These parameters can be passed to the submodel in the `use` statement:

``` { .on }
use cylinder(delta_g=delta_ghuman) as c
```

Designs and tests can be combined to create a verification design, a design for a system with known performance and tests that check whether the model accurately predicts its known performance.

## Notes and Comments

Oneil defines "notes" and "comments" differently. Notes describe the model and justify the choice of values or provide the derivation of an equation. Comments are "notes to self" contained in the specification file. When the model is exported, notes are included, but comments are not.

Oneil recognizes notes as any line that is not blank and begins with whitespace. When a note is found, Oneil will tie it to the most recently-defined parameter or test (immediately above the commend in the file). If none are found, Oneil will tie the note to the model itself. On export, notes are processed as LaTeX.

Oneil recognizes any line starting with `#` as a comment.

In the following example, "O'neill cylinder for..." is a note tied to the model while `cylinder radius` has no note and `standard Earth gravity` has "From \href..." as its note. "#TODO..." is ignored as a comment.

``` { .on }
    O'neill cylinder for supporting long-term human habitation in deep space.

#TODO: refactor this as a function of the diameter
Cylinder diameter: d = 0.5 :km

Standard Earth gravity: g_E = 9.81 m/s^2

    From \href{https://en.wikipedia.org/wiki/Gravity_of_Earth}{wikipedia}.
```

## Breakpoints and Debugging

Debugging capabilities are limited and little tested, but you should be able to get some debugging if you prepend a line with `*` to add a breakpoint on that line.

## Using the Interpreter

See the [quickstart](#quickstart) for how to start the interpreter and load a model. Once a model has been loaded in the interpreter, the model can be explored and evaluated using any expression or the interpreter functions described in the following sections.

### Queries and Expressions

The interpreter can be used to query any parameter in the model or evaluate an expression consisting of numbers and parameters. For example:

``` { Oneil interpreter }
(cylinder) >>> r
250 km
(cylinder) >>>  (r/10)*omega**2
2.795 g
(cylinder) >>> (r/30)*omega**2 < R_E
True
```

The following interpreter commands are reserved. If you use one of them as an ID in your model, you won't be able to query that ID, because Oneil will prefer the command.

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

``` { Oneil interpreter }
(cylinder) >>> all
m: 1000000.0 kg -- Mass
D: 500.0 m -- Diameter
r: 250.0 m -- Radius
omega: 60.0 °/s -- Rotation rate
g_a: 27.95 g -- Artificial gravity
```

### Dependents

Print all parameters dependent on the given parameter, for example:

``` { Oneil interpreter }
(cylinder) >>> dependents omega
['g_a', 't_day']
```

### Summarize

Summarize the design:

``` { Oneil interpreter }
(cylinder) >>> summarize
--------------------------------------------------------------------------------
Model: cylinder
Design: default
Parameters: 5 (4 independent, 1 dependent, 0 constants)
Tests: 1
--------------------------------------------------------------------------------
g_a: 27.95 g
```

### Test

Run tests on the model and any added designs:

``` { Oneil interpreter }
>>> test
Test (cylinder): g_E*0.9 <= g_a <= g_E*1.1
    Result: fail
    g_E: 1.0 g
    g_a: 27.95 g
    g_E: 1.0 g
```

### Export

Export a model (if no args) or a list of parameters (if args) to a typeset pdf:

``` { Oneil interpreter }
>>> export <param 1> <param 2> ... <param n>
```

This saves the pdf as `export.pdf` if you have [installed LaTeX locally](https://www.latex-project.org/get/).

If there are issues with the PDF, you can review the `export.tex` file. In VSCode the [LaTeX Workshop](https://marketplace.visualstudio.com/items?itemName=James-Yu.latex-workshop) extension is helpful. Also, VS Code doesn't handle LaTeX errors well. So if you run into lots of issues, use a more established TeX IDE, like [TexMaker](https://www.xm1math.net/texmaker/).

The current implementation uses biblatex for references.

### Design

Write a design onto the model:

``` { Oneil interpreter }
(your-model) >>> design design-name [variation-name]
----------------------------------------
Model: your-model
Design: variation-of-variation
Parameters: <count>
Tests: <count>
----------------------------------------
(variation-name@design-name@your-model) >>>
```

The interpreter writes the first design to the model, overwriting any overlapping model defaults. If there are further designs, it writes the first variation, overwriting any overlapping defaults or values from the first design. It continues overwriting until it reaches the last design. See below for more details on designs.

### Load

Load a new model:

``` { Oneil interpreter }
>>> load model-name
```

### Quit

Exit the interpreter:

``` { Oneil interpreter }
>>> quit
```

or

``` { .on }
>>> quit()
```

## Error Handling in the Interpreter

Ideally, if there is a problem with your Oneil code, the interpreter will tell you about it. If you get a python error, then it should be a problem with Oneil.

If you trigger an error in your initial model, the interpreter will go back to ask you for a model. If your model loads successfully and you trigger an error with an interpreter command, ideally the interpreter will go back to the model to accept another command.

This approach is new, so there are bound to be a lot of holes. The interpreter doesn't yet support step by step debugging, so you'll have to use [VSCode for this](#oneil-has-a-bug) for now.

## Known Issues and Limitations

* Range/option values are given in base units, *not specified units*.
* UnitError doesn't tell you what supported units are if you use an unsupported unit.
* There isn't a way to specify desired output units. Units specified on dependent parameters are only used to check that the cooresponding base units match.
* Scientific notation is supported in value assignments, but not limits. It should be supported in expressions, but this hasn't been tested.
* The Vim syntax highlighter gets *really* slow if you try to paste large amounts of LaTeX in. For now, make sure to paste large blocks of LaTeX using a different text editor or temporarily remove the ".on" file extension while you do.
* The Vim syntax highlighter breaks for the rest of the file after a LaTeX syntax error in a note. As a result, the rest of the file will be highlighted as a note.
* Currently can't use python functions for design overrides.
* (many more listed in Airtable and Patrick's Notion)

## Troubleshooting

### Something funny is happening with angular frequencies and frequencies

The funny thing about Hz and rad/s is that `1 Hz != 1` rad/s even though `1 Hz = 1/s` and `1 rad/s = 1/s`. You can thank the [International System of Units](https://iopscience.iop.org/article/10.1088/1681-7575/ac0240) for this madness. To escape this issue, Oneil doesn't recognize the SI definition of Hz. If you specify Hz as a unit, Oneil will internally convert it to rad/s by multiplying by 2 pi. If you want to use a frequency in an equation that expects Hz, you need to make sure the equation converts your frequency to Hz. For example, instead of `c=lambda*f` for the speed of light, you would use `c=lambda*f/(2*pi)`.

> As a side note, some people have suggested that this problem is solved if you use `cycles` as a base unit and let `Hz = 1 cycle/s`, but this quickly becomes messy as cycles will get propagated throughout your model. It's much easier to remember to convert rad/s to Hz in equations that need it.

### Oneil has a bug

Ideally, just tell Patrick about the bug, but if you want to try and fix it yourself, here are some things that will help:

To edit Oneil, you'll need to clone it and install it with the editable flag (-e), which ensures the pip install tracks your local changes.

``` { .sh }
git clone git@github.com/careweather/oneil.git

pip install -e oneil
```

To debug Oneil, you can call the interpreter from a python file:

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

## About

The initial methodology that inspired Oneil was proposed in Chapter 3 of [Concepts for Rapid-refresh, Global Ocean Surface Wind Measurement Evaluated Using Full-system Parametric Extrema Modeling](https://scholarsarchive.byu.edu/cgi/viewcontent.cgi?article=10166&context=etd), by M. Patrick Walton. For that work, the methodology was painfully implemented in a Google sheet. The conclusion provided ideas and inspiration for early versions of Oneil.

Oneil was developed at Care Weather Technologies, Inc. to support development of the Veery scatterometer. Veery was designed to perform as well as $100M heritage scatterometers at 3 orders of magnitude less cost. This dramatic improvement was facilitated in part by Oneil's streamlined system engineering capabilities.

Oneil is named after American physicist and space activist [Gerard K. O'Neill](https://en.wikipedia.org/wiki/Gerard_K._O%27Neill) who proposed the gargantuan space settlements known as [O'Neill cylinders](https://en.wikipedia.org/wiki/O%27Neill_cylinder). We built Oneil to meet our own needs, but we hope it stitches together the many domains required to make O'Neill cylinders and move humanity up the [Kardashev scale](https://en.wikipedia.org/wiki/Kardashev_scale).
