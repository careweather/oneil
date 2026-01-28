"""Test functions for function caching."""

import os

# Use a file to track call count across module reloads
_COUNTER_FILE = "/tmp/oneil_test_call_count.txt"

def _get_persistent_count():
    """Get the call count from file."""
    try:
        with open(_COUNTER_FILE, 'r') as f:
            return int(f.read().strip())
    except (FileNotFoundError, ValueError):
        return 0

def _set_persistent_count(count):
    """Set the call count in file."""
    with open(_COUNTER_FILE, 'w') as f:
        f.write(str(count))

def expensive_calculation(x):
    """A function that tracks its call count."""
    count = _get_persistent_count() + 1
    _set_persistent_count(count)
    # Return a Parameter-compatible value (a number)
    if hasattr(x, 'min') and hasattr(x, 'max'):
        return (x.min + x.max) / 2 * 2
    return x * 2

def get_call_count():
    """Returns the current call count."""
    return _get_persistent_count()

def reset_call_count():
    """Resets the call count."""
    _set_persistent_count(0)
    # Also remove the file for cleanliness
    try:
        os.remove(_COUNTER_FILE)
    except FileNotFoundError:
        pass

