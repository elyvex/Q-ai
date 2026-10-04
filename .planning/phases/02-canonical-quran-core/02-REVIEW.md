---
phase: 02-canonical-quran-core
reviewed: 2026-10-04T15:18:10Z
depth: standard
files_reviewed: 2
files_reviewed_list:
  - crates/application/src/quran_tools.rs
  - crates/application/tests/answer_path_ledger.rs
findings:
  critical: 0
  warning: 3
  info: 3
  total: 6
status: issues_found
---

# Phase 02: Code Review Report (incremental — G-02-3 delta)

**Reviewed:** 2026-10-04T15:18:10Z
**Depth:** standard
**Files Reviewed:** 2
**Status:** issues_found

## Summary

This is an incremental review of the plan 02-08 gap-closure delta for phase 02 (UAT gap G-02-3). The
delta is: (1) a corrected D-15 exemption-ledger doc comment on `ReaderCitationSource`
(`quran_tools.rs:456-488`), and (2) a new source-scan guard,
`crates/application/tests/answer_path_ledger.rs` (6 tests).

The doc correction is accurate: I cross-checked every handler it names against the actual emission
markers in `crates/server/src/api.rs` (`.arabic_text()` ×3, `search_response(&headers,` ×5,
`ok_envelope(result.results` ×2) and against `docs/06-progress/phase-02-evidence.md` §3 — the
mechanism taxonomy (enforced / `AyahView.canonical` / `SearchHit.quotation` / hand-built token
surface / no-text / debug-only) and the path lists agree. `cargo test -p application --test
answer_path_ledger` passes 6/6 and the hard-coded occurrence counts match the tree.

The new guard works as intended for the API file, but it is weaker than its own documentation claims.
The most material gaps are: the `LEDGER` omits two paths the doc/evidence explicitly name as part of
the "complete sweep" (including the *enforced* HTTP path `citation_handler`), and the marker scan is
restricted to `api.rs` even though the module doc promises any new emitting handler fails the build.

I also found one genuine correctness bug in the file under review (`search_meta` None-branch
mis-attribution). It is **pre-existing phase-03 code**, not part of the 02-08 delta; it is recorded
here for cross-reference and should not be attributed to the gap-closure work.

## Warnings

### WR-01: `LEDGER` omits paths the doc/evidence claim are part of the guarded "complete sweep"

**File:** `crates/application/tests/answer_path_ledger.rs:37-58`
**Issue:** The `LEDGER` is presented as the machine-readable enumeration of the D-15 answer-path
sweep, and `quran_tools.rs:486-488` claims "The complete sweep is enumerated in
`docs/06-progress/phase-02-evidence.md` §3 and guarded mechanically by
`crates/application/tests/answer_path_ledger.rs`." But `every_ledger_symbol_resolves_under_its_declared_kind`
only iterates `LEDGER`, and `LEDGER` omits two paths that both the in-code doc (`quran_tools.rs:465-467`,
`:483-484`) and evidence §3 (`docs/06-progress/phase-02-evidence.md:199`, `:244`) explicitly name:

- `citation_handler` (`crates/server/src/api.rs:596`) — the **enforced** HTTP stored-verdict path
  (re-serves a persisted verdict; hard-fails via `require_exact`). This is the most safety-relevant
  path in the sweep and, being enforced like `cmd_verify_quotation` (which *is* in the ledger), it is
  inconsistent that it is not recorded.
- `resolve_handler` (`crates/server/src/api.rs:572`) — listed in evidence §3 §"Exempt, no text
  emitted (recorded for completeness)".

Because neither is in `LEDGER`, a rename/removal of either will not fail the build, so the ledger's
completeness claim is not actually enforced for the sweep it claims to guard.
**Fix:**
```rust
    (API, "citation_handler", "async fn"),
    (API, "resolve_handler", "async fn"),
```
(and, since `citation_handler` is enforced rather than exempt, either add it to `LEDGER` or narrow
the doc/evidence "complete sweep…guarded mechanically" wording.)

### WR-02: Emission-marker scan is API-only, so the doc's "new emitting handler fails the build" is false elsewhere

