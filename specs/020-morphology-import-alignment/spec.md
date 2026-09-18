# Feature Specification: Morphology Import Alignment

**Feature Branch**: `020-morphology-import-alignment`

**Created**: 2026-09-18

**Status**: Draft

**Input**: User description: "edition-relative morphology import and alignment: multi-analysis Layer-D morphology records keyed by edition, version, surah, ayah and token position, explicit alignment tables bridging disagreeing tokenizations without re-tokenizing canonical text, declared root normalization conventions, dataset attribution and license records, typed dataset-unavailable errors, and a small public-domain test lexicon covering the fixture edition. Gap: ADR-0203 is new and quran-morphology is a placeholder."

**Constitution compliance**: `.specify/memory/constitution.md` v1.2.0, Principles I (canonical text never generated/corrected; canonical token order immutable), II (Layer D computational annotation with algorithm, version, confidence, timestamp, input version, verification status; computational suggestions never become verified edges without human review), IV (roots/lemmas are interpretive claims, never canonical text; no synthesized resolution), VIII (morphology is edition-relative; disagreeing tokenizations never force-matched; no LLM-generated roots/lemmas stored as dataset-supplied; unknowns stay unknown; pending owner decisions never block engineering). Proposal: ADR-0203 (Draft, pending sign-off).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Import edition-relative multi-analysis morphology records (Priority: P1)

A curator imports a morphology dataset built against one specific edition and version. Every record attaches to a canonical token by the key (edition, edition version, surah, ayah, token position) and carries its dataset identity, source version, alignment method, confidence, verification status, and attribution. A token may carry zero, one, or many analyses; competing analyses coexist with no single "correct" flag. A reviewer can trace any record back to the exact dataset release and edition text it was aligned against.

**Why this priority**: Edition-relative keying is the load-bearing invariant: a root index built for one edition must never be presented as analysis of another. Everything else (alignment, search, family edges) depends on this key being right.

**Independent Test**: Can be fully tested by importing a small crafted dataset against the synthetic fixture edition and confirming every stored record carries the full key plus provenance, and that a token with two competing analyses returns both with no correctness flag.

**Acceptance Scenarios**:

1. **Given** a morphology dataset built against a known edition and version, **When** it is imported, **Then** every stored record carries the edition, edition version, surah, ayah, token position, dataset identity, source version, alignment method, confidence, verification status, and attribution.
2. **Given** a token with two competing analyses from the import, **When** queried, **Then** both analyses are returned side by side with their provenance, and neither is marked as the correct one.
3. **Given** an import naming an edition or version that does not match the target corpus, **When** submitted, **Then** it is rejected with a diagnostic naming the mismatch, and no records are stored.

---

### User Story 2 - Bridge disagreeing tokenizations with explicit alignment tables (Priority: P2)

A curator imports a dataset whose word-splitting disagrees with the canonical tokenization (e.g. a clitic split differently). The import bridges the disagreement with an explicit, auditable alignment table mapping dataset token indices to canonical (edition, surah, ayah, position) keys — the canonical text and its tokenization are never modified. Tokens that cannot be aligned stay unaligned and are reported, never force-matched; verse-numbering mismatches are recorded, not silently remapped.

**Why this priority**: Re-tokenizing canonical text to fit a dataset would mutate the canonical key space and corrupt every downstream citation. The alignment table is what makes third-party datasets usable without touching canonical data.

**Independent Test**: Can be fully tested with a crafted dataset that splits one canonical token into two: the alignment table maps both dataset tokens to the one canonical position, the canonical token list is byte-identical before and after, and a deliberately unalignable token is reported unaligned.

**Acceptance Scenarios**:

1. **Given** a dataset whose tokenization disagrees with the canonical tokenization, **When** imported, **Then** an alignment table maps each dataset token to its canonical key, and the canonical token sequence is unchanged (verified identical before and after).
2. **Given** a dataset token that cannot be aligned to any canonical token, **When** imported, **Then** the token is recorded as unaligned with its evidence preserved, and the import report names every unaligned token.
3. **Given** a dataset whose verse numbering differs from the edition's numbering, **When** imported, **Then** the mismatch is recorded as data in the import report, and numbering is never silently remapped to fit.

---

### User Story 3 - Declare root conventions with dataset attribution and licence records (Priority: P1)

A reviewer examining morphology output sees, per dataset, the declared root normalization convention (e.g. how roots are spelled, how hamza-like forms are treated), the attribution string to display, and the licence record governing bundling and redistribution. Root and lemma family relations are typed (root, lemma, stem, form, computational, verified) and carry provenance. Machine-generated analyses are labeled computational with needs-review status and are never stored as dataset-supplied.

