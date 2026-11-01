# `qai` CLI surface — evidence for capability-matrix coverage

Source of truth: `crates/cli/src/` only. A repo-wide grep for
`clap::(Parser|Subcommand|Args)` finds clap derives only in
`crates/cli/src/{lib,main,quran,hadith,tafsir,scripture,links,isnad,rag}.rs` —
no other crate registers CLI commands. No commands invented below; every path
maps to an enum variant in the code cited.

Global flags (all commands, `crates/cli/src/lib.rs:24-42`):
`--config <PathBuf>` opt, `--data-dir <String>` opt, `--log-level <String>` opt,
`--json` bool opt, `--quiet` bool opt, `--yes` bool opt.

"Backend" = the `application::` / `tool_registry::` call the dispatch arm
invokes. R/O = read-only (no DB/filesystem writes); MUT = mutating or
enqueueing durable work. `confirm()` (`crates/cli/src/quran.rs:1702`) gates
MUT owner actions: requires global `--yes` or interactive `APPROVE`.

## 1. Top-level chassis (`crates/cli/src/lib.rs:52-165`)

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai status` | lib.rs:54 | none | none (prints static string) | R/O |
| `qai version` | lib.rs:56 | none | `CARGO_PKG_VERSION` | R/O |
| `qai doctor` | lib.rs:58 | `--json --core --sources --indexes --models --tools --quran --deep --repair-preview` all `bool` opt (note: only `json,repair_preview,quran,deep,indexes` are read in dispatch lib.rs:274; `core/sources/models/tools` accepted but ignored) | `application::db::probe_database`, `verify_migrations`, `doctor::doctor_report` / `run_index_checks` / `run_checks` | R/O (read-only probe; asserts byte-identical DB in doctor.rs tests) |
| `qai serve` | lib.rs:118 | `--bind <String>` opt, default `127.0.0.1:8737`; refused unless loopback | `application::db::database_readiness`, `quran_cli::open_reader`, `quran_search_api`, `quran_lexicon_api`, `quran_graph_api`, `verse_link`, `job_queue::run_worker_host`, `server::api::serve` | R/O gate + long-lived server/worker-host process (never migrates; fails fast if DB not current) |
| `qai tui` | lib.rs:125 | none | same read backends as `serve` via `tui::run` | R/O |
| `qai completions` | lib.rs:161 | `shell: String` req positional | static string table (lib.rs:1405; lists only 12 of 19 commands — stale, omits hadith/tafsir/scripture/links/isnad/rag/tui) | R/O |

## 2. `qai config` (`lib.rs:88-90`, actions `lib.rs:167-188`)

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai config show` | lib.rs:169 | `--explain --defaults --json` bool opt | `Config::default()` / effective `Config` + `OriginMap`, `application::redaction` | R/O |
| `qai config get` | lib.rs:182 | `key: String` req positional | dotted-key lookup over serialized `Config` | R/O |
| `qai config validate` | lib.rs:184 | `--file <PathBuf>` opt | `Config::validate()` | R/O |

## 3. `qai db` (`lib.rs:93-96`, actions `lib.rs:190-213`)

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai db migrate` | lib.rs:192 | none | `application::db::migrate_database` | MUT (applies migrations, creates DB) |
| `qai db status` | lib.rs:194 | none | `application::db::migration_status` | R/O |
| `qai db verify` | lib.rs:196 | none | `application::db::verify_migrations` (checksum compare) | R/O |
| `qai db plan` | lib.rs:198 | none | `application::db::migration_status` (same arm as status, lib.rs:1057) | R/O |
| `qai db backup` | lib.rs:201 | `path: String` req positional | `application::db::backup_database` (`VACUUM INTO`) | MUT (writes backup file; DB itself untouched) |
| `qai db restore` | lib.rs:206 | `path: String` req positional; `--yes` bool req (`true` required, else USAGE refusal) | `application::db::restore_database` | MUT (destructive; keeps `.pre-restore`) |

## 4. `qai secret` (`lib.rs:98-101`, actions `lib.rs:215-223`)

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai secret list` | lib.rs:218 | none | `config::EnvSecretStore::list_refs` | R/O |
| `qai secret set` | lib.rs:220 | `ref: String` req | none — refused with USAGE (Phase 11 gate, lib.rs:1387) | n/a (stub refusal) |
| `qai secret delete` | lib.rs:222 | `ref: String` req | none — refused with USAGE (lib.rs:1394) | n/a (stub refusal) |

