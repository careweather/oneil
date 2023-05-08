import re
import numpy as np
import inspect
from pkg_resources import get_distribution
from pathlib import Path
from pytexit import py2tex
import os
import copy
from beautifultable import BeautifulTable
import units as un
import importlib
import sys
from functools import partial

class bcolors:
    HEADER = '\033[95m'
    OKBLUE = '\033[94m'
    OKCYAN = '\033[96m'
    OKGREEN = '\033[92m'
    WARNING = '\033[93m'
    FAIL = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'

def isfloat(num):
    try:
        float(num)
        return True
    except ValueError:
        return False

__version__ = get_distribution("oneil").version

FUNCTIONS = {"sin": "par_sin", "cos": "par_cos", "tan": "par_tan", "asin": "par_asin", "acos": "par_acos", "atan": "par_atan", "sinh": "par_arcsinh", "cosh": "par_cosh", "tanh": "par_tanh", "min": "par_min", "max": "par_max", "sqrt": "par_sqrt", "abs": "par_abs", "mnmx": "par_minmax", "log": "par_log", "log10": "par_log10", "floor": "par_floor"}

MATH_CONSTANTS = {"pi": np.pi, "e": np.exp(1), "inf": np.inf}

EQUATION_OPERATORS = ["+", "-", "*", "/", "**", "//", "%", "(", ")", "=", "<", ">", "!"]

OPERATOR_OVERRIDES = {"--": "|minus|"}

class Infix():
    def __init__(self, func):
        self.func = func
    def __or__(self, other):
        return self.func(other)
    def __ror__(self, other):
        return Infix(partial(self.func, other))
    def __call__(self, v1, v2):
        return self.func(v1, v2)
    
@Infix
def minus(x, y):
    if isinstance(x, Parameter):
        return x._minus(y)
    elif isinstance(y, Parameter):
        return y._rminus(x)
    else:
        return x - y
    

def parse_file(file_name, parent_model=None):
    parameters = []
    submodels = {}
    imports = []
    tests = []
    note = ""
    prev_line = ''
    design_overrides = {}
    last_line_blank = False
    section = ""
    multiplier = 0

    with open(file_name, 'r') as f:
        final_line = 0
        for i, line in enumerate(f.readlines()):
            final_line = i
            if line == '\n':
                multiplier = 0
                last_line_blank = True
                continue
            elif '#' in line and line.strip()[0] == '#':
                multiplier = 0
                last_line_blank = False
                continue
            elif line[0] == '\t' or line[0:1] == ' ':
                if last_line_blank: line = "\n\n" + line
                if line.strip() and line.strip()[0] == '{':
                    arguments = []
                    assert multiplier != 0, ValueError("Multiplier should never be zero.")
                    parameter, arguments = parse_piecewise(line, parameters[-1].units, parameters[-1].id, imports, file_name.replace(".on", ""), i+1, multiplier)
                    parameters[-1].add_piece(parameter, arguments)
                else:
                    multiplier = 0
                    if prev_line == 'param':
                        parameters[-1].notes.append(line.replace("\t", "", 1).replace(" "*4, "", 1))
                        parameters[-1].note_lines.append(i+1)
                    elif prev_line == 'test':
                        tests[-1].notes.append(line.replace("\t", "", 1).replace(" "*4, "", 1))
                        tests[-1].note_line_nos.append(i+1)
                    elif prev_line == 'design':
                        last_key = list(design_overrides.keys())[-1]
                        design_overrides[last_key].notes.append(line.replace("\t", "", 1).replace(" "*4, "", 1))
                        design_overrides[last_key].note_lines.append(i+1)
                    elif prev_line == '':
                        note += line.strip()
                    else:
                        raise ValueError("Invalid prev line type: " + line)
                last_line_blank = False

            elif line[:4] == 'use ':
                multiplier = 0
                try:
                    assert(re.search(r"^use\s+\w+(\(.+=.+\))?\s+as\s+\w+\s*$", line))
                except:
                    SyntaxError(file_name, i+1, line, "Use includes must be of the form \"use <model> as <symbol>\"")
                
                last_line_blank = False
                include = line.replace("use", "")
                model = include.split('as')[0].strip()

                if '(' in model:
                    test_inputs = {l.split('=')[0].strip():l.split('=')[1].strip() for l in model.split('(')[1].split(')')[0].split(',')}
                    model = model.split('(')[0].strip()
                else:
                    test_inputs = []

                if not os.path.exists(model + ".on"):
                    ModelError(file_name, "", ["parse_file"]).throw(parent_model, "(line " + str(i+1) + ") " + line + "- " + "File \"" + model + ".on\" does not exist.")
                symbol = include.split('as')[1].strip()

                if symbol in submodels.keys():
                    ModelError(file_name, "", ["parse_file"]).throw(parent_model, "(line " + str(i+1) + ") " + line + "- " + "Submodel symbol \"" + symbol + "\" has duplicate definitions.")

                submodels[symbol] = {'model': Model(model + ".on"), 'inputs': test_inputs, 'path': [model], 'line_no': i+1, 'line': line}
            elif line[:5] == 'from ':
                multiplier = 0
                try:
                    assert(re.search(r"^from\s+\w+(\.\w+)*\s+use\s+\w+(\(.+=.+\))?\s+as\s+\w+\s*$", line))
                except:
                    SyntaxError(file_name, i+1, line, "From includes must be of the form \"from <source> use <model> as <symbol>\"")

                last_line_blank = False
                include = line.replace("from", "")
                source = include.split('use')[0].strip()
                model = include.split('use')[1].split("as")[0].strip()

                if '(' in model:
                    test_inputs = {l.split('=')[0].strip():l.split('=')[1].strip() for l in model.split('(')[1].split(')')[0].split(',')}
                    model = model.split('(')[0].strip()
                else:
                    test_inputs = []

                if not os.path.exists(model + ".on"):
                    ModelError(file_name, "", ["parse_file"]).throw(parent_model, "(line " + str(i+1) + ") " + line + "- " + "File \"" + model + ".on\" does not exist.")

                path = source.split('.') + [model] if '.' in source else [source, model]
                symbol = include.split('use')[1].split("as")[1].strip()

                if symbol in submodels.keys():
                    ModelError(file_name, "", ["parse_file"]).throw(parent_model, "(line " + str(i+1) + ") " + line + "- " + "Submodel symbol \"" + symbol + "\" has duplicate definitions.")

                submodels[symbol] = {'path': path, 'inputs': test_inputs, 'line_no': i+1, 'line': line}
            elif line[:7] == 'import ':
                multiplier = 0
                try:
                    assert(re.search(r"^import\s+\w+\s*$", line))
                except:
                    SyntaxError(file_name, i+1, line, "Python imports must be of the form \"import <module>\"")
                
                last_line_blank = False
                sys.path.append(os.getcwd())
                module = line.replace("import", "").strip()

                try:
                    imports.append(importlib.import_module(module))
                except:
                    ImportError(parent_model, file_name, i+1, line, module + ".py")

            elif line[:8] == 'section ':
                multiplier = 0
                try:
                    assert(re.search(r"^section\s+[\w\s]*$", line))
                except:
                    SyntaxError(file_name, i+1, line, "Sections must be of the form \"section <name>\" where <name> is only word characters and whitespace.")
                
                last_line_blank = False
                section = line.replace("section", "").strip()
            elif line[0:4] == 'test' or line.replace(" ", "").replace("\t", "")[0:5] == '*test':
                multiplier = 0
                try:
                    assert(re.search(r"^(\*{1,2}\s*)?test\s*(\{\w+(,\s*\w+)*\})?:.*$", line))
                except:
                    SyntaxError(file_name, i+1, line, "Tests must be of the form \"test {<input 1>, <input 2>, ... ,<input n>}: <expression>\" where {<input 1>, <input 2>, ... ,<input n>} is optional, each <input> consists of word characters only, and <expression> is a valid python expression with valid parameters and constants.")
                
                last_line_blank = False
                tests.append(Test(line, i+1, file_name.replace(".on", ""), section=section))
                prev_line = 'test'
            elif re.search(r"^(\*{1,2}\s*)?\w+(\.\w+)?\s*=[^:]+(:.*)?$", line):
                multiplier = 0
                last_line_blank = False
                value_ID = line.split('=')[0].strip()
                design_overrides[value_ID] = parse_value(line, i+1, file_name.replace(".on", ""), section)
                prev_line='design'
            elif re.search(r"^[^\s]+[^:]*:\s*\w+\s*=[^:]+(:.*)?$", line):
                multiplier = 0
                last_line_blank = False
                parameter, multiplier = parse_parameter(line, i+1, file_name.replace(".on", ""), imports, section)
                parameters.append(parameter)
                prev_line = 'param'
            else:
                SyntaxError(parent_model, file_name, i+1, line, "Invalid syntax.")

    params = {p.id: p for p in parameters}

    if not params and not tests and not design_overrides:
        ModelError(file_name, "", ["parse_file"]).throw(None, "(final line) " + final_line + "- " + "End of File\n", "Empty model. No parameters, design values, or tests found.")

    return note, params, submodels, tests, design_overrides

def parse_parameter(line, line_number, file_name, imports, section=""):
    trace = ''

    if line[0] == '$':
        performance = True
        line = line[1:].strip()
    else:
        performance = False

    if line[0] == '*':
        if line[1] == '*':
            import pdb;
            breakpoint
        trace = 'calc'
        line = line[1:].strip()

    preamble = line.split(':')[0]
    body = line.split(':')[1]
    if len(line.split(':')) > 2:
        try:
            units, multiplier = un.parse(line.split(':')[2].strip("\n").strip())
        except:
            UnitError([], "", ["parse_parameter"]).throw(file_name, "(line " + str(line_number) + ") " + line + "- " + "Failed to parse units: " + line.split(':')[2].strip("\n"))
    else: 
        units = {}
        multiplier = 1

    # Parse the preamble
    if '(' and ')' in preamble:
        name = preamble.split('(')[0].strip()
        limits = []
        for l in preamble.replace(" ", "").split('(')[1].split(')')[0].split(','):
            if l.replace('.','').isnumeric():
                limits.append(float(l))
            elif l in MATH_CONSTANTS:
                limits.append(MATH_CONSTANTS[l])
            elif any(character in EQUATION_OPERATORS for character in l):
                limits.append(eval(l, MATH_CONSTANTS))
            else:
                SyntaxError(None, file_name, line_number, line, "Parse parameter: invalid limit: " + l)
        options = tuple(limits)
    elif '[' and ']' in preamble:
        name = preamble.split('[')[0].strip()
        options = preamble.replace(" ", "").split('[')[1].split(']')[0].split(',')
    else:
        name = preamble
        options = (0, np.inf)

    if not name:
        SyntaxError(None, file_name, line_number, line, "Parse parameter: name cannot be empty.")

    # Parse the body
    id = body.split('=')[0].strip()
    assignment = "=".join(body.split('=')[1:]).strip()

    if assignment.strip()[0] == '{':
        equation, arguments = parse_piecewise(assignment, units, id, imports, file_name, line_number, multiplier)
        equation = [equation]
    else:
        equation, arguments = parse_equation(assignment.replace(' ', ''), units, id, imports, file_name, line_number, multiplier)

    return Parameter(equation, units, id, model=file_name, line_no=line_number, line=line, name=name, options=options, arguments=arguments, trace=trace, section=section, performance=performance), multiplier

def parse_piecewise(assignment, units, id, imports, file_name, line_number, multiplier):
    eargs = []
    cargs = []
    equation = ""
    condition = ""
    assignment = assignment.strip().strip('{')
    equation, eargs = parse_equation(assignment.split('if')[0].strip(), units, id, imports, file_name, line_number, multiplier)
    condition, cargs = parse_equation(assignment.split('if')[1].strip(), units, id, imports, file_name, line_number, multiplier)
    return (Parameter(equation, units, id + ":eqpiece"), Parameter(condition, {}, id + ":condpiece")), eargs + cargs

def parse_equation(assignment, units, id, imports, file_name, line_number, multiplier):
    arguments = []
    mathless_assignment = assignment
    
    if any(op in mathless_assignment for op in MATH_CONSTANTS):
        for x in MATH_CONSTANTS:
            mathless_assignment = mathless_assignment.replace(x, '')

    if re.search('[a-zA-Z]', mathless_assignment):
        if '|' in assignment:
            mineq, minargs = convert_functions(assignment.split('|')[0], imports, file_name, line_number) 
            maxeq, maxargs = convert_functions(assignment.split('|')[1], imports, file_name, line_number)
            equation = (Parameter(mineq, units, id + ":mineq"), Parameter(maxeq, units, id + ":maxeq"))
            arguments = minargs + maxargs
        else:
            equation, arguments = convert_functions(assignment, imports, file_name, line_number)
        
    else:
        if '|' in assignment:
            min = multiplier*eval((assignment.split('|')[0]), MATH_CONSTANTS)
            max = multiplier*eval((assignment.split('|')[1]), MATH_CONSTANTS)
            equation = (min, max)
        else:
            equation = multiplier*eval(assignment, MATH_CONSTANTS)


    return equation, arguments

