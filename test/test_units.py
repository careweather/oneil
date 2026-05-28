#!/usr/bin/env python3
"""Tests for oneil.units, focused on the unit-parsing and unit-display logic.

Run directly:

    python test/test_units.py

Or:

    pytest test/test_units.py
"""

import math
import os
import sys

# Make the in-repo `src/oneil` importable without installing.
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))

from oneil import units as un  # noqa: E402


TAU = 2 * math.pi  # The codebase defines Hz with a 2*pi multiplier.


# ---------------------------------------------------------------------------
# Small assertion helpers
# ---------------------------------------------------------------------------

def assert_close(actual, expected, rel_tol=1e-9, abs_tol=1e-12, label=""):
    assert math.isclose(actual, expected, rel_tol=rel_tol, abs_tol=abs_tol), (
        f"{label}: expected {expected!r}, got {actual!r}"
    )


def assert_raises(exc_type, fn, *args, **kwargs):
    try:
        fn(*args, **kwargs)
    except exc_type:
        return
    raise AssertionError(
        f"Expected {exc_type.__name__} from {getattr(fn, '__name__', fn)}"
        f"({args}, {kwargs}), but no exception was raised."
    )


def _section(title):
    print("\n" + "=" * 70)
    print(title)
    print("=" * 70)


# ---------------------------------------------------------------------------
# Base / simple / prefixed / legacy / dimensionless / alias parsing
# ---------------------------------------------------------------------------

def test_base_units():
    _section("Base units parse as themselves with identity multiplier")
    for base in ["m", "s", "kg", "K", "A", "b", "$"]:
        units, fx = un.parse(base)
        assert units == {base: 1}, f"{base}: units={units}"
        assert fx(1.0) == 1.0
        assert fx(42.0) == 42.0
    print("  ok")


def test_si_derived_units():
    _section("SI derived units (W, Hz, J, N, V, ...)")
    units, fx = un.parse("W")
    assert units == {"kg": 1, "m": 2, "s": -3}
    assert_close(fx(1.0), 1.0, label="W multiplier")

    units, fx = un.parse("J")
    assert units == {"kg": 1, "m": 2, "s": -2}
    assert_close(fx(1.0), 1.0, label="J multiplier")

    # Hz in this codebase carries a 2*pi factor (rad/s convention).
    units, fx = un.parse("Hz")
    assert units == {"s": -1}
    assert_close(fx(1.0), TAU, label="Hz multiplier")

    units, fx = un.parse("N")
    assert units == {"kg": 1, "m": 1, "s": -2}
    assert_close(fx(1.0), 1.0, label="N multiplier")

    units, fx = un.parse("V")
    assert units == {"kg": 1, "m": 2, "s": -3, "A": -1}
    print("  ok")


def test_si_prefixes():
    _section("SI prefixes on simple units (km, mW, MHz, ...)")
    units, fx = un.parse("km")
    assert units == {"m": 1}
    assert_close(fx(1.0), 1000.0, label="km")

    units, fx = un.parse("mW")
    assert units == {"kg": 1, "m": 2, "s": -3}
    assert_close(fx(1.0), 1e-3, label="mW")

    units, fx = un.parse("MHz")
    assert units == {"s": -1}
    assert_close(fx(1.0), 1e6 * TAU, label="MHz")

    units, fx = un.parse("ug")  # microgram
    assert units == {"kg": 1}
    assert_close(fx(1.0), 1e-9, label="ug")
    print("  ok")


def test_legacy_units():
    _section("Legacy units (hr, ft, lb, mph, ...)")
    units, fx = un.parse("hr")
    assert units == {"s": 1}
    assert_close(fx(1.0), 3600.0, label="hr")

    units, fx = un.parse("ft")
    assert units == {"m": 1}
    assert_close(fx(1.0), 0.3048, label="ft")

    units, fx = un.parse("lb")
    assert units == {"kg": 1}
    assert_close(fx(1.0), 0.45359237, label="lb")

    units, fx = un.parse("mph")
    assert units == {"m": 1, "s": -1}
    assert_close(fx(1.0), 0.44704, label="mph")
    print("  ok")


def test_dimensionless_units():
    _section("Dimensionless units (%, ppm, rad, '')")
    units, fx = un.parse("%")
    assert units == {}
    assert_close(fx(100.0), 1.0, label="100%")

    units, fx = un.parse("ppm")
    assert units == {}
    assert_close(fx(1.0), 1e-6, label="ppm")

    units, fx = un.parse("rad")
    assert units == {}
    assert_close(fx(1.0), 1.0, label="rad")

    units, fx = un.parse("")
    assert units == {}
    assert_close(fx(42.0), 42.0, label="empty string")
    print("  ok")


