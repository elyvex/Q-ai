# Phase 1 — Technology, Library & Engine Integration Register

**Phase:** P1 — Canonical Quran Core
**Status of this document:** living — update when a technology choice changes
**Purpose:** state, in one place, exactly which technologies, libraries, and engines Phase 1
integrates with the code; which of those are **already locked** by Phase 0; and which are
**still open and can be changed now** (e.g. the database engine, the web framework, the cache).
**Sources of truth:** `plan.md`, `../phase-00-foundation/plan.md`, `Cargo.toml` (workspace deps),
`xtask/allowlist.toml` (dependency-direction rules), and the current `Cargo.lock` /
local registry cache.

> Phase 1 code has not been written yet. Everything in the **🟡 Open now** rows is therefore a
> real decision that can still be made without breaking anything. Everything in the **🔒 Locked**
> rows is already fixed by Phase 0 and by shipped migrations; changing it requires a new ADR and
> a migration/version strategy, not a `Cargo.toml` edit.

---

## 1. Status Legend

| Symbol | Meaning |
|---|---|
| 🔒 | **Locked.** Already shipped in Phase 0 or encoded in applied migrations. Changing it is a breaking, ADR-level change. |
| 🟡 | **Open now.** Not yet implemented. Phase 1 can still pick a different library/engine before the relevant task starts. |
| ⏸ | **Deferred.** Explicitly assigned to a later phase; Phase 1 must only reserve fields/columns, not integrate. |
| ✅ | Available in the local Cargo registry cache (offline install feasible — no network needed to add it). |
| 🌐 | Would need a network fetch to add (not currently cached). |

---

## 2. Decision Register — The One-Page Answer