def convert_functions(assignment, imports, file_name, line_number):
    arguments = []
    if isfloat(assignment):
        return float(assignment), arguments
    # If assignment has a function ("name(var1, var2, varn)") in it, replace it with the appropriate callable
    if re.search('\w+\(', assignment) and not any(func + '(' in assignment for func in FUNCTIONS):
        equation = None
        func = assignment.strip('(').split('(')[0]
        for i in imports:
            if func in i.__dict__.keys():
                equation = i.__dict__[func]
                arguments = assignment.split('(')[1].split(')')[0].split(',')
                break
        if not equation:
            SyntaxError(None, file_name, line_number, assignment, "Parse parameter: invalid function: " + func)

    else:
        equation = assignment.strip("\n").strip()

    return equation, arguments

def parse_value(line, line_no, file_name, section=""):
    value_ID = line.split('=')[0].strip()

    if ':' in line:
        try:
            value_units, multiplier = un.parse(line.split(':')[1].strip())
        except:
            UnitError([], "", ["parse_value"]).throw(None, "in " + file_name + " (line " + str(line_no) + ") " + line + "- " + "Failed to parse units: " + line.split(':')[1].strip("\n"))
        value_assignment = line.split('=')[1].split(':')[0].strip()
    else:
        value_units = {}
        multiplier = 1
        value_assignment = line.split('=')[1].strip()

    if '|' in value_assignment:
        min = multiplier*eval(value_assignment.split('|')[0], MATH_CONSTANTS)
        max = multiplier*eval(value_assignment.split('|')[1], MATH_CONSTANTS)
        equation = (min, max)
    elif value_assignment in MATH_CONSTANTS or value_assignment.replace(".","").replace("-","").isnumeric():
        equation = multiplier*eval(value_assignment, MATH_CONSTANTS)
    else: 
        equation = value_assignment

    return Parameter(equation, value_units, value_ID, model=file_name.replace(".on", ""), line_no=line_no, line=line, name=value_ID + " from " + file_name, section=section)


def _process_minmax_par_inputs(val1, val2):
    if pass_errors(val1, val2): return pass_errors(val1, val2, caller="process_minmax_par_inputs")

    if isinstance(val2, Parameter) and not isinstance(val1, Parameter):
        tempval = val1
        val1 = val2.copy()
        val2 = tempval

    return val1, val2


def par_minmax(val1, val2):
    if pass_errors(val1, val2): return pass_errors(val1, val2, caller="par_minmax")

    if val1.units != val2.units:
        return Parameter((np.nan, np.nan), val1.units, "minmax(({}), ({}))".format(val1.id, val2.id), error=UnitError([val1, val2], "Cannot compare " + un.hr_units(val1.units) + " to " + un.hr_units(val2.units) + ".", ["par_minmax"]))
    return Parameter((min(val1.min, val2.min), max(val1.max, val2.max)), val1.units, "Min/max({},{})".format(val1.name, val2.name))


def par_min(val1, val2=None):
    if pass_errors(val1, val2): return pass_errors(val1, val2, caller="par_min")

    if not val2:
        if isinstance(val1, Parameter):
            if val1.units == {}:
                return val1.min
            else:
             return Parameter((val1.min, val1.min), val1.units, f"min({val1.name})")
        elif isinstance(val1, (int, float)):
            return val1
        
    val1, val2 = _process_minmax_par_inputs(val1, val2)

    if isinstance(val2, Parameter):
        if val1.id == val2.id:
            return Parameter((val1.min, val1.min), val1.units, "min({})".format(val1.name))
        
        if val1.units != val2.units:
            return Parameter((np.nan, np.nan), val1.units, "min(({}), ({}))".format(val1.id, val2.id), error=UnitError([val1, val2], "Cannot compare " + un.hr_units(val1.units) + " to " + un.hr_units(val2.units) + ".", ["par_min"]))

        return Parameter((min(val1.min, val2.min), min(val1.max, val2.max)), val1.units, "min({},{})".format(val1.name, val2.name))
    elif isinstance(val2, (int, float)):
        if val1.units != {}:
            return Parameter((np.nan, np.nan), val1.units, "min(({}), ({}))".format(val1.id, str(val2)), error=UnitError([val1, val2], "Cannot compare " + un.hr_units(val1.units) + " to a unitless number.", ["par_min"]))
        return Parameter((min(val1.min, val2), min(val1.max, val2)), val1.units, "min({},{})".format(val1.name, val2))
    
    raise TypeError("Second input to min() must be of type Parameter, int, or float.")


def par_max(val1, val2=None):
    if pass_errors(val1, val2): return pass_errors(val1, val2, caller="par_max")

    if not val2:
        if isinstance(val1, Parameter):
            if val1.units == {}:
                return val1.max
            else:
                return Parameter((val1.max, val1.max), val1.units, "max({})".format(val1.name))
        elif isinstance(val1, (int, float)):
            return val1
    
    val1, val2 = _process_minmax_par_inputs(val1, val2)

    if isinstance(val1, Parameter):
        if isinstance(val2, Parameter):
            if val1.id == val2.id: 
                return Parameter((val1.max, val1.max), val1.units, "max({})".format(val1.name))
            if val1.units != val2.units:
                return Parameter((np.nan, np.nan), val1.units, "min(({}), ({}))".format(val1.id, val2.id), error=UnitError([val1, val2], "Cannot compare " + un.hr_units(val1.units) + " to " + un.hr_units(val2.units) + ".", ["par_min"]))
            return Parameter((max(val1.min, val2.min), max(val1.max, val2.max)), val1.units, "max({},{})".format(val1.name, val2.name))
        elif isinstance(val2, (int, float)):
            if val1.units != {}:
                return Parameter((np.nan, np.nan), val1.units, "min(({}), ({}))".format(val1.id, str(val2)), error=UnitError([val1, val2], "Cannot compare " + un.hr_units(val1.units) + " to a unitless number.", ["par_min"]))
            return Parameter((max(val1.min, val2), max(val1.max, val2)), val1.units, "max({},{})".format(val1.name, val2))
    elif isinstance(val1, (int, float)):
        if isinstance(val2, (int, float)):
            return max(val1, val2)
    
    raise TypeError("Inputs to max() must be of type Parameter, int, or float.")




def par_sin(val):
    if pass_errors(val): return pass_errors(val, caller="par_sin")

    if isinstance(val, Parameter):
        if val.units != {}:
            return Parameter((np.nan, np.nan), val.units, "sin({})".format(val.id), error=UnitError([val], "Input to sine must be unitless.", ["par_sin"]))
        results = [np.sin(val.min), np.sin(val.max)]
        return Parameter((min(results), max(results)), {}, "sin({})".format(val.id))
    elif isinstance(val, (int, float)):
        return Parameter((np.sin(val), np.sin(val)), {}, "sin({})".format(val))
    else:
        raise TypeError("Input to sin() must be of type Parameter, int, or float.")


def par_cos(val):
    if pass_errors(val): return pass_errors(val, caller="par_cos")

    if isinstance(val, Parameter):
        if val.units != {}:
            return Parameter((np.nan, np.nan), val.units, "cos({})".format(val.id), error=UnitError([val], "Input to cosine must be unitless.", ["par_cos"]))
        results = [np.cos(val.min), np.cos(val.max)]
        return Parameter((min(results), max(results)), {}, "cos({})".format(val.id))
    elif isinstance(val, (int, float)):
        return Parameter((np.cos(val), np.cos(val)), {}, "cos({})".format(val))
    else:
        raise TypeError("Input to cos() must be of type Parameter, int, or float.")


def par_tan(val):
    if pass_errors(val): return pass_errors(val, caller="par_tan")

    if isinstance(val, Parameter):
        if val.units != {}:
            return Parameter((np.nan, np.nan), val.units, "tan({})".format(val.id), error=UnitError([val], "Input to tangent must be unitless.", ["par_tan"]))
        results = [np.tan(val.min), np.tan(val.max)]
        return Parameter((min(results), max(results)), {}, "tan({})".format(val.id))
    elif isinstance(val, (int, float)):
        return Parameter((np.tan(val), np.tan(val)), {}, "tan({})".format(val))
    else:
        raise TypeError("Input to tan() must be of type Parameter, int, or float.")


def apar_sin(val):
    if pass_errors(val): return pass_errors(val, caller="apar_sin")

    if isinstance(val, Parameter):
        if val.units != {}:
            return Parameter((np.nan, np.nan), val.units, "asin({})".format(val.id), error=UnitError([val], "Input to arcsine must be unitless.", ["apar_sin"]))
        if not -1 <= val.min <= 1 or not -1 <= val.max <= 1:
            return Parameter((np.nan, np.nan), val.units, "asin({})".format(val.id), error=ParameterError([val], "Input to arcsine must be between -1 and 1.", ["apar_sin"]))
        results = [np.arcsin(val.min), np.arcsin(val.max)]
        return Parameter((min(results), max(results)), {}, "asin({})".format(val.id))
    elif isinstance(val, (int, float)):
        if not -1 <= val <= 1:
            return Parameter((np.nan, np.nan), {}, "asin({})".format(val), error=ParameterError([val], "Input to arcsine must be between -1 and 1.", ["apar_sin"]))
        return Parameter((np.arcsin(val), np.arcsin(val)), {}, "asin({})".format(val))
    else:
        raise TypeError("Input to asin() must be of type Parameter, int, or float.")


def apar_cos(val):
    if pass_errors(val): return pass_errors(val, caller="apar_cos")

    if isinstance(val, Parameter):
        if val.units != {}:
            return Parameter((np.nan, np.nan), val.units, "acos({})".format(val.id), error=UnitError([val], "Input to arccosine must be unitless.", ["apar_cos"]))
        if not -1 <= val.min <= 1 or not -1 <= val.max <= 1:
            return Parameter((np.nan, np.nan), val.units, "acos({})".format(val.id), error=ParameterError([val], "Input to arccosine must be between -1 and 1.", ["apar_cos"]))
        results = [np.arccos(val.min), np.arccos(val.max)]
        return Parameter((min(results), max(results)), {}, "acos({})".format(val.id))
    elif isinstance(val, (int, float)):
        if not -1 <= val <= 1:
            return Parameter((np.nan, np.nan), {}, "acos({})".format(val), error=ParameterError([val], "Input to arccosine must be between -1 and 1.", ["apar_cos"]))
        return Parameter((np.arccos(val), np.arccos(val)), {}, "acos({})".format(val))
    else:
        raise TypeError("Input to acos() must be of type Parameter, int, or float.")


def apar_tan(val):
    if pass_errors(val): return pass_errors(val, caller="apar_tan")

    if isinstance(val, Parameter):
        if val.units != {}:
            return Parameter((np.nan, np.nan), val.units, "atan({})".format(val.id), error=UnitError([val], "Input to arctangent must be unitless.", ["apar_tan"]))
        results = [np.arctan(val.min), np.arctan(val.max)]
        return Parameter((min(results), max(results)), {}, "atan({})".format(val.id))
    elif isinstance(val, (int, float)):
        return Parameter((np.arctan(val), np.arctan(val)), {}, "atan({})".format(val))
    else:
        raise TypeError("Input to atan() must be of type Parameter, int, or float.")


def par_sqrt(val):
    if pass_errors(val): return pass_errors(val, caller="par_sqrt")

    new_units = {k: v / 2 for k, v in val.units.items()}

    if isinstance(val, Parameter):
        if not val >= 0:
            return Parameter((np.nan, np.nan), val.units, "sqrt({})".format(val.id), error=ParameterError([val], "Input to sqrt must be >0.", ["par_sqrt"]))
        return Parameter((np.sqrt(val.min), np.sqrt(val.max)), new_units, "sqrt({})".format(val.id))
    elif isinstance(val, (int, float)):
        if not val >= 0:
            return Parameter((np.nan, np.nan), {}, "sqrt({})".format(val), error=ParameterError([val], "Input to sqrt must be >0.", ["par_sqrt"]))
        return Parameter((np.sqrt(val), np.sqrt(val)), new_units, "sqrt({})".format(val))
    else:
        raise TypeError("Input to sqrt() must be of type Parameter, int, or float.")


