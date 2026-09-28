//! High-level Quran command implementations for the CLI (D1.10).
//!
//! # application::quran_cli
//!
//! Every command runs here against a database path so `cli` never touches
//! storage directly. Outputs come in two shapes: human text without volatile
//! ids (snapshot determinism) and full `--json` structures. Exit codes mirror
//! `cli::exit_code` (`OK` 0 · `GENERIC` 1 · `USAGE` 2 · `VALIDATION` 3 ·
//! `POLICY` 4 · `NOT_FOUND` 5 · `CONFLICT` 6 · `CANCELLED` 7 · `INTERNAL` 70).

use std::sync::Arc;

use quran_core::{AyahOptions, ContextBoundary, ContextSpec, EditionSelector, QuranRef};
use quran_corpus::error::QuranDiagnostic as _;
use storage::Database as _;
use storage::error::StorageError;
use storage_sqlite::SqliteDatabase;

use crate::audit_bridge::AuditedMutation;

/// Exit codes (mirrors `cli::exit_code` without depending on it).
pub mod exit {
    /// Ok.
    pub const OK: i32 = 0;
    /// Generic failure.
    pub const GENERIC: i32 = 1;
    /// Usage error.
    pub const USAGE: i32 = 2;
    /// Validation failed.
    pub const VALIDATION: i32 = 3;
    /// Denied by policy.
    pub const POLICY: i32 = 4;
    /// Not found.
    pub const NOT_FOUND: i32 = 5;
    /// Conflict/state.
    pub const CONFLICT: i32 = 6;
    /// Cancelled.
    pub const CANCELLED: i32 = 7;
    /// Internal error.
    pub const INTERNAL: i32 = 70;
}

/// Command output in both shapes.
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// Process exit code.
    pub exit: i32,
    /// Human text.
    pub human: String,
    /// Machine JSON.
    pub json: serde_json::Value,
}

impl CommandOutput {
    /// Success with both shapes.
    pub fn ok(human: String, json: serde_json::Value) -> Self {
        Self { exit: exit::OK, human, json }
    }

    /// Failure with a message.
    pub fn err(exit: i32, message: String) -> Self {
        Self {
            exit,
            human: format!("error: {message}"),
            json: serde_json::json!({"error": {"message": message}}),
        }
    }
}

/// Local operator identity (single-user interim; Phase 11 owns identity).
pub const LOCAL_PRINCIPAL: &str = "00000000-0000-0000-0000-000000000000";

/// Existing-database guard for ordinary commands (D-05, T-01-MIG).
///
/// Returns the typed [`StorageError::MigrationRequired`] diagnostic when no
/// database file exists yet, instead of implicitly creating one through the
/// creating constructor. Database creation and schema mutation stay confined
/// to the explicit application migration function reached by `qai db migrate`;
/// an existing file still opens through the read/write pool because mutation
/// commands (import, activate, …) need writes.
async fn open_db(db_path: &str) -> Result<SqliteDatabase, StorageError> {
    if std::fs::symlink_metadata(db_path).is_err() {
        return Err(StorageError::MigrationRequired { at_schema: 0, required: 0 });
    }
    SqliteDatabase::new(db_path, 4, true).await
}

/// Map an `open_db` failure to the operator surface.
///
/// A missing database is a validation failure (exit 3, matching doctor's
/// missing-database treatment) carrying the typed remedy and the runnable
/// `qai db migrate` next command; every other open failure stays internal.
fn map_open_error(error: StorageError) -> (i32, String) {
    match &error {
        StorageError::MigrationRequired { .. } => (
            exit::VALIDATION,
            format!(
                "{error}; remedy: {}; next: qai db migrate",
                error.remedy().unwrap_or("run `qai db migrate`")
            ),
        ),
        _ => (exit::INTERNAL, error.to_string()),
    }
}

/// Render an `open_db` failure as a [`CommandOutput`].
fn open_error_output(error: StorageError) -> CommandOutput {
    let (exit, message) = map_open_error(error);
    CommandOutput::err(exit, message)
}

fn map_reader_error(error: crate::quran_reader::ReaderError) -> (i32, String) {
    use crate::quran_reader::ReaderError as E;
    match &error {
        E::InvalidReference(_) => (exit::USAGE, error.to_string()),
        E::EditionNotFound(_)
        | E::AyahNotFound(_)
        | E::TranslationNotFound(_)
        | E::DivisionNotFound(_)
        | E::PrimaryNotDeclared => (exit::NOT_FOUND, error.to_string()),
        E::PrimaryAmbiguous => (exit::CONFLICT, error.to_string()),
        E::Storage(_) => (exit::INTERNAL, error.to_string()),
    }
}

fn map_activation_error(error: super::quran::ActivationError) -> (i32, String) {
    use super::quran::ActivationError as E;
    match &error {
        E::ApprovalMissing { .. }
        | E::ApprovalNotGranted { .. }
        | E::ApprovalSubjectMismatch { .. }
        | E::Provenance(_) => (exit::POLICY, error.to_string()),
        E::EmptyReviewer => (exit::USAGE, error.to_string()),
        E::NotStaged { .. } | E::AlreadyActive { .. } => (exit::CONFLICT, error.to_string()),
        E::EditionNotFound { .. } => (exit::NOT_FOUND, error.to_string()),
        E::Storage(_) | E::Audit(_) => (exit::INTERNAL, error.to_string()),
    }
}

fn map_corpus_error(error: quran_corpus::CorpusError) -> (i32, String) {
    let code = error.code().to_string();
    if code == "QAI-QUR-0204" {
        (exit::VALIDATION, error.to_string())
    } else if code == "QAI-QUR-0205" {
        (exit::CANCELLED, error.to_string())
    } else {
        (exit::INTERNAL, error.to_string())
    }
}

/// Map a `citations` hard-failure error to a CLI exit code (D-15, criterion 5).
///
/// `Mismatch` is a validation failure (3), a missing location/edition is
/// not-found (5), access denial is a policy failure (4), and an unparseable
/// reference is usage (2). Codes are delegated to
/// [`citations::CitationError::code`].
fn map_citation_error(error: &citations::CitationError) -> (i32, String) {
    use citations::CitationError as E;
    match error {
        E::QuotationMismatch { .. } => (exit::VALIDATION, error.to_string()),
        E::LocationNotFound | E::EditionNotFound => (exit::NOT_FOUND, error.to_string()),
        E::AccessDenied => (exit::POLICY, error.to_string()),
        E::InvalidReference { .. } => (exit::USAGE, error.to_string()),
        E::Backend { .. } => (exit::INTERNAL, error.to_string()),
    }
}

fn reader(db: &Arc<SqliteDatabase>) -> crate::quran_reader::QuranReaderService {
    crate::quran_reader::QuranReaderService::new(db.clone())
}

fn parse_selector(raw: &str) -> Result<EditionSelector, CommandOutput> {
    if raw.is_empty() {
        return Ok(EditionSelector::Active);
    }
    // Reserved selector keyword (D-07): the explicitly flagged primary edition.
    if raw == "primary" {
        return Ok(EditionSelector::Primary);
    }
    match raw.split_once('@') {
        Some((slug, version)) => match version.parse() {
            Ok(version) => Ok(EditionSelector::Pinned { slug: slug.to_string(), version }),
            Err(_) => Err(CommandOutput::err(exit::USAGE, format!("bad edition `{raw}`"))),
        },
        None => Ok(EditionSelector::Slug(raw.to_string())),
    }
}

/// Open a reader over a database file (serve/CLI entry point helper).
pub async fn open_reader(
    db_path: &str,
) -> Result<crate::quran_reader::QuranReaderService, StorageError> {
    use crate::quran_reader::QuranReaderService;
    let db = std::sync::Arc::new(open_db(db_path).await?);
    Ok(QuranReaderService::new(db))
}

/// `quran get`.
pub async fn cmd_get(
    db_path: &str,
    reference: &str,
    translations: Option<&str>,
    tokens: bool,
    glosses: bool,
) -> CommandOutput {
    use crate::quran_reader::QuranReader;
    let db = match open_db(db_path).await {
        Ok(db) => Arc::new(db),
        Err(err) => return open_error_output(err),
    };
    let parsed = match quran_core::parse(reference) {
        Ok(parsed) => parsed,
        Err(err) => return CommandOutput::err(exit::USAGE, err.to_string()),
    };
    let options = AyahOptions {
        translations: translations
            .unwrap_or("")
            .split(',')
            .filter(|part| !part.trim().is_empty())
            .map(|part| part.trim().to_string())
            .collect(),
        glosses,
        tokens,
    };
    let view = match reader(&db).get_ayah(&parsed, &options).await {
        Ok(view) => view,
        Err(err) => {
            let (exit, message) = map_reader_error(err);
            return CommandOutput::err(exit, message);
        }
    };
    let mut human = format!("{}\n— {}", view.canonical.arabic_text(), view.canonical.reference());
    // Criterion 2 on the operator surface: the pinned edition reference (above)
    // together with the STORED per-ayah canonical hash carried on the served
    // view. The hash is read from `AyahView`, never recomputed from the printed
    // text (QC-02), so a lookup can be verified against the stored bytes.
    human.push_str(&format!("\n  text_hash: sha256:{}", view.canonical.text_hash().hex));
    for translation in &view.translations {
        human.push_str(&format!("\n[{}] {}", translation.translator(), translation.text()));
    }
    if let Some(glosses) = &view.word_glosses {
        for gloss in glosses {
            // Position is not carried in `AttributedGloss`; the dataset id keeps
            // each gloss attributed while the JSON carries the full rows.
            human.push_str(&format!("\n[{}] {}", gloss.dataset, gloss.gloss));
        }
    }
    CommandOutput::ok(human, serde_json::to_value(&view).unwrap_or_default())
}

/// `quran context`.
pub async fn cmd_context(
    db_path: &str,
    reference: &str,
    before: u16,
    after: u16,
    boundary: &str,
) -> CommandOutput {
    use crate::quran_reader::QuranReader;
    let boundary = match boundary {
        "surah" => ContextBoundary::Surah,
        "juz" => ContextBoundary::Juz,
        "ruku" => ContextBoundary::Ruku,
        "page" => ContextBoundary::Page,
        "none" => ContextBoundary::None,
        _ => return CommandOutput::err(exit::USAGE, format!("unknown boundary `{boundary}`")),
    };
    let db = match open_db(db_path).await {
        Ok(db) => Arc::new(db),
        Err(err) => return open_error_output(err),
    };
    let parsed = match quran_core::parse(reference) {
        Ok(parsed) => parsed,
        Err(err) => return CommandOutput::err(exit::USAGE, err.to_string()),
    };
    let spec = ContextSpec { before, after, boundary, include_surah_header: true, max_ayahs: 100 };
    let view = match reader(&db).get_context(&parsed, &spec).await {
        Ok(view) => view,
        Err(err) => {
            let (exit, message) = map_reader_error(err);
            return CommandOutput::err(exit, message);
        }
    };
    let mut human = String::new();
    for item in view.before.iter().chain(std::iter::once(&view.focal)).chain(view.after.iter()) {
        human.push_str(&format!("{}\n", item.canonical.arabic_text()));
    }
    human.push_str(&format!("— {}", view.canonical_reference));
    CommandOutput::ok(human, serde_json::to_value(&view).unwrap_or_default())
}

/// `quran surah`.
pub async fn cmd_surah(db_path: &str, number: u16, metadata: bool) -> CommandOutput {
    use crate::quran_reader::QuranReader;
    let db = match open_db(db_path).await {
        Ok(db) => Arc::new(db),
        Err(err) => return open_error_output(err),
    };
    let surah_number = match quran_core::SurahNumber::new(number) {
        Ok(number) => number,
        Err(err) => return CommandOutput::err(exit::USAGE, err.to_string()),
    };
    let reference = QuranRef::Surah { edition: EditionSelector::Active, surah: surah_number };
    let views = match reader(&db).get_ayahs(&reference, &AyahOptions::default()).await {
        Ok(views) => views,
        Err(err) => {
            let (exit, message) = map_reader_error(err);
            return CommandOutput::err(exit, message);
        }
    };
    if metadata {
        let first = match views.first() {
            Some(view) => view,
            None => return CommandOutput::err(exit::NOT_FOUND, "empty surah".to_string()),
        };
        let human =
            format!("{}\n{}", first.canonical.surah_name_arabic(), first.canonical.reference());
        return CommandOutput::ok(human, serde_json::json!({"ayahs": views.len()}));
    }
    let mut human = String::new();
    for view in &views {
        human.push_str(&format!("{}\n", view.canonical.arabic_text()));
    }
    CommandOutput::ok(human, serde_json::to_value(&views).unwrap_or_default())
}

/// `quran division`.
pub async fn cmd_division(db_path: &str, kind: &str, number: u32) -> CommandOutput {
    use crate::quran_reader::QuranReader;
    let division = match kind {
        "juz" => quran_core::DivisionKind::Juz,
        "hizb" => quran_core::DivisionKind::Hizb,
        "rub" => quran_core::DivisionKind::Rub,
        "manzil" => quran_core::DivisionKind::Manzil,
        "page" => quran_core::DivisionKind::Page,
        "ruku" => quran_core::DivisionKind::Ruku,
        "sajdah" => quran_core::DivisionKind::Sajdah,
        _ => return CommandOutput::err(exit::USAGE, format!("unknown division `{kind}`")),
    };
    let db = match open_db(db_path).await {
        Ok(db) => Arc::new(db),
        Err(err) => return open_error_output(err),
    };
    let reference = QuranRef::Division { edition: EditionSelector::Active, kind: division, number };
    let views = match reader(&db).get_ayahs(&reference, &AyahOptions::default()).await {
        Ok(views) => views,
        Err(err) => {
            let (exit, message) = map_reader_error(err);
            return CommandOutput::err(exit, message);
        }
    };
    let mut human = format!("{kind} {number} ({} ayahs)\n", views.len());
    for view in &views {
        human.push_str(&format!(
            "{} {}\n",
            view.canonical.reference(),
            view.canonical.arabic_text()
        ));
    }
    CommandOutput::ok(human, serde_json::to_value(&views).unwrap_or_default())
}

/// `quran resolve`.
pub async fn cmd_resolve(db_path: &str, reference: &str) -> CommandOutput {
    use crate::quran_reader::QuranReader;
    let db = match open_db(db_path).await {
        Ok(db) => Arc::new(db),
        Err(err) => return open_error_output(err),
    };
    match reader(&db).resolve(reference).await {
        Ok(resolved) => {
            let edition = reader(&db)
                .get_edition(&quran_core::EditionSelector::Active)
                .await
                .map(|edition| format!("{}@{}", edition.slug, edition.version))
                .unwrap_or_default();
            CommandOutput::ok(
                format!("{}\nactive edition: {edition}\n", resolved.canonical),
                serde_json::json!({
                    "canonical": resolved.canonical,
                    "reference": quran_core::serialize(&resolved.reference),
                }),
            )
        }
        Err(err) => {
            let (exit, message) = map_reader_error(err);
            CommandOutput::err(exit, message)
        }
    }
}

/// `quran verify-quotation` — verify an externally supplied quotation against
/// the canonical source of truth (read-only; D-15, criterion 5).
///
/// The pinned `slug@version` governs the edition; it is never inferred from the
/// supplied text. On success the verdict and the RESOLVED canonical text hash
/// are printed (the hash is read from the canonical row, never computed from
/// the supplied text). A mismatch exits [`exit::VALIDATION`] (3) with a typed
/// `QAI-QUR-*` code; a missing location/edition exits [`exit::NOT_FOUND`] (5).
pub async fn cmd_verify_quotation(
    db_path: &str,
    edition: &str,
    surah: u16,
    ayah: u32,
    text: &str,
) -> CommandOutput {
    let (slug, version) = match edition.split_once('@') {
        Some((slug, version)) if !slug.is_empty() && !version.is_empty() => (slug, version),
        _ => {
            return CommandOutput::err(
                exit::USAGE,
                format!("bad edition `{edition}` (expected slug@version)"),
            );
        }
    };
    let db = match open_db(db_path).await {
        Ok(db) => Arc::new(db),
        Err(err) => return open_error_output(err),
    };
    let reader = Arc::new(reader(&db));
    match crate::quran_tools::verify_canonical_quotation(&reader, slug, version, surah, ayah, text)
        .await
    {
        Ok((verdict, text_hash)) => CommandOutput::ok(
            format!(
                "verdict: {}\n  text_hash: {}\n  reference: quran:{slug}@{version}:{surah}:{ayah}",
                verdict.label(),
                text_hash
            ),
            serde_json::json!({
                "edition": format!("{slug}@{version}"),
                "surah": surah,
                "ayah": ayah,
                "verdict": verdict.label(),
                "text_hash": text_hash,
            }),
        ),
        Err(err) => {
            let (exit, message) = map_citation_error(&err);
            CommandOutput::err(exit, message)
        }
    }
}

/// `quran edition list`.
pub async fn cmd_edition_list(db_path: &str) -> CommandOutput {
    use crate::quran_reader::QuranReader;
    let db = match open_db(db_path).await {
        Ok(db) => Arc::new(db),
        Err(err) => return open_error_output(err),
    };
    let editions =
        match reader(&db).list_editions(crate::quran_reader::EditionFilter::default()).await {
            Ok(editions) => editions,
            Err(err) => {
                let (exit, message) = map_reader_error(err);
                return CommandOutput::err(exit, message);
            }
        };
    let mut human = String::new();
    for edition in &editions {
        human.push_str(&format!(
            "{}@{} [{}]\n",
            edition.slug,
            edition.version,
            edition_status(edition)
        ));
    }
    CommandOutput::ok(human, serde_json::to_value(&editions).unwrap_or_default())
}

fn edition_status(edition: &quran_core::QuranEdition) -> &'static str {
    match edition.status {
        quran_core::EditionStatus::Staged => "staged",
        quran_core::EditionStatus::Approved => "approved",
        quran_core::EditionStatus::Active => "active",
        quran_core::EditionStatus::Deprecated => "deprecated",
        quran_core::EditionStatus::Quarantined => "quarantined",
    }
}

/// `quran edition show`.
pub async fn cmd_edition_show(
    db_path: &str,
    edition: &str,
    statistics: bool,
    hashes: bool,
) -> CommandOutput {
    use crate::quran_reader::QuranReader;
    let selector = match parse_selector(edition) {
        Ok(selector) => selector,
        Err(output) => return output,
    };
    let db = match open_db(db_path).await {
        Ok(db) => Arc::new(db),
        Err(err) => return open_error_output(err),
    };
    let edition = match reader(&db).get_edition(&selector).await {
        Ok(edition) => edition,
        Err(err) => {
            let (exit, message) = map_reader_error(err);
            return CommandOutput::err(exit, message);
        }
    };
    let mut human = format!(
        "{}@{} — {}\nstatus: {}\n",
        edition.slug,
        edition.version,
        edition.name,
        edition_status(&edition)
    );
    // Declared upstream identity and primary/default designation, printed only
    // when present so the line stays deterministic for undeclared editions.
    if let Some(upstream) = &edition.upstream_edition_slug {
        human.push_str(&format!("upstream: {upstream}\n"));
    }
    if let Some(qai_id) = &edition.qai_edition_id {
        human.push_str(&format!("qai-id: {qai_id}\n"));
    }
    if edition.is_primary {
        human.push_str("primary: true\n");
    }
    // Declared license status is read verbatim from the canonical row, not
    // inferred: the typed reader maps unmodelled statuses (e.g. a manifest's
    // `verified`) to `Unknown`, so the raw declared string is surfaced here and
    // never invented (T-02-36, OD-01). Printed only when a license was
    // declared, so undeclared editions stay byte-identical.
    let declared_license_status = {
        let mut uow = match db.write().await {
            Ok(uow) => uow,
            Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
        };
        // A degraded database must error, never render a confidently
        // incomplete record: a fetch failure is propagated instead of
        // silently omitting the `license:` line (WR-08). The write handle is
        // opened only because the typed edition reader lives on the unit of
        // work (`ReadTx` exposes raw SQL only, which this layer must not
        // interpolate); it is always rolled back, never committed — the same
        // read-only discipline as every other read in this module.
        let row = match uow
            .quran()
            .get_edition_by_slug_version(&edition.slug, &edition.version.to_string())
            .await
        {
            Ok(row) => row,
            Err(err) => {
                let _ = uow.rollback().await;
                return CommandOutput::err(exit::INTERNAL, err.to_string());
            }
        };
        let _ = uow.rollback().await;
        row.and_then(|row| serde_json::from_str::<serde_json::Value>(&row.license_json).ok())
            .and_then(|value| {
                value.get("status").and_then(|status| status.as_str()).map(str::to_string)
            })
    };
    let declared_license = declared_license_status
        .as_deref()
        .filter(|status| !status.is_empty() && *status != "Unknown");
    if let Some(status) = declared_license {
        human.push_str(&format!("license: {status}\n"));
    }
    if statistics {
        human.push_str(&format!(
            "surahs={} ayahs={} tokens={}\n",
            edition.statistics.surah_count,
            edition.statistics.ayah_count,
            edition.statistics.token_count
        ));
    }
    if hashes {
        human.push_str(&format!(
            "text={}\nstructure={}\norder={}\n",
            edition.text_hash.hex, edition.structure_hash.hex, edition.token_order_hash.hex
        ));
    }
    let mut json = serde_json::to_value(&edition).unwrap_or_default();
    if let (Some(object), Some(status)) = (json.as_object_mut(), declared_license) {
        object.insert(
            "declared_license_status".to_string(),
            serde_json::Value::String(status.to_string()),
        );
    }
    CommandOutput::ok(human, json)
}

