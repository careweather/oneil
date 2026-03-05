//! Standard builtin unit prefixes (e.g. `k`, `m`, `M`).

/// A builtin unit prefix (e.g. "k" for kilo, "m" for milli).
#[derive(Debug, Clone)]
pub struct BuiltinPrefix {
    pub prefix: &'static str,
    pub value: f64,
    pub description: &'static str,
}

/// Returns an iterator over all standard builtin prefixes.
#[expect(clippy::too_many_lines, reason = "this is a list of builtin prefixes")]
pub fn builtin_prefixes_complete() -> impl Iterator<Item = (&'static str, BuiltinPrefix)> {
    [
        BuiltinPrefix {
            prefix: "q",
            value: 1e-30,
            description: "quecto",
        },
        BuiltinPrefix {
            prefix: "r",
            value: 1e-27,
            description: "ronto",
        },
        BuiltinPrefix {
            prefix: "y",
            value: 1e-24,
            description: "yocto",
        },
        BuiltinPrefix {
            prefix: "z",
            value: 1e-21,
            description: "zepto",
        },
        BuiltinPrefix {
            prefix: "a",
            value: 1e-18,
            description: "atto",
        },
        BuiltinPrefix {
            prefix: "f",
            value: 1e-15,
            description: "femto",
        },
        BuiltinPrefix {
            prefix: "p",
            value: 1e-12,
            description: "pico",
        },
        BuiltinPrefix {
            prefix: "n",
            value: 1e-9,
            description: "nano",
        },
        BuiltinPrefix {
            prefix: "u",
            value: 1e-6,
            description: "micro",
        },
        BuiltinPrefix {
            prefix: "m",
            value: 1e-3,
            description: "milli",
        },
        BuiltinPrefix {
            prefix: "k",
            value: 1e3,
            description: "kilo",
        },
        BuiltinPrefix {
            prefix: "M",
            value: 1e6,
            description: "mega",
        },
        BuiltinPrefix {
            prefix: "G",
            value: 1e9,
            description: "giga",
        },
        BuiltinPrefix {
            prefix: "T",
            value: 1e12,
            description: "tera",
        },
        BuiltinPrefix {
            prefix: "P",
            value: 1e15,
            description: "peta",
        },
        BuiltinPrefix {
            prefix: "E",
            value: 1e18,
            description: "exa",
        },
        BuiltinPrefix {
            prefix: "Z",
            value: 1e21,
            description: "zetta",
        },
        BuiltinPrefix {
            prefix: "Y",
            value: 1e24,
            description: "yotta",
        },
        BuiltinPrefix {
            prefix: "R",
            value: 1e27,
            description: "ronna",
        },
        BuiltinPrefix {
            prefix: "Q",
            value: 1e30,
            description: "quetta",
        },
    ]
    .into_iter()
    .map(|prefix| (prefix.prefix, prefix))
}