| # | Concern | Phase-1 choice (recommended) | Library / engine | Status | Changeable now? | Governing ADR | Main alternatives |
|---|---|---|---|---|---|---|---|
| 1 | **Relational database engine** | **SQLite** (file, WAL) | SQLite 3 (bundled via `sqlx`) | 🔒 | ❌ No — Phase 0 ADR-0001; Postgres is Phase 12 | ADR-0001 (Phase 0); layout ADR-0106 | PostgreSQL (⏸ Phase 12), DuckDB |
| 2 | **Database access library** | **`sqlx` 0.8** (`runtime-tokio`, `sqlite`) | `sqlx` | 🔒 | ❌ No — already in `storage-sqlite` | ADR-0001 | `rusqlite`, `diesel`, `sea-orm` |
| 3 | **Migrations** | Numbered, checksummed SQL files + Phase-0 runner | hand-rolled (`storage-sqlite::migrate`) | 🔒 | ❌ No — append-only files `0010`–`0015` | ADR-0002 | `refinery`, `sqlx::migrate!`, `flyway` |
| 4 | **Corpus hashing** | **SHA-256** via `sha2` | `sha2` 0.10 | 🔒 | ⚠️ Recipe frozen by ADR-0108 once first import happens; tag allows future algorithm | ADR-0108 | `blake3` ✅ (faster, different digest) |
| 5 | **Unicode grapheme handling** | `unicode-segmentation` | `unicode-segmentation` 1.12/1.13 ✅ | 🟡 | ✅ Yes (task P1-T06/T21) | ADR-0104 | `unicode-properties`, `unicode-script`, hand-rolled |
| 6 | **Unicode normalization (NFC/NFD check)** | `unicode-normalization` | `unicode-normalization` 0.1.25 ✅ | 🟡 | ✅ Yes (task P1-T21) | ADR-0104 | `icu` crates, `unicode-normalization-alignments` |
| 7 | **RTL rendering (debug reader only)** | `unicode-bidi` | `unicode-bidi` 0.3 ✅ | 🟡 | ✅ Yes (task P1-T54) | ADR-0111 (deep links) | CSS `direction: rtl` only (no crate) |
| 8 | **Reference-grammar parser** | **Hand-written recursive descent** | none (pure Rust) | 🟡 | ✅ Yes (task P1-T08) | ADR-0102 | `winnow`, `nom`, `pest`, `chumsky` |
| 9 | **Intermediate format serialization** | `serde` + `serde_json` | `serde` 1, `serde_json` 1 | 🔒 | ❌ No — Phase 0 standard | ADR-0007 | `serde_yaml`, `rmp-serde` |
| 10 | **JSON adapter (dataset shape 1)** | `serde_json` | `serde_json` | 🔒 | ❌ No | D1.2 | — |
| 11 | **CSV adapter (dataset shape 2)** | `csv` crate | `csv` 1.3 ✅ | 🟡 | ✅ Yes (task P1-T17) | D1.2 | `csv-core`, manual splitting |
| 12 | **XML adapter (optional dataset shape)** | `quick-xml` | `quick-xml` 0.37–0.41 ✅ | 🟡 | ✅ Yes (task P1-T17) | D1.2 | `roxmltree`, `serde-xml-rs` |
| 13 | **Edition difference (`DifferenceReport`)** | `similar` (char/word diff) | `similar` 2.7/3.2 ✅ | 🟡 | ✅ Yes (task P1-T27) | ADR-0109 | `imara-diff`, `diff`, hand-rolled Myers |
| 14 | **Canonical lookup cache** | `lru` (Mutex-guarded) | `lru` 0.12–0.18 ✅ | 🟡 | ✅ Yes (task P1-T35) | ADR-0113 | `moka` 🌐, `quick_cache`, `dashmap` |
| 15 | **Async runtime** | `tokio` multi-thread | `tokio` 1 | 🔒 | ❌ No — Phase 0 standard | — | `async-std`, `smol` |
| 16 | **Job execution** | Phase-0 DB-backed job system | hand-rolled (`jobs`) | 🔒 | ❌ No | ADR-0003 | Redis/BullMQ, `apalis`, external broker |
| 17 | **HTTP server for Quran read API v1** | **`axum` + `tower-http`** | `axum` 0.8.9 ✅, `tower-http` 0.6/0.7 (in lock) | ✅ ratified (OD-06, 2026-09-22) | ❌ Ratified — workspace-pinned + lockfile; loopback only; no CORS layer; per-install bearer token; timeouts/body-limits via tower middleware | — | current hand-rolled `tokio::net` server; `hyper` directly; `actix-web`; `poem` |
| 18 | **CLI framework** | `clap` derive | `clap` 4 | 🔒 | ❌ No — Phase 0 standard | ADR-0010 | `argh`, `pico-args` |
| 19 | **Error types / diagnostics** | `thiserror` + Phase-0 `Diagnostic` trait | `thiserror` 2, `anyhow` 1 | 🔒 | ❌ No | ADR-0010 | `snafu`, `miette` |
| 20 | **Observability** | `tracing`, `metrics`; OTLP opt-in | `tracing` 0.1, `tracing-subscriber` 0.3, `metrics` 0.24 | 🔒 (OTLP ⏸) | ❌ Core no; OTLP deferred | ADR-0011 | `log`, `prometheus`, `opentelemetry` (⏸) |
| 21 | **JSON Schema generation (manifests, doctor, API)** | `schemars` for derived schemas; `jsonschema` for validation tests | `schemars` 0.8–1.2 ✅, `jsonschema` 0.46/0.48 ✅ | 🟡 | ✅ Yes (tasks P1-T15/T52) | ADR-0007 | hand-rolled JSON (current `xtask::gen-schema`), `valico` |
| 22 | **Property testing** | `proptest` | `proptest` 1 | 🔒 | ❌ No — Phase 0 standard | — | `quickcheck`, `arbitrary` |
| 23 | **Snapshot testing (CLI/API)** | `insta` + `trycmd` | `insta` 1, `trycmd` 0.15 | 🔒 | ❌ No | — | `insta-cmd`, golden files |
| 24 | **Temp files in tests** | `tempfile` | `tempfile` 3 | 🔒 | ❌ No | — | `assert_fs` |
| 25 | **Manifest signature verification** | `ed25519-dalek` + `base64` | `ed25519-dalek` 2, `base64` 0.22 | 🔒 | ❌ No — Phase 0 | ADR-0007 | `ring`, `p256` |
| 26 | **Secrets encryption at rest** | `chacha20poly1305` + `zeroize` | Phase 0 `config` backend | 🔒 | ❌ No | ADR-0005 | `age`, OS keychain (`keyring`) |
| 27 | **Search index / vector store / embeddings** | none | — | ⏸ | ❌ Must **not** be integrated in Phase 1 | — | Phase 2/3 |
| 28 | **LLM / model provider** | none | — | ⏸ | ❌ Must **not** be integrated in Phase 1 | I2 | Phase 9 |

