import re
import numpy as np
import inspect
from pytexit import py2tex
import os, sys
import copy
from beautifultable import BeautifulTable
import importlib
from functools import partial

from . import bcolors
from . import errors as err
from . import console
from . import units as un
from .errors import OneilError

np.seterr(all='raise')

def isfloat(num):
    try:
        float(num)
        return True
    except ValueError:
        return False

FUNCTIONS = {"sin": "par_sin", "cos": "par_cos", "tan": "par_tan", "asin": "par_asin", "acos": "par_acos", "atan": "par_atan", "sinh": "par_arcsinh", "cosh": "par_cosh", "tanh": "par_tanh", "min": "par_min", "max": "par_max", "sqrt": "par_sqrt", "abs": "par_abs", "mnmx": "par_minmax", "log": "par_log", "log2": "par_log2", "log10": "par_log10", "ln": "par_log", "floor": "par_floor", "ceiling": "par_ceiling", "extent": "par_extent", "range": "par_range", "strip": "par_strip", "mid": "par_mid", "sign": "par_sign"}

MATH_CONSTANTS = {"pi": np.pi, "e": np.exp(1), "inf": np.inf}

EQUATION_OPERATORS = ["+", "-", "*", "/", "//", "%", "(", ")", "=", "<", ">", "!"]

OPERATOR_OVERRIDES = {"--": "|minus|", "^": "**"}

BOOLEAN_OPERATORS = ["and", "or", "not"]

OPERATORS = EQUATION_OPERATORS + BOOLEAN_OPERATORS + list(OPERATOR_OVERRIDES.keys())

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
@err.add_trace
def minus(x, y):
    if isinstance(x, Parameter):
        return x._minus(y)
    elif isinstance(y, Parameter):
        return y._rminus(x)
    else:
        return x - y
    

def parse_file(file_name):
    parameters = []
    submodels = {}
    imports = []
    tests = []
    note = ""
    prev_line = ''
    design_overrides = {}
    last_line_blank = False
    section = ""

    with open(file_name, 'r') as f:
        final_line = 0
        for i, line in enumerate(f.readlines()):
            unit_fx = lambda x:x
            final_line = i
            if line == '\n':
                last_line_blank = True
                continue
            elif '#' in line and line.strip()[0] == '#':
                last_line_blank = False
                continue
            elif line[0] == '\t' or line[0:1] == ' ':
                if last_line_blank: line = "\n\n" + line
                if line.strip() and line.strip()[0] == '{':
                    arguments = []
                    parameter, arguments = parse_piecewise(line, parameters[-1].units, parameters[-1].id, imports, file_name.replace(".on", ""), i+1, unit_fx, pointer=parameters[-1].pointer)
                    parameters[-1].add_piece(parameter, arguments)
                else:
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
                try:
                    assert(re.search(r"^use\s+\w+(\(.+=.+\))?\s+as\s+\w+\s*$", line))
                except:
                    raise SyntaxError(file_name, i+1, line, "Use includes must be of the form \"use <model> as <symbol>\"")
                
                last_line_blank = False
                include = line.replace("use", "")
                model = include.split('as')[0].strip()

                if '(' in model:
                    test_inputs = {l.split('=')[0].strip():l.split('=')[1].strip() for l in model.split('(')[1].split(')')[0].split(',')}
                    model = model.split('(')[0].strip()
                else:
                    test_inputs = {}

                if not os.path.exists(model + ".on"):
                    raise ModelLoadingError(file_name, i + 1, f"File \"{model}.on\" does not exist.")
                symbol = include.split('as')[1].strip()

                if symbol in submodels.keys():
                    raise ModelLoadingError(file_name, i + 1, f"Submodel symbol \"{symbol}\" has duplicate definitions.")

                submodels[symbol] = {'model': Model(model + ".on"), 'inputs': test_inputs, 'path': [model], 'line_no': i+1, 'line': line}
            elif line[:5] == 'from ':
                try:
                    assert(re.search(r"^from\s+\w+(\.\w+)*\s+use\s+\w+(\(.+=.+\))?\s+as\s+\w+\s*$", line))
                except:
                    raise SyntaxError(file_name, i+1, line, "From includes must be of the form \"from <source> use <model> as <symbol>\"")

                last_line_blank = False
                include = line.replace("from", "")
                source = include.split('use')[0].strip()
                model = include.split('use')[1].split("as")[0].strip()

                if '(' in model:
                    test_inputs = {l.split('=')[0].strip():l.split('=')[1].strip() for l in model.split('(')[1].split(')')[0].split(',')}
                    model = model.split('(')[0].strip()
                else:
                    test_inputs = {}

                if not os.path.exists(model + ".on"):
                    raise ModelLoadingError(file_name, i + 1, f"File \"{model}.on\" does not exist.")

                path = source.split('.') + [model] if '.' in source else [source, model]
                symbol = include.split('use')[1].split("as")[1].strip()

                if symbol in submodels.keys():
                    raise ModelLoadingError(file_name, i + 1, f"Submodel symbol \"{symbol}\" has duplicate definitions.")

                submodels[symbol] = {'path': path, 'inputs': test_inputs, 'line_no': i+1, 'line': line}
            elif line[:7] == 'import ':
                try:
                    assert(re.search(r"^import\s+\w+\s*$", line))
                except:
                    raise SyntaxError(file_name, i+1, line, "Python imports must be of the form \"import <module>\"")
                
                last_line_blank = False
                sys.path.append(os.getcwd())
                module = line.replace("import", "").strip()

                try:
                    imports.append(importlib.import_module(module))
                except Exception as e:
                    raise ImportError(file_name, i+1, line, module + ".py", e)

            elif line[:8] == 'section ':
                try:
                    assert(re.search(r"^section\s+[\w\s]*$", line))
                except:
                    raise SyntaxError(file_name, i+1, line, "Sections must be of the form \"section <name>\" where <name> is only word characters and whitespace.")
                
                last_line_blank = False
                section = line.replace("section", "").strip()
            elif line[0:4] == 'test' or line.replace(" ", "").replace("\t", "")[0:5] == '*test':
                try:
                    assert(re.search(r"^(\*{1,2}\s*)?test\s*(\{\w+(,\s*\w+)*\})?:.*$", line))
                except:
                    raise SyntaxError(file_name, i+1, line, "Tests must be of the form \"test {<input 1>, <input 2>, ... ,<input n>}: <expression>\" where {<input 1>, <input 2>, ... ,<input n>} is optional, each <input> consists of word characters only, and <expression> is a valid python expression with valid parameters and constants.")
                
                last_line_blank = False
                tests.append(Test(line, i+1, file_name.replace(".on", ""), section=section))
                prev_line = 'test'
            elif re.search(r"^(\*{1,2}\s*)?\w+(\.\w+)?\s*=>?[^:]+(:.*)?$", line):
                last_line_blank = False

                id, equation, arguments, units, unit_fx, hrunits, pointer = parse_body(line.split(":"), line, i+1, file_name.replace(".on", ""), imports)
                isdiscrete = True if not pointer and isinstance(equation, str) else False
                options = [equation] if not pointer and isinstance(equation, str) else None
                design_overrides[id] = Parameter(equation, units, id, hr_units=hrunits, model=file_name.replace(".on", ""), line_no=i+1, line=line, name=f"{id} from {file_name}", options=options, section=section, pointer=pointer)
                
                prev_line='design'
            elif re.search(r"^[^\s]+[^:]*:\s*\w+\s*=[^:]+(:.*)?$", line):
                last_line_blank = False
                parameter, unit_fx = parse_parameter(line, i+1, file_name.replace(".on", ""), imports, section)
                parameters.append(parameter)
                prev_line = 'param'
            else:
                raise SyntaxError(file_name, i+1, line, "Invalid syntax.")

        params = {p.id: p for p in parameters}

        if not params and not tests and not design_overrides:
            raise ModelLoadingError(file_name, final_line, "Empty model. No parameters, design values, or tests found.")

        return note, params, submodels, tests, design_overrides

def parse_parameter(line, line_number, file_name, imports, section=""):
    trace = False

    if line[0] == '$':
        performance = True
        line = line[1:].strip()
    else:
        performance = False

    if line[0] == '*':
        if line[1] == '*':
            import pdb;
            breakpoint
        trace = True
        line = line[1:].strip()

    # Split into preamble and body around the first colon
    preamble = line.split(':')[0]
    body = line.split(':')[1:]

    # Parse the body
    id, equation, arguments, units, unit_fx, hrunits, pointer = parse_body(body, line, line_number, file_name, imports)

    # Parse the preamble
    if '(' and ')' in preamble:
        name = preamble.split('(')[0].strip()
        limits = []
        for l in preamble.replace(" ", "").split('(')[1].split(')')[0].split(','):
            if l.replace('.','').isnumeric():
                limits.append((unit_fx)(float(l)))
            elif l in MATH_CONSTANTS:
                limits.append((unit_fx)(MATH_CONSTANTS[l]))
            elif any(character in EQUATION_OPERATORS + list(OPERATOR_OVERRIDES.keys()) for character in l):
                try:
                    limits.append((unit_fx)(eval(l, MATH_CONSTANTS)))
                except ZeroDivisionError as e:
                    raise DivideByZeroError((file_name, line_number))
            else:
                raise SyntaxError(file_name, line_number, line, "Parse parameter: invalid limit: " + l)
        options = tuple(limits)
    elif '[' and ']' in preamble:
        name = preamble.split('[')[0].strip()
        options = preamble.replace(" ", "").split('[')[1].split(']')[0].split(',')
    else:
        name = preamble
        options = (0, np.inf)

    if not name:
        raise SyntaxError(file_name, line_number, line, "Parse parameter: name cannot be empty.")

    return Parameter(equation, units, id, hr_units=hrunits, model=file_name, line_no=line_number, line=line, name=name, options=options, arguments=arguments, trace=trace, section=section, performance=performance, pointer=pointer), unit_fx

