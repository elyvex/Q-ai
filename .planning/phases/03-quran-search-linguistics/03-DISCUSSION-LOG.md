# Phase 3: Quran Search & Linguistics - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-26
**Phase:** 3-Quran Search & Linguistics
**Areas discussed:** Scope & finish line, Morphology dataset/licensing/linguist gates, User-facing surface boundary, Linguistics tool depth

---

## Scope & Finish Line

| Option | Description | Selected |
|--------|-------------|----------|
| Roadmap-SC only | Complete when the 5 roadmap success criteria are satisfied; implement only those gaps | |
| Legacy-board completion | Complete only when the legacy `phase-02-rag` board (AC-P2-01…50, 14 ADRs, all waves) is closed | |
| Roadmap-SC + selected legacy hardening | Roadmap SCs govern; pull in `doctor --indexes`, golden sets, and performance budgets as exit evidence; defer the legacy tail | ✓ |
| Other | — | |

**User's choice:** Option 3, with an added **alpha-release bar** — after Phase 3, Phases 1–3 must together form a coherent, runnable alpha (search/linguistics reachable at real user surfaces, not service-only).
**Notes:** Scope confined to Phase 3's own surface; Phases 1–2 are not re-planned.

---

## Morphology Dataset, Licensing & Linguist Gates

| Option | Description | Selected |
|--------|-------------|----------|
| ADR-0203 Option B | Ship schema + importer + synthetic labeled test lexicon; root/lemma degrade to typed errors | |
| Option B + machine-enforced license/attribution gate | Adds source/license matrix and an activation-rejection gate | |
| Bundle a real licensed dataset now | Select + license a provider and bundle; linguist sign-off before completion | |
| All of the above (reconciled) | Engineering = Option B + license gate; real datasets pursued as a separate owner/linguist-gated track | ✓ |

**User's choice:** "All 1,2,3" — engineering deliverables #1+#2 plus a required, owner/linguist-gated dataset track.
**Notes:**
- **Datasets named:** **Quranic Arabic Corpus (QAC)** as the primary morphology reference (tags, lemmas, roots, conventions); **`fawazahmed0/quran-api`** local repo as an additional Quran/reference source.
- **Bundling posture (follow-up):** **Verify-then-bundle** — build importer + alignment + license matrix + activation gate now; QAC stays the documented default provider; bundle QAC only if redistribution verifies compatible, else ship ADR-0203 Option B with user-supplied import and typed "no dataset active" errors.
- **License discipline:** quran-api repo Unlicense is separate from each bundled edition/translation license; record exact upstream source, revision, provenance, attribution, and redistribution status in ADR-0203 + license matrix.
- **Linguist sign-off:** required gate for tagset/root/alignment conventions; if unavailable, record explicitly **BLOCKED**, never silently finalized.

---

## User-Facing Surface Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| CLI + HTTP API only | Satisfy SC1–SC4 via CLI + versioned API with explainability payload; defer UI to Phase 5 | ✓ |
| CLI + API + minimal terminal inspector | Adds an interactive terminal word-inspector | |
| Include a minimal Web view now | Thin web search + word inspector (overlaps Phase 5) | |
| Other | — | |

**User's choice:** Option 1 — researcher/developer-facing alpha.
**Notes:** Explainability payload included; TUI inspector, Web GUI, word-inspector UI, and result-contract/research checksum deferred to Phase 5, consistent with the legacy scope fence and roadmap.

---

## Linguistics Tool Depth

| Option | Description | Selected |
|--------|-------------|----------|
| SC-aligned core set | exact/normalized/phrase/concatenated/regex search; root + lemma search; multi-analysis morphology; word family; frequency + distribution + co-occurrence | ✓ |
| Core + full counting family | Adds collocation, interval, first/last-occurrence, numeric report with full CountingRules | |
| Full legacy 23-tool inventory | Adds all discovery tools | |
| Other | — | |

**User's choice:** Option 1 — implement only what SC1–SC4 require.
**Notes:** Provenance/attribution on every tool; `CountingRules` on all numeric outputs. Defer collocation, interval, first/last-occurrence, hapax, unusual-usage, near-duplicate, and missing-expected-form to later phases.

---

## the agent's Discretion

- Exact normalization rule ordering, profile ladder versions, SpanMap storage format, index manifest/hash function, checkpoint payload schemas, CLI flag naming, golden-set composition, and operator-facing wording — follow existing project conventions (Phase 1 D-16 convention).
- Precise QAC revision/URL, adapter shape, and tag-mapping details are for research to confirm; the owner ratifies the license/attribution entry.

## Deferred Ideas

- Interactive TUI word inspector, Web GUI, word-inspector UI, and `REQ-quran-result-contract` research checksum → Phase 5.
- Collocation, interval, first/last-occurrence, hapax, unusual-usage, near-duplicate, missing-expected-form, numeric-report tool → later phases.
- Legacy `phase-02-rag` tail (discovery depth, doc depth, eval-harness polish) → follow-ups.
- Transliteration (ADR-0206) and full-quality fuzzy L8 (ADR-0216) → later phases.
- Graph nodes/edges from roots/lemmas → Phase 4.
