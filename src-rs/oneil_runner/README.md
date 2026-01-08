
# Oneil Runner

This is a small crate that provides implementations for `BuiltinRef` and
`ModelFileLoader`, as are required by `oneil_model_resolver`. This code
formerly existed within the regular `oneil` crate but was split up so that
`oneil_lsp` could use them too. As mutual dependencies grow, they may be added
to this crate.

