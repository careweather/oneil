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

# Level colors for tree hierarchy (cycling through distinct colors)
LEVEL_COLORS = [
    '\033[38;5;39m',   # Level 1: Bright blue
    '\033[38;5;208m',  # Level 2: Orange
    '\033[38;5;35m',   # Level 3: Green
    '\033[38;5;170m',  # Level 4: Pink/magenta
    '\033[38;5;220m',  # Level 5: Gold
    '\033[38;5;51m',   # Level 6: Cyan
    '\033[38;5;196m',  # Level 7: Red
    '\033[38;5;141m',  # Level 8: Purple
]

def level_color(level: int) -> str:
    """
    Get the color for a given tree level (1-indexed, cycles through colors)
    """
    if level <= 0:
        return ''
    return LEVEL_COLORS[(level - 1) % len(LEVEL_COLORS)]

def error(msg: str):
    """
    Wrap the message in red
    """

    return f"{FAIL}{BOLD}{msg}{ENDC}"
