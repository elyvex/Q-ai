#!/bin/sh
# verify-phase1-records.sh — record-specific Phase 1 closure assertions
# (Phase 1 / TASK-001, plan 01-05 contract, created by task 01-05-02).
#
# Each required token is checked in exactly one designated record via
# require_tokens <file> <label> <tokens...>: a token present in one record
# never satisfies another record's requirement. There is no union search,
# no rg across records, and no suppressed command errors (set -eu).
#
# After the per-record assertions, the script delegates per-path byte/hunk
# preservation to the companion verifier before accepting closure.
#
# Usage (from the repository root):
#   sh scripts/verify-phase1-records.sh
set -eu

ROOT=$(git rev-parse --show-toplevel 2>/dev/null || echo ".")
cd "$ROOT" || exit 1

VALIDATION=".planning/phases/01-foundations/01-VALIDATION.md"
COMPLETED_TASK="docs/04-tasks/completed/TASK-001-foundation-gap-closure.md"
ACTIVE_TASK="docs/04-tasks/active/TASK-001-foundation-gap-closure.md"
ACTIVE_INDEX="docs/04-tasks/active/README.md"
COMPLETED_INDEX="docs/04-tasks/completed/README.md"
ROLLUP="docs/06-progress/task-done-rollup.md"
STATUS="docs/06-progress/status.md"
CHANGELOG="CHANGELOG.md"
PRESERVATION="scripts/verify-phase1-preservation.sh"

TASKS="01-01-01 01-01-02 01-01-03 01-02-01 01-02-02 01-03-01 01-03-02 01-03-03 01-04-01 01-04-02 01-04-03 01-05-01 01-05-02 01-05-03"
PLANS="01-01 01-02 01-03 01-04 01-05"
REQS="REQ-product-vision REQ-product-principles REQ-goals-non-goals REQ-engineering-baseline REQ-storage-architecture REQ-cli-api REQ-architecture-principles-quality"
CRITERIA="C1 C2 C3 C4 C5"
DECISIONS="D-01 D-02 D-03 D-04 D-05 D-06 D-07 D-08 D-09 D-10 D-11 D-12 D-13 D-14 D-15 D-16"
PROBES="PROBE-01 PROBE-02 PROBE-03 PROBE-04 PROBE-05 PROBE-06 PROBE-07 PROBE-08 PROBE-09 PROBE-10"
THREATS="T-01-CFG T-01-DOCTOR T-01-MIG T-01-AUDIT T-01-SQL T-02-ATOMICITY T-02-PROVENANCE T-02-CANONICAL T-02-RECOVERY T-02-SQL T-03-LEASE T-03-RETRY T-03-CHECKPOINT T-03-SECRET T-03-CONCURRENCY T-04-HOST T-04-SHUTDOWN T-04-ENQUEUE T-04-LEASE T-04-SECRET T-04-SCOPE T-05-ARCH T-05-EXTERNAL T-05-SUPPLYCHAIN T-05-STATUS T-05-SCOPE T-05-INFO"
GATE_FAMILY="cargo fmt cargo clippy cargo test --workspace arch-check migrate-check"

fail=0
note() { printf 'verify-phase1-records: %s\n' "$1"; }
err() { printf 'verify-phase1-records: FAIL: %s\n' "$1" >&2; fail=1; }

# require_file <path> <label> — the designated record must exist.
require_file() {
  if [ ! -f "$1" ]; then
    err "$2: missing record: $1"
    return 1
  fi
  return 0
}

# require_tokens <file> <label> <token...> — every token must occur in the
# one named file (whole-word fixed-string match, WR-12). Missing files fail;
# grep errors abort. Substring matching is banned: a record mentioning only
# `C10` must not satisfy `C1`, and `TASK-001x` must not satisfy `TASK-001`.
require_tokens() {
  file=$1
  label=$2
  shift 2
  require_file "$file" "$label" || return 0
  # shellcheck disable=SC2086
  missing=""
  for tok in "$@"; do
    if ! grep -F -q -w -- "$tok" "$file"; then
      missing="$missing [$tok]"
    fi
  done
  if [ -n "$missing" ]; then
    err "$label: missing token(s) in $file:$missing"
  else
    note "$label: ok ($file)"
  fi
}

TMPDIR_WORK=$(mktemp -d "${TMPDIR:-/tmp}/phase1-records.XXXXXX")
trap 'rm -rf "$TMPDIR_WORK"' EXIT INT TERM

# section_from_header <file> <header-prefix> <out> — lines from the first
# line starting with <header-prefix> (inclusive) to the next '## ' header
# (exclusive). Used to scope newest-section checks to their own record part.
# IN-06(b): callers must pass the newest section's title pattern, never bare
# "## " — the file's first '## ' header is not necessarily the wanted one.
section_from_header() {
  awk -v hdr="$2" 'index($0, hdr) == 1 {found = 1; print; next} found && /^## / {exit} found {print}' "$1" > "$3"
}

note "records root: $ROOT"

# (a) Validation matrix carries every task/requirement/criterion/decision/
# probe/threat token plus both completion flags — independently. The flags
# are scoped to the YAML frontmatter so a prose mention elsewhere in the
# file cannot satisfy them.
# shellcheck disable=SC2086
require_tokens "$VALIDATION" "validation.tasks" $TASKS
# shellcheck disable=SC2086
require_tokens "$VALIDATION" "validation.requirements" $REQS
# shellcheck disable=SC2086
require_tokens "$VALIDATION" "validation.criteria" $CRITERIA
# shellcheck disable=SC2086
require_tokens "$VALIDATION" "validation.decisions" $DECISIONS
# shellcheck disable=SC2086
require_tokens "$VALIDATION" "validation.probes" $PROBES
# shellcheck disable=SC2086
require_tokens "$VALIDATION" "validation.threats" $THREATS
VALIDATION_FRONTMATTER="$TMPDIR_WORK/validation-frontmatter.md"
if require_file "$VALIDATION" "validation.flags"; then
  # IN-06(a): frontmatter is the block between line-1 `---` and the next
  # `---`. A missing opening or closing delimiter fails instead of
  # over-permissively matching prose `---` rules elsewhere in the file.
  first=$(head -1 "$VALIDATION")
  closes=$(awk 'NR>1 && /^---$/ {print NR; exit}' "$VALIDATION")
  if [ "$first" != "---" ] || [ -z "$closes" ]; then
    err "validation.flags: malformed frontmatter delimiters in $VALIDATION"
  else
    awk 'NR>1 && /^---$/ {exit} NR>1 {print}' "$VALIDATION" > "$VALIDATION_FRONTMATTER"
    require_tokens "$VALIDATION_FRONTMATTER" "validation.flags" "nyquist_compliant: true" "wave_0_complete: true"
  fi
fi

# (b) Completed task carries its own ID, status, plans, requirements,
# criteria, decisions, probes, and threats — independently.
# shellcheck disable=SC2086
require_tokens "$COMPLETED_TASK" "completed-task.identity" "TASK-001-foundation-gap-closure" "Status: Completed"
# shellcheck disable=SC2086
require_tokens "$COMPLETED_TASK" "completed-task.plans" $PLANS
# shellcheck disable=SC2086
require_tokens "$COMPLETED_TASK" "completed-task.requirements" $REQS
# shellcheck disable=SC2086
require_tokens "$COMPLETED_TASK" "completed-task.criteria" $CRITERIA
# shellcheck disable=SC2086
require_tokens "$COMPLETED_TASK" "completed-task.decisions" $DECISIONS
# shellcheck disable=SC2086
require_tokens "$COMPLETED_TASK" "completed-task.probes" $PROBES
# shellcheck disable=SC2086
require_tokens "$COMPLETED_TASK" "completed-task.threats" $THREATS

# (c) Newest Phase 1 rollup section carries TASK-001, plans,
# criteria, decisions, and the final-gate command family — independently.
# Anchored to the Phase 1 title pattern (IN-06(b)): the file's first '## '
# header belongs to a newer phase and must not satisfy Phase 1 checks.
ROLLUP_NEWEST="$TMPDIR_WORK/rollup-newest.md"
if require_file "$ROLLUP" "rollup.newest"; then
  section_from_header "$ROLLUP" "## Phase 1" "$ROLLUP_NEWEST"
  if [ ! -s "$ROLLUP_NEWEST" ]; then
    err "rollup.newest: no '## Phase 1' section in $ROLLUP"
  else
  # shellcheck disable=SC2086
  require_tokens "$ROLLUP_NEWEST" "rollup.newest.task" "TASK-001"
  # shellcheck disable=SC2086
  require_tokens "$ROLLUP_NEWEST" "rollup.newest.plans" $PLANS
  # shellcheck disable=SC2086
  require_tokens "$ROLLUP_NEWEST" "rollup.newest.criteria" $CRITERIA
  # shellcheck disable=SC2086
  require_tokens "$ROLLUP_NEWEST" "rollup.newest.decisions" $DECISIONS
  # shellcheck disable=SC2086
  require_tokens "$ROLLUP_NEWEST" "rollup.newest.gate" $GATE_FAMILY
  fi