def par_abs(val):
    if pass_errors(val): return pass_errors(val, caller="par_abs")

    if isinstance(val, Parameter):
        # ERR option ETC
        if abs(val.min) < abs(val.max):
            return Parameter((abs(val.min), abs(val.max)), val.units, "|{}|".format(val.id))
        else:
            return Parameter((abs(val.max), abs(val.min)), val.units, "|{}|".format(val.id))
    elif isinstance(val, (int, float)):
        return Parameter((abs(val), abs(val)), val.units, "|{}|".format(val))
    else:
        raise TypeError("Input to abs() must be of type Parameter, int, or float.")

def par_log(val):
    if pass_errors(val): return pass_errors(val, caller="par_log")

    if isinstance(val, Parameter):
        # ERR option ETC
        if np.log(val.min) < np.log(val.max):
            return Parameter((np.log(val.min), np.log(val.max)), {}, "|{}|".format(val.id))
        else:
            return Parameter((np.log(val.max), np.log(val.min)), {}, "|{}|".format(val.id))
    elif isinstance(val, (int, float)):
        return Parameter((np.log(val), np.log(val)), {}, "|{}|".format(val))
    else:
        raise TypeError("Input to log() must be of type Parameter, int, or float.")

def par_log10(val):
    if pass_errors(val): return pass_errors(val, caller="par_log10")

    if isinstance(val, Parameter):
        # ERR option ETC
        if np.log10(val.min) < np.log10(val.max):
            return Parameter((np.log10(val.min), np.log10(val.max)), {}, "|{}|".format(val.id))
        else:
            return Parameter((np.log10(val.max), np.log10(val.min)), {}, "|{}|".format(val.id))
    elif isinstance(val, (int, float)):
        return Parameter((np.log10(val), np.log10(val)), {}, id="|{}|".format(val))
    else:
        raise TypeError("Input to log10() must be of type Parameter, int, or float.")

def par_floor(val):
    if pass_errors(val): return pass_errors(val, caller="par_floor")

    if isinstance(val, Parameter):
        # ERR option ETC
        return Parameter((np.floor(val.min), np.floor(val.max)), val.units, "floor({})".format(val.id))
    elif isinstance(val, (int, float)):
        return Parameter((np.floor(val), np.floor(val)), {}, "floor({})".format(val))
    else:
        raise TypeError("Input to floor() must be of type Parameter, int, or float.")

class Error:
    def __init__(self):
        pass

class DesignError(Error):
    def __init__(self, model, filename):
        error = bcolors.FAIL + "DesignError" + bcolors.ENDC
        print(error + " can't find " + filename)
        interpreter(model)

class UnitError(Error):
    def __init__(self, parameters, source_message, source):
        self.error_tag = bcolors.FAIL + "UnitError" + bcolors.ENDC
        self.parameters = parameters
        self.source = list(source)
        self.source_message = source_message
        
    def throw(self, model, throw_message, debug=False):
        model_name = model if isinstance(model, str) else model.name
        print(f"{self.error_tag} in {model_name}: {throw_message}")
        print("Source: " + str(self.source))
        print(self.source_message)
        for parameter in self.parameters:
            if isinstance(parameter, Parameter):
                model_text = "" if not parameter.model else " in model " + parameter.model
                parameter_text = f"{parameter.name} ({parameter.id})" if parameter.name != parameter.id else parameter.name
                print(f"  - ({un.hr_units(parameter.units)}) in {parameter_text} from line {parameter.line_no}{model_text}")
            else:
                print("  - " + str(parameter))
        
        if model and isinstance(model, Model):
            if model.calculated: 
                interpreter(model)
            elif debug:
                debugger(model)
        
        quit()

class ParameterError(Error):
    def __init__(self, parameter, source_message, source):
        self.error_tag = bcolors.FAIL + "ParameterError" + bcolors.ENDC
        self.parameter = parameter
        self.source = list(source)
        self.source_message = source_message
        
    def throw(self, model, throw_message, debug=False):
        if model:
            name = model.name
        else:
            name = ""
        print(f"{self.error_tag} in {name}: {throw_message}")
        print("Source: " + str(self.source))
        if self.source_message: print(self.source_message)
        if isinstance(self.parameter, Parameter):
            print(f"  - ({str(self.parameter)}) in {self.parameter.name} ({self.parameter.id}) from line {str(self.parameter.line_no)} in model {model}")
        else:
            print("  - " + str(self.parameter))
        
        if model:
            if model.calculated: 
                interpreter(model)
            elif debug:
                debugger(model)
        
        quit()

class NoteError(Error):
    def __init__(self, model, parameter, message):
        error = bcolors.FAIL + "NoteError" + bcolors.ENDC
        print(f"Note line {parameter.note_line_no}: {parameter.note}")
        interpreter(model)

class SyntaxError(Error):
    def __init__(self, model, filename, line_no, line, message):
        error = bcolors.FAIL + "SyntaxError" + bcolors.ENDC
        print(f"{error} in {filename}: (line {line_no}) {line} - {message}")
        if model and model.calculated:
            interpreter(model)
        else:
            loader([])

class IDError(Error):
    def __init__(self, model, ID, message):
        error = bcolors.FAIL + "IDError" + bcolors.ENDC
        print(f"{error} ({ID}) in {model.name}: {message}")
        interpreter(model)

class ImportError(Error):
    def __init__(self, model, filename, line_no, line, imprt):
        error = bcolors.FAIL + "ImportError" + bcolors.ENDC
        print(f"{error} in {filename}: (line {line_no}) {line} - Failed to import {imprt}. Does the import run by itself?")
        if model:
            interpreter(model)
        else:
            loader([])

class ModelError(Error):
    def __init__(self, filename, source_message="", source=None):
        self.error_tag = bcolors.FAIL + "ModelError" + bcolors.ENDC
        self.filename = filename
        self.source = list(source)
        self.source_message = source_message
        
    def throw(self, return_model, throw_message):
        print(f"{self.error_tag} in {self.filename}: {throw_message}")
        if self.source: 
            print("Source: " + str(self.source))
            print(self.source_message)
        if return_model:
            interpreter(return_model)
        else:
            loader([])

class PythonError(Error):
    def __init__(self, parameter, message):
        error = bcolors.FAIL + "PythonError" + bcolors.ENDC
        print(f"{error} in {parameter.equation}: (line {parameter.line_no}) {parameter.line} - {message}")
        

def pass_errors(*args, caller=None):
    for arg in args:
        if isinstance(arg, Parameter):
            if arg.error:
                if caller:
                    arg.error.source.append(caller)
                return arg
    return False

class Test:
    def __init__(self, line, line_no, model, section=""):
        self.model = model
        self.line_no = line_no
        self.notes = []
        self.note_line_nos = []
        self.section = section

        # Parse the line
        if line[0] == '*':
            self.trace = True
            self.line = line[1:]
        else:
            self.trace = False
            self.line = line
        
        if '{' in line.split(':')[0]:
            try:
                self.refs = [l.strip() for l in line.split(':')[0].split('{')[1].split('}')[0].split(',')]
            except:
                SyntaxError(None, model, line_no, line, "Invalid syntax for test references.")
        else:
            self.refs = []

        self.expression = line.split(':')[1].strip()

        if not self.expression:
            SyntaxError(None, model, line_no, line, "Empty test expression.")

        for old, new in FUNCTIONS.items():
            if "." + old not in self.expression:
                self.expression = self.expression.replace(old + "(", new + "(")
        for old, new in OPERATOR_OVERRIDES.items():
            self.expression = self.expression.replace(old, new)

        self.args = [x for x in re.findall("(?!\d+)\w+\.?\w*", self.expression) if x not in FUNCTIONS]


