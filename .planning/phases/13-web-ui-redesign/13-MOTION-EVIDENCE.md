---
phase: 13-web-ui-redesign
gap: G-13-2
date: 2026-11-15
tool: lighthouse 13.6.0 (in-tree binary, option A)
status: measured
---

# Motion performance evidence (G-13-2)

Closes the verification gap "Motion performance verified (LCP/CLS/60fps) —
roadmap SC7" from `13-VERIFICATION.md`.

**Path chosen:** Option A — measure now with the Lighthouse binary already in
`web/node_modules/.bin/`, and keep the `web/package.json` /
`web/package-lock.json` devDependency diff (`lighthouse`, `@playwright/test`,
`@axe-core/playwright`) as this plan's only dependency change. All three are
dev-only and never enter the runtime bundle.

**Tooling state found:** committed `HEAD` carried no measurement package; the
working tree carried the devDependency diff with binaries installed. Nothing
new was installed for this measurement.

## Commands

```bash
cd web
npm run build
npm run preview -- --port 4173 --strictPort

./node_modules/.bin/lighthouse http://localhost:4173/ \
  --preset=desktop --only-categories=performance,accessibility \
  --chrome-flags="--headless=new --no-sandbox" --output=json --output-path=/tmp/lh-desk-root.json --quiet

./node_modules/.bin/lighthouse http://localhost:4173/ \
  --only-categories=performance,accessibility \
  --chrome-flags="--headless=new --no-sandbox" --output=json --output-path=/tmp/lh-mob-root.json --quiet

./node_modules/.bin/lighthouse http://localhost:4173/search \
  --preset=desktop --only-categories=performance,accessibility \
  --chrome-flags="--headless=new --no-sandbox" --output=json --output-path=/tmp/lh-desk-search.json --quiet

./node_modules/.bin/lighthouse http://localhost:4173/search \
  --only-categories=performance,accessibility \
  --chrome-flags="--headless=new --no-sandbox" --output=json --output-path=/tmp/lh-mob-search.json --quiet
```

Note: no Rust engine was running for these runs. The shell, the design system
and the animated field are fully measurable; data-dependent views render their
empty/error states. The animation was running (not paused) during every run.

## Results

| Route | Form factor | Perf | A11y | LCP | CLS | TBT |
|-------|-------------|------|------|-----|-----|-----|
| `/` | desktop | 0.71 (cold first run) | 1.00 | 452 ms | 0.0000 | 869 ms |
| `/` | desktop (re-run) | 1.00 | 1.00 | 452 ms | 0.0002 | 0 ms |
| `/` | mobile (4x CPU, slow 4G) | 0.98 | 1.00 | 1907 ms | 0.0000 | 32 ms |
| `/search` | desktop | 1.00 | 1.00 | 448 ms | 0.0001 | 0 ms |
| `/search` | mobile (4x CPU, slow 4G) | 0.98 | 1.00 | 1942 ms | 0.0003 | 30 ms |

The 0.71 desktop run was the first Chrome launch of the session and did not
reproduce: the same command on the same build scored 1.00, and `/search`
scored 1.00 with 0 ms blocking time. Recorded as observed, not discarded.

Bundle at this commit (`vite build`): `index.js` 525.93 kB raw / **156.99 kB
gzip**, `index.css` 111.33 kB raw / **31.22 kB gzip**. No third-party network
origin: all fonts are `@fontsource` assets bundled by Vite.

## Budget table

| Budget (roadmap SC7/SC11) | Measured | Verdict |
|---------------------------|----------|---------|
| Lighthouse performance ≥ 80 | 0.98–1.00 desktop and mobile (one cold outlier 0.71) | PASS |
| Lighthouse accessibility ≥ 90 | 1.00 on all four runs | PASS |
| CLS < 0.1 | ≤ 0.0003 on all four runs | PASS |
| LCP ≤ 1.5 s (desktop) | 448–452 ms | PASS |
| LCP ≤ 1.5 s (mobile emulation, 4x CPU / slow 4G) | 1907–1942 ms | FAIL |
| Initial bundle ≤ 350 kB gzipped | 156.99 kB JS + 31.22 kB CSS | PASS |

## Remediation

**Mobile LCP misses the 1.5 s budget by ~0.4 s** under Lighthouse's mobile
emulation (4x CPU throttling, slow 4G). Desktop is comfortably inside budget
and every other row passes.

Concrete next step, in order of leverage:

1. **Route-level code splitting** (`React.lazy` + `Suspense` in
   `web/src/router.tsx`) — the single 526 kB entry chunk is parsed and
   executed before first paint on a throttled device. Splitting the 13 views
   is the change Phase 14 SC11 already owns ("initial bundle ≤ 350 KB
   gzipped via code-split").
2. **Preload the critical font subset** — `@fontsource` ships `woff2` plus a
   `woff` fallback per weight; the Arabic families are the largest assets
   (108–140 kB each) and are referenced in `index.css` before LCP.
3. If neither closes the gap, the honest alternative is an owner decision to
   record a 2.0 s mobile LCP budget, with the desktop 1.5 s target unchanged.

Non-negotiable in any remediation: the field keeps `pointer-events: none`,
`aria-hidden`, `z-index: -1`, opacity ≤ 0.15, hidden-tab pause, and
`prefers-reduced-motion` stop.

## Not measured here

- axe scan on every route and a keyboard-only walkthrough of the core flows
  (both remain recorded as human-verification items in `13-VERIFICATION.md`;
  `@axe-core/playwright` is installed for Phase 14's E2E lane).
- 60 fps motion sampling. The field animates a composited transform only and
  the layout-shift readings above are zero, but no frame-rate trace exists.