/// `quran edition verify`: record editorial verification under approval.
///
/// The reviewer name and method are operator-supplied (OD-02); this command
/// records them, never invents them. Empty values fail closed with usage.
pub async fn cmd_edition_verify(
    db_path: &str,
    edition: &str,
    reviewer: &str,
    method: &str,
) -> CommandOutput {
    let (slug, version) = match edition.split_once('@') {
        Some((slug, version)) if !slug.is_empty() && !version.is_empty() => (slug, version),
        _ => {
            return CommandOutput::err(
                exit::USAGE,
                format!("edition must be `slug@version`, got `{edition}`"),
            );
        }
    };
    if reviewer.trim().is_empty() || method.trim().is_empty() {
        return CommandOutput::err(
            exit::USAGE,
            "reviewer and method must both be non-empty (OD-02)".to_string(),
        );
    }
    let (db, at, approval_id) = match approval_flow(db_path).await {
        Ok(flow) => flow,
        Err(output) => return output,
    };
    let subject = format!("quran-edition:{slug}@{version}");
    // Operator-recorded under a human approval (single-user interim, like
    // activation): the reviewer name lands in `verified_by` and the audit
    // payload, never in a principal FK column. The exit ritual (P1-T60) must
    // confirm the reviewer is real, qualified, and not the implementer.
    if let Err(err) = super::quran::record_approval(
        &*db,
        &approval_id,
        &subject,
        LOCAL_PRINCIPAL,
        LOCAL_PRINCIPAL,
        "{}",
        &at,
    )
    .await
    {
        return CommandOutput::err(exit::INTERNAL, err.to_string());
    }
    let principal: domain::PrincipalId = match LOCAL_PRINCIPAL.parse() {
        Ok(principal) => principal,
        Err(_) => return CommandOutput::err(exit::INTERNAL, "bad local principal".to_string()),
    };
    let at_ts = match at.parse::<domain::Timestamp>() {
        Ok(at) => at,
        Err(_) => return CommandOutput::err(exit::INTERNAL, "bad timestamp".to_string()),
    };
    match super::quran::record_edition_verification(
        &*db,
        slug,
        version,
        &super::quran::EditionVerification { reviewer, method },
        &principal,
        &approval_id,
        &at_ts,
    )
    .await
    {
        Ok(()) => CommandOutput::ok(
            format!("verified {slug}@{version} by {reviewer} ({method})\n"),
            serde_json::json!({
                "slug": slug, "version": version,
                "verified_by": reviewer, "verification_method": method,
            }),
        ),
        Err(err) => {
            let (exit, message) = map_activation_error(err);
            CommandOutput::err(exit, message)
        }
    }
}
/// `quran edition active`.
pub async fn cmd_edition_active(db_path: &str) -> CommandOutput {
    use crate::quran_reader::QuranReader;
    let db = match open_db(db_path).await {
        Ok(db) => Arc::new(db),
        Err(err) => return open_error_output(err),
    };
    match reader(&db).get_edition(&EditionSelector::Active).await {
        Ok(edition) => CommandOutput::ok(
            format!("{}@{}\n", edition.slug, edition.version),
            serde_json::json!({"slug": edition.slug, "version": edition.version.to_string()}),
        ),
        Err(err) => {
            let (exit, message) = map_reader_error(err);
            CommandOutput::err(exit, message)
        }
    }
}

/// Read a manifest file or fail with usage.
fn read_manifest(path: &str) -> Result<String, CommandOutput> {
    std::fs::read_to_string(path).map_err(|err| {
        CommandOutput::err(exit::USAGE, format!("cannot read manifest `{path}`: {err}"))
    })
}

/// Ensure the catalog source + version rows the importer FKs into.
///
/// An audited mutation (D-09, D-10): the source/version rows and the
/// `SourceImported` audit event are staged in the same `UnitOfWork` and
/// become visible together on commit. No provenance row or projection outbox
/// event is required here — the dataset provenance and validation record
/// land in the import pipeline's own staging transaction, and no consumer
/// projects the catalog rows.
async fn ensure_source_version(
    db: &Arc<SqliteDatabase>,
    slug: &str,
    version: &str,
    at: &str,
) -> Result<String, CommandOutput> {
    ensure_principal_or_err(db, at).await?;
    // The source row keeps a stable human id; every import mints a FRESH version
    // row id (a bare UUID): each import is a new catalog version, and the reader
    // maps version ids back to typed UUIDs.
    let source_id = format!("src-{slug}");
    let version_id = uuid::Uuid::new_v4().to_string();
    let mut uow = db
        .write()
        .await
        .map_err(|err: StorageError| CommandOutput::err(exit::INTERNAL, err.to_string()))?;
    if uow
        .sources()
        .get(&source_id)
        .await
        .map_err(|err: StorageError| CommandOutput::err(exit::INTERNAL, err.to_string()))?
        .is_none()
    {
        uow.sources()
            .insert_source(storage::repository::SourceRow {
                id: source_id.clone(),
                title: slug.to_string(),
                content_type: "quran_edition".to_string(),
                language: Some("ar".to_string()),
                created_at: at.to_string(),
            })
            .await
            .map_err(|err: StorageError| CommandOutput::err(exit::INTERNAL, err.to_string()))?;
    }
    // Every import mints a fresh version row: each import is a new catalog version.
    uow.sources()
        .insert_version(storage::repository::SourceVersionRow {
            id: version_id.clone(),
            source_id: source_id.clone(),
            version: version.to_string(),
            state: "Staged".to_string(),
            trust_level: "ImportedUnverified".to_string(),
            license_status: "Unknown".to_string(),
            content_hash: None,
            manifest_blob_id: None,
        })
        .await
        .map_err(|err: StorageError| CommandOutput::err(exit::INTERNAL, err.to_string()))?;
    let principal: domain::PrincipalId = LOCAL_PRINCIPAL
        .parse()
        .map_err(|_| CommandOutput::err(exit::INTERNAL, "bad local principal".to_string()))?;
    let mut mutation = AuditedMutation::new(
        audit::Actor::Principal { principal_id: principal },
        audit::AuditAction::SourceImported,
        domain::SubjectRef(format!("quran-source:{source_id}")),
    );
    mutation.after = Some(serde_json::json!({
        "source_id": source_id,
        "source_version_id": version_id,
        "version": version,
    }));
    mutation
        .stage(&mut *uow)
        .await
        .map_err(|err| CommandOutput::err(exit::INTERNAL, err.to_string()))?;
    uow.commit()
        .await
        .map_err(|err: StorageError| CommandOutput::err(exit::INTERNAL, err.to_string()))?;
    Ok(version_id)
}

/// `quran import`.
pub async fn cmd_import(
    db_path: &str,
    manifest: &str,
    adapter: &str,
    dry_run: bool,
    reference: Option<&str>,
) -> CommandOutput {
    let text = match read_manifest(manifest) {
        Ok(text) => text,
        Err(output) => return output,
    };
    if adapter != "json" {
        return CommandOutput::err(exit::USAGE, format!("unknown adapter `{adapter}`"));
    }
    // Pre-validate before spending a job (authoritative gate stays in the job).
    match super::quran::dry_run_validate(&text) {
        Ok(report) if report.has_fatal() => {
            return CommandOutput::err(
                exit::VALIDATION,
                format!("manifest has {} fatal findings", report.fatal_count),
            );
        }
        Err(err) => {
            let (exit, message) = map_corpus_error(err);
            return CommandOutput::err(exit, message);
        }
        _ => {}
    }
    if dry_run {
        return match super::quran::dry_run_validate(&text) {
            Ok(report) => CommandOutput::ok(
                format!(
                    "dry run: {} fatal, {} errors, {} warnings\n",
                    report.fatal_count, report.error_count, report.warning_count
                ),
                serde_json::to_value(&report).unwrap_or_default(),
            ),
            Err(err) => {
                let (exit, message) = map_corpus_error(err);
                CommandOutput::err(exit, message)
            }
        };
    }
    let db = match open_db(db_path).await {
        Ok(db) => Arc::new(db),
        Err(err) => return open_error_output(err),
    };
    let at = now_rfc3339();
    if let Err(output) = ensure_principal_or_err(&db, &at).await {
        return output;
    }
    // Slug/version come from the manifest itself.
    let doc: quran_corpus::EditionSource = match serde_json::from_str(&text) {
        Ok(doc) => doc,
        Err(err) => return CommandOutput::err(exit::VALIDATION, format!("bad manifest: {err}")),
    };
    let at = now_rfc3339();
    let source_version_id =
        match ensure_source_version(&db, &doc.edition.slug, &doc.edition.version.to_string(), &at)
            .await
        {
            Ok(id) => id,
            Err(output) => return output,
        };
    // License identity comes from the manifest's declared license, verbatim.
    // A repository being open source never implies its text is
    // redistributable (A3); an undeclared license stays explicit `Unknown`
    // and is recorded as owner gate OD-01. Nothing is inferred from the
    // publisher or repository, and no redistribution permission is invented.
    let (license_status, license_json) = match &doc.edition.license {
        Some(license) => (
            license.status.clone(),
            serde_json::json!({
                "status": license.status.clone(),
                "expression": license.expression.clone(),
                "source_url": license.source_url.clone(),
                "spdx_id": license.expression.clone(),
                "attribution_required": false,
                "redistribution_allowed": false,
                "export_allowed": false,
                "notes": "declared at import; preserved verbatim (OD-01)",
            })
            .to_string(),
        ),
        None => (
            "Unknown".to_string(),
            serde_json::json!({
                "status": "Unknown",
                "attribution_required": false,
                "redistribution_allowed": false,
                "export_allowed": false,
                "notes": "no license declared in the manifest; owner gate OD-01",
            })
            .to_string(),
        ),
    };
    // The independent reference corpus is operator-supplied (D-09): its bytes
    // travel on the import payload and are compared byte-exact, fail-closed.
    // Identity/license/pins come verbatim from the document; nothing is
    // inferred (OD-03 stays an open owner gate).
    let reference_text = match reference {
        Some(path) => match read_manifest(path) {
            Ok(text) => Some(text),
            Err(output) => return output,
        },
        None => None,
    };
    let input = quran_corpus::import::ImportInput {
        run_id: uuid::Uuid::new_v4().to_string(),
        job_id: None,
        source_version_id,
        adapter: adapter.to_string(),
        manifest_text: text,
        declared_manifest_hash: None,
        invoked_by: LOCAL_PRINCIPAL.to_string(),
        license_status,
        license_json,
        created_at: at,
        reference_manifest_text: reference_text,
    };
    // Enqueue-only boundary (D-13): the one-shot command validates, persists
    // the catalog rows above and one queued job row, then reports the job id.
    // No worker is constructed or drained here; `qai serve` owns execution
    // and the edition stays unstaged until the host processes the job.
    match super::quran::enqueue_import_job(&db, input).await {
        Ok(enqueued) => {
            // OD-01 B-track: synthetic pipeline-exercise data is never
            // canonical — label it on the human surface every time.
            let mut human = format!(
                "queued {}@{} import as job {} (state: Queued)\ninspect with `{}`; `qai serve` processes queued work\n",
                doc.edition.slug, doc.edition.version, enqueued.job_id, enqueued.inspect_command,
            );
            if doc.edition.synthetic {
                human.push_str("note: synthetic test data — non-canonical (ADR-0101 fallback)\n");
            }
            CommandOutput::ok(
                human,
                serde_json::json!({
                    "job_id": enqueued.job_id,
                    "kind": enqueued.kind,
                    "state": enqueued.state,
                    "max_attempts": enqueued.max_attempts,
                    "inspect": enqueued.inspect_command,
                    "slug": doc.edition.slug,
                    "version": doc.edition.version.to_string(),
                }),
            )
        }
        // The job never ran, so no validation report exists to consult: an
        // enqueue failure is storage/queue-level and internal.
        Err(err) => CommandOutput::err(exit::INTERNAL, err.to_string()),
    }
}

/// `quran validate`.
pub async fn cmd_validate(db_path: &str, target: &str, report_path: Option<&str>) -> CommandOutput {
    let report = if std::path::Path::new(target).exists() {
        let text = match read_manifest(target) {
            Ok(text) => text,
            Err(output) => return output,
        };
        match super::quran::dry_run_validate(&text) {
            Ok(report) => report,
            Err(err) => {
                let (exit, message) = map_corpus_error(err);
                return CommandOutput::err(exit, message);
            }
        }
    } else {
        let (slug, version) = match target.split_once('@') {
            Some((slug, version)) if !slug.is_empty() && !version.is_empty() => (slug, version),
            _ => {
                return CommandOutput::err(
                    exit::USAGE,
                    format!("target must be a manifest path or `slug@version`, got `{target}`"),
                );
            }
        };
        let db = match open_db(db_path).await {
            Ok(db) => Arc::new(db),
            Err(err) => return open_error_output(err),
        };
        match super::quran::validate_staged(&*db, slug, version).await {
            Ok(report) => report,
            Err(err) => {
                let message = err.to_string();
                return CommandOutput::err(exit::NOT_FOUND, message);
            }
        }
    };
    if let Some(path) = report_path {
        let json = serde_json::to_string_pretty(&report).unwrap_or_default();
        if let Err(err) = std::fs::write(path, json) {
            return CommandOutput::err(exit::INTERNAL, format!("cannot write report: {err}"));
        }
    }
    let human = format!(
        "{}: {} fatal, {} errors, {} warnings ({})\n",
        target,
        report.fatal_count,
        report.error_count,
        report.warning_count,
        match report.outcome {
            quran_corpus::Outcome::Pass => "pass",
            quran_corpus::Outcome::PassWithWarnings => "pass_with_warnings",
            quran_corpus::Outcome::Fail => "fail",
        }
    );
    let exit = if report.has_fatal() { exit::VALIDATION } else { exit::OK };
    CommandOutput {
        exit,
        human: human.clone(),
        json: serde_json::to_value(&report).unwrap_or_default(),
    }
}

/// `quran catalog`: parse an upstream `editions.json` catalog file into
/// edition/translation metadata (no database, no text import).
///
/// With `database` set to a directory holding `chapterverse/` and
/// `linebyline/` mirrors (see `fixtures/upstream/README.md`), every catalog
/// entry is additionally matched to its `<slug>.txt` text files: the files are
/// parsed, counted, hashed (sha256 over the file bytes), and reported.
/// Entries with no text files stay catalog-only (`text: null`).
///
/// Human output is a deterministic summary (snapshot-safe); `--json` carries
/// the full entry array. A malformed catalog is a validation failure, never an
/// internal error; an unreadable path or report destination is a usage error.
pub async fn cmd_catalog(
    catalog_path: &str,
    revision: Option<&str>,
    report_path: Option<&str>,
    database: Option<&str>,
) -> CommandOutput {
    let text = match read_manifest(catalog_path) {
        Ok(text) => text,
        Err(output) => return output,
    };
    let entries = match quran_corpus::parse_catalog(&text, revision) {
        Ok(entries) => entries,
        Err(err) => return CommandOutput::err(exit::VALIDATION, err.to_string()),
    };
    let mut kind_counts = std::collections::BTreeMap::new();
    for entry in &entries {
        *kind_counts.entry(format!("{:?}", entry.content_kind)).or_insert(0usize) += 1;
    }
    let named: Vec<String> = entries
        .iter()
        .filter(|entry| entry.riwayah.is_known())
        .map(|entry| {
            format!(
                "{}: {} ({})",
                entry.upstream_edition_slug,
                entry.riwayah,
                entry.transmission_evidence.as_deref().unwrap_or("upstream-declared")
            )
        })
        .collect();
    let revision_value =
        revision.filter(|revision| !revision.trim().is_empty()).map(str::to_string);
    let texts: Vec<serde_json::Value> = match database {
        Some(dir) => entries
            .iter()
            .map(|entry| catalog_text_entry(dir, entry.upstream_edition_slug.as_str()))
            .collect(),
        None => entries.iter().map(|_| serde_json::Value::Null).collect(),
    };
    /// A mirror file counts as present only when it parsed (objects carrying
    /// `error` are broken, not coverage).
    fn parsed(value: &serde_json::Value, format: &str) -> bool {
        value.get(format).is_some_and(|file| file.is_object() && file.get("error").is_none())
    }
    let with_chapterverse = texts.iter().filter(|value| parsed(value, "chapterverse")).count();
    let with_linebyline = texts.iter().filter(|value| parsed(value, "linebyline")).count();
    let with_cr = texts
        .iter()
        .filter(|value| {
            value
                .get("chapterverse")
                .and_then(|file| file.get("cr_lines"))
                .and_then(serde_json::Value::as_u64)
                .is_some_and(|count| count > 0)
        })
        .count();
    let summary = serde_json::json!({
        "catalog": catalog_path,
        "entries": entries.len(),
        "kinds": kind_counts,
        "named_transmissions": named,
        "revision": revision_value,
        "pinned": revision_value.is_some(),
        "licenses": "all_unknown",
        "database": database,
        "with_chapterverse": with_chapterverse,
        "with_linebyline": with_linebyline,
        "with_cr_lines": with_cr,
    });
    if let Some(path) = report_path {
        let report = serde_json::json!({"summary": summary, "entries": entries, "texts": texts});
        let json = serde_json::to_string_pretty(&report).unwrap_or_default();
        if let Err(err) = std::fs::write(path, json) {
            return CommandOutput::err(exit::USAGE, format!("cannot write report `{path}`: {err}"));
        }
    }
    let mut human = format!(
        "catalog: {catalog_path}\nentries: {} (quran_text: {}, translation: {}, transliteration: {}, tafsir: {}, reference: {}, checksum: {})\n",
        entries.len(),
        kind_counts.get("QuranText").copied().unwrap_or(0),
        kind_counts.get("Translation").copied().unwrap_or(0),
        kind_counts.get("Transliteration").copied().unwrap_or(0),
        kind_counts.get("Tafsir").copied().unwrap_or(0),
        kind_counts.get("Reference").copied().unwrap_or(0),
        kind_counts.get("Checksum").copied().unwrap_or(0),
    );
    human.push_str(&format!("named transmissions: {}\n", named.len()));
    for line in &named {
        human.push_str(&format!("  {line}\n"));
    }
    human.push_str(&format!(
        "revision: {}\nlicenses: all unknown (redistribution not verified)\n",
        revision_value.as_deref().unwrap_or("unpinned"),
    ));
    if let Some(dir) = database {
        human.push_str(&format!(
            "texts ({dir}): {with_chapterverse}/{} chapterverse, {with_linebyline}/{} linebyline, {with_cr} with CR lines\n",
            entries.len(),
            entries.len(),
        ));
    }
    CommandOutput::ok(
        human,
        serde_json::json!({"summary": summary, "entries": entries, "texts": texts}),
    )
}

/// Match one catalog slug to its mirrored text files. Parse failures are
/// reported per file (`error`) rather than failing the whole catalog run:
/// the operator sees exactly which mirror files need attention.
fn catalog_text_entry(database: &str, slug: &str) -> serde_json::Value {
    let chapterverse = read_text_file(database, "chapterverse", slug, |bytes| {
        let text = std::str::from_utf8(bytes).map_err(|err| format!("not valid UTF-8: {err}"))?;
        let file = quran_corpus::upstream_text::parse_chapterverse(slug, text)
            .map_err(|err| err.to_string())?;
        Ok(serde_json::json!({
            "verses": file.verses.len(),
            "annotations": file.annotations.len(),
            "cr_lines": file.cr_lines,
            "sha256": quran_corpus::hashing::sha256_hex(bytes),
        }))
    });
    let linebyline = read_text_file(database, "linebyline", slug, |bytes| {
        let text = std::str::from_utf8(bytes).map_err(|err| format!("not valid UTF-8: {err}"))?;
        let file = quran_corpus::upstream_text::parse_linebyline(slug, text)
            .map_err(|err| err.to_string())?;
        Ok(serde_json::json!({
            "lines": file.lines.len(),
            "cr_lines": file.cr_lines,
            "sha256": quran_corpus::hashing::sha256_hex(bytes),
        }))
    });
    serde_json::json!({"chapterverse": chapterverse, "linebyline": linebyline})
}

/// Read and describe one mirror file: missing files stay `null` (catalog-only
/// entries); unreadable or unparsable files become `{"error": …}`.
fn read_text_file(
    database: &str,
    format: &str,
    slug: &str,
    describe: impl FnOnce(&[u8]) -> Result<serde_json::Value, String>,
) -> serde_json::Value {
    let path = format!("{database}/{format}/{slug}.txt");
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return serde_json::Value::Null,
        Err(err) => return serde_json::json!({"error": format!("cannot read `{path}`: {err}")}),
    };
    match describe(&bytes) {
        Ok(value) => value,
        Err(message) => serde_json::json!({"error": message}),
    }
}

