# Feature Specification: Upstream Source Coverage

**Feature Branch**: `018-upstream-source-coverage`

**Created**: 2026-09-18

**Status**: Draft

**Input**: User description: "upstream source registry and coverage reporting: pinned upstream repository records with revision, role and data-redistribution posture, plus a reproducible coverage/gap report over the Arabic edition inventory distinguishing slug-named transmissions from unknown qiraah/riwayah and excluding non-Unicode, transliteration and commentary entries from canonical candidacy; report metadata only, no restricted text vendored. Gap: sources::upstream types exist un-specced; extends 013-source-catalog."

**Constitution compliance**: `.specify/memory/constitution.md` v1.2.0, Principles II (layered trust / provenance per record), III (traceability / reproducibility), IV (scholarly honesty — unknowns stay unknown, no merged corruption verdicts), VI (local-first, deny-by-default), VIII (multi-edition governance: verbatim upstream slugs, staged ingestion, per-family licensing, metadata-only for unverified data).

**Extends**: `013-source-catalog` (source lifecycle, approval gates, lineage). This feature specs the upstream-repository registry layer that 013 left un-specced, plus the coverage/gap report built on top of it.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Review pinned upstream repository records (Priority: P1)

A researcher or operator opens the upstream registry and sees every known upstream repository with its stable identifier, repository location, role in Q-ai (edition catalog, database reference, or integrity reference), repository licence, data-redistribution posture, exact pinned revision, verification date, and a short factual note. Repository licence and data-redistribution posture are shown as separate fields so nobody reads "open repository" as "redistributable text."

**Why this priority**: Everything downstream (catalog import, integrity checks, licensing decisions) depends on knowing exactly which upstream revision the recorded facts were verified at, and on never confusing repository openness with data rights. This is the trust anchor for the coverage report.

**Independent Test**: With no network access, list all known upstreams and confirm each record shows a full-length pinned revision, a verification date, a role, and a redistribution posture; confirm that looking up an unknown identifier yields an explicit "not found" rather than a guess.

**Acceptance Scenarios**:

1. **Given** the registry with three verified upstreams, **When** an operator lists all records, **Then** each record shows repository, role, repository licence, data-redistribution posture, pinned revision, verification date, and factual notes.
2. **Given** an operator looks up an unknown repository identifier, **When** the lookup runs, **Then** the result is explicitly "unknown / not found" (fail-closed), never a best-match substitution.
3. **Given** an operator filters by role (e.g. integrity reference), **When** the filter runs, **Then** only repositories with exactly that role are returned.

---

### User Story 2 - Generate reproducible Arabic-edition coverage/gap report (Priority: P1)

A curator generates the coverage/gap report over the Arabic edition inventory at the pinned catalog revision. The report states total catalog size, language count, Arabic-entry count, and for each Arabic entry its verbatim upstream identifiers, declared transmission (only where the upstream slug/name states one), unknown qiraah/riwayah where the upstream declares none, content classification, quality flags, and licence posture. Re-running the report against the same pinned revision produces byte-identical output.

**Why this priority**: This is the deliverable that turns "we have an upstream catalog" into an auditable statement of what transmissions are actually covered and where the gaps are — without vendoring any restricted text.

**Independent Test**: Generate the report twice from the same pinned revision and compare outputs for equality; check that the report header names the exact upstream revision and retrieval facts, and that the Arabic inventory section accounts for every `ara-*` entry at that revision.

**Acceptance Scenarios**:

1. **Given** the pinned edition-catalog revision, **When** the curator generates the report, **Then** the report header records repository, revision, generation date, inventory counts (total entries, language count, Arabic-entry count), and the report body accounts for every Arabic entry at that revision.
2. **Given** the same pinned revision, **When** the report is generated twice, **Then** both outputs are identical (reproducible).
3. **Given** an upstream revision different from the pinned one, **When** the report is generated, **Then** the header revision differs and the output is not presented as the pinned-revision report.

---

### User Story 3 - Distinguish canonical candidates from excluded entries (Priority: P2)

A curator reviewing the report can tell at a glance which Arabic entries are candidates for canonical consideration and which are excluded, and why: slug-named transmissions (transmission read from the upstream name, marked as inferred) vs. unknown qiraah/riwayah (upstream declares none — stays unknown, never guessed); non-Unicode editions, transliterations, and commentary entries are explicitly marked excluded from canonical candidacy with the reason stated.

**Why this priority**: Principle VIII / scholarly honesty — unknowns must stay unknown, script/transmission/edition must stay separate, and non-text or non-importable entries must never silently enter the canonical corpus.

**Independent Test**: Inspect the report's classification column for the known inventory: slug-named transmission entries carry an "inferred from upstream name" marker, undeclared entries show unknown reading/transmission, and every non-Unicode, transliteration, and commentary entry shows an exclusion reason.

**Acceptance Scenarios**:

1. **Given** an Arabic entry whose upstream name states a transmission (e.g. Warsh, Qalun), **When** shown in the report, **Then** the transmission is recorded with evidence marked as inferred from the upstream name, not as an upstream-declared field.
2. **Given** an Arabic entry whose upstream record declares no transmission, **When** shown in the report, **Then** reading and transmission are shown as unknown, never filled in by analogy (e.g. script name never implies a transmission).
3. **Given** a non-Unicode, transliteration, or commentary entry, **When** shown in the report, **Then** it is marked excluded from canonical candidacy with a stated reason (non-Unicode needs conversion / not Arabic script / not bare Quran text), and appears only as metadata.

---

### Edge Cases

