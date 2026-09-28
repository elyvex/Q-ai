---
phase: 01-foundations
reviewed: 2026-09-28T12:00:00Z
depth: standard
files_reviewed: 27
files_reviewed_list:
  - scripts/verify-phase1-preservation.sh
  - scripts/verify-phase1-records.sh
  - crates/application/src/audit_bridge.rs
  - crates/application/src/db.rs
  - crates/application/src/job_queue.rs
  - crates/application/src/quran.rs
  - crates/application/src/quran_cli.rs
  - crates/cli/src/doctor.rs
  - crates/cli/src/lib.rs
  - crates/cli/src/quran.rs
  - crates/config/src/lib.rs
  - crates/audit/src/lib.rs
  - crates/jobs/src/lib.rs
  - crates/jobs/src/queue.rs
  - crates/jobs/src/registry.rs
  - crates/jobs/src/worker.rs
  - crates/storage/src/repository.rs
  - crates/storage/src/workflows.rs
  - crates/storage-sqlite/src/lib.rs
  - crates/cli/tests/foundation.rs
  - crates/cli/tests/jobs.rs
  - crates/cli/tests/quran.rs
  - crates/application/tests/phase1_foundation.rs
  - crates/application/tests/phase1_jobs_host.rs
  - crates/testkit/tests/config_precedence.rs
  - xtask/src/arch.rs
  - xtask/allowlist.toml
findings:
  critical: 0
  warning: 13
  info: 9
  total: 22
status: issues
---

# Phase 01 (Foundations): Code Review Report

**Reviewed:** 2026-09-28T12:00:00Z
**Depth:** standard
**Files Reviewed:** 27 (all `feat`/`test`/`fix` commits with scope `01-01`…`01-05` on main, plus key-files from `01-01-SUMMARY.md`…`01-05-SUMMARY.md`)
**Status:** issues

## Summary

Reviewed the full Phase 01 diff set (01-01 dispatch/config/doctor guards, 01-02 `AuditedMutation` seam,
01-03 job checkpoint/retry/cancel/audit, 01-04 serve-owned host + enqueue-only import, 01-05
`xtask` external policy + record checkers) against current file state. Foreign concurrent work
(`03-03`/`03-05`/`03-06` hunks touching `quran_cli.rs`, `cli/lib.rs`, morphology/lexicon) was
excluded by per-commit attribution; only `01-0x` hunks are cited.

