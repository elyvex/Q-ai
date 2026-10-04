---
phase: "02"
slug: "canonical-quran-core"
status: verified
threats_open: 0
asvs_level: 1
created: "2026-09-26"
---

# Phase 02 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.
> Source: `<threat_model>` blocks in `02-01-PLAN.md` … `02-07-PLAN.md` (all seven
> plans authored with a parseable threat model → `register_authored_at_plan_time: true`).
> Posture: ASVS Level 1; unmitigated `high` blocks. L1 verification depth applied;
> short-circuit taken per workflow (threats_open: 0, register at plan time, ASVS 1).

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| any SQL client → canonical tables | Raw INSERT/UPDATE/DELETE can be issued outside the application; triggers are the last line of defence | SQL writes (untrusted) → canonical rows |
| persisted approval row → canonical publication | Only a granted approval naming the exact edition URN may authorize publication | approval decision → publication capability |
| importer → staging | Untrusted manifest bytes reach staging only; the importer must never hold a publication capability | manifest/CSV bytes (untrusted) → staging tables |
| operator → rollback | Rollback is a canonical pointer change, approval-gated exactly like activation | operator command → active-edition pointer |
| operator-supplied reference → comparison | Externally supplied reference document compared byte-exact, fail-closed | reference JSON (untrusted) → verdict |
| HTTP client → citation answer path | Stored quotation verdicts re-enforced on the answer path, never downgraded | quotation request → verdict + canonical text |
| fixture author → test corpus | All fixtures synthetic; no real dataset, publisher, license, or signer may be named | synthetic rows → tests/snapshots |

---

## Threat Register

