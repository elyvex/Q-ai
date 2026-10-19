//! Knowledge domains for cross-domain research.
//!
//! # application::knowledge_domain
//!
//! Defines the five knowledge domains (Quran, Hadith, Tafsir, Scripture, Books)
//! and the query/hit types used by [`crate::cross_domain`]. Default domain set
//! is all five; exclusion is explicit opt-out only.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// A knowledge domain that can participate in cross-domain research.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum KnowledgeDomain {
    Quran,
    Hadith,
    Tafsir,
    Scripture,
    Books,
}

impl KnowledgeDomain {
    /// All five domains (the default set).
    pub fn all() -> BTreeSet<KnowledgeDomain> {
        let mut s = BTreeSet::new();
        s.insert(KnowledgeDomain::Quran);
        s.insert(KnowledgeDomain::Hadith);
        s.insert(KnowledgeDomain::Tafsir);
        s.insert(KnowledgeDomain::Scripture);
        s.insert(KnowledgeDomain::Books);
        s
    }

    pub fn is_quran(&self) -> bool { matches!(self, KnowledgeDomain::Quran) }
    pub fn is_hadith(&self) -> bool { matches!(self, KnowledgeDomain::Hadith) }
    pub fn is_tafsir(&self) -> bool { matches!(self, KnowledgeDomain::Tafsir) }
    pub fn is_scripture(&self) -> bool { matches!(self, KnowledgeDomain::Scripture) }
    pub fn is_books(&self) -> bool { matches!(self, KnowledgeDomain::Books) }

    pub fn as_str(&self) -> &'static str {
        match self {
            KnowledgeDomain::Quran => "quran",
            KnowledgeDomain::Hadith => "hadith",
            KnowledgeDomain::Tafsir => "tafsir",
            KnowledgeDomain::Scripture => "scripture",
            KnowledgeDomain::Books => "books",
        }
    }
}

impl std::fmt::Display for KnowledgeDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A cross-domain research query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossDomainQuery {
    pub query: String,
    pub domains: BTreeSet<KnowledgeDomain>,
}

impl CrossDomainQuery {
    /// Construct with all domains by default.
    pub fn new(query: impl Into<String>) -> Self {
        Self { query: query.into(), domains: KnowledgeDomain::all() }
    }

    /// Construct with an explicit domain set; empty normalizes to all.
    pub fn with_domains(query: impl Into<String>, domains: BTreeSet<KnowledgeDomain>) -> Self {
        let mut q = Self { query: query.into(), domains };
        q.normalize();
        q
    }

    /// Normalize empty domain set to all five.
    pub fn normalize(&mut self) {
        if self.domains.is_empty() {
            self.domains = KnowledgeDomain::all();
        }
    }
}

/// A single hit from cross-domain research, tagged with domain + source + confidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossDomainHit {
    pub domain: KnowledgeDomain,
    pub source: String,
    pub reference: String,
    pub snippet: String,
    pub score: f64,
    pub confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enum_has_five_variants() {
        assert_eq!(KnowledgeDomain::all().len(), 5);
    }

    #[test]
    fn all_returns_all_five() {
        let all = KnowledgeDomain::all();
        assert!(all.contains(&KnowledgeDomain::Quran));
        assert!(all.contains(&KnowledgeDomain::Hadith));
        assert!(all.contains(&KnowledgeDomain::Tafsir));
        assert!(all.contains(&KnowledgeDomain::Scripture));
        assert!(all.contains(&KnowledgeDomain::Books));
    }

    #[test]
    fn empty_domains_normalize_to_all() {
        let mut q = CrossDomainQuery { query: "q".into(), domains: BTreeSet::new() };
        q.normalize();
        assert_eq!(q.domains, KnowledgeDomain::all());
    }

    #[test]
    fn new_defaults_all() {
        let q = CrossDomainQuery::new("q");
        assert_eq!(q.domains, KnowledgeDomain::all());
    }

    #[test]
    fn with_domains_empty_normalizes() {
        let q = CrossDomainQuery::with_domains("q", BTreeSet::new());
        assert_eq!(q.domains, KnowledgeDomain::all());
    }

    #[test]
    fn helpers() {
        assert!(KnowledgeDomain::Quran.is_quran());
        assert!(!KnowledgeDomain::Hadith.is_quran());
    }
}
