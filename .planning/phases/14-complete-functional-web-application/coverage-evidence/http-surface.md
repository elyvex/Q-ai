# HTTP Surface — Coverage Evidence (read-only collection)

Sources: `crates/server/src/api.rs`, `crates/server/src/routes/scripture.rs`,
`crates/tool-registry/src/lib.rs`, `crates/application/src/quran_tools.rs`,
`crates/cli/src/{quran,hadith,tafsir,scripture,isnad,rag}.rs`.
No endpoints, params, or fields invented — everything below cites `file:line`.

Envelope (all JSON routes): `{api_version:"v1", data:{…}, meta:{edition:{slug,version,text_hash,script,riwayah,numbering_scheme}, corpus_generation, canonical_reference, deep_link, execution_time_ms, reproducibility, research_checksum, warnings}}` (`api.rs:66-98`).
Error body: Phase-0 `Diagnostic` shape `{error:{code,summary,location,why,remedy,next_command}}` (`api.rs:100-122`).
Cross-cutting: `ETag` from `(text_hash, corpus_generation)` (`api.rs:223`), `Cache-Control: public, max-age=3600` + `Content-Language: ar` on canonical payloads (`api.rs:270-287`), 1 MiB body limit / 30 s timeout / concurrency 128 (`api.rs:2283-2288`).

## 1. REST routes (method + path + handler)

Defined in `router()` (`api.rs:2214-2290`) + `routes::scripture::router()` (`scripture.rs:88-98`).

