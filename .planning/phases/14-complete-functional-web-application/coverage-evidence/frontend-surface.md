# Frontend coverage surface — evidence (web/src only)

Source of truth: `web/src/router.tsx`, `web/src/api/client.ts`, `web/src/views/*.tsx`,
`web/src/lib/tools.ts`, `web/src/lib/words.ts`. All citations are file:line in that tree.
No routes or calls invented. Status labels: **functional** (real backend call, renders
parsed/verbatim result, has empty/loading/error states), **partial** (real call but raw
transcript or missing sub-feature), **placeholder/sealed** (no call possible by design).

## Route inventory

`ROUTES` (`router.tsx:111-125`) declares 13 entries; `DIRECTORY` (`router.tsx:84-109`)
lists 12. `App.tsx:11-12` passes `window.location.pathname + window.location.search`
as `path`, so query strings (`?citation=`) are part of routing. Effective route count:
**13 static routes + 1 deep-link pattern (`/read/:edition/:ref`) + 1 query overlay
(`?citation=<id>`) = 15 addressable surfaces.**

Deep-link parsing: `parseReadPath` (`router.tsx:22-27`) accepts only
`/read/<edition>/<surah>:<ayah>`; `parseCitationId` (`router.tsx:30-35`) reads
`?citation=<id>`; `ReferenceForm` validates against `DEEP_LINK` (`router.tsx:39`)
and navigates via `window.location.assign` (`router.tsx:53`).

## Typed client surface (`web/src/api/client.ts`)

- `fetchAyahs(reference)` → `GET /api/v1/quran/ayahs/{reference}` (`client.ts:92-94`)
- `resolveReference(reference)` → `GET /api/v1/quran/resolve?reference=` (`client.ts:97-99`)
- `fetchCitation(id)` → `GET /api/v1/quran/citations/{id}` (`client.ts:102-104`)
- `runTool(name, params)` → `POST /api/v1/quran/tool/{name}` (`client.ts:107-109`)
- `ragQuery(project, query)` → `runTool('rag.query', …)` (`client.ts:112-114`)
- `ragDebug(project, query)` → `runTool('rag.debug', …)` (`client.ts:117-119`)
- `search(mode, query)` → `POST /api/v1/quran/search/{exact|normalized|phrase|concatenated|regex}` (`client.ts:126-128`); `SearchMode` (`client.ts:123`)
- `family(kind, id)` → `POST /api/v1/quran/family` (`client.ts:131-133`)
- `rootFrequency(root)` → `POST /api/v1/quran/count/root-frequency` (`client.ts:136-138`)
- `lemmaFrequency(lemma)` → `POST /api/v1/quran/count/lemma-frequency` (`client.ts:140-142`)
- `frequency(target, profile)` → `POST /api/v1/quran/count/frequency` (`client.ts:146-148`)
- `distribution(target, profile)` → `POST /api/v1/quran/count/distribution` (`client.ts:151-153`)
- `cooccurrence(target, profile, window)` → `POST /api/v1/quran/count/cooccurrence` (`client.ts:156-162`)
- `graphNeighbors(node)` → `POST /api/v1/quran/graph/neighbors` (`client.ts:167-169`)
- `graphPath(from, to)` → `POST /api/v1/quran/graph/path` (`client.ts:172-174`)

## Tool catalogue (`web/src/lib/tools.ts`) and words parser (`web/src/lib/words.ts`)

- `TOOLS`: 27 callable specs across 7 domains (`tools.ts:56-515`, `DOMAIN_ORDER` at `tools.ts:46-54`).
  Quran 7 (`tools.ts:57-190`), graph 5 (`tools.ts:191-313`), hadith 3 (`tools.ts:314-360`),
  tafsir 2 (`tools.ts:361-400`), scripture 5 (`tools.ts:401-463`), isnad 3 (`tools.ts:464-494`),
  rag 2 (`tools.ts:495-514`).