def parse_body(body, line, line_number, file_name, imports):
    
    if '=>' in body[0]:
        id = body[0].split('=>')[0].strip()
        assignment = "=".join(body[0].split('=>')[1:]).strip()
        pointer = True
    else:
        id = body[0].split('=')[0].strip()
        assignment = "=".join(body[0].split('=')[1:]).strip()
        pointer=False
        
    
    if len(body) > 1:
        hrunits = body[1].strip("\n").strip()
        test=1
        try:
            units, unit_fx = un.parse(hrunits)
        except Exception as e:
            raise UnitParseError(file_name, line_number, hrunits)
    elif len(body) > 2:
        raise SyntaxError(file_name, line_number, line, "Parse parameter: too many colons.")
    else: 
        units = {}
        unit_fx = lambda x:x
        hrunits=''

    equation, arguments = parse_equation(assignment, units, id, imports, file_name, line_number, unit_fx, pointer=pointer)

    return id, equation, arguments, units, unit_fx, hrunits, pointer

def parse_equation(assignment, units, id, imports, file_name, line_number, unit_fx, pointer):
    
    if assignment.strip()[0] == '{':
        equation, arguments = parse_piecewise(assignment, units, id, imports, file_name, line_number, unit_fx, pointer)
        equation = [equation]
    else:
        assignment = assignment.replace(' ', '')
        arguments = []
        mathless_assignment = assignment
        
        if any(op in mathless_assignment for op in MATH_CONSTANTS):
            for x in MATH_CONSTANTS:
                mathless_assignment = mathless_assignment.replace(x, '')

        if re.search('[a-zA-Z]', mathless_assignment):
            if '|' in assignment:
                mineq, minargs = convert_functions(assignment.split('|')[0], imports, file_name, line_number) 
                maxeq, maxargs = convert_functions(assignment.split('|')[1], imports, file_name, line_number)
                equation = (Parameter(mineq, units, id + ":mineq", pointer=pointer), Parameter(maxeq, units, id + ":maxeq", pointer=pointer))
                arguments = minargs + maxargs
            else:
                equation, arguments = convert_functions(assignment, imports, file_name, line_number)
            
        else:
            if '|' in assignment:
                try:
                    min = (unit_fx)(eval((assignment.split('|')[0]), MATH_CONSTANTS))
                    max = (unit_fx)(eval((assignment.split('|')[1]), MATH_CONSTANTS))
                except ZeroDivisionError as e:
                    raise DivideByZeroError((file_name, line_number))
                equation = (min, max)
            else:
                try:
                    equation = (unit_fx)(eval(assignment, MATH_CONSTANTS))
                except ZeroDivisionError as e:
                    raise DivideByZeroError((file_name, line_number))

    return equation, arguments

def parse_piecewise(assignment, units, id, imports, file_name, line_number, unit_fx, pointer=False):
    eargs = []
    cargs = []
    equation = ""
    condition = ""
    if '{' not in assignment: raise SyntaxError(file_name, line_number, assignment, "Missing { from piecewise definition.")
    assignment = assignment.strip().strip('{')
    if 'if' not in assignment: raise SyntaxError(file_name, line_number, assignment, "Missing condition (\"if\") from piecewise definition.")
    equation, eargs = parse_equation(assignment.split('if')[0].strip(), units, id, imports, file_name, line_number, unit_fx, pointer)
    condition, cargs = parse_equation(assignment.split('if')[1].strip(), units, id, imports, file_name, line_number, unit_fx, pointer)
    return (Parameter(equation, units, id + ":eqpiece", pointer=pointer), Parameter(condition, {}, id + ":condpiece")), eargs + cargs

def convert_functions(assignment, imports, file_name, line_number):
    arguments = []
    if isfloat(assignment):
        return float(assignment), arguments
    
    # Regex to find function calls in the form "name(arg1, arg2, ...)"
    pattern = re.compile(r'(\w+)\(([^()]*)\)')
    results = pattern.findall(assignment)

    # If assignment has a function ("name(var1, var2, varn)") in it, replace it with the appropriate callable
    if results:
        func, arg_str = results[0]

    if results and not func in FUNCTIONS:
        equation = []
        arguments = arg_str.split(",")
        for i in imports:
            if func in i.__dict__.keys():
                equation = i.__dict__[func]
                break
        if not equation:
            raise SyntaxError(file_name, line_number, assignment, "Parse parameter: invalid function: " + func)
    else:
        equation = assignment.strip("\n").strip()

    return equation, arguments

# Ensures the val1 is a Parameter for use in the min and max functions.
def _process_minmax_par_inputs(val1, val2):
    if isinstance(val2, Parameter) and not isinstance(val1, Parameter):
        return val2, val1
    else:
        return val1, val2

@err.add_trace
def par_extent(val1, val2=None):
    if not val2:
        if isinstance(val1, Parameter):
            if val1.units == {}:
                return max(abs(val1.min), abs(val1.max))
            else:
             return Parameter(max(abs(val1.min), abs(val1.max)), val1.units, f"extent({val1.name})")
        elif isinstance(val1, (int, float)):
            return val1
        
    val1, val2 = _process_minmax_par_inputs(val1, val2)

    if isinstance(val2, Parameter):
        if val1.id == val2.id:
            return Parameter(max(abs(val1.min), abs(val1.max)), val1.units, "extent({})".format(val1.name))
        
        if val1.units != val2.units:
            raise UnitEvaluationError(f"Cannot compare {un.hr_units(val1.units)} to {un.hr_units(val2.units)} (par_extent).", [val1, val2])

        return Parameter((max(abs(val1.min), abs(val1.max), abs(val2.min), abs(val2.max))), val1.units, "extent({},{})".format(val1.name, val2.name))
    elif isinstance(val2, (int, float)):
        if val1.units != {}:
            raise UnitEvaluationError(f"Cannot compare {un.hr_units(val1.units)} to a unitless number (par_extent).", [val1, val2])
        return Parameter((max(abs(val1.min), abs(val1.max), abs(val2))), val1.units, "extent({},{})".format(val1.name, val2))
    
    raise TypeError("Second input to extent() must be of type Parameter, int, or float.")

@err.add_trace
def par_minmax(val1, val2):
    if val1.units != val2.units:
        raise UnitEvaluationError(f"Cannot compare {un.hr_units(val1.units)} to {un.hr_units(val2.units)} (par_minmax).", [val1, val2])
    return Parameter((min(val1.min, val2.min), max(val1.max, val2.max)), val1.units, "Min/max({},{})".format(val1.name, val2.name))

@err.add_trace
def par_range(val):
    if isinstance(val, Parameter):
        return Parameter(val.max - val.min, val.units, "range({})".format(val.name))
    else:
        raise TypeError("Input to range() must be of type Parameter.")
    
@err.add_trace
def par_mid(val):
    if isinstance(val, Parameter):
        return Parameter((val.max + val.min)/2, val.units, "mid({})".format(val.name))
    else:
        raise TypeError("Input to mid() must be of type Parameter.")
    
@err.add_trace
def par_sign(val):
    if isinstance(val, Parameter):
        return Parameter((np.sign(val.min), np.sign(val.max)), {}, "sign({})".format(val.name))
    else:
        raise TypeError("Input to sign() must be of type Parameter.")
    
@err.add_trace
def par_strip(val):
    if isinstance(val, Parameter):
        return Parameter((val.min, val.max), {}, "strip({})".format(val.name))
    else:
        raise TypeError("Input to strip() must be of type Parameter.")

@err.add_trace
def par_min(val1, val2=None):
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
            raise UnitEvaluationError(f"Cannot compare {un.hr_units(val1.units)} to {un.hr_units(val2.units)} (par_minmax).", [val1, val2])

        return Parameter((min(val1.min, val2.min), min(val1.max, val2.max)), val1.units, "min({},{})".format(val1.name, val2.name))
    elif isinstance(val2, (int, float)):
        if val1.units != {}:
            raise UnitEvaluationError(f"Cannot compare {un.hr_units(val1.units)} to a unitless number (par_minmax).", [val1, val2])
        return Parameter((min(val1.min, val2), min(val1.max, val2)), val1.units, "min({},{})".format(val1.name, val2))
    else:
        raise TypeError("Second input to min() must be of type Parameter, int, or float.")

@err.add_trace
def par_max(val1, val2=None):
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
                raise UnitEvaluationError(f"Cannot compare {un.hr_units(val1.units)} to {un.hr_units(val2.units)} (par_minmax).", [val1, val2])
            return Parameter((max(val1.min, val2.min), max(val1.max, val2.max)), val1.units, "max({},{})".format(val1.name, val2.name))
        elif isinstance(val2, (int, float)):
            if val1.units != {}:
                raise UnitEvaluationError(f"Cannot compare {un.hr_units(val1.units)} to a unitless number (par_minmax).", [val1, val2])
            return Parameter((max(val1.min, val2), max(val1.max, val2)), val1.units, "max({},{})".format(val1.name, val2))
    elif isinstance(val1, (int, float)):
        if isinstance(val2, (int, float)):
            return max(val1, val2)
    else:
        raise TypeError("Inputs to max() must be of type Parameter, int, or float.")

@err.add_trace
def par_sin(val):
    if isinstance(val, Parameter):
        if val.units != {}:
            raise UnitEvaluationError("Input to sine must be unitless.", [val])
        results = [np.sin(val.min), np.sin(val.max)]
        return Parameter((min(results), max(results)), {}, "sin({})".format(val.id))
    elif isinstance(val, (int, float)):
        return Parameter((np.sin(val), np.sin(val)), {}, "sin({})".format(val))
    else:
        raise TypeError("Input to sin() must be of type Parameter, int, or float.")

@err.add_trace
def par_cos(val):
    if isinstance(val, Parameter):
        if val.units != {}:
            raise UnitEvaluationError("Input to cosine must be unitless.", [val])
        results = [np.cos(val.min), np.cos(val.max)]
        return Parameter((min(results), max(results)), {}, "cos({})".format(val.id))
    elif isinstance(val, (int, float)):
        return Parameter((np.cos(val), np.cos(val)), {}, "cos({})".format(val))
    else:
        raise TypeError("Input to cos() must be of type Parameter, int, or float.")