| Threat ID | Category | Component | Severity | Disposition | Mitigation | Status |
|-----------|----------|-----------|----------|-------------|------------|--------|
| T-02-01 | Tampering | migration 0020 (`ALTER TABLE quran_editions` / `quran_stg_editions`) | medium | mitigate | Append-only new file; 0007–0019 untouched; `checksums.json` +1 entry; `xtask migrate-check` green (21 ordered) | closed |
| T-02-02 | Repudiation | `cmd_import` license synthesis | high | mitigate | Manifest-declared license persisted verbatim; absent stays explicit `Unknown`; never inferred (OD-01 recorded) | closed |
| T-02-03 | Spoofing | `EditionSelector::Primary` resolution | medium | mitigate | Never falls back to active pointer; unset flag returns typed error | closed |
| T-02-04 | Tampering | new columns reachable by raw `UPDATE` before fence lands | medium | accept (transient, escalated) | Window closed by 02-04: migration 0021 `BEFORE UPDATE` triggers + adversarial tests; only activation `INSERT..SELECT` wrote them before | closed |
| T-02-05 | Information disclosure | edition identity/license in CLI output | low | accept | Non-secret provenance metadata; no secret or credential crosses | closed |
| T-02-06 | Repudiation | `quran.reference_corpus` check / `verify` reference family | high | mitigate | Status derived only from persisted QV-015 finding; `skipped` serialized/printed as `skipped`, never `pass` (`quran_doctor.rs`); report artifact committed | closed |
| T-02-07 | Tampering | `qai quran verify` write path | high | mitigate | Opens DB via `open_read_only` (`quran_cli.rs`); read-only proven by unchanged-`corpus_generation` assertion; not wrapped in `confirm` helper | closed |
| T-02-08 | Tampering | operator-supplied reference manifest | medium | mitigate | Typed JSON adapter; byte-exact comparison, fail-closed on difference/missing/duplicate/policy mismatch | closed |
| T-02-09 | Spoofing | reference corpus identity | high | mitigate | `reference_corpus_id`/`reference_text_hash` verbatim from document; nothing inferred; OD-03 stays blocked | closed |
| T-02-10 | DoS | richer fixture size / deep verify runtime | low | accept | Synthetic small fixtures (dozens of ayahs); `verify` reuses check path, runs in seconds | closed |
| T-02-11 | Tampering / elevation of privilege | canonical tables without triggers | high | mitigate | Migration 0021: 16 `RAISE(ABORT…)` handlers with unique `QAI-QUR-*` codes; per-table adversarial abort tests | closed |
| T-02-12 | Spoofing | forgeable `ApprovalToken` | high | mitigate | Constructor `pub(crate)` (`provenance/src/lib.rs`); only public mint path requires `ApprovalRow` with `decision="approved"` + non-empty subject; behavioural tests + source scan | closed |
| T-02-13 | Tampering | unguarded canonical publication path | high | mitigate | `activate_edition` / `rollback_edition` / `record_edition_verification` open+commit a `CanonicalWriter` session bound to token subject in the same `UnitOfWork`; subject mismatch aborts pre-mutator | closed |
| T-02-14 | Repudiation | rollback without approval | high | mitigate | Rollback rejection coverage for missing/denied/mismatched approvals; pointer+generation asserted unchanged | closed |
| T-02-15 | Tampering | editing an applied migration | medium | mitigate | New-file-only; `checksums.json` updated; gated by `migrate-check` | closed |
| T-02-16 | Information disclosure | trigger error messages | low | accept | Messages name table, verb, stable code only; no row content or principal data echoed | closed |
| T-02-17 | Repudiation | `verify_quotation` with no production caller | high | mitigate | Shared verdict→hard-failure mapping (`is_hard_failure`/`require_exact`) + `qai quran verify-quotation` + HTTP enforcement; trycmd exit-code assertions (mismatch=3, not-found=5) | closed |
| T-02-18 | Spoofing | HTTP returning failed stored verdict as success | high | mitigate | `require_exact` enforced in `citation_handler`; server test asserts failure path; failed verdicts return errors, never success | closed |
| T-02-19 | Tampering | hash computed from supplied text, not canonical row | high | mitigate | Verifier returns resolved canonical hash from reader-backed resolver; tamper test asserts hard failure, never success with synthesised hash | closed |
| T-02-20 | Tampering | normalization altering displayed canonical text | medium | mitigate | No normalization rules applied in phase; `MatchAfterDeclaredNormalization` documented unreachable; returned text always the canonical row value | closed |
| T-02-21 | Information disclosure | CLI printing supplied text | low | accept | Supplied text is operator-provided, non-secret; CLI prints verdict + canonical hash | closed |
| T-02-22 | Spoofing | presenting a translation as the Quran | high | mitigate | Type-level separation (`AyahView.canonical` is canonical quotation type); non-empty translator + aligned edition required; negative test: no canonical constructor accepts translation text | closed |
| T-02-23 | Tampering | frozen v1 hash recipe mutated by translation-hash edit | high | mitigate | New domain-separated recipe only (`qai-translation-hash-v1`, `hashing.rs`); separation test proves digest differs from each frozen v1 digest; v1 tests green | closed |
| T-02-24 | Repudiation | synthesized translation license | high | mitigate | Declared license persisted verbatim; undeclared stays explicit `Unknown` with no invented permission; OD-01 remains owner gate | closed |
| T-02-25 | Tampering | order-dependent translation hash | medium | mitigate | Hash over passages sorted ascending `(surah, ayah)`; order-independence test | closed |
| T-02-26 | Information disclosure | translation license metadata surfaced | low | accept | Public provenance metadata, not a secret | closed |
| T-02-27 | Repudiation | owner gates marked satisfied without human decision | high | mitigate | OD-01/02/03 recorded blocked with named closing action+command; explicit agent-uncloseable statement; verification greps marker absence + command text | closed |
| T-02-28 | Repudiation | integrity report overstating a skipped family | high | mitigate | Artifact+test require reference family from state (`skipped`/`fail`/`pass`) and forbid `skipped`-as-`pass` serialization | closed |
| T-02-29 | Tampering | coverage threshold lowered silently | medium | mitigate | Thresholds below legacy floor require recorded shortfall entry with measured value+gap; existing rows immutable (`xtask/src/coverage.rs`) | closed |
| T-02-30 | Repudiation | misattributing a red suite to another phase | medium | mitigate | Evidence document carries actual observed output; quiet-worktree rule before gate reads | closed |
| T-02-31 | Tampering | committed artifact with volatile/fabricated identity | medium | mitigate | Artifact forbids timestamps/temp paths/hostnames/random ids; regenerable by documented ritual from rich fixture; guarded by `corpus_integrity` tests | closed |
| T-02-32 | Repudiation | rich/identity/reference fixture identity fields | high | mitigate | Every fixture `synthetic: true`; tests assert synthetic-ness and name no real dataset/publisher/license/signer; OD-01/OD-03 remain owner gates | closed |
| T-02-33 | Tampering | existing fixtures overwritten not extended | medium | mitigate | `test-edition-min` + 16 adversarial corpora byte-identical; `git status --short` gate fails on modification | closed |
| T-02-34 | Spoofing | reference fixture presented as independent corpus | medium | mitigate | Reference explicitly synthetic, pinned by existing hashing form; real corpus identity owner-gated (OD-03) | closed |
| T-02-35 | Information disclosure | UV/encoding stressors in fixture text | low | accept | Synthetic Quranic-shape data, non-secret by design | closed |
| T-02-36 | Spoofing | `edition show` rendering fabricated identity/primary/license | medium | mitigate | Renders only activation-carried canonical-row values; snapshot asserts declared values; no runtime flag setter introduced | closed |
| T-02-37 | Repudiation | spec-less edge-probe row silently dropped | high | mitigate | All six rows enumerated with answering plan/criterion in evidence doc; gate asserts every category token + requirement id present | closed |
| T-02-38 | Repudiation | D-15 read-path exemption closed without owner knowledge | high | mitigate | Recorded as explicit owner-ratifiable interpretation with per-path basis; never presented as satisfied locked decision | closed |
| T-02-39 | Repudiation | task marked complete before acceptance criteria hold | medium | mitigate | TASK-002 moved active→completed only after gate evidence + evidence doc exist; rollup entry required | closed |
| T-02-SC | Tampering | npm/pip/cargo installs | high | mitigate | No package install in phase: zero `Cargo.toml`/`Cargo.lock` changes in any `(02-0X)` plan commit (the `sha2` addition predates phase execution, 2026-09-24); future installs require package-legitimacy gate + blocking human checkpoint | closed |

