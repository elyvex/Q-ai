---
schema_version: 1
open_count: 5
waived_count: 0
fixed_count: 0
total_count: 5
last_updated: 2026-09-28T05:34:23.948Z
---

# Broken Windows Ledger

> Cross-phase defect register. With `workflow.windows_enforce` enabled, `/gsd-ship` blocks while `open_count > 0`.
> Waive with `gsd-tools windows waive <id> "<reason>"` (reason required).
> Mark fixed with `gsd-tools windows fixed <id>`.

| id | phase | kind | file | line | description | status | reason | recorded_at | resolved_at |
|----|-------|------|------|------|-------------|--------|--------|-------------|-------------|
| 1 | 3 | deviation | crates/cli/tests/quran/family_s2.trycmd |  | CLI family snapshot pins typed unavailability (QAI-MORPH-0004, exit 5) + the usage guard; the attributed-relation branch is covered by 03-04 application family goldens and the new server contract test because no CLI path mints a morphology activation approval. | open |  | 2026-09-28T03:58:37.160Z |  |
| 2 | 3 | todo | docs/08-api/quran-v1-openapi.json |  | The three new lexicon routes (family, count/root-frequency, count/lemma-frequency) are served and contract-tested but missing from the published v1 OpenAPI document. | open |  | 2026-09-28T03:58:39.016Z |  |
| 3 | 03 | deviation | crates/application/tests/morphology_import.rs |  | Task 3 is tdd=true but its subject (the license gate) is delivered by Task 1 in the same plan; no distinct RED commit was possible — the license-gate integration tests were verified against the implemented gate (QAI-MORPH-0006 pin) and the full suite. | open |  | 2026-09-28T05:34:22.186Z |  |
| 4 | 03 | todo | fixtures/quran/morphology/license-matrix.json |  | qac matrix entry stays pending_license_review / redistribution_allowed:false until the owner captures licenses/qac/ (source_url+capture_date+capturer) and ratifies OD-11 (ADR-0203 Option A); until then QAC ships via user-supplied import only. | open |  | 2026-09-28T05:34:23.087Z |  |
| 5 | 03 | deviation | crates/cli/src/lib.rs |  | Three new Option<String> license flags on MorphologyAction::Import tripped clippy::large_enum_variant on the top-level Commands enum; resolved with a targeted #[allow] (CLI subcommand shape), outside the plan's files_modified. | open |  | 2026-09-28T05:34:23.948Z |  |

````json
[
  {
    "id": 1,
    "kind": "deviation",
    "phase": "3",
    "file": "crates/cli/tests/quran/family_s2.trycmd",
    "line": null,
    "description": "CLI family snapshot pins typed unavailability (QAI-MORPH-0004, exit 5) + the usage guard; the attributed-relation branch is covered by 03-04 application family goldens and the new server contract test because no CLI path mints a morphology activation approval.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-28T03:58:37.160Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 2,
    "kind": "todo",
    "phase": "3",
    "file": "docs/08-api/quran-v1-openapi.json",
    "line": null,
    "description": "The three new lexicon routes (family, count/root-frequency, count/lemma-frequency) are served and contract-tested but missing from the published v1 OpenAPI document.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-28T03:58:39.016Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 3,
    "kind": "deviation",
    "phase": "03",
    "file": "crates/application/tests/morphology_import.rs",
    "line": null,
    "description": "Task 3 is tdd=true but its subject (the license gate) is delivered by Task 1 in the same plan; no distinct RED commit was possible — the license-gate integration tests were verified against the implemented gate (QAI-MORPH-0006 pin) and the full suite.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-28T05:34:22.186Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 4,
    "kind": "todo",
    "phase": "03",
    "file": "fixtures/quran/morphology/license-matrix.json",
    "line": null,
    "description": "qac matrix entry stays pending_license_review / redistribution_allowed:false until the owner captures licenses/qac/ (source_url+capture_date+capturer) and ratifies OD-11 (ADR-0203 Option A); until then QAC ships via user-supplied import only.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-28T05:34:23.087Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 5,
    "kind": "deviation",
    "phase": "03",
    "file": "crates/cli/src/lib.rs",
    "line": null,
    "description": "Three new Option<String> license flags on MorphologyAction::Import tripped clippy::large_enum_variant on the top-level Commands enum; resolved with a targeted #[allow] (CLI subcommand shape), outside the plan's files_modified.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-28T05:34:23.948Z",
    "resolved_at": null,
    "milestone": null
  }
]
````
