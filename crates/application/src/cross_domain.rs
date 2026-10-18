//! Cross-domain research aggregator.
//!
//! # application::cross_domain
//!
//! Fans out a [`CrossDomainQuery`] to per-domain search services and the RAG
//! books store, merging results tagged with domain + source + confidence. Per-domain
//! results are preserved (no collapse into a single consensus).

use std::collections::BTreeSet;
use std::sync::Arc;

use async_trait::async_trait;


use crate::knowledge_domain::{CrossDomainHit, CrossDomainQuery, KnowledgeDomain};
use crate::rag_search::RagSearchService;

/// Error type for domain searchers.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum CrossDomainError {
    #[error("domain search failed: {0}")]
    Search(String),
}

/// A per-domain search service abstraction (mockable).
#[async_trait]
pub trait DomainSearcher: Send + Sync {
    async fn search(&self, query: &str) -> Result<Vec<CrossDomainHit>, CrossDomainError>;
}

/// Cross-domain research aggregator.
pub struct CrossDomainResearchService {
    quran: Option<Arc<dyn DomainSearcher>>,
    hadith: Option<Arc<dyn DomainSearcher>>,
    tafsir: Option<Arc<dyn DomainSearcher>>,
    scripture: Option<Arc<dyn DomainSearcher>>,
    books: Option<Arc<dyn DomainSearcher>>,
}

impl CrossDomainResearchService {
    pub fn new(
        quran: Option<Arc<dyn DomainSearcher>>,
        hadith: Option<Arc<dyn DomainSearcher>>,
        tafsir: Option<Arc<dyn DomainSearcher>>,
        scripture: Option<Arc<dyn DomainSearcher>>,
        books: Option<Arc<dyn DomainSearcher>>,
    ) -> Self {
        Self { quran, hadith, tafsir, scripture, books }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn with_services(
        quran: Option<Arc<dyn DomainSearcher>>,
        hadith: Option<Arc<dyn DomainSearcher>>,
        tafsir: Option<Arc<dyn DomainSearcher>>,
        scripture: Option<Arc<dyn DomainSearcher>>,
        books: Option<Arc<dyn DomainSearcher>>,
    ) -> Self {
        Self::new(quran, hadith, tafsir, scripture, books)
    }

    pub fn with_quran(mut self, s: Option<Arc<dyn DomainSearcher>>) -> Self {
        self.quran = s; self
    }
    pub fn with_hadith(mut self, s: Option<Arc<dyn DomainSearcher>>) -> Self {
        self.hadith = s; self
    }
    pub fn with_tafsir(mut self, s: Option<Arc<dyn DomainSearcher>>) -> Self {
        self.tafsir = s; self
    }
    pub fn with_scripture(mut self, s: Option<Arc<dyn DomainSearcher>>) -> Self {
        self.scripture = s; self
    }
    pub fn with_books(mut self, s: Option<Arc<dyn DomainSearcher>>) -> Self {
        self.books = s; self
    }

    /// Convenience: accept a RagSearchService for the books domain.
    pub fn with_rag_service(self, rag: Option<Arc<RagSearchService>>) -> Self {
        let books = rag.map(|r| Arc::new(RagBooksAdapter(r)) as Arc<dyn DomainSearcher>);
        self.with_books(books)
    }

    /// Generic domain assignment.
    pub fn with_domain(mut self, domain: KnowledgeDomain, s: Arc<dyn DomainSearcher>) -> Self {
        match domain {
            KnowledgeDomain::Quran => self.quran = Some(s),
            KnowledgeDomain::Hadith => self.hadith = Some(s),
            KnowledgeDomain::Tafsir => self.tafsir = Some(s),
            KnowledgeDomain::Scripture => self.scripture = Some(s),
            KnowledgeDomain::Books => self.books = Some(s),
        }
        self
    }

    pub fn get(&self, domain: KnowledgeDomain) -> Option<Arc<dyn DomainSearcher>> {
        match domain {
            KnowledgeDomain::Quran => self.quran.clone(),
            KnowledgeDomain::Hadith => self.hadith.clone(),
            KnowledgeDomain::Tafsir => self.tafsir.clone(),
            KnowledgeDomain::Scripture => self.scripture.clone(),
            KnowledgeDomain::Books => self.books.clone(),
        }
    }

    /// Fan out to selected domains only.
    pub async fn research(&self, mut query: CrossDomainQuery) -> Vec<CrossDomainHit> {
        query.normalize();
        let domains = query.domains.clone();
        let mut hits = Vec::new();
        let mut queried: BTreeSet<KnowledgeDomain> = BTreeSet::new();
        for domain in domains {
            let svc = self.get(domain);
            if let Some(svc) = svc {
                queried.insert(domain);
                if let Ok(mut res) = svc.search(&query.query).await {
                    // ensure domain tagging? trust service; but force domain if default? leave
                    for h in &mut res {
                        if h.domain != domain {
                            // keep as-is? Tag with queried domain if unset? We'll not override.
                        }
                    }
                    hits.extend(res);
                }
            }
        }
        // sort by score desc, stable by domain then source
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.domain.cmp(&b.domain))
                .then_with(|| a.source.cmp(&b.source))
        });
        hits
    }

    /// Alias.
    pub async fn search(&self, query: CrossDomainQuery) -> Vec<CrossDomainHit> {
        self.research(query).await
    }

    /// Alias.
    pub async fn aggregate(&self, query: CrossDomainQuery) -> Vec<CrossDomainHit> {
        self.research(query).await
    }
}