@err.add_trace
def par_tan(val):
    if isinstance(val, Parameter):
        if val.units != {}:
            raise UnitEvaluationError("Input to tangent must be unitless.", [val])
        results = [np.tan(val.min), np.tan(val.max)]
        return Parameter((min(results), max(results)), {}, "tan({})".format(val.id))
    elif isinstance(val, (int, float)):
        return Parameter((np.tan(val), np.tan(val)), {}, "tan({})".format(val))
    else:
        raise TypeError("Input to tan() must be of type Parameter, int, or float.")

@err.add_trace
def apar_sin(val):
    if isinstance(val, Parameter):
        if val.units != {}:
            raise UnitEvaluationError("Input to arcsine must be unitless.", [val])
        if not -1 <= val.min <= 1 or not -1 <= val.max <= 1:
            raise ParameterError("Input to arcsine must be between -1 and 1 (apar_sin).", val)
        results = [np.arcsin(val.min), np.arcsin(val.max)]
        return Parameter((min(results), max(results)), {}, "asin({})".format(val.id))
    elif isinstance(val, (int, float)):
        if not -1 <= val <= 1:
            raise ParameterError("Input to arcsine must be between -1 and 1 (apar_sin).", val)
        return Parameter((np.arcsin(val), np.arcsin(val)), {}, "asin({})".format(val))
    else:
        raise TypeError("Input to asin() must be of type Parameter, int, or float.")

@err.add_trace
def apar_cos(val):
    if isinstance(val, Parameter):
        if val.units != {}:
            raise UnitEvaluationError("Input to arccosine must be unitless.", [val])
        if not -1 <= val.min <= 1 or not -1 <= val.max <= 1:
            raise ParameterError("Input to arccosine must be between -1 and 1 (apar_cos).", val)
        results = [np.arccos(val.min), np.arccos(val.max)]
        return Parameter((min(results), max(results)), {}, "acos({})".format(val.id))
    elif isinstance(val, (int, float)):
        if not -1 <= val <= 1:
            raise ParameterError("Input to arccosine must be between -1 and 1 (apar_cos).", val)
        return Parameter((np.arccos(val), np.arccos(val)), {}, "acos({})".format(val))
    else:
        raise TypeError("Input to acos() must be of type Parameter, int, or float.")

@err.add_trace
def apar_tan(val):
    if isinstance(val, Parameter):
        if val.units != {}:
            raise UnitEvaluationError("Input to arctangent must be unitless.", [val])
        results = [np.arctan(val.min), np.arctan(val.max)]
        return Parameter((min(results), max(results)), {}, "atan({})".format(val.id))
    elif isinstance(val, (int, float)):
        return Parameter((np.arctan(val), np.arctan(val)), {}, "atan({})".format(val))
    else:
        raise TypeError("Input to atan() must be of type Parameter, int, or float.")

@err.add_trace
def par_sqrt(val):
    if isinstance(val, Parameter):
        if not val >= 0:
            raise ParameterError("Input to sqrt must be >0 (par_sqrt).", val)
        new_units = {k: v / 2 for k, v in val.units.items()}
        return Parameter((np.sqrt(val.min), np.sqrt(val.max)), new_units, "sqrt({})".format(val.id))
    elif isinstance(val, (int, float)):
        if not val >= 0:
            raise ParameterError("Input to sqrt must be >0 (par_sqrt).", val)
        return Parameter((np.sqrt(val), np.sqrt(val)), {}, "sqrt({})".format(val))
    else:
        raise TypeError("Input to sqrt() must be of type Parameter, int, or float.")

@err.add_trace
def par_abs(val):
    if isinstance(val, Parameter):
        # ERR option ETC
        if abs(val.min) < abs(val.max):
            return Parameter((abs(val.min), abs(val.max)), val.units, "|{}|".format(val.id))
        else:
            return Parameter((abs(val.max), abs(val.min)), val.units, "|{}|".format(val.id))
    elif isinstance(val, (int, float)):
        return Parameter((abs(val), abs(val)), {}, "|{}|".format(val))
    else:
        raise TypeError("Input to abs() must be of type Parameter, int, or float.")

@err.add_trace
def par_log(val):
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

@err.add_trace
def par_log10(val):
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
    
@err.add_trace
def par_log2(val):
    if isinstance(val, Parameter):
        # ERR option ETC
        if np.log2(val.min) < np.log2(val.max):
            return Parameter((np.log2(val.min), np.log2(val.max)), {}, "|{}|".format(val.id))
        else:
            return Parameter((np.log2(val.max), np.log2(val.min)), {}, "|{}|".format(val.id))
    elif isinstance(val, (int, float)):
        return Parameter((np.log2(val), np.log2(val)), {}, id="|{}|".format(val))
    else:
        raise TypeError("Input to log2() must be of type Parameter, int, or float.")

@err.add_trace
def par_floor(val):
    if isinstance(val, Parameter):
        # ERR option ETC
        return Parameter((np.floor(val.min), np.floor(val.max)), val.units, "floor({})".format(val.id))
    elif isinstance(val, (int, float)):
        return Parameter((np.floor(val), np.floor(val)), {}, "floor({})".format(val))
    else:
        raise TypeError("Input to floor() must be of type Parameter, int, or float.")
    
@err.add_trace
def par_ceiling(val):
    if isinstance(val, Parameter):
        # ERR option ETC
        return Parameter((np.ceil(val.min), np.ceil(val.max)), val.units, "ceiling({})".format(val.id))
    elif isinstance(val, (int, float)):
        return Parameter((np.ceil(val), np.ceil(val)), {}, "ceiling({})".format(val))
    else:
        raise TypeError("Input to ceiling() must be of type Parameter, int, or float.")

class DesignError(OneilError):
    def __init__(self, filenames: list[str]):
        self.filenames = filenames
        
    def kind(self):
        return "DesignError"
        
    def context(self):
        return None
        
    def message(self):
        files_str = ", ".join(self.filenames)
        return f"Can't find design files: [{files_str}]"

class UnitParseError(OneilError):
    def __init__(self, filename: str, line_no: int, hrunits: str):
        self.filename = filename
        self.line_no = line_no
        self.hrunits = hrunits
        
    def kind(self) -> str:
        return "UnitParseError"
        
    def context(self) -> str | None:
        return f"in {self.filename} (line {self.line_no})"
        
    def message(self) -> str:
        return f"Failed to parse units '{self.hrunits}'"

class UnitEvaluationError(OneilError):
    def __init__(self, message: str, vals: list):
        self.message_ = message
        self.vals = vals
        self.context_: Parameter | Model | None = None
        self.notes_ = []
        for val in self.vals:
            if isinstance(val, Parameter):
                model_text = "" if not val.model else " in model " + val.model
                parameter_text = f"{val.name} ({val.id})" if val.name != val.id else val.name
                self.notes_.append(f"({un.hr_units(val.units)}) in {parameter_text} from line {val.line_no}{model_text}")
            else:
                self.notes_.append(str(val))

    def with_context(self, context):
        self.context_ = context
        return self
        
    def kind(self) -> str:
        return "UnitEvaluationError"
        
    def context(self) -> str | None:
        if self.context_:
            if isinstance(self.context_, Parameter):
                return f"in '{self.context_.name}' ({self.context_.id}, line {self.context_.line_no}, model '{self.context_.model}')"
            elif isinstance(self.context_, Model):
                return f"in model '{self.context_.name}'"
            else:
                raise TypeError(f"Invalid context type: {type(self.context_)}")
        else:
            return None
        
    def message(self) -> str:
        message = self.message_
        return message

class ParameterError(OneilError):
    def __init__(self, message, parameter):
        self.parameter = parameter
        self.message_ = message

        if isinstance(self.parameter, Parameter):
            self.notes_ = [f"({str(self.parameter)}) in {self.parameter.name} ({self.parameter.id}) from line {str(self.parameter.line_no)} in model {self.parameter.model}"]
        else:
            self.notes_ = [str(self.parameter)]

    def kind(self) -> str:
        return "ParameterError"

    def context(self) -> str | None:
        if self.parameter:
            return f"in {self.parameter.name} ({self.parameter.id}) from line {self.parameter.line_no} in model {self.parameter.model}"
        return None
    
    def message(self) -> str:
        return self.message_

class DivideByZeroError(OneilError):
    def __init__(self, parameter_or_location):
        if isinstance(parameter_or_location, Parameter):
            self.parameter = parameter_or_location
            self.location = None
        else:
            self.parameter = None
            self.location = parameter_or_location
        
    def kind(self) -> str:
        return "DivideByZeroError"
        
    def context(self) -> str | None:
        if self.parameter != None:
            return f"in {self.parameter.name} ({self.parameter.id}) from line {self.parameter.line_no} in model {self.parameter.model}"
        elif self.location != None:
            return f"in {self.location[0]} (line {self.location[1]})"
        else:
            return None
        
    def message(self) -> str:
        if self.parameter != None:
            return f"Cannot divide by zero ({self.parameter.id} is 0)"
        else:
            return f"Cannot divide by zero"

class SyntaxError(OneilError):
    def __init__(self, filename: str, line_no: int, line: str, message: str):
        self.filename = filename
        self.line_no = line_no
        self.line = line
        self.message_ = message

    def kind(self) -> str:
        return "SyntaxError"
        
    def context(self) -> str | None:
        return f"in {self.filename} (line {self.line_no})"
        
    def message(self) -> str:
        return self.message_

class IDError(OneilError):
    def __init__(self, model, ID, message):
        self.model = model
        self.ID = ID
        self.message_ = message
        
    def kind(self) -> str:
        return "IDError"
        
    def context(self) -> str | None:
        return f"in model {self.model.name} (ID: {self.ID})"
        
    def message(self) -> str:
        return f"{self.message_}"


class ImportError(OneilError):
    def __init__(self, filename, line_no, line, imprt, error):
        self.filename = filename
        self.line_no = line_no
        self.imprt = imprt
        self.error = error
        
    def kind(self) -> str:
        return "ImportError"
        
    def context(self) -> str | None:
        return f"in {self.filename} (line {self.line_no})"
        
    def message(self) -> str:
        return f"Failed to import '{self.imprt}': {self.error}. Does the import run by itself?"