class Parameter:
    def __init__(self, equation, units, id, model="", line_no=None, line="", name=None, options=None, performance=False, trace=False, section="", arguments=[], error=None):
        if trace == 'init':            
            import pdb
            breakpoint()

        
        self.id = id
        self.name = name if name else id
        self.line_no = line_no
        self.line = line
        self.model = model if model else None
        self.performance = performance
        self.independent = False
        self.trace = trace
        self.callable = False
        self.isdiscrete = False
        self.min = self.max = None
        self.equation = None
        self.args = copy.deepcopy(arguments)
        self.section = section
        self.error = error
        self.pointer = False
        self.piecewise = False
        self.minmax_equation = False
        
        # note
        self.notes = []
        self.note_lines = []

        # options
        if options:
            if isinstance(options, list):
                self.isdiscrete = True
            elif not isinstance(options, tuple):
                raise TypeError("Options must be either a list (discrete options) or a tuple (continuous range).")

            self.options = options
        else:
            self.options = None

        # units
        if isinstance(units, dict):
            self.units = units
            for unit in units:
                if unit not in un.BASE_UNITS:
                    UnitError([self], "", ["Parameter.__init__"]).throw(None, unit + " is not currently a supported input unit. Only " + str(un.BASE_UNITS) + " are supported. Refactor " + unit + " in terms of " + str(un.BASE_UNITS))
        else:
            raise TypeError('Units must be of type dict. Type "' + str(type(units)) + " was given.")

        # equation
        if callable(equation):
            self.callable = True
            self.equation = equation
        elif isinstance(equation, (float, int)):
            self.assign(equation)
            self.independent = True
        elif isinstance(equation, str):
            if any(character in EQUATION_OPERATORS for character in equation):
                # Find parameter names including "." imports
                self.args = [x for x in re.findall("(?!\d+)\w+\.?\w*", re.sub('[\'|\"].*[\'|\"]','',equation)) if x not in FUNCTIONS]

                # Trim duplicate args
                self.args = list(set(self.args))

                for old, new in FUNCTIONS.items():
                    if "." + old not in equation:
                        equation = equation.replace(old + "(", new + "(")
                for old, new in OPERATOR_OVERRIDES.items():
                    equation = equation.replace(old, new)

                self.equation = equation
                self.independent = False
            else:
                if self.options and isinstance(self.options, list):
                    self.assign(equation)
                    self.independent = True
                else:
                    self.equation = equation
                    self.args = [equation]
                    self.pointer = True
        elif isinstance(equation, tuple):
            if isinstance(equation[0], (float, int)) and isinstance(equation[1], (float, int)):
                self.assign(equation)
                self.independent = True
            else:
                self.minmax_equation = True
                self.equation = equation
                self.args.extend(list(set(equation[0].args + equation[1].args)))
        elif isinstance(equation, list):
            self.equation = equation
            piece_args = equation[0][0].args + equation[0][1].args
            if not piece_args:
                self.error = ParameterError([self], "Piecewise parameters must be dependent on another parameter.", ["Parameter.__init__"])
            else:
                self.args.extend(piece_args)
        else:
            ParameterError([self], "", ["Parameter.__init__"]).throw(None, "Parameter equation must be a callable, a float, an int, or a string. If callable, did you forget the preceding underscore. Or did you forget to place an equation in quotes?")

    def add_piece(self, piece, callable_args):
        self.piecewise = True
        self.equation.append(piece)
        piece_args = piece[0].args + piece[1].args + callable_args
        if not piece_args:
            self.error = ParameterError([self], "Piecewise parameters must be dependent on another parameter.", ["Parameter.__init__"])
        else:
            self.args = list(set(self.args + piece_args))

    def assign(self, value):
        self.write(value)

    def write(self, value):
        if isinstance(value, Parameter):
            if self.options and value.equation in self.options:
                value.min = value.max = value.equation
                value.equation = None
                value.isdiscrete = True
                
            if value.min is not None and value.max is not None:
                if value.units != self.units:
                    self.error = UnitError([self], "Input or calculated units (" + str(value.units) + ") do not match the required units: (" + str(self.units) + ").", ["Parameter.write()"])
                self.min = value.min
                self.max = value.max
                # TODO: consider whether we should allow an independent value to cause a dependent parameter to be independent
            elif not value.independent:
                self.equation = value.equation
                self.args = value.args
                self.callable = value.callable
                self.minmax_equation = value.minmax_equation
                self.piecewise = value.piecewise
                self.performance = value.performance
                self.pointer = value.pointer
                self.min = self.max = None
                self.independent = False
            else:
                raise ValueError("Parameter " + value.id + " cannot be written to " + self.id + ", because it is empty and independent.")
            
            if value.model: self.model = value.model

            self.line_no = {'model line': self.line_no, 'design line': value.line_no}
            self.line = {'model line': self.line, 'design line': value.line}
            self.notes = value.notes
            self.note_lines = value.note_lines
            self.section = value.section
            self.isdiscrete = value.isdiscrete
            self.error = value.error
            self.trace = value.trace
        elif isinstance(value, tuple):
            if self.isdiscrete:
                self.error = ParameterError(self, "Multiple discrete values aren't supported.", ["Parameter.write()"])
            self.write_one(value[0], "min")
            self.write_one(value[1], "max")
        else:
            self.write_one(value, "minmax")

        if self.min and self.max:
            if self.min > self.max:
                self.error = ParameterError(self, "Parameter min is greater than Parameter max.", ["Parameter.write()"])

            if self.options and self.min and self.max:
                if self.isdiscrete:
                    if not (self.min in self.options and self.max in self.options):
                        self.error = ParameterError(self, "Parameter was given a value that is not among its options.", ["Parameter.write()"])
                else:
                    if not self.options[1] >= self.options[0]:
                        self.error = ParameterError(self, "Minimum limit > maximum limit.", ["Parameter.write()"])
                    if not (self.min >= self.options[0] and self.max <= self.options[1]):
                        self.error = ParameterError(self, f"Values out of bounds [{self.options[0]}:{self.options[1]}]. Revise values or limits.", ["Parameter.write()"])

    def write_one(self, value, minmax):

        if isinstance(value, (int, float)):
            if minmax == "minmax":
                self.min = self.max = value
            elif minmax == "min":
                self.min = value
            elif minmax == "max":
                self.max = value
        elif isinstance(value, (bool, np.bool_)):
            if minmax == "minmax":
                self.min = self.max = bool(value)
            elif minmax == "min":
                self.min = bool(value)
            elif minmax == "max":
                self.max = bool(value)
        elif isinstance(value, str):
            if value not in self.options:
                self.error = ParameterError(self, "Parameter was assigned an option that is not among its options.", ["Parameter.write()"])
            
            if minmax == "minmax":
                self.min = self.max = value
            elif minmax == "min":
                self.min = value
            elif minmax == "max":
                self.max = value
        else:
            raise TypeError('Parameter value must be of type Parameter, tuple, int, float, str, or bool. Type "' + str(type(value)) + " was given.")


    def calculate(self, expression, glob, eval_params, eval_args):
        if not self.equation:
            return ParameterError(self, "Parameter needs an equation or value defined.", ["Parameter.calculate()"])
        if (self.min or self.max):
            return ParameterError(self, "Parameters cannot be re-calculated.", ["Parameter.calculate()"])

        if self.trace == 'calc':
            import pdb
            breakpoint()

        if self.callable:
            if not all(k in eval_params for k in eval_args):
                return ParameterError(self, "Parameter is missing one or more arguments: " + str([arg for arg in eval_args if arg not in eval_params]), ["Parameter.calculate()"])

            function_args = [eval_params[arg] for arg in eval_args]

            try:
                return self.equation(*function_args)
            except:
                PythonError(self, "Calculation error.")
                import pdb

                breakpoint()
                return self.equation(*function_args)
        else:
            try:
                return eval(expression, glob, eval_params | MATH_CONSTANTS)
            except:
                PythonError(self, "Calculation error.")
                import pdb

                breakpoint()

    # Parameter Printing

    def __repr__(self, sigfigs=4, indent=0, verbose=False, submodel_id=""):
        output = " " * indent
        output += self.id + "." + submodel_id if submodel_id else self.id
        output += ": " + self.human_readable(sigfigs)
        output += " -- " + self.name if verbose else ""
        output += " (" + self.model + ")" if verbose and self.model else ""
        return output

    def print(self):
        print("<---" + self.id + "--->")
        for key, val in self.__dict__.items():
            print(key + ": " + self)
        print("\n")

    def short_print(self, sigfigs=4, indent=0, verbose=False, submodel_id=""):
        print(self.__repr__(sigfigs=sigfigs, indent=indent, verbose=verbose, submodel_id=submodel_id))

    def hprint(self, sigfigs=4, indent=0):
        output = ("\n" + self.name + "\n--------------------\n" if self.performance else "")
        output += " " * indent + self.id + ": "
        output += self.human_readable(sigfigs)
        print(output)

    def human_readable(self, sigfigs=4):
        if self.isdiscrete:
            return self.min + " | " + self.max
        else:
            if self.min is not None and self.max is not None:
                if isinstance(self.min, str):
                    return self.min if self.min == self.max else self.min + " | " + self.max
                else:
                    return un.hr_vals_and_units((self.min,self.max), self.units, sigfigs)
            else:
                return un.hr_units(self.units, sigfigs=sigfigs)

    def copy(self):
        return Parameter((self.min, self.max), self.units, "copy of " + self.name, model=self.model, line_no=self.line_no, line=self.line)

    def __str__(self):
        return self.human_readable(4)

    def _clone(self, parameter):
        #warnings.warn("A value, " + str(parameter) + ", was cloned.")
        self.min = parameter.min
        self.max = parameter.max
        self.units = parameter.units
        self.name = parameter.name

    # "+" Addition, left-hand, all cases 
    def __add__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__add__")

        if isinstance(other, Parameter):
            if self.units != other.units:
                return Parameter((np.nan, np.nan), self.units, "({}) + ({})".format(self.id, other.id), error=UnitError([self, other], "Cannot add " + un.hr_units(other.units) + " to " + un.hr_units(self.units) + ".", ["Parameter.__add__"]))
            return Parameter((self.min + other.min, self.max + other.max), self.units, "({}) + ({})".format(self.id, other.id))
        elif self.units == {}:
            return Parameter((self.min + other, self.max + other), {}, "({}) + ({})".format(self.id, str(other)))
        else:
            return Parameter((np.nan, np.nan), self.units, "({}) + ({})".format(self.id, str(other)), error=UnitError([self, other], "Cannot add " + un.hr_units(self.units) + " to a unitless number", ["Parameter.__add__"]))

    # "-" Subtraction, left-hand, extreme
    def __sub__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__sub__")

        if isinstance(other, Parameter):
            if self.units != other.units:
                return Parameter((np.nan, np.nan), self.units, "({}) - ({})".format(self.id, other.id), error=UnitError([self, other], "Cannot subtract " + un.hr_units(other.units) + " from " + un.hr_units(self.units) + ".", ["Parameter.__sub__"]))
            if self.id == other.id and self.model == other.model: return Parameter(0, {}, "({}) - ({})".format(self.id, other.id))
            return Parameter((self.min - other.max, self.max - other.min), self.units, "({}) - ({})".format(self.id, other.id))
        elif self.units == {}:
            return Parameter((self.min - other, self.max - other), {}, "({}) - ({})".format(self.id, str(other)))
        else:
            return Parameter((np.nan, np.nan), self.units, "({}) - ({})".format(self.id, str(other)), error=UnitError([self, other], "Cannot subtract a unitless number from " + un.hr_units(self.units), ["Parameter.__sub__"]))

    # "--" Subtraction, left-hand, standard
    def _minus(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.minus")

        if isinstance(other, Parameter):
            if self.units != other.units:
                return Parameter((np.nan, np.nan), self.units, "({}) -- ({})".format(self.id, other.id), error=UnitError([self, other], "Cannot subtract " + un.hr_units(other.units) + " from " + un.hr_units(self.units) + ".", ["Parameter.minus"]))
            results = [self.min - other.min, self.max - other.max]
            return Parameter((min(results), max(results)), self.units, "({}) -- ({})".format(self.id, other.id))
        elif self.units == {}:
            results = [self.min - other, self.max - other]
            return Parameter((min(results), max(results)), {}, "({}) -- ({})".format(self.id, str(other)))
        else:
            return Parameter((np.nan, np.nan), self.units, "({}) -- ({})".format(self.id, str(other)), error=UnitError([self, other], "Cannot subtract a unitless number from " + un.hr_units(self.units), ["Parameter.minus"]))

    # "*" Multiplication, left-hand, all cases
    def __mul__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__mul__")

        if isinstance(other, Parameter):
            new_units = {k: v for k, v in self.units.items()}
            for k, v in other.units.items():
                if k in new_units:
                    new_units[k] += v
                    if new_units[k] == 0:
                        del new_units[k]
                else:
                    new_units[k] = v
            results = [self.min * other.min, self.max * other.max]
            return Parameter((min(results), max(results)), new_units, "({}) * ({})".format(self.id, other.id))
        elif isinstance(other, (int, float)):
            results = [self.min * other, self.max * other]
            return Parameter((min(results), max(results)), self.units, "({}) * ({})".format(self.id, str(other)))
        else:
            TypeError("Multiplication must be between two Parameters or a Parameter and a number.")

    # "/" Division, left-hand, extreme
    def __truediv__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__truediv__")

        if isinstance(other, Parameter):
            if self.id == other.id and self.model == other.model: 
                return Parameter(1, {}, "({}) / ({})".format(self.id, other.id))
            new_units = {k: v for k, v in self.units.items()}
            for k, v in other.units.items():
                if k in new_units:
                    new_units[k] -= v
                    if new_units[k] == 0:
                        del new_units[k]
                else:
                    new_units[k] = -v
            results = [self.min / other.max, self.max / other.min]
            return Parameter((min(results), max(results)), new_units, "({}) / ({})".format(self.id, other.id))
        elif isinstance(other, (int, float)):
            results = [self.min / other, self.max / other]
            return Parameter((min(results), max(results)), self.units, "({}) / ({})".format(self.id, str(other)))
        else:
            raise TypeError("Division must be between two Parameters or a Parameter and a number.")

    # "//" Division, left-hand, standard
    def __floordiv__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__floordiv__")

        if isinstance(other, Parameter):
            new_units = {k: v for k, v in self.units.items()}
            for k, v in other.units.items():
                if k in new_units:
                    new_units[k] -= v
                    if new_units[k] == 0:
                        del new_units[k]
                else:
                    new_units[k] = -v
            results = [self.min / other.min, self.max / other.max]
            return Parameter((min(results), max(results)), new_units, "({}) // ({})".format(self.id, other.id))
        elif isinstance(other, (int, float)):
            results = [self.min / other, self.max / other]
            return Parameter((min(results), max(results)), self.units, "({}) // ({})".format(self.id, str(other)))
        else:
            raise TypeError("Division must be between two Parameters or a Parameter and a number.")

    # "**" Exponentiation, left-hand, all cases
    def __pow__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__pow__")

        if isinstance(other, Parameter):
            if self.min != self.max or other.units != {}:
                return Parameter((np.nan, np.nan), self.units, "({})**({})".format(self.id, other.id), error=UnitError([self, other], "Exponent must be a single unitless Parameter or number.", ["Parameter.__pow__"]))
            new_units = {k: v * other.min for k, v in self.units.items()}
            results = [self.min**other.min, self.max**other.max, self.min**other.max, self.max**other.min]
            return Parameter((min(results), max(results)), new_units, "({})**({})".format(self.id, other.id))
        elif isinstance(other, (int, float)):
            new_units = {k: v * other for k, v in self.units.items()}
            results = [self.min**other, self.max**other]
            return Parameter((min(results), max(results)), new_units, "({})**({})".format(self.id, str(other)))
        else:
            raise TypeError("Exponent must be a single unitless Parameter or number.")

    # "+" Addition, right-hand, all cases
    def __radd__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__radd__")

        if isinstance(other, (int, float)):
            if self.units == {}:
                return Parameter((other + self.min, other + self.max), {}, "({}) + ({})".format(str(other), self.id))
            else:
                return Parameter((np.nan, np.nan), self.units, "({}) + ({})".format(other, self.id), error=UnitError([self, other], "Cannot add " + un.hr_units(self.units) + " to a unitless number.", ["Parameter.__radd__"]))
        else:
            raise TypeError("Addition must be between two Parameters or a Parameter and a number.")

    # "-" Subtraction, right-hand, extreme
    def __rsub__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__rsub__")

        if isinstance(other, (int, float)):
            if self.units == {}:
                return Parameter((other - self.max, other - self.min), {}, "({}) - ({})".format(str(other), self.id))
            else:
                return Parameter((np.nan, np.nan), self.units, "({}) - ({})".format(other, self.id), error=UnitError([self, other], "Cannot subtract " + un.hr_units(self.units) + " from a unitless number.", ["Parameter.__rsub__"]))
        else:
            raise TypeError("Addition must be between two Parameters or a Parameter and a number.")

    # "--" Subtraction, right-hand, standard
    def _rminus(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__rsub__")

        if isinstance(other, (int, float)):
            if self.units == {}:
                return Parameter((other - self.min, other - self.max), {}, "({}) - ({})".format(str(other), self.id))
            else:
                return Parameter((np.nan, np.nan), self.units, "({}) - ({})".format(other, self.id), error=UnitError([self, other], "Cannot subtract " + un.hr_units(self.units) + " from a unitless number.", ["Parameter.__rsub__"]))
        else:
            raise TypeError("Addition must be between two Parameters or a Parameter and a number.")

    # "*" Multiplication, right-hand, all cases
    def __rmul__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__rmul__")

        if isinstance(other, (int, float)):
            return Parameter((self.min * other, self.max * other), self.units, "({})({})".format(str(other), self.id))
        else:
            raise TypeError("Multiplication must be between a Parameter and a number.")

    # "/" Division, right-hand, extreme
    def __rtruediv__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__rtruediv__")

        if isinstance(other, (int, float)):
            new_units = {k: -v for k, v in self.units.items()}
            return Parameter((other / self.max, other / self.min), new_units, "({})/({})".format(str(other), self.id))
        else:
            raise TypeError("Division must be between a Parameter and a number.")

    # "//" Division, right-hand, standard
    def __rfloordiv__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__rfloordiv__")

        if isinstance(other, (int, float)):
            new_units = {k: -v for k, v in self.units.items()}
            return Parameter((other / self.min, other / self.max), new_units, "({})//({})".format(str(other), self.id))
        else:
            raise TypeError("Division must be between a Parameter and a number.")

    # "**" Exponentiation, right-hand, all cases
    def __rpow__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__rpow__")

        if isinstance(other, (int, float)):
            if self.units != {}:
                return Parameter((np.nan, np.nan), self.units, "({})**({})".format(other, self.id), error=UnitError([self, other], "Exponent must be a single unitless Parameter or number.", ["Parameter.__rpow__"]))
            return Parameter((other**self.min, other**self.max), {}, "({})**({})".format(str(other), self.id))
        else:
            raise TypeError("Exponentiation must be between a Parameter and a number.")

    # "-" Unary operator
    def __neg__(self):
        return Parameter((-self.max, -self.min), self.units, "-({})".format(self.id))

    # "<" Less than, left-hand, all cases
    def __lt__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__lt__")

        if isinstance(other, Parameter):
            if self.units != other.units:
                return Parameter((np.nan, np.nan), self.units, "({}) < ({})".format(other.id, self.id), error=UnitError([self, other], "Cannot compare unit \"" + un.hr_units(self.units) + "\" to " + un.hr_units(self.units) + ".", ["Parameter.__lt__"]))
            return self.min < other.min and self.max < other.max
        elif isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                return Parameter((np.nan, np.nan), self.units, "({}) < ({})".format(other, self.id), error=UnitError([self, other], "Cannot compare unit \"" + un.hr_units(self.units) + "\" to a unitless number.", ["Parameter.__lt__"]))
            return self.min < other and self.max < other
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # ">" Greater than, left-hand, all cases
    def __gt__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__gt__")

        if isinstance(other, Parameter):
            if self.units != other.units:
                return Parameter((np.nan, np.nan), self.units, "({}) > ({})".format(other.id, self.id), error=UnitError([self, other], "Cannot compare unit \"" + un.hr_units(self.units) + "\" to " + un.hr_units(self.units) + ".", ["Parameter.__gt__"]))
            return self.min > other.min and self.max > other.max
        elif isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                return Parameter((np.nan, np.nan), self.units, "({}) > ({})".format(other, self.id), error=UnitError([self, other], "Cannot compare unit \"" + un.hr_units(self.units) + "\" to a unitless number.", ["Parameter.__gt__"]))
            return self.min > other and self.max > other
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # ">=" Greater than or equal to, left-hand, all cases
    def __le__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__le__")

        if isinstance(other, Parameter):
            if self.units != other.units:
                return Parameter((np.nan, np.nan), self.units, "({}) <= ({})".format(other.id, self.id), error=UnitError([self, other], "Cannot compare unit \"" + un.hr_units(self.units) + "\" to " + un.hr_units(self.units) + ".", ["Parameter.__le__"]))
            return self.min <= other.min and self.max <= other.max
        elif isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                return Parameter((np.nan, np.nan), self.units, "({}) <= ({})".format(other, self.id), error=UnitError([self, other], "Cannot compare unit \"" + un.hr_units(self.units) + "\" to a unitless number.", ["Parameter.__le__"]))
            return self.min <= other and self.max <= other
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # "<=" Less than or equal to, left-hand, all cases
    def __ge__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__ge__")

        if isinstance(other, Parameter):
            if self.units != other.units:
                return Parameter((np.nan, np.nan), self.units, "({}) >= ({})".format(other.id, self.id), error=UnitError([self, other], "Cannot compare unit \"" + un.hr_units(self.units) + "\" to " + un.hr_units(self.units) + ".", ["Parameter.__ge__"]))
            return self.min >= other.min and self.max >= other.max
        elif isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                return Parameter((np.nan, np.nan), self.units, "({}) >= ({})".format(other, self.id), error=UnitError([self, other], "Cannot compare unit \"" + un.hr_units(self.units) + "\" to a unitless number.", ["Parameter.__ge__"]))
            return self.min >= other and self.max >= other
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # "==" Equal to, left-hand, all cases
    def __eq__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__eq__")

        if isinstance(other, Parameter):
            if self.units != other.units:
                return Parameter((np.nan, np.nan), self.units, "({}) == ({})".format(other.id, self.id), error=UnitError([self, other], "Cannot compare unit \"" + un.hr_units(self.units) + "\" to " + un.hr_units(self.units) + ".", ["Parameter.__eq__"]))
            return self.min == other.min and self.max == other.max
        elif isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                return Parameter((np.nan, np.nan), self.units, "({}) == ({})".format(other, self.id), error=UnitError([self, other], "Cannot compare unit \"" + un.hr_units(self.units) + "\" to a unitless number.", ["Parameter.__eq__"]))
            return self.min == other and self.max == other
        elif isinstance(other, str):
            return self.min == other
        elif isinstance(other, bool):
            return bool
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # "!=" Not equal to, left-hand, all cases
    def __ne__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__ne__")

        if isinstance(other, Parameter):
            if self.units != other.units:
                return Parameter((np.nan, np.nan), self.units, "({}) != ({})".format(other.id, self.id), error=UnitError([self, other], "Cannot compare unit \"" + un.hr_units(self.units) + "\" to " + un.hr_units(self.units) + ".", ["Parameter.__ne__"]))
            return self.min != other.min and self.max != other.max
        elif isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                return Parameter((np.nan, np.nan), self.units, "({}) != ({})".format(other, self.id), error=UnitError([self, other], "Cannot compare unit \"" + un.hr_units(self.units) + "\" to a unitless number.", ["Parameter.__ne__"]))
            return self.min != other and self.max != other
        elif isinstance(other, str):
            return self.min == other
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # "<" Less than, right-hand, all cases
    def __rlt__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__rlt__")

        if isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                return Parameter((np.nan, np.nan), self.units, "({}) < ({})".format(other, self.id), error=UnitError([self, other], "Cannot compare a unitless number to unit \"" + un.hr_units(self.units) + "\".", ["Parameter.__rlt__"]))
            return other < self.min and other < self.max
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # "<=" Less than or equal to, right-hand, all cases
    def __rgt__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__rgt__")

        if isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                return Parameter((np.nan, np.nan), self.units, "({}) > ({})".format(other, self.id), error=UnitError([self, other], "Cannot compare a unitless number to unit \"" + un.hr_units(self.units) + "\".", ["Parameter.__rgt__"]))
            return other > self.min and other > self.max
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # ">=" Greater than or equal to, right-hand, all cases
    def __rle__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__rle__")

        if isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                return Parameter((np.nan, np.nan), self.units, "({}) <= ({})".format(other, self.id), error=UnitError([self, other], "Cannot compare a unitless number to unit \"" + un.hr_units(self.units) + "\".", ["Parameter.__rle__"]))
            return other <= self.min and other <= self.max
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # ">=" Greater than or equal to, right-hand, all cases
    def __rge__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__rge__")

        if isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                return Parameter((np.nan, np.nan), self.units, "({}) >= ({})".format(other, self.id), error=UnitError([self, other], "Cannot compare a unitless number to unit \"" + un.hr_units(self.units) + "\".", ["Parameter.__rge__"]))
            return other >= self.min and other >= self.max
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # "==" Equal to, right-hand, all cases
    def __req__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__req__")

        if isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                return Parameter((np.nan, np.nan), self.units, "({}) == ({})".format(other, self.id), error=UnitError([self, other], "Cannot compare a unitless number to unit \"" + un.hr_units(self.units) + "\".", ["Parameter.__req__"]))
            return other == self.min and other == self.max
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # "!=" Not equal to, right-hand, all cases
    def __rne__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__rne__")

        if isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                return Parameter((np.nan, np.nan), self.units, "({}) != ({})".format(other, self.id), error=UnitError([self, other], "Cannot compare a unitless number to unit \"" + un.hr_units(self.units) + "\".", ["Parameter.__rne__"]))
            return other != self.min and other != self.max
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # "|" Logical OR, left-hand, all cases
    def __or__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__or__")
        
        if isinstance(other, Infix):
            return other.__ror__(self)

        if self.units != {}:
            return Parameter((np.nan, np.nan), self.units, "({}) | ({})".format(other, self.id), error=UnitError([self, other], "| is only valid for unitless parameters with boolean values.", ["Parameter.__or__"]))
        
        if isinstance(other, Parameter):
            if other.units != {}:
                return Parameter((np.nan, np.nan), self.units, "({}) | ({})".format(other, self.id), error=UnitError([self, other], "| is only valid for unitless parameters with boolean values.", ["Parameter.__or__"]))
            return self.min or other.min or self.max or other.max
        elif isinstance(other, (bool)):
            return self.min or other or self.max
        else:
            raise TypeError("OR operator is only valid between two Parameters or a Parameter and a boolean.")

    # "&" Logical AND, left-hand, all cases
    def __and__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__and__")
        
        if self.units != {}:
            return Parameter((np.nan, np.nan), self.units, "({}) & ({})".format(other, self.id), error=UnitError([self, other], "& is only valid for unitless parameters with boolean values.", ["Parameter.__or__"]))
        
        if isinstance(other, Parameter):
            if other.units != {}:
                return Parameter((np.nan, np.nan), self.units, "({}) & ({})".format(other, self.id), error=UnitError([self, other], "& is only valid for unitless parameters with boolean values.", ["Parameter.__or__"]))
            return self.min and other.min and self.max and other.max
        elif isinstance(other, (bool)):
            return self.min and other and self.max
        else:
            raise TypeError("AND operator is only valid between two Parameters or a Parameter and a boolean.")

    # "|" Logical OR, right-hand, all cases
    def __ror__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__ror__")
        
        if self.units != {}:
            return Parameter((np.nan, np.nan), self.units, "({}) | ({})".format(other, self.id), error=UnitError([self, other], "| is only valid for unitless parameters with boolean values.", ["Parameter.__or__"]))
        
        if isinstance(other, (bool)):
            return self.min or other or self.max
        else:
            raise TypeError("OR operator is only valid between two Parameters or a Parameter and a boolean.")

    # "&" Logical AND, right-hand, all cases
    def __rand__(self, other):
        if pass_errors(self, other): return pass_errors(self, other, caller="Parameter.__rand__")

        if self.units != {}:
            return Parameter((np.nan, np.nan), self.units, "({}) & ({})".format(other, self.id), error=UnitError([self, other], "& is only valid for unitless parameters with boolean values.", ["Parameter.__or__"]))

        if isinstance(other, (bool)):
            return self.min and other and self.max
        else:
            raise TypeError("AND operator is only valid between two Parameters or a Parameter and a boolean.")


class Model:
    def __init__(self, model_filename, design_filename=None):
        self.note, self.parameters, self.submodels, self.tests, _ = parse_file(model_filename)

        self.name = model_filename.replace(".on", "")
        self.design = "default"
        self.constants = MATH_CONSTANTS
        self.calculated = False
        self.defaults = []
        
        if design_filename:
            self.overwrite(design_filename)

        #ERR ID error
        self._check_namespace()

        for key, param in self.parameters.items():
            # For equations that aren't strings that aren't option cases
            if param.pointer:
                if param.equation in self.parameters.keys():
                    param.args = [param.equation]
                else:
                    param.error = ParameterError(param, "Parameter " + param.id + " (line " + str(param.line_no + 1) + ") in " + param.model + " has a string, non-equation assignment (" + param.equation + ") that is not in the model and has no options defined. If it's supposed to be a case, specify options. If it's supposed to be assigned to another value, make sure that value is also defined.", ["Model.__init__"])
        
        for param in self.parameters.values():
            if param.error:
                param.error.throw(self, "Problem parsing parameter " + param.id + " in " + model_filename + ".")
    
    # Checks that all of the arguments to each parameter are defined
    def _check_namespace(self, verbose=False):
        undefined = []
        
        for k, param in self.parameters.items():
            for arg in param.args:
                if '.' in arg:
                    arg_id, source = arg.split('.')
                    # if source in self.submodels:
                    #     if arg_id not in self._retrieve_model(self.submodels[source]['path']).parameters:
                    #         undefined.append(arg + " from " + param.id + " (line " + str(param.line + 1) +") in " + param.model)
                    # else:
                    #     raise ImportError("Submodel " + source + " not found in " + self.name + ".on")
                elif arg not in self.constants and arg not in self.parameters:
                    undefined.append(arg + " from " + param.id + " (line " + param.line +") in " + param.model)
                elif verbose:
                    # Report the submodule parameters of the same ID.
                    for key in self.submodels:
                        self._check_namespace_recursively(self._retrieve_model(self.submodels[key]['path']), arg, param)

        # Return a list of all args that aren't defined.
        if undefined:
            raise NameError(self.name.capitalize() + " has the following undefined arguments:\n- " + "\n- ".join(undefined))

    # Recursively report all submodule paramaters with the same ID
    def _check_namespace_recursively(self, submodel, arg, param, trail=[]):
        if arg in submodel.parameters:
            print("Argument " + arg + " from " + param.model + " (line " + param.line + ") " + "is also in " + submodel.name + " (line " + submodel.parameters[arg].line +")")
        for i in submodel.submodels:
            if i in trail:
                # return IDError() #TODO
                print("IDError")
            new_trail = copy.copy(trail)
            new_trail.append(i)

            source = self._retrieve_model(submodel.submodels[i]['path'])
            if isinstance(source, Error):
                source.throw(self, "Problem parsing parameter " + param.id + " in " + self.name + ".on")

            submodel._check_namespace_recursively(source, arg, param, new_trail)

    # Convert an empty model to a modeled design with all parameters assigned a value.
    def overwrite(self, design_files, quiet=False):
        if not design_files:
            # TODO: ERR
            return
        
        # Import design parameters.
        if isinstance(design_files, str):
            if not os.path.exists(design_files):
                DesignError(self, design_files)
            _, design_params, _, tests, design = parse_file(design_files, self)
        elif isinstance(design_files, list):
            for file in design_files:
                if not os.path.exists(file):
                    DesignError(self, file)
            _, design_params, _, tests, design = parse_file(design_files[0], self)
            if len(design_files) > 1:
                for design_file in design_files[1:]:
                    _, overdesign_params, _, overtests, overdesign = parse_file(design_file, self)
                    for ID, parameter in overdesign.items():
                        design[ID] = parameter
                    for ID, parameter in overdesign_params.items():
                        design_params[ID] = parameter
                    
                    tests.extend(overtests)
        else:
            raise TypeError("Design file must be a string or list of strings.")
                

        for ID, parameter in design.items():
            if "." in ID:
                ID, submodel = ID.split(".")
                path = copy.copy(self.submodels[submodel]['path'])
            else:
                path = []
            result = self._rewrite_parameter(ID, parameter, path)
            if isinstance(result, ModelError):
                result.throw(self, "(line " + str(self.submodels[submodel]['line_no']) + ") " + self.submodels[submodel]['line'] + "- " + "Couldn't find " + submodel + " in path " + ".".join(path) + " while overwriting " + ID + " in " + design_files + ".")
            elif isinstance(result, ParameterError):
                result.throw(self, "Error rewriting parameter " + ID + " in " + design_files + ".")

        self.defaults.append(list[set(self.parameters).difference(design)])
        
        for ID, parameter in design_params.items():
            self.parameters[ID] = parameter

        self.tests.extend(tests)

        design_files.reverse()
        design_files = [file.strip(".on") for file in design_files]
        new_design = "@".join(design_files)
        if self.design != new_design:
            self.design = new_design + "@" + self.design if self.design != "default" else new_design

        self._reset_recursively()
        self.calculate()

        if not quiet: self.summarize()

    def _param2latex(self, param_ID):
        # Replace parameter functions with their normal function names
        for k, f in FUNCTIONS.items():
            param_ID = param_ID.replace(f + "(", k + "(")

        # Replace submodel imports with backside superscripts (pre-step)
        param_ID = re.sub(r'([a-zA-Z]+\w*_?\w*)\.([a-zA-Z]+)', r'(\2ONEILSUBMODEL*\1)', param_ID)

        # Replace lambd with \lambda since it is a reserved word in Python.
        latex_ID = (py2tex(param_ID).strip("$$").replace("lambd", "\\lambda"))

        # Replace oneilsubmodel<submodel>_<model> with \underset{<submodel>}{<model>}
        latex_ID = re.sub(r'([a-zA-Z]+)ONEILSUBMODEL\s(\\{0,2}[a-zA-Z]+\w*_?\{?\\{0,2}\w*\}?)', r'\\underset{\\bar{\1}}{\2}', latex_ID)

        assert "ONEILSUBMODEL" not in latex_ID, "Submodel conversion failed."

        # Escape functions
        for k in FUNCTIONS:
            latex_ID = latex_ID.replace(k + "(", "\\" + k + "(")

        return latex_ID

    def param_snippet(self, param_ID, param):

        # Latexify the parameter ID
        latex_ID = self._param2latex(param_ID)

        if param.callable:
            equation = "\\verb!" + param.equation.__qualname__ + "!"

            latex_args = []
            for arg in param.args:
                latex_args.append(self._param2latex(arg))

            snippet = "\n\\subsubsection{" + param.name.title() + "}"
            snippet += "\n\\label{sssec:" + param.id + "}\n"
            snippet += param.name + " is found as\n"
            snippet += "\\begin{equation}\n"
            snippet += "\\label{eq:" + param.id + "}\n"
            snippet += latex_ID + " = " + equation + "(" + ", ".join(latex_args) + ")\n"
            snippet += "\\end{equation}\n\n"
            # Introduce the snippet
            snippet += "The code for $" + latex_ID + "=" + equation + "$ () is shown below."
            # Create a code snippet for the function
            snippet += "\n\\begin{lstlisting}[language=Python, caption=Code for " + param.name + ".]\n"
            # Add a caption
            snippet += inspect.getsource(param.equation)
            snippet += "\n\\end{lstlisting}\n"
        elif param.equation:
            # Restore normal function names
            param_eq = param.equation
            for func, vfunc in FUNCTIONS.items(): param_eq = param_eq.replace(vfunc, func)

            snippet = "\n\\subsubsection{" + param.name.title() + "}"
            snippet += "\label{sssec:" + param_ID + "}\n"
            snippet += 'The equation for ' + param.name + " is \n"

            snippet += "\\begin{equation}\n"
            snippet += "\t\\label{eq:" + param_ID + "}\n"
            snippet += "\t\\boxed{" + latex_ID + " = " + self._param2latex(param_eq) + "}~.\n"
            snippet += "\\end{equation}\n"

        else:
            snippet = "\n\\subsubsection{" + param.name.title() + "}"
            snippet += "\label{sssec:" + param_ID + "}\n"
            snippet += param.name + " is a constant value of\n"
            snippet += "\\begin{equation}\n"
            snippet += "\t\\label{eq:" + param_ID + "}\n"
            snippet += "\t\\boxed{" + latex_ID + " = " + param.human_readable().replace(" ", "~") + "}~.\n"
            snippet += "\\end{equation}\n"

        snippet += ''.join(param.notes)

        return snippet

    def test_snippet(self, ID, test):
        snippet = "\n\\subsubsection{" + ID.replace("_", "~").title() + "}"
        snippet += "\label{sssec:" + ID.replace("_", "-") + "}\n"
        snippet += "\\begin{equation}\n"
        snippet += "\t\\label{eq:" + ID.replace("_", "-") + "}\n"
        snippet += "\t\\boxed{" + self._param2latex(test.expression) + "}~.\n"
        snippet += "\\end{equation}\n"

        snippet += ''.join(test.notes)

        return snippet

    def export_pdf(self, display_IDs={}, design=""):

        if not display_IDs:
            display_IDs = {}
            for ID, param in self.parameters.items():
                if design:
                    if param.model == design:
                        display_IDs[param.id] = param
                else:
                    display_IDs[param.id] = param
            
            if design:
                for key, entry in self.submodels.items():
                    if 'model' in entry:
                        submodel = entry['model']
                    else:
                        submodel_path = entry['path']
                        submodel = self._retrieve_model(submodel_path)
                        if isinstance(submodel, ModelError):
                            submodel.throw(self, "(line " + str(entry['line_no']) + ") " + entry['line'] + "- " +
"Could not find submodel in path: " + submodel_path)

                    for ID, param in submodel.parameters.items():
                        if design:
                            if param.model == design:
                                display_IDs[param.id + "." + key] = param
                        else:
                            display_IDs[param.id] = param

                for i, test in enumerate(self.tests):
                    if test.model == design:
                        display_IDs["test_" + str(i)] = test

            else:
                for i, test in enumerate(self.tests):
                    display_IDs["test_" + str(i)] = test

        elif isinstance(display_IDs, str):
            display_IDs = {display_IDs: self.parameters[display_IDs]}
        elif isinstance(display_IDs, list):
            if display_IDs:
                display_IDs = {ID: self.parameters[ID] for ID in display_IDs}
            else:
                display_IDs = {}
        else:
            raise TypeError("display_IDs must be a list of strings or a string.")

        # Build the document
        with open("preamble.tex", "r") as preamble:
            document = "%"*30 + "\n"
            document += "%"*10 + " Preamble Start " + "%"*10 + "\n"
            document += "%"*30 + "\n"
            document += preamble.read() + "\n"
            document += "%"*30 + "\n"
            document += "%"*10 + " Preamble End " + "%"*10 + "\n"
            document += "%"*30 + "\n"

        document += "\n\\begin{document}\n\n" 

        with open("body.tex", "r") as body:
            document += "%"*30 + "\n"
            document += "%"*10 + " Body Start " + "%"*10  + "\n"
            document += "%"*30 + "\n"
            document += body.read()  + "\n"
            document += "%"*30 + "\n"
            document += "%"*10 + " Body End " + "%"*10 + "\n" 
            document += "%"*30 + "\n"

        document += "\n\\section{" + self.name.replace("_", " ").title() + " Model}"
        document += "\\label{sec:" + self.name.replace("_", "-") + "}\n\n"
        document += "%"*30 + "\n"
        document += "%"*10 + " Parameter Note Start " + "%"*10 + "\n" 
        document += "%"*30

        # Add the parameter and test snippets
        section = list(self.parameters.values())[0].section.replace(" ", "-")
        if section == "":
            document += "\n\n\\subsection{Parameters}"
            document += "\\label{ssec:parameters}\n\n"
        else:
            document += "\n\n\\subsection{" + section + "}"
            document += "\n\\label{ssec:" + section + "}\n\n"

        for ID, entry in display_IDs.items():
            if entry.section != section:
                section = entry.section
                document += "\n\n\\subsection{" + section + "}"
                document += "\\label{ssec:" + section.lower().replace(" ", "-") + "}\n\n"
            if isinstance(entry, Parameter):
                document += self.param_snippet(ID, entry)
            elif isinstance(entry, Test):
                document += self.test_snippet(ID, entry)

        if "\cite" in document:
            document += "\n\printbibliography\\end{document}"
        else:
            document += "\n\\end{document}"
        
        with open("export.tex", "w") as export:
            # Write the output file.
            export.write(document)

        # Compile the output file using pdflatex => biblatex => pdflatex 2X.
        for i in (1, 2):
            os.system("pdflatex -synctex=1 -interaction=nonstopmode export.tex")
            os.system("biber export")
            os.system("pdflatex -synctex=1 -interaction=nonstopmode export.tex")
        os.system("rm export.aux export.log export.bbl export.blg export.bcf export.run.xml export.synctex.gz")

    def dependents(self, search_IDs):
        for search_ID in search_IDs:
            # Print all parameters that depend on the given parameter.
            dependents = []
            for ID, parameter in self.parameters.items():
                if search_ID in parameter.args: dependents.append(ID)
            if dependents: 
                print(dependents)
            elif search_ID not in self.parameters.keys():
                IDError(self, search_ID, "Could not find parameter in model.")
            else: 
                print(search_ID + " has no dependents.")

    def _reset(self):
        for ID, parameter in self.parameters.items():
            if not parameter.independent: 
                parameter.min = parameter.max = None
                if parameter.piecewise:
                    for piece in parameter.equation:
                        for part in piece:
                            if not part.independent:
                                part.min = part.max = None
                                if part.minmax_equation: # support for minmax equations in piecewise functions
                                    for extreme in part.equation:
                                        if not extreme.independent:
                                            extreme.min = extreme.max = None
                elif parameter.minmax_equation:
                    for extreme in parameter.equation:
                        if not extreme.independent:
                            extreme.min = extreme.max = None

    def _reset_recursively(self):
        self._reset()
        for key, entry in self.submodels.items():
            if 'model' in entry.keys():
                entry['model']._reset_recursively()

    def calculate(self, quiet=False):
        # Calculate imports
        for key, entry in self.submodels.items():
            if 'model' in entry.keys():
                entry['model'].calculate(quiet=True)

        # Calculate dependent parameters.
        self._calculate_recursively(self.parameters)
        if not quiet: self.summarize()
        self.calculated = True

    def eval(self, expression):
        # Make a dict of calculation parameters from the submodels
        submodel_parameters = {}
        expression_args = [x for x in re.findall("(?!\d+)\w+\.?\w*", expression) if x not in FUNCTIONS]
        
        for f, pf in FUNCTIONS.items():
            expression = re.sub(r"(?<!\w)" + re.escape(f), re.escape(pf), expression)

        for k, v in OPERATOR_OVERRIDES.items():
            expression = expression.replace(k, v)
            
        for i, arg in enumerate(expression_args):
            if "." in arg:
                parameter_ID, submodel_ID = arg.split(".")

                if submodel_ID in self.submodels:
                    submodel = self.submodels[submodel_ID]
                else:
                    ModelError(submodel_ID, source=["interpreter eval"]).throw(self, "Submodel ID \"" + submodel_ID + "\" not found.")
                path = copy.copy(submodel['path'])
                
                prefix = '_'.join(path) + "_"
                result = self._retrieve_parameter(parameter_ID, path)
                if isinstance(result, ModelError):
                    result.throw(self, "Couldn't find parameter " + parameter_ID + ". Invalid in submodel path " + str(submodel['path']) + ".")
                elif isinstance(result, Parameter):
                    submodel_parameters[prefix + parameter_ID] = result
                elif isinstance(result, (int, float, str, np.int64, np.float64, np.float32, np.float16)):
                    return result
                else:
                    ParameterError(self, expression, "Eval failed.").throw(self, "(in interpreter) Eval failed.")
                expression = re.sub(r"(?<!\w)" + re.escape(arg), re.escape(prefix + parameter_ID), expression)

        eval_params = self.parameters | submodel_parameters | self.constants

        try:
            result = eval(expression, globals(), eval_params)
        except Exception as e:
            IDError(self, expression, str(e))

        if isinstance(result, Parameter):
            if result.error:
                result.error.throw(self, "(in interpreter) Eval failed.", debug=True)
            else:
                return result
        elif isinstance(result, (bool, np.bool_)):
            # Convert numpy bools to python bools
            return bool(result)
        elif isinstance(result, (int, float, str, np.int64, np.float64, np.float32, np.float16)):
            return result
        else:
            ParameterError(self, expression, "Eval failed.").throw(self, "(in interpreter) Eval failed.")

    def test_submodels(self, verbose=True):
        passes = 0

        for submodel_ID in self.submodels:
            
            # Prepare test inputs
            test_inputs = {}
            if self.submodels[submodel_ID]['inputs']:
                for arg, input in self.submodels[submodel_ID]['inputs'].items():
                    if '.' in input:
                        input_ID, source = input.split('.')
                        retrieval_path = copy.copy(self.submodels[source]['path'])

                        result = self._retrieve_parameter(input_ID, retrieval_path)
                        if isinstance(result, ModelError):
                            result.source_message = "Couldn't find parameter " + input_ID + ". Invalid in submodel path " + retrieval_path + "."
                            return result
                        elif isinstance(result, Parameter):
                            test_inputs[arg] = result
                        else:
                            raise TypeError("Invalid result type: " + str(type(result)))

                    elif input in self.parameters:
                        test_inputs[arg] = self.parameters[input]
                    else:
                        return ParameterError(input, "Test input " + input + " for submodel " + submodel_ID + " not found in " + self.name + ".", source=["Model.test_submodels"])
                test_path = copy.copy(self.submodels[submodel_ID]['path'])
                new_passes = self._test_submodel_recursively(test_path, test_inputs, verbose=verbose)
                new_passes = self._test_submodel_recursively(test_path, test_inputs, verbose=verbose) # TODO: Why does this fail the first time sometimes???
                passes += new_passes

        return passes

    def _test_submodel_recursively(self, path, test_params, trail=[], verbose=True):
        passes = 0
        new_trail = copy.copy(trail)
        new_trail.append(self.name)
        if path:
            submodel_name = path.pop(0)
            submodel = [model['model'] for k, model in self.submodels.items() if 'model' in model and model['model'].name == submodel_name]
            if submodel:
                submodel = submodel[0]
                new_passes = submodel._test_submodel_recursively(path, test_params, verbose=verbose)
                new_passes = submodel._test_submodel_recursively(path, test_params, verbose=verbose)
                passes += new_passes
            else:
                return ModelError(submodel_name, "Submodel not found.", new_trail)
        else:
            passes += self.test(test_inputs=test_params, top=False, verbose=verbose)
            return passes

    def test(self, test_inputs={}, verbose=True, top=True):
        if top: 
            passes = self.test_submodels(verbose=verbose)
        else:
            passes = 0

        if not isinstance(passes, int):
            passes.throw(self, "(in Model.test) Submodel test failed.")

        # Eval each test expression, using self.parameters and the reference models
        for test in self.tests:
            test_params = {}
            run_expression = test.expression

            # Only run tests with inputs if inputs are present
            if not any(ref not in test_inputs for ref in test.refs):
                if verbose: print("Test (" + self.name + "): " + run_expression)
                for i, arg in enumerate(test.args):
                    if "." in arg:
                        arg_ID, submodel_ID = arg.split(".")

                        if submodel_ID in self.submodels:
                            submodel = self.submodels[submodel_ID]
                        else:
                            ModelError(submodel_ID, source=["interpreter eval"]).throw(self, "(in Model.test) Submodel ID \"" + submodel_ID + "\" not found.")

                        path = copy.copy(submodel['path'])
                        
                        prefix = '_'.join(path) + "_"

                        result = self._retrieve_parameter(arg_ID, path)
                        if isinstance(result, ModelError):
                            result.throw(self, "Couldn't find parameter " + arg_ID + ". Invalid in submodel path " + submodel['path'] + ".")
                        elif isinstance(result, Parameter):
                            test_params[prefix + arg_ID] = result
                        else:
                            raise TypeError("Invalid result type: " + str(type(result)))

                        run_expression = run_expression.replace(arg, prefix + arg_ID)
                    elif arg in test_inputs:
                        test_params[arg] = test_inputs[arg]
                    elif arg not in FUNCTIONS.values() and not any([arg in v for v in OPERATOR_OVERRIDES.values()]) and not arg in self.constants:
                        test_params[arg] = self.parameters[arg]


                test_params = test_params | self.constants

                if test.trace == True:
                    print("Breakpoint for test: " + test.expression)
                    
                    import pdb

                    breakpoint()
                    eval(run_expression, globals(), test_params)

                calculation = eval(run_expression, globals(), test_params)

                if isinstance(calculation, Parameter):
                    if calculation.error:
                        calculation.error.throw(self, "Test \"" + test.expression + " from model " + self.name + " failed to calculate.")
                    else:
                        raise ValueError("Test expression returned a parameter without an error. That shouldn't happen. Parameters are vessels for errors when it comes to comparison operators right now. I know...it's dumb and needs to be fixed.")
                elif isinstance(calculation, (bool, np.bool_)):
                    result = bcolors.OKGREEN + "pass" + bcolors.ENDC if calculation else bcolors.FAIL + "fail" + bcolors.ENDC

                if verbose: print("\tResult: " + str(result))
                if result == "fail" and verbose:
                    # Print the args and values
                    for k, v in test_params.items():
                        print("\t" + v.__repr__())
                else:
                    passes += 1
            elif verbose:
                print("Test (" + self.name + "): " + test.expression + " SKIPPED")

        return passes


    def compare(self, alternate_design_file, parameter_IDs):
        alternate = copy.deepcopy(self)
        alternate._reset()
        alternate.overwrite(alternate_design_file)
        alternate.calculate(quiet=True)

        table = BeautifulTable()
        table.columns.header = ["Parameter", self.design, alternate.design, self.design + "\n" + "-"*max(len(self.design), len(alternate.design)) + "\n" + alternate.design]
        table.columns.align = "rccc"
        self_parameters = {ID: self.parameters[ID] for ID in parameter_IDs}
        alternate_parameters = {ID: alternate.parameters[ID] for ID in parameter_IDs}
        for ID in parameter_IDs:
            table.append_row([ID, str(self_parameters[ID]), str(alternate_parameters[ID]), str(self_parameters[ID].max / alternate_parameters[ID].max)])

        print(table)

    def users(self, parameter_ID):
        users = [param for ID, param in self.parameters.items() if parameter_ID in param.args]
        for p in users: print(p.id)

    def all(self):
        self.tree(list(self.parameters.keys()), levels=0, verbose=True, turtles=False)

    def summarize(self, sigfigs=4, verbose=False):
        print("-" * 80)
        print(bcolors.OKBLUE + "Model: " + self.name + bcolors.ENDC)
        print(bcolors.OKGREEN + "Design: " + self.design + bcolors.ENDC)
        print("Parameters: " + str(len(self.parameters) + len(self.constants)) 
        + " (" + str(len([p for ID, p in self.parameters.items() if p.independent])) + " independent, " 
        + str(len([p for ID, p in self.parameters.items() if not p.independent])) + " dependent, "
        + str(len(self.constants)) + " constants)")
        print("Tests: " + str(self.test(verbose=False)) + "/" + str(len(self.tests)))
        print("-" * 80)

        summary_parameters = list[self.parameters.keys()] if verbose else [k for k, v in self.parameters.items() if v.performance]
        self.tree(summary_parameters, sigfigs=sigfigs, verbose=verbose, levels=0, turtles=False)

    def tree(self, parameter_IDs=[], indent=0, sigfigs=4, levels=3, verbose=False, up=False, turtles=True):
        if isinstance(parameter_IDs, str):
            if parameter_IDs == "performance": 
                parameter_IDs = [ID for ID, param in self.parameters.items() if param.performance]
            else: 
                parameter_IDs = [parameter_IDs]
        if isinstance(parameter_IDs, list):
            if not parameter_IDs and verbose: parameter_IDs = list(self.parameters.keys())
            self._tree_recursively(parameter_IDs, indent=indent, sigfigs=sigfigs, levels=levels, verbose=verbose, up=up, turtles=turtles)
        else:
            raise TypeError("parameter_IDs must be a string or list.")

    def _tree_recursively(self, parameter_IDs=[], indent=0, sigfigs=4, levels=12, verbose=False, up=False, trail=[], turtles=True, submodel_id=""):
        if indent < levels:
            for parameter_ID in parameter_IDs:
                parameter = self.parameters[parameter_ID]
                if indent == 0:
                    parameter.hprint(sigfigs)
                else:
                    parameter.short_print(sigfigs, indent=indent * 4, verbose=verbose, submodel_id=submodel_id)

                # For dependent parameters, continue the recursion
                if parameter.equation:
                    arg_params = []
                    print("    " * indent + "=", end="")
                    if parameter.callable:
                        header_side = "-"*((80 - len(str(parameter.equation.__name__))) // 2)
                        print(" " + header_side + parameter.equation.__name__ + header_side)
                        code = " " + inspect.getsource(parameter.equation)
                        code = "    " * indent + " |" + code
                        code = code.replace("\n", "\n" + "    " * (indent) + " |")
                        print(str(code))
                        print(" " + "    " * indent + "-" * 80)
                        arg_params = [arg for arg in parameter.args if arg in self.parameters]
                    elif parameter.piecewise:
                        print("{" + str(parameter.equation[0][0].equation)  + " if " + str(parameter.equation[0][1].equation), end="")
                        for i, piece in enumerate(parameter.equation):
                            if i > 0:
                                if piece[0].equation:
                                    print("    " * (indent) + " {" + str(piece[0].equation)  + " if " + str(piece[1].equation), end="")
                                else:
                                    print("    " * (indent) + " {" + str(piece[0])  + "if " + str(piece[1].equation), end="")
                                
                            if piece[1].min and piece[1].max:
                                arg_params = [arg for arg in piece[0].args if arg in self.parameters]
                                print(" <------")
                            else:
                                print("")
                                
                    elif parameter.minmax_equation:
                        print(str(parameter.equation[0].equation) + " | " + str(parameter.equation[1].equation))
                        arg_params = [arg for arg in parameter.args if arg in self.parameters]
                    else:
                        print(f"{parameter.equation}")
                        arg_params = [arg for arg in parameter.args if arg in self.parameters]
            
                    # Update the trail to catch circular dependencies
                    new_trail = copy.copy(trail)

                    parameter_trail_id = f"{parameter.id}.{submodel_id}" if submodel_id else parameter.id

                    if parameter_trail_id in trail:
                        ParameterError(parameter, "", source=["Model._tree_recursively"]).throw(self, "Circular dependency found in path: " + "=>".join(trail) + ".")

                    if new_trail:
                        if new_trail[-1] != parameter_trail_id:
                            new_trail.append(parameter_trail_id)
                    else:
                        new_trail = [parameter_trail_id]
                            
                    # Recursively continue tree for args in model self
                    self._tree_recursively(arg_params, indent + 1, levels=levels, verbose=verbose, trail=new_trail, submodel_id=submodel_id)
                    [print("    " * (indent + 1) + arg + ": " + str(self.constants[arg])) for arg in parameter.args if arg in self.constants]

                    # Recursively continue tree for args in self's submodels
                    # The submodel key is given in the form "parameter.submodel"
                    for arg in arg_params:
                        if "." in arg:
                            submodel = self._retrieve_model(self.submodels[arg.split(".")[1]]['path'])
                            submodel._tree_recursively([arg.split(".")[0]], indent + 1, levels=levels, verbose=verbose, trail=new_trail, submodel_id=arg.split(".")[1])
        else:
            [self.parameters[ID].short_print(sigfigs, indent=indent * 4, verbose=verbose) for ID in parameter_IDs if ID in self.parameters | self.constants]
            if any([self.parameters[ID].args for ID in parameter_IDs]) and turtles: print("    " * indent + "🐢🐢🐢")

    def _calculate_recursively(self, parameters, trail=[]):
        for parameter in parameters.values():
            if isinstance(parameter, Parameter) and any([parameter.min is None, parameter.max is None]):
                
                # Update the trail for recursive errors.
                new_trail = copy.copy(trail)

                if new_trail:
                    if new_trail[-1] != parameter.id:
                        new_trail.append(parameter.id)
                else:
                    new_trail = [parameter.id]

                if parameter.piecewise:
                    piece_parameters = {}
                    for i, piece in enumerate(parameter.equation):
                        piece_parameters.update({piece[0].id + str(i): piece[0], piece[1].id + str(i): piece[1]})
                    self._calculate_recursively(piece_parameters, new_trail)
                elif parameter.minmax_equation:
                    minmax_equation_parameters = {}
                    for i, eq in enumerate(parameter.equation):
                        minmax_equation_parameters.update({eq.id + str(i): eq})
                    self._calculate_recursively(minmax_equation_parameters, new_trail)
                else:
                    if not parameter.args:
                        raise ValueError(f"In Model._calculate_recursively for model {self.name}: Parameter has no args nor a set value.")

                    # Calculate any model parameters that haven't been calculated yet
                    arg_parameters = {arg: self.parameters[arg] for arg in [x for x in parameter.args if x in self.parameters]}

                    if not all([True if all([parameter.min, parameter.max]) else False for arg, parameter in arg_parameters.items()]):
                        calc_args = {k: self.parameters[k] for k in [x for x in parameter.args if x in self.parameters] if "." not in k}
                        
                        if parameter.id in trail:
                            ParameterError(parameter, "", source=["Model._calculate_recursively"]).throw(self, "Circular dependency found in path: " + "=>".join(trail) + ".")
                        
                        self._calculate_recursively(calc_args, new_trail)

                    # Make a dict of calculation parameters from the submodels
                    expression = parameter.equation
                    submodel_parameters = {}
                    calc_args = []
                    for i, arg in enumerate(parameter.args):
                        if "." in arg:
                            parameter_ID, submodel_ID = arg.split(".")

                            if submodel_ID in self.submodels:
                                submodel = self.submodels[submodel_ID]
                            else:
                                ModelError(submodel_ID, source=["interpreter eval"]).throw(self, "(in Model._calculate_recursively) Submodel ID \"" + submodel_ID + "\" not found.")

                            path = copy.copy(submodel['path'])
                            
                            prefix = '_'.join(path) + "_"
                            
                            result = self._retrieve_parameter(parameter_ID, path)
                            if isinstance(result, ModelError):
                                result.throw(self, "Couldn't find parameter " + parameter_ID + ". Invalid in submodel path " + str(submodel['path']) + ".")
                            elif isinstance(result, Parameter):
                                submodel_parameters[prefix + parameter_ID] = result
                            else:
                                raise TypeError("Invalid result type: " + str(type(result)))

                            calc_args.append(prefix + parameter_ID)
                            if not parameter.callable:
                                expression = re.sub(r"(?<!\w)" + re.escape(arg), re.escape(prefix + parameter_ID), expression)
                        else:
                            calc_args.append(arg)

                # Calculate the parameter
                calculation = None
                if parameter.pointer:
                    calculation = self.parameters[parameter.equation]
                elif parameter.piecewise:
                    for piece in parameter.equation:
                        if piece[1].min and piece[1].max:
                            calculation = piece[0]
                    if not calculation:
                        ParameterError(parameter, "No piecewise condition was met.", source=["Model._calculate_recursively"]).throw(self, "Parameter \"" + parameter.id + "\" (line " + str(parameter.line_no) + " from model " + parameter.model + ") failed to calculate.\n\"" + parameter.line.strip() + "\"" + "\n" + str(parameter.equation))
                elif parameter.minmax_equation:
                    calculation = (parameter.equation[0].min, parameter.equation[1].max)
                else:
                    calculation = parameter.calculate(expression, globals(), self.parameters | submodel_parameters | self.constants, calc_args)                    
                    if isinstance(calculation, Parameter) and calculation.error:
                        model = None if not parameter.model else parameter.model
                        calculation.error.throw(self, f"Parameter \"{parameter.id}\" (line {parameter.line_no} from model {model}) failed to calculate.\n{parameter.line}", debug=True)
                
                parameter.assign(calculation)
                if parameter.error:
                    parameter.error.throw(self, "Failed to calculate parameter \"" + parameter.id + "\" (line " + str(parameter.line_no) + " from model " + str(parameter.model), debug=True)


    # Recursively retrieve a parameter from a submodel or submodel of a submodel, etc.
    def _retrieve_parameter(self, parameter_ID, path, trail=[]):
        new_trail = copy.copy(trail)
        new_trail.append(self.name)
        if path:
            submodel_name = path.pop(0)
            submodel = [model['model'] for k, model in self.submodels.items() if 'model' in model and model['model'].name == submodel_name]
            if submodel:
                submodel = submodel[0]
                return submodel._retrieve_parameter(parameter_ID, path, trail)
            else:
                return ModelError(parameter_ID, "Submodel not found.", new_trail)
        else:
            if parameter_ID in self.parameters:
                return self.parameters[parameter_ID]
            else:
                return ModelError(parameter_ID, "Parameter not found.", new_trail)

    # Recursively retrieve a model from a submodel or submodel of a submodel, etc.
    def _retrieve_model(self, path, trail=[]):
        new_trail = copy.copy(trail)
        new_trail.append(self.name)
        path = copy.copy(path)
        if path:
            submodel_name = path.pop(0)
            submodels = [model['model'] for k, model in self.submodels.items() if 'model' in model and model['model'].name == submodel_name]
            if submodels:
                submodels = submodels[0]
                return submodels._retrieve_model(path, trail)
            else:
                return ModelError(submodel_name, "Submodel not found.", new_trail)
        else:
            return self

    # Recursively rewrite a parameter from a submodel or submodel of a submodel, etc.
    def _rewrite_parameter(self, parameter_ID, parameter, path, trail=[]):
        new_trail = copy.copy(trail)
        new_trail.append(self.name)
        if path:
            submodel_name = path.pop(0)
            submodel = [model['model'] for k, model in self.submodels.items() if 'model' in model and model['model'].name == submodel_name]
            if submodel:
                submodel = submodel[0]
                return submodel._rewrite_parameter(parameter_ID, parameter, path, new_trail)
            else:
                return ModelError(parameter_ID, "Submodel not found.", trail)
        else:
            self.parameters[parameter_ID].write(parameter)
            if self.parameters[parameter_ID].error:
                return self.parameters[parameter_ID]
            return True

def handler(model:Model, inpt):
    args = inpt.split(" ")
    cmd = args.pop(0)
    opt_list = [arg for arg in args if "=" in arg]
    opts = {}
    for arg in opt_list:
        args.remove(arg)
        arg_value = arg.split("=")[1]
        arg_value = float(arg_value) if isfloat(arg_value) else arg_value
        opts[arg.split("=")[0]] = arg_value
    
    if cmd == "tree":
        model.tree(args, **opts)
    elif cmd == "summarize":
        model.summarize()
    elif cmd == "all":
        model.all()
    elif cmd == "dependents":
        model.dependents(args)
    elif cmd == "design":
        if any([arg for arg in args if "." in arg and ".on" not in arg]):
            print("Only .on files are allowed.")
            interpreter(model)
        if model.name in [arg.strip(".on") for arg in args]:
            print("Cannot overwrite model with itself.")
            interpreter(model)
        args = [arg if "." in arg else arg + ".on" for arg in args]
        model.overwrite(args)
    elif cmd == "test":
        model.test()
    elif cmd == "export":
        model.export_pdf(args)
    elif cmd == "load":
        loader(inpt.split(" "))
    elif cmd == "help":
        print(help_text)
        interpreter(model)
    elif cmd == "quit":
        sys.exit()
    elif cmd == "quit()":
        sys.exit()
    elif cmd == "exit":
        sys.exit()
    else:
        print(model.eval(inpt))

help_text = """
Commands:
    tree [param 1] [param 2] ... [param n]
        Print a tree for the entire model or just for the specified parameters.

    summarize
        Print a summary of the model.

    all
        Print all parameters.

    dependents [param 1] [param 2] ... [param n]
        Print all parameters that depend on the specified parameters.

    design [design 1] [design 2] ... [design n]
        Overwrite the current model with the specified designs.

    test
        Run all tests on the model and any loaded designs.

    export [param 1] [param 2] ... [param n]
        Export the entire model to a PDF file or just the specified parameters.

    load model
        Load a new model (starting over from scratch).

    help
        Print this help text.

    quit
    quit()
    exit
        Exit the program.
"""

def loader(args=[]):

    if len(args) > 0:
        inp = args[0]
    else:
        inp = ""
    model = None

    while not model:
        if inp:
            if inp == "help":
                print(loader_help)
                inp = ""
                continue
            if inp == "quit" or inp == "quit()" or inp == "exit":
                sys.exit()
            if "." not in inp: 
                inp += ".on"
            else:
                if ".on" not in inp:
                    print("Only .on files are allowed.")
                    inp = ""
                    continue
            if os.path.exists(inp):
                print("Loading model " + inp + "...")
                model = Model(inp)
                model.calculate()
            else:
                print("Model " + inp + " not found.")
                inp = ""
                continue
        else:
            inp = input("Enter a model: ")

    for arg in args[1:]: # Handle commands after the first as cli commands. 
        print("(" + bcolors.OKBLUE + model.name + bcolors.ENDC + ") >>> " + arg)
        handler(model, arg)
    interpreter(model)

loader_help = """"
    Commands:
        [model-name]
        [model-name].on [args]
            Load a model. [args] after the model name are run as seperate cli args

        help
            Print this help text.

        quit
        quit()
        exit
            Exit the program.

    You are in the loader. To access other commands, you need to load a model. For more information, see the README.
"""

def interpreter(model):
    while True:
        if model.design == "default":
            handler(model, input(f"({bcolors.OKBLUE}{model.name}{bcolors.ENDC}) >>>"))
        else:
            handler(model, input(f"({bcolors.OKGREEN}{model.design}@{bcolors.ENDC}{bcolors.OKBLUE}{model.name}{bcolors.ENDC}) >>>"))

def debugger(model):
    print("Enterring debug mode. Type 'quit' to exit.")
    while True:
        handler(model, input(f"{bcolors.FAIL}debugger{bcolors.ENDC} ({bcolors.OKBLUE}{model.name}{bcolors.ENDC}) >>>"))
    
def main(args=sys.argv[1:]):
    print("Oneil " + __version__)
    print("Type 'help' for a list of commands or see the README for more information.")
    print("-"*80)
    loader(args)