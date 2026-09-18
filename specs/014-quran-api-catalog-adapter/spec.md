# Feature Specification: quran-api Catalog Adapter

**Feature Branch**: `014-quran-api-catalog-adapter`

**Created**: 2026-09-18

**Status**: Draft

**Input**: User description: "quran-api catalog adapter: offline ingestion of the pinned upstream editions.json catalog into Q-ai staging as metadata-only entries that preserve the exact upstream_edition_slug, catalog key, author, language, direction, source, comments and quality flags, with repository revision pinning and per-edition license defaulting to unknown; classify type as quran_text vs translation vs tafsir vs transliteration; never activate canonical text and never vendor restricted translation text."

**Constitution compliance**: Principles II (layered trust, staged import), IV (translations attributed, never presented as original), VI (local-first, deny-by-default, untrusted internet sources, no network fetch), VIII (verbatim upstream slugs, per-family licensing, metadata-only on unverified terms, unknowns stay unknown).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Ingest pinned catalog as metadata-only staged entries (Priority: P1)

An operator supplies a locally saved copy of the upstream edition catalog
together with the repository revision it was taken from. The adapter reads
every entry into staging as a metadata-only record: the upstream slug and
catalog key are preserved byte-for-byte, descriptive fields are copied
verbatim, each entry is classified (Quran text, translation, tafsir, or
transliteration), quality flags are derived from the upstream notes, and the
license starts at unknown. Nothing ingested here can become active or visible
as canonical text.

**Why this priority**: This is the trust boundary for all upstream data. Every
downstream step (coverage review, integrity manifests, licensed import) depends
on identifiers surviving ingestion unchanged and on unverified text never
leaking into the canonical corpus.

**Independent Test**: Ingest a sample catalog file with a known revision; every
valid entry appears in staging with byte-identical identifiers and metadata,
correct classification, license unknown, and zero entries Active.

**Acceptance Scenarios**:

1. **Given** a valid catalog file and a revision pin, **When** the operator
   runs the ingest, **Then** every valid entry is staged with its upstream
   slug, catalog key, author, language, direction, source, comments, and links
   preserved exactly, each entry classified, each license set to unknown.
2. **Given** a staged entry, **When** the operator inspects visibility,
   **Then** the entry is metadata-only and unreachable as canonical text
   through any read path.
3. **Given** an ingest request without a repository revision, **When** the
   operator runs it, **Then** the ingest is refused before reading any entry.

---

### User Story 2 - Review the coverage and gap report (Priority: P2)

After ingestion, the operator reviews a coverage report over the Arabic-script
inventory: which entries are Quran-text candidates, which carry a
slug-stated transmission, which have unknown reading/transmission, and which
are excluded from canonical candidacy (transliterations, non-Unicode
encodings, commentary). The report names every exclusion reason so the
operator can confirm nothing eligible was silently dropped.

**Why this priority**: Selection of any canonical dataset is an owner decision
that must rest on a complete, honest inventory — not on an assumption that a
catalog of variants equals a set of verified readings.

**Independent Test**: Ingest the sample catalog and read the coverage report;
counts by classification reconcile with the staged entries, and every excluded
entry names its reason.

**Acceptance Scenarios**:

1. **Given** a completed ingest, **When** the operator opens the coverage
   report, **Then** it lists counts by classification plus the per-entry
   reasons for every exclusion from canonical candidacy.
2. **Given** an Arabic-script text entry whose transmission is not stated,
   **When** it appears in the report, **Then** its reading and transmission
   are shown as unknown — never filled in by inference.

---

### User Story 3 - Reproducible re-ingestion (Priority: P3)

