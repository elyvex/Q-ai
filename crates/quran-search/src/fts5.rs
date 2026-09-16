//! Phase 2 — FTS5 backend adapter (P2-T30, DEV-05).
//!
//! SQLite FTS5 behind the [`FullTextIndex`] port. Generations are directories:
//! `<root>/gen-<N>/` holds `index.db` (one FTS5 table + manifest file).
//! Staging builds a fresh generation directory; activation (pointer flip)
//! belongs to the build job (P2-T34) via `index_pointers`, not to this
//! adapter.
//!
//! Normalization boundary: [`FtsDoc`](crate::model::FtsDoc) fields carry
//! source text; this adapter normalizes every `text_*` field through the
//! shared [`TokenizerFamily`] on **both** paths (documents in
//! [`FullTextIndex::add_batch`], terms in [`FullTextIndex::search`]). Rules
//! are idempotent, so pre-normalized inputs (e.g. stored forms from the
//! build job) pass through unchanged.
//!
//! Regex (I16) runs DFA-only over the term dictionary (`fts5vocab`):
//! anchored patterns expand to a bounded OR of matching terms; leading
//! `.*`/`.+` is rejected; expansions cap at 128 terms.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};

use crate::error::IndexError;
use crate::index::FullTextIndex;
use crate::model::{
    CommitStamp, FieldId, FtsBackend, FtsDoc, FtsHit, FtsIntegrityReport, FtsQuery, FtsResults,
    FtsSchema, FtsStats, IndexManifest, ResultOrder, SearchOpts,
};
use crate::tokenizer::{INDEXED_FIELDS, TokenizerFamily};

/// Indexed text columns in FTS table order (highlight addressing depends on it).
const TEXT_COLUMNS: [&str; 7] = INDEXED_FIELDS;
/// Lexicon columns (populated from M4; empty strings until then).
const LEX_COLUMNS: [&str; 5] = ["roots", "lemmas", "stems", "pos_tags", "patterns"];
/// I16 budgets (construction budgets live in [`crate::regex`]).
const MAX_REGEX_EXPANSION: usize = 128;
const MAX_VOCAB_SCAN: usize = 50_000;

/// FTS5-backed [`FullTextIndex`], bound to one generation directory.
#[derive(Debug)]
pub struct Fts5Index {
    root: PathBuf,
    /// Build generation backing this instance (`gen-<N>` directory).
    /// Independent from `manifest.corpus_generation`: several builds may
    /// target one corpus generation (retained for single-step rollback).
    generation: u64,
    manifest: IndexManifest,
    family: TokenizerFamily,
    pool: SqlitePool,
}

/// Compiled MATCH plan: an expression, possibly with regex provenance.
///
/// `Unsatisfiable` matches nothing (an emptied term, an empty expansion);
/// `Unconstrained` is `FtsQuery::All`. Only a top-level regex carries its
/// expansion stats; nested regexes contribute their expression alone.
#[derive(Debug, Clone, PartialEq, Eq)]
enum MatchPlan {
    Unsatisfiable,
    Unconstrained,
    Expr(String),
    Regex { expr: String, terms: Vec<String>, examined: u64 },
}

impl Fts5Index {
    /// Generation directory for `generation` under `root`.
    fn gen_dir(root: &Path, generation: u64) -> PathBuf {
        root.join(format!("gen-{generation}"))
    }

    /// Database file inside a generation directory.
    fn db_path(root: &Path, generation: u64) -> PathBuf {
        Self::gen_dir(root, generation).join("index.db")
    }

    async fn connect(path: &Path, create: bool) -> Result<SqlitePool, IndexError> {
        SqlitePoolOptions::new()
            .max_connections(4)
            .connect_with(SqliteConnectOptions::new().filename(path).create_if_missing(create))
            .await
            .map_err(|err| IndexError::BuildFailed {
                stage: "connect".to_string(),
                detail: err.to_string(),
            })
    }

