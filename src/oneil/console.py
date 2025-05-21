from pkg_resources import get_distribution

import bcolors

__version__ = get_distribution("oneil").version

def print_welcome_message():
    print("Oneil " + __version__)
    print("Type 'help' for a list of commands or see the README for more information.")
    print("-"*80)

def print_error(msg : str):
    print(f"{bcolors.FAIL}{bcolors.BOLD}ERROR{bcolors.ENDC}: {msg}")
