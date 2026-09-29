# Quran Reindex Runbook

> **Audience:** operators performing index rebuilds and maintenance.
> **Implementation:** `crates/application/src/quran_index.rs`
> **Status:** Code-implemented

## 1. Overview

This runbook describes the index rebuild lifecycle: rebuild forms, rebuild
indexes, re-import morphology, and verify with doctor.

## 2. Rebuild Lifecycle

### 2.1 Rebuild Forms

```bash
qai quran forms rebuild
```

**What it does:**
1. Resolves the active edition
2. Runs MV-018 pre-check (canonical unchanged)
3. Stages a fresh generation directory
4. Streams ayah documents from stored derived forms
5. Commits the new generation
6. Reopens with the counted manifest
7. Verifies the build
8. Runs MV-018 post-check
9. Flips the pointer in one SQLite transaction

**Job kind:** `quran.forms.rebuild`

**Output:** `IndexBuildReport` with generation, doc count, manifest hash,
and MV-018 result.

### 2.2 Rebuild Indexes

```bash
qai quran index build
```

**What it does:**
1. Resolves the active edition
2. Runs MV-018 pre-check
3. Stages a fresh generation directory
4. Streams ayah docs (stored derived forms feed the text fields)
5. Commits the new generation
6. Reopens with the counted manifest
7. Verifies the build
8. Runs MV-018 post-check
9. Flips the pointer and marks runs in ONE SQLite transaction

**Job kind:** `quran.index.build`

**Output:** `IndexBuildReport` with generation, doc count, manifest hash,
trigram stats, and MV-018 result.

### 2.3 Re-import Morphology

```bash
qai quran morphology import --dataset path/to/dataset.json
qai quran morphology activate --dataset path/to/dataset.json
```

**What it does:**
1. Import: validates the dataset, stages it, and requires approval
2. Activate: validates license evidence, flips the dataset state to active,
   and enqueues an FTS rebuild

**Job kind:** `quran.morphology.import`

## 3. Order of Operations

The correct order for a full rebuild:

```
1. qai quran forms rebuild          # Rebuild derived forms
2. qai quran index build            # Rebuild search indexes
3. qai quran morphology import      # Import morphology dataset
4. qai quran morphology activate     # Activate morphology dataset
5. qai doctor --indexes             # Verify all indexes
```

**Important:** Forms must be rebuilt before indexes, because indexes are
built from derived forms. Morphology must be imported and activated after
indexes, because the FTS lexicon projection depends on the index.

## 4. Job IDs and Audit Trail

Each rebuild produces a job ID that can be inspected:

```bash
qai job show --id <job-id>
```

The audit trail records:
- Job kind and parameters
- Start and end timestamps
- Checkpoint progress
- Final status (Succeeded, Failed, DeadLettered)

## 5. What Drift Looks Like

### 5.1 `qai doctor --indexes`

```bash
qai doctor --indexes
```

This command runs 19 stable index checks. After a successful rebuild, all
19 checks should be green.

### 5.2 QAI-IDX-0101 Stale-Index Warning

When the serving index generation does not match the corpus generation
(e.g., after activating a new edition version without rebuilding), doctor
reports:

```
QAI-IDX-0101: stale-index warning
```

This means the serving index is out of date relative to the canonical text.
**Doctor is read-only** — it reports drift but never repairs it.

### 5.3 Other Drift Indicators

- `quran.index.drift` — warn-level check for index/corpus generation mismatch
- `quran.forms.drift` — warn-level check for forms/corpus generation mismatch
- `quran.morphology.drift` — warn-level check for morphology/index mismatch

## 6. Confirmed Repair Commands

When drift is detected, the confirmed repair commands are:

```bash
# Rebuild forms
qai quran forms rebuild

# Rebuild indexes
qai quran index build

# Re-import and activate morphology
qai quran morphology import --dataset <path>
qai quran morphology activate --dataset <path>
```

**Important:** These commands are confirmed (require `--yes` or explicit
confirmation). They are never automatic.

## 7. What Doctor Will Never Do

Doctor is strictly read-only. It will **never**:
- Rebuild forms or indexes
- Activate datasets
- Modify canonical data
- Repair drift automatically

Repair is always an explicit, confirmed command.

## 8. Single-Step Rollback

If a build fails, the previous generation stays on disk:

```bash
qai quran index rollback
```

This flips the pointer back to the previous verified generation. A partial
index can never serve queries.

## 9. Retention and GC

Index generations are retained for rollback. To clean up old generations:

```bash
qai quran index gc
```

This removes generations that are no longer needed for rollback, preserving
the active generation and the previous one.

## 10. Performance Budgets

The concatenated search p99 budget is ≤ 150 ms (ADR-0207). The fixture
harness gates against this bound:

```bash
cargo test -p application --test search_latency
```

Full-corpus performance rows are OD-11-dependent (require a licensed
dataset).

## 11. Related Documents

- `docs/07-technical/quran-search-cookbook.md` — search mode examples
- `docs/07-technical/quran-normalization-spec.md` — rule catalog
- `docs/02-architecture/decisions/ADR-0207-concatenated-search.md` — performance budget
