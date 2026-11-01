# Phase 14: Complete Functional Web Application — Specification

**Created:** 2026-11-01
**Ambiguity score:** 0.20 (gate: ≤ 0.20)
**Requirements:** 14 locked

## Goal

The web UI becomes a complete working environment for the real Q-ai backend: all 13 existing routes become fully working interfaces over real data, every reachable capability (including 5 backend areas that need new HTTP routes) is visible and testable from the browser, and every core flow is proven by Playwright E2E tests running locally and in CI.

## Background

Phase 13 shipped a 13-route Concordance UI (forest-green on warm white, `13-UI-SPEC.md` amended, `13-UI-REVIEW.md` 22/24) with `npm run build` green and 151 vitest tests passing. Verified gaps against `crates/server/src/api.rs`: 8 count instruments (`occurrences`, `hapax`, `collocation`, `numeric-report`, `missing-form`, `near-duplicates`, `interval`, `unusual-usage`), 10 REST endpoints (`meta`, `editions`, `surahs`, `context`, `divisions`, `tokens`, `normalization/preview` + `profiles`, `verse/links`), and 3 graph tools (`subgraph`, `pattern`, `root-family`) have no UI surface. Five reference views (Ask/agents, Tool lifecycle, Source catalog, Operations, System deploy) have no HTTP routes at all. No E2E suite, no axe/Lighthouse gates, and CI has no web jobs (Rust-only). `cargo test -p application --test layer_separation` fails on the deleted `WordInspector.tsx` path. This phase closes all of it.

## Requirements

1. **Ask/agent run + trace**: asking a research question from the browser runs a controlled agent and shows progress, tools, evidence, and a cancellable run trace.
   - Current: No HTTP route exists for agents; no ask UI exists anywhere.
   - Target: New HTTP routes (submit question, poll/stream progress + trace, cancel) plus an Ask view rendering the trace, selected tools, evidence snippets, and a final answer; run controls (cancel, bounded-usage display) work.
   - Acceptance: A question submitted from the browser returns a traced run; cancelling mid-run stops it and marks the trace cancelled; every cited claim carries a working re-verify control.

2. **Answer contract**: agent answers are structured evidence answers with claim-level citations and side-by-side disagreement.
   - Current: No answer shape exists; `RAG Debug` shows raw payloads only.
   - Target: Answers render as sections with per-claim citations; citations re-verify server-side (green/red inline); disagreements render in parallel columns, never merged.
   - Acceptance: Every claim in a sample answer links to a citation that re-verifies green; a stored mismatch renders red with the server error, never a softened success.

3. **Tool lifecycle**: tool authors can run describe → review → generate → test → security-review → approve → publish → revoke from UI-backed routes.
   - Current: No tool-authoring HTTP routes; `ToolsView` only runs the 27 callable read tools.
   - Target: New HTTP routes for each lifecycle step plus a Tool Studio view; each step shows its input, output, and gate verdict; revoke is instant and visible.
   - Acceptance: A tool walks the full lifecycle in order; skipping a step is refused with a typed error; revoking removes it from the callable set.

4. **Source catalog**: curators manage sources through discover → quarantine → license-review → stage → diff → approve → rollback, with trust profiles.
   - Current: No catalog HTTP routes; no catalog UI.
   - Target: New HTTP routes for each pipeline state plus a Catalog view with per-state cards, action buttons, diff preview, and per-source trust profiles.
   - Acceptance: A source moves through every state with audit events; rollback restores the prior version byte-identically; license-gated activation is enforced server-side.

5. **Operations dashboard**: jobs, audit trail, budgets, sessions, and doctor health are inspectable and actionable from the browser.
   - Current: No management HTTP routes; no ops UI.
   - Target: New HTTP routes plus an Operations view: job table (cancel, checkpoint detail), filterable audit log, budgets/run-events/sessions panels, doctor health cards with explicit repair actions and last-run timestamps.
   - Acceptance: A running job can be cancelled from the UI and shows cancelled state; doctor lists corpus/index/cross-store health with a working repair action.

6. **Unsurfaced count instruments**: all 8 remaining count endpoints render with rules blocks and checksums.
   - Current: Only `frequency`, `distribution`, `root-frequency`, `lemma-frequency`, `cooccurrence` are surfaced (`WordsView`).
   - Target: `occurrences`, `hapax`, `collocation`, `numeric-report`, `missing-form`, `near-duplicates`, `interval`, `unusual-usage` each render as ruled results with their `rules` block and checksum; zero counts use the proven-zero empty state.
   - Acceptance: Each of the 8 returns a server payload rendered with its rules and checksum; a zero renders "zero under these rules", never a blank panel.

