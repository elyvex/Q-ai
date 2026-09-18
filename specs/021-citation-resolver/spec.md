# Feature Specification: Citation Resolver and Quotation Verification

**Feature Branch**: `021-citation-resolver`

**Created**: 2026-09-19

**Status**: Draft

**Input**: User description: "citation resolver and quotation verification: module-level reverse specification of the implemented citations crate (crates/citations/src/lib.rs, D1.9, ADR-0111, PRD section 12), derived from code truth. Document resolve(), QuotationVerdict (Match/Mismatch/LocationNotFound), edition-pinned quotations, deep links, and hard-failure on answer paths. Code is source of truth. Gap: no spec; 005 covers quran-core quotation types only, not the resolver."

**Source of truth**: `crates/citations/src/lib.rs` (resolver v1 + quotation verification + deep links, D1.9). Related: ADR-0111 (citation identity and deep-link format), PRD §12 (Quran tool result contract) / §12.1 (research checksum) / §21.2–21.3 (claim-level citations, validation) / §35.3 (citation validation), AC-P1-21. Complements `005-quran-core`, which covers the `QuranQuotation` quotation *types* only — this spec covers the *resolver* that verifies quotations against the corpus.

**Constitution compliance**: `.specify/memory/constitution.md` v1.2.0 — Principle I (canonical text never generated/corrected by a model; resolver reads canonical text via `CitationSource`, never model memory), Principle III (every quotation edition-pinned with hash; claim-level citations validated before presentation; reproducibility via stored hash + ingestion version), Principle IV (translations never presented as original; no synthesized consensus).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Verified edition-pinned quotation on every cited ayah (Priority: P1)

A researcher cites a Quran ayah in an answer. The system resolves the citation against the identified edition (`slug@version`), checks the location exists, and verifies the quoted text matches the canonical text byte-for-byte (or after whitespace normalization). The resolved citation carries the verdict, the canonical `sha256:<hex>` hash, and a deep link that opens the exact ayah, not a document homepage.

**Why this priority**: No-fabrication guarantee (PRD §21.3, §35.3). A quotation that does not match its claimed location must never be presented as verified.

**Independent Test**: Call `CitationResolver::resolve()` against a stub `CitationSource` with an exact quotation, a whitespace variant, a tampered quotation, an unknown edition, and an unknown location; assert the five distinct outcomes (ExactMatch / MatchAfterWhitespaceNormalization / Mismatch / EditionNotFound / LocationNotFound) plus hash and deep-link presence on the match paths.

**Acceptance Scenarios**:

1. **Given** a citation whose `quoted_text` is byte-identical to the canonical ayah text in the pinned edition, **When** resolved, **Then** the verdict is `ExactMatch` and the result carries `text_hash` (`sha256:<hex>`) and `deep_link` (`/read/{slug}@{version}/{surah}:{ayah}`).
2. **Given** a citation whose quoted text differs only in whitespace (extra spaces, newlines), **When** resolved, **Then** the verdict is `MatchAfterWhitespaceNormalization` with hash and deep link still present.
3. **Given** a citation whose quoted text differs in content, **When** resolved, **Then** the verdict is `Mismatch` carrying `first_difference_at` (character index) and `expected_hash` (SHA-256 hex of the canonical text).
4. **Given** a citation naming an edition that does not exist, **When** resolved, **Then** the verdict is `EditionNotFound` with no hash and no deep link.
5. **Given** a citation naming a surah:ayah absent from the edition, **When** resolved, **Then** the verdict is `LocationNotFound` with no hash and no deep link.

---

### User Story 2 - Failed quotations block answers, never warn-and-continue (Priority: P1)

A `Mismatch` or `LocationNotFound` verdict on an answer path is a hard failure: the unsupported claim is removed, qualified, or marked as analysis — never rendered as a verified quotation. This is the mechanism the later (Phase 9) citation verifier reuses.

**Why this priority**: AC-P1-21 hard-failure rule. An advisory warning lets fabricated quotations reach readers.

**Independent Test**: Drive an answer-construction path with a citation that resolves to `Mismatch` and one that resolves to `LocationNotFound`; assert the answer pipeline rejects or downgrades the claim (removal, qualification, or analysis label) rather than presenting the quotation as verified.

**Acceptance Scenarios**:

1. **Given** a resolved citation with verdict `Mismatch`, **When** the answer is assembled, **Then** the quotation is not presented as verified (claim removed, qualified, or labeled analysis).
2. **Given** a resolved citation with verdict `LocationNotFound`, **When** the answer is assembled, **Then** the same hard-failure handling applies.

---

### User Story 3 - Stored citations re-verify years later (Priority: P2)

Citations persist with their content hash and ingestion version (`citations` table: `content_hash` + `ingestion_version`). Later, `resolve_stored()` re-opens the same edition/location, recomputes the live hash, and reports `ExactMatch` (hash unchanged) or `Mismatch` (corpus changed) — enabling the reproducibility checksum story (PRD §12.1) and the `GET /api/v1/quran/citations/{id}` re-verification endpoint.

