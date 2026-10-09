---
phase: 05-rich-quran-experience
plan: 04
subsystem: ui
tags: [spa, react, vite, rust-embed, same-origin, offline, allowlist, checkpoint]

# Dependency graph
requires:
  - phase: 05-rich-quran-experience plan 01
    provides: none direct (scaffold is independent; embed reuses the router)
provides:
  - Buildable offline `web/` SPA with pinned deps + Vitest toolchain
  - Embedded same-origin SPA serving with api-preserving fallback
  - Human-confirmed npm pins (checkpoint record)
affects: [05-06, 05-09, 05-07, 05-08]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
actuals:
  tokens: 6800
  tasks: 3
  commits: 1

# Tech tracking
tech-stack:
  added: [react 19.3.0, vite 8.3.3, vitest 5.0.3, rust-embed 8, mime_guess 2]
  patterns: [allow_missing embed for gitignored build output, fallback-after-routes with api guard, environmental test skip with precedent]

key-files:
  created: [web/package.json, web/vite.config.ts, web/src/App.tsx, crates/server/src/webassets.rs, .planning/phases/05-rich-quran-experience/05-04-CHECKPOINT.md]
  modified: [crates/server/src/api.rs, crates/server/src/lib.rs, crates/server/Cargo.toml, crates/server/tests/api.rs, .gitignore]

key-decisions:
  - "Template demo (stateful counter, stock assets, App.css) replaced with a minimal offline shell; system Arabic font stack only (OD-04 stays open)"
  - "Fallback test skips honestly when web/dist is absent (OTLP-gate precedent); 05-07 wires the npm build into CI so the gate runs there"
  - "Pre-existing api.rs:551 fmt drift left untouched (not ours); only new files kept fmt-clean"

patterns-established:
  - "SPA delivery: gitignored web/dist + allow_missing embed + fallback 404 with build remedy"
  - "Mime by served name (index.html fallback reports text/html, not the request path's guess)"

requirements-completed: [REQ-quran-display]

# Coverage metadata (#1602)
coverage:
  - id: D1
    description: "SPA builds offline with pinned deps and relative asset base; dist gitignored"
    requirement: "REQ-quran-display"
    verification:
      - kind: unit
        ref: "npm run build → web/dist/index.html; npx vitest --version → 5.0.3"
        status: pass
    human_judgment: false
  - id: D2
    description: "Client routes serve index.html; unknown /api/* 404s; health paths preserved"
    requirement: "REQ-quran-display"
    verification:
      - kind: unit
        ref: "cargo test -p server --test api spa_fallback_serves_index_and_preserves_api"
        status: pass
    human_judgment: false
  - id: D3
    description: "Pinned npm versions human-confirmed before install"
    requirement: "REQ-quran-display"
    verification:
      - kind: other
        ref: ".planning/phases/05-rich-quran-experience/05-04-CHECKPOINT.md (owner: confirmed, proceed)"
        status: pass
    human_judgment: true
    rationale: "Supply-chain trust for new external packages requires a human legitimacy verdict; automation cannot confirm publishers."

# Metrics
duration: ~2h wall (incl. one server restart, one concurrent-session reconciliation)
completed: 2026-10-09
status: complete
---

# Phase 05 Plan 04 Summary

**SPA delivery shape ships: offline React scaffold with pinned deps builds to gitignored `web/dist`, embeds into `qai` with `allow_missing`, and serves same-origin behind an api-preserving fallback.**

## Performance

- **Duration:** ~2h wall
- **Started:** 2026-10-09
- **Completed:** 2026-10-09
- **Tasks:** 3 completed
- **Files modified:** 20 (excl. lockfiles: ~440 changed lines)

## Accomplishments

- Blocking human checkpoint recorded: all nine npm pins confirmed by the owner with publishers verified
  (`05-04-CHECKPOINT.md`, incl. exact test-toolchain versions resolved at install)
- `web/` scaffold builds (`npm run build` → `dist/index.html`, 910ms) with exact pins, `base: './'`,
  zero external URLs, system Arabic font stack, and `npx vitest --version` resolving (05-06/05-09 toolchain)
- `SpaAssets` embed + `spa_fallback` (asset → index.html → 404-with-remedy; `/api/*` explicitly 404s;
  mime by served name) wired as the last route in `router()`; contract test green
