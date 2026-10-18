//! Scripture reference: (edition, book, chapter, verse).

use serde::{Deserialize, Serialize};

use crate::verse::ScriptureVerseId;

/// A scripture reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptureRef {
    /// Edition slug or id.
    pub edition: String,
    /// Book slug.
    pub book: String,
    /// Chapter number (1-based).
    pub chapter: u32,
    /// Verse number (1-based).
    pub verse: u32,
}

impl ScriptureRef {
    /// Parse a reference string `edition:book:chapter:verse`.
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 4 {
            return None;
        }
        let chapter: u32 = parts[2].parse().ok()?;
        let verse: u32 = parts[3].parse().ok()?;
        if chapter == 0 || verse == 0 {
            return None;
        }
        if parts[0].is_empty() || parts[1].is_empty() {
            return None;
        }
        Some(Self {
            edition: parts[0].to_string(),
            book: parts[1].to_string(),
            chapter,
            verse,
        })
    }

    /// Serialize as `edition:book:chapter:verse`.
    pub fn to_ref_string(&self) -> String {
        format!("{}:{}:{}:{}", self.edition, self.book, self.chapter, self.verse)
    }

    /// Build a ScriptureVerseId if edition id provided? Helper takes edition id.
    pub fn verse_id(&self, edition_id: &crate::edition::ScriptureEditionId) -> ScriptureVerseId {
        ScriptureVerseId::from_parts(edition_id, &self.book, self.chapter, self.verse)
    }
}

impl std::fmt::Display for ScriptureRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}:{}", self.edition, self.book, self.chapter, self.verse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_roundtrip() {
        let r = ScriptureRef::parse("tanakh:genesis:1:1").unwrap();
        assert_eq!(r.edition, "tanakh");
        assert_eq!(r.book, "genesis");
        assert_eq!(r.chapter, 1);
        assert_eq!(r.verse, 1);
        assert_eq!(r.to_string(), "tanakh:genesis:1:1");
    }

    #[test]
    fn parse_rejects_invalid() {
        assert!(ScriptureRef::parse("tanakh:genesis:0:1").is_none());
        assert!(ScriptureRef::parse("tanakh:genesis:1").is_none());
    }
}