class ModelLoadingError(OneilError):
    def __init__(self, filename: str, line_no: int, message: str):
        self.filename = filename
        self.line_no = line_no
        self.message_ = message

    def kind(self) -> str:
        return "ModelLoadingError"
        
    def context(self) -> str | None:
        return f"in {self.filename} (line {self.line_no})"
        
    def message(self) -> str:
        return self.message_

class ModelError(OneilError):
    def __init__(self, filename: str, source: list[str] = [], message: str = ""):
        self.filename = filename
        self.source = source
        
    def kind(self) -> str:
        return "ModelError"
        
    def context(self) -> str | None:
        return f"in {self.filename}"
        
    def message(self) -> str:
        return f"Submodel not found (source: {' -> '.join(self.source)})"

class ImportedFunctionError(OneilError):
    def __init__(self, parameter, error):
        self.parameter = parameter
        self.error = error

    def kind(self) -> str:
        return "ImportedFunctionError"
        
    def context(self) -> str | None:
        return f"in {self.parameter.equation} (line {self.parameter.line_no})"
        
    def message(self) -> str:
        return f"{self.error}"

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
                raise SyntaxError(model, line_no, line, "Invalid syntax for test references.")
        else:
            self.refs = []

        self.expression = line.split(':')[1].strip()

        if not self.expression:
            raise SyntaxError(model, line_no, line, "Empty test expression.")

        for old, new in FUNCTIONS.items():
            if "." + old not in self.expression:
                self.expression = self.expression.replace(old + "(", new + "(")
        for old, new in OPERATOR_OVERRIDES.items():
            self.expression = self.expression.replace(old, new)

        self.args = [x for x in re.findall(r"\b(?!\d+)(?<!')(?<!\")\b\w+\.?\w*\b(?!')(?!\")\b", self.expression) if x not in FUNCTIONS.values() and x not in BOOLEAN_OPERATORS]