No Critical findings: pattern sweep is clean (no secrets, no injection sinks, no `eval`, no
`TODO`/`FIXME`, redaction applied on every new emission path, no production `unwrap`/`expect`
except WR-02's signal handler). The 13 Warnings below are all provable from the cited lines and
cluster in five areas: (1) worker-host liveness (silent host death, deadline-free shutdown,
no re-reap, owner-unchecked reschedule), (2) audit-verifier diagnosis semantics (gap reporting,
tamper cascade), (3) filesystem-guard accuracy (symlink handling, error conflation, readonly-bit
heuristic), (4) backend divergence (in-memory drops results/progress), (5) verifier-script
soundness (substring tokens, merge-file retention proof). Nine Info items record design warts
and explicitly-deferred scope.

## Warnings

### WR-01: Transient storage error kills the worker host silently while serve stays up

**File:** `crates/jobs/src/worker.rs:188,195`
**Issue:** `run_until_shutdown` propagates both `claim_next(...).await?` (line 188) and
`process_claimed(...).await?` (line 195) with `?`, returning `Err` and terminating the host on
the first transient failure (e.g. SQLite busy/lock, audit-stage conflict). The serve composition
(`crates/cli/src/lib.rs`, `d5183ff` select) only `await`s the host handle *after* the server
future or shutdown signal resolves, so a dead host goes unnoticed: the API keeps serving while
no job ever runs again until the process is restarted. A single `busy_timeout` expiry therefore
becomes a silent total stall of durable execution.
**Fix:**
```rust
let Some(job) = match self.queue.claim_next(&self.owner, self.config.lease).await {
    Ok(job) => job,
    Err(e) => {
        warn!(error = %e, "transient claim failure; backing off");
        tokio::select! {
            _ = shutdown.changed() => break,
            _ = tokio::time::sleep(self.config.poll_interval * 10) => continue,
        }
    }
};
```
And in serve, `select!` over the host `JoinHandle` so host death is surfaced immediately
instead of at teardown.

### WR-02: Shutdown join has no deadline; a non-cooperative handler blocks SIGINT/SIGTERM forever

**File:** `crates/jobs/src/worker.rs:267-282`, `crates/cli/src/lib.rs` (serve `select!` arms, `d5183ff`)
**Issue:** On shutdown the worker does `handler_future.await` with no timeout, and serve
`host.await`s with no timeout. A handler that never observes cancellation (or a 30s-sleep style
long stage) hangs `qai serve` shutdown indefinitely; a second Ctrl-C cannot force exit because
`shutdown_signal()` was already consumed. Service managers will SIGKILL after their grace
period, turning a "joined shutdown" into a kill mid-handler. Related: `shutdown_signal`
(`crates/cli/src/lib.rs:914`) uses `.expect("SIGTERM handler")`, panicking instead of
returning an exit code when signal installation fails (restricted sandbox).
**Fix:**
```rust
match tokio::time::timeout(SHUTDOWN_GRACE, handler_future).await {
    Ok(r) => r,
    Err(_) => { warn!("handler exceeded shutdown grace; leaving for lease recovery"); /* return, let reap mark Interrupted */ }
}
```
Replace `.expect(...)` with a fallback that logs and proceeds with Ctrl-C only (or exits 70).

### WR-03: Failure-path `reschedule` bypasses the owner/state guard a stale worker can exploit

**File:** `crates/jobs/src/worker.rs:386`, `crates/storage-sqlite/src/lib.rs:942-967`
**Issue:** Every terminal path uses owner-checked `finish_owned`, but `handle_failure` calls the
plain `reschedule`, whose SQLite implementation is `UPDATE jobs SET state='Queued', ...
WHERE id=?` — no owner predicate, no state predicate. A worker whose lease expired mid-run
(watchdog starved past lease expiry) can resurrect or clobber a job now owned by someone else,
clearing its lease and wiping `error_json` (overwritten with the `"retry"` reason, destroying
the failure detail/disposition). The T-03-LEASE owner-safety the plan claims is therefore not
uniform: the single most common terminal path (retry) is exempt from it.
**Fix:** Add `reschedule_owned(job_id, owner, delay, reason)` with
`WHERE id=? AND lease_owner=? AND state='Running'`, and fall back to warn-and-return
(`Interrupted` recovery owns the job now) when it affects zero rows.

### WR-04: Dead-letter paths discard the `finish_owned` outcome, reporting transitions never persisted

**File:** `crates/jobs/src/worker.rs:213,228,329,368`
**Issue:** Unknown-kind (213), invalid-payload (228), non-idempotent (329), and max-attempts
(368) paths use `let _ = ...finish_owned(...).await?;` — the `?` propagates transport errors
but the returned `bool` (lease still held?) is silently dropped. After lease loss the worker
still returns `UnknownKind`/`DeadLettered` although the row is unchanged (still `Running`, to
be reaped later). `finish_succeeded`/`finish_cancelled` warn on `!applied`; these four paths do
not, so operators get confidently-wrong outcomes.
**Fix:** Mirror the success path:
```rust
let applied = self.queue.finish_owned(...).await?;
if !applied { warn!(job_id, owner = %self.owner, "lease lost before dead-letter finish"); }
```

### WR-05: Gap report names the post-hole survivor, and the gap cascades into false tamper flags

**File:** `crates/application/src/audit_bridge.rs:431-442`
**Issue:** Two defects in `verify_persisted_audit`: (a) on a sequence break it pushes
`event.sequence` — the *present* row after the hole, not the missing sequences. Deleting rows
3–4 from `[1..5]` reports `gaps:[5]` while the remedy says "restore the missing sequences",
sending the operator to restore an intact row. (b) `previous` is not reset across a gap, so the
first survivor's `prev_chain_hash` (pointing at the deleted row) mismatches and it is *also*
flagged in `tampered_sequences` — a pure deletion is misdiagnosed as tampering, implying
modification that never happened.
**Fix:**
```rust
if Some(event.sequence) != expected_sequence {
    let from = expected_sequence.unwrap_or(event.sequence);
    report.gaps.extend(from..event.sequence); // name the missing rows
    previous = event.prev_chain_hash.clone(); // don't cascade the hole into a tamper flag
    // (then run the tamper check against the resynced link)
}
```

### WR-06: `symlink_metadata` + `is_dir()` false-FAILs symlinked data dirs and reads link permissions

**File:** `crates/cli/src/doctor.rs:244-275` (`data_dir_writable`), `:615-648` (`filesystem_object_store_writable`), via `32f8b56`
**Issue:** `symlink_metadata` does not follow the final component, and `Metadata::is_dir()` is
false for symlinks — a data dir (or object root) that *is* a symlink to a healthy directory now
reports FAIL "is not a directory". Conversely, when the path is a symlink the `readonly()` bits
inspected are the link's (conventionally `lrwxrwxrwx` → never readonly), so a read-only target
passes. The check is wrong in both directions for exactly the paths the commit intended to
handle more carefully.
**Fix:** Follow the link for type/permission inspection (`std::fs::metadata`), and separately
assert the configured path itself if link-ness matters:
```rust
let target_meta = std::fs::metadata(&dir)?; // follows links: real type + real perms
let is_link = std::fs::symlink_metadata(&dir)?.file_type().is_symlink();
```

### WR-07: Any filesystem error is misreported as "missing database — run migrate"

**File:** `crates/application/src/quran_cli.rs:81-83`, `crates/application/src/job_queue.rs:324,397`, `crates/application/src/db.rs:280`
**Issue:** All four existence guards use `symlink_metadata(...).is_err()` → `Missing` /
`MigrationRequired` / `DatabaseMissing`. A file that exists but is unreadable (permission
denied), or an I/O error on the parent, yields "run `qai db migrate`" — a remedy that cannot
help and can confuse (migrate will fail the same way). `DatabaseReadiness::Unreadable` with
the correct remedy exists but `open_db` can never produce it: it conflates before it
discriminates, and synthesizes `MigrationRequired { at_schema: 0, required: 0 }`, discarding
real version info.
**Fix:**
```rust
match std::fs::symlink_metadata(db_path) {
    Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Err(StorageError::MigrationRequired { ... }),
    Err(e) => return Err(StorageError::Io { ... }), // maps to INTERNAL + permission remedy
    Ok(_) => {}
}
```

### WR-08: `qai audit verify` emits three different JSON shapes for three outcomes

**File:** `crates/cli/src/lib.rs:745-810` (`render_audit_verify`, via `193e8a7`)
**Issue:** Valid → `{valid, checked_events, gaps, tampered_sequences}`; invalid →
`{valid, code, summary, remedy, next_command, gaps, tampered_sequences}` (no `checked_events`);
infrastructure error → `{valid, error, remedy}` as a raw string literal (no `code`, no
sequences). Machine consumers must branch on shape instead of fields, contradicting the
"same fields in human and JSON" contract the commit claims. The error path also bypasses
`serde_json::json!`, so a future remedy string containing `"` would emit invalid JSON.
**Fix:** One envelope for all outcomes:
```rust
serde_json::json!({ "valid": report.valid, "code": code_or_null, "checked_events": ...,
  "gaps": ..., "tampered_sequences": ..., "remedy": ..., "next_command": ... })
```
built with `json!`, never a format string.

### WR-09: In-memory queue silently drops terminal results and progress (backend divergence)

**File:** `crates/jobs/src/queue.rs:250-262` (`finish`), `:364-382` (`finish_owned`), `:234-248`/`340-362` (`checkpoint`/`checkpoint_owned`)
**Issue:** `finish`/`finish_owned` take `_result` and never store it; `checkpoint` variants
accept `progress` and never store it (`JobRecord` in `crates/storage/src/repository.rs:404-419`
has neither a result nor a progress field). SQLite persists dispositions/results into
`error_json` and progress into `progress_json`, so the same worker code produces observable
dispositions via `qai job show` on SQLite but loses them on the in-memory backend — including
every `CancellationDisposition` the 01-03 contract promises to persist. Contract tests running
against `InMemoryJobQueue` cannot observe what production persists.
**Fix:** Add `result_json: Option<String>` / `progress_json: Option<String>` to `JobRecord`
and store them in all four methods; or mark the in-memory backend explicitly lossy in docs and
stop asserting cross-backend equivalence for disposition content.

### WR-10: `record_approval` fail-open actor attribution on unparseable principal

**File:** `crates/application/src/quran.rs:673-677` (via `8d05b06`)
**Issue:** `decided_by.parse::<PrincipalId>().unwrap_or_else(|_| Actor::System { name:
decided_by.to_string() })` — any non-UUID `decided_by` is silently recorded as a `System`
actor carrying an arbitrary, caller-controlled name. All current callers pass
`LOCAL_PRINCIPAL` so the fallback is dead today, but the audit chain (the anti-fabrication
control per AGENTS.md) accepts spoofed attribution by construction for any future caller.
Fail-open is the wrong direction for audit integrity.
**Fix:** Return `ActivationError::storage`/a typed error on parse failure instead of
synthesizing a `System` actor.

### WR-11: Writability inferred from the owner-write bit, which does not reflect effective access

**File:** `crates/cli/src/doctor.rs:244-275,615-648`
**Issue:** `meta.permissions().readonly()` reports the owner write bit only. Running as root
(a readonly dir is still writable → false "read-only" warn), group/ACL-granted write (owner
bit clear → false warn), or a read-only mount with the bit set (→ false pass) all misreport.
Combined with WR-06, neither filesystem check measures what it claims ("writable").
**Fix:** Attempt effective access the read-only way (open the dir fd with `O_RDONLY`, or
`access(W_OK)` via `nix`/`rustix` if available) — or downgrade these checks from pass/warn to
informational presence reporting so doctor stops asserting what it cannot know.

### WR-12: Closure-checker tokens match as substrings — criteria can pass vacuously

**File:** `scripts/verify-phase1-records.sh:33,61-65`
**Issue:** `require_tokens` uses `grep -F -q -- "$tok"` (substring). `CRITERIA="C1 C2 C3 C4 C5"`
means a record mentioning only `C10`…`C19` satisfies `C1`; `TASK-001` matches any
`TASK-001x` suffix. The checker's purpose is record reliability; substring matching lets the
acceptance-criteria gate pass without the criteria being present.
**Fix:** Match whole words/lines, e.g. `grep -F -q -w -- "$tok"` (or anchor the C-tokens as
`"C1 "`/`"(C1)"` forms actually used in the records).

### WR-13: MERGE preservation proof cannot detect deletion of untouched pre-existing hunks

**File:** `scripts/verify-phase1-preservation.sh:188-205` (`check_merge`)
**Issue:** The check runs `git merge-file ours base theirs` and accepts exit 0 as proof of
retention. But when the captured state did not touch hunk H (`ours_H == base_H`) and the
current tree deleted it (`theirs_H = ∅`), the merge cleanly takes the deletion with exit 0 —
the check passes although pre-existing bytes were destroyed. The script's core claim ("proves
each discovered path is retained through a conflict-free three-way merge") is therefore
unsound for the exact interference pattern this phase experienced (concurrent session
removing files/hunks).
**Fix:** After a clean merge, additionally assert no base hunk was deleted: e.g.
`git diff base theirs` must contain no deletions outside task-owned regions, or
`comm`-compare base line-set ⊆ merged line-set for EXCLUDE-classified content.

## Info

### IN-01: Read-only verifier acquires a write-flavored unit of work

**File:** `crates/application/src/audit_bridge.rs:417-446`
**Issue:** `verify_persisted_audit` opens the DB with `open_read_only` (whose `write_pool`
aliases the read pool, `crates/storage-sqlite/src/lib.rs:122`) and then calls `db.write()`.
Reads work, so this is functional, but a read-only diagnostic holding a "write" transaction is
misleading and couples the verifier to the pooling alias — any future write through that UoW
path would fail opaquely against a read-only connection.
**Fix:** Add an explicit read-snapshot accessor (`db.read()`) for diagnostics, or document why
the write-flavored UoW is required here.

### IN-02: Success results and dispositions live in the `error_json` column

**File:** `crates/storage-sqlite/src/lib.rs:1106-1135` (`finish_owned`), `crates/application/src/db.rs:409-442` (`get_job`)
**Issue:** The schema (`migrations/sqlite/0004_jobs.up.sql`) has `error_json` but no result
column, so handler success payloads and `CompletedBeforeObservation` dispositions are stored
under `error_json` and surfaced to operators under an `"error_json"` key. Pre-existing schema
constraint, but new 01-03 code leans on the channel (disposition round-trip, audit parsing).
**Fix:** Long-term: add a `result_json` column; short-term: alias the envelope key by state
(`result` when `Succeeded`, `error` otherwise).

### IN-03: `ChainVerificationFailed { sequence: 0 }` sentinel leaks into operator output

**File:** `crates/application/src/audit_bridge.rs:407`
**Issue:** When a report is invalid with empty gaps/tampered lists, the diagnosis fabricates
sequence `0` — a sequence that can never exist — and its rendering names it. Minor confusion
for the exact case that means "something is wrong but unattributed".
**Fix:** Use `Option<u64>` (None) for the sequence in that variant.

### IN-04: Audit-composition enforcement is documentary, not structural

**File:** `crates/storage/src/workflows.rs:10-24` (via `8d05b06`)
**Issue:** The D-10 "MUST compose through `AuditedMutation`" rule plus the summary claim that
"direct callers cannot bypass the audit contract" are enforced by doc comments only; any
caller can invoke `record_source_activation` and commit without staging an audit event, and
neither the type system nor `arch-check` detects it.
**Fix:** Either gate the workflows behind a token only `AuditedMutation` can mint (e.g. take
`&AuditStaging` proof), or soften the claim to a convention with a `grep`-based CI check.

### IN-05: External-policy gate only recognizes `registry+`/`git+` schemes

**File:** `xtask/src/arch.rs:64-71`
**Issue:** A `sparse+https://…` source (the modern crates.io index scheme) classifies as
`Unknown` → violation. Fail-closed is the safe direction (loud build break, not silent
permissiveness), and live metadata on this toolchain reports `registry+`, so this is
forward-compat brittleness, not a live bug.
**Fix:** Accept `sparse+` (and `sparse+`-with-registry) as `Registry`:
`Some(s) if s.starts_with("registry+") || s.starts_with("sparse+") => ...`.

### IN-06: Records checker has three fail-open/fragile scoping assumptions

**File:** `scripts/verify-phase1-records.sh:103,128,194-205`
**Issue:** (a) Frontmatter extraction prints everything after a lone `---` when the closing
delimiter is absent (over-permissive fallback). (b) `section_from_header "$ROLLUP" "## "`
assumes the first `##` header is the newest Phase 1 section. (c)
`completed_count == 1` counts *lines*, so any second mention of the task id fails closure
(fail-closed, but brittle against benign references).
**Fix:** Require exactly two `---` delimiters (else fail); anchor the rollup header to the
newest-section title pattern; count whole-word occurrences or exempt quoted references.

### IN-07: Preservation store quirks (key collision, write-only diffs, tree-wide whitespace gate)

**File:** `scripts/verify-phase1-preservation.sh:47,124-129,260`
**Issue:** (a) `sanitize` maps `/` and space to `_`, so `a/b` and `a_b` collide in `$BYTES`.
(b) `$DIFFS` snapshots are never consumed by `check` (and for tracked files contain
whole-file `/dev/null` diffs due to the always-true branch at line 124). (c) `git diff --check`
runs tree-wide, so foreign whitespace errors fail our gate (observed repeatedly this phase).
**Fix:** Use a collision-free encoding (`%`-encoding or `sha256(path)` suffix); drop or consume
`$DIFFS`; scope the whitespace check to task-owned paths.

### IN-08: Serve-path creating constructors outside the guard (known deferred scope)

**File:** `crates/cli/src/lib.rs` (serve block, `d5183ff`), `crates/application/src/quran_cli.rs` (`open_reader`)
**Issue:** `SearchApiService::open` and the serve-host `open_reader` error path can still
create state outside the `open_db`/readiness guards. Explicitly deferred in 01-01 ("later-phase
domains, out of the foundation seven-group scope"), so recorded here for follow-up, not as a
regression: the readiness gate runs first, leaving only a TOCTOU window (gate Current → file
deleted → open creates unmigrated DB).
**Fix:** Route serve's opens through `open_host_database`-style existence backstops in the
owning phase.

### IN-09: Inconsistent subject URN schemes in newly staged audit events

**File:** `crates/application/src/quran_cli.rs` (`ensure_source_version`, via `8d05b06`)
**Issue:** The `SourceImported` event uses `SubjectRef(format!("quran-source:{source_id}"))`
while job lifecycle events use `urn:qai:job:<id>` and editions use `edition_urn(...)`. Three
schemes in one chain complicate subject-scoped queries and lifecycle agreement checks.
**Fix:** Standardize on `urn:qai:<domain>:<id>` (e.g. `urn:qai:source:<id>`).

---

_Reviewed: 2026-09-28T12:00:00Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