- `DENIED`: 13 sealed tools with policy codes (`tools.ts:521-534`): 7× `rag.*`
  (`QAI-RAG-0326`, `tools.ts:522-528`) + 6× `isnad.identity_*` (durable-state reason,
  `tools.ts:529-533`, reason text `tools.ts:517-519`).
- `paramsJson` / `cliCommand` / `httpRequest` derivation helpers (`tools.ts:544-611`).
- `parseWordInput` shapes: `surah:ayah:position` → token, `surah:ayah` → ayah, else
  lexicon text; empty/malformed → null (`words.ts:22-34`); `tokenId` (`words.ts:37-39`);
  `familyKindFor` (`words.ts:45-53`).

---

## Route 1 — `/` Reader (empty state + directory)

- View file: none (inline in `router.tsx:173-213`); `ReferenceForm` (`router.tsx:41-82`).
- Backend calls: none on this state.
- Params user can enter: deep-link text, default `/read/test@0.1.0/2:1` (`router.tsx:42`);
  invalid → `role="alert"` (`router.tsx:75-79`).
- Empty state: `EmptyState "No reference is open"` (`router.tsx:182-185`); directory nav
  lists all 12 entries (`router.tsx:193-211`).
- Status: **functional** (static entry point; no backend needed).

## Route 2 — `/read/:edition/:surah:ayah` Reader page (deep-link pattern)

- View file: `views/ReadingView.tsx` (mounted `router.tsx:248`); overlay
  `views/CitationView.tsx` when `?citation=` present (`router.tsx:245-250`).
- Backend calls issued:
  - Router: `fetchAyahs(reference)` where reference = trailing `surah:ayah` from
    `parseReadPath` (`router.tsx:132-145`).
  - ReadingView per-verse on demand: `runTool('tafsir.search', {query: seed, limit: 5})`
    + `runTool('hadith.search', {query: seed, limit: 5})` via `RelatedBlock.lookup`
    (`ReadingView.tsx:94-96`); seed = `seedQuery(arabicText)` (import `ReadingView.tsx:19`).
- Params user can enter: none directly (reference comes from URL); margin controls:
  translation/layout/inspector mode + text size (`ReadingView.tsx:419-447`),
  bookmark add/remove persisting to `localStorage` (`ReadingView.tsx:193-209`,
  `451-491`); per-verse `related` button (`ReadingView.tsx:110-115`).
- Empty/loading/error states: `LoadingLine "unrolling the page…"` (`router.tsx:239-243`);
  fail-closed `"No edition is loaded"` panel with CLI remedy, never fabricated text
  (`router.tsx:214-237`); per-verse empty-translation notice (`ReadingView.tsx:297-303`);
  related loading `disabled` (`ReadingView.tsx:119-127`), done with seed + counts or
  per-corpus errors + disclaimer (`ReadingView.tsx:129-157`); lexical inspector is an
  explicit standby (no word payload on ayah endpoint, `ReadingView.tsx:380-403`).
- Status: **functional**.
- Dead control: `onResolve` prop is accepted (`ReadingView.tsx:167-172`) then discarded
  with `void onResolve` (`ReadingView.tsx:182`); Router passes `() => {}` (`router.tsx:248`).
  `resolveReference` (`client.ts:97-99`) consequently has **zero callers** in `web/src`.

## Route 3 — `/read/…?citation=<id>` Citation overlay

- View file: `views/CitationView.tsx` (mounted `router.tsx:249`).
- Backend calls: `fetchCitation(citationId)` → `GET /api/v1/quran/citations/{id}`
  (`CitationView.tsx:20`, `client.ts:102-104`).
- Params: none (id from query string).
- States: loading `role="status"` (`CitationView.tsx:45-53`); mismatch `role="alert"`
  hard failure (`CitationView.tsx:55-70`); verified panel with `CitationCopy` urn/deep-link
  (`CitationView.tsx:72-84`).
- Status: **functional**.

## Route 4 — `/search` Search

- View file: `views/SearchView.tsx` (mounted `router.tsx:150`).
- Backend calls: `search(mode, term)` → `POST /api/v1/quran/search/{mode}`
  (`SearchView.tsx:43`, `client.ts:126-128`).
