# Development tooling

This directory is reserved; it currently contains no executable scripts. Repository automation lives in [`../xtask/`](../xtask/) and the Cargo alias in [`../.cargo/config.toml`](../.cargo/config.toml).

Run from the repository root:

```bash
cargo xtask --help
cargo xtask arch-check
cargo xtask migrate-check
```

See the [project README](../README.md#development-and-verification) and [contribution guide](../CONTRIBUTING.md) for build, test and lint commands.