/// `quran diff`.
pub async fn cmd_diff(
    db_path: &str,
    slug: &str,
    from: &str,
    to: &str,
    format: &str,
) -> CommandOutput {
    if !matches!(format, "text" | "json" | "unified") {
        return CommandOutput::err(exit::USAGE, format!("unknown format `{format}`"));
    }
    let db = match open_db(db_path).await {
        Ok(db) => Arc::new(db),
        Err(err) => return open_error_output(err),
    };
    let mut uow = match db.write().await {
        Ok(uow) => uow,
        Err(err) => return open_error_output(err),
    };
    let mut load = async |version: &str| -> Result<Vec<(u16, u32, String)>, CommandOutput> {
        let edition = uow
            .quran()
            .get_edition_by_slug_version(slug, version)
            .await
            .map_err(|err: StorageError| CommandOutput::err(exit::INTERNAL, err.to_string()))?
            .ok_or_else(|| {
                CommandOutput::err(exit::NOT_FOUND, format!("no edition {slug}@{version}"))
            })?;
        let rows = uow
            .quran()
            .list_ayahs_range(&edition.id, 1, i64::MAX)
            .await
            .map_err(|err: StorageError| CommandOutput::err(exit::INTERNAL, err.to_string()))?;
        Ok(rows.into_iter().map(|row| (row.surah as u16, row.ayah as u32, row.text)).collect())
    };
    let old = match load(from).await {
        Ok(old) => old,
        Err(output) => return output,
    };
    let new = match load(to).await {
        Ok(new) => new,
        Err(output) => return output,
    };
    if let Err(err) = uow.rollback().await {
        return CommandOutput::err(exit::INTERNAL, err.to_string());
    }
    let diff = quran_corpus::diff_ayahs(&old, &new, false);
    match format {
        "json" => CommandOutput::ok(
            serde_json::to_string_pretty(&diff).unwrap_or_default(),
            serde_json::to_value(&diff).unwrap_or_default(),
        ),
        "unified" => {
            let mut human = format!("--- {slug}@{from}\n+++ {slug}@{to}\n");
            for change in &diff.changes {
                match change.kind {
                    quran_corpus::ChangeKind::Added => human.push_str(&format!(
                        "+ {}:{} {}\n",
                        change.surah,
                        change.ayah,
                        change.new_text.as_deref().unwrap_or("")
                    )),
                    quran_corpus::ChangeKind::Removed => {
                        human.push_str(&format!("- {}:{}\n", change.surah, change.ayah));
                    }
                    quran_corpus::ChangeKind::Changed => human.push_str(&format!(
                        "@@ {}:{} @@\n- {}\n+ {}\n",
                        change.surah,
                        change.ayah,
                        change.old_text.as_deref().unwrap_or(""),
                        change.new_text.as_deref().unwrap_or("")
                    )),
                }
            }
            CommandOutput::ok(human, serde_json::to_value(&diff).unwrap_or_default())
        }
        _ => {
            let mut human = format!(
                "{slug}: {from} → {to}: {} added, {} removed, {} changed, {} unchanged\n",
                diff.added, diff.removed, diff.changed, diff.unchanged
            );
            for change in diff.changes.iter().take(20) {
                human.push_str(&format!(
                    "{} {}:{}\n",
                    match change.kind {
                        quran_corpus::ChangeKind::Added => "+",
                        quran_corpus::ChangeKind::Removed => "-",
                        quran_corpus::ChangeKind::Changed => "~",
                    },
                    change.surah,
                    change.ayah
                ));
            }
            if diff.changes.len() > 20 {
                human.push_str(&format!("… and {} more\n", diff.changes.len() - 20));
            }
            CommandOutput::ok(human, serde_json::to_value(&diff).unwrap_or_default())
        }
    }
}

async fn approval_flow(
    db_path: &str,
) -> Result<(Arc<SqliteDatabase>, String, String), CommandOutput> {
    let db = Arc::new(open_db(db_path).await.map_err(open_error_output)?);
    let at = now_rfc3339();
    super::quran::ensure_principal(&*db, LOCAL_PRINCIPAL, "local operator", &at)
        .await
        .map_err(|err| CommandOutput::err(exit::INTERNAL, err.to_string()))?;
    Ok((db, at, uuid::Uuid::new_v4().to_string()))
}

/// `quran activate`.
pub async fn cmd_activate(db_path: &str, edition: &str) -> CommandOutput {
    let (slug, version) = match edition.split_once('@') {
        Some((slug, version)) if !slug.is_empty() && !version.is_empty() => (slug, version),
        _ => {
            return CommandOutput::err(
                exit::USAGE,
                format!("edition must be `slug@version`, got `{edition}`"),
            );
        }
    };
    let (db, at, approval_id) = match approval_flow(db_path).await {
        Ok(flow) => flow,
        Err(output) => return output,
    };
    let subject = format!("quran-edition:{slug}@{version}");
    if let Err(err) = super::quran::record_approval(
        &*db,
        &approval_id,
        &subject,
        LOCAL_PRINCIPAL,
        LOCAL_PRINCIPAL,
        "{}",
        &at,
    )
    .await
    {
        return CommandOutput::err(exit::INTERNAL, err.to_string());
    }
    let principal: domain::PrincipalId = match LOCAL_PRINCIPAL.parse() {
        Ok(principal) => principal,
        Err(_) => return CommandOutput::err(exit::INTERNAL, "bad local principal".to_string()),
    };
    let at_ts = match at.parse::<domain::Timestamp>() {
        Ok(at) => at,
        Err(_) => return CommandOutput::err(exit::INTERNAL, "bad timestamp".to_string()),
    };
    match super::quran::activate_edition(&*db, slug, version, &principal, &approval_id, &at_ts)
        .await
    {
        Ok(generation) => CommandOutput::ok(
            format!("activated {slug}@{version} (generation {generation})\n"),
            serde_json::json!({"slug": slug, "version": version, "corpus_generation": generation}),
        ),
        Err(err) => {
            let (exit, message) = map_activation_error(err);
            CommandOutput::err(exit, message)
        }
    }
}

/// `quran rollback`.
pub async fn cmd_rollback(db_path: &str, slug: &str, to: &str) -> CommandOutput {
    let (db, at, approval_id) = match approval_flow(db_path).await {
        Ok(flow) => flow,
        Err(output) => return output,
    };
    let subject = format!("quran-edition:{slug}@{to}");
    if let Err(err) = super::quran::record_approval(
        &*db,
        &approval_id,
        &subject,
        LOCAL_PRINCIPAL,
        LOCAL_PRINCIPAL,
        "{}",
        &at,
    )
    .await
    {
        return CommandOutput::err(exit::INTERNAL, err.to_string());
    }
    let principal: domain::PrincipalId = match LOCAL_PRINCIPAL.parse() {
        Ok(principal) => principal,
        Err(_) => return CommandOutput::err(exit::INTERNAL, "bad local principal".to_string()),
    };
    let at_ts = match at.parse::<domain::Timestamp>() {
        Ok(at) => at,
        Err(_) => return CommandOutput::err(exit::INTERNAL, "bad timestamp".to_string()),
    };
    match super::quran::rollback_edition(&*db, slug, to, &principal, &approval_id, &at_ts).await {
        Ok(generation) => CommandOutput::ok(
            format!("rolled back {slug} to {to} (generation {generation})\n"),
            serde_json::json!({"slug": slug, "version": to, "corpus_generation": generation}),
        ),
        Err(err) => {
            let (exit, message) = map_activation_error(err);
            CommandOutput::err(exit, message)
        }
    }
}

/// `quran deprecate`.
pub async fn cmd_deprecate(db_path: &str, edition: &str) -> CommandOutput {
    let (slug, version) = match edition.split_once('@') {
        Some((slug, version)) if !slug.is_empty() && !version.is_empty() => (slug, version),
        _ => {
            return CommandOutput::err(
                exit::USAGE,
                format!("edition must be `slug@version`, got `{edition}`"),
            );
        }
    };
    let (db, at, approval_id) = match approval_flow(db_path).await {
        Ok(flow) => flow,
        Err(output) => return output,
    };
    let subject = format!("quran-edition:{slug}@{version}");
    if let Err(err) = super::quran::record_approval(
        &*db,
        &approval_id,
        &subject,
        LOCAL_PRINCIPAL,
        LOCAL_PRINCIPAL,
        "{}",
        &at,
    )
    .await
    {
        return CommandOutput::err(exit::INTERNAL, err.to_string());
    }
    let principal: domain::PrincipalId = match LOCAL_PRINCIPAL.parse() {
        Ok(principal) => principal,
        Err(_) => return CommandOutput::err(exit::INTERNAL, "bad local principal".to_string()),
    };
    let at_ts = match at.parse::<domain::Timestamp>() {
        Ok(at) => at,
        Err(_) => return CommandOutput::err(exit::INTERNAL, "bad timestamp".to_string()),
    };
    match super::quran::deprecate_edition(&*db, slug, version, &principal, &approval_id, &at_ts)
        .await
    {
        Ok(()) => CommandOutput::ok(
            format!("deprecated {slug}@{version}\n"),
            serde_json::json!({"slug": slug, "version": version}),
        ),
        Err(err) => {
            let (exit, message) = map_activation_error(err);
            CommandOutput::err(exit, message)
        }
    }
}

/// `quran hashes`.
pub async fn cmd_hashes(db_path: &str, edition: &str) -> CommandOutput {
    use crate::quran_reader::QuranReader;
    let (slug, version) = match edition.split_once('@') {
        Some((slug, version)) if !slug.is_empty() && !version.is_empty() => (slug, version),
        _ => {
            return CommandOutput::err(
                exit::USAGE,
                format!("edition must be `slug@version`, got `{edition}`"),
            );
        }
    };
    let db = match open_db(db_path).await {
        Ok(db) => Arc::new(db),
        Err(err) => return open_error_output(err),
    };
    let reader = reader(&db);
    let selector = match parse_selector(&format!("{slug}@{version}")) {
        Ok(selector) => selector,
        Err(output) => return output,
    };
    let edition = match reader.get_edition(&selector).await {
        Ok(edition) => edition,
        Err(err) => {
            let (exit, message) = map_reader_error(err);
            return CommandOutput::err(exit, message);
        }
    };
    // Recompute from canonical rows through the doctor path is heavyweight here;
    // compare the three stored hashes for presence and shape instead. Full
    // recomputation lives in `qai doctor --quran --deep`.
    let human = format!(
        "{}@{}:\n  text_hash: {}\n  structure_hash: {}\n  token_order_hash: {}\n",
        edition.slug,
        edition.version,
        edition.text_hash.hex,
        edition.structure_hash.hex,
        edition.token_order_hash.hex
    );
    CommandOutput::ok(
        human,
        serde_json::json!({
            "slug": edition.slug,
            "version": edition.version.to_string(),
            "text_hash": edition.text_hash.hex,
            "structure_hash": edition.structure_hash.hex,
            "token_order_hash": edition.token_order_hash.hex,
        }),
    )
}

/// `quran translation list`.
pub async fn cmd_translation_list(db_path: &str) -> CommandOutput {
    let db = match open_db(db_path).await {
        Ok(db) => Arc::new(db),
        Err(err) => return open_error_output(err),
    };
    let mut uow = match db.write().await {
        Ok(uow) => uow,
        Err(err) => return open_error_output(err),
    };
    let rows = match uow.quran().list_translation_editions().await {
        Ok(rows) => rows,
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
    };
    if let Err(err) = uow.rollback().await {
        return CommandOutput::err(exit::INTERNAL, err.to_string());
    }
    let mut human = String::new();
    for row in &rows {
        human.push_str(&format!(
            "{}@{} — {} [{}]\n",
            row.slug, row.version, row.translator, row.language
        ));
    }
    CommandOutput::ok(
        human,
        serde_json::json!(
            rows.iter()
                .map(|row| serde_json::json!({
                    "slug": row.slug, "version": row.version,
                    "translator": row.translator, "language": row.language,
                }))
                .collect::<Vec<_>>()
        ),
    )
}

/// `quran translation import`.
pub async fn cmd_translation_import(db_path: &str, manifest: &str) -> CommandOutput {
    let text = match std::fs::read_to_string(manifest) {
        Ok(text) => text,
        Err(err) => {
            return CommandOutput::err(exit::USAGE, format!("cannot read `{manifest}`: {err}"));
        }
    };
    let db = match open_db(db_path).await {
        Ok(db) => Arc::new(db),
        Err(err) => return open_error_output(err),
    };
    let at = now_rfc3339();
    if let Err(output) = ensure_principal_or_err(&db, &at).await {
        return output;
    }
    let principal: domain::PrincipalId = match LOCAL_PRINCIPAL.parse() {
        Ok(principal) => principal,
        Err(_) => return CommandOutput::err(exit::INTERNAL, "bad local principal".to_string()),
    };
    let at_ts = match at.parse::<domain::Timestamp>() {
        Ok(at) => at,
        Err(_) => return CommandOutput::err(exit::INTERNAL, "bad timestamp".to_string()),
    };
    // Source version for the translation dataset (provisioned like imports).
    let source_version_id = match ensure_translation_source(&db, &text, &at).await {
        Ok(id) => id,
        Err(output) => return output,
    };
    match super::quran::import_translations(&*db, &text, &source_version_id, &principal, &at_ts)
        .await
    {
        Ok(id) => {
            CommandOutput::ok(format!("imported translation {id}\n"), serde_json::json!({"id": id}))
        }
        Err(err) => {
            let (exit, message) = map_activation_error(err);
            CommandOutput::err(exit, message)
        }
    }
}

async fn ensure_principal_or_err(db: &Arc<SqliteDatabase>, at: &str) -> Result<(), CommandOutput> {
    super::quran::ensure_principal(&**db, LOCAL_PRINCIPAL, "local operator", at)
        .await
        .map_err(|err| CommandOutput::err(exit::INTERNAL, err.to_string()))
}

async fn ensure_translation_source(
    db: &Arc<SqliteDatabase>,
    manifest_text: &str,
    at: &str,
) -> Result<String, CommandOutput> {
    let manifest: super::quran::TranslationManifest = serde_json::from_str(manifest_text)
        .map_err(|err| CommandOutput::err(exit::VALIDATION, format!("bad manifest: {err}")))?;
    ensure_source_version(db, &manifest.translation.slug, &manifest.translation.version, at).await
}

/// `quran translation show`.
pub async fn cmd_translation_show(db_path: &str, slug: &str) -> CommandOutput {
    let db = match open_db(db_path).await {
        Ok(db) => Arc::new(db),
        Err(err) => return open_error_output(err),
    };
    let mut uow = match db.write().await {
        Ok(uow) => uow,
        Err(err) => return open_error_output(err),
    };
    let rows = match uow.quran().list_translation_editions().await {
        Ok(rows) => rows,
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
    };
    if let Err(err) = uow.rollback().await {
        return CommandOutput::err(exit::INTERNAL, err.to_string());
    }
    match rows.into_iter().find(|row| row.slug == slug) {
        Some(row) => CommandOutput::ok(
            format!("{}@{} — {} [{}]\n", row.slug, row.version, row.translator, row.language),
            serde_json::json!({
                "slug": row.slug, "version": row.version,
                "translator": row.translator, "language": row.language,
            }),
        ),
        None => CommandOutput::err(exit::NOT_FOUND, format!("no translation `{slug}`")),
    }
}

/// `quran gloss import`.
pub async fn cmd_gloss_import(db_path: &str, manifest: &str) -> CommandOutput {
    let text = match std::fs::read_to_string(manifest) {
        Ok(text) => text,
        Err(err) => {
            return CommandOutput::err(exit::USAGE, format!("cannot read `{manifest}`: {err}"));
        }
    };
    let db = match open_db(db_path).await {
        Ok(db) => Arc::new(db),
        Err(err) => return open_error_output(err),
    };
    let at = now_rfc3339();
    if let Err(output) = ensure_principal_or_err(&db, &at).await {
        return output;
    }
    let principal: domain::PrincipalId = match LOCAL_PRINCIPAL.parse() {
        Ok(principal) => principal,
        Err(_) => return CommandOutput::err(exit::INTERNAL, "bad local principal".to_string()),
    };
    let at_ts = match at.parse::<domain::Timestamp>() {
        Ok(at) => at,
        Err(_) => return CommandOutput::err(exit::INTERNAL, "bad timestamp".to_string()),
    };
    // The gloss dataset source row (provisioned like imports).
    if let Err(output) = ensure_gloss_source(&db, &text, &at).await {
        return output;
    }
    match super::quran::import_glosses(&*db, &text, &principal, &at_ts).await {
        Ok(count) => CommandOutput::ok(
            format!("imported {count} glosses\n"),
            serde_json::json!({"count": count}),
        ),
        Err(err) => {
            let (exit, message) = map_activation_error(err);
            CommandOutput::err(exit, message)
        }
    }
}

async fn ensure_gloss_source(
    db: &Arc<SqliteDatabase>,
    manifest_text: &str,
    at: &str,
) -> Result<(), CommandOutput> {
    let manifest: super::quran::GlossManifest = serde_json::from_str(manifest_text)
        .map_err(|err| CommandOutput::err(exit::VALIDATION, format!("bad manifest: {err}")))?;
    let mut uow = db
        .write()
        .await
        .map_err(|err: StorageError| CommandOutput::err(exit::INTERNAL, err.to_string()))?;
    if uow
        .sources()
        .get(&manifest.dataset.id)
        .await
        .map_err(|err: StorageError| CommandOutput::err(exit::INTERNAL, err.to_string()))?
        .is_none()
    {
        uow.sources()
            .insert_source(storage::repository::SourceRow {
                id: manifest.dataset.id.clone(),
                title: manifest.dataset.id.clone(),
                content_type: "quran_gloss".to_string(),
                language: None,
                created_at: at.to_string(),
            })
            .await
            .map_err(|err: StorageError| CommandOutput::err(exit::INTERNAL, err.to_string()))?;
    }
    uow.commit()
        .await
        .map_err(|err: StorageError| CommandOutput::err(exit::INTERNAL, err.to_string()))?;
    Ok(())
}

fn now_rfc3339() -> String {
    domain::Timestamp::now().to_string()
}

/// `doctor --quran`: the 19 corpus checks over a read-only database.
pub async fn cmd_doctor_quran(db_path: &str, deep: bool) -> CommandOutput {
    let db = match storage_sqlite::SqliteDatabase::open_read_only(db_path).await {
        Ok(db) => db,
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
    };
    let checks = match super::quran_doctor::run_quran_checks(&db, deep).await {
        Ok(checks) => checks,
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
    };
    let mut human = String::from("QURAN\n");
    for check in &checks {
        human.push_str(&format!(
            "[{}] {} — {}\n",
            check.status.as_str().to_uppercase(),
            check.id,
            check.summary
        ));
        if let Some(remedy) = &check.remedy {
            human.push_str(&format!("      remedy: {remedy}\n"));
        }
        if let Some(next) = &check.next_command {
            human.push_str(&format!("      next: {next}\n"));
        }
    }
    let json = serde_json::json!({
        "checks": checks
            .iter()
            .map(|check| serde_json::json!({
                "id": check.id,
                "status": check.status.as_str(),
                "summary": check.summary,
                "remedy": check.remedy,
                "next_command": check.next_command,
            }))
            .collect::<Vec<_>>(),
    });
    let exit = if checks.iter().any(|check| check.status == super::quran_doctor::CheckLevel::Fail) {
        exit::VALIDATION
    } else {
        exit::OK
    };
    CommandOutput { exit, human, json }
}

/// The six corpus-integrity families (D-10) and the doctor checks each covers.
///
/// Every family is derived from the existing 19-check doctor engine — no second
/// integrity engine is introduced.
const VERIFY_FAMILIES: [(&str, &[&str]); 6] = [
    ("counts", &["quran.surah_count", "quran.ayah_count"]),
    (
        "addressing",
        &[
            "quran.edition_active",
            "quran.ayah_identifiers",
            "quran.division_coverage",
            "quran.basmala_policy",
        ],
    ),
    ("unicode", &["quran.unicode_form"]),
    ("checksums", &["quran.edition_checksum", "quran.structure_hash", "quran.token_order_hash"]),
    ("roundtrip", &["quran.token_roundtrip"]),
    ("reference_comparison", &["quran.reference_corpus"]),
];

/// `quran verify` (D-10/D-11): classify the six integrity families from
/// persisted state.
///
/// Read-only by construction (T-02-07): it opens the database read-only, reuses
/// `quran_doctor::run_quran_checks`, reads the persisted QV-015 finding through
/// that engine, and is never wrapped in `confirm`. A `skipped` family is
/// serialized as `skipped` and never as `pass` (D-10).
pub async fn cmd_quran_verify(db_path: &str, edition: &str, deep: bool) -> CommandOutput {
    use super::quran_doctor::CheckLevel;
    if !edition.is_empty() && edition != "active" {
        return CommandOutput::err(
            exit::USAGE,
            format!("`quran verify` evaluates the active edition; got `{edition}`"),
        );
    }
    let db = match storage_sqlite::SqliteDatabase::open_read_only(db_path).await {
        Ok(db) => db,
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
    };
    let checks = match super::quran_doctor::run_quran_checks(&db, deep).await {
        Ok(checks) => checks,
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
    };

    let mut human = String::from("QURAN VERIFY\n");
    let mut json = serde_json::Map::new();
    let mut gate: Option<&str> = None;
    let mut any_fail = false;
    for (family, ids) in VERIFY_FAMILIES {
        let members: Vec<&super::quran_doctor::QuranDoctorCheck> =
            checks.iter().filter(|check| ids.contains(&check.id)).collect();
        let (status, summary) = if members.is_empty() {
            ("skipped", "not evaluable for this edition".to_string())
        } else if members.iter().any(|check| check.status == CheckLevel::Fail) {
            let failed: Vec<String> = members
                .iter()
                .filter(|check| check.status == CheckLevel::Fail)
                .map(|check| check.summary.clone())
                .collect();
            ("fail", failed.join("; "))
        } else if members.iter().any(|check| check.status == CheckLevel::Skipped) {
            let skipped: Vec<String> = members
                .iter()
                .filter(|check| check.status == CheckLevel::Skipped)
                .map(|check| check.summary.clone())
                .collect();
            ("skipped", skipped.join("; "))
        } else {
            ("pass", format!("{} checks pass", members.len()))
        };
        if status == "fail" {
            any_fail = true;
            if gate.is_none() {
                gate = Some(family);
            }
        }
        human.push_str(&format!("{family}: {status} — {summary}\n"));

        let mut entry = serde_json::Map::new();
        entry.insert("status".to_string(), serde_json::json!(status));
        entry.insert(
            "checks".to_string(),
            serde_json::json!(members.iter().map(|check| check.id).collect::<Vec<_>>()),
        );
        entry.insert("summary".to_string(), serde_json::json!(summary));
        // Name the exit gate: a skipped family exits OK but is never `pass`; a
        // failed family is what makes the command exit non-zero (D-10).
        entry.insert(
            "gate".to_string(),
            serde_json::json!(if status == "fail" { "exit::VALIDATION" } else { "exit::OK" }),
        );
        if status != "pass" {
            entry.insert(
                "reason".to_string(),
                serde_json::json!(if status == "skipped" {
                    "not evaluated; skipped is never counted as pass (D-10)"
                } else {
                    "one or more integrity checks failed"
                }),
            );
        }
        json.insert(family.to_string(), serde_json::Value::Object(entry));
    }
    let code = if any_fail { exit::VALIDATION } else { exit::OK };
    if let Some(family) = gate {
        human.push_str(&format!("gate: {family}\n"));
    }
    human.push_str(&format!("exit: {code}\n"));
    CommandOutput { exit: code, human, json: serde_json::Value::Object(json) }
}

