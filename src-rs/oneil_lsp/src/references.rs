//! Find-all-references support using [`crate::occurrences`] lookup.

use oneil_runtime::Runtime;
use oneil_shared::paths::ModelPath;
use tower_lsp_server::ls_types::Location;

use crate::{
    location::span_to_location,
    occurrences::{collect_occurrences, resolve_search_target},
    symbol_lookup::SymbolAtPosition,
};

/// Returns all reference locations for `symbol` across loaded models.
pub fn reference_locations(
    symbol: &SymbolAtPosition,
    runtime: &mut Runtime,
    current_model_path: &ModelPath,
) -> Option<Vec<Location>> {
    let target = resolve_search_target(symbol, runtime, current_model_path)?;

    let locations = collect_occurrences(&target, runtime)
        .into_iter()
        .map(|occurrence| span_to_location(&occurrence.model_path, &occurrence.span))
        .collect();

    Some(locations)
}
