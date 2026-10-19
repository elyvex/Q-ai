//! Scripture book model.

use serde::{Deserialize, Serialize};

use crate::edition::ScriptureEditionId;

/// Identifies a scripture book.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct ScriptureBookId(uuid::Uuid);

impl ScriptureBookId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
    pub const fn inner(&self) -> uuid::Uuid {
        self.0
    }
}
impl Default for ScriptureBookId {
    fn default() -> Self {
        Self::new()
    }
}
impl std::fmt::Display for ScriptureBookId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::str::FromStr for ScriptureBookId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = uuid::Uuid::parse_str(s).map_err(|e| e.to_string())?;
        Ok(Self(id))
    }
}
impl From<ScriptureBookId> for uuid::Uuid {
    fn from(id: ScriptureBookId) -> Self {
        id.0
    }
}

/// A book within a scripture edition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptureBook {
    /// Stable book identity.
    pub id: ScriptureBookId,
    /// The edition this book belongs to.
    pub edition_id: ScriptureEditionId,
    /// URL/CLI-safe slug.
    pub slug: String,
    /// Human-readable name.
    pub name: String,
    /// Ordering within the edition.
    pub book_order: i32,
    /// Number of chapters.
    pub chapter_count: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edition::ScriptureEditionId;

    #[test]
    fn book_roundtrips_serde() {
        let book = ScriptureBook {
            id: ScriptureBookId::new(),
            edition_id: ScriptureEditionId::new(),
            slug: "genesis".to_string(),
            name: "Genesis".to_string(),
            book_order: 1,
            chapter_count: 50,
        };
        let json = serde_json::to_string(&book).unwrap();
        let back: ScriptureBook = serde_json::from_str(&json).unwrap();
        assert_eq!(back, book);
    }
}
