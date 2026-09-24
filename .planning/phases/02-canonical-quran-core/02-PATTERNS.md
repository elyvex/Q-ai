# Phase 2: Canonical Quran Core - Pattern Map

**Mapped:** 2026-09-25
**Files analyzed:** 28 (new + modified)
**Analogs found:** 25 / 28

> This is a brownfield hardening phase. Every "new" file copies an existing
> in-tree pattern; the gap register QC-01…QC-13 names the exact seam. No analog
> below is a gitignored mirror — every path was confirmed with
> `git ls-files -- <path>` (tracked source), and migrations live in the tracked
> `migrations/sqlite/` tree.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `migrations/sqlite/0020_canonical_write_fence.up.sql` (new) | migration | DDL / triggers | `migrations/sqlite/0008_quran_structure.up.sql` + `0007_quran_editions.up.sql` | exact (trigger block) |
| `crates/storage-sqlite/tests/quran.rs` | test | integration (real SQLite) | itself (`canonical_triggers_abort_raw_writes_with_codes`) | exact |
| `crates/storage-sqlite/src/quran.rs` | service | CRUD / transaction | itself (`activate_edition` move list) | exact |
| `crates/storage/src/quran.rs` | port trait + test | request-response / source-scan | itself (`canonical_write_surface_is_gated_to_three_mutators`) | exact |
| `crates/provenance/src/lib.rs` | utility / gate | request-response | itself (`ApprovalToken`, `CanonicalWriter`) | exact (self) |
| `crates/application/src/quran.rs` | service | transaction / request-response | itself (`activate_edition`, `import_translations`) | exact |
| `crates/application/src/quran_cli.rs` | controller (CLI) | request-response | itself (`cmd_hashes`, `cmd_doctor_quran`, `cmd_validate`) | exact |
| `crates/cli/src/quran.rs` | controller (CLI wiring) | request-response | itself (`QuranAction::Hashes`, `confirm`) | exact |
| `crates/application/src/quran_doctor.rs` | service (read-only checks) | request-response | itself (`run_quran_checks` check list) | exact |
| `crates/quran-corpus/src/import.rs` | service (pipeline) | batch / transform | itself (`compared`, `compare_reference`) | exact |
| `crates/quran-corpus/src/hashing.rs` | utility | transform (hash) | itself (`text_hash`, `feed`/`finish`) | exact |
| `crates/quran-core/src/enums.rs` | model | enum | itself (`EditionSelector`) | exact |
| `crates/quran-core/src/edition.rs` | model | struct | itself (`QuranEdition`) | exact |
| `crates/application/src/quran_reader.rs` | service (read) | CRUD / cache | itself (`resolve_edition` / `build_view`) | exact |
| `crates/citations/src/lib.rs` | utility (domain) | transform / verdict | itself (`QuotationVerdict`, `CitationError`) | exact |
| `crates/application/src/quran_tools.rs` | service (answer path) | request-response | itself (`ReaderCitationSource`) | exact |
| `crates/server/src/api.rs` | controller (HTTP) | request-response | itself (`api_citation`) | role-match |
| `crates/application/tests/quran_import.rs` | test | integration | itself (`rejected_activations_leave_canonical_state_untouched`) | exact |
| `crates/application/tests/quran_translation.rs` | test | integration | itself (`import_translations_persists_*`) | exact |
| `crates/application/tests/quran_doctor.rs` | test | integration | itself (`deep_scan_of_the_fixture_has_no_failures`) | exact |
| `crates/cli/tests/quran/verify.trycmd` (new) | test | CLI snapshot | `crates/cli/tests/quran/read_flow.trycmd` | exact |
| `crates/cli/tests/quran/read_flow.trycmd` | test | CLI snapshot | itself | exact |
| `fixtures/quran/test-edition-rich/manifest.json` (new) | fixture | file-I/O (static data) | `fixtures/quran/test-edition-min/manifest.json` | exact (data) |
| `fixtures/quran/test-edition-rich/reference.json` (new) | fixture | file-I/O (static data) | `fixtures/quran/test-edition-min/manifest.json` | exact (data) |
| `fixtures/quran/golden/ayah_texts.jsonl` | fixture | file-I/O (golden data) | itself | exact |
| `xtask/src/coverage.rs` | config / gate | batch | itself (`THRESHOLDS`) | exact |
| Committed corpus-integrity report artifact (e.g. `docs/06-progress/corpus-integrity-report.json`) | artifact | file-I/O | `cmd_doctor_quran` JSON shape + `cmd_validate --report` writer | role-match |
| Owner gates OD-01/OD-02/OD-03 records in phase docs | doc | — | Phase 1 plan evidence pattern | no code analog |

---

## Pattern Assignments

### A. Fixture + integrity-evidence surface (D-08, D-10, D-11; QC-02, QC-03, QC-04, QC-10, QC-12, QC-13)

