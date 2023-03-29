import re
import numpy as np

BASE_UNITS = ["kg", "m", "s", "K", "A", "b", "$", "capacities"] # kilograms, meters, seconds, Kelvins, Amps, bits, dollars

UNIT_OPERATORS = ["*", "/", "^"]

DERIVED_UNITS = {"V": (1, {'kg': 1, 'm': 2, 's': -3, 'A': -1}),
                 "W": (1, {'kg': 1, 'm': 2, 's': -3}),
                 "minutes": (60, {"s": 1}),
                 "hours": (3600, {"s": 1}),
                 "days": (86400, {"s": 1}),
                 "weeks": (604800, {"s": 1}),
                 "months": (2629746, {"s": 1}),
                 "years": (31556952, {"s": 1}),
                 "decades": (315569520, {"s": 1}),
                 "centuries": (3155695200, {"s": 1}),
                 "millenniums": (31556952000, {"s": 1}),
                 "Hz": (2*np.pi, {"s": -1}),
                 "kHz": (2*np.pi*1e3, {"s": -1}),
                 "MHz": (2*np.pi*1e6, {"s": -1}),
                 "GHz": (2*np.pi*1e9, {"s": -1}),
                 "hr": (3600, {"s": 1}),
                 "min": (60, {"s": 1}),
                 "rotations": (1, {}),
                 "revolutions": (1, {}),
                 "cycles": (1, {}),
                 "yr": (31556952, {"s": 1}),
                 "mm": (1e-3, {"m": 1}),
                 "cm": (1e-2, {"m": 1}),
                 "km": (1e3, {"m": 1}),
                 "um": (1e-6, {"m": 1}),
                 "nm": (1e-9, {"m": 1}),
                 "g": (1e-3, {"kg": 1}),
                 "mg": (1e-6, {"kg": 1}),
                 "ug": (1e-9, {"kg": 1}),
                 "J": (1, {"kg": 1, "m": 2, "s": -2}),
                 "kJ": (1e3, {"kg": 1, "m": 2, "s": -2}),
                 "MJ": (1e6, {"kg": 1, "m": 2, "s": -2}),
                 "mJ": (1e-3, {"kg": 1, "m": 2, "s": -2}),
                 "kV": (1e3, {"kg": 1, "m": 2, "s": -3, "A": -1}),
                 "MV": (1e6, {"kg": 1, "m": 2, "s": -3, "A": -1}),
                 "mV": (1e-3, {"kg": 1, "m": 2, "s": -3, "A": -1}),
                 "kW": (1e3, {"kg": 1, "m": 2, "s": -3}),
                 "MW": (1e6, {"kg": 1, "m": 2, "s": -3}),
                 "mW": (1e-3, {"kg": 1, "m": 2, "s": -3}),
                 "uW": (1e-6, {"kg": 1, "m": 2, "s": -3}),
                 "nW": (1e-9, {"kg": 1, "m": 2, "s": -3}),
                 "pW": (1e-12, {"kg": 1, "m": 2, "s": -3}),
                 "kWh": (3.6e6, {"kg": 1, "m": 2, "s": -2}),
                 "MWh": (3.6e9, {"kg": 1, "m": 2, "s": -2}),
                 "mWh": (3.6, {"kg": 1, "m": 2, "s": -2}),
                 "Wh": (3.6e3, {"kg": 1, "m": 2, "s": -2}),
                 "mA": (1e-3, {"A": 1}),
                 "uA": (1e-6, {"A": 1}),
                 "kA": (1e3, {"A": 1}),
                 "Ah": (3600, {"A": 1, "s": 1}),
                 "mAh": (3.6, {"A": 1, "s": 1}),
                 "B": (8, {"b": 1}),
                 "kB": (8e3, {"b": 1}),
                 "MB": (8e6, {"b": 1}),
                 "GB": (8e9, {"b": 1}),
                 "TB": (8e12, {"b": 1}),
                 "PB": (8e15, {"b": 1}),
                 "bps": (1, {"b": 1, "s": -1}),
                 "kbps": (1e3, {"b": 1, "s": -1}),
                 "Mbps": (1e6, {"b": 1, "s": -1}),
                 "Gbps": (1e9, {"b": 1, "s": -1}),
                 "deg": (np.pi/180, {}),
                 "rad": (1, {}),
                 "radians": (1, {}),
                 "degrees": (np.pi/180, {}),
                 "°/s": (np.pi/180, {"s": -1}),
                 "°/min": (60*np.pi/180, {"s": -1}),
                 "°/hr": (3600*np.pi/180, {"s": -1}),
                 "deg/s": (np.pi/180, {"s": -1}),
                 "deg/min": (60*np.pi/180, {"s": -1}),
                 "deg/hr": (3600*np.pi/180, {"s": -1}),
                 "rpm": (2*np.pi/60, {"s": -1}),
                 "k$": (1e3, {"$": 1}),
                 "M$": (1e6, {"$": 1}),
                 "B$": (1e9, {"$": 1}),
                 "T$": (1e12, {"$": 1}),
                 "%": (1e-2, {}),
                 "ms": (1e-3, {"s": 1}),
                 "us": (1e-6, {"s": 1}),
                 "ns": (1e-9, {"s": 1}),
                 "ps": (1e-12, {"s": 1}),
                 "T": (1, {"kg": 1, "s": -2, "A": -1}),
                 "mT": (1e-3, {"kg": 1, "s": -2, "A": -1}),
                 "uT": (1e-6, {"kg": 1, "s": -2, "A": -1}),
}
                 