- Params: 5 mode buttons (`SearchView.tsx:63-80`, modes `SearchView.tsx:9-15`) + free-text
  query input (`SearchView.tsx:86-96`); Enter or Search runs (`SearchView.tsx:90-92,100`).
- States: idle `EmptyState` per mode (`SearchView.tsx:132-136`); running disables button +
  spinner (`SearchView.tsx:97-105`); `ErrorNote` (`SearchView.tsx:108`); zero hits
  explicit `EmptyState "No matches"` (`SearchView.tsx:112-113`); hits in service order,
  never re-sorted (`SearchView.tsx:115-128`, header comment `SearchView.tsx:1-3`).
- Status: **functional**.

## Route 5 — `/research` Research Console (router passes `query=""`)

- View file: `views/ResearchView.tsx` (mounted `router.tsx:152` with `query=""`).
- Backend calls (all in `runKind`, `ResearchView.tsx:61-84`):
  - search → `search('normalized', term)` (mode hardcoded `ResearchView.tsx:64`);
  - root → `runTool('quran.root', {root})` (`:68`); lemma → `runTool('quran.lemma', {lemma})` (`:70`);
  - family → `family('token', query)` — kind hardcoded `token` (`:72`);
  - root-frequency → `rootFrequency(query)` (`:74`); lemma-frequency → `lemmaFrequency(query)` (`:76`);
  - frequency → `frequency(query, 'default')` — profile hardcoded (`:78`);
  - distribution → `distribution(query, 'default')` (`:80`);
  - cooccurrence → `cooccurrence(query, 'default', 5)` — profile/window hardcoded (`:82`).
- Params: 9 instrument buttons (`ResearchView.tsx:37-47,164-183`) + one free-text query
  (`ResearchView.tsx:191-201`); header chip shows `idle` / `N hits` (`ResearchView.tsx:150-152`).
- States: idle `EmptyState` (`ResearchView.tsx:228-234`); running disables Run
  (`ResearchView.tsx:202-210`); `ErrorNote` (`ResearchView.tsx:213`); zero hits
  `EmptyState` with query echo (`ResearchView.tsx:86-95`); findings list service order
  (`ResearchView.tsx:96-114,215-226`).
- Status: **functional** (9 instruments; hardening choices — normalized-only search,
  token-only family, default profile, window 5 — are intentional single-field simplifications,
  not dead controls).

## Route 6 — `/words` Words & Roots

- View file: `views/WordsView.tsx` (mounted `router.tsx:168`).
- Backend calls:
  - Morphology: `runTool('quran.morphology', {surah, ayah, position})`
    (`WordsView.tsx:553-557`); text-kind input short-circuits to a notice, no call
    (`WordsView.tsx:544-547`).
  - Lexicon: `runTool('quran.root'|'quran.lemma'|'quran.family', …)` (`WordsView.tsx:602-609`).
  - Counts: `rootFrequency` / `lemmaFrequency` / `frequency(target, profile)` /
    `distribution(target, profile)` / `cooccurrence(target, profile, window)`
    (`WordsView.tsx:571-575`).
- Params: single input shaped by `parseWordInput` (`WordsView.tsx:634-650,506-508`);
  lexicon mode seg (`WordsView.tsx:699-711`) + kind select + member-id
  (`WordsView.tsx:718-752`); count target + profile (default `L3.diacritics`) + window
  (default `5`) (`WordsView.tsx:792-834`).
- States: per-section `LoadingLine`/`ErrorNote`/`EmptyState` (`WordsView.tsx:668-695,
  767-785, 853-867`); frequency/distribution/cooccurrence render rules block + by-surah
  table + checksum (`WordsView.tsx:303-431`); zero is a proven zero, never absence
  (`WordsView.tsx:310-313,339-342,394-397`); occurrence rows link to reader only through
  the server-pinned edition (`WordsView.tsx:116-180`); unrecognized shapes print a
  margin note, never a verdict (`WordsView.tsx:305,326,381,386`).
