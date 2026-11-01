# Coverage evidence — tests + Phase 13 history

Collected 2026-11-01 for Phase 14 (`14-SPEC.md`) capability-coverage analysis.
Citations are `file:line` or `(doc §section)`.

## 1. Test-suite inventory

### 1a. Frontend (`web/`) — 20 files / 151 tests, all passing per 13-UI-REVIEW §Pillar 6

Command: `cd web && npx vitest run` (package script `test` = `vitest run`; `web/package.json:8`).
Build gate: `npm run build` (`tsc -b && vite build`; `web/package.json:7`).

| Suite | Covers | Command |
|---|---|---|
| `web/src/router.test.tsx` | ROUTES/DIRECTORY/NAV wiring, `isActive` + `also` ordering, deep-link + citation-id parsing, idle branches | `cd web && npx vitest run src/router.test.tsx` |
| `web/src/views/ReadingView.test.tsx` | Reader states, bookmark toggle/remove, related-lookup block (seed/counts/error/loading/no-prefetch), `research_checksum` fixture shape | same runner, path-adjusted |
| `web/src/views/ResearchView.test.tsx` | Instrument index, query band, service-order hits, typed empty states | same |
| `web/src/views/GraphView.test.tsx` | Ruled query bench, edge ledger, truncation banner | same |
| `web/src/views/RagDebugView.test.tsx` | Sealed RAG state, denied `QAI-RAG-0326` rows, absent-as-absent fields (5 tests) | same |
| `web/src/views/ExplorerView.test.tsx` | Route allow-list presets (fails on invented paths), method toggle, 7 presets | same |
| `web/src/views/WordsView.test.tsx` | Morphology/root/lemma/family + five counts with `rules` blocks, checksums | same |
| `web/src/views/ToolsView.test.tsx` | Catalogue rendering, denied rows, honest-failure block | same |
| `web/src/views/CitationsWorkbenchView.test.tsx` | Verdict panel, tamper simulator (3 tests), idle `EmptyState` (+1 test in audit-fix pass) | same |
| `web/src/views/IsnadView.test.tsx` | trace/narrator/rijal via ToolLedger, inert `QAI-ISNAD-0326` rows (5 tests) | same |
| `web/src/components/ToolLedger.test.tsx` | Shared ToolResult renderer, `research_checksum (server)` line | same |
| `web/src/components/AyahColumn.test.tsx` | Canonical byte-exactness, RTL, Arabic invariants | same |
| `web/src/components/CitationCopy.test.tsx` | Citation parsing, URN/deep-link verbatim, copy-failure surfacing | same |
| `web/src/display/DisplayModeControl.test.tsx` | Size/layout persistence contract | same |
| `web/src/lib/tool-result.test.ts` | ToolResult contract shared by all later views | same |
| `web/src/lib/tools.test.ts` | 27-name callable set, `paramsJson`/`cliCommand`/`httpRequest` derivations, `DENIED` length 12 + code partition (13 tests) | same |
| `web/src/lib/words.test.ts` | `parseWordInput`, `tokenId`, `familyKindFor` | same |
| `web/src/lib/byte-diff.test.ts` | Code-point diff (`Array.from`, empty-string missing side) | same |
| `web/src/lib/reader-storage.test.ts` | Prefs/bookmarks localStorage contract, tolerant per-field fallback | same |
| `web/src/lib/related.test.ts` | Seeded corpus-trace helper, literal-length counts | same |

### 1b. Backend integration tests (`crates/*/tests/`)

General command: `cargo test -p <crate> [--test <name>]`, e.g.
`cargo test -p application --test layer_separation`.
Full gate: `cargo test --workspace` (slow; includes `soak_full`, trycmd goldens).