---

## 3. The Database Question, Answered Directly

The user's example was "the database — which library". The answer in full:

- **Engine: SQLite** (a single local file, WAL journal mode, `synchronous=FULL`,
  `foreign_keys=ON`, `busy_timeout=5000`). This is not a Phase-1 choice — it is **locked by
  Phase 0**. The `storage-sqlite` crate already opens a dual-pool setup (one serialized write
  connection, N read-only connections) and the checksummed migration runner is already shipped.
  Phase 1 adds migrations `0010`–`0015` **as SQLite SQL** and cannot switch engines without
  invalidating every applied migration and every Phase-0 parity test.
- **Library: `sqlx` 0.8** with `default-features = false` and features
  `["runtime-tokio", "sqlite"]`. Chosen in Phase 0 (ADR-0001). `sqlx` is compile-time query
  checked and uses the same async runtime as the rest of the workspace.
- **Not `rusqlite`, not `diesel`, not an ORM.** Phase 0 repository code is hand-written SQL over
  `sqlx`; Phase 1's `QuranRepository` must follow the same pattern.
- **PostgreSQL is Phase 12, not Phase 1.** The plan's "portability rule" applies: Phase 1 writes
  no SQLite-only SQL into code paths shared with a future Postgres backend; dialect-specific SQL
  stays inside `storage-sqlite`.
- **Changeable now?** Only in the sense that no Phase-1 *migration* has been applied yet, so the
  *schema* (the DDL in `plan.md` §6) can still be adjusted before task P1-T12. The *engine and
  library* are not open.

> **One practical caveat for this environment:** adding brand-new external crates requires the
> crate to be present in the local registry cache, because the workspace currently resolves
> offline. The ✅/🌐 column in §2 records which candidate libraries are already cached. Where a
> choice would otherwise require a network fetch, a cached alternative is noted.

---

## 4. The Biggest Genuinely-Open Choice: The HTTP Framework

This deserves its own section because it is the one integration where the "obvious" recommendation
is not yet in the workspace.

- **What Phase 0 ships today:** `crates/server` is a **hand-rolled HTTP/1.1 server** over
  `tokio::net::TcpListener`, serving `GET /healthz` and `GET /readyz` and returning 404 for
  everything else. Its only dependencies are `tokio`, `serde_json`, `thiserror`, and `config`.
- **What Phase 1 (D1.7) needs:** routing with path parameters (`/api/v1/quran/ayahs/:reference`),
  query-string extraction, a stable response envelope, `ETag` handling based on
  `(text_hash, corpus_generation)`, `Cache-Control`, content negotiation (`Content-Language` for
  Arabic), request size/timeout/concurrency limits, and a consistent error body from the Phase-0
  `Diagnostic`. Doing all of that on a hand-rolled socket loop is error-prone and wastes
  engineering days.
- **Recommendation:** adopt **`axum` 0.8 + `tower-http`** for the Quran read API and keep the
  Phase-0 health endpoints. Both are available in the local cache; `tower-http` is already pulled
  into `Cargo.lock` transitively. This becomes a new `server -> application` wiring and a new
  `server` dependency; because `server` is already allowlisted to depend on
  `application`/`config`/`observability`, only *external* crate additions are needed.
- **Alternatives, all changeable now:** keep the hand-rolled server and add just enough routing
  (cheapest, highest long-term risk); use `hyper` directly (already in lock); `actix-web` or
  `poem` (heavier, different runtime conventions); or `warp`.
