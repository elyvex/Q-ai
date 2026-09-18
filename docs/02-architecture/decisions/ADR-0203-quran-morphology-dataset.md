# ADR-0203 — Quran Morphology Dataset, Multi-Edition Alignment, and Attribution

- Status: **Draft — pending human sign-off (do not mark Accepted)**
- Phase: 2 — Search / Normalization / Linguistics
- Date: 2026-09-18
- Related decisions: ADR-0101 (edition model), ADR-0105 (tokenization),
  ADR-0109 (difference algorithm), ADR-0114 (comparison), ADR-0201 (FTS)
- Requirements: PRD §9, §11, §38; invariants I11, I12, I13
- Owner: _unassigned_ (P2-X01 swimlane; carried from P0-X02 / P1-X04)
- Companion: `docs/02-architecture/upstream-sources.md`,
  `docs/05-followups/owner-decisions.md`

---

## Status note

**Draft.** The owner has **not** selected a final morphology provider. This ADR
records the model, the alignment strategy, the licensing rule, and the fallback
so engineering can proceed without falsely marking an unselected dataset final.

- **Owner Decision** — the multi-edition morphology model and alignment
  guarantee in this ADR (2026-09-18).
- **Source Verification Pending** — dataset identity, version, licence,
  attribution string, root convention, and linguist sign-off.
- **Explicitly Unknown** — provider, version, licence, coverage, and whether it
  supplies one or several analyses per token.

---

## Context

Morphological analysis (root, lemma, stem, features, patterns) must attach to
Q-ai's canonical corpus without weakening invariants:

- **I11** — competing morphological analyses coexist (no `is_correct` column).
- **I12** — machine linguistics is Layer D with algorithm/version/confidence/status.
- **I13** — family relations are typed and carry provenance.

The earlier framing assumed a single Quran text whose tokens could be indexed
1:1 by morphology output. That assumption is now invalid: Q-ai represents
**multiple Quran editions/readings** (ADR-0101), and a morphology dataset is
produced against **one specific edition and tokenization**. A root index built
for one edition must never be presented as analysis of another.

## Decision (proposed)

### 1. Morphology is edition-relative

The model does **not** assume `one Quran text = one token sequence`. It is:

```text
Quran (logical corpus)
  └── Quran Edition (script, qiraah, riwayah, version, source, integrity)
        └── Ayah (surah:ayah under the edition's numbering)
              └── Token (edition's canonical tokenization, ordered position)
                    └── Morphological analysis (0..n, Layer D)
```

Every morphology record carries its `quran_edition_id`. Alignment is performed
against the exact **source edition + edition version + token order**, never
against "the Quran" in the abstract.

### 2. Morphology record carries explicit source and confidence

A record is capable of carrying at least:

```yaml
quran_edition_id:   "<edition id>"
edition_version:    "<version>"
surah:              <int>
ayah:               <int>
token_index:        <int>          # position within the edition's tokenization
token_text:         "<surface form as stored in the edition>"
lemma:              "<or unknown>"
root:               "<or unknown>"
stem:               "<or unknown>"
pattern:            "<or unknown>"
pos:                "<or unknown>"
features:           { "<key>": "<value>" }
source:             "<dataset id>"
source_version:     "<version | unversioned>"
alignment_method:   exact_index | alignment_table | manual | unknown
confidence:         <0.0..1.0 | unknown>
verification_status: unverified | needs_review | verified | rejected
attribution:        "<dataset attribution string>"
license:            { status, expression, source_url, redistribution_allowed, ... }
```

Machine-generated analyses use `Attribution::Computational` (I12) and
`verification_status = needs_review`; they are never stored as dataset-supplied.

### 3. Alignment strategy (non-negotiable)

- Alignment maps dataset token indices to the Phase-1
  `(edition, surah, ayah, position)` key.
- A **disagreeing tokenization is bridged by an explicit, auditable alignment
  table** — *never* by re-tokenizing canonical text (which would mutate the
  canonical key space).
- Alignment identifies the **exact source edition/version**; an alignment entry
  valid for edition A does not apply to edition B.
- An unalignable token stays unaligned and is reported, never force-matched.
- Where the dataset's verse numbering differs from the edition's, the mismatch
  is recorded, not silently remapped.

### 4. Multi-analysis, typed roots (I11, I13)

