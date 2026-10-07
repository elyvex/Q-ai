# API Coverage — Phase 5: Rich Quran Experience

> Full coverage by default. Opt-outs are explicit, reasoned decisions.

No external API integration: Phase 5 consumes the project's **own local** `qai serve` `/api/v1`
surface (and the shared `crates/application` services layer) — it wraps no third-party external
API/SDK/service and makes no network egress (`security.allow_network_egress = false`). The
`api-coverage` detector fired only on the phrase "consuming the versioned HTTP API (`/api/v1`)",
which is this project's first-party interface, not an external integration.
