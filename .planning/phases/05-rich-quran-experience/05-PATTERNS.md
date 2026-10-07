# Phase 5: Rich Quran Experience - Pattern Map

**Mapped:** 2026-10-07
**Files analyzed:** 24 (14 modified, 10 new)
**Analogs found:** 17 / 24 (SPA + TUI crate internals have no in-repo analog — use RESEARCH patterns)

> **Tracked-source note:** every analog path below is git-tracked source (verified with `git ls-files`). No `.gsd/capabilities/**` mirror paths are emitted.

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/tools/src/lib.rs` (mod) | model / contract | transform | same file `reproducibility()` + `content_hash_of` | exact |
| `crates/tool-registry/src/lib.rs` (mod) | service / registry | request-response | same file `get_ayah` / `get_context` / `attributed()` | exact |
| `crates/server/src/api.rs` (mod) | route / controller | request-response | `router()` + `debug_reader_handler` + `meta_from_tool` (same file) | exact |
| `crates/server/src/webassets.rs` (new) | route / static-asset handler | file-I/O (embedded) | `debug_font_handler` + `debug_reader_handler` (`crates/server/src/api.rs`) | role-match |
| `crates/cli/src/lib.rs` (mod) | controller / command | request-response | `Commands::Serve` dispatch arm (same file) | exact |
| `crates/tui/Cargo.toml` (mod) | config | — | `crates/server/Cargo.toml` | role-match |
| `crates/tui/src/lib.rs` (mod) | module root | — | `crates/application/src/lib.rs` (module inventory) | role-match |
| `crates/tui/src/app.rs` (new) | component / event loop | event-driven | **none in repo** (no ratatui/crossterm exists) | none |
| `crates/tui/src/palette.rs` (new) | component | event-driven | **none in repo** | none |
| `crates/tui/src/screens/*.rs` (new) | component / view | request-response + event-driven | **none in repo** | none |
| `crates/tui/src/main.rs` (new) | entrypoint | event-driven | `crates/cli/src/main.rs` (binary shape) | role-match |
| `xtask/src/parity.rs` (new) | test harness | batch (golden) | `xtask/src/ci.rs` + `xtask/src/arch.rs` | role-match |
| `xtask/src/main.rs` (mod) | config / dispatch | — | same file `Commands` enum + `run()` | exact |
| `xtask/src/ci.rs` (mod) | config | batch | same file step sequence | exact |
| `xtask/allowlist.toml` (mod) | config | — | `[server]` / `[cli]` / `[tools]` blocks (same file) | exact |
| `docs/08-api/quran-v1-openapi.json` (mod) | config / schema | — | `Meta` schema in same file | exact |
| `crates/server/tests/api.rs` (mod) | test | request-response | same file `test_state()` / `serve_router` / required-keys assertion | exact |
| `crates/cli/tests/quran.rs` + `*.trycmd` (mod) | test | batch (golden) | `crates/cli/tests/quran/graph_s1.trycmd` + runner `quran.rs` | exact |
| `web/package.json`, `vite.config.ts`, `tsconfig.json`, `index.html` (new) | config | — | **none in repo** (`web/` does not exist) | none |
| `web/src/main.tsx`, `api/client.ts` (new) | service / client | request-response | `Envelope<T>` DTO (`crates/server/src/api.rs`) | partial (types only) |
| `web/src/views/*.tsx` (new) | component | request-response | **none in repo** | none |
| `web/src/components/*.tsx` (new) | component | request-response | `AyahView` / `AttributedTranslation` (`crates/quran-core/src/view.rs`) | partial (contract only) |
| `web/src/tokens/layers.css` (new) | config / tokens | — | `debug_reader_handler` font-stack CSS (`crates/server/src/api.rs`) | partial |
| `web/vitest.config.ts` + smoke test (new) | test | — | **none in repo** | none |

---

## Pattern Assignments

### `crates/tools/src/lib.rs` (model/contract, transform)

**Analog:** same file — `reproducibility()` (lines 128-165), `content_hash_of` (123-126).

**Imports pattern** (lines 10-14):
```rust
use std::collections::BTreeMap;
use domain::{Confidence, ContentHash, HashAlgorithm, SemVer};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
```

**Core pattern — existing checksum (to extend)** (lines 122-150):
```rust
/// SHA-256 [`ContentHash`] of canonical JSON bytes.
pub fn content_hash_of(value: &serde_json::Value) -> ContentHash {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    ContentHash { algorithm: HashAlgorithm::Sha256, hex: sha256_hex(&bytes) }
}
// … in reproducibility():
let checksum = content_hash_of(&serde_json::json!({
    "tool": format!("{tool_name}@{tool_version}"),
    "query_hash": query_hash.hex,
    "editions": edition_versions,
    "sources": source_versions,
    "generation": corpus_generation,
}));
```

**D-14/D-15 change:** add a *required* `research_checksum: ContentHash` field to `ToolResult<T>` (struct at lines 54-83) and a `research_checksum(tool_name, tool_version, normalized_params, canonical_inputs, result_payload)` fn that reuses `domain::hashing::canonical_json_bytes` + `content_hash_of` with a `"qai-research-checksum-v1"` domain tag. **Do not** rewrite `canonical_json_bytes` (Pitfall 6 — invalidates every stored `ContentHash`).

**Test pattern** (lines 171-204): mirror `checksums_are_deterministic_and_sensitive` — build the checksum twice, assert equal; mutate the payload, assert `assert_ne!`.

---

### `crates/tool-registry/src/lib.rs` (service/registry, request-response)

**Analog:** same file — `get_ayah` (463-505), `get_context` (508-565), test helper `attributed()` (773-804).

**ToolResult construction literal to copy** (lines 478-503):
```rust
let result = ToolResult {
    tool_name: "quran.get_ayah".to_string(),
    tool_version: GET_AYAH_VERSION,
    query: query.clone(),
    normalization_rules: Vec::new(),
    edition_id: Some(meta.edition_id.clone()),
    edition_version: Some(meta.edition_version.clone()),
    canonical_references: vec![view.canonical.reference().to_string()],
    analysis_sources: vec![AnalysisSource { kind: "canonical".to_string(), reference: /* … */ }],
    results: vec![view],
    confidence: None,
    warnings: Vec::new(),
    execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
    reproducibility: reproducibility("quran.get_ayah", GET_AYAH_VERSION, &query, …),
};
```

**Change:** every one of the 12 `TOOL_NAMES` (lines 439-452) — 10 of which funnel through the `attributed()` helper after backend delegation — must populate the new field. The single-constructor approach noted in Pitfall 5 is the lazy path: add a `finish()` / `ResearchChecksum::compute(...)` helper and migrate the two direct `ToolResult { … }` literals (478, 541) plus the `attributed()` literal (778) rather than editing 12 call sites by hand — the other 10 tools delegate to the backend and already return through that shared shape.

**Pitfall:** `ToolResult` derives no `Default` (line 55) — adding a field breaks every literal constructor. Fix all in one commit.

---

### `crates/server/src/api.rs` (route/controller, request-response)

**Analog:** same file — `Meta`/`Envelope` (65-93), `meta_from_tool` (185-208), `debug_reader_handler` (2002-2068), `router()` (2112-2179), `serve()` (2182).

**Envelope DTO to extend** (lines 66-93):
```rust
pub struct Meta {
    pub edition: EditionMeta,
    pub corpus_generation: u64,
    pub canonical_reference: String,
    pub deep_link: String,
    pub execution_time_ms: f64,
    pub reproducibility: serde_json::Value,
    pub warnings: Vec<String>,
}
pub struct Envelope<T> { pub api_version: &'static str, pub data: T, pub meta: Meta }
```
Add `research_checksum: String` (or `Option<String>`) here, populate it in `meta_from_tool` (185-208) from `result.research_checksum`, and add it to `empty_meta()` (166-183).

**SPA-fallback wiring** — append at the **end** of `router()` (lines 2112-2179), after all `/api/v1/*` + `/healthz` routes and before/after the layers take effect; `.fallback(webassets::spa_fallback)`:
```rust
.route("/debug/read/{edition}/{surah}", get(debug_reader_handler))
.fallback(webassets::spa_fallback)              // NEW
.layer(tower_http::trace::TraceLayer::new_for_http())
// … existing layers …
.with_state(state)
```
Add the new module in `crates/server/src/lib.rs` (`pub mod api;` at line 6 → add `pub mod webassets;`).

**Static-byte response pattern to copy** (`debug_font_handler`, lines 2099-2108) — content-type + bytes response:
```rust
async fn debug_font_handler(axum::Extension(font): axum::Extension<DebugFont>) -> Response {
    (
        [("content-type", "font/woff2"), ("cache-control", "no-store"),
         ("x-content-type-options", "nosniff")],
        font.0.to_vec(),
    ).into_response()
}
```

**RTL/font-stack precedent to reuse** (`debug_reader_handler`, lines 2027-2040): system font stack `KFGQPC Uthmanic Script HAFS → Amiri Quran → Scheherazade New → Noto Naskh Arabic → Traditional Arabic → serif`, `lang="ar" dir="rtl"`, and **never** bundle a font (OD-04 gate — Pitfall 1). `escape_html` (2072-2084) is the HTML-escaping precedent the SPA must keep as text (no `dangerouslySetInnerHTML`).

**Error/status mapping to reuse:** `tool_status` (218-…) and the `Diagnostic` body (`ErrorBody`/`ErrorDetail`, 96-117).

---

### `crates/server/src/webassets.rs` (route/static handler, file-I/O)

**Analog:** `debug_font_handler` (2099-2108) + `debug_reader_handler` (2002-2068) — both serve in-memory bytes with an explicit content-type. This is the closest *repo* precedent; the `rust-embed` mechanics are new (RESEARCH Pattern 1 / Example 1).

**Shape (RESEARCH Example 1):**
```rust
use axum::body::Body;
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../../web/dist/"]
#[allow_missing = true] // keeps `cargo build` succeeding before `npm run build`
struct SpaAssets;

pub async fn spa_fallback(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if path.starts_with("api/") || path == "healthz" || path == "readyz" {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    let asset = SpaAssets::get(path).or_else(|| SpaAssets::get("index.html"));
    match asset {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], Body::from(file.data.into_owned())).into_response()
        }
        None => (StatusCode::NOT_FOUND, "SPA assets missing").into_response(),
    }
}
```
**Pitfall 3:** `#[derive(Embed)]` resolves the folder at compile time; `allow_missing = true` is mandatory for the Node-less Rust CI. Decide `web/dist` commit vs Node CI step (Assumption A6) before landing.

---

### `crates/cli/src/lib.rs` (controller/command, request-response)

**Analog:** same file — `Commands::Serve` arm (298-447), `Cli`/`Commands` enums (16-127).

**Command enum pattern** (lines 111-116):
```rust
/// Start the health server.
Serve {
    /// Bind address (must be loopback in Phase 0).
    #[arg(long, default_value = "127.0.0.1:8737")]
    bind: String,
},
```
Add a `Tui` variant alongside it, and a `Commands::Tui { … }` dispatch arm in `dispatch()` (218) that builds the shared services stack (see the `Serve` arm, lines 330-403 — `quran_cli::open_reader`, `quran_tools::ReaderToolBackend::registry_with_index_root`, `quran_search_api::SearchApiService::open`, `quran_lexicon_api::LexiconApiService::open`, `quran_graph_api::FileGraphBackend::structural`) and hands `Arc<dyn …>` handles to `tui::App`. **No HTTP, no shelling out** (D-12).

**JSON-vs-human output pattern** (quran.rs, lines 1669-1676): `if json { serde_json::to_string_pretty(&output.json) } else { print!("{human}") }` — reuse to expose `research_checksum` in both modes.

---

### `crates/tui/*` (component/view, event-driven)

**Analog:** **none in repo** — `rg ratatui|crossterm` returns nothing; `crates/tui/src/lib.rs` is a 4-line `//! Phase 9 placeholder`. Use RESEARCH Pattern 2 + Example 3. Label the crate's phase placeholder → Phase 5 while implementing.

**Data-access boundary (D-12) — analog: shared service traits.**
- `application::quran_graph_api::GraphBackend` (`crates/application/src/quran_graph_api.rs:696-708`) — `#[async_trait] pub trait GraphBackend: Send + Sync`.
- `application::quran_search_api::SearchBackend`, `quran_lexicon_api::LexiconBackend` — same shape (see `AppState` fields, `crates/server/src/api.rs:151-164`).
The TUI consumes these `Arc<dyn …>` handles; the `crates/cli` `Serve` arm (330-403) is the wiring blueprint.

**Runloop shape (RESEARCH Pattern 2):** `ratatui::init()` / `restore()`, `tokio::select!` over a tick interval and `crossterm::event::EventStream`, panic-hook restore.

**TUI screen test (RESEARCH Example 3):** `ratatui::backend::TestBackend::new(80, 24)` + `Terminal::draw` + `assert_buffer_lines`. Stub backends like `DisabledGraph` (`crates/server/tests/api.rs:579-677`) are the pattern for screens whose backend is absent (RAG debug → typed "unavailable", D-11).

**Security (Pitfall 11):** strip ANSI/control chars from dataset text before `ratatui` render — HTML-side analogue is `escape_html` (`crates/server/src/api.rs:2072-2084`).

---

### `xtask/src/parity.rs` + `xtask/src/main.rs` + `xtask/src/ci.rs` (test harness, batch)

**Analog:** `xtask/src/ci.rs` (whole file, 104 lines) and `xtask/src/main.rs` (`Commands` enum + `run()` dispatch, 21-75).

**Command registration pattern** (`main.rs:36-49`):
```rust
#[derive(Subcommand)]
enum Commands {
    ArchCheck,
    Ci,
    // …
}
fn run(cli: Cli) -> Result<(), anyhow::Error> {
    match cli.command {
        Commands::ArchCheck => arch::run(),
        Commands::Ci => ci::run(),
        // …
    }
}
```
Add `Parity` to `Commands`, `mod parity;`, and `Commands::Parity => parity::run()`.

**Step-sequence + fail-fast pattern to copy** (`ci.rs:9-29`):
```rust
fn run_cmd(cmd: &mut Command) -> Result<()> {
    let status = cmd.status().with_context(|| format!("failed to spawn {cmd:?}"))?;
    anyhow::ensure!(status.success(), "command failed: {cmd:?}");
    Ok(())
}
```

**Parity comparison unit (D-16, Pitfall 8):** compare the serialized `ToolResult` (minus `execution_time_ms`), not the HTTP `Envelope`. Three legs: (a) tool-registry/service call, (b) `router(state)` + `tower::ServiceExt::oneshot`, (c) CLI `qai quran … --json`. The server test's request pattern is the HTTP-leg template.

**CI insertion (Pitfall 3):** add the SPA build + parity step to the existing 9-step sequence (`ci.rs:18-102`); keep `.fallback` allow_missing so earlier Rust steps still `cargo build` without Node.

---

### `xtask/allowlist.toml` (config)

**Analog:** same file — `[cli]` (41-42), `[server]` (44-50), `[server.external.registry]` (166-169), `[tools]` (80-81).

**Pitfall 2 — the fail-closed gap:** there is **no `[tui]` block at all** (`rg '^\[tui\]'` returns nothing), so `crates/tui` is treated as `allow = []`. Must add:
```toml
[tui]
workspace = { allow = ["application", "config", "observability", "tools", "tool-registry", "quran-core", "domain"] }
[tui.external.registry]
allow = ["ratatui", "crossterm", "serde", "serde_json", "thiserror", "tokio"]
[tui.external.git]
allow = []
```
Plus extend in the **same commit**: `[server.external.registry]` += `rust-embed` (+ `mime_guess` if hand-rolled), `[cli]` workspace allow += `"tui"`. (Exact lists per RESEARCH §Recommended Project Structure; the toolchain reads live `cargo metadata`.)

---

### `docs/08-api/quran-v1-openapi.json` (schema)

**Analog:** `components.schemas.Meta` in the same file. Add `"research_checksum"` to `Meta.required` and to `Meta.properties`, mirroring the existing `canonical_reference`/`deep_link` string properties; add the citation deep-link route if it becomes a first-class endpoint. **Must stay in lock-step** with the assertion in `crates/server/tests/api.rs:778-789`.

---

### `crates/server/tests/api.rs` (test)

**Analog:** same file — `test_state()` (566-574), `serve_router()` (683-689), required-keys assertion (769-793).

**Required-keys assertion to update** (lines 778-789):
```rust
let meta_required = schemas["Meta"]["required"].as_array().unwrap();
for key in [
    "edition", "corpus_generation", "canonical_reference", "deep_link",
    "execution_time_ms", "reproducibility", "warnings",
] {
    assert!(meta_required.iter().any(|v| v == key), "Meta missing required {key}");
}
```
Add `"research_checksum"` here and to the OpenAPI schema above. Add an SPA-fallback test (`GET /read/…` → `index.html`; `GET /api/v1/unknown` → 404) using `router(test_state())` + `serve_router`.

---

### `crates/cli/tests/quran.rs` + `*.trycmd` (test, golden)

**Analog:** runner `crates/cli/tests/quran.rs` (`trycmd::TestCases` / `guard.run_segments`) and case file `crates/cli/tests/quran/graph_s1.trycmd`.

**Case-file format** (`graph_s1.trycmd`):
```console
$ qai db migrate
migrations applied; schema version [..]

```
Add a parity/checksum case (or a dedicated `xtask parity` case) asserting the `research_checksum` appears identically in `qai quran read --json` output.

---

### `web/**` (SPA — new)

**Analog:** **none** — no `web/` directory exists (`ls web` → absent). Use RESEARCH §Standard Stack (React 19.3.0 / Vite 8.3.3 / TS pinned ~6.x), Pattern 3 (layer tokens), Example 5 (layer-safe components), and the `Envelope<T>`/`Meta` DTO from `crates/server/src/api.rs:66-93` for the typed client. The `AyahView`/`AttributedTranslation` shape (`crates/quran-core/src/view.rs:20-106`) is the contract the reading view renders: `canonical: QuranQuotation` is Arabic-only and there is **no constructor path** for a translation into the canonical slot (ADR-0112 — enforce structurally in the component API).

---

## Shared Patterns

### Research checksum (D-14/D-15) — cross-cutting contract
**Source:** `crates/tools/src/lib.rs:122-165` (`content_hash_of`, `reproducibility`) + `crates/domain/src/hashing.rs:54-58` (`canonical_json_bytes`).
**Apply to:** `crates/tools`, `crates/tool-registry`, `crates/server` (Meta), `crates/cli` (output), `xtask/parity`, OpenAPI, server tests.
**Rule:** reuse `domain::hashing::canonical_json_bytes` + `ContentHash`; domain-separate as `"qai-research-checksum-v1"`. **Do not** modify `canonical_json_bytes` (Pitfall 6 — would invalidate every stored `ContentHash`).

### Shared services layer (D-12) — parity by construction
**Source:** `crates/application/src/quran_graph_api.rs:696` (`GraphBackend` trait) and the CLI `Serve` wiring (`crates/cli/src/lib.rs:330-403`).
**Apply to:** TUI, CLI, HTTP handlers. All three converge on `Arc<dyn …>` services from `application`; no surface may shell out or talk HTTP to another surface.

### Layer separation (ADR-0112 / D-08/D-09) — cross-cutting
**Source:** token/CSS vocabulary from RESEARCH Pattern 3; mirrored in TUI theme map + CLI labels.
**Apply to:** every reading/translation/annotation render. No prop/CSS/TUI-style path may place a translation or annotation in a canonical slot; `debug_reader_handler`'s RTL/system-font-stack (`crates/server/src/api.rs:2027-2040`) is the shipped precedent.

### Local-first / deny egress (D-04)
**Source:** `security.allow_network_egress = false`; the system-font-stack default (`crates/server/src/api.rs:2036-2040`).
**Apply to:** SPA build + runtime, TUI. No CDN, no Google Fonts, no runtime URL outside `/api/` or `/`. Bundled font is gated on OD-04 (owner decision — never close it).

### Golden-snapshot gate (D-16)
**Source:** `crates/server/tests/api.rs` (router + fake state), `crates/cli/tests/quran/*.trycmd` (CLI snapshots), `xtask/src/ci.rs` (command home).
**Apply to:** the cross-surface parity gate — one fixture → service + HTTP + CLI; assert normalized payload bytes AND checksum identical.

---

## No Analog Found

Files with no close match in the codebase (planner must use RESEARCH.md patterns instead):

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `crates/tui/src/app.rs` | component / event loop | event-driven | No ratatui/crossterm code exists anywhere in the repo (`crates/tui` is a placeholder) |
| `crates/tui/src/palette.rs` | component | event-driven | Same — no TUI code exists |
| `crates/tui/src/screens/*.rs` | component / view | request-response + event-driven | Same — no TUI code exists |
| `web/**` (all SPA sources) | component / service / config | request-response | No `web/` directory exists; no JS/TS in the workspace |
| `web/vitest.config.ts` + smoke test | test | — | No frontend test tooling exists |
| `crates/tui/src/main.rs` | entrypoint | event-driven | Binary shape is inferable from `crates/cli/src/main.rs`, but no ratatui entrypoint precedent |

---

## Metadata

**Analog search scope:** `crates/tools`, `crates/tool-registry`, `crates/server` (+ tests), `crates/cli` (+ tests), `crates/application`, `crates/domain`, `crates/quran-core`, `crates/quran-graph`, `crates/tui`, `xtask/`, `docs/08-api`.
**Files scanned:** 17 read in full/part; `rg` sweeps across `crates/*` and `xtask/*`.
**Tracked-source gate:** all analog paths verified git-tracked via `git ls-files`; no mirror paths emitted. `crates/server/src/webassets.rs` does not exist yet (correctly absent from `git ls-files`).
**Pattern extraction date:** 2026-10-07
