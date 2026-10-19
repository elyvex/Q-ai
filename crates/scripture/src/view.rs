//! Read views for scripture.

use serde::{Deserialize, Serialize};

use crate::verse::ScriptureVerse;

/// A scripture view: canonical verse plus (future) translations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptureView {
    /// The canonical verse.
    pub verse: ScriptureVerse,
    /// Translations (empty placeholder until 09-02).
    pub translations: Vec<serde_json::Value>,
}

impl ScriptureView {
    pub fn new(verse: ScriptureVerse) -> Self {
        Self { verse, translations: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edition::ScriptureEditionId;
    use crate::status::ScriptureStatus;
    use crate::verse::ScriptureVerse;
    use domain::{ContentHash, HashAlgorithm, Timestamp};

    fn content_hash(byte: &str) -> ContentHash {
        ContentHash::try_new(HashAlgorithm::Sha256, byte.repeat(32)).unwrap()
    }

    #[test]
    fn view_new_empty_translations() {
        let edition = ScriptureEditionId::new();
        let id = crate::verse::ScriptureVerseId::from_parts(&edition, "genesis", 1, 1);
        let verse = ScriptureVerse::new(
            id,
            edition,
            "genesis".to_string(),
            1,
            1,
            "text".to_string(),
            content_hash("ab"),
            ScriptureStatus::Active,
            Timestamp::from_ymd_hms(2026, 1, 15, 0, 0, 0).unwrap(),
        )
        .unwrap();
        let view = ScriptureView::new(verse);
        assert!(view.translations.is_empty());
    }
}
