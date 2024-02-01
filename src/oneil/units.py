import re
import numpy as np


#################################################
# UNIT DEFINITIIONS
#################################################

"""
Oneil base units are structured as dicts of base units and their exponents. 
For example, 1 m/s^2 would be represented as {"m": 1, "s": -2}.

Collections of units follow the following format: 
{key: ({oneil base units}, multiplier, {optional characteristics})}.
Optional characteristics include:
    "alt": list of alternate names for the unit, if the alternate name has a non (s) plural, 
           then both singular and plural forms are given as a tuple in the list.
    "SI min": minimum of the range of values for which the SI prefixes are commonly used
    "SI max": maximum of the range of values for which the SI prefixes are commonly used

The BASE_UNITS collection is used to confirm that only valid units are used in oneil base unit dicts
and to provide characteristics for the base units.
"""
BASE_UNITS = {
    "kg"       :({"kg": 1}, 1, {"alt":["kilogram", "kilo"], "SI min": 1, "SI max": 1}),
    "m"         :({"m": 1}, 1, {"alt":["meter", "metre"], "SI max": 1e3}),
    "s"         :({"s": 1}, 1, {"alt":["second", "sec"], "SI max": 1}),
    "K"         :({"K": 1}, 1, {"alt":["Kelvin"], "SI min": 1, "SI max": 1}),
    "A"         :({"A": 1}, 1, {"alt":["Ampere", "Amp"]}),
    "b"         :({"b": 1}, 1, {"alt":["bit"], "SI min": 1, "SI max": 1}),
    "$"         :({"$": 1}, 1, {"alt":["dollar"], "SI min": 1, "SI max": 1}),
    "cap"       :({"cap": 1}, 1, {"alt":[("capacity", "capacities")], "SI min": 1, "SI max": 1}),
    "cd"        :({"cd": 1}, 1, {"alt":["candela"]}),
    "sr"        :({"sr": 1}, 1, {"alt":["steradian"]}),
    "mol"       :({"mol": 1}, 1, {"alt":["mole"]}),
}

# Find non-base units used in Oneil base unit dicts.
def invalid_units(UNIT_DICT):
    for k, v in UNIT_DICT.items():
        for u in v[0]:
            if u not in BASE_UNITS:
                return u
    return False

# Make sure each unit given in BASE_UNITS uses itself as the only base unit.
if any(v[0] != {k: 1} for k, v in BASE_UNITS.items()):
    raise ValueError("Invalid unit in BASE_UNITS.")

# Expand the unit collections to include the natural language variations.
def alt(UNITS):
    ALT_UNITS = {}
    for k, v in UNITS.items():
        if len(v) == 3:
            if "alt" in v[2]:
                for alt in v[2]["alt"]:
                    # Handle plural alternates
                    if isinstance(alt, tuple):
                        if alt[0] != alt[1]:
                            ALT_UNITS[alt[1]] = (v[0], v[1])
                        else:
                            ALT_UNITS[alt[0]] = (v[0], v[1])
                    elif isinstance(alt, str):
                        ALT_UNITS[alt] = (v[0], v[1])
                        ALT_UNITS[alt + "s"] = (v[0], v[1])
                    else:
                        raise ValueError("Invalid alternate format for unit alternate:", alt)
                    
    return ALT_UNITS


UNIT_OPERATORS = ["*", "/", "^"]

SI_PREFIXES = {
   "q": ({}, 1e-30, "quecto"),
   "r": ({}, 1e-27, "ronto"),
   "y": ({}, 1e-24, "yocto"),
   "z": ({}, 1e-21, "zepto"),
   "a": ({}, 1e-18, "atto" ),
   "f": ({}, 1e-15, "femto"),
   "p": ({}, 1e-12, "pico" ),
   "n": ({}, 1e-9 , "nano" ),
   "u": ({}, 1e-6 , "micro"),
   "m": ({}, 1e-3 , "milli"),
   "" : ({}, 1    , ""     ),
   "k": ({}, 1e3  , "kilo" ),
   "M": ({}, 1e6  , "mega" ),
   "G": ({}, 1e9  , "giga" ),
   "T": ({}, 1e12 , "tera" ),
   "P": ({}, 1e15 , "peta" ),
   "E": ({}, 1e18 , "exa"  ),
   "Z": ({}, 1e21 , "zetta"),
   "Y": ({}, 1e24 , "yotta"),
   "R": ({}, 1e27 , "ronna" ),
   "Q": ({}, 1e30 , "quetta"),
}

