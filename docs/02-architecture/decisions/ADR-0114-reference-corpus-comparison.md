# ADR-0114 — Typed Corpus Comparison: Integrity vs. Edition/Readings Differences

- Status: **Draft — pending human sign-off (do not mark Accepted)**
- Phase: 1 — Canonical Quran Core (extends to Phase 2+ comparison surfaces)
- Date: 2026-09-14 (created); 2026-09-18 (expanded for typed comparison)
- Related decisions: ADR-0101 (edition model), ADR-0108 (hashing),
  ADR-0109 (edition difference algorithm), ADR-0105 (tokenization), ADR-0203
- Requirements: PRD §35.1 (QV-015), §12.1 (reproducibility)
- Owner: _unassigned_ (swimlane P1-X03)
- Companion: `docs/02-architecture/upstream-sources.md`

---

## Status note

**Draft.** The reference corpus, procedure, and signer are not named. This
revision records the **comparison taxonomy** decided on 2026-09-18 and the rule
that incompatible representations are never reported as simple corruption.
Implementation of the taxonomy does **not** make this ADR Accepted.

---

## Context

QV-015 compares imported text against an *independent* reference corpus: zero
ayah-text differences, or differences explicitly acknowledged. Historically
this was framed as a binary "match or corruption".

Q-ai now represents **several explicitly identified editions/readings**
(ADR-0101). That breaks the binary framing in two ways:

1. Q-ai may legitimately compare **different kinds** of data: canonical
   text vs. canonical text (integrity), canonical vs. alternate edition/riwayah
   (readings comparison), text vs. translation(s), text vs. reference corpus,
   text vs. checksum manifest.
2. A textual difference between two legitimate editions is **not** evidence of
   corruption. Treating it as one would be a defect.

## Decision (proposed)

### 1. Comparison is typed

Every comparison declares its **operand kinds** and its **compatibility class**:

| Left | Right | Meaning |
|---|---|---|
| canonical edition | same edition @ same version | **integrity** comparison |
| canonical edition | same edition @ other version | **version** comparison (ADR-0109) |
| canonical edition | alternate edition / other riwayah | **readings** comparison |
| canonical text | translation | **translation** comparison |
| canonical text | reference corpus | **reference** comparison (QV-015) |
| text | checksum manifest | **integrity** verification |

The engine compares only **compatible representations by default**. Comparing
incompatible representations requires an explicit opt-in naming the comparison
kind; the result is never labeled "corruption".

### 2. Differences are classified

A reported difference carries a closed classification, at minimum:

```text
same
normalization_only
orthographic_difference
script_difference
riwayah_difference
edition_difference
tokenization_difference
translation_difference
unknown_difference
```

Classification requires identifying the difference at or below the level that
produced it (code point, carrier, word spacing, token boundary). A difference
that cannot be classified is `unknown_difference`, not silently failed.

### 3. Integrity comparison is not cross-riwayah comparison

- `Hafs Uthmani` vs `Hafs Uthmani` (same source, same normalization) may be a
  direct integrity comparison: a mismatch is a real integrity signal.
- `Hafs` vs `Warsh` **must not** be reported as a simple "corruption diff". It
  is an edition/readings comparison whose differences are expected; the report
  classifies them (`riwayah_difference` / `orthographic_difference` / …).
- A checksum manifest is single-edition. Comparing a Hafs manifest against a
  Warsh text is a typed mismatch, not a failed verification.

### 4. QV-015 behavior

- The comparator diffs every ayah of the imported edition against the
  configured reference and fails QV-015 unless the difference is acknowledged
  in the manifest with reviewer identity.
- **Until a reference corpus is configured, QV-015 is recorded as skipped
  (Info finding), never silently passed.** The importer implements this today.
- Where an alternates/readings comparison is desired (not QV-015), it runs as a
  separate, typed report and never feeds QV-015's pass/fail.
- Sign-off requires a named reviewer distinct from the implementer, recorded
  alongside `verified_by`.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| **A. Single binary comparator** | Simple to implement | Reports legitimate edition differences as corruption; false alarms |
| **B. Typed comparator with classification (chosen direction)** | Correct for multi-edition; reproducible reports | More design; requires classification heuristics + explicit opt-in |
| **C. Defer until reference corpus named** | No interim risk | Blocks comparator/testing work only for editorial, not technical, reasons |

## Accuracy implications

Misclassifying an edition difference as corruption (or vice versa) corrupts
confidence in the corpus and in downstream citations. Reports must state the
compared editions/versions, the normalization applied, the operand kinds, and
the classification of each difference; `unknown_difference` must remain visible
rather than being forced into a benign class.

## Religious-Source implications

Presenting one riwayah's differences against another as "errors" would
misrepresent recognized readings. The comparator must present readings
differences side by side with attribution and must never imply that one
recognized reading is a corruption of another.

## Licensing implications

Reference/alternate corpora and their comparison outputs inherit the licence of
their source. A comparison report may quote differences; redistribution of the
underlying text still follows per-source terms. No reference corpus is bundled
until its licence is cleared (owner decision ODV-08).

## Security implications

Comparison inputs are untrusted; parsing, normalization, and diffing run with
size caps and no network access in the deterministic path. Reports are
deterministic and reproducible.

## Operational implications

QV-015 remains a recorded skip until configured; operator surfaces must show
the skip explicitly. Typed comparison reports become part of the edition's
difference/validation artifacts and feed `qai quran diff` and doctor.

## Migration Strategy

Additive: existing `quran_edition_v1` differ remains for same-edition version
diffs (ADR-0109); the typed layer wraps it and adds operand-kind-aware
classification. Stored difference reports remain valid; new reports carry the
classification field with a default of `unknown_difference` for legacy data.

## Reversal cost

Low while Draft. Once difference reports are consumed downstream, changing the
classification vocabulary is a breaking change to stored reports; extend the
enum rather than repurpose values.

## Consequences

- The comparator can be built and tested against synthetic/adversarial fixtures
  now, without a real reference corpus.
- QV-015 stays a recorded skip until OD-03 lands; it is never silently passed.
- Cross-riwayah comparisons are possible and correctly typed, but do not satisfy
  QV-015.

## Follow-ups

- P1-X03 / OD-03: name the reference corpus, procedure, and signer.
- Implement the classification enum in the differ and cover it with fixtures.
- Ensure every difference report records operand kinds + normalization +
  editions/versions for reproducibility.