/// Adapter wrapping RagSearchService for books domain.
struct RagBooksAdapter(Arc<RagSearchService>);

#[async_trait]
impl DomainSearcher for RagBooksAdapter {
    async fn search(&self, query: &str) -> Result<Vec<CrossDomainHit>, CrossDomainError> {
        let mut filters = std::collections::BTreeMap::new();
        filters.insert("domain".to_string(), "books".to_string());
        match self.0.query("books", query, filters, 20u32).await {
            Ok((hits, _rd)) => Ok(hits
                .into_iter()
                .map(|h| CrossDomainHit {
                    domain: KnowledgeDomain::Books,
                    source: h.source,
                    reference: h.doc_id,
                    snippet: String::new(),
                    score: h.score as f64,
                    confidence: 1.0,
                })
                .collect()),
            Err(e) => Err(CrossDomainError::Search(e.to_string())),
        }
    }
}

// Implement DomainSearcher for concrete services.
use crate::hadith_search::{HadithSearchService, HadithSearchParams};
use crate::tafsir_search::{TafsirSearchService, TafsirSearchParams};
use crate::scripture_search::{ScriptureSearchService, ScriptureSearchParams};

#[async_trait]
impl DomainSearcher for HadithSearchService {
    async fn search(&self, query: &str) -> Result<Vec<CrossDomainHit>, CrossDomainError> {
        let params = HadithSearchParams { query: query.to_string(), collection_slug: None, limit: Some(20) };
        match self.search(&params).await {
            Ok(res) => Ok(res.into_iter().map(|r| CrossDomainHit {
                domain: KnowledgeDomain::Hadith,
                source: r.collection_slug.clone(),
                reference: r.canonical_reference.clone(),
                snippet: r.snippet.clone(),
                score: 1.0,
                confidence: 1.0,
            }).collect()),
            Err(e) => Err(CrossDomainError::Search(e.to_string())),
        }
    }
}

#[async_trait]
impl DomainSearcher for TafsirSearchService {
    async fn search(&self, query: &str) -> Result<Vec<CrossDomainHit>, CrossDomainError> {
        let params = TafsirSearchParams { query: query.to_string(), work_slug: None, limit: Some(20) };
        match self.search(&params).await {
            Ok(res) => Ok(res.into_iter().map(|r| CrossDomainHit {
                domain: KnowledgeDomain::Tafsir,
                source: r.work_slug.clone(),
                reference: r.canonical_reference.clone(),
                snippet: r.snippet.clone(),
                score: 1.0,
                confidence: 1.0,
            }).collect()),
            Err(e) => Err(CrossDomainError::Search(e.to_string())),
        }
    }
}

#[async_trait]
impl DomainSearcher for ScriptureSearchService {
    async fn search(&self, query: &str) -> Result<Vec<CrossDomainHit>, CrossDomainError> {
        let params = ScriptureSearchParams { query: query.to_string(), edition: None, limit: Some(20) };
        match self.search(&params).await {
            Ok(res) => Ok(res.into_iter().map(|r| CrossDomainHit {
                domain: KnowledgeDomain::Scripture,
                source: r.edition.clone(),
                reference: r.verse_id.to_string(),
                snippet: r.snippet.clone(),
                score: r.score,
                confidence: 1.0,
            }).collect()),
            Err(e) => Err(CrossDomainError::Search(e.to_string())),
        }
    }
}

