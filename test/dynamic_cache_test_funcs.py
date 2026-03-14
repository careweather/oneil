"""Test functions for runtime-import dependency caching."""

import importlib
import os

# Use a file to track call count across module reloads
_COUNTER_FILE = "/tmp/oneil_dynamic_test_call_count.txt"


def _get_persistent_count():
    """Get the call count from file."""
    try:
        with open(_COUNTER_FILE, "r") as f:
            return int(f.read().strip())
    except (FileNotFoundError, ValueError):
        return 0


def _set_persistent_count(count):
    """Set the call count in file."""
    with open(_COUNTER_FILE, "w") as f:
        f.write(str(count))


def dynamic_expensive_calculation(x):
    """Import the helper dynamically so cache invalidation must track runtime imports."""
    count = _get_persistent_count() + 1
    _set_persistent_count(count)
    helper = importlib.import_module("dynamic_cache_test_helper")
    return helper.multiply_by_three(x)


def get_call_count():
    """Returns the current call count."""
    return _get_persistent_count()


def reset_call_count():
    """Resets the call count."""
    _set_persistent_count(0)
    try:
        os.remove(_COUNTER_FILE)
    except FileNotFoundError:
        pass
