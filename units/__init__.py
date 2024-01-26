import re
import numpy as np
import functools

"""
Oneil units are structured as dicts of base units and their exponents. 
For example, 1 m/s^2 would be represented as {"m": 1, "s": -2}. 
The base units are as follows:
"""
BASE_UNITS = {
    "kg"        :({"kg": 1}, 1, {"alt":("kilogram", "kilo")}),
    "m"         :({"m": 1}, 1, {"alt":("meter", "metre")}),
    "s"         :({"s": 1}, 1, {"alt":("second", "sec")}),
    "K"         :({"K": 1}, 1, {"alt":("Kelvin")}),
    "A"         :({"A": 1}, 1, {"alt":("Ampere", "Amp")}),
    "b"         :({"b": 1}, 1, {"alt":("bit")}),
    "$"         :({"$": 1}, 1, {"alt":("dollar")}),
    "capacity"  :({"capacity": 1}, 1, {"plural":"capacities"}),
    "cd"        :({"cd": 1}, 1, {"alt":("candela")}),
    "sr"        :({"sr": 1}, 1, {"alt":("steradian")}),
    "mol"       :({"mol": 1}, 1, {"alt":("mole")}),
} # kilograms, meters, seconds, Kelvins, Amps, bits, dollars, capacities, candelas, steradians

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

"""
DERIVED UNITS
In the background, Oneil doesn't keep track of derived units.
It uses the following dictionary to convert derived units to base units.
When parsing units on a parameter, a multiplier is used to convert the value to the correct magnitude.
When displaying a parameter, a human-readable threshold is used to decide the write derived unit to display.
    The parameter is displayed using the derived unit with the largest threshold that is less than the parameter's value. 
This dictionary units follow the following format: {unit, ({oneil base units}, multiplier, human-readable threshold)}.
"""

# SI units are those derived units for which the SI prefixes are widely used and no exceptions exist.
SI_UNITS = {
    "V": ({"kg": 1, "m": 2, "s": -3, "A": -1}, 1, {"alt": ("Volt")}),
    "W": ({"kg": 1, "m": 2, "s": -3}, 1, {"alt": ("Watt")}),
    "Hz": ({"s": -1}, 6.283185307179586, {"alt": ("Hertz"), "plural": "Hertz"}),
    "gm": ({"kg": 1}, 0.001, {"alt": ("gram")}),
    "A": ({"A": 1}, 1, {"alt": ("Amp")}),
    "B": ({"b": 1}, 8, {"alt": ("Byte")}),
    "cd": ({"cd": 1}, 1, {"alt": ("candela")}),
    "J": ({"kg": 1, "m": 2, "s": -2}, 1, {"alt": ("Joule")}),
    "Wh": ({"kg": 1, "m": 2, "s": -2}, 3600.0, {"alt": ("Watt-hour")}),
    "Ah": ({"A": 1, "s": 1}, 3600, {"alt": ("Amp-hour")}),
    "T": ({"kg": 1, "s": -2, "A": -1}, 1, {"alt": ("Tesla")}),
    "Ohm": ({"kg": 1, "m": 2, "s": -3, "A": -2}, 1, {"alt": ("Ohm")}),
    "N": ({"kg": 1, "m": 1, "s": -2}, 1, {"alt": ("Newton")}),
    "Gs": ({"kg": 1, "s": -2, "A": -1}, 0.0001, {"alt": ("Gauss")}),
    "lm": ({"cd": 1, "sr": 1}, 1, {"alt": ("lumen")}),
    "lx": ({"cd": 1, "sr": 1, "m": -2}, 1, {"alt": ("lux"), "plural": "lux"}),
    "bps": ({"b": 1, "s": -1}, 1, {"alt": ("bit/second"), "plural": "bits/second"}),
}

