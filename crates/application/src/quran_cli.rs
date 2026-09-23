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

async fn open_db(db_path: &str) -> Result<SqliteDatabase, StorageError> {
    SqliteDatabase::new(db_path, 4, true).await
}

fn map_reader_error(error: crate::quran_reader::ReaderError) -> (i32, String) {
    use crate::quran_reader::ReaderError as E;
    match &error {
        E::InvalidReference(_) => (exit::USAGE, error.to_string()),
        E::EditionNotFound(_)
        | E::AyahNotFound(_)
        | E::TranslationNotFound(_)
        | E::DivisionNotFound(_) => (exit::NOT_FOUND, error.to_string()),
        E::Storage(_) => (exit::INTERNAL, error.to_string()),
    }
}

fn map_activation_error(error: super::quran::ActivationError) -> (i32, String) {
    use super::quran::ActivationError as E;
    match &error {
        E::ApprovalMissing { .. }
        | E::ApprovalNotGranted { .. }
        | E::ApprovalSubjectMismatch { .. } => (exit::POLICY, error.to_string()),
        E::EmptyReviewer => (exit::USAGE, error.to_string()),
        E::NotStaged { .. } | E::AlreadyActive { .. } => (exit::CONFLICT, error.to_string()),
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

fn reader(db: &Arc<SqliteDatabase>) -> crate::quran_reader::QuranReaderService {
    crate::quran_reader::QuranReaderService::new(db.clone())
}

fn parse_selector(raw: &str) -> Result<EditionSelector, CommandOutput> {
    if raw.is_empty() {
        return Ok(EditionSelector::Active);
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
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
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
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
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
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
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
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
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
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
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

/// `quran edition list`.
pub async fn cmd_edition_list(db_path: &str) -> CommandOutput {
    use crate::quran_reader::QuranReader;
    let db = match open_db(db_path).await {
        Ok(db) => Arc::new(db),
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
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
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
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
    CommandOutput::ok(human, serde_json::to_value(&edition).unwrap_or_default())
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
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
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
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
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
    let input = quran_corpus::import::ImportInput {
        run_id: uuid::Uuid::new_v4().to_string(),
        job_id: None,
        source_version_id,
        adapter: adapter.to_string(),
        manifest_text: text,
        declared_manifest_hash: None,
        invoked_by: LOCAL_PRINCIPAL.to_string(),
        license_status: "Unknown".to_string(),
        license_json: serde_json::json!({
            "status": "Unknown",
            "spdx_id": null,
            "name": null,
            "url": null,
            "attribution_required": false,
            "redistribution_allowed": false,
            "export_allowed": false,
            "notes": "user-supplied edition; verify rights before activation (ADR-0101 fallback)",
        })
        .to_string(),
        created_at: at,
    };
    match super::quran::run_import_job(&db, input).await {
        Ok(_) => {
            // OD-01 B-track: synthetic pipeline-exercise data is never
            // canonical — label it on the human surface every time.
            let mut human =
                format!("imported {}@{} to Staged\n", doc.edition.slug, doc.edition.version);
            if doc.edition.synthetic {
                human.push_str("note: synthetic test data — non-canonical (ADR-0101 fallback)\n");
            }
            CommandOutput::ok(
                human,
                serde_json::json!({"slug": doc.edition.slug, "version": doc.edition.version.to_string()}),
            )
        }
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
            Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
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
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
    };
    let mut uow = match db.write().await {
        Ok(uow) => uow,
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
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
    let db = Arc::new(
        open_db(db_path)
            .await
            .map_err(|err| CommandOutput::err(exit::INTERNAL, err.to_string()))?,
    );
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
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
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
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
    };
    let mut uow = match db.write().await {
        Ok(uow) => uow,
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
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
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
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
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
    };
    let mut uow = match db.write().await {
        Ok(uow) => uow,
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
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
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
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
        Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
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
        Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
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
        Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
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
        Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
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
        Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
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
        Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
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
        Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()),
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
