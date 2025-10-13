# Oneil Model Resolver

The Oneil Model Resolver resolves a model and it's dependencies.

The purpose of model resolution is:

- ensure that all declared dependencies (submodels, references, and imports)
  exist and resolve their path
- ensure that all parameter references in parameter and test expressions exist
- normalize parameter units


## Algorithm

The algorithm mainly occurs in [`resolver/mod.rs`](src/resolver/mod.rs). It
follows these steps.
  1. Parse the model
  2. Split the AST declarations into python imports, model imports, parameters, and tests.
  3. Resolve python imports
     - **Currently, this is mostly a validation step to ensure that the python file exists.** In the future, we may decide to do more analysis, such as what functions the files contain. This would allow us to resolve functions to the Python file they are from.
  4. Resolve dependency models
     - We want to make sure the models that the current model references have already been loaded
  5. Resolve the path for each model import
     - This step creates a map of submodels and a map of references, mapping them to their resolved location
     - Note that every submodel is also treated as a reference
  6. Resolve variables and units in parameter expressions
  7. Resolve variables in test expressions


## References and Submodels

<!-- TODO: move this to the main language overview -->

One of the purposes of Oneil's models is to be able to represent **collections of systems and subsystems**. To this end, Oneil provides two different ways to import a model.

The first way to import a model is as a **reference**. When a model is imported as a reference, all of the *reference model parameters* are made available through the *reference alias*. The *reference alias* is either the alias provided or, if there isn't one, the name of the model.

```oneil
# === constants.on ===
Gravity of Earth: g = 9.8 :m/s^2


# === my_model.on ===
Mass of box: m_b = 5 :kg

# reference with alias
ref constants as c
Weight of box: w_b = m_b * g.c :N

# reference without alias
ref constants
Weight of box: w_b = m_b * g.constants :N
```

The second way to import a model is as a **submodel**. Like with a reference, all of the *submodel parameters* are available through the *submodel alias*. In addition to this, the model is also exported as a *submodel* of the current model. This means that the imported model can be referenced as `model.submodel`.

```oneil
# === radar.on ===
Radar cost: cost = 1000 :$


# === solar_panel.on ===
Solar panel cost: cost = 500 :$


# === satellite.on ===
use radar
use solar_panel as solar

Satellite cost: cost = cost.radar + cost.solar :$


# === product.on ===
use satellite
ref satellite.radar
ref satellite.solar_panel as solar
# ... or using `with` syntax ...
use satellite with [radar, solar_panel as solar]
```

Note that in the case of a submodel, *the submodel and reference name may be different*. If an alias is provided, it will be used as the reference name, but not as the submodel name. The submodel name will always be the name of the model.