UNIT_PREFIXES = {"y": 1e-24,
                 "z": 1e-21,
                 "a": 1e-18,
                 "f": 1e-15,
                 "p": 1e-12,
                 "n": 1e-9,
                 "u": 1e-6,
                 "m": 1e-3,
                 "c": 1e-2,
                 "d": 1e-1,
                 "da": 1e1,
                 "h": 1e2,
                 "k": 1e3,
                 "M": 1e6,
                 "G": 1e9,
                 "T": 10*12,
                 "P": 1e15,
                 "E": 1e18,
                 "Z": 1e21,
                 "Y": 1e24}

def _round(num, n=3):
    formatstr = "%." + str(n) + "g"
    return float(formatstr % num)

def parse(unit_str):
    if unit_str in BASE_UNITS:
        units = {unit_str: 1}
        multiplier = 1
    elif unit_str in DERIVED_UNITS:
        units = DERIVED_UNITS[unit_str][1]
        multiplier = DERIVED_UNITS[unit_str][0]
    else:
        units, multiplier = parse_compound_units(unit_str)

    return units, multiplier

def parse_compound_units(unit_str):
    # Parse the unit string based on operators /, *, ^
    unit_list = [x for x in re.findall("[A-Za-z]+", unit_str) if x not in UNIT_OPERATORS]

    # Find the indices of the above matches
    indices = [m.span() for m in re.finditer("[A-Za-z]+", unit_str)]
    
    # Initialize zero unit
    units = {unit: 0 for unit in BASE_UNITS}

    multiplier = 1

    # Iterate through the indices and unit_list together
    for index, unit in zip(indices, unit_list):
        if unit in BASE_UNITS:
            if index[0] == 0 or unit_str[index[0]-1] == "*":
                if index[1] < len(unit_str) and unit_str[index[1]] == "^":
                    units[unit] += int(unit_str[index[1]+1])
                else:
                    units[unit] += 1

            elif unit_str[index[0]-1] == "/":
                if index[1] < len(unit_str) and unit_str[index[1]] == "^":
                    units[unit] -= int(unit_str[index[1]+1])
                else:
                    units[unit] -= 1
            multiplier *= 1
        elif unit in DERIVED_UNITS:
            if index[0] == 0 or unit_str[index[0]-1] == "*":
                if index[1] < len(unit_str) and unit_str[index[1]] == "^":
                    exponent = int(unit_str[index[1]+1])
                    for key, value in DERIVED_UNITS[unit][1].items():
                        units[key] += value * exponent
                        multiplier *= DERIVED_UNITS[unit][0]**exponent
                else:
                    for key, value in DERIVED_UNITS[unit][1].items():
                        units[key] += value
                        multiplier *= DERIVED_UNITS[unit][0]
            elif unit_str[index[0]-1] == "/":
                if index[1] < len(unit_str) and unit_str[index[1]] == "^":
                    exponent = int(unit_str[index[1]+1])
                    for key, value in DERIVED_UNITS[unit][1].items():
                        units[key] += value * exponent
                        multiplier /= DERIVED_UNITS[unit][0]**exponent
                else:
                    for key, value in DERIVED_UNITS[unit][1].items():
                        units[key] -= value
                        multiplier /= DERIVED_UNITS[unit][0]
        else:
            raise ValueError("Invalid unit: " + unit)

    # Strip zero units
    units = {key: value for key, value in units.items() if value != 0}

    return units, multiplier