def prefix_units(prefixes, units):
    prefixed_units = {}
    for ku, vu in units.items():
        for kp, vp in prefixes.items():
            if len(vu) == 3:
                language = {}
                if vu[2].get("plural"):
                    language["plural"] = vp[2] + vu[2]["plural"]
                elif vu[2].get("alt"):
                    language["alt"] = vp[2] + vu[2]["alt"]
            prefixed_units[kp + ku] = (vu[0], vp[1] * vu[1], {})

    return prefixed_units

SI_MULTIPLES = prefix_units(SI_PREFIXES, SI_UNITS)

# Legacy units are those derived units for which the SI prefixes are not widely used or exceptions exist.
LEGACY_UNITS = {
    "day": ({"s": 1}, 8.64e4),
    "week": ({"s": 1}, 6.048e5),
    "month": ({"s": 1}, 2.629746e6),
    "year": ({"s": 1}, 3.1556952e7, {"alt": ("yr")}),
    "decade": ({"s": 1}, 3.1556952e8),
    "century": ({"s": 1}, 3.1556952e9, {"plural": ("centuries")}),
    "millennium": ({"s": 1}, 3.1556952e10, {"plural": ("millenia")}),
    "hour": ({"s": 1}, 3600, {"alt": ("hr")}),
    "minute": ({"s": 1}, 60, {"alt": ("min")}),
    "rotation": ({}, 1, {"alt": ("revolution")}),
    "cycle": ({}, 1),
    "radian": ({}, 1, {"alt": ("rad")}),
    "degree": ({}, 0.017453292519943295, {"alt": ("°")}),
    "degree/second": ({"s": -1}, 0.017453292519943295, {"alt": ("°/s")}),
    "degree/minute": ({"s": -1}, 1.0471975511965976, {"alt": ("°/min")}),
    "degree/hour": ({"s": -1}, 62.83185307179586, {"alt": ("°/hr")}),
    "rotations/minute": ({"s": -1}, 0.10471975511965977, {"alt": ("rpm", "rotations/min", "revolutions/minute", "revolutions/min")}),
    "k$": ({"$": 1}, 1000.0, {"alt": ("thousand dollars")}),
    "M$": ({"$": 1}, 1e6, {"alt": ("million dollars")}),
    "B$": ({"$": 1}, 1e9, {"alt": ("billion dollars")}),
    "T$": ({"$": 1}, 1e12, {"alt": ("trillion dollars")}),
    "%":  ({}, 0.01, {"alt": ("percent"), "plural": "percent"}),
    "km": ({"m": 1}, 1000, {"alt": ("kilometer")}),
    "m": ({"m": 1}, 1, {"alt": ("meter", "metre")}),
    "cm": ({"m": 1}, 0.01, {"alt": ("centimeter")}),
    "mm": ({"m": 1}, 0.001, {"alt": ("millimeter")}),
    "um": ({"m": 1}, 1e-6, {"alt": ("micrometer")}),
    "nm": ({"m": 1}, 1e-9, {"alt": ("nanometer")}),
    "g":  ({"m": 1, "s": -2}, 9.81, {"alt": ("Earth gravity"), "plural": "Earth gravities"}),
}

DERIVED_UNITS = SI_MULTIPLES | LEGACY_UNITS

ALT_UNITS = {}
for k, v in DERIVED_UNITS.items():
    if len(v) == 3:
        if "alt" in v[2]:
            for alt in v[2]["alt"]:
                ALT_UNITS[alt] = (v[0], v[1])
        if "plural" in v[2]:
            ALT_UNITS[v[2]["plural"]] = (v[0], v[1])
        else:
            ALT_UNITS[k + "s"] = (v[0], v[1])
    else:
        ALT_UNITS[k + "s"] = (v[0], v[1])

NON_BASE_UNITS = DERIVED_UNITS | ALT_UNITS

# @functools.cache
def find_derived_unit(base_units, value):
    unit = ""
    
    for k, v in LEGACY_UNITS.items():
        if base_units == v[0]:

            if not unit:
                unit = k
            elif LEGACY_UNITS[unit][1] > v[1]:
                if LEGACY_UNITS[unit][1] > value:
                    unit = k

    return unit