- Upstream moves: the pinned revision in the registry no longer matches the upstream's current head — the report header must still name the pinned revision it was generated from; a stale-revision warning is shown rather than silently mixing revisions.
- Unverified licence: any entry whose per-item redistribution terms are not verified is reported as metadata only (`unknown` / `pending review`) and no text from it is vendored, regardless of how open the containing repository is.
- Commentary ambiguity: an entry whose commentary status is inferred (e.g. from author names) rather than declared in its slug must be flagged as needing verification before final classification.
- Empty or partial inventory: if the catalog snapshot cannot be read or contains zero Arabic entries, report generation fails with a clear error rather than producing an empty "full coverage" report.
- Unknown identifier handling: any lookup, filter, or report reference to a repository or edition identifier that is not in the registry resolves to an explicit unknown, never to a nearest match.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST maintain a registry of known upstream repositories where each record includes stable internal identifier, repository location, role (edition catalog / database reference / integrity reference), repository licence, data-redistribution posture (separate from repository licence), exact pinned revision, verification date, and factual notes.
- **FR-002**: System MUST require every registry record to carry a pinned full revision and verification date; records without a pinned revision MUST be distinguishable as unpinned and MUST NOT be presented as verified.
- **FR-003**: System MUST resolve unknown repository identifiers fail-closed (explicit "not found"), and MUST filter records by role returning only exact-role matches.
- **FR-004**: System MUST treat repository licence as covering only software/packaging: an open repository licence MUST NEVER by itself mark contained data text as redistributable; only an explicit per-item or hash-only posture permits redistribution reasoning.
- **FR-005**: System MUST produce a coverage/gap report over the Arabic edition inventory at a named pinned revision, recording in its header the repository, revision, generation date, and measured counts (total entries, distinct languages, Arabic entries).
- **FR-006**: System MUST list every Arabic entry in the report with verbatim upstream identifiers (both identifier forms where the upstream provides two), upstream-stated author/source/notes, declared-vs-inferred transmission with evidence label, content classification, quality flags, and licence posture.
- **FR-007**: System MUST mark transmission assignments derived from upstream slug/name as inferred, and MUST leave reading/transmission as unknown wherever the upstream record declares none; the system MUST NOT infer transmission from script, orthography, or catalog size.
- **FR-008**: System MUST mark non-Unicode editions as reference-only (not directly importable as canonical text without a declared conversion), transliterations as not Arabic script, and commentary entries as not bare Quran text — all excluded from canonical candidacy with a stated reason.
- **FR-009**: System MUST flag commentary classifications that rest on inference (rather than a slug declaration) as needing verification before final classification.
- **FR-010**: System MUST make the report reproducible: the same pinned revision and inputs MUST yield identical output; the report MUST be metadata only and MUST NOT vendor verse text, translation text, or any redistribution-restricted material.
- **FR-011**: System MUST keep the registry and the verified-facts reference document consistent: any change to a pinned revision, role, licence, or redistribution posture in one MUST be reflected in the other.

### Key Entities

- **Upstream Repository Record**: One external repository Q-ai reads from or references; attributes: stable identifier, repository location, role, repository licence, data-redistribution posture, pinned revision, verification date, factual notes.
- **Arabic Edition Inventory Entry**: One `ara-*` catalog entry at the pinned revision; attributes: verbatim upstream identifiers, upstream-stated author/source/notes, content classification (Quran text / translation / commentary / transliteration / reference), transmission assignment plus evidence label (declared vs. inferred vs. unknown), quality flags, licence posture, canonical-candidacy verdict with reason.
- **Coverage/Gap Report**: Reproducible, metadata-only document over the Arabic inventory; attributes: upstream repository + pinned revision, generation date, measured counts, per-entry classifications, gap conclusions (named transmissions covered only as slug names, unknowns, exclusions), staleness indicator when the upstream has moved.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An operator can list all known upstream records with revision, role, and redistribution posture in under 2 minutes, with zero records missing a pinned revision.
- **SC-002**: Re-running the coverage report against the same pinned revision produces identical output on every run (100% reproducibility over repeated generations).
- **SC-003**: The report accounts for 100% of Arabic entries at the pinned revision (every entry appears exactly once with a classification and candidacy verdict — no silent drops).
- **SC-004**: 100% of entries whose upstream record declares no transmission are shown as unknown (zero guessed readings/transmissions), and 100% of slug-derived transmissions carry an inferred-evidence label.
- **SC-005**: 100% of non-Unicode, transliteration, and commentary entries are marked excluded from canonical candidacy with a stated reason, and the report contains zero vendored verse/translation text passages.
- **SC-006**: 9 out of 10 curators reviewing the report correctly identify which transmissions are covered, which are gaps, and which entries are excluded — without mistaking repository openness for data redistribution rights.

## Assumptions

- The three already-verified upstreams (edition catalog, database reference, integrity reference) and their pinned revisions are the starting registry content; adding a fourth upstream is out of scope for this feature.
- The Arabic inventory counts at the pinned edition-catalog revision (total entries, language count, Arabic-entry count) are taken from the verified-facts reference; recounting them from a live upstream fetch is out of scope.
- Report consumers are curators, reviewers, and downstream import planners — not end readers of Quran text; the report is an internal governance artifact.
- Licence verification workflow (per-item clearance) lives in the translation-registry/licensing area; this feature only reports the posture, it does not clear licences.
- Actual adapter ingestion of edition files (downloading/converting text) is out of scope; this feature covers registry records and the metadata-only report.
- The verified-facts reference document remains the human-readable companion to the registry; the report does not replace it.
