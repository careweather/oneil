"""Test functions for fallback calculation testing."""

def working_function(x):
    """A function that works."""
    if hasattr(x, 'min') and hasattr(x, 'max'):
        return (x.min + x.max) / 2 * 3
    return x * 3

def broken_function(x):
    """A function that always fails."""
    raise RuntimeError("This function is intentionally broken")

def missing_dependency_function(x):
    """A function that simulates a missing import."""
    import nonexistent_module_xyz  # This will fail
    return x * 2