- **Where the decision is recorded:** there is no ADR for the web framework yet. Adopting one
  should be a short ADR (or an addition to the Phase-0 ADR-0011 observability/API stack) before
  P1-T39 starts, because it also fixes how `tower` middleware (timeouts, limits, compression) is
  applied.
- **Ratified 2026-09-22 (OD-06, owner series):** `axum` 0.8.x + `tower-http` 0.6.x,
  workspace-pinned + lockfile; bind `127.0.0.1` only; no CORS layer; per-install
  random bearer token on every route (deny-by-default even on localhost);
  timeouts/body-limits via tower middleware. AC-P1-14 wording unblocked; health
  endpoints unchanged.

---

## 5. New Workspace Dependencies Phase 1 Is Expected To Add

These are the only external crates Phase 1 is expected to add beyond what Phase 0 already uses.
Each still needs to be entered in the root `Cargo.toml` `[workspace.dependencies]` and referenced
by the owning crate.

| Crate | Proposed version | Used by | Task | Cached? | Notes |
|---|---|---|---|---|---|
| `unicode-segmentation` | 1.12 or 1.13 | `quran-core`, `quran-corpus` | P1-T06, T21 | ✅ | grapheme-cluster counts and offsets; `char_count` is defined in graphemes |
| `unicode-normalization` | 0.1.25 | `quran-corpus::validation` | P1-T21 | ✅ | NFC/NFD/NFKC/NFKD detection and comparison |
| `lru` | 0.12–0.18 | `application`/`quran-corpus` reader cache | P1-T35 | ✅ | plan says "`moka`-style LRU"; `moka` is not cached, `lru` is |
| `similar` | 2.7 or 3.2 | `quran-corpus::differ` | P1-T27 | ✅ | char-level edition diff |
| `csv` | 1.3 | `quran-corpus::adapters` | P1-T17 | ✅ | second adapter to prove extensibility |
| `quick-xml` | 0.37–0.41 | `quran-corpus::adapters` | P1-T17 | ✅ | optional XML dataset shape |
| `schemars` | 0.8–1.2 | intermediate format, doctor JSON schema | P1-T15, T52 | ✅ | JSON Schema derivation |
| `jsonschema` | 0.46/0.48 | validation tests (manifest, doctor, API) | P1-T15, T52 | ✅ | schema-conformance assertions |
| `axum` | 0.8 | `server` | P1-T39 | ✅ | see §4 |
| `tower-http` | 0.6/0.7 | `server` | P1-T39 | ✅ (in lock) | limits, timeout, compression, tracing |
| `unicode-bidi` | 0.3 | `server` debug reader | P1-T54 | ✅ | only if server-side bidi shaping is needed; CSS RTL may suffice |

**Optional / only if the chosen dataset requires them:** a detection library for encoding
(`encoding_rs` ✅ is cached) if the source is not UTF-8; an Arabic reshaping library if the debug
reader must render without a browser font fallback.

**Explicitly not added in Phase 1:** any `llm`, `embeddings`, `retrieval`, vector-store, or graph
crate. This is invariant I2 and is enforced by the architecture check; adding one is a gate
failure.

---

## 6. Frozen by Phase 0 — Do Not Relitigate In Phase 1

| Concern | Locked choice | Why it is locked |
|---|---|---|
| DB engine + driver | SQLite + `sqlx` 0.8 | Applied migrations and `storage-sqlite` code |
| Migrations | append-only checksummed SQL, Phase-0 runner | `checksums.json` would break |
| Async runtime | `tokio` | Every async trait/repository is written against it |
| Serialization | `serde` + `serde_json` | Domain/storage/sources types derive it |
| Hashing primitive | `sha2` SHA-256 + `ContentHash{algorithm,hex}` | `sources` and migration checksums already use it |
| Signatures | `ed25519-dalek` + `base64` | `sources::manifest` already verifies with it |
| Error model | `thiserror` + `Diagnostic` + `QAI-*` codes | CLI/doctor render it; codes are public API |
| CLI | `clap` derive | Phase-0 command tree and completions |
| Job system | in-process DB-backed `jobs` crate | `quran.import` must reuse its checkpoints/cancel |
| Test tooling | `proptest`, `insta`, `trycmd`, `tempfile` | Existing suites and CI |
| Job/secret/observability stack | `tracing`, `metrics`, `chacha20poly1305`, `zeroize` | Phase-0 gates |