## 5. `qai source` (`lib.rs:103-106`, actions `lib.rs:225-233`)

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai source list` | lib.rs:228 | none | `application::db::list_sources` | R/O |
| `qai source show` | lib.rs:230 | `id: String` req | `application::db::get_source` | R/O |
| `qai source import` | lib.rs:232 | `manifest_path: String` req | none — refused with USAGE; cataloged import deferred to Phase 1 (lib.rs:1250) | n/a (stub refusal) |

## 6. `qai job` (`lib.rs:108-111`, actions `lib.rs:235-245`)

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai job list` | lib.rs:238 | none | `application::db::list_jobs` | R/O |
| `qai job show` | lib.rs:240 | `id: String` req | `application::db::get_job` | R/O |
| `qai job cancel` | lib.rs:242 | `id: String` req | `application::job_queue::request_job_cancel` (durable cooperative cancel flag + audit) | MUT |
| `qai job retry` | lib.rs:244 | `id: String` req | `application::job_queue::retry_job` (eligible states only) | MUT |

## 7. `qai audit` (`lib.rs:113-116`, actions `lib.rs:247-253`)

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai audit list` | lib.rs:250 | none | `application::db::list_audit_events` | R/O |
| `qai audit verify` | lib.rs:252 | none | `application::audit_bridge::verify_persisted_audit` + `diagnose_invalid_audit` | R/O |

## 8. `qai quran` (`lib.rs:126-129`, actions `crates/cli/src/quran.rs:14-271`)

Direct reads:

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai quran get` | quran.rs:16 | `reference: String` req; `--translations <String>` opt; `--tokens --glosses` bool opt default false | `application::quran_cli::cmd_get` | R/O |
| `qai quran context` | quran.rs:30 | `reference: String` req; `--before --after <u16>` opt default 3; `--boundary <String>` opt default `surah` | `cmd_context` | R/O |
| `qai quran surah` | quran.rs:44 | `number: u16` req; `--metadata` bool opt | `cmd_surah` | R/O |
| `qai quran division` | quran.rs:52 | `kind: String` req (juz/hizb/rub/manzil/page/ruku/sajdah); `number: u32` req | `cmd_division` | R/O |
| `qai quran resolve` | quran.rs:59 | `reference: String` req | `cmd_resolve` | R/O |
| `qai quran import` | quran.rs:70 | `manifest: String` req; `--adapter <String>` default `json`; `--dry-run` bool; `--reference <String>` opt | `cmd_import` (dry-run validates only; else enqueues `quran.import` job for `qai serve` to stage — never terminal staging) | MUT (enqueue) / R/O with `--dry-run` |
| `qai quran validate` | quran.rs:85 | `target: String` req (manifest path or `slug@version`); `--report <String>` opt | `cmd_validate` | R/O |
| `qai quran catalog` | quran.rs:92 | `catalog: String` req; `--revision --report --database <String>` opt | `cmd_catalog` (no DB, no text import) | R/O |
| `qai quran diff` | quran.rs:109 | `edition: String` req; `--from --to <String>` req; `--format <String>` default `text` | `cmd_diff` | R/O |
| `qai quran activate` | quran.rs:123 | `edition: String` req (`slug@version`) | `cmd_activate` via `confirm()` | MUT (approval-gated) |
| `qai quran rollback` | quran.rs:128 | `edition: String` req; `--to <String>` req | `cmd_rollback` via `confirm()` | MUT (approval-gated) |
| `qai quran deprecate` | quran.rs:136 | `edition: String` req | `cmd_deprecate` via `confirm()` | MUT (approval-gated) |
| `qai quran hashes` | quran.rs:141 | `edition: String` req | `cmd_hashes` (recompute + compare) | R/O |
| `qai quran verify-quotation` | quran.rs:145 | `--edition <String>` req; `--surah <u16>` req; `--ayah <u32>` req; `--text <String>` req | `cmd_verify_quotation` (mismatch = hard failure; no `confirm`, no write) | R/O |
| `qai quran verify` | quran.rs:162 | `--edition <String>` default `active`; `--deep` bool | `cmd_quran_verify` (six corpus-integrity families) | R/O |
| `qai quran search` | quran.rs:212 | `text: String` req + `QuranSearchArgs` (quran.rs:275-347): `--exact --phrase --concatenated --regex` bool (exactly one); `--edition --field --profile --rules --surah --juz --page --revelation-place --global-range <String>` opt; `--match-mode` default `whole_token`; `--phrase-mode` default `ordered_exact`; `--slop` u32 default 0; `--cross-ayah --explain --highlight` bool; `--max-ayah-span` u32 default 3; `--limit` u32 default 20; `--offset` u32 default 0; `--timeout-ms` u64 default 3000 | `cmd_search` (Exact/Normalized/Phrase/Concatenated/Regex tool modes) | R/O |
| `qai quran normalize` | quran.rs:194 | `text: String` opt positional; `--profile --rules --show-rule <String>` opt; `--explain --list-profiles` bool | `cmd_normalize` / `cmd_normalize_list_profiles` / `cmd_normalize_show_rule` (no canonical reads) | R/O |
| `qai quran freq` | quran.rs:228 | `target: String` req; `--profile` default `L3.diacritics`; `--mode` default `single-source` | `cmd_freq` | R/O |
| `qai quran family` | quran.rs:240 | `kind: String` req; `id: String` req (`token:<s>:<a>:<pos>`) | `cmd_family` | R/O |
| `qai quran lemma` | quran.rs:246 | `lemma: String` req | `cmd_morphology_lemma` | R/O |
| `qai quran tool` | quran.rs:257 | `name: String` req (e.g. `quran.get_ayah`); `--params <String>` req (JSON object) | `cmd_tool` (generic registered-tool dispatch; emits `research_checksum`) | depends on named tool (read or write) |

