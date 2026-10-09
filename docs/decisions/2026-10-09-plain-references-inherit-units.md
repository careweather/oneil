# Plain Parameter References Inherit Units

## Status

Accepted

## Context

A parameter whose value has physical dimensions must declare its unit with a `:unit` annotation, or evaluation fails with "parameter is missing a unit". On a calculation such as `v = d/t :m/s`, the annotation states the intended unit, and Oneil checks it against the calculated one.

A parameter whose value is only a reference to another parameter, such as `P_l = P_t.r`, gets no such check from an annotation. The referenced parameter already declares its unit, so the annotation repeats it and has to be kept in sync with the source. Without it, the whole model fails to evaluate.

A dimensionless value without an annotation, such as a reference to `18 :dB`, already evaluates to a plain number, so it displays as `63.1`. Some operations read a value in its display unit. `strip` returns the number in the display unit, and Oneil reads limits without units in the display unit. If a reference to `90 :%` kept `%`, `strip` would return `90` instead of `0.9`, and the limits `(0, 1)` would mean 0 % to 1 %.

## Decision

A parameter whose value is a single reference to another parameter with physical dimensions, either in the same model (`x = y`) or in another model (`x = y.r`), and that has no unit annotation, takes the referenced value with its unit and display unit unchanged.

Every other expression with physical dimensions, including a reference combined with any operator or function, still requires an annotation. Dimensionless references and calculations keep their existing behavior: without an annotation, they become plain dimensionless values. An annotation on a reference is still allowed and is checked as before.

## Consequences

- References to parameters in other models no longer duplicate the source's unit.
- Only parameters that failed with "parameter is missing a unit" change. Every model that evaluated before gives the same values, display units, and test results.
- A reference to a dimensionless parameter displays as a plain number. To keep the referenced unit, such as `dB`, the reference needs an annotation.
- Wrapping a reference in a calculation, such as `x = 1*y`, still requires an annotation, so the rule depends on the form of the expression.
