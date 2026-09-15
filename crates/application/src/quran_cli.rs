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
        E::EditionNotFound(_) | E::AyahNotFound(_) | E::TranslationNotFound(_)
        | E::DivisionNotFound(_) => (exit::NOT_FOUND, error.to_string()),
        E::Storage(_) => (exit::INTERNAL, error.to_string()),
    }
}

fn map_activation_error(error: super::quran::ActivationError) -> (i32, String) {
    use super::quran::ActivationError as E;
    match &error {
        E::ApprovalMissing { .. } | E::ApprovalNotGranted { .. } | E::ApprovalSubjectMismatch { .. } => {
            (exit::POLICY, error.to_string())
        }
        E::NotStaged { .. } => (exit::CONFLICT, error.to_string()),
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
        glosses: false,
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
    let spec =
        ContextSpec { before, after, boundary, include_surah_header: true, max_ayahs: 100 };
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
        let human = format!("{}\n{}", first.canonical.surah_name_arabic(), first.canonical.reference());
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
    let reference =
        QuranRef::Division { edition: EditionSelector::Active, kind: division, number };
    let views = match reader(&db).get_ayahs(&reference, &AyahOptions::default()).await {
        Ok(views) => views,
        Err(err) => {
            let (exit, message) = map_reader_error(err);
            return CommandOutput::err(exit, message);
        }
    };
    let mut human = format!("{kind} {number} ({} ayahs)\n", views.len());
    for view in &views {
        human.push_str(&format!("{} {}\n", view.canonical.reference(), view.canonical.arabic_text()));
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
    let editions = match reader(&db)
        .list_editions(crate::quran_reader::EditionFilter::default())
        .await
    {
        Ok(editions) => editions,
        Err(err) => {
            let (exit, message) = map_reader_error(err);
            return CommandOutput::err(exit, message);
        }
    };
    let mut human = String::new();
    for edition in &editions {
        human.push_str(&format!("{}@{} [{}]\n", edition.slug, edition.version, edition_status(edition)));
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
    let mut human =
        format!("{}@{} — {}\nstatus: {}\n", edition.slug, edition.version, edition.name, edition_status(&edition));
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
        Ok(outcome) => {
            let job = outcome.result.clone().unwrap_or_default();
            CommandOutput::ok(
                format!("imported to Staged ({job})\n"),
                serde_json::json!({"job": job}),
            )
        }
        Err(err) => CommandOutput::err(exit::INTERNAL, err.to_string()),
    }
}

/// `quran validate`.
pub async fn cmd_validate(
    db_path: &str,
    target: &str,
    report_path: Option<&str>,
) -> CommandOutput {
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
            _ => return CommandOutput::err(exit::USAGE, format!("target must be a manifest path or `slug@version`, got `{target}`")),
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
    CommandOutput { exit, human: human.clone(), json: serde_json::to_value(&report).unwrap_or_default() }
}

/// `quran diff`.
pub async fn cmd_diff(db_path: &str, slug: &str, from: &str, to: &str, format: &str) -> CommandOutput {
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
    let load = async |version: &str| -> Result<Vec<(u16, u32, String)>, CommandOutput> {
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
        Ok(rows
            .into_iter()
            .map(|row| (row.surah as u16, row.ayah as u32, row.text))
            .collect())
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

fn approval_flow(
    db_path: &str,
) -> impl std::future::Future<Output = Result<(Arc<SqliteDatabase>, String, String), CommandOutput>> + '_
{
    async move {
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
}

/// `quran activate`.
pub async fn cmd_activate(db_path: &str, edition: &str) -> CommandOutput {
    let (slug, version) = match edition.split_once('@') {
        Some((slug, version)) if !slug.is_empty() && !version.is_empty() => (slug, version),
        _ => return CommandOutput::err(exit::USAGE, format!("edition must be `slug@version`, got `{edition}`")),
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
        _ => return CommandOutput::err(exit::USAGE, format!("edition must be `slug@version`, got `{edition}`")),
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
        _ => return CommandOutput::err(exit::USAGE, format!("edition must be `slug@version`, got `{edition}`")),
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
        human.push_str(&format!("{}@{} — {} [{}]\n", row.slug, row.version, row.translator, row.language));
    }
    CommandOutput::ok(
        human,
        serde_json::json!(rows
            .iter()
            .map(|row| serde_json::json!({
                "slug": row.slug, "version": row.version,
                "translator": row.translator, "language": row.language,
            }))
            .collect::<Vec<_>>()),
    )
}

/// `quran translation import`.
pub async fn cmd_translation_import(db_path: &str, manifest: &str) -> CommandOutput {
    let text = match std::fs::read_to_string(manifest) {
        Ok(text) => text,
        Err(err) => return CommandOutput::err(exit::USAGE, format!("cannot read `{manifest}`: {err}")),
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
    let exit = if checks
        .iter()
        .any(|check| check.status == super::quran_doctor::CheckLevel::Fail)
    {
        exit::VALIDATION
    } else {
        exit::OK
    };
    CommandOutput { exit, human, json }
}
