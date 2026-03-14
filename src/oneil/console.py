try:
    from importlib.metadata import version as get_version
except ImportError:
    from pkg_resources import get_distribution

    def get_version(package_name):
        return get_distribution(package_name).version

from . import bcolors

__version__ = get_version("oneil")

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