| Suite | Covers | Command |
|---|---|---|
| `crates/application/tests/` (~40 files: `alpha_e2e`, `alpha_smoke`, `answer_path_ledger`, `canonical_display_identity`, `counting`, `doctor_indexes`, `family_goldens`, `forms_rebuild`, `graph_*` (build/doctor/explain/export/neighbors/paths/review/tools), `index_*` (build/lifecycle/verify), `isnad_*` (identity/reader), `layer_separation`, `morphology_import`, `normalization_seed`, `phase1_foundation`, `phase1_jobs_host`, `quran_*` (catalog/doctor/gloss/identity/import/reader/reference/tools/translation/verification), `rag_search`, `rag_tracer`, `search_*` (cache/goldens/latency/tools), `soak_full`) + `common/` helpers | End-to-end service behaviour, golden vectors, graph/isnad/quran tool paths, answer-path ledger, layer-separation source scan | `cargo test -p application [--test <name>]` |
| `crates/application/tests/layer_separation.rs:140-154` | ADR-0112/D-04/D-12 guard: no translation prop on canonical slot, no canonical class in non-canonical components, layer-token families, no bundled `@font-face url(`, TUI purity | `cargo test -p application --test layer_separation` — **currently FAILS**, see §3 open items |
| `crates/cli/tests/` (`catalog`, `corpus_integrity`, `doctor_json`, `foundation`, `jobs`, `parity_cross_surface`, `quran`, `rag_parity`, `rag_parity_runner`, `*.trycmd`) | CLI surface goldens; `rag_parity.rs:1-9` = Phase 8 cross-surface parity smoke (service leg) | `cargo test -p cli` |
| `crates/cli/tests/quran/*.trycmd` (~25 files: `counting_graph_s1-4`, `edition_identity_s1-2`, `family_s1-2`, `graph_s1-2`, `index_verify`, `normalize_s1-2`, `parity_s0-1`, `read_flow_s1-4`, `reference_s1-6`, `search_s1-2`, `verify_quotation_s1-2`, `verify_s1-2`, `catalog.trycmd`) + `quran/fixtures/` (`editions-mini.json`, `database/`, `normalization/`) | Phase 5 parity fixture suite: CLI transcript goldens over miniature editions | `cargo test -p cli` (trycmd runner) |
| `crates/quran-corpus/tests/` (`adversarial`, `fixtures`, `import_path`) | Corpus import paths, adversarial inputs, fixture loading | `cargo test -p quran-corpus` |
| `crates/quran-core/tests/` (`normalization_vectors`, `reference_grammar`) + `vectors/` | Reference grammar (`surah:ayah` parsing), normalization vectors | `cargo test -p quran-core` |
| `crates/quran-normalization/tests/` (`deterministic_rules`, `golden_harness`, `normalization_pipeline`, `normalization_vectors`, `spanmap_properties`) | Deterministic normalization rules, span-map properties, golden harness | `cargo test -p quran-normalization` |
| `crates/quran-search/tests/` (`fts5_backend`, `regex_dos`, `search_parity`, `trigram_index`) | Search backends parity, regex-DoS guard, trigram index | `cargo test -p quran-search` |
| `crates/quran-graph/tests/` (`conformance`, `no_query_language`, `structural`, `traversal`) | Graph traversal/structural conformance, no-query-language rule | `cargo test -p quran-graph` |
| `crates/quran-morphology/tests/` (`adapters`, `adversarial`, `family_policy`) | Morphology adapters, family policy, adversarial inputs | `cargo test -p quran-morphology` |
| `crates/isnad-graph/tests/conformance.rs` | Isnad graph conformance | `cargo test -p isnad-graph` |
| `crates/evaluation/tests/harness.rs` | Evaluation harness | `cargo test -p evaluation` |
| `crates/domain/tests/primitives.rs` | Domain primitives | `cargo test -p domain` |
| `crates/server/tests/` (`api`, `citation_open`, `graph`) | Axum API: general routes, citation open, graph endpoints (frontend mocks these via Playwright route stubs because no engine runs in CI) | `cargo test -p server` |
| `crates/storage-sqlite/tests/` (`backup_restore`, `commit_bounds_outbox`, `fts5_available`, `generation_monotonicity`, `integrity_audit`, `integrity_provenance`, `integrity_sources`, `outbox_idempotency`, `outbox_relay`, `phase1_jobs_audit`, `quran`, `quran_forms`, `recovery_jobs`, `tombstone_before_visibility`) + `common/` | SQLite storage: migrations, outbox relay/idempotency, integrity audits, FTS5 availability, tombstone-before-visibility | `cargo test -p storage-sqlite` |
| `crates/testkit/src/lib.rs:1-11` + `crates/testkit/tests/` (`config_precedence`, `diagnostics`, `error_codes`, `secret_leak`, `security_archive_guard`, `security_path_guard`, `security_ssrf_guard`, `telemetry_privacy`) | Shared fixture vocabulary (`temp_dir`, `sample_config`, `sample_source_version`, `MockAuditRepo`); its own tests pin security guards (path/SSRF/archive, secret-leak, telemetry privacy) | `cargo test -p testkit` |
| `crates/tool-registry/tests/conformance.rs` | Tool registry conformance (the 27-tool callable set the `/tools` view mirrors) | `cargo test -p tool-registry` |
| `crates/tui/tests/` (`render`, `screens`) | TUI rendering/screens | `cargo test -p tui` |