- Disabled-with-reason (all reasoned, not dead): morphology Run `disabled={reason !== null
  || loading}` with `title={reason}` (`WordsView.tsx:651-656`); lexicon Look up
  (`WordsView.tsx:755-764`); each count Run (`WordsView.tsx:842-850`); reason strings
  `EMPTY_REASON`/`SHAPE_REASON` (`WordsView.tsx:27-28`) and `countReason` (`WordsView.tsx:529-533`).
- Status: **functional**.

## Route 7 — `/tools` Research Tools

- View file: `views/ToolsView.tsx` (mounted `router.tsx:170`).
- Backend calls: `runTool(spec.name, paramsJson(spec, values))` →
  `POST /api/v1/quran/tool/{name}` (`ToolsView.tsx:111-136`, `client.ts:107-109`).
- Params: catalogue picker grouped by `DOMAIN_ORDER` (`ToolsView.tsx:165-200`); typed form
  generated from `spec.fields` — text/number/json/bool incl. bool toggle + JSON live check
  (`ToolsView.tsx:260-327`, `jsonErrorFor` `ToolsView.tsx:64-72`); surface seg
  `ui|cli|http` (`ToolsView.tsx:227-240`).
- States: Run disabled with reason (`missingRequired`/`fieldErrors`, `ToolsView.tsx:94-108,329-340`);
  `LoadingLine`/`ErrorNote`/`EmptyState` (`ToolsView.tsx:394-424`); result via `ToolLedger`
  (`ToolsView.tsx:410-413`); `cli`/`http` panels derive from the sent params beside the last
  successful ledger, with empty-mirror `EmptyState` until a ui run succeeds
  (`ToolsView.tsx:344-392`, header comment `ToolsView.tsx:1-10`); copy with clipboard-failure
  fallback (`ToolsView.tsx:138-147,374-390`).
- Sealed (by policy, not dead): `DENIED` list renders as `aria-disabled` rows with code +
  reason, no form, nothing issuable (`ToolsView.tsx:202-221`, `tools.ts:521-534`).
- Status: **functional**.

## Route 8 — `/citations` Citations workbench (`/citations?citation=<id>`)

- View file: `views/CitationsWorkbenchView.tsx` (mounted `router.tsx:172` with
  `initialId={parseCitationId(path) ?? ''}`).
- Backend calls: `fetchCitation(trimmed)` → `GET /api/v1/quran/citations/{id}`
  (`CitationsWorkbenchView.tsx:111`); then `fetchAyahs(referenceOf(deepLink))`
  (`CitationsWorkbenchView.tsx:136`) for the tamper-simulation baseline.
- Params: citation-id input (`CitationsWorkbenchView.tsx:171-182`); Open/Re-verify
  (`CitationsWorkbenchView.tsx:183-185,228-234`); tamper textarea + Restore
  (`CitationsWorkbenchView.tsx:288-317`).
- States: idle `EmptyState` (`:189-195`); loading lines (`:197-201,:267-271`);
  failure `ErrorNote` with code/remedy (`:203-219`); ready verdict panel — id, verdict
  label, text_hash, urn, deep link + verify counter (`:221-265`); `none` state when no deep
  link (`:273-279`); tamper panel with byte-diff readout + edition-mismatch note
  (`:281-338`); Open disabled on empty id with hint (`:101,:183-186`).
- Status: **functional**.

## Route 9 — `/hadith` Hadith

- View file: `views/HadithView.tsx` (mounted `router.tsx:154`).
- Backend calls (raw `fetch`, not the typed client):
  - `GET /api/v1/hadith/{collection}/{number}` (`HadithView.tsx:20`);
  - `POST /api/v1/hadith/search` body `{query, collection_slug, limit: 20}` (`HadithView.tsx:35-39`).
- Params: collection (default `bukhari`, `HadithView.tsx:8`), number (default `1`,
  `HadithView.tsx:9`), search query (`HadithView.tsx:10`); Fetch hadith + Search buttons
  (`HadithView.tsx:70-73,91-93`).
