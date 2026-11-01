# Q-ai Capability-to-UI Coverage Matrix

**Created:** 2026-11-01 · **For:** Phase 14 planning (`14-SPEC.md` R1–R14)
**Method:** static evidence only — no invented endpoints, params, fields, or capabilities. Every row cites `file:line`.
**Evidence:** `coverage-evidence/{cli-surface,http-surface,frontend-surface,tests-and-history}.md` (4 collector agents, same directory).

## Status legend

| Code | Meaning |
|---|---|
| ✅ FULL | Fully implemented and accessible — real backend, working UI, states covered |
| 🟡 PARTIAL | Implemented but partially exposed — UI exists but drops sub-features or renders raw transcript |
| 🔴 EXPOSED-0 | Implemented but not exposed — backend exists, zero UI surface |
| ⛔ NO-BE | Backend missing — no HTTP route (and, where noted, no CLI either); needs new backend work |
| ⏸ DEFER | Intentionally deferred — CLI-only by design (approval-gated / owner / operator-local actions) |
| 🛑 BLOCK | Blocked by an architectural/technical limitation — cannot be a browser feature as specified |

**Integration strategies used in the last column:**
- **UI** = build/extend a dedicated UI surface (no backend change)
- **TOOLS** = already covered by the generic `/tools` runner; no dedicated view needed
- **CLI-REF** = show in the CLI reference view with copyable commands (browser cannot spawn processes)
- **NEW-HTTP** = needs a new HTTP route first (Phase 14 backend scope: SPEC R1–R5)
- **CLI-ONLY** = stays a terminal operator action (destructive, approval-gated, or localhost-local)

## Summary counts

| Status | Count | Where |
|---|---|---|
| ✅ FULL | 12 surfaces | Reader, citation overlay, search (modes), research (9 instruments), words (morphology+lexicon+5 counts), tools (27 callable), citations workbench, isnad reads, graph (neighbors+path), rag-debug reads, explorer, directory landing |
| 🟡 PARTIAL | 8 | Search filters/streaming, hadith structured, tafsir structured, scripture parallel/verse/translations, graph subgraph/pattern/root-family dedicated views, cross-domain multi-domain, count `collocation`-et-al (counted under EXPOSED-0 below), research hardening simplifications (documented, not gaps) |
| 🔴 EXPOSED-0 | 19 | context, surahs, divisions, tokens, resolve (dead client fn), editions mgmt reads, normalization UI, 8 count instruments, verse links, meta/system, root-list browse, morphology diff/affix/datasets views, graph export UI, verify-quotation UI, hadith gradings view, tafsir get-by-ref view, scripture get/verse/translations views |
| ⛔ NO-BE | 9 groups | ask/agent run+trace, answer contract, tool lifecycle, source catalog states, ops (jobs/audit/budgets/sessions/doctor HTTP), verse-link mutations HTTP, RAG project-ops HTTP, models/routing management, secret/source-import stubs |
| ⏸ DEFER (CLI-ONLY) | 10 groups | edition activate/rollback/deprecate/verify, translation/gloss import, forms rebuild, index rebuild/gc/rollback/verify-job, morphology import/activate, graph build/review/repair, link confirm/reject, db migrate/restore/backup, serve/config, secret set/delete refusals |
| 🛑 BLOCK | 1 | Executing CLI/`qai` processes from the browser (all CLI exposure is display+copy only) |

---

## A. Quran text, reference, editions