def test_aliases():
    _section("Aliases (meter, gram, Hertz, second)")
    units, fx = un.parse("meter")
    assert units == {"m": 1}
    assert_close(fx(1.0), 1.0, label="meter")

    units, fx = un.parse("meters")
    assert units == {"m": 1}
    assert_close(fx(1.0), 1.0, label="meters")

    units, fx = un.parse("gram")
    assert units == {"kg": 1}
    assert_close(fx(1.0), 1e-3, label="gram")
    print("  ok")


# ---------------------------------------------------------------------------
# Compound units
# ---------------------------------------------------------------------------

def test_compound_units_basic():
    _section("Compound units (W/Hz, kg*m/s^2, W/m^2, ...)")
    # W/Hz: {kg:1, m:2, s:-3} - {s:-1} = {kg:1, m:2, s:-2}
    units, fx = un.parse("W/Hz")
    assert units == {"kg": 1, "m": 2, "s": -2}, f"got {units}"
    assert_close(fx(1.0), 1.0 / TAU, label="W/Hz")

    # kg*m/s^2 = N
    units, fx = un.parse("kg*m/s^2")
    assert units == {"kg": 1, "m": 1, "s": -2}, f"got {units}"
    assert_close(fx(1.0), 1.0, label="kg*m/s^2")

    # W/m^2 → {kg:1, s:-3}
    units, fx = un.parse("W/m^2")
    assert units == {"kg": 1, "s": -3}, f"got {units}"
    assert_close(fx(1.0), 1.0, label="W/m^2")

    # m^2 (positive-only compound)
    units, fx = un.parse("m^2")
    assert units == {"m": 2}, f"got {units}"
    assert_close(fx(1.0), 1.0, label="m^2")

    # Mixed prefixes inside compound: MHz/m
    units, fx = un.parse("MHz/m")
    assert units == {"s": -1, "m": -1}, f"got {units}"
    assert_close(fx(1.0), 1e6 * TAU, label="MHz/m")
    print("  ok")


# ---------------------------------------------------------------------------
# dB on simple units (existing behavior must still work)
# ---------------------------------------------------------------------------

def test_db_simple():
    _section("dB on simple units (dBW, dBmW, dBHz)")
    units, fx = un.parse("dBW")
    assert units == {"kg": 1, "m": 2, "s": -3}
    assert_close(fx(0.0), 1.0, label="0 dBW")
    assert_close(fx(10.0), 10.0, label="10 dBW")
    assert_close(fx(20.0), 100.0, label="20 dBW")
    assert_close(fx(-10.0), 0.1, label="-10 dBW")

    # dBmW = dB referenced to milliwatt
    units, fx = un.parse("dBmW")
    assert units == {"kg": 1, "m": 2, "s": -3}
    assert_close(fx(0.0), 1e-3, label="0 dBmW")
    assert_close(fx(30.0), 1.0, label="30 dBmW")

    # dBHz uses Hz's 2*pi multiplier
    units, fx = un.parse("dBHz")
    assert units == {"s": -1}
    assert_close(fx(0.0), TAU, label="0 dBHz")
    assert_close(fx(10.0), 10.0 * TAU, label="10 dBHz")
    print("  ok")


def test_db_dimensionless():
    _section("dB on dimensionless (plain 'dB' as logarithmic ratio)")
    units, fx = un.parse("dB")
    assert units == {}
    assert_close(fx(0.0), 1.0, label="0 dB")
    assert_close(fx(10.0), 10.0, label="10 dB")
    assert_close(fx(20.0), 100.0, label="20 dB")
    assert_close(fx(-3.0), 10 ** (-0.3), label="-3 dB")
    print("  ok")


# ---------------------------------------------------------------------------
# dB on compound units (NEW behavior the fix introduces)
# ---------------------------------------------------------------------------