7. **Unsurfaced REST endpoints**: editions, surahs, context, divisions, tokens, normalization, verse links, and meta render as browsable views.
   - Current: No UI reaches these 10 endpoints; edition/surah navigation is manual URL entry.
   - Target: Browsable surfaces (edition + surah browser, context/division lookup, token inspector, normalization preview against profiles, verse-link panel, meta/system view), all transcript-style with deep links.
   - Acceptance: Each endpoint is reachable from navigation or a view control and prints its payload verbatim; normalization preview never alters displayed canonical text.

8. **Graph dedicated views**: `subgraph`, `pattern`, and `root-family` get first-class views (not only the generic Tools forms).
   - Current: Reachable only through untyped `ToolsView` JSON fields.
   - Target: Subgraph export (JSON/DOT/SVG download), pattern builder with typed edge steps, root-family tree with surah:ayah links — all through `ToolLedger` with checksums.
   - Acceptance: A subgraph exports downloadable JSON/DOT/SVG; a pattern query renders matched paths with per-edge provenance; root-family rows link to the reader.

9. **CLI reference + equivalents**: a `qai` command-tree reference view exists, and every runnable UI surface shows copyable CLI + HTTP equivalents.
   - Current: CLI/HTTP toggle exists only in `ToolsView`; no command reference anywhere.
   - Target: A CLI view documents the `qai` tree from `--help` output shape; every runnable surface (tools, counts, RAG, ask, ops actions) prints the exact `qai … --params '<json>' --json` and `POST /api/v1/…` for the params actually run, with copy buttons and clipboard-failure messages.
   - Acceptance: The CLI view lists every top-level `qai` command; each runnable surface's CLI line replays the identical checksum when pasted into a terminal (spot-checked, not executed by the browser).

10. **Audit minor close-out**: the three remaining 13-UI-REVIEW (22/24) fix groups are resolved.
    - Current: `ToolsView` 18rem / `ExplorerView` 14rem sidebars vs §6 13rem; CTA deviations ("Dispatch request", per-tool "Run X", raw "awaiting dispatch"); fractional spacing utilities; dead `tokens/themes.css`.
    - Target: Sidebars adopt 13rem or record a §6 exception; CTAs match §19 contract strings and every pre-run transcript is a typed `EmptyState`; spacing snaps to scale or records exceptions; `themes.css` deleted if unreferenced.
    - Acceptance: Re-audit scores Spacing ≥ 3 and Copywriting ≥ 3 with zero open minor rows, or exception rows in UI-SPEC.

11. **Gates green (defect fix)**: `layer_separation` passes; full suites stay green.
    - Current: `spa_non_canonical_components_wear_no_canonical_class` fails on the deleted `WordInspector.tsx` path; web suite 151/151 green.
    - Target: Remove the deleted path from the test array (one-line defect fix, no behavior change); `npm run build`, full vitest, and the layer_separation suite pass.
    - Acceptance: `cargo test -p application --test layer_separation` passes 7/7; web build + vitest remain green.

12. **E2E suite (local + CI)**: Playwright drives core flows against a live engine with a real data snapshot, locally and in a new CI job.
    - Current: No E2E suite; only manual captures; CI has no web jobs.
    - Target: Playwright suite (system Chrome) covering read → search → words → graph → citations → tools → hadith/tafsir/scripture/isnad/RAG-debug + ask/catalog/ops flows; runs against a local engine seeded from a real cataloged snapshot; new GitHub Actions job runs build + vitest + E2E.
    - Acceptance: Suite passes locally against the snapshot and passes in CI on a clean runner; a failing flow fails the job (no silent green).

13. **Accessibility + responsive bar**: axe zero violations and Lighthouse ≥ 90 gate every route; keyboard-only flows; 390px verified.
    - Current: No axe/Lighthouse automation; mobile evidence is screenshots only; post-interaction states code-verified only.
    - Target: axe runs per route in CI (zero serious/critical to pass); Lighthouse accessibility ≥ 90 per route in CI; core flows (read, search, copy citation, ask, approve a source) completable keyboard-only; every route re-captured at 390px + 1440px.
    - Acceptance: CI fails on any axe serious/critical or Lighthouse < 90; keyboard-only walkthrough of the 5 core flows passes; current mobile + desktop screenshots exist for every route.

