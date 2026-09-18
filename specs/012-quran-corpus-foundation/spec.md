# Feature Specification: Quran Corpus Foundation

**Feature Branch**: `012-quran-corpus-foundation`

**Created**: 2026-09-18

**Status**: Draft

**Input**: User description: "Q-ai needs a Quran corpus foundation that supports a primary/default Quran representation while also supporting multiple Quran readings, Arabic editions, and multilingual translations, with explicit provenance, attribution, license/data-rights metadata, and integrity verification."

**Constitution compliance**: `.specify/memory/constitution.md` v1.1.0 — Principles I (canonical immutability), II (layered trust, provenance per row), III (traceability, hashes), IV (attributed translations, no canonical/translation conflation), VI (deny-by-default), VII (schema extensible, no blind merging).

**Existing architecture preserved**: This specification extends, does not replace, the `005-quran-core`, `006-quran-corpus`, `007-quran-normalization`, `008-quran-search` specifications and ADRs `0101`–`0114` (dataset, hashing, difference algorithm, translation attribution, reference-corpus comparison). The `QuranEdition` model, `quran_active_edition` singleton, three frozen hash recipes (ADR-0108), and the 13-gate import pipeline remain the substrate this foundation builds upon.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A new edition is imported without disturbing the primary/default representation (Priority: P1)

An operator imports a verified alternate reading (e.g. Warsh 'an Nafi'). The system records it as a distinct, fully attributed edition. The existing primary/default Hafs 'Uthmani edition remains the active canonical representation, and no schema redesign is required.

**Why this priority**: Multi-riwayah support is the core value of this feature. If importing an alternate reading required a schema change, the foundation fails.

**Independent Test**: Import a second edition with a different `qiraah`/`riwayah`; verify `qai quran edition list` shows both; verify the primary/default edition is unchanged and still addressable; verify no canonical rows were modified by the import (import stops at approval-requested).

**Acceptance Scenarios**:

1. **Given** a manifest for an alternate riwayah with valid provenance, **When** `qai quran import` completes, **Then** the edition appears as `Staged` with its own `id`, `slug`, `riwayah`, `qiraah`, and `script`, and the primary/default edition's hashes and activation pointer are untouched.
2. **Given** an edition whose `riwayah` field is empty/unknown, **When** imported, **Then** it is accepted with `riwayah: unknown` and `qiraah: unknown`, never inferred from script or publisher name.

---

### User Story 2 - A translation is independently identifiable and attributed, not merely keyed by language (Priority: P1)

A user browses English translations. Each translation is listed by its own source/edition identifier and provenance metadata — not collapsed into a single "English" entry. Two English translations from different publishers appear as separate, independently attributable records.

**Why this priority**: The current model already enforces `translator NOT NULL` and `aligned_edition_id` (ADR-0112). This user story extends that to guarantee multiple translations per language remain distinct and traceable.

**Independent Test**: Import two English translations with different translators; verify each is separately addressable; verify neither is presented as canonical; verify each carries its own license, source version, and integrity hash.

**Acceptance Scenarios**:

1. **Given** two English translations with different `translator` values, **When** both are imported and activated, **Then** each is independently selectable and carries its own `slug`, `version`, `translator`, `license`, and `text_hash`.
2. **Given** a translation whose translator name is missing, **When** validated, **Then** import is rejected (`QAI-QUR-0011`), preserving Principle IV.

---

### User Story 3 - An operator inspects the full provenance and data-rights status of any imported dataset (Priority: P2)

An operator runs `qai source show <source-id> --provenance` or the equivalent edition inspector and sees, for each imported data family: the upstream repository, commit/tag/revision, upstream file/path, retrieval timestamp, source hash, transformation history, license state, and attribution. Where any field is unknown, it displays `unknown` / `pending_verification` explicitly — never a guessed value.

**Why this priority**: Provenance and data-rights transparency is a non-negotiable requirement (Principle II, III, IV; constitution §II). Without it, the system cannot distinguish a verified edition from an unverified one, and data-rights compliance becomes unenforceable.

**Independent Test**: For any imported dataset, `qai source show` emits the provenance fields; fields left unresolved by the owner are rendered as `unknown` / `pending_verification`, never omitted or fabricated.

**Acceptance Scenarios**:

1. **Given** an imported edition with a recorded upstream repository and commit, **When** inspected, **Then** the repository, commit/tag, upstream path, and retrieval timestamp are visible.
2. **Given** an edition whose license is unresolved, **When** inspected, **Then** `license: unknown`, `redistribution: unknown`, `status: pending_verification` are shown and the edition cannot be presented as editorially approved.

---

### User Story 4 - The system distinguishes canonical Arabic text, alternate editions, translations, reference corpus, and integrity manifests (Priority: P2)

A researcher compares two compatible Quran datasets. The system classifies each difference as one of: identical, normalization-only, orthographic difference, script difference, qira'ah/riwayah difference, edition difference, tokenization difference, translation difference, or unknown difference — and never labels a qira'ah/riwayah difference as "corruption."

**Why this priority**: This is the conceptual boundary the user explicitly requires (canonical vs alternate vs translation vs reference vs integrity). Without it, legitimate reading variation is misreported as corruption.

**Independent Test**: Invoke the edition differ between two compatible editions; verify differences are classified with a type code; verify a qira'ah difference is never emitted as corruption.

**Acceptance Scenarios**:

1. **Given** two compatible editions differing only in script form, **When** compared, **Then** the difference is classified as `script_difference` or `normalization-only`, not `corruption`.
2. **Given** two editions of different riwayat, **When** compared, **Then** the difference is classified as `qira'ah_riwayah_difference` and the comparison is labeled as comparing compatible-but-distinct datasets.

---

### Edge Cases

- What happens when an upstream identifier changes (repository renamed, commit rewritten)? The external identifier is preserved as-recorded at import time; a new retrieval record is appended, not silently overwritten. Transformation history is append-only.
- How does the system handle a dataset that is open-source but whose redistribution rights are unverified? The dataset is imported with `license: unknown`, `redistribution: unknown`, `status: pending_verification`, and must never be activated as canonical until the owner verifies.
- What if the owner has not named the reference corpus? The reference-corpus comparison (ADR-0114 QV-015) remains a recorded skip, never a silent pass (per existing behavior).
- What if a translation lacks a distinct source identifier? The system requires one; a translation identified only by language code is rejected.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST model at least the following distinct entity types, each with its own identity and lifecycle: **Quran corpus**, **Quran edition**, **Script**, **Qira'ah**, **Riwayah**, **Edition version**, **Source**, **Provenance**, **Attribution**, **License/data-rights**, **Checksum/integrity manifest**, **Language**, **Translation**, **Reference corpus**. (User requirement: "Q-ai must not model the Quran as a single undifferentiated text.")
- **FR-002**: Each edition MUST carry an immutable `external_id` (upstream identifier) preserved exactly as supplied by the upstream source. The `slug` is the Q-ai-internal identifier and MUST NOT silently replace or overwrite the upstream identifier. Unknown upstream identifiers are recorded as `unknown`, never fabricated.
- **FR-003**: Each edition MUST declare `script`, `qiraah`, and `riwayah` explicitly. The system MUST NOT infer `riwayah` from `script` alone, from a publisher name (e.g. "Uthman Taha"), or from any other derived signal. Where the reading is unknown, `riwayah` and `qiraah` are recorded as `unknown`.
- **FR-004**: The system MUST designate one edition as the primary/default representation (currently intended: Script = Uthmani, Riwayah = Hafs 'an Asim). This designation MUST NOT prevent other recognized readings/riwayat and Arabic editions from being imported, activated, and consulted alongside it.
- **FR-005**: Each translation MUST be independently identifiable by a source/edition identifier and provenance metadata, not only by language code. A language MAY have multiple translations, each with its own `slug`, `version`, `translator`, `license`, and `text_hash`.
- **FR-006**: Every imported dataset MUST be traceable to its upstream source via an append-only provenance record recording at least: upstream repository, commit/tag/revision, upstream file/path, upstream identifier, retrieval timestamp, source hash, transformation history, license state, attribution, and validation status.
- **FR-007**: For each imported data family, license/data-rights status MUST be tracked separately. If rights are unknown, the system MUST record `license: unknown`, `redistribution: unknown`, `status: pending_verification`, and MUST NOT invent a license.
- **FR-008**: The system MUST support independent checksums/integrity manifests per compatible Quran dataset, verified against the frozen hash recipes (ADR-0108: `text_hash`, `structure_hash`, `token_order_hash`). Each compatible dataset has its own checksum family; the corpus from one project is NOT treated as a universal checksum for every riwayah.
- **FR-009**: The system MUST preserve source comments and warnings, including OCR-related provenance when supplied by the upstream source. The system MUST NOT assign an invented global "trusted" status; trust is per-source and per-data-family.
- **FR-010**: The system MUST distinguish five data categories: (1) primary/default canonical representation, (2) alternate editions/readings, (3) translation corpus, (4) reference corpus, (5) integrity/checksum manifests. A difference between two riwayat is NOT automatically a corruption.
- **FR-011**: The edition difference/compare model MUST classify differences using at least the taxonomy: `identical`, `normalization-only`, `orthographic_difference`, `script_difference`, `qira'ah_riwayah_difference`, `edition_difference`, `tokenization_difference`, `translation_difference`, `unknown_difference`.
- **FR-012**: The system MUST remain usable even when owner/source metadata (license, publisher, verification status) is still unresolved. Unresolved values are rendered as `unknown`/`pending_verification` and MUST NOT block import, staging, or the integrity pipeline. Only activation of a canonical edition as the primary representation requires the owner's explicit approval.
- **FR-013**: The system MUST enforce deny-by-default on the canonical path: no model may generate or correct canonical text; the canonical path has zero LLM/vector/embedding dependencies; activation requires an `ApprovalToken` derived from a persisted human approval (Principle I, constitution §I).

### Key Entities

- **QuranEdition**: An edition of the Quran identified by `slug` (Q-ai-internal), `external_id` (upstream, preserved verbatim), `script`, `qiraah`, `riwayah`, `version`, `name`, `language`, hashes, `license`, `provenance`, `status`. Extends the existing `QuranEdition` domain model.
- **QuranCorpus**: The overarching corpus container; holds a set of editions and designates the primary/default edition; does not itself carry text — text lives in editions.
- **PrimaryDefaultDesignation**: A record (not a separate table necessarily) identifying which edition is the primary/default representation, tied to the existing `quran_active_edition` singleton concept plus an explicit "primary/default" flag.
- **TranslationEdition**: An attributed translation aligned to a specific `QuranEdition`, with independent `slug`, `version`, `translator`, `language`, `license`, `text_hash`, `trust_level`. Extends the existing `translation_editions` model.
- **ProvenanceRecord**: Append-only upstream trace — repository, commit/tag/revision, upstream file/path, upstream identifier, retrieval timestamp, source hash, transformation history (append-only), license state, attribution, validation status.
- **LicenseRecord / DataRights**: Per-data-family status — `license`, `redistribution`, `status` (`pending_verification` when unknown). Extends the existing `LicenseRecord` / `LicenseStatus` domain model.
- **IntegrityManifest**: Per-edition/per-dataset checksum family — `text_hash`, `structure_hash`, `token_order_hash`, `manifest_hash` — verified against the frozen ADR-0108 recipes.
- **ReferenceCorpus**: An independent corpus used for validation (ADR-0114); comparison results are classified per the difference taxonomy (FR-011). May be absent; QV-015 remains a recorded skip when absent.
- **DifferenceClassification**: The typed result of comparing two compatible datasets, using the taxonomy in FR-011.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The system can represent the primary/default Hafs 'Uthmani edition and at least one additional verified edition/reading (e.g. Warsh 'an Nafi') without schema redesign; both are independently addressable via their `slug` and `external_id`.
- **SC-002**: Importing an edition with an empty/unknown `riwayah` or `qiraah` succeeds with the value recorded as `unknown`; the system never infers riwayah from script or publisher name.
- **SC-003**: Every imported dataset exposes complete provenance fields; any unresolved field is rendered as `unknown`/`pending_verification` and never omitted or fabricated.
- **SC-004**: Each language supports multiple independently identifiable translations, each carrying its own source identifier, translator attribution, license, and hash; no translation is identified only by language code.
- **SC-005**: The edition differ classifies differences using at least the nine-type taxonomy (FR-011); a qira'ah/riwayah difference is never emitted as `corruption`.
- **SC-006**: A dataset with `license: unknown` and `status: pending_verification` can be staged and its integrity verified, but cannot be activated as the primary/default canonical representation without explicit owner approval.
- **SC-007**: Canonical reading and exact verse lookup remain deterministic and model-free (zero LLM/vector/embedding dependency) across all editions, as required by Principle I and VII.
- **SC-008**: The architecture supports adding new editions, scripts, qira'at, and languages without modifying the core edition schema (schema extensibility via the existing `QuranEdition` record and the source catalog).

## Assumptions

- The primary/default designation (Hafs 'Uthmani) is recorded as an explicit flag on the edition record and feeds the existing `quran_active_edition` singleton pointer; the precise CLI mechanism for changing it is an owner/Phase decision and is not specified here.
- `external_id` is a new field added to the edition record; it is populated from the upstream manifest and preserved verbatim. If the upstream provides no identifier, the value is `unknown`.
- The upstream source repositories (`fawazahmed0/quran-api`, `gaitco/quran-database`, `spqrxi/quranchecksum`) are used as reference sources for edition inventory, identifiers, structure, integrity concepts, and checksum methodology. They are NOT blindly copied into the schema; useful concepts are adapted to the existing domain model. The current checksum corpus from `quranchecksum` is treated as a specific compatible representation, not a universal checksum for every riwayah.
- Data-rights per data family is tracked via the existing `source_versions.license_status` / `license_json` and extended with `redistribution` and `status` fields where needed; the "pending_verification" status is a valid, explicitly-rendered state.
- Reference-corpus comparison (ADR-0114, QV-015) remains a recorded skip when no reference corpus is configured; this behavior is preserved and not changed by this feature.
- No license is invented. The owner's sign-off on the dataset, its licensing, and the named reviewer are external prerequisites (ADR-0101) that this feature does not substitute for.
- The existing 13-gate import pipeline (`006-quran-corpus`) and the 12 frozen hash recipes (ADR-0108) remain the integrity substrate; this feature adds fields and classification taxonomies on top.
- "Unknown" values are persisted as explicit sentinels (e.g., the string `unknown` or a `PendingVerification` enum variant), never as `NULL` or omitted, so that the display layer can render them distinctly.
