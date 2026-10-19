//! A document going into a retrieval backend.

use serde::{Deserialize, Serialize};

/// One retrievable document.
///
/// `text` is the only field a backend is required to index. `metadata` and
/// `id` are carried through untouched so a hit can be traced back to where it
/// came from — the same provenance discipline the rest of Q-ai follows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    /// Caller-assigned identifier. Backends that mint their own ids ignore it.
    #[serde(default)]
    pub id: Option<String>,
    /// Short human-readable label, e.g. a work title.
    #[serde(default)]
    pub title: Option<String>,
    /// The indexable body text.
    pub text: String,
    /// Arbitrary caller metadata, preserved verbatim.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl Document {
    /// A document with just text set.
    pub fn new(text: impl Into<String>) -> Self {
        Self { id: None, title: None, text: text.into(), metadata: serde_json::Value::Null }
    }

    /// Attach a caller-assigned id.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Attach a title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Attach arbitrary metadata.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_sets_fields() {
        let doc = Document::new("body")
            .with_id("doc-1")
            .with_title("Work")
            .with_metadata(serde_json::json!({"surah": 1}));
        assert_eq!(doc.id.as_deref(), Some("doc-1"));
        assert_eq!(doc.title.as_deref(), Some("Work"));
        assert_eq!(doc.text, "body");
        assert_eq!(doc.metadata["surah"], 1);
    }

    #[test]
    fn minimal_document_needs_only_text() {
        let doc: Document = serde_json::from_str(r#"{"text":"body"}"#).unwrap();
        assert_eq!(doc.text, "body");
        assert!(doc.id.is_none());
    }
}