def _round(num, n=3):
    formatstr = "%." + str(n) + "g"
    return float(formatstr % num)

# @functools.cache
def parse(unit_str):
    if unit_str in BASE_UNITS:
        units = {unit_str: 1}
        multiplier = 1
    elif unit_str in NON_BASE_UNITS:
        units = NON_BASE_UNITS[unit_str][0]
        multiplier = NON_BASE_UNITS[unit_str][1]
    else:
        units, multiplier = parse_compound_units(unit_str)

    return units, multiplier


def parse_compound_units(unit_str):
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
        elif unit in NON_BASE_UNITS:
            if index[0] == 0 or unit_str[index[0] - 1] == "*":
                if index[1] < len(unit_str) and unit_str[index[1]] == "^":
                    exponent = int(unit_str[index[1] + 1])
                    for key, value in NON_BASE_UNITS[unit][0].items():
                        units[key] += value * exponent
                    multiplier *= NON_BASE_UNITS[unit][1] ** exponent
                else:
                    for key, value in NON_BASE_UNITS[unit][0].items():
                        units[key] += value
                    multiplier *= NON_BASE_UNITS[unit][1]
            elif unit_str[index[0] - 1] == "/":
                if index[1] < len(unit_str) and unit_str[index[1]] == "^":
                    exponent = int(unit_str[index[1] + 1])
                    for key, value in NON_BASE_UNITS[unit][0].items():
                        units[key] -= value * exponent
                    multiplier /= NON_BASE_UNITS[unit][1] ** exponent
                else:
                    for key, value in NON_BASE_UNITS[unit][0].items():
                        units[key] -= value
                    multiplier /= NON_BASE_UNITS[unit][1]
        else:
            raise ValueError("Invalid unit: " + unit)

    # Strip zero units
    units = {key: value for key, value in units.items() if value != 0}

    return units, multiplier

def hr_vals_and_units(vals, units, sigfigs=3):
    hrvals, hrunits = hr_parts(vals, units, sigfigs)

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


def hr_units(units, vals=[0, 0], sigfigs=3):
    _, hrunits = hr_parts(vals, units, sigfigs)

    return hrunits[0]


