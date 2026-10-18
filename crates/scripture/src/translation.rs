//! Scripture translation records (Phase 9, plan 09-02).

use domain::{ContentHash, Language, Timestamp};
use serde::{Deserialize, Serialize};

use crate::status::ScriptureStatus;
use crate::verse::ScriptureVerseId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptureTranslation {
    pub id: String,
    pub verse_id: ScriptureVerseId,
    pub language: Language,
    pub translator: Option<String>,
    pub text: String,
    pub text_hash: ContentHash,
    pub status: ScriptureStatus,
    pub created_at: Timestamp,
}

impl ScriptureTranslation {
    pub fn new(
        id: String,
        verse_id: ScriptureVerseId,
        language: Language,
        translator: Option<String>,
        text: String,
        text_hash: ContentHash,
        status: ScriptureStatus,
        created_at: Timestamp,
    ) -> Self {
        Self { id, verse_id, language, translator, text, text_hash, status, created_at }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verse::ScriptureVerseId;
    use crate::edition::ScriptureEditionId;
    use domain::{HashAlgorithm, Timestamp};

    fn content_hash(byte: &str) -> ContentHash {
        ContentHash::try_new(HashAlgorithm::Sha256, byte.repeat(32)).unwrap()
    }

    #[test]
    fn translation_roundtrips_serde() {
        let edition = ScriptureEditionId::new();
        let verse_id = ScriptureVerseId::from_parts(&edition, "genesis", 1, 1);
        let t = ScriptureTranslation {
            id: "tr-1".to_string(),
            verse_id: verse_id.clone(),
            language: "en".parse().unwrap(),
            translator: Some("Translator".to_string()),
            text: "In the beginning".to_string(),
            text_hash: content_hash("ab"),
            status: ScriptureStatus::Active,
            created_at: Timestamp::from_ymd_hms(2026, 1, 15, 0, 0, 0).unwrap(),
        };
        let json = serde_json::to_string(&t).unwrap();
        let back: ScriptureTranslation = serde_json::from_str(&json).unwrap();
        assert_eq!(back, t);
        assert_eq!(back.verse_id, verse_id);
    }
}
