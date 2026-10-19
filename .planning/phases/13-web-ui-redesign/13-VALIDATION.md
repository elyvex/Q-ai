# Phase 13 Validation Map

| Plan | Task | Automated verify |
|------|------|------------------|
| 13-01 | Tokens, typography, Reachability Generator, Component Library & Reader TRACER | `npm test -- src/components/ui src/tokens src/tests/reachability.test.ts src/views/reader/ReaderView.test.tsx` |
| 13-02 | App shell, responsive sidebar, routing scaffold, command palette & background | `npm test -- src/components/layout src/router.test.tsx` |
| 13-03 | Quran reader views, word inspector, citation copy URN & search views with sparklines | `npm test -- src/views/reader src/views/search` |
| 13-04 | Linguistics explorer, paradigm tables, root/lemma charts, word-family tree | `npm test -- src/views/linguistics` |
| 13-05 | Graph explorer, neighbor expansion, shortest-path canvas, JSON/DOT export | `npm test -- src/views/graph` |
| 13-06 | Hadith browser, tafsir parallel columns, isnad DAG visualization & rijal review | `npm test -- src/views/hadith src/views/tafsir src/views/isnad` |
| 13-07 | RAG dashboard, project list, agent research streaming, comparative scripture | `npm test -- src/views/rag src/views/agents src/views/scripture` |
| 13-08 | Management dashboard, source catalog pipeline, jobs, audit trail & doctor cards | `npm test -- src/views/management` |
| 13-09 | Motion polish, accessibility audit, bundle budget, parity gates & owner gates | `npm run build && npm test && cargo test -p server -- spa_fallback && cargo xtask arch-check` |

## Phase Gate
- `npm run build` in `web/` — green ($\le 350\text{ KB}$ gzipped initial bundle)
- `npm test` in `web/` — 100% passed
- Feature Reachability Matrix test (`src/tests/reachability.test.ts`) — 100% green
- `cargo test -p application --test layer_separation` — green
- `cargo test -p server --test api -- spa_fallback_serves_index_and_preserves_api` — green
- `cargo xtask arch-check` — green
