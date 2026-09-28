---
phase: "03"
slug: "quran-search-linguistics"
status: verified
threats_open: 0
asvs_level: 1
created: "2026-09-28"
---

# Phase 03 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.
> Source: `<threat_model>` blocks in `03-01-PLAN.md` … `03-08-PLAN.md` (all eight
> plans authored with a parseable threat model → `register_authored_at_plan_time: true`).
> Posture: ASVS Level 1; unmitigated `high` blocks. L1 verification depth applied;
> short-circuit taken per workflow (threats_open: 0, register authored at plan time,
> ASVS 1). No new network/auth trust boundary class was introduced beyond the
> lexicon HTTP routes, which sit behind the existing shared router layers.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| untrusted query text → normalization + FTS | CLI/HTTP arg or fixture crosses into the shared normalization pipeline and indexed search | query text (untrusted) → normalized term/FTS |
| canonical tables → derived index | Read-only derive; the index/lexicon paths must never write canonical back | canonical rows → derived tables |
| derived store → verified hit | Recall candidates must be re-verified against canonical with trace + span + quotation | skeleton/trigram/lexicon rows → result contract |
| cross-ayah window → ayah-level result | A window match must not be presented as one verse | window match → per-ayah result |
| CLI arg → counting service | Untrusted root/lemma/profile/mode crosses into SQL aggregation | selectors (untrusted) → aggregate SQL |
| derived lexicon rows → family relations | Relations must be typed + explained, never merged | analyses → typed family edge |
| fixture golden data → assertions | Synthetic data must stay labeled and must not become ground truth | synthetic rows → tests/evidence |
| HTTP body → lexicon backend | Untrusted kind/id/root/lemma strings cross into SQL-bound lookups | request body (untrusted) → storage lookup |
| backend → HTTP response | Lexicon results must carry attribution + `CountingRules`, never bare data | derived rows → attributed envelope |
| operator-supplied license file → import params | Untrusted JSON crosses into the dataset row | license JSON (untrusted) → license evidence |
| staged dataset row → activation | Activation must fail closed without license evidence | dataset row → active pointer |
| agent/tool caller → registry | Untrusted tool input crosses into search/lexicon backends | tool input (untrusted) → `ToolResult` |
| agent-produced records → owner decisions | Gates must not be closed by an agent | records → owner decision |
| synthetic fixture → alpha evidence | Synthetic results must stay labeled | fixture → alpha report |

---

## Threat Register

