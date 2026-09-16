# Sibling-relative Python import paths

## Status

Accepted

## Context

`submodel` and `reference` already accept a `DirectoryPath` (`utils/math`, `../shared/constants`) resolved relative to the importing model file. Python `import` only accepted a single identifier, so `import functions` always looked for `functions.py` next to that `.on` file.

That blocked two common layouts: a model importing a `.py` in another folder (`import ../testing/helpers`), and a model in a subfolder importing the parent's `functions.py` (`import ../functions`).

Dotted Python-style names (`import foo.bar`) would collide with Oneil's `param.alias` meaning and would not match model import syntax. Searching parent directories for a bare `import functions` would be implicit when two `functions.py` files exist.

## Decision

Python imports use the same sibling-relative slash paths as model imports:

```ebnf
ImportDecl = "import", [ DirectoryPath ], Identifier, EndOfLine ;
```

Examples from the importing model's directory:

```oneil
import functions
import ../functions
import ../testing/helpers
import simulations/compass/simmodel/simgeom
```

Bare `import functions` stays same-folder. Resolved paths collapse `.` and `..` for cache and duplicate keys. Each load puts that file's folder on `sys.path` and evicts only same-named local siblings from `sys.modules`, so `import util` finds the script's own folder without unloading site-packages or virtualenv modules. The executed module uses a reserved `__name__` so a user `inspect.py` cannot replace the standard library.

## Consequences

- Model and Python imports share one path syntax.
- A subfolder model must write `import ../functions` rather than inheriting a parent `functions.py`.
- `import foo.bar` remains a parse error.
- Identifiers still cannot contain hyphens, so files such as `discharge-curve.py` cannot be named in an `import` line.
