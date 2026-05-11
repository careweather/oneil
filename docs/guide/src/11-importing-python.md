# Importing Python Functions

For functions not supported by the above equation formats, you can define a
python function and link it.

Functions are stored in a separate python file, which must be imported in the
Oneil file.

```oneil
import <name of functions file>
```

That file should simply define functions matching the name used in the
parameter:

```py
import numpy as np

def temperature(transit_mode):
    ...
```

In the Oneil file, give the python function on the right hand of the equation,
including other parameters as inputs:

```oneil
Temperature: T = temperature(D) :K
```

## Fallback Calculations

Python functions may have dependencies that aren't always available, or may take
a long time to run. You can specify a fallback calculation using the `?`
operator. If the Python function fails (e.g., missing dependencies, runtime
errors), Oneil will use the fallback and warn the user:

```oneil
Temperature: T = expensive_simulation(D) ? D * 0.5 + 273 :K
```

In this example, if `expensive_simulation` fails, Oneil will calculate
`D * 0.5 + 273` instead and display a warning that the Python function should be
run for greater accuracy.

This is particularly useful for:

- Sharing models with users who may not have all Python dependencies installed
- Providing quick approximations during iterative design
- Graceful degradation when simulations fail

## Function Caching

> [!NOTE]
> This is not currently implemented in the Rust implementation but will be
> implemented soon.

Python function results are automatically cached to avoid re-running expensive calculations. The cache:

- **Persists across REPL sessions** - Close and reopen Oneil, cached results
  remain
- **Is version-controllable** - Stored as one cache file per model under
  `__oncache__/`
- **Is human-readable** - Each model cache stores the simulation function,
  simulation file, and parameter input/output snapshots (`min`, `max`, `units`)
  as JSON for cleaner git diffs
- **Is shareable** - Other users can use cached results even without Python
  dependencies as long as function arguments match the cached arguments
- **Only rewrites changed entries** - Re-running Oneil leaves the cache file
  untouched unless a simulation's latest cached inputs or output changed
- **Auto-invalidates** when imported Python source files, their local Python
  dependencies, or the simulation inputs change
