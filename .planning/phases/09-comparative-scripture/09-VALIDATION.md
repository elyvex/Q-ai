# Phase 9 Validation Map

| Plan | Task | Automated verify |
|------|------|------------------|
| 09-01 | scripture domain types + migrations 0039-0041 | cargo test -p scripture |
| 09-02 | translations + search FTS + tools + CLI | cargo test -p application -- scripture; cargo test -p cli -- scripture |
| 09-03 | parallel passages + verse_links + tool | cargo test -p application -- scripture |
| 09-04 | KnowledgeDomain filter + cross-domain aggregator | cargo test -p application -- cross_domain |
| 09-05 | HTTP routes + TUI panels + parity + CLI research | cargo test -p server -- scripture; cargo test -p tui -- scripture; cargo test -p cli -- scripture; cargo test -p tool-registry |
| 09-06 | owner gates + deferrals + validation + phase gate | cargo test --workspace (changed crates); xtask arch-check |

## Phase Gate
- `cargo test --workspace` (changed crates) — green
- `cargo xtask arch-check` — green
- `cargo build --workspace` — green