    /// Stage a fresh generation for building (wipes any previous staging).
    ///
    /// # Errors
    ///
    /// Returns [`IndexError::ManifestMismatch`] when the tokenizer family
    /// version disagrees with the manifest, or [`IndexError::BuildFailed`]
    /// when the directory cannot be prepared.
    pub async fn stage(
        root: &Path,
        generation: u64,
        manifest: IndexManifest,
        family: TokenizerFamily,
    ) -> Result<Self, IndexError> {
        if family.version() != manifest.tokenizer_version {
            return Err(IndexError::ManifestMismatch {
                detail: format!(
                    "tokenizer family {} disagrees with manifest {}",
                    family.version(),
                    manifest.tokenizer_version
                ),
            });
        }
        let dir = Self::gen_dir(root, generation);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).map_err(|err| IndexError::BuildFailed {
                stage: "stage".to_string(),
                detail: err.to_string(),
            })?;
        }
        std::fs::create_dir_all(&dir).map_err(|err| IndexError::BuildFailed {
            stage: "stage".to_string(),
            detail: err.to_string(),
        })?;
        let pool = Self::connect(&Self::db_path(root, generation), true).await?;
        let index = Self { root: root.to_path_buf(), generation, manifest, family, pool };
        index.create_tables().await?;
        let manifest_json = serde_json::to_string_pretty(&index.manifest).map_err(|err| {
            IndexError::BuildFailed { stage: "stage".to_string(), detail: err.to_string() }
        })?;
        std::fs::write(dir.join("manifest.json"), manifest_json).map_err(|err| {
            IndexError::BuildFailed { stage: "stage".to_string(), detail: err.to_string() }
        })?;
        Ok(index)
    }

    /// Open a previously built generation for serving.
    ///
    /// # Errors
    ///
    /// Returns [`IndexError::BuildFailed`] when the generation directory or
    /// database is missing.
    pub async fn open(
        root: &Path,
        generation: u64,
        manifest: IndexManifest,
        family: TokenizerFamily,
    ) -> Result<Self, IndexError> {
        let path = Self::db_path(root, generation);
        if !path.exists() {
            return Err(IndexError::BuildFailed {
                stage: "open".to_string(),
                detail: format!("missing generation database {}", path.display()),
            });
        }
        let pool = Self::connect(&path, false).await?;
        Ok(Self { root: root.to_path_buf(), generation, manifest, family, pool })
    }

    async fn create_tables(&self) -> Result<(), IndexError> {
        let mut columns = vec!["doc_id UNINDEXED".to_string()];
        columns.extend(TEXT_COLUMNS.iter().map(|c| (*c).to_string()));
        columns.extend(LEX_COLUMNS.iter().map(|c| (*c).to_string()));
        columns.extend(
            ["surah", "ayah", "global_index", "juz", "page", "generation"]
                .iter()
                .map(|c| format!("{c} UNINDEXED")),
        );
        columns.push("revelation UNINDEXED".to_string());
        let ddl = format!(
            "CREATE VIRTUAL TABLE IF NOT EXISTS ayah_fts USING fts5({})",
            columns.join(", ")
        );
        sqlx::query(&ddl).execute(&self.pool).await.map_err(|err| IndexError::BuildFailed {
            stage: "create".to_string(),
            detail: err.to_string(),
        })?;
        sqlx::query("CREATE VIRTUAL TABLE IF NOT EXISTS vocab USING fts5vocab(ayah_fts, 'col')")
            .execute(&self.pool)
            .await
            .map_err(|err| IndexError::BuildFailed {
                stage: "create".to_string(),
                detail: err.to_string(),
            })?;
        Ok(())
    }

    /// Normalize one document's text fields through the shared family.
    fn normalize_doc(&self, doc: &FtsDoc) -> Result<BTreeMap<FieldId, String>, IndexError> {
        let mut out = BTreeMap::new();
        for field in TEXT_COLUMNS {
            let source = doc.fields.get(field).map(String::as_str).unwrap_or("");
            out.insert(field.to_string(), self.family.tokenize(field, source)?);
        }
        Ok(out)
    }
}