class Parameter:
    def __init__(self, equation, units, id, hr_units="", model="", line_no=None, line="", name=None, options=None, performance=False, trace=False, section="", arguments=[], pointer=False):
        if trace:            
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
        self.pointer = pointer
        self.piecewise = True if isinstance(equation, list) else False
        self.minmax_equation = False
        self.hr_units = hr_units
        
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
                    raise UnitEvaluationError(f"{unit} is not currently a supported input unit. Only {un.BASE_UNITS} are supported. Refactor {unit} in terms of {un.BASE_UNITS}", []).with_context(self)
        else:
            raise TypeError('Units must be of type dict. Type "' + str(type(units)) + " was given.")

        # equation
        if callable(equation):
            self.callable = True
            self.equation = equation
            self.pointer = False
        elif isinstance(equation, (float, int)):
            self.assign(equation)
            self.independent = True
            self.pointer = False
        elif isinstance(equation, str):
            if any(character in EQUATION_OPERATORS + list(OPERATOR_OVERRIDES.keys()) for character in equation):
                # Find parameter names including "." imports
                self.args = [x for x in re.findall(r"(?!\d+)\w+\.?\w*", re.sub('[\'|\"].*[\'|\"]','',equation)) if x not in FUNCTIONS]

                # Trim duplicate args
                self.args = list(set(self.args))

                for old, new in FUNCTIONS.items():
                    if "." + old not in equation:
                        equation = equation.replace(old + "(", new + "(")
                for old, new in OPERATOR_OVERRIDES.items():
                    equation = equation.replace(old, new)

                self.equation = equation
                self.independent = False
                self.pointer = False
            else:
                if self.isdiscrete:
                    self.assign(equation)
                    self.independent = True
                    if self.pointer:
                        raise ParameterError("Discrete parameters cannot use pointers.", self)
                elif self.pointer:
                    self.equation = equation
                    self.args = [equation]
                else:
                    raise ParameterError("Invalid parameter equation.", self)
                    
        elif isinstance(equation, tuple):
            if isinstance(equation[0], (float, int)) and isinstance(equation[1], (float, int)):
                self.assign(equation)
                self.independent = True
            else:
                self.minmax_equation = True
                self.equation = equation
                self.args.extend(list(set(equation[0].args + equation[1].args)))
        elif isinstance(equation, list):
            self.equation=[]
            for piece in equation:
                self.add_piece(piece, [])
        else:
            raise ParameterError(f"Parameter equation must be a callable, a float, an int, or a string. (type {type(equation)} was given.)", self)

    def add_piece(self, piece, callable_args):
        self.equation.append(piece)
        piece_args = piece[0].args + piece[1].args + callable_args
        if not piece_args:
            raise ParameterError("Piecewise parameters must be dependent on another parameter.", self)
        else:
            self.args = list(set(self.args + piece_args))

    def assign(self, value):
        if value is not None:
            self.write(value)
        else:
            raise ParameterError("Value is empty.", self)

    def write(self, value):
        if isinstance(value, Parameter):
            if self.isdiscrete:
                if value.equation in self.options:
                    value.min = value.max = value.equation
                    value.equation = None
                    value.isdiscrete = True
                else:
                    raise ParameterError(f"Parameter was given a value that is not among its options: {value.options}", self)
                
            if value.min is not None and value.max is not None:
                if value.units != self.units:
                    raise UnitEvaluationError(f"Input or calculated units ({value.units}) do not match the required units: ({self.units}).", [self, value]).with_context(self)
                self.min = value.min
                self.max = value.max
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
                raise ParameterError(f"Parameter {value.id} cannot be written to {self.id}, because it is empty and independent.", self)
            
            if value.model: self.model = value.model

            self.line_no = {'model line': self.line_no, 'design line': value.line_no}
            self.line = {'model line': self.line, 'design line': value.line}
            self.notes = value.notes
            self.note_lines = value.note_lines
            self.section = value.section
            self.isdiscrete = value.isdiscrete
            self.trace = value.trace
        elif isinstance(value, tuple):
            if self.isdiscrete:
                raise ParameterError("Multiple discrete values aren't supported.", self)
            self.write_one(value[0], "min")
            self.write_one(value[1], "max")
        else:
            self.write_one(value, "minmax")

        if self.min and self.max:
            if self.min > self.max:
                raise ParameterError("Parameter min is greater than Parameter max.", self)

            if self.options and self.min and self.max:
                if self.isdiscrete:
                    if not (self.min in self.options and self.max in self.options):
                        raise ParameterError(f"Parameter was given a value that is not among its options: {self.options}", self)
                else:
                    if not self.options[1] >= self.options[0]:
                        raise ParameterError("Minimum limit > maximum limit.", self)
                    if not (self.min >= self.options[0] and self.max <= self.options[1]):
                        raise ParameterError(f"Values out of bounds [{un.hr_vals_and_units(self.options,self.units,self.hr_units)}]. Revise values or limits.", self)

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
                raise ParameterError(f"Parameter was assigned an option that is not among its options: {self.options}", self)
            
            if minmax == "minmax":
                self.min = self.max = value
            elif minmax == "min":
                self.min = value
            elif minmax == "max":
                self.max = value
        else:
            raise TypeError('Parameter value must be of type Parameter, tuple, int, float, str, or bool. Type "' + str(type(value)) + " was given.")


    def calculate(self, expression, glob, eval_params, eval_args):
        if self.trace:
            import pdb
            breakpoint()
        
        if not self.equation:
            raise ParameterError("Parameter needs an equation or value defined.", self)
        if (self.min or self.max):
            raise ParameterError("Parameters cannot be re-calculated.", self)

        if self.callable:
            if not all(k in eval_params for k in eval_args):
                raise ParameterError("Parameter is missing one or more arguments: " + str([arg for arg in eval_args if arg not in eval_params]), self)

            function_args = [eval_params[arg] for arg in eval_args]

            try:
                result = self.equation(*function_args)
                return result
            except OneilError as e:
                raise e
            except Exception as e:
                raise ImportedFunctionError(self, e)
        else:
            try:
                result = eval(expression, glob, eval_params | MATH_CONSTANTS)
                return result
            except OneilError as e:
                raise e
            except Exception as e:
                raise ImportedFunctionError(self, e)

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

    def long_print(self, sigfigs=8, indent=0, pref=None):
        output = " " * indent + self.name + ": "
        output += self.id + " = "
        output += self.human_readable(sigfigs, pref)
        print(output)

    def hprint(self, sigfigs=4, indent=0, pref=None):
        output = ("\n" + self.name + "\n--------------------\n")
        output += " " * indent + self.id + ": "
        output += self.human_readable(sigfigs, pref) + "\n"
        print(output)

    def human_readable(self, sigfigs=4, pref=None):
        if self.isdiscrete:
            return f"{self.min} | {self.max}"
        else:
            if not pref and self.hr_units is not None:
                pref = self.hr_units
            if self.min is not None and self.max is not None:
                if isinstance(self.min, str):
                    return self.min if self.min == self.max else f"{self.min} | {self.max}"
                else:
                    return un.hr_vals_and_units((self.min,self.max), self.units, pref, sigfigs)
            else:
                return "None"

    def copy(self):
        return Parameter((self.min, self.max), self.units, "copy of " + self.name, model=self.model, line_no=self.line_no, line=self.line)

    def __str__(self):
        return self.human_readable(4)

    # "+" Addition, left-hand, all cases 
    @err.add_trace
    def __add__(self, other):
        if isinstance(other, Parameter):
            if self.units != other.units:
                raise UnitEvaluationError(f"Cannot add {un.hr_units(other.units)} to {un.hr_units(self.units)}.", [self, other])
            return Parameter((self.min + other.min, self.max + other.max), self.units, "({}) + ({})".format(self.id, other.id))
        elif self.units == {}:
            return Parameter((self.min + other, self.max + other), {}, "({}) + ({})".format(self.id, str(other)))
        else:
            raise UnitEvaluationError(f"Cannot add {un.hr_units(self.units)} to a unitless number.", [self, other])

    # "-" Subtraction, left-hand, extreme
    @err.add_trace
    def __sub__(self, other):
        if isinstance(other, Parameter):
            if self.units != other.units:
                raise UnitEvaluationError(f"Cannot subtract {un.hr_units(other.units)} from {un.hr_units(self.units)}.", [self, other])
            if self.id == other.id and self.model == other.model: 
                return Parameter(0, {}, "({}) - ({})".format(self.id, other.id))
            return Parameter((self.min - other.max, self.max - other.min), self.units, "({}) - ({})".format(self.id, other.id))
        elif self.units == {}:
            return Parameter((self.min - other, self.max - other), {}, "({}) - ({})".format(self.id, str(other)))
        else:
            raise UnitEvaluationError(f"Cannot subtract a unitless number from {un.hr_units(self.units)}.", [self, other])

    # "--" Subtraction, left-hand, standard
    @err.add_trace
    def _minus(self, other):
        if isinstance(other, Parameter):
            if self.units != other.units:
                raise UnitEvaluationError(f"Cannot subtract {un.hr_units(other.units)} from {un.hr_units(self.units)}.", [self, other])
            results = [self.min - other.min, self.max - other.max]
            return Parameter((min(results), max(results)), self.units, "({}) -- ({})".format(self.id, other.id))
        elif self.units == {}:
            results = [self.min - other, self.max - other]
            return Parameter((min(results), max(results)), {}, "({}) -- ({})".format(self.id, str(other)))
        else:
            raise UnitEvaluationError(f"Cannot subtract a unitless number from {un.hr_units(self.units)}.", [self, other])

    # "*" Multiplication, left-hand, all cases
    @err.add_trace
    def __mul__(self, other):
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
    @err.add_trace
    def __truediv__(self, other):
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
            try:
                results = [self.min / other.max, self.max / other.min]
            except FloatingPointError:
                raise DivideByZeroError(other)
            else:
                return Parameter((min(results), max(results)), new_units, "({}) / ({})".format(self.id, other.id))
        elif isinstance(other, (int, float)):
            results = [self.min / other, self.max / other]
            return Parameter((min(results), max(results)), self.units, "({}) / ({})".format(self.id, str(other)))
        else:
            raise TypeError("Division must be between two Parameters or a Parameter and a number.")

    # "//" Division, left-hand, standard
    @err.add_trace
    def __floordiv__(self, other):
        if isinstance(other, Parameter):
            new_units = {k: v for k, v in self.units.items()}
            for k, v in other.units.items():
                if k in new_units:
                    new_units[k] -= v
                    if new_units[k] == 0:
                        del new_units[k]
                else:
                    new_units[k] = -v
            try:
                results = [self.min / other.min, self.max / other.max]
            except FloatingPointError:
                raise DivideByZeroError(other)
            else:
                return Parameter((min(results), max(results)), new_units, "({}) // ({})".format(self.id, other.id))
        elif isinstance(other, (int, float)):
            results = [self.min / other, self.max / other]
            return Parameter((min(results), max(results)), self.units, "({}) // ({})".format(self.id, str(other)))
        else:
            raise TypeError("Division must be between two Parameters or a Parameter and a number.")

    # "**" Exponentiation, left-hand, all cases
    @err.add_trace
    def __pow__(self, other):
        if isinstance(other, Parameter):
            if self.min != self.max or other.units != {}:
                raise UnitEvaluationError(f"Exponent must be a single unitless Parameter or number.", [self, other])
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
    @err.add_trace
    def __radd__(self, other):
        if isinstance(other, (int, float)):
            if self.units == {}:
                return Parameter((other + self.min, other + self.max), {}, "({}) + ({})".format(str(other), self.id))
            else:
                raise UnitEvaluationError(f"Cannot add {un.hr_units(self.units)} to a unitless number.", [self, other])
        else:
            raise TypeError("Addition must be between two Parameters or a Parameter and a number.")

    # "-" Subtraction, right-hand, extreme
    @err.add_trace
    def __rsub__(self, other):
        if isinstance(other, (int, float)):
            if self.units == {}:
                return Parameter((other - self.max, other - self.min), {}, "({}) - ({})".format(str(other), self.id))
            else:
                raise UnitEvaluationError(f"Cannot subtract {un.hr_units(self.units)} from a unitless number (__rsub__).", [self, other])
        else:
            raise TypeError("Subtraction must be between two Parameters or a Parameter and a number.")

    # "--" Subtraction, right-hand, standard
    @err.add_trace
    def _rminus(self, other):
        if isinstance(other, (int, float)):
            if self.units == {}:
                return Parameter((other - self.min, other - self.max), {}, "({}) - ({})".format(str(other), self.id))
            else:
                raise UnitEvaluationError(f"Cannot subtract {un.hr_units(self.units)} from a unitless number.", [self, other])
        else:
            raise TypeError("Subtraction must be between two Parameters or a Parameter and a number.")

    # "*" Multiplication, right-hand, all cases
    @err.add_trace
    def __rmul__(self, other):
        if isinstance(other, (int, float)):
            return Parameter((self.min * other, self.max * other), self.units, "({})({})".format(str(other), self.id))
        else:
            raise TypeError("Multiplication must be between a Parameter and a number.")

    # "/" Division, right-hand, extreme
    @err.add_trace
    def __rtruediv__(self, other):
        if isinstance(other, (int, float)):
            new_units = {k: -v for k, v in self.units.items()}
            return Parameter((other / self.max if self.max > 0 else np.inf, other / self.min if self.min > 0 else np.inf), new_units, "({})/({})".format(str(other), self.id))
        else:
            raise TypeError("Division must be between a Parameter and a number.")

    # "//" Division, right-hand, standard
    @err.add_trace
    def __rfloordiv__(self, other):
        if isinstance(other, (int, float)):
            new_units = {k: -v for k, v in self.units.items()}
            return Parameter((other / self.min if self.min > 0 else np.inf, other / self.max if self.max > 0 else np.inf), new_units, "({})//({})".format(str(other), self.id))
        else:
            raise TypeError("Division must be between a Parameter and a number.")

    # "**" Exponentiation, right-hand, all cases
    @err.add_trace
    def __rpow__(self, other):
        if isinstance(other, (int, float)):
            if self.units != {}:
                raise UnitEvaluationError(f"Exponent must be a single unitless Parameter or number.", [self, other])
            return Parameter((other**self.min, other**self.max), {}, "({})**({})".format(str(other), self.id))
        else:
            raise TypeError("Exponentiation must be between a Parameter and a number.")

    # "-" Unary operator
    @err.add_trace
    def __neg__(self):
        return Parameter((-self.max, -self.min), self.units, "-({})".format(self.id))

    # "<" Less than, left-hand, all cases
    @err.add_trace
    def __lt__(self, other):
        if isinstance(other, Parameter):
            if self.units != other.units:
                raise UnitEvaluationError(f"Cannot compare {un.hr_units(self.units)} to {un.hr_units(other.units)} (less than).", [self, other])
            return self.min < other.min and self.max < other.max
        elif isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                raise UnitEvaluationError(f"Cannot compare {un.hr_units(self.units)} to a unitless number (less than).", [self, other])
            return self.min < other and self.max < other
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # ">" Greater than, left-hand, all cases
    @err.add_trace
    def __gt__(self, other):
        if isinstance(other, Parameter):
            if self.units != other.units:
                raise UnitEvaluationError(f"Cannot compare {un.hr_units(self.units)} to {un.hr_units(other.units)} (greater than).", [self, other])
            return self.min > other.min and self.max > other.max
        elif isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                raise UnitEvaluationError(f"Cannot compare {un.hr_units(self.units)} to a unitless number (greater than).", [self, other])
            return self.min > other and self.max > other
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # ">=" Greater than or equal to, left-hand, all cases
    @err.add_trace
    def __le__(self, other):
        if isinstance(other, Parameter):
            if self.units != other.units:
                raise UnitEvaluationError(f"Cannot compare {un.hr_units(self.units)} to {un.hr_units(other.units)} (less than or equal).", [self, other])
            return self.min <= other.min and self.max <= other.max
        elif isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                raise UnitEvaluationError(f"Cannot compare {un.hr_units(self.units)} to a unitless number (less than or equal).", [self, other])
            return self.min <= other and self.max <= other
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # "<=" Less than or equal to, left-hand, all cases
    @err.add_trace
    def __ge__(self, other):
        if isinstance(other, Parameter):
            if self.units != other.units:
                raise UnitEvaluationError(f"Cannot compare {un.hr_units(self.units)} to {un.hr_units(other.units)} (greater than or equal).", [self, other])
            return self.min >= other.min and self.max >= other.max
        elif isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                raise UnitEvaluationError(f"Cannot compare {un.hr_units(self.units)} to a unitless number (greater than or equal).", [self, other])
            return self.min >= other and self.max >= other
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # "==" Equal to, left-hand, all cases
    @err.add_trace
    def __eq__(self, other):
        if isinstance(other, Parameter):
            if self.units != other.units:
                message = f"Cannot compare {un.hr_units(self.units)} to {un.hr_units(other.units)} (equal)."
                raise UnitEvaluationError(message, [self, other])
            return self.min == other.min and self.max == other.max
        elif isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                message = f"Cannot compare {un.hr_units(self.units)} to a unitless number (equal)."
                raise UnitEvaluationError(message, [self, other])
            return self.min == other and self.max == other
        elif isinstance(other, str):
            return self.min == other
        elif isinstance(other, bool):
            return bool
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # "!=" Not equal to, left-hand, all cases
    @err.add_trace
    def __ne__(self, other):
        if isinstance(other, Parameter):
            if self.units != other.units:
                message = f"Cannot compare {un.hr_units(self.units)} to {un.hr_units(other.units)} (not equal)."
                raise UnitEvaluationError(message, [self, other])
            return self.min != other.min and self.max != other.max
        elif isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                message = f"Cannot compare {un.hr_units(self.units)} to a unitless number (not equal)."
                raise UnitEvaluationError(message, [self, other])
            return self.min != other and self.max != other
        elif isinstance(other, str):
            return self.min == other
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # "<" Less than, right-hand, all cases
    @err.add_trace
    def __rlt__(self, other):
        if isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                message = f"Cannot compare a unitless number to unit \"{un.hr_units(self.units)}\" (less than)."
                raise UnitEvaluationError(message, [self, other])
            return other < self.min and other < self.max
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # "<=" Less than or equal to, right-hand, all cases
    @err.add_trace
    def __rgt__(self, other):
        if isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                message = f"Cannot compare a unitless number to unit \"{un.hr_units(self.units)}\" (greater than)."
                raise UnitEvaluationError(message, [self, other])
            return other > self.min and other > self.max
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # ">=" Greater than or equal to, right-hand, all cases
    @err.add_trace
    def __rle__(self, other):
        if isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                message = f"Cannot compare a unitless number to unit \"{un.hr_units(self.units)}\" (less than or equal)."
                raise UnitEvaluationError(message, [self, other])
            return other <= self.min and other <= self.max
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # ">=" Greater than or equal to, right-hand, all cases
    @err.add_trace
    def __rge__(self, other):
        if isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                message = f"Cannot compare a unitless number to unit \"{un.hr_units(self.units)}\" (greater than or equal)."
                raise UnitEvaluationError(message, [self, other])
            return other >= self.min and other >= self.max
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # "==" Equal to, right-hand, all cases
    @err.add_trace
    def __req__(self, other):
        if isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                message = f"Cannot compare a unitless number to unit \"{un.hr_units(self.units)}\" (equal)."
                raise UnitEvaluationError(message, [self, other])
            return other == self.min and other == self.max
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # "!=" Not equal to, right-hand, all cases
    @err.add_trace
    def __rne__(self, other):
        if isinstance(other, (int, float)):
            if other != 0 and self.units != {}:
                message = f"Cannot compare a unitless number to unit \"{un.hr_units(self.units)}\" (not equal)."
                raise UnitEvaluationError(message, [self, other])
            return other != self.min and other != self.max
        else:
            raise TypeError("Comparison must be between a two Parameters or a Parameter and a number.")

    # "|" Logical OR, left-hand, all cases
    @err.add_trace
    def __or__(self, other):
        if isinstance(other, Infix):
            return other.__ror__(self)

        if self.units != {}:
            message = f"| is only valid for unitless parameters with boolean values."
            raise UnitEvaluationError(message, [self, other])
        
        if isinstance(other, Parameter):
            if other.units != {}:
                message = f"| is only valid for unitless parameters with boolean values."
                raise UnitEvaluationError(message, [self, other])
            return self.min or other.min or self.max or other.max
        elif isinstance(other, (bool)):
            return self.min or other or self.max
        else:
            raise TypeError("OR operator is only valid between two Parameters or a Parameter and a boolean.")

    # "&" Logical AND, left-hand, all cases
    @err.add_trace
    def __and__(self, other):
        if self.units != {}:
            message = f"& is only valid for unitless parameters with boolean values."
            raise UnitEvaluationError(message, [self, other])
        
        if isinstance(other, Parameter):
            if other.units != {}:
                message = f"& is only valid for unitless parameters with boolean values."
                raise UnitEvaluationError(message, [self, other])
            return self.min and other.min and self.max and other.max
        elif isinstance(other, (bool)):
            return self.min and other and self.max
        else:
            raise TypeError("AND operator is only valid between two Parameters or a Parameter and a boolean.")

    # "|" Logical OR, right-hand, all cases
    @err.add_trace
    def __ror__(self, other):
        if self.units != {}:
            message = f"| is only valid for unitless parameters with boolean values."
            raise UnitEvaluationError(message, [self, other])
        
        if isinstance(other, (bool)):
            return self.min or other or self.max
        else:
            raise TypeError("OR operator is only valid between two Parameters or a Parameter and a boolean.")

    # "&" Logical AND, right-hand, all cases
    @err.add_trace
    def __rand__(self, other):
        if self.units != {}:
            message = f"& is only valid for unitless parameters with boolean values."
            raise UnitEvaluationError(message, [self, other])

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
        self.fail_count = 0
        self.test_count = 0
        
        if design_filename:
            self.overwrite(design_filename)

        try:
            self._check_namespace()
        except OneilError as e:
            raise e.with_note(f"In namespace check for {self.name}")

        for key, param in self.parameters.items():
            if param.pointer:
                if param.piecewise:
                    for eq, cond in param.equation:
                        if eq.pointer:
                            if eq.minmax_equation:
                                for p in eq.equation:
                                    if p.equation in self.parameters.keys():
                                        param.args.append(p.equation)
                            else:
                                if eq.equation in self.parameters.keys():
                                    param.args.append(eq.equation)
                                else:
                                    raise ParameterError("Parameter " + param.id + " (line " + str(param.line_no + 1) + ") in " + param.model + " has a string, non-equation assignment (" + param.equation + ") that is not in the model and has no options defined. If it's supposed to be a case, specify options. If it's supposed to be assigned to another value, make sure that value is also defined.", param)
                            
                else:
                    if param.minmax_equation:
                        for p in param.equation:
                            if p.equation in self.parameters.keys():
                                param.args.append(p.equation)
                    else:
                        if param.equation in self.parameters.keys():
                            param.args.append(param.equation)
                        else:
                            raise ParameterError("Parameter " + param.id + " (line " + str(param.line_no + 1) + ") in " + param.model + " has a string, non-equation assignment (" + param.equation + ") that is not in the model and has no options defined. If it's supposed to be a case, specify options. If it's supposed to be assigned to another value, make sure that value is also defined.", param)
        
    # Checks that all of the arguments to each parameter are defined
    def _check_namespace(self, verbose=False):
        undefined = {}

        # TODO: check for reserved symbols: `if arg in FUNCTIONS.values() or any([arg in v for v in OPERATOR_OVERRIDES.values()]) or arg in self.constants or arg in BOOLEAN_OPERATORS`
        
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
                    undefined[arg] = f"{arg} from {param.id} (line {param.line}) in {param.model}"
                elif verbose:
                    # Report the submodule parameters of the same ID.
                    for key in self.submodels:
                        self._check_namespace_recursively(self._retrieve_model(self.submodels[key]['path']), arg, param)

        # Return a list of all args that aren't defined.
        if undefined:
            error = IDError(self, f"{undefined.keys()}", self.name.capitalize() + " has undefined arguments")
            for value in undefined.values():
                error = error.with_note(value)
            raise error

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

            submodel._check_namespace_recursively(source, arg, param, new_trail)

    # Convert an empty model to a modeled design with all parameters assigned a value.
    def overwrite(self, design_files, quiet=False):
        if not design_files:
            raise DesignError([])
        
        # Import design parameters.
        if isinstance(design_files, str):
            if not os.path.exists(design_files):
                raise DesignError([design_files])
            _, design_params, _, tests, design = parse_file(design_files, self)
        elif isinstance(design_files, list):
            missing_files = [file for file in design_files if not os.path.exists(file)]
            if len(missing_files) > 0:
                raise DesignError(missing_files)
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
                
        # Rewrite parameters with appended submodel paths.
        for ID, parameter in design.items():
            if "." in ID:
                ID, submodel = ID.split(".")
                path = copy.copy(self.submodels[submodel]['path'])
            else:
                submodel = ID
                path = []
            try:
                self._rewrite_parameter(ID, parameter, path)
            except ModelError as e:
                raise e.with_note(f"Couldn't find {submodel} in path {'.'.join(path)} while overwriting {ID} in {design_files} (line {self.submodels[submodel]['line_no']})")
            except ParameterError as e:
                raise e.with_note(f"In parameter {ID} in {design_files}")

        self.defaults.append(list[set(self.parameters).difference(design)])
        
        for ID, parameter in design.items():
            self.parameters[ID] = parameter

        self.tests.extend(tests)

        design_files.reverse()
        design_files = [file.strip(".on") for file in design_files]
        new_design = "@".join(design_files)
        if self.design != new_design:
            self.design = new_design + "@" + self.design if self.design != "default" else new_design

        self._reset_recursively()
        self.build()

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
            snippet += "\\label{sssec:" + param_ID + "}\n"
            snippet += 'The equation for ' + param.name + " is \n"

            snippet += "\\begin{equation}\n"
            snippet += "\t\\label{eq:" + param_ID + "}\n"
            snippet += "\t\\boxed{" + latex_ID + " = " + self._param2latex(param_eq) + "}~.\n"
            snippet += "\\end{equation}\n"

        else:
            snippet = "\n\\subsubsection{" + param.name.title() + "}"
            snippet += "\\label{sssec:" + param_ID + "}\n"
            snippet += param.name + " is a constant value of\n"
            snippet += "\\begin{equation}\n"
            snippet += "\t\\label{eq:" + param_ID + "}\n"
            snippet += "\t\\boxed{" + latex_ID + " = " + param.human_readable().replace(" ", "~") + "}~.\n"
            snippet += "\\end{equation}\n"

        snippet += ''.join(param.notes)

        return snippet

    def test_snippet(self, ID, test):
        snippet = "\n\\subsubsection{" + ID.replace("_", "~").title() + "}"
        snippet += "\\label{sssec:" + ID.replace("_", "-") + "}\n"
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

        if "\\cite" in document:
            document += "\n\\printbibliography\\end{document}"
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
                raise IDError(self, search_ID, "Could not find parameter in model.")
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

    def build(self, quiet=False):
        # Calculate imports
        self._calculate_models_recursively(quiet)

        # Run tests
        self.test(verbose=False)

        if not quiet: self.summarize()

        self.calculated = True

    def _calculate_models_recursively(self, quiet=False):
        # Initiate recursive calculation of submodel parameters.
        for key, entry in self.submodels.items():
                    if 'model' in entry.keys():
                        entry['model']._calculate_models_recursively(quiet=True)

        # Calculate this models dependent parameters.
        self._calculate_parameters_recursively(self.parameters)

    def eval(self, expression):
        # Make a dict of calculation parameters from the submodels
        submodel_parameters = {}
        result = None
        expression_args = [x for x in re.findall(r"(?!\d+)\w+\.?\w*", expression) if x not in FUNCTIONS]
        
        for f, pf in FUNCTIONS.items():
            expression = re.sub(r"(?<!\w)" + re.escape(f), re.escape(pf), expression)

        for k, v in OPERATOR_OVERRIDES.items():
            expression = expression.replace(k, v)
            
        for i, arg in enumerate(expression_args):
            if "." in arg:
                result, prefixed_ID = self.retrieve_parameter_from_submodel(arg)

                if isinstance(result, Parameter):
                    submodel_parameters[prefixed_ID] = result
                elif isinstance(result, (int, float, str, np.int64, np.float64, np.float32, np.float16)):
                    return result
                else:
                    raise ParameterError("Eval failed.", expression)
                expression = re.sub(r"(?<!\w)" + re.escape(arg), re.escape(prefixed_ID), expression)

        eval_params = self.parameters | submodel_parameters | self.constants

        try:
            result = eval(expression, globals(), eval_params)
        except UnitEvaluationError as e:
            raise e.with_context(self)
        except OneilError as e:
            raise e
        except Exception as e:
            raise IDError(self, expression, str(e))

        if isinstance(result, Parameter):
            return result
        elif isinstance(result, (bool, np.bool_)):
            # Convert numpy bools to python bools
            return bool(result)
        elif isinstance(result, (int, float, str, np.int64, np.float64, np.float32, np.float16)):
            return result
        else:
            raise ParameterError("Eval failed.", expression)

    def _test_recursively(self, log, path=[], test_inputs={}, trail=[], verbose=True):
        fails = 0
        tests = 0
            
        # Test this model. Eval each test expression, using self.parameters and the reference models
        for test in self.tests:
            tests += 1
            test_params = {}
            run_expression = test.expression

            # Only run tests with inputs if inputs were found
            if not any(ref not in test_inputs for ref in test.refs):
                if verbose: print("Test (" + self.name + "): " + run_expression)
                for i, arg in enumerate(test.args):
                    if "." in arg:
                        result, prefixed_ID = self.retrieve_parameter_from_submodel(arg)

                        if isinstance(result, Parameter):
                            test_params[prefixed_ID] = result
                        else:
                            raise TypeError("Invalid result type: " + str(type(result)))

                        run_expression = run_expression.replace(arg, prefixed_ID)
                    elif arg in test_inputs:
                        test_params[arg] = test_inputs[arg]
                    elif arg in FUNCTIONS.values() or any([arg==v for v in OPERATOR_OVERRIDES.values()]) or arg in self.constants or arg in BOOLEAN_OPERATORS:
                        print(test.__dict__)
                        raise IDError(self, arg, f"Test argument ({arg}) uses a reserved keyword.")
                    else:
                        test_params[arg] = self.parameters[arg]


                test_params = test_params | self.constants

                if test.trace:
                    print("Breakpoint for test: " + test.expression)
                    
                    import pdb

                    breakpoint()
                    eval(run_expression, globals(), test_params)

                try:
                    calculation = eval(run_expression, globals(), test_params)
                except UnitEvaluationError as e:
                    raise e.with_context(self)
                except ZeroDivisionError as e:
                    raise DivideByZeroError((run_expression.file_name, run_expression.line_number))
                except OneilError as e:
                    raise e
                except Exception as e:
                    raise ImportedFunctionError(self, e)

                if isinstance(calculation, (bool, np.bool_)):
                    result = bcolors.OKGREEN + "pass" + bcolors.ENDC if calculation else bcolors.FAIL + "fail" + bcolors.ENDC

                if verbose: print("\tResult: " + str(result))

                if not calculation:
                    fails += 1
                    if verbose:
                        # Print the test explanation
                        if test.notes: print("\t" + test.notes[0].strip("\n"))
                        
                        # Print the args and values
                        for k, v in test_params.items():
                            if isinstance(v, Parameter):
                                print("\t" + v.__repr__())
            else:
                fails += 1
                if verbose:
                    print("Test (" + self.name + "): " + test.expression + " (" + bcolors.FAIL + "skipped" + bcolors.ENDC + ")")

        # Initiate testing of this model's submodels
        for submodel_ID in {k:v for (k,v) in self.submodels.items() if 'model' in v and v['model'].name not in log}:
            submodel_entry = self.submodels[submodel_ID]
            submodel = submodel_entry['model']
            log.append(submodel.name)
            
            # Prepare test inputs for the submodel
            if submodel_entry['inputs']:
                for arg, inp in submodel_entry['inputs'].items():                 
                    if isinstance(inp, Parameter):
                        test_inputs[arg] = inp
                    elif isinstance(inp, str):
                        if '.' in inp:
                            result, _ = self.retrieve_parameter_from_submodel(inp)

                            if isinstance(result, Parameter):
                                test_inputs[arg] = result
                            else:
                                raise TypeError("Invalid result type: " + str(type(result)))
                        elif inp in self.parameters:
                            test_inputs[arg] = self.parameters[inp]
                        else:
                            raise ParameterError(f"Test input {inp} for submodel {submodel_ID} not found in {self.name}.", inp)
                    else:
                        raise TypeError("Invalid test input type: " + str(type(inp)))
                
                    if '.' in arg:
                        arg_ID, source = arg.split('.')
                        passthrough_path = copy.copy(self.submodels[source]['path'])
                        end_model = passthrough_path.pop()
                        passthrough_model = self._retrieve_model(passthrough_path)
                        end_model_ID = [k for k, v in passthrough_model.submodels.items() if v['path'] == [end_model]][0]
                        passthrough_model.submodels[end_model_ID]['inputs'][arg_ID] = test_inputs[arg]
                        del test_inputs[arg]

            # Test the submodel
            path = copy.copy(self.submodels[submodel_ID]['path'])
            new_trail = copy.copy(trail)
            new_trail.append(self.name)
            if path:
                submodel_name = path.pop(0)
                submodel = [model['model'] for k, model in self.submodels.items() if 'model' in model and model['model'].name == submodel_name]
                if submodel:
                    submodel = submodel[0]
                    new_fails, new_tests = submodel._test_recursively(log, copy.copy(path), test_inputs, new_trail, verbose=verbose)
                    fails += new_fails
                    tests += new_tests
                else:
                    raise ModelError(submodel_name, new_trail)

        return fails, tests
    
    def test(self, verbose=True):
        log = []
        fail_count, test_count = self._test_recursively(log, verbose=verbose)

        self.fail_count = fail_count
        self.test_count = test_count

        return fail_count, test_count


    def compare(self, alternate_design_file, parameter_IDs):
        alternate = copy.deepcopy(self)
        alternate._reset()
        alternate.overwrite(alternate_design_file)
        alternate.build(quiet=True)

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
        # Sort the parameter keys alphabetically and wrap in a list
        parameter_keys = list(self.parameters.keys())
        parameter_keys.sort()
        self.tree(parameter_keys, levels=0, verbose=True, turtles=False)

    def independent(self, indent=0):
        print(f"{' ' * indent}{self.name}:")
        for param in self.parameters.values():
            if param.independent:
                param.long_print(indent=indent+4)
        
        for submodel_dict in self.submodels.values():
            if 'model' in submodel_dict:
                print(f"{' ' * indent}{submodel_dict['model'].name}:")
                for param in submodel_dict['model'].parameters.values():
                    if param.independent:
                        param.long_print(indent=indent+4)

    def summarize(self, sigfigs=4, verbose=False):
        print("-" * 80)
        print(bcolors.OKBLUE + "Model: " + self.name + bcolors.ENDC)
        print(bcolors.ORANGE + "Design: " + self.design + bcolors.ENDC)
        print("Parameters: " + str(len(self.parameters) + len(self.constants)) 
        + " (" + str(len([p for ID, p in self.parameters.items() if p.independent])) + " independent, " 
        + str(len([p for ID, p in self.parameters.items() if not p.independent])) + " dependent, "
        + str(len(self.constants)) + " constants)")
        print(f"Tests: {self.test_count - self.fail_count}/{self.test_count}", end="")
        if self.fail_count:
            print(" (" + bcolors.BOLD + bcolors.FAIL + "FAIL" + bcolors.ENDC + ")")
        else:
            print(" (" + bcolors.OKGREEN + "PASS" + bcolors.ENDC + ")")
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
                if parameter_ID in self.parameters:
                    parameter = self.parameters[parameter_ID]
                elif "." in parameter_ID:
                    result, _ = self.retrieve_parameter_from_submodel(parameter_ID)

                    if isinstance(result, Parameter):
                        parameter = result
                    else:
                        raise TypeError("Invalid result type: " + str(type(result)))
                else:
                    raise IDError(self, parameter_ID, "Parameter not found in model.")

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
                        raise ParameterError("Circular dependency found in path: " + "=>".join(trail) + ".", parameter)

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

    def _calculate_parameters_recursively(self, parameters, trail=[]):
        for parameter in parameters.values():
            if isinstance(parameter, Parameter) and any([parameter.min is None, parameter.max is None]):
                submodel_parameters = {}

                # Update the trail for recursive errors.
                new_trail = copy.copy(trail)

                if new_trail:
                    if new_trail[-1] != parameter.id:
                        new_trail.append(parameter.id)
                else:
                    new_trail = [parameter.id]

                # If parameter has subparameters (piecewise, minmax), calculate them, else, calculate this parameter directly.
                if parameter.piecewise:
                    piece_equations = {}
                    piece_conditions = {}
                    true_equations = {}
                    
                    if len(parameter.equation) < 2:
                        raise ParameterError(f"Piecewise function requires at least two pieces.", parameter)
                    
                    for i, piece in enumerate(parameter.equation):
                        piece_equations.update({piece[0].id + str(i): piece[0]})
                        piece_conditions.update({piece[1].id + str(i): piece[1]})
                    self._calculate_parameters_recursively(piece_conditions, new_trail)

                    for cond in piece_conditions:
                        if piece_conditions[cond].min and piece_conditions[cond].max:
                            true_equations.update({cond.replace("cond", "eq"): piece_equations[cond.replace("cond", "eq")]})
                    self._calculate_parameters_recursively(true_equations, new_trail)

                elif parameter.minmax_equation:
                    minmax_equation_parameters = {}
                    for i, eq in enumerate(parameter.equation):
                        minmax_equation_parameters.update({eq.id + str(i): eq})
                    self._calculate_parameters_recursively(minmax_equation_parameters, new_trail)
                else:
                    if not parameter.args:
                        raise ParameterError("Parameter has no args nor a set value.", parameter)

                    # Calculate any argument parameters that haven't been calculated yet
                    arg_parameters = {arg: self.parameters[arg] for arg in [x for x in parameter.args if x in self.parameters]}

                    if not all([True if all([parameter.min, parameter.max]) else False for arg, parameter in arg_parameters.items()]):
                        calc_args = {k: self.parameters[k] for k in [x for x in parameter.args if x in self.parameters] if "." not in k}
                        
                        if parameter.id in trail:
                            raise ParameterError("Circular dependency found in path: " + "=>".join(trail) + ".", parameter)
                        
                        self._calculate_parameters_recursively(calc_args, new_trail)

                    # Make a dict of calculation parameters from the submodels
                    calc_args = []
                    for i, arg in enumerate(parameter.args):
                        if "." in arg:
                            result, prefixed_ID = self.retrieve_parameter_from_submodel(arg)

                            if isinstance(result, Parameter):
                                submodel_parameters[prefixed_ID] = result
                            else:
                                raise TypeError("Invalid result type: " + str(type(result)))    
                            
                            calc_args.append(prefixed_ID)                        
                        else:
                            calc_args.append(arg)

                    # Substitute submodel params in the expression
                    # This has to be done sorted or longer params could get accidentally replaced by a shorter params that match the beginning of the longer param
                    expression = parameter.equation
                    sorted_parameter_args = [arg for arg in parameter.args]
                    sorted_parameter_args.sort(key=len, reverse=True)
                    for i, arg in enumerate(sorted_parameter_args):
                        if "." in arg:
                            result, prefixed_ID = self.retrieve_parameter_from_submodel(arg)

                            if not parameter.callable:
                                expression = re.sub(r"(?<!\w)" + re.escape(arg), re.escape(prefixed_ID), expression)

                # all_params = self.parameters | submodel_parameters | self.constants

                # Calculate the parameter
                calculation = None
                if parameter.piecewise:
                    for piece in parameter.equation:
                        if piece[1].min and piece[1].max:
                            calculation = piece[0]
                            break
                    if calculation is None or calculation.min is None or calculation.max is None:
                        raise ParameterError("No piecewise condition was met.", parameter)
                elif parameter.minmax_equation:
                    calculation = (parameter.equation[0].min, parameter.equation[1].max)
                else:
                    if parameter.pointer:
                        calculation = (self.parameters | submodel_parameters | self.constants)[expression]
                    else:
                        calculation = parameter.calculate(expression, globals(), self.parameters | submodel_parameters | self.constants, calc_args)                    
                parameter.assign(calculation)

    def retrieve_parameter_from_submodel(self, ID):
        parameter_ID, submodel_ID = ID.split(".")
        path = []
        prefixed_ID = ""
        
        if submodel_ID in self.submodels:
            path = copy.copy(self.submodels[submodel_ID]['path'])
            prefixed_ID = '_'.join(path) + "_" + parameter_ID
            result = self._retrieve_parameter_recursively(parameter_ID, path)
        else:
            raise IDError(self, ID, f"Submodel ID \"{submodel_ID}\" not found.")

        return result, prefixed_ID
    
    # Recursively retrieve a parameter from a submodel or submodel of a submodel, etc.
    def _retrieve_parameter_recursively(self, parameter_ID, path, trail=[]):
        new_trail = copy.copy(trail).append(self.name)

        if path:
            submodel_name = path.pop(0)
            submodel = [model['model'] for k, model in self.submodels.items() if 'model' in model and model['model'].name == submodel_name]
            if submodel:
                submodel = submodel[0]
                result = submodel._retrieve_parameter_recursively(parameter_ID, path, trail)
            else:
                raise IDError(submodel, parameter_ID, f"Submodel name \"{submodel_name}\" not found while retrieving parameter ID \"{parameter_ID}\" from path ({', '.join(new_trail.append(submodel_name).append(path))}).")
        else:
            if parameter_ID in self.parameters:
                result = self.parameters[parameter_ID]
            else:
                raise IDError(self, parameter_ID, f"Parameter ID \"{parameter_ID}\" not found in path ({self.name + (new_trail or '')}).")
        
        return result

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
                raise ModelError(submodel_name, new_trail)
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
                raise ModelError(parameter_ID, trail)
        else:
            try:
                self.parameters[parameter_ID].write(parameter)
            except Exception as e:
                if parameter_ID not in self.parameters:
                    raise IDError(self, parameter_ID, f"Parameter from design file {parameter.model} not found in model {self.name}.")
                else:
                    raise e
            else:
                return True
            # if self.parameters[parameter_ID].error:
            #     return self.parameters[parameter_ID].error

