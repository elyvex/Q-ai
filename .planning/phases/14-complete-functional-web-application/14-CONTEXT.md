# Phase 14: Complete Functional Web Application - Context

**Gathered:** 2026-11-16
**Status:** Ready for planning
**Mode:** Autonomous smart discuss (grey areas proposed as batch tables; all recommendations accepted)

<domain>
## Phase Boundary

The web UI becomes a complete working environment over the real Q-ai backend:
every reachable capability is visible, testable and manageable from the browser
with complete data presentation, and every core flow is proven by end-to-end
browser tests against a live engine. Responsive and accessible on desktop and
mobile.

In scope: the 15 locked requirements of `14-SPEC.md` — ask/agent run + trace,
the answer contract, tool lifecycle, source catalog, operations dashboard, the
8 unsurfaced count instruments, the 10 unsurfaced REST endpoints, 3 first-class
graph views, CLI reference + copyable equivalents, the 13-UI-REVIEW minor
close-out, green gates, a Playwright suite (local + CI), an axe/Lighthouse/
keyboard accessibility bar, reference-driven working interfaces that preserve
the Concordance identity, and the RAG project lifecycle.

Out of scope: unrelated backend behaviour changes outside those requirements,
TUI cockpit work (`REQ-tui-cli` stays separate), native mobile shells, new
third-party registries. Dark theme stays deferred.

</domain>

<decisions>
## Implementation Decisions

### Backend route shape for the new areas

- New HTTP routes live beside the existing per-domain prefixes, mirroring
  `quran` / `hadith` / `tafsir`: `/api/v1/agents/*`, `/api/v1/tools/*`,
  `/api/v1/catalog/*`, `/api/v1/ops/*`. No single `/api/v1/admin/*` bundle.
- Agent run progress: `POST /api/v1/agents/runs` creates a run, `GET
  …/runs/{id}` returns a snapshot, and `GET …/runs/{id}/events` streams
  progress (SSE). Polling the snapshot stays supported as the fallback. The
  "cancellable run trace" requirement cannot be met by polling alone.
- Cancellation is cooperative: a persisted cancel flag/token the run loop
  observes, so a cancelled run stops mid-flight and its trace records
  `cancelled` (never a silent stop that looks like completion).
- Errors reuse the existing typed `QAI-*` envelope and status mapping already
  used by the Quran/hadith/tafsir routes; new routes invent no second error
  shape.
- Auth reuses the existing bearer middleware; no per-area tokens.
- Durable-state writes (tool lifecycle, catalog, RAG projects) follow the same
  deny-by-default approval pattern as other durable writes, with audit events.

### UI placement in the existing shell

- The new surfaces are top-level routes so success criterion 1 (every
  capability has a route) is mechanically checkable: `/ask`, `/studio`,
  `/catalog`, `/operations`.
- The header rail already carries 13 destinations. The new and advanced routes
  join one secondary navigation group rather than extending the flat list
  indefinitely; the existing active-route indent/`isActive` behaviour is
  preserved.
- Components come from the existing primitives — `Card`, `Tabs`, `Table`
  atoms, `EmptyState`, `ErrorState`, `Skeleton`, `Toast` — and follow the
  Concordance grammar in `13-UI-SPEC.md`. No new component family, no second
  design language.
- Instrument surfacing: dedicated first-class views only where the requirement
  names one (`subgraph`, `pattern`, `root-family`); the 8 count instruments and
  10 REST endpoints render through the existing instrument registry patterns
  (rules block + checksum), not one bespoke view each.

### E2E and CI

- Playwright (installed as a devDependency in `web/`) drives flows against a
  real engine: a global setup boots `qai serve` on a temporary SQLite database
  seeded with the existing committed fixtures, and tears it down afterwards.
  No API mocking in the browser.
- A new web CI job is added to GitHub Actions (the repo is Rust-only today):
  install, typecheck/build, vitest, Playwright, and axe.