**Why this priority**: Root spelling conventions differ between datasets; without a declared convention, cross-dataset root searches silently disagree. Attribution and licence records are what keep dataset reuse lawful and honest.

**Independent Test**: Can be fully tested by registering two datasets with different declared root conventions and confirming each dataset's records resolve roots under its own declared convention, with attribution and licence shown per dataset.

**Acceptance Scenarios**:

1. **Given** a dataset with a declared root normalization convention, **When** its records are queried by root, **Then** matching follows the declared convention recorded for that dataset, and the convention is visible to the reviewer.
2. **Given** a machine-generated analysis, **When** stored, **Then** it is labeled computational with needs-review verification status, never as dataset-supplied, and it never appears as a verified edge.
3. **Given** a dataset whose redistribution licence is unverified, **When** registered, **Then** it is marked accordingly (metadata only / pending licence review), its text is not bundled, and the attribution string that would be shown is still recorded.

---

### User Story 4 - Degrade to a typed error with a public-domain test lexicon fallback (Priority: P2)

A researcher uses root/lemma search when no licensed morphology dataset is available. Instead of guessed data or a crash, they receive a typed dataset-unavailable error naming the missing capability. Meanwhile developers and tests exercise the full multi-analysis schema, importer, and alignment validator against a small, clearly non-authoritative public-domain test lexicon covering the synthetic fixture edition.

**Why this priority**: Root/lemma search must never fabricate linguistic data to fill a gap. The typed error plus test lexicon is the ADR-0203 fallback (option B): engineering and testing proceed at full fidelity while dataset selection stays honestly pending.

**Independent Test**: Can be fully tested by querying root search with no dataset installed (typed error naming the capability) and by running the importer/validator suite against the test lexicon over the fixture edition (all green, lexicon labeled non-authoritative).

**Acceptance Scenarios**:

1. **Given** no morphology dataset installed, **When** a root/lemma search is attempted, **Then** the result is a typed dataset-unavailable error naming the missing capability, never guessed roots and never an unhandled failure.
2. **Given** the public-domain test lexicon and the synthetic fixture edition, **When** the import and alignment validation run, **Then** they succeed end to end, and every test-lexicon record is labeled non-authoritative test data.
3. **Given** the test lexicon, **When** shown to a researcher, **Then** it is identified as test data covering only the fixture edition, never presented as scholarly linguistic authority.

---

### Edge Cases

- What happens when a morphology manifest arrives malformed, oversized, or with schema violations? It is rejected under the deny-by-default posture with a diagnostic; no partial records become queryable and no import auto-activates.
- What happens when an alignment entry valid for one edition is offered against a different edition? It is rejected: alignment entries apply only to the exact edition and version they were built for.
- What happens when a token has zero analyses after import? The token is simply unanalyzed (a representable state), and queries over it report absence of analysis rather than absence of the token.
- What happens when a dataset supplies several analyses per token but a query wants one? The query records its own preference policy; the stored data keeps all analyses with no correctness flag.
- What happens when a reviewer promotes a computational analysis to verified? Only explicit human review changes the status, and the promotion records who reviewed, what evidence was checked, and when.
- What happens when a dataset update supersedes earlier records? Earlier analyses are deprecated, never deleted, so existing citations to them remain resolvable.
- What happens when confidence is unknown for a record? It stays unknown; unknown confidence is never defaulted to a number.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Every morphology record MUST be keyed by edition, edition version, surah, ayah, and token position within the edition's canonical tokenization, and MUST carry dataset identity, source version, alignment method, confidence, verification status, and attribution.
- **FR-002**: The system MUST represent zero, one, or many analyses per token; stored analyses MUST NOT carry a correctness flag, and queries MUST record their own preference policy when selecting among competing analyses.
- **FR-003**: Importing morphology MUST NEVER modify canonical text or the canonical tokenization; tokenization disagreements MUST be bridged by an explicit alignment table mapping dataset token indices to canonical keys.
- **FR-004**: Alignment entries MUST apply only to the exact edition and version they were built for; an entry offered against a different edition or version MUST be rejected with a diagnostic.
- **FR-005**: Tokens that cannot be aligned MUST remain unaligned with evidence preserved and MUST be named in the import report; the system MUST never force-match an unalignable token.
- **FR-006**: Verse-numbering differences between a dataset and the edition MUST be recorded as data in the import report, never silently remapped.
- **FR-007**: Alignment MUST be validated (token text, order, verse coverage) before any imported record becomes queryable; records failing validation MUST NOT be queryable.
- **FR-008**: Each dataset MUST declare its root normalization convention as data, and root/lemma matching for that dataset's records MUST follow its declared convention; the convention MUST be visible to reviewers.
- **FR-009**: Root/lemma/family relations MUST be typed (root, lemma, stem, form, computational, verified) and MUST carry provenance; machine-generated analyses MUST be stored as computational with needs-review status, never as dataset-supplied, and MUST NOT appear as verified edges without explicit human review.
- **FR-010**: Every dataset MUST carry an attribution string and a licence record (status, terms, source, redistribution rights); datasets with unverified redistribution terms MUST be marked metadata-only / pending licence review and MUST NOT be bundled.
- **FR-011**: When no morphology dataset is available for a requested operation, the system MUST return a typed dataset-unavailable error naming the missing capability; it MUST NEVER return guessed linguistic data.
- **FR-012**: The system MUST ship a small public-domain test lexicon covering the synthetic fixture edition, labeled everywhere as non-authoritative test data, sufficient to exercise the schema, importer, alignment validator, and query path end to end.
- **FR-013**: Superseded analyses MUST be deprecated, never deleted; citations to earlier analyses MUST remain resolvable.
- **FR-014**: Morphology manifests MUST be treated as untrusted input: schema-validated, size-capped, with no network fetch in the deterministic path; a failed or incomplete import MUST NOT auto-activate and MUST NOT leave partial records queryable.

