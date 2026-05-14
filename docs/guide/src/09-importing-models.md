# Importing models

Models rarely stand alone. When your system is made of parts — subsystems,
environments, shared constants — you'll want to pull those in from other files.
Oneil gives you two ways to do that: **reference** and **submodel**.

## References

A **reference** makes another model's parameters available under an alias. Use
it for shared parameters and models — physical constants, material properties,
standard environments — that belong to the world your system lives in, not to
the system itself.

For example with the following constants:
```oneil
# constants.on
Speed of light: c = 299792458 :m/s
Planck constant: h = 6.626e-34 :J*s
Boltzmann constant: k_B = 1.38e-23 :J/K
Gravitational constant: G = 6.674e-11 :N*m^2/kg^2
```

We can model both a photon's energy

```oneil
# photon.on
reference constants as c

Photon frequency: f = 5.09e14 :Hz
Photon energy: E = h.c * f :J
```

As well as an radio link

```oneil
# link_budget.on
reference constants as c

Distance: d = 384400 :km
Signal frequency: f = 2.4e9 :Hz
Path loss: L_fs = (4 * pi * d * f / c.c)^2
```

Creating a short alias using `as alias_name` is optional. Without the alias you access parameters using the model's name:

```oneil
# photon_ref_constants.on
reference constants # no alias 
Photon frequency: f = 5.09e14 :Hz
Photon energy: E = h.constants * f :J
```

## Submodels

A **submodel** adds a model as part of the current model and gives access to the other model's parameters.
Unlike a reference, each `submodel` statement creates an independent model — if you import the same model twice
under different aliases each one has its own parameters, which you can change independently by **applying** a **design file** to that instance (see [Designs](./10-designs.md)).

Planets are a natural fit — a mission might visit multiple planets, and each needs its own independent parameters.

```oneil
# planet.on
Surface gravity: g = 9.81 :m/s^2
Radius: R = 6371 :km
Mass: M = 5.97e24 :kg
```

The planet model defaults to Earth's values on `earth`, but **`submodel mars`** imports the `mars` design file which applies a mars design to the `planet` instance (`m`): its bindings **override** `g`, `R`, and `M`, and **add** Mars-specific parameters. The two instances stay independent.

```oneil
# mission_earth_mars.on
submodel planet as earth
submodel mars as m

Spacecraft mass: m = 500 :kg

Weight on Earth: W_e = m * g.earth :N
Weight on Mars:  W_m = m * g.m  :N
```

Like **reference** imports, the name after `as` is an *alias*. If no alias is given, the default alias is the model name:

```oneil
# submodel_planet_default_alias.on
submodel planet

Surface gravity seen: g_l = g.planet :m/s^2
```

It is an error for two submodels to have the same alias.

## Accessing submodel parameters

To access a parameter inside a reference or submodel, write `parameter_id.alias`
— the **parameter comes first**, the model's alias second.

```oneil
# satellite.on
reference constants as c
submodel planet as p

Orbital speed: v = sqrt(G.c * M.p / R.p) :m/s
```

> [!NOTE]
> This is the reverse of the `object.property` convention in most programming languages, and it is intentional. 
> In engineering equations parameters are primary, whereas subscripts often qualify which subsystem or model the parameter is part of.

## Referring to a nested submodel

A submodel is also *exported* as part of the current model's structure. 
A parent of `satellite.on` can reach nested parameters by importing a reference to a nested submodel using a local alias.

For example with solar system defined as follows:
```oneil
# solar_system.on
submodel planet as earth
submodel mars

Star mass: M_s = 1.989e30 :kg
Earth orbital period: T_e = 365.25 :days
Earth surface gravity: g_s = g.earth :m/s^2
```

The syntax `[alias]` or `[alias as local_alias]` at the end of the `submodel` line exposes the nested submodels to use locally. In this example we use the solar system in a mission and access earth parameters through a local alias.

```oneil
# mission_sol.on
# Import the solar system and declare `e` as a local alias for `earth`,
# so it can be used directly at the mission level.
submodel solar_system as sol [earth as e]

Probe mass: m_p = 800 :kg

# Access the parameters of e here
Landing weight: W = m_p * g.e :N

# Access solar-system parameters as normal
Star mass: M_s = M_s.sol :kg
Earth orbit: T = T_e.sol :days
```

You can pull in multiple aliases by separating them with commas:

```oneil
# mission_two_targets.on
submodel solar_system as sol [
    earth as e, 
    mars as t # landing target
]

Probe mass: m_p = 800 :kg
Weight on Earth: W_e = m_p * g.e :N
Landing weight on target: W_t = m_p * g.t :N
```

[Designs](./10-designs.md) explains **design files** (`design <target>` in a `.one` file): you **apply** a design to specific `reference` / `submodel` instances. A design **overrides** existing parameters (same name as on the target model) and can **add** new ones.