### 8.1 `qai quran edition` (`quran.rs:64-67`, actions `quran.rs:351-379`)

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai quran edition list` | quran.rs:353 | none | `cmd_edition_list` | R/O |
| `qai quran edition show` | quran.rs:355 | `edition: String` req; `--statistics --hashes` bool | `cmd_edition_show` | R/O |
| `qai quran edition active` | quran.rs:364 | none | `cmd_edition_active` | R/O |
| `qai quran edition verify` | quran.rs:369 | `edition: String` req; `--reviewer --method <String>` req | `cmd_edition_verify` via `confirm()` (records `verified_by`; reviewer identity is an owner act) | MUT (approval-gated) |

### 8.2 `qai quran translation` (`quran.rs:172-175`, actions `quran.rs:383-396`)

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai quran translation list` | quran.rs:385 | none | `cmd_translation_list` | R/O |
| `qai quran translation import` | quran.rs:387 | `manifest: String` req | `cmd_translation_import` | MUT |
| `qai quran translation show` | quran.rs:391 | `slug: String` req | `cmd_translation_show` | R/O |

### 8.3 `qai quran gloss` (`quran.rs:177-180`, actions `quran.rs:400-406`)

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai quran gloss import` | quran.rs:403 | `manifest: String` req | `cmd_gloss_import` | MUT |

### 8.4 `qai quran forms` (`quran.rs:182-185`, actions `quran.rs:410-417`)

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai quran forms rebuild` | quran.rs:413 | `edition: String` req (must be active; `slug@version`) | `cmd_forms_rebuild` (derived token/ayah forms; canonical text untouched) | MUT |