| # | Capability | Backend location | CLI | HTTP | Params | Response | UI route/component | Exposed? | Functional? | Missing UI | Backend? | Status + strategy |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| A1 | Get ayah (full opts: translations, glosses, tokens) | `tool-registry` get_ayah; `api.rs:443` | `quran get` (`quran.rs:16`) | GET `/quran/ayahs/{ref}` + tool `quran.get_ayah` | ref; opt edition, translations csv, glosses, tokens | `AyahView[]` + envelope meta/checksum | `/read/…` ReadingView; ToolsView | yes | yes (tokens opt never requested — morphology used instead; cosmetic) | none blocking | yes | ✅ FULL — UI |
| A2 | Context window | `get_context`; `api.rs:478` | `quran context` (`quran.rs:30`) | GET `/quran/context/{ref}` + tool `quran.get_context` | ref; before/after=3, boundary, max_ayahs=11 | `ContextView` | none | no | — | context lookup section | yes | 🔴 EXPOSED-0 — UI (SPEC R7) |
| A3 | Surah browse + metadata | `api.rs:359,384` | `quran surah` (`quran.rs:44`) | GET `/quran/surahs`, `/quran/surahs/{n}` | opt edition; number | `[{number,name_arabic,name_transliteration,ayah_count}]`; surah `{number,…,ayahs[]}` | none | no | — | surah/edition browser | yes | 🔴 EXPOSED-0 — UI (SPEC R7) |
| A4 | Divisions (juz/hizb/rub/manzil/page/ruku/sajdah) | `api.rs:590` | `quran division` (`quran.rs:52`) | GET `/quran/divisions/{kind}/{n}` | kind+number; opt edition | `[{canonical_reference,arabic_text}]` | none | no | — | division lookup | yes | 🔴 EXPOSED-0 — UI (SPEC R7) |
| A5 | Token stream (surface/offsets) | `api.rs:628` | — (via `morphology token`, `quran.rs:654`) | GET `/quran/tokens/{ref}` | ref; opt edition | `[{position,surface,char/byte_start/end,is_pause_mark}]` | none (WordsView uses `quran.morphology` tool instead) | no | — | token inspector data | yes | 🔴 EXPOSED-0 — UI (SPEC R7; complements morphology) |
| A6 | Reference resolution | `api.rs:664` | `quran resolve` (`quran.rs:59`) | GET `/quran/resolve?ref=` | ref | `{canonical_reference,edition_version}` | none — `resolveReference` (`client.ts:97`) has **zero callers**; `ReadingView onResolve` stubbed with `void` | no | — (dead client fn) | wire into ReferenceForm/reader | yes | 🔴 EXPOSED-0 — UI (small fix; dead-code hookup) |
| A7 | Edition mgmt reads (list/show/active) | `api.rs:308,338` | `quran edition list/show/active` (`quran.rs:353-364`) | GET `/quran/editions`, `/quran/editions/{slug}` | opt status, language | edition objects incl. `text_hash,statistics` | none | no | — | edition browser | yes | 🔴 EXPOSED-0 — UI (SPEC R7) |
| A8 | Edition mutations (import/validate/catalog/diff/activate/rollback/deprecate/hashes/verify; translation/gloss import; forms rebuild; index rebuild/verify-job/gc/rollback; morphology import/activate) | `quran_cli` + jobs | `quran import/activate/rollback/deprecate/…` etc. (`quran.rs:70-146,387-461,610-652`) | none (no HTTP) | — | — | none (CLI-REF only) | no | — | read-only ones → CLI-REF; mutating stay terminal | partial (CLI) | ⏸ DEFER — CLI-ONLY (approval-gated owner/operator actions; `confirm()`/`--yes`) |
| A9 | Normalization preview + profiles | `api.rs:756,798` | `quran normalize` subset (`quran.rs:194`) | POST `/quran/normalization/preview`; GET `/quran/normalization/profiles` | text+profile XOR rules | `{input,profile,output,trace,steps}`; `{profiles[]}` | none | no | — | normalization workbench (preview must never alter canonical display) | yes | 🔴 EXPOSED-0 — UI (SPEC R7) |
| A10 | Verse-link reads (+ suggest) | `api.rs:2441`; `VerseLinkService` | `quran/scripture link list|suggest`, `links verse` (`scripture.rs:48-69`, `links.rs:12`) | GET `/api/v1/verse/{s}/{a}/links` | surah, ayah; opt include_uncertain | `[VerseLink…]` (empty when service absent) | none (reader uses seeded tafsir/hadith search instead) | no | — | verse-link panel on reader | yes (reads) | 🔴 EXPOSED-0 — UI (SPEC R7); suggest has no HTTP → CLI-REF; confirm/reject owner-gated MUT → ⏸ DEFER CLI-ONLY |

## B. Search (5 lexical modes)