#### `fixtures/quran/test-edition-rich/manifest.json` + companion `reference.json` (fixture, file-I/O)

**Analog:** `fixtures/quran/test-edition-min/manifest.json`

The manifest is a static JSON document consumed by `JsonAdapter`; keep the same
top-level shape and add the D-08 breadth (more surahs/ayahs, basmala inside an
ayah body, pause marks U+06D6…U+06ED, mark-separated token, long ayah, page
boundary, muqattaʿat, repeated refrain, U+0670).

**Identity/shape header pattern** (`fixtures/quran/test-edition-min/manifest.json:1-23`):
```json
{
  "format": "qai.quran.edition",
  "format_version": 1,
  "edition": {
    "slug": "test-edition-min",
    "name": "Synthetic test edition (non-canonical, ADR-0101 fallback)",
    "script": "uthmani",
    "riwayah": null,
    "qiraah": null,
    "language": "ar",
    "verse_numbering_scheme": "hafs",
    "unicode_normalization": "nfc",
    "basmala_policy": "per_surah",
    "publisher": null,
    "version": "0.1.0",
    "synthetic": true
  },
  "expected": {
    "surah_count": 5,
    "ayah_count": 14,
    "reference_corpus_id": null,
    "reference_text_hash": null
  },
  "surahs": [ ... ],
  "ayahs": [ ... ]
}
```

**Hard rule — the fixture must stay `synthetic: true`.** There is an existing
assertion that enforces exactly this (`crates/quran-corpus/tests/fixtures.rs:106-118`):
```rust
let source: EditionSource = JsonAdapter.parse(MIN_MANIFEST).unwrap();
assert_eq!(source.edition.slug, "test-edition-min");
// OD-01 B-track: the pipeline-exercise fixture is flagged synthetic and
// must stay that way — it is never canonical.
assert!(source.edition.synthetic, "test-edition-min must stay synthetic");
```
Add the same `synthetic: true` shape assertion for any new fixture.

**Companion reference edition (QC-03/QC-04 operator path).** The reference is
just another `EditionSource` supplied to `ImportOptions.reference`
(`crates/quran-corpus/src/import.rs:142-160`); it needs a declared
`reference_corpus_id` + `reference_text_hash` pin in the import source's
`expected` block. The exact comparison mechanics to satisfy are
`crates/quran-corpus/src/import.rs:925-1045`.

#### `crates/application/src/quran_doctor.rs` — QC-10 reference check (service, request-response)

**Analog:** the same file's check-construction pattern.

The reference check is currently a hardcoded `warn`
(`crates/application/src/quran_doctor.rs:433-442`). Replace it with a
state-derived `Pass`/`Skipped`/`Fail`, reusing the existing helper style:
```rust
// Current (QC-10 seam) — unconditional:
checks.push(warn(
    "quran.reference_corpus",
    "no reference corpus configured; QV-015 skips (ADR-0114 pending)".to_string(),
    "configure a reference corpus and sign-off procedure",
));
```
The sibling state-derived checks that show the target pattern are
`quran.corpus_generation` and `quran.license_status`
(`crates/application/src/quran_doctor.rs:433-457`):
```rust
checks.push(pass(
    "quran.corpus_generation",
    format!("current generation is {}", active.corpus_generation),
));
let license_status = serde_json::from_str::<serde_json::Value>(&edition.license_json)
    .ok()
    .and_then(|value| value.get("status").and_then(|s| s.as_str()).map(str::to_string))
    .unwrap_or_default();
checks.push(if license_status != "Unknown" && !license_status.is_empty() {
    pass("quran.license_status", format!("active edition license: {license_status}"))
} else {
    warn("quran.license_status", "active edition license is unknown".to_string(),
         "record the dataset license (ADR-0101)")
});
```
Doctor is **read-only** — do not write probes or files inside it
(`.agent/coding-rules.md`). The QV-015 skip state must be read from the
persisted validation report / staging state, never fabricated.

#### `crates/application/src/quran_cli.rs` — integrity/verify surface (QC-03) + hash on lookup line (QC-02)

**Analog:** `cmd_doctor_quran` (`crates/application/src/quran_cli.rs:1415-1457`)
for a read-only check surface, and `cmd_validate` (`:674-746`) for the
`--report` artifact writer.

Doctor-surface loop + JSON + exit mapping to copy:
```rust
pub async fn cmd_doctor_quran(db_path: &str, deep: bool) -> CommandOutput {
    let db = match storage_sqlite::SqliteDatabase::open_read_only(db_path).await {
        Ok(db) => db,
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
    };
    let checks = match super::quran_doctor::run_quran_checks(&db, deep).await {
        Ok(checks) => checks,
        Err(err) => return CommandOutput::err(exit::INTERNAL, err.to_string()),
    };
    // ... human rendering of id/status/summary/remedy/next ...
    let json = serde_json::json!({ "checks": [ /* id,status,summary,remedy,next_command */ ] });
    let exit = if checks.iter().any(|c| c.status == CheckLevel::Fail) {
        exit::VALIDATION
    } else { exit::OK };
    CommandOutput { exit, human, json }
}
```