*Status: open · closed · open — below high threshold (non-blocking)*
*Severity: critical > high > medium > low — only open threats at or above workflow.security_block_on count toward threats_open*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-02-01 | T-02-04 | Transient pre-fence window; escalation landed in 02-04 (migration 0021 triggers + adversarial tests) | plan 02-01 / verified 02-04 | 2026-09-26 |
| AR-02-02 | T-02-05 | Edition identity/license output is non-secret provenance metadata | plan 02-01 | 2026-09-26 |
| AR-02-03 | T-02-10 | Synthetic small fixtures; verify path runs in seconds | plan 02-03 | 2026-09-26 |
| AR-02-04 | T-02-16 | Trigger errors carry table/verb/code only, no row or principal data | plan 02-04 | 2026-09-26 |
| AR-02-05 | T-02-21 | Printed text is operator-supplied input, non-secret | plan 02-05 | 2026-09-26 |
| AR-02-06 | T-02-26 | Translation license metadata is public provenance | plan 02-06 | 2026-09-26 |
| AR-02-07 | T-02-35 | Fixture stressor text is synthetic, non-secret by design | plan 02-02 | 2026-09-26 |

*Accepted risks do not resurface in future audit runs.*

---

## Residual Observations (non-blocking, below high threshold)

From `02-REVIEW.md` (`issues_found`; 0 critical). All are fail-closed; none reopens a
register threat. Tracked as follow-ups, not open threats:

| Ref | Observation | Threat linkage | Posture |
|-----|-------------|----------------|---------|
| WR-01 | HTTP maps all citation codes QAI-QUR-0323…0326 to 500 (not-found/access-denied misclassified; CLI maps them correctly) | T-02-18 stays mitigated — failed verdicts still never return success | follow-up: cross-surface parity |
| WR-02 | Canonical `text_hash`/`token_order_hash` manifest-order-dependent; validator permits unsorted manifests; failure surfaces late at roundtrip, fail-closed | T-02-08/T-02-25 hold (fail-closed); translation path already sorts | follow-up: canonical sort or validation-time reject |
| WR-06 | Translation import hardcodes `attribution_required: false` for `Unknown` licenses (relaxed from `true`) | T-02-24 holds — license still verbatim, `Unknown` explicit, no invented permission | follow-up: revisit enforcement posture |
| WR-09 | `rollback_edition` rejects only `Active` targets; `Quarantined` reactivatable (approval still required) | T-02-14 holds — no approval-less rollback exists | follow-up: status allowlist |

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-09-26 | 40 | 40 | 0 | secure-phase L1 (plan-register + grep evidence + VERIFICATION 5/5) |
| 2026-10-04 | 40 | 40 | 0 | secure-phase L1 re-run — gap plan 02-08 (D-15 ledger rebuild) is docs-only + one source-scan test; no threat-surface change, no SUMMARY threat flags |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-09-26
