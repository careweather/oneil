"""Helpers that import another module from the same directory."""

import util


def square_area(side):
    """Return the area of a square with side length `side`."""
    import util as sibling_util

    return sibling_util.area(side)
