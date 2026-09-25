//! SQLite implementation of the Quran corpus repository (D1.5).
//!
//! All canonical writes flow through [`SqliteQuranRepository::activate_edition`]
//! (staging → canonical move plus the pointer flip in one transaction) or
//! [`SqliteQuranRepository::rollback_edition`]. There is deliberately no
//! row-level canonical insert: together with the insert-only triggers this
//! closes the API path behind AC-P1-09.

use async_trait::async_trait;
use sqlx::Row;
use storage::error::StorageError;
use storage::quran::{
    ActiveEditionRow, AlignmentRow, AyahFormRow, AyahRow, CitationRow, DifferenceReportRow,
    DivisionRow, FamilyRelationRow, FormColumn, ImportRunRow, IndexBuildRunRow, IndexPointerRow,
    LexiconLemmaRow, LexiconProvenance, LexiconRootRow, MorphReviewItemRow, MorphemeRow,
    MorphologyFindingRow, NormalizationProfileRow, NormalizationRuleRow, QuranDatasetRow,
    QuranEditionRow, QuranRepository, SearchCacheRow, SeparatorRow, SkeletonRow, StagedEditionRef,
    StagedMorphRow, StagingBatchRow, SurahRow, TokenAnalysisRow, TokenFormRow, TokenRow,
    TranslationEditionRow, TranslationPassageRow, ValidationReportRow, WordGlossRow,
};

use super::{SharedTx, map_sqlx_error};

pub(crate) struct SqliteQuranRepository {
    tx: SharedTx,
}

impl SqliteQuranRepository {
    pub(crate) fn new(tx: SharedTx) -> Self {
        Self { tx }
    }
}

fn decode_edition(row: &sqlx::sqlite::SqliteRow) -> QuranEditionRow {
    QuranEditionRow {
        id: row.get("id"),
        slug: row.get("slug"),
        version: row.get("version"),
        name: row.get("name"),
        script: row.get("script"),
        riwayah: row.get("riwayah"),
        qiraah: row.get("qiraah"),
        publisher: row.get("publisher"),
        source_url: row.get("source_url"),
        upstream_edition_slug: row.get("upstream_edition_slug"),
        qai_edition_id: row.get("qai_edition_id"),
        is_primary: row.get("is_primary"),
        language: row.get("language"),
        verse_numbering_scheme: row.get("verse_numbering_scheme"),
        basmala_policy: row.get("basmala_policy"),
        unicode_normalization: row.get("unicode_normalization"),
        license_json: row.get("license_json"),
        text_hash: row.get("text_hash"),
        structure_hash: row.get("structure_hash"),
        token_order_hash: row.get("token_order_hash"),
        manifest_hash: row.get("manifest_hash"),
        source_version_id: row.get("source_version_id"),
        statistics_json: row.get("statistics_json"),
        status: row.get("status"),
        imported_at: row.get("imported_at"),
        verified_at: row.get("verified_at"),
        verified_by: row.get("verified_by"),
        verification_method: row.get("verification_method"),
        activated_at: row.get("activated_at"),
        deprecated_at: row.get("deprecated_at"),
    }
}

fn decode_surah(row: &sqlx::sqlite::SqliteRow) -> SurahRow {
    SurahRow {
        edition_id: row.get("edition_id"),
        number: row.get("number"),
        name_arabic: row.get("name_arabic"),
        name_transliteration: row.get("name_transliteration"),
        name_translations_json: row.get("name_translations_json"),
        ayah_count: row.get("ayah_count"),
        revelation_place: row.get("revelation_place"),
        revelation_order: row.get("revelation_order"),
        basmala: row.get("basmala"),
        ruku_count: row.get("ruku_count"),
        metadata_provenance_id: row.get("metadata_provenance_id"),
    }
}

fn decode_ayah(row: &sqlx::sqlite::SqliteRow) -> AyahRow {
    AyahRow {
        edition_id: row.get("edition_id"),
        surah: row.get("surah"),
        ayah: row.get("ayah"),
        text: row.get("text"),
        text_hash: row.get("text_hash"),
        char_count: row.get("char_count"),
        token_count: row.get("token_count"),
        global_ayah_index: row.get("global_ayah_index"),
        juz: row.get("juz"),
        hizb: row.get("hizb"),
        rub: row.get("rub"),
        manzil: row.get("manzil"),
        ruku: row.get("ruku"),
        page: row.get("page"),
        sajdah: row.get("sajdah"),
        provenance_id: row.get("provenance_id"),
    }
}

fn decode_token(row: &sqlx::sqlite::SqliteRow) -> TokenRow {
    TokenRow {
        edition_id: row.get("edition_id"),
        surah: row.get("surah"),
        ayah: row.get("ayah"),
        position: row.get("position"),
        surface: row.get("surface"),
        surface_hash: row.get("surface_hash"),
        char_start: row.get("char_start"),
        char_end: row.get("char_end"),
        byte_start: row.get("byte_start"),
        byte_end: row.get("byte_end"),
        is_pause_mark: row.get::<i64, _>("is_pause_mark") != 0,
        global_token_index: row.get("global_token_index"),
    }
}

#[async_trait]
impl QuranRepository for SqliteQuranRepository {
    async fn insert_import_run(&mut self, row: ImportRunRow) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO quran_import_runs
                (run_id, job_id, edition_slug, edition_version, adapter, state, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&row.run_id)
        .bind(&row.job_id)
        .bind(&row.edition_slug)
        .bind(&row.edition_version)
        .bind(&row.adapter)
        .bind(&row.state)
        .bind(&row.created_at)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn get_import_run(&self, run_id: &str) -> Result<Option<ImportRunRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query(
            "SELECT run_id, job_id, edition_slug, edition_version, adapter, state, created_at
             FROM quran_import_runs WHERE run_id = ?",
        )
        .bind(run_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(row.map(|r| ImportRunRow {
            run_id: r.get("run_id"),
            job_id: r.get("job_id"),
            edition_slug: r.get("edition_slug"),
            edition_version: r.get("edition_version"),
            adapter: r.get("adapter"),
            state: r.get("state"),
            created_at: r.get("created_at"),
        }))
    }

    async fn set_import_run_state(
        &mut self,
        run_id: &str,
        state: &str,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query("UPDATE quran_import_runs SET state = ? WHERE run_id = ?")
            .bind(state)
            .bind(run_id)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn delete_import_run(&mut self, run_id: &str) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query("DELETE FROM quran_import_runs WHERE run_id = ?")
            .bind(run_id)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn count_staging_orphans(&self) -> Result<i64, StorageError> {
        let mut tx = self.tx.lock().await;
        let mut total = 0_i64;
        for table in [
            "quran_stg_editions",
            "quran_stg_surahs",
            "quran_stg_ayahs",
            "quran_stg_tokens",
            "quran_stg_token_separators",
            "quran_stg_segments",
            "quran_stg_divisions",
        ] {
            let sql = format!(
                "SELECT COUNT(*) AS n FROM {table} AS s
                 JOIN quran_import_runs AS r ON s.import_run_id = r.run_id
                 WHERE r.state IN ('Cancelled', 'Failed')"
            );
            let row = sqlx::query(&sql).fetch_one(&mut **tx).await.map_err(map_sqlx_error)?;
            total += row.get::<i64, _>("n");
        }
        Ok(total)
    }

    async fn find_staged_edition(
        &self,
        slug: &str,
        version: &str,
    ) -> Result<Option<StagedEditionRef>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query(
            "SELECT import_run_id AS run_id, id AS edition_id
             FROM quran_stg_editions WHERE slug = ? AND version = ? LIMIT 1",
        )
        .bind(slug)
        .bind(version)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(row
            .map(|r| StagedEditionRef { run_id: r.get("run_id"), edition_id: r.get("edition_id") }))
    }

    async fn get_stg_edition(
        &self,
        run_id: &str,
        edition_id: &str,
    ) -> Result<Option<QuranEditionRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query(
            "SELECT id, slug, version, name, script, riwayah, qiraah, publisher,
                    source_url, language, verse_numbering_scheme, basmala_policy,
                    unicode_normalization, license_json, text_hash, structure_hash,
                    token_order_hash, manifest_hash, source_version_id, statistics_json,
                    status, imported_at,
                    NULL AS verified_at, NULL AS verified_by, NULL AS verification_method,
                    NULL AS activated_at, NULL AS deprecated_at
             FROM quran_stg_editions WHERE import_run_id = ? AND id = ?",
        )
        .bind(run_id)
        .bind(edition_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(row.map(|r| decode_edition(&r)))
    }

