#!/usr/bin/env python3
"""Tests for arithmetic operations on Parameter intervals.

Run directly:

    python test/test_parameter_ops.py

Or:

    pytest test/test_parameter_ops.py
"""

import math
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))

from oneil import Parameter, DivideByZeroError  # noqa: E402


def assert_close(actual, expected, rel_tol=1e-9, abs_tol=1e-12, label=""):
    assert math.isclose(actual, expected, rel_tol=rel_tol, abs_tol=abs_tol), (
        f"{label}: expected {expected!r}, got {actual!r}"
    )


def assert_interval(p, lo, hi, label=""):
    assert p.min <= p.max, f"{label}: min ({p.min}) > max ({p.max})"
    assert_close(p.min, lo, label=f"{label} min")
    assert_close(p.max, hi, label=f"{label} max")


def _section(title):
    print("\n" + "=" * 70)
    print(title)
    print("=" * 70)


# ---------------------------------------------------------------------------
# __rmul__ : scalar * Parameter
# ---------------------------------------------------------------------------

def test_rmul_positive_scalar():
    _section("__rmul__: positive scalar * positive interval")
    p = Parameter((2.0, 3.0), {"m": 1}, "p")
    r = 2.0 * p
    assert_interval(r, 4.0, 6.0, "2 * [2,3]")
    print("  ok")


def test_rmul_negative_scalar():
    """Regression test: (-1.5) * R_E**2 used to throw ParameterError
    because __rmul__ did not reorder min/max after sign flip."""
    _section("__rmul__: negative scalar * positive interval (regression)")
    p = Parameter((2.0, 3.0), {"m": 1}, "p")
    r = -2.0 * p
    assert_interval(r, -6.0, -4.0, "-2 * [2,3]")

    R_E = Parameter((6356752.314, 6378137.0), {"m": 1}, "R_E")
    omega = (-1.5) * R_E**2
    assert omega.min <= omega.max, "(-3/2)*R_E^2 must have min <= max"
    assert omega.min < 0 and omega.max < 0, "(-3/2)*R_E^2 must be negative"
    print("  ok")


def test_rmul_zero_scalar():
    _section("__rmul__: zero scalar")
    p = Parameter((2.0, 3.0), {"m": 1}, "p")
    r = 0.0 * p
    assert_interval(r, 0.0, 0.0, "0 * [2,3]")
    print("  ok")


def test_rmul_negative_interval():
    _section("__rmul__: scalar * negative interval")
    p = Parameter((-3.0, -2.0), {"m": 1}, "p")
    r = 2.0 * p
    assert_interval(r, -6.0, -4.0, "2 * [-3,-2]")
    r = -2.0 * p
    assert_interval(r, 4.0, 6.0, "-2 * [-3,-2]")
    print("  ok")


def test_rmul_mixed_sign_interval():
    _section("__rmul__: scalar * mixed-sign interval")
    p = Parameter((-2.0, 3.0), {"m": 1}, "p")
    r = 2.0 * p
    assert_interval(r, -4.0, 6.0, "2 * [-2,3]")
    r = -2.0 * p
    assert_interval(r, -6.0, 4.0, "-2 * [-2,3]")
    print("  ok")


# ---------------------------------------------------------------------------
# __rtruediv__ : scalar / Parameter
# ---------------------------------------------------------------------------

def test_rtruediv_positive_scalar():
    _section("__rtruediv__: positive scalar / positive interval")
    p = Parameter((2.0, 3.0), {"m": 1}, "p")
    r = 6.0 / p
    assert_interval(r, 2.0, 3.0, "6 / [2,3]")
    print("  ok")


def test_rtruediv_negative_scalar():
    _section("__rtruediv__: negative scalar / positive interval (regression)")
    p = Parameter((2.0, 3.0), {"m": 1}, "p")
    r = -6.0 / p
    assert_interval(r, -3.0, -2.0, "-6 / [2,3]")
    print("  ok")


def test_rtruediv_negative_interval():
    _section("__rtruediv__: positive scalar / negative interval")
    p = Parameter((-3.0, -2.0), {"m": 1}, "p")
    r = 6.0 / p
    assert_interval(r, -3.0, -2.0, "6 / [-3,-2]")
    print("  ok")


def test_rtruediv_lower_bound_zero():
    """Regression: when an interval bound is 0, division should extend to ±inf,
    not raise (matches original behavior, which used np.inf for this case)."""
    _section("__rtruediv__: bound at 0 extends to ±inf, does not raise")
    p = Parameter((0.0, 2.0), {"m": 1}, "p")
    r = 1.0 / p
    assert r.min == 0.5 and r.max == float("inf"), f"1/[0,2] should be (0.5, inf), got ({r.min}, {r.max})"
    r = -1.0 / p
    assert r.min == float("-inf") and r.max == -0.5, f"-1/[0,2] should be (-inf, -0.5), got ({r.min}, {r.max})"
    print("  ok")