**Why this priority**: Citations must stay re-verifiable across corpus updates; otherwise checksums cannot detect drift.

**Independent Test**: Persist a resolved citation, then call `resolve_stored()` with unchanged corpus (expect `ExactMatch` + current hash + deep link) and with changed corpus text (expect `Mismatch`).

**Acceptance Scenarios**:

1. **Given** a stored citation whose live corpus text hashes to the stored `quoted_text_hash`, **When** re-verified, **Then** the verdict is `ExactMatch` with the current `text_hash` and deep link.
2. **Given** a stored citation whose live text hash differs from the stored hash, **When** re-verified, **Then** the verdict is `Mismatch` (with `first_difference_at: 0` and `expected_hash` of the live text, per current code), with current hash and deep link.

---

### User Story 4 - Stable identity: deep links and URNs (Priority: P2)

Every resolved citation renders a reader deep link and every stored citation has a stable URN, both frozen by ADR-0111, so readers and stored records address the exact ayah in the exact edition version.

**Why this priority**: Citation identity must be fixed before the first citation is stored; links must open the exact location (PRD §13.3).

**Independent Test**: Assert `deep_link("s","1.0.0",2,255) == "/read/s@1.0.0/2:255"` and `citation_urn("s","1.0.0",2,255) == "qai://quran/s@1.0.0/2:255"`.

**Acceptance Scenarios**:

1. **Given** edition `s@1.0.0`, surah 2, ayah 255, **When** the deep link is rendered, **Then** it is exactly `/read/s@1.0.0/2:255`.
2. **Given** the same coordinates, **When** the storage URN is rendered, **Then** it is exactly `qai://quran/s@1.0.0/2:255`.

---

### Edge Cases

- Non-single-ayah locators (ranges, divisions, token refs) parse successfully but resolve to hard `LocationNotFound` — the documented v1 limitation, in the safe direction (never a false match).
- Unparseable `canonical_reference` is an infrastructure error, not a verdict: `CitationError::InvalidReference` (`QAI-QUR-0321`). Backend failures surface as `CitationError::Backend` (`QAI-QUR-0322`); verdicts cover content outcomes only.
- `MatchAfterDeclaredNormalization{rules}` and `AccessDenied` are enum variants reserved by the contract: the current `verdict()` constructor never produces the former (only exact / whitespace / mismatch), and the resolver never denies in single-user mode (documents as reserved). Both remain matchable by downstream consumers (e.g. `verdict_str` persistence mapping covers all seven variants).
- `resolve_stored()` compares hashes only (no quoted text needed); a corrupt stored location or missing row surfaces as a backend error on the API path, not a verdict.
- The resolver never touches storage directly: all corpus access goes through the async `CitationSource` trait (`edition_exists`, `fetch_ayah_text`), implemented by the application layer (`ReaderCitationSource` over `QuranReaderService`, pinned edition selector).
- `first_difference_at` is a character (not byte) index: position of the first differing `char`, or the shorter char-count when one side is a prefix of the other.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Resolver MUST expose `CitationResolver::resolve()` which, for a `Citation` (id, kind, canonical_reference, quoted_text, edition_slug, edition_version, surah, ayah): parses the canonical reference (unparseable → `InvalidReference` / `QAI-QUR-0321`); returns `LocationNotFound` for any non-single-ayah reference; returns `EditionNotFound` when the pinned edition is absent; returns `LocationNotFound` when the ayah text is absent; otherwise verifies the quotation and returns a `ResolvedCitation` (citation_id, verdict, `sha256:<hex>` text_hash, deep link).
- **FR-002**: Quotation comparison MUST implement three outcomes: byte-identical → `ExactMatch`; equal after `split_whitespace`-join-with-single-space on both sides → `MatchAfterWhitespaceNormalization`; otherwise → `Mismatch{first_difference_at, expected_hash}` where `expected_hash` is the SHA-256 hex of the canonical text and `first_difference_at` is the first differing character index.
- **FR-003**: Resolver MUST expose `verify_quotation(citation, quoted)` returning the `QuotationVerdict` for a candidate string by resolving a probe copy of the citation with the substituted quoted text.
- **FR-004**: Resolver MUST expose `resolve_stored(stored: &StoredCitation)` which re-verifies without quoted text: non-ayah reference → `LocationNotFound`; missing edition → `EditionNotFound`; missing ayah → `LocationNotFound`; otherwise compares live `sha256:<hex>` against `stored.quoted_text_hash` → `ExactMatch` on equality, `Mismatch{first_difference_at: 0, expected_hash: <live hex>}` otherwise — always with current hash and deep link on the found path.
- **FR-005**: Every quotation under verification MUST be edition-pinned: `Citation`/`StoredCitation` carry `edition_slug` + `edition_version`, the canonical reference embeds `slug@version` (`quran:{slug}@{version}:{surah}:{ayah}` per ADR-0102/ADR-0111), and resolution opens that exact edition/location — never model memory, never an unversioned lookup.
- **FR-006**: Resolver MUST render stable links: reader deep link `/read/{slug}@{version}/{surah}:{ayah}` and storage URN `qai://quran/{slug}@{version}/{surah}:{ayah}` (ADR-0111 frozen formats; `?highlight=token:{n}` suffix reserved by the ADR for future use).
- **FR-007**: `Mismatch` and `LocationNotFound` verdicts MUST be hard failures on every answer path (claim removed, qualified, or labeled analysis — never presented as verified). The resolver returns verdicts; answer assembly enforces the failure. This is the mechanism the Phase 9 citation verifier reuses.
- **FR-008**: Resolved citations MUST persist with `content_hash` (`sha256:<hex>` of the canonical text at resolve time) and `ingestion_version`, plus verdict string covering all seven variants, so `resolve_stored()` and the checksum story (PRD §12.1) can re-verify later.
- **FR-009**: Corpus access MUST go through the `CitationSource` trait only (`edition_exists(slug, version)`, `fetch_ayah_text(slug, version, surah, ayah)`); the citations crate MUST NOT depend on storage, I/O runtimes, models, or embeddings. Infrastructure failures MUST surface as `CitationError::Backend` (`QAI-QUR-0322`), distinct from content verdicts.
- **FR-010**: Resolver MUST expose `content_hash_of(text)` returning the shared content-hash type (SHA-256) used for stored citations.

### Key Entities

- **Citation**: A single-ayah quotation under verification — id, kind (`Quran` in v1), canonical_reference (`quran:{slug}@{version}:{surah}:{ayah}`), quoted_text, edition_slug, edition_version, surah, ayah.
- **QuotationVerdict**: Seven-variant outcome — `ExactMatch`, `MatchAfterWhitespaceNormalization`, `MatchAfterDeclaredNormalization{rules}` (reserved, never constructed by current code), `Mismatch{first_difference_at, expected_hash}`, `LocationNotFound`, `EditionNotFound`, `AccessDenied` (reserved; never denied in single-user mode).
- **ResolvedCitation**: Resolution result — citation_id, verdict, optional `text_hash` (`sha256:<hex>`, present exactly when the location resolved), optional `deep_link` (present on the same condition).
- **StoredCitation**: Persisted citation for later re-verification — id, canonical_reference, quoted_text_hash (`sha256:<hex>`), edition_slug, edition_version, surah, ayah (no quoted text).
- **CitationSource**: Async corpus surface (`edition_exists`, `fetch_ayah_text`) implemented by the application layer; keeps the crate storage-free.
- **CitationError**: Infrastructure errors only — `InvalidReference{reference}` (`QAI-QUR-0321`), `Backend{detail}` (`QAI-QUR-0322`), each with a stable `code()`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every cited ayah in a research answer carries a verdict, a `sha256:` content hash, and a deep link that opens the exact ayah in the pinned edition — a reader clicking any citation lands on the quoted location, not a homepage (PRD §13.3).
- **SC-002**: Zero quotations that differ from their claimed source reach readers presented as verified: all `Mismatch` and `LocationNotFound` outcomes on answer paths end in removal, qualification, or analysis labeling (AC-P1-21 hard-failure rule).
- **SC-003**: Any stored citation re-verifies deterministically: unchanged corpus reproduces `ExactMatch`; changed corpus surfaces `Mismatch` — supporting checksum-based drift detection (PRD §12.1).
- **SC-004**: Resolver unit suite (exact match with hash/link, whitespace variants and mismatches, missing edition/location/unparseable reference, non-ayah range → `LocationNotFound`, link rendering) passes with zero failures.

## Assumptions

- v1 verifies single ayahs only; ranges, divisions, and token locators are intentionally `LocationNotFound` (documented limitation, safe direction) until a later version extends coverage.
- `MatchAfterDeclaredNormalization{rules}` is part of the frozen verdict contract (AC-P1-21, ADR-0111) but has no producer yet — normalization-rule-aware matching arrives with the normalization/search integration; persistence and display already handle the variant.
- Access control is single-user: `AccessDenied` is reserved in the contract and never produced today; multi-user ACL enforcement is out of scope for this spec.
- Quotation *construction* invariants (restricted `QuranQuotation` type, mandatory edition + hash) are specified in `005-quran-core`; this spec covers *resolution and verification* only.
- The application-layer `ReaderCitationSource` (pinned `EditionSelector`, `QuranReaderService.get_ayah`) and the `citations`-table persistence (`content_hash` + `ingestion_version`, seven-way `verdict_str`) are the current integrations; the resolver itself stays integration-agnostic behind `CitationSource`.