/// `doctor --indexes`: the 19 Phase-2 index/linguistics checks (P2-T105).
///
/// Opens the database read-only; `deep` upgrades samples to full-corpus scans.
/// Mirrors `cmd_doctor_quran` shapes so the merged `--json` document keeps
/// validating against `docs/schemas/doctor.v1.schema.json`.
pub async fn cmd_doctor_indexes(db_path: &str, deep: bool) -> CommandOutput {
    let db = match storage_sqlite::SqliteDatabase::open_read_only(db_path).await {
        Ok(db) => db,
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
    };
    let data_dir = super::quran_index::index_root_for_db(db_path);
    let checks = match super::quran_doctor_indexes::run_index_checks(&db, &data_dir, deep).await {
        Ok(checks) => checks,
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
    };
    let mut human = String::from("INDEXES\n");
    for check in &checks {
        human.push_str(&format!(
            "[{}] {} — {}\n",
            check.status.as_str().to_uppercase(),
            check.id,
            check.summary
        ));
        if let Some(remedy) = &check.remedy {
            human.push_str(&format!("      remedy: {remedy}\n"));
        }
        if let Some(next) = &check.next_command {
            human.push_str(&format!("      next: {next}\n"));
        }
    }
    let json = serde_json::json!({
        "checks": checks
            .iter()
            .map(|check| serde_json::json!({
                "id": check.id,
                "status": check.status.as_str(),
                "summary": check.summary,
                "remedy": check.remedy,
                "next_command": check.next_command,
            }))
            .collect::<Vec<_>>(),
    });
    let exit = if checks.iter().any(|check| check.status == super::quran_doctor::CheckLevel::Fail) {
        exit::VALIDATION
    } else {
        exit::OK
    };
    CommandOutput { exit, human, json }
}

/// `quran normalize` — normalize text through a profile or adhoc rule list.
///
/// The pipeline runs from the seeded profile definitions (proving the
/// migration seed on every call); implementations come from code. `--explain`
/// prints the rule-by-rule transformation with offset-map fidelity notes.
pub async fn cmd_normalize(
    db_path: &str,
    text: Option<&str>,
    profile: Option<&str>,
    rules: Option<&str>,
    explain: bool,
) -> CommandOutput {
    use quran_normalization::error::Diagnostic as _;

    let Some(text) = text else {
        return CommandOutput::err(exit::USAGE, "provide text to normalize".to_string());
    };
    if profile.is_some() && rules.is_some() {
        return CommandOutput::err(
            exit::USAGE,
            "use either --profile or --rules, never both".to_string(),
        );
    }
    if profile.is_none() && rules.is_none() {
        return CommandOutput::err(exit::USAGE, "use --profile or --rules".to_string());
    }

    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    let rows = match db.write().await {
        Ok(mut uow) => match uow.quran().list_normalization_profiles().await {
            Ok(rows) => rows,
            Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
        },
        Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
    };
    let registry = match crate::quran_normalize::registry_from_rows(&rows) {
        Ok(registry) => registry,
        Err(error) => return CommandOutput::err(exit::INTERNAL, error.summary()),
    };

    let preview = if let Some(profile) = profile {
        let (id, version) = match crate::quran_normalize::parse_profile_spec(profile) {
            Ok(spec) => spec,
            Err(error) => return CommandOutput::err(exit::USAGE, error.summary()),
        };
        match crate::quran_normalize::preview(&registry, text, id, version) {
            Ok(preview) => preview,
            Err(error) => {
                return CommandOutput::err(map_norm_error(&error), error.summary());
            }
        }
    } else {
        let rule_ids = match crate::quran_normalize::parse_rule_list(rules.unwrap_or("")) {
            Ok(ids) => ids,
            Err(error) => return CommandOutput::err(exit::USAGE, error.summary()),
        };
        match crate::quran_normalize::preview_adhoc(text, &rule_ids) {
            Ok(preview) => preview,
            Err(error) => {
                return CommandOutput::err(map_norm_error(&error), error.summary());
            }
        }
    };

    let human = if explain {
        explain_human(&preview)
    } else {
        format!(
            "input: {}\noutput: {}\nprofile: {}",
            preview.input, preview.output, preview.profile
        )
    };
    let json = serde_json::json!({
        "input": preview.input,
        "profile": preview.profile,
        "output": preview.output,
        "trace": preview.trace,
        "steps": preview.steps,
    });
    CommandOutput::ok(human, json)
}

fn map_norm_error(error: &quran_normalization::error::NormalizationError) -> i32 {
    use quran_normalization::error::NormalizationError as E;
    match error {
        E::UnknownRule { .. } | E::UnknownProfile { .. } => exit::USAGE,
        E::ProfileImmutable { .. }
        | E::SpanOutOfRange { .. }
        | E::InvalidMapping { .. }
        | E::EmptyProfile => exit::INTERNAL,
    }
}

fn explain_human(preview: &crate::quran_normalize::Preview) -> String {
    let mut out = format!(
        "input: {}\nprofile: {}\noutput: {}\nrules applied ({}):",
        preview.input,
        preview.profile,
        preview.output,
        preview.steps.len()
    );
    for (step, applied) in preview.steps.iter().zip(preview.trace.rules_applied.iter()) {
        let kind = if applied.kind == quran_normalization::RuleKind::Heuristic {
            "heuristic"
        } else {
            "deterministic"
        };
        out.push_str(&format!(
            "\n  {} {} [{kind}] v{}\n    → {}",
            applied.rule,
            applied.rule.name(),
            applied.version,
            step.text
        ));
    }
    if preview.trace.contains_heuristic_rules {
        out.push_str("\nnote: matched using heuristic affix stripping — not verified scholarship");
    }
    out
}

/// `quran normalize --list-profiles` — profiles from the seeded catalog.
pub async fn cmd_normalize_list_profiles(db_path: &str) -> CommandOutput {
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    let rows = match db.write().await {
        Ok(mut uow) => match uow.quran().list_normalization_profiles().await {
            Ok(rows) => rows,
            Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
        },
        Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
    };
    let mut human = String::new();
    for row in &rows {
        let mut flags = if row.indexed { "indexed" } else { "query-time" }.to_string();
        if row.heuristic {
            flags.push_str(", heuristic");
        }
        if row.experimental {
            flags.push_str(", experimental");
        }
        human.push_str(&format!("{}@{} — {} [{flags}]\n", row.profile_id, row.version, row.label));
    }
    let json = serde_json::json!({
        "profiles": rows.iter().map(|row| serde_json::json!({
            "profile_id": row.profile_id,
            "version": row.version,
            "label": row.label,
            "rules": serde_json::from_str::<serde_json::Value>(&row.rules_json)
                .unwrap_or(serde_json::Value::Null),
            "indexed": row.indexed,
            "heuristic": row.heuristic,
            "experimental": row.experimental,
        })).collect::<Vec<_>>(),
    });
    CommandOutput::ok(human.trim_end().to_string(), json)
}

/// `quran normalize --show-rule` — one rule from the seeded catalog.
pub async fn cmd_normalize_show_rule(db_path: &str, rule: &str) -> CommandOutput {
    use quran_normalization::error::Diagnostic as _;

    let id = match quran_normalization::RuleId::parse(rule) {
        Ok(id) => id,
        Err(error) => return CommandOutput::err(exit::USAGE, error.summary()),
    };
    if id.is_reserved() {
        let message =
            format!("{id} {} is reserved for Phase 4 (transliteration/phonetics)", id.name());
        return CommandOutput::ok(
            message.clone(),
            serde_json::json!({"rule_id": id.as_str(), "reserved": true, "note": message}),
        );
    }
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    let rows = match db.write().await {
        Ok(mut uow) => match uow.quran().list_normalization_rules().await {
            Ok(rows) => rows,
            Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
        },
        Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
    };
    let Some(row) = rows.iter().find(|r| r.rule_id == id.as_str()) else {
        return CommandOutput::err(
            exit::NOT_FOUND,
            format!("rule {id} has no seeded implementation"),
        );
    };
    let kind = if row.kind == "heuristic" { "heuristic" } else { "deterministic" };
    let mut human =
        format!("{} {} [{kind}] v{}\n{}", row.rule_id, id.name(), row.version, row.description);
    if row.kind == "heuristic" {
        human.push_str("\nheuristic: results using this rule must be labeled");
    }
    let json = serde_json::json!({
        "rule_id": row.rule_id,
        "name": id.name(),
        "version": row.version,
        "kind": row.kind,
        "description": row.description,
        "heuristic": row.kind == "heuristic",
        "reserved": false,
    });
    CommandOutput::ok(human, json)
}

/// `quran forms rebuild` — rebuild derived forms for an edition, inline.
///
/// Runs the same [`super::quran_forms::rebuild_forms`] the job handler runs;
/// MV-018 failures surface as validation errors, never silent drift.
pub async fn cmd_forms_rebuild(db_path: &str, edition: &str) -> CommandOutput {
    use super::quran_forms::{FormsError, RebuildParams};
    use std::sync::atomic::AtomicBool;

    let (slug, version) = match edition.split_once('@') {
        Some((slug, version)) if !slug.is_empty() && !version.is_empty() => (slug, version),
        _ => return CommandOutput::err(exit::USAGE, "use slug@version".to_string()),
    };
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    let params = RebuildParams {
        edition_slug: slug.to_string(),
        edition_version: version.to_string(),
        invoked_by: LOCAL_PRINCIPAL.to_string(),
        run_tag: format!("cli-{}", uuid::Uuid::new_v4()),
    };
    match super::quran_forms::rebuild_forms(&db, &params, &AtomicBool::new(false), |_| {}).await {
        Ok(report) => {
            let human = format!(
                "rebuilt forms for {}: {} ayahs, {} tokens, {} skeletons (generation {})\nMV-018 canonical-unchanged: pass ({})\nprovenance: {}",
                report.edition_urn,
                report.ayahs,
                report.tokens,
                report.skeletons,
                report.generation,
                report.mv018.actual_hash,
                report.provenance_id
            );
            let json = serde_json::json!({
                "edition_urn": report.edition_urn,
                "generation": report.generation,
                "ayahs": report.ayahs,
                "tokens": report.tokens,
                "token_forms": report.token_forms,
                "ayah_forms": report.ayah_forms,
                "skeletons": report.skeletons,
                "provenance_id": report.provenance_id,
                "rule_set_version": report.rule_set_version,
                "mv018": {
                    "unchanged": report.mv018.unchanged,
                    "expected_hash": report.mv018.expected_hash,
                    "actual_hash": report.mv018.actual_hash,
                },
            });
            CommandOutput::ok(human, json)
        }
        Err(error) => {
            let exit = match &error {
                FormsError::Cancelled => exit::CANCELLED,
                FormsError::EditionNotFound { .. } => exit::NOT_FOUND,
                FormsError::NotActive { .. } => exit::CONFLICT,
                FormsError::Normalization(_) => exit::VALIDATION,
                FormsError::Storage(_) | FormsError::Index(_) => exit::INTERNAL,
            };
            CommandOutput::err(exit, error.to_string())
        }
    }
}

/// `quran index rebuild` — build an index generation and activate it.
///
/// Runs the same [`super::quran_index::rebuild_index`] the job handler runs.
/// The index root sits beside the database file (`<db-dir>/index`).
pub async fn cmd_index_rebuild(
    db_path: &str,
    index: Option<&str>,
    edition: Option<&str>,
) -> CommandOutput {
    use super::quran_index::{IndexBuildError, IndexBuildParams, QURAN_AYAH_INDEX_ID};
    use std::sync::atomic::AtomicBool;

    let index_id = index.unwrap_or(QURAN_AYAH_INDEX_ID).to_string();
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    let (slug, version) = match edition {
        Some(spec) => match spec.split_once('@') {
            Some((slug, version)) if !slug.is_empty() && !version.is_empty() => {
                (slug.to_string(), version.to_string())
            }
            _ => return CommandOutput::err(exit::USAGE, "use slug@version".to_string()),
        },
        None => {
            let mut uow = match db.write().await {
                Ok(uow) => uow,
                Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
            };
            let active = match uow.quran().get_active().await {
                Ok(active) => active,
                Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
            };
            let Some(active) = active else {
                return CommandOutput::err(exit::NOT_FOUND, "no active edition".to_string());
            };
            let edition = match uow.quran().get_edition(&active.edition_id).await {
                Ok(edition) => edition,
                Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
            };
            if uow.rollback().await.is_err() {
                return CommandOutput::err(exit::INTERNAL, "rollback failed".to_string());
            }
            let Some(edition) = edition else {
                return CommandOutput::err(exit::NOT_FOUND, "active edition missing".to_string());
            };
            (edition.slug, edition.version)
        }
    };
    let params = IndexBuildParams {
        index_id: index_id.clone(),
        edition_slug: slug,
        edition_version: version,
        invoked_by: LOCAL_PRINCIPAL.to_string(),
        run_tag: format!("cli-{}", uuid::Uuid::new_v4()),
        data_dir: super::quran_index::index_root_for_db(db_path),
    };
    match super::quran_index::rebuild_index(&db, &params, &AtomicBool::new(false), |_| {}).await {
        Ok(report) => {
            let human = format!(
                "index {} generation {} active: {} docs (corpus generation {})\nmanifest: {}\nprevious generation: {}\ntrigrams: {} postings over {} skeletons ({} verified)",
                report.index_id,
                report.generation,
                report.doc_count,
                report.corpus_generation,
                report.manifest_hash,
                report.previous_generation.map_or("none".to_string(), |g| g.to_string()),
                report.trigram_postings,
                report.trigram_skeletons,
                report.trigram_verified,
            );
            let json = serde_json::json!({
                "index_id": report.index_id,
                "generation": report.generation,
                "corpus_generation": report.corpus_generation,
                "doc_count": report.doc_count,
                "manifest_hash": report.manifest_hash,
                "previous_generation": report.previous_generation,
                "trigram_skeletons": report.trigram_skeletons,
                "trigram_postings": report.trigram_postings,
                "trigram_verified": report.trigram_verified,
                "mv018": {
                    "unchanged": report.mv018.unchanged,
                    "expected_hash": report.mv018.expected_hash,
                    "actual_hash": report.mv018.actual_hash,
                },
            });
            CommandOutput::ok(human, json)
        }
        Err(error) => {
            let exit = match &error {
                IndexBuildError::Cancelled => exit::CANCELLED,
                IndexBuildError::Forms(forms) => match forms {
                    super::quran_forms::FormsError::EditionNotFound { .. } => exit::NOT_FOUND,
                    super::quran_forms::FormsError::NotActive { .. } => exit::CONFLICT,
                    super::quran_forms::FormsError::Cancelled => exit::CANCELLED,
                    _ => exit::INTERNAL,
                },
                IndexBuildError::Storage(_) | IndexBuildError::Index(_) => exit::INTERNAL,
                IndexBuildError::NoPreviousGeneration { .. }
                | IndexBuildError::PreviousGenerationEvicted { .. } => exit::CONFLICT,
            };
            CommandOutput::err(exit, error.to_string())
        }
    }
}

/// `quran index verify` — verify the serving generation of an index.
pub async fn cmd_index_verify(db_path: &str, index: Option<&str>) -> CommandOutput {
    use super::quran_index::QURAN_AYAH_INDEX_ID;

    let index_id = index.unwrap_or(QURAN_AYAH_INDEX_ID);
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    let pointer = {
        let mut uow = match db.write().await {
            Ok(uow) => uow,
            Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
        };
        let pointer = match uow.quran().get_index_pointer(index_id).await {
            Ok(pointer) => pointer,
            Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
        };
        if uow.rollback().await.is_err() {
            return CommandOutput::err(exit::INTERNAL, "rollback failed".to_string());
        }
        pointer
    };
    let Some(pointer) = pointer else {
        return CommandOutput::err(exit::NOT_FOUND, format!("no pointer for index {index_id}"));
    };
    let manifest: quran_search::IndexManifest = match serde_json::from_str(&pointer.manifest_json) {
        Ok(manifest) => manifest,
        Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
    };
    let registry = crate::quran_normalize::builtin_registry();
    let family = match quran_search::TokenizerFamily::new(&registry, manifest.tokenizer_version) {
        Ok(family) => family,
        Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
    };
    let root = super::quran_index::index_root_for_db(db_path);
    let index =
        match quran_search::Fts5Index::open(&root, pointer.generation as u64, manifest, family)
            .await
        {
            Ok(index) => index,
            Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
        };
    let report = match quran_search::FullTextIndex::verify(&index).await {
        Ok(report) => report,
        Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
    };
    let human = if report.ok {
        format!(
            "index {index_id} generation {} healthy: {} docs",
            pointer.generation, report.doc_count
        )
    } else {
        format!(
            "index {index_id} generation {} FAILED: {}",
            pointer.generation,
            report.findings.join("; ")
        )
    };
    let json = serde_json::json!({
        "index_id": index_id,
        "generation": pointer.generation,
        "ok": report.ok,
        "doc_count": report.doc_count,
        "findings": report.findings,
    });
    CommandOutput { exit: if report.ok { exit::OK } else { exit::VALIDATION }, human, json }
}

/// Enforce index-generation retention: `qai quran index gc` (P2-T35).
pub async fn cmd_index_gc(db_path: &str, index: Option<&str>, keep: usize) -> CommandOutput {
    use super::quran_index::{GcParams, QURAN_AYAH_INDEX_ID, gc_index};
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    let params = GcParams {
        index_id: index.unwrap_or(QURAN_AYAH_INDEX_ID).to_string(),
        keep,
        invoked_by: LOCAL_PRINCIPAL.to_string(),
        data_dir: super::quran_index::index_root_for_db(db_path),
    };
    match gc_index(&db, &params, &std::sync::atomic::AtomicBool::new(false)).await {
        Ok(report) => {
            let human = format!(
                "index {} gc: active generation {}, kept {:?}, removed {} generation(s)",
                report.index_id,
                report.active_generation.map_or("none".to_string(), |g| g.to_string()),
                report.kept,
                report.removed.len(),
            );
            let json = serde_json::json!({
                "index_id": report.index_id,
                "active_generation": report.active_generation,
                "kept": report.kept,
                "removed": report.removed.iter().map(|r| serde_json::json!({
                    "generation": r.generation,
                    "run_id_deleted": r.run_id_deleted,
                    "docs_removed": r.docs_removed,
                })).collect::<Vec<_>>(),
            });
            CommandOutput::ok(human, json)
        }
        Err(error) => {
            use super::quran_index::IndexBuildError;
            let exit = match &error {
                IndexBuildError::Cancelled => exit::CANCELLED,
                _ => exit::INTERNAL,
            };
            CommandOutput::err(exit, error.to_string())
        }
    }
}

/// Restore the previous serving generation: `qai quran index rollback` (P2-T35).
///
/// Single-step undo of the last index activation. The newer generation stays
/// on disk and in the run history; only the pointer and run states move.
pub async fn cmd_index_rollback(db_path: &str, index: Option<&str>) -> CommandOutput {
    use super::quran_index::{QURAN_AYAH_INDEX_ID, RollbackParams, rollback_index_single_step};
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    let params = RollbackParams {
        index_id: index.unwrap_or(QURAN_AYAH_INDEX_ID).to_string(),
        invoked_by: LOCAL_PRINCIPAL.to_string(),
        data_dir: super::quran_index::index_root_for_db(db_path),
    };
    match rollback_index_single_step(&db, &params).await {
        Ok(report) => {
            let human = format!(
                "index {} rolled back: generation {} → {} (previous generation retained on disk)",
                report.index_id, report.from_generation, report.to_generation,
            );
            let json = serde_json::json!({
                "index_id": report.index_id,
                "from_generation": report.from_generation,
                "to_generation": report.to_generation,
            });
            CommandOutput::ok(human, json)
        }
        Err(error) => {
            use super::quran_index::IndexBuildError;
            let exit = match &error {
                IndexBuildError::NoPreviousGeneration { .. }
                | IndexBuildError::PreviousGenerationEvicted { .. } => exit::CONFLICT,
                IndexBuildError::Cancelled => exit::CANCELLED,
                _ => exit::INTERNAL,
            };
            CommandOutput::err(exit, error.to_string())
        }
    }
}

