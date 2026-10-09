# Plain Parameter References Inherit Units

## Status

Accepted

## Context

A parameter whose value has a unit must declare that unit with a `:unit` annotation, or evaluation fails with "parameter is missing a unit". On a calculation such as `v = d/t :m/s`, the annotation states the intended unit, and Oneil checks it against the calculated one.

A parameter whose value is only a reference to another parameter, such as `P_l = P_t.r`, gets no such check from an annotation. The referenced parameter already declares its unit, so the annotation repeats it and has to be kept in sync with the source. Without it, the whole model fails to evaluate.

Unannotated references to dimensionless parameters were already accepted, but were converted to a plain number, so a reference to `18 :dB` displayed as `63.1`.

## Decision

A parameter whose value is a single reference to another parameter, either in the same model (`x = y`) or in another model (`x = y.r`), and that has no unit annotation, takes the referenced value with its unit and display unit unchanged. This applies to dimensionless units as well, so a reference to `18 :dB` displays as `18 :dB`.

Every other expression with a unit, including a reference combined with any operator or function, still requires an annotation. An annotation on a reference is still allowed and is checked as before.

## Consequences

- References to parameters in other models no longer duplicate the source's unit.
- Unannotated references to dimensionless parameters display in the referenced unit instead of as a plain number. Values and test results do not change.
- Wrapping a reference in a calculation, such as `x = 1*y`, still requires an annotation, so the rule depends on the form of the expression.