Report-artifact writer to copy (`cmd_validate`, `crates/application/src/quran_cli.rs:709-714`):
```rust
if let Some(path) = report_path {
    let json = serde_json::to_string_pretty(&report).unwrap_or_default();
    if let Err(err) = std::fs::write(path, json) {
        return CommandOutput::err(exit::INTERNAL, format!("cannot write report: {err}"));
    }
}
```

**Important constraint:** `qai quran hashes` explicitly does *not* recompute
(`crates/application/src/quran_cli.rs:1188-1198`). The new verify command must
reuse `run_quran_checks` / `validate_staged`, **not** add a second reader.

`CommandOutput` shape to return (`crates/application/src/quran_cli.rs:41-66`):
```rust
pub struct CommandOutput { pub exit: i32, pub human: String, pub json: serde_json::Value }
impl CommandOutput {
    pub fn ok(human: String, json: serde_json::Value) -> Self { ... }
    pub fn err(exit: i32, message: String) -> Self { ... }
}
```

**QC-02 (byte-exact lookup + hash on the human line).** `cmd_get` builds the
human line from `arabic_text` + reference. The stored `text_hash` is already in
`AyahView` JSON (`read_flow.trycmd:62-65`), so surface it on the human line or
assert it in the existing snapshot — do not add a reader.

#### `crates/cli/src/quran.rs` — new verb wiring (controller)

**Analog:** `QuranAction::Hashes` variant + dispatch arm, and the `confirm`
wrapper (`crates/cli/src/quran.rs:135-139, 653-1010`).

Variant pattern:
```rust
/// Recompute and compare stored hashes.
Hashes {
    /// `slug@version`.
    edition: String,
},
```
Dispatch arm (note: read-only verbs call `application::quran_cli::cmd_*`
directly; mutating verbs are wrapped in `confirm`):
```rust
QuranAction::Hashes { edition } => {
    application::quran_cli::cmd_hashes(db_path, &edition).await
}
```
```rust
QuranAction::Activate { edition } => {
    confirm("activate", &edition, yes,
        application::quran_cli::cmd_activate(db_path, &edition)).await
}
```
The `confirm` helper (`crates/cli/src/quran.rs:1010-1035`) enforces
`--yes`/terminal-prompt for `activate`/`rollback`/`deprecate`. A new read-only
`verify` verb does not use it.

#### `crates/quran-corpus/src/import.rs` — QC-04 typed comparison wiring (pipeline)

**Analog:** `compared()` itself, plus `differ.rs` re-exports.

Current `compared()` uses raw `compare_reference` and embeds the vocabulary
name only (`crates/quran-corpus/src/import.rs:982-1002`):
```rust
findings.push(Finding::new(
    "QV-015", Severity::Info,
    format!("quran-edition:{reference_id}@{version}"),
    serde_json::json!({
        "method": "exact-ayah-bytes-v1",
        "comparison_kind": "reference",
        "classification_vocabulary": crate::differ::CLASSIFICATION_VOCABULARY,
        "normalization_applied": [],
        "reference_corpus_id": reference_id,
        "reference_version": version,
        "reference_text_hash": hash,
        "reference_snapshot_hash": snapshot_hash,
        "outcome": if fatal { "fail" } else { "pass" },
    }).to_string(),
));
```
`diff_ayahs_typed` / `ComparisonKind` / `DifferenceClass` exist and are unit-tested
but unwired. Per ADR-0114 §4, QV-015 pass/fail stays **byte-only**; the typed
classification is report metadata. Either call `diff_ayahs_typed` to attach a
`DifferenceClass` per difference, or record explicitly that v1 QV-015 is
byte-only. Do **not** invent a new vocabulary. The skip shape to preserve
(`crates/quran-corpus/src/import.rs:176-183`):
```rust
let Some(reference) = reference else {
    return vec![Finding::new("QV-015", Severity::Info, "edition",
        "reference-corpus comparison skipped: no reference corpus configured")];
};
```
A skipped family must be classified **skipped**, never passed (D-10).

#### `xtask/src/coverage.rs` — QC-12 threshold reconciliation (gate)