def prefix_units(units):
    prefixed_units = {}
    for ku, vu in units.items():
        SI_min = vu[2].get("SI min", -np.inf)
        SI_max = vu[2].get("SI max", np.inf)
        for kp, vp in SI_PREFIXES.items():
            if SI_min <= vp[1] <= SI_max:
                if len(vu) == 3:
                    language = {}
                    if vu[2].get("alt"):
                        language["alt"] = []
                        for alt in vu[2]["alt"]:
                            if isinstance(alt, tuple):
                                language["alt"].append((kp + alt[0], kp + alt[1]))
                            else:
                                language["alt"].append(kp + alt)
                prefixed_units[kp + ku] = (vu[0], vp[1] * vu[1], language)

    return prefixed_units

"""
DERIVED UNITS
In the background, Oneil doesn't keep track of derived units.
It uses the following collecitons to convert derived units to base units, and vice versa.
When parsing units on a parameter, a multiplier is used to convert the value to the correct magnitude.
When displaying a parameter, the same multiplier is used to decide the write derived unit to display.
    The parameter is displayed using the derived unit with the multiplier closest to the parameter's value. 
"""

# SI units are those derived units for which the SI prefixes are commonly used.
SI_UNITS = {
    "V": ({"kg": 1, "m": 2, "s": -3, "A": -1}, 1, {"alt": ["Volt"]}),
    "W": ({"kg": 1, "m": 2, "s": -3}, 1, {"alt": ["Watt"]}),
    "Hz": ({"s": -1}, 6.283185307179586, {"alt": [("Hertz", "Hertz")], "SI min": 1}),
    "g": ({"kg": 1}, 0.001, {"alt": ["gram"]}),
    "cd": ({"cd": 1}, 1, {"alt": ["candela"]}),
    "J": ({"kg": 1, "m": 2, "s": -2}, 1, {"alt": ["Joule"]}),
    "Wh": ({"kg": 1, "m": 2, "s": -2}, 3600.0, {"alt": ["Watt-hour"]}),
    "Ah": ({"A": 1, "s": 1}, 3600, {"alt": ["Amp-hour"]}),
    "T": ({"kg": 1, "s": -2, "A": -1}, 1, {"alt": ["Tesla"]}),
    "Ohm": ({"kg": 1, "m": 2, "s": -3, "A": -2}, 1, {"alt": ["Ohm"]}),
    "N": ({"kg": 1, "m": 1, "s": -2}, 1, {"alt": ["Newton"]}),
    "Gs": ({"kg": 1, "s": -2, "A": -1}, 0.0001, {"alt": ["Gauss"]}),
    "lm": ({"cd": 1, "sr": 1}, 1, {"alt": ["lumen"]}),
    "lx": ({"cd": 1, "sr": 1, "m": -2}, 1, {"alt": [("lux", "lux")]}),
    "bps": ({"b": 1, "s": -1}, 1, {"alt": [("bit/second", "bits/second")], "SI min": 1}),
    "B": ({"b": 1}, 8, {"alt": ["byte"], "SI min": 1}),
    "W/m^2": ({"kg": 1, "s": -3}, 1, {"alt": [("Watt/meter^2", "Watts/meter^2")]}),
    "m/s": ({"m": 1, "s": -1}, 1, {"alt": [("meter/second", "meters/second")]}),
    "m/s^2": ({"m": 1, "s": -2}, 1, {"alt": [("meter/second^2", "meters/second^2")]}),
}

if invalid_units(SI_UNITS):
    raise ValueError("Invalid unit in SI_UNITS: " + invalid_units(SI_UNITS))

SI_MULTIPLES = prefix_units(SI_UNITS) | prefix_units(BASE_UNITS)

# Legacy units are those derived units for which the SI prefixes are not used.
LEGACY_UNITS = {
    "mil.": ({"s": 1}, 3.1556952e10, {"alt": ("millenium", "millenia")}),
    "cen.": ({"s": 1}, 3.1556952e9, {"alt": ("century", "centuries")}),
    "dec.": ({"s": 1}, 3.1556952e8, {"alt": ("decade")}), 
    "yr": ({"s": 1}, 3.1556952e7, {"alt": ["year", "yr"]}),
    "mon": ({"s": 1}, 2.629746e6, {"alt": ["month"]}),
    "week": ({"s": 1}, 6.048e5),
    "day": ({"s": 1}, 8.64e4, {"alt": ["day"]}),
    "hr": ({"s": 1}, 3600, {"alt": ["hour", "hr"]}),
    "min": ({"s": 1}, 60, {"alt": ["minute", "min"]}),
    "°/s": ({"s": -1}, 0.017453292519943295, {"alt": [("degree/second", "degrees/second")]}),
    "°/min": ({"s": -1}, 1.0471975511965976, {"alt": [("degree/minute", "degrees/minute")]}),
    "°/hr": ({"s": -1}, 62.83185307179586, {"alt": [("degree/hour", "degrees/hour")]}),
    "rpm": ({"s": -1}, 0.10471975511965977, {"alt": [("rotation/min", "rotations/min"), ("revolution/minute", "revolutions/minute"), ("revolution/min", "revolutions/min")]}),
    "k$": ({"$": 1}, 1000.0, {"alt": ["thousand dollars"]}),
    "M$": ({"$": 1}, 1e6, {"alt": ["million dollars"]}),
    "B$": ({"$": 1}, 1e9, {"alt": ["billion dollars"]}),
    "T$": ({"$": 1}, 1e12, {"alt": ["trillion dollars"]}),
    "g_E": ({"m": 1, "s": -2}, 9.81, {"alt": [("Earth gravity", "Earth gravities")]}),
    "cm": ({"m": 1}, 0.01, {"alt": ["centimeter"]}),
}