- States: `EmptyState "No transmission"` (`HadithView.tsx:99-100`); `ErrorNote`
  (`HadithView.tsx:97`); loading disables both buttons + spinner (`HadithView.tsx:70-73`);
  result is a raw transcript `pre` (`status + body text`, `HadithView.tsx:102-104`).
- Status: **partial** — real endpoints, but unparsed verbatim transcript (no structured
  rendering, no gradings lookup; `hadith.gradings` from `tools.ts:354-360` is reachable only
  via `/tools`).

## Route 10 — `/isnad` Isnad

- View file: `views/IsnadView.tsx` (mounted `router.tsx:156`).
- Backend calls: `runTool('isnad.trace'|'isnad.narrator'|'isnad.rijal', …)` →
  `POST /api/v1/quran/tool/isnad.*` (`IsnadView.tsx:62-72`).
- Params: instrument seg (`IsnadView.tsx:96-108`); trace: from/to/k-optional
  (`IsnadView.tsx:117-160`); narrator/rijal: id (`IsnadView.tsx:162-175`).
- States: Run disabled with inline reason (`required: From/To/Id`, `IsnadView.tsx:44-54,177-184`);
  `LoadingLine`/`ErrorNote`/verbatim `ToolLedger` (`IsnadView.tsx:187-212`).
- Sealed: `isnad.identity_*` (6 tools) listed `aria-disabled` with `qai isnad …` pointer,
  no form (`IsnadView.tsx:214-234`, `tools.ts:529-533`).
- Status: **functional** (read-only surface complete by design).

## Route 11 — `/tafsir` Tafsir

- View file: `views/TafsirView.tsx` (mounted `router.tsx:158`).
- Backend calls (raw `fetch`):
  - `GET /api/v1/tafsir/{work}` (`TafsirView.tsx:19`);
  - `POST /api/v1/tafsir/search` body `{query, work_slug: work || null, limit: 20}` (`TafsirView.tsx:34-38`).
- Params: work slug (default `''`, `TafsirView.tsx:8`), search query (`TafsirView.tsx:9`).
- States: `EmptyState "No transmission"` (`TafsirView.tsx:94-95`); `ErrorNote`
  (`TafsirView.tsx:92`); Fetch work disabled when `loading || work === ''`
  (`TafsirView.tsx:65`); Search disabled only while loading; result raw transcript `pre`
  (`TafsirView.tsx:96-99`).
- Status: **partial** — real endpoints, raw transcript only (tafsir passage-by-reference
  forms `tafsir.get` `tools.ts:362-376` exist only via `/tools`).

## Route 12 — `/scripture` Scripture

- View file: `views/ScriptureView.tsx` (mounted `router.tsx:160`).
- Backend calls (raw `fetch` via `call(url, init)`, `ScriptureView.tsx:14-27`):
  - `GET /api/v1/scripture/editions` (`ScriptureView.tsx:42`);
  - `GET /api/v1/scripture/search?query=&limit=20` (`ScriptureView.tsx:48`);
  - `POST /api/v1/scripture/research` body `{query, domains: 'scripture'}` (`ScriptureView.tsx:57-61`).
- Params: `query` input doubles as "Query / edition" (`ScriptureView.tsx:68-71`); Search and
  Research disabled when `loading || query === ''` (`ScriptureView.tsx:49,63`); List editions
  disabled only while loading (`ScriptureView.tsx:42`); result raw transcript `pre`
  (`ScriptureView.tsx:82-85`); `role="status" consulting…` while loading (`ScriptureView.tsx:87-92`).
- Status: **partial** — editions/search/research work but transcript-only; the lede promises
  "parallel columns" (`ScriptureView.tsx:34`) yet no parallel/verse/translations control exists
  (`scripture.get` / `scripture.translations` / `scripture.parallel_passages`,
  `tools.ts:409-463`, reachable only via `/tools` + Explorer presets).
