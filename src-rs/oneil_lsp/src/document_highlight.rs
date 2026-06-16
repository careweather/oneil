//! Document highlight support using [`crate::occurrences`] lookup.

use oneil_runtime::Runtime;
use oneil_shared::paths::ModelPath;
use tower_lsp_server::ls_types::{DocumentHighlight, DocumentHighlightKind};

use crate::{
    location::span_to_range,
    occurrences::{collect_occurrences, resolve_search_target},
    symbol_lookup::SymbolAtPosition,
};

/// Returns highlight ranges for `symbol` occurrences in `current_model_path`.
pub fn document_highlights(
    symbol: &SymbolAtPosition,
    runtime: &mut Runtime,
    current_model_path: &ModelPath,
) -> Option<Vec<DocumentHighlight>> {
    let target = resolve_search_target(symbol, runtime, current_model_path)?;

    let highlights = collect_occurrences(&target, runtime)
        .into_iter()
        .filter(|occurrence| occurrence.model_path == *current_model_path)
        .map(|occurrence| DocumentHighlight {
            range: span_to_range(&occurrence.span),
            kind: Some(DocumentHighlightKind::READ),
        })
        .collect();

    Some(highlights)
}