if invalid_units(LEGACY_UNITS):
    raise ValueError("Invalid unit in LEGACY_UNITS: " + invalid_units(LEGACY_UNITS))

STANDARD_UNITS = SI_MULTIPLES | LEGACY_UNITS

DIMENSIONLESS_UNITS = {
    "rev": ({}, 1, {"alt": ["revolution", "rotation", "rev"]}),
    "cyc": ({}, 1, {"alt": ["cycle"]}),
    "rad": ({}, 1, {"alt": ["radian"]}),
    "°": ({}, 0.017453292519943295, {"alt": [("deg", "deg"), "degree"]}),
    "%":  ({}, 0.01, {"alt": [("percent", "percent")]}),
    "ppm": ({}, 1e-6, {"alt": [("part per million", "parts per million")]}),
    "ppb": ({}, 1e-9, {"alt": [("part per billion", "parts per billion")]}),
    "": ({}, 1),
    "'": ({}, 0.0002908882086657216, {"alt": ["arcminute", "arcmin"]}),
    '"': ({}, 4.84813681109536e-06, {"alt": ["arcsecond", "arcsec"]}),
}

if any(u for v in DIMENSIONLESS_UNITS.values() for u in v[0]):
    raise ValueError("Units in DIMENSIONLESS_UNITS should be {}.")

LINEAR_UNITS = STANDARD_UNITS | alt(STANDARD_UNITS) | DIMENSIONLESS_UNITS | alt(DIMENSIONLESS_UNITS)

#################################################
# UNIT PARSING
#################################################

def parse(unit_str):
    if unit_str in BASE_UNITS:
        units = {unit_str: 1}
        unit_fx = lambda x:x
    elif unit_str in LINEAR_UNITS:
        units = LINEAR_UNITS[unit_str][0]
        unit_fx = lambda x:x*LINEAR_UNITS[unit_str][1]
    elif unit_str.strip("dB") in LINEAR_UNITS:
        units = LINEAR_UNITS[unit_str.strip("dB")][0]
        unit_fx = lambda x:10**(x/10)*LINEAR_UNITS[unit_str][1]
    else:
        units, multiplier = _parse_compound_units(unit_str)
        unit_fx = lambda x:x*multiplier

    return units, unit_fx


def _parse_compound_units(unit_str):
    # Parse the unit string based on operators /, *, ^
    unit_list = [
        x for x in re.findall("[A-Za-z]+", unit_str) if x not in UNIT_OPERATORS
    ]

    # Find the indices of the above matches
    indices = [m.span() for m in re.finditer("[A-Za-z]+", unit_str)]

    # Initialize zero unit
    units = {unit: 0 for unit in BASE_UNITS}

    multiplier = 1

    # Iterate through the indices and unit_list together
    for index, unit in zip(indices, unit_list):
        if unit in BASE_UNITS:
            if index[0] == 0 or unit_str[index[0] - 1] == "*":
                if index[1] < len(unit_str) and unit_str[index[1]] == "^":
                    units[unit] += int(unit_str[index[1] + 1])
                else:
                    units[unit] += 1

            elif unit_str[index[0] - 1] == "/":
                if index[1] < len(unit_str) and unit_str[index[1]] == "^":
                    units[unit] -= int(unit_str[index[1] + 1])
                else:
                    units[unit] -= 1
        elif unit in LINEAR_UNITS:
            if index[0] == 0 or unit_str[index[0] - 1] == "*":
                if index[1] < len(unit_str) and unit_str[index[1]] == "^":
                    exponent = int(unit_str[index[1] + 1])
                    for key, value in LINEAR_UNITS[unit][0].items():
                        units[key] += value * exponent
                    multiplier *= LINEAR_UNITS[unit][1] ** exponent
                else:
                    for key, value in LINEAR_UNITS[unit][0].items():
                        units[key] += value
                    multiplier *= LINEAR_UNITS[unit][1]
            elif unit_str[index[0] - 1] == "/":
                if index[1] < len(unit_str) and unit_str[index[1]] == "^":
                    exponent = int(unit_str[index[1] + 1])
                    for key, value in LINEAR_UNITS[unit][0].items():
                        units[key] -= value * exponent
                    multiplier /= LINEAR_UNITS[unit][1] ** exponent
                else:
                    for key, value in LINEAR_UNITS[unit][0].items():
                        units[key] -= value
                    multiplier /= LINEAR_UNITS[unit][1]
        else:
            raise ValueError("Invalid unit: " + unit)

    # Strip zero units
    units = {key: value for key, value in units.items() if value != 0}

    return units, multiplier



