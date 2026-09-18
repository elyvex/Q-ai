# Feature Specification: Translation Registry and Per-Edition Licensing

**Feature Branch**: `017-translation-registry-licensing`

**Created**: 2026-09-18

**Status**: Draft

**Input**: User description: "/speckit-specify translation registry and per-edition licensing: multiple translations per language with stable upstream/source identity, per-translation license records (status, expression, source URL, redistribution, modification and attribution flags), metadata_only and pending_license_review states, surfacing in the quran.license_status doctor check; translations must never be served or stored as canonical text. Gap: migration 0010 stores translations; no lifecycle/licensing spec."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Browse translations for a language with license visibility (Priority: P1)

A researcher studying a verse wants to see every available translation in their language, know exactly which translation each rendering comes from (translator, source, version), and know whether each rendering is cleared for reading, quotation, or redistribution — before relying on it.

**Why this priority**: This is the core user value: many translations per language without conflation, each attributed and license-labeled. Without it, users cannot distinguish translators, and unverified texts risk being treated as authoritative or redistributable.

**Independent Test**: Can be fully tested by opening the translation inventory for one language containing at least two translations from different translators, confirming each entry shows its own identity and license state, and selecting one translation to read a verse rendering with its attribution attached.

**Acceptance Scenarios**:

1. **Given** a language with two translations from different translators, **When** the researcher lists translations for that language, **Then** both entries appear as separate records, each showing translator name, source/version identity, and its own license state — never merged under the language code alone.
2. **Given** a researcher reading a translated verse, **When** the rendering is displayed, **Then** it is visibly labeled as a translation with its translator and source attribution, visually and structurally distinct from the original Arabic text.
3. **Given** a translation whose license review is not complete, **When** the researcher views it, **Then** only its metadata (translator, language, source identity, license state) is visible and the full translated text is withheld with an explanation, not silently served.

---

### User Story 2 - Record and review per-translation license lifecycle (Priority: P2)

A collection maintainer registers a newly obtained translation, records its license terms from the rights holder or upstream source, and moves it through review (pending review → cleared or restricted / metadata-only) so that only license-cleared translations become readable and redistributable.

**Why this priority**: Per-edition licensing is the governance gate: a repository being openly available never implies its contained translation text is redistributable. Without an explicit per-translation license lifecycle, unverified text could be stored, served, or exported unlawfully.

**Independent Test**: Can be fully tested by registering a new translation with license terms unknown, confirming it is held out of reading flows, then recording verified license terms and confirming it becomes readable/quotable only after the review state clears.

**Acceptance Scenarios**:

1. **Given** a newly registered translation with unverified terms, **When** it is stored, **Then** its license state is `pending_license_review`, its full text is not served to readers, and its registry entry states what is missing and who must verify it.
2. **Given** a translation whose terms forbid redistribution, **When** the maintainer records its license, **Then** the registry shows redistribution as not allowed, reading/quotation remain governed by the recorded terms, and any export or redistribution flow refuses or warns accordingly.
3. **Given** a translation with completed license review, **When** its terms (license expression, source location, redistribution / modification / attribution flags) change, **Then** the change is recorded as a new review event with author and date, and the previous terms remain visible in history rather than being silently overwritten.

---

### User Story 3 - Health check surfaces translation license posture (Priority: P3)

An operator running the routine system health check wants a single license posture summary covering the active canonical reading and all registered translations, so that unknown or pending licenses are noticed and remediated rather than discovered during research or export.

**Why this priority**: Surfacing license state in the existing `quran.license_status` health check makes licensing drift visible in normal operations. It does not change reading behavior itself, so it is sequenced after registry and lifecycle.

**Independent Test**: Can be fully tested by running the health check with (a) all licenses cleared, (b) one translation pending review, and (c) one translation with unknown terms, and confirming the reported status and guidance differ appropriately in each case.

**Acceptance Scenarios**:

