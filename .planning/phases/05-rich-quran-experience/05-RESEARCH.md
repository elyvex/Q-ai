# Phase 5: Rich Quran Experience — Research

**Researched:** 2026-10-07
**Domain:** Rust workspace + embedded React/Vite SPA + ratatui/crossterm TUI + typed result contract
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Web GUI Delivery Shape**
- **D-01:** Ship the Web GUI as a **single-page application** (not server-rendered HTML), consuming the versioned HTTP API (`/api/v1`) — **Reversibility:** costly.
- **D-02:** Build the SPA with **JavaScript + TypeScript, React + Vite** (not Rust/WASM). The `qai` binary stays Rust; the SPA is a separate build artifact — **Reversibility:** costly.
- **D-03:** **Embed the built SPA assets in the `qai` binary** and serve them same-origin with `/api/v1`. Not serve-from-disk, not dev-server-only — **Reversibility:** costly.
- **D-04:** The SPA must run **fully offline against a local `qai serve`**; no CDN or network egress by default.

**Reading View Layout**
- **D-05:** Primary reading layout is a **single centered RTL column with collapsible side panels** (navigation, translation, word inspector, research tools).
- **D-06:** **Translations are user-switchable among all three modes**: (a) inline beneath each ayah, (b) side-by-side columns, (c) toggleable side panel.
- **D-07:** **Word inspector is user-switchable among all three interaction modes**: (a) click/tap popover, (b) persistent side panel, (c) popover with a pin-to-panel action.
- **D-08:** **SC5 visual layer distinction uses all three approaches**: a **shared layer token set** (canonical / translation / annotation) as the baseline applied everywhere, **per-view styling**, **and user-configurable themes**.
- **D-09:** Canonical text, translations, and annotations remain **visually and structurally distinct in every view**; no view may render a translation or annotation in a canonical slot (ADR-0112).

**TUI Cockpit Depth**
- **D-10:** Ship an **ops cockpit now**: command palette, doctor/health, jobs, index & corpus status, plus reader + search, plus **graph navigation and annotation-review screens**.
- **D-11:** Build the **RAG debug screen now against the typed `Envelope`**, rendering a typed "RAG not configured / unavailable" state until Phase 8 wires retrieval. Do **not** stub retrieval or defer the screen.
- **D-12:** The TUI reads data through the **shared application services layer** (same services / typed tools the API and CLI use).
- **D-13:** TUI framework is **ratatui + crossterm** (PRD §25).

**Result Contract & Research Checksum**
- **D-14:** The **research checksum covers inputs + tool identity + result payload**: canonical inputs (refs / edition / `corpus_generation`), tool id + version, normalized parameters, and the result payload — computed with the frozen canonical-JSON + `sha256` recipe (ADR-0006) and domain-separated per ADR-0108 style.
- **D-15:** The checksum is a **required field on the shared `ToolResult` / `Envelope` contract** (extend `crates/tools`), not computed only at the presentation layer.
- **D-16:** Prove UI/API/CLI parity with a **cross-surface golden gate**: one shared fixture drives every tool through CLI, HTTP, and the UI service path and asserts **byte-identical payload + identical checksum**.
- **D-17:** Citation copy yields a **stable citation URN + deep link** (ADR-0111); opening it resolves and **re-verifies via `verify_quotation`**, with **mismatch as a hard failure**.

### the agent's Discretion
- Component library and styling approach within React (so long as the shared layer token set and offline constraint hold).
- Routing / deep-link URL scheme, navigation structure, and page composition within the single-column shell.
- SPA build/bundling specifics (Vite config, asset embedding mechanism, dev workflow) so long as the binary is self-contained and offline.
- TUI keybindings, palette command inventory, widget decomposition, and terminal theme mapping so long as ratatui/crossterm, the shared services layer, and the layer token set are preserved.
- Exact `research_checksum` encoding/length and field naming within the frozen hashing recipe.
- Which research tools appear in which surface first (ordering), so long as every in-scope tool reaches UI + API + CLI with parity.

### Deferred Ideas (OUT OF SCOPE)
- **Real multi-RAG retrieval, routing, reranking, and the populated RAG debug view** — Phase 8.
- **Interactive graph explorer beyond navigation/annotation-review** — later graph/retrieval work.
- **Streaming answers / research-query interface, server dashboard, auth/RBAC** — Phases 10–11.
- **Hadith/tafsir/isnad UI surfaces** — Phases 6–7.
- **Comparative-scripture views** — Phase 9.
- **Additional frontend niceties** (offline PWA install, keyboard power mode, export/print polish) — revisit after core flows land.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REQ-quran-display | Web GUI reading view, research view, citation interaction, visual canonical/translation/annotation distinction (PRD §13) | Standard Stack (React/Vite + rust-embed); Architecture Patterns 1 & 3 (SPA same-origin + CSS layer tokens); Pitfalls 1, 9, 12 (font gate, RTL/bidi, XSS); Code Examples 1 & 3 |
| REQ-quran-research-tools | First-class typed tools usable by UI/API/workflow/agent (PRD §11) | Existing 12-name registry read (tool-registry); every registered tool already has CLI + HTTP surface — Phase 5 closes UI reach + parity (D-16) |
| REQ-quran-result-contract | Every Quran tool result includes the stated result fields incl. a research checksum (PRD §12) | `ToolResult`/`ReproducibilityData` read verbatim; D-14/D-15 gap analysis (Pitfall 4); Code Example 4 |
| REQ-tui-cli | ratatui/crossterm TUI operational cockpit + consistent `qai` CLI tree (PRD §25–§25.4, §60, §78) | Standard Stack (ratatui 0.30.2 / crossterm 0.29); Architecture Pattern 2; TUI TestBackend validation; arch-check allowlist gap (Pitfall 2) |
</phase_requirements>

## Summary

Phase 5 is an **interface-and-contract layer over a fully-built engine**. Phases 1–4 shipped the canonical reader, search/linguistics, counting, graph, tool registry, versioned HTTP API, and CLI. Phase 5 adds three user-facing surfaces (React SPA, ratatui TUI) and completes one backend contract (the `research_checksum` on every tool result) plus a cross-surface parity gate. No canonical, linguistic, graph, or hashing *semantics* change.

The single highest-leverage finding: **the result contract already exists but does not satisfy D-14/D-15**. `crates/tools::ToolResult` carries a `reproducibility.checksum` computed by `reproducibility()`, but that checksum covers only `tool@version + query_hash + editions + sources + generation` — it **omits the result payload and normalized parameters**, and it is **not domain-separated** in the ADR-0108 sense. D-14 explicitly requires the payload *in* the checksum. The planner must treat "extend `crates/tools` with a required, payload-covering, domain-separated `research_checksum`" as a first-class workstream that ripples into every one of the 12 registered tools, the HTTP `Envelope`/`Meta`, the OpenAPI spec, the CLI JSON output, and the parity gate.

The second highest-leverage finding: **the SPA delivery shape collides with two existing hard gates**. (a) `rust-embed` embeds at compile time, so if `web/dist/` is absent the `qai` build fails — and the CI matrix is Rust-only with no Node step. (b) `xtask arch-check` fails closed: `crates/tui` has **no `[tui]` allowlist entry at all**, meaning it is currently constrained to *zero* workspace dependencies; wiring ratatui/crossterm + the shared `application` services layer requires an explicit allowlist + external-registry addition, and so does adding `rust-embed` to `[server]` and a `tui` edge to `[cli]`.

