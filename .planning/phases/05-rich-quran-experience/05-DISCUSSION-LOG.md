# Phase 5: Rich Quran Experience - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-10-06
**Phase:** 05-rich-quran-experience
**Areas discussed:** Web GUI delivery shape, Reading view layout, TUI cockpit depth, Result contract + checksum

---

## Web GUI delivery shape

| Option | Description | Selected |
|--------|-------------|----------|
| Server HTML | Extend the existing axum debug-reader precedent; zero new toolchain; plain HTML+CSS RTL | |
| SPA or WASM frontend | Rich interactivity (word-inspector popovers, live search); adds a build pipeline | ✓ |
| API only, no GUI | Smallest scope, but SC1 reading view unmet | |

| Option | Description | Selected |
|--------|-------------|----------|
| Rust WASM | One language, offline-friendly, no JS toolchain; heavier build/bundle | |
| JS SPA + API | Mature ecosystem + RTL libraries; adds a JS toolchain | ✓ |
| Server HTML + islands | Most richness with less pipeline; limits on popovers/live search | |

| Option | Description | Selected |
|--------|-------------|----------|
| Embed in binary | qai embeds the built SPA, serves same-origin with /api/v1; true single binary, offline, no CORS | ✓ |
| Serve from disk | qai serves a static dir; qai alone no longer self-contained | |
| Dev server now | Separate dev server; production decision deferred | |

| Option | Description | Selected |
|--------|-------------|----------|
| React + Vite | React + TypeScript, Vite, plain fetch; largest RTL/Arabic ecosystem | ✓ |
| Vue or Svelte | Lighter bundle, smaller RTL ecosystem | |
| No framework | Vanilla TS; smallest footprint, most hand-rolled UI | |

**User's choice:** SPA → JS SPA + API → embed in binary → React + Vite
**Notes:** User rejected server-rendered HTML in favor of a full SPA, then chose a JS/TS stack over Rust/WASM. Offline-first and single-binary constraints carried through.

---

## Reading view layout

| Option | Description | Selected |
|--------|-------------|----------|
| Single column + panels | Centered RTL ayah flow with collapsible side panels | ✓ |
| Three-pane workspace | Persistent nav / canonical / translation panes | |
| Continuous mushaf page | Page-like continuous layout | |

| Option | Description | Selected |
|--------|-------------|----------|
| Inline under each ayah | Translation beneath each ayah | ✓ (as one mode) |
| Side-by-side columns | Parallel canonical/translation columns | ✓ (as one mode) |
| Toggleable side panel | Translations in an off-by-default panel | ✓ (as one mode) |

**User's choice:** "switchable between all three"
**Notes:** User wants all three translation presentations available as a user-switchable display mode, not a single hardcoded choice.

| Option | Description | Selected |
|--------|-------------|----------|
| Popover on click | Inline popover with lemma/root/analyses/family | ✓ (as one mode) |
| Persistent side panel | Selection fills a persistent panel | ✓ (as one mode) |
| Popover + pin to panel | Peek + pin action | ✓ (as one mode) |

**User's choice:** "switchable all three"
**Notes:** Same flexibility theme — all three word-inspector interactions, switchable.

| Option | Description | Selected |
|--------|-------------|----------|
| Shared layer token set | One token vocabulary applied everywhere (anti-drift) | ✓ |
| Per-view styling | Each view styles layers independently | ✓ |
| User-configurable themes | User-tunable layer styling | ✓ |

**User's choice:** "all of above"
**Notes:** SC5 uses all three: shared token set as baseline + per-view styling + user themes.

---

## TUI cockpit depth

| Option | Description | Selected |
|--------|-------------|----------|
| Ops cockpit now | Palette, doctor, jobs, index/corpus status, reader + search; RAG view scaffolded | ✓ |
| Full GUI-parity TUI | Reading + research + inspector + graph nav | |
| Thin TUI + CLI | Reader + search + palette; rely on CLI for ops | |