def base_units(units):
        undefined_unit = False

        unitstr = ""
        if units == {"m": 1, "s": -2}:
            unitstr = " m/s^2"
        if units == {"m": 1, "s": -1}:
            unitstr = " m/s"
        elif units == {"m": 1}:
            unitstr = " m"
        elif units == {"m": 2}:
            unitstr = " m^2"
        elif units == {"m": 3}:
            unitstr = " m^3"
        elif units == {"kg": 1}:
            unitstr = " kg"
        elif units == {"kg": 1, "m": 2, "s": -2}:
            unitstr = " J"
        elif units == {"kg": 1, "m": 2, "s": -3, "A": -1}:
            unitstr = " V"
        elif units == {"kg": 1, "m": 2, "s": -3}:
            unitstr = " W"
        elif units == {"kg": 1, "s": -3}:
            unitstr = " W/m^2"
        elif units == {"A": 1}:
            unitstr = " A"
        elif units == {"b": 1}:
            unitstr = " B"
        elif units == {"b": 1, "s": -1}:
            unitstr = " bps"
        elif units == {"s": 1}:
            unitstr = " s"
        elif units == {"s": -1}:
            unitstr = " rad/s"
        elif units == {"$": 1} or units == {"$": -1}:
            unitstr = "$"
        elif not units:
            pass
        else:
            undefined_unit = True

        if undefined_unit:
            nums = []
            dens = []
            if units:
                for unit, exp in units.items():
                    if exp > 0:
                        nums.append(unit)
                    elif exp < 0:
                        dens.append(unit)
                    else:
                        raise ValueError("Unit with zero exponent should have been purged before display.")
                if len(nums) > 0:
                    if len(nums) > 1 and len(dens) > 0:
                        unitstr += "("
                    for i, num in enumerate(nums):
                        if i > 0:
                            unitstr += " "
                        unitstr += num
                        if units[num] != 1:
                            unitstr += "^" + str(units[num])
                    if len(nums) > 1 and len(dens) > 0:
                        unitstr += ")"
                else:
                    unitstr += "1"
                if len(dens) > 0:
                    unitstr += "/"
                    if len(dens) > 1:
                        unitstr += "("
                    for i, den in enumerate(dens):
                        if i > 0:
                            unitstr += " "
                        unitstr += den
                        if units != 1:
                            unitstr += "^" + str(-units[den])
                    if len(dens) > 1:
                        unitstr += ")"

        return unitstr

def hr_vals_and_units(vals, units, sigfigs=3):
    hrvals, hrunits = hr_parts(vals, units, sigfigs)

    hrstr = str(_round(hrvals[0], sigfigs))

    if len(hrvals) > 1 and hrvals[0] != hrvals[1]:
        if hrunits[0] != hrunits[1]:
            hrstr += " " + hrunits[0] + " | " + str(_round(hrvals[1], sigfigs)) + " " + hrunits[1]
        else:
            hrstr += "|" + str(_round(hrvals[1], sigfigs)) + " " + hrunits[0]
    else:
        hrstr += " " + hrunits[0]

    return hrstr

def hr_units(units, vals=[0, 0], sigfigs=3):
    _, hrunits = hr_parts(vals, units, sigfigs)

    return hrunits[0]

def hr_parts(vals, units, sigfigs=3):
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
        for i, val in enumerate(vals):
            if abs(val) >= 1e3:
                hrvals.append(val / 1e3)
                hrunits.append("km")
            elif abs(val) >= 0.5:
                hrvals.append(val)
                hrunits.append("m")
            elif abs(val) >= 0.01:
                hrvals.append(val * 100)
                hrunits.append("cm")
            elif abs(val) >= 0.0001:
                hrvals.append(val * 1e3)
                hrunits.append("mm")
            elif abs(val) >= 0.0000001:
                hrvals.append(val * 1e6)
                hrunits.append("um")
            elif val != 0:
                hrvals.append(val * 1e9)
                hrunits.append("nm")
            else:
                hrvals.append(0)
                hrunits.append("m")
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
            if abs(val)/(2*np.pi) >= 1e9:
                hrvals.append(val/(2*np.pi*1e9))
                hrunits.append("GHz")
            elif abs(val)/(2*np.pi) >= 1e6:
                hrvals.append(val/(2*np.pi*1e6))
                hrunits.append("MHz")
            elif abs(val)/(2*np.pi) >= 1e3:
                hrvals.append(val/(2*np.pi*1e3))
                hrunits.append("kHz")
            elif abs(val)/(2*np.pi) >= 1:
                hrvals.append(val/(2*np.pi))
                hrunits.append("Hz")
            # rad/s * (360 deg / 2pi rad) = deg/s
            elif abs(val)*180/np.pi >= 1:
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
    else:
        # Undefined units
        hrvals = vals
        unitstr = " "
        nums = []
        dens = []
        if units:
            for unit, exp in units.items():
                if exp > 0:
                    nums.append(unit)
                elif exp < 0:
                    dens.append(unit)
                else:
                    raise ValueError("Unit with zero exponent should have been purged before display.")
            if len(nums) > 0:
                if len(nums) > 1 and len(dens) > 0:
                    unitstr += "("
                for i, num in enumerate(nums):
                    if i > 0:
                        unitstr += " "
                    unitstr += num
                    if units[num] != 1:
                        unitstr += "^" + str(units[num])
                if len(nums) > 1 and len(dens) > 0:
                    unitstr += ")"
            else:
                unitstr += "1"
            if len(dens) > 0:
                unitstr += "/"
                if len(dens) > 1:
                    unitstr += "("
                for i, den in enumerate(dens):
                    if i > 0:
                        unitstr += " "
                    unitstr += den
                    if units != 1:
                        unitstr += "^" + str(-units[den])
                if len(dens) > 1:
                    unitstr += ")"
        hrunits = [unitstr.strip()] * len(vals)
                    
    return hrvals, hrunits