The third: **OD-04 is unanswered and directly gates PRD §13.1 ("High-quality Arabic font")**. Bundling any woff2 is an owner decision; the shipped default must be a system font stack (as the existing `debug_reader_handler` already does), with the bundled-font path recorded as a blocked owner gate — never closed by an agent.

**Primary recommendation:** Sequence the phase contract-first and gate-first: (1) research_checksum + tool contracts; (2) arch-check/deny/CI plumbing + the parity gate skeleton; (3) TUI via the shared services layer (ratatui 0.30.2, tested with `TestBackend`); (4) SPA build/embed + reading/research views with a CSS layer-token system; (5) citation deep-link + re-verify; (6) RAG-debug screen against the typed envelope with an honest "unavailable" state.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Arabic reading (RTL, ayah rendering, translation panels, word inspector) | Browser / SPA | — | Pure presentation over `/api/v1` JSON; `AyahView` already carries canonical + attributed translations + tokens |
| Citation copy / deep-link resolution / `verify_quotation` | API / Backend | Browser (link emission) | Verification must run against canonical rows (ADR-0111); the browser only emits the frozen URN/deep link |
| Research tools (search/root/lemma/family/graph/count) | API / Backend (shared `application` services) | Browser, TUI, CLI are thin callers | D-12: TUI, CLI, and API all converge on `crates/application` services so parity is structural |
| `research_checksum` | API / Backend (contract) | — | Required field on `ToolResult`; computed once in `crates/tools`, surfaced by every tier |
| Canonical/translation/annotation layer distinction | Browser (CSS tokens) + TUI (theme map) + CLI (labels) | — | D-08 shared token vocabulary is the anti-drift mechanism across all three |
| TUI cockpit (palette, doctor, jobs, index status, reader, search, graph nav, RAG debug) | Rust TUI (`crates/tui`) | Shared `application` services | D-12: no shelling out to CLI, no HTTP; call services directly |
| Embedded SPA asset serving | API / Backend (`crates/server`) | — | D-03: same-origin static fallback in the existing axum `router()` |
| Graph navigation / annotation-review views | Browser + TUI | API / Backend | Renders frozen Phase-4 `quran-graph-json-v1` + explanation payloads; no new graph semantics |
| RAG debug view | Both (Browser + TUI) | API / Backend (typed envelope) | D-11: renders typed "unavailable" until Phase 8; shape consumed by Phase 8 |

## Standard Stack

