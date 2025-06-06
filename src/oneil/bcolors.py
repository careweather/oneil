"""
Colors that can be used to change the text on the console
"""

MAGENTA = '\033[95m'
OKBLUE = '\033[94m'
OKCYAN = '\033[96m'
OKGREEN = '\033[92m'
YELLOW = '\033[93m'
ORANGE = '\033[38;5;208m'
FAIL = '\033[91m'
ENDC = '\033[0m'
BOLD = '\033[1m'
UNDERLINE = '\033[4m'
ITALIC = '\033[3m'

def error(msg: str):
    """
    Wrap the message in red
    """

    return f"{FAIL}{BOLD}{msg}{ENDC}"