- Dead control: `edition` state (`ScriptureView.tsx:8`) and its `Edition (for verse)` input
  (`ScriptureView.tsx:72-75`) have **no consumer** — no button or call reads `edition`.

## Route 13 — `/graph` Knowledge Graph

- View file: `views/GraphView.tsx` (mounted `router.tsx:162` with `lines={[]} truncated={false}`).
- Backend calls: `graphNeighbors(node)` → `POST /api/v1/quran/graph/neighbors`
  (`GraphView.tsx:29`); `graphPath(from, to)` → `POST /api/v1/quran/graph/path`
  (`GraphView.tsx:49`).
- Params: node (default `ayah:1:1`, `GraphView.tsx:18`), from/to (defaults `ayah:1:1` /
  `ayah:1:2`, `GraphView.tsx:19-20`); `Open neighbors` / `Trace path` (`GraphView.tsx:90-131`).
- States: chip shows `N edges` (`GraphView.tsx:73`); running disables both buttons
  (`GraphView.tsx:94,127`); `ErrorNote` (`GraphView.tsx:135`); truncation banner
  (`GraphView.tsx:137-141`); zero lines `EmptyState` (`GraphView.tsx:144-147`); edges print
  verbatim, never re-sorted (`GraphView.tsx:150-162`, header `GraphView.tsx:1-3`).
- Note: router-injected `lines`/`truncated` are always `[]`/`false`, so the frozen-payload
  path is inert in production; all live content comes from the two buttons above.
- Status: **functional**.

## Route 14 — `/rag-debug` RAG Debug (sealed vault surface)

- View file: `views/RagDebugView.tsx` (mounted `router.tsx:164`); DIRECTORY labels it
  "sealed vault — retrieval lands in Phase 8" (`router.tsx:107`) but the view itself is a
  working read-only observer (header `RagDebugView.tsx:1-5`).
- Backend calls: `ragDebug(project, query)` / `ragQuery(project, query)` →
  `runTool('rag.debug'|'rag.query', …)` (`RagDebugView.tsx:76`, `client.ts:112-119`).
- Params: mode seg `debug|query` (`RagDebugView.tsx:109-122`), project (default `default`,
  `RagDebugView.tsx:65`), query (`RagDebugView.tsx:66`); `Run {mode} query` disabled while
  loading (`RagDebugView.tsx:148`).
- States: `LoadingLine` (`RagDebugView.tsx:158`); `ErrorNote` with code (`RagDebugView.tsx:160-167`);
  `ToolLedger` + route-decision block (10 fields, `RagDebugView.tsx:16-27,169-198`);
  `EmptyState` (`RagDebugView.tsx:201-208`); 7 mutating `rag.*` tools listed `aria-disabled`
  with `QAI-RAG-0326` (`RagDebugView.tsx:29-38,210-230`).
- Status: **functional** read-only observer (writes sealed by policy).

## Route 15 — `/explorer` API Explorer

- View file: `views/ExplorerView.tsx` (mounted `router.tsx:166`).
- Backend calls: arbitrary `fetch(path, {method, body})` (`ExplorerView.tsx:34-50`); 7 presets
  (`ExplorerView.tsx:9-17`): `GET /api/v1/quran/ayahs/2:1`, `POST /api/v1/quran/search/normalized`,
  `POST /api/v1/quran/graph/neighbors`, `POST /api/v1/quran/tool/scripture.editions`,
  `POST /api/v1/quran/tool/scripture.search`, `POST /api/v1/quran/tool/hadith.search`,
  `POST /api/v1/quran/tool/rag.debug`.
- Params: preset list (`ExplorerView.tsx:68-91`), GET/POST radio (`ExplorerView.tsx:101-122`),
  path input (`ExplorerView.tsx:123-128`), JSON body textarea (POST only,
  `ExplorerView.tsx:144-156`).
- States: transcript panel with `no transmission` / status-line coloring
  (`ExplorerView.tsx:159-176`); Dispatch disabled only while loading (`ExplorerView.tsx:129-133`);
  failures print verbatim into the transcript (`ExplorerView.tsx:45-46`).