---

## 7. What Can Change Right Now — Explicit List

Before the owning task starts, each of these may still be changed, with no migration or
compatibility cost. The "decide by" column names the task that closes the choice.

| Open choice | Recommended | Alternative(s) | Decide by | ADR / owner |
|---|---|---|---|---|
| HTTP framework | `axum` + `tower-http` | hand-rolled, `hyper`, `actix-web`, `poem`, `warp` | P1-T39 (D1.7) | needs a short ADR |
| Cache library | `lru` | `moka`, `quick_cache`, `dashmap`, hand-rolled | P1-T35 (D1.6) | ADR-0113 |
| Diff library | `similar` | `imara-diff`, hand-rolled Myers | P1-T27 (D1.3) | ADR-0109 |
| CSV/XML adapter crates | `csv`, `quick-xml` | manual parsing, other XML/csv crates | P1-T17 (D1.2) | D1.2 |
| Schema tooling | `schemars` + `jsonschema` | hand-rolled JSON (current `xtask`) | P1-T15/T52 | ADR-0007 |
| Reference parser technique | hand-written | `winnow`/`nom`/`pest`/`chumsky` | P1-T08 (D1.1) | ADR-0102 |
| Grapheme library | `unicode-segmentation` | `unicode-properties`, manual | P1-T06/T21 | ADR-0104 |
| Bidi shaping for debug reader | CSS `direction: rtl` | `unicode-bidi` crate | P1-T54 (D1.12) | D1.12 |
| Unicode source format policy | NFC as stored | NFD, NFKC, NFKD | P1-T04 (ADR-0104) | ADR-0104 |
| Hashing recipe contents | SHA-256 over a documented byte stream | include/exclude separators, add headers | P1-T23 **before** ADR-0108 freeze | ADR-0108 |
| DB **schema DDL** details | plan §6 | adjust columns/indexes before applying | P1-T12 (D1.5) | ADR-0106 |
| Basmala policy storage | edition metadata (per-surah) | fixed enum per edition | P1-T04 | ADR-0110 |

---

## 8. How To Change One Of These Choices (Procedure)

Changing a technology choice is allowed; changing it *silently* is not. The procedure:

1. **Check the status.** If the row is 🔒, it is not a Phase-1 change — open an ADR and a
   migration/version strategy. If it is 🟡, proceed.
2. **Write or amend an ADR** (`ADR-01nn`, phase-coded) with the §48 fields, especially
   *Accuracy implications*, *Religious-source implications*, and *Licensing implications*. Record
   the alternatives and why they were rejected.
3. **Add the crate** to `[workspace.dependencies]` in the root `Cargo.toml`, then reference it
   from the owning crate's `Cargo.toml` using `{ workspace = true }`.
4. **Update `xtask/allowlist.toml`.** Phase 1 creates new path edges
   (`quran-core -> domain`, `quran-corpus -> quran-core/domain/sources/storage`,
   `citations -> quran-core/domain`). Until the allowlist lists these crates, `cargo xtask
   arch-check` treats them as "allowed to depend on nothing" and **fails** the build — that is the
   intended fail-closed behaviour. Register the allowed workspace deps for each new crate.
5. **Respect I2.** Do not add `llm`, `embeddings`, `retrieval`, or vector-store deps to
   `quran-core`/`quran-corpus`, regardless of ADR.
6. **Run the gates:** `cargo xtask ci` (fmt, clippy `-D warnings`, test, deny, arch-check,
   migrate-check). `cargo deny` must accept the new crate's license.
7. **Record it** in `tasks.md` (the task that made the choice) and `done.md` if the choice was
   made as part of a completed task.