1. **Given** all registered translations have cleared licenses, **When** the operator runs the health check, **Then** the `quran.license_status` result reports a passing license posture with per-translation license states available on request.
2. **Given** at least one translation is in `pending_license_review` or has unknown terms, **When** the operator runs the health check, **Then** the `quran.license_status` result flags the condition as needing attention, names the affected translations, and states the remediation (record/verify license terms, owner review).
3. **Given** a translation marked `metadata_only`, **When** the operator runs the health check, **Then** it is reported as intentionally metadata-only (not an error), distinct from `pending_license_review` (which requires action).

---

### Edge Cases

- What happens when two translations share the same language and translator name but come from different upstream sources or versions? Both must coexist as distinct registry entries keyed by stable upstream/source identity, never deduplicated by language + translator alone.
- How does the system handle a translation whose upstream source disappears or changes its terms? The registry retains the last verified license record, marks the entry as needing re-verification, and withholds full text until re-verified.
- What happens when a reader requests a verse in a language with no license-cleared translation? The system states that no cleared translation exists for that language, lists what exists in a restricted state without serving its text, and never substitutes canonical Arabic text as if it were a translation (or vice versa).
- How does the system handle a translation record missing required license fields (status, source location)? It defaults to the most restrictive holding state (`pending_license_review`), never to an open/cleared state, and the missing fields are shown explicitly as unverified rather than omitted or guessed.
- What happens when an export, quotation, or redistribution flow requests a translation that forbids redistribution? The flow refuses or degrades with a license explanation and offers only the permitted uses (e.g., on-screen reading with attribution), never silently exporting the text.
- How does the system prevent a translation from ever being served or stored as canonical text? Any read path that returns canonical text rejects translation-sourced content by construction, and any ingestion path that accepts canonical text rejects translation-sourced content, with a failed check recorded rather than a silent fallback.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST maintain a translation registry in which each translation is a first-class record with its own stable identity, independent of language code.
- **FR-002**: System MUST support multiple coexisting translations per language, each independently identifiable by translator, upstream/source identity, and version.
- **FR-003**: System MUST preserve each translation's upstream/source identifier verbatim as supplied by the source, kept separate from any internally assigned identifier, so the mapping between the two is explicit.
- **FR-004**: System MUST attach a per-translation license record to every registry entry containing: license status, license expression (human-readable license name/identifier), source location where the terms were found, and redistribution-allowed, modification-allowed, and attribution-required flags.
- **FR-005**: System MUST support at minimum the license states `cleared` (terms verified, uses governed by recorded flags), `metadata_only` (intentionally metadata-only; full text withheld by policy), `pending_license_review` (terms unverified; full text withheld pending verification), and `unknown` (no terms recorded; treated as withheld).
- **FR-006**: System MUST withhold the full translated text of any translation in `pending_license_review` or `unknown` state from reading, quotation, search-result display, and export flows, while still showing its metadata (translator, language, source identity, license state, what is missing).
- **FR-007**: System MUST clearly distinguish `metadata_only` (a settled, intentional state — not an error) from `pending_license_review` (an actionable state requiring verification) in every display and report.
- **FR-008**: System MUST record every license determination and license change as a review event carrying who recorded it, when, the evidence/source location consulted, and the resulting status and flags; prior terms MUST remain retrievable and MUST NOT be silently overwritten.
- **FR-009**: System MUST surface per-translation license posture in the existing `quran.license_status` health check: passing when all registered translations are cleared or intentionally `metadata_only`; attention-required when any translation is `pending_license_review` or `unknown`, naming the affected translations and the remediation.
- **FR-010**: System MUST label every served translation rendering with its translator, source/version identity, and license-relevant attribution, displayed distinctly from canonical text.
- **FR-011**: System MUST NEVER serve translated text through any canonical-text read path, and MUST NEVER store translated text as canonical text; translation content and canonical content MUST remain in separate stores/collections with enforced separation.
- **FR-012**: System MUST govern quotation, display, and redistribution of each translation by its own recorded flags: a translation flagged redistribution-not-allowed MUST NOT be exportable or redistributable, and a translation flagged attribution-required MUST carry its attribution in every rendering.
- **FR-013**: System MUST default any translation with missing or unverified license fields to the most restrictive holding state (`pending_license_review`), never to a cleared or open state, and MUST render unverified fields explicitly as unverified rather than omitting or inventing values.
- **FR-014**: System MUST keep translation integrity evidence (content hash) separate from canonical-text integrity evidence, so no translation checksum can certify canonical text and no canonical checksum can certify a translation.