    async fn list_stg_surahs(&self, run_id: &str) -> Result<Vec<SurahRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT edition_id, number, name_arabic, name_transliteration,
                    name_translations_json, ayah_count, revelation_place, revelation_order,
                    basmala, ruku_count, metadata_provenance_id AS metadata_provenance_id
             FROM quran_stg_surahs WHERE import_run_id = ? ORDER BY number",
        )
        .bind(run_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_surah).collect())
    }

    async fn list_stg_divisions(&self, run_id: &str) -> Result<Vec<DivisionRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT edition_id, kind, number, start_surah, start_ayah, end_surah, end_ayah,
                    start_global, end_global, label, provenance_id
             FROM quran_stg_divisions WHERE import_run_id = ? ORDER BY kind, number",
        )
        .bind(run_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows
            .iter()
            .map(|r| DivisionRow {
                edition_id: r.get("edition_id"),
                kind: r.get("kind"),
                number: r.get("number"),
                start_surah: r.get("start_surah"),
                start_ayah: r.get("start_ayah"),
                end_surah: r.get("end_surah"),
                end_ayah: r.get("end_ayah"),
                start_global: r.get("start_global"),
                end_global: r.get("end_global"),
                label: r.get("label"),
                provenance_id: r.get("provenance_id"),
            })
            .collect())
    }

    async fn set_edition_status(&mut self, id: &str, status: &str) -> Result<(), StorageError> {
        if !["Staged", "Approved", "Active", "Deprecated", "Quarantined"].contains(&status) {
            return Err(StorageError::ConstraintViolation {
                message: format!("unknown edition status `{status}`"),
            });
        }
        let mut tx = self.tx.lock().await;
        sqlx::query("UPDATE quran_editions SET status = ? WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn set_edition_verification(
        &mut self,
        id: &str,
        verified_by: &str,
        verified_at: &str,
        verification_method: &str,
    ) -> Result<(), StorageError> {
        if verified_by.trim().is_empty() {
            return Err(StorageError::ConstraintViolation {
                message: "verified_by must name the reviewer".to_string(),
            });
        }
        if verification_method.trim().is_empty() {
            return Err(StorageError::ConstraintViolation {
                message: "verification_method must describe the comparison".to_string(),
            });
        }
        let mut tx = self.tx.lock().await;
        // Only the `verified_*` metadata columns: slug, version, and hashes
        // stay trigger-guarded, and ayah/token rows are untouched.
        sqlx::query(
            "UPDATE quran_editions
                SET verified_by = ?, verified_at = ?, verification_method = ?
              WHERE id = ?",
        )
        .bind(verified_by)
        .bind(verified_at)
        .bind(verification_method)
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn insert_stg_edition(
        &mut self,
        run_id: &str,
        row: QuranEditionRow,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO quran_stg_editions
                (import_run_id, id, slug, version, name, script, riwayah, qiraah,
                 publisher, source_url, upstream_edition_slug, qai_edition_id, is_primary,
                 language, verse_numbering_scheme, basmala_policy,
                 unicode_normalization, license_json, text_hash, structure_hash,
                 token_order_hash, manifest_hash, source_version_id, statistics_json,
                 status, imported_at)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(run_id)
        .bind(&row.id)
        .bind(&row.slug)
        .bind(&row.version)
        .bind(&row.name)
        .bind(&row.script)
        .bind(&row.riwayah)
        .bind(&row.qiraah)
        .bind(&row.publisher)
        .bind(&row.source_url)
        .bind(&row.upstream_edition_slug)
        .bind(&row.qai_edition_id)
        .bind(row.is_primary)
        .bind(&row.language)
        .bind(&row.verse_numbering_scheme)
        .bind(&row.basmala_policy)
        .bind(&row.unicode_normalization)
        .bind(&row.license_json)
        .bind(&row.text_hash)
        .bind(&row.structure_hash)
        .bind(&row.token_order_hash)
        .bind(&row.manifest_hash)
        .bind(&row.source_version_id)
        .bind(&row.statistics_json)
        .bind(&row.status)
        .bind(&row.imported_at)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn insert_stg_surah(&mut self, run_id: &str, row: SurahRow) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO quran_stg_surahs
                (import_run_id, edition_id, number, name_arabic, name_transliteration,
                 name_translations_json, ayah_count, revelation_place, revelation_order,
                 basmala, ruku_count, metadata_provenance_id)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(run_id)
        .bind(&row.edition_id)
        .bind(row.number)
        .bind(&row.name_arabic)
        .bind(&row.name_transliteration)
        .bind(&row.name_translations_json)
        .bind(row.ayah_count)
        .bind(&row.revelation_place)
        .bind(row.revelation_order)
        .bind(&row.basmala)
        .bind(row.ruku_count)
        .bind(&row.metadata_provenance_id)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn insert_stg_ayah(&mut self, run_id: &str, row: AyahRow) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO quran_stg_ayahs
                (import_run_id, edition_id, surah, ayah, text, text_hash, char_count,
                 token_count, global_ayah_index, juz, hizb, rub, manzil, ruku, page,
                 sajdah, provenance_id)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(run_id)
        .bind(&row.edition_id)
        .bind(row.surah)
        .bind(row.ayah)
        .bind(&row.text)
        .bind(&row.text_hash)
        .bind(row.char_count)
        .bind(row.token_count)
        .bind(row.global_ayah_index)
        .bind(row.juz)
        .bind(row.hizb)
        .bind(row.rub)
        .bind(row.manzil)
        .bind(row.ruku)
        .bind(row.page)
        .bind(&row.sajdah)
        .bind(&row.provenance_id)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn insert_stg_token(&mut self, run_id: &str, row: TokenRow) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO quran_stg_tokens
                (import_run_id, edition_id, surah, ayah, position, surface, surface_hash,
                 char_start, char_end, byte_start, byte_end, is_pause_mark, global_token_index)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(run_id)
        .bind(&row.edition_id)
        .bind(row.surah)
        .bind(row.ayah)
        .bind(row.position)
        .bind(&row.surface)
        .bind(&row.surface_hash)
        .bind(row.char_start)
        .bind(row.char_end)
        .bind(row.byte_start)
        .bind(row.byte_end)
        .bind(i64::from(row.is_pause_mark))
        .bind(row.global_token_index)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn insert_stg_separator(
        &mut self,
        run_id: &str,
        row: SeparatorRow,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO quran_stg_token_separators
                (import_run_id, edition_id, surah, ayah, after_position, separator)
             VALUES (?,?,?,?,?,?)",
        )
        .bind(run_id)
        .bind(&row.edition_id)
        .bind(row.surah)
        .bind(row.ayah)
        .bind(row.after_position)
        .bind(&row.separator)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn insert_stg_division(
        &mut self,
        run_id: &str,
        row: DivisionRow,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO quran_stg_divisions
                (import_run_id, edition_id, kind, number, start_surah, start_ayah,
                 end_surah, end_ayah, start_global, end_global, label, provenance_id)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(run_id)
        .bind(&row.edition_id)
        .bind(&row.kind)
        .bind(row.number)
        .bind(row.start_surah)
        .bind(row.start_ayah)
        .bind(row.end_surah)
        .bind(row.end_ayah)
        .bind(row.start_global)
        .bind(row.end_global)
        .bind(&row.label)
        .bind(&row.provenance_id)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn count_stg_ayahs(&self, run_id: &str) -> Result<i64, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query("SELECT COUNT(*) AS n FROM quran_stg_ayahs WHERE import_run_id = ?")
            .bind(run_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(row.get("n"))
    }

    async fn clear_staging(&mut self, run_id: &str) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        for table in [
            "quran_stg_token_separators",
            "quran_stg_tokens",
            "quran_stg_ayahs",
            "quran_stg_surahs",
            "quran_stg_segments",
            "quran_stg_divisions",
            "quran_stg_editions",
        ] {
            let sql = format!("DELETE FROM {table} WHERE import_run_id = ?");
            sqlx::query(&sql).bind(run_id).execute(&mut **tx).await.map_err(map_sqlx_error)?;
        }
        Ok(())
    }

    async fn list_stg_ayahs(&self, run_id: &str) -> Result<Vec<AyahRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT edition_id, surah, ayah, text, text_hash, char_count, token_count,
                    global_ayah_index, juz, hizb, rub, manzil, ruku, page, sajdah, provenance_id
             FROM quran_stg_ayahs WHERE import_run_id = ? ORDER BY surah, ayah",
        )
        .bind(run_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_ayah).collect())
    }

    async fn list_stg_tokens(
        &self,
        run_id: &str,
        edition_id: &str,
        surah: i64,
        ayah: i64,
    ) -> Result<Vec<TokenRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT edition_id, surah, ayah, position, surface, surface_hash, char_start,
                    char_end, byte_start, byte_end, is_pause_mark, global_token_index
             FROM quran_stg_tokens
             WHERE import_run_id = ? AND edition_id = ? AND surah = ? AND ayah = ?
             ORDER BY position",
        )
        .bind(run_id)
        .bind(edition_id)
        .bind(surah)
        .bind(ayah)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_token).collect())
    }

    async fn list_stg_separators(
        &self,
        run_id: &str,
        edition_id: &str,
        surah: i64,
        ayah: i64,
    ) -> Result<Vec<SeparatorRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT edition_id, surah, ayah, after_position, separator
             FROM quran_stg_token_separators
             WHERE import_run_id = ? AND edition_id = ? AND surah = ? AND ayah = ?
             ORDER BY after_position",
        )
        .bind(run_id)
        .bind(edition_id)
        .bind(surah)
        .bind(ayah)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows
            .iter()
            .map(|r| SeparatorRow {
                edition_id: r.get("edition_id"),
                surah: r.get("surah"),
                ayah: r.get("ayah"),
                after_position: r.get("after_position"),
                separator: r.get("separator"),
            })
            .collect())
    }

    async fn activate_edition(
        &mut self,
        run_id: &str,
        edition_id: &str,
        activated_by: &str,
        approval_id: &str,
        activated_at: &str,
    ) -> Result<i64, StorageError> {
        let mut tx = self.tx.lock().await;
        let state_row = sqlx::query("SELECT state FROM quran_import_runs WHERE run_id = ?")
            .bind(run_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        match state_row.map(|row| row.get::<String, _>("state")) {
            Some(state) if state == "Staged" => {}
            Some(state) => {
                return Err(StorageError::ConstraintViolation {
                    message: format!("import run {run_id} is {state}, not Staged"),
                });
            }
            None => {
                return Err(StorageError::NotFound { urn: run_id.to_string() });
            }
        }

        sqlx::query(
            "INSERT INTO quran_editions
                (id, slug, version, name, script, riwayah, qiraah, publisher, source_url,
                 upstream_edition_slug, qai_edition_id, is_primary,
                 language, verse_numbering_scheme, basmala_policy, unicode_normalization,
                 license_json, text_hash, structure_hash, token_order_hash, manifest_hash,
                 source_version_id, statistics_json, status, imported_at, activated_at)
             SELECT id, slug, version, name, script, riwayah, qiraah, publisher, source_url,
                 upstream_edition_slug, qai_edition_id, is_primary,
                 language, verse_numbering_scheme, basmala_policy, unicode_normalization,
                 license_json, text_hash, structure_hash, token_order_hash, manifest_hash,
                 source_version_id, statistics_json, 'Active', imported_at, ?
             FROM quran_stg_editions WHERE import_run_id = ? AND id = ?",
        )
        .bind(activated_at)
        .bind(run_id)
        .bind(edition_id)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        for (staging, canonical, columns) in [
            (
                "quran_stg_surahs",
                "quran_surahs",
                "edition_id, number, name_arabic, name_transliteration, name_translations_json, ayah_count, revelation_place, revelation_order, basmala, ruku_count, metadata_provenance_id",
            ),
            (
                "quran_stg_ayahs",
                "quran_ayahs",
                "edition_id, surah, ayah, text, text_hash, char_count, token_count, global_ayah_index, juz, hizb, rub, manzil, ruku, page, sajdah, provenance_id",
            ),
            (
                "quran_stg_tokens",
                "quran_tokens",
                "edition_id, surah, ayah, position, surface, surface_hash, char_start, char_end, byte_start, byte_end, is_pause_mark, global_token_index",
            ),
            (
                "quran_stg_token_separators",
                "quran_token_separators",
                "edition_id, surah, ayah, after_position, separator",
            ),
            (
                "quran_stg_divisions",
                "quran_divisions",
                "edition_id, kind, number, start_surah, start_ayah, end_surah, end_ayah, start_global, end_global, label, provenance_id",
            ),
        ] {
            let sql = format!(
                "INSERT INTO {canonical} ({columns}) SELECT {columns} FROM {staging} WHERE import_run_id = ?"
            );
            sqlx::query(&sql).bind(run_id).execute(&mut **tx).await.map_err(map_sqlx_error)?;
        }

        sqlx::query(
            "UPDATE quran_editions SET status = 'Deprecated', deprecated_at = ?
             WHERE status = 'Active' AND id != ?",
        )
        .bind(activated_at)
        .bind(edition_id)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        let current: Option<i64> =
            sqlx::query_scalar("SELECT corpus_generation FROM quran_active_edition")
                .fetch_optional(&mut **tx)
                .await
                .map_err(map_sqlx_error)?;
        let generation = current.unwrap_or(0) + 1;
        sqlx::query("DELETE FROM quran_active_edition")
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        sqlx::query(
            "INSERT INTO quran_active_edition
                (singleton, edition_id, corpus_generation, activated_at, activated_by, approval_id)
             VALUES (1, ?, ?, ?, ?, ?)",
        )
        .bind(edition_id)
        .bind(generation)
        .bind(activated_at)
        .bind(activated_by)
        .bind(approval_id)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        sqlx::query("DELETE FROM quran_import_runs WHERE run_id = ?")
            .bind(run_id)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(generation)
    }

    async fn rollback_edition(
        &mut self,
        slug: &str,
        version: &str,
        activated_by: &str,
        approval_id: &str,
        activated_at: &str,
    ) -> Result<i64, StorageError> {
        let mut tx = self.tx.lock().await;
        let target =
            sqlx::query("SELECT id, status FROM quran_editions WHERE slug = ? AND version = ?")
                .bind(slug)
                .bind(version)
                .fetch_optional(&mut **tx)
                .await
                .map_err(map_sqlx_error)?;
        let (target_id, target_status): (String, String) = match target {
            Some(row) => (row.get("id"), row.get("status")),
            None => {
                return Err(StorageError::NotFound { urn: format!("{slug}@{version}") });
            }
        };
        if target_status == "Active" {
            return Err(StorageError::Conflict);
        }
        sqlx::query(
            "UPDATE quran_editions SET status = 'Deprecated', deprecated_at = ?
             WHERE status = 'Active'",
        )
        .bind(activated_at)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        sqlx::query(
            "UPDATE quran_editions SET status = 'Active', activated_at = ?, deprecated_at = NULL
             WHERE id = ?",
        )
        .bind(activated_at)
        .bind(&target_id)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        let current: Option<i64> =
            sqlx::query_scalar("SELECT corpus_generation FROM quran_active_edition")
                .fetch_optional(&mut **tx)
                .await
                .map_err(map_sqlx_error)?;
        let generation = current.unwrap_or(0) + 1;
        sqlx::query("DELETE FROM quran_active_edition")
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        sqlx::query(
            "INSERT INTO quran_active_edition
                (singleton, edition_id, corpus_generation, activated_at, activated_by, approval_id)
             VALUES (1, ?, ?, ?, ?, ?)",
        )
        .bind(&target_id)
        .bind(generation)
        .bind(activated_at)
        .bind(activated_by)
        .bind(approval_id)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(generation)
    }

    async fn get_edition(&self, id: &str) -> Result<Option<QuranEditionRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query("SELECT * FROM quran_editions WHERE id = ?")
            .bind(id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(row.map(|r| decode_edition(&r)))
    }

    async fn get_edition_by_slug_version(
        &self,
        slug: &str,
        version: &str,
    ) -> Result<Option<QuranEditionRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query("SELECT * FROM quran_editions WHERE slug = ? AND version = ?")
            .bind(slug)
            .bind(version)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(row.map(|r| decode_edition(&r)))
    }

    async fn list_editions(&self) -> Result<Vec<QuranEditionRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query("SELECT * FROM quran_editions ORDER BY slug, version")
            .fetch_all(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_edition).collect())
    }

    async fn get_active(&self) -> Result<Option<ActiveEditionRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query(
            "SELECT edition_id, corpus_generation, activated_at, activated_by, approval_id
             FROM quran_active_edition WHERE singleton = 1",
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(row.map(|r| ActiveEditionRow {
            edition_id: r.get("edition_id"),
            corpus_generation: r.get("corpus_generation"),
            activated_at: r.get("activated_at"),
            activated_by: r.get("activated_by"),
            approval_id: r.get("approval_id"),
        }))
    }

    async fn get_surah(
        &self,
        edition_id: &str,
        number: i64,
    ) -> Result<Option<SurahRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query("SELECT * FROM quran_surahs WHERE edition_id = ? AND number = ?")
            .bind(edition_id)
            .bind(number)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(row.map(|r| decode_surah(&r)))
    }

    async fn list_surahs(&self, edition_id: &str) -> Result<Vec<SurahRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query("SELECT * FROM quran_surahs WHERE edition_id = ? ORDER BY number")
            .bind(edition_id)
            .fetch_all(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_surah).collect())
    }

    async fn get_ayah(
        &self,
        edition_id: &str,
        surah: i64,
        ayah: i64,
    ) -> Result<Option<AyahRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query(
            "SELECT * FROM quran_ayahs WHERE edition_id = ? AND surah = ? AND ayah = ?",
        )
        .bind(edition_id)
        .bind(surah)
        .bind(ayah)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(row.map(|r| decode_ayah(&r)))
    }

    async fn get_ayah_by_global(
        &self,
        edition_id: &str,
        global: i64,
    ) -> Result<Option<AyahRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row =
            sqlx::query("SELECT * FROM quran_ayahs WHERE edition_id = ? AND global_ayah_index = ?")
                .bind(edition_id)
                .bind(global)
                .fetch_optional(&mut **tx)
                .await
                .map_err(map_sqlx_error)?;
        Ok(row.map(|r| decode_ayah(&r)))
    }

    async fn list_ayahs_range(
        &self,
        edition_id: &str,
        start_global: i64,
        end_global: i64,
    ) -> Result<Vec<AyahRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT * FROM quran_ayahs
             WHERE edition_id = ? AND global_ayah_index BETWEEN ? AND ?
             ORDER BY global_ayah_index",
        )
        .bind(edition_id)
        .bind(start_global)
        .bind(end_global)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_ayah).collect())
    }

    async fn get_tokens(
        &self,
        edition_id: &str,
        surah: i64,
        ayah: i64,
    ) -> Result<Vec<TokenRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT * FROM quran_tokens
             WHERE edition_id = ? AND surah = ? AND ayah = ? ORDER BY position",
        )
        .bind(edition_id)
        .bind(surah)
        .bind(ayah)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_token).collect())
    }

    async fn get_separators(
        &self,
        edition_id: &str,
        surah: i64,
        ayah: i64,
    ) -> Result<Vec<SeparatorRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT edition_id, surah, ayah, after_position, separator
             FROM quran_token_separators
             WHERE edition_id = ? AND surah = ? AND ayah = ? ORDER BY after_position",
        )
        .bind(edition_id)
        .bind(surah)
        .bind(ayah)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows
            .iter()
            .map(|r| SeparatorRow {
                edition_id: r.get("edition_id"),
                surah: r.get("surah"),
                ayah: r.get("ayah"),
                after_position: r.get("after_position"),
                separator: r.get("separator"),
            })
            .collect())
    }

    async fn list_divisions(
        &self,
        edition_id: &str,
        kind: &str,
    ) -> Result<Vec<DivisionRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT * FROM quran_divisions WHERE edition_id = ? AND kind = ? ORDER BY number",
        )
        .bind(edition_id)
        .bind(kind)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows
            .iter()
            .map(|r| DivisionRow {
                edition_id: r.get("edition_id"),
                kind: r.get("kind"),
                number: r.get("number"),
                start_surah: r.get("start_surah"),
                start_ayah: r.get("start_ayah"),
                end_surah: r.get("end_surah"),
                end_ayah: r.get("end_ayah"),
                start_global: r.get("start_global"),
                end_global: r.get("end_global"),
                label: r.get("label"),
                provenance_id: r.get("provenance_id"),
            })
            .collect())
    }

    async fn count_ayahs(&self, edition_id: &str) -> Result<i64, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query("SELECT COUNT(*) AS n FROM quran_ayahs WHERE edition_id = ?")
            .bind(edition_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(row.get("n"))
    }

    async fn count_tokens(&self, edition_id: &str) -> Result<i64, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query("SELECT COUNT(*) AS n FROM quran_tokens WHERE edition_id = ?")
            .bind(edition_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(row.get("n"))
    }

    async fn insert_validation_report(
        &mut self,
        row: ValidationReportRow,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO validation_reports
                (id, subject_urn, validator, validator_version, outcome,
                 fatal_count, error_count, warning_count, findings_json, created_at)
             VALUES (?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(&row.id)
        .bind(&row.subject_urn)
        .bind(&row.validator)
        .bind(&row.validator_version)
        .bind(&row.outcome)
        .bind(row.fatal_count)
        .bind(row.error_count)
        .bind(row.warning_count)
        .bind(&row.findings_json)
        .bind(&row.created_at)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn get_validation_report(
        &self,
        id: &str,
    ) -> Result<Option<ValidationReportRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query("SELECT * FROM validation_reports WHERE id = ?")
            .bind(id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(row.map(|r| ValidationReportRow {
            id: r.get("id"),
            subject_urn: r.get("subject_urn"),
            validator: r.get("validator"),
            validator_version: r.get("validator_version"),
            outcome: r.get("outcome"),
            fatal_count: r.get("fatal_count"),
            error_count: r.get("error_count"),
            warning_count: r.get("warning_count"),
            findings_json: r.get("findings_json"),
            created_at: r.get("created_at"),
        }))
    }

    async fn insert_difference_report(
        &mut self,
        row: DifferenceReportRow,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO difference_reports
                (id, subject_urn, from_version, to_version, differ, differ_version,
                 summary_json, details_json, created_at)
             VALUES (?,?,?,?,?,?,?,?,?)",
        )
        .bind(&row.id)
        .bind(&row.subject_urn)
        .bind(&row.from_version)
        .bind(&row.to_version)
        .bind(&row.differ)
        .bind(&row.differ_version)
        .bind(&row.summary_json)
        .bind(&row.details_json)
        .bind(&row.created_at)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn insert_citation(&mut self, row: CitationRow) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO citations
                (id, kind, canonical_reference, source_id, source_version_id, edition_ref,
                 location_json, quoted_text_hash, ingestion_version, resolved_at, verdict)
             VALUES (?,?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(&row.id)
        .bind(&row.kind)
        .bind(&row.canonical_reference)
        .bind(&row.source_id)
        .bind(&row.source_version_id)
        .bind(&row.edition_ref)
        .bind(&row.location_json)
        .bind(&row.quoted_text_hash)
        .bind(&row.ingestion_version)
        .bind(&row.resolved_at)
        .bind(&row.verdict)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn get_citation(&self, id: &str) -> Result<Option<CitationRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query("SELECT * FROM citations WHERE id = ?")
            .bind(id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(row.map(|r| CitationRow {
            id: r.get("id"),
            kind: r.get("kind"),
            canonical_reference: r.get("canonical_reference"),
            source_id: r.get("source_id"),
            source_version_id: r.get("source_version_id"),
            edition_ref: r.get("edition_ref"),
            location_json: r.get("location_json"),
            quoted_text_hash: r.get("quoted_text_hash"),
            ingestion_version: r.get("ingestion_version"),
            resolved_at: r.get("resolved_at"),
            verdict: r.get("verdict"),
        }))
    }

    async fn list_citations_by_ref(
        &self,
        canonical_reference: &str,
    ) -> Result<Vec<CitationRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query("SELECT * FROM citations WHERE canonical_reference = ?")
            .bind(canonical_reference)
            .fetch_all(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(rows
            .iter()
            .map(|r| CitationRow {
                id: r.get("id"),
                kind: r.get("kind"),
                canonical_reference: r.get("canonical_reference"),
                source_id: r.get("source_id"),
                source_version_id: r.get("source_version_id"),
                edition_ref: r.get("edition_ref"),
                location_json: r.get("location_json"),
                quoted_text_hash: r.get("quoted_text_hash"),
                ingestion_version: r.get("ingestion_version"),
                resolved_at: r.get("resolved_at"),
                verdict: r.get("verdict"),
            })
            .collect())
    }

    async fn insert_translation_edition(
        &mut self,
        row: TranslationEditionRow,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO translation_editions
                (id, slug, version, name, translator, language, aligned_edition_id,
                 numbering_scheme, license_json, trust_level, source_version_id,
                 text_hash, status, imported_at)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(&row.id)
        .bind(&row.slug)
        .bind(&row.version)
        .bind(&row.name)
        .bind(&row.translator)
        .bind(&row.language)
        .bind(&row.aligned_edition_id)
        .bind(&row.numbering_scheme)
        .bind(&row.license_json)
        .bind(&row.trust_level)
        .bind(&row.source_version_id)
        .bind(&row.text_hash)
        .bind(&row.status)
        .bind(&row.imported_at)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn insert_translation_passage(
        &mut self,
        row: TranslationPassageRow,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO translation_passages
                (translation_edition_id, surah, ayah, text, footnotes_json, provenance_id)
             VALUES (?,?,?,?,?,?)",
        )
        .bind(&row.translation_edition_id)
        .bind(row.surah)
        .bind(row.ayah)
        .bind(&row.text)
        .bind(&row.footnotes_json)
        .bind(&row.provenance_id)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn get_translation_passage(
        &self,
        translation_edition_id: &str,
        surah: i64,
        ayah: i64,
    ) -> Result<Option<TranslationPassageRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query(
            "SELECT * FROM translation_passages
             WHERE translation_edition_id = ? AND surah = ? AND ayah = ?",
        )
        .bind(translation_edition_id)
        .bind(surah)
        .bind(ayah)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(row.map(|r| TranslationPassageRow {
            translation_edition_id: r.get("translation_edition_id"),
            surah: r.get("surah"),
            ayah: r.get("ayah"),
            text: r.get("text"),
            footnotes_json: r.get("footnotes_json"),
            provenance_id: r.get("provenance_id"),
        }))
    }

    async fn list_translation_editions(&self) -> Result<Vec<TranslationEditionRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query("SELECT * FROM translation_editions ORDER BY slug, version")
            .fetch_all(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(rows
            .iter()
            .map(|r| TranslationEditionRow {
                id: r.get("id"),
                slug: r.get("slug"),
                version: r.get("version"),
                name: r.get("name"),
                translator: r.get("translator"),
                language: r.get("language"),
                aligned_edition_id: r.get("aligned_edition_id"),
                numbering_scheme: r.get("numbering_scheme"),
                license_json: r.get("license_json"),
                trust_level: r.get("trust_level"),
                source_version_id: r.get("source_version_id"),
                text_hash: r.get("text_hash"),
                status: r.get("status"),
                imported_at: r.get("imported_at"),
            })
            .collect())
    }

    async fn insert_word_gloss(&mut self, row: WordGlossRow) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO word_glosses
                (gloss_dataset_id, edition_id, surah, ayah, position,
                 language, gloss, provenance_id)
             VALUES (?,?,?,?,?,?,?,?)",
        )
        .bind(&row.gloss_dataset_id)
        .bind(&row.edition_id)
        .bind(row.surah)
        .bind(row.ayah)
        .bind(row.position)
        .bind(&row.language)
        .bind(&row.gloss)
        .bind(&row.provenance_id)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn list_word_glosses(
        &self,
        edition_id: &str,
        surah: i64,
        ayah: i64,
    ) -> Result<Vec<WordGlossRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT * FROM word_glosses
             WHERE edition_id = ? AND surah = ? AND ayah = ?
             ORDER BY gloss_dataset_id, position, language",
        )
        .bind(edition_id)
        .bind(surah)
        .bind(ayah)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows
            .iter()
            .map(|r| WordGlossRow {
                gloss_dataset_id: r.get("gloss_dataset_id"),
                edition_id: r.get("edition_id"),
                surah: r.get("surah"),
                ayah: r.get("ayah"),
                position: r.get("position"),
                language: r.get("language"),
                gloss: r.get("gloss"),
                provenance_id: r.get("provenance_id"),
            })
            .collect())
    }

    async fn list_normalization_rules(&self) -> Result<Vec<NormalizationRuleRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT rule_id, version, kind, description
             FROM normalization_rules ORDER BY rule_id, version",
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows
            .iter()
            .map(|r| NormalizationRuleRow {
                rule_id: r.get("rule_id"),
                version: r.get("version"),
                kind: r.get("kind"),
                description: r.get("description"),
            })
            .collect())
    }

    async fn list_normalization_profiles(
        &self,
    ) -> Result<Vec<NormalizationProfileRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT profile_id, version, label, rules_json, indexed, heuristic, experimental
             FROM normalization_profiles ORDER BY profile_id, version",
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_profile).collect())
    }

    async fn get_normalization_profile(
        &self,
        profile_id: &str,
        version: &str,
    ) -> Result<Option<NormalizationProfileRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query(
            "SELECT profile_id, version, label, rules_json, indexed, heuristic, experimental
             FROM normalization_profiles WHERE profile_id = ? AND version = ?",
        )
        .bind(profile_id)
        .bind(version)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(row.map(|r| decode_profile(&r)))
    }

    async fn insert_normalization_profile(
        &mut self,
        row: NormalizationProfileRow,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO normalization_profiles
                (profile_id, version, label, rules_json, indexed, heuristic, experimental)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&row.profile_id)
        .bind(&row.version)
        .bind(&row.label)
        .bind(&row.rules_json)
        .bind(row.indexed as i64)
        .bind(row.heuristic as i64)
        .bind(row.experimental as i64)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn insert_token_forms(&mut self, rows: Vec<TokenFormRow>) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        for row in &rows {
            sqlx::query(
                "INSERT INTO quran_token_forms
                    (edition_id, surah, ayah, position, simple, bare, hamza_folded, folded,
                     affix_stripped, transliteration, phonetic, rule_set_id, rule_set_version,
                     corpus_generation, provenance_id)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&row.edition_id)
            .bind(row.surah)
            .bind(row.ayah)
            .bind(row.position)
            .bind(&row.simple)
            .bind(&row.bare)
            .bind(&row.hamza_folded)
            .bind(&row.folded)
            .bind(&row.affix_stripped)
            .bind(&row.transliteration)
            .bind(&row.phonetic)
            .bind(&row.rule_set_id)
            .bind(&row.rule_set_version)
            .bind(row.corpus_generation)
            .bind(&row.provenance_id)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        }
        Ok(())
    }

    async fn insert_ayah_forms(&mut self, rows: Vec<AyahFormRow>) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        for row in &rows {
            sqlx::query(
                "INSERT INTO quran_ayah_forms
                    (edition_id, surah, ayah, simple, bare, hamza_folded, folded,
                     transliteration, rule_set_id, rule_set_version,
                     corpus_generation, provenance_id)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&row.edition_id)
            .bind(row.surah)
            .bind(row.ayah)
            .bind(&row.simple)
            .bind(&row.bare)
            .bind(&row.hamza_folded)
            .bind(&row.folded)
            .bind(&row.transliteration)
            .bind(&row.rule_set_id)
            .bind(&row.rule_set_version)
            .bind(row.corpus_generation)
            .bind(&row.provenance_id)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        }
        Ok(())
    }

    async fn insert_skeletons(&mut self, rows: Vec<SkeletonRow>) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        for row in &rows {
            sqlx::query(
                "INSERT INTO quran_skeletons
                    (edition_id, surah, ayah_start, ayah_end, skeleton,
                     rule_set_id, rule_set_version, corpus_generation, provenance_id)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&row.edition_id)
            .bind(row.surah)
            .bind(row.ayah_start)
            .bind(row.ayah_end)
            .bind(&row.skeleton)
            .bind(&row.rule_set_id)
            .bind(&row.rule_set_version)
            .bind(row.corpus_generation)
            .bind(&row.provenance_id)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        }
        Ok(())
    }

    async fn list_token_forms(
        &self,
        edition_id: &str,
        surah: i64,
        ayah: i64,
    ) -> Result<Vec<TokenFormRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT edition_id, surah, ayah, position, simple, bare, hamza_folded, folded,
                    affix_stripped, transliteration, phonetic, rule_set_id, rule_set_version,
                    corpus_generation, provenance_id
             FROM quran_token_forms
             WHERE edition_id = ? AND surah = ? AND ayah = ? ORDER BY position",
        )
        .bind(edition_id)
        .bind(surah)
        .bind(ayah)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_token_form).collect())
    }

    async fn count_tokens_matching_form(
        &self,
        edition_id: &str,
        column: FormColumn,
        value: &str,
    ) -> Result<i64, StorageError> {
        let mut tx = self.tx.lock().await;
        let sql = format!(
            "SELECT COUNT(*) FROM quran_token_forms WHERE edition_id = ? AND {} = ?",
            column.column_name()
        );
        let count: i64 = sqlx::query_scalar(&sql)
            .bind(edition_id)
            .bind(value)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(count.max(0))
    }

    async fn count_tokens_matching_form_by_surah(
        &self,
        edition_id: &str,
        column: FormColumn,
        value: &str,
    ) -> Result<Vec<(i64, i64)>, StorageError> {
        let mut tx = self.tx.lock().await;
        let sql = format!(
            "SELECT surah, COUNT(*) FROM quran_token_forms
             WHERE edition_id = ? AND {} = ? GROUP BY surah ORDER BY surah",
            column.column_name()
        );
        let rows = sqlx::query(&sql)
            .bind(edition_id)
            .bind(value)
            .fetch_all(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(rows
            .iter()
            .map(|r| {
                use sqlx::Row as _;
                (r.get::<i64, _>("surah"), r.get::<i64, _>("COUNT(*)"))
            })
            .collect())
    }

    async fn list_distinct_forms(
        &self,
        edition_id: &str,
        column: FormColumn,
    ) -> Result<Vec<(String, i64)>, StorageError> {
        let mut tx = self.tx.lock().await;
        let sql = format!(
            "SELECT {}, COUNT(*) AS n FROM quran_token_forms
             WHERE edition_id = ? GROUP BY {} ORDER BY n DESC, {} ASC",
            column.column_name(),
            column.column_name(),
            column.column_name()
        );
        let rows = sqlx::query(&sql)
            .bind(edition_id)
            .fetch_all(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(rows
            .iter()
            .map(|r| {
                use sqlx::Row as _;
                (r.get::<String, _>(column.column_name()), r.get::<i64, _>("n"))
            })
            .collect())
    }

    async fn list_all_token_forms(
        &self,
        edition_id: &str,
    ) -> Result<Vec<TokenFormRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT edition_id, surah, ayah, position, simple, bare, hamza_folded, folded,
                    affix_stripped, transliteration, phonetic, rule_set_id, rule_set_version,
                    corpus_generation, provenance_id
             FROM quran_token_forms
             WHERE edition_id = ? ORDER BY surah, ayah, position",
        )
        .bind(edition_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_token_form).collect())
    }

    async fn get_ayah_form(
        &self,
        edition_id: &str,
        surah: i64,
        ayah: i64,
    ) -> Result<Option<AyahFormRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query(
            "SELECT edition_id, surah, ayah, simple, bare, hamza_folded, folded,
                    transliteration, rule_set_id, rule_set_version,
                    corpus_generation, provenance_id
             FROM quran_ayah_forms
             WHERE edition_id = ? AND surah = ? AND ayah = ?",
        )
        .bind(edition_id)
        .bind(surah)
        .bind(ayah)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(row.map(|r| decode_ayah_form(&r)))
    }

    async fn list_skeletons(
        &self,
        edition_id: &str,
        surah: i64,
    ) -> Result<Vec<SkeletonRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT edition_id, surah, ayah_start, ayah_end, skeleton,
                    rule_set_id, rule_set_version, corpus_generation, provenance_id
             FROM quran_skeletons
             WHERE edition_id = ? AND surah = ? ORDER BY ayah_start, ayah_end",
        )
        .bind(edition_id)
        .bind(surah)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_skeleton).collect())
    }

    async fn count_token_forms(&self, edition_id: &str) -> Result<i64, StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_scalar("SELECT COUNT(*) FROM quran_token_forms WHERE edition_id = ?")
            .bind(edition_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_sqlx_error)
    }

    async fn count_ayah_forms(&self, edition_id: &str) -> Result<i64, StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_scalar("SELECT COUNT(*) FROM quran_ayah_forms WHERE edition_id = ?")
            .bind(edition_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_sqlx_error)
    }

    async fn delete_forms_for_edition(&mut self, edition_id: &str) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        for table in ["quran_token_forms", "quran_ayah_forms", "quran_skeletons"] {
            sqlx::query(&format!("DELETE FROM {table} WHERE edition_id = ?"))
                .bind(edition_id)
                .execute(&mut **tx)
                .await
                .map_err(map_sqlx_error)?;
        }
        Ok(())
    }

    async fn get_index_pointer(
        &self,
        index_id: &str,
    ) -> Result<Option<IndexPointerRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query(
            "SELECT index_id, generation, manifest_json, updated_at, updated_by
             FROM index_pointers WHERE index_id = ?",
        )
        .bind(index_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(row.map(|r| IndexPointerRow {
            index_id: r.get("index_id"),
            generation: r.get("generation"),
            manifest_json: r.get("manifest_json"),
            updated_at: r.get("updated_at"),
            updated_by: r.get("updated_by"),
        }))
    }

    async fn upsert_index_pointer(&mut self, row: IndexPointerRow) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO index_pointers (index_id, generation, manifest_json, updated_at, updated_by)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT (index_id) DO UPDATE SET
                generation = excluded.generation,
                manifest_json = excluded.manifest_json,
                updated_at = excluded.updated_at,
                updated_by = excluded.updated_by",
        )
        .bind(&row.index_id)
        .bind(row.generation)
        .bind(&row.manifest_json)
        .bind(&row.updated_at)
        .bind(&row.updated_by)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn insert_build_run(&mut self, row: IndexBuildRunRow) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO index_build_runs
                (id, index_id, generation, corpus_generation, state, doc_count,
                 manifest_hash, error, started_at, finished_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&row.id)
        .bind(&row.index_id)
        .bind(row.generation)
        .bind(row.corpus_generation)
        .bind(&row.state)
        .bind(row.doc_count)
        .bind(&row.manifest_hash)
        .bind(&row.error)
        .bind(&row.started_at)
        .bind(&row.finished_at)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn set_build_run_state(
        &mut self,
        id: &str,
        state: &str,
        doc_count: i64,
        manifest_hash: &str,
        error: Option<&str>,
        finished_at: &str,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "UPDATE index_build_runs
             SET state = ?, doc_count = ?, manifest_hash = ?, error = ?, finished_at = ?
             WHERE id = ?",
        )
        .bind(state)
        .bind(doc_count)
        .bind(manifest_hash)
        .bind(error)
        .bind(finished_at)
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn list_build_runs(&self, index_id: &str) -> Result<Vec<IndexBuildRunRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT id, index_id, generation, corpus_generation, state, doc_count,
                    manifest_hash, error, started_at, finished_at
             FROM index_build_runs WHERE index_id = ? ORDER BY generation",
        )
        .bind(index_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_build_run).collect())
    }

    async fn max_build_generation(&self, index_id: &str) -> Result<i64, StorageError> {
        let mut tx = self.tx.lock().await;
        let max: Option<i64> =
            sqlx::query_scalar("SELECT MAX(generation) FROM index_build_runs WHERE index_id = ?")
                .bind(index_id)
                .fetch_one(&mut **tx)
                .await
                .map_err(map_sqlx_error)?;
        Ok(max.unwrap_or(0))
    }

    async fn delete_build_run(&mut self, id: &str) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query("DELETE FROM index_build_runs WHERE id = ?")
            .bind(id)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn cache_get(&self, key: &str) -> Result<Option<SearchCacheRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query(
            "SELECT key, generation, payload_json, bytes, created_at, last_hit_at
             FROM search_result_cache WHERE key = ?",
        )
        .bind(key)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(row.map(|r| SearchCacheRow {
            key: r.get("key"),
            generation: r.get("generation"),
            payload_json: r.get("payload_json"),
            bytes: r.get("bytes"),
            created_at: r.get("created_at"),
            last_hit_at: r.get("last_hit_at"),
        }))
    }

    async fn cache_put(&mut self, row: SearchCacheRow) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO search_result_cache
                (key, generation, payload_json, bytes, created_at, last_hit_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT (key) DO UPDATE SET
                generation = excluded.generation,
                payload_json = excluded.payload_json,
                bytes = excluded.bytes,
                created_at = excluded.created_at,
                last_hit_at = excluded.last_hit_at",
        )
        .bind(&row.key)
        .bind(row.generation)
        .bind(&row.payload_json)
        .bind(row.bytes)
        .bind(&row.created_at)
        .bind(&row.last_hit_at)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn cache_touch(&mut self, key: &str, at: &str) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query("UPDATE search_result_cache SET last_hit_at = ? WHERE key = ?")
            .bind(at)
            .bind(key)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn cache_delete(&mut self, key: &str) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query("DELETE FROM search_result_cache WHERE key = ?")
            .bind(key)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn cache_stats(&self) -> Result<(i64, i64), StorageError> {
        let mut tx = self.tx.lock().await;
        let row: (i64, Option<i64>) =
            sqlx::query_as("SELECT COUNT(*), SUM(bytes) FROM search_result_cache")
                .fetch_one(&mut **tx)
                .await
                .map_err(map_sqlx_error)?;
        Ok((row.0, row.1.unwrap_or(0)))
    }

    async fn cache_enforce_cap(&mut self, max_bytes: i64) -> Result<u64, StorageError> {
        let mut tx = self.tx.lock().await;
        let deleted: u64 = sqlx::query_scalar::<_, String>(
            "WITH ranked AS (
               SELECT key,
                      SUM(bytes) OVER (ORDER BY last_hit_at DESC ROWS UNBOUNDED PRECEDING) AS running
               FROM search_result_cache
             )
             DELETE FROM search_result_cache WHERE key IN (SELECT key FROM ranked WHERE running > ?)
             RETURNING key",
        )
        .bind(max_bytes)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?
        .len() as u64;
        Ok(deleted)
    }

    async fn cache_delete_stale(&mut self, keep_generation: i64) -> Result<u64, StorageError> {
        let mut tx = self.tx.lock().await;
        let deleted: u64 = sqlx::query_scalar::<_, String>(
            "DELETE FROM search_result_cache WHERE generation != ? RETURNING key",
        )
        .bind(keep_generation)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?
        .len() as u64;
        Ok(deleted)
    }

    // ─── Phase 2 — morphology lexicon + staging ──────────────────────

    async fn upsert_dataset(&mut self, row: QuranDatasetRow) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO quran_datasets
               (id, slug, version, title, license_status, license_json, attribution,
                root_convention, tagset_version, state, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               title = excluded.title, license_status = excluded.license_status,
               license_json = excluded.license_json, attribution = excluded.attribution,
               root_convention = excluded.root_convention,
               tagset_version = excluded.tagset_version",
        )
        .bind(&row.id)
        .bind(&row.slug)
        .bind(&row.version)
        .bind(&row.title)
        .bind(&row.license_status)
        .bind(&row.license_json)
        .bind(&row.attribution)
        .bind(&row.root_convention)
        .bind(&row.tagset_version)
        .bind(&row.state)
        .bind(&row.created_at)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn get_dataset(
        &self,
        slug: &str,
        version: &str,
    ) -> Result<Option<QuranDatasetRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query(
            "SELECT id, slug, version, title, license_status, license_json, attribution,
                    root_convention, tagset_version, state, created_at
             FROM quran_datasets WHERE slug = ? AND version = ?",
        )
        .bind(slug)
        .bind(version)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(row.map(|r| decode_dataset(&r)))
    }

    async fn list_datasets(&self) -> Result<Vec<QuranDatasetRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT id, slug, version, title, license_status, license_json, attribution,
                    root_convention, tagset_version, state, created_at
             FROM quran_datasets ORDER BY slug, version",
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_dataset).collect())
    }

    async fn active_dataset(&self) -> Result<Option<QuranDatasetRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query(
            "SELECT id, slug, version, title, license_status, license_json, attribution,
                    root_convention, tagset_version, state, created_at
             FROM quran_datasets WHERE state = 'active' LIMIT 1",
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(row.map(|r| decode_dataset(&r)))
    }

    async fn set_dataset_state(&mut self, id: &str, state: &str) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query("UPDATE quran_datasets SET state = ? WHERE id = ?")
            .bind(state)
            .bind(id)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn insert_staging_batch(&mut self, row: StagingBatchRow) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO morphology_staging_batches
               (id, dataset_slug, dataset_version, adapter, source_manifest_hash,
                state, checkpoint, created_at, finished_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&row.id)
        .bind(&row.dataset_slug)
        .bind(&row.dataset_version)
        .bind(&row.adapter)
        .bind(&row.source_manifest_hash)
        .bind(&row.state)
        .bind(&row.checkpoint)
        .bind(&row.created_at)
        .bind(&row.finished_at)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn update_staging_batch(
        &mut self,
        id: &str,
        state: &str,
        checkpoint: &str,
        finished_at: Option<&str>,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "UPDATE morphology_staging_batches SET state = ?, checkpoint = ?, finished_at = ?
             WHERE id = ?",
        )
        .bind(state)
        .bind(checkpoint)
        .bind(finished_at)
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn get_staging_batch(&self, id: &str) -> Result<Option<StagingBatchRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query(
            "SELECT id, dataset_slug, dataset_version, adapter, source_manifest_hash,
                    state, checkpoint, created_at, finished_at
             FROM morphology_staging_batches WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(row.map(|r| decode_staging_batch(&r)))
    }

    async fn replace_staging_rows(
        &mut self,
        batch_id: &str,
        rows: Vec<StagedMorphRow>,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query("DELETE FROM morphology_staging_rows WHERE batch_id = ?")
            .bind(batch_id)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        for row in &rows {
            sqlx::query(
                "INSERT INTO morphology_staging_rows
                   (id, batch_id, reference, surah, ayah, token_position,
                    payload_json, native_tags_json, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&row.id)
            .bind(&row.batch_id)
            .bind(&row.reference)
            .bind(row.surah)
            .bind(row.ayah)
            .bind(row.token_position)
            .bind(&row.payload_json)
            .bind(&row.native_tags_json)
            .bind(&row.created_at)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        }
        Ok(())
    }

    async fn list_staging_rows(&self, batch_id: &str) -> Result<Vec<StagedMorphRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT id, batch_id, reference, surah, ayah, token_position,
                    payload_json, native_tags_json, created_at
             FROM morphology_staging_rows WHERE batch_id = ?
             ORDER BY surah, ayah, token_position",
        )
        .bind(batch_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_staged_row).collect())
    }

    async fn replace_alignment_rows(
        &mut self,
        batch_id: &str,
        rows: Vec<AlignmentRow>,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query("DELETE FROM morphology_alignment WHERE batch_id = ?")
            .bind(batch_id)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        for row in &rows {
            sqlx::query(
                "INSERT INTO morphology_alignment
                   (id, batch_id, direct_key, edition_id, surah, ayah, token_position,
                    alignment_kind, alignment_hash, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&row.id)
            .bind(&row.batch_id)
            .bind(&row.direct_key)
            .bind(&row.edition_id)
            .bind(row.surah)
            .bind(row.ayah)
            .bind(row.token_position)
            .bind(&row.alignment_kind)
            .bind(&row.alignment_hash)
            .bind(&row.created_at)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        }
        Ok(())
    }

    async fn list_alignment(&self, batch_id: &str) -> Result<Vec<AlignmentRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT id, batch_id, direct_key, edition_id, surah, ayah, token_position,
                    alignment_kind, alignment_hash, created_at
             FROM morphology_alignment WHERE batch_id = ? ORDER BY surah, ayah, token_position",
        )
        .bind(batch_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_alignment).collect())
    }

    async fn replace_findings(
        &mut self,
        batch_id: &str,
        rows: Vec<MorphologyFindingRow>,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query("DELETE FROM morphology_findings WHERE batch_id = ?")
            .bind(batch_id)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        for row in &rows {
            sqlx::query(
                "INSERT INTO morphology_findings
                   (id, batch_id, rule_id, severity, reference, detail, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&row.id)
            .bind(&row.batch_id)
            .bind(&row.rule_id)
            .bind(&row.severity)
            .bind(&row.reference)
            .bind(&row.detail)
            .bind(&row.created_at)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        }
        Ok(())
    }

    async fn list_findings(
        &self,
        batch_id: &str,
    ) -> Result<Vec<MorphologyFindingRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT id, batch_id, rule_id, severity, reference, detail, created_at
             FROM morphology_findings WHERE batch_id = ? ORDER BY rule_id, reference",
        )
        .bind(batch_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_finding).collect())
    }

    async fn insert_roots(&mut self, rows: Vec<LexiconRootRow>) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        for row in &rows {
            sqlx::query(
                "INSERT INTO quran_roots
                   (id, dataset_id, root, root_normalized, provenance_layer, algorithm,
                    algorithm_version, confidence, reviewer, status, corpus_generation, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&row.id)
            .bind(&row.dataset_id)
            .bind(&row.root)
            .bind(&row.root_normalized)
            .bind(&row.provenance.layer)
            .bind(&row.provenance.algorithm)
            .bind(&row.provenance.algorithm_version)
            .bind(row.provenance.confidence)
            .bind(&row.provenance.reviewer)
            .bind(&row.provenance.status)
            .bind(row.corpus_generation)
            .bind(&row.created_at)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        }
        Ok(())
    }

    async fn insert_lemmas(&mut self, rows: Vec<LexiconLemmaRow>) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        for row in &rows {
            sqlx::query(
                "INSERT INTO quran_lemmas
                   (id, dataset_id, lemma, root_id, pos_unified, pos_native, provenance_layer,
                    algorithm, algorithm_version, confidence, reviewer, status,
                    corpus_generation, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&row.id)
            .bind(&row.dataset_id)
            .bind(&row.lemma)
            .bind(&row.root_id)
            .bind(&row.pos_unified)
            .bind(&row.pos_native)
            .bind(&row.provenance.layer)
            .bind(&row.provenance.algorithm)
            .bind(&row.provenance.algorithm_version)
            .bind(row.provenance.confidence)
            .bind(&row.provenance.reviewer)
            .bind(&row.provenance.status)
            .bind(row.corpus_generation)
            .bind(&row.created_at)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        }
        Ok(())
    }

    async fn insert_analyses(&mut self, rows: Vec<TokenAnalysisRow>) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        for row in &rows {
            sqlx::query(
                "INSERT INTO quran_token_analyses
                   (id, dataset_id, edition_id, surah, ayah, token_position, analysis_index,
                    surface, lemma_id, root_id, stem, pos_unified, pos_native, features_json,
                    segments_json, provenance_layer, algorithm, algorithm_version, confidence,
                    reviewer, status, corpus_generation, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&row.id)
            .bind(&row.dataset_id)
            .bind(&row.edition_id)
            .bind(row.surah)
            .bind(row.ayah)
            .bind(row.token_position)
            .bind(row.analysis_index)
            .bind(&row.surface)
            .bind(&row.lemma_id)
            .bind(&row.root_id)
            .bind(&row.stem)
            .bind(&row.pos_unified)
            .bind(&row.pos_native)
            .bind(&row.features_json)
            .bind(&row.segments_json)
            .bind(&row.provenance.layer)
            .bind(&row.provenance.algorithm)
            .bind(&row.provenance.algorithm_version)
            .bind(row.provenance.confidence)
            .bind(&row.provenance.reviewer)
            .bind(&row.provenance.status)
            .bind(row.corpus_generation)
            .bind(&row.created_at)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        }
        Ok(())
    }

    async fn insert_morphemes(&mut self, rows: Vec<MorphemeRow>) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        for row in &rows {
            sqlx::query(
                "INSERT INTO quran_morphemes (id, analysis_id, kind, surface, features_json, created_at)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&row.id)
            .bind(&row.analysis_id)
            .bind(&row.kind)
            .bind(&row.surface)
            .bind(&row.features_json)
            .bind(&row.created_at)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        }
        Ok(())
    }

    async fn analyses_for_token(
        &self,
        dataset_id: Option<&str>,
        edition_id: &str,
        surah: i64,
        ayah: i64,
        position: i64,
    ) -> Result<Vec<TokenAnalysisRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = if let Some(dataset) = dataset_id {
            sqlx::query(
                "SELECT id, dataset_id, edition_id, surah, ayah, token_position, analysis_index,
                        surface, lemma_id, root_id, stem, pos_unified, pos_native, features_json,
                        segments_json, provenance_layer, algorithm, algorithm_version, confidence,
                        reviewer, status, corpus_generation, created_at
                 FROM quran_token_analyses
                 WHERE dataset_id = ? AND edition_id = ? AND surah = ? AND ayah = ?
                   AND token_position = ? ORDER BY analysis_index",
            )
            .bind(dataset)
            .bind(edition_id)
            .bind(surah)
            .bind(ayah)
            .bind(position)
            .fetch_all(&mut **tx)
            .await
            .map_err(map_sqlx_error)?
        } else {
            sqlx::query(
                "SELECT id, dataset_id, edition_id, surah, ayah, token_position, analysis_index,
                        surface, lemma_id, root_id, stem, pos_unified, pos_native, features_json,
                        segments_json, provenance_layer, algorithm, algorithm_version, confidence,
                        reviewer, status, corpus_generation, created_at
                 FROM quran_token_analyses
                 WHERE edition_id = ? AND surah = ? AND ayah = ? AND token_position = ?
                 ORDER BY dataset_id, analysis_index",
            )
            .bind(edition_id)
            .bind(surah)
            .bind(ayah)
            .bind(position)
            .fetch_all(&mut **tx)
            .await
            .map_err(map_sqlx_error)?
        };
        Ok(rows.iter().map(decode_analysis).collect())
    }

    async fn list_analyses(&self, dataset_id: &str) -> Result<Vec<TokenAnalysisRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT id, dataset_id, edition_id, surah, ayah, token_position, analysis_index,
                    surface, lemma_id, root_id, stem, pos_unified, pos_native, features_json,
                    segments_json, provenance_layer, algorithm, algorithm_version, confidence,
                    reviewer, status, corpus_generation, created_at
             FROM quran_token_analyses
             WHERE dataset_id = ?
             ORDER BY surah, ayah, token_position, analysis_index",
        )
        .bind(dataset_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_analysis).collect())
    }

    async fn analyses_for_root(
        &self,
        dataset_id: &str,
        root_normalized: &str,
    ) -> Result<Vec<TokenAnalysisRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT a.id, a.dataset_id, a.edition_id, a.surah, a.ayah, a.token_position,
                    a.analysis_index, a.surface, a.lemma_id, a.root_id, a.stem, a.pos_unified,
                    a.pos_native, a.features_json, a.segments_json, a.provenance_layer,
                    a.algorithm, a.algorithm_version, a.confidence, a.reviewer, a.status,
                    a.corpus_generation, a.created_at
             FROM quran_token_analyses a JOIN quran_roots r ON a.root_id = r.id
             WHERE a.dataset_id = ? AND r.root_normalized = ?
             ORDER BY a.surah, a.ayah, a.token_position, a.analysis_index",
        )
        .bind(dataset_id)
        .bind(root_normalized)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_analysis).collect())
    }

    async fn analyses_for_lemma(
        &self,
        dataset_id: &str,
        lemma: &str,
    ) -> Result<Vec<TokenAnalysisRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT a.id, a.dataset_id, a.edition_id, a.surah, a.ayah, a.token_position,
                    a.analysis_index, a.surface, a.lemma_id, a.root_id, a.stem, a.pos_unified,
                    a.pos_native, a.features_json, a.segments_json, a.provenance_layer,
                    a.algorithm, a.algorithm_version, a.confidence, a.reviewer, a.status,
                    a.corpus_generation, a.created_at
             FROM quran_token_analyses a JOIN quran_lemmas l ON a.lemma_id = l.id
             WHERE a.dataset_id = ? AND l.lemma = ?
             ORDER BY a.surah, a.ayah, a.token_position, a.analysis_index",
        )
        .bind(dataset_id)
        .bind(lemma)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_analysis).collect())
    }

    async fn list_roots(&self, dataset_id: &str) -> Result<Vec<LexiconRootRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT id, dataset_id, root, root_normalized, provenance_layer, algorithm,
                    algorithm_version, confidence, reviewer, status, corpus_generation, created_at
             FROM quran_roots WHERE dataset_id = ? ORDER BY root_normalized",
        )
        .bind(dataset_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_root).collect())
    }

    async fn list_lemmas(&self, dataset_id: &str) -> Result<Vec<LexiconLemmaRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT id, dataset_id, lemma, root_id, pos_unified, pos_native, provenance_layer,
                    algorithm, algorithm_version, confidence, reviewer, status,
                    corpus_generation, created_at
             FROM quran_lemmas WHERE dataset_id = ? ORDER BY lemma",
        )
        .bind(dataset_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_lemma).collect())
    }

    async fn insert_family_relations(
        &mut self,
        rows: Vec<FamilyRelationRow>,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        for row in &rows {
            sqlx::query(
                "INSERT INTO word_family_relations
                   (id, relation, from_kind, from_id, to_kind, to_id, explanation, dataset_id,
                    provenance_layer, algorithm, algorithm_version, confidence, reviewer, status,
                    evidence_json, corpus_generation, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&row.id)
            .bind(&row.relation)
            .bind(&row.from_kind)
            .bind(&row.from_id)
            .bind(&row.to_kind)
            .bind(&row.to_id)
            .bind(&row.explanation)
            .bind(&row.dataset_id)
            .bind(&row.provenance.layer)
            .bind(&row.provenance.algorithm)
            .bind(&row.provenance.algorithm_version)
            .bind(row.provenance.confidence)
            .bind(&row.provenance.reviewer)
            .bind(&row.status)
            .bind(&row.evidence_json)
            .bind(row.corpus_generation)
            .bind(&row.created_at)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        }
        Ok(())
    }

    async fn family_relations_for(
        &self,
        kind: &str,
        id: &str,
    ) -> Result<Vec<FamilyRelationRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT id, relation, from_kind, from_id, to_kind, to_id, explanation, dataset_id,
                    provenance_layer, algorithm, algorithm_version, confidence, reviewer, status,
                    evidence_json, corpus_generation, created_at
             FROM word_family_relations
             WHERE (from_kind = ? AND from_id = ?) OR (to_kind = ? AND to_id = ?)",
        )
        .bind(kind)
        .bind(id)
        .bind(kind)
        .bind(id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_family_relation).collect())
    }

    async fn insert_review_item(&mut self, row: MorphReviewItemRow) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO morphology_review_queue
               (id, kind, subject_json, evidence_json, status, reviewer, decided_at, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO NOTHING",
        )
        .bind(&row.id)
        .bind(&row.kind)
        .bind(&row.subject_json)
        .bind(&row.evidence_json)
        .bind(&row.status)
        .bind(&row.reviewer)
        .bind(&row.decided_at)
        .bind(&row.created_at)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn list_review_items(
        &self,
        status: &str,
    ) -> Result<Vec<MorphReviewItemRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT id, kind, subject_json, evidence_json, status, reviewer, decided_at, created_at
             FROM morphology_review_queue WHERE status = ? ORDER BY created_at",
        )
        .bind(status)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.iter().map(decode_review_item).collect())
    }

    async fn decide_review_item(
        &mut self,
        id: &str,
        reviewer: &str,
        decision: &str,
        decided_at: &str,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "UPDATE morphology_review_queue SET status = ?, reviewer = ?, decided_at = ?
             WHERE id = ? AND status = 'pending'",
        )
        .bind(decision)
        .bind(reviewer)
        .bind(decided_at)
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }
}