### Key Entities

- **Morphology Record**: One Layer-D analysis attached to a canonical token — full edition/version/surah/ayah/position key plus lemma, root, stem, pattern, part of speech, features, dataset identity, source version, alignment method, confidence, verification status, attribution, and licence reference; unknown fields stay unknown.
- **Alignment Table**: The explicit, auditable bridge between a dataset's token indices and canonical keys for one exact edition and version; disagreements are data, unalignable tokens stay unaligned.
- **Root Normalization Convention**: A dataset's declared rules for root spelling and comparison, stored as data per dataset rather than baked into shared logic.
- **Dataset Attribution and Licence Record**: Per-dataset display attribution plus licence status, terms, source, and redistribution rights governing bundling and reuse.
- **Dataset-Unavailable Error**: The typed response when morphology is requested without an installed dataset, naming the missing capability instead of guessing.
- **Test Lexicon**: A small public-domain morphology dataset covering only the synthetic fixture edition, labeled non-authoritative, used to exercise the full path without a licensed provider.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A reviewer given any stored morphology record can identify the exact edition, version, verse, token position, dataset release, and verification status without consulting anything beyond the record and its import report.
- **SC-002**: In trials with a dataset that splits tokens differently from the canonical tokenization, 100% of dataset tokens are either mapped through the alignment table or reported unaligned with evidence — zero force-matches and zero changes to the canonical token sequence.
- **SC-003**: In trials with two competing analyses on one token, both are returned with provenance in 100% of queries, and no output marks either as correct.
- **SC-004**: In trials with two datasets declaring different root conventions, each dataset's root queries follow its own declared convention in 100% of cases, and reviewers can state which convention governed each answer.
- **SC-005**: In every trial with no dataset installed, root/lemma search returns the typed dataset-unavailable error naming the capability — zero guessed roots across all trials — and with only the test lexicon installed, the full import-to-query path over the fixture edition succeeds.
- **SC-006**: A curator can complete an import-to-review cycle (import, alignment validation, human review promotion of one analysis, deprecation by a newer dataset release) with every step's actor, evidence, and timestamp recorded and the earlier analysis still resolvable afterward.

## Assumptions

- The canonical tokenizer and token ordering come from the Phase-1 corpus work (ADR-0105); this feature consumes the canonical tokenization as fixed input and never redefines it.
- The choice of production morphology provider (dataset identity, version, licence, coverage, single- vs. multi-analysis) remains a pending owner/linguist decision (P2-X01); this spec defines the schema, importer, alignment, and fallback shape so engineering is not blocked by that selection.
- The test lexicon covers only the synthetic fixture edition (test-edition-min class data), which is not Quran text and never implies editorial approval of a real corpus.
- Derived root/lemma indexes record the dataset and version as first-class dependencies in their manifests and generation stamps, so reproducibility checks can detect stale derivations.
- Promotion of an analysis to verified status is a human-only gate: it cannot be performed or simulated by an automated process.