### 8.5 `qai quran index` (`quran.rs:187-190`, actions `quran.rs:421-461`)

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai quran index rebuild` | quran.rs:423 | `--index <String>` opt (default `quran.ayah.v1`); `--edition <String>` opt (default active) | `cmd_index_rebuild` (build + atomic activate) | MUT |
| `qai quran index verify` | quran.rs:429 | `--index <String>` opt | `cmd_index_verify` (serving generation) | R/O |
| `qai quran index verify-job` | quran.rs:433 | `--index <String>` opt; `--deep` bool | `cmd_index_verify_job` (enqueues `quran.index.verify` job) | MUT (enqueue; check itself read-only) |
| `qai quran index gc` | quran.rs:439 | `--index <String>` opt; `--keep <usize>` default 2 | `cmd_index_gc` (generation retention) | MUT |
| `qai quran index rollback` | quran.rs:454 | `--index <String>` opt | `cmd_index_rollback` (single-step, serving generation) | MUT |

### 8.6 `qai quran root` (`quran.rs:219-221`, actions `quran.rs:465-475`)

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai quran root list` | quran.rs:467 | `--prefix <String>` opt; `--limit <usize>` default 50 (clamped 1..=500) | `cmd_morphology_root_list` | R/O |

### 8.7 `qai quran count` (`quran.rs:223-226`, actions `quran.rs:479-604`)

All read-only, rule-relative; every row takes `--profile <String>` (default `L3.diacritics`).

| Command | Def | Extra args | Backend | R/W |
|---|---|---|---|---|
| `frequency` | quran.rs:481 | `target: String` req | `cmd_count_frequency` | R/O |
| `distribution` | quran.rs:489 | `target` req | `cmd_count_distribution` | R/O |
| `occurrences` | quran.rs:497 | `target` req | `cmd_count_occurrences` | R/O |
| `hapax` | quran.rs:504 | `--limit <usize>` default 100 | `cmd_count_hapax` | R/O |
| `cooccurrence` | quran.rs:512 | `target` req; `--window <usize>` default 3; `--limit` default 25 | `cmd_count_cooccurrence` | R/O |
| `collocation` | quran.rs:528 | same shape as cooccurrence | `cmd_count_collocation` (PMI/LLR/t-score) | R/O |
| `numeric-report` | quran.rs:541 | `target` req | `cmd_count_numeric_report` | R/O |
| `missing-form` | quran.rs:549 | `target` req | `cmd_count_missing_form` | R/O |
| `near-duplicates` | quran.rs:557 | `--threshold <f64>` default 0.8; `--limit` default 25 | `cmd_count_near_duplicates` | R/O |
| `root-frequency` | quran.rs:565 | `root: String` req; `--mode` default `single-source` | `cmd_count_root_frequency` | R/O |
| `lemma-frequency` | quran.rs:577 | `lemma: String` req; `--mode` default `single-source` | `cmd_count_lemma_frequency` | R/O |
| `interval` | quran.rs:589 | `target` req | `cmd_count_interval` | R/O |
| `unusual-usage` | quran.rs:595 | `target` req | `cmd_count_unusual_usage` (lexicon-gated) | R/O |