def test_db_compound_psd():
    _section("dB on compound units (dBW/Hz, dBmW/Hz, etc.)")
    # dBW/Hz: 0 dBW/Hz == 1 W/Hz in base units == 1/(2*pi) in {kg:1,m:2,s:-2}.
    units, fx = un.parse("dBW/Hz")
    assert units == {"kg": 1, "m": 2, "s": -2}, f"got {units}"
    assert_close(fx(0.0), 1.0 / TAU, label="0 dBW/Hz")
    assert_close(fx(10.0), 10.0 / TAU, label="10 dBW/Hz")
    assert_close(fx(-20.0), 0.01 / TAU, label="-20 dBW/Hz")

    # dBmW/Hz: 0 dBmW/Hz == 1 mW/Hz == 1e-3/(2*pi)
    units, fx = un.parse("dBmW/Hz")
    assert units == {"kg": 1, "m": 2, "s": -2}, f"got {units}"
    assert_close(fx(0.0), 1e-3 / TAU, label="0 dBmW/Hz")
    assert_close(fx(30.0), 1.0 / TAU, label="30 dBmW/Hz")

    # Equivalence: 0 dBW/Hz should equal 30 dBmW/Hz in base units.
    _, fx_db_w_hz = un.parse("dBW/Hz")
    _, fx_db_mw_hz = un.parse("dBmW/Hz")
    assert_close(fx_db_w_hz(0.0), fx_db_mw_hz(30.0), label="0 dBW/Hz == 30 dBmW/Hz")
    print("  ok")


def test_db_compound_with_exponent():
    _section("dB on compound units with exponents (dBW/m^2, dBV^2)")
    units, fx = un.parse("dBW/m^2")
    assert units == {"kg": 1, "s": -3}, f"got {units}"
    assert_close(fx(0.0), 1.0, label="0 dBW/m^2")
    assert_close(fx(10.0), 10.0, label="10 dBW/m^2")
    print("  ok")


# ---------------------------------------------------------------------------
# Display (hr_units / hr_vals_and_units) with explicit prefs
# ---------------------------------------------------------------------------

def _first_number(s):
    """Pull the leading numeric token off a human-readable string."""
    tok = s.split()[0]
    return float(tok)


def test_display_simple_pref():
    _section("Display of simple units honors `pref`")
    # Parameter holding 1.0 W (base units) requested as 'W'.
    s = un.hr_vals_and_units([1.0, 1.0], {"kg": 1, "m": 2, "s": -3}, pref="W")
    assert "W" in s
    assert_close(_first_number(s), 1.0, label="1 W displayed")

    # Same parameter requested as 'mW'.
    s = un.hr_vals_and_units([1.0, 1.0], {"kg": 1, "m": 2, "s": -3}, pref="mW")
    assert "mW" in s
    assert_close(_first_number(s), 1000.0, label="1 W as mW")
    print("  ok")


def test_display_db_simple():
    _section("Display of simple dB units (dBW, dBmW)")
    # 1 W base displayed as dBW -> 0 dBW
    s = un.hr_vals_and_units([1.0, 1.0], {"kg": 1, "m": 2, "s": -3}, pref="dBW")
    assert "dBW" in s
    assert_close(_first_number(s), 0.0, abs_tol=1e-9, label="1 W as dBW")

    # 1 W = 1000 mW -> 30 dBmW
    s = un.hr_vals_and_units([1.0, 1.0], {"kg": 1, "m": 2, "s": -3}, pref="dBmW")
    assert "dBmW" in s
    assert_close(_first_number(s), 30.0, abs_tol=1e-9, label="1 W as dBmW")
    print("  ok")


def test_display_compound_pref():
    _section("Display of compound units honors `pref` (W/Hz, kg*m/s^2)")
    # Build a base value equal to 1 W/Hz, then display as W/Hz -> should be 1.
    _, fx = un.parse("W/Hz")
    base_val = fx(1.0)
    s = un.hr_vals_and_units([base_val, base_val], {"kg": 1, "m": 2, "s": -2}, pref="W/Hz")
    assert "W/Hz" in s
    assert_close(_first_number(s), 1.0, label="1 W/Hz round-trip")
    print("  ok")


def test_display_db_compound():
    _section("Display of dB compound units (dBW/Hz)")
    # Base value equal to 1 W/Hz -> 0 dBW/Hz.
    _, fx = un.parse("W/Hz")
    base_val = fx(1.0)
    s = un.hr_vals_and_units([base_val, base_val], {"kg": 1, "m": 2, "s": -2}, pref="dBW/Hz")
    assert "dBW/Hz" in s, f"got {s!r}"
    assert_close(_first_number(s), 0.0, abs_tol=1e-9, label="1 W/Hz as dBW/Hz")

    # Base value equal to 1000 W/Hz -> 30 dBW/Hz.
    base_val_1000 = fx(1000.0)
    s = un.hr_vals_and_units(
        [base_val_1000, base_val_1000], {"kg": 1, "m": 2, "s": -2}, pref="dBW/Hz"
    )
    assert_close(_first_number(s), 30.0, abs_tol=1e-9, label="1000 W/Hz as dBW/Hz")

    # Same base value, displayed as dBmW/Hz -> +30 dB shift.
    s_mw = un.hr_vals_and_units(
        [base_val, base_val], {"kg": 1, "m": 2, "s": -2}, pref="dBmW/Hz"
    )
    assert "dBmW/Hz" in s_mw
    assert_close(_first_number(s_mw), 30.0, abs_tol=1e-9, label="1 W/Hz as dBmW/Hz")
    print("  ok")


