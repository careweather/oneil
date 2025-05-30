from pkg_resources import get_distribution

from . import bcolors

__version__ = get_distribution("oneil").version

def print_welcome_message():
    print("Oneil " + __version__)
    print("Type 'help' for a list of commands or see the README for more information.")
    print("-"*80)

def print_error(error):
    notes = ''.join(list(map(lambda note: f"\n  - {note}", error.notes())))
    if error.context() == None:
        print(f"{bcolors.error(error.kind())}: {error.message()}{notes}")
    else:
        print(f"{bcolors.error(error.kind())} {error.context()}: {error.message()}{notes}")