#[async_trait]
impl DomainSearcher for RagSearchService {
    async fn search(&self, query: &str) -> Result<Vec<CrossDomainHit>, CrossDomainError> {
        // Default books project; filter domain=books.
        let mut filters = std::collections::BTreeMap::new();
        filters.insert("domain".to_string(), "books".to_string());
        match self.query("books", query, filters, 20u32).await {
            Ok((hits, _rd)) => Ok(hits.into_iter().map(|h| CrossDomainHit {
                domain: KnowledgeDomain::Books,
                source: h.source,
                reference: h.doc_id,
                snippet: String::new(),
                score: h.score as f64,
                confidence: 1.0,
            }).collect()),
            Err(e) => Err(CrossDomainError::Search(e.to_string())),
        }
    }
}

// Remove unused with_rag_service duplicate above? We defined books via adapter; simplify:
// Override with_rag_service to use adapter.
impl CrossDomainResearchService {
    pub fn with_rag(self, rag: Option<Arc<RagSearchService>>) -> Self {
        let books = rag.map(|r| Arc::new(RagBooksAdapter(r)) as Arc<dyn DomainSearcher>);
        self.with_books(books)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct MockSearcher {
        called: Arc<AtomicUsize>,
        hits: Vec<CrossDomainHit>,
    }

    #[async_trait]
    impl DomainSearcher for MockSearcher {
        async fn search(&self, _query: &str) -> Result<Vec<CrossDomainHit>, CrossDomainError> {
            self.called.fetch_add(1, Ordering::SeqCst);
            Ok(self.hits.clone())
        }
    }

    fn hit(domain: KnowledgeDomain) -> CrossDomainHit {
        CrossDomainHit {
            domain,
            source: "src".into(),
            reference: "ref".into(),
            snippet: "snip".into(),
            score: 0.5,
            confidence: 0.9,
        }
    }

    #[tokio::test]
    async fn excluded_domain_not_called() {
        let called_excluded = Arc::new(AtomicUsize::new(0));
        let excluded = Arc::new(MockSearcher {
            called: called_excluded.clone(),
            hits: vec![hit(KnowledgeDomain::Scripture)],
        });
        let svc = CrossDomainResearchService::new(
            None,
            None,
            None,
            Some(excluded),
            None,
        );
        let q = CrossDomainQuery::with_domains("q", [KnowledgeDomain::Quran].into_iter().collect());
        // quran has no service; scripture excluded -> not called
        let res = svc.research(q).await;
        assert!(res.is_empty());
        assert_eq!(called_excluded.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn default_all_and_tagged() {
        let called = Arc::new(AtomicUsize::new(0));
        let mock = Arc::new(MockSearcher {
            called: called.clone(),
            hits: vec![hit(KnowledgeDomain::Hadith)],
        });
        let svc = CrossDomainResearchService::new(None, Some(mock), None, None, None);
        // empty domains normalize to all -> hadith service called
        let q = CrossDomainQuery { query: "q".into(), domains: BTreeSet::new() };
        let res = svc.research(q).await;
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].domain, KnowledgeDomain::Hadith);
        assert_eq!(called.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn disagreement_preserved() {
        let h1 = Arc::new(MockSearcher {
            called: Arc::new(AtomicUsize::new(0)),
            hits: vec![CrossDomainHit {
                domain: KnowledgeDomain::Quran,
                source: "q".into(),
                reference: "r1".into(),
                snippet: "A".into(),
                score: 0.9,
                confidence: 1.0,
            }],
        });
        let h2 = Arc::new(MockSearcher {
            called: Arc::new(AtomicUsize::new(0)),
            hits: vec![CrossDomainHit {
                domain: KnowledgeDomain::Hadith,
                source: "h".into(),
                reference: "r2".into(),
                snippet: "B".into(),
                score: 0.8,
                confidence: 1.0,
            }],
        });
        let svc = CrossDomainResearchService::new(Some(h1), Some(h2), None, None, None);
        let q = CrossDomainQuery::new("q");
        // restrict to two domains
        let mut q = q;
        q.domains = [KnowledgeDomain::Quran, KnowledgeDomain::Hadith].into_iter().collect();
        let res = svc.research(q).await;
        assert_eq!(res.len(), 2);
    }

    #[test]
    fn hit_serialization() {
        let h = hit(KnowledgeDomain::Books);
        let json = serde_json::to_string(&h).unwrap();
        let back: CrossDomainHit = serde_json::from_str(&json).unwrap();
        assert_eq!(h, back);
    }
}
