"""Helpers that import another module from the same directory."""

import util


def run(x):
    """Return `util.double(x)`, importing `util` at call time."""
    import util as imported_util

    return imported_util.double(x)


def late_run():
    """Import the sibling `util` module inside the function body."""
    import util as imported_util

    return imported_util.double(21)