- Status: **functional**.

---

## Dead / disabled-without-reason / mock control list

1. `resolveReference` (`client.ts:97-99`) — exported typed function with **zero callers** in
   `web/src`; its intended hookup (`ReadingView onResolve`) is stubbed out (see 2).
2. `ReadingView onResolve` prop (`ReadingView.tsx:167-172`) — neutralized by
   `void onResolve` (`ReadingView.tsx:182`); Router passes a noop (`router.tsx:248`).
3. `ScriptureView edition` input (`ScriptureView.tsx:8,72-75`) — state + `Edition (for verse)`
   field with **no consumer**; no fetch-verse/parallel/translations button exists in the view.
4. Scripture "parallel columns" promise (`ScriptureView.tsx:34`, DIRECTORY note `router.tsx:105`) —
   no parallel UI; `scripture.parallel_passages` / `scripture.translations` / `scripture.get`
   (`tools.ts:409-463`) reachable only via `/tools`.
5. Hadith/Tafsir structured views — absent; both views render raw `status + body` transcripts
   (`HadithView.tsx:102-104`, `TafsirView.tsx:96-99`); `hadith.gradings` (`tools.ts:354-360`)
   and `tafsir.get` by reference similarly `/tools`-only.
6. GraphView frozen-payload props — Router hardcodes `lines={[]} truncated={false}`
   (`router.tsx:162`), so the "frozen payload" rendering path is inert; live buttons are the
   only content source.
7. `TafsirView` Search button (`TafsirView.tsx:86`) — enabled with empty query (sends
   `{query: '', …}`); contrast Fetch work which correctly guards `work === ''`
   (`TafsirView.tsx:65`). Minor validation gap, not a backend break.
8. Loading spinners / `disabled={loading}` on every Run/Search/Dispatch
   (`SearchView.tsx:101`, `ResearchView.tsx:206`, `HadithView.tsx:70,91`, `TafsirView.tsx:65,86`,
   `ScriptureView.tsx:42,49,63`, `GraphView.tsx:94,127`, `RagDebugView.tsx:148`,
   `ExplorerView.tsx:133`, `WordsView.tsx:654,759,846`, `ToolsView.tsx:329`,
   `CitationsWorkbenchView.tsx:183`) — all reasoned anti-double-submit, **not dead**.
9. Sealed-by-policy `aria-disabled` rows (not dead, intentionally non-callable): full `DENIED`
   list on `/tools` (`ToolsView.tsx:202-221`); `isnad.identity_*` subset on `/isnad`
   (`IsnadView.tsx:214-234`); `DENIED_RAG_TOOLS` on `/rag-debug` (`RagDebugView.tsx:210-230`).
10. ReadingView lexical-inspector standby copy (`ReadingView.tsx:380-403`) — explicit placeholder
    text ("lexical notes will dock here…"), honest about the missing word payload, not a mock
    result.
11. No mocked hits, fabricated verses, or client-side sorting anywhere in the surveyed files:
    service-order rendering is asserted in `SearchView.tsx:1-3`, `ResearchView.tsx:1-5`,
    `GraphView.tsx:1-3`; zero-result states are explicit (see each route above).

## Functional roll-up

- Fully functional (10): `/` directory, `/read/…` reader, `?citation=` overlay, `/search`,
  `/research` (9 instruments), `/words` (morphology + lexicon + 5 counts), `/tools`
  (27 callable + 13 sealed), `/citations` workbench, `/isnad` (3 read-only instruments),
  `/graph` (neighbors + path), `/rag-debug` (read-only observer), `/explorer` (7 presets +
  free dispatch). Note: that is 12 headed surfaces plus the overlay; `/` makes 13 static
  routes total.
- Partially functional, transcript-only (3): `/hadith`, `/tafsir`, `/scripture`
  (real endpoints, raw `pre` output; structured/parallel sub-features live only in `/tools`).
- Placeholder / sealed (1 class): mutating `rag.*` + `isnad.identity_*` tools — listed with
  policy codes, no form, CLI-only by design.