**Analog:** `THRESHOLDS` (`xtask/src/coverage.rs:26-34`).
```rust
pub const THRESHOLDS: &[Threshold] = &[
    Threshold { krate: "domain", path_prefix: "crates/domain/", min_percent: 85.0 },
    Threshold { krate: "provenance", path_prefix: "crates/provenance/", min_percent: 85.0 },
    Threshold { krate: "audit", path_prefix: "crates/audit/", min_percent: 85.0 },
    Threshold { krate: "sources", path_prefix: "crates/sources/", min_percent: 85.0 },
    Threshold { krate: "config", path_prefix: "crates/config/", min_percent: 75.0 },
    Threshold { krate: "jobs", path_prefix: "crates/jobs/", min_percent: 75.0 },
    Threshold { krate: "storage-sqlite", path_prefix: "crates/storage-sqlite/", min_percent: 75.0 },
];
```
Either add `quran-core`/`quran-corpus`/`citations` rows here (`>=85%`/`>=90%` per
legacy §4) or record OD-07's floors as unimplemented and re-scope. The unit
tests in the same file (`coverage_by_crate`, `violations`) are the pattern for
verifying the new rows; the gate entry point is `run()` (`:89-116`).

#### `crates/cli/tests/quran/verify.trycmd` (new) — CLI snapshot

**Analog:** `crates/cli/tests/quran/read_flow.trycmd`, driven by
`crates/cli/tests/quran.rs:11-20`.
```rust
fn run_cases(pattern: &str) {
    let dir = tempfile::tempdir().unwrap();
    trycmd::TestCases::new()
        .default_bin_name("qai")
        .env("QAI_DATA_DIR", dir.path().to_str().unwrap())
        .case("tests/quran/*.toml")
        .case(pattern);
    drop(dir);
}
```
Register the new file with a `#[test]` in `crates/cli/tests/quran.rs`. Each
`.trycmd` file gets its own `QAI_DATA_DIR`; begin with `$ qai db migrate`, then
`import`/`activate`, then the new verb. Exit-code assertions use the `? N` form
(`read_flow.trycmd:244-257`):
```
$ qai quran activate test-edition-min@0.1.0 --yes
? 6
error: no staged edition test-edition-min@0.1.0
```

#### Committed corpus-integrity report artifact (D-11, artifact, file-I/O)