| Option | Description | Selected |
|--------|-------------|----------|
| Screen + typed unavailable | Build the RAG debug screen now, render typed "not configured" until Phase 8 | ✓ |
| Defer RAG screen | Ship palette, defer RAG view to Phase 8 | |
| Stub retrieval now | Minimal retrieval stub to populate the view | |

| Option | Description | Selected |
|--------|-------------|----------|
| Shared services layer | TUI calls the same application services/typed tools as API and CLI | ✓ |
| Shell out to CLI | TUI renders CLI output | |
| Talk to HTTP API | TUI talks HTTP to the local server | |

| Option | Description | Selected |
|--------|-------------|----------|
| Reader+search+ops set | Reader, search, palette, doctor, jobs, index/corpus status | ✓ (included) |
| Plus graph screens | Add graph navigation + annotation review | ✓ (included) |
| Ops only, no reader | Palette + doctor + jobs only | |

**User's choice:** Ops cockpit now → typed-unavailable RAG screen → shared services layer → "1 and 2" (reader+search+ops **and** graph screens)
**Notes:** User wants an honest SC4 story (typed unavailable, no faked retrieval), structural parity via the shared services layer, and a broader screen inventory that includes Phase-4 graph navigation + annotation review.

---

## Result contract + checksum

| Option | Description | Selected |
|--------|-------------|----------|
| Inputs+tool+payload | Covers canonical inputs, tool id+version, normalized params, result payload (canonical-JSON + sha256) | ✓ |
| Payload only | Covers only result bytes; collides across different queries | |
| Whole envelope minus volatile | Full envelope minus timestamps/durations/trace ids | |

| Option | Description | Selected |
|--------|-------------|----------|
| Field on ToolResult | Required `research_checksum` on the shared contract | ✓ |
| Presentation-layer only | Computed in UI/CLI from the envelope | |
| Separate verify endpoint | Clients call a verify endpoint | |

| Option | Description | Selected |
|--------|-------------|----------|
| Cross-surface golden gate | One fixture runs every tool through CLI/HTTP/UI; asserts byte-identical payload + checksum | ✓ |
| Per-surface tests | Unit tests per surface (does not prove identity) | |
| Manual UAT only | Document contract, verify by hand | |

| Option | Description | Selected |
|--------|-------------|----------|
| URN + verify on open | Copy yields citation URN + deep link; opening re-verifies via verify_quotation; hard-fail on mismatch | ✓ |
| Plain quote + source | Plain-text quote with source line; manual verification | |
| Both, user-toggleable | Offer both formats with a toggle | |

**User's choice:** Inputs+tool+payload → field on ToolResult → cross-surface golden gate → URN + verify on open
**Notes:** Checksum reuses the frozen canonical-JSON/sha256 recipe; parity is proven by a gate, not asserted; citations follow the locked ADR-0111 identity + hard-fail semantics.

---

## Agent's Discretion

- Component library and styling approach within React (subject to shared layer token set + offline constraint).
- Routing/deep-link URL scheme, navigation structure, page composition within the single-column shell.
- SPA build/bundling specifics (Vite config, asset embedding, dev workflow).
- TUI keybindings, palette command inventory, widget decomposition, terminal theme mapping.
- Exact `research_checksum` encoding/length and field naming within the frozen recipe.
- Tool ordering across surfaces, so long as every in-scope tool reaches UI + API + CLI with parity.

## Deferred Ideas

- Real multi-RAG retrieval/routing/reranking + populated RAG debug view — Phase 8.
- Deeper interactive graph explorer beyond navigation/annotation-review — later graph/retrieval work.
- Streaming answers, research-query interface, server dashboard, auth/RBAC — Phases 10–11.
- Hadith/tafsir/isnad UI surfaces — Phases 6–7.
- Comparative-scripture views — Phase 9.
- Frontend niceties (offline PWA install, keyboard-driven power mode, export/print polish) — after core flows land.