fn decode_token_form(row: &sqlx::sqlite::SqliteRow) -> TokenFormRow {
    TokenFormRow {
        edition_id: row.get("edition_id"),
        surah: row.get("surah"),
        ayah: row.get("ayah"),
        position: row.get("position"),
        simple: row.get("simple"),
        bare: row.get("bare"),
        hamza_folded: row.get("hamza_folded"),
        folded: row.get("folded"),
        affix_stripped: row.get("affix_stripped"),
        transliteration: row.get("transliteration"),
        phonetic: row.get("phonetic"),
        rule_set_id: row.get("rule_set_id"),
        rule_set_version: row.get("rule_set_version"),
        corpus_generation: row.get("corpus_generation"),
        provenance_id: row.get("provenance_id"),
    }
}

fn decode_ayah_form(row: &sqlx::sqlite::SqliteRow) -> AyahFormRow {
    AyahFormRow {
        edition_id: row.get("edition_id"),
        surah: row.get("surah"),
        ayah: row.get("ayah"),
        simple: row.get("simple"),
        bare: row.get("bare"),
        hamza_folded: row.get("hamza_folded"),
        folded: row.get("folded"),
        transliteration: row.get("transliteration"),
        rule_set_id: row.get("rule_set_id"),
        rule_set_version: row.get("rule_set_version"),
        corpus_generation: row.get("corpus_generation"),
        provenance_id: row.get("provenance_id"),
    }
}

