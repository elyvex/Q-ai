---
phase: 02-canonical-quran-core
reviewed: 2026-09-25T00:00:00Z
depth: deep
files_reviewed: 34
files_reviewed_list:
  - crates/application/src/quran.rs
  - crates/application/src/quran_cli.rs
  - crates/application/src/quran_doctor.rs
  - crates/application/src/quran_reader.rs
  - crates/application/src/quran_tools.rs
  - crates/citations/src/lib.rs
  - crates/cli/src/quran.rs
  - crates/provenance/src/lib.rs
  - crates/quran-core/src/edition.rs
  - crates/quran-core/src/enums.rs
  - crates/quran-core/src/reference/serializer.rs
  - crates/quran-corpus/src/format.rs
  - crates/quran-corpus/src/hashing.rs
  - crates/quran-corpus/src/import.rs
  - crates/quran-corpus/src/lib.rs
  - crates/quran-corpus/src/validation.rs
  - crates/server/src/api.rs
  - crates/storage-sqlite/src/quran.rs
  - crates/storage/src/quran.rs
  - crates/storage/src/lib.rs
  - crates/domain/src/hashing.rs
  - crates/quran-core/src/numbers.rs
  - migrations/sqlite/0020_quran_edition_identity.up.sql
  - migrations/sqlite/0021_canonical_write_fence.up.sql
  - migrations/sqlite/0007_quran_editions.up.sql
  - migrations/sqlite/0011_quran_staging.up.sql
  - migrations/sqlite/checksums.json
  - xtask/src/coverage.rs
  - fixtures/quran/test-edition-rich/manifest.json
  - fixtures/quran/test-edition-identity/manifest.json
  - fixtures/quran/golden/ayah_texts.jsonl
  - fixtures/quran/test-edition-rich/reference.json
  - fixtures/quran/test-edition-rich/ayahs.csv
  - crates/server/tests/api.rs
findings:
  critical: 0
  warning: 9
  info: 5
  total: 14
status: issues_found
---

# Phase 02: Code Review Report (Canonical Quran Core)

**Reviewed:** 2026-09-25T00:00:00Z
**Depth:** deep
**Files Reviewed:** 34
**Status:** issues_found
**Scope:** `git diff 2a590c6..HEAD -- crates migrations xtask fixtures` (56 files, +4569/−147).
Test files and trycmd snapshots were sampled, not line-audited; the two known
pre-existing trycmd failures (`normalize.trycmd`, `search.trycmd`) were excluded
per instructions.

## Summary

The phase adds the edition-identity columns, the 0021 write fence, the
`ApprovalToken`/`ApprovalGate` canonical-write wiring, translation content
hashing, quotation hard-failure plumbing (`require_exact` shared by CLI and
HTTP), the operator reference path (QV-015), `quran verify` / `verify-quotation`
verbs, and the `Primary` edition selector. Cross-file tracing (importer →
storage → application gate → HTTP/CLI surfaces → triggers) shows the security
properties mostly hold: denied approvals are rejected before any write,
staging cleanup cascades (`0011 … ON DELETE CASCADE`), `UNIQUE(slug, version)`
prevents duplicate canonical editions, QV-015 failures clear staging
fail-closed, and the new translation hash is domain-separated from the frozen
canonical recipes. No BLOCKER-grade defect was found — every integrity-relevant
gap below still fails closed or requires a human-held approval — but there are
9 WARNINGs that degrade correctness, API-contract parity, or robustness, and
5 informational observations. Migrations are append-only (only `0020`/`0021`
added, `checksums.json` updated, no edits to `0007–0019`).

## Warnings

### WR-01: New citation hard-failure codes fall through to HTTP 500

