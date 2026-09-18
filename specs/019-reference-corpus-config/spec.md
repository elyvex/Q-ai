# Feature Specification: Reference Corpus Configuration

**Feature Branch**: `019-reference-corpus-config`

**Created**: 2026-09-18

**Status**: Draft

**Input**: User description: "reference corpus configuration for QV-015: named independent reference corpora with retrieval pins, the typed reference comparison procedure, difference acknowledgment carrying reviewer identity, and explicit recorded-skip behavior with operator-visible status while no corpus is configured. Gap: ADR-0114 proposes it; no spec."

**Constitution compliance**: `.specify/memory/constitution.md` v1.2.0, Principles I (canonical text never generated/corrected; QV-015 gates activation), II (import staging → validation → human approval → activation → audit; no self-activating import), III (claim-level traceability, edition-pinned retrieval, reproducibility), IV (readings differences side-by-side, never corruption verdicts), VIII (comparison is typed — reference is one kind; differences classified; unverified fields stay unknown; pending owner decisions never block unrelated engineering). Companion proposal: ADR-0114 (Draft, pending sign-off); sibling specs: 015-edition-integrity-manifests, 016-typed-corpus-comparison.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Pin a named reference corpus for a validation run (Priority: P1)

A curator preparing an import names exactly which independent reference corpus the run must compare against, pinning it to a specific recorded version so the comparison is reproducible. The run records the corpus name and pin alongside its results, so a later reviewer can tell precisely which reference text was used — not just "a reference was used".

**Why this priority**: Without a named, pinned reference, a QV-015 verdict is unverifiable: nobody can re-run it or know whether the reference changed underneath it. This is the foundation everything else builds on.

**Independent Test**: Can be fully tested by registering two named reference corpora, pinning a run to one of them, and confirming the run record names that corpus and pin — then re-running with the same pin and confirming identical comparison inputs.

**Acceptance Scenarios**:

1. **Given** a registry holding at least two named reference corpora, **When** a run pins one of them by name and version, **Then** the run record states that corpus name and pin, and the comparison uses that exact text.
2. **Given** a run pinned to a specific reference version, **When** the reference corpus is later updated to a new version, **Then** re-running with the old pin still compares against the old text (the pin is stable, never silently re-pointed).
3. **Given** a run request naming a reference corpus that is not registered or not retrievable, **When** submitted, **Then** it is rejected with a diagnostic naming the missing reference, and the run never proceeds as if a reference had been compared.

---

### User Story 2 - Run the typed reference comparison procedure (Priority: P1)

A reviewer runs the QV-015 reference check as a declared kind-reference comparison: every verse of the imported edition is compared against the pinned reference, and every difference carries exactly one classification from the shared comparison vocabulary (same, normalization_only, orthographic, script, riwayah, edition, tokenization, translation, unknown_difference). The report states the declared kind, both operand identities, and the normalization applied, so the verdict is reproducible from the report alone.

**Why this priority**: This is the QV-015 decision procedure itself. Typing it (kind: reference) is what stops a legitimate readings delta from being misread as corruption, per ADR-0114 and Constitution Principle VIII.

**Independent Test**: Can be fully tested with a small crafted edition/reference pair containing a known identical verse, a known whitespace-only delta, and a known wording delta, confirming the report carries kind reference and the expected per-difference classifications.

**Acceptance Scenarios**:

1. **Given** a pinned reference and an imported edition with identical text, **When** the reference comparison runs, **Then** the report is labeled kind reference with zero substantive differences and a passing QV-015 outcome.
2. **Given** a pinned reference and an edition differing in known ways, **When** the comparison runs, **Then** every differing verse is listed with exactly one classification, and unattributable differences are labeled unknown_difference with evidence preserved.
3. **Given** a reference whose script, transmission, numbering scheme, or basmala policy is incompatible with the imported edition, **When** the comparison runs, **Then** the report records a typed incompatibility finding (wrong reference for this edition), never a corruption verdict against either text.

---

### User Story 3 - Acknowledge differences with reviewer identity (Priority: P1)

A named human reviewer examines the reference comparison report and explicitly acknowledges each substantive difference (or the report as a whole), recording their identity alongside the acknowledgment. The import/activation gate fails QV-015 on any unacknowledged substantive difference, and an acknowledgment without a recorded reviewer identity is rejected. The reviewer must be someone other than the person who prepared the import.