**File:** `crates/application/tests/answer_path_ledger.rs:112-127`
**Issue:** `emitting_markers_resolve_to_recorded_paths` scans only `source_for(API)` and checks
membership only against `ledger_symbols(API)`. The three markers are not API-exclusive: `.arabic_text()`
occurs 4× in `crates/application/src/quran_cli.rs` and 1× in `crates/application/src/quran_tools.rs`
(`fetch_ayah_text`, line 557). A newly added emitting handler in `quran_cli.rs` or `quran_tools.rs`
that emits canonical text via `.arabic_text()` (or the other markers) will **not** fail the build,
directly contradicting the module doc (`:6-8`, "...a *new* emitting handler that uses one of the three
known emission markers also fails the build"). The documented ceiling at `:14-23` covers a *new
emission shape* matching none of the markers; it does not disclose this per-file blind spot.
**Fix:** scan all three ledger files per marker with per-file expected counts, or explicitly narrow
the doc claim to "HTTP emitting handlers in `api.rs`". E.g. iterate
`[(API, 3), (CLI, 4), (TOOLS, 1)]` for `.arabic_text()` and assert each hit name is in the recorded
set for that file (adding `fetch_ayah_text` to `LEDGER` or excluding it by design).

### WR-03: `search_meta(None)` names the active edition while `quran.search` serves the indexed edition

**File:** `crates/application/src/quran_tools.rs:129-141` (reached from `backend_search` at `:228`)
**Issue:** *(pre-existing phase-03 code, outside the 02-08 delta — recorded for cross-reference.)*
`search_meta` with `edition = None` delegates to `active_meta` (the **active** edition), but
`search_normalized` never treats the active pointer as the served edition: `open_serving`
(`crates/application/src/quran_search.rs:223-314`) serves `manifest.edition_id` (the **indexed**
edition) whenever no edition is requested, and explicitly models the divergent state by emitting
`Warning::stale_index` when the indexed edition differs from the active one (`:293-299`). On a stale
index, `backend_search` therefore returns a `ToolResult` carrying two edition identities at once:

- envelope `edition_id`/`edition_version` and `reproducibility` = the **active** edition (via
  `search_meta(None)`);
- `canonical_references` = the **indexed** edition (from `SearchHit.reference()`, built from
  `parts.edition`, `crates/quran-search/src/hit.rs:189-196`).

The `search_meta` doc (`:126-128`) even states the envelope "must name the same edition the hits come
from, never the active pointer" — which the `None` branch contradicts. This is the same WR-02 class
of bug (envelope attributed to the wrong edition) for the default path.
**Fix:** resolve the `None` case from the index manifest / served edition (e.g. have
`search_normalized` return the served edition in `SearchOutput`), not from the active pointer; or
fail closed when `edition` is absent and the indexed edition ≠ active.

## Info

### IN-01: `tool_search_path_is_present` is effectively vacuous

**File:** `crates/application/tests/answer_path_ledger.rs:135-138`
**Issue:** The test asserts `source_for(TOOLS).contains("\"quran.search\"")`. That literal also
appears in the error message at `quran_tools.rs:116` and in the unit test at `:689`, so deleting or
renaming `backend_search` — the actual emission path this test was added to disposition — would still
pass. It does not verify `backend_search`'s definition. Given G-02-3's root cause was "tool
`quran.search` was never dispositioned", a guard that cannot detect its removal is false assurance.
**Fix:** assert `source_for(TOOLS).contains("async fn backend_search")`, or add
`(TOOLS, "backend_search", "async fn")` to `LEDGER` to match the evidence §3 anchor
(`quran_tools.rs#ReaderToolBackend (backend_search arm)`).

### IN-02: `scan_marker` attribution heuristic is coupled to a brittle split

**File:** `crates/application/tests/answer_path_ledger.rs:81-95`
**Issue:** Splitting on `"async fn "` attributes every marker to the nearest preceding `async fn`.
Markers in non-`async` helpers, in comments/doc-comments, in `const`s, or before the first
`async fn` are mis-attributed or silently dropped; `async fn` in trait declarations also starts a
chunk. The hard-coded expected counts (`[3,5,2]`) are coupled to this heuristic, so a refactor that
moves a marker into a helper changes the attributed name without changing behavior.
**Fix:** anchor attribution on the enclosing `fn`/brace scope (or a symbol table) rather than a flat
`split`, or document the heuristic's assumptions alongside the ceiling.

### IN-03: Review scope vs `diff_base` discrepancy

**File:** (config) `diff_base: d3a0a99..HEAD`
**Issue:** The scope note states "the only source changes since `d3a0a99` are ... doc correction ...
and ... a NEW source-scan guard test". That is not accurate for the reviewed files:
`crates/application/src/quran_tools.rs` is 298 lines at `d3a0a99` and 691 at HEAD (+393), including
`backend_search`/`backend_root`/`backend_lemma`/`backend_morphology`/`backend_family` and
`search_meta`. The 02-08 gap-closure commit itself only touches 42 lines of that file, but the
declared base predates phase 03. Downstream fixers should not treat WR-03 as a 02-08 regression.
**Fix:** none required in code; align the review's declared base with the actual gap-closure parent
(or state the phase-03 carry-over explicitly) so the delta is unambiguous.

---

_Reviewed: 2026-10-04T15:18:10Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
