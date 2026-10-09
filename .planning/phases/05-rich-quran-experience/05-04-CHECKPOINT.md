# 05-04 Task 1 Checkpoint Record — Pinned Frontend Versions

**Date:** 2026-10-09
**Gate:** blocking-human (T-05-SC)
**Recorded by:** orchestrator, on the owner's explicit confirmation

## Confirmed pins (exact, no `^`)

| Package | Version | Publisher (canonical) |
|---|---|---|
| react | 19.3.0 | facebook/react |
| react-dom | 19.3.0 | facebook/react |
| vite | 8.3.3 | vitejs/vite |
| @vitejs/plugin-react | 6.1.2 | vitejs/vite |
| typescript | ~6.0.2 | microsoft/TypeScript (6.x template pin; do not chase 7.x) |
| vitest | confirmed | vitest-dev/vitest |
| jsdom | confirmed | jsdom/jsdom |
| @testing-library/react | confirmed | testing-library/react-testing-library |
| @testing-library/dom | confirmed | testing-library/dom-testing-library |

## Human confirmation

confirmed — owner verified the pins against npmjs.com and authorized `npm install` with the versions above.

## Authorization scope

- `npm install` / `npm ci` inside `web/` only, with the exact pins above.
- No Arabic webfont bundled (OD-04 stays open); no CDN/network references (D-04).
- `web/dist` + `web/node_modules` stay gitignored build artifacts.