| # | Capability | Backend location | CLI | HTTP | Params | Response | UI route/component | Exposed? | Functional? | Missing UI | Backend? | Status + strategy |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| B1 | Search all 5 modes | `api.rs:1114-1238` | `quran search --exact/--phrase/--concatenated/--regex` (`quran.rs:212`) | POST `/quran/search/{exact,normalized,phrase,concatenated,regex}` | text/pattern + filters (surah/juz/page/revelation-place/range), match_mode, phrase_mode/slop, cross_ayah, explain/highlight, limit/offset, timeout | `SearchOutput{hits,total_matches,truncated,rule_set,generation}`; SSE stream supported | `/search` SearchView (text only, 5 mode buttons) | yes | yes for basic; filters/paging/SSE unsent | filter panel, limit/offset pagination, explain/highlight toggles | yes | 🟡 PARTIAL — UI (SPEC R14 per-route filtering/sorting/pagination) |
| B2 | Generic `quran.search` tool | `tool-registry` (`lib.rs:596`) | via `quran tool` | POST `/quran/tool/quran.search` | text, edition?, limit? | ToolResult | ToolsView | yes | yes | none | yes | ✅ FULL — TOOLS (dedicated routes cover the rest) |

## C. Linguistics (roots, lemmas, morphology, families)

| # | Capability | Backend location | CLI | HTTP | Params | Response | UI route/component | Exposed? | Functional? | Missing UI | Backend? | Status + strategy |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| C1 | root / lemma / morphology / family tools | `tool-registry` (`lib.rs:611-643`) | `quran lemma/family`, `morphology root/lemma/token/compare` | POST `/quran/tool/quran.{root,lemma,morphology,family}` (+ dedicated POST `/quran/family`) | root / lemma / {surah,ayah,position} / {kind,id} | occurrence rows, paradigm/family relations, ToolResult+checksum | `/words` WordsView | yes | yes (research_checksum, deep links, rules) | none blocking | yes | ✅ FULL — UI |
| C2 | Root list browse | `cmd_morphology_root_list` (`quran.rs:467`) | `quran root list --prefix --limit` | none | — | root strings | none | no | — | browsable root index | partial (CLI only) | 🔴 EXPOSED-0 — CLI-REF now; dedicated view needs NEW-HTTP |
| C3 | Morphology diff / affix / datasets | `quran.rs:652-690` | `morphology diff/affix/datasets` | none | — | diff text / affix analyses / dataset list | none | no | — | dataset picker, affix lookup | partial (CLI only) | 🔴 EXPOSED-0 — CLI-REF now; views need NEW-HTTP |
| C4 | `quran freq` alias | `quran.rs:228` | `quran freq` (alias of frequency) | (same as D1) | target, profile | same as D1 | WordsView counts | yes | yes | none | yes | ✅ FULL (alias, no separate work) |

## D. Count instruments (13 rule-relative reports)

| # | Capability | Backend location | CLI | HTTP | Params | Response | UI route/component | Exposed? | Functional? | Missing UI | Backend? | Status + strategy |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| D1–D5 | frequency, distribution, root-frequency, lemma-frequency, cooccurrence | `api.rs:1477-1572` | `quran count …` (`quran.rs:481-528`) | POST `/quran/count/{…}` | target/root/lemma; profile; window/limit | frequency/distribution/`{rules,hits}` reports | `/words` counts section (rules block + by-surah + checksum; proven-zero state) | yes | yes | none | yes | ✅ FULL — UI |
| D6–D13 | occurrences, hapax, collocation, numeric-report, missing-form, near-duplicates, interval, unusual-usage | `api.rs:1537-1648` | `quran count …` (`quran.rs:497-604`) | POST `/quran/count/{…}` (same shapes; hapax/near-duplicates take limit/threshold) | target (+limit/threshold) | respective reports | none (not in `client.ts`, not in WordsView) | no | — | 8 count sections mirroring D1–D5 | yes | 🔴 EXPOSED-0 — UI (SPEC R6; no backend change) |

## E. Graph (structural projection over active edition)