def hr_parts(vals, units, sigfigs=3):  # TODO: add sigfigs
    vals = vals if vals[0] != vals[1] else [vals[0]]
    hrunits = []
    hrvals = []

    if units == {"m": 1, "s": -2}:
        for i, val in enumerate(vals):
            if abs(val) >= 5:
                hrvals.append(val / 9.81)
                hrunits.append("g")
            elif abs(val) >= 0.1:
                hrvals.append(val)
                hrunits.append("m/s^2")
            elif abs(val) >= 0.0001:
                hrvals.append(val * 1e3)
                hrunits.append("mm/s^2")
            elif abs(val) >= 0.0000001:
                hrvals.append(val * 1e6)
                hrunits.append("um/s^2")
            elif val != 0:
                hrvals.append(val * 1e9)
                hrunits.append("nm/s^2")
            else:
                hrvals.append(0)
                hrunits.append("m/s^2")
    elif units == {"m": 1}:
        unit = ""
        for val in vals:
            for k, v in DERIVED_UNITS.items():
                if units == v[0]:
                    if not unit:
                        unit = k
                    elif abs(val - v[1]) < abs(val - DERIVED_UNITS[unit][1]):
                        unit = k

            hrvals.append(val / DERIVED_UNITS[unit][1])
            hrunits.append(unit)
        # for i, val in enumerate(vals):
        #     if abs(val) >= 1e3:
        #         hrvals.append(val / 1e3)
        #         hrunits.append("km")
        #     elif abs(val) >= 0.5:
        #         hrvals.append(val)
        #         hrunits.append("m")
        #     elif abs(val) >= 0.01:
        #         hrvals.append(val * 100)
        #         hrunits.append("cm")
        #     elif abs(val) >= 0.0001:
        #         hrvals.append(val * 1e3)
        #         hrunits.append("mm")
        #     elif abs(val) >= 0.0000001:
        #         hrvals.append(val * 1e6)
        #         hrunits.append("um")
        #     elif val != 0:
        #         hrvals.append(val * 1e9)
        #         hrunits.append("nm")
        #     else:
        #         hrvals.append(0)
        #         hrunits.append("m")
    elif units == {"m": 2}:
        for i, val in enumerate(vals):
            if abs(val) >= 1e6:
                hrvals.append(val / 1e6)
                hrunits.append("km^2")
            elif abs(val) >= 1:
                hrvals.append(val)
                hrunits.append("m^2")
            elif abs(val) >= 0.0001:
                hrvals.append(val * 10000)
                hrunits.append("cm^2")
            elif abs(val) >= 0.00000001:
                hrvals.append(val * 1e6)
                hrunits.append("mm^2")
            elif abs(val) >= 0.00000000000001:
                hrvals.append(val * 1e12)
                hrunits.append("um^2")
            elif val != 0:
                hrvals.append(val * 1e18)
                hrunits.append("nm^2")
            else:
                hrvals.append(0)
                hrunits.append("m^2")
    elif units == {"m": 3}:
        for i, val in enumerate(vals):
            if abs(val) >= 1e8:
                hrvals.append(val / 1e9)
                hrunits.append("km^3")
            elif abs(val) >= 1:
                hrvals.append(val)
                hrunits.append("m^3")
            elif abs(val) >= 0.000001:
                hrvals.append(val * 1000000)
                hrunits.append("cm^3")
            elif abs(val) >= 0.000000000001:
                hrvals.append(val * 1e9)
                hrunits.append("mm^3")
            elif abs(val) >= 0.000000000000000000001:
                hrvals.append(val * 1e12)
                hrunits.append("um^3")
            elif val != 0:
                hrvals.append(val * 1e27)
                hrunits.append("nm^3")
            else:
                hrvals.append(0)
                hrunits.append("m^3")
    elif units == {"kg": 1}:
        for i, val in enumerate(vals):
            if abs(val) >= 0.5:
                hrvals.append(val)
                hrunits.append("kg")
            elif abs(val) >= 0.0001:
                hrvals.append(val * 1e3)
                hrunits.append("g")
            elif abs(val) >= 0.0000001:
                hrvals.append(val * 1e6)
                hrunits.append("mg")
            elif val != 0:
                hrvals.append(val * 1e9)
                hrunits.append("ug")
            else:
                hrvals.append(0)
                hrunits.append("kg")

    elif units == {"kg": 1, "m": 2, "s": -2}:
        for i, val in enumerate(vals):
            if abs(val) >= 1e15:
                hrvals.append(val / 1e15)
                hrunits.append("PJ")
            elif abs(val) >= 1e12:
                hrvals.append(val / 1e12)
                hrunits.append("TJ")
            elif abs(val) >= 1e9:
                hrvals.append(val / 1e9)
                hrunits.append("GJ")
            elif abs(val) >= 1e6:
                hrvals.append(val / 1e6)
                hrunits.append("MJ")
            elif abs(val) >= 1e3:
                hrvals.append(val / 1e3)
                hrunits.append("kJ")
            elif abs(val) >= 1:
                hrvals.append(val)
                hrunits.append("J")
            elif abs(val) >= 0.001:
                hrvals.append(val * 1e3)
                hrunits.append("mJ")
            elif abs(val) >= 0.000001:
                hrvals.append(val * 1e6)
                hrunits.append("uJ")
            elif abs(val) >= 0.000000001:
                hrvals.append(val * 1e9)
                hrunits.append("nJ")
            elif val != 0:
                hrvals.append(val * 1e12)
                hrunits.append("pJ")
            else:
                hrvals.append(0)
                hrunits.append("J")
    elif units == {"kg": 1, "m": 2, "s": -3, "A": -1}:
        for i, val in enumerate(vals):
            if abs(val) >= 1e15:
                hrvals.append(val / 1e15)
                hrunits.append("PV")
            elif abs(val) >= 1e12:
                hrvals.append(val / 1e12)
                hrunits.append("TV")
            elif abs(val) >= 1e9:
                hrvals.append(val / 1e9)
                hrunits.append("GV")
            elif abs(val) >= 1e6:
                hrvals.append(val / 1e6)
                hrunits.append("MV")
            elif abs(val) >= 1e3:
                hrvals.append(val / 1e3)
                hrunits.append("kV")
            elif abs(val) >= 1:
                hrvals.append(val)
                hrunits.append("V")
            elif abs(val) >= 0.001:
                hrvals.append(val * 1e3)
                hrunits.append("mV")
            elif abs(val) >= 0.000001:
                hrvals.append(val * 1e6)
                hrunits.append("uV")
            elif abs(val) >= 0.000000001:
                hrvals.append(val * 1e9)
                hrunits.append("nV")
            elif val != 0:
                hrvals.append(val * 1e12)
                hrunits.append("pV")
            else:
                hrvals.append(0)
                hrunits.append("V")
    elif units == {"kg": 1, "m": 2, "s": -3}:
        for i, val in enumerate(vals):
            if abs(val) >= 1e15:
                hrvals.append(val / 1e15)
                hrunits.append("PW")
            elif abs(val) >= 1e12:
                hrvals.append(val / 1e12)
                hrunits.append("TW")
            elif abs(val) >= 1e9:
                hrvals.append(val / 1e9)
                hrunits.append("GW")
            elif abs(val) >= 1e6:
                hrvals.append(val / 1e6)
                hrunits.append("MW")
            elif abs(val) >= 1e3:
                hrvals.append(val / 1e3)
                hrunits.append("kW")
            elif abs(val) >= 1:
                hrvals.append(val)
                hrunits.append("W")
            elif abs(val) >= 0.001:
                hrvals.append(val * 1e3)
                hrunits.append("mW")
            elif abs(val) >= 0.000001:
                hrvals.append(val * 1e6)
                hrunits.append("uW")
            elif abs(val) >= 0.000000001:
                hrvals.append(val * 1e9)
                hrunits.append("nW")
            elif val != 0:
                hrvals.append(val * 1e12)
                hrunits.append("pW")
            else:
                hrvals.append(0)
                hrunits.append("W")
    elif units == {"kg": 1, "s": -3}:
        for i, val in enumerate(vals):
            if abs(val) >= 1e15:
                hrvals.append(val / 1e15)
                hrunits.append("PW/m^2")
            elif abs(val) >= 1e12:
                hrvals.append(val / 1e12)
                hrunits.append("TW/m^2")
            elif abs(val) >= 1e9:
                hrvals.append(val / 1e9)
                hrunits.append("GW/m^2")
            elif abs(val) >= 1e6:
                hrvals.append(val / 1e6)
                hrunits.append("MW/m^2")
            elif abs(val) >= 1e3:
                hrvals.append(val / 1e3)
                hrunits.append("kW/m^2")
            elif abs(val) >= 1:
                hrvals.append(val)
                hrunits.append("W/m^2")
            elif abs(val) >= 0.001:
                hrvals.append(val * 1e3)
                hrunits.append("mW/m^2")
            elif abs(val) >= 0.000001:
                hrvals.append(val * 1e6)
                hrunits.append("uW/m^2")
            elif abs(val) >= 0.000000001:
                hrvals.append(val * 1e9)
                hrunits.append("nW/m^2")
            elif val != 0:
                hrvals.append(val * 1e12)
                hrunits.append("pW/m^2")
            else:
                hrvals.append(0)
                hrunits.append("W/m^2")
    elif units == {"A": 1}:
        for i, val in enumerate(vals):
            if abs(val) >= 1e15:
                hrvals.append(val / 1e15)
                hrunits.append("PA")
            elif abs(val) >= 1e12:
                hrvals.append(val / 1e12)
                hrunits.append("TA")
            elif abs(val) >= 1e9:
                hrvals.append(val / 1e9)
                hrunits.append("GA")
            elif abs(val) >= 1e6:
                hrvals.append(val / 1e6)
                hrunits.append("MA")
            elif abs(val) >= 1e3:
                hrvals.append(val / 1e3)
                hrunits.append("kA")
            elif abs(val) >= 1:
                hrvals.append(val)
                hrunits.append("A")
            elif abs(val) >= 0.001:
                hrvals.append(val * 1e3)
                hrunits.append("mA")
            elif abs(val) >= 0.000001:
                hrvals.append(val * 1e6)
                hrunits.append("uA")
            elif abs(val) >= 0.000000001:
                hrvals.append(val * 1e9)
                hrunits.append("nA")
            elif val != 0:
                hrvals.append(val * 1e12)
                hrunits.append("pA")
            else:
                hrvals.append(0)
                hrunits.append("A")
    elif units == {"b": 1}:
        for i, val in enumerate(vals):
            if abs(val) >= 8e15:
                hrvals.append(val / 8e15)
                hrunits.append("PB")
            elif abs(val) >= 8e12:
                hrvals.append(val / 8e12)
                hrunits.append("TB")
            elif abs(val) >= 8e9:
                hrvals.append(val / 8e9)
                hrunits.append("GB")
            elif abs(val) >= 8e6:
                hrvals.append(val / 8e6)
                hrunits.append("MB")
            elif abs(val) >= 8e3:
                hrvals.append(val / 8e3)
                hrunits.append("kB")
            elif abs(val) >= 8:
                hrvals.append(val / 8)
                hrunits.append("B")
            else:
                hrvals.append(val)
                hrunits.append("b")
    elif units == {"b": 1, "s": -1}:
        for i, val in enumerate(vals):
            if abs(val) >= 1e15:
                hrvals.append(val / 1e15)
                hrunits.append("Pbps")
            elif abs(val) >= 1e12:
                hrvals.append(val / 1e12)
                hrunits.append("Tbps")
            elif abs(val) >= 1e9:
                hrvals.append(val / 1e9)
                hrunits.append("Gbps")
            elif abs(val) >= 1e6:
                hrvals.append(val / 1e6)
                hrunits.append("Mbps")
            elif abs(val) >= 1e3:
                hrvals.append(val / 1e3)
                hrunits.append("kbps")
            else:
                hrvals.append(val)
                hrunits.append("bps")
    elif units == {"s": 1}:
        for i, val in enumerate(vals):
            if abs(val) >= 3.1556926e10:
                hrvals.append(val / 3.1556926e10)
                hrunits.append("millenia")
            elif abs(val) >= 3.1556926e9:
                hrvals.append(val / 3.1556926e9)
                hrunits.append("centuries")
            elif abs(val) >= 3.1556926e8:
                hrvals.append(val / 3.1556926e8)
                hrunits.append("decades")
            elif abs(val) >= 3.1556926e7:
                hrvals.append(val / 3.1556926e7)
                hrunits.append("years")
            elif abs(val) >= 2629743.83:
                hrvals.append(val / 2629743.83)
                hrunits.append("months")
            elif abs(val) >= 604800:
                hrvals.append(val / 604800)
                hrunits.append("weeks")
            elif abs(val) >= 86400:
                hrvals.append(val / 86400)
                hrunits.append("days")
            elif abs(val) >= 7200:
                hrvals.append(val / 3600)
                hrunits.append("hours")
            elif abs(val) >= 60:
                hrvals.append(val / 60)
                hrunits.append("mins")
            elif abs(val) >= 1:
                hrvals.append(val)
                hrunits.append("s")
            elif abs(val) >= 0.001:
                hrvals.append(val * 1e3)
                hrunits.append("ms")
            elif abs(val) >= 0.000001:
                hrvals.append(val * 1e6)
                hrunits.append("us")
            elif abs(val) >= 0.000000001:
                hrvals.append(val * 1e9)
                hrunits.append("ns")
            elif val != 0:
                hrvals.append(val * 1e12)
                hrunits.append("ps")
            else:
                hrvals.append(0)
                hrunits.append("s")
    elif units == {"s": -1}:
        for i, val in enumerate(vals):
            if abs(val) / (2 * np.pi) >= 1e9:
                hrvals.append(val / (2 * np.pi * 1e9))
                hrunits.append("GHz")
            elif abs(val) / (2 * np.pi) >= 1e6:
                hrvals.append(val / (2 * np.pi * 1e6))
                hrunits.append("MHz")
            elif abs(val) / (2 * np.pi) >= 1e3:
                hrvals.append(val / (2 * np.pi * 1e3))
                hrunits.append("kHz")
            elif abs(val) / (2 * np.pi) >= 1:
                hrvals.append(val / (2 * np.pi))
                hrunits.append("Hz")
            # rad/s * (360 deg / 2pi rad) = deg/s
            elif abs(val) * 180 / np.pi >= 1:
                hrvals.append(val * 180 / np.pi)
                hrunits.append("°/s")
            # rad/s * (1 rotation/ 2 * pi rad) = rotations/s * (60 s / 1 min) = rpm
            elif abs(val) * 30 / np.pi >= 1:
                hrvals.append(val * 15 / np.pi)
                hrunits.append("rpm")
            # rad/s * (180 deg / pi rad) = deg/s * (60 s / 1 min) = deg/min
            elif val != 0:
                hrvals.append(val * (180 / np.pi) * 60)
                hrunits.append("°/min")
            else:
                hrvals.append(0)
                hrunits.append("rad/s")
    elif units == {"$": 1} or units == {"$": -1}:
        for i, val in enumerate(vals):
            if abs(val) >= 1e12:
                hrvals.append(val / 1e12)
                hrunits.append("$T")
            elif abs(val) >= 1e9:
                hrvals.append(val / 1e9)
                hrunits.append("$B")
            elif abs(val) >= 1e6:
                hrvals.append(val / 1e6)
                hrunits.append("$M")
            elif abs(val) >= 1e3:
                hrvals.append(val / 1e3)
                hrunits.append("$k")
            else:
                hrvals.append(val / 1e12)
    elif units == {"kg": 1, "A": -1, "s": -2}:
        for i, val in enumerate(vals):
            if abs(val) >= 1e12:
                hrvals.append(val / 1e12)
                hrunits.append("TT")
            elif abs(val) >= 1e9:
                hrvals.append(val / 1e9)
                hrunits.append("GT")
            elif abs(val) >= 1e6:
                hrvals.append(val / 1e6)
                hrunits.append("MT")
            elif abs(val) >= 1e3:
                hrvals.append(val / 1e3)
                hrunits.append("kT")
            elif abs(val) >= 1:
                hrvals.append(val)
            elif abs(val) >= 1e-3:
                hrvals.append(val * 1e3)
                hrunits.append("mT")
            elif abs(val) >= 1e-6:
                hrvals.append(val * 1e6)
                hrunits.append("µT")
            elif abs(val) >= 1e-9:
                hrvals.append(val * 1e9)
                hrunits.append("nT")
            else:
                hrvals.append(val * 1e12)
                hrunits.append("pT")
    else:  # Undefined units
        hrvals = vals
        unitstr = build_compound_unit_str(units)
        hrunits = [unitstr.strip()] * len(vals)

    return hrvals, hrunits


def build_compound_unit_str(units):
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
    print(parse_compound_units("weeks"))