impl Fts5Index {
    async fn match_expression(&self, query: &FtsQuery) -> Result<MatchPlan, IndexError> {
        match query {
            FtsQuery::Term { field, term } => {
                let normalized = self.family.tokenize(field, term)?;
                if normalized.trim().is_empty() {
                    return Ok(MatchPlan::Unsatisfiable);
                }
                Ok(MatchPlan::Expr(format!("{{ {field} }} : {}", quote(&normalized))))
            }
            FtsQuery::Phrase { field, terms, slop, ordered } => {
                let mut normalized = Vec::with_capacity(terms.len());
                for term in terms {
                    let piece = self.family.tokenize(field, term)?;
                    if !piece.trim().is_empty() {
                        normalized.push(piece);
                    }
                }
                if normalized.is_empty() {
                    return Ok(MatchPlan::Unsatisfiable);
                }
                if *ordered && *slop == 0 {
                    let phrase = normalized
                        .iter()
                        .map(|t| quote_phrase_term(t))
                        .collect::<Vec<_>>()
                        .join(" ");
                    Ok(MatchPlan::Expr(format!("{{ {field} }} : \"{phrase}\"")))
                } else {
                    let inner = normalized.join(" ");
                    Ok(MatchPlan::Expr(format!("{{ {field} }} : NEAR({inner}, {slop})")))
                }
            }
            FtsQuery::Boolean { must, should, must_not } => {
                // FTS5 exclusion is `<expr> NOT <expr>` (there is no `AND NOT`
                // and no leading `NOT`). A `must_not`-only query is therefore
                // inexpressible — and always a caller bug, since search tools
                // build positive clauses — so it is rejected, never widened.
                if must.is_empty() && should.is_empty() && !must_not.is_empty() {
                    return Err(IndexError::QueryRejected {
                        detail: "must_not requires a positive clause".to_string(),
                    });
                }
                // `None` from a subquery means unsatisfiable: an unsatisfiable
                // `must` poisons the conjunction, unsatisfiable `should`s are
                // dropped (an OR of nothing with no `must` stays unsatisfiable),
                // and unsatisfiable `must_not`s constrain nothing.
                let mut parts = Vec::new();
                for sub in must {
                    match Box::pin(self.match_expression(sub)).await? {
                        MatchPlan::Expr(expr) | MatchPlan::Regex { expr, .. } => {
                            parts.push(format!("({expr})"))
                        }
                        MatchPlan::Unsatisfiable => return Ok(MatchPlan::Unsatisfiable),
                        MatchPlan::Unconstrained => {}
                    }
                }
                if !should.is_empty() {
                    let mut options = Vec::new();
                    for sub in should {
                        if let MatchPlan::Expr(expr) | MatchPlan::Regex { expr, .. } =
                            Box::pin(self.match_expression(sub)).await?
                        {
                            options.push(format!("({expr})"));
                        }
                    }
                    if options.is_empty() {
                        if parts.is_empty() {
                            return Ok(MatchPlan::Unsatisfiable);
                        }
                    } else {
                        parts.push(format!("({})", options.join(" OR ")));
                    }
                }
                let mut expression = parts.join(" AND ");
                for sub in must_not {
                    if let MatchPlan::Expr(expr) | MatchPlan::Regex { expr, .. } =
                        Box::pin(self.match_expression(sub)).await?
                    {
                        if expression.is_empty() {
                            expression = format!("NOT ({expr})");
                        } else {
                            expression.push_str(&format!(" NOT ({expr})"));
                        }
                    }
                }
                if expression.is_empty() || expression.starts_with("NOT (") {
                    return Ok(MatchPlan::Unsatisfiable);
                }
                Ok(MatchPlan::Expr(expression))
            }
            FtsQuery::Range { .. } => Err(IndexError::QueryRejected {
                detail:
                    "range queries are metadata-only; use SearchOpts filters or a top-level scan"
                        .to_string(),
            }),
            FtsQuery::Regex { field, pattern } => self.regex_expression(field, pattern).await,
            FtsQuery::All => Ok(MatchPlan::Unconstrained),
        }
    }

    /// Expand a DFA-safe pattern over the term dictionary into a bounded OR.
    ///
    /// Guard chain (I16): length cap → anchor rule → DFA-only compile with
    /// construction budgets → bounded dictionary scan → expansion cap.
    /// A pattern matching no terms yields `Unsatisfiable`.
    async fn regex_expression(
        &self,
        field: &str,
        pattern: &str,
    ) -> Result<MatchPlan, IndexError> {
        if TEXT_COLUMNS.iter().all(|col| *col != field) {
            return Err(IndexError::QueryRejected {
                detail: format!("regex is only allowed against indexed text fields, not '{field}'"),
            });
        }
        let dfa = compile_dfa(pattern)?;
        let terms: Vec<String> = sqlx::query_scalar("SELECT term FROM vocab WHERE col = ?")
            .bind(field)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| IndexError::BuildFailed {
                stage: "search".to_string(),
                detail: err.to_string(),
            })?;
        let mut matched = Vec::new();
        let mut matched_terms = Vec::new();
        for (examined, term) in terms.iter().enumerate() {
            if examined >= MAX_VOCAB_SCAN {
                return Err(IndexError::QueryRejected {
                    detail: format!(
                        "regex scanned over {MAX_VOCAB_SCAN} terms; narrow the pattern"
                    ),
                });
            }
            if dfa.is_match(term) {
                matched.push(format!("{{ {field} }} : {}", quote(term)));
                matched_terms.push(term.clone());
                if matched.len() > MAX_REGEX_EXPANSION {
                    return Err(IndexError::QueryRejected {
                        detail: format!(
                            "regex expands to over {MAX_REGEX_EXPANSION} terms; anchor the pattern"
                        ),
                    });
                }
            }
        }
        if matched.is_empty() {
            return Ok(MatchPlan::Unsatisfiable);
        }
        Ok(MatchPlan::Regex {
            expr: matched.join(" OR "),
            terms: matched_terms,
            examined: terms.len() as u64,
        })
    }

    /// WHERE clause for metadata filters (UNINDEXED columns allow plain SQL).
    fn filter_clause(
        filters: &[crate::model::Filter],
    ) -> Result<(String, Vec<String>), IndexError> {
        use crate::model::Filter;
        let mut clauses = Vec::new();
        let mut args = Vec::new();
        for filter in filters {
            match filter {
                Filter::Surah(ids) => {
                    if ids.is_empty() {
                        clauses.push("1 = 0".to_string());
                    } else {
                        let list =
                            ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",");
                        clauses.push(format!("surah IN ({list})"));
                    }
                }
                Filter::JuzRange(lo, hi) => {
                    clauses.push(format!("juz BETWEEN {lo} AND {hi}"));
                }
                Filter::Page(pages) => {
                    if pages.is_empty() {
                        clauses.push("1 = 0".to_string());
                    } else {
                        let list =
                            pages.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",");
                        clauses.push(format!("page IN ({list})"));
                    }
                }
                Filter::RevelationPlace(place) => {
                    clauses.push("revelation = ?".to_string());
                    args.push(place.clone());
                }
                Filter::GlobalRange(lo, hi) => {
                    clauses.push(format!("global_index BETWEEN {lo} AND {hi}"));
                }
            }
        }
        Ok((clauses.join(" AND "), args))
    }
}

/// Quote one term for FTS5 (double internal quotes).
fn quote(term: &str) -> String {
    format!("\"{}\"", term.replace('"', "\"\""))
}

/// Quote one term inside a phrase (terms were pre-normalized; spaces inside a
/// single term would change phrase arity, so they are kept verbatim — the
/// pipeline never emits internal spaces except N01's single separator).
fn quote_phrase_term(term: &str) -> String {
    term.replace('"', "\"\"")
}

/// Compile a pattern through the single DFA engine ([`crate::regex`]).
fn compile_dfa(pattern: &str) -> Result<regex_automata::dfa::regex::Regex, IndexError> {
    crate::regex::compile_dfa(pattern)
}

/// Metadata column allowlist for range queries.
const META_COLUMNS: [&str; 4] = ["surah", "juz", "page", "global_index"];

/// Range predicate over a metadata column (validated allowlist, integers only).
fn range_clause(field: &str, lo: Option<i64>, hi: Option<i64>) -> Result<String, IndexError> {
    if META_COLUMNS.iter().all(|col| *col != field) {
        return Err(IndexError::QueryRejected {
            detail: format!("range queries support surah/juz/page/global_index, not '{field}'"),
        });
    }
    match (lo, hi) {
        (Some(lo), Some(hi)) => Ok(format!("{field} BETWEEN {lo} AND {hi}")),
        (Some(lo), None) => Ok(format!("{field} >= {lo}")),
        (None, Some(hi)) => Ok(format!("{field} <= {hi}")),
        (None, None) => Ok("1 = 1".to_string()),
    }
}

/// ORDER BY clause for a result order.
fn order_clause(order: ResultOrder) -> &'static str {
    match order {
        ResultOrder::CanonicalOrder => "ORDER BY surah, ayah",
        ResultOrder::Relevance => "ORDER BY rank",
    }
}

/// Compiled predicate: WHERE sql plus bindings (MATCH first, then filters).
struct Predicate {
    where_sql: String,
    match_expr: Option<String>,
    args: Vec<String>,
    /// True when the query is unsatisfiable (e.g. a term normalizing to
    /// empty): search returns zero hits without touching the engine.
    unsatisfiable: bool,
    /// Regex expansion provenance (top-level regex queries only).
    regex_terms: Vec<String>,
    /// Dictionary terms examined (top-level regex queries only).
    terms_examined: u64,
}

impl Fts5Index {
    /// Compile a query + filters into one predicate.
    async fn predicate(
        &self,
        query: &FtsQuery,
        filters: &[crate::model::Filter],
    ) -> Result<Predicate, IndexError> {
        // `FtsQuery::All` is the only query with no constraint at all; every
        // other query form that yields no MATCH expression is unsatisfiable.
        if matches!(query, FtsQuery::All) {
            let (filter_sql, args) = Self::filter_clause(filters)?;
            let where_sql =
                if filter_sql.is_empty() { String::new() } else { format!("WHERE {filter_sql}") };
            return Ok(Predicate {
                where_sql,
                match_expr: None,
                args,
                unsatisfiable: false,
                regex_terms: Vec::new(),
                terms_examined: 0,
            });
        }
        let mut clauses = Vec::new();
        let mut match_expr = None;
        let mut unsatisfiable = false;
        let mut regex_terms = Vec::new();
        let mut terms_examined = 0u64;
        if let FtsQuery::Range { field, lo, hi } = query {
            clauses.push(range_clause(field, *lo, *hi)?);
        } else {
            match self.match_expression(query).await? {
                MatchPlan::Expr(expr) => {
                    clauses.push("ayah_fts MATCH ?".to_string());
                    match_expr = Some(expr);
                }
                MatchPlan::Regex { expr, terms, examined } => {
                    clauses.push("ayah_fts MATCH ?".to_string());
                    match_expr = Some(expr);
                    regex_terms = terms;
                    terms_examined = examined;
                }
                MatchPlan::Unsatisfiable => unsatisfiable = true,
                MatchPlan::Unconstrained => {}
            }
        }
        let (filter_sql, args) = Self::filter_clause(filters)?;
        if !filter_sql.is_empty() {
            clauses.push(filter_sql);
        }
        let where_sql = if clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", clauses.join(" AND "))
        };
        Ok(Predicate {
            where_sql,
            match_expr,
            args,
            unsatisfiable,
            regex_terms,
            terms_examined,
        })
    }

    /// Exact count behind one predicate.
    async fn count_predicate(&self, predicate: &Predicate) -> Result<u64, IndexError> {
        let sql = format!("SELECT COUNT(*) FROM ayah_fts {}", predicate.where_sql);
        let mut query = sqlx::query_scalar::<_, i64>(&sql);
        if let Some(expr) = &predicate.match_expr {
            query = query.bind(expr);
        }
        for arg in &predicate.args {
            query = query.bind(arg);
        }
        let count: i64 = query.fetch_one(&self.pool).await.map_err(|err| {
            IndexError::BuildFailed { stage: "search".to_string(), detail: err.to_string() }
        })?;
        Ok(count.max(0) as u64)
    }

    /// Primary field for scoring/highlight addressing: first Term/Phrase/
    /// Regex field in the query, else the primary search field.
    fn primary_field(query: &FtsQuery) -> &str {
        match query {
            FtsQuery::Term { field, .. }
            | FtsQuery::Phrase { field, .. }
            | FtsQuery::Regex { field, .. } => field,
            FtsQuery::Boolean { must, should, .. } => must
                .iter()
                .chain(should.iter())
                .find_map(|sub| match sub {
                    FtsQuery::Term { field, .. }
                    | FtsQuery::Phrase { field, .. }
                    | FtsQuery::Regex { field, .. } => Some(field.as_str()),
                    _ => None,
                })
                .unwrap_or("text_bare"),
            FtsQuery::Range { .. } | FtsQuery::All => "text_bare",
        }
    }
}

#[async_trait::async_trait]
impl FullTextIndex for Fts5Index {
    fn backend(&self) -> FtsBackend {
        FtsBackend::Fts5
    }

    fn manifest(&self) -> IndexManifest {
        self.manifest.clone()
    }

    async fn create(&self, _schema: &FtsSchema) -> Result<(), IndexError> {
        // Tables exist from `stage`; recreate idempotently for reopened
        // instances (the schema is fixed in Phase 2).
        self.create_tables().await
    }

    async fn add_batch(&self, docs: Vec<FtsDoc>) -> Result<(), IndexError> {
        let mut tx = self.pool.begin().await.map_err(|err| IndexError::BuildFailed {
            stage: "add_batch".to_string(),
            detail: err.to_string(),
        })?;
        for doc in &docs {
            let normalized = self.normalize_doc(doc)?;
            let cell = |field: &str| normalized.get(field).cloned().unwrap_or_default();
            let int_meta =
                |key: &str| doc.metadata.get(key).and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
            sqlx::query(
                "INSERT INTO ayah_fts
                    (doc_id, text_exact, text_ws, text_marks, text_bare, text_hamza,
                     text_folded, text_affix, roots, lemmas, stems, pos_tags, patterns,
                     surah, ayah, global_index, juz, page, generation, revelation)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, '', '', '', '', '', ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&doc.id)
            .bind(cell("text_exact"))
            .bind(cell("text_ws"))
            .bind(cell("text_marks"))
            .bind(cell("text_bare"))
            .bind(cell("text_hamza"))
            .bind(cell("text_folded"))
            .bind(cell("text_affix"))
            .bind(doc.surah as i64)
            .bind(doc.ayah as i64)
            .bind(doc.global_index as i64)
            .bind(int_meta("juz"))
            .bind(int_meta("page"))
            .bind(doc.generation as i64)
            .bind(doc.metadata.get("revelation").cloned().unwrap_or_default())
            .execute(&mut *tx)
            .await
            .map_err(|err| IndexError::BuildFailed {
                stage: "add_batch".to_string(),
                detail: err.to_string(),
            })?;
        }
        tx.commit().await.map_err(|err| IndexError::BuildFailed {
            stage: "add_batch".to_string(),
            detail: err.to_string(),
        })?;
        Ok(())
    }

    async fn commit(&self) -> Result<CommitStamp, IndexError> {
        let doc_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ayah_fts")
            .fetch_one(&self.pool)
            .await
            .map_err(|err| IndexError::BuildFailed {
                stage: "commit".to_string(),
                detail: err.to_string(),
            })?;
        Ok(CommitStamp {
            generation: self.generation,
            doc_count: doc_count.max(0) as u64,
            content_hash: self.manifest.content_hash.clone(),
        })
    }

