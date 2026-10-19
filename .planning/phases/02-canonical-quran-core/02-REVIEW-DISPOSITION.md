---
phase: 02
review: 02-REVIEW.md
titles: json
findings:
  - id: WR-01
    severity: warning
    disposition: fixed
    title: "`LEDGER` omits paths the doc claims are mechanically guarded"
  - id: WR-02
    severity: warning
    disposition: fixed
    title: "Emission-marker scan is API-only; the build-failure claim is false elsewhere"
  - id: IN-01
    severity: info
    disposition: open
    title: "`tool_search_path_is_present` is effectively vacuous"
  - id: IN-02
    severity: info
    disposition: open
    title: "`scan_marker` attribution heuristic is coupled to a brittle split"
  - id: IN-03
    severity: info
    disposition: open
    title: "`Citation` carries edition identity twice with no cross-check"
  - id: IN-04
    severity: info
    disposition: open
    title: "`ApprovalGate::abort` is unconditional and session ids collide by construction"
  - id: IN-05
    severity: info
    disposition: open
    title: "Read-only paths hold a write transaction for full scans"
  - id: WR-03
    severity: warning
    disposition: fixed
    title: "tool `quran.search` default path mis-attributes the envelope and drops the stale-index advisory"
  - id: WR-04
    severity: warning
    disposition: fixed
    title: "Corrupt validation report degrades to misleading `Skipped`"
  - id: WR-05
    severity: warning
    disposition: fixed
    title: "Unreachable `MatchAfterDeclaredNormalization` defaults to success"
  - id: WR-06
    severity: warning
    disposition: fixed
    title: "Translation import relaxes `attribution_required` to `false`, including for `Unknown` licenses"
  - id: WR-07
    severity: warning
    disposition: fixed
    title: "Out-of-range surah/ayah mapped to internal error instead of caller error"
  - id: WR-08
    severity: warning
    disposition: fixed
    title: "Read-only operator paths open write transactions and swallow storage errors"
  - id: WR-09
    severity: warning
    disposition: fixed
    title: "`rollback_edition` can reactivate a `Quarantined` edition"
open: 5
total: 14
unparsed: 1
recorded: 2026-10-13T05:02:18.523Z
---

# Phase 02: Code Review Disposition

| Finding | Severity | Disposition | Source |
|---------|----------|-------------|--------|
| WR-01 | warning | fixed | 02-REVIEW-FIX.md |
| WR-02 | warning | fixed | 02-REVIEW-FIX.md |
| IN-01 | info | open | - |
| IN-02 | info | open | - |
| IN-03 | info | open | - |
| IN-04 | info | open | - |
| IN-05 | info | open | - |
| WR-03 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |
| WR-04 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |
| WR-05 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |
| WR-06 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |
| WR-07 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |
| WR-08 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |
| WR-09 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |

Dispositions: `open` (recorded, not yet triaged), `fixed`, `skipped`, `deferred`.
Set `deferred` by hand and put the reason in the Source cell; both are preserved. A `|` in the reason is kept as prose and escaped on the next run.
Re-running the gate keeps every row it can. A row the current review no longer reports is kept and its Source cell flagged, so a finding does not leave this record silently. ONE exception: when a finding id is REUSED by a different finding, the earlier decision cannot keep a row — the id is taken — and it is dropped. A RECORDED decision (anything but `open`) is named on the console when that happens; a row still at `open` is replaced silently, because `open` records no decision to lose.
