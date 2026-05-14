# Designs

A **design file** (extension `.one` starting with `design <target>`) is how you **apply** changes without editing the target model. Bindings in the file either **override** parameters that already exist on the target (same identifier) or **add** new parameters. Designs can also be applied to specific `reference` / `submodel` instances, and they can override parameters on those models directly using `<param>.<alias> = <value>` syntax.
Use design files to explore "what if" scenarios: alternative materials, different components, different environments, different mission parameters, and so on.

<!-- outline:
  1. Design files — syntax and running directly
  2. Applying designs: `submodel <design>`, `apply … to …`, CLI `--design`
  3. Design isolation — reference vs submodel, scoped overrides
  4. Applying designs to submodel aliases
-->

## Design files

Start your design file with a `design <target>` declaring the model you're refining. For the parameters themselves, you can use shorthand — just `identifier = expression` with an optional `:unit`, skipping the label that model files require.

With the following model of a planet:

```oneil
# planet.on
Surface gravity: g = 9.81 :m/s^2
Radius: R = 6371 :km
Mass: M = 5.97e24 :kg
```

You can define Mars by **applying** a design file that **overrides** `planet`’s base parameters and **adds** extra ones:

```oneil
# mars.one
design planet

g = 3.72 :m/s^2
R = 3390 :km
M = 6.417e23 :kg

Number of active rovers: rovers = 2
Mars solar day: t_sol = 24.6 :hr
Surface area: A = 4 * pi * R^2 :km^2
```

> [!NOTE]
> * If a name matches a parameter on the target model (`g`, `R`, `M`), your value or equation **overrides** it.
> * If the name does not match any existing parameters (rovers, t_sol, A), the design **adds** a new parameter to the model.
> * Equations may refer to parameters from the design file or target model file.
> * Equations and parameters are evaluated after all models in the system are built and every design in play has been **applied**.

### Running a design file directly

Because the design file declares its target model, you can run it directly:
```sh
oneil eval mars.one -P all
```

This command evaluates `planet.on` with the `mars.one` design applied and shows all parameters, including the new ones we've added:

```oneil-eval-output
g = 3.72 : m/s^2  # Surface gravity
R = 3390 : km  # Radius
M = 6.417e23: kg  # Mass
rovers = 2 # Number of active rovers
t_sol = 24.6 : hr  # Mars solar day
A = 1.444e8 : km^2  # Surface area
```

Alternatively you can pass the `--design` flag after the model file.

```sh
oneil eval planet.on --design mars.one
```

## Applying a design from within a model file

In model files you can **apply** a design to a submodel by naming the design file in a `submodel` clause. Oneil reads the `design <target>` line in that file, loads the underlying model, and **applies** the design.

```oneil
# mission.on
submodel mars as m

Spacecraft mass: m = 500 :kg
$ Surface weight: W = m * g.m :N
```

```sh
oneil eval mission.on
```

```oneil-eval-output
W = 1860 : N  # Surface weight
```

If the design file does not exist, or its `design` declaration names a model that cannot be loaded, you will get an error.

You can also **apply** a design to an instance you already imported using `apply <design_file> to <alias>`

```oneil
submodel planet as m
apply mars_design to m
```

Inside a design file, use `apply <design> to <alias>` for submodels and references that are already declared on the **target** model; a design file cannot declare new `submodel` lines of its own.

## Design isolation and scoped parameter overrides

Whether you use `reference` or `submodel` determines how broadly an **applied** design takes effect. 
Dotted `param.alias = value` syntax in a design file is how you write **scoped overrides** that change parameters within the imported models.

**`reference`** creates a **shared instance**: every model that imports the same file under any alias sees the same parameters.
A **scoped override** of `param.ref` changes the parameter everywhere the model is imported as a reference.

**`submodel`** creates an **independent instance**: two `submodel` imports of the same file are completely isolated.
A **scoped override** on one alias never changes the other.

In the following example, `mission_budget.on` uses both kinds of imports.
`constants` is a reference — mission **delta-v** `dv` is shared.
`rover_a` and `rover_b` are independent submodels — a design file can **override** parameters on each rover separately.

The propellant model is a single-burn maneuver obeying the rocket equation, **Δv = v_e ln(m_wet / m_dry)**, rearranged to **m_req = m_dry (e^(Δv / v_e) − 1)** as in the model below.
`v_e` is effective exhaust velocity, packaging engine efficiency (specific impulse).
**Dry mass** here means mass *after* the maneuver — stage structure plus payload (`m_bus` plus rover masses). **Wet mass** is that dry mass plus the propellant needed for the burn.

```oneil
# constants.on
Delta-v budget: dv = 2850 :m/s
```

```oneil
# rover.on
Rover mass: m = 1000 :kg
```