fn decode_skeleton(row: &sqlx::sqlite::SqliteRow) -> SkeletonRow {
    SkeletonRow {
        edition_id: row.get("edition_id"),
        surah: row.get("surah"),
        ayah_start: row.get("ayah_start"),
        ayah_end: row.get("ayah_end"),
        skeleton: row.get("skeleton"),
        rule_set_id: row.get("rule_set_id"),
        rule_set_version: row.get("rule_set_version"),
        corpus_generation: row.get("corpus_generation"),
        provenance_id: row.get("provenance_id"),
    }
}

fn decode_profile(row: &sqlx::sqlite::SqliteRow) -> NormalizationProfileRow {
    NormalizationProfileRow {
        profile_id: row.get("profile_id"),
        version: row.get("version"),
        label: row.get("label"),
        rules_json: row.get("rules_json"),
        indexed: row.get::<i64, _>("indexed") != 0,
        heuristic: row.get::<i64, _>("heuristic") != 0,
        experimental: row.get::<i64, _>("experimental") != 0,
    }
}

fn decode_build_run(row: &sqlx::sqlite::SqliteRow) -> IndexBuildRunRow {
    IndexBuildRunRow {
        id: row.get("id"),
        index_id: row.get("index_id"),
        generation: row.get("generation"),
        corpus_generation: row.get("corpus_generation"),
        state: row.get("state"),
        doc_count: row.get("doc_count"),
        manifest_hash: row.get("manifest_hash"),
        error: row.get("error"),
        started_at: row.get("started_at"),
        finished_at: row.get("finished_at"),
    }
}