// ─── Counting & discovery commands (P2-T104) ────────────────────────────

fn json_or_err<T: serde::Serialize>(human: String, value: &T) -> CommandOutput {
    match serde_json::to_value(value) {
        Ok(json) => CommandOutput::ok(human, json),
        Err(error) => CommandOutput::err(exit::INTERNAL, error.to_string()),
    }
}

fn counting_exit(error: &super::quran_counting::CountingError) -> i32 {
    use super::quran_counting::CountingError as C;
    match error {
        C::UnknownProfile(_) | C::EmptyTarget => exit::USAGE,
        C::UnavailableDataset { .. } => exit::NOT_FOUND,
        C::ProfileNotCountable(_) => exit::VALIDATION,
        C::Storage(_) => exit::INTERNAL,
    }
}

/// `qai quran count frequency`.
pub async fn cmd_count_frequency(db_path: &str, target: &str, profile: &str) -> CommandOutput {
    use super::quran_counting::frequency;
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    match frequency(&db, target, profile).await {
        Ok(report) => json_or_err(
            format!(
                "{}: {} occurrence(s) under {} (checksum {})",
                report.target, report.count, report.rules.profile, report.checksum
            ),
            &report,
        ),
        Err(error) => CommandOutput::err(counting_exit(&error), error.to_string()),
    }
}

/// Parse the `--mode` flag into a multi-analysis handling policy.
///
/// Shared with the HTTP lexicon surface ([`crate::quran_lexicon_api`]) so an
/// API count and a CLI count are described by the same convention.
pub fn parse_multi_analysis_mode(
    mode: &str,
) -> Result<super::quran_counting::MultiAnalysisHandling, String> {
    use super::quran_counting::MultiAnalysisHandling as M;
    match mode {
        "single-source" | "single" => Ok(M::SingleSource),
        "all-analyses" | "all" => Ok(M::AllAnalyses),
        "one-vote-per-token" | "one-vote" => Ok(M::OneVotePerToken),
        other => Err(format!(
            "unknown multi-analysis mode: {other} \
             (use single-source, all-analyses, or one-vote-per-token)"
        )),
    }
}

/// `qai quran count root-frequency` (SC4/G-01): exact lexicon count with the
/// active dataset and the requested multi-analysis mode in the rules block.
pub async fn cmd_count_root_frequency(
    db_path: &str,
    root: &str,
    profile: &str,
    mode: &str,
) -> CommandOutput {
    use super::quran_counting::root_frequency;
    let mode = match parse_multi_analysis_mode(mode) {
        Ok(mode) => mode,
        Err(message) => return CommandOutput::err(exit::USAGE, message),
    };
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    match root_frequency(&db, root, profile, mode).await {
        Ok(report) => json_or_err(
            format!(
                "{}: {} occurrence(s) under {} [{}] (checksum {})",
                report.target,
                report.count,
                report.rules.profile,
                report.rules.datasets.join(","),
                report.checksum
            ),
            &report,
        ),
        Err(error) => CommandOutput::err(counting_exit(&error), error.to_string()),
    }
}

/// `qai quran count lemma-frequency` (SC4/G-01): the lemma analogue.
pub async fn cmd_count_lemma_frequency(
    db_path: &str,
    lemma: &str,
    profile: &str,
    mode: &str,
) -> CommandOutput {
    use super::quran_counting::lemma_frequency;
    let mode = match parse_multi_analysis_mode(mode) {
        Ok(mode) => mode,
        Err(message) => return CommandOutput::err(exit::USAGE, message),
    };
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    match lemma_frequency(&db, lemma, profile, mode).await {
        Ok(report) => json_or_err(
            format!(
                "{}: {} occurrence(s) under {} [{}] (checksum {})",
                report.target,
                report.count,
                report.rules.profile,
                report.rules.datasets.join(","),
                report.checksum
            ),
            &report,
        ),
        Err(error) => CommandOutput::err(counting_exit(&error), error.to_string()),
    }
}

/// `qai quran freq <text>` (D-10): a thin alias for surface-form frequency.
/// Surface counts are single-source; a non-default `--mode` is refused rather
/// than silently ignored.
pub async fn cmd_freq(db_path: &str, target: &str, profile: &str, mode: &str) -> CommandOutput {
    if mode != "single-source" && mode != "single" {
        return CommandOutput::err(
            exit::USAGE,
            format!(
                "--mode {mode} applies to `qai quran count root-frequency` / \
                 `lemma-frequency`, not `freq`"
            ),
        );
    }
    cmd_count_frequency(db_path, target, profile).await
}

/// `qai quran count distribution`.
pub async fn cmd_count_distribution(db_path: &str, target: &str, profile: &str) -> CommandOutput {
    use super::quran_counting::distribution;
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    match distribution(&db, target, profile).await {
        Ok(report) => json_or_err(
            format!(
                "{}: {} occurrence(s) across {} surah(s); {}",
                report.frequency.target,
                report.frequency.count,
                report.frequency.by_surah.len(),
                report.partition_provenance
            ),
            &report,
        ),
        Err(error) => CommandOutput::err(counting_exit(&error), error.to_string()),
    }
}

/// `qai quran count occurrences`.
pub async fn cmd_count_occurrences(db_path: &str, target: &str, profile: &str) -> CommandOutput {
    use super::quran_counting::first_last_occurrence;
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    match first_last_occurrence(&db, target, profile).await {
        Ok(report) => json_or_err(
            format!(
                "first {:?}, last {:?}, span {:?}\n{}",
                report.first, report.last, report.ayah_span, report.disclaimer
            ),
            &report,
        ),
        Err(error) => CommandOutput::err(counting_exit(&error), error.to_string()),
    }
}

/// `qai quran count hapax`.
pub async fn cmd_count_hapax(db_path: &str, profile: &str, limit: usize) -> CommandOutput {
    use super::quran_counting::hapax_search;
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    match hapax_search(&db, profile, limit).await {
        Ok(report) => json_or_err(
            format!("profile {}: {} hapax form(s)", report.profile, report.hapax.len()),
            &report,
        ),
        Err(error) => CommandOutput::err(counting_exit(&error), error.to_string()),
    }
}

/// `qai quran count cooccurrence`.
pub async fn cmd_count_cooccurrence(
    db_path: &str,
    target: &str,
    profile: &str,
    window: usize,
    limit: usize,
) -> CommandOutput {
    use super::quran_counting::cooccurrence;
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    match cooccurrence(&db, target, profile, window, limit).await {
        Ok((rules, hits)) => json_or_err(
            format!("co-occurrence within token:{} — {} candidate(s)", window, hits.len()),
            &serde_json::json!({"rules": rules, "hits": hits}),
        ),
        Err(error) => CommandOutput::err(counting_exit(&error), error.to_string()),
    }
}

/// `qai quran count collocation`.
pub async fn cmd_count_collocation(
    db_path: &str,
    target: &str,
    profile: &str,
    window: usize,
    limit: usize,
) -> CommandOutput {
    use super::quran_counting::collocation;
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    match collocation(&db, target, profile, window, limit).await {
        Ok((rules, hits)) => json_or_err(
            format!("collocations (LLR-ranked) — {} candidate(s)", hits.len()),
            &serde_json::json!({"rules": rules, "hits": hits}),
        ),
        Err(error) => CommandOutput::err(counting_exit(&error), error.to_string()),
    }
}

/// `qai quran count numeric-report`.
pub async fn cmd_count_numeric_report(db_path: &str, target: &str, profile: &str) -> CommandOutput {
    use super::quran_counting::numeric_report;
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    match numeric_report(&db, target, profile).await {
        Ok(report) => json_or_err(
            format!("{}: {}\n{}", report.frequency.target, report.frequency.count, report.note),
            &report,
        ),
        Err(error) => CommandOutput::err(counting_exit(&error), error.to_string()),
    }
}

/// `qai quran count missing-form`.
pub async fn cmd_count_missing_form(db_path: &str, target: &str, profile: &str) -> CommandOutput {
    use super::quran_counting::missing_expected_form;
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    match missing_expected_form(&db, target, profile).await {
        Ok(report) => json_or_err(
            format!(
                "{}: 0 occurrence(s) under {}\n{}",
                report.target, report.rules.profile, report.disclaimer
            ),
            &report,
        ),
        Err(error) => CommandOutput::err(counting_exit(&error), error.to_string()),
    }
}

/// `qai quran count near-duplicates`.
pub async fn cmd_count_near_duplicates(
    db_path: &str,
    threshold: f64,
    limit: usize,
) -> CommandOutput {
    use super::quran_counting::near_duplicate_passages;
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    match near_duplicate_passages(&db, threshold, limit).await {
        Ok((rules, hits)) => json_or_err(
            format!("near-duplicates (>= {threshold}) — {} pair(s)", hits.len()),
            &serde_json::json!({"rules": rules, "hits": hits}),
        ),
        Err(error) => CommandOutput::err(counting_exit(&error), error.to_string()),
    }
}

// ─── Morphology commands (import/activate separation, T69/T90) ──────────

fn morphology_exit(error: &super::quran_morphology::MorphologyJobError) -> i32 {
    use super::quran_morphology::MorphologyJobError as M;
    match error {
        M::Cancelled => exit::CANCELLED,
        M::BlockingFindings(..) | M::UnmatchedRemaining(..) => exit::VALIDATION,
        M::LicenseEvidence { .. } => exit::VALIDATION,
        M::Approval(_) => exit::CONFLICT,
        M::BadState { .. } => exit::CONFLICT,
        M::CanonicalChanged => exit::CONFLICT,
        _ => exit::INTERNAL,
    }
}

fn tool_exit(error: &super::quran_morphology::MorphologyToolError) -> i32 {
    use super::quran_morphology::MorphologyToolError as T;
    match error {
        T::UnavailableDataset { .. } => exit::NOT_FOUND,
        T::UnavailablePatternField { .. } => exit::VALIDATION,
        T::Morphology(quran_morphology::MorphologyError::UnknownDataset { .. }) => exit::NOT_FOUND,
        T::Morphology(_) => exit::VALIDATION,
        T::Storage(_) => exit::INTERNAL,
    }
}

/// Resolve operator license evidence into `(license_status, license_json)`.
///
/// `--license-evidence` points at a capture object (the `licenses/README.md`
/// `capture.json` shape) or a machine-readable matrix with an `artifacts` map
/// (`fixtures/quran/morphology/license-matrix.json`), selecting the entry that
/// matches the dataset slug (or the sole entry). A missing/unreadable file or
/// invalid JSON is a typed usage error — never a silent default.
fn resolve_license_evidence(
    dataset: &str,
    license_status: Option<&str>,
    license_json: Option<&str>,
    license_evidence: Option<&str>,
) -> Result<(String, String), String> {
    let Some(path) = license_evidence else {
        return Ok((
            license_status.unwrap_or("Unspecified").to_string(),
            license_json.unwrap_or("{}").to_string(),
        ));
    };
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("read license evidence {path}: {error}"))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|error| format!("license evidence {path} is not valid JSON: {error}"))?;
    let entry = select_evidence_entry(&value, dataset)?;
    let json = serde_json::to_string(entry)
        .map_err(|error| format!("license evidence {path} cannot be re-encoded: {error}"))?;
    let status = license_status.map(str::to_string).unwrap_or_else(|| derive_license_status(entry));
    Ok((status, json))
}

/// Select one capture entry from an evidence file (matrix or bare capture).
fn select_evidence_entry<'a>(
    value: &'a serde_json::Value,
    dataset: &str,
) -> Result<&'a serde_json::Value, String> {
    if let Some(artifacts) = value.get("artifacts").and_then(serde_json::Value::as_object) {
        if let Some(entry) = artifacts.get(dataset) {
            return Ok(entry);
        }
        if artifacts.len() == 1 {
            return Ok(artifacts.values().next().expect("one entry"));
        }
        return Err(format!(
            "license evidence matrix has {} entries and none matches dataset `{dataset}`",
            artifacts.len()
        ));
    }
    if value.is_object() {
        return Ok(value);
    }
    Err("license evidence must be a JSON object".to_string())
}

/// Derive a permissive status from captured `spdx_id`/`redistribution_allowed`.
///
/// Never invents permissiveness: without a redistribution grant it falls back
/// to `Unspecified`, which the activation gate rejects.
fn derive_license_status(entry: &serde_json::Value) -> String {
    let has_spdx = entry
        .get("spdx_id")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| !value.trim().is_empty());
    if has_spdx {
        "OpenLicense".to_string()
    } else if entry.get("redistribution_allowed").and_then(serde_json::Value::as_bool) == Some(true)
    {
        "PermissionGranted".to_string()
    } else {
        "Unspecified".to_string()
    }
}

/// `qai quran morphology import`.
#[allow(clippy::too_many_arguments)]
pub async fn cmd_morphology_import(
    db_path: &str,
    file: &str,
    dataset: &str,
    version: &str,
    adapter: &str,
    edition: &str,
    attribution: &str,
    batch: Option<&str>,
    license_status: Option<&str>,
    license_json: Option<&str>,
    license_evidence: Option<&str>,
) -> CommandOutput {
    use super::quran_morphology::{MorphologyImportParams, run_morphology_import};
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    let document_text = match std::fs::read_to_string(file) {
        Ok(text) => text,
        Err(error) => {
            return CommandOutput::err(exit::NOT_FOUND, format!("read {file}: {error}"));
        }
    };
    let (slug, edition_version) = match edition.split_once('@') {
        Some((slug, version)) if !slug.is_empty() && !version.is_empty() => {
            (slug.to_string(), version.to_string())
        }
        _ => return CommandOutput::err(exit::USAGE, "use --edition slug@version".to_string()),
    };
    let (license_status, license_json) =
        match resolve_license_evidence(dataset, license_status, license_json, license_evidence) {
            Ok(pair) => pair,
            Err(message) => return CommandOutput::err(exit::USAGE, message),
        };
    let params = MorphologyImportParams {
        dataset_slug: dataset.to_string(),
        dataset_version: version.to_string(),
        adapter: adapter.to_string(),
        document_text,
        edition_slug: slug,
        edition_version,
        invoked_by: LOCAL_PRINCIPAL.to_string(),
        batch_id: batch.map(str::to_string),
        attribution: attribution.to_string(),
        license_status,
        license_json,
    };
    match run_morphology_import(&db, &params, &std::sync::atomic::AtomicBool::new(false), |_| {})
        .await
    {
        Ok(report) => json_or_err(
            format!(
                "imported {} batch {}: {} matched, {} table-mapped, state {}",
                report.dataset, report.batch_id, report.matched, report.table_mapped, report.state
            ),
            &serde_json::json!({
                "batch_id": report.batch_id,
                "dataset": report.dataset,
                "matched": report.matched,
                "table_mapped": report.table_mapped,
                "unmatched_by_surah": report.unmatched_by_surah,
                "findings": report.findings,
                "state": report.state,
                "mv018_unchanged": report.mv018_unchanged,
            }),
        ),
        Err(error) => CommandOutput::err(morphology_exit(&error), error.to_string()),
    }
}

/// `qai quran morphology activate`.
pub async fn cmd_morphology_activate(db_path: &str, batch: &str, approval: &str) -> CommandOutput {
    use super::quran_morphology::{MorphologyActivateParams, activate_morphology};
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    let principal: domain::PrincipalId = match LOCAL_PRINCIPAL.parse() {
        Ok(principal) => principal,
        Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
    };
    let params = MorphologyActivateParams {
        batch_id: batch.to_string(),
        approval_id: approval.to_string(),
        invoked_by: LOCAL_PRINCIPAL.to_string(),
    };
    match activate_morphology(&db, &params, &principal).await {
        Ok(report) => json_or_err(
            format!(
                "activated {}: {} roots, {} lemmas, {} analyses, {} morphemes",
                report.dataset,
                report.promoted.0,
                report.promoted.1,
                report.promoted.2,
                report.promoted.3
            ),
            &serde_json::json!({
                "dataset": report.dataset,
                "roots": report.promoted.0,
                "lemmas": report.promoted.1,
                "analyses": report.promoted.2,
                "morphemes": report.promoted.3,
                "previous_dataset": report.previous_dataset,
            }),
        ),
        Err(error) => CommandOutput::err(morphology_exit(&error), error.to_string()),
    }
}

/// `qai quran morphology datasets`.
pub async fn cmd_morphology_datasets(db_path: &str) -> CommandOutput {
    use storage::Database as _;
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    let mut uow = match db.write().await {
        Ok(uow) => uow,
        Err(error) => return open_error_output(error),
    };
    let datasets = match uow.quran().list_datasets().await {
        Ok(datasets) => datasets,
        Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
    };
    let _ = uow.rollback().await;
    let rows: Vec<serde_json::Value> = datasets
        .iter()
        .map(|d| {
            serde_json::json!({
                "slug": d.slug, "version": d.version, "state": d.state,
                "attribution": d.attribution, "license_status": d.license_status,
            })
        })
        .collect();
    let human = if datasets.is_empty() {
        "no morphology datasets registered".to_string()
    } else {
        datasets
            .iter()
            .map(|d| format!("{}@{} [{}]", d.slug, d.version, d.state))
            .collect::<Vec<_>>()
            .join("\n")
    };
    CommandOutput::ok(human, serde_json::json!({ "datasets": rows }))
}

/// `qai quran root list`.
pub async fn cmd_morphology_root_list(
    db_path: &str,
    prefix: Option<&str>,
    limit: usize,
) -> CommandOutput {
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    match super::quran_morphology::browse_roots(&db, prefix, limit).await {
        Ok((dataset, roots)) => {
            let mut human = vec![format!("{}: {} root(s)", dataset, roots.len())];
            for root in &roots {
                human.push(format!("{} [{}] {}", root.root, root.normalized, root.status));
            }
            json_or_err(
                human.join("\n"),
                &serde_json::json!({
                    "dataset": dataset,
                    "prefix": prefix.unwrap_or_default(),
                    "roots": roots,
                }),
            )
        }
        Err(error) => CommandOutput::err(tool_exit(&error), error.to_string()),
    }
}

/// `qai quran morphology diff`.
pub async fn cmd_morphology_diff(
    db_path: &str,
    from: &str,
    to: &str,
    format: &str,
) -> CommandOutput {
    use super::quran_morphology::{MorphologyDiffKind, diff_datasets};
    if !matches!(format, "text" | "json") {
        return CommandOutput::err(exit::USAGE, format!("unknown format `{format}`"));
    }
    let (from_slug, from_version) = match dataset_spec(from) {
        Ok(parts) => parts,
        Err(message) => return CommandOutput::err(exit::USAGE, message.to_string()),
    };
    let (to_slug, to_version) = match dataset_spec(to) {
        Ok(parts) => parts,
        Err(message) => return CommandOutput::err(exit::USAGE, message.to_string()),
    };
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    match diff_datasets(&db, &from_slug, &from_version, &to_slug, &to_version).await {
        Ok(report) => {
            let human = format!(
                "{} → {}: {} added, {} removed, {} changed, {} unchanged",
                report.from_dataset,
                report.to_dataset,
                report.added,
                report.removed,
                report.changed,
                report.unchanged
            );
            let mut lines = vec![human];
            for change in report.changes.iter().take(20) {
                let label = match change.kind {
                    MorphologyDiffKind::Added => "+",
                    MorphologyDiffKind::Removed => "-",
                    MorphologyDiffKind::Changed => "~",
                };
                lines.push(format!("{label} {}", change.reference));
            }
            if report.changes.len() > 20 {
                lines.push(format!("… {} more change(s)", report.changes.len() - 20));
            }
            if format == "json" {
                CommandOutput::ok(
                    lines.join("\n"),
                    serde_json::to_value(&report).unwrap_or_default(),
                )
            } else {
                json_or_err(lines.join("\n"), &report)
            }
        }
        Err(error) => CommandOutput::err(tool_exit(&error), error.to_string()),
    }
}

fn dataset_spec(dataset: &str) -> Result<(String, String), &'static str> {
    match dataset.split_once('@') {
        Some((slug, version)) if !slug.is_empty() && !version.is_empty() => {
            Ok((slug.to_string(), version.to_string()))
        }
        _ => Err("use dataset slug@version"),
    }
}

fn edition_spec(edition: &str) -> Result<(String, String), &'static str> {
    match edition.split_once('@') {
        Some((slug, version)) if !slug.is_empty() && !version.is_empty() => {
            Ok((slug.to_string(), version.to_string()))
        }
        _ => Err("use --edition slug@version"),
    }
}

/// `qai quran morphology token`.
pub async fn cmd_morphology_token(
    db_path: &str,
    edition: &str,
    surah: i64,
    ayah: i64,
    position: i64,
) -> CommandOutput {
    use super::quran_morphology::morphology_for_token;
    let (slug, version) = match edition_spec(edition) {
        Ok(parts) => parts,
        Err(message) => return CommandOutput::err(exit::USAGE, message.to_string()),
    };
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    match morphology_for_token(&db, &slug, &version, surah, ayah, position).await {
        Ok((dataset, analyses)) => json_or_err(
            format!("{}:{}:{} — {} analysis(es)", surah, ayah, position, analyses.len()),
            &serde_json::json!({"dataset": dataset, "analyses": analyses}),
        ),
        Err(error) => CommandOutput::err(tool_exit(&error), error.to_string()),
    }
}