`research_checksum` (server-computed over tool name/version/params) is asserted
in `web/src/components/ToolLedger.test.tsx:40` and rendered by
`web/src/components/ToolLedger.tsx:27`; the parity claim (UI/CLI/HTTP print the
same checksum) is stated in `web/src/views/ToolsView.tsx:157,242` but only
proven live (D4 open item below).

## 2. Phase 13 settled/decided — DO NOT re-litigate

1. **Forest-green palette (approved by amendment).** `--primary oklch(0.42 0.09 145)`,
   `--accent oklch(0.95 0.018 145)`, `--ring oklch(0.5 0.1 145)`, warm-white ground
   `oklch(0.99 0.004 90)`, warm-charcoal dark. Shipped `web/src/index.css:79-96`
   (`:root`) and `:126-143` (`.dark`) match amended `13-UI-SPEC.md` §3
   token-for-token; 13-UI-REVIEW Pillar 3 went 1/4 → 4/4 by amending the contract,
   not the code (IMPLEMENTATION-REPORT §10; 13-UI-REVIEW §Pillar 3).
2. **Concordance tokens.** Single semantic set in `web/src/index.css` (`@theme` +
   `:root` + `.dark`; 13-UI-SPEC §18); layer contract in `web/src/tokens/layers.css:1-49`
   unchanged — canonical text carries only canonical tokens, backgrounds default
   `transparent`, separation by rules/labels/accent bars. Provenance quartet:
   canon verdigris / trans violet-ink / annot ochre / comp madder
   (`web/src/index.css:102-109`; 13-UI-SPEC §3 table).
3. **9-destination top rail, no sidebar.** `NAV` in `web/src/components/layout/site-header.tsx:5-15`
   (Reader, Search, Research, Hadith, Tafsir, Scripture, Graph, RAG, Explorer) with
   `also` groupings (`/words`+`/tools` under Research, `/isnad` under Hadith,
   `/citations` under Reader); 56 px sticky rail, 86 rem measure, 2 px inset active
   rule, `aria-current="page"` (13-UI-REVIEW §Pillar 2, 4/4). Left sidebar deleted
   (`app-sidebar.tsx` removed; IMPLEMENTATION-REPORT §4).
4. **No-backend-changes constraint.** 13-CONTEXT §Phase Boundary: "The redesign adds
   no backend capability; the Rust backend has no diff beyond defect fixes found
   during the phase." Reaffirmed in IMPLEMENTATION-REPORT §5 ("No backend, API
   contract, schema, Rust toolchain … touched") and §9 ("No `crates/` edits, no new
   dependencies, no invented data"). UI plans may not edit `crates/` — the
   `layer_separation` one-line fix was deliberately left undone for this reason
   (IMPLEMENTATION-REPORT §9, final paragraph).
5. **Denied tools render inert, never callable.** `DENIED` in `web/src/lib/tools.ts:521-534`:
   7 RAG (`rag.create_project/create/import/source_status/source_activate/source_deactivate/source_tombstone`,
   code `QAI-RAG-0326`) + 5 isnad identity mutations (`identity_propose/accept/reject/dispute/supersede`,
   code `QAI-ISNAD-0326`), all with `DURABLE_STATE_REASON` (`tools.ts:517-519`:
   browser never calls durable-state writers; trusted surface is the local CLI).
   Isnad denied rows reuse the catalogue filtered to `isnad.identity_*` (13-14-SUMMARY
   key-decisions). Pinned by `tools.test.ts:95-98` (`DENIED` length 12, code partition).
