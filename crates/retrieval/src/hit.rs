//! One result from a retrieval backend.

use serde::{Deserialize, Serialize};

/// A single retrieval hit.
///
/// `score` is **retrieval relevance only** — how well the backend matched the
/// query. It is not a confidence in the document's claims, not a scholarly
/// grading, and carries no religious weight. Callers must not present it as any
/// of those.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchHit {
    /// The backend's identifier for the matched document.
    pub document_id: String,
    /// Backend relevance score; higher means more relevant. Not comparable
    /// across backends.
    pub score: f32,
    /// Short excerpts around the match, when the backend provides them.
    #[serde(default)]
    pub snippets: Vec<String>,
    /// Which backend produced this hit.
    pub backend: String,
}

impl SearchHit {
    /// Build a hit, stamping it with the backend name.
    pub fn new(
        document_id: impl Into<String>,
        score: f32,
        snippets: Vec<String>,
        backend: impl Into<String>,
    ) -> Self {
        Self { document_id: document_id.into(), score, snippets, backend: backend.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_serializes_with_backend_stamp() {
        let hit = SearchHit::new("frame-7", 0.5, vec!["excerpt".into()], "memvid");
        let json = serde_json::to_value(&hit).unwrap();
        assert_eq!(json["backend"], "memvid");
        assert_eq!(json["document_id"], "frame-7");
    }

    #[test]
    fn snippets_default_to_empty() {
        let hit: SearchHit =
            serde_json::from_str(r#"{"document_id":"f1","score":1.0,"backend":"memvid"}"#).unwrap();
        assert!(hit.snippets.is_empty());
    }
}