/// `qai quran morphology compare`.
pub async fn cmd_morphology_compare(
    db_path: &str,
    edition: &str,
    surah: i64,
    ayah: i64,
    position: i64,
) -> CommandOutput {
    use super::quran_morphology::morphology_compare;
    let (slug, version) = match edition_spec(edition) {
        Ok(parts) => parts,
        Err(message) => return CommandOutput::err(exit::USAGE, message.to_string()),
    };
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    match morphology_compare(&db, &slug, &version, surah, ayah, position).await {
        Ok(verdicts) => json_or_err(
            format!(
                "{}:{}:{} — {} field verdict(s); no resolution given",
                surah,
                ayah,
                position,
                verdicts.len()
            ),
            &serde_json::json!({"verdicts": verdicts}),
        ),
        Err(error) => CommandOutput::err(tool_exit(&error), error.to_string()),
    }
}

/// `qai quran morphology root`.
pub async fn cmd_morphology_root(db_path: &str, root: &str) -> CommandOutput {
    use super::quran_morphology::root_search;
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    match root_search(&db, root).await {
        Ok((dataset, occurrences)) => json_or_err(
            format!("root {root}: {} occurrence(s) in {dataset}", occurrences.len()),
            &serde_json::json!({"dataset": dataset, "occurrences": occurrences}),
        ),
        Err(error) => CommandOutput::err(tool_exit(&error), error.to_string()),
    }
}

/// `qai quran morphology lemma`.
pub async fn cmd_morphology_lemma(db_path: &str, lemma: &str) -> CommandOutput {
    use super::quran_morphology::lemma_search;
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    match lemma_search(&db, lemma).await {
        Ok((dataset, occurrences)) => json_or_err(
            format!("lemma {lemma}: {} occurrence(s) in {dataset}", occurrences.len()),
            &serde_json::json!({"dataset": dataset, "occurrences": occurrences}),
        ),
        Err(error) => CommandOutput::err(tool_exit(&error), error.to_string()),
    }
}

/// `qai quran family <kind> <id>` (SC3/G-02/D-10): typed and explained family
/// relations for one lexicon member, with the dataset named in the payload.
///
/// With no active dataset the read path returns the typed
/// `UnavailableDataset` (`QAI-MORPH-0004`) mapped through `tool_exit` — never
/// an empty relation list an operator could read as "no such family".
pub async fn cmd_family(db_path: &str, kind: &str, id: &str) -> CommandOutput {
    use super::quran_morphology::word_family;
    if kind.trim().is_empty() || id.trim().is_empty() {
        return CommandOutput::err(
            exit::USAGE,
            "family needs a non-empty <kind> and <id>".to_string(),
        );
    }
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    match word_family(&db, kind, id).await {
        Ok((dataset, members)) => json_or_err(
            format!("{kind} {id}: {} family relation(s) in {dataset}", members.len()),
            &serde_json::json!({"dataset": dataset, "relations": members}),
        ),
        Err(error) => CommandOutput::err(tool_exit(&error), error.to_string()),
    }
}

/// `qai quran morphology affix`.
pub async fn cmd_morphology_affix(db_path: &str, affix: &str, profile: &str) -> CommandOutput {
    use super::quran_morphology::affix_search;
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    match affix_search(&db, affix, profile).await {
        Ok(hits) => json_or_err(
            format!("affix {affix}: {} hit(s)", hits.len()),
            &serde_json::json!({"hits": hits}),
        ),
        Err(error) => CommandOutput::err(tool_exit(&error), error.to_string()),
    }
}

// ─── Knowledge-graph commands (TASK-424 slice) ──────────────────────────

/// On-disk projection document: manifest + node/edge sets.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct GraphFile {
    /// Projection manifest (identity, versions, status).
    manifest: quran_graph::ProjectionManifest,
    /// Nodes sorted by stable id.
    nodes: Vec<quran_graph::GraphNode>,
    /// Edges sorted by `(src, edge, dst)`.
    edges: Vec<quran_graph::GraphEdge>,
}

fn read_graph_file(file: &str) -> Result<GraphFile, String> {
    let text = std::fs::read_to_string(file).map_err(|error| format!("read {file}: {error}"))?;
    serde_json::from_str(&text).map_err(|error| format!("parse {file}: {error}"))
}

fn stage_graph(graph: &GraphFile) -> Result<quran_graph::MemGraphStore, String> {
    use quran_graph::{GraphStore, MemGraphStore};
    let mut store = MemGraphStore::new(graph.manifest.projection_id.clone());
    store.stage_nodes(graph.nodes.clone()).map_err(|error| error.to_string())?;
    store.stage_edges(graph.edges.clone()).map_err(|error| error.to_string())?;
    Ok(store)
}

/// `qai quran graph build`: structural projection from the active edition,
/// persisted to SQLite (staged-batch build with fenced publish) with an
/// optional JSON document on disk.
pub async fn cmd_graph_build(db_path: &str, out: Option<&str>) -> CommandOutput {
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    let collected = match super::quran_graph_build::collect_structural_input(&db).await {
        Ok(Some(collected)) => collected,
        Ok(None) => {
            return CommandOutput::err(
                exit::NOT_FOUND,
                "no active edition; import one first".to_string(),
            );
        }
        Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
    };
    let edition_id = collected.edition_id.clone();
    let report = match super::quran_graph_build::publish_structural_build(db_path, &collected).await
    {
        Ok(report) => report,
        Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
    };
    let document = GraphFile {
        manifest: report.manifest,
        nodes: report.built.nodes,
        edges: report.built.edges,
    };
    let json = match serde_json::to_value(&document) {
        Ok(json) => json,
        Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
    };
    if let Some(path) = out {
        let text = serde_json::to_string_pretty(&document).unwrap_or_default();
        if let Err(error) = std::fs::write(path, text) {
            return CommandOutput::err(exit::INTERNAL, format!("write {path}: {error}"));
        }
    }
    CommandOutput::ok(
        format!(
            "structural projection: {} nodes, {} edges (edition {edition_id})",
            document.nodes.len(),
            document.edges.len()
        ),
        json,
    )
}

/// Map a [`quran_graph::GraphError`] to the operator surface: unknown nodes
/// and projections are not-found, budget/pattern violations are validation
/// failures (never silent truncation), denials are policy, build failures
/// are internal.
fn graph_error_exit(error: &quran_graph::GraphError) -> i32 {
    use quran_graph::GraphError as E;
    match error {
        E::NodeNotFound { .. } | E::UnknownProjection { .. } | E::UnknownAssertion { .. } => {
            exit::NOT_FOUND
        }
        E::BudgetExceeded { .. } | E::PatternRejected { .. } => exit::VALIDATION,
        E::AuthzDenied { .. } => exit::POLICY,
        E::BuildFailed { .. } => exit::INTERNAL,
    }
}

/// Render a [`quran_graph::GraphError`] with its stable `QAI-GRAPH-*` code,
/// remedy, and next command (never a bare message without the code).
fn graph_error_output(error: &quran_graph::GraphError) -> CommandOutput {
    use quran_graph::Diagnostic as _;
    CommandOutput::err(graph_error_exit(error), error.render_human())
}

/// Map a read-service failure to the operator surface. Graph errors render
/// through the `QAI-GRAPH-*` human form; morphology errors carry their
/// `QAI-MORPH-*` code, so the word-root dataset gate stays a typed exit 5
/// with `QAI-MORPH-0004` in the message.
fn graph_api_output(error: &super::quran_graph_api::GraphApiError) -> CommandOutput {
    use super::quran_graph_api::GraphApiError as E;
    match error {
        E::Graph(inner) => graph_error_output(inner),
        E::Morphology(inner) => {
            use storage::error::Diagnostic as _;
            CommandOutput::err(tool_exit(inner), format!("[{}] {inner}", inner.code()))
        }
    }
}

/// Operator-supplied budget overrides for graph reads. `None` keeps the
/// `QueryBudgets::default()` value; explicit out-of-range values fail
/// pre-flight (never clamp).
#[derive(Debug, Clone, Default)]
pub struct GraphBudgets {
    /// Max distinct nodes collected per query.
    pub max_nodes: Option<usize>,
    /// Max edge relaxations performed per query.
    pub max_edges: Option<usize>,
    /// Max paths returned per path query.
    pub max_paths: Option<usize>,
    /// Max neighbors expanded per single node visit.
    pub max_fanout: Option<usize>,
    /// Wall-clock budget in milliseconds.
    pub timeout_ms: Option<u64>,
}

/// Fold per-verb `--hops` plus budget flags into one [`quran_graph::QueryBudgets`].
fn graph_budgets(hops: usize, flags: &GraphBudgets) -> quran_graph::QueryBudgets {
    let defaults = quran_graph::QueryBudgets::default();
    quran_graph::QueryBudgets {
        max_hops: hops,
        max_nodes: flags.max_nodes.unwrap_or(defaults.max_nodes),
        max_edges: flags.max_edges.unwrap_or(defaults.max_edges),
        max_paths: flags.max_paths.unwrap_or(defaults.max_paths),
        max_fanout: flags.max_fanout.unwrap_or(defaults.max_fanout),
        timeout_ms: flags.timeout_ms.unwrap_or(defaults.timeout_ms),
    }
}

/// Path search mode selector (`--mode reachability|shortest|paths`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphPathMode {
    /// Two-node reachability check (min-hop proof, never a path render).
    Reachability,
    /// Shortest min-hop path between two nodes.
    Shortest,
    /// Up-to-K ranked paths between two nodes.
    Paths,
}

impl GraphPathMode {
    /// Parse the `--mode` selector. Unknown spellings are usage errors
    /// (validated before any I/O).
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "reachability" => Some(Self::Reachability),
            "shortest" => Some(Self::Shortest),
            "paths" => Some(Self::Paths),
            _ => None,
        }
    }

    /// Mode name for JSON output.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Reachability => "reachability",
            Self::Shortest => "shortest",
            Self::Paths => "paths",
        }
    }
}

/// Parse one `--step` (`EDGE[:Kind]`) into a typed pattern step. Unknown
/// predicates and malformed steps are [`quran_graph::GraphError::PatternRejected`],
/// never silent skips; the full pattern is re-validated before any I/O.
fn parse_pattern_step(raw: &str) -> Result<quran_graph::PatternStep, quran_graph::GraphError> {
    let reject = |detail: String| quran_graph::GraphError::PatternRejected { detail };
    let (edge, kind) = match raw.split_once(':') {
        Some((edge, kind)) => (edge, Some(kind)),
        None => (raw, None),
    };
    if edge.is_empty() {
        return Err(reject(format!("empty edge in pattern step `{raw}`")));
    }
    let node_kind = match kind {
        None => None,
        Some(name) => Some(match name.to_lowercase().as_str() {
            "edition" => quran_graph::NodeKind::Edition,
            "surah" => quran_graph::NodeKind::Surah,
            "ayah" => quran_graph::NodeKind::Ayah,
            "token" => quran_graph::NodeKind::Token,
            "division" => quran_graph::NodeKind::Division,
            "root" => quran_graph::NodeKind::Root,
            "lemma" => quran_graph::NodeKind::Lemma,
            "concept" => quran_graph::NodeKind::Concept,
            "entity" => quran_graph::NodeKind::Entity,
            "annotation" => quran_graph::NodeKind::Annotation,
            _ => {
                return Err(reject(format!(
                    "unknown node kind `{name}` in pattern step `{raw}`; \
                     use edition, surah, ayah, token, division, root, lemma, \
                     concept, entity, or annotation"
                )));
            }
        }),
    };
    Ok(quran_graph::PatternStep { edge: edge.to_string(), node_kind })
}

/// Append the truncation marker plus the verbatim reason to human output.
/// Truncation is never masked as an empty list: the marker plus a non-empty
/// reason always render together.
fn push_truncation(human: &mut String, truncated: bool, reason: &Option<String>) {
    if truncated {
        human.push_str(" (truncated)");
    }
    if let Some(detail) = reason {
        human.push_str(&format!("\nincomplete: {detail}"));
    }
}

/// `qai quran graph inspect`: manifest plus node/edge counts, from a
/// projection file or from the active SQLite projection (`--db`).
///
/// Exactly one of `--file` or `--db` is required; both absent or both
/// present is a usage error.
pub async fn cmd_graph_inspect(db_path: &str, file: Option<&str>, use_db: bool) -> CommandOutput {
    match (file, use_db) {
        (Some(path), false) => inspect_graph_file(path),
        (None, true) => inspect_graph_db(db_path).await,
        _ => CommandOutput::err(exit::USAGE, "specify exactly one of --file or --db".to_string()),
    }
}

/// File-backed inspect: the fixture/debug path (unchanged semantics).
fn inspect_graph_file(file: &str) -> CommandOutput {
    use quran_graph::GraphStore;
    let graph = match read_graph_file(file) {
        Ok(graph) => graph,
        Err(message) => return CommandOutput::err(exit::NOT_FOUND, message),
    };
    // Stage into the reference backend to exercise the port (zero dangling
    // edges validated by `stage_edges`).
    let store = match stage_graph(&graph) {
        Ok(store) => store,
        Err(error) => return CommandOutput::err(exit::VALIDATION, error),
    };
    let inspection = store.inspect();
    let capabilities = store.capabilities();
    CommandOutput::ok(
        format!(
            "projection {}: {} nodes, {} edges; staged {} nodes / {} edges; capabilities: {}",
            graph.manifest.projection_id,
            graph.nodes.len(),
            graph.edges.len(),
            inspection.node_count,
            inspection.edge_count,
            capabilities.join(", ")
        ),
        serde_json::json!({
            "manifest": graph.manifest,
            "nodes": graph.nodes.len(),
            "edges": graph.edges.len(),
            "staged": {"nodes": inspection.node_count, "edges": inspection.edge_count},
            "capabilities": capabilities,
        }),
    )
}

/// SQLite-backed inspect: the active projection's manifest (identity plus
/// the generation stamp read from the active edition row at build time).
async fn inspect_graph_db(db_path: &str) -> CommandOutput {
    use quran_graph::GraphStore;
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    let store = match super::quran_graph_store::SqliteGraphStore::open_active(
        &db,
        quran_graph::STRUCTURAL_PROJECTION_ID,
    )
    .await
    {
        Ok(store) => store,
        Err(error) => return CommandOutput::err(graph_error_exit(&error), error.to_string()),
    };
    let inspection = store.inspect();
    let capabilities = store.capabilities();
    let manifest = store.manifest();
    CommandOutput::ok(
        format!(
            "projection {}: {} nodes, {} edges (build {}, generation {}, status active); capabilities: {}",
            manifest.projection_id,
            inspection.node_count,
            inspection.edge_count,
            store.projection_row_id(),
            manifest.corpus_generation,
            capabilities.join(", ")
        ),
        serde_json::json!({
            "manifest": manifest,
            "nodes": inspection.node_count,
            "edges": inspection.edge_count,
            "build_row_id": store.projection_row_id(),
            "capabilities": capabilities,
        }),
    )
}

/// `qai quran graph neighbors` (budgeted, explicit truncation), from a
/// projection file or from the active SQLite projection (`--db`, with every
/// ayah hit resolved to a pinned canonical reference through the reader).
///
/// Exactly one of `--file` or `--db` is required; both absent or both
/// present is a usage error.
pub async fn cmd_graph_neighbors(
    db_path: &str,
    file: Option<&str>,
    use_db: bool,
    node: &str,
    hops: usize,
    budgets: &GraphBudgets,
) -> CommandOutput {
    match (file, use_db) {
        (Some(path), false) => neighbors_graph_file(path, node, hops, budgets),
        (None, true) => neighbors_graph_db(db_path, node, hops, budgets).await,
        _ => CommandOutput::err(exit::USAGE, "specify exactly one of --file or --db".to_string()),
    }
}

/// File-backed neighbors: the fixture/debug path (unchanged semantics, typed
/// error mapping).
fn neighbors_graph_file(
    file: &str,
    node: &str,
    hops: usize,
    budgets: &GraphBudgets,
) -> CommandOutput {
    use quran_graph::{AuthzScope, EdgeFilter, GraphStore};
    let graph = match read_graph_file(file) {
        Ok(graph) => graph,
        Err(message) => return CommandOutput::err(exit::NOT_FOUND, message),
    };
    let store = match stage_graph(&graph) {
        Ok(store) => store,
        Err(error) => return CommandOutput::err(exit::VALIDATION, error),
    };
    let qb = graph_budgets(hops, budgets);
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let scope = AuthzScope::all_visible();
    match store.neighbors(node, &EdgeFilter::any(), &qb, &cancel, &scope) {
        Ok(result) => {
            let mut human = format!(
                "neighbors of {node}: {} node(s), {} edge(s)",
                result.nodes.len(),
                result.edges.len(),
            );
            push_truncation(&mut human, result.truncated, &result.incomplete_reason);
            json_or_err(
                human,
                &serde_json::json!({
                    "nodes": result.nodes,
                    "edges": result.edges,
                    "truncated": result.truncated,
                    "incomplete_reason": result.incomplete_reason,
                }),
            )
        }
        Err(error) => graph_error_output(&error),
    }
}

/// SQLite-backed neighbors over the active structural projection.
///
/// Every ayah-kind hit resolves through the canonical reader to a pinned
/// reference (`quran:<slug>@<version>:<surah>:<ayah>`) plus its stored text
/// hash. Graph records carry refs and hashes only; display text never flows
/// from graph rows. A hit that no longer resolves fails closed (never a
/// fabricated quotation).
async fn neighbors_graph_db(
    db_path: &str,
    node: &str,
    hops: usize,
    budgets: &GraphBudgets,
) -> CommandOutput {
    use super::quran_graph_api::{GraphApiService, GraphBackend as _, NeighborsArgs, ReadOptions};
    use storage::Database as _;
    let service = match GraphApiService::open(db_path).await {
        Ok(service) => service,
        Err(error) => return graph_error_output(&error),
    };
    let output = match service
        .neighbors(NeighborsArgs {
            node: node.to_string(),
            options: ReadOptions {
                budgets: graph_budgets(hops, budgets),
                ..ReadOptions::default()
            },
        })
        .await
    {
        Ok(output) => output,
        Err(error) => return graph_api_output(&error),
    };
    let manifest = service.manifest().clone();
    let build_row_id = manifest.id.clone();

    // Staleness is advisory, never a failure: the manifest pins the
    // generation the projection was built from.
    let db = service.database();
    let mut uow = match db.write().await {
        Ok(uow) => uow,
        Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
    };
    let active_generation = uow
        .quran()
        .get_active()
        .await
        .map(|active| active.map(|row| row.corpus_generation))
        .unwrap_or(None);
    let _ = uow.rollback().await;
    let stale = active_generation
        .is_some_and(|generation| manifest.corpus_generation != generation.max(0) as u64);

    let reader = super::quran_reader::QuranReaderService::new(db.clone());
    let mut quotations = Vec::new();
    for hit in &output.nodes {
        if hit.kind != quran_graph::NodeKind::Ayah {
            continue;
        }
        let Some((surah, ayah)) = parse_ayah_stable_id(&hit.stable_id) else {
            return CommandOutput::err(
                exit::INTERNAL,
                format!("unparseable ayah stable ID '{}'", hit.stable_id),
            );
        };
        let reference = quran_core::QuranRef::Ayah {
            edition: quran_core::EditionSelector::Active,
            surah: match quran_core::SurahNumber::new(surah) {
                Ok(number) => number,
                Err(_) => {
                    return CommandOutput::err(
                        exit::INTERNAL,
                        format!("unparseable ayah stable ID '{}'", hit.stable_id),
                    );
                }
            },
            ayah: match quran_core::AyahNumber::new(ayah) {
                Ok(number) => number,
                Err(_) => {
                    return CommandOutput::err(
                        exit::INTERNAL,
                        format!("unparseable ayah stable ID '{}'", hit.stable_id),
                    );
                }
            },
        };
        use super::quran_reader::QuranReader as _;
        let view = match reader.get_ayah(&reference, &quran_core::AyahOptions::default()).await {
            Ok(view) => view,
            Err(error) => {
                return CommandOutput::err(
                    exit::INTERNAL,
                    format!("quotation unavailable for '{}': {error}", hit.stable_id),
                );
            }
        };
        quotations.push(serde_json::json!({
            "node": hit.stable_id,
            "reference": view.canonical.reference(),
            "text_hash": view.canonical.text_hash().hex,
        }));
    }

    let mut human = format!(
        "neighbors of {node}: {} node(s), {} edge(s){}",
        output.nodes.len(),
        output.edges.len(),
        if stale { " (stale projection)" } else { "" }
    );
    push_truncation(&mut human, output.truncated, &output.incomplete_reason);
    for quotation in &quotations {
        human.push_str(&format!(
            "\n  {} -> {}",
            quotation["node"].as_str().unwrap_or("?"),
            quotation["reference"].as_str().unwrap_or("?")
        ));
    }
    json_or_err(
        human,
        &serde_json::json!({
            "nodes": output.nodes,
            "edges": output.edges,
            "truncated": output.truncated,
            "incomplete_reason": output.incomplete_reason,
            "explanation": output.explanation,
            "quotations": quotations,
            "manifest": {
                "projection_id": manifest.projection_id,
                "build_row_id": build_row_id,
                "corpus_generation": manifest.corpus_generation,
            },
            "stale": stale,
        }),
    )
}

/// Parse an `ayah:<surah>:<ayah>` stable ID into canonical numbers.
fn parse_ayah_stable_id(stable_id: &str) -> Option<(u16, u32)> {
    let rest = stable_id.strip_prefix("ayah:")?;
    let mut parts = rest.split(':');
    let surah: u16 = parts.next()?.parse().ok()?;
    let ayah: u32 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((surah, ayah))
}

