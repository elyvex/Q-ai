---
phase: "5"
slug: "rich-quran-experience"
# status lifecycle: draft (seeded by plan-phase) → validated (set by validate-phase §6)
# audit-milestone §5.5 distinguishes NOT-VALIDATED (draft) from PARTIAL (validated + nyquist_compliant: false) (#2117)
status: draft
nyquist_compliant: false
wave_0_complete: true
created: "2026-10-07"
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` / `#[tokio::test]` + `proptest` + `trycmd`; ratatui `TestBackend` buffer snapshots for TUI; optional Vitest for the SPA |
| **Config file** | none for Rust (inline + per-crate `tests/`); SPA would need `web/vitest.config.ts` (Wave 0, only if frontend tests are in scope) |
| **Quick run command** | `cargo test -p tools -p tool-registry -p server -p tui` |
| **Full suite command** | `cargo xtask ci` (11 steps after plan 05-07 adds the SPA build + the cross-surface parity test `cargo test -p cli --test parity_cross_surface`; the 9 pre-existing steps are fmt/clippy/test/deny/arch-check/migrate-check/adr-lint/gen-schema/doctor-schema) |
| **Estimated runtime** | ~180–600 seconds (full CI incl. arch-check, deny, parity) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p <touched crate>`
- **After every plan wave:** Run `cargo xtask ci` + `cargo test -p cli --test parity_cross_surface`
- **Before `/gsd-verify-work`:** Full suite must be green (incl. `arch-check`, `deny`, parity)
- **Max feedback latency:** 600 seconds

---

## Per-Task Verification Map

> Task IDs are finalized by the planner; rows below are seeded from RESEARCH.md §Validation Architecture and map each phase requirement to its required automated evidence. The planner fills concrete task IDs on the same rows.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 05-01 Task 1: research_checksum contract | 05-01 | 1 | REQ-quran-result-contract | — | payload-sensitive, deterministic checksum; no payload substitution | unit | `cargo test -p tools research_checksum` | ✅ | ✅ green |
| 05-01 Task 2: shared dispatcher | 05-01 | 1 | REQ-quran-result-contract | — | every registered tool populates `research_checksum` | integration | `cargo test -p tool-registry` | ✅ | ✅ green |
| 05-03 Task 1+2: parity harness + CLI leg | 05-03 | 2 | REQ-quran-research-tools | T-05-07/08 | all 12 `TOOL_NAMES` byte-identical payload + identical checksum across service leg / generic `POST /api/v1/quran/tool/{name}` HTTP leg / CLI leg | integration (golden) | `cargo test -p cli --test parity_cross_surface` | ✅ | ✅ green |
| 05-06 Task 3: reading view | 05-06 | 3 | REQ-quran-display | T-05-15/17 | canonical/translation/annotation rendered distinctly; no canonical-slot leak | component (Vitest) | `npx vitest run` in `web/` | ✅ | ✅ green |
| 05-08 Task 2: citation open flow | 05-08 | 5 | REQ-quran-display | T-05-16/22 | citation copy → deep link re-verifies at exact source; mismatch hard-fails | integration | `cargo test -p server --test citation_open` (+ `cargo test -p citations -p server`) | ✅ | ✅ green |
| 05-05 Tasks 1–3: cockpit screens | 05-05 | 2 | REQ-tui-cli | T-05-12/13/14 | cockpit screens + palette navigate; buffer snapshots stable | unit (buffer snapshot) | `cargo test -p tui` | ✅ | ✅ green |
| 05-05 Task 3: RAG debug screen | 05-05 | 2 | REQ-tui-cli | T-05-14 | RAG debug renders typed "unavailable" (no faked retrieval) | unit (buffer snapshot) | `cargo test -p tui render_rag_debug` | ✅ | ✅ green |
| 05-07 Task 2: layer scan (+ 05-06 Task 2 type guard) | 05-07 | 6 | REQ-quran-display | T-05-17/20 | no view renders a translation/annotation in a canonical slot (SC5) | source scan / type test | `cargo test -p application --test layer_separation` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `crates/tui/tests/` — `TestBackend` harness + screen snapshots (covers REQ-tui-cli)
- [x] `crates/tools` — `research_checksum` unit tests + a payload-sensitivity test
- [x] `crates/cli/tests/parity_cross_surface.rs` — one fixture through CLI / service / HTTP (deliberate relocation from `xtask/src/parity.rs`; `xtask` has `workspace.allow = []`, so the CLI test crate — which already depends on `server` + `application` and owns `CARGO_BIN_EXE_qai` — is the correct home)
- [x] `web/` test setup (`vitest.config.ts` + a smoke test) — only if frontend tests are in scope
- [x] `crates/server/tests/` — SPA-fallback + citation deep-link test (router `oneshot`)
- [x] `crates/server/tests/api.rs` — generic typed-tool route test (`POST /api/v1/quran/tool/{name}`)
- [x] Update `crates/server/tests/api.rs` required-keys assertions + `docs/08-api/quran-v1-openapi.json` for the new checksum field and any new routes

## Build-Artifact Policy (`web/dist`)

- `web/dist` and `web/node_modules` are gitignored build artifacts; only `web/` source is tracked.
- The server embeds via `rust-embed` with `#[allow_missing = true]`, so plain `cargo build` / `cargo test`
  succeed before `npm run build` has ever run (the fallback 404s with a build remedy until then).
- `cargo xtask ci` step 3/11 runs `npm ci && npm run build` in `web/` ahead of all Rust steps, so CI always
  tests against a populated embed (11 steps total: fmt/clippy/spa-build/test/deny/parity/arch-check/
  migrate-check/adr-lint/gen-schema/doctor-schema).
- Dev loop: `qai serve` on loopback + Vite dev server proxying `/api` (see the `xtask ci` SPA step comment).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| RTL reading layout, translation panels, word-inspector switchability, user themes | REQ-quran-display | Visual/UX judgement across all three display modes and three inspector modes | Launch `qai serve`, open the SPA, toggle each translation mode and inspector mode; confirm RTL, panel collapse, theme switch |
| Unmistakable visual distinction of canonical/translation/annotation | REQ-quran-display (SC5) | Perceptual; automated source scan is the backstop | Inspect each view; confirm no view renders translation in a canonical slot |
| TUI cockpit daily-ops ergonomics (command palette, RAG debug view) | REQ-tui-cli | Terminal UX / keybinding feel | Run the TUI, drive the palette, open the RAG debug screen; confirm typed "unavailable" state |
| Bundled Arabic font quality (PRD §13.1) | REQ-quran-display | Gated on OD-04 (owner-only web-font + license decision) | Default is the system font stack; a bundled font requires the owner gate to close first |

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 600s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
