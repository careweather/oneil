"""Helpers that import `util` from this directory."""

import util


def run():
    """Return this directory's `util.value()`, importing it at call time."""
    import util as imported_util

    return imported_util.value()