---

## 9. Integration Map by Phase-1 Crate

| Phase-1 crate | Tech it integrates | External libs (new in **bold**) |
|---|---|---|
| `quran-core` | pure domain; reference grammar; `QuranQuotation` | `domain`, `serde`, `thiserror`, **`unicode-segmentation`** |
| `quran-corpus` | intermediate format, adapters, tokenizer, validator, differ, importer logic | `quran-core`, `domain`, `sources`, `storage`, `serde`, `serde_json`, **`unicode-segmentation`**, **`unicode-normalization`**, **`similar`**, **`csv`**, **`quick-xml`**, `sha2` |
| `citations` | citation resolver v1, deep links, `verify_quotation` | `quran-core`, `domain`, `serde`, `thiserror`, `sha2` |
| `storage-sqlite` (extended) | Quran repositories, staging, activation transaction | `sqlx`, `storage`, `domain`, `quran-core`, `sha2`, `time` |
| `application` (extended) | `QuranReader`, caching, import/validate/activate services | `quran-core`, `quran-corpus`, `citations`, `storage`, `jobs`, **`lru`** |
| `jobs` (reused) | `quran.import` job with 13 checkpoints, cancel/resume | existing Phase-0 `jobs` |
| `server` (extended) | Quran read API v1, debug reader | **`axum`**, **`tower-http`**, `application`, `config`, `observability` |
| `cli` (extended) | `qai quran …`, `doctor --quran` | `application`, `clap`, `serde_json`, **`jsonschema`** (schema checks) |
| `testkit` (extended) | golden/edge/adversarial fixture harness | `tempfile`, `proptest`, **`jsonschema`** |
| `xtask` (extended) | `gen-schema` for `quran-edition-source.v1` | existing `xtask` |

**Bold** = a dependency not yet present anywhere in the workspace and therefore subject to the
"can change now" rules in §7.

---

## 10. Engine-Level Clarifications

- **Database engine:** SQLite, invoked through `sqlx`, not through a server process. There is no
  external database daemon in Phase 1. PostgreSQL is Phase 12.
- **Full-text / search engine:** none in Phase 1. SQLite FTS5 and any external index are Phase 2.
  Phase 1 must not create a search index; QV-028 exists specifically to assert that normalizing
  for a future index never alters stored canonical text.
- **Vector / embedding engine:** none in Phase 1 (I2). No `embeddings`/`retrieval`/vector-store
  crate may appear in `quran-core` or `quran-corpus`.
- **Graph engine:** none in Phase 1 (Phase 3).
- **Parser engine:** no parser generator. The reference grammar is a hand-written recursive
  descent parser (ADR-0102), and dataset adapters are hand-written per shape.
- **Job engine:** the Phase-0 in-process, DB-backed job system — no Redis, RabbitMQ, or other
  broker.
- **HTTP engine:** *open*. Today a hand-rolled TCP loop (Phase 0); recommended `axum`/
  `tower-http` for Phase 1 (see §4).
- **Hashing engine:** `sha2` (SHA-256) in-process; the algorithm tag inside `ContentHash` leaves
  room for `blake3` later without reinterpreting existing digests.

---

## 11. Summary

Phase 1 does **not** introduce exotic engines: it is SQLite through `sqlx` for storage, SHA-256
for hashing, `tokio` for async, `serde` for data, tokens and separators in plain SQL tables, and a
hand-written reference parser. The genuinely new integrations are a small, well-bounded set of
libraries — Unicode segmentation/normalization, a diff library, an LRU cache, CSV/XML adapters for
proving extensibility, schema tooling, and (the one real architectural choice) an HTTP framework
for the Quran read API. The database engine and driver are **already fixed by Phase 0** and cannot
be swapped in Phase 1; the schema DDL, however, can still be adjusted before migration `0010` is
applied. Every open choice has a "decide by" task and an ADR in §7, and every change follows the
procedure in §8 — recorded in an ADR, wired through `Cargo.toml` and `xtask/allowlist.toml`, and
gated by `cargo xtask ci`.