### 8.8 `qai quran morphology` (`quran.rs:252-255`, actions `quran.rs:608-711`)

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai quran morphology import` | quran.rs:610 | `--file --dataset --version --adapter(default json) --edition --attribution --license-status --license-json --license-evidence --batch <String>`; `file/dataset/version/edition` effectively req (no defaults); stages only, never activates | `cmd_morphology_import` | MUT (staging write) |
| `qai quran morphology activate` | quran.rs:643 | `--batch <String>` req; `--approval <String>` req | `cmd_morphology_activate` | MUT (approval-gated) |
| `qai quran morphology datasets` | quran.rs:652 | none | `cmd_morphology_datasets` | R/O |
| `qai quran morphology token` | quran.rs:654 | `--edition <String>` req; `--surah --ayah <i64>` req; `--position <i64>` default 1 | `cmd_morphology_token` | R/O |
| `qai quran morphology compare` | quran.rs:662 | same shape as `token` | `cmd_morphology_compare` | R/O |
| `qai quran morphology diff` | quran.rs:669 | `from: String` req; `to: String` req (dataset `slug@version`); `--format` default `text` | `cmd_morphology_diff` | R/O |
| `qai quran morphology root` | quran.rs:676 | `root: String` req | `cmd_morphology_root` | R/O |
| `qai quran morphology lemma` | quran.rs:683 | `lemma: String` req | `cmd_morphology_lemma` | R/O |
| `qai quran morphology affix` | quran.rs:690 | `affix: String` req; `--profile` default `L7.affix` | `cmd_morphology_affix` | R/O |

### 8.9 `qai quran graph` (`quran.rs:257-260`, actions `quran.rs:739-887`)

Budget flags shared by read subcommands (`GraphBudgetFlags`, quran.rs:717-733):
`--max-nodes --max-edges --max-paths --max-fanout --timeout-ms`, all opt
(`Option<usize|u64>`; explicit out-of-range fails pre-flight).

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai quran graph build` | quran.rs:741 | `--out <String>` opt | `cmd_graph_build` (structural projection from active edition, SQLite; optional JSON file write) | MUT (builds/persists projection) |
| `qai quran graph inspect` | quran.rs:747 | `--file <String>` opt; `--db` bool | `cmd_graph_inspect` | R/O |
| `qai quran graph neighbors` | quran.rs:756 | `--file` opt; `--db` bool; `--node <String>` req; `--hops <usize>` default 1; `--projection <String>` opt; + budgets | `cmd_graph_neighbors` | R/O |
| `qai quran graph path` | quran.rs:777 | `--file` opt; `--db` bool default false; `--from --to <String>` req; `--hops` default 4; `--mode` default `paths` (reachability/shortest/paths); `--paths <usize>` opt; + budgets | `cmd_graph_path` | R/O |
| `qai quran graph subgraph` | quran.rs:806 | `--seed <String>` repeatable req; `--hops` default 4; + budgets | `cmd_graph_subgraph` | R/O |
| `qai quran graph pattern` | quran.rs:817 | `--seed` repeatable req; `--step <String>` (`EDGE[:Kind]`) repeatable req; `--hops` default 4; + budgets | `cmd_graph_pattern` | R/O |
| `qai quran graph root-family` | quran.rs:833 | `root: String` req; `--limit <usize>` default 25 | `cmd_graph_root_family` (lexicon-gated) | R/O |
| `qai quran graph export` | quran.rs:841 | `--file` opt; `--db` bool; `--format` default `json` (json/dot/svg); `--seed --out --projection` opt | `cmd_graph_export` | R/O (writes `--out` file only if given) |
| `qai quran graph doctor` | quran.rs:873 | `--deep` bool | `cmd_graph_doctor` (projection health; repair lives under doctor-repair) | R/O |
| `qai quran graph review …` | quran.rs:861 (actions quran.rs:895-1031) | see 8.10 | `application::quran_graph_annotations` via `cmd_graph_review_*` | MUT (review-queue writes; CLI-only, no HTTP route) |
| `qai quran graph doctor-repair …` | quran.rs:881 (actions quran.rs:1041-1065) | see 8.11 | `application::quran_graph_doctor` via `cmd_graph_doctor_repair_*`, each wrapped in `confirm()` + one audit event | MUT (approval-gated) |

### 8.10 `qai quran graph review` (actions `quran.rs:895-1031`)

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `review propose` | quran.rs:897 | `--src --edge --dst --source-id --source-location --author <String>` req; `--id --kind(default annotation) --evidence --projection --edition --scope` opt | `cmd_graph_review_propose` (layer B, pending) | MUT |
| `review suggest` | quran.rs:935 | propose-shape + `--algorithm --algorithm-version <String>` req; `--confidence <f64>` req | `cmd_graph_review_suggest` (layer D, pending) | MUT |
| `review accept` | quran.rs:981 | `--id --reviewer <String>` req; `--decided-at` opt | `cmd_graph_review_decide("accept")` | MUT |
| `review reject` | quran.rs:987 | same shape as accept | `cmd_graph_review_decide("reject")` (tombstoned, retained) | MUT |
| `review correct` | quran.rs:1004 | `--id --reviewer` req; `--decided-at --src --edge --dst --evidence --source-location` opt | `cmd_graph_review_correct` (superseding accepted row) | MUT |

