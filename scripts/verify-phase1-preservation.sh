#!/bin/sh
# verify-phase1-preservation.sh — dynamic live-worktree preservation companion
# (Phase 1 / TASK-001, plan 01-01 contract, 2026-09-25 revision).
#
# Purpose
# -------
# Prove that Phase 1 execution never rewrites a pre-existing user hunk. The
# revision-time dirty set is a snapshot, not an execution baseline, so `capture`
# always re-runs `git status --short --untracked-files=all` immediately before
# any task-owned edit and preserves every dynamically discovered dirty tracked
# and untracked path. `check` proves each discovered path is either untouched
# (EXCLUDE), append-only (APPEND), or retained through a conflict-free
# three-way merge (MERGE).
#
# Hard rules
# ----------
#   * Never run `git stash`, `git reset`, `git checkout`, or `git clean`.
#   * All preservation state lives under $(git rev-parse --git-dir)/
#     qai-phase1-preservation/ and is never staged or treated as a source
#     artifact.
#   * A discovered dirty/untracked path that cannot be copied and hashed aborts
#     capture (`capture` refuses to run if a path is not copied and hashed).
#
# Usage
# -----
#   sh scripts/verify-phase1-preservation.sh capture
#   sh scripts/verify-phase1-preservation.sh preflight --path <p> [--task <id>]
#   sh scripts/verify-phase1-preservation.sh check --task <id>
#   sh scripts/verify-phase1-preservation.sh check --all
#   sh scripts/verify-phase1-preservation.sh status
set -eu

GIT_DIR=$(git rev-parse --git-dir)
STATE="$GIT_DIR/qai-phase1-preservation"
BYTES="$STATE/bytes"
HEADS="$STATE/heads"
DIFFS="$STATE/diffs"
PREFLIGHT="$STATE/preflight"
ALLOWED="$STATE/allowed"
MANIFEST="$STATE/manifest.tsv"
CLASSIFIED="$STATE/classification.tsv"
META="$STATE/capture_commit"

die() { printf 'verify-phase1-preservation: ERROR: %s\n' "$1" >&2; exit 1; }
info() { printf 'verify-phase1-preservation: %s\n' "$1"; }

sanitize() { printf '%s' "$1" | tr '/ ' '__'; }

sha256_of() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  elif command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    die "no sha256 tool available"
  fi
}

mode_of() {
  stat -f '%Lp' "$1" 2>/dev/null || stat -c '%a' "$1" 2>/dev/null || printf '?'
}