def test_display_db_compound_zero_value():
    _section("Display of 0 in dB compound units yields -inf")
    s = un.hr_vals_and_units([0.0, 0.0], {"kg": 1, "m": 2, "s": -2}, pref="dBW/Hz")
    assert "dBW/Hz" in s
    # First token should parse as -inf.
    val = _first_number(s)
    assert math.isinf(val) and val < 0, f"expected -inf, got {val}"
    print("  ok")


# ---------------------------------------------------------------------------
# Error handling
# ---------------------------------------------------------------------------

def test_invalid_unit_strings():
    _section("Invalid unit strings raise ValueError")
    assert_raises(ValueError, un.parse, "foo")
    # `.strip("dB")` latent bug — "WdB" must NOT be silently treated as "dB+W".
    assert_raises(ValueError, un.parse, "WdB")
    # dB applied to an invalid inner unit.
    assert_raises(ValueError, un.parse, "dBfoo")
    # Compound with invalid inner term.
    assert_raises(ValueError, un.parse, "dBW/foo")
    print("  ok")


def test_compound_format_errors():
    _section("Malformed compound strings raise ValueError")
    assert_raises(ValueError, un.parse, "/m")
    assert_raises(ValueError, un.parse, "m/")
    assert_raises(ValueError, un.parse, "m**s")  # consecutive operators
    print("  ok")


def test_display_dimension_mismatch():
    _section("Display with a `pref` of wrong dimensions raises ValueError")
    # Asking to display a length as dBW/Hz must fail.
    assert_raises(
        ValueError,
        un.hr_vals_and_units,
        [1.0, 1.0],
        {"m": 1},
        pref="dBW/Hz",
    )
    # Likewise for plain compound mismatch.
    assert_raises(
        ValueError,
        un.hr_vals_and_units,
        [1.0, 1.0],
        {"m": 1},
        pref="W/Hz",
    )
    # And for simple unit mismatch.
    assert_raises(
        ValueError,
        un.hr_vals_and_units,
        [1.0, 1.0],
        {"m": 1},
        pref="W",
    )
    print("  ok")


# ---------------------------------------------------------------------------
# Round-trips: parse -> base -> display in the same unit recovers the input
# ---------------------------------------------------------------------------

def _roundtrip(unit_str, input_value, abs_tol=1e-6, sigfigs=8):
    units, fx = un.parse(unit_str)
    base = fx(input_value)
    s = un.hr_vals_and_units(
        [base, base], units, pref=unit_str, sigfigs=sigfigs
    )
    out = _first_number(s)
    assert_close(out, input_value, abs_tol=abs_tol, label=f"roundtrip {unit_str}")


def test_roundtrips():
    _section("Round-trips: parse(unit).fx(x) then display(pref=unit) == x")
    _roundtrip("W", 5.0)
    _roundtrip("mW", 250.0)
    _roundtrip("Hz", 7.5)
    _roundtrip("MHz", 2.4)
    _roundtrip("ft", 12.5)
    _roundtrip("hr", 3.25)
    _roundtrip("W/Hz", 4.2)
    _roundtrip("kg*m/s^2", 9.81)
    _roundtrip("dBW", 13.0)
    _roundtrip("dBmW", -7.0)
    _roundtrip("dBW/Hz", 0.0)
    _roundtrip("dBW/Hz", -174.0)  # the canonical kTB noise floor reference
    _roundtrip("dBmW/Hz", -120.5)
    print("  ok")


# ---------------------------------------------------------------------------
# Test runner
# ---------------------------------------------------------------------------

ALL_TESTS = [
    test_base_units,
    test_si_derived_units,
    test_si_prefixes,
    test_legacy_units,
    test_dimensionless_units,
    test_aliases,
    test_compound_units_basic,
    test_db_simple,
    test_db_dimensionless,
    test_db_compound_psd,
    test_db_compound_with_exponent,
    test_display_simple_pref,
    test_display_db_simple,
    test_display_compound_pref,
    test_display_db_compound,
    test_display_db_compound_zero_value,
    test_invalid_unit_strings,
    test_compound_format_errors,
    test_display_dimension_mismatch,
    test_roundtrips,
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