### 8.11 `qai quran graph doctor-repair` (actions `quran.rs:1041-1065`)

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `doctor-repair quarantine-dangling` | quran.rs:1044 | none | `cmd_graph_doctor_repair_quarantine` via `confirm()` | MUT |
| `doctor-repair tombstone-gc` | quran.rs:1046 | `--retention-days <u32>` default 90 | `cmd_graph_doctor_repair_gc` via `confirm()` | MUT |
| `doctor-repair rebuild-projection` | quran.rs:1054 | `--projection <String>` req; `--seed-file` opt (annotated family) | `cmd_graph_doctor_repair_rebuild` via `confirm()` | MUT |

## 9. `qai hadith` (`lib.rs:131-134`, actions `crates/cli/src/hadith.rs:12-25`)

Backend: `application::{hadith_reader,hadith_search,hadith_tools}` via
`tool_registry::HadithBackend`; `--profile` accepted but ignored (`_`,
hadith.rs:83,98).

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai hadith get` | hadith.rs:14 | `reference: String` req | `backend_get_hadith` | R/O |
| `qai hadith search` | hadith.rs:16 | `query: String` req; `--collection <String>` opt; `--profile <String>` opt (ignored) | `backend_search_hadith` | R/O |
| `qai hadith gradings` | hadith.rs:24 | `reference: String` req | `backend_get_hadith` + `backend_get_gradings` | R/O |

## 10. `qai tafsir` (`lib.rs:135-138`, actions `crates/cli/src/tafsir.rs:12-26`)

Backend: `application::{tafsir_reader,tafsir_search,tafsir_tools}` via
`tool_registry::TafsirBackend`; `--profile` accepted but ignored (tafsir.rs:88,100).

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai tafsir get` | tafsir.rs:14 | `reference: String` req (`work_slug[:volume[:page]]` / `work_slug:chapter`) | `backend_get_tafsir` | R/O |
| `qai tafsir search` | tafsir.rs:19 | `query: String` req; `--work <String>` opt; `--profile` opt (ignored) | `backend_search_tafsir` | R/O |

## 11. `qai scripture` (`lib.rs:140-143`, actions `crates/cli/src/scripture.rs:10-43`)

Backend: `application::{scripture_reader,scripture_tools,scripture_search,verse_link,cross_domain}`.

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai scripture editions` | scripture.rs:12 | none | `backend_list_editions` | R/O |
| `qai scripture get` | scripture.rs:14 | `edition: String` req; `book: String` req; `chapter: u32` req; `verse: u32` req | `backend_get_scripture` | R/O |
| `qai scripture search` | scripture.rs:21 | `query: String` req; `--edition <String>` opt | `backend_search_scripture` | R/O |
| `qai scripture translations` | scripture.rs:26 | `verse_id: String` req; `--language <String>` opt | `backend_get_translation` | R/O |
| `qai scripture research` | scripture.rs:38 | `query: String` req; `--domains <Vec<String>>` opt, comma-delimited (`quran,hadith,tafsir,scripture,books`) | `CrossDomainResearchService` (scripture+tafsir+hadith searchers) | R/O |
| `qai scripture link suggest` | scripture.rs:48 | `scripture_verse_id: String` req | `VerseLinkService::suggest_scripture_links` (uncertain suggestions) | R/O |
| `qai scripture link confirm` | scripture.rs:52 | `link_id: String` req; `--reviewer <String>` req | `VerseLinkService::confirm_link` (owner gate) | MUT |
| `qai scripture link reject` | scripture.rs:58 | `link_id: String` req; `--reviewer <String>` req | `VerseLinkService::reject_link` (owner gate) | MUT |
| `qai scripture link list` | scripture.rs:64 | `quran_ref: String` req (`surah:ayah`); `--include-uncertain` bool | `VerseLinkService::get_links_for_verse` | R/O |

## 12. `qai links` (`lib.rs:145-149`, actions `crates/cli/src/links.rs:10-18`)

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai links verse` | links.rs:12 | `surah: u16` req; `ayah: u32` req; `--include-uncertain` bool | `VerseLinker::get_links_for_verse` (`VerseLinkService`) | R/O |