| # | Method | Path | Handler (`api.rs` / `scripture.rs`) | Params (required / optional) | Response `data` key fields |
|---|--------|------|-------------------------------------|------------------------------|----------------------------|
| 1 | GET | `/healthz` | inline `ok` (`api.rs:2216`) | — | `ok` (text) |
| 2 | GET | `/readyz` | inline `ready` (`api.rs:2217`) | — | `ready` (text) |
| 3 | GET | `/api/v1/meta` | inline (`api.rs:2218-2235`) | — | `{name, version}` |
| 4 | GET | `/api/v1/quran/editions` | `editions_handler` (`api.rs:308`) | opt query `status: string`, `language: string` (`api.rs:302-306`) | `[{slug,version,name,script,riwayah,language,verse_numbering_scheme,status,text_hash,statistics}]` (`api.rs:2503-2516`) |
| 5 | GET | `/api/v1/quran/editions/{slug}` | `edition_handler` (`api.rs:338`) | req path `slug` | same edition object as #4 |
| 6 | GET | `/api/v1/quran/surahs` | `surahs_handler` (`api.rs:359`) | opt query `edition` (`api.rs:354-357`; empty = active, `slug` or `slug@version`, `api.rs:2485-2501`) | `[{number,name_arabic,name_transliteration,ayah_count}]` (`api.rs:368-378`) |
| 7 | GET | `/api/v1/quran/surahs/{number}` | `surah_handler` (`api.rs:384`) | req path `number: u16`; opt query `edition` | `{number,name_arabic,ayahs:[arabic_text…]}` (`api.rs:396-400`) |
| 8 | GET | `/api/v1/quran/ayahs/{reference}` | `ayahs_handler` (`api.rs:443`) | req path `reference`; opt query `edition, translations (csv), glosses: bool, tokens: bool` (`api.rs:411-417`) | `[AyahView…]` via `ToolRegistry::get_ayah` (`api.rs:456`, `lib.rs:1028`) |
| 9 | GET | `/api/v1/quran/context/{reference}` | `context_handler` (`api.rs:478`) | req path `reference`; opt query `edition, before=3: u16, after=3: u16, boundary=surah (surah\|juz\|ruku\|page\|none), max_ayahs=11: u16` (`api.rs:469-476,485-497`) | `ContextView` via `get_context` (`api.rs:505`) |
| 10 | POST | `/api/v1/quran/tool/{name}` | `tool_handler` (`api.rs:524`) | req path `name`; req JSON body = tool params object (per-tool) | verbatim serialized `ToolResult`; `meta.research_checksum=sha256:<hex>` (`api.rs:570-584`) |
| 11 | GET | `/api/v1/hadith/{collection}/{source_number}` | `hadith_get_handler` (`api.rs:2310`) | req path `collection, source_number` (joined `collection:source_number`, `api.rs:2314`) | `ToolResult<HadithView>` (`api.rs:2321`) |
| 12 | POST | `/api/v1/hadith/search` | `hadith_search_handler` (`api.rs:2333`) | req JSON `{query: string; collection_slug?: string; limit?: u32}` (`api.rs:2292-2299`) | `ToolResult<Value>` from `search_hadith` |
| 13 | GET | `/api/v1/hadith/{collection}/{source_number}/gradings` | `hadith_gradings_handler` (`api.rs:2359`) | req path `collection, source_number` | `ToolResult<Vec<GradingRecord>>` via `get_gradings(hadith_id)` (`api.rs:2364-2370`) |
| 14 | GET | `/api/v1/tafsir/{work_slug}` | `tafsir_get_handler` (`api.rs:2387`) | req path `work_slug` (verbatim `reference`, `api.rs:2391`) | `ToolResult<TafsirView>` (`api.rs:2397`) |
| 15 | POST | `/api/v1/tafsir/search` | `tafsir_search_handler` (`api.rs:2409`) | req JSON `{query: string; work_slug?: string; limit?: u32}` (`api.rs:2301-2308`) | `ToolResult<Value>` from `search_tafsir` |
| 16 | GET | `/api/v1/verse/{surah}/{ayah}/links` | `verse_links_handler` (`api.rs:2441`) | req path `surah: u16, ayah: u32`; opt query `include_uncertain=false: bool` (`api.rs:2435-2439`) | `[VerseLink…]` (empty array when service absent or error, `api.rs:2447-2460`) |
| 17 | GET | `/api/v1/quran/divisions/{kind}/{number}` | `divisions_handler` (`api.rs:590`) | req path `kind` (`juz\|hizb\|rub\|manzil\|page\|ruku\|sajdah`, `api.rs:2594-2607`), `number: u32`; opt query `edition` | `[{canonical_reference,arabic_text}]` (`api.rs:610-618`) |
| 18 | GET | `/api/v1/quran/tokens/{reference}` | `tokens_handler` (`api.rs:628`) | req path `reference`; opt query `edition` | `[{position,surface,char_start,char_end,byte_start,byte_end,is_pause_mark}]` (`api.rs:639-652`) |
| 19 | GET | `/api/v1/quran/resolve` | `resolve_handler` (`api.rs:664`) | req query `ref: string` (`api.rs:658-662`) | `{canonical_reference,edition_version}` (`api.rs:679-682`) |
| 20 | GET | `/api/v1/quran/citations/{id}` | `citation_handler` (`api.rs:688`) | req path `id` | `ResolvedCitation` (hard-failure verdicts → typed error, never 200, `api.rs:694-703`) |
| 21 | POST | `/api/v1/quran/normalization/preview` | `normalization_preview_handler` (`api.rs:756`) | req JSON `{text: string; profile?: string ("L3.diacritics" default); rules?: string}` — profile XOR rules (`api.rs:718-727,761-764`) | `{input,profile,output,trace,steps}` (`api.rs:786-792`) |
| 22 | GET | `/api/v1/quran/normalization/profiles` | `normalization_profiles_handler` (`api.rs:798`) | — | `{profiles:[…]}` (`api.rs:802-804`) |
| 23 | POST | `/api/v1/quran/search/exact` | `search_exact_handler` (`api.rs:1114`) | req JSON `{text; field?: text_exact\|text_ws; filters?: {surah?: u16[], juz?: "2"\|"1-5", page?: u32[], revelation_place?: makki\|madani, global_range?: "10-99"}; edition?, match_mode?: whole_token\|substring\|ayah_prefix, limit? (cap 1000), offset?, explain?: bool, highlight?: bool}` (`api.rs:818-860`). `Accept: text/event-stream` → SSE (`hit` + terminal `totals` events, `api.rs:1021-1076`) | `SearchOutput {hits,total_matches,truncated,rule_set,generation}` (`api.rs:1069-1074`) |
| 24 | POST | `/api/v1/quran/search/normalized` | `search_normalized_handler` (`api.rs:1142`) | req JSON `{text; profile?: "L3.diacritics[@version]"; rules?; + filters/paging as #23}` (`api.rs:863-875`) | same as #23 |
| 25 | POST | `/api/v1/quran/search/phrase` | `search_phrase_handler` (`api.rs:1170`) | req JSON `{text; profile?; rules?; phrase_mode?: ordered_exact\|ordered_near\|unordered_near; slop?: u32; + filters/paging}` (`api.rs:877-894`) | same as #23 |
| 26 | POST | `/api/v1/quran/search/concatenated` | `search_concatenated_handler` (`api.rs:1206`) | req JSON `{text; allow_cross_ayah?: bool; max_ayah_span?: u32 (≥1, default 3); + filters/paging}` (`api.rs:896-909`) | same as #23 |
| 27 | POST | `/api/v1/quran/search/regex` | `search_regex_handler` (`api.rs:1238`) | req JSON `{pattern; field? (default text_bare); timeout_ms? (default 3000, clamp 1..10000); + filters/paging}`; `x-principal` header (default `api-anonymous`, `api.rs:1250-1255`) | same as #23 |
| 28 | POST | `/api/v1/quran/family` | `lexicon_family_handler` (`api.rs:1461`) | req JSON `{kind: string ("token"); id: string ("token:surah:ayah:pos")}` (`api.rs:1286-1292`) | word-family relations, `meta.reproducibility={tool}` (`api.rs:1443-1450`) |
| 29 | POST | `/api/v1/quran/count/root-frequency` | `lexicon_root_frequency_handler` (`api.rs:1477`) | req JSON `{root: string; profile?: string; mode?: string}` (`api.rs:1294-1302`) | frequency report |
| 30 | POST | `/api/v1/quran/count/lemma-frequency` | `lexicon_lemma_frequency_handler` (`api.rs:1494`) | req JSON `{lemma: string; profile?: string; mode?: string}` (`api.rs:1304-1312`) | frequency report |
| 31 | POST | `/api/v1/quran/count/frequency` | `count_frequency_handler` (`api.rs:1513`) | req JSON `{target: string; profile?="L3.diacritics"}` (`api.rs:1314-1320`) | frequency report |
| 32 | POST | `/api/v1/quran/count/distribution` | `count_distribution_handler` (`api.rs:1525`) | req JSON `{target; profile?}` (`api.rs:1322-1328`) | distribution report |
| 33 | POST | `/api/v1/quran/count/occurrences` | `count_occurrences_handler` (`api.rs:1537`) | req JSON `{target; profile?}` (`api.rs:1330-1336`) | occurrences report |
| 34 | POST | `/api/v1/quran/count/hapax` | `count_hapax_handler` (`api.rs:1549`) | req JSON `{profile?; limit?=100}` (`api.rs:1338-1344`) | hapax report |
| 35 | POST | `/api/v1/quran/count/cooccurrence` | `count_cooccurrence_handler` (`api.rs:1562`) | req JSON `{target; profile?; window?=3; limit?=25}` (`api.rs:1346-1355`) | `{rules,hits}` (`api.rs:1572`) |
| 36 | POST | `/api/v1/quran/count/collocation` | `count_collocation_handler` (`api.rs:1579`) | req JSON `{target; profile?; window?=3; limit?=25}` (`api.rs:1358-1367`) | `{rules,hits}` (`api.rs:1589`) |
| 37 | POST | `/api/v1/quran/count/numeric-report` | `count_numeric_report_handler` (`api.rs:1596`) | req JSON `{target; profile?}` (`api.rs:1369-1376`) | numeric report |
| 38 | POST | `/api/v1/quran/count/missing-form` | `count_missing_form_handler` (`api.rs:1608`) | req JSON `{target; profile?}` (`api.rs:1378-1384`) | missing-form report |
| 39 | POST | `/api/v1/quran/count/near-duplicates` | `count_near_duplicates_handler` (`api.rs:1620`) | req JSON `{threshold?=0.8; limit?=25}` (`api.rs:1386-1392`) | `{rules,hits}` (`api.rs:1629`) |
| 40 | POST | `/api/v1/quran/count/interval` | `count_interval_handler` (`api.rs:1636`) | req JSON `{target; profile?}` (`api.rs:1394-1400`) | interval report |
| 41 | POST | `/api/v1/quran/count/unusual-usage` | `count_unusual_usage_handler` (`api.rs:1648`) | req JSON `{target; profile?}` (`api.rs:1402-1408`) | unusual-usage report |
| 42 | POST | `/api/v1/quran/graph/neighbors` | `graph_neighbors_handler` (`api.rs:1861`) | req JSON `{node: string; projection?="quran-structural-v1"; edge_types?: string[]; direction?: both\|outgoing\|incoming; budgets?: {max_hops,max_nodes,max_edges,max_paths,max_fanout,timeout_ms}}` (`api.rs:1695-1709`); default hops=1 (`api.rs:1868`) | graph output + `meta.reproducibility={tool,projection_id,builder_version,corpus_generation}` (`api.rs:1839-1850`) |
| 43 | POST | `/api/v1/quran/graph/path` | `graph_path_handler` (`api.rs:1902`) | req JSON `{from; to; mode?=paths (reachability\|shortest\|paths); paths?: usize (=max_paths default); edge_types?; direction?; budgets?}` (`api.rs:1711-1730`); default hops=4 (`api.rs:1915`) | same envelope as #42 |
| 44 | POST | `/api/v1/quran/graph/subgraph` | `graph_subgraph_handler` (`api.rs:2001`) | req JSON `{seeds: string[]; edge_types?; direction?; budgets?}` (`api.rs:1732-1743`); default hops=4 | same envelope as #42 |
| 45 | POST | `/api/v1/quran/graph/pattern` | `graph_pattern_handler` (`api.rs:2038`) | req JSON `{seeds: string[]; steps: [{edge, node_kind?}] (1..=8); budgets?}` (`api.rs:1745-1753`) | same envelope as #42 |
| 46 | POST | `/api/v1/quran/graph/root-family` | `graph_root_family_handler` (`api.rs:2074`) | req JSON `{root: string; limit?=25}` (`api.rs:1755-1761`) | output + `reproducibility={tool,dataset,corpus_generation}` (`api.rs:2096-2100`) |
| 47 | GET | `/api/v1/scripture/editions` | `editions_handler` (`scripture.rs:100`) | — | `ToolResult` from `list_scripture_editions` |
| 48 | GET+POST | `/api/v1/scripture/search` | `search_handler_get/post` (`scripture.rs:131,149`) | GET query or POST JSON `{query: string; edition?: string; limit?: u32}` (`scripture.rs:15-22,57-65`) | `ToolResult` from `search_scripture` |
| 49 | GET | `/api/v1/scripture/{edition}/{book}/{chapter}/{verse}` | `get_handler` (`scripture.rs:113`) | req path `edition, book, chapter: u32, verse: u32` (joined `edition:book:chapter:verse`, `scripture.rs:117`) | `ToolResult<ScriptureView>` |
| 50 | GET | `/api/v1/scripture/translations/{verse_id}` | `translations_handler` (`scripture.rs:167`) | req path `verse_id`; opt query `language` (`scripture.rs:45-49`) | `ToolResult` from `get_scripture_translations` |
| 51 | GET+POST | `/api/v1/scripture/parallel` | `parallel_handler_get/post` (`scripture.rs:184,188`) | GET query or POST JSON `{reference: string}` (`scripture.rs:24-28,51-55`) | `ToolResult` from `parallel_passages_scripture` |
| 52 | GET+POST | `/api/v1/scripture/research` | `research_handler_get/post` (`scripture.rs:205,209`) | GET query or POST JSON `{query: string; domains?: "quran,hadith,…"}` (`scripture.rs:30-43`); unknown domains ignored, empty = all (`scripture.rs:67-86`) | `[CrossDomainHit…] {domain,source,reference,snippet,score,confidence}` sorted by score (`scripture.rs:213-282`); `meta.research_checksum="cross-domain:<query>"` |
| 53 | GET+POST | `/api/v1/cross-domain/research` | same `research_handler_*` (`scripture.rs:96`) | same as #52 | same as #52 (alias) |
| 54 | GET+POST | `/api/v1/research/cross-domain` | same `research_handler_*` (`scripture.rs:97`) | same as #52 | same as #52 (alias) |
| D1 | GET | `/debug/read/{edition}/{surah}` | `debug_reader_handler` (`api.rs:2104`) | req path `edition: string, surah: u16` | HTML debug reader (not product UI; out of verification scope) |
| D2 | GET | `/debug/assets/reader.woff2` | `debug_font_handler` (`api.rs:2201`) | — (only mounted by `router_with_debug_font`, `api.rs:2195-2199`) | `font/woff2`, `cache-control: no-store` |