**Why this priority**: "Zero differences, or differences explicitly acknowledged" is the QV-015 rule (ADR-0114 §4). An acknowledgment without an identity is unauditable; self-acknowledgment defeats the two-person gate.

**Independent Test**: Can be fully tested by running a comparison with one substantive difference: unacknowledged it fails the gate; acknowledged with a named reviewer distinct from the preparer it satisfies the gate; acknowledged without identity (or by the preparer) it is rejected.

**Acceptance Scenarios**:

1. **Given** a reference report with one substantive difference and no acknowledgment, **When** the gate evaluates QV-015, **Then** the outcome is FAIL naming the unacknowledged difference.
2. **Given** the same report with the difference acknowledged by a named reviewer distinct from the import preparer, **When** the gate evaluates QV-015, **Then** the outcome records the acknowledgment with reviewer identity and no longer fails on that difference.
3. **Given** an acknowledgment missing reviewer identity, or signed by the same person who prepared the import, **When** submitted, **Then** it is rejected with a diagnostic stating the requirement (named reviewer, distinct from preparer).

---

### User Story 4 - See the explicit recorded skip while no corpus is configured (Priority: P2)

An operator checking system status with no reference corpus configured sees an explicit, operator-visible statement that the reference check was skipped because no reference is available — in the validation report and on the status surface. The skip means "not evaluated", never "verified" and never a failure, and it never blocks unrelated work.

**Why this priority**: QV-015 behavior must not change silently: a skip must stay visible as a skip (never a hidden pass, never a new failure) while the editorial decision on the actual corpus (owner decision OD-03) is still pending.

**Independent Test**: Can be fully tested by running the check with no reference configured and confirming the report and status surface both show the skip with reason, then configuring a reference and confirming the skip disappears in favor of a real comparison.

**Acceptance Scenarios**:

1. **Given** no reference corpus is configured, **When** the reference check runs, **Then** the outcome is a recorded skip stating the reason (reference unavailable), and the overall run neither passes on its behalf nor fails because of it.
2. **Given** no reference corpus is configured, **When** an operator views the status/health surface, **Then** it shows the reference-corpus gap explicitly (check skipped, configuration pending) rather than omitting it.
3. **Given** historical reports recorded before any reference was configured, **When** read after this feature, **Then** their skip entries still read as "not evaluated", never reinterpreted as verification.

---

### Edge Cases

- What happens when a run manifest requests a specific reference corpus but that corpus is unavailable at run time? The run fails closed with a diagnostic naming the missing corpus; the comparison is never silently skipped in place of a requested reference.
- What happens when the pinned reference text changes content under an existing pin (integrity mismatch on retrieval)? The run is rejected as a stale/compromised pin rather than compared against substituted text.
- What happens when normalization-only differences appear in a reference comparison? They classify as normalization_only relative to the declared normalization and do not require reviewer acknowledgment as substantive differences.
- What happens when an acknowledgment references a different comparison run than the one being gated (stale acknowledgment replayed onto new evidence)? The gate rejects it; acknowledgments bind to the exact comparison evidence they reviewed.
- What happens when several named reference corpora exist? Each run pins exactly one; reports never merge findings across references into a single verdict.
- What happens when a translation is offered as a reference corpus? It is rejected as a reference operand: reference comparison covers text-to-text only, and translations are never framed as variants of the original.
- What happens when legacy QV-015 reports (pre-pinning, pre-typing) are read? Their skip semantics are unchanged, and any legacy difference entries without classification are treated as unknown_difference, never re-labeled retroactively.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST maintain a registry of named, independent reference corpora, where each entry carries a stable name, its source identity, its version, and a retrieval pin binding the entry to the exact text to be compared.
- **FR-002**: Every validation run that performs the reference check MUST record the pinned corpus name and pin alongside its results, sufficient for an independent reviewer to retrieve the identical reference text later.
- **FR-003**: A run requesting a named reference corpus that is unavailable or fails its integrity check MUST fail closed with a diagnostic naming the missing corpus; the system MUST never substitute a different corpus and MUST never silently convert a requested comparison into a skip.
- **FR-004**: The reference comparison MUST execute as a declared kind-reference comparison under the shared typed-comparison rules: the report states the kind, both operand identities, and the normalization applied, and every reported difference carries exactly one classification from the closed vocabulary.
- **FR-005**: A reference corpus whose script, transmission, numbering scheme, or basmala policy is incompatible with the compared edition MUST produce a typed incompatibility finding; the system MUST NOT report it as corruption of either text.
- **FR-006**: The gate MUST fail QV-015 on any substantive reference difference that has no recorded acknowledgment; differences classified as normalization_only or same MUST NOT require acknowledgment.
- **FR-007**: Every difference acknowledgment MUST record the reviewer's identity; an acknowledgment without reviewer identity MUST be rejected, never stored.
- **FR-008**: The acknowledging reviewer MUST be distinct from the person who prepared the import; self-acknowledgment MUST be rejected with a diagnostic.
- **FR-009**: Each acknowledgment MUST bind to the exact comparison evidence it reviewed; replaying an acknowledgment against different evidence MUST be rejected.
- **FR-010**: While no reference corpus is configured for a run, the QV-015 outcome MUST be an explicit recorded skip stating the reason (reference unavailable); the skip MUST mean "not evaluated" — never "verified" and never a failure.
- **FR-011**: The operator-visible status surface MUST show the reference-corpus gap explicitly while unconfigured (check skipped, configuration pending); it MUST NOT omit the check and MUST NOT present the skip as verification.
- **FR-012**: Legacy QV-015 reports MUST keep their meaning after this feature: historical skips still read as "not evaluated", and legacy difference entries without classification MUST be treated as unknown_difference, never retroactively re-labeled.