def handler(model: Model, inpt: str):
    args = inpt.strip().split(" ")
    cmd = args.pop(0)
    opt_list = [arg for arg in args if "=" in arg]
    opts = {}
    for arg in opt_list:
        args.remove(arg)
        arg_value = arg.split("=")[1]
        arg_value = float(arg_value) if isfloat(arg_value) else arg_value
        opts[arg.split("=")[0]] = arg_value
    
    try:
        if cmd == "tree":
            model.tree(args, **opts)
        elif cmd == "summarize":
            model.summarize()
        elif cmd == "all":
            model.all()
        elif cmd == "dependents":
            model.dependents(args)
        elif cmd == "independent":
            model.independent()
        elif cmd == "design":
            if any([arg for arg in args if "." in arg and ".on" not in arg]):
                print("Only .on files are allowed.")
                return
            if model.name in [arg.strip(".on") for arg in args]:
                print("Cannot overwrite model with itself.")
                return
            args = [arg if "." in arg else arg + ".on" for arg in args]
            model.overwrite(args)
        elif cmd == "test":
            model.test()
        elif cmd == "export":
            model.export_pdf(args)
        elif cmd == "load":
            model_name, model_designs, commands = parse_args(args)

            loader(model_name, model_designs)

            for command in commands:
                print("(" + bcolors.OKBLUE + model.name + bcolors.ENDC + ") >>> " + command)
                handler(model, command)

        elif cmd == "reload":
            if model.design == "default":
                loader(model.name, [])
            else:
                loader(model.name, [model.design])
        elif cmd == "help":
            print(help_text)
            return
        elif cmd == "units":
            print(un.print_all())
        elif cmd == "quit":
            sys.exit()
        elif cmd == "quit()":
            sys.exit()
        elif cmd == "exit":
            sys.exit()
        elif ":" in inpt:
            param_expr = inpt.split(":")[0].strip()
            requested_units = inpt.split(":")[1].strip()
            
            if param_expr in model.parameters.keys():
                try:
                    model.parameters[param_expr].hprint(pref=requested_units)
                except ValueError:
                    print(f"Requested units ({requested_units}) do not match parameter base units ({un._build_compound_unit_str(model.parameters[param_expr].units)}).")
            else:
                # Try to evaluate the expression on the left side
                try:
                    result = model.eval(param_expr)
                    if isinstance(result, Parameter):
                        try:
                            result.hprint(pref=requested_units)
                        except ValueError:
                            print(f"Requested units ({requested_units}) do not match calculated expression units ({un._build_compound_unit_str(result.units)}).")
                    else:
                        print(f"Result: {result} (unitless, cannot convert to {requested_units})")
                except Exception as e:
                    print(f"Could not evaluate expression '{param_expr}': {str(e)}") 
        elif any([op in inpt for op in OPERATORS]):
            try:
                print(model.eval(inpt))
            except Exception as e:
                print(f"Could not evaluate expression '{inpt}': {str(e)}")
        elif inpt in model.parameters:
            model.parameters[inpt].hprint()
        elif "." in inpt:
            result, _ = model.retrieve_parameter_from_submodel(inpt)
            if isinstance(result, Parameter):
                result.hprint()
            else:
                raise TypeError("Invalid result type: " + str(type(result)))
        else:
            print(f"Command {inpt} not found. Type 'help' for a list of commands.")
            return
    except OneilError as err:
        console.print_error(err)
        return
            

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

    independent
        Print all independent parameters.

    design [design 1] [design 2] ... [design n]
        Overwrite the current model with the specified designs.

    test
        Run all tests on the model and any loaded designs.

    export [param 1] [param 2] ... [param n]
        Export the entire model to a PDF file or just the specified parameters.

    load model
        Load a new model (starting over from scratch).

    reload
        Reload the current model with all designs (starting over from scratch).

    help
        Print this help text.

    units
        Print all units supported in Oneil.

    quit
    quit()
    exit
        Exit the program.