/// `qai quran graph path`: reachability, shortest, or up-to-K paths between
/// two nodes (budgeted, deterministic), from a projection file or from the
/// active SQLite projection (`--db`).
///
/// Exactly one of `--file` or `--db` is required; both absent or both
/// present is a usage error.
#[allow(clippy::too_many_arguments)]
pub async fn cmd_graph_path(
    db_path: &str,
    file: Option<&str>,
    use_db: bool,
    from: &str,
    to: &str,
    hops: usize,
    mode: &str,
    k: Option<usize>,
    budgets: &GraphBudgets,
) -> CommandOutput {
    let mode = match GraphPathMode::parse(mode) {
        Some(mode) => mode,
        None => {
            return CommandOutput::err(
                exit::USAGE,
                format!("unknown path mode `{mode}`; use reachability, shortest, or paths"),
            );
        }
    };
    match (file, use_db) {
        (Some(path), false) => path_file(path, from, to, hops, mode, k, budgets),
        (None, true) => path_db(db_path, from, to, hops, mode, k, budgets).await,
        _ => CommandOutput::err(exit::USAGE, "specify exactly one of --file or --db".to_string()),
    }
}

/// SQLite-backed path search through the read service.
async fn path_db(
    db_path: &str,
    from: &str,
    to: &str,
    hops: usize,
    mode: GraphPathMode,
    k: Option<usize>,
    budgets: &GraphBudgets,
) -> CommandOutput {
    use super::quran_graph_api::{GraphApiService, GraphBackend as _, PathArgs, PathsArgs};
    let service = match GraphApiService::open(db_path).await {
        Ok(service) => service,
        Err(error) => return graph_error_output(&error),
    };
    let qb = graph_budgets(hops, budgets);
    let options = || super::quran_graph_api::ReadOptions {
        budgets: qb.clone(),
        ..super::quran_graph_api::ReadOptions::default()
    };
    match mode {
        GraphPathMode::Reachability => match service
            .reachability(PathArgs {
                from: from.to_string(),
                to: to.to_string(),
                options: options(),
            })
            .await
        {
            Ok(output) => render_reachability(from, to, &output),
            Err(error) => graph_api_output(&error),
        },
        GraphPathMode::Shortest => match service
            .shortest_path(PathArgs {
                from: from.to_string(),
                to: to.to_string(),
                options: options(),
            })
            .await
        {
            Ok(output) => render_shortest(from, to, &output),
            Err(error) => graph_api_output(&error),
        },
        GraphPathMode::Paths => {
            let k = k.unwrap_or(qb.max_paths);
            match service
                .paths(PathsArgs {
                    from: from.to_string(),
                    to: to.to_string(),
                    k,
                    options: options(),
                })
                .await
            {
                Ok(output) => render_paths(from, to, k, &output),
                Err(error) => graph_api_output(&error),
            }
        }
    }
}

/// File-backed path search: the fixture/debug path over the staged
/// reference backend with the same mode shapes as `--db`.
fn path_file(
    file: &str,
    from: &str,
    to: &str,
    hops: usize,
    mode: GraphPathMode,
    k: Option<usize>,
    budgets: &GraphBudgets,
) -> CommandOutput {
    use quran_graph::AuthzScope;
    let graph = match read_graph_file(file) {
        Ok(graph) => graph,
        Err(message) => return CommandOutput::err(exit::NOT_FOUND, message),
    };
    let store = match stage_graph(&graph) {
        Ok(store) => store,
        Err(error) => return CommandOutput::err(exit::VALIDATION, error),
    };
    let qb = graph_budgets(hops, budgets);
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let scope = AuthzScope::all_visible();
    match mode {
        GraphPathMode::Reachability => {
            match quran_graph::traverse::min_hops(&store, from, to, &qb, &cancel, &scope) {
                Ok(outcome) => {
                    let (reachable, hops) = if outcome.truncated {
                        (None, None)
                    } else {
                        (Some(outcome.hops.is_some()), outcome.hops)
                    };
                    let output = super::quran_graph_api::ReachabilityOutput {
                        reachable,
                        hops,
                        explanation: explain_file_read(
                            &graph.manifest,
                            &store,
                            Some(from.to_string()),
                            Some(to.to_string()),
                            &[],
                            &[],
                            &[],
                            &qb,
                            outcome.truncated,
                            outcome.incomplete_reason.clone(),
                        ),
                    };
                    render_reachability(from, to, &output)
                }
                Err(error) => graph_error_output(&error),
            }
        }
        GraphPathMode::Shortest | GraphPathMode::Paths => {
            let k = if mode == GraphPathMode::Shortest { 1 } else { k.unwrap_or(qb.max_paths) };
            if let Err(error) = super::quran_graph_api::validate_k(k, &qb) {
                return graph_error_output(&error);
            }
            match quran_graph::traverse::up_to_k_paths(&store, from, to, k, &qb, &cancel, &scope) {
                Ok(result) => {
                    let node_ids: Vec<String> = result
                        .paths
                        .first()
                        .map(|found| found.node_ids.clone())
                        .unwrap_or_default();
                    let edges: Vec<quran_graph::GraphEdge> =
                        result.paths.first().map(|found| found.edges.clone()).unwrap_or_default();
                    if mode == GraphPathMode::Shortest {
                        let output = super::quran_graph_api::ShortestOutput {
                            path: result.paths.first().cloned(),
                            explanation: explain_file_read(
                                &graph.manifest,
                                &store,
                                Some(from.to_string()),
                                Some(to.to_string()),
                                &node_ids,
                                &edges,
                                &result.paths,
                                &qb,
                                result.truncated,
                                result.incomplete_reason.clone(),
                            ),
                        };
                        render_shortest(from, to, &output)
                    } else {
                        let output = super::quran_graph_api::PathsOutput {
                            paths: result.paths.clone(),
                            truncated: result.truncated,
                            incomplete_reason: result.incomplete_reason.clone(),
                            explanation: explain_file_read(
                                &graph.manifest,
                                &store,
                                Some(from.to_string()),
                                Some(to.to_string()),
                                &[],
                                &[],
                                &result.paths,
                                &qb,
                                result.truncated,
                                result.incomplete_reason.clone(),
                            ),
                        };
                        render_paths(from, to, k, &output)
                    }
                }
                Err(error) => graph_error_output(&error),
            }
        }
    }
}

/// Explain a file-backed read with the same payload contract as the
/// service: snapshot from the file manifest, per-edge provenance against
/// the staged store (no authority table, so asserted edges report the
/// gap), and the effective direction-agnostic filter for path modes.
#[allow(clippy::too_many_arguments)]
fn explain_file_read(
    manifest: &quran_graph::ProjectionManifest,
    store: &quran_graph::MemGraphStore,
    start: Option<String>,
    end: Option<String>,
    nodes: &[String],
    edges: &[quran_graph::GraphEdge],
    paths: &[quran_graph::GraphPath],
    budgets: &quran_graph::QueryBudgets,
    truncated: bool,
    incomplete_reason: Option<String>,
) -> super::quran_graph_api::Explanation {
    use super::quran_graph_api as api;
    let lookup = |edge: &quran_graph::GraphEdge| {
        edge.assertion_id.as_deref().and_then(|id| store.get_assertion(id))
    };
    api::Explanation {
        start,
        end,
        nodes: nodes.to_vec(),
        edges: edges.iter().map(|edge| api::explain_edge_with(edge, lookup(edge))).collect(),
        paths: paths
            .iter()
            .map(|path| api::PathExplanation {
                node_ids: path.node_ids.clone(),
                edges: path
                    .edges
                    .iter()
                    .map(|edge| api::explain_edge_with(edge, lookup(edge)))
                    .collect(),
            })
            .collect(),
        applied_filters: api::AppliedFilters {
            budgets: budgets.clone(),
            edge_types: None,
            direction: "both".to_string(),
            authz: api::describe_authz(&quran_graph::AuthzScope::all_visible()),
        },
        snapshot: api::SnapshotIdentity::from(manifest),
        truncated,
        incomplete_reason,
        duration_ms: 0,
    }
}

fn render_reachability(
    from: &str,
    to: &str,
    output: &super::quran_graph_api::ReachabilityOutput,
) -> CommandOutput {
    let status = match (output.reachable, output.explanation.truncated) {
        (Some(true), _) => format!("yes ({} hop(s))", output.hops.unwrap_or(0)),
        (Some(false), _) => "no (complete search)".to_string(),
        (None, _) => "unknown (truncated)".to_string(),
    };
    let mut human = format!("reachable {from} -> {to}: {status}");
    push_truncation(
        &mut human,
        output.explanation.truncated,
        &output.explanation.incomplete_reason,
    );
    json_or_err(
        human,
        &serde_json::json!({
            "mode": "reachability",
            "from": from,
            "to": to,
            "reachable": output.reachable,
            "hops": output.hops,
            "truncated": output.explanation.truncated,
            "incomplete_reason": output.explanation.incomplete_reason,
            "explanation": output.explanation,
        }),
    )
}

fn render_shortest(
    from: &str,
    to: &str,
    output: &super::quran_graph_api::ShortestOutput,
) -> CommandOutput {
    let status = match (&output.path, output.explanation.truncated) {
        (Some(path), _) => {
            format!("{} hop(s): {}", path.edges.len(), path.node_ids.join(" -> "))
        }
        (None, false) => "no path (complete search)".to_string(),
        (None, true) => "unknown (truncated)".to_string(),
    };
    let mut human = format!("shortest {from} -> {to}: {status}");
    push_truncation(
        &mut human,
        output.explanation.truncated,
        &output.explanation.incomplete_reason,
    );
    json_or_err(
        human,
        &serde_json::json!({
            "mode": "shortest",
            "from": from,
            "to": to,
            "path": output.path,
            "no_path_proven": output.no_path_proven(),
            "truncated": output.explanation.truncated,
            "incomplete_reason": output.explanation.incomplete_reason,
            "explanation": output.explanation,
        }),
    )
}

fn render_paths(
    from: &str,
    to: &str,
    k: usize,
    output: &super::quran_graph_api::PathsOutput,
) -> CommandOutput {
    let mut human = format!(
        "paths {from} -> {to}: {} of up to {k} ({} hop budget)",
        output.paths.len(),
        output.explanation.applied_filters.budgets.max_hops,
    );
    push_truncation(&mut human, output.truncated, &output.incomplete_reason);
    for (index, path) in output.paths.iter().enumerate() {
        human.push_str(&format!("\n  {}. {}", index + 1, path.node_ids.join(" -> ")));
    }
    json_or_err(
        human,
        &serde_json::json!({
            "mode": "paths",
            "from": from,
            "to": to,
            "k": k,
            "paths": output.paths,
            "no_path_proven": output.no_path_proven(),
            "truncated": output.truncated,
            "incomplete_reason": output.incomplete_reason,
            "explanation": output.explanation,
        }),
    )
}

/// `qai quran graph subgraph`: bounded multi-seed subgraph over the active
/// SQLite projection (unknown seeds are skipped; all-unknown is
/// complete-empty, never an error).
pub async fn cmd_graph_subgraph(
    db_path: &str,
    seeds: &[String],
    hops: usize,
    budgets: &GraphBudgets,
) -> CommandOutput {
    use super::quran_graph_api::{GraphApiService, GraphBackend as _, ReadOptions, SubgraphArgs};
    if seeds.is_empty() {
        return CommandOutput::err(exit::USAGE, "provide at least one --seed".to_string());
    }
    let service = match GraphApiService::open(db_path).await {
        Ok(service) => service,
        Err(error) => return graph_error_output(&error),
    };
    let output = match service
        .subgraph(SubgraphArgs {
            seeds: seeds.to_vec(),
            options: ReadOptions {
                budgets: graph_budgets(hops, budgets),
                ..ReadOptions::default()
            },
        })
        .await
    {
        Ok(output) => output,
        Err(error) => return graph_api_output(&error),
    };
    let mut human = format!(
        "subgraph of {} seed(s): {} node(s), {} edge(s)",
        seeds.len(),
        output.nodes.len(),
        output.edges.len(),
    );
    push_truncation(&mut human, output.truncated, &output.incomplete_reason);
    json_or_err(
        human,
        &serde_json::json!({
            "seeds": seeds,
            "nodes": output.nodes,
            "edges": output.edges,
            "truncated": output.truncated,
            "incomplete_reason": output.incomplete_reason,
            "explanation": output.explanation,
        }),
    )
}

/// `qai quran graph pattern`: typed pattern query from seeds over the
/// active SQLite projection. Steps are `EDGE[:Kind]`; unknown predicates
/// and malformed steps are validation failures, never silent skips.
pub async fn cmd_graph_pattern(
    db_path: &str,
    seeds: &[String],
    steps: &[String],
    hops: usize,
    budgets: &GraphBudgets,
) -> CommandOutput {
    use super::quran_graph_api::{GraphApiService, GraphBackend as _, PatternArgs, ReadOptions};
    if seeds.is_empty() {
        return CommandOutput::err(exit::USAGE, "provide at least one --seed".to_string());
    }
    if steps.is_empty() {
        return CommandOutput::err(exit::USAGE, "provide at least one --step".to_string());
    }
    let mut typed = Vec::with_capacity(steps.len());
    for raw in steps {
        match parse_pattern_step(raw) {
            Ok(step) => typed.push(step),
            Err(error) => return graph_error_output(&error),
        }
    }
    let pattern = quran_graph::Pattern::new(typed);
    if let Err(error) = quran_graph::validate_pattern(&pattern) {
        return graph_error_output(&error);
    }
    let service = match GraphApiService::open(db_path).await {
        Ok(service) => service,
        Err(error) => return graph_error_output(&error),
    };
    let output = match service
        .pattern(PatternArgs {
            pattern,
            seeds: seeds.to_vec(),
            options: ReadOptions {
                budgets: graph_budgets(hops, budgets),
                ..ReadOptions::default()
            },
        })
        .await
    {
        Ok(output) => output,
        Err(error) => return graph_api_output(&error),
    };
    let mut human = format!(
        "pattern [{}] from {} seed(s): {} node(s), {} edge(s)",
        steps.join(", "),
        seeds.len(),
        output.nodes.len(),
        output.edges.len(),
    );
    push_truncation(&mut human, output.truncated, &output.incomplete_reason);
    json_or_err(
        human,
        &serde_json::json!({
            "steps": steps,
            "seeds": seeds,
            "nodes": output.nodes,
            "edges": output.edges,
            "truncated": output.truncated,
            "incomplete_reason": output.incomplete_reason,
            "explanation": output.explanation,
        }),
    )
}

/// `qai quran graph root-family`: lexicon-gated ranked ayahs (no active
/// dataset is the typed `QAI-MORPH-0004` unavailable error, never an empty
/// family). Ranking is shared with the read service, so both surfaces agree.
pub async fn cmd_graph_root_family(db_path: &str, root: &str, limit: usize) -> CommandOutput {
    use super::quran_morphology::root_search;
    let start = std::time::Instant::now();
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    match root_search(&db, root).await {
        Ok((dataset, occurrences)) => {
            let (ayahs, cap) = super::quran_graph_api::rank_family_ayahs(occurrences, limit);
            let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
            json_or_err(
                format!("root family {root}: {} ranked ayah(s) in {dataset}", ayahs.len()),
                &serde_json::json!({
                    "dataset": dataset,
                    "ayahs": ayahs,
                    "limit": cap,
                    "duration_ms": duration_ms,
                }),
            )
        }
        Err(error) => {
            use storage::error::Diagnostic as _;
            CommandOutput::err(tool_exit(&error), format!("[{}] {error}", error.code()))
        }
    }
}

/// `qai quran graph export`: Graph JSON with identities + truncation flags,
/// from a projection file or from the active SQLite projection (`--db`,
/// policy-filtered so only effective assertions for kept edges ship).
///
/// Exactly one of `--file` or `--db` is required; both absent or both
/// present is a usage error.
pub async fn cmd_graph_export(
    db_path: &str,
    file: Option<&str>,
    use_db: bool,
    out: Option<&str>,
) -> CommandOutput {
    match (file, use_db) {
        (Some(path), false) => export_graph_file(path, out),
        (None, true) => export_graph_db(db_path, out).await,
        _ => CommandOutput::err(exit::USAGE, "specify exactly one of --file or --db".to_string()),
    }
}

/// File-backed export: the fixture/debug path (unchanged semantics).
fn export_graph_file(file: &str, out: Option<&str>) -> CommandOutput {
    use quran_graph::export_json;
    let graph = match read_graph_file(file) {
        Ok(graph) => graph,
        Err(message) => return CommandOutput::err(exit::NOT_FOUND, message),
    };
    let json = export_json(&graph.nodes, &graph.edges, &[], &graph.manifest);
    match out {
        Some(path) => {
            let text = serde_json::to_string_pretty(&json).unwrap_or_default();
            if let Err(error) = std::fs::write(path, text) {
                return CommandOutput::err(exit::INTERNAL, format!("write {path}: {error}"));
            }
            CommandOutput::ok(format!("exported {} nodes to {path}", graph.nodes.len()), json)
        }
        None => {
            CommandOutput::ok(format!("exported {} nodes as Graph JSON", graph.nodes.len()), json)
        }
    }
}

/// SQLite-backed export over the active structural projection: the full
/// pinned sets through the pre-serialization policy filter, serialized
/// with the complete notice (bounded reads with truncation travel through
/// subgraph/pattern exports in plan 04-05 surfaces).
async fn export_graph_db(db_path: &str, out: Option<&str>) -> CommandOutput {
    use super::quran_graph_annotations::visible_export_sets;
    use quran_graph::{ExportNotice, export_json_with_notice};
    let service = match super::quran_graph_api::GraphApiService::open(db_path).await {
        Ok(service) => service,
        Err(error) => return graph_error_output(&error),
    };
    let (nodes, edges, assertions) = service.export_sets();
    let (nodes, edges, traveling) = visible_export_sets(&nodes, &edges, &assertions);
    let manifest = service.manifest().clone();
    let json =
        export_json_with_notice(&nodes, &edges, &traveling, &manifest, &ExportNotice::complete());
    match out {
        Some(path) => {
            let text = serde_json::to_string_pretty(&json).unwrap_or_default();
            if let Err(error) = std::fs::write(path, text) {
                return CommandOutput::err(exit::INTERNAL, format!("write {path}: {error}"));
            }
            CommandOutput::ok(
                format!(
                    "exported {} nodes, {} edges, {} assertions to {path} (projection {})",
                    nodes.len(),
                    edges.len(),
                    traveling.len(),
                    manifest.projection_id,
                ),
                json,
            )
        }
        None => CommandOutput::ok(
            format!(
                "exported {} nodes, {} edges, {} assertions as Graph JSON (projection {})",
                nodes.len(),
                edges.len(),
                traveling.len(),
                manifest.projection_id,
            ),
            json,
        ),
    }
}

/// Options for `qai quran graph review propose` (thin wrapper: parsing only,
/// the annotation service owns validation and writes).
#[derive(Debug, Clone)]
pub struct ReviewProposeOptions {
    /// Caller assertion ID (generated when absent).
    pub id: Option<String>,
    /// Assertion family spelling.
    pub kind: String,
    /// Edge source stable ID.
    pub src: String,
    /// Allowlisted edge predicate.
    pub edge: String,
    /// Edge destination stable ID.
    pub dst: String,
    /// Evidence JSON object (default `{}`).
    pub evidence: Option<String>,
    /// PRD 10.3 source ID.
    pub source_id: String,
    /// PRD 10.3 source location.
    pub source_location: String,
    /// Creating human (never invented).
    pub author: String,
    /// Projection family (default `quran-structural-v1`).
    pub projection: Option<String>,
    /// Edition scope (default: active edition).
    pub edition: Option<String>,
    /// Dataset scope (default: unscoped).
    pub scope: Option<String>,
}

/// Options for `qai quran graph review suggest` (layer D; algorithm triple
/// required).
#[derive(Debug, Clone)]
pub struct ReviewSuggestOptions {
    /// Caller assertion ID (generated when absent).
    pub id: Option<String>,
    /// Assertion family spelling.
    pub kind: String,
    /// Edge source stable ID.
    pub src: String,
    /// Allowlisted edge predicate.
    pub edge: String,
    /// Edge destination stable ID.
    pub dst: String,
    /// Evidence JSON object shown in the review queue.
    pub evidence: Option<String>,
    /// PRD 10.3 source ID.
    pub source_id: String,
    /// PRD 10.3 source location.
    pub source_location: String,
    /// Producing algorithm (never invented).
    pub algorithm: String,
    /// Algorithm version.
    pub algorithm_version: String,
    /// Attributed confidence in [0,1].
    pub confidence: f64,
    /// Projection family (default `quran-structural-v1`).
    pub projection: Option<String>,
    /// Edition scope (default: active edition).
    pub edition: Option<String>,
    /// Dataset scope (default: unscoped).
    pub scope: Option<String>,
}

/// Options for `qai quran graph review correct` (unset claim fields default
/// to the old row's values).
#[derive(Debug, Clone)]
pub struct ReviewCorrectOptions {
    /// Assertion ID being corrected.
    pub id: String,
    /// Correcting reviewer (never invented).
    pub reviewer: String,
    /// Decision timestamp (default: now).
    pub decided_at: Option<String>,
    /// Corrected source.
    pub src: Option<String>,
    /// Corrected predicate.
    pub edge: Option<String>,
    /// Corrected destination.
    pub dst: Option<String>,
    /// Corrected evidence JSON object.
    pub evidence: Option<String>,
    /// Corrected source location.
    pub source_location: Option<String>,
}

fn review_projection(raw: Option<String>) -> String {
    raw.filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| quran_graph::STRUCTURAL_PROJECTION_ID.to_string())
}

fn review_scope(raw: Option<String>) -> String {
    raw.unwrap_or_default()
}