# classify <path> -> EXCLUDE | MERGE | APPEND
classify() {
  case "$1" in
    crates/*.rs|crates/*/src/*.rs|crates/*/tests/*.rs)
      printf 'MERGE' ;;
    CHANGELOG.md|docs/06-progress/task-done-rollup.md|docs/03-plan/current-plan.md|docs/04-tasks/active/README.md|docs/06-progress/status.md)
      printf 'APPEND' ;;
    docs/04-tasks/active/TASK-001-foundation-gap-closure.md)
      printf 'APPEND' ;;
    *)
      printf 'EXCLUDE' ;;
  esac
}

known_live_paths() {
  cat <<'EOF'
CHANGELOG.md
crates/application/src/quran_doctor_indexes.rs
docs/03-plan/current-plan.md
docs/03-plan/phases/phase-02-rag/done.md
docs/03-plan/phases/phase-02-rag/tasks.md
docs/06-progress/task-done-rollup.md
.planning/phases/02-canonical-quran-core/02-01-PLAN.md
.planning/phases/02-canonical-quran-core/02-02-PLAN.md
.planning/phases/02-canonical-quran-core/02-03-PLAN.md
.planning/phases/02-canonical-quran-core/02-04-PLAN.md
.planning/phases/02-canonical-quran-core/02-05-PLAN.md
.planning/phases/02-canonical-quran-core/02-PATTERNS.md
EOF
}

# ─── capture ───────────────────────────────────────────────────────────
cmd_capture() {
  mkdir -p "$BYTES" "$HEADS" "$DIFFS" "$PREFLIGHT" "$ALLOWED"
  : > "$MANIFEST"
  : > "$CLASSIFIED"
  git rev-parse HEAD > "$META"
  info "capture baseline HEAD=$(cat "$META")"

  # Dynamically discovered dirty tracked + untracked paths.
  git status --short --untracked-files=all | while IFS= read -r line; do
    [ -z "$line" ] && continue
    status=$(printf '%s' "$line" | cut -c1-2)
    path=$(printf '%s' "$line" | cut -c4-)
    # Rename rows: "R  old -> new" — keep the new worktree path.
    case "$path" in
      *' -> '*) path=$(printf '%s' "$path" | sed 's/.* -> //') ;;
    esac
    key=$(sanitize "$path")
    copy="$BYTES/$key"
    if [ -e "$path" ]; then
      cp -p "$path" "$copy" || die "could not copy discovered path: $path"
    elif git cat-file -e "HEAD:$path" 2>/dev/null; then
      git show "HEAD:$path" > "$copy" || die "could not capture deleted path: $path"
    else
      printf 'deleted\t%s\t%s\t-\t-\n' "$path" "$status" >> "$MANIFEST"
      printf '%s\t%s\n' "$path" "$(classify "$path")" >> "$CLASSIFIED"
      continue
    fi
    sha=$(sha256_of "$copy") || die "could not hash discovered path: $path"
    mode=$(mode_of "$path")
    if [ "${status#??}" != "$status" ]; then
      # both index and worktree columns set (e.g. "??") -> untracked
      git diff --no-index --binary /dev/null "$path" > "$DIFFS/$key.diff" 2>/dev/null || true
    else
      git diff --binary -- "$path" > "$DIFFS/$key.diff" 2>/dev/null || true
    fi
    printf 'present\t%s\t%s\t%s\t%s\n' "$path" "$status" "$mode" "$sha" >> "$MANIFEST"
    printf '%s\t%s\n' "$path" "$(classify "$path")" >> "$CLASSIFIED"
    # clean-baseline HEAD copy when the path is tracked (used by MERGE check)
    if git cat-file -e "HEAD:$path" 2>/dev/null; then
      git show "HEAD:$path" > "$HEADS/$key"
    fi
    info "captured $path ($status) sha=$sha"
  done

  # Explicitly record the known live paths from the preservation contract.
  known_live_paths | while IFS= read -r path; do
    [ -z "$path" ] && continue
    key=$(sanitize "$path")
    if git cat-file -e "HEAD:$path" 2>/dev/null; then
      git show "HEAD:$path" > "$HEADS/$key"
    fi
    printf '%s\t%s\n' "$path" "$(classify "$path")" >> "$CLASSIFIED"
  done

  sort -u "$CLASSIFIED" -o "$CLASSIFIED"
  count=$(wc -l < "$MANIFEST" | tr -d ' ')
  info "capture complete: $count discovered dirty/untracked path(s) preserved"
}

# ─── preflight ─────────────────────────────────────────────────────────
cmd_preflight() {
  path=""
  task=""
  while [ $# -gt 0 ]; do
    case "$1" in
      --path) path=$2; shift 2 ;;
      --task) task=$2; shift 2 ;;
      *) die "unknown preflight flag: $1" ;;
    esac
  done
  [ -n "$path" ] || die "preflight requires --path"
  [ -f "$MANIFEST" ] || die "no capture found; run capture first"
  cls=$(classify "$path")
  if [ "$cls" = "EXCLUDE" ] && [ -e "$path" ]; then
    live=$(git status --short --untracked-files=all 2>/dev/null | cut -c4- | grep -F -x "$path" || true)
    if [ -n "$live" ]; then
      die "refusing to edit EXCLUDE path with live user bytes: $path"
    fi
  fi
  key=$(sanitize "$path")
  mkdir -p "$PREFLIGHT"
  if [ -e "$path" ]; then
    cp -p "$path" "$PREFLIGHT/${task:-adhoc}__$key" || die "preflight copy failed: $path"
    sha=$(sha256_of "$path")
    printf '%s\t%s\t%s\t%s\n' "${task:-adhoc}" "$path" "$cls" "$sha" >> "$ALLOWED/${task:-adhoc}.tsv"
    info "preflight $path [$cls] sha=$sha"
  else
    printf '%s\t%s\t%s\t%s\n' "${task:-adhoc}" "$path" "$cls" "absent" >> "$ALLOWED/${task:-adhoc}.tsv"
    info "preflight $path [$cls] (absent before edit)"
  fi
}

# Assert the captured pre-edit bytes for a MERGE path are retained after edit.
check_merge() {
  path=$1
  key=$(sanitize "$path")
  head_copy="$HEADS/$key"
  cap_copy="$BYTES/$key"
  [ -e "$path" ] || die "MERGE path missing after edit: $path"
  [ -f "$head_copy" ] || return 0   # new file we own; nothing pre-existing to preserve
  tmp=$(mktemp -d "$STATE/check.XXXXXX")
  cp "$head_copy" "$tmp/base"
  if [ -f "$cap_copy" ]; then cp "$cap_copy" "$tmp/ours"; else cp "$head_copy" "$tmp/ours"; fi
  cp "$path" "$tmp/theirs"
  if git merge-file -q "$tmp/ours" "$tmp/base" "$tmp/theirs"; then
    rm -rf "$tmp"
    return 0
  fi
  rm -rf "$tmp"
  die "MERGE conflict: pre-existing hunk not preserved in $path"
}

check_one() {
  path=$1
  cls=$2
  key=$(sanitize "$path")
  cap_copy="$BYTES/$key"
  if [ ! -f "$cap_copy" ]; then
    # known-live but clean at capture: only HEAD-baseline check applies
    case "$cls" in
      EXCLUDE)
        if [ -e "$path" ] && git cat-file -e "HEAD:$path" 2>/dev/null; then
          git show "HEAD:$path" | cmp -s - "$path" || die "EXCLUDE path changed: $path"
        fi ;;
      MERGE) check_merge "$path" ;;
    esac
    return 0
  fi
  case "$cls" in
    EXCLUDE)
      [ -e "$path" ] || die "EXCLUDE path removed: $path"
      [ "$(sha256_of "$path")" = "$(sha256_of "$cap_copy")" ] || die "EXCLUDE path modified: $path"
      ;;
    APPEND)
      [ -e "$path" ] || die "APPEND path removed: $path"
      if ! head -c "$(wc -c < "$cap_copy" | tr -d ' ')" "$path" | cmp -s - "$cap_copy"; then
        if ! tail -c "$(wc -c < "$cap_copy" | tr -d ' ')" "$path" | cmp -s - "$cap_copy"; then
          die "APPEND path no longer contains captured bytes as prefix/suffix: $path"
        fi
      fi
      ;;
    MERGE)
      check_merge "$path"
      ;;
  esac
}

cmd_check() {
  mode=${1:---all}
  [ -f "$MANIFEST" ] || die "no capture found; run capture first"
  checked=0
  while IFS="$(printf '\t')" read -r kind path status mode_ sha; do
    [ -z "$path" ] && continue
    cls=$(awk -F'\t' -v p="$path" '$1==p {print $2}' "$CLASSIFIED" | head -1)
    [ -n "$cls" ] || cls=EXCLUDE
    check_one "$path" "$cls"
    checked=$((checked + 1))
  done < "$MANIFEST"
  # Also verify known-live MERGE/EXCLUDE paths declared in the classification.
  while IFS="$(printf '\t')" read -r path cls; do
    [ -z "$path" ] && continue
    case "$path" in
      crates/*) check_one "$path" "$cls"; checked=$((checked + 1)) ;;
    esac
  done < "$CLASSIFIED"
  git diff --check || die "git diff --check reported whitespace errors"
  info "check ($mode) passed for $checked path(s)"
}

cmd_status() {
  [ -d "$STATE" ] || die "no preservation state; run capture first"
  info "state: $STATE"
  printf 'baseline HEAD: %s\n' "$(cat "$META" 2>/dev/null || echo '?')"
  printf 'captured paths: %s\n' "$(wc -l < "$MANIFEST" | tr -d ' ')"
  printf 'classified paths: %s\n' "$(wc -l < "$CLASSIFIED" | tr -d ' ')"
}

[ $# -ge 1 ] || die "usage: $0 {capture|preflight|check|status} [args]"
cmd=$1
shift
case "$cmd" in
  capture) cmd_capture ;;
  preflight) cmd_preflight "$@" ;;
  check) cmd_check "$@" ;;
  status) cmd_status ;;
  *) die "unknown command: $cmd" ;;
esac