"""

def loader(inp: str, designs: list[str]) -> Model:
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
                try:
                    model = Model(inp)
                    model.build()
                except OneilError as err:
                    console.print_error(err)
                    model = None
                    inp = ""
            else:
                print("Model " + inp + " not found.")
                inp = ""
                continue
        else:
            inp = input("Enter a model: ")

    # Load designs
    for design in designs:
        handler(model, "design " + design)

    return model

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
            handler(model, input(f"({bcolors.OKBLUE}{model.name}{bcolors.ENDC}) >>> "))
        else:
            handler(model, input(f"({bcolors.ORANGE}{model.design}@{bcolors.ENDC}{bcolors.OKBLUE}{model.name}{bcolors.ENDC}) >>> "))

def debugger(model):
    print("Enterring debug mode. Type 'quit' to exit.")
    while True:
        handler(model, input(f"{bcolors.FAIL}debugger{bcolors.ENDC} ({bcolors.OKBLUE}{model.name}{bcolors.ENDC}) >>> "))

def parse_args(args: list[str]) -> tuple[str, list[str], list[str]]:
    # if there are no arguments, then the user needs to be prompted for details
    # so we return empty data
    if len(args) == 0:
        return "", [], []

    # args example: `cooper_station@oneill_cylinder all test`
    inputs = args[0].split("@")
    commands = args[1:]

    # the model is the last one listed
    input_model = inputs[-1]

    # the designs are the others listed
    # NOTE: I'm not sure why it's reversed, but that's what the previous code did
    input_designs = list(reversed(inputs[:-1]))

    return input_model, input_designs, commands
    
def main(args=sys.argv[1:]):
    console.print_welcome_message()

    # parse the files and commands
    inp, designs, commands = parse_args(args)

    # load the model
    model = loader(inp, designs)

    # Handle commands after the first as cli commands.
    for command in commands:
        print("(" + bcolors.OKBLUE + model.name + bcolors.ENDC + ") >>> " + command)
        handler(model, command)

    if len(args) > 2:
        quit() 

    # run the interpeter on the model
    interpreter(model)