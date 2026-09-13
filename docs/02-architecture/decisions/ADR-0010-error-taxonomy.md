# ADR-0010 — Error Taxonomy & CLI Exit Codes

- Status: Accepted
- Phase: 0 — Foundations
- Date: 2026-09-11
- Related decisions: ADR-0001, ADR-0003
- Requirements: PRD §§25.13, 25.4, 26

## Context

Every Q-ai error must answer five questions: **what** happened, **why**, **where**
(file/key/source id), **how to fix**, and **what command to run next**. Errors are
a first-class diagnostic surface — shell scripts and IDE integrations parse them,
so codes must be stable forever once published.

Two separate axis are needed: an error *code* namespace (what class of failure),
and a CLI *exit code* (how the shell reacts).

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| **Namespace + sequential (`QAI-DOM-0001`)** | Human-readable prefix; unambiguous mapping to module | Longer codes |
| Single global counter (`QAI-0001`) | Short | No module context; useless for routing |
| Plain string messages | Flexible | Unstable across versions; unparseable by scripts |
| HTTP-style status codes | Familiar | Wrong axis (transport vs. domain); too few codes |

## Decision

Two independent encodings:

### 1. Error codes (semantic, in `Diagnostic.code`)

Namespaces with `nnnn` sequential IDs, registered in `domain/src/error.rs`:

```text
QAI-CFG-nnnn  configuration        (D0.4)
QAI-SEC-nnnn  secrets / security   (D0.5)
QAI-DB-dnnn   storage / migration  (D0.6 — lowercase 'd' distinguishes from storage layer)
QAI-JOB-nnnn  job system           (D0.9)
QAI-SRC-nnnn  source catalog       (D0.10)
QAI-PROV-nnnn provenance           (D0.11)
QAI-AUD-nnnn  audit                (D0.12)
QAI-CLI-nnnn  CLI usage            (D0.13)
QAI-QUR-nnnn  Quran corpus         (reserved, Phase 1)
QAI-NORM-nnnn normalization        (reserved, Phase 2)
QAI-IDX-nnnn  indexes              (reserved, Phase 2)
```

Each library error type implements `Diagnostic` (summary, cause chain, location,
remedy, next command, is_retryable, redacted). A unit test asserts every emitted
code is unique and registered.

### 2. CLI exit codes (shell axis, 0..70)

| Code | Meaning |
|---|---|
| 0 | OK |
| 1 | Generic error |
| 2 | Usage error |
| 3 | Validation failed |
| 4 | Denied by policy |
| 5 | Not found |
| 6 | Conflict / state |
| 7 | Cancelled |
| 70 | Internal error |

`-` prefixed, exit is `1 | 2 | 3` family per error `Diagnostic` mapping; denied
by policy exits `4`.

## Accuracy and Religious-Source Implications

None directly. Stable error codes are a hygiene requirement for any future
automation (e.g., a `qai` wrapper that retries on `QAI-DB-0003` but stops on
`QAI-SEC-0001`).

## Licensing Implications

None. Error codes are definitions, not code.

## Security Implications

- `redacted = true` errors must never include the offending secret even in
  `--json` form (JSON schema validated).
- `QAI-SEC-*` codes can surface during security audits; their messages must
  remain non-leaking.

## Operational Implications

- Exit-code stability is guaranteed by CONTRIBUTING (codes are public API).
- `--json` error output is schema-stable for tool integration.

## Migration Strategy

Codes are assigned sequentially within each namespace and never reused. New
phases pick up the next available ID in their reserved range.

## Reversal Cost

High for published codes; low while still in Phase 0 (no consumers yet).

## Acceptance Criteria

- Every library error type implements `Diagnostic`; a conformance test verifies.
- All emitted codes are unique (uniqueness test).
- `--json` output matches `docs/schemas/error.v1.schema.json`.
- Exit codes match the 0..70 table on every path.
