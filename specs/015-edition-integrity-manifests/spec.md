# Feature Specification: Per-Edition Integrity Manifests

**Feature Branch**: `015-edition-integrity-manifests`

**Created**: 2026-09-18

**Status**: Draft

**Input**: User description: "per-edition integrity manifests: generation and verification of verse/surah/edition-scope SHA-256 manifests with explicit normalization per Quran edition, quranchecksum-compatible rollups, one independent manifest per edition and riwayah, plus hash-only translation manifests with a root key distinct from the Quran root; a Hafs manifest must never verify a non-Hafs text."

**Constitution compliance**: `.specify/memory/constitution.md` v1.2.0, Principles I (canonical text never generated/corrected; checksums gate activation), III (traceability, content hashes, reproducibility), IV (differences between legitimate readings are readings comparisons, never corruption verdicts), VIII (one manifest per edition; one reading's checksum never certifies another; translation integrity hash-only with distinct root key).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Generate an integrity manifest for one Quran edition (Priority: P1)

A curator holding an approved Quran edition text (a specific script + transmission + source release, e.g. Uthmani script Hafs ʿan ʿĀṣim from a named source version) generates an integrity manifest for exactly that edition. The manifest records a hash for every verse (6,236-class scope), a rollup hash per surah (114), and a single edition-scope root hash, together with an explicit statement of the hash algorithm and the text normalization applied before hashing.

**Why this priority**: Without a per-edition manifest there is nothing to verify against. This is the foundation for tamper detection, reproducible imports, and approval-gated activation.

**Independent Test**: Can be fully tested by generating a manifest from a fixed sample edition twice and comparing outputs, and delivers a stable, storable integrity record for that edition alone.

**Acceptance Scenarios**:

1. **Given** a fixed edition text with known identity (edition, version, script, transmission where declared), **When** a manifest is generated, **Then** it contains one hash entry per verse, one rollup per surah, and one edition root, plus the algorithm name and normalization description.
2. **Given** the same edition text, **When** generation is run twice, **Then** both manifests are byte-identical (deterministic output).
3. **Given** any change to a single verse character, **When** the manifest is regenerated, **Then** that verse hash, its surah rollup, and the edition root all differ from the previous manifest.

---

### User Story 2 - Verify an edition text against its own manifest (Priority: P1)

A reviewer or automated gate verifies a candidate copy of an edition text against the manifest generated for that same edition, and receives a clear pass/fail verdict with the exact location of any mismatch (verse, surah rollup, or edition root level).

**Why this priority**: Verification is the operational payoff: it catches corruption, truncated imports, encoding damage, and wrong-source substitution before text becomes visible to researchers.

**Independent Test**: Can be fully tested by verifying an untouched copy (passes) and a copy with one altered verse (fails naming that verse), without needing any other edition.

**Acceptance Scenarios**:

1. **Given** an unaltered copy of the edition text and its manifest, **When** verification runs, **Then** the result is PASS with matching verse, surah, and edition-root levels.
2. **Given** a copy with one altered verse, **When** verification runs, **Then** the result is FAIL identifying that verse and its surah rollup as mismatched.
3. **Given** a copy with verses reordered or a verse missing, **When** verification runs, **Then** the result is FAIL (order and completeness are covered, not just per-verse content).

---

### User Story 3 - A Hafs manifest never verifies a non-Hafs text (Priority: P1)

A reviewer attempts to verify the text of one reading/transmission (e.g. Warsh) against the manifest of a different reading (e.g. Hafs). The system refuses to report a pass and instead reports an edition-identity mismatch — it never presents cross-reading differences as corruption of either text.

**Why this priority**: This is the core safety invariant. Readings legitimately differ; certifying one with another's checksum would produce false corruption verdicts and false passes. Constitution Principle VIII makes this non-negotiable.

**Independent Test**: Can be fully tested with two small sample editions that differ in at least one verse: cross-verification must fail with an identity-mismatch verdict even where most verses happen to be identical.

**Acceptance Scenarios**:

1. **Given** a Hafs manifest and a non-Hafs text (different transmission), **When** verification is attempted, **Then** the result is refusal/FAIL on edition identity, never a verse-by-verse corruption report and never PASS.
2. **Given** two manifests for two transmissions, **When** their edition roots are compared, **Then** each root is bound to its own edition identity so the roots cannot be substituted for one another.
3. **Given** two legitimate editions that differ in wording, **When** their texts are compared, **Then** the outcome is labeled a readings difference, not a corruption or tampering verdict.

---

### User Story 4 - Hash-only integrity for translations with a distinct root (Priority: P2)

A curator generates and verifies integrity manifests for translated texts. Translation manifests store only hashes (never the translated text itself) and carry a root key unmistakably different from the Quran-text root, so a translation root can never be mistaken for a Quran root.

**Why this priority**: Translations need tamper detection too, but they must never be confused with canonical text (Constitution Principles II, IV, VIII). The distinct root key makes confusion structurally impossible.

**Independent Test**: Can be fully tested by generating a translation manifest, confirming it contains no readable translated sentences, and confirming its root field name differs from the Quran root field name.

**Acceptance Scenarios**:

1. **Given** a translated text with its identity (translator, language, source release), **When** a manifest is generated, **Then** it contains hashes and identity metadata only — no translated sentences.
2. **Given** a translation manifest root and a Quran manifest root, **When** their field names are inspected, **Then** they are different keys (a reader cannot mistake one for the other).
3. **Given** an altered translation copy, **When** verified against its manifest, **Then** verification FAILs identifying the affected unit.

---

### User Story 5 - Cross-check against the external checksum reference (Priority: P2)

A maintainer compares a locally generated manifest for the Uthmani/Hafs baseline against the external quranchecksum-compatible reference manifest and gets a match-or-classified-difference answer using the same rollup construction (verse hashes concatenated in order into surah rollups, surah rollups concatenated in order into the edition root).

**Why this priority**: Compatibility with the established external reference lets maintainers independently confirm the baseline instead of trusting only internal tooling.

**Independent Test**: Can be fully tested by regenerating the baseline manifest from the pinned reference source text and comparing roots with the published reference root.

**Acceptance Scenarios**:

1. **Given** the pinned baseline source text, **When** a local manifest is generated with the declared normalization, **Then** its edition root matches the published reference root.
2. **Given** a manifest generated with a different normalization than declared, **When** compared to the reference, **Then** the comparison reports a normalization mismatch, not a silent pass.

### Edge Cases

- What happens when the manifest declares an unknown or unsupported normalization? Verification must refuse with a clear diagnostic rather than guessing or silently substituting a default.
- How does the system handle a manifest whose verse count or surah list does not match the text under verification (truncated import, extra verses, wrong numbering)? Verification must fail on structure before comparing hashes.
- What happens when normalization-relevant characters appear (whitespace variants, Unicode composed vs. decomposed forms)? The declared normalization decides the outcome, and the manifest must state it explicitly so re-verification reproduces the same result.
- How are non-Unicode, transliteration, or commentary-bearing sources handled? They must never receive a canonical-text manifest; at most they receive a reference-only or translation-class manifest with their true kind labeled.
- What happens when a translation manifest is presented where a Quran manifest is expected (or vice versa)? The kinds must be distinguished by their root keys and kind labels so substitution is rejected.
- How are historical manifests treated when an edition releases a new version? The old manifest stays valid for the old version only; the new version requires a newly generated manifest linked by version history, never an edit of the old one.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST generate one independent integrity manifest per Quran edition and per transmission (riwayah), where the manifest identity includes edition, edition version, script, and transmission where the source declares one.
- **FR-002**: Each Quran manifest MUST contain verse-scope hashes (one per verse), surah-scope rollups (one per surah, computed by concatenating that surah's verse hashes in ayah order and hashing the concatenation), and a single edition-scope root (computed by concatenating the surah rollups in surah order and hashing the concatenation).
- **FR-003**: Each Quran manifest MUST explicitly declare the hash algorithm (SHA-256) and the exact text normalization applied before hashing (including Unicode form and whitespace handling); verification MUST apply the declared normalization and MUST refuse when the declaration is missing, unknown, or unsupported.
- **FR-004**: Generation MUST be deterministic: the same edition text plus the same declared normalization MUST always produce a byte-identical manifest.
- **FR-005**: System MUST verify a candidate edition text only against the manifest bearing the same edition identity; verification against a manifest of a different edition or transmission MUST result in an identity-mismatch refusal, never a pass and never a corruption verdict against either text.
- **FR-006**: Verification MUST report mismatch location at verse, surah-rollup, and edition-root levels, and MUST detect reordering, missing verses, and extra verses as failures.
- **FR-007**: Differences between two legitimate editions/transmissions MUST be classified as readings/version differences, never reported as corruption or tampering of either edition.
- **FR-008**: System MUST generate hash-only manifests for translations: hashes plus identity metadata (translator, language, source release, retrieval provenance) with no translated sentences stored in the manifest.
- **FR-009**: Translation manifests MUST use a root key name distinct from the Quran edition root key (e.g. a translation-specific root field), and MUST carry a kind label distinguishing them from Quran-text manifests, so the two can never be substituted for one another.
- **FR-010**: Each manifest MUST record its provenance (source text identity and version, retrieval or generation date, and where applicable the upstream source reference) sufficient to reproduce the verification independently.
- **FR-011**: A manifest for one edition version MUST remain valid only for that version; a new edition version MUST receive a newly generated manifest, with version history linking old and new rather than editing the old manifest.
- **FR-012**: Sources that are not bare canonical text in their declared script (non-Unicode encodings without a declared conversion, transliterations, commentary-bearing texts) MUST NOT receive a canonical-text manifest.

### Key Entities

- **Quran Edition Manifest**: Integrity record for exactly one edition + version + transmission; contains verse hashes, surah rollups, one edition root, algorithm and normalization declarations, and provenance.
- **Verse Hash Record**: Hash of one normalized verse text, addressable by surah and ayah number; the unit at which tampering is located.
- **Surah Rollup**: Hash over the ordered concatenation of one surah's verse hashes; the unit at which surah-level completeness is confirmed.
- **Edition Root**: Single hash over the ordered concatenation of all surah rollups; the unit at which whole-edition identity is confirmed and cross-checked against external references.
- **Normalization Declaration**: Explicit statement of the text transformations applied before hashing (Unicode form, whitespace rules, and any other step); part of every manifest and required input to verification.
- **Translation Manifest**: Hash-only integrity record for one translation release (translator, language, source release); carries a kind label and a translation-specific root key distinct from the Quran root.
- **Verification Verdict**: Outcome of checking a candidate text against a manifest: pass, content mismatch with locations, structural mismatch, normalization/identity refusal, or kind-substitution refusal.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A reviewer can generate a manifest for a sample edition and re-generate it on demand, obtaining byte-identical results on every repeat run.
- **SC-002**: Verification of an unaltered sample text against its own manifest reports PASS covering all verses, all surahs, and the edition root within a single check run.
- **SC-003**: Altering any single verse in a sample text is located by verification (correct verse and surah named) in 100% of trials, and reordering or removing a verse is reported as a failure in 100% of trials.
- **SC-004**: Attempting to verify a sample text of one transmission against the manifest of another transmission results in an identity-mismatch refusal in 100% of trials — zero passes and zero corruption verdicts on cross-reading checks.
- **SC-005**: Every translation manifest in a review sample contains zero readable translated sentences (hashes and metadata only) and carries a root field name visibly different from the Quran root field name.
- **SC-006**: A baseline manifest regenerated from the pinned reference source text matches the published quranchecksum-compatible reference root exactly, confirming compatible rollup construction.

## Assumptions

- Hash algorithm is SHA-256 for all manifests in scope; alternative algorithms are out of scope.
- Baseline normalization follows the external checksum reference design: a declared Unicode normal form, stripping of leading/trailing whitespace, and no other silent transformation; each edition's manifest states its own normalization explicitly and different editions may declare different normalizations.
- Rollup construction (concatenate verse hashes in ayah order per surah; concatenate surah rollups in surah order for the edition root) matches the quranchecksum reference design for comparability.
- Verse/surah scope follows the 114-surah, 6,236-verse-class structure of the baseline; editions with legitimately different verse counts structure their manifests to their own declared structure and are never force-fitted to the baseline count.
- Manifests are integrity evidence only: they never substitute for the text source, never authorize redistribution, and never carry licensing authority (per-edition licence records remain separate).
- Transmission (riwayah) identity is taken from what the source declares or from a recorded inference with its basis stated; the system never infers script-to-transmission mappings silently (e.g. a script name alone never implies a transmission).
