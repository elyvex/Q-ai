# Source layout

Q-ai is a Cargo workspace; application source lives in [`../crates/`](../crates/), not this directory. The root Cargo manifest is a virtual workspace manifest.

- [`cli`](../crates/cli/src/main.rs): the `qai` executable.
- [`application`](../crates/application/src/): composition and use-case services.
- [`server`](../crates/server/src/): loopback HTTP API and debug reader.
- [`quran-core`](../crates/quran-core/src/) and [`quran-corpus`](../crates/quran-corpus/src/): canonical models and corpus pipeline.
- [`quran-normalization`](../crates/quran-normalization/src/) and [`quran-search`](../crates/quran-search/src/): derived forms and search implementation.

See the [project README](../README.md) for current capabilities and placeholder boundaries.