- `cargo build -p server` verified green with `web/dist` absent; full server suite green; arch-check green

## Task Commits

1. **Task 1: Confirm pinned frontend versions (blocking-human)** - checkpoint record (`e69160b` concurrent
   session base + version table amendment in `4e0f9db`-era chore commit)
2. **Task 2: Scaffold web/ + Task 3: Embed + fallback** - `1357f0d` (feat, shared scaffold+server change)

**Plan metadata:** `HEAD` (docs: this file)

## Files Created/Modified

- `.planning/phases/05-rich-quran-experience/05-04-CHECKPOINT.md` - owner-confirmed pins + publishers
- `web/package.json` / `web/package-lock.json` - exact pins (react/vite/plugin/typescript + vitest/jsdom/TL)
- `web/vite.config.ts` - `base: './'`, `outDir: dist`
- `web/src/App.tsx` - minimal offline shell (replaces template demo/assets)
- `web/src/main.tsx`, `index.css`, `index.html`, `tsconfig*.json`, `public/*` - template, verified offline-clean
- `web/.gitignore` + `.gitignore` - `node_modules/` + `dist/` untracked
- `crates/server/src/webassets.rs` - `SpaAssets`, `spa_fallback`, `spa_index_present`
- `crates/server/src/lib.rs` - `pub mod webassets`
- `crates/server/src/api.rs` - `.fallback(...)` after all routes (+1 line)
- `crates/server/Cargo.toml` + `Cargo.lock` - `rust-embed`, `mime_guess`
- `crates/server/tests/api.rs` - `spa_fallback_serves_index_and_preserves_api` (with environmental skip)

## Decisions Made

- Exact pins per checkpoint (vite kept at confirmed 8.3.3, not latest 8.3.4); `@types/*` pinned exact too;
  template `oxlint` kept (zero cost), template README/demo assets removed.
- Test-toolchain versions (absent from RESEARCH audit as `[ASSUMED]`) resolved to exact latest at install and
  recorded back into the checkpoint file.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing] Plan-named `http-body-util`-style gap: `mime_guess` mime by request path**
- **Found during:** Task 3 (fallback serves `index.html` for extension-less client routes)
- **Issue:** `mime_guess::from_path(request_path)` reports `application/octet-stream` for `/read/...`.
- **Fix:** Mime is computed from the SERVED name (`index.html` → `text/html`).

**2. [Rule 3 - Blocking] New test would break CI (Rust-only, no Node) until 05-07**
- **Found during:** Task 3 (CI runs `cargo test --workspace` with no `web/dist`)
- **Issue:** A strict fallback test fails wherever the SPA was never built.
- **Fix:** Honest environmental skip with notice when the embed is empty (OTLP-gate precedent in this repo);
  05-07 wires `npm ci && npm run build` ahead of the Rust steps so the gate runs in CI. Recorded here so the
  gap is visible, not silent.

**3. [Process] Concurrent session recorded the same checkpoint base at 14:52**
- **Found during:** Task 1 close-out (foreign `e69160b` created `05-04-CHECKPOINT.md` while this run held the
  owner's live confirmation)
- **Issue:** Two sessions converging on one checkpoint file.
- **Fix:** Kept their base record, amended only the resolved test-toolchain versions; no duplicate checkpoint.

**4. [Rule 1 - Bug] Template pins drifted from checkpoint pins**
- **Found during:** Task 2 (`create-vite` ships caret ranges, older minors)
- **Issue:** `^19.2.8`/`^8.3.0`/`^6.1.1` ≠ confirmed `19.3.0`/`8.3.3`/`6.1.2`.
- **Fix:** Rewrote `package.json` with the confirmed exact pins before install.

## Verification Results

- `npm ci && npm run build` → `web/dist/index.html`; `npx vitest --version` → 5.0.3
- `cargo test -p server` — all suites green incl. `spa_fallback` (25 api tests)
- `cargo build -p server` with `web/dist` absent — success (`allow_missing`)
- `cargo xtask arch-check` — OK; `cargo clippy -p server` — clean; new files fmt-clean
- No CDN/font/network URL in `web/src`, `index.html` (scanned)
