# Phase 08 — UI Review

**Audited:** 2026-10-19
**Baseline:** 08-UI-SPEC.md (approved design contract)
**Screenshots:** captured (3/3) — Vite dev server on :5173; captures show the default template landing page (Web RAG surface is a stub route, see findings)
**Interaction captures:** off (workflow.ui_interaction_capture is false)

---

## Pillar Scores

| Pillar | Score | Key Finding |
|--------|-------|-------------|
| 1. Copywriting | 2/4 | CLI happy-path copy matches contract, but rag errors lack `[QAI-RAG-NNNN]` + `Remedy:` + `Next:` and Web/TUI surfaces still show Phase-5 "unavailable" stub text |
| 2. Visuals | 2/4 | TUI `RagState::Available` render path is dead code (always `Unavailable`); Web `RagDebugView` is still the Phase-5 stub; declared focal points never render on those surfaces |
| 3. Color | 3/4 | CLI/TUI use terminal default (zero color codes) and Web tokens match UI-SPEC accent `#aa3bff`/`#c084fc`; Web RAG view is unstyled |
| 4. Typography | 3/4 | CLI/TUI use terminal defaults; Web template CSS uses 7 sizes/weight 500 vs declared 4 sizes/2 weights (18/14/24/32, 400/600) — inherited, not Phase-8-specific |
| 5. Spacing | 3/4 | CLI/TUI cell-based per spec; Web template uses off-scale 20px margin and 1126px root width; Phase 8 added no spacing of its own |
| 6. Experience Design | 2/4 | Tombstone `--yes`/`APPROVE` confirmation and CLI empty states work; but Web/TUI RAG surfaces are undelivered, errors lack typed format, and `--json` is ignored on rag success paths |

**Overall: 15/24**

---

## Top 3 Priority Fixes

1. **TUI `RagDebug` always renders `RagState::Unavailable`** — the `Available` variant (and `from_route_decision`) is never constructed, so the declared route trace (Block title `RAG debug`, indented Paragraph rows) never appears; the plan's must-have "no longer Unavailable stub" is unmet — wire a query input/command-palette path that builds `RagState::Available` from a `RouteDecision` (or honestly keep the stub and update UI-SPEC/plan). Files: `crates/tui/src/app.rs:571`, `crates/tui/src/screens/rag_debug.rs`.
2. **Web `RagDebugView` is still the Phase-5 "unavailable" stub** — UI-SPEC declares an upgraded `/rag-debug` route (query form + results + route panel + source status badges); none of it exists; `web/src/views/RagDebugView.tsx` and its test still assert the stub. Either implement the Web surface or amend the contract/plan to CLI+TUI-only. Files: `web/src/views/RagDebugView.tsx`, `web/src/views/RagDebugView.test.tsx`, `web/src/router.tsx`.
3. **rag CLI errors are generic (`error: {msg}`) instead of typed** — UI-SPEC requires `[QAI-RAG-NNNN] <detail>` + `Remedy:` + `Next:` on every failure; `handle_rag_async` only uses `CommandOutput::err` with `GENERIC`/`USAGE` and raw messages, and ignores `json` on success paths (so `--json` produces no JSON for rag commands). Files: `crates/cli/src/rag.rs` (all success branches print directly; error branches use `CommandOutput::err`).

---

## Detailed Findings

### Pillar 1: Copywriting (2/4)