6. **Registry safety.** 13-UI-SPEC §20: bespoke manual design system, no third-party
   registries, no runtime component installs, no remote CDNs; all primitives vendored
   under `components/ui/`; fonts self-hosted via `@fontsource` (IMPLEMENTATION-REPORT §8
   removed the last unused font package). 13-UI-REVIEW §Registry Safety: no
   `fetch(`/`XMLHttpRequest`/`eval(` in any view/lib file.
7. **Reference rejections stand.** From `docs/ui-redesign/REFERENCE-ANALYSIS.md` and
   13-UI-SPEC §2: no left sidebar, no forest-green/cream/parchment … (note: green
   later *adopted* as the instrument accent by amendment — the rejected item was the
   references' green/parchment *mood*), no gold ornament, no card-per-item lists, no
   ambient orbs, no landing-hero compositions implying unshipped API features.
   References are evidence, never a base (IMPLEMENTATION-REPORT §2).
8. **Copy/contract strings.** 13-UI-SPEC §19 exact strings ("No reference is open",
   "no citation is open", "Run instrument", "Dispatch", …); 13-UI-REVIEW Pillar 1
   3/4 with only pre-existing per-tool CTA deviations outstanding.

## 3. Open items (Phase 14 must close or explicitly carry)

- **Live-engine D4/D5 items** (jsdom stubs `fetch`; each needs a running Rust engine
  and/or served build):
  - 13-10 D4: submit `/rag-debug` against a live engine with a configured RAG backend.
  - 13-11 D2 (live morphology submit for 2:1:3) + D5 (cold-load `/words` in served build).
  - 13-12 D4: `quran.get_ayah 2:1` in browser prints the same `research_checksum` the CLI prints (the UI/CLI/HTTP parity proof); D5 (cold-load `/tools`).
  - 13-13 D4: served-build persistence (size/layout across reload), bookmarks, seeded related lookup.
  - 13-14 D4: `/citations?citation=<id>` byte-identical baseline + one-char tamper against a live engine; D5: `/isnad` trace with inert denied rows.
  - 13-UI-REVIEW §Pillar 6 notes these as open human-judgment items, "not defects".
- **Audit minors** (13-UI-REVIEW §§Top 3 / Pillars 1,5,6 — code-hygiene, no task breakage):
  1. Instrument-index widths off-contract: `ToolsView.tsx:161` (`18rem`) and
     `ExplorerView.tsx:64` (`14rem`) vs UI-SPEC §6 13-rem index (compliant: `ResearchView.tsx:156`, `SearchView.tsx:60`, `TafsirView.tsx:57`).
  2. CTA deviations from §19: `ExplorerView.tsx:140` "Dispatch request" (contract "Dispatch");
     per-tool "Run \<name\>" (contract "Run instrument"); raw "awaiting dispatch" text
     (`ExplorerView.tsx:174`) instead of typed `EmptyState`.
  3. Fractional off-scale utilities (`gap-2.5`, `px-3.5`/`py-2.5`, `size-3.5`, `gap-0.5`)
     outside the §5 {4,8,16,24,32,48} scale + dead `web/src/tokens/themes.css`
     (pre-Concordance hardcoded hex, unreferenced under `next-themes` class strategy).
- **`layer_separation` FAILURE (pre-existing, blocks `cargo test -p application`).**
  `spa_non_canonical_components_wear_no_canonical_class`
  (`crates/application/tests/layer_separation.rs:148-154`) does
  `read_to_string("web/src/components/WordInspector.tsx").expect("component exists")`,
  but that file was deleted in `f86be81` (IMPLEMENTATION-REPORT §8: fake-`•` inspector
  replaced by honest standby; type moved into `DisplayModeControl`). Fix is one line
  (drop the deleted path) but lives in `crates/`, so Phase 13 left it undone per the
  no-backend-changes constraint. Phase 14 owns the fix.
- **Known limitations** (IMPLEMENTATION-REPORT §7): reader landing prefill
  `/read/test@0.1.0/2:1` resolves only with that fixture edition loaded; command
  palette has no content-index endpoint to search; RAG/Words/Tools/Citations/Isnad
  without an engine render typed empty/error states by design.
