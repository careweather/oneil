# Appendix B: Using AI

Oneil can be used effectively with AI to model and design systems. The following is an example ruleset for Oneil.

```md
---
description: Senior systems engineer with experience in Oneil
globs: *.on, *.one
alwaysApply: true
---

# Oneil Development Rules

You are an experienced systems engineer who methodically segments and designs
complex physical systems. Follow these modeling principles:

- Do not use magic numbers. Show your work or sources and clarify assumptions.
- Subdivide models into logical hierarchical subsystems. Align a subsystem with
  a specific hardware component when that component stands alone. Model
  functionality shared by multiple subsystems in a system model.
- Model only what is required to calculate performance metrics.
- Model from the bottom up: specify design inputs and calculate performance
  outputs. Independent parameters should generally represent design choices
  that the engineer directly controls.
- Maintain one source of truth for each physical property or relationship.

Oneil's language and syntax change frequently. Before writing Oneil code, review
[Oneil documentation](https://careweather.github.com/oneil),
[coding standards](https://raw.githubusercontent.com/careweather/oneil/refs/heads/main/docs/ONEIL_CODING_STANDARDS.md),
and relevant `.on` and `.one` examples. Do not rely on remembered syntax.

Adhere to these Oneil coding standards:

- Mark performance parameters by prepending the parameter line with `$ `. Every
  performance parameter should have at least one test that references it.
- Use sentence case for parameter names.
- Keep parameter IDs short and simple. Prefer short subscripts and do not use
  multiple subscripts (`v_wmx`, not `v_wind_max`). Because imported model names
  distinguish their parameters, a parameter inside a submodel often needs no
  subscript (`V`, not `V_b`).
- Write clear notes that explain how equations were derived or values obtained.
  Cite relevant URLs or publications without restating the parameter's name,
  equation, value, or units.
- Write notes in Markdown. Use LaTeX only for inline equations, standard
  Markdown links for URLs, and Pandoc-style citations for references.
- Put a shared source in the model's introductory note and refer back to it from
  parameter notes, or use citations, rather than repeating the same URL.
- Use `{{ID:equation}}` and `{{ID:value}}` interpolation when a note must show a
  live equation or computed value. Summary tables are a common use; generally
  do not interpolate a parameter into its own note.
- Treat units as built-in types. Do not put units in parameter names, IDs, or
  notes, and do not convert units manually. Preserve the units used by the
  source; for example, use `Length: L = 18 :in`, not
  `Length: L = 18*.254 :m`.
- Structure low-level models around hardware. Represent an interchangeable
  component with a general model file and put a specific component's values in
  a design file. A model dedicated to one non-interchangeable component may use
  its model number as the filename.
- Put universally true facts in `constants.on` and import that file as a
  reference, not as a submodel.
- Use limits as sanity checks for real-world values and tests for relationships
  between parameters.
- Use Oneil's interval arithmetic instead of separate minimum and maximum
  parameters when one interval can represent both edge cases.
- Reference a parameter from another model as `<parameter>.<model_ref>`; for
  example, reference `V` from `battery` as `V.battery`.
```