## 13. `qai isnad` (`lib.rs:151-154`, actions `crates/cli/src/isnad.rs:71-92`)

Reads via `application::{isnad_reader::IsnadReaderService,isnad_tools::IsnadToolBackend}`;
identity writes via `tool_registry::IsnadBackend`.

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai isnad narrator` | isnad.rs:72 | `--id <String>` req | `backend_get_narrator` | R/O |
| `qai isnad rijal` | isnad.rs:76 | `--id <String>` req | `backend_get_rijal` | R/O |
| `qai isnad trace` | isnad.rs:80 | `--from --to <String>` req; `--k <usize>` default 3 | `backend_trace` | R/O |
| `qai isnad identity propose` | isnad.rs:13 | `--claim --evidence --provenance-layer <String>` req (JSON); `--source-location --algorithm --algorithm-version <String>` opt; `--confidence <f64>` opt | `backend_propose_identity` | MUT |
| `qai isnad identity accept` | isnad.rs:28 | `--id --reviewer <String>` req | `backend_accept_identity` | MUT |
| `qai isnad identity reject` | isnad.rs:34 | `--id --reviewer <String>` req; `--yes` bool req (refused without it) | `backend_reject_identity` | MUT |
| `qai isnad identity dispute` | isnad.rs:42 | `--id --reviewer <String>` req | `backend_dispute_identity` | MUT |
| `qai isnad identity supersede` | isnad.rs:48 | `--id --claim --evidence --provenance-layer --reviewer` req; `--source-location --algorithm --algorithm-version` opt; `--confidence` opt | `backend_supersede_identity` | MUT |

## 14. `qai rag` (`lib.rs:156-159`, actions `crates/cli/src/rag.rs:11-62`)

Note: RAG commands operate on a project directory (`<project>/index.db`),
not the main `qai.db`. `--filter` on `query` and `--limit` are accepted but
ignored (rag.rs:188,221).

| Command | Def | Args / flags | Backend | R/W |
|---|---|---|---|---|
| `qai rag create` | rag.rs:13 | `name: String` req; `--dir <PathBuf>` opt | `SqliteRagReader::create_project` (+ writes `config.toml`, migrates `index.db`) | MUT |
| `qai rag import` | rag.rs:19 | `project: String` req; `docs: PathBuf` req (must be dir); `--glob <String>` opt | `SqliteRagReader::import_documents` | MUT |
| `qai rag query` | rag.rs:26 | `project: String` req; `query: String` req; `--filter <String>` repeatable (ignored); `--limit <u32>` opt (ignored) | `RagToolBackend::backend_query` (hybrid-RRF + rerank) | R/O |
| `qai rag debug` | rag.rs:35 | `project: String` req; `query: String` req | `RagToolBackend::backend_debug` (route decision + checksum) | R/O |
| `qai rag source activate` | rag.rs:48 | `project: String` req; `source: String` req | `backend_source_status(…, "active")` | MUT |
| `qai rag source deactivate` | rag.rs:52 | same shape | `backend_source_status(…, "deactivated")` | MUT |
| `qai rag source tombstone` | rag.rs:56 | same shape + `--yes` bool (else interactive APPROVE) | `backend_source_status(…, "tombstoned")` + audit seq | MUT |

## Totals

19 top-level commands; 131 leaf command paths (6 bare top-level + 21
chassis subcommands + 21 direct `quran` leaves + 53 nested `quran`
sub-subcommands + 3 hadith + 2 tafsir + 9 scripture + 1 links + 8 isnad + 7
rag). Mutating leaves: `db` migrate/backup/restore (3), `job` cancel/retry
(2), `quran` import(enqueue)/activate/rollback/deprecate/edition-verify (5),
translation-import (1), gloss-import (1), forms-rebuild (1),
index rebuild/verify-job/gc/rollback (4), morphology import/activate (2),
graph build (1), graph-review (5), graph doctor-repair (3),
`scripture link` confirm/reject (2), `isnad identity` ×5 (5), `rag`
create/import/source×3 (5) = 42 mutating-or-enqueueing leaves; remainder
read-only (plus 2 refusal stubs: `secret set/delete`, `source import`).