14. **Reference-driven working interfaces (Concordance preserved)**: every route becomes a useful working interface using the strongest patterns from `temp-inspire/`, without losing the Concordance identity.
    - Current: Views are transcript-first; filtering/sorting/pagination/expandable details are uneven across routes.
    - Target: Per-route upgrades (search/filter hits, sort where order is not service-mandated, paginate long lists, expandable row details, contextual actions, copy/export/navigation) composed from reference patterns; Concordance tokens, hairlines, margins, mono references, and Arabic/RTL rules unchanged.
    - Acceptance: Every visible control works or is disabled with a stated reason (re-audit Experience Design = 4); no fabricated data; no mock results for unavailable backends; re-audit overall ≥ 22/24.

## Boundaries

**In scope:**
- New HTTP routes + views for ask/agents, tool lifecycle, source catalog, operations (R1–R5)
- Surfacing all unsurfaced read endpoints and tools (R6–R8)
- CLI reference view + CLI/HTTP equivalents everywhere runnable (R9)
- Audit minor close-out + layer_separation defect fix (R10–R11)
- Playwright E2E suite + new CI web job (R12)
- Axe + Lighthouse gates, keyboard flows, responsive screenshots (R13)
- Per-route working-interface upgrades within Concordance tokens (R14)

**Out of scope:**
- TUI cockpit work — REQ-tui-cli stays a separate surface
- Native mobile shells — responsive web only
- New foundation models or provider integrations beyond existing routing — model work is not this phase
- Executing CLI commands from the browser — display and copy only; the browser cannot spawn processes
- Sectarian authority claims or generated religious rulings — product principles forbid; analysis stays traceable
- Third-party UI registries or remote CDNs — Concordance stays vendored and self-hosted

## Constraints

- No fabricated data: every rendered value comes from a typed API response; unavailable backends render named empty/error states, never mocks.
- Deny-by-default preserved: mutating RAG/isnad tools stay refused (`QAI-RAG-0326`/`QAI-ISNAD-0326`) unless the operator opts in; new mutating routes follow the same policy pattern.
- Concordance contract (`13-UI-SPEC.md` as amended): forest-green accent, warm-white ground, hairline structure, mono references, Amiri Arabic metrics — new views reuse `PageHeader`/`EmptyState`/`ErrorNote`/`LoadingLine`/`ToolLedger`.
- E2E runs against a real cataloged snapshot (interview decision), with system Chrome locally and in GitHub Actions.
- Axe zero serious/critical + Lighthouse accessibility ≥ 90 per route gate CI (interview decision).
- Performance budgets for new backend routes and views are a planner assumption (not locked in interview) — planner inherits Phase 13 targets where applicable.

## Acceptance Criteria

- [ ] Ask flow runs from the browser with trace, cancel, and re-verifiable claim citations (R1–R2)
- [ ] Tool lifecycle completes describe→publish→revoke with instant revocation (R3)
- [ ] Source moves through all catalog states with rollback byte-identical (R4)
- [ ] Operations view cancels a job and runs a doctor repair action (R5)
- [ ] All 8 unsurfaced count instruments render with rules + checksums (R6)
- [ ] All 10 unsurfaced REST endpoints reachable from UI (R7)
- [ ] Subgraph exports JSON/DOT/SVG; pattern shows per-edge provenance (R8)
- [ ] CLI reference view lists the `qai` tree; every runnable surface prints copyable CLI+HTTP (R9)
- [ ] Re-audit Spacing and Copywriting have zero open minor rows (R10)
- [ ] `layer_separation` 7/7, web build + vitest green (R11)
- [ ] Playwright suite passes locally and in the new CI job (R12)
- [ ] Axe zero + Lighthouse ≥ 90 gate CI; keyboard-only core flows pass; 390px screenshots current (R13)
- [ ] Re-audit overall ≥ 22/24 with every control working or disabled-with-reason (R14)

## Implementation Acceptance Criteria (locked 2026-11-01)

Mandatory for every implementation plan in this phase (frontend and backend). A plan is not done when it builds — it is done when it verifies. These criteria supplement (never replace) the checklist above.

**Application completeness**
- [ ] Every route has a stated purpose and complete information architecture (no heading-only, placeholder-only, or permanently empty content areas where the backend provides data)
- [ ] Every backend capability in agreed scope has a UI entry point; anything still lacking one names the missing endpoint explicitly — zero silent gaps
- [ ] Every data view consumes real API responses — no fabricated data, no mock results for unavailable backends
- [ ] Every form has validation and meaningful submission feedback (success, failure with remedy, disabled-with-reason)
- [ ] Every interactive component has a working behavior or is disabled with a stated reason
- [ ] Every page handles loading, empty, error, and success states with typed, named states (never blank panels)
- [ ] Every data structure renders in an appropriate format — structured views, tables, ledgers — never indiscriminate raw-JSON dumps
- [ ] Complex data supports exploration: detail views, expandable sections, filters, sorting, pagination, and navigation where the payload warrants it
- [ ] Deep links and browser navigation (back/forward, cold-load URLs, query strings) keep working on every route
- [ ] Responsive layouts hold at 390px and 1440px; dark mode stays functional on every route
- [ ] Arabic RTL, bidi isolation, and Unicode correctness hold on every route (Amiri metrics, no mirrored Latin identifiers)