- Zero, one, or many analyses per token are all representable. There is no
  `is_correct` flag; the *query* records its preference policy.
- Root/lemma/family relations are typed (`root`, `lemma`, `stem`, `form`,
  `computational`, `verified`) and carry provenance.
- Root normalization convention (bare letters vs. separators; hamza treatment)
  is **declared as data**, not baked into code, and recorded per dataset.

### 5. Dataset selection — candidates only, none selected

Candidate families exist (e.g. Quranic Arabic Corpus lineage, QUL/Tarteel
resources, `quran-api`-adjacent morphology, academic treebanks), but **none has
been evaluated for coverage, depth, licence, or redistribution**. No candidate
is named here as chosen because that would assert unverified facts. Evaluation
must record, at minimum:

- coverage (all tokens? per-edition?), depth (POS/features/lemma/root/stem/pattern);
- licence/redistribution rights and attribution requirement (§38);
- whether it supplies one or several analyses per token;
- which edition/tokenization it was built against;
- root normalization convention.

### 6. Fallback if no dataset can be bundled

Ship the full multi-analysis schema, importer, alignment validator, adapter
interfaces, and tools. Anchor tests on a **small, clearly non-authoritative
public-domain test lexicon** covering the synthetic fixture edition. Expose a
typed "dataset unavailable" error naming the missing capability. Root/lemma
search degrades to that typed error — **never to guessed data**.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| **A. Bundle a licensed morphology dataset** | Out-of-box root/lemma/family analysis | Requires explicit redistribution rights; edition-tokenization alignment work |
| **B. Ship multi-analysis schema + test lexicon; user imports** | No licensing exposure; model fully testable | Users must obtain a licensed dataset |
| **C. Defer all morphology work** | No interim risk | Stalls root/lemma search and graph work indefinitely |

**Current fallback: B** (engineering proceeds; dataset selection pending).

---

## Accuracy implications

A morphology record aligned to the wrong edition, ayah, or token corrupts every
root/lemma search, family edge, and tafsir link that consumes it. Alignment must
therefore be validated (token text, order, verse count) before any record is
queryable, and confidence must never be promoted to `verified` without human
review. Disagreements between tokenizations are data, not errors to hide.

## Religious-Source implications

Roots and lemmas are interpretive linguistic claims, not canonical text. They
must never be displayed as if they were the Arabic text (I2/Layer separation),
and no LLM may generate and store roots/lemmas as dataset-supplied (explicit
prohibition in the Phase-2 plan). Competing analyses remain side by side.

## Licensing implications

The dataset licence governs whether morphological output may be bundled or must
be user-supplied. This is independent of the Quran text licence and of the
repository licence of any tool. If redistribution is unverified, ship schema +
tooling + test lexicon only, mark the dataset `metadata_only` /
`pending_license_review`, and record the attribution string that would be shown.

## Security implications

Morphology manifests are untrusted input: parse under the deny-by-default
posture, validate schema and token alignment, cap artifact sizes, and never let
an import auto-activate. No network fetch in the deterministic path.

## Operational implications

`qai quran morphology import <manifest>` (planned) is the single entry point;
activation is human-gated like the corpus. Dataset/version is a first-class
dependency for reproducibility (§12.1) and is recorded in derived-index
manifests and generation stamps (I14).

## Migration Strategy

Additive: new Layer-D tables keyed by edition identity; no change to canonical
tables. Adding a second morphology source or a second edition-relative alignment
is new rows, not a schema break. Prior analyses are deprecated, never deleted.

## Reversal cost

Low while Draft (no dataset shipped). Once records are active, swapping provider
requires new records + validation + a rebuild of derived indexes; the canonical
corpus is unaffected.

## Consequences

- Root/lemma/family architecture, importer, alignment validator, and typed
  errors can land before a provider is chosen.
- `AC-P2-01` stays open until this ADR is Accepted with a licensed dataset or a
  documented fallback.
- No morphology dataset is claimed as final; unknowns stay unknown.

## Follow-ups

- P2-X01/P2-X02: dataset + licence + linguist engagement (owner/legal).
- Author golden morphology sets against the chosen dataset and edition.
- Extend the QV validators with an alignment validator that fails closed on
  edition mismatch.
- Reference the selected dataset revision in `docs/02-architecture/upstream-sources.md`
  once chosen.