def test_rtruediv_upper_bound_zero():
    _section("__rtruediv__: upper bound at 0 extends to ±inf, does not raise")
    p = Parameter((-2.0, 0.0), {"m": 1}, "p")
    r = 1.0 / p
    assert r.min == float("-inf") and r.max == -0.5, f"1/[-2,0] should be (-inf, -0.5), got ({r.min}, {r.max})"
    r = -1.0 / p
    assert r.min == 0.5 and r.max == float("inf"), f"-1/[-2,0] should be (0.5, inf), got ({r.min}, {r.max})"
    print("  ok")


def test_rtruediv_straddling_zero():
    _section("__rtruediv__: interval strictly straddles zero -> unbounded both sides")
    p = Parameter((-1.0, 1.0), {"m": 1}, "p")
    r = 1.0 / p
    assert r.min == float("-inf") and r.max == float("inf")
    print("  ok")


def test_rtruediv_pure_zero_raises():
    _section("__rtruediv__: division by [0, 0] raises DivideByZeroError")
    p = Parameter((0.0, 0.0), {"m": 1}, "p")
    raised = False
    try:
        _ = 1.0 / p
    except DivideByZeroError:
        raised = True
    assert raised, "Expected DivideByZeroError for 1/[0,0]"
    print("  ok")


# ---------------------------------------------------------------------------
# __rfloordiv__ : scalar // Parameter
# ---------------------------------------------------------------------------

def test_rfloordiv_positive_scalar():
    """Regression test: 6 // [2, 3] used to throw because the formula
    swapped min/max in the result."""
    _section("__rfloordiv__: positive scalar // positive interval (regression)")
    p = Parameter((2.0, 3.0), {"m": 1}, "p")
    r = 6.0 // p
    assert_interval(r, 2.0, 3.0, "6 // [2,3]")
    print("  ok")


def test_rfloordiv_negative_scalar():
    _section("__rfloordiv__: negative scalar // positive interval")
    p = Parameter((2.0, 3.0), {"m": 1}, "p")
    r = -6.0 // p
    assert_interval(r, -3.0, -2.0, "-6 // [2,3]")
    print("  ok")


def test_rfloordiv_lower_bound_zero():
    _section("__rfloordiv__: bound at 0 extends to ±inf, does not raise")
    p = Parameter((0.0, 2.0), {"m": 1}, "p")
    r = 1.0 // p
    assert r.min == 0.5 and r.max == float("inf")
    r = -1.0 // p
    assert r.min == float("-inf") and r.max == -0.5
    print("  ok")


def test_rfloordiv_pure_zero_raises():
    _section("__rfloordiv__: division by [0, 0] raises DivideByZeroError")
    p = Parameter((0.0, 0.0), {"m": 1}, "p")
    raised = False
    try:
        _ = 1.0 // p
    except DivideByZeroError:
        raised = True
    assert raised, "Expected DivideByZeroError for 1//[0,0]"
    print("  ok")


# ---------------------------------------------------------------------------
# __rpow__ : scalar ** Parameter
# ---------------------------------------------------------------------------

def test_rpow_base_greater_than_one():
    _section("__rpow__: base > 1, positive exponent interval")
    p = Parameter((2.0, 3.0), {}, "p")
    r = 2.0 ** p
    assert_interval(r, 4.0, 8.0, "2 ** [2,3]")
    print("  ok")


def test_rpow_base_between_zero_and_one():
    """Regression test: 0.5 ** [2, 3] used to throw because the result
    is monotonically decreasing in the exponent."""
    _section("__rpow__: 0 < base < 1, positive exponent interval (regression)")
    p = Parameter((2.0, 3.0), {}, "p")
    r = 0.5 ** p
    assert_interval(r, 0.125, 0.25, "0.5 ** [2,3]")
    print("  ok")


def test_rpow_negative_exponent_interval():
    _section("__rpow__: base > 1, negative exponent interval")
    p = Parameter((-3.0, -2.0), {}, "p")
    r = 2.0 ** p
    assert_interval(r, 0.125, 0.25, "2 ** [-3,-2]")
    print("  ok")


# ---------------------------------------------------------------------------
# __neg__, __rsub__, __radd__ (already correct; lock behavior in)
# ---------------------------------------------------------------------------

def test_neg():
    _section("__neg__: -interval correctly swaps bounds")
    p = Parameter((2.0, 3.0), {"m": 1}, "p")
    r = -p
    assert_interval(r, -3.0, -2.0, "-[2,3]")
    print("  ok")


