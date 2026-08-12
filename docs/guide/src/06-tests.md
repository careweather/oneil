# Tests

Alongside parameters, Oneil provides tests, which allows users to verify that
certain properties, requirements, and expectations hold.

The syntax for tests is `test: <test-expression>`.

```oneil
test: 1 + 1 == 2

Component A Length: L_A = 5 :cm
Component B Length: L_B = 3 :cm
Max Length: L_max = 10 :cm

test: L_A + L_B <= L_max
```

A test expression can be any expression that produces a boolean (`true` or
`false`). For more information, see [Booleans](04-value-types.md#booleans)
and [Number operations](04-value-types.md#operations).

## Examples

The point of a test is to encode a requirement you care about: margins,
safety limits, or physical feasibility.

### Thrust versus gravity

For a vehicle to accelerate upward, thrust must exceed weight.

```oneil
Dry mass: m_dry = 420 :kg
Propellant mass: m_prop = 180 :kg
Liftoff mass: m = m_dry + m_prop :kg

Sea-level gravity: g = 9.81 :m/s^2
Liftoff thrust: F_thrust = 7500 :N

test: F_thrust > m * g
```

### Stress and material limit

Keep computed stress below the allowable value derived from yield strength
and a safety factor:

```oneil
Yield strength: sigma_y = 250 :MPa
Safety factor: SF = 2

Allowable stress: sigma_allow = sigma_y / SF :MPa
Working stress: sigma_work = 95 :MPa

test: sigma_work < sigma_allow
```

### Operating temperature

Confirm a junction stays inside the part’s rated range. Temperature uses
[Kelvin](05-units.md#supported-dimensions) as the underlying dimension:

```oneil
Ambient: T_amb = 260 :K
Self-heating: delta_T = 40 :K
Junction temperature: T_j = T_amb + delta_T :K

Rated maximum junction: T_max = 400 :K

test: T_j <= T_max
```

### Power budget

Check that available electrical power covers peak demand with headroom:

```oneil
Supply capability: P_supply = 48 :W
Peak load: P_peak = 35 :W
Minimum design margin: P_margin_min = 5 :W

test: P_supply >= P_peak + P_margin_min
```

### String modes and requirements

Tests can combine numeric checks with string equality if the model uses
a parameter to indicate the mode:

```oneil
Array configuration: config = 'series'
Cell count: n_cells = 12
Cells required for target voltage: n_req = 12

test: config == 'series' and n_cells == n_req
```

To run these checks automatically in a model repository's CI, see
[Appendix C: Continuous Integration](./c-ci-setup.md).