- Accessibility gate: axe on every route with zero serious/critical
  violations, plus keyboard-only completion of the core flows; Lighthouse
  accessibility ≥ 90 remains recorded evidence (desktop form factor, per the
  phase-13 criterion clarification).
- If scope overruns, nothing is dropped silently: every unfinished item gets
  an explicit deferral row naming it, per success criterion 1's "zero silent
  gaps".

### Claude's Discretion

- Exact SSE event schema (event names, payload fields) and the run-trace
  rendering granularity.
- Wave sequencing across the 15 requirements, provided each requirement lands
  as a working vertical slice (route → UI → test).
- Where the CLI-reference view is mounted and how command trees are generated
  from the existing CLI definition.
- Which `temp-inspire/` patterns are adopted per view, within the Concordance
  identity.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets

- `web/src/router.tsx` — the single dispatch point; `ROUTES`, `NAVIGATION`
  (in `components/command/command-menu.tsx`) and `DIRECTORY` are asserted by
  `web/src/tests/reachability.test.ts`, so a new route must be added to all
  three to stay honest.
- `web/src/components/ui/*` — the primitive set (`Button`, `Input`, `Select`,
  `Tabs`, `Card`, `Skeleton`, `Spinner`, `Toast`, `EmptyState`, `ErrorState`,
  `ErrorBoundary`) plus the `.kicker` / `.display` / `.sheet` atoms in
  `web/src/index.css`.
- `web/src/api/client.ts` — the typed client and envelope handling the views
  already use; new routes extend it rather than adding a second fetch layer.
- `web/src/views/*` — 13 working views; `ExplorerView` already dispatches
  arbitrary requests, which is the natural place to expose newly added
  endpoints before a dedicated view exists.
- `web/src/tests/reachability.test.ts` — the feature-to-route matrix and the
  Concordance design-contract guard; the matrix is where "zero silent gaps" is
  enforced mechanically.

### Established Patterns

- Typed request/response envelopes with `QAI-*` error codes; provenance
  layers (`--layer-canonical/translation/annotation`) never collapse.
- Instruments render a rules block plus a `sha256:` checksum, and the UI never
  fabricates a result for an unconfigured backend (it shows the typed error).
- Tokens live only in `web/src/index.css` plus `tokens/layers.css`;
  components carry no colour literals.
- Durable writes go through explicit approval + audit; the web UI never
  executes shell commands (it shows copyable `qai …` equivalents instead).

### Integration Points

- `crates/server/src/api.rs` — the axum router (~line 2236 onward) where new
  `/api/v1/{agents,tools,catalog,ops}` routes attach.
- `crates/application/src/quran_tools.rs` — the tool dispatch arms that the
  8 count instruments and 3 graph tools already flow through.
- `.github/workflows/` — the new web job attaches beside the existing Rust
  jobs.
- `web/src/tests/` — the reachability matrix and the new Playwright specs.

</code_context>

<specifics>
## Specific Ideas

- `temp-inspire/` holds the reference implementations whose strongest patterns
  the phase should adopt per route, without losing the Concordance identity.
- `13-UI-REVIEW.md` (22/24) lists the exact remaining minors: sidebar widths,
  CTA wording, spacing scale, and the dead `tokens/themes.css` item (the file
  itself is live — re-check before "removing" it).
- The known one-line defect `cargo test -p application --test layer_separation`
  referenced a deleted `WordInspector.tsx`; it was already fixed in
  `27924a3` — confirm before re-fixing.

</specifics>

<deferred>
## Deferred Ideas

- Dark theme (deferred per the phase-13 owner gate; the `.dark` block exists
  but is out of scope).
- Mobile-first LCP budget: the phase-13 criterion was clarified to a desktop
  form factor; a mobile performance budget would be its own phase.
- TUI cockpit work (`REQ-tui-cli`) stays outside this phase.
- Native mobile shells.

</deferred>