**File:** `crates/server/src/api.rs:212-221`
**Issue:** `tool_status` maps `QAI-QUR-0306…0309` and `0322` to 404 and
everything else to 500. The four new codes from this phase — `0323`
(QuotationMismatch), `0324` (LocationNotFound), `0325` (EditionNotFound),
`0326` (AccessDenied) — therefore all surface as `500 INTERNAL_SERVER ERROR`.
The CLI maps the same errors correctly (`map_citation_error`: mismatch →
VALIDATION/3, missing → NOT_FOUND/5, denied → POLICY/4), so surface parity
claimed in `crates/application/src/quran_tools.rs` ("a mismatch can never be
softened into a warning on one surface while it fails on another") holds for
codes but not for HTTP statuses: a not-found becomes a 500, and an access
denial becomes a 500 instead of 403/404. The security property (never 200)
holds — the bundled test only asserts `!= OK` — so this is contract
correctness, not a bypass.
**Fix:**
```rust
ToolError::Backend { code, .. } => match code.as_str() {
    "QAI-QUR-0306" | "QAI-QUR-0307" | "QAI-QUR-0308" | "QAI-QUR-0309"
    | "QAI-QUR-0322" | "QAI-QUR-0324" | "QAI-QUR-0325" => StatusCode::NOT_FOUND,
    "QAI-QUR-0326" => StatusCode::FORBIDDEN,
    _ => StatusCode::INTERNAL_SERVER_ERROR,
},
```

### WR-02: Canonical hashes are manifest-order-dependent; validator permits unsorted manifests

**File:** `crates/quran-corpus/src/import.rs:560-599` (see also `crates/quran-corpus/src/validation.rs:263-282`)
**Issue:** `hashes_computed` feeds `text_hash`/`token_order_hash` in manifest
(`doc.ayahs`) order, and `global_ayah_index` is assigned in manifest order,
while `roundtrip_verified` recomputes from `list_stg_ayahs` (`ORDER BY
surah, ayah`). QV-005 sorts a *copy* for the density check, so an unsorted
manifest passes validation and then fails late at roundtrip with
"text_hash recomputed from staged rows differs (QV-014)" — a confusing error
for a validation-passing input. This phase fixed the identical problem on the
translation path by sorting before hashing (QC-08,
`crates/application/src/quran.rs` ordered-passages block) but left the
canonical path order-sensitive. Direction is fail-closed (no corrupt hash is
persisted), hence WARNING, not BLOCKER.
**Fix:** Sort a canonical `(surah, ayah)` view once in `hashes_computed` (and
assign `global_ayah_index` from it), or add a QV ordering rule rejecting
unsorted manifests at validation time — mirroring the translation-path fix.

### WR-03: Nonexistent edition reported as `NotStaged` in three canonical mutators

**File:** `crates/application/src/quran.rs:497,568,1273`
**Issue:** `record_edition_verification`, `rollback_edition`, and
`deprecate_edition` all map `get_edition_by_slug_version → None` (edition does
not exist at all) to `ActivationError::NotStaged` ("Import the edition to
Staged first."). For rollback/verify/deprecate the edition is looked up in the
*canonical* table, where "Staged" is never a valid state, so the remedy
misdirects the operator. The rollback instance is new in this phase (the
pre-check was added with the gate wiring).
**Fix:** Add a dedicated variant (e.g. `EditionNotFound { slug, version }`,
remedy "Import and activate the edition first.") and use it for the three
canonical-table lookups; reserve `NotStaged` for staging-table lookups
(`activate_edition:393`, `validate_staged:707,716`, which are correct).

### WR-04: Corrupt validation report degrades to misleading `Skipped`

**File:** `crates/application/src/quran_doctor.rs:486-510`
**Issue:** `reference_corpus_check` uses
`serde_json::from_str(raw).unwrap_or_default()` (line 493): an unparseable
persisted `findings_json` yields an empty finding set and falls through to
`Skipped` with "no reference corpus configured (ADR-0114; OD-03 owner gate)" —
a message that asserts a configuration fact the code did not establish. A
tampered/corrupt report therefore reads as a benign skip. Pass detection
(line 505) additionally substring-matches `"outcome":"pass"` inside the
serialized message instead of parsing the outcome field.
**Fix:**
```rust
let findings: Vec<Finding> = serde_json::from_str(raw).map_err(|_| {
    fail("quran.reference_corpus",
         "persisted validation report is corrupt; integrity unknown".to_string(),
         "re-import the edition so a QV-015 outcome is recorded")
})?;
```
and match the parsed `outcome` value rather than `message.contains(...)`.

### WR-05: Unreachable `MatchAfterDeclaredNormalization` defaults to success

**File:** `crates/citations/src/lib.rs:85-100,187-199`
**Issue:** `is_hard_failure` returns `false` and `require_exact` returns `Ok`
for `MatchAfterDeclaredNormalization`, a variant the code itself documents as
having "no producer" in v1. If any future producer emits it, every answer path
(CLI, HTTP, tools) silently succeeds on a non-exact match with no
declared-normalization verification — a fail-open default for an integrity
verdict.
**Fix:** Map it to `Err` until a producer exists, e.g.
```rust
QuotationVerdict::MatchAfterDeclaredNormalization { .. } => Err(CitationError::Backend {
    detail: "declared-normalization matches are not supported in v1".to_string(),
}),
```
and return `true` from `is_hard_failure` (or remove the variant until v2).

### WR-06: Translation import relaxes `attribution_required` to `false`, including for `Unknown` licenses

**File:** `crates/application/src/quran.rs:940-960`
**Issue:** Both license branches hardcode `"attribution_required": false` —
the `None` (undeclared, owner gate OD-01) branch and the `Some(spdx_id)`
(`OpenLicense`) branch. The pre-phase code used `true`. For an `Unknown`
license, `false` invents a permission relaxation (the conservative default is
`true`); for an SPDX id naming an attribution-requiring license (CC-BY-*),
`false` misstates the obligation. `redistribution_allowed`/`export_allowed`
stay `false` (deny-by-default, good), which bounds the blast radius — hence
WARNING, not BLOCKER. The same `false` literal in `quran_cli.rs:775,786`
predates the phase and is noted for consistency.
**Fix:** Emit `true` when no license is declared, and do not hardcode the
field when an SPDX id is present (omit it or carry a declared value); never
persist `false` for `Unknown`.

### WR-07: Out-of-range surah/ayah mapped to internal error instead of caller error

**File:** `crates/application/src/quran_tools.rs:187-192,265-275`
**Issue:** `SurahNumber::new`/`AyahNumber::new` rejections (e.g. surah `0` or
`>114`, ayah `0`) are mapped to `CitationError::Backend`, which surfaces as
CLI exit INTERNAL and HTTP 500. These are caller-supplied out-of-range values,
not backend failures; valid-but-absent locations already map correctly to
`LocationNotFound` via the `Ok(None)` path (line 196). The new
`verify_canonical_quotation` (lines 265-275) duplicates the pre-existing
`fetch_ayah_text` pattern instead of fixing it.
**Fix:** Map construction failures to `CitationError::InvalidReference`
(CLI → USAGE/2, HTTP → 400 via `InvalidInput`), keeping `Backend` for genuine
storage/resolver failures.

### WR-08: Read-only operator paths open write transactions and swallow storage errors

**File:** `crates/application/src/quran_cli.rs:486-512` (also `:837-841`)
**Issue:** `cmd_edition_show` — a pure read — opens `db.write()`, reads one
row, then `rollback()`s. This serializes every `edition show` against writers
through the write mutex despite a `Database::read()` API existing
(`crates/storage/src/lib.rs:78`). Storage failure is additionally swallowed:
`.ok().flatten()` (line 500) silently omits the `license:` line instead of
erroring, so a degraded database renders a confidently incomplete record. The
`cmd_import` QV-015 error branch (line 837) repeats the write-tx-for-read
pattern.
**Fix:** Use `db.read()` for both read paths; propagate the edition-row fetch
error (`map_err` → `exit::INTERNAL`) instead of `.ok().flatten()`.

### WR-09: `rollback_edition` can reactivate a `Quarantined` edition

**File:** `crates/storage-sqlite/src/quran.rs:761-827` (check at `:783`); caller `crates/application/src/quran.rs:552-600`
**Issue:** The storage layer rejects only `target_status == "Active"`; any
other canonical status — `Deprecated`, `Approved`, or `Quarantined` — can be
flipped to `Active` with pointer bump. The application layer adds no status
allowlist. A URN-bound approval is still required (so this is not a bypass of
the human gate), but quarantine exists to keep content unserved, and the
rollback path contains no quarantine check or warning.reactivating an
`Approved`-but-unverified edition has the same shape.
**Fix:** Allowlist rollback targets to `Deprecated` (and `Approved` only if
that is an intended source state), rejecting `Quarantined` explicitly, e.g.
```rust
if !["Deprecated"].contains(&target_status.as_str()) {
    return Err(StorageError::ConstraintViolation {
        message: format!("rollback target {slug}@{version} is {target_status}, not Deprecated"),
    });
}
```

## Info

### IN-01: Gate sessions are in-memory only; approvals are replayable

**File:** `crates/provenance/src/lib.rs:281-330`
**Issue:** `ApprovalGate::{begin,commit,abort}_canonical_change` mutate only a
stack-local `CanonicalChangeSession` struct; `commit` is infallible whenever
the session is `Open` (which it always is at the call sites) and persists
nothing. Consequently the same granted `approval_id` authorizes unlimited
sequential activations/rollbacks/deprecations for its URN — no consumption,
nonce, or expiry. Pre-existing model (the old `check_approval` also had no
consumption), and per-URN human approval arguably intends repeat use, so
recorded as Info: if one-time-use is ever required, it needs a durable
consumed-approval record, which no migration in this phase provides.

### IN-02: `verify-quotation` serves a recomputed digest, not the stored per-ayah hash

**File:** `crates/citations/src/lib.rs:295`
**Issue:** `resolve()` returns
`text_hash: Some(sha256Hex(canonical_text))` recomputed from the fetched text,
while the docstrings on `verify_canonical_quotation` and `cmd_verify_quotation`
claim the hash is "read from the canonical row" / "never recomputed" (QC-02).
Values agree today because `fetch_ayah_text` returns raw row bytes and
`AyahRow.text_hash` is the same SHA-256, but there are two sources of truth:
any future normalization in the read path would silently diverge the served
hash from the stored bytes. Recommend threading the stored `AyahRow.text_hash`
through the reader view instead of hashing at resolve time.

### IN-03: `quran verify` output does not record sampled vs full-corpus depth

**File:** `crates/application/src/quran_cli.rs` (family-entry builder; `gate`/`reason` fields)
**Issue:** Family entries carry `status`/`checks`/`summary`/`gate`/`reason`
but no depth marker, so `verify` (sampled roundtrip) and `verify --deep`
(full roundtrip) both report `roundtrip: pass` indistinguishably. An auditor
cannot tell from the artifact which was run. Consider adding
`"depth": "sampled"|"full"` per family (at minimum for `roundtrip`).

### IN-04: `Primary` selector has two latent ambiguities

**File:** `crates/application/src/quran_cli.rs:139`, `crates/quran-core/src/reference/serializer.rs:14-19`
**Issue:** (a) Bare `primary` is hijacked as the reserved selector, so an
edition actually slugged `primary` (legal per `is_valid_slug`) cannot be
addressed bare. (b) `Primary` serializes identically to `Active` (no prefix);
the code excludes it from the round-trip property by policy ("resolves before
any reference is emitted"), but any future emission site silently produces a
reference that parses back as `Active`. Both are contained by current
call-site discipline; noted so the exemption stays deliberate.

### IN-05: Stale comment in the QV-015 fail-closed path

**File:** `crates/quran-corpus/src/import.rs:1082-1087`
**Issue:** The comment reads "Fail closed and leave the stage untouched" while
the code immediately calls `clear_staging` (correct per T-02-08: failed
comparisons must not persist staging rows). "Untouched" presumably means
"unpromoted to canonical", but as written it contradicts the next statement.
Reword to "leave canonical untouched; staging is cleared".

---

_Positively noted (no action):_ decision check duplicated at token mint
(`from_approval_row`) and application (`check_approval:257`) — defense in
depth; `UPDATE OF`-column trigger (0021:66-71) composes correctly with the 0007
hash trigger and the `verified_*`/`status` mutators; staging cascade cleanup
and `UNIQUE(slug, version)` verified; QV-015 retry-evidence branch now clears
staging before marking `Failed`; translation hash sorts passages and is
domain-separated from frozen v1 recipes.

_Reviewed: 2026-09-25T00:00:00Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
