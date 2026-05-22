# Designs, overlays, and imports

## Design files and `design`

A file may declare that it parameterizes another model:

```oneil
design radar
```

The target model can include a directory path:

```oneil
design ../models/satellite
design sensors/radar
```

- The rest of the file is interpreted relative to that **target model** (resolves like a normal model path, relative to this file's location).
- Parameter lines use the **shorthand** form `id = expr` (optional `: unit`) for overrides and simple additions. For **new parameters** (additions), the full form `Label [Limits]: [{RenderName}] id = expr` is also supported — same syntax as in ordinary model files. On overrides, the full or shorthand form may optionally include `[Limits]` to **adjust** the target parameter's bounds; label and render-name overrides use the same full form. Other metadata on overrides (performance marker, trace level, etc.) still comes from the target model.
- **Sections:** parameters declared inside a `section` block in the design file are grouped under that section in the composed view. Overrides moved into a design section are removed from their original section; unsectioned overrides keep the target model's section membership.
- **Scoped overrides:** use `param.ref = value` to override a parameter on a nested reference instance (e.g., `thrust.main_engine = 500 :N` overrides `thrust` on the instance bound to `main_engine`).
- Parameters set in a design file that don't exist on the target model are introduced as new parameters on the target at evaluation time (see [parameter additions](#parameter-additions)).

## Direct evaluation of design files

Design files (`.one`) can be evaluated directly since they specify their target model via the `design` declaration. Running `oneil mydesign.one` evaluates the target model with the design applied.

## `apply`

Design files loaded by `apply <path> to <ref>` use the **`.one`** extension (e.g., `apply foo to net` loads `foo.one`). Ordinary models remain **`.on`**.

The design file path supports directory prefixes:

```oneil
apply ../designs/network_design to net
apply configs/antenna_design to a
```

Apply a design file to a specific reference path on the current model:

```oneil
apply network_design to net
apply antenna_design to a
```

- `to <ref>(.<ref>)*` selects the reference/submodel alias path in the current file's import graph.
- Nested applies live in a `[ … ]` block under the outer `apply` and omit the leading `apply` keyword.

Multiple `apply` lines stack; **later** entries win for the same parameter.

## Scoped parameter overrides

Inside a design file, use dotted syntax to override parameters on nested reference instances:

```oneil
design spacecraft

thrust.main_engine = 1000 :N
thrust.aux_thruster = 200 :N
```

- `thrust` is the parameter name on the referenced model (e.g., a thruster model has a `thrust` parameter).
- `main_engine` is the reference alias in the target model (e.g., `submodel thruster as main_engine`).
- The value `1000 :N` replaces the evaluated value for the `thrust` parameter on the `main_engine` instance.

## Parameter additions

A design file may introduce parameters that do not exist on the target model:

```oneil
design cylinder

radius = 3 :m
diameter = 2 * radius       # new — not on cylinder.on
circumference = pi * diameter

# Full form with limits (additions only):
Temperature (0, 100): T = 300
```

These parameters are added to the target instance at evaluation time and are accessible from an enclosing model via `new_param.ref` syntax.

## Evaluation model

- Resolved IR keeps one structure per on-disk model file.
- Before evaluation, an **instance graph** is built that walks the model tree once and stamps out one composed instance per `(model_path, instance_path)`. Design composition — parameter overrides and parameter additions — is performed exclusively in that build pass.
- **Evaluation** drives the graph lazily: each parameter starts pending in a memo table and is forced on demand, with cycle detection on re-entrance. External references `param.alias` use `alias` to look up the correct child instance, so the same file imported under two aliases yields two different evaluated results when overlays differ.

See [`../architecture/design-overlays.md`](../architecture/design-overlays.md) for the
developer-facing implementation guide, and [grammar.ebnf](grammar.ebnf) for the formal
syntax.