fi

# (d) Dated Phase 1 closure status section carries TASK-001, plans,
# criteria, decisions, and the completed status — independently.
STATUS_SECTION="$TMPDIR_WORK/status-section.md"
if require_file "$STATUS" "status.closure"; then
  section_from_header "$STATUS" "## Phase 1 closure" "$STATUS_SECTION"
  if [ ! -s "$STATUS_SECTION" ]; then
    err "status.closure: no '## Phase 1 closure' section in $STATUS"
  else
    # shellcheck disable=SC2086
    require_tokens "$STATUS_SECTION" "status.closure.task" "TASK-001" "Status: Completed"
    # shellcheck disable=SC2086
    require_tokens "$STATUS_SECTION" "status.closure.plans" $PLANS
    # shellcheck disable=SC2086
    require_tokens "$STATUS_SECTION" "status.closure.criteria" $CRITERIA
    # shellcheck disable=SC2086
    require_tokens "$STATUS_SECTION" "status.closure.decisions" $DECISIONS
  fi
fi

# (e) Unreleased changelog entry carries TASK-001, plans, criteria,
# decisions, and the brownfield/non-expansion boundary — independently.
CHANGELOG_UNRELEASED="$TMPDIR_WORK/changelog-unreleased.md"
if require_file "$CHANGELOG" "changelog.unreleased"; then
  section_from_header "$CHANGELOG" "## [Unreleased]" "$CHANGELOG_UNRELEASED"
  if [ ! -s "$CHANGELOG_UNRELEASED" ]; then
    err "changelog.unreleased: no '## [Unreleased]' section in $CHANGELOG"
  else
    # shellcheck disable=SC2086
    require_tokens "$CHANGELOG_UNRELEASED" "changelog.unreleased.task" "TASK-001"
    # shellcheck disable=SC2086
    require_tokens "$CHANGELOG_UNRELEASED" "changelog.unreleased.plans" $PLANS
    # shellcheck disable=SC2086
    require_tokens "$CHANGELOG_UNRELEASED" "changelog.unreleased.criteria" $CRITERIA
    # shellcheck disable=SC2086
    require_tokens "$CHANGELOG_UNRELEASED" "changelog.unreleased.decisions" $DECISIONS
    require_tokens "$CHANGELOG_UNRELEASED" "changelog.unreleased.boundary" "brownfield" "non-expansion"
  fi
fi

# Lifecycle counts and path agreement: the active task path must be absent,
# the completed task path present, the active index free of TASK-001, and
# the completed index naming it exactly once.
if [ -e "$ACTIVE_TASK" ]; then
  err "lifecycle: active task path still present: $ACTIVE_TASK"
else
  note "lifecycle: active task path absent (ok)"
fi
if [ -f "$COMPLETED_TASK" ]; then
  note "lifecycle: completed task path present (ok)"
else
  err "lifecycle: completed task path missing: $COMPLETED_TASK"
fi
if require_file "$ACTIVE_INDEX" "lifecycle.active-index" && require_file "$COMPLETED_INDEX" "lifecycle.completed-index"; then
  active_count=$(grep -F -c -- "TASK-001-foundation-gap-closure" "$ACTIVE_INDEX" || true)
  # IN-06(c): count index ENTRIES, not lines or raw occurrences — one entry
  # per line in this `- [id](url)` list, so a link target doubling the id on
  # its entry's line can neither hide a missing entry nor fake a second one.
  completed_count=$(grep -F -c -- "- [TASK-001-foundation-gap-closure]" "$COMPLETED_INDEX" || true)
  if [ "$active_count" -eq 0 ]; then
    note "lifecycle: active index count is 0 (ok)"
  else
    err "lifecycle: active index count is $active_count, want 0"
  fi
  if [ "$completed_count" -eq 1 ]; then
    note "lifecycle: completed index count is 1 (ok)"
  else
    err "lifecycle: completed index count is $completed_count, want 1"
  fi
fi

# Preservation delegation: every shared-ledger addition is accepted only
# after the companion verifier proves per-path byte/hunk preservation.
if [ ! -f "$PRESERVATION" ]; then
  err "preservation: companion verifier missing: $PRESERVATION"
else
  if sh "$PRESERVATION" check --all; then
    note "preservation: companion check --all passed (ok)"
  else
    err "preservation: companion check --all failed (see output above)"
  fi
fi

if [ "$fail" -ne 0 ]; then
  printf 'verify-phase1-records: RESULT FAIL\n' >&2
  exit 1
fi
printf 'verify-phase1-records: RESULT PASS\n'