**Testing requirements**
- [ ] New/updated Vitest tests ship with every non-trivial frontend change
- [ ] Every route is covered by at least one test
- [ ] Every major user interaction is tested (run/submit/toggle/navigate/copy/re-verify flows)
- [ ] Real API integration is tested against a running backend (SPEC R12 E2E suite, not jsdom stubs alone)
- [ ] Browser screenshots are captured for all routes (light + dark + mobile) with zero page errors
- [ ] Browser console errors and failed network requests are checked on every tested route — zero unexplained entries
- [ ] Empty and error responses are tested (typed states render; failures print code + remedy)
- [ ] No-fake-data verification: tests/captures confirm rendered values come from typed API responses
- [ ] Production build (`npm run build`) passes
- [ ] All relevant existing frontend tests still pass (no regressions)

**Scope protection**
- [ ] No modification to Rust crates, domain architecture, database schema, pinned Rust toolchain, or the canonical Quran engine outside the explicitly scoped R1–R5 backend plans
- [ ] Any frontend plan that needs a not-yet-built R1–R5 route documents it as an explicit dependency, renders a named empty state until it lands, and never invents an endpoint, response field, or capability
- [ ] No silent backend edits: every backend change ships in its own atomic commit with tests

**Definition of Done (phase-level)**
- [ ] The application is usable as a real web-based interface for exploring, testing, and operating the existing Q-ai functionality
- [ ] Completion = functional verification + real data presentation + route coverage + visual consistency with the amended design contract — a green build alone is not completion

## Edge Coverage

**Coverage:** probes skipped by author choice — author selected "Done talking — write it" at the spec gate. Planner must treat edge analysis as an assumption and surface edge cases during planning.

## Prohibitions (must-NOT)

**Coverage:** probes skipped by author choice (same as above). Standing product prohibitions apply: MUST NOT fabricate verses/hadith/citations/gradings; MUST NOT present analysis as scholarly consensus; MUST NOT execute processes from the browser; MUST NOT weaken deny-by-default. Planner must encode these as `must_haves.prohibitions`.

## Ambiguity Report

| Dimension          | Score | Min  | Status | Notes                                              |
|--------------------|-------|------|--------|----------------------------------------------------|
| Goal Clarity       | 0.85  | 0.75 | ✓      | Backend scope + reference method locked in round 2 |
| Boundary Clarity   | 0.80  | 0.70 | ✓      | TUI/native/execution explicitly out                |
| Constraint Clarity | 0.70  | 0.65 | ✓      | CI env + a11y bars locked; perf budgets assumed    |
| Acceptance Criteria| 0.80  | 0.70 | ✓      | 14 pass/fail criteria                              |
| **Ambiguity**      | 0.20  | ≤0.20| ✓      |                                                    |

## Interview Log

| Round | Perspective          | Question summary                                  | Decision locked                                          |
|-------|----------------------|---------------------------------------------------|----------------------------------------------------------|
| 1     | Researcher           | OD-16: UI-only or backend-inclusive?              | Backend-inclusive: all 5 areas need new HTTP routes      |
| 1     | Researcher           | Where does E2E run and against what engine?       | Local + new CI job, live engine, real snapshot           |
| 1     | Researcher           | What does CLI exposure mean (browser can't spawn)?| Both: CLI/HTTP equivalents + qai tree reference view    |
| 2     | Researcher+Simplifier| Irreducible backend core if forced to cut?        | No cut: all five areas required                          |
| 2     | Researcher           | E2E fixture data source?                          | Real cataloged snapshot (not test fixture)               |
| 2     | Simplifier           | Exact a11y bar?                                   | Axe zero serious/critical + Lighthouse ≥ 90 in CI        |
| Gate  | Seed Closer          | Ambiguity 0.20 — proceed?                         | "Write it" + reference-driven directive (temp-inspire patterns, Concordance preserved, per-route working interfaces, real data only) |

---

*Phase: 14-complete-functional-web-application*
*Spec created: 2026-11-01*
*Next step: /gsd-discuss-phase 14 — implementation decisions (how to build what's specified above)*
