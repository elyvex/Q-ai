# CLI Contract: `qai quran search`

**Feature**: `030-search-service-wiring` | **Date**: 2026-09-18

## Synopsis

```text
qai quran search --query <text> --tool <tool> [options]
qai quran search --query <text> [--json]   # defaults: --tool search_normalized
```

## Flags

| Flag | Values / default | Notes |
|---|---|---|
| `--query <text>` | required | Raw query text |
| `--tool <tool>` | `search_exact \| search_normalized \| search_phrase \| search_concatenated \| search_regex`; default `search_normalized` | Selects the application service |
| `--edition <slug@version>` | optional | Defaults to indexed edition |
| `--profile <id[@version]>` | e.g. `L3.diacritics`, `L3.diacritics@1.2.0` | Registry profile; conflicts with `--rules` |
| `--rules <list>` | e.g. `N01,N03` | Adhoc rule list; conflicts with `--profile` |
| `--field` | `text_exact \| text_ws`; default `text_exact` | Exact tool only |
| `--match-mode` | `whole_token \| substring \| ayah_prefix`; default `whole_token` | Exact tool only |
| `--phrase-mode` | `ordered_exact \| ordered_near \| unordered_near` | Phrase tool only |
| `--slop <N>` | `u32` | Phrase tool only |
| `--allow-cross-ayah` | flag | Concatenated tool only |
| `--max-ayah-span <N>` | `u32`, `>= 1` | Concatenated tool only |
| `--regex-field <field>` | indexed text field | Regex tool only |
| `--pattern <expr>` | required with regex tool | DFA-compiled; backend syntax never exposed |
| `--principal <id>` | rate-limit identity | Regex tool |
| `--timeout-ms <N>` | default 2000, clamp 1..10000 | Regex tool (other tools use engine default) |
| `--surah <n...>` | `u16` list | AND-combined filters |
| `--juz <a-b>` | inclusive range | |
| `--page <n...>` | `u32` list | |
| `--revelation-place` | `makki \| madani` | |
| `--global-range <a-b>` | inclusive range | |
| `--limit <N>` | default 100, ceiling 1000 | Clamped; effective value reported |
| `--offset <M>` | default 0 | Deterministic paging |
| `--explain` | flag | Relevance order + score breakdowns |
| `--highlight` | flag | `<b>` display markers on rendered copy |
| `--json` | flag | Full section-12 contract output |

## Exit codes

`0` success · `2` usage (both `--profile`+`--rules`, missing required) · `3` validation · `4` policy/rate-limit · `5` not indexed/found · `6` state conflict (edition mismatch) · `70` internal. Errors print `error: {message}` (human) / `{"error":…}` (JSON); never stack traces.

## JSON output

Top-level section-12 envelope per [data-model.md](../data-model.md) Entity 2 (tool identity, query, rules, edition, hits, totals, truncation, generation, warnings, regex report, references, timing, reproducibility checksum).

## Human output

Ranked or canonically ordered hits with reference + quotation + trace label; normalization shown as labelled trace/rules only — canonical text never edited. Reports total, truncation, generation, and warnings footer.

## Examples

```bash
qai quran search --query "word" --tool search_normalized --profile L3.diacritics --limit 20
qai quran search --query "word" --tool search_normalized --profile L3.diacritics --limit 20 --json
qai quran search --query "phrase text" --tool search_phrase --phrase-mode ordered_near --slop 2
qai quran search --query "x" --tool search_regex --regex-field text_bare --pattern "expr" --principal cli
```
