---
phase: 13-web-ui-redesign
plan: 15
subsystem: web
tags: [web, ui, gap-closure, g-13-1, g-13-2, animated-background, design-tokens, lighthouse, motion]

requires:
  - phase: 13-web-ui-redesign
    provides: the Concordance token sheet in `web/src/index.css` and the 13-route SPA shell
provides:
  - AnimatedBackground mounted once by the app shell on every route, painted from a tokenised `.gradient-field` atom and one compositor-only keyframe
  - the page ground moved to `body` so a negative-z-index field is not repainted over
  - one design-token source (`index.css` + `tokens/layers.css`) with the stale TypeScript/CSS mirrors deleted
  - a mount + visual-contract regression test and a design-contract guard that fail on drift
  - measured motion evidence (Lighthouse desktop + mobile, LCP/CLS/TBT/bundle) with the one missed budget recorded as remediation
affects: [phase 14 E2E + performance lane, any future view that assumes a motionless ground]

actuals:
  tokens: 12000
  tasks: 4
  commits: 1

tech-stack:
  added: [lighthouse (dev), @playwright/test (dev), @axe-core/playwright (dev)]
  patterns:
    - "Decorative layers carry zero colour literals: the component sets behaviour (z-index, pointer-events, opacity clamp, pause, reduced-motion) and the CSS atom sets paint from `--primary`/`--ring` via `color-mix`."
    - "Page ground belongs to `body`; any opaque wrapper above a negative-z-index layer hides it. The regression test asserts the ancestor chain, not just presence."
    - "Design tokens have exactly one source; a guard test pins the palette anchors and asserts the retired mirrors stay absent, so drift cannot re-enter through a second palette."
---

# Phase 13 Plan 15: Gap closure for the animated field and motion evidence

## What shipped

**G-13-1 — the animated background is real.** `AnimatedBackground` was
implemented and unit-tested but imported nowhere, and `App.tsx` painted an
opaque `bg-background` page ground that would have hidden it anyway. It also
hard-coded the superseded teal gradient and set `animation-play-state` with no
keyframes behind it, so mounting alone would have produced a static wash.

- `AppShell` mounts `<AnimatedBackground />` as its first child, once, for
  every route.
- The paint moved out of the component into a `.gradient-field` atom in
  `index.css`, built from `color-mix(in oklch, var(--primary) …)` and
  `var(--ring)`; `@keyframes gradient-drift` drifts a composited transform
  (24 s, alternate). No colour literal remains in the component.
- `App.tsx` no longer paints the page ground; `body` already does, where it
  propagates to the canvas beneath the field.
- Behaviour is unchanged: `aria-hidden`, `pointer-events: none`,
  `z-index: -1`, opacity ≤ 0.15, pause while the tab is hidden, 0 ms under
  `prefers-reduced-motion`.

**Token mirrors retired (advisory from the verification report).**
`web/src/tokens/index.ts` and `colors/elevation/typography/motion.css` all
declared the superseded teal/dark palette and were loaded only by
`reachability.test.ts`. Deleted rather than reconciled so a second palette
cannot survive; `layers.css` (read by `cargo test -p application --test
layer_separation`) and `themes.css` are untouched. `reachability.test.ts`
keeps its route matrix and gains a Concordance design-contract guard.

**G-13-2 — motion performance measured.** `13-MOTION-EVIDENCE.md` records
five Lighthouse runs (desktop + mobile emulation, `/` and `/search`) against
the roadmap budgets. A11y 1.00, CLS ≤ 0.0003, desktop LCP 448–452 ms, bundle
156.99 kB gz JS + 31.22 kB gz CSS. One budget misses: **mobile-emulated LCP
1.9 s vs the 1.5 s target** — recorded with a remediation path (route-level
code splitting, which Phase 14 SC11 already owns).

## Deviations from the plan text

1. **Task 4 option A was pre-approved by the operator** during this run, so
   the checkpoint resolved without a second prompt. The devDependency diff is
   this session's install (approved), not pre-existing operator work as the
   plan's context assumed.
2. **`tsconfig.app.json` gained `"node"` in `types`.** The new guards read
   sheet sources from disk; `@types/node` was already a devDependency but not
   in the `types` list, so `tsc -b` rejected `node:fs`.
3. **Case E of the guard was tightened, not copied.** The plan asked the
   component to contain both `gradient-field` and `var(--`, but Task 2 moves
   all colour into the atom. The test now asserts the atom carries
   `var(--primary)` and that the component carries `gradient-field` and no
   `radial-gradient`/`rgb(` literal.
4. **File reads use `process.cwd()`-resolved paths.** `import.meta.url` is not
   a `file:` URL under this vitest transform (`TypeError: The URL must be of
   scheme file`), so `new URL(...)` could not be used in the `.ts` guard.

## Evidence

| Gate | Result |
|------|--------|
| `npm test` (24 files / 187 tests) | PASS — includes 5 new shell cases |
| `npm run build` (`tsc -b && vite build`) | PASS — 156.99 kB gz JS |
| `cargo test -p application --test layer_separation` | PASS — 7/7 |
| Regression proof | the guard was written first and failed with `AnimatedBackground is NOT MOUNTED in the app shell` before the mount existed |
| `13-MOTION-EVIDENCE.md` | present, per-budget verdicts, one FAIL + remediation |

## Not done

- 60 fps frame sampling for the field (composited transform + zero CLS only).
- Route-level code splitting; it is the named remediation for the mobile LCP
  miss and belongs to Phase 14's bundle-budget work.
- axe-per-route scan and keyboard-only walkthrough — both remain human
  verification items in `13-VERIFICATION.md`.