| Threat ID | Category | Component | Severity | Disposition | Mitigation | Status |
|-----------|----------|-----------|----------|-------------|------------|--------|
| T-03-01 | Tampering | normalized search query binding | high | mitigate | Query normalized then bound as FTS term/substring; filters stay typed. Empty query is a typed invalid-input error, never an all-match or panic (`quran_tools.rs` "empty query never reaches here"; `search_goldens.rs` empty-skeleton assertion) | closed |
| T-03-02 | Tampering | displayed canonical text | critical | mitigate | Per-hit byte-identity test (`canonical_display_identity.rs`); MV-018 canonical-unchanged pre/post-checks in the rebuild paths (`quran_forms.rs`, `quran_morphology.rs`) | closed |
| T-03-03 | Spoofing | result without verified quotation | high | mitigate | Only the validating `SearchHit` path (`quran-search/src/hit.rs`) builds hits with canonical quotation + span + trace; no new result type bypasses it | closed |
| T-03-04 | Denial of Service | unbounded result sets | medium | mitigate | `limit` clamped `1..=1000` (`quran_cli.rs::cmd_search` L3624; `quran_search_api.rs::search_params` L242) | closed |
| T-03-05 | Repudiation | heuristic match presented as exact | medium | mitigate | Trace mandatory on every hit; `SearchHit::explanation()` returns `NormalizationTrace`; `assert_search_contract` asserts non-empty explanation | closed |
| T-03-SC | Tampering | cargo/npm installs | high | accept | No new packages proposed; all changes close inside existing workspace crates. Recorded as accepted risk AR-03-01; `arch-check` remains the edge gate | closed |
| T-03-06 | Tampering | concatenated query binding | high | mitigate | Query normalized through `L6.skeleton` then bound; `must_not_contain` precision anchors in every golden row; no SQL concatenation | closed |
| T-03-07 | Spoofing | cross-verse fragment presented as one verse | high | mitigate | Cross-ayah golden rows carry `allow_cross_ayah`/`max_ayah_span` selectors; runner asserts `spans_ayah_boundary` for boundary hits + ayah-level-win dedup (`run_search`, `search_goldens.rs`) | closed |
| T-03-08 | Denial of Service | unbounded trigram recall / scan fallback | medium | mitigate | Recall bounded by trigram postings with scan fallback (`trigram.rs`); `limit` clamped `1..=1000`; `max_ayah_span` defaults to 3 | closed |
| T-03-09 | Repudiation | heuristic concatenated match without explanation | medium | mitigate | Every returned hit carries non-empty `segmentation`; golden rows assert the tiling; `assert_display_identity` asserts concatenated hits carry segmentation | closed |
| T-03-10 | Tampering | new lexicon count SQL | high | mitigate | Typed bound parameters only; no string-concatenated SQL; tests exercise root/lemma values | closed |
| T-03-11 | Repudiation | numeric output without stated rules | high | mitigate | Every report embeds `CountingRules`; tests assert `rules.datasets` + `rules.multi_analysis_handling` (`assert_rules_complete`) | closed |
| T-03-12 | Information disclosure | dataset attribution missing | medium | mitigate | Root/lemma reports name the active `<slug>@<version>` from the dataset row (`dataset_source`); no guessed dataset name | closed |
| T-03-13 | Denial of Service | unbounded aggregation | low | accept | Aggregate SQL over indexed `dataset_id` joins; result sets are scalars, not row dumps. Recorded as accepted risk AR-03-02 | closed |
| T-03-14 | Tampering | family relation construction | high | mitigate | Relations go through `FamilyMember::new` + `relation_name()`; `relation_names_match_schema_domain` asserts CHECK-domain relation strings and non-empty explanations | closed |
| T-03-15 | Spoofing | synthetic data presented as ground truth | high | mitigate | Fixture rows carry `synthetic_test_only: true`; headers keep `reviewed_by: pending-linguist` (`search_goldens.rs`, `family_goldens.rs`); phase-3 ADRs stay Draft/Proposed (ADR-0203/0204/0205/0209/0210/0211/0215), none Accepted | closed |
| T-03-16 | Repudiation | silent empty result for unavailable capability | high | mitigate | Silent `Ok(Vec::new())` replaced by typed `QAI-MORPH-0004`; contract tests `affix_search_typed_unavailable`, `compare_and_unavailable_tools` pin it | closed |
| T-03-17 | Elevation of privilege | automatic analysis merge / preferred flag | critical | mitigate | No merge path; computational suggestions stay behind `ReviewPromotion` (reviewer + timestamp + evidence required); suggestions never auto-verify (`family.rs`) | closed |
| T-03-18 | Tampering | lexicon route input binding | high | mitigate | Non-empty validation + typed args; storage lookups are bound parameters; no string-concatenated SQL or paths | closed |
| T-03-19 | Information disclosure | unattributed lexicon result | medium | mitigate | Handlers include dataset id/attribution; contract test asserts attribution on the active-dataset path (`analysis_sources` in `quran_tools.rs`) | closed |
| T-03-20 | Denial of Service | new routes without the shared layers | medium | mitigate | Routes registered on the shared router carrying a 1 MiB body limit (`RequestBodyLimitLayer::new(1024 * 1024)`), 30 s timeout (`request_timeout_ms: 30_000`), concurrency 128 (`server/src/api.rs`, `config/src/lib.rs`) | closed |
| T-03-21 | Spoofing | empty 200 for unavailable capability | high | mitigate | `lexicon_error_status` maps the typed `UnavailableDataset` to an error status; contract test pins it | closed |
| T-03-22 | Elevation of privilege | activation without license evidence | critical | mitigate | Gate 6 in `activate_morphology` rejects any non-permissive/absent evidence with a typed error before promotion; fail-closed `license_gate_rejects` tests pin reject + pass paths | closed |
| T-03-23 | Repudiation | invented license/attribution values | high | mitigate | Evidence is read from operator-supplied files; license matrix records `source_url`/`capture_date`/`capturer`; nothing is inferred (`quran-morphology/src/license.rs`) | closed |
| T-03-24 | Tampering | untrusted license JSON parsing | medium | mitigate | Parsed into typed `LicenseEvidence::from_status_and_json`; malformed/missing fields raise typed `LicenseEvidenceError`; module is pure with no I/O | closed |
| T-03-25 | Tampering | canonical/hash drift via the gate change | high | mitigate | The gate only reads dataset rows and fails closed; no canonical write path and no hash recipe is touched (MV-018 unchanged) | closed |
| T-03-26 | Tampering | registry poisoning | high | mitigate | New tools return `ToolResult` envelopes built from verified hits/attributed analyses; lexicon tools cannot return results without the active dataset (`analysis_sources`) | closed |
| T-03-27 | Repudiation | doctor auto-repair | high | mitigate | Doctor is read-only; the soak re-asserts byte-identical state and drift is reported as `QAI-IDX-0101` (`doctor_indexes.rs`); no repair path is added | closed |
| T-03-28 | Denial of Service | unbounded tool latency | medium | mitigate | Budget artifact gates fixture runs; regex/limit caps unchanged; the tool surface reuses the clamped search paths | closed |
| T-03-29 | Information disclosure | tool result without attribution | medium | mitigate | `analysis_sources`/`normalization_rules` are required by tests on every new tool (`quran_tools.rs`, `tool-registry/src/lib.rs`) | closed |
| T-03-30 | Repudiation | owner gate recorded as resolved | critical | mitigate | `docs/05-followups/phase-03-owner-gates.md` states it closes nothing; `decisions-needed.md` stays the source of truth; closing path (`licenses/qac/...`) and both gate ids (OD-11/OD-12) recorded; `alpha_e2e.rs` cites the gates BLOCKED | closed |
| T-03-31 | Spoofing | synthetic data presented as scholarly | high | mitigate | `alpha_end_to_end_synthetic` asserts synthetic labeling; the deferral ledger and owner-gates record the limit | closed |
| T-03-32 | Tampering | numeric output without rules | high | mitigate | `assert_rules_complete` gates every numeric report in the alpha test | closed |
| T-03-33 | Repudiation | silent drop of deferred scope | medium | mitigate | `docs/05-followups/phase-03-deferrals.md` names every CONTEXT.md Deferred Idea + D-14 tool (`hapax_search`) | closed |

*Status: open · closed · open — below high threshold (non-blocking)*
*Severity: critical > high > medium > low — only open threats at or above workflow.security_block_on count toward threats_open*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-03-01 | T-03-SC | No package install in phase; changes stay inside existing workspace crates; `arch-check` edge gate remains | plan 03-01…03-08 | 2026-09-28 |
| AR-03-02 | T-03-13 | Counting aggregation is scalar SQL over indexed `dataset_id` joins; no row dumps | plan 03-03 | 2026-09-28 |

*Accepted risks do not resurface in future audit runs.*

---

## Residual Observations (non-blocking, below high threshold)

Owner gates OD-11 / OD-12 remain 🔴 human-only and are **not** security threats to
the engine — they gate licensed activation and scholarly ratification. They are
tracked in `docs/05-followups/phase-03-owner-gates.md` and
`docs/05-followups/decisions-needed.md`; `T-03-30` verifies no agent has closed
them. No residual security observation at or above the high threshold was found.

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-09-28 | 34 | 34 | 0 | secure-phase L1 (plan-register + grep evidence + SUMMARY threat flags) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-09-28
