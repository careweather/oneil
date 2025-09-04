use oneil_ir as ir;
use oneil_model_loader::BuiltinRef;

// TODO: later, this will hold the actual values/functions that are built into the language
//       right now, it just holds the names of the builtins
pub struct Builtins;

impl Builtins {
    pub const fn new() -> Self {
        Self
    }
}

impl BuiltinRef for Builtins {
    fn has_builtin_value(&self, identifier: &ir::Identifier) -> bool {
        matches!(identifier.as_str(), "pi" | "e" | "inf")
    }

    fn has_builtin_function(&self, identifier: &ir::Identifier) -> bool {
        matches!(
            identifier.as_str(),
            "min"
                | "max"
                | "sin"
                | "cos"
                | "tan"
                | "asin"
                | "acos"
                | "atan"
                | "sqrt"
                | "ln"
                | "log"
                | "log10"
                | "floor"
                | "ceiling"
                | "extent"
                | "range"
                | "abs"
                | "sign"
                | "mid"
                | "strip"
                | "mnmx"
        )
    }
}
