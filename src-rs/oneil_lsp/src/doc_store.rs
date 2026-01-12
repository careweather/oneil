use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;
use tower_lsp_server::lsp_types::{
    Position, TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentItem, Uri,
    VersionedTextDocumentIdentifier,
};

#[derive(Debug, Clone)]
pub struct DocumentSnapshot {
    pub text: String,
    pub version: i32,
}

#[derive(Debug)]
struct Document {
    text: String,
    version: i32,
    line_offsets: Vec<usize>,
}

impl Document {
    fn new(text: String, version: i32) -> Self {
        let line_offsets = compute_line_offsets(&text);
        Self {
            text,
            version,
            line_offsets,
        }
    }

    fn apply_change(
        &mut self,
        version: i32,
        change: TextDocumentContentChangeEvent,
    ) -> Result<(), String> {
        if version < self.version {
            return Err(format!(
                "out-of-order change (current version {}, new version {})",
                self.version, version
            ));
        }

        match change.range {
            None => {
                // Full-document replacement
                self.text = change.text;
            }
            Some(range) => {
                let start = self
                    .position_to_offset(range.start)
                    .ok_or_else(|| "invalid start position in change".to_string())?;
                let end = self
                    .position_to_offset(range.end)
                    .ok_or_else(|| "invalid end position in change".to_string())?;

                if end < start {
                    return Err("change end before start".to_string());
                }

                // Slice into the old text, insert the new stuff
                let mut new_text =
                    String::with_capacity(self.text.len() - (end - start) + change.text.len());
                new_text.push_str(&self.text[..start]);
                new_text.push_str(&change.text);
                new_text.push_str(&self.text[end..]);
                self.text = new_text;
            }
        }

        self.line_offsets = compute_line_offsets(&self.text);
        self.version = version;
        Ok(())
    }

    fn position_to_offset(&self, position: Position) -> Option<usize> {
        let line = usize::try_from(position.line).ok()?;
        if line >= self.line_offsets.len() {
            return None;
        }

        let (line_start, line_end) = self.line_bounds(line)?;
        let line_text = &self.text[line_start..line_end];
        let utf16_index = usize::try_from(position.character).ok()?;
        let byte_offset_in_line = utf16_to_byte_offset(line_text, utf16_index)?;

        Some(line_start + byte_offset_in_line)
    }

    fn line_bounds(&self, line: usize) -> Option<(usize, usize)> {
        let start = *self.line_offsets.get(line)?;
        let end = if line + 1 < self.line_offsets.len() {
            self.line_offsets[line + 1]
        } else {
            self.text.len()
        };
        Some((start, end))
    }
}

/// Thread-safe in-memory document storage for open buffers.
#[derive(Debug, Clone)]
pub struct DocumentStore {
    docs: Arc<RwLock<HashMap<Uri, Document>>>,
}

impl DocumentStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            docs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn open(&self, doc: TextDocumentItem) {
        let mut docs = self.docs.write().await;
        let document = Document::new(doc.text, doc.version);
        docs.insert(doc.uri, document);
    }

    pub async fn close(&self, identifier: TextDocumentIdentifier) {
        let mut docs = self.docs.write().await;
        docs.remove(&identifier.uri);
    }

    pub async fn apply_changes(
        &self,
        identifier: VersionedTextDocumentIdentifier,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Result<(), String> {
        let mut docs = self.docs.write().await;
        let Some(document) = docs.get_mut(&identifier.uri) else {
            return Err("document not open".to_string());
        };

        let version = identifier.version;
        for change in changes {
            document.apply_change(version, change)?;
        }

        Ok(())
    }

    pub async fn get(&self, uri: &Uri) -> Option<DocumentSnapshot> {
        let docs = self.docs.read().await;
        docs.get(uri).map(|doc| DocumentSnapshot {
            text: doc.text.clone(),
            version: doc.version,
        })
    }

    /// Converts an LSP position to a byte offset in the document.
    pub async fn position_to_offset(&self, uri: &Uri, position: Position) -> Option<usize> {
        let docs = self.docs.read().await;
        docs.get(uri)?.position_to_offset(position)
    }
}

fn compute_line_offsets(text: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    for (idx, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            offsets.push(idx + 1);
        }
    }
    offsets
}

fn utf16_to_byte_offset(line_text: &str, utf16_index: usize) -> Option<usize> {
    let mut utf16_count = 0;
    let mut byte_count = 0;

    for ch in line_text.chars() {
        if utf16_count == utf16_index {
            return Some(byte_count);
        }

        utf16_count += ch.len_utf16();
        byte_count += ch.len_utf8();
    }

    if utf16_count == utf16_index {
        Some(byte_count)
    } else {
        None
    }
}