| # | Capability | Backend location | CLI | HTTP | Params | Response | UI route/component | Exposed? | Functional? | Missing UI | Backend? | Status + strategy |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| E1 | neighbors + path | `api.rs:1861-1915` | `quran graph neighbors/path` (`quran.rs:756-804`) | POST `/quran/graph/{neighbors,path}` | node / from+to+mode; budgets, hops | graph output + reproducibility (projection/builder/generation) | `/graph` GraphView (defaults, budgets fixed) | yes | yes (budget controls absent — acceptable simplification) | budget inputs (optional) | yes | ✅ FULL — UI |
| E2 | subgraph / pattern / root-family | `api.rs:2001-2096` | `quran graph subgraph/pattern/root-family` (`quran.rs:806-833`) | POST `/quran/graph/{subgraph,pattern,root-family}` | seeds[]; steps[] (1..8); root+limit | same envelope family | ToolsView generic forms only | partial (typed but unshaped JSON) | works, poor UX | dedicated views: subgraph export (JSON/DOT/SVG), pattern step builder, root-family tree | yes | 🟡 PARTIAL — UI (SPEC R8) |
| E3 | graph export file | `cmd_graph_export` (`quran.rs:841`) | `quran graph export --format json/dot/svg` | none (client-side derivable from E2 reads) | — | file write (CLI) | none | no | — | export buttons on subgraph view | yes (via E2) | 🔴 EXPOSED-0 — UI (SPEC R8; serialize client-side) |
| E4 | graph build / inspect / doctor | `quran.rs:741-873` | `quran graph build/inspect/doctor` | none | — | projection health | none | no | — | operator visibility | partial (CLI) | ⏸ DEFER — CLI-ONLY (projection build persists; operator action) unless ops scope extends |
| E5 | graph annotation review (propose/suggest/accept/reject/correct) | `quran_graph_annotations` (`quran.rs:895-1031`) | `quran graph review …` (5 verbs) | none — explicitly no HTTP route by design (`quran.rs:889-893`) | — | pending/accepted rows | none | no | — | review queue UI | partial (CLI) | ⏸ DEFER — CLI-ONLY by design (layer-B/D writes; needs backend scope decision to expose) |
| E6 | graph doctor-repair (quarantine/gc/rebuild) | `quran_graph_doctor` (`quran.rs:1041-1065`) | `quran graph doctor-repair …` (3 verbs, `confirm()`+audit) | none | — | repair effects | none | no | — | operator repair UI | partial (CLI) | ⏸ DEFER — CLI-ONLY (approval-gated, destructive) |

## F–H. Hadith, tafsir, scripture (transcript shells)

| # | Capability | Backend location | CLI | HTTP | Params | Response | UI route/component | Exposed? | Functional? | Missing UI | Backend? | Status + strategy |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| F1 | Hadith get + search | `hadith_reader/search` | `hadith get/search` (`hadith.rs:14-19`) | GET `/hadith/{c}/{n}`; POST `/hadith/search` | collection+number; query+collection_slug+limit | ToolResult HadithView / values | `/hadith` (raw `pre` transcript) | yes | works, unstructured | structured matn/isnad/grading rendering | yes | 🟡 PARTIAL — UI (SPEC R14) |
| F2 | Hadith gradings | `backend_get_gradings` | `hadith gradings` (`hadith.rs:24`) | GET `/hadith/{c}/{n}/gradings` (+ tool) | collection+number | `[GradingRecord]` | ToolsView only | partial | works, undiscoverable | gradings panel on Hadith view | yes | 🔴 EXPOSED-0 — UI (small; SPEC R14) |
| G1 | Tafsir get + search | `tafsir_reader/search` | `tafsir get/search` (`tafsir.rs:14-19`) | GET `/tafsir/{work}`; POST `/tafsir/search` | work_slug; query+work_slug+limit | ToolResult TafsirView / values | `/tafsir` (raw `pre` transcript) | yes | works, unstructured | passage-by-reference form; parallel-column disagreements | yes | 🟡 PARTIAL — UI (SPEC R14) |
| H1 | Scripture editions/search/research | `scripture_*` | `scripture editions/search/research` (`scripture.rs:12-43`) | GET+POST `/scripture/{editions,search,research}` (+ `cross-domain` aliases) | query/edition/limit; domains csv | ToolResult / `[CrossDomainHit]` scored | `/scripture` (raw `pre`; research pinned to `domains:'scripture'`) | yes | works, unstructured | edition/verse pickers; multi-domain research; structured rendering | yes | 🟡 PARTIAL — UI (SPEC R14); cross-domain full use = small extension |
| H2 | Scripture get / translations / parallel | `scripture_reader/tools` | `scripture get/translations` (`scripture.rs:14-26`) | GET `/scripture/{e}/{b}/{c}/{v}`, `/scripture/translations/{id}`, `/scripture/parallel` (+ tools) | edition:book:chapter:verse; verse_id+language; reference | ToolResult views | ToolsView + Explorer presets only | partial | works, undiscoverable | verse fetch + translations + parallel-columns UI (lede promises parallel — unfulfilled) | yes | 🔴 EXPOSED-0 — UI (SPEC R7/R14) |
| H3 | Dead control: `edition` state/input | — | — | — | — | — | `ScriptureView.tsx:8,72-75` (no consumer) | — | **dead** | wire to H2 verse fetch or delete | — | Fix in SPEC R14 pass (wire or remove) |