fn decode_dataset(row: &sqlx::sqlite::SqliteRow) -> QuranDatasetRow {
    QuranDatasetRow {
        id: row.get("id"),
        slug: row.get("slug"),
        version: row.get("version"),
        title: row.get("title"),
        license_status: row.get("license_status"),
        license_json: row.get("license_json"),
        attribution: row.get("attribution"),
        root_convention: row.get("root_convention"),
        tagset_version: row.get("tagset_version"),
        state: row.get("state"),
        created_at: row.get("created_at"),
    }
}

fn decode_staging_batch(row: &sqlx::sqlite::SqliteRow) -> StagingBatchRow {
    StagingBatchRow {
        id: row.get("id"),
        dataset_slug: row.get("dataset_slug"),
        dataset_version: row.get("dataset_version"),
        adapter: row.get("adapter"),
        source_manifest_hash: row.get("source_manifest_hash"),
        state: row.get("state"),
        checkpoint: row.get("checkpoint"),
        created_at: row.get("created_at"),
        finished_at: row.get("finished_at"),
    }
}

fn decode_staged_row(row: &sqlx::sqlite::SqliteRow) -> StagedMorphRow {
    StagedMorphRow {
        id: row.get("id"),
        batch_id: row.get("batch_id"),
        reference: row.get("reference"),
        surah: row.get("surah"),
        ayah: row.get("ayah"),
        token_position: row.get("token_position"),
        payload_json: row.get("payload_json"),
        native_tags_json: row.get("native_tags_json"),
        created_at: row.get("created_at"),
    }
}

fn decode_alignment(row: &sqlx::sqlite::SqliteRow) -> AlignmentRow {
    AlignmentRow {
        id: row.get("id"),
        batch_id: row.get("batch_id"),
        direct_key: row.get("direct_key"),
        edition_id: row.get("edition_id"),
        surah: row.get("surah"),
        ayah: row.get("ayah"),
        token_position: row.get("token_position"),
        alignment_kind: row.get("alignment_kind"),
        alignment_hash: row.get("alignment_hash"),
        created_at: row.get("created_at"),
    }
}

fn decode_finding(row: &sqlx::sqlite::SqliteRow) -> MorphologyFindingRow {
    MorphologyFindingRow {
        id: row.get("id"),
        batch_id: row.get("batch_id"),
        rule_id: row.get("rule_id"),
        severity: row.get("severity"),
        reference: row.get("reference"),
        detail: row.get("detail"),
        created_at: row.get("created_at"),
    }
}

fn decode_provenance(row: &sqlx::sqlite::SqliteRow) -> LexiconProvenance {
    LexiconProvenance {
        layer: row.get("provenance_layer"),
        algorithm: row.get("algorithm"),
        algorithm_version: row.get("algorithm_version"),
        confidence: row.get("confidence"),
        reviewer: row.get("reviewer"),
        status: row.get("status"),
    }
}

