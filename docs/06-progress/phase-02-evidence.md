# Phase 2 — Canonical Quran Core: Evidence of Record

> **Phase contract (D-02).** The phase being evidenced here is `.planning/ROADMAP.md`
> §Phase 2 ("One validated Quran edition is importable, addressable, and provably
> immutable") plus its three requirements in `.planning/REQUIREMENTS.md`:
> `REQ-quran-corpus`, `REQ-data-separation-layers`, `REQ-ingestion-validation-eval`.
> The legacy board (`docs/03-plan/phases/phase-01-core/`), the
> `specs/005·006·012·015·016·019` documents, and ADRs 0101–0114 are **authoritative
> evidence/reference only**. Their own recorded status — for example the legacy
> ledger's **0 of 21 criteria verified** — is *not* this phase's completion and is
> **not** restated as such here.
>
> **Legacy numbering drift (stated, not resolved).**
> `docs/03-plan/phases/phase-01-core/` is the legacy canonical core (this
> roadmap's Phase 2); `docs/03-plan/phases/phase-02-rag/` is search / linguistics /
> normalization (this roadmap's **Phase 3**). Directories are **not** renumbered by
> hand.
>
> **Recorded:** 2026-09-25 by plan `02-07`. Every observed result below was produced
> by running the named command on this working tree; nothing is asserted from prose.

---

## 1. The five Phase 2 success criteria → command → observed result

### Criterion 1 — Operator can import a Quran edition through staging → validation → atomic activation with rollback

| Field | Value |
|---|---|
| Command | `cargo test -p application --test quran_import` |
| Observed | `15 passed; 0 failed` |
| Command | `cargo test -p storage-sqlite --test quran` |
| Observed | `10 passed; 0 failed` |
| Command | `cargo test -p cli --test quran -- quran_snapshots` |
| Observed | `1 passed; 0 failed` (`read_flow.trycmd`: migrate → import → activate → read → import v2 → activate → diff → rollback → hashes → error exits 5/6) |
| Proving artifact/test | `crates/application/tests/quran_import.rs` (`crash_matrix_all_thirteen_checkpoints_leave_active_untouched`, `rejected_activations_leave_canonical_state_untouched`, `rollback_service_restores_the_prior_version`, `rejected_rollbacks_leave_canonical_state_untouched`), `crates/storage-sqlite/tests/quran.rs`, `crates/cli/tests/quran/read_flow.trycmd` |
| Owner | Engineering (plan 02-04). The rollback-*rejection* branch (missing/denied/mismatched approval → pointer, rows, and `corpus_generation` unchanged) was the previously-uncovered gap and is now proven. |

### Criterion 2 — User can look up any `surah:ayah` and receive byte-exact canonical Arabic with pinned edition reference

| Field | Value |
|---|---|
| Command | `qai quran get 1:1` (on the rich fixture) |
| Observed | `الٓمٓ` / `— quran:test-edition-rich@0.1.0:1:1` / `text_hash: sha256:49c5355bd921dc0d810e3be457432004847a72d5dbe754562be895be707be6c3` |
| Command | `cargo test -p application --test quran_reader` |
| Observed | `13 passed; 0 failed` (incl. `cache_serves_no_stale_text_after_activation`, `cache_serves_no_stale_text_after_rollback`, `lookup_performance_smoke`) |
| Command | `cargo test -p quran-core` |
| Observed | `39 + 1 + 6 passed; 0 failed` (331 golden reference-grammar cases; round-trip; never-panics) |
| Proving artifact/test | `crates/cli/tests/quran/read_flow.trycmd` (pins the stored hash on the operator line), `crates/application/tests/quran_reader.rs`, `crates/quran-core/tests/reference_grammar.rs` |
| Owner | Engineering (plan 02-01 identity, plan 02-03 operator hash line). The hash printed is the **stored** per-ayah hash read from `AyahView`, never recomputed from the returned text. |

### Criterion 3 — Corpus integrity checks (counts, addressing, Unicode, checksums, round-trip, reference comparison) pass

| Field | Value |
|---|---|
| Command | `qai quran verify` / `qai quran verify --json` (on the rich fixture) |
| Observed | all six families `pass`; `exit: 0`. `reference_comparison: pass` is backed by evidence `{ "method": "exact-ayah-bytes-v1", "comparison_kind": "reference", "reference_corpus_id": "synthetic-rich-reference", "outcome": "pass" }` |
| Command | `cargo test -p application --test quran_doctor` |
| Observed | `4 passed; 0 failed` |
| Command | `cargo test -p cli --test doctor_json` |
| Observed | `2 passed; 0 failed` |
| Command | `cargo test -p quran-corpus` |
| Observed | `50 lib + 6 adversarial + 13 fixtures + 4 import_path passed; 0 failed` |
| Command | `cargo test -p cli --test corpus_integrity` |
| Observed | `4 passed; 0 failed` |
| Proving artifact/test | **`docs/06-progress/corpus-integrity-report.json`** (committed evidence of record), `crates/application/tests/quran_doctor.rs`, `crates/cli/tests/corpus_integrity.rs` |
| Owner | Engineering (plan 02-03 verify surface + operator reference path; plan 02-07 committed artifact). |

**The committed artifact** is `docs/06-progress/corpus-integrity-report.json`. It
exposes **exactly** the six family keys plus the pinned `edition.slug`/`edition.version`,
carries **no** timestamp, temporary path, host name, or random id, and is
regenerated by the ritual below. It was produced by importing `test-edition-rich`
**with its companion synthetic reference** (`fixtures/quran/test-edition-rich/reference.json`),
so `reference_comparison` is an evaluated `pass` — a real comparison, not a
fabricated label. When **no** reference corpus is configured the same family is
serialized `skipped` (see `crates/cli/tests/quran/verify.trycmd`), and a skipped
family is **never** written as `pass` (D-10).

**Regeneration ritual** (run from the repository root; produces a byte-identical
artifact):

```bash
tmp="$(mktemp -d)"
export QAI_DATA_DIR="$tmp"
qai db migrate
qai quran import fixtures/quran/test-edition-rich/manifest.json \
    --reference fixtures/quran/test-edition-rich/reference.json
qai quran activate test-edition-rich@0.1.0 --yes
qai quran verify --json > "$tmp/verify.json"
qai quran edition active > "$tmp/active.txt"
# Compose the artifact: edition identity + the six families + the persisted
# QV-015 evidence object (the operator import path's own recorded finding).
python3 - "$tmp" <<'PY'
import json, sqlite3, sys
tmp = sys.argv[1]
verify = json.load(open(f"{tmp}/verify.json"))
row = sqlite3.connect(f"{tmp}/qai.db").execute(
    "SELECT findings_json FROM validation_reports WHERE subject_urn = ?",
    ("quran-edition:test-edition-rich@0.1.0",),
).fetchone()
finding = next(f for f in json.loads(row[0])
               if f.get("rule_id") == "QV-015" and f.get("severity") == "info")
slug, version = open(f"{tmp}/active.txt").read().strip().split("@", 1)
artifact = {"schema": "qai.corpus-integrity.v1",
            "edition": {"slug": slug, "version": version},
            "families": verify}
artifact["families"]["reference_comparison"]["evidence"] = json.loads(finding["message"])
json.dump(artifact, open("docs/06-progress/corpus-integrity-report.json", "w"),
          indent=2, sort_keys=True, ensure_ascii=False)
PY
```

### Criterion 4 — Canonical tables reject all non-approved writes; importer has no code path to canonical tables

| Field | Value |
|---|---|
| Command | `cargo test -p storage-sqlite --test quran` |
| Observed | `10 passed; 0 failed` (`canonical_triggers_abort_raw_writes_with_codes` proves one coded abort per canonical table, `QAI-QUR-0001…0021`; `canonical_tables_declare_the_trigger_set`) |
| Command | `cargo test -p quran-corpus --test import_path` |
| Observed | `4 passed; 0 failed` (importer holds no `ApprovalToken`/`CanonicalWriter`, calls no activation/rollback mutator, inserts into no canonical table, ends `Staged`) |
| Command | `cargo test -p storage --lib quran` |
| Observed | `4 passed; 0 failed` (gated-mutator write-surface source scan) |
| Command | `cargo test -p provenance --lib` |
| Observed | `10 passed; 0 failed` (`ApprovalToken` minted only from a persisted granted approval row) |
| Command | `cargo run -q -p xtask -- migrate-check` |
| Observed | `OK — 21 migration(s) ordered; checksums stable.` |
| Proving artifact/test | `migrations/sqlite/0021_canonical_write_fence.up.sql`, `crates/storage-sqlite/tests/quran.rs`, `crates/quran-corpus/tests/import_path.rs`, `crates/provenance/src/lib.rs` |
| Owner | Engineering (plan 02-04, D-13 a/b/c: SQL triggers + type-level gate + importer code-path audit). |

### Criterion 5 — Every quotation verifies via `verify_quotation` with mismatch as a hard failure

| Field | Value |
|---|---|
| Command | `cargo test -p cli --test quran -- quran_verify_quotation_snapshots` |
| Observed | `1 passed; 0 failed` (exact → exit 0; tampered → exit 3 with a typed code; unknown edition → exit 5) |
| Command | `cargo test -p citations` |
| Observed | `7 passed; 0 failed` (`require_exact` maps every hard failure to a distinct `QAI-QUR-*` code) |
| Command | `cargo test -p application --test quran_tools` |
| Observed | `6 passed; 0 failed` |
| Command | `cargo test -p server --test api` |
| Observed | `18 passed; 0 failed` (a stored hard-failing verdict is not a 200 envelope) |
| Proving artifact/test | `crates/cli/tests/quran/verify_quotation.trycmd`, `crates/citations/src/lib.rs`, `crates/application/src/quran_tools.rs`, `crates/server/src/api.rs` |
| Owner | Engineering (plan 02-05, D-15). The two production surfaces that accept an **externally supplied** quotation (`qai quran verify-quotation`, `GET /api/v1/quran/citations/{id}`) enforce the hard-failure mapping; the direct-read paths are structurally exempt (see §3). |

---

## 2. Spec-less edge-probe disposition

The requirements have no per-requirement spec files, so a fallback edge-probe run
over `REQ-quran-corpus`, `REQ-data-separation-layers`, and
`REQ-ingestion-validation-eval` surfaced **six unresolved rows**. Every row is
enumerated below and dispositioned to the plan / acceptance criterion that
answers it. **No row is dropped or left without a disposition.**

| # | Requirement | Edge-probe category | Disposition (answering plan / criterion) | Concrete evidence |
|---|---|---|---|---|
| 1 | `REQ-quran-corpus` | **empty** | Plan 02-01 absent-identity / absent-license criteria (absent values stay NULL/None/false and never invent a value); plan 02-03 recorded QV-015 **skip** criterion (a missing reference is a recorded `skipped`, never a `pass`). | `crates/application/tests/quran_identity.rs#undeclared_identity_stays_absent`, `#undeclared_license_is_unknown_without_invented_permissions`; `crates/cli/tests/quran/verify.trycmd` (reference `skipped`) |
| 2 | `REQ-quran-corpus` | **encoding** | Plan 02-01 byte-exact identity (no trim / case-fold / normalization); plan 02-02 byte-exact, NFC-clean synthetic fixtures; plan 02-03 `sha256:<hex>` reference-hash criterion. | `quran_identity.rs#declared_identity_and_primary_survive_to_the_operator_surface`; `crates/quran-corpus/tests/fixtures.rs`; `reference_comparison.evidence.reference_text_hash` in the committed artifact |
| 3 | `REQ-data-separation-layers` | **unclassified** | Plan 02-06 layer-separation negative test: a translation can never reach a canonical slot. | `crates/application/tests/quran_translation.rs#translation_cannot_reach_a_canonical_slot`; `crates/quran-core/src/view.rs#translation_requires_translator_and_edition` |
| 4 | `REQ-ingestion-validation-eval` | **empty** | Plan 02-01 absent-identity criterion; plan 02-03 QV-015 recorded-skip criterion. | `quran_identity.rs#undeclared_identity_stays_absent`; `crates/quran-corpus/src/import.rs::compare_reference` (the `None` reference branch emits a recorded `Info` skip) |
| 5 | `REQ-ingestion-validation-eval` | **encoding** | Plan 02-01 / plan 02-02 byte-exact identity and NFC criteria; plan 02-03 `sha256:<hex>` reference-hash criterion. | `crates/quran-corpus/tests/fixtures.rs` (validate-clean rich/reference fixtures); the committed artifact's `reference_text_hash` |
| 6 | `REQ-ingestion-validation-eval` | **concurrency** | Plan 02-01 append-only additive migration + the interrupted-import safety criterion; plan 02-04 concurrent pointer-change safety criterion (rejected rollbacks and generation-keyed cache invalidation leave pointer/rows/generation consistent). | `crates/application/tests/quran_import.rs#crash_matrix_all_thirteen_checkpoints_leave_active_untouched`, `#rejected_rollbacks_leave_canonical_state_untouched`; `crates/application/tests/quran_reader.rs#cache_serves_no_stale_text_after_rollback` |

**No silent drop:** all six rows name a requirement id, a category, and the
answering plan/criterion. Rows 1 and 4 are distinct requirement surfaces answered
by the same absent-value discipline; rows 2 and 5 are distinct requirement
surfaces answered by the same byte/encoding discipline; row 3 is the sole
`REQ-data-separation-layers` row; row 6 is the sole concurrency row.

---

## 3. Answer-path verification: enforced vs structurally exempt (D-15, owner-ratifiable)

D-15 ("enforce `verify_quotation` on every current answer path") is delivered as:
**enforce on every path that accepts an externally supplied quotation; record the
direct-read paths as structurally exempt because they serve bytes from the
canonical source of truth and cannot mismatch by construction.** This is an
**owner-ratifiable interpretation**, not a closed decision — see
`docs/05-followups/phase-02-owner-gates.md` §"Owner-ratifiable interpretation —
D-15 read-path exemption" (owner should ratify or overrule).

| Answer path | Kind | Basis |
|---|---|---|
| `qai quran verify-quotation` | **enforced** | `crates/application/src/quran_tools.rs:253`, `crates/application/src/quran_cli.rs:362` |
| `GET /api/v1/quran/citations/{id}` | **enforced** | `crates/server/src/api.rs:584`; route `crates/server/src/api.rs:1314` |
| tool `quran.get_ayah` | exempt | `crates/application/src/quran_tools.rs:89-95` → `crates/application/src/quran_reader.rs:355` |
| tool `quran.get_context` | exempt | `crates/application/src/quran_tools.rs:99-105` |
| CLI `cmd_get` / `cmd_context` / `cmd_surah` / `cmd_division` | exempt | `crates/application/src/quran_cli.rs:161/214/255/290` |
| HTTP `context_handler` / `tokens_handler` / `resolve_handler` / `ayahs_handler` | exempt | `crates/server/src/api.rs:451/524/560/1309` |

All exempt paths read the canonical source of truth; wrapping them in
`verify_quotation` would compare canonical text against itself (research Pitfall 4).

---

## 4. Known legacy baseline (not this phase's scope)

`cargo test -p cli --test quran` currently reports **7 passed / 2 failed**. The two
failures are `quran_search_snapshots` (`crates/cli/tests/quran/search.trycmd`) and
`quran_normalize_snapshots` (`crates/cli/tests/quran/normalize.trycmd`); both
predate plan 02-01 and stem from a non-deterministic index manifest hash. They
belong to the **legacy search / normalization phase (this roadmap's Phase 3)**, not
to the canonical core. Their snapshots were **not** overwritten. Per the phase
validation contract, re-check them on a **quiet worktree** before reading them as
part of any phase gate; do **not** attribute their failure to Phase 2.

`cargo-deny` is not installed locally, so `cargo xtask ci` step 4 warns and skips;
this is an existing local-tool limitation (CI enforces it), not a Phase 2 failure.

**Pre-existing `cargo fmt` drift.** `cargo fmt --all -- --check` currently reports
formatting-only drift in five files that plan 02-07 does **not** touch (all landed
by earlier plans of this same phase and left unformatted): `crates/provenance/src/lib.rs`,
`crates/quran-corpus/src/import.rs`, `crates/quran-corpus/tests/fixtures.rs`,
`crates/quran-corpus/tests/import_path.rs`, `crates/storage-sqlite/tests/quran.rs`.
Consequently `cargo run -p xtask -- ci` stops at its **step 1 (fmt)** before it can
reach step 3 (the two snapshot failures above). This drift is **not** absorbed or
re-attributed by plan 02-07; it is recorded here so a later `cargo fmt --all` pass
closes it explicitly. The new plan-02-07 file `crates/cli/tests/corpus_integrity.rs`
**is** fmt-clean.


---

## 5. Gate summary (observed on this tree, 2026-09-25)

| Gate | Command | Observed |
|---|---|---|
| Application Quran suites | `cargo test -p application --test quran_import --test quran_reader --test quran_translation --test quran_verification --test quran_doctor --test quran_tools` | 54 passed, 0 failed |
| Storage Quran | `cargo test -p storage-sqlite --test quran` | 10 passed, 0 failed |
| CLI Quran + doctor | `cargo test -p cli --test quran --test doctor_json` | quran 7 passed / 2 failed (documented baseline); doctor_json 2 passed |
| Server API | `cargo test -p server --test api` | 18 passed, 0 failed |
| Citations | `cargo test -p citations` | 7 passed, 0 failed |
| Quran core | `cargo test -p quran-core` | 46 passed, 0 failed |
| Quran corpus | `cargo test -p quran-corpus` | 73 passed, 0 failed |
| Committed artifact test | `cargo test -p cli --test corpus_integrity` | 4 passed, 0 failed |
| Architecture | `cargo run -q -p xtask -- arch-check` | OK — no forbidden dependency edges |
| Migrations | `cargo run -q -p xtask -- migrate-check` | OK — 21 migration(s) ordered; checksums stable |
| Full pre-merge gate | `cargo run -p xtask -- ci` | stops at 1/9 (fmt) on pre-existing drift in five files not touched by plan 02-07 (§4); step 3 would then stop on the two documented legacy snapshots |

A red result is attributed to its real owner above (the two legacy snapshots →
the search/normalization phase; the missing local `cargo-deny` → local tool
limitation). No failing check is re-attributed without re-running it and recording
the actual output.

---

## 6. What this phase deliberately does not claim

- No real canonical dataset, publisher, release, hash, or license is named
  (OD-01), no reviewer is named (OD-02), and no reference corpus or signer is
  named (OD-03). All three remain 🔴 and block **real** canonical activation; they
  are recorded in `docs/05-followups/phase-02-owner-gates.md` with their closing
  actions and commands.
- `translation_editions` are non-canonical; canonical `quran_segments` stays empty
  (reserved for the morphology phase).
- The D-15 read-path exemption above is an **owner-ratifiable interpretation**,
  not a closed locked decision.
- The legacy board's own status (0/21) is evidence/reference only and is not this
  phase's completion.

---

*Phase: 2-Canonical Quran Core · Evidence of record created 2026-09-25 by plan 02-07.*