def test_rsub():
    _section("__rsub__: scalar - interval")
    p = Parameter((2.0, 5.0), {}, "p")
    r = 1.0 - p
    assert_interval(r, -4.0, -1.0, "1 - [2,5]")
    print("  ok")


def test_radd():
    _section("__radd__: scalar + interval")
    p = Parameter((2.0, 5.0), {}, "p")
    r = 1.0 + p
    assert_interval(r, 3.0, 6.0, "1 + [2,5]")
    print("  ok")


# ---------------------------------------------------------------------------
# __pow__ : Parameter ** scalar (also affects user's case via R_E**2)
# ---------------------------------------------------------------------------

def test_pow_positive_interval_squared():
    _section("__pow__: positive interval squared")
    p = Parameter((2.0, 3.0), {"m": 1}, "p")
    r = p**2
    assert_interval(r, 4.0, 9.0, "[2,3]**2")
    assert r.units == {"m": 2}, f"units should be m^2, got {r.units}"
    print("  ok")


def test_pow_then_negative_scalar_multiply():
    """End-to-end of the user's reported expression: (-3/2) * R_E**2."""
    _section("End-to-end: (-3/2) * R_E**2 (user-reported case)")
    R_E = Parameter((6356752.314, 6378137.0), {"m": 1}, "R_E")
    omega = (-3 / 2) * R_E**2
    assert omega.min <= omega.max
    assert omega.units == {"m": 2}
    expected_min = -1.5 * 6378137.0**2
    expected_max = -1.5 * 6356752.314**2
    assert_close(omega.min, expected_min, label="omega min")
    assert_close(omega.max, expected_max, label="omega max")
    print("  ok")


# ---------------------------------------------------------------------------
# __eq__ / __ne__ with None : needed so DivideByZeroError can render properly
# ---------------------------------------------------------------------------

def test_eq_ne_with_none():
    """Regression: error rendering paths use `param != None` / `param == None`.
    These must not raise TypeError."""
    _section("__eq__/__ne__ with None do not raise")
    p = Parameter((2.0, 3.0), {"m": 1}, "p")
    assert (p == None) is False  # noqa: E711
    assert (p != None) is True  # noqa: E711
    assert (None == p) is False  # noqa: E711
    assert (None != p) is True  # noqa: E711
    print("  ok")


def test_divide_by_zero_error_renders():
    """Regression: DivideByZeroError.__str__ used to fail because its context()
    method invoked Parameter.__ne__(None), which raised TypeError."""
    _section("DivideByZeroError str() works when constructed with a Parameter")
    p = Parameter((0.0, 0.0), {"m": 1}, "p_zero")
    err = DivideByZeroError(p)
    s = str(err)
    assert "DivideByZeroError" in s
    assert "p_zero" in s
    print("  ok")


# ---------------------------------------------------------------------------
# Test runner
# ---------------------------------------------------------------------------

ALL_TESTS = [
    test_rmul_positive_scalar,
    test_rmul_negative_scalar,
    test_rmul_zero_scalar,
    test_rmul_negative_interval,
    test_rmul_mixed_sign_interval,
    test_rtruediv_positive_scalar,
    test_rtruediv_negative_scalar,
    test_rtruediv_negative_interval,
    test_rtruediv_lower_bound_zero,
    test_rtruediv_upper_bound_zero,
    test_rtruediv_straddling_zero,
    test_rtruediv_pure_zero_raises,
    test_rfloordiv_positive_scalar,
    test_rfloordiv_negative_scalar,
    test_rfloordiv_lower_bound_zero,
    test_rfloordiv_pure_zero_raises,
    test_rpow_base_greater_than_one,
    test_rpow_base_between_zero_and_one,
    test_rpow_negative_exponent_interval,
    test_neg,
    test_rsub,
    test_radd,
    test_pow_positive_interval_squared,
    test_pow_then_negative_scalar_multiply,
    test_eq_ne_with_none,
    test_divide_by_zero_error_renders,
]


def main():
    failures = []
    for t in ALL_TESTS:
        try:
            t()
        except AssertionError as e:
            failures.append((t.__name__, repr(e)))
            print(f"  FAIL: {t.__name__}: {e}")
        except Exception as e:  # noqa: BLE001
            failures.append((t.__name__, repr(e)))
            print(f"  ERROR: {t.__name__}: {type(e).__name__}: {e}")

    print("\n" + "=" * 70)
    if failures:
        print(f"FAILED: {len(failures)} / {len(ALL_TESTS)}")
        for name, msg in failures:
            print(f"  - {name}: {msg}")
        print("=" * 70)
        return 1
    print(f"PASSED: {len(ALL_TESTS)} / {len(ALL_TESTS)}")
    print("=" * 70)
    return 0


if __name__ == "__main__":
    sys.exit(main())