### Core — Rust

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ratatui | 0.30.2 | TUI widgets, layout, rendering, `init()`/`restore()`, `TestBackend` | The PRD-mandated TUI crate (PRD §25, D-13); MIT, MSRV 1.88.0 [CITED: https://crates.io/crates/ratatui / https://docs.rs/ratatui/latest] |
| crossterm | 0.29.0 | Terminal backend + event/input handling | ratatui's default backend (`crossterm_0_29` feature); re-exported as `ratatui::crossterm` so version unification is automatic [CITED: https://docs.rs/ratatui/latest/ratatui/index.html] |
| rust-embed | 8.12.0 | Embed the built `web/dist/` SPA into the `qai` binary at compile time | De-facto standard static embedding; MIT, MSRV 1.80; supports `debug-embed`, `compression`, `include-exclude`, `allow_missing` [CITED: https://crates.io/crates/rust-embed/8.12.0 / https://docs.rs/rust-embed/latest] |

### Core — Frontend (SPA)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| react + react-dom | 19.3.0 | SPA UI | Vite's first-class template; the ecosystem baseline [CITED: https://docs.rs — n/a; Context7 `/react/react`] |
| vite | 8.3.3 | SPA build tool + dev server | Context7 `/vitejs/vite`; `base: "./"` supported precisely for embedded deployment [CITED: https://vite.dev/config/shared-options] |
| typescript | 6.x (template-pinned) | Type safety for the SPA | create-vite `template-react-ts` pins `~6.0.2`; npm's latest tag is 7.0.2 — **pin to the template value, do not chase 7.x** [ASSUMED] |
| @vitejs/plugin-react | 6.1.2 | React fast-refresh + JSX transform | Standard companion to Vite for React |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `mime_guess` | crates.io latest | Content-Type for embedded asset responses | Only if hand-rolling the asset handler with `rust-embed` (not needed if using `ServeEmbed`) |
| `fuzzy` filtering | ratatui built-in / simple sub/prefix filter | TUI command palette "searchable by fuzzy text" (PRD §25.2) | Default to substring+prefix filtering; do not add a low-trust dep (see Audit) |
| Vitest + @testing-library/react | latest | SPA unit tests | Wave 0 if the plan wants frontend test coverage; the phase's hard evidence (parity) is Rust-side |
| insta | 1.x (already declared, unused) | Snapshot assertions | Optional for TUI buffer/JSON snapshots alongside `TestBackend` |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `rust-embed` + hand-rolled fallback | `axum-embed` 0.1.0 | Provides `ServeEmbed` (index/fallback/ETag/pre-compressed) but the crate is unchanged since 2023-12-17 (1 version, 9 commits). Viable, but the ~30-line custom handler removes the staleness risk. |
| `rust-embed` (compile-time) | `include_dir!` | Similar; `rust-embed` adds dev-mode FS reads + compression + `allow_missing`, which matters for the CI-without-Node problem |
| `rust-embed` | `tower_http::services::ServeDir` | Serves from disk only — violates D-03 (self-contained binary). Use only in a dev-only path. |
| React + Vite (D-02 locked) | Yew/Leptos (Rust/WASM) | Off the table by D-02; also bloats the binary and splits the frontend ecosystem |
| `tower-http` compression layer | `rust-embed` `compression` feature or pre-compressed `.br`/`.gz` sidecars | Optional; measure binary size before adding |

**Installation:**
```bash
# Rust (workspace Cargo.toml [workspace.dependencies] + per-crate)
cargo add ratatui@0.30.2 --package tui
cargo add rust-embed@8 --features mime-guess --package server
# crossterm comes via ratatui's default feature (re-exported)

# Frontend (repo-root web/)
npm create vite@latest web -- --template react-ts
cd web && npm install
```

**Version verification:** Verified this session.
- Rust registry versions: ratatui 0.30.2, crossterm 0.29.0, rust-embed 8.12.0, axum-embed 0.1.0 [CITED: crates.io / docs.rs pages fetched via WebSearch].
- npm: `npm view react version` → 19.3.0, `vite` → 8.3.3, `typescript` → 7.0.2, `@vitejs/plugin-react` → 6.1.2 [VERIFIED: npm registry].
- Local toolchain: `rustc 1.97.1`, `cargo 1.97.1`, `node v24.18.0`, `npm 12.2.0` [VERIFIED: shell probe].
- ratatui 0.30.2 requires Rust ≥ 1.88.0; rust-embed ≥ 1.80 — both satisfied by the pinned 1.97.1 [CITED: crates.io metadata].

## Package Legitimacy Audit

> Run via `gsd-tools query package-legitimacy check` this session.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| ratatui | crates | since 2023-02 | ~1.6M/wk | github.com/ratatui/ratatui | OK | Approved |
| crossterm | crates | since 2018-01 | ~3.7M/wk | github.com/crossterm-rs/crossterm | OK | Approved |
| rust-embed | crates | since 2017-03 | ~1.3M/wk | pyrossh.dev/repos/rust-embed | OK | Approved |
| axum-embed | crates | since 2023-12 | ~7.7k/wk | github.com/informationsea/axum-embed | OK | Approved but **avoid** — stale (1 version, 2023) and unnecessary |
| react | npm | 2013 | ~224M/wk | github.com/facebook/react | SUS (`too-new`) | Flagged — heuristic false positive (latest 19.3.0 was published 2026-09-09); pin exact version |
| react-dom | npm | 2013 | ~212M/wk | github.com/facebook/react | SUS (`too-new`) | Flagged — same false positive; pin exact version |
| vite | npm | 2020 | ~232M/wk | github.com/vitejs/vite | SUS (`too-new`) | Flagged — latest 8.3.3 published 2026-10-06; pin exact version |
| @vitejs/plugin-react | npm | 2020 | ~117M/wk | github.com/vitejs/vite-plugin-react | SUS (`too-new`) | Flagged — same; pin exact version |
| typescript | npm | 2012 | ~365M/wk | github.com/microsoft/TypeScript | OK | Approved |
| fuzzy-matcher | crates | 2015 | 6/wk | none | SUS (`low-downloads`, `no-repository`) | **Do not adopt** — implement simple filtering |

**Packages removed due to [SLOP] verdict:** none.
**Packages flagged as suspicious [SUS]:** `react`, `react-dom`, `vite`, `@vitejs/plugin-react` (crash-heuristic false positives on latest-publish date for packages with >100M weekly downloads — no slopsquat signal; still, the planner should add a `checkpoint:human-verify` immediately before the scaffold install and **pin exact versions**, not `^`). `fuzzy-matcher` is genuinely low-trust (6 downloads/wk, no repo) — do not add it.

*Note on npm packages: name-legitimacy rests on Context7 library resolution (`/vitejs/vite`, `/react/react`) + registry existence + the >100M/wk download signal. The `SUS` tags above are the seam's heuristic reacting to the latest-release timestamp; treat them as a "pin versions" instruction, not a slop alarm.*

## Architecture Patterns

### System Architecture Diagram

```
                         ┌─────────────────────────────── Browser ───────────────────────────────┐
                         │  React SPA (embedded in qai binary, served same-origin, offline)       │
                         │  ─ Reading view (RTL ayah + translation panels + word inspector)       │
                         │  ─ Research view (search/root/lemma/family/graph/count)                │
                         │  ─ Citation copy → URN + deep link                                     │
                         └───────────────┬───────────────────────────────┬────────────────────────┘
                                         │  fetch /api/v1/* (same origin, no CORS, no egress)
                                         ▼
  CLI `qai quran …` ──┐          ┌──────────────────────────────┐          ┌──────────────────┐
  CLI `qai tui`    ──┼─ calls ─►│  axum router() (crates/server)│◄─ calls ─┤  TUI (crates/tui)│
                     │          │  ├ /api/v1/quran/* (Envelope) │          │  ratatui+crossterm│
                     │          │  ├ /api/v1/quran/graph/*      │          │  palette / doctor │
                     │          │  ├ /api/v1/quran/citations/{id}          │  jobs / index     │
                     │          │  └ fallback → embedded SPA    │          │  graph nav        │
                     │          └───────────┬──────────────────┘          │  RAG debug view   │
                     │                      │  (D-12: NO http, NO shell)     └────────┬─────────┘
                     │                      ▼                                         │
                     └──────────►┌──────────────────────────────────────┐◄────────────┘
                                 │  crates/application — SHARED SERVICES │  ← parity is structural:
                                 │  quran_reader · quran_search_api ·    │    all three surfaces call
                                 │  quran_lexicon_api · quran_graph_api ·│    the SAME services
                                 │  quran_tools · quran_counting         │
                                 └──────────────┬───────────────────────┘
                                                ▼
                                 ┌──────────────────────────────────────┐
                                 │  crates/tools — ToolResult +          │
                                 │  research_checksum (NEW required field)│
                                 │  domain::ContentHash (sha256, ADR-0006)│
                                 └──────────────┬───────────────────────┘
                                                ▼
                                 SQLite (canonical / derived) — read-only on every Phase-5 path
```

**Trace the primary use case:** user opens a deep link `/read/hafs-uthmani@1.0.0/2:255?highlight=token:3` → axum fallback serves `index.html` → SPA router parses the path → `GET /api/v1/quran/ayahs/quran:hafs-uthmani@1.0.0:2:255` → `ReaderBackend` → `application::quran_reader` → `ToolResult<Vec<AyahView>>` with `research_checksum` → SPA renders RTL ayah + translation panel; the citation button copies `qai://quran/hafs-uthmani@1.0.0/2:255`; opening it re-resolves and calls `verify_quotation`, hard-failing on `Mismatch`.

### Recommended Project Structure

```
Q-ai/
├── crates/
│   ├── tools/            # ToolResult + ReproducibilityData + NEW research_checksum (contract owner)
│   ├── tool-registry/    # 12 registered tools — populate research_checksum on every result
│   ├── server/           # axum router(): add embedded-SPA fallback + citation deep-link route
│   │   └── webassets.rs  # NEW: #[derive(Embed)] struct over ../../web/dist + fallback handler
│   ├── tui/              # currently `//! Phase 9 placeholder` — implement the cockpit here
│   │   ├── app.rs        # App state + event loop (init/restore)
│   │   ├── screens/      # dashboard, reader, search, graph, rag_debug, doctor, jobs, index
│   │   └── palette.rs    # command palette
│   ├── cli/              # add `qai tui` verb + expose research_checksum in human/JSON output
│   └── application/      # NO new backend semantics; may add thin view-assembly helpers only
├── web/                  # NEW: SPA (not a Cargo member — keeps arch-check scoped to crates/*)
│   ├── index.html
│   ├── package.json      # pinned deps; `npm run build` → web/dist
│   ├── vite.config.ts    # base: './' (embedded deployment)
│   └── src/
│       ├── tokens/layers.css   # shared layer token set (canonical/translation/annotation) + themes
│       ├── api/                # typed client over /api/v1
│       ├── views/              # reading, research, citation, graph, rag-debug
│       └── components/         # AyahColumn, TranslationPanel, WordInspector, CitationCopy
├── xtask/                # add `parity` gate command; extend ci.rs
└── xtask/allowlist.toml  # add [tui], [tui.external.registry]; extend [server], [cli]
```

**Why `web/` at the repo root, not `crates/web`:** `xtask arch-check` and the Rust workspace treat only `crates/*` members; a root `web/` folder is a non-Cargo build artifact and does not need allowlist edges or workspace membership. It mirrors the existing root-level `fixtures/` and `docs/` pattern.

### Pattern 1: Same-origin embedded SPA fallback in the existing axum router (D-01/D-03/D-04)

**What:** A single `#[derive(Embed)]` struct over `web/dist/` plus a fallback service that serves `index.html` for client-side routes and returns 404 for unknown `/api/*` paths. API routes win because they are registered on the router before the fallback.

**When to use:** Always for the SPA in production; the dev workflow may proxy Vite's dev server to `qai serve`.

**Example:**
```rust
// crates/server/src/webassets.rs  (new)
use axum::body::Body;
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../../web/dist/"]
#[allow_missing = true] // keeps `cargo build` succeeding before `npm run build` has run
struct SpaAssets;

/// Fallback: API/health paths must 404 (never be swallowed by index.html).
pub async fn spa_fallback(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if path.starts_with("api/") || path == "healthz" || path == "readyz" {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    // Client-side routes (/read/..., /search, ...) resolve to index.html.
    let asset = SpaAssets::get(path)
        .or_else(|| SpaAssets::get("index.html"));
    match asset {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], Body::from(file.data.into_owned()))
                .into_response()
        }
        None => (StatusCode::NOT_FOUND, "SPA assets missing").into_response(),
    }
}
```
Wire with `.fallback(spa_fallback)` at the **end** of `router()` — after all `/api/v1/*` and `/healthz` routes are registered [CITED: axum static-file-server example; rust-embed docs].

### Pattern 2: ratatui application lifecycle + shared-services data access (D-10/D-12/D-13)

**What:** `ratatui::init()` / `ratatui::restore()` (0.30) with a panic hook that restores the terminal; a `tokio::select!` event loop multiplexing a render tick and `crossterm::event::EventStream`. Data comes from `Arc<dyn Backend>` traits implemented by `crates/application` — **not** from HTTP or `std::process::Command`.

**When to use:** The whole `crates/tui` crate.

**Example:**
```rust
// crates/tui/src/main.rs (shape)
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let terminal = ratatui::init();
    let result = App::new(services).run(terminal).await; // services: Arc<dyn ...> from application
    ratatui::restore();
    result
}

async fn run(mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
    let period = Duration::from_millis(100);
    let mut tick = tokio::time::interval(period);
    let mut events = crossterm::event::EventStream::new();
    while !self.should_quit {
        tokio::select! {
            _ = tick.tick() => terminal.draw(|f| self.render(f))?,
            Some(Ok(ev)) = events.next() => self.handle_event(ev).await?,
        }
    }
    Ok(())
}
```
[Source: ratatui README + async-github example, https://github.com/ratatui/ratatui]

### Pattern 3: Shared layer-token set + themes for SC5 visual distinction (D-08/D-09)

**What:** A single CSS custom-property vocabulary shared by the SPA and mirrored as a colour/style map in the TUI and as labels in the CLI. Themes override the token values via a `data-theme` attribute; components never hard-code colours.

**Example:**
```css
/* web/src/tokens/layers.css */
:root {
  --layer-canonical-fg: #1b1b1f;
  --layer-canonical-bg: #ffffff;
  --layer-translation-fg: #334155;
  --layer-translation-bg: #f1f5f9;   /* visually distinct from canonical */
  --layer-annotation-fg: #7c2d12;
  --layer-annotation-bg: #fff7ed;
  --layer-canonical-accent: #0f766e; /* canonical-only used for the Arabic column */
}
[data-theme="dark"]   { --layer-canonical-bg: #0b1020; --layer-canonical-fg: #e5e7eb; /* … */ }
[data-theme="sepia"]  { /* … */ }
[data-theme="contrast"] { /* … */ }
```
Apply strictly: canonical text lives only inside an element using `--layer-canonical-*`; a translation never receives a canonical token (enforces ADR-0112 structurally in CSS **and** in the component API). [CITED: MDN/CSS-Tricks design-token pattern]

### Pattern 4: Domain-separated research checksum (D-14/D-15)

**What:** Extend the `crates/tools` contract with a required `research_checksum` computed over a canonical JSON object with a `qai-research-checksum-v1` domain tag, tool id+version, normalized params, canonical inputs, and the result payload — reusing `domain::ContentHash` (sha256).

See Code Example 4 for the shape. Reuse the existing `ContentHash` type and the ADR-0108 "domain tag + canonical serialization" discipline; **do not** invent a new digest format. [CITED: ADR-0006, ADR-0108]

### Anti-Patterns to Avoid

- **Serving the SPA from disk in production** — violates D-03 and the "single self-contained binary" guarantee.
- **Loading fonts from a CDN (Google Fonts, QPC CDN)** — violates D-04 (`security.allow_network_egress = false`); the QUL/QF tutorials recommend CDN fonts, which this project must not do.
- **`dangerouslySetInnerHTML` for translation/tafsir text** — the Quran Foundation tutorial explicitly recommends it because translation text may contain footnote HTML; that is an XSS vector from an untrusted dataset (Phase 6+). Render as text.
- **Rendering a translation or annotation into a canonical slot** — violates ADR-0112/D-09; there must be no component prop or CSS path that allows it.
- **Rendering a truncated graph as "no path"/absence** — ADR-0217 rule 3; surface `truncated` + `incomplete_reason` verbatim.
- **TUI shelling out to `qai` or talking HTTP to the server** — violates D-12 and makes parity unprovable.
- **Business logic in `crates/server` or `crates/tui`** — `.agent/coding-rules.md`: "No domain code depends on `api`, `server`, `tui`, `cli`, or any concrete provider."
- **Chasing TypeScript 7.x when the create-vite template pins 6.x** — unverified ecosystem compatibility.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Static asset embedding + SPA fallback | A `build.rs` that generates `include_bytes!` maps | `rust-embed` `#[derive(Embed)]` | Handles recreation, compression, `allow_missing`, dev-FS mode; ~30 lines of handler vs a bespoke codegen |
| MIME type detection | A hard-coded extension→type table | `mime_guess` (via `rust-embed` `mime-guess` feature) | Correct for the long tail of asset extensions |
| Canonical JSON + SHA-256 | A new serialization/hash helper | `domain::ContentHash` + the ADR-0108 domain-separation discipline | ADR-0006 is frozen; a second digest format breaks reproducibility |
| Citation identity / deep links / URNs | String formatting in the SPA and TUI | `citations::deep_link` / `citations::citation_urn` (frozen per `quran-citation-spec.md`) | Deep link/URN formats are frozen; duplicating them guarantees drift |
| Quotation verification | Client-side string comparison | `application` verifier → `citations` resolver (`verify_quotation`) | Stored hash is a comparison baseline, never a substitute for re-reading authoritative rows |
| TUI tables/lists/gauges/tabs | Custom cell buffers | ratatui `Table`, `List`, `Gauge`, `Tabs`, `Clear` | Battle-tested layout/state handling |
| RTL/bidi layout | Manual character reordering | `dir="rtl"` + CSS logical properties + Unicode bidi | Bidi is the hardest part of Arabic rendering; browsers implement UAX#9 |
| Graph JSON parsing/rendering | A bespoke graph model | The frozen `quran-graph-json-v1` envelope (`crates/quran-graph/src/export.rs`) | Format + truncation semantics are a contract |
| Design token system | Scattered hex literals | CSS custom properties on `:root` + `[data-theme]` | One change propagates everywhere; enables the four required themes |

**Key insight:** Every hard problem in this phase already has a frozen contract or a mature crate. The value Phase 5 adds is *wiring and consistency*, not new algorithms — so the plan should spend its complexity budget on the cross-surface parity gate and the layer-token discipline, which are the two things that are genuinely easy to get subtly wrong.

## Common Pitfalls

### Pitfall 1: OD-04 gates the "high-quality Arabic font" requirement
**What goes wrong:** The plan bundles a woff2 (Amiri Quran, QPC Hafs, etc.) to satisfy PRD §13.1, silently closing an owner decision.
**Why it happens:** Every Quran web-app tutorial says "add a Quran font"; the requirement literally names one.
**How to avoid:** Ship the **system font stack** as the default (the existing `debug_reader_handler` already uses `KFGQPC Uthmanic Script HAFS → Amiri Quran → Scheherazade New → Noto Naskh Arabic → Traditional Arabic → serif`). Record bundled-font delivery as a blocked OD-04 owner gate with the exact closing action, exactly as Phases 3/3.5/4 did. Never close OD-04.
**Warning signs:** A committed `.woff2` under `web/` with no OD-04 answer in `decisions-needed.md`.
**Evidence:** OD-04 is `🔴 unanswered`: *"Which web font ships with the debug reader (bundled `@font-face`), and is its license cleared for redistribution?"* [VERIFIED: `docs/05-followups/decisions-needed.md:82-91`].

### Pitfall 2: `xtask arch-check` fails closed — `[tui]` has no allowlist entry
**What goes wrong:** Adding `ratatui`/`crossterm`/`application` to `crates/tui` (or `rust-embed` to `crates/server`, or a `tui` edge to `crates/cli`) fails `cargo xtask arch-check` in CI step 5/9.
**Why it happens:** A crate not listed in `xtask/allowlist.toml` is treated as `allow = []` — zero workspace deps. `crates/tui` is absent entirely.
**How to avoid:** Add `[tui]`, `[tui.external.registry]`, and `[tui.external.git]` blocks; extend `[server.external.registry]` with `rust-embed` (+ `mime_guess` if used) and `[cli]` workspace allow with `"tui"`. The toolchain detects these from live `cargo metadata` (CI step). Do this in the same commit as the dependency addition.
**Warning signs:** `arch-check` green locally because the new dep was added to a crate that already had a matching allow entry; red in CI.
**Evidence (verbatim):**
- `[cli]` allow = `["application", "config", "observability", "server"]` [VERIFIED: `xtask/allowlist.toml:41-42`]
- `[server]` allow = `["application", "config", "observability", "tool-registry", "tools", "storage", "citations", "quran-core"]` [VERIFIED: `xtask/allowlist.toml:44-50`]
- `[tools]` allow = `["quran-core", "domain"]` [VERIFIED: `xtask/allowlist.toml:80-81`]
- No `[tui]` block exists (confirmed by exhaustive `rg '^\[tui\]'` returning nothing).

### Pitfall 3: `rust-embed` compiles at build time and the CI matrix has no Node
**What goes wrong:** (a) `cargo build` fails if `web/dist/` does not exist; (b) CI step 3 (`cargo test --workspace`) and step 9 (`cargo build --bin qai`) run with no Node setup, so `web/dist` is never built.
**Why it happens:** `#[derive(Embed)]` resolves the folder at compile time; the repo's CI is Rust-only.
**How to avoid:** Use `#[allow_missing = true]` so the crate compiles with an empty asset set, and add the SPA build as an explicit step (or make the parity gate build it). Decide deliberately: commit `web/dist/` (simple, binary-in-git) **or** add a Node setup + `npm ci && npm run build` before the Rust steps. Do not rely on the developer's machine having run `npm run build`.
**Warning signs:** `qai serve` returns "SPA assets missing" in CI-built artifacts; `cargo build` errors with a missing-folder message.
**Evidence:** `rust-embed` docs: *"In debug and when `debug-embed` feature is not enabled, the folder path is resolved relative to where the binary is run from. In release or when `debug-embed` is enabled, the folder path is resolved relative to where `Cargo.toml` is."* [CITED: https://crates.io/crates/rust-embed/8.12.0]. CI is 9 Rust steps [VERIFIED: `xtask/src/ci.rs:15-104`].

### Pitfall 4: The existing checksum does not satisfy D-14/D-15
**What goes wrong:** The plan assumes the contract is "done" because `ReproducibilityData.checksum` exists, then fails SC3 ("identical results + research checksum") because the checksum omits the payload.
**Why it happens:** The existing `reproducibility()` computes `checksum` over `{tool, query_hash, editions, sources, generation}` only.
**How to avoid:** Add a required `research_checksum` covering **inputs + tool identity + normalized parameters + result payload**, domain-separated per ADR-0108, and populate it in all 12 registered tools. Decide whether it *replaces* or *supplements* `reproducibility.checksum` (recommend: add the new field and keep the old for continuity; the agent's discretion covers naming/encoding).
**Warning signs:** Two different payloads for the same query hash produce the same checksum.
**Evidence (verbatim):**
```
    let checksum = content_hash_of(&serde_json::json!({
        "tool": format!("{tool_name}@{tool_version}"),
        "query_hash": query_hash.hex,
        "editions": edition_versions,
        "sources": source_versions,
        "generation": corpus_generation,
    }));
```
[VERIFIED: `crates/tools/src/lib.rs:144-150`]. The result payload is not in that object; `ToolResult` has no `research_checksum` field [VERIFIED: `crates/tools/src/lib.rs:54-83`].

### Pitfall 5: `ToolResult` has no `Default`; adding a required field breaks every constructor
**What goes wrong:** Adding `research_checksum: ContentHash` to `ToolResult<T>` breaks every literal construction site (12 tools in `tool-registry`, service wrappers in `application`, and server test fakes).
**Why it happens:** `ToolResult` derives only `Debug, Clone, PartialEq, Serialize, Deserialize` — no `Default`, and it is built field-by-field in each tool.
**How to avoid:** Introduce a single constructor/`finish()` helper on `ToolResult` (or a `ResearchChecksum::compute(...)`) and migrate all sites in one pass; add the field to the OpenAPI `Meta`/`Envelope` schemas and to the server test's required-keys assertion.
**Warning signs:** `clippy`/compiler errors at every `ToolResult { … }` literal; `openapi_spec_schemas_resolve_and_cover_json_responses` fails on the required-keys list.
**Evidence:** server test asserts required keys `["edition","corpus_generation","canonical_reference","deep_link","execution_time_ms","reproducibility","warnings"]` on `Meta` [VERIFIED: `crates/server/tests/api.rs:778-789`]; `Meta` field list [VERIFIED: `crates/server/src/api.rs:66-82`].

### Pitfall 6: `canonical_json_bytes` does not actually normalise to NFC
**What goes wrong:** D-14 says "frozen canonical-JSON recipe (ADR-0006)"; the implementation named `canonical_json_bytes` is a plain `serde_json::to_vec` and applies **no** NFC normalization, despite its doc comment.
**Why it happens:** ADR-0006 mandates "NFC prior to serialization", but the function body never calls `unicode_normalization`. Determinism currently holds only because `serde_json`'s default map ordering is sorted (`preserve_order` is not enabled anywhere).
**How to avoid:** Reuse the existing function as-is for the checksum (it *is* deterministic for the values hashed), and record the NFC gap as an assumption/open question. Do **not** "fix" `canonical_json_bytes` in this phase — changing it invalidates every stored `ContentHash` (ADR-0006 "Reversal Cost: High") and requires a global re-hash migration.
**Warning signs:** A plan that "adds NFC normalization to the checksum helper" without an ADR.
**Evidence (verbatim):**
```
pub fn canonical_json_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, HashingError> {
    let bytes =
        serde_json::to_vec(value).map_err(|e| HashingError::SerializationFailed(e.to_string()))?;
    Ok(bytes)
}
```
[VERIFIED: `crates/domain/src/hashing.rs:54-58`] (doc comment claims "NFC-preserving" at line 52).

### Pitfall 7: The citation deep link contains `@` and `:` and must survive client routing
**What goes wrong:** `/read/{slug}@{version}/{surah}:{ayah}` is used directly as a browser path; if the SPA uses a hash-less history router, a hard refresh hits the server and 404s unless the fallback serves `index.html`.
**Why it happens:** The deep-link format is frozen with reserved-looking characters; the server must treat it as an opaque client route.
**How to avoid:** Rely on the SPA fallback (Pattern 1) for any non-`/api/`, non-`/health*` path; parse the deep link inside the SPA. Also ensure the `highlight=token:{n}` query param survives.
**Warning signs:** Deep link 404s on refresh but works via in-app navigation.
**Evidence:** Deep link = `/read/{slug}@{version}/{surah}:{ayah}`; URN = `qai://quran/{slug}@{version}/{surah}:{ayah}` [CITED: `docs/07-technical/quran-citation-spec.md:6-15`, read this session].

### Pitfall 8: Cross-surface parity is not the same as "parity with the CLI"
**What goes wrong:** The gate compares CLI output to CLI output (tautology) or to HTTP output that is a different serialization (`Meta` is `serde_json::Value`), so "byte-identical" fails for formatting-only reasons.
**Why it happens:** D-16 requires one fixture through CLI + HTTP + the UI service path. The "UI service path" is unspecified; HTTP envelopes add `meta` that tool results lack, and timing fields (`execution_time_ms`) are nondeterministic.
**How to avoid:** Define the comparison unit explicitly — compare the **`ToolResult` serialization** (excluding `execution_time_ms`), not the HTTP `Envelope`, across: (a) `tool-registry` call (the code path both HTTP and SPA reach), (b) CLI `--json` output, (c) HTTP handler's `data` field. Normalise the timing field. Land the gate as an `xtask` command so it is a repeatable check.
**Warning signs:** A gate that passes but whose "UI" leg is actually the HTTP leg duplicated.
**Evidence:** `Meta.reproducibility` is untyped `serde_json::Value` and `execution_time_ms` is wall-clock [VERIFIED: `crates/server/src/api.rs:66-82`, `185-207`].

### Pitfall 9: Arabic rendering is bidi + font-fallback, not just `dir="rtl"`
**What goes wrong:** Arabic renders with wrong ligatures or falls back to a Latin font; Quranic marks (U+06D6…U+06ED) show as boxes.
**Why it happens:** Quranic marks require a Quran-capable font; browsers differ on Arabic ligature shaping (documented Chrome vs Firefox differences); mixed Latin/Arabic (translation panel next to Arabic) triggers bidi reordering surprises.
**How to avoid:** Set `dir="rtl"` + `lang="ar"` on the Arabic container; use CSS logical properties (`margin-inline-start`, etc.); rely on the browser's UAX#9 bidi (do not reorder manually); provide a font fallback chain; for purely Arabic blocks consider `unicode-bidi: isolate` on the block. Keep canonical Arabic in its own element separate from translation.
**Warning signs:** Mark rendering as tofu; punctuation jumping to the line start in mixed content.
**Evidence:** W3C Arabic layout requirements; QF/QUL font-rendering tutorials both emphasise fallback chains + `dir="rtl"` / `unicode-bidi` [CITED: https://www.w3.org/TR/alreq, QF API docs]. (No font decision — see Pitfall 1.)

### Pitfall 10: TUI Arabic/RTL is best-effort; do not over-invest
**What goes wrong:** The plan spends effort on terminal bidi that terminals cannot do.
**Why it happens:** PRD §25 lists Arabic/RTL as TUI-required "as far as terminal capabilities allow".
**How to avoid:** Render Arabic in the TUI where it helps (search hits, token inspection) but treat the Web GUI as the authoritative reading experience (PRD §25/§25.1). Do not hand-roll bidi.
**Evidence:** PRD §25: *"The TUI must support Arabic and right-to-left text as far as terminal capabilities allow. The Web GUI is the authoritative rich Quran reading experience when terminal rendering is insufficient."* [CITED: `docs/01-requirements/requirements.md:1785`, read this session].

### Pitfall 11: Terminal escape-sequence injection from dataset text (TUI security)
**What goes wrong:** Untrusted dataset text containing ANSI/control sequences is written straight to the terminal, manipulating the operator's screen.
**Why it happens:** `crossterm`/`ratatui` render the characters you give them; a mangled dataset can carry `\x1b[...]`.
**How to avoid:** Sanitize control characters out of any dataset-derived string before rendering in the TUI (the existing `escape_html` in the debug reader is the HTML-side analogue).
**Warning signs:** No sanitization step in the TUI render path.

### Pitfall 12: `security.allow_network_egress = false` is a hard constraint
**What goes wrong:** The SPA build pulls a font or the app fetches from a CDN at runtime; offline mode breaks.
**Why it happens:** Tutorials assume internet.
**How to avoid:** Bundle everything; no Google Fonts, no QPC CDN, no external map/tile libs. Serve fonts (if bundled, post-OD-04) and JS from the binary.
**Warning signs:** Any runtime URL not starting with `/api/` or `/`.
**Evidence:** Local-first / deny-egress constraint [CITED: `docs/01-requirements/requirements.md` §25.13; `AGENTS.md` key principles; `.agent/coding-rules.md`].

## Code Examples

Verified patterns from official sources and this repo.

### Example 1: Registering the SPA fallback in the existing router
```rust
// crates/server/src/api.rs — appended to `router()` *after* all API routes
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(|| async { "ready" }))
        // … all /api/v1/... routes (unchanged) …
        .route("/debug/read/{edition}/{surah}", get(debug_reader_handler))
        .fallback(webassets::spa_fallback) // NEW — client routes → index.html; /api/* → 404
        .layer(tower_http::trace::TraceLayer::new_for_http())
        // … existing layers …
        .with_state(state)
}
```
[Source: current `router()` at `crates/server/src/api.rs:2112-2179` (read this session); axum SPA-fallback pattern]

### Example 2: Cross-surface parity gate (one fixture, three paths, one checksum)
```rust
// xtask/src/parity.rs (new) — sketch
// One fixture drives every registered tool through:
//   (a) the shared service path (tool-registry call) — the code HTTP and the SPA reach
//   (b) HTTP: `router(state)` + tower::ServiceExt::oneshot(fixture request)
//   (c) CLI: `qai quran <verb> --json` against the same DB
// Assert: normalized payload bytes identical AND research_checksum identical.
fn normalize(v: &serde_json::Value) -> Vec<u8> {
    let mut v = v.clone();
    // strip nondeterministic timing before comparison
    if let Some(obj) = v.as_object_mut() { obj.remove("execution_time_ms"); }
    domain::hashing::canonical_json_bytes(&v).unwrap()
}
```
[Source: `crates/server/tests/api.rs` uses `router()` with a fake `AppState` (read this session); `crates/cli/tests/quran/*.trycmd` is the CLI snapshot pattern; `xtask` is the CI-command home]

### Example 3: TUI screen test with `TestBackend`
```rust
use ratatui::{backend::TestBackend, Terminal};

#[test]
fn dashboard_renders_palette_hint() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| render_dashboard(f, &state())).unwrap();
    terminal.backend().assert_buffer_lines([
        // one &str per terminal row; assert the full screen
    ]);
}
```
[Source: ratatui `TestBackend::assert_buffer_lines` + `Buffer::with_lines`, https://github.com/ratatui/ratatui]

### Example 4: Domain-separated `research_checksum`
```rust
// crates/tools/src/lib.rs — new (shape; naming/encoding is the agent's discretion)
const RESEARCH_CHECKSUM_DOMAIN: &str = "qai-research-checksum-v1";

/// Hash inputs + tool identity + normalized params + result payload, domain-separated.
pub fn research_checksum(
    tool_name: &str,
    tool_version: SemVer,
    normalized_params: &serde_json::Value,
    canonical_inputs: &serde_json::Value, // refs / edition / corpus_generation
    result_payload: &serde_json::Value,
) -> Result<ContentHash, HashingError> {
    let canonical = serde_json::json!({
        "domain": RESEARCH_CHECKSUM_DOMAIN,
        "tool": format!("{tool_name}@{tool_version}"),
        "params": normalized_params,
        "inputs": canonical_inputs,
        "payload": result_payload,
    });
    let bytes = domain::hashing::canonical_json_bytes(&canonical)?; // reuse ADR-0006 helper
    Ok(ContentHash { algorithm: HashAlgorithm::Sha256, hex: sha256_hex(&bytes) })
}
```
[Source: reuses `domain::canonical_json_bytes` + `ContentHash` (read this session); ADR-0108 domain-tag discipline]

### Example 5: CSS layer-token separation enforced against ADR-0112
```tsx
// Canonical text has NO prop that accepts a translation; the component API cannot violate ADR-0112.
function CanonicalAyah({ canonical }: { canonical: { arabicText: string; reference: string } }) {
  return <p className="ayah-canonical" dir="rtl" lang="ar">{canonical.arabicText}</p>;
}
function TranslationPanel({ translation }: { translation: { text: string; translator: string } }) {
  return <aside className="ayah-translation">{translation.text}</aside>; // styled with --layer-translation-*
}
```
[Source: `AyahView.canonical` is a `QuranQuotation` (Arabic only) with no translation path [VERIFIED: `crates/quran-core/src/view.rs:96-106`] and `AttributedTranslation` has no constructor without translator + edition [VERIFIED: `crates/quran-core/src/view.rs:29-45`].]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `ratatui` monolith pre-0.30 | ratatui 0.30 modular workspace (`ratatui-core`, `ratatui-widgets`, `ratatui-crossterm`) with `init()`/`restore()`/`run()` helpers | 2025-12 (v0.30.0) | Use `ratatui` (app crate), not `ratatui-core`; crossterm version selected by feature, re-exported |
| Manual `Terminal::new(CrosstermBackend::new(io::stdout()))` | `ratatui::init()` + panic-hook `restore()` | 0.28.1 / 0.30.0 | Fewer terminal-corruption bugs |
| `OutDir` + `base: /` for embedded apps | Vite `base: "./"` (relative) for embedded/no-known-base deployments | Vite 5+ docs current | Asset URLs resolve relative to `index.html`; `import.meta` required |
| Hand-rolled static embedding via `include_str!` | `rust-embed` derive with `debug-embed`/`allow_missing` | long-standing | `allow_missing` is the key for CI-without-Node |
| Torch/`datasets`-style heavy frontends | Vite + React 19 with `@vitejs/plugin-react` | current | — |

**Deprecated/outdated:**
- `axum-frontend` (0.1.3, first published ~2 days ago): too new to adopt for a frozen-contract phase. Prefer `rust-embed` + a small handler.
- `axum-embed` (0.1.0, 2023): usable but stale; the handler is ~30 lines, so owning it removes the risk.
- `serde_json::to_vec` for "canonical" JSON: nominally canonical but does **not** apply the ADR-0006 NFC step (see Pitfall 6).

## Assumptions Log

> List all claims tagged `[ASSUMED]`. Planner and discuss-phase use this to identify decisions needing user confirmation.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | TypeScript should be pinned to the create-vite template value (~6.x), not npm's `latest` (7.0.2) | Standard Stack | TS 7 could be fine; chasing it risks tooling incompatibility (`vite-tsconfig-paths` docs note TS 7 peer-dep concerns). Mitigation: pin the scaffolded version. |
| A2 | A root-level `web/` folder (non-Cargo member) is the right home for the SPA | Project Structure | If placed under `crates/`, it needs workspace membership + allowlist handling; low risk either way but affects arch-check. |
| A3 | The "UI service path" in D-16 means the shared `tool-registry`/`application` call the HTTP handler wraps | Pitfall 8 / Open Q2 | If it means something else (e.g. a JS-side harness), the gate design changes. Needs confirmation. |
| A4 | `research_checksum` should supplement (not replace) the existing `reproducibility.checksum` | Pitfall 4 | Replace vs supplement changes the contract surface and the OpenAPI schema; both are defensible. |
| A5 | Vitest is the SPA test framework if frontend tests are added | Validation Architecture | Alternative is Playwright/component testing; only matters if the plan adds frontend tests. |
| A6 | Committing `web/dist/` vs adding Node to CI is undecided | Pitfall 3 | Affects repo size vs CI complexity; a deliberate choice is required before the embed lands. |

**If this table is not empty:** A1–A6 need a decision (mostly the plan/agent's discretion; A3/A4 worth a direct confirmation).

## Open Questions

1. **How is the SPA built and embedded across local dev and CI (no Node in the Rust matrix)?**
   - What we know: `rust-embed` compiles at build time; CI is 9 Rust steps; `node v24` is available locally.
   - What's unclear: commit `web/dist/` vs add a Node step to `xtask ci` vs a `web`-feature-gated embed.
   - Recommendation: use `#[allow_missing = true]` + add an explicit SPA-build step to `xtask ci` (and the CI workflow); document the dev loop (`npm run dev` with a Vite proxy to `qai serve`).

2. **What exactly is the "UI service path" leg of the parity gate (D-16)?**
   - What we know: TUI/CLI/HTTP all reach `crates/application`.
   - What's unclear: whether "UI" means the HTTP leg (what the SPA consumes) or a direct service call.
   - Recommendation: treat the HTTP `data` payload as the UI leg, and add the direct service call as the third leg (CLI, service, HTTP) — that gives three genuinely distinct code paths converging on one `ToolResult`.

3. **Does `research_checksum` replace or supplement `reproducibility.checksum`, and where does it live on the HTTP contract?**
   - What we know: D-15 says a required field on `ToolResult`/`Envelope`; `Meta.reproducibility` is untyped.
   - What's unclear: field location (`ToolResult` top level vs inside `ReproducibilityData`; `Meta` vs `Envelope`).
   - Recommendation: add it as a typed top-level `ToolResult.research_checksum: ContentHash`, surface it in `Meta` as a typed `research_checksum: String`, and add it to the OpenAPI `Meta` required list (naming/encoding is the agent's discretion per CONTEXT).

4. **Can the NFC gap in `canonical_json_bytes` be left as-is?**
   - What we know: it is deterministic today (no `preserve_order`), but does not implement ADR-0006's NFC step.
   - Recommendation: leave it; reusing it is the frozen-contract-compliant choice. Record the gap as a follow-up for an ADR-level decision, not a Phase-5 fix.

5. **Which font is bundled, if any?** — owner gate OD-04. Recommendation: system stack; gate the bundled font behind OD-04 with the closing action recorded.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Everything | ✓ | 1.97.1 (ratatui needs ≥1.88) | — |
| cargo | Build/test | ✓ | 1.97.1 | — |
| Node.js | SPA build | ✓ | v24.18.0 | — |
| npm | SPA build | ✓ | 12.2.0 | pnpm/yarn/bun also present per AGENTS.md |
| git | Repo | ✓ | clean working tree on `main` | — |
| `cargo-deny` | CI step 4 | unknown (skips with a warning if absent) | — | CI enforces; local skip is non-blocking |
| Arabic-capable system font | Reading view default | assumed ✓ on macOS dev | — | font fallback chain; bundled font gated on OD-04 |
| `insta` | Optional TUI snapshots | declared, not locked | 1.x | `TestBackend::assert_buffer_lines` needs no extra crate |

**Missing dependencies with no fallback:** none identified.
**Missing dependencies with fallback:** `cargo-deny` (local), Arabic font (system stack fallback + OD-04 gate).

## Validation Architecture

> `workflow.nyquist_validation` is absent from `.planning/config.json` → treat as **enabled**.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` / `#[tokio::test]` + `proptest` + `trycmd`; ratatui `TestBackend` for TUI; optional Vitest for the SPA |
| Config file | none for Rust (inline + per-crate `tests/`); SPA would need `web/vitest.config.ts` (Wave 0) |
| Quick run command | `cargo test -p tools -p tool-registry -p server -p tui` |
| Full suite command | `cargo xtask ci` (9 steps) + the new parity gate |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| REQ-quran-result-contract | `research_checksum` present, payload-sensitive, deterministic | unit | `cargo test -p tools research_checksum` | ❌ Wave 0 |
| REQ-quran-result-contract | Every registered tool populates it | integration | `cargo test -p tool-registry` | ❌ Wave 0 (extend) |
| REQ-quran-research-tools | CLI/HTTP/UI parity, byte-identical payload + checksum | integration (golden) | `cargo run -p xtask -- parity` | ❌ Wave 0 |
| REQ-quran-display | Reading view renders canonical/translation/annotation distinctly | component (Vitest) or manual UAT | `npx vitest run` (Wave 0) / UAT | ❌ Wave 0 |
| REQ-quran-display | Citation copy → deep link re-verifies; mismatch hard-fails | integration | `cargo test -p citations` + `-p server` | ✅ (existing resolver) / ❌ deep-link route |
| REQ-tui-cli | TUI screens render; palette navigates | unit (buffer snapshot) | `cargo test -p tui` | ❌ Wave 0 |
| REQ-tui-cli | RAG debug renders typed "unavailable" | unit (buffer snapshot) | `cargo test -p tui rag_debug` | ❌ Wave 0 |
| SC5 | No view renders translation in a canonical slot | source scan / type test | extend `crates/application/tests/answer_path_ledger.rs` pattern | ✅ pattern exists |

### Sampling Rate
- **Per task commit:** `cargo test -p <touched crate>`
- **Per wave merge:** `cargo xtask ci` + `cargo run -p xtask -- parity`
- **Phase gate:** full suite green (including arch-check, deny, parity) before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `crates/tui/tests/` — `TestBackend` harness + screen snapshots (covers REQ-tui-cli)
- [ ] `crates/tools` — `research_checksum` unit tests + a payload-sensitivity test
- [ ] `xtask/src/parity.rs` (+ `main.rs` verb) — one fixture through CLI/service/HTTP
- [ ] `web/` test setup (`vitest.config.ts` + a smoke test) — only if frontend tests are in scope
- [ ] `crates/server/tests/` — SPA-fallback + deep-link test (router `oneshot`)
- [ ] Update `crates/server/tests/api.rs` required-keys assertions + `docs/08-api/quran-v1-openapi.json` for the new checksum field and any new routes

## Security Domain

> `security_enforcement` absent → enabled.

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | single-user localhost; auth is Phase 11 |
| V3 Session Management | no | no session state in v1 |
| V4 Access Control | yes | loopback-only bind guard (`crates/server/src/lib.rs::loopback_addr`); ADR-0004 hard-fail on `0.0.0.0` without TLS+auth |
| V5 Input Validation | yes | `quran-core` reference grammar (QAI-QUR-0100…0112, never panics); `ToolError::InvalidInput`; `serde` deserialization; graph budgets (ADR-0217) |
| V6 Cryptography | yes | `domain::ContentHash` (SHA-256) only; never hand-roll; source-manifest signatures via ed25519 (existing) |
| V7 Logging/Monitoring | yes | `observability` redaction; never log secrets or dataset bytes |
| V14 Data Protection | yes | local-first, deny egress (`security.allow_network_egress = false`) |

### Known Threat Patterns for {Rust + React SPA + axum}

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| XSS via translation/tafsir text rendered as HTML | Tampering / Elevation | Render as text (React default); **never** `dangerouslySetInnerHTML` for dataset text; note the QF tutorial recommends it for footnote HTML — reject |
| Terminal escape injection via dataset text in the TUI | Tampering | Strip control characters before `ratatui` rendering |
| Path traversal in static-asset serving | Information Disclosure | `rust-embed::get` only resolves embedded relative paths; reject `..`; never join user input onto a filesystem root |
| Truncated graph rendered as "no relationship" | Spoofing (false negative claim) | Surface `truncated`/`incomplete_reason` verbatim (ADR-0217) |
| Fabricated quotation / citation | Spoofing | Quoted text travels only inside `QuranQuotation`; `verify_quotation` hard-fails on mismatch (ADR-0111) |
| DoS via unbounded graph expansion | Denial of Service | `QueryBudgets` pre-flight validation + in-expansion enforcement (ADR-0217) |
| Resource exhaustion via regex search | Denial of Service | DFA-only `regex-automata` (existing, no backtracking) |
| Exfiltration through graph/SPA export | Information Disclosure | Export strips canonical text (refs+hashes only), authz + tombstone filtering before serialization (ADR-0218) |

## Sources

### Primary (HIGH confidence)
- `crates/tools/src/lib.rs` (read) — `ToolResult`, `ReproducibilityData`, `reproducibility()` checksum shape
- `crates/domain/src/hashing.rs` (read) — `ContentHash`, `canonical_json_bytes` (and its NFC gap)
- `crates/server/src/api.rs` (read) — `Envelope`/`Meta`, `router()`, `debug_reader_handler`, layers
- `crates/server/src/lib.rs` (read) — `loopback_addr` bind guard
- `crates/cli/src/lib.rs` (read) — command tree `dispatch()`, `serve` wiring
- `crates/application/src/lib.rs` (read) — shared services module inventory
- `crates/quran-core/src/view.rs` (read) — `AyahView`, `AttributedTranslation` layer guards
- `crates/tui/src/lib.rs` (read) — Phase-9 placeholder (to be implemented here)
- `xtask/allowlist.toml` (read) — `[application]`, `[cli]`, `[server]`, `[tools]`; no `[tui]`
- `xtask/src/ci.rs` (read) — 9-step CI gate
- `docs/05-followups/decisions-needed.md` (read) — OD-04 font gate, OD-11/OD-12
- `docs/02-architecture/decisions/ADR-0006`, `ADR-0108`, `ADR-0111`, `ADR-0112`, `ADR-0217`, `ADR-0218` (read)
- `docs/07-technical/quran-citation-spec.md` (read) — frozen deep-link/URN/verdict contract
- `docs/01-requirements/requirements.md` §11–§13, §24–§25.4 (read) — authoritative requirements
- Context7 `/ratatui/ratatui` — lifecycle, `TestBackend`, widgets
- Context7 `/vitejs/vite`, `/react/react` — resolution confirmed

### Secondary (MEDIUM confidence)
- WebSearch — rust-embed 8.12 features/version; axum-embed; axum static-file-server SPA fallback; Vite `base: "./"` embedded deployment; ratatui 0.30.2/crossterm 0.29 versions
- WebSearch — Arabic/Quran rendering (W3C alreq, QF/QUL font-rendering tutorials, design-token patterns)

### Tertiary (LOW confidence)
- Package-legitimacy heuristic `SUS` verdicts for react/react-dom/vite/@vitejs/plugin-react — false positives from latest-publish timestamps (see Audit).

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — crate versions verified against crates.io/docs.rs + Context7; npm versions verified via registry; legitimacy seam run.
- Architecture: HIGH — derived from locked ADRs + code read verbatim this session; two integration gates (arch-check, CI-without-Node) confirmed by reading the files.
- Pitfalls: HIGH for repo-specific (read this session); MEDIUM for general web/rendering guidance.
- Frontend specifics: MEDIUM — Vite/React versions verified, but the exact scaffold/embed/CI decision is an open question (A6).

**Research date:** 2026-10-07
**Valid until:** 2026-11-06 (stable Rust contracts/hashing/locked ADRs) — but re-verify the fast-moving frontend/ratatui versions (Vite 8, React 19, ratatui 0.30) before locking, since all were released within the last ~3 months.