fn decode_root(row: &sqlx::sqlite::SqliteRow) -> LexiconRootRow {
    LexiconRootRow {
        id: row.get("id"),
        dataset_id: row.get("dataset_id"),
        root: row.get("root"),
        root_normalized: row.get("root_normalized"),
        provenance: decode_provenance(row),
        corpus_generation: row.get("corpus_generation"),
        created_at: row.get("created_at"),
    }
}

fn decode_lemma(row: &sqlx::sqlite::SqliteRow) -> LexiconLemmaRow {
    LexiconLemmaRow {
        id: row.get("id"),
        dataset_id: row.get("dataset_id"),
        lemma: row.get("lemma"),
        root_id: row.get("root_id"),
        pos_unified: row.get("pos_unified"),
        pos_native: row.get("pos_native"),
        provenance: decode_provenance(row),
        corpus_generation: row.get("corpus_generation"),
        created_at: row.get("created_at"),
    }
}

fn decode_analysis(row: &sqlx::sqlite::SqliteRow) -> TokenAnalysisRow {
    TokenAnalysisRow {
        id: row.get("id"),
        dataset_id: row.get("dataset_id"),
        edition_id: row.get("edition_id"),
        surah: row.get("surah"),
        ayah: row.get("ayah"),
        token_position: row.get("token_position"),
        analysis_index: row.get("analysis_index"),
        surface: row.get("surface"),
        lemma_id: row.get("lemma_id"),
        root_id: row.get("root_id"),
        stem: row.get("stem"),
        pos_unified: row.get("pos_unified"),
        pos_native: row.get("pos_native"),
        features_json: row.get("features_json"),
        segments_json: row.get("segments_json"),
        provenance: decode_provenance(row),
        corpus_generation: row.get("corpus_generation"),
        created_at: row.get("created_at"),
    }
}

fn decode_family_relation(row: &sqlx::sqlite::SqliteRow) -> FamilyRelationRow {
    FamilyRelationRow {
        id: row.get("id"),
        relation: row.get("relation"),
        from_kind: row.get("from_kind"),
        from_id: row.get("from_id"),
        to_kind: row.get("to_kind"),
        to_id: row.get("to_id"),
        explanation: row.get("explanation"),
        dataset_id: row.get("dataset_id"),
        provenance: decode_provenance(row),
        status: row.get("status"),
        evidence_json: row.get("evidence_json"),
        corpus_generation: row.get("corpus_generation"),
        created_at: row.get("created_at"),
    }
}

fn decode_review_item(row: &sqlx::sqlite::SqliteRow) -> MorphReviewItemRow {
    MorphReviewItemRow {
        id: row.get("id"),
        kind: row.get("kind"),
        subject_json: row.get("subject_json"),
        evidence_json: row.get("evidence_json"),
        status: row.get("status"),
        reviewer: row.get("reviewer"),
        decided_at: row.get("decided_at"),
        created_at: row.get("created_at"),
    }
}