The operator re-runs the ingest later with the same file and revision (for
example, to rebuild staging or to verify a colleague's environment). The
outcome is identical to the first run: same entries, same classifications,
same flags — reported as a match, not duplicated.

**Why this priority**: Reproducibility is what makes the catalog a stable
reference for research and for every downstream import decision.

**Independent Test**: Run the ingest twice with identical inputs; the second
run reports an identical result with no duplicate staged entries.

**Acceptance Scenarios**:

1. **Given** a completed ingest, **When** the operator repeats it with the
   same file and revision, **Then** the result is reported identical and no
   duplicate entries are created.
2. **Given** a repeated ingest with a different revision, **When** it runs,
   **Then** it is recorded as a separate ingest pinned to the new revision,
   not merged into the old one.

---

### Edge Cases

- The catalog file is missing, unreadable, or not valid JSON: the ingest fails
  with an operator-facing error and zero entries are staged (atomic — no
  partial ingest).
- The top-level shape is not an object of entries: same fail-closed behavior
  as malformed JSON.
- An entry is missing required identity fields (slug, catalog key, language):
  that entry is quarantined and listed in the ingest report; all other valid
  entries still ingest.
- Two entries share a slug or key: both are flagged as duplicates in the
  report; duplicates never resolve silently by position or order.
- An entry carries fields beyond the known set: the known fields ingest
  normally and the extra fields are noted in the report, never silently
  promoted into the record.
- Comments mention OCR, non-Unicode encoding, missing diacritics, or a
  source version: the corresponding quality flags are attached; flags never
  block ingestion.
- A commentary entry is identifiable only by author names with no explicit
  type field: it is classified as a Quran-text candidate with unknown
  transmission plus a review flag, surfaced in the coverage report for human
  confirmation — never auto-labeled tafsir, never auto-labeled a reading.
- Links to edition files are recorded verbatim and never fetched; the ingest
  performs no network access.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The adapter MUST accept a catalog file and a repository revision
  pin as required inputs, and MUST refuse the ingest when the revision is
  missing.
- **FR-002**: The adapter MUST parse the catalog as an object of entries and
  MUST fail closed with zero staged entries when the file is unreadable,
  malformed, or wrongly shaped.
- **FR-003**: The adapter MUST preserve each entry's upstream slug and catalog
  key byte-for-byte; renaming, normalizing, or merging identifiers is
  forbidden.
- **FR-004**: The adapter MUST copy author, language, direction, source,
  comments, and edition links verbatim into the staged record.
- **FR-005**: The adapter MUST classify every entry with deterministic rules
  applied in this order: (1) explicit commentary evidence in the slug
  (tafsir/tafseer/commentary) → tafsir; (2) transliteration evidence
  (a `-la`/`-lad` slug suffix, a Latin-direction entry, or
  phonetic/transliteration wording in the slug) → transliteration;
  (3) Arabic-language right-to-left entry → quran_text;
  (4) anything else → translation.
- **FR-006**: The adapter MUST leave reading and transmission as unknown for
  every entry unless the upstream slug or name states the transmission, and
  MUST record which slug/name wording the assignment came from when it does;
  inferences such as script-implies-transmission are forbidden.
- **FR-007**: The adapter MUST derive quality flags from the upstream notes,
  at minimum: OCR-derived, non-Unicode encoding, missing diacritics,
  transliteration, stated source version, and missing source.
- **FR-008**: The adapter MUST default every entry's license to unknown and
  MUST NEVER inherit the upstream repository's own license onto entry data;
  operator-supplied per-edition terms MUST be recorded together with their
  evidence.
- **FR-009**: Every ingested entry MUST land in staging as metadata-only; no
  path through this adapter may make an entry active or visible as canonical
  text.
- **FR-010**: The adapter MUST NOT fetch linked edition files or perform any
  network access during ingest; links are recorded metadata only.
- **FR-011**: The adapter MUST record repository, revision, file identity,
  and retrieval date on every ingest, and re-ingestion with identical inputs
  MUST report an identical result without duplicating entries.
- **FR-012**: Entries missing required identity fields MUST be quarantined
  and listed; duplicate slugs or keys MUST be reported; neither case may
  block the valid remainder of the file.
- **FR-013**: The adapter MUST produce an ingest report (entry counts by
  classification, flags attached, quarantined entries, duplicates) and a
  coverage report (canonical candidates, unknown transmissions, exclusions
  with reasons) for operator review.

### Key Entities

- **CatalogIngest**: One operator-run ingestion of a catalog file, pinned to a
  repository revision; carries file identity, retrieval date, and outcome.
- **CatalogEntry**: A staged metadata-only record for one upstream edition:
  verbatim slug, catalog key, descriptive fields, classification, quality
  flags, license (default unknown), and retrieval provenance.
- **Classification**: Exactly one of quran_text, translation, tafsir,
  transliteration, assigned by the deterministic FR-005 rules.
- **QualityFlag**: An observed provenance/quality note (OCR-derived,
  non-Unicode, missing diacritics, transliteration, stated version, missing
  source, review-needed).
- **LicenseRecord**: Per-entry rights state — status, terms wording, terms
  source, and redistribution/modification/attribution flags; unknown until
  evidence is supplied.
- **IngestReport / CoverageReport**: The operator-facing outcomes of a run —
  counts, flags, quarantined entries, duplicates, candidates, unknowns, and
  exclusion reasons.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of valid entries are staged with byte-identical upstream
  slugs, catalog keys, and descriptive metadata.
- **SC-002**: Zero ingested entries are reachable as active or visible
  canonical text through any read path.
- **SC-003**: 100% of entries carry license unknown unless the operator
  supplied per-edition terms with recorded evidence.
- **SC-004**: Re-ingestion with identical file and revision reproduces an
  identical result with zero duplicate entries.
- **SC-005**: 100% of Arabic-script text entries without a stated transmission
  are recorded with unknown reading/transmission; zero silent assignments.
- **SC-006**: A malformed or unreadable catalog file stages zero entries and
  reports a single operator-facing failure.
- **SC-007**: An operator can ingest the full catalog and review the coverage
  report end to end in under 10 minutes.

## Assumptions

- The operator supplies the catalog file from local storage; the adapter
  performs no download, discovery, or network access (constitution VI, VIII).
- The catalog shape matches the verified upstream format: a JSON object keyed
  by catalog ids, each value carrying slug, author, language, direction,
  source, comments, and edition links.
- The repository revision (commit or tag) is supplied by the operator; the
  adapter does not resolve, guess, or fetch revisions.
- Staging uses the existing source-catalog lifecycle: ingested entries rest in
  a staged, metadata-only state and require separate human approval for any
  promotion.
- The full catalog is on the order of five hundred entries; scale beyond an
  order of magnitude more is out of scope for this version.
- Out of scope: downloading edition texts, activating any canonical corpus,
  clearing per-edition licenses (owner/legal decision), checksum manifests,
  morphology, and reference-corpus comparison — each is a separate concern
  with its own decision record.