async fn review_edition_id(
    db: &SqliteDatabase,
    raw: Option<String>,
) -> Result<String, CommandOutput> {
    if let Some(edition) = raw.filter(|value| !value.trim().is_empty()) {
        return Ok(edition);
    }
    let mut uow = match db.write().await {
        Ok(uow) => uow,
        Err(error) => return Err(CommandOutput::err(exit::INTERNAL, error.to_string())),
    };
    let active = match uow.quran().get_active().await {
        Ok(active) => active,
        Err(error) => return Err(CommandOutput::err(exit::INTERNAL, error.to_string())),
    };
    let _ = uow.rollback().await;
    active.map(|row| row.edition_id).ok_or_else(|| {
        CommandOutput::err(exit::NOT_FOUND, "no active edition; import one first".to_string())
    })
}

fn review_evidence(raw: Option<String>) -> Result<serde_json::Value, CommandOutput> {
    match raw {
        None => Ok(serde_json::json!({})),
        Some(text) => serde_json::from_str(&text).map_err(|error| {
            CommandOutput::err(exit::USAGE, format!("bad evidence JSON: {error}"))
        }),
    }
}

fn review_report(
    verb: &str,
    report: super::quran_graph_annotations::AnnotationReport,
) -> CommandOutput {
    use super::quran_graph_annotations::AnnotationReport;
    let AnnotationReport { assertion, edge_staged, build_row_id, reused } = report;
    // Pending rows have no reviewer yet: attribute the author (propose) or
    // algorithm (suggest) from the seven-field claim provenance instead.
    let actor = assertion.reviewer.as_deref().unwrap_or_else(|| {
        assertion
            .claim
            .get("provenance")
            .and_then(|provenance| provenance.get("author_or_algorithm"))
            .and_then(|value| value.as_str())
            .unwrap_or("unreviewed")
    });
    let human = format!(
        "{verb} {} ({}) by {} at {}{}",
        assertion.id,
        match assertion.decision {
            quran_graph::AssertionDecision::Pending => "pending",
            quran_graph::AssertionDecision::Accepted => "accepted",
            quran_graph::AssertionDecision::Rejected => "rejected",
            quran_graph::AssertionDecision::Superseded => "superseded",
            quran_graph::AssertionDecision::Disputed => "disputed",
        },
        assertion.reviewer.as_deref().unwrap_or(actor),
        assertion.decided_at.as_deref().unwrap_or("undecided"),
        if reused { " (reused)" } else { "" },
    );
    json_or_err(
        human,
        &serde_json::json!({
            "assertion": assertion,
            "edge_staged": edge_staged,
            "build_row_id": build_row_id,
            "reused": reused,
        }),
    )
}

/// `qai quran graph review propose`: thin wrapper over the annotation
/// service (no business logic in the CLI crate).
pub async fn cmd_graph_review_propose(
    db_path: &str,
    options: ReviewProposeOptions,
) -> CommandOutput {
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    let edition = match review_edition_id(&db, options.edition).await {
        Ok(edition) => edition,
        Err(output) => return output,
    };
    let evidence = match review_evidence(options.evidence) {
        Ok(evidence) => evidence,
        Err(output) => return output,
    };
    let kind = match super::quran_graph_annotations::parse_assertion_kind(&options.kind) {
        Ok(kind) => kind,
        Err(error) => return CommandOutput::err(graph_error_exit(&error), error.to_string()),
    };
    match super::quran_graph_annotations::propose(
        db_path,
        super::quran_graph_annotations::ProposeInput {
            id: options.id,
            kind,
            src: options.src,
            edge: options.edge,
            dst: options.dst,
            evidence,
            source_id: options.source_id,
            source_location: options.source_location,
            author: options.author,
            invoked_by: LOCAL_PRINCIPAL.to_string(),
            projection_id: review_projection(options.projection),
            edition_id: edition,
            dataset_scope: review_scope(options.scope),
        },
    )
    .await
    {
        Ok(report) => review_report("proposed", report),
        Err(error) => CommandOutput::err(graph_error_exit(&error), error.to_string()),
    }
}

/// `qai quran graph review suggest`: thin wrapper over the annotation
/// service (no business logic in the CLI crate).
pub async fn cmd_graph_review_suggest(
    db_path: &str,
    options: ReviewSuggestOptions,
) -> CommandOutput {
    let db = match open_db(db_path).await {
        Ok(db) => db,
        Err(error) => return open_error_output(error),
    };
    let edition = match review_edition_id(&db, options.edition).await {
        Ok(edition) => edition,
        Err(output) => return output,
    };
    let evidence = match review_evidence(options.evidence) {
        Ok(evidence) => evidence,
        Err(output) => return output,
    };
    let kind = match super::quran_graph_annotations::parse_assertion_kind(&options.kind) {
        Ok(kind) => kind,
        Err(error) => return CommandOutput::err(graph_error_exit(&error), error.to_string()),
    };
    match super::quran_graph_annotations::suggest(
        db_path,
        super::quran_graph_annotations::SuggestInput {
            id: options.id,
            kind,
            src: options.src,
            edge: options.edge,
            dst: options.dst,
            evidence,
            source_id: options.source_id,
            source_location: options.source_location,
            algorithm: options.algorithm,
            algorithm_version: options.algorithm_version,
            confidence: options.confidence,
            invoked_by: LOCAL_PRINCIPAL.to_string(),
            projection_id: review_projection(options.projection),
            edition_id: edition,
            dataset_scope: review_scope(options.scope),
        },
    )
    .await
    {
        Ok(report) => review_report("suggested", report),
        Err(error) => CommandOutput::err(graph_error_exit(&error), error.to_string()),
    }
}

/// `qai quran graph review accept|reject`: thin wrapper over the annotation
/// service (no business logic in the CLI crate).
pub async fn cmd_graph_review_decide(
    db_path: &str,
    verb: &str,
    id: &str,
    reviewer: &str,
    decided_at: Option<&str>,
) -> CommandOutput {
    let input = super::quran_graph_annotations::DecideInput {
        id: id.to_string(),
        reviewer: reviewer.to_string(),
        decided_at: decided_at.map(str::to_string),
        invoked_by: LOCAL_PRINCIPAL.to_string(),
    };
    let result = match verb {
        "accept" => super::quran_graph_annotations::accept(db_path, input).await,
        _ => super::quran_graph_annotations::reject(db_path, input).await,
    };
    match result {
        Ok(report) => review_report(if verb == "accept" { "accepted" } else { "rejected" }, report),
        Err(error) => CommandOutput::err(graph_error_exit(&error), error.to_string()),
    }
}

/// `qai quran graph review correct`: thin wrapper over the annotation
/// service (no business logic in the CLI crate).
pub async fn cmd_graph_review_correct(
    db_path: &str,
    options: ReviewCorrectOptions,
) -> CommandOutput {
    let evidence = match options.evidence {
        None => None,
        Some(text) => match serde_json::from_str(&text) {
            Ok(evidence) => Some(evidence),
            Err(error) => {
                return CommandOutput::err(exit::USAGE, format!("bad evidence JSON: {error}"));
            }
        },
    };
    match super::quran_graph_annotations::correct(
        db_path,
        super::quran_graph_annotations::CorrectInput {
            id: options.id,
            reviewer: options.reviewer,
            decided_at: options.decided_at,
            src: options.src,
            edge: options.edge,
            dst: options.dst,
            evidence,
            source_location: options.source_location,
            invoked_by: LOCAL_PRINCIPAL.to_string(),
        },
    )
    .await
    {
        Ok(report) => {
            let supersedes = report.assertion.supersedes_id.clone().unwrap_or_default();
            let output = review_report("corrected", report);
            CommandOutput {
                exit: output.exit,
                human: format!("{} (supersedes {supersedes})", output.human),
                json: output.json,
            }
        }
        Err(error) => CommandOutput::err(graph_error_exit(&error), error.to_string()),
    }
}
/// Search mode requested on the CLI (one flag family per tool).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchCliMode {
    /// `quran.search_exact` (default unless another tool flag is given).
    Exact,
    /// `quran.search_normalized` (explicit `--profile`/`--rules` also selects this).
    Normalized,
    /// `quran.search_phrase` (`--phrase`).
    Phrase,
    /// `quran.search_concatenated` (`--concatenated`).
    Concatenated,
    /// `quran.search_regex` (`--regex`).
    Regex,
}

/// All `quran search` options in one struct (CLI parsing lives in `cli`).
#[derive(Debug, Clone)]
pub struct SearchCliOptions {
    /// Query text (or regex pattern with `--regex`).
    pub text: String,
    /// Tool selector.
    pub mode: SearchCliMode,
    /// Edition `slug@version` (defaults to the indexed edition).
    pub edition: Option<String>,
    /// Exact-search field (`text_exact`|`text_ws`); regex field with `--regex`.
    pub field: Option<String>,
    /// Token match mode (`whole_token`|`substring`|`ayah_prefix`).
    pub match_mode: String,
    /// Registry profile (`L3.diacritics`, optionally `@version`-pinned).
    pub profile: Option<String>,
    /// Explicit rule list (`N01,N03`); never with `profile`.
    pub rules: Option<String>,
    /// Phrase mode (`ordered_exact`|`ordered_near`|`unordered_near`).
    pub phrase_mode: String,
    /// Max intervening tokens for `*_near` phrase modes.
    pub slop: u32,
    /// Allow 3-ayah window matches (concatenated only).
    pub allow_cross_ayah: bool,
    /// Max ayahs per window match (concatenated only, `>= 1`).
    pub max_ayah_span: u32,
    /// Surah filter (`1,2,3`).
    pub surah: Option<String>,
    /// Juz filter (`2` or `1-5`).
    pub juz: Option<String>,
    /// Page filter (`3,4`).
    pub page: Option<String>,
    /// Revelation-place filter (`makki`|`madani`).
    pub revelation_place: Option<String>,
    /// Global ayah-index filter (`10-99`).
    pub global_range: Option<String>,
    /// Result cap (ceiling 1000).
    pub limit: u32,
    /// Result offset.
    pub offset: u32,
    /// Relevance order with per-hit BM25 breakdowns.
    pub explain: bool,
    /// Wrap hit spans in `<b>` display markers.
    pub highlight: bool,
    /// Regex wall-clock budget in ms (regex only, ceiling 10000).
    pub timeout_ms: u64,
}

fn map_search_error(error: super::quran_search::SearchError) -> (i32, String) {
    use super::quran_search::SearchError as E;
    let message = error.to_string();
    let exit = match &error {
        E::Normalization(inner) => {
            use quran_normalization::error::NormalizationError as NE;
            match inner {
                NE::UnknownRule { .. } | NE::UnknownProfile { .. } => exit::USAGE,
                _ => exit::INTERNAL,
            }
        }
        E::Index(inner) => {
            use quran_search::IndexError as IE;
            match inner {
                IE::QueryRejected { .. } => exit::USAGE,
                IE::RateLimited { .. } => exit::POLICY,
                _ => exit::INTERNAL,
            }
        }
        E::EditionNotIndexed { .. } | E::NoServingIndex { .. } => exit::NOT_FOUND,
        E::RateLimited(_) => exit::POLICY,
        E::Storage(_) => exit::INTERNAL,
    };
    (exit, message)
}

fn parse_id_list(raw: &str, what: &str) -> Result<Vec<u16>, CommandOutput> {
    raw.split(',')
        .map(|part| {
            part.trim().parse::<u16>().map_err(|_| {
                CommandOutput::err(exit::USAGE, format!("bad {what} list `{raw}`; use `1,2,3`"))
            })
        })
        .collect()
}

fn parse_u32_list(raw: &str, what: &str) -> Result<Vec<u32>, CommandOutput> {
    raw.split(',')
        .map(|part| {
            part.trim().parse::<u32>().map_err(|_| {
                CommandOutput::err(exit::USAGE, format!("bad {what} list `{raw}`; use `3,4`"))
            })
        })
        .collect()
}

fn parse_search_filters(
    opts: &SearchCliOptions,
) -> Result<Vec<quran_search::Filter>, CommandOutput> {
    let surah = opts.surah.as_deref().map(|raw| parse_id_list(raw, "surah")).transpose()?;
    let page = opts.page.as_deref().map(|raw| parse_u32_list(raw, "page")).transpose()?;
    super::quran_search_api::build_filters(
        surah,
        opts.juz.as_deref(),
        page,
        opts.revelation_place.as_deref(),
        opts.global_range.as_deref(),
    )
    .map_err(|err| CommandOutput::err(exit::USAGE, err.to_string()))
}

fn parse_search_match_mode(raw: &str) -> Result<super::quran_search::MatchMode, CommandOutput> {
    super::quran_search_api::parse_match_mode(Some(raw))
        .map_err(|err| CommandOutput::err(exit::USAGE, err.to_string()))
}

fn parse_normalized_profile(
    profile: Option<&str>,
    rules: Option<&str>,
) -> Result<super::quran_search::NormalizedProfile, CommandOutput> {
    super::quran_search_api::parse_normalized_profile(profile, rules)
        .map_err(|err| CommandOutput::err(exit::USAGE, err.to_string()))
}

/// `quran search` — all five lexical tools behind one command (P2-T52).
///
/// Human output is one block per hit (`reference — text`, spans and traces
/// with `--explain`); `--json` carries the full [`super::quran_search::SearchOutput`].
pub async fn cmd_search(db_path: &str, opts: &SearchCliOptions) -> CommandOutput {
    use super::quran_search::{ExactField, PhraseMode, SearchParams};
    use super::quran_search_api::{
        ConcatenatedArgs, ExactArgs, NormalizedArgs, PhraseArgs, RegexArgs,
    };

    if opts.text.trim().is_empty() {
        return CommandOutput::err(exit::USAGE, "provide query text".to_string());
    }
    let service = match super::quran_search_api::SearchApiService::open(db_path).await {
        Ok(service) => service,
        Err(message) => return CommandOutput::err(exit::INTERNAL, message),
    };
    let filters = match parse_search_filters(opts) {
        Ok(filters) => filters,
        Err(output) => return output,
    };
    let limit = opts.limit.clamp(1, 1000);
    let base = SearchParams {
        text: opts.text.clone(),
        edition: opts.edition.clone(),
        mode: match parse_search_match_mode(&opts.match_mode) {
            Ok(mode) => mode,
            Err(output) => return output,
        },
        filters,
        limit,
        offset: opts.offset,
        explain: opts.explain,
        highlight: opts.highlight,
    };
    let output = match opts.mode {
        SearchCliMode::Exact => {
            let field = match opts.field.as_deref().unwrap_or("text_exact") {
                "text_exact" => ExactField::TextExact,
                "text_ws" => ExactField::TextWs,
                other => {
                    return CommandOutput::err(
                        exit::USAGE,
                        format!("unknown exact field `{other}`; use `text_exact` or `text_ws`"),
                    );
                }
            };
            use super::quran_search_api::SearchBackend as _;
            service.search_exact(ExactArgs { params: base, field }).await
        }
        SearchCliMode::Normalized => {
            let profile =
                match parse_normalized_profile(opts.profile.as_deref(), opts.rules.as_deref()) {
                    Ok(profile) => profile,
                    Err(output) => return output,
                };
            use super::quran_search_api::SearchBackend as _;
            service.search_normalized(NormalizedArgs { params: base, profile }).await
        }
        SearchCliMode::Phrase => {
            let mode = match opts.phrase_mode.as_str() {
                "ordered_exact" => PhraseMode::OrderedExact,
                "ordered_near" => PhraseMode::OrderedNear,
                "unordered_near" => PhraseMode::UnorderedNear,
                other => {
                    return CommandOutput::err(
                        exit::USAGE,
                        format!(
                            "unknown phrase mode `{other}`; use `ordered_exact`, `ordered_near`, or `unordered_near`"
                        ),
                    );
                }
            };
            let profile =
                match parse_normalized_profile(opts.profile.as_deref(), opts.rules.as_deref()) {
                    Ok(profile) => profile,
                    Err(output) => return output,
                };
            use super::quran_search_api::SearchBackend as _;
            service.search_phrase(PhraseArgs { params: base, profile, mode, slop: opts.slop }).await
        }
        SearchCliMode::Concatenated => {
            use super::quran_search_api::SearchBackend as _;
            service
                .search_concatenated(ConcatenatedArgs {
                    params: base,
                    allow_cross_ayah: opts.allow_cross_ayah,
                    max_ayah_span: opts.max_ayah_span.max(1),
                })
                .await
        }
        SearchCliMode::Regex => {
            let field = opts.field.clone().unwrap_or_else(|| "text_bare".to_string());
            use super::quran_search_api::SearchBackend as _;
            service
                .search_regex(RegexArgs {
                    params: base,
                    field,
                    pattern: opts.text.clone(),
                    principal: "cli".to_string(),
                    timeout_ms: opts.timeout_ms.clamp(1, 10_000),
                })
                .await
        }
    };
    let output = match output {
        Ok(output) => output,
        Err(err) => {
            let (exit, message) = map_search_error(err);
            return CommandOutput::err(exit, message);
        }
    };
    let mut human = format!(
        "{} matches (total {}, {})\nrule set: {} · generation {}\n",
        output.hits.len(),
        output.total_matches,
        if output.truncated { "truncated" } else { "complete" },
        output.rule_set,
        output.generation
    );
    for hit in &output.hits {
        let value = serde_json::to_value(hit).unwrap_or_default();
        let reference = value.get("reference").and_then(|v| v.as_str()).unwrap_or("?");
        let text = value
            .get("quotation")
            .and_then(|v| v.get("arabic_text"))
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        human.push_str(&format!("{reference} — {text}\n"));
        if opts.explain {
            let rules = value
                .get("explanation")
                .and_then(|v| v.get("rules_applied"))
                .and_then(|v| v.as_array())
                .map(|rules| {
                    rules
                        .iter()
                        .filter_map(|r| r.get("rule").and_then(|v| v.as_str()))
                        .collect::<Vec<_>>()
                        .join(",")
                })
                .unwrap_or_default();
            let score =
                value.get("score").map(|v| v.to_string()).unwrap_or_else(|| "—".to_string());
            human.push_str(&format!("  rules: {rules} · score: {score}\n"));
        }
    }
    for warning in &output.warnings {
        human.push_str(&format!("warning [{}]: {}\n", warning.code, warning.message));
    }
    CommandOutput::ok(human, serde_json::to_value(&output).unwrap_or_default())
}

#[cfg(test)]
mod license_evidence_tests {
    use super::{derive_license_status, resolve_license_evidence, select_evidence_entry};

    const MATRIX: &str = r#"{
        "matrix_version": "1",
        "artifacts": {
            "synthetic-test-lexicon": {
                "artifact": "synthetic-test-lexicon",
                "source_url": "https://example.invalid/synthetic",
                "capture_date": "2026-09-28",
                "capturer": "qai-test-fixtures",
                "spdx_id": "CC0-1.0",
                "redistribution_allowed": true
            },
            "qac": {
                "artifact": "qac",
                "source_url": "https://corpus.quran.com/",
                "capture_date": null,
                "capturer": null,
                "spdx_id": null,
                "redistribution_allowed": false,
                "license_status": "pending_license_review"
            }
        }
    }"#;

    #[test]
    fn license_evidence_selects_matrix_entry_by_dataset_slug() {
        let value: serde_json::Value = serde_json::from_str(MATRIX).unwrap();
        let entry = select_evidence_entry(&value, "synthetic-test-lexicon").unwrap();
        assert_eq!(entry.get("artifact").and_then(|v| v.as_str()), Some("synthetic-test-lexicon"));
    }

    #[test]
    fn license_evidence_selects_sole_entry_when_slug_unknown() {
        let value: serde_json::Value =
            serde_json::from_str(r#"{"artifacts":{"only":{"source_url":"u","capture_date":"d","capturer":"c","redistribution_allowed":true}}}"#)
                .unwrap();
        let entry = select_evidence_entry(&value, "other").unwrap();
        assert_eq!(entry.get("capturer").and_then(|v| v.as_str()), Some("c"));
    }

    #[test]
    fn license_evidence_missing_file_is_typed_error() {
        let err = resolve_license_evidence(
            "synthetic-test-lexicon",
            None,
            None,
            Some("/nonexistent/qai-license-evidence-xyz.json"),
        )
        .unwrap_err();
        assert!(err.contains("read license evidence"), "{err}");
    }

    #[test]
    fn license_evidence_invalid_json_is_typed_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.json");
        std::fs::write(&path, "{ not json").unwrap();
        let err = resolve_license_evidence(
            "synthetic-test-lexicon",
            None,
            None,
            Some(path.to_str().unwrap()),
        )
        .unwrap_err();
        assert!(err.contains("not valid JSON"), "{err}");
    }

    #[test]
    fn license_evidence_derives_status_from_capture() {
        let with_spdx: serde_json::Value =
            serde_json::from_str(r#"{"spdx_id":"CC0-1.0","redistribution_allowed":true}"#).unwrap();
        assert_eq!(derive_license_status(&with_spdx), "OpenLicense");
        let granted: serde_json::Value =
            serde_json::from_str(r#"{"redistribution_allowed":true}"#).unwrap();
        assert_eq!(derive_license_status(&granted), "PermissionGranted");
        let unknown: serde_json::Value = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(derive_license_status(&unknown), "Unspecified");
    }

    #[test]
    fn license_evidence_absent_flag_keeps_placeholder_status() {
        let (status, json) =
            resolve_license_evidence("synthetic-test-lexicon", None, None, None).unwrap();
        assert_eq!(status, "Unspecified");
        assert_eq!(json, "{}");
    }
}