Route count: **54 REST method+path pairs** (46 in `api.rs::router` incl. health/meta/debug + 8 in `scripture.rs::router` counting GET+POST as separate pairs on 4 shared paths) + 1 generic dispatcher row (#10) that multiplexes all tools below.

## 2. Tool-dispatcher table (`dispatch_registered_tool`, `quran_tools.rs:768-990`)

`POST /api/v1/quran/tool/{name}` serves every name below through the same name→method table the CLI `quran tool` verb uses (`quran_tools.rs:759-767`). Unknown names / bad params → typed 4xx, never panic (`quran_tools.rs:774-776,985-988`). Param structs live in `tool-registry/src/lib.rs`.

| Tool name | Params (struct, `lib.rs` line) | Read / Mutating | HTTP-reachable |
|---|---|---|---|
| `quran.get_ayah` | `GetAyahParams {reference: string; translations: string[] = []; glosses: bool = false; tokens: bool = false}` (`lib.rs:549-563`) | read | **yes** (+ dedicated GET #8) |
| `quran.get_context` | `GetContextParams {reference; before: u16 = 3; after: u16 = 3; boundary: ContextBoundaryArg = none (HTTP default surah); max_ayahs: u16 = 11}` (`lib.rs:565-594`) | read | **yes** (+ dedicated GET #9) |
| `quran.search` | `SearchToolParams {text; edition?: string; limit?: u32 (=20, clamp 1..1000)}` (`lib.rs:596-609`); empty text rejected (`lib.rs:1162-1175`) | read | **yes** (generic only; dedicated search routes #23-27 use different param shapes) |
| `quran.root` | `RootToolParams {root: string}` (`lib.rs:611-616`); non-empty required | read | **yes** (generic only) |
| `quran.lemma` | `LemmaToolParams {lemma: string}` (`lib.rs:618-623`); non-empty required | read | **yes** (generic only) |
| `quran.morphology` | `MorphologyToolParams {surah: u16; ayah: u32; position: u32}` (`lib.rs:625-634`); all ≥1 required | read | **yes** (generic only) |
| `quran.family` | `FamilyToolParams {kind: string; id: string}` (`lib.rs:636-643`); non-empty required | read | **yes** (+ dedicated POST #28) |
| `quran.graph_neighbors` | `GraphNeighborsParams {node; projection?: string; edge_types?: string[]; direction?: string; budgets: GraphBudgetsParams}` (`lib.rs:672-691`); node non-empty | read | **yes** (+ dedicated POST #42) |
| `quran.graph_path` | `GraphPathParams {from; to; mode?: string; paths?: usize; edge_types?; direction?; budgets}` (`lib.rs:693-717`); from/to non-empty | read | **yes** (+ dedicated POST #43) |
| `quran.graph_subgraph` | `GraphSubgraphParams {seeds: string[]; edge_types?; direction?; budgets}` (`lib.rs:719-733`); ≥1 non-empty seed | read | **yes** (+ dedicated POST #44) |
| `quran.graph_pattern` | `GraphPatternParams {seeds: string[]; steps: GraphPatternStep[{edge; node_kind?}]}` (`lib.rs:735-755`); seeds + ≥1 step | read | **yes** (+ dedicated POST #45) |
| `quran.graph_root_family` | `GraphRootFamilyParams {root: string; limit?: usize (=25)}` (`lib.rs:757-765`); root non-empty | read | **yes** (+ dedicated POST #46) |
| `hadith.get` | `GetHadithParams {reference: "collection:source_number"}` (`lib.rs:267-272`) | read | **yes** (+ dedicated GET #11) |
| `hadith.search` | `HadithSearchToolParams {query; collection_slug?: string; limit?: u32}` (`lib.rs:274-285`); query non-empty | read | **yes** (+ dedicated POST #12) |
| `hadith.gradings` | body JSON `{hadith_id: string}` (ad-hoc extraction, `quran_tools.rs:857-863`) | read | **yes** (+ dedicated GET #13) |
| `tafsir.get` | `GetTafsirParams {reference: "work_slug[:chapter]"}` (`lib.rs:393-398`) | read | **yes** (+ dedicated GET #14) |
| `tafsir.search` | `TafsirSearchToolParams {query; work_slug?: string; limit?: u32}` (`lib.rs:400-411`); query non-empty (backend guard) | read | **yes** (+ dedicated POST #15) |
| `isnad.narrator` | `GetNarratorParams {id: string}` (`lib.rs:2173-2177`) | read | **yes** (generic; never gated) |
| `isnad.rijal` | `GetNarratorParams {id: string}` (reused, `quran_tools.rs:883-887`) | read | **yes** (generic; never gated) |
| `isnad.trace` | `TraceParams {from: string; to: string; k?: usize}` (`lib.rs:2179-2186`) | read | **yes** (generic; never gated) |
| `isnad.identity_propose` | `IdentityProposalParams {claim: Value; evidence: Value; source_location = ""; provenance_layer: string; algorithm?; algorithm_version?; confidence?}` (`lib.rs:2188-2202`) | **mutating** | gated — 403 `QAI-ISNAD-0326` by default (`api.rs:533-540`) |
| `isnad.identity_accept` | `IdentityDecisionParams {id; reviewer}` (`lib.rs:2204-2209`) | **mutating** | gated — 403 `QAI-ISNAD-0326` by default |
| `isnad.identity_reject` | `IdentityDecisionParams {id; reviewer}` | **mutating** | gated — 403 `QAI-ISNAD-0326` by default |
| `isnad.identity_dispute` | `IdentityDecisionParams {id; reviewer}` (arm `quran_tools.rs:903-912`) | **mutating** | gated — 403 `QAI-ISNAD-0326` by default |
| `isnad.identity_supersede` | body JSON `{id: string; new_proposal: IdentityProposalParams; reviewer: string}` (ad-hoc, `quran_tools.rs:913-923`) | **mutating** | gated — 403 `QAI-ISNAD-0326` by default |
| `rag.create_project` / `rag.create` (alias, `quran_tools.rs:924`) | body JSON `{name: string; config_toml: string}` (ad-hoc, `quran_tools.rs:925-931`) | **mutating** | gated — 403 `QAI-RAG-0326` by default (`api.rs:543-550`) |
| `rag.import` | body JSON `{project: string; docs: Value[]}` (ad-hoc, `quran_tools.rs:933-937`) | **mutating** | gated — 403 `QAI-RAG-0326` by default |
| `rag.query` | body JSON `{project: string; query: string}` (ad-hoc, `quran_tools.rs:938-942`) | read | **yes** (generic; NOT in mutating list, never gated) |
| `rag.debug` | body JSON `{project: string; query: string}` (ad-hoc, `quran_tools.rs:943-947`) | read | **yes** (generic; NOT in mutating list, never gated) |
| `rag.source_status` / `rag.source_activate` / `rag.source_deactivate` / `rag.source_tombstone` | body JSON `{project; source; status?}` — status derived from name for the three aliases (`quran_tools.rs:948-961`) | **mutating** | gated — 403 `QAI-RAG-0326` by default |
| `scripture.get` | `GetScriptureParams {reference: "edition:book:chapter:verse"}` (`lib.rs:481-486`) | read | **yes** (+ dedicated GET #49) |
| `scripture.search` | `ScriptureSearchToolParams {query; edition?: string; limit?: u32}` (`lib.rs:488-499`); query non-empty | read | **yes** (+ dedicated GET/POST #48) |
| `scripture.translations` | `(verse_id: string, language?: string)` — positional, not a params struct (`quran_tools.rs:972-977`) | read | **yes** (+ dedicated GET #50) |
| `scripture.editions` | no params (`quran_tools.rs:978-980`) | read | **yes** (+ dedicated GET #47) |
| `scripture.parallel_passages` | `(reference: string)` — positional (`quran_tools.rs:981-984`) | read | **yes** (+ dedicated GET/POST #51) |

Tool count: **39 dispatched names** in `dispatch_registered_tool` (`quran_tools.rs:780-984`; 12 quran + 3 hadith + 2 tafsir + 8 isnad + 9 rag incl. aliases + 5 scripture). `ToolRegistry::TOOL_NAMES` lists only the 22 non-isnad/non-rag names (`lib.rs:993-1017`); `ISNAD_TOOL_NAMES` lists 6 (omits `identity_dispute`, `identity_supersede`, `lib.rs:2251-2258`); `RAG_TOOL_NAMES` lists 9 (`lib.rs:2314-2324`).
Dedicated-route coverage: 17 of the 39 dispatch names have a dedicated REST route (all except `quran.search/root/lemma/morphology`, isnad ×8, rag ×9); the 13 `count/*` + `family` + graph HTTP routes (#28-46) call lexicon/graph backends directly, NOT through the dispatcher (no `quran.count.*` tool names exist).

## 3. Deny-by-default policy codes

| Code | Gate (`api.rs`) | What it blocks |
|---|---|---|
| `QAI-ISNAD-0326` | `tool_handler` (`api.rs:533-540`, predicate `is_mutating_isnad_tool`, `lib.rs:2267-2276`): `isnad.identity_propose/accept/reject/dispute/supersede` → 403 `tool … mutates narrator-identity state and is disabled on the HTTP route` unless `policy.tool_execution_default == "allow"` (wired via `AppState.allow_isnad_mutations`, `api.rs:172-174`). Read-only `isnad.narrator/rijal/trace` never gated. | narrator-identity writes on the unauthenticated generic tool route; trusted surface is the local `qai isnad …` CLI |
| `QAI-RAG-0326` | `tool_handler` (`api.rs:542-550`, predicate `is_mutating_rag_tool`, `lib.rs:2327-2337`): `rag.create_project/create, rag.import, rag.source_status/activate/deactivate/tombstone` → 403 `tool … mutates RAG state and is disabled on the HTTP route` unless the same `allow` opt-in (shares `allow_isnad_mutations` flag). `rag.query`/`rag.debug` are read-only and NOT gated. | RAG durable-state writes on HTTP |
| (status mapping) | `tool_status` (`api.rs:231-249`): both codes → `403 FORBIDDEN`; `QAI-QUR-0326` (citation access-denied) also → 403; `QAI-QUR-0306/0307/0308/0309/0322/0324/0325` → 404; `QAI-QUR-0321` → 400. | — |

## 4. CLI-mapped capabilities with NO HTTP route

(CLI enums in `crates/cli/src/quran.rs`, `isnad.rs`, `rag.rs`, `scripture.rs`, `hadith.rs`, `tafsir.rs`. "No HTTP route" = neither a dedicated route (§1) nor the generic `POST /api/v1/quran/tool/{name}` dispatcher (§2).)

- **Edition lifecycle** (`quran.rs:63-144`): `quran edition list/show/active/verify`, `import`, `validate`, `catalog`, `diff`, `activate`, `rollback`, `deprecate`, `hashes` — no HTTP route (read-only `list/show/active` also absent; only single-edition GET #5 exists).
- **Corpus management**: `quran translation list/import/show` (`quran.rs:382-396`), `quran gloss import` (`quran.rs:399-406`), `quran forms rebuild` (`quran.rs:410-417`), `quran index rebuild/verify/verify-job/gc/rollback` (`quran.rs:420-461`), `quran morphology import/activate/datasets/diff/affix` (`quran.rs:608-711` — `token/compare/root/lemma` overlap the `quran.morphology/root/lemma` tools via the generic dispatcher, but `import/activate/datasets/diff/affix` do not).
- **Verification/attestation**: `quran verify-quotation` (`quran.rs:146-160`), `quran verify` integrity families (`quran.rs:161-170`), `quran normalize --explain/--list-profiles/--show-rule` introspection flags (`quran.rs:192-210`; base preview IS route #21).
- **Graph build/review/repair** (`quran.rs:739-887+`): `quran graph build/inspect/export/review/propose/suggest/doctor/doctor-repair` — explicitly CLI-only by design ("no HTTP mutation route and no new mutation agent tool", `quran.rs:889-893`); only the five read ops have routes (#42-46).
- **Lexicon browse**: `quran root list` (`quran.rs:465-475`) — no route (only `root-frequency` #29).
- **Count alias**: `quran freq` (`quran.rs:228-237`) — no separate route (same backend as #31, different CLI spelling).
- **Verse-link mutations**: `quran/scripture link suggest/confirm/reject` (`scripture.rs:46-69`, `scripture.rs:188-231`) — no HTTP route; only read list via #16.
- **RAG local project ops**: `qai rag create/import/source` CLI (`rag.rs:10-62`) shells to per-project SQLite files directly, bypassing the HTTP tool route (which additionally gates them under `QAI-RAG-0326`).
- **Isnad CLI parity**: all `qai isnad narrator/rijal/trace/identity …` verbs (`isnad.rs:70-92`) exist on HTTP only through the generic dispatcher (no dedicated isnad routes); the 5 mutating verbs are additionally 403-gated there, so CLI is the sole mutating surface.
- **Hadith/tafsir CLI flags**: `hadith search --profile`, `tafsir search --profile` (`hadith.rs:16-25`, `tafsir.rs:18-25`) are accepted-but-ignored client-side (`_: profile`); HTTP search bodies have no `profile` field either (#12/#15) — parity gap is nil, recorded to avoid inventing a param.
- **Misc CLI**: `qai doctor`, `qai serve`, citation `verify` paths — no HTTP route (citation *read* is #20).