```oneil
# mission_budget.on
reference constants as c

submodel rover as rover_a
submodel rover as rover_b

Vehicle dry mass — structure and engines, excluding propellant and rovers: m_bus = 4000 :kg
Rover payload mass: m_pl = m.rover_a + m.rover_b :kg
$ Dry mass after maneuver: m_dry = m_bus + m_pl :kg
Effective exhaust velocity: v_e = 3000 :m/s
Loaded propellant available: m_p0 = 15000 :kg

Tsiolkovsky propellant required: m_req = m_dry * (e ^ (dv.c / v_e) - 1) :kg
$ Wet mass at ignition: m_wet = m_dry + m_req :kg
$ Propellant margin(-1e30, 1e30): m_mg = m_p0 - m_req :kg

$ Rover A mass: m_a = m.rover_a :kg
$ Rover B mass: m_b = m.rover_b :kg

test: m_req < m_p0
```

Rover mass impacts `m_dry` through `m_pl`, so heavier rovers increase required propellant.
The basic mission profile passes with extra propellant margin.

```sh
oneil mission_budget.on
```

```oneil-eval-output
────────────────────────────────────────────────────────────────────────────────
Model: mission_budget.on
Tests: 1/1 (PASS)
────────────────────────────────────────────────────────────────────────────────
m_dry = 6e3 : kg  # Dry mass after maneuver
m_wet = 15514 : kg  # Wet mass at ignition
m_mg = 5486 : kg  # Propellant margin
m_a = 1e3 : kg  # Rover A mass
m_b = 1e3 : kg  # Rover B mass
```

The following design **overrides** the shared **`dv.c`** binding in `constants` to model a costlier **Δv** requirement. 

```oneil
# high_dv.one
design mission_budget

dv.c = 4000 :m/s
```

```sh
oneil high_dv.one
```

```oneil-eval-output
────────────────────────────────────────────────────────────────────────────────
Model: mission_budget.on
Tests: 0/1 (FAIL)
────────────────────────────────────────────────────────────────────────────────
FAILING TESTS
mission_budget.on
test: m_req < m_p0
  - m_req = 16762 : kg
  - m_p0 = 1.5e4 : kg
────────────────────────────────────────────────────────────────────────────────
m_dry = 6e3 : kg  # Dry mass after maneuver
m_wet = 22762 : kg  # Wet mass at ignition
m_mg = -1762 : kg  # Propellant margin
m_a = 1e3 : kg  # Rover A mass
m_b = 1e3 : kg  # Rover B mass
```

You can **override** rover mass on one submodel only. The mission design below raises **`m.rover_a`** and causes the mission to break the propellant budget; **`m.rover_b`** stays at its model default.

```oneil
# heavy_rover.one
~ Heavy-duty rover A: too massive for the propellant budget.

design mission_budget

m.rover_a = 8000 :kg
```

```sh
oneil heavy_rover.one
```

```oneil-eval-output
────────────────────────────────────────────────────────────────────────────────
Model: mission_budget.on
Tests: 0/1 (FAIL)
────────────────────────────────────────────────────────────────────────────────
FAILING TESTS
mission_budget.on
test: m_req < m_p0
  - m_req = 20614 : kg
  - m_p0 = 1.5e4 : kg
────────────────────────────────────────────────────────────────────────────────
m_dry = 1.3e4 : kg  # Dry mass after maneuver
m_wet = 33614 : kg  # Wet mass at ignition
m_mg = -5614 : kg  # Propellant margin
m_a = 8e3 : kg  # Rover A mass
m_b = 1e3 : kg  # Rover B mass
```

> [!NOTE]
> * If a **scoped override** assigns an equation, it is evaluated in the design file’s scope, so you can refer to other bindings in that same file.
> * A value with incompatible units, or an **override** of a name that does not exist on the instance you targeted with a scoped override, is an error.
> * To change many parameters on one submodel — or to **add** parameters to it — prefer a dedicated design file whose `design` target is that submodel’s base model.
> * **Rule of thumb:** import shared environmental data (constants, tables, etc) as `reference`.
> Import components and system elements as `submodel`.

## Applying designs to submodel aliases

When you declare a local alias for a nested submodel (described in [Importing models](./09-importing-models.md)), the local alias and the one inside the intermediate model are two names for the **same model**.
A design change applied through either name takes effect on both.

Assuming the following galaxy model:

```oneil
# galaxy.on
submodel solar_system as sol [earth as e]

Probe mass: m_p = 800 :kg
$ Landing weight: W = m_p * g.e :N
$ Sol gravity reading: g_s = g_s.sol :m/s^2
```

We can **apply** the `mars` design to the `e` instance so Earth’s parameters are **overridden** by the Mars design file:

```oneil
# earth_as_mars.one
design galaxy

apply mars to e
```

```sh
oneil earth_as_mars.one
```

```oneil-eval-output
W = 2976 : N  # Landing weight
g_s = 3.72 : m/s^2  # Sol gravity reading
```

Because `galaxy.e` and `sol.e` are both aliases to the same model, both parameters pick up the change to gravity — `W` directly through `g.e`, and `g_s` through `sol`.