- CLI happy-path copy matches contract: `created {name} at {path}` (`crates/cli/src/rag.rs:133`), `imported {N} document(s) (skipped {M})` (`:178`), `{N} hit(s)` + `{rank}. {doc-id} — {score}` + `route: {router} (checksum {hex})` (`:205-220`), empty `0 hit(s) for "..."` (`:207`), debug trace `route:`/`tools:`/`sources:`/`candidates:`/`merged (RRF):`/`reranked:`/`final:`/`checksum:`/`excluded (non-active):` (`:250-270`), tombstone `tombstoned {source} (audit #{seq})` and refusal `refusing to tombstone without --yes` (`:332`, `:353`). ✔
- BLOCKER: errors are `error: {message}` (via `CommandOutput::err`, `crates/application/src/quran_cli.rs:61-67`) with raw storage messages; no `[QAI-RAG-NNNN]` code, no `Remedy:`, no `Next:`. Only server surface emits a typed code (`QAI-RAG-0326` for mutating tools, `crates/server/src/api.rs:542-550`). Contract row "Error state" violated on CLI.
- WARNING: Web `RagDebugView` still says "RAG not configured / unavailable" and "Retrieval debugging lands in Phase 8" (`web/src/views/RagDebugView.tsx:7-8`), not the Web empty-state copy (`No results for “{query}”.`).
- WARNING: Web/TUI never surface status words (`active`/`deactivated`/`tombstoned`) or the `(s)`-suffix count lines; data model carries them but no surface renders them.
- Minor: `--filter` and `--limit` are accepted but ignored (`crates/cli/src/rag.rs:188` `filter: _`, `:221` `let _ = limit;`).

### Pillar 2: Visuals (2/4)

- BLOCKER: TUI `Screen::RagDebug` always calls `render_rag_debug(frame, &RagState::Unavailable)` (`crates/tui/src/app.rs:571`); `RagState::Available` is never constructed anywhere in the codebase (grep: zero matches). The declared focal point (route summary line) never renders; the Available render path (`crates/tui/src/screens/rag_debug.rs:52-75`) is dead code.
- BLOCKER: Web `/rag-debug` route renders the Phase-5 stub (`web/src/views/RagDebugView.tsx`); the UI-SPEC Web surfaces (project form, import dropzone, query form + results, route panel, source list with status badges) are not implemented. Only nav link exists (`web/src/router.tsx`).
- ✔ CLI focal point is the `{N} hit(s)` count line then ranked hits; debug focal point is `route:` summary line first, then indented trace — matches contract.
- ✔ TUI Available variant (if wired) uses `Block` title `RAG debug` with borders and a `Paragraph` (`crates/tui/src/screens/rag_debug.rs:78`) — matches declared TUI visual.
- TUI query-result table columns (rank 4 / ref Min(10) / score 8 / source 12) are never rendered anywhere; no TUI query surface exists.

### Pillar 3: Color (3/4)

- ✔ CLI and TUI emit zero color codes; terminal default dominates (60% dominant per UI-SPEC).
- ✔ Web tokens match contract: accent `#aa3bff` (light) / `#c084fc` (dark) (`web/src/index.css:7,40`); secondary `--layer-translation-bg: #f1f5f9` (`web/src/tokens/layers.css:12`); destructive red is not used in Phase-8 surfaces.
- ✔ Provenance layer tokens (B/C/D) reused for corpus text; canonical text uses `--layer-canonical-*` (no hardcoded component colors).
- WARNING: Web RagDebugView is unstyled (no `rag-state`/`rag-result` CSS exists anywhere in `web/src`), so it inherits default template styling.
- Minor: Web template ships a `h1` at 56px and uses weight 500; not a color issue but noted under Typography.

### Pillar 4: Typography (3/4)

- ✔ CLI/TUI use terminal defaults only (no custom font), per contract.
- ✔ Web body is 18px/145% (`web/src/index.css:18`), matching declared Body 18px; mono token is `ui-monospace` (`:16`), matching `--mono` requirement.
- WARNING (inherited, pre-existing template): Web CSS declares sizes 56/36/24/20/18/16/15 and weight 500, while UI-SPEC declares 4 sizes (18/14/24/32) and 2 weights (400/600). Display is 32px per spec but template ships 56px; semibold is 600 but template uses 500. Phase 8 added no typography of its own, so this is inherited debt rather than a Phase-8 regression.
- Score numbers/checksums render in `--mono` on Web (`web/src/index.css:100`) and terminal mono on CLI/TUI — matches "never a separate size token".

### Pillar 5: Spacing (3/4)

