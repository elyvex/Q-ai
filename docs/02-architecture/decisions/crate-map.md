# Crate Map: Architecture (ADRs live at `docs/02-architecture/decisions/`)



---
##
### 0. Crate Map: The Core System Architecture



This crate organizes all Phase 0 ADRs, Core, and Provenance
 together with `docs/03-plan/current-plan.md` for navigation, and the primary
technical files of the project (workspace layout, error taxonomy, configuration precedence).

| ADR | Title | Phase | Location |
|---|---|---|
| ADR-0001 | Relational Store: SQLite + sqlx, PostgreSQL-Portable SQL | P0 | `docs/02-architecture/decisions/ADR-0001-relational-store.md` |
| ADR-0002 | Migration Strategy: Append-Only Checksummed SQL, Forward-Only Canonical | P0 | `docs/02-architecture/decisions/ADR-0002-migration-strategy.md` |
| ADR-0003 | Durable Job System: DB-Backed Leased Queue (No External Broker) | P0 | `docs/02-architecture/decisions/ADR-0003-durable-job-system.md` |
| ADR-0004 | Configuration & Precedence Model | P0 | `docs/02-architecture/decisions/ADR-0004-config-precedence.md` |
| ADR-0005 | Secret Storage per OS (Env / Keychain / Encrypted File) | P0 | `docs/02-architecture/decisions/ADR-0005-secret-storage.md` |
| ADR-0006 | Hashing & Canonical Serialization: SHA-256, Canonical JSON, NFC Policy | P0 | `docs/02-architecture/decisions/ADR-0006-hashing-canonical.md` |
| ADR-0007 | Source Manifest Format & Signing (ed25519 Detached over Canonical JSON) | P0 | `docs/02-architecture/decisions/ADR-0007-manifest-format-signing.md` |
| ADR-0008 | Provenance Representation: Single Universal Record + Typed Attribution | P0 | `docs/02-architecture/decisions/ADR-0008-provenance-representation.md` |
| ADR-0009 | Audit Integrity: Hash Chain + Append-Only Triggers | P0 | `docs/02-architecture/decisions/ADR-0009-audit-integrity.md` |
| ADR-0010 | Error Taxonomy & CLI Exit Codes | P0 | `docs/02-architecture/decisions/ADR-0010-error-taxonomy.md` |
| ADR-0011 | Observability Stack: Tracing + Metrics + Optional OTLP (Telemetry Opt-In) | P0 | `docs/02-architecture/decisions/ADR-0011-observability.md` |


---
##
##
## 2. Reading order matters

Read order matters:
