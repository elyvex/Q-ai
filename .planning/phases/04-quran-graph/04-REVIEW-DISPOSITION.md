---
phase: 04
review: 04-REVIEW.md
titles: json
findings:
  - id: WR-01
    severity: warning
    disposition: open
    title: "DOT/SVG export renders raw sets, bypassing the tombstone policy filter"
  - id: WR-02
    severity: warning
    disposition: open
    title: "Word-root and annotated projections are built but not readable on any surface"
  - id: WR-03
    severity: warning
    disposition: open
    title: "HTTP path handler silently discards caller-supplied edge_types/direction"
  - id: WR-04
    severity: warning
    disposition: open
    title: "Shallow tombstone probe truncation is unreported (bounded pass presented as full pass)"
  - id: WR-05
    severity: warning
    disposition: open
    title: "Tombstone-GC retention math may silently never match (julianday vs RFC 3339 input)"
  - id: WR-06
    severity: warning
    disposition: open
    title: "Quarantine repair accounting is wrong (`edges_after: 0` hardcoded)"
  - id: WR-07
    severity: warning
    disposition: open
    title: "`decided_at` is free-form and never validated as a timestamp"
  - id: WR-08
    severity: warning
    disposition: open
    title: "Review decision has a probe-then-act race (TOCTOU outside the transaction)"
  - id: WR-09
    severity: warning
    disposition: open
    title: "`review --edition` accepts arbitrary strings with no existence check"
  - id: WR-10
    severity: warning
    disposition: fixed
    title: "Annotated-seed parse checks dst membership but not src membership"
  - id: WR-11
    severity: warning
    disposition: fixed
    title: "Neighbor node-budget truncation keeps edges pointing at dropped nodes"
  - id: WR-12
    severity: warning
    disposition: fixed
    title: "Export human summary counts pre-filter sets while the payload is filtered"
  - id: IN-01
    severity: info
    disposition: open
    title: "`escape_dot` does not escape newlines/control characters"
  - id: IN-02
    severity: info
    disposition: open
    title: "`FileGraphBackend` opens a fresh database pool per request without closing"
  - id: IN-03
    severity: info
    disposition: open
    title: "Tool references degrade to `quran:@:s:a` with no active edition"
  - id: IN-04
    severity: info
    disposition: open
    title: "Server error bodies can carry filesystem paths (CLI-only paths retained)"
  - id: IN-05
    severity: info
    disposition: open
    title: "Direction parsing is case-sensitive, node-kind parsing is not"
  - id: IN-06
    severity: info
    disposition: open
    title: "Structural collection holds a write transaction over N+1 reads"
  - id: IN-07
    severity: info
    disposition: open
    title: "`live_rows_refused` counts global live rows, not scope-relevant ones"
  - id: IN-08
    severity: info
    disposition: open
    title: "File-backed export silently drops asserted edges"
  - id: IN-09
    severity: info
    disposition: open
    title: "Graph read paths materialize an empty database on a typo'd path (new)"
  - id: IN-10
    severity: info
    disposition: open
    title: "Caller-supplied assertion `--id` accepts the empty string (new)"
open: 19
total: 22
recorded: 2026-10-13T06:25:14.420Z
---

# Phase 04: Code Review Disposition

| Finding | Severity | Disposition | Source |
|---------|----------|-------------|--------|
| WR-01 | warning | open | - |
| WR-02 | warning | open | - |
| WR-03 | warning | open | - |
| WR-04 | warning | open | - |
| WR-05 | warning | open | - |
| WR-06 | warning | open | - |
| WR-07 | warning | open | - |
| WR-08 | warning | open | - |
| WR-09 | warning | open | - |
| WR-10 | warning | fixed | 04-REVIEW-FIX.md |
| WR-11 | warning | fixed | 04-REVIEW-FIX.md |
| WR-12 | warning | fixed | 04-REVIEW-FIX.md |
| IN-01 | info | open | - |
| IN-02 | info | open | - |
| IN-03 | info | open | - |
| IN-04 | info | open | - |
| IN-05 | info | open | - |
| IN-06 | info | open | - |
| IN-07 | info | open | - |
| IN-08 | info | open | - |
| IN-09 | info | open | - |
| IN-10 | info | open | - |

Dispositions: `open` (recorded, not yet triaged), `fixed`, `skipped`, `deferred`.
Set `deferred` by hand and put the reason in the Source cell; both are preserved. A `|` in the reason is kept as prose and escaped on the next run.
Re-running the gate keeps every row it can. A row the current review no longer reports is kept and its Source cell flagged, so a finding does not leave this record silently. ONE exception: when a finding id is REUSED by a different finding, the earlier decision cannot keep a row — the id is taken — and it is dropped. A RECORDED decision (anything but `open`) is named on the console when that happens; a row still at `open` is replaced silently, because `open` records no decision to lose.