**No direct analog** — no integrity artifact exists in the tree. Use the JSON
shape produced by `cmd_doctor_quran` (`crates/application/src/quran_cli.rs:1439-1450`)
and write it with the `cmd_validate --report` pattern (`:709-713`). Suggested
location: `docs/06-progress/` (the project's progress/evidence tree per AGENTS.md).

---

### B. Canonical-write fence (D-13, D-14; QC-05, QC-06)

#### `migrations/sqlite/0020_canonical_write_fence.up.sql` (new) (migration, DDL/triggers)

**Analog:** `migrations/sqlite/0008_quran_structure.up.sql:88-96` (compact trigger
form) and `migrations/sqlite/0007_quran_editions.up.sql:47-52` (multi-line form);
append-many-triggers form in `0013_quran_normalization.up.sql:25-47`.

**Hard rule (ADR-0002 / Pitfall 5):** migrations are append-only and
checksum-verified. **Never edit 0007–0019.** Latest applied is `0019`, so the new
file is `0020`. `cargo run -q -p xtask -- migrate-check` is the gate.

Compact trigger form to copy for `quran_surahs`/`quran_token_separators`/etc.:
```sql
CREATE TRIGGER trg_ayah_no_update BEFORE UPDATE ON quran_ayahs
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0001: canonical ayah text is immutable'); END;
CREATE TRIGGER trg_ayah_no_delete BEFORE DELETE ON quran_ayahs
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0003: canonical ayah rows cannot be deleted'); END;
```
Existing codes in use: `QAI-QUR-0001` (ayah update), `0002` (edition
identity/hash update), `0003` (ayah delete), `0004` (token update), `0005`
(token delete). New tables need **new** codes (`QAI-QUR-0006…`), one per
table/verb, and the test table below must assert each.

Multi-line form (used where a message is repeated across verbs):
```sql
CREATE TRIGGER trg_edition_immutable_hashes
BEFORE UPDATE OF text_hash, structure_hash, token_order_hash, slug, version
ON quran_editions
BEGIN
  SELECT RAISE(ABORT, 'QAI-QUR-0002: edition identity and hashes are immutable');
END;
```

Untriggered canonical tables to fence: `quran_surahs`,
`quran_token_separators`, `quran_segments`, `quran_divisions`,
`translation_editions`, `translation_passages`, `word_glosses`; plus a DELETE
trigger on `quran_editions` (only UPDATE is covered today).

**QC-11 primary/default flag + QC-09 upstream identity columns (same migration).**
`quran_editions` has no primary flag and no upstream/license columns
(`migrations/sqlite/0007_quran_editions.up.sql:3-35`). **There is no in-tree
`ALTER TABLE … ADD COLUMN` precedent** (grep of `migrations/sqlite/*.up.sql`
returns none) — both options are valid:
1. `ALTER TABLE quran_editions ADD COLUMN is_primary INTEGER NOT NULL DEFAULT 0;`
   (SQLite supports it; the existing `trg_edition_immutable_hashes` fires only
   on its named UPDATE columns, so writes to new columns are unaffected).
2. A sidecar table (precedent: every `quran_*` table is standalone).

The primary flag feeds the existing `quran_active_edition` singleton — do not
redefine the pointer (`migrations/sqlite/0007_quran_editions.up.sql:37-45`).

#### `crates/storage-sqlite/tests/quran.rs` — QC-05 trigger test extension (test, integration)

**Analog:** the same file's two trigger tests.

Extend the statement table in `canonical_triggers_abort_raw_writes_with_codes`
(`crates/storage-sqlite/tests/quran.rs:248-258`) one row per newly triggered
table + code:
```rust
for (sql, code) in [
    ("UPDATE quran_ayahs SET text = 'x' WHERE edition_id = 'ed-1'", "QAI-QUR-0001"),
    ("DELETE FROM quran_ayahs WHERE edition_id = 'ed-1'", "QAI-QUR-0003"),
    ("UPDATE quran_tokens SET surface = 'x' WHERE edition_id = 'ed-1'", "QAI-QUR-0004"),
    ("DELETE FROM quran_tokens WHERE edition_id = 'ed-1'", "QAI-QUR-0005"),
    ("UPDATE quran_editions SET text_hash = 'x' WHERE id = 'ed-1'", "QAI-QUR-0002"),
] {
    let err = sqlx::query(sql).execute(&pool).await.unwrap_err();
    let message = err.to_string();
    assert!(message.contains(code), "{sql} must abort with {code}, got: {message}");
}
```
Extend the expected-name list in `canonical_tables_declare_the_trigger_set`
(`:270-283`) to the full expected trigger set:
```rust
let rows = sqlx::query("SELECT name FROM sqlite_master WHERE type = 'trigger'")
    .fetch_all(&pool).await.unwrap();
let names: Vec<String> = rows.iter().map(|r| r.get("name")).collect();
for expected in ["trg_ayah_no_update", /* ... full set ... */] {
    assert!(names.iter().any(|n| n == expected), "missing trigger {expected}");
}
```
The `migrated_db()` + raw `SqlitePoolOptions` fixture helpers are already at the
top of this file.

#### `crates/storage-sqlite/src/quran.rs` — QC-01 segments copy (service, transaction)

**Analog:** the move list inside `activate_edition`
(`crates/storage-sqlite/src/quran.rs:676-707`).

`quran_stg_segments` → `quran_segments` is omitted, so canonical segments stay
empty. Either add a tuple to the copied-table list or record explicitly that
segments are staging-only/reserved for the morphology phase (Open Question 5).
Copy pattern:
```rust
for (staging, canonical, columns) in [
    ("quran_stg_surahs", "quran_surahs", "edition_id, number, ..."),
    ("quran_stg_ayahs", "quran_ayahs", "edition_id, surah, ayah, text, ..."),
    // ... tokens, token_separators, divisions ...
] {
    let sql = format!(
        "INSERT INTO {canonical} ({columns}) SELECT {columns} FROM {staging} WHERE import_run_id = ?"
    );
    sqlx::query(&sql).bind(run_id).execute(&mut **tx).await.map_err(map_sqlx_error)?;
}
```
Whole move + pointer flip + `DELETE`/`INSERT quran_active_edition` with
`generation = current + 1` is one transaction (`:631-748`).

#### `crates/provenance/src/lib.rs` — QC-06 wire-or-retire (utility/gate)

**Analog:** the file itself.

`ApprovalToken` is publicly constructible (`crates/provenance/src/lib.rs:155-165`)
and `CanonicalWriter` has no implementor (`:201-216`). Two acceptable
resolutions:
- **Wire:** gate token construction behind the persisted granted approval row,
  mirroring `check_approval` (`crates/application/src/quran.rs:238-260`), and
  route `activate_edition`/`rollback_edition` through it.
- **Retire:** record an ADR-level change; do not leave the `.agent/coding-rules.md`
  invariant as dead code.

The vacuous test to fix/replace (`crates/provenance/src/lib.rs:295-303`):
```rust
#[test]
fn approval_token_cannot_be_constructed_directly() {
    let token = ApprovalToken::new(ApprovalId::new(), "urn:qai:source:test".to_string(), PrincipalId::new());
    assert_eq!(token.approval_id().to_string(), token.approval_id().to_string()); // vacuous
}
```

#### `crates/storage/src/quran.rs` — gated-mutator surface test (port + source-scan)

**Analog:** `canonical_write_surface_is_gated_to_three_mutators` and
`gated_mutators_fail_closed_without_backend` (`crates/storage/src/quran.rs:1902-1957`).
Extend the source-scan list if the fence adds new public canonical mutators; do
**not** add an `insert_ayah`/`insert_surah`/`write_canonical` method (the test
forbids it by name).

#### `crates/application/tests/quran_import.rs` — QC-01 rollback rejection + QC-06 importer-no-token (test, integration)

**Analog:** `rejected_activations_leave_canonical_state_untouched`
(`crates/application/tests/quran_import.rs:688-767`) and
`rollback_service_restores_the_prior_version` (`:769-819`).

Copy the rejection harness for rollback — missing/denied/mismatched approval
must leave the active pointer unchanged:
```rust
// Missing approval.
let err = activate_edition(&db, "test-edition-min", "0.1.0", &principal(), "missing", &timestamp())
    .await.unwrap_err();
assert!(matches!(err, ActivationError::ApprovalMissing { .. }));
assert_no_canonical(&db).await;

// Denied and mismatched approvals:
uow.sources().insert_approval(storage::repository::ApprovalRow {
    id: "appr-denied".into(),
    subject_urn: V1_URN.into(),
    kind: "CanonicalChange".into(),
    requested_by: Some(PRINCIPAL.into()),
    decided_by: Some(PRINCIPAL.into()),
    decision: Some("denied".into()),
    request_payload: "{}".into(),
    decision_note: None,
    requested_at: CREATED_AT.into(),
    decided_at: Some(CREATED_AT.into()),
}).await.unwrap();
```
The `assert_no_canonical` helper (`:681-686`) and the v1→v2→rollback sequencing
(`:769-819`) are the templates. Add a rollback **cache-invalidation** test
(no stale v2 text after returning to v1) and an importer-holds-no-`ApprovalToken`
assertion for D-13(c)/QC-06.

#### `crates/application/src/quran.rs` — activation/rollback + QC-01 (service)

**Analog:** `check_approval` + `activate_edition` + `rollback_edition`
(`crates/application/src/quran.rs:238-343, 429-468`). `check_approval` is the
behavior to preserve (granted AND exact URN):
```rust
if approval.decision.as_deref() != Some("approved") {
    return Err(ActivationError::ApprovalNotGranted { id: approval_id.to_string() });
}
if approval.subject_urn != expected_subject {
    return Err(ActivationError::ApprovalSubjectMismatch { id, actual, expected });
}
```
One unit of work: check approval → storage mutator → `audit_activation` →
`uow.commit()` (`:301-343`).

---

### C. Quotation hard-failure wiring (D-15; QC-07)

#### `crates/citations/src/lib.rs` — verdict→hard-failure mapping (domain utility)

**Analog:** the `QuotationVerdict` enum + `CitationError::code()`
(`crates/citations/src/lib.rs:52-117`).

`QuotationVerdict` (`:53-77`) already encodes the outcome; add a shared mapping
(the code-stable diagnostic pattern):
```rust
impl CitationError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidReference { .. } => "QAI-QUR-0321",
            Self::Backend { .. } => "QAI-QUR-0322",
        }
    }
}
```
Add e.g. `QuotationVerdict::is_hard_failure()` (or a `require_exact` helper)
covering `Mismatch`, `LocationNotFound`, `EditionNotFound`, `AccessDenied` →
failure; `ExactMatch` / `MatchAfterWhitespaceNormalization` → success. Note
`MatchAfterDeclaredNormalization` is unreachable in v1 (`:301-312`) — do not
claim it. The resolver entry points are `resolve` (`:175-222`) and
`verify_quotation` (`:224-233`).

#### `crates/application/src/quran_cli.rs` — verifying surface (QC-07)

**Analog:** a read-only command like `cmd_resolve` (`:297-325`) for parse +
bounds; `map_reader_error` / `map_activation_error` (`:75-97`) for the
exit-code mapping convention. A new verb accepts an externally supplied
quotation plus a citation, calls `CitationResolver::verify_quotation`, and maps
a non-`ExactMatch` verdict to a hard failure via the new helper.

#### `crates/application/src/quran_tools.rs` — answer-path enforcement (QC-07)

**Analog:** `ReaderCitationSource` + `persist_citation`
(`crates/application/src/quran_tools.rs:108-205`). This is where a `CitationResolver`
is already constructible; wire the verdict mapping into the tool result path.
Per F-6/Pitfall 4, the direct-read paths (`cmd_get`, HTTP `AyahView`) are
**structurally exempt** — they serve from the source of truth and cannot
mismatch; record that rationale explicitly rather than wrapping them.

#### `crates/server/src/api.rs` — optional HTTP verifying endpoint (QC-07)

**Analog:** `api_citation` (`crates/server/src/api.rs:584-598, 1531-1534`), which
currently calls `resolve_stored`. A `verify_quotation` endpoint (accepting a
supplied quotation) is a role-match extension; keep the response shape and
error-code mapping consistent with the existing handler.

#### `crates/application/tests/quran_tools.rs` — hard-failure mapping test (QC-07)

**Analog:** the existing `verify_quotation` assertion (`:140-160`): a tampered
string returns `Mismatch`, the real text returns `ExactMatch`. Add an assertion
on the new verdict→error mapping (non-`ExactMatch` → typed `QAI-QUR-*` diagnostic
+ non-zero exit).

#### `crates/cli/tests/quran/verify.trycmd` — CLI exit-code snapshot (QC-07)

Same trycmd pattern as QC-03 (see above); assert the hard-failure exit code via
`? N`.

---

### D. Translation layer + primary/default (D-04, D-07; QC-08, QC-09, QC-11)

#### `crates/quran-corpus/src/hashing.rs` — additive translation recipe (QC-08)

**Analog:** `text_hash` + the frozen `feed`/`finish` primitives
(`crates/quran-corpus/src/hashing.rs:27-62`).

**Frozen rule (D-12):** never touch `qai-text-hash-v1`. Add a new,
domain-separated recipe (e.g. `qai-translation-hash-v1`):
```rust
fn feed(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}
fn finish(hasher: Sha256) -> ContentHash {
    ContentHash { algorithm: HashAlgorithm::Sha256, hex: format!("{:x}", hasher.finalize()) }
}
pub fn text_hash(slug: &str, version: &str, texts_in_order: &[&str]) -> ContentHash {
    let mut hasher = Sha256::new();
    feed(&mut hasher, b"qai-text-hash-v1");
    feed(&mut hasher, slug.as_bytes());
    feed(&mut hasher, version.as_bytes());
    for text in texts_in_order { feed(&mut hasher, text.as_bytes()); }
    finish(hasher)
}
```
The domain-separation test (`:179-206`) is the pattern for proving the new
recipe differs from the three v1 digests. Store form is `sha256:<hex>` via
`tagged` (`:44-50`).

#### `crates/application/src/quran.rs` — compute translation hash + persist upstream identity/license (QC-08, QC-09)

**Analog:** `import_translations` (`crates/application/src/quran.rs:692-858`).

QC-08 seam — `text_hash` is currently written empty (`:805`):
```rust
uow.quran().insert_translation_edition(storage::quran::TranslationEditionRow {
    // ...
    license_json,
    trust_level: "ImportedUnverified".to_string(),
    source_version_id: source_version_id.to_string(),
    text_hash: String::new(),   // <-- QC-08: compute over passages in (surah, ayah) order
    status: "Staged".to_string(),
    imported_at: at.to_string(),
}).await.map_err(ActivationError::storage)?;
```
QC-09 seam — license is synthesised as `Unknown` (`:783-791`); persist the
manifest-declared license + `upstream_edition_slug` instead of dropping them.
`EditionMeta` already parses `upstream_edition_slug`, `qai_edition_id`,
`license`, `verified_by`, `source` (`crates/quran-corpus/src/format.rs:46-95`) —
they are simply not carried to the canonical row. `cmd_import` also hardcodes
`license_status: "Unknown"` + a synthetic license JSON
(`crates/application/src/quran_cli.rs:641-652`).

#### `crates/application/tests/quran_translation.rs` — QC-08 assertion (test)

**Analog:** `import_translations_persists_attributed_edition_and_passages`
(`crates/application/tests/quran_translation.rs:36-77`). Add an assertion that
`edition.text_hash` is a 64-hex `sha256:` value (currently only slug/version/
translator/language/status/alignment are asserted).

#### `crates/quran-core/src/enums.rs` — primary/default selector (QC-11)

**Analog:** `EditionSelector` (`crates/quran-core/src/enums.rs:114-129`).
```rust
pub enum EditionSelector {
    Active,
    Slug(String),
    Pinned { slug: String, version: SemVer },
}
```
Add a `Primary` variant (or record the primary flag without one, per F-8) that
feeds the existing pointer — "primary default" ≠ "only edition". Keep the
serde `snake_case` convention.

#### `crates/quran-core/src/edition.rs` — primary/default field (QC-11)

**Analog:** `QuranEdition` (`crates/quran-core/src/edition.rs:37-96`). Add an
`is_primary` field parallel to existing policy fields; `is_readable()` (`:91-96`)
shows the derived-method style.

#### `crates/application/src/quran_reader.rs` — resolve the primary selector (QC-11)

**Analog:** `resolve_edition` / selector arms (`crates/application/src/quran_reader.rs:363-407`)
and `build_view` byte-exact construction (`:454-482`). Add the primary arm
without introducing a second construction path that omits `text_hash`.

#### `crates/storage/src/quran.rs` + `crates/storage-sqlite/src/quran.rs` — new-column plumbing (QC-09, QC-11)

**Analog:** the existing `QuranEditionRow` field mapping and the
`INSERT INTO quran_editions … SELECT … FROM quran_stg_editions` statement
(`crates/storage-sqlite/src/quran.rs:657-674`). Add the new columns to both the
staging insert path and the canonical move; keep the same bound-parameter /
column-list style.

---

## Shared Patterns

### Error taxonomy: stable `QAI-<NS>-NNNN` codes → CLI exit
**Source:** `crates/application/src/quran_cli.rs:20-108`,
`crates/provenance/src/lib.rs:250-274`, `crates/citations/src/lib.rs:109-117`.
**Apply to:** every new command, trigger, and verdict mapping.
```rust
pub mod exit {
    pub const OK: i32 = 0; pub const GENERIC: i32 = 1; pub const USAGE: i32 = 2;
    pub const VALIDATION: i32 = 3; pub const POLICY: i32 = 4; pub const NOT_FOUND: i32 = 5;
    pub const CONFLICT: i32 = 6; pub const CANCELLED: i32 = 7; pub const INTERNAL: i32 = 70;
}
```
Approval failures map to `exit::POLICY`; not-found to `exit::NOT_FOUND`;
validation to `exit::VALIDATION` (see `map_activation_error`,
`map_reader_error`).

### CommandOutput — the uniform command return shape
**Source:** `crates/application/src/quran_cli.rs:41-66`.
**Apply to:** every new `cmd_*` in `quran_cli.rs`.
```rust
pub struct CommandOutput { pub exit: i32, pub human: String, pub json: serde_json::Value }
```
Human output must avoid volatile ids (snapshot determinism); `--json` emits the
full structure (`crates/cli/src/quran.rs:998-1007`).

### Approval-gated canonical mutation (D-16, ADR-0107)
**Source:** `crates/application/src/quran.rs:238-260, 301-343`.
**Apply to:** activation, rollback, deprecation, edition verification, and any
new canonical pointer change. Approval must be `approved` **and** name the exact
`quran-edition:{slug}@{version}` URN; move + audit commit in one `UnitOfWork`.

### Migration append-only + checksum (ADR-0002)
**Source:** `migrations/sqlite/0008_quran_structure.up.sql:1-4`,
`migrations/sqlite/0007_quran_editions.up.sql:47-52`.
**Apply to:** the QC-05/QC-09/QC-11 migration. New file only; no `.down.sql`;
gate with `cargo run -q -p xtask -- migrate-check`.

### Frozen hash recipes (D-12, ADR-0108)
**Source:** `crates/quran-corpus/src/hashing.rs:8-27`.
**Apply to:** QC-08. Additive, domain-separated recipes only; never change
`qai-text-hash-v1` / `structure_hash` / `token_order_hash`.

### Type-level layer separation (ADR-0112, D-04)
**Source:** `crates/quran-core/src/view.rs` (`AyahView.canonical: QuranQuotation`)
and `crates/application/src/quran_reader.rs:537-584` (alignment recheck).
**Apply to:** QC-08 work — a translation must have no constructor path into a
canonical slot; prove it with a negative test rather than a comment.

### trycmd real-binary snapshot harness
**Source:** `crates/cli/tests/quran.rs:11-20`.
**Apply to:** every new CLI surface (`verify`, `verify-quotation`). One
`QAI_DATA_DIR` per `.trycmd` file; `db migrate` first; deterministic human output.

---

## No Analog Found

Files with no close harness match in the tree (planner should use RESEARCH.md
patterns instead):

| File | Role | Data Flow | Reason |
|---|---|---|---|
| Committed corpus-integrity report artifact | artifact | file-I/O | No integrity artifact exists anywhere in the tree; build its shape from `cmd_doctor_quran` JSON + `cmd_validate --report`. |
| `ALTER TABLE quran_editions ADD COLUMN …` in migration `0020` | migration | DDL | No existing migration alters an existing table; all prior DDL creates new tables. Choose `ALTER TABLE` (supported by SQLite) or a sidecar table deliberately. |
| Owner-gate records (OD-01/OD-02/OD-03) | doc | — | Governance records, not code; follow the Phase 1 plan/evidence-in-doc pattern. |

---

## Metadata

**Analog search scope:** `migrations/sqlite/`, `crates/{quran-core,quran-corpus,citations,provenance,storage,storage-sqlite,application,cli,server}/`, `fixtures/quran/`, `xtask/`.
**Files scanned:** 28 analogs/excerpt sites (all git-tracked; verified with `git ls-files`).
**Pattern extraction date:** 2026-09-25

**Tracked-source gate:** every analog path above is tracked source. No
`.gsd/capabilities/*` mirror path appears in this document.