#################################################
# UNIT DISPLAY
#################################################

def _round(num, n=3):
    formatstr = "%." + str(n) + "g"
    return float(formatstr % num)

def hr_vals_and_units(vals, units, pref=None, sigfigs=3):
    hrvals, hrunits = _hr_parts(vals, units, pref)

    hrstr = str(_round(hrvals[0], sigfigs))

    if len(hrvals) > 1 and hrvals[0] != hrvals[1]:
        if hrunits[0] != hrunits[1]:
            hrstr += (
                " "
                + hrunits[0]
                + " | "
                + str(_round(hrvals[1], sigfigs))
                + " "
                + hrunits[1]
            )
        else:
            hrstr += "|" + str(_round(hrvals[1], sigfigs)) + " " + hrunits[0]
    else:
        hrstr += " " + hrunits[0]

    return hrstr


def hr_units(units, vals=[0, 0]):
    pref=None
    _, hrunits = _hr_parts(vals, units, pref)

    return hrunits[0]


def _hr_parts(vals, units, pref=None):
    vals = vals if vals[0] != vals[1] else [vals[0]]
    hrunits = []
    hrvals = []

    hrunit = ""
    hrval = ""
    for val in vals:
        hrval, hrunit = _find_derived_unit(units, val, pref)

        if not hrunit:
            # Just build a raw unit string
            hrval = val
            unitstr = _build_compound_unit_str(units)
            hrunit = unitstr.strip()
        hrvals.append(hrval)
        hrunits.append(hrunit)
    return hrvals, hrunits

def _find_derived_unit(base_units, value, pref=None):
    hrval = ""
    hrunit = ""

    # If a unit was specified by the user, use it.
    if pref:
        if pref in LINEAR_UNITS:
            return value / LINEAR_UNITS[pref][1], pref
        elif pref.strip("dB") in LINEAR_UNITS:
            return 10*np.log10(value / LINEAR_UNITS[pref.strip("dB")][1]), pref
            

    # Search for derived units with matching base and closest matching value.
    # Search includes powers of the collection of base units.
    for i in range(1, 4):
        unpowered_units = {k: v / i for k, v in base_units.items()}
        for k, v in STANDARD_UNITS.items():
            if unpowered_units == v[0]:
                if not hrunit:
                    hrunit = k
                elif abs(value - v[1]**i) < abs(value - STANDARD_UNITS[hrunit][1]**i):
                    hrunit = k
                hrval = value / STANDARD_UNITS[hrunit][1]**i
        if hrunit:
            break

    return hrval, hrunit

def _build_compound_unit_str(units):
    compound_unit_str = ""
    nums = []
    dens = []
    if units:
        for unit, exp in units.items():
            if exp > 0:
                nums.append(unit)
            elif exp < 0:
                dens.append(unit)
            else:
                raise ValueError(
                    "Unit with zero exponent should have been purged before display."
                )
        if len(nums) > 0:
            if len(nums) > 1 and len(dens) > 0:
                compound_unit_str += "("
            for i, num in enumerate(nums):
                if i > 0:
                    compound_unit_str += " "
                compound_unit_str += num
                if units[num] != 1:
                    compound_unit_str += "^" + str(units[num])
            if len(nums) > 1 and len(dens) > 0:
                compound_unit_str += ")"
        else:
            compound_unit_str += "1"
        if len(dens) > 0:
            compound_unit_str += "/"
            if len(dens) > 1:
                compound_unit_str += "("
            for i, den in enumerate(dens):
                if i > 0:
                    compound_unit_str += " "
                compound_unit_str += den
                if units[den] != -1:
                    compound_unit_str += "^" + str(-units[den])
            if len(dens) > 1:
                compound_unit_str += ")"

    return compound_unit_str

if __name__ == "__main__":
    print(_find_derived_unit({}, 100, "dB"))