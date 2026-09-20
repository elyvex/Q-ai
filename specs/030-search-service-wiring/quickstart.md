# Quickstart: Search-Service Wiring Validation

**Feature**: `030-search-service-wiring` | **Date**: 2026-09-18

Runnable validation scenarios proving the wiring works end-to-end. Implementation details belong in `tasks.md`; contracts in [contracts/](contracts/), data shapes in [data-model.md](data-model.md).

## Prerequisites

```bash
export QAI_DATA_DIR="$(mktemp -d)"
./target/debug/qai db migrate
./target/debug/qai quran import fixtures/quran/test-edition-min/manifest.json
./target/debug/qai quran activate test-edition-min@0.1.0 --yes
./target/debug/qai quran forms rebuild test-edition-min@0.1.0
./target/debug/qai quran index rebuild
./target/debug/qai quran index verify
```

## Scenario 1 — CLI search, all five tools (SC-001, SC-003)

For each `--tool {search_exact,search_normalized,search_phrase,search_concatenated,search_regex}` (regex adds `--regex-field` + `--pattern` + `--principal`):

```bash
./target/debug/qai quran search --query "<fixture term>" --tool <tool> --limit 20 --json
```

- **Expect**: exit 0, schema-valid section-12 JSON (tool identity, query, rules, edition, hits with reference/quotation/span/trace/generation, totals, warnings, timing, checksum).

## Scenario 2 — Pagination exactness (SC-004)

```bash
./target/debug/qai quran search --query "<multi-page term>" --limit 5 --offset 0 --json > p1.json
./target/debug/qai quran search --query "<multi-page term>" --limit 5 --offset 5 --json > p2.json
```

- **Expect**: identical repeats return identical pages; concatenated pages equal the untruncated ordered set; `total_matches` exact with `truncated` flag.

## Scenario 3 — HTTP envelope + ETag (SC-002, SC-003)

```bash
./target/debug/qai serve --bind 127.0.0.1:8737 &
curl "http://127.0.0.1:8737/api/v1/quran/search/normalized?q=<term>&profile=<id>&limit=20&offset=0"
```

- **Expect**: 200 `Envelope{data,meta}` with ETag; hit set identical (references, spans, order, totals) to the CLI `--json` output for the same query/edition/profile/pagination; repeat with `If-None-Match` short-circuits.

## Scenario 4 — SSE stream

```bash
curl -H "Accept: text/event-stream" "http://127.0.0.1:8737/api/v1/quran/search/stream?q=<term>&profile=<id>"
```

- **Expect**: N `hit` events in paged hit schema + exactly one terminal `summary` event (total, truncated, warnings, checksum).

## Scenario 5 — Misuse matrix (SC-005)

| Invocation | CLI expect | HTTP expect |
|---|---|---|
| `--profile X` + `--rules Y` | exit 2 | 400 |
| unknown profile | non-zero Diagnostic | 400/404 Diagnostic |
| regex non-indexed field | policy/state non-zero | 400/409 |
| `--limit 5000` | clamped to 1000, reported | clamped, meta shows effective |
| unbuilt index (fresh `QAI_DATA_DIR`, skip `index rebuild`) | exit 5, rebuild remedy | 404 |

## Scenario 6 — No-regression gate (SC-006)

```bash
cargo test --workspace
cargo run -p xtask -- arch-check
```

- **Expect**: engine suites, golden reference set, and all gates green — identical to pre-wiring baseline.
