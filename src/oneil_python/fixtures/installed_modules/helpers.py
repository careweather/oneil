"""Uses a standard-library module the way models use site-packages."""

import math


def circle_area():
    """Return a coarse integer area using `math.pi`."""
    return int(math.pi)


def late_math():
    """Import `math` at call time so `sys.modules` must still have it."""
    import math as imported_math

    return int(imported_math.pi)
