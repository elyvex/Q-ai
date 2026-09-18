# Feature Specification: Typed Corpus Comparison

**Feature Branch**: `016-typed-corpus-comparison`

**Created**: 2026-09-18

**Status**: Draft

**Input**: User description: "typed corpus comparison: extend the edition differ with operand kinds (integrity, version, readings, translation, reference) and difference classification (same, normalization_only, orthographic, script, riwayah, edition, tokenization, translation, unknown_difference); cross-riwayah comparison must never be reported as corruption; QV-015 behavior unchanged with an explicit recorded skip while unconfigured."

**Constitution compliance**: `.specify/memory/constitution.md` v1.2.0, Principles I (canonical text never generated/corrected; QV validation gates), III (traceability, reproducibility), IV (disputed views side-by-side; differences between legitimate readings are readings comparisons, never corruption verdicts), VIII (comparison is typed — integrity, version, readings, translation, reference — every difference classified; one reading's checksum never certifies another; QV-015 reference-corpus comparison stays a recorded skip until its reference is configured).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Compare two texts with a declared comparison kind (Priority: P1)

A reviewer compares two corpus texts (two editions, two versions, or an edition against a reference) after declaring what kind of comparison this is: integrity (same edition identity, expecting sameness), version (same edition line, different release), readings (different transmissions), translation (translated texts), or reference (against an external reference text). The comparison report states the declared kind alongside every finding, so readers know what question was asked.

**Why this priority**: An untyped difference ("text A differs from text B") is uninterpretable: the same character delta means corruption in an integrity check and expected variation in a readings check. The declared kind is what makes the verdict honest.

**Independent Test**: Can be fully tested by running the same pair of texts under two different declared kinds and confirming the reports carry the different kind labels with kind-appropriate verdicts.

**Acceptance Scenarios**:

1. **Given** two texts and a declared kind of integrity with matching edition identity, **When** compared, **Then** the report is labeled integrity and any difference is reported as a mismatch against expectation.
2. **Given** the same two texts with a declared kind of readings, **When** compared, **Then** the report is labeled readings and differences are reported as readings variation, never as corruption.
3. **Given** a comparison request with no declared kind, **When** submitted, **Then** it is rejected with a diagnostic asking for the kind rather than guessed.

---

### User Story 2 - Every difference carries a classification (Priority: P1)

A reviewer examining a comparison report sees each difference labeled with exactly one classification: same, normalization_only, orthographic, script, riwayah, edition, tokenization, translation, or unknown_difference — so that trivial encoding effects are never confused with substantive textual divergence.

**Why this priority**: Without classification, a whitespace or Unicode-form artifact looks identical to a changed word. Classification is what lets reviewers dismiss noise and focus on substance.

**Independent Test**: Can be fully tested with crafted text pairs that isolate each class (e.g. only-whitespace change, only-diacritic change, only-transmission wording change) and confirming each report carries the expected label.

**Acceptance Scenarios**:

1. **Given** two texts differing only in declared-normalization effects (e.g. whitespace or Unicode form covered by the normalization declaration), **When** compared, **Then** the affected differences are classified normalization_only, not as substantive changes.
2. **Given** two texts with genuinely different wording from different transmissions, **When** compared under the readings kind, **Then** the differences are classified riwayah (or edition where the edition line differs), never normalization_only and never corruption.
3. **Given** a difference the comparison cannot attribute to any known class, **When** reported, **Then** it is classified unknown_difference with the evidence preserved, never silently forced into another class.

---

### User Story 3 - Cross-transmission comparison never reports corruption (Priority: P1)

A reviewer compares texts of different transmissions (e.g. Hafs wording against Warsh wording). The report presents the deltas as readings differences with riwayah classification and explicitly states that neither text is judged corrupt — even when the character-level delta is large.

**Why this priority**: This is the core safety invariant (Constitution Principles IV and VIII). A corruption verdict on a legitimate reading would be a false scholarly claim and a trust-destroying error.

**Independent Test**: Can be fully tested with two sample texts of different transmissions sharing most verses: the report must contain zero corruption/tampering language and must label the relationship as a readings comparison.

**Acceptance Scenarios**:

1. **Given** texts of two different transmissions, **When** compared, **Then** the verdict states a readings comparison and no finding uses corruption, tampering, or invalid language about either text.
2. **Given** identical passages occurring inside two different transmissions, **When** compared, **Then** those passages are classified same while the overall verdict remains a readings comparison (sameness of a passage never promotes one transmission's integrity claim onto the other).
3. **Given** a readings comparison report, **When** read by a non-specialist, **Then** the wording makes clear both texts may be legitimate and the report judges neither.

---

### User Story 4 - Reference comparison keeps its explicit recorded skip (Priority: P2)

A reviewer runs an import or comparison that includes the reference-corpus check (QV-015). While no reference corpus is configured, the check outcome is a recorded skip stating the reason (reference unavailable), visible in the report — identical in behavior to today, except the skip is now expressed through the typed comparison (kind: reference) rather than as an untyped exception.

**Why this priority**: QV-015 behavior must not change silently: a skip must stay visible as a skip, never become a hidden pass or a new failure, while the comparison work moves to the typed framework.

**Independent Test**: Can be fully tested by running the check with no reference configured and confirming the report shows a skip with reason, and by configuring a reference and confirming the skip disappears.

**Acceptance Scenarios**:

1. **Given** no reference corpus is configured, **When** the reference check runs, **Then** the outcome is a recorded skip naming the missing reference, and the overall run neither passes on its behalf nor fails because of it.
2. **Given** a reference corpus is configured, **When** the reference check runs, **Then** a real kind-reference comparison executes with classified differences instead of a skip.
3. **Given** historical reports from before this feature, **When** compared with new reports, **Then** the skip semantics are unchanged (skip still means "not evaluated", never "verified").

### Edge Cases

- What happens when the declared kind contradicts the operand identities (e.g. kind integrity declared for two different edition identities)? The comparison must refuse or re-label rather than produce a misleading integrity verdict.
- What happens when a translation is compared against canonical Arabic text? The kind system must reject the pairing (translation kind covers translation-to-translation; cross-kind pairing with canonical text is refused) so translations are never framed as variants of the original.
- What happens when tokenization differs but the underlying characters are identical (different word splits)? Differences must classify as tokenization, not as wording changes.
- What happens when script differs (e.g. Uthmani vs. simplified orthography) with the same transmission? Differences must classify as script or orthographic per the declared distinction, not as riwayah.
- What happens when both normalization effects and substantive changes occur in the same verse? Each difference span must carry its own classification; a verse is never reduced to a single label that hides either aspect.
- What happens when comparison inputs are structurally mismatched (different verse counts, reordered verses)? The report must flag the structural mismatch before classifying character-level differences, so alignment artifacts are not misclassified as wording changes.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Every comparison MUST declare exactly one operand kind from the closed set: integrity, version, readings, translation, reference; a comparison without a declared kind MUST be rejected, never defaulted.
- **FR-002**: Kind integrity MUST apply only to operands sharing one edition identity (same edition, version, script, transmission); kind version to operands sharing an edition line across releases; kind readings to operands of different transmissions; kind translation to translation operands only; kind reference to an operand checked against a configured external reference text.
- **FR-003**: A declared kind that contradicts the operand identities (e.g. integrity for two different edition identities, translation pairing a translation with canonical text) MUST be refused with a diagnostic, never executed as declared.
- **FR-004**: Every reported difference MUST carry exactly one classification from the closed set: same, normalization_only, orthographic, script, riwayah, edition, tokenization, translation, unknown_difference.
- **FR-005**: Differences attributable solely to the declared normalization MUST classify as normalization_only; differences in spelling conventions without transmission change MUST classify as orthographic; script-system differences MUST classify as script; transmission wording differences MUST classify as riwayah; edition-line wording differences MUST classify as edition; word-split-only differences MUST classify as tokenization; translation-text differences MUST classify as translation.
- **FR-006**: Any difference not attributable to a known class MUST classify as unknown_difference with its evidence preserved; the comparison MUST never force an unattributable difference into another class.
- **FR-007**: A cross-transmission (cross-riwayah) comparison MUST be verdict-labeled a readings comparison and MUST NOT use corruption, tampering, invalid, or wrong language about either operand in any finding or summary.
- **FR-008**: Passages identical across two transmissions MUST classify as same at the passage level while the overall verdict remains a readings comparison; passage sameness MUST NOT be presented as integrity certification of either operand by the other.
- **FR-009**: Structural mismatches (different verse counts, missing/extra verses, reordering) MUST be reported as structural findings before character-level classification, so alignment artifacts are not misclassified as wording changes.
- **FR-010**: The reference-corpus check (QV-015) MUST keep its current behavior: while no reference corpus is configured, the outcome MUST be an explicit recorded skip stating the reason; the skip MUST mean "not evaluated", never "verified" and never a failure. When a reference is configured, a real kind-reference comparison with classified differences MUST execute instead of the skip.
- **FR-011**: Each comparison report MUST state the declared kind, the operand identities, the normalization applied, and the per-difference classifications sufficient for an independent reviewer to reproduce the verdict.

### Key Entities

- **Comparison Kind**: The declared question being asked — integrity, version, readings, translation, or reference; determines which verdicts are legitimate for the run.
- **Difference Classification**: The per-difference label — same, normalization_only, orthographic, script, riwayah, edition, tokenization, translation, or unknown_difference; exactly one per reported difference.
- **Comparison Report**: The verdict artifact for one run: declared kind, operand identities, normalization, structural findings, classified differences, and overall verdict wording.
- **Reference Check Outcome**: The QV-015 result: either an explicit recorded skip (reference unconfigured, reason stated) or a classified kind-reference comparison (reference configured).
- **Structural Finding**: A report entry for count/order mismatches (missing, extra, or reordered verses) raised before character-level classification.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A reviewer can run the same pair of sample texts under each of the five kinds and receives five reports each correctly labeled with its kind and carrying kind-appropriate verdicts.
- **SC-002**: In trials with crafted pairs isolating each difference class, 100% of the differences carry the expected classification, and unattributable differences are labeled unknown_difference with evidence intact.
- **SC-003**: In every cross-transmission trial, the report contains zero occurrences of corruption-family language (corrupt, tampering, invalid, wrong) about either text, and the verdict is labeled a readings comparison.
- **SC-004**: With no reference corpus configured, the reference check reports an explicit recorded skip with reason in 100% of runs — never a silent pass and never a failure; with a reference configured, a classified comparison runs instead.
- **SC-005**: A reviewer given only a comparison report (kind, identities, normalization, classifications) can restate what was compared and what was found without needing to re-run the comparison.

## Assumptions

- The closed kind set (integrity, version, readings, translation, reference) and classification set (same, normalization_only, orthographic, script, riwayah, edition, tokenization, translation, unknown_difference) are fixed for this feature; adding new kinds or classes is out of scope.
- Normalization declarations come from the per-edition integrity manifests (sibling feature): normalization_only classification is judged relative to the declared normalization, never guessed.
- Transmission identity follows the same rule as elsewhere: taken from what the source declares or a recorded inference with stated basis; script alone never implies transmission.
- Reports are review evidence only: they never modify canonical text, never activate an edition, and never carry licensing authority.
