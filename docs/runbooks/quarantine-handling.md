# Runbook — Quarantine Handling

**When to use:** a source version has entered `Quarantined`, or a validation/signature check
failed and you must decide whether to review, correct, or remove it.

## Trigger
- A source transitions to `Quarantined` (illegal transition, license `Unknown`, hash mismatch,
  failed structural validation, or a broken signature).
- `qai doctor` reports `sources.license_unknown_count` or `sources.multiple_active_versions`.

## Why quarantine exists
PRD §22: no internet or LLM-recommended source may become active without explicit human review.
Quarantine is the safe holding state — retrieval must not see quarantined content.

## Steps
1. Inspect: `qai source show <source-id> --versions` and read `quarantine_reason`.
2. Establish the cause:
   - `Unknown` license → obtain licensing evidence before proceeding.
   - Hash mismatch → re-fetch/re-import; the declared vs observed hash differs.
   - Failed validation/signature → contact the publisher or re-download the manifest.
3. Review with evidence (PRD §10.6): reviewers must see the evidence before approving.
4. Re-approval path: correct the input, re-run validation, then transition
   `Quarantined → Staged → Approved` (requires hash + known license + validation report +
   a human `ApprovalRecord`).
5. If irreparable, transition to `Removed` and record an audit event.

## Verify
- `qai doctor` shows `sources.multiple_active_versions` = 0 and no unknown licenses for active
  versions.