### Key Entities

- **Translation Edition**: One published translation of the corpus (or a defined part of it) by a specific translator/source. Attributes: stable internal identity, verbatim upstream/source identifier, language, translator, aligned canonical edition/version, version string, integrity hash of its own text, current license state. Relationships: many translation editions per language; each aligned to one canonical edition/version; each carrying exactly one current license record plus license history.
- **License Record**: The per-translation data-rights record. Attributes: license status (`cleared` / `metadata_only` / `pending_license_review` / `unknown`), license expression (name/identifier of the terms), source location where terms were found, redistribution-allowed flag, modification-allowed flag, attribution-required flag, reviewer identity, review date, evidence notes. Relationships: belongs to exactly one translation edition; superseded records retained as history, never deleted.
- **Registry Entry (read model)**: The researcher- and operator-facing view of one translation edition plus its current license record and review history. Attributes: display identity (translator, language, source, version), license state with plain-language explanation of permitted uses, attribution string, availability (readable / metadata-only / withheld-pending-review). Relationships: derived from one translation edition and its license record; the health-check summary aggregates across all registry entries.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A researcher can list all translations for a language with at least two translations from different translators and correctly identify the translator, source, version, and license state of each entry on first attempt (100% of entries correctly attributed in acceptance walkthrough).
- **SC-002**: A newly registered translation with unverified terms is withheld from reading, quotation, and export flows while its metadata remains visible, verified in acceptance testing across all three flows (0 servings of withheld text).
- **SC-003**: Every served translation rendering carries its translator and source attribution distinctly from canonical text, verified across a sample of at least 20 renderings (100% labeled, 0 renderings confusable with canonical text).
- **SC-004**: The health check reports a passing license posture when all translations are cleared or intentionally metadata-only, and an attention-required result naming affected translations within one check run whenever any translation is pending review or unknown.
- **SC-005**: No acceptance run serves translated text as canonical text or stores translated text as canonical text (0 cross-contamination events across the full reading and ingestion acceptance suite).
- **SC-006**: Every license change is recorded with reviewer, date, evidence location, and resulting terms, and prior terms remain retrievable (100% of sampled changes traceable to a review event).

## Assumptions

- The existing translation storage (verse-level passages with translator attribution and per-passage provenance) is the persistence substrate; this feature adds the registry identity, per-translation license lifecycle, and health-check surfacing on top of it rather than replacing it.
- License-state vocabulary maps to existing domain concepts as follows: `cleared` corresponds to a verified license record (open, public-domain, permission-granted, or user-owned terms with evidence); `unknown` and `pending_license_review` correspond to unverified terms that withhold full text; `metadata_only` is an intentional settled state. Exact internal naming is a planning concern; user-visible meanings are as defined in FR-005.
- A repository being openly available never implies its contained translation text is redistributable; every translation requires its own verified license record regardless of repository openness.
- License verification and dataset sign-off by the named owner/editorial reviewer are external human prerequisites that this feature records and gates on but does not substitute for; pending human decisions do not block building the registry, lifecycle, and health-check mechanics.
- The primary/default canonical representation (Uthmani script + Hafs transmission) and the active-edition pointer are unaffected; translation licensing never changes which canonical edition is active, and no translation checksum participates in canonical integrity.
- Upstream translation sources are treated as untrusted until validated through the staged import path (staging → validation → human approval → activation → audit); network retrieval details and storage mechanics are planning concerns outside this specification.
