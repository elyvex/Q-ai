# Test layout

This root directory is reserved; tests currently live alongside workspace crates.

- `crates/*/src/`: unit tests, including property tests.
- [`crates/application/tests/`](../crates/application/tests/): corpus lifecycle, reading, normalization, indexing, search and diagnostic integration tests.
- [`crates/storage-sqlite/tests/`](../crates/storage-sqlite/tests/): real SQLite integrity, migration, recovery and consistency tests.
- [`crates/cli/tests/`](../crates/cli/tests/): CLI acceptance cases and doctor JSON tests.
- [`crates/server/tests/`](../crates/server/tests/): HTTP API tests.
- [`crates/testkit/`](../crates/testkit/): shared fixtures and cross-cutting security/configuration tests.
- [`fixtures/quran/`](../fixtures/quran/): synthetic and adversarial editions plus reference-parser golden data. Synthetic editions are not Quran text.

Run from the repository root:

```bash
cargo test --workspace
cargo test -p application -- --test-threads=1
cargo test -p cli
cargo test -p server
```

The serial application command is useful when investigating the parallel-run timeout recorded in the [follow-ups](../docs/05-followups/followups.md); it does not replace the full workspace gate. Fixture tests do not substitute for licensed-corpus validation, editorial review or recorded phase acceptance.

See the [project README](../README.md#development-and-verification) for lint/check commands.
