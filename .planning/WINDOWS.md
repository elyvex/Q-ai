---
schema_version: 1
open_count: 2
waived_count: 0
fixed_count: 0
total_count: 2
last_updated: 2026-09-28T03:58:39.016Z
---

# Broken Windows Ledger

> Cross-phase defect register. With `workflow.windows_enforce` enabled, `/gsd-ship` blocks while `open_count > 0`.
> Waive with `gsd-tools windows waive <id> "<reason>"` (reason required).
> Mark fixed with `gsd-tools windows fixed <id>`.

| id | phase | kind | file | line | description | status | reason | recorded_at | resolved_at |
|----|-------|------|------|------|-------------|--------|--------|-------------|-------------|
| 1 | 3 | deviation | crates/cli/tests/quran/family_s2.trycmd |  | CLI family snapshot pins typed unavailability (QAI-MORPH-0004, exit 5) + the usage guard; the attributed-relation branch is covered by 03-04 application family goldens and the new server contract test because no CLI path mints a morphology activation approval. | open |  | 2026-09-28T03:58:37.160Z |  |
| 2 | 3 | todo | docs/08-api/quran-v1-openapi.json |  | The three new lexicon routes (family, count/root-frequency, count/lemma-frequency) are served and contract-tested but missing from the published v1 OpenAPI document. | open |  | 2026-09-28T03:58:39.016Z |  |

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
  }
]
````