- ✔ CLI list indent is 2 spaces and remedy indent 6 spaces (none needed for rag; rag uses 2-space trace indent per spec `crates/cli/src/rag.rs`).
- ✔ TUI debug rows indented 2 cells (`crates/tui/src/screens/rag_debug.rs:64-73`), matching "Paragraph rows indented 2 cells".
- ✔ Web grid spec: `gap: var(--space-md)` (16px) between cards; Phase 8 Web adds no layout.
- WARNING (inherited): Web template uses `margin: 20px 0` on `h1` (`web/src/index.css:82`) and `#root { width: 1126px }` (`:54`); 20px is not in the declared scale (4/8/16/24/32/48/64) and 1126px is an arbitrary container width. Phase 8 added no spacing, so inherited.
- No arbitrary `[px]`/`[rem]` values introduced by Phase 8 (grep of Phase-8-touched files: none).

### Pillar 6: Experience Design (2/4)

- ✔ Loading: reads run synchronously to completion with bounded budgets (per UI-SPEC "No loading state by design"); no spinner needed.
- ✔ Empty states on CLI: `0 hit(s) for "..."`, `imported 0 document(s) (skipped 0)`, project/source commands print counts; never a blank panel.
- ✔ Destructive confirmation: tombstone requires `--yes` or interactive `Type APPROVE to tombstone ...` (`crates/cli/src/rag.rs:323-334`); refusal prints `refusing to tombstone without --yes`; success prints `tombstoned {source} (audit #{seq})`. Matches contract.
- ✔ Truncated/partial results never rendered as absence (rag query has no truncation path; none claimed).
- BLOCKER: Web and TUI RAG surfaces are undelivered stubs (see Pillar 2), so empty/error/loading states for those surfaces are not implemented at all.
- BLOCKER: rag errors lack typed code + remedy + next (see Pillar 1), so the error-state contract is not met on the CLI surface that was actually built.
- WARNING: `--json` is ignored on all rag success paths (Query/Debug/Create/Import/Source print directly and never consult the `json` flag), so the declared `CommandOutput { human, json }` cross-surface parity is broken for rag commands.
- Minor: tombstone refusal prints the same `refusing to tombstone without --yes` for both the non-terminal and wrong-input cases; contract distinguishes `refusing to tombstone without --yes` (non-terminal) vs `not approved`. Negligible.

---

## Registry Safety

shadcn not initialized (`shadcn_initialized: false` in UI-SPEC frontmatter; no `components.json`). Registry Safety audit skipped per contract — no third-party registries declared ("third-party: none"). Note: `Registry audit: 0 third-party blocks checked, not required (Tool: none)`.

---

## Files Audited

- `.planning/phases/08-multi-rag-platform/08-UI-SPEC.md`
- `.planning/phases/08-multi-rag-platform/08-CONTEXT.md`
- `.planning/phases/08-multi-rag-platform/08-0{1..6}-SUMMARY.md`
- `.planning/phases/08-multi-rag-platform/08-0{1..6}-PLAN.md`
- `crates/cli/src/rag.rs`
- `crates/tui/src/screens/rag_debug.rs`
- `crates/tui/src/app.rs` (render dispatch)
- `crates/tui/tests/screens.rs`
- `crates/server/src/api.rs` (rag mutating guard)
- `crates/tool-registry/src/lib.rs` (rag tool table + `is_mutating_rag_tool`)
- `web/src/views/RagDebugView.tsx`
- `web/src/views/RagDebugView.test.tsx`
- `web/src/router.tsx`
- `web/src/index.css`
- `web/src/tokens/layers.css`
- `web/src/tokens/themes.css`
- `crates/application/src/quran_cli.rs` (`CommandOutput`)
- `crates/application/src/rag_reader.rs`, `rag_search.rs`, `rag_tools.rs` (spot-checked for truncation/status handling)

Screenshots: `.planning/ui-reviews/08-20261019-082356/` (desktop.png, mobile.png, tablet.png) — default Vite template landing page (Web RAG surface is a stub route).