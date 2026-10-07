---
phase: "5"
slug: "rich-quran-experience"
# status lifecycle: draft (seeded by plan-phase) → validated (set by validate-phase §6)
# audit-milestone §5.5 distinguishes NOT-VALIDATED (draft) from PARTIAL (validated + nyquist_compliant: false) (#2117)
status: draft
nyquist_compliant: false
wave_0_complete: false
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
| TBD (checksum) | TBD | 1 | REQ-quran-result-contract | — | payload-sensitive, deterministic checksum; no payload substitution | unit | `cargo test -p tools research_checksum` | ❌ W0 | ⬜ pending |
| TBD (all tools) | TBD | 1 | REQ-quran-result-contract | — | every registered tool populates `research_checksum` | integration | `cargo test -p tool-registry` | ❌ W0 (extend) | ⬜ pending |
| TBD (parity gate) | TBD | 2 | REQ-quran-research-tools | — | all 12 `TOOL_NAMES` byte-identical payload + identical checksum across service leg / generic `POST /api/v1/quran/tool/{name}` HTTP leg / CLI leg | integration (golden) | `cargo test -p cli --test parity_cross_surface` | ❌ W0 | ⬜ pending |
| TBD (read view) | TBD | 3 | REQ-quran-display | — | canonical/translation/annotation rendered distinctly; no canonical-slot leak | component (Vitest) or manual UAT | `npx vitest run` (W0) / UAT | ❌ W0 | ⬜ pending |
| TBD (citation) | TBD | 2 | REQ-quran-display | — | citation copy → deep link re-verifies at exact source; mismatch hard-fails | integration | `cargo test -p citations -p server` | ✅ resolver / ❌ deep-link route | ⬜ pending |
| TBD (TUI screens) | TBD | 2 | REQ-tui-cli | — | cockpit screens + palette navigate; buffer snapshots stable | unit (buffer snapshot) | `cargo test -p tui` | ❌ W0 | ⬜ pending |
| TBD (RAG debug) | TBD | 2 | REQ-tui-cli | — | RAG debug renders typed "unavailable" (no faked retrieval) | unit (buffer snapshot) | `cargo test -p tui rag_debug` | ❌ W0 | ⬜ pending |
| TBD (layer sep) | TBD | 3 | REQ-quran-display | — | no view renders a translation/annotation in a canonical slot (SC5) | source scan / type test | extend `crates/application/tests/answer_path_ledger.rs` pattern | ✅ pattern exists | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/tui/tests/` — `TestBackend` harness + screen snapshots (covers REQ-tui-cli)
- [ ] `crates/tools` — `research_checksum` unit tests + a payload-sensitivity test
- [ ] `crates/cli/tests/parity_cross_surface.rs` — one fixture through CLI / service / HTTP (deliberate relocation from `xtask/src/parity.rs`; `xtask` has `workspace.allow = []`, so the CLI test crate — which already depends on `server` + `application` and owns `CARGO_BIN_EXE_qai` — is the correct home)
- [ ] `web/` test setup (`vitest.config.ts` + a smoke test) — only if frontend tests are in scope
- [ ] `crates/server/tests/` — SPA-fallback + citation deep-link test (router `oneshot`)
- [ ] `crates/server/tests/api.rs` — generic typed-tool route test (`POST /api/v1/quran/tool/{name}`)
- [ ] Update `crates/server/tests/api.rs` required-keys assertions + `docs/08-api/quran-v1-openapi.json` for the new checksum field and any new routes

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
