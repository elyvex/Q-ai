# Evaluation Datasets

**Purpose.** Versioned datasets for the evaluation harness (`crates/evaluation`).
Each dataset measures a specific metric set against the synthetic fixture
(`test-edition-min` + `test-morph`). The harness gates **mechanical**
regressions only — it is not a linguistic quality verdict (ADR-0204/0211 are
Draft).

## Files

| File | Version | Class | Measures |
|------|---------|-------|----------|
| `search-v1.jsonl` | 1 | `synthetic_test_only` | Search golden-agreement rate, empty-result rate, determinism rate |
| `lexicon-v1.jsonl` | 1 | `synthetic_test_only` | Lexicon attribution-completeness rate, typed-unavailable rate, determinism rate |
| `baseline.json` | — | — | Committed baseline values with explicit tolerances |

## Dataset format

Each `.jsonl` file has a header row:

```json
{"header": true, "version": 1, "reviewed_by": "pending-linguist", "dataset_class": "synthetic_test_only", "note": "..."}
```

- `version` — dataset version. The loader refuses an unknown or missing version.
- `reviewed_by` — always `pending-linguist` until OD-12 (linguist sign-off) closes.
- `dataset_class` — always `synthetic_test_only` for fixture data.

## Regenerating

The datasets are committed fixtures. To regenerate:

1. **search-v1.jsonl** — reuse the rows from `fixtures/quran/search/queries.jsonl`
   (400 rows: 200 exact, 100 normalized, 60 phrase, 40 regex). The rows are
   derived from the independent Python oracle (`scripts/gen_search_goldens.py`).
2. **lexicon-v1.jsonl** — derive rows from the synthetic lexicon's deterministic
   scheme (`root = k % 3`, `lemma = k % 9`). Every row is `synthetic_test_only`.

No fixture text is ever invented. Every Quranic string traces to a committed
fixture or an owner-supplied import.

## Baseline

`baseline.json` records the current measured value for each metric with an
explicit `tolerance` column. The harness diffs a fresh run against it. A metric
outside tolerance fails; a metric inside tolerance passes. The comparison is
exact against the declared tolerance, never a fuzzy similarity.

## Scope

These datasets are **synthetic** and **pending linguistic review**. They prove
mechanical behavior on the 14-ayah fixture, never scholarly ground truth. The
linguist/dataset ratification stays BLOCKED (OD-11/OD-12,
`docs/05-followups/phase-03-owner-gates.md`).