## I–L. Links, citations, isnad, RAG policy boundary

| # | Capability | Backend location | CLI | HTTP | Params | Response | UI route/component | Exposed? | Functional? | Missing UI | Backend? | Status + strategy |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| I1 | Verse-link reads | `VerseLinkService` (`api.rs:2441`) | `links verse`, `scripture link list` | GET `/verse/{s}/{a}/links` | surah, ayah, include_uncertain | `[VerseLink]` (empty when absent) | none | no | — | verse-link panel (reader margin) | yes | 🔴 EXPOSED-0 — UI (SPEC R7) |
| J1 | Citation read + verify | `citations` crate (`api.rs:688`) | `quran verify-quotation` (`quran.rs:145`, R/O) | GET `/quran/citations/{id}` (hard-fail on mismatch, never 200) | id | `ResolvedCitation` + envelope meta | `/citations` workbench + `?citation=` overlay | yes | yes (baseline diff, tamper sim, counter) | CLI verify-quotation → CLI-REF entry | yes | ✅ FULL — UI (+ CLI-REF for the CLI spelling) |
| K1 | Isnad trace/narrator/rijal | `isnad_reader/tools` | `isnad narrator/rijal/trace` (`isnad.rs:72-84`) | POST `/quran/tool/isnad.{trace,narrator,rijal}` (never gated) | from/to/k; id | ToolResult via ledger | `/isnad` (3 instruments, disabled-with-reason) | yes | yes (read-only by design) | none | yes | ✅ FULL — UI |
| K2 | Isnad identity propose/accept/reject/dispute/supersede | `tool_registry::IsnadBackend` | `isnad identity …` (`isnad.rs:13-69`, reviewer/`--yes` gates) | 403 `QAI-ISNAD-0326` unless policy allow (`api.rs:533-540`) | claim/evidence/layer (+reviewer) | 403 Diagnostic by default | inert `aria-disabled` rows (`IsnadView`, ToolsView DENIED) | sealed (policy) | n/a (intentional) | none — CLI is the surface | gated | ⏸ DEFER — CLI-ONLY (durable identity writes; UI lists them honestly) |
| L1 | RAG query/debug | `RagToolBackend` | `rag query/debug` (`rag.rs:26-43`) | POST `/quran/tool/rag.{query,debug}` (never gated) | project, query | payload + route decision + `research_checksum` | `/rag-debug` observer (mode seg, decision block, absent-as-absent) | yes | yes (read-only) | live-backend proof (SPEC R12 E2E D4) | yes | ✅ FULL — UI |
| L2 | RAG project ops (create/import/source activate/deactivate/tombstone) | `SqliteRagReader` (per-project `index.db`) | `rag create/import/source …` (`rag.rs:13-62`) | 403 `QAI-RAG-0326` unless allow (`api.rs:542-550`) — **Update 1 resolved: SPEC R15 owns new HTTP lifecycle routes + project manager view** | project(+docs glob/source) | 403 Diagnostic by default | inert rows, no forms | sealed (policy) until R15 lands | gated → R15 routes | ⏸ DEFER-UNTIL-R15 — CLI-ONLY today (durable state; inert rows + CLI pointer when denied; audit per mutation once live) |

## M–Q. Backend-missing groups (Phase 14 backend scope per SPEC R1–R5)