### Key Entities

- **Reference Corpus**: A named, versioned, independently sourced text registered for comparison use only; carries source identity, version, retrieval pin, and its own licence terms (comparison outputs inherit the source licence; no corpus is bundled until its licence is cleared).
- **Retrieval Pin**: The stable binding between a registry entry and the exact reference text bytes for one version; re-running with the same pin always yields the same comparison inputs.
- **Reference Comparison Report**: The kind-reference verdict artifact for one run: declared kind, both operand identities, normalization applied, per-difference classifications, and structural findings — reproducible from the report alone.
- **Difference Acknowledgment**: A reviewer's explicit acceptance of a substantive difference (or report), carrying reviewer identity, the preparer's identity it was checked against, and the exact evidence binding.
- **Skip Record / Status Entry**: The operator-visible artifact while unconfigured: a recorded QV-015 skip with reason in the validation report plus an explicit gap entry on the status surface.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A curator can register two named reference corpora, pin a run to either one, and a reviewer reading only the run record can identify which corpus and version was compared without asking the curator.
- **SC-002**: Re-running a comparison with the same pin after the reference corpus is updated still compares the originally pinned text (100% of pin-stability trials), never the updated text.
- **SC-003**: In trials with crafted edition/reference pairs, 100% of reported differences carry exactly one classification from the shared vocabulary, and unattributable differences appear as unknown_difference with evidence intact.
- **SC-004**: In every trial where the reference is incompatible in script, transmission, numbering, or basmala policy, the report contains zero corruption-family words (corrupt, tampering, invalid, wrong) about either text.
- **SC-005**: In trials with one substantive difference, the gate fails without acknowledgment in 100% of runs, accepts a distinct-reviewer's named acknowledgment in 100% of runs, and rejects missing-identity or self-acknowledgment in 100% of runs.
- **SC-006**: With no reference configured, 100% of runs record a skip stating the reason — never a silent pass, never a failure — and the status surface shows the configuration gap in 100% of checks; with a reference configured, a real comparison runs instead of the skip.
- **SC-007**: A reviewer given only a reference comparison report plus its acknowledgments can restate what was compared, what differed, who accepted each substantive difference, and why the gate outcome followed — without re-running anything.

## Assumptions

- The comparison kind vocabulary (integrity, version, readings, translation, reference) and difference classification vocabulary come from the sibling typed-comparison work (spec 016 / ADR-0114 taxonomy); this feature adds the reference-corpus configuration and acknowledgment procedure around them, not a new vocabulary.
- Normalization declarations come from the per-edition integrity manifests (spec 015): normalization_only classification is judged relative to declared normalization, never guessed.
- Transmission identity follows the project-wide rule: taken from what the source declares or a recorded inference with stated basis; script alone never implies transmission.
- The actual choice of reference text (which source, which release) remains a pending owner decision (OD-03 / P1-X03); this spec defines the registry, pinning, procedure, and acknowledgment shape so engineering is not blocked by that editorial decision.
- Reference and alternate corpora inherit per-source licence terms; no reference corpus is bundled until its licence is cleared (owner decision ODV-08).
- Reports and acknowledgments are review evidence only: they never modify canonical text, never activate an edition, and activation still requires the existing human approval gate.
