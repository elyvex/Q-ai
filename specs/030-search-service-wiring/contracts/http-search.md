# HTTP Contract: Search Routes

**Feature**: `030-search-service-wiring` | **Date**: 2026-09-18

All routes inherit the existing loopback-only guard and the `Envelope{data,meta}` + `Diagnostic` + ETag + `Content-Language` conventions. Error statuses per [data-model.md](../data-model.md) Entity 4.

## Routes

```text
GET /api/v1/quran/search/exact?q=...        # search_exact
GET /api/v1/quran/search/normalized?q=...   # search_normalized
GET /api/v1/quran/search/phrase?q=...       # search_phrase
GET /api/v1/quran/search/concatenated?q=... # search_concatenated
GET /api/v1/quran/search/regex?...          # search_regex
GET /api/v1/quran/search/stream?...         # SSE, normalized-search param set
```

## Query parameters (paged routes)

| Param | Notes |
|---|---|
| `q` | required; empty-after-normalization → zero hits, never match-all |
| `edition` | optional `slug@version`; mismatch → 409 |
| `profile` \| `rules` | mutually exclusive; both → 400; `profile` accepts `id[@version]` |
| `phrase_mode`, `slop` | phrase route only |
| `field` (`text_exact`\|`text_ws`), `match_mode` | exact route only |
| `regex_field`, `pattern`, `principal` | regex route only; non-indexed field → 400/409 typed rejection |
| `surah`, `juz`, `page`, `revelation_place`, `global` | AND-combined filters; unsatisfiable → 200 with zero hits |
| `limit` | default 100, ceiling 1000 (clamped, effective value in meta) |
| `offset` | default 0 |
| `explain`, `highlight` | flags |
| `timeout_ms` | default 2000, clamp 1..10000 |

## 200 response

`Envelope<SearchEnvelope>` (shape per [data-model.md](../data-model.md) Entity 2) with `ETag` header; matching `If-None-Match` short-circuits the body (existing `json_response` behavior). `meta` carries edition descriptor, `corpus_generation`, timing, `reproducibility` JSON, warnings.

## Errors

`Diagnostic` error bodies: `400` usage/validation · `404` no serving index · `409` edition/state mismatch · `429` rate-limited · `5xx` internal. Never 200-with-empty on failure.

## SSE stream (`/search/stream`, `Accept: text/event-stream`)

- `hit` events: one per hit, paged hit schema.
- Exactly one terminal `summary` event: `{total_matches, truncated, warnings, reproducibility_checksum}`.
- Disconnect: server stops promptly, no above-warning log.

## Examples

```bash
curl "http://127.0.0.1:8737/api/v1/quran/search/normalized?q=text&profile=L3.diacritics&limit=20&offset=0"
curl "http://127.0.0.1:8737/api/v1/quran/search/regex?regex_field=text_bare&pattern=expr&principal=gui"
curl -H "Accept: text/event-stream" "http://127.0.0.1:8737/api/v1/quran/search/stream?q=text&profile=L3.diacritics"
```