| # | Capability | Backend location | CLI | HTTP | Params | Response | UI route/component | Exposed? | Functional? | Missing UI | Backend? | Status + strategy |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| M1 | Ask / agentic research run + trace + cancel | none (agents runtime exists, no HTTP) | none (TUI `qai tui` reads same backends; no ask verb) | none | — | — | none | no | — | Ask view (progress, evidence, cancel, claim citations) | **no** | ⛔ NO-BE — NEW-HTTP then UI (SPEC R1–R2) |
| M2 | Answer contract (sections, claim citations, side-by-side) | none | none | none | — | — | none (RAG Debug shows raw payloads) | no | — | structured answer renderer with re-verify per claim | **no** | ⛔ NO-BE — NEW-HTTP then UI (SPEC R2) |
| N1 | Tool lifecycle (describe→review→generate→test→security-review→approve→publish→revoke) | none | none | none | — | — | none (`/tools` runs existing tools only) | no | — | Tool Studio view with per-step gates | **no** | ⛔ NO-BE — NEW-HTTP then UI (SPEC R3) |
| O1 | Source catalog states + trust profiles | `application::db::{list_sources,get_source}` (reads only) | `source list/show` (R/O); `source import` = USAGE refusal stub (`lib.rs:1250`) | none | — | — | none | no | — | Catalog view (pipeline cards, diff, rollback, trust) | partial (CLI reads) | ⛔ NO-BE — NEW-HTTP then UI (SPEC R4) |
| P1 | Jobs / audit / budgets / sessions / doctor HTTP | `db::{list_jobs,…}`, `job_queue`, `doctor::*` | `job list/show/cancel/retry`, `audit list/verify`, `doctor [--repair-preview]` | none | — | — | none | no | — | Operations view (tables, cancel, repair actions) | partial (CLI) | ⛔ NO-BE — NEW-HTTP then UI (SPEC R5) |
| Q1 | Local operator chassis (`serve`, `config`, `db migrate/status/verify/plan/backup/restore`, `secret list`, `status/version`) | `application::db`, config, server | full (`lib.rs:54-223`) | `/healthz`, `/readyz`, `/meta` only | — | — | none (`/meta` unsurfed; `/explorer` can hit it manually) | no | — | System/about panel fed by `/meta` (tiny); rest stays terminal | partial | 🔴 EXPOSED-0 for `/meta` — UI (SPEC R7, one small view); rest ⏸ DEFER — CLI-ONLY (localhost-local, some destructive/approval-gated) |

## R. Explicit non-capabilities (do not plan UI for these)

| # | Item | Why not |
|---|---|---|
| R1 | Executing `qai …` from the browser | 🛑 BLOCK — browsers cannot spawn processes; all CLI exposure is display + copy (established Phase 13, SPEC R9) |
| R2 | TUI screens (`qai tui`) | Out of scope — separate surface (SPEC boundary; REQ-tui-cli stays separate) |
| R3 | `secret set/delete`, `source import` | Refusal stubs in CLI (`lib.rs:1387-1394,1250`) — no backend to expose; not UI work |
| R4 | `hadith/tafsir search --profile`, `rag query --filter/--limit` | Accepted-but-ignored flags (`_: profile`); HTTP has no such params — parity gap is nil, do not invent params |
| R5 | Stale `completions` table (12/19 commands) | Minor CLI hygiene, not a capability; fix opportunistically, never a UI task |
| R6 | `quran search` SSE streaming | Backend supports `Accept: text/event-stream` (`api.rs:1021-1076`); UI uses unary JSON — viable R14 upgrade, not a gap in capability |

---

## Recommended build order (cheapest honest coverage first)

1. **Zero-backend wins (UI only):** D6–D13 (8 counts), A2–A5+A7+A9 (context/surahs/divisions/tokens/editions/normalization), I1 (verse links), Q1-meta view, F2/H2 panels (gradings, get/translations/parallel), E3 export buttons, B1 filters/pagination, H3 dead-control fix, A6 resolve hookup.
2. **Small defect + hygiene:** layer_separation one-liner (SPEC R11), audit minors (SPEC R10).
3. **New HTTP + UI (SPEC R1–R5):** M1–M2 (ask/answer), N1 (tool lifecycle), O1 (catalog), P1 (ops) — each needs route design + policy review before any screen.
4. **Proof:** E2E suite + CI job (SPEC R12), axe/Lighthouse gates + screenshots (SPEC R13), reference-driven per-route upgrades (SPEC R14).

*14-SPEC.md coverage check: every SPEC R1–R15 requirement maps to ≥1 row above (L2 now owned by R15; Update 1 closed the gap).*