    async fn search(&self, query: &FtsQuery, opts: &SearchOpts) -> Result<FtsResults, IndexError> {
        let opts = opts.clone().normalized();
        let predicate = self.predicate(query, &opts.filters).await?;
        if predicate.unsatisfiable {
            return Ok(FtsResults {
                hits: Vec::new(),
                total_matches: 0,
                truncated: false,
                regex_terms: Vec::new(),
                terms_examined: 0,
            });
        }
        let total_matches = self.count_predicate(&predicate).await?;
        let order = order_clause(opts.order);
        let sql = format!(
            "SELECT doc_id, bm25(ayah_fts) AS rank FROM ayah_fts {} {order} LIMIT ? OFFSET ?",
            predicate.where_sql
        );
        let mut fetch = sqlx::query(&sql);
        if let Some(expr) = &predicate.match_expr {
            fetch = fetch.bind(expr);
        }
        for arg in &predicate.args {
            fetch = fetch.bind(arg);
        }
        let limit = opts.limit.min(1000) as i64;
        fetch = fetch.bind(limit).bind(opts.offset as i64);
        let rows = fetch.fetch_all(&self.pool).await.map_err(|err| IndexError::BuildFailed {
            stage: "search".to_string(),
            detail: err.to_string(),
        })?;
        let field = Self::primary_field(query).to_string();
        let mut hits = Vec::with_capacity(rows.len());
        for row in rows {
            use sqlx::Row as _;
            let rank: f64 = row.try_get("rank").unwrap_or(0.0);
            hits.push(FtsHit {
                doc_id: row.try_get("doc_id").map_err(|err| IndexError::BuildFailed {
                    stage: "search".to_string(),
                    detail: err.to_string(),
                })?,
                score: match opts.order {
                    ResultOrder::Relevance => Some(rank as f32),
                    ResultOrder::CanonicalOrder => None,
                },
                matched_field: field.clone(),
            });
        }
        // NOTE (T49): per-hit highlight markers ride on `highlight()` over
        // the matched column; there is no carrier on `FtsHit` yet, so M3
        // wires snippet retrieval into `SearchHit` there.
        let truncated = u64::try_from(hits.len()).unwrap_or(u64::MAX) >= u64::from(opts.limit)
            && total_matches > u64::from(opts.limit);
        Ok(FtsResults {
            hits,
            total_matches,
            truncated,
            regex_terms: predicate.regex_terms,
            terms_examined: predicate.terms_examined,
        })
    }

    async fn count(&self, query: &FtsQuery) -> Result<u64, IndexError> {
        let predicate = self.predicate(query, &[]).await?;
        if predicate.unsatisfiable {
            return Ok(0);
        }
        self.count_predicate(&predicate).await
    }

    async fn delete_by_generation(&self, generation: u64) -> Result<u64, IndexError> {
        let dir = Self::gen_dir(&self.root, generation);
        if !dir.exists() {
            return Ok(0);
        }
        // Count inside the target generation before removing it.
        let path = Self::db_path(&self.root, generation);
        let pool = Self::connect(&path, false).await?;
        let removed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ayah_fts")
            .fetch_one(&pool)
            .await
            .map_err(|err| IndexError::BuildFailed {
                stage: "delete".to_string(),
                detail: err.to_string(),
            })?;
        pool.close().await;
        std::fs::remove_dir_all(&dir).map_err(|err| IndexError::BuildFailed {
            stage: "delete".to_string(),
            detail: err.to_string(),
        })?;
        Ok(removed.max(0) as u64)
    }

    async fn stats(&self) -> Result<FtsStats, IndexError> {
        let doc_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ayah_fts")
            .fetch_one(&self.pool)
            .await
            .map_err(|err| IndexError::BuildFailed {
                stage: "stats".to_string(),
                detail: err.to_string(),
            })?;
        Ok(FtsStats {
            backend: FtsBackend::Fts5,
            doc_count: doc_count.max(0) as u64,
            generation: self.generation,
        })
    }

    async fn verify(&self) -> Result<FtsIntegrityReport, IndexError> {
        let doc_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ayah_fts")
            .fetch_one(&self.pool)
            .await
            .map_err(|err| IndexError::BuildFailed {
                stage: "verify".to_string(),
                detail: err.to_string(),
            })?;
        let doc_count = doc_count.max(0) as u64;
        let mut findings = Vec::new();
        if doc_count != self.manifest.doc_count {
            findings.push(format!(
                "doc_count {doc_count} disagrees with manifest {}",
                self.manifest.doc_count
            ));
        }
        Ok(FtsIntegrityReport { ok: findings.is_empty(), doc_count, findings })
    }
}
