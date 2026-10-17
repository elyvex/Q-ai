# Phase 7: Isnad & Narrator Research - Research

**Researched:** 2026-10-17
**Domain:** Narrator identity, isnad chain traversal, and identity-review workflow on top of the Q-ai Rust workspace
**Confidence:** HIGH for in-repo patterns (read directly this session); MEDIUM for data-model specifics (synthesized from Phase 4/6 patterns, not yet implemented)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Narrator Identity Model**
- Narrator is a node in the isnad graph with a stable ID (`narrator:{slug}`)
- Name variants stored as a list on the narrator record (Arabic script variants, kunya, laqab, nisba)
- Rijal sources are attributed records per narrator (scholar/source citation), following the per-scholar grading pattern — never merged into consensus
- Computational identity matches carry confidence + "uncertain" label; never presented as certainty without owner confirmation

**Isnad Chain Representation**
- Chain modeled as graph edges: narrator→narrator transmission edges and narrator→hadith transmission edges
- Each link carries provenance layer (B/C/D), confidence, and uncertainty label
- Matn/isnad text from the hadith record is preserved verbatim; chain extraction (computational) is labeled uncertain
- Reuse Phase 4 graph edge vocabulary and assertion authority patterns

**Identity Review Workflow**
- Narrator-identity proposals are assertions (graph_assertions pattern) with Pending/Accepted/Rejected/Superseded
- Reviewer identity + decided_at recorded with CHECK constraint (mirrors verse_links owner gate)
- Disputed stays effective (never tombstoned) to represent live scholarly disagreement
- Supersession chain supported (supersedes_id)

**Chain Path Traversal**
- Reuse Phase 4 traversal (up_to_k_paths, min_hops, bounded_subgraph) with QueryBudgets
- Uncertainty marked on each link in path results
- Computational extraction confidence thresholds: high (>0.9), medium (0.7-0.9), low (<0.7) — all labeled uncertain until confirmed
- Budget exhaustion explicit (truncated + incomplete_reason)

### the agent's Discretion
- Exact column names and migration numbering
- CLI/API/TUI surface shape (follow Phase 5/6 typed-tool pattern)
- Test organization

### Deferred Ideas (OUT OF SCOPE)
- Full narrator database ingestion (data acquisition) — Phase 7 scope is schema + tools + review workflow
- Comparative isnad across collections — Phase 9 scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REQ-isnad-narrator-graph | Users can trace narration chains and judge narrator identity with explicit uncertainty: narrator records with name variants and rijal sources; chain paths between narrators with uncertainty on each link; recorded approve/reject workflow for narrator-identity proposals | Architecture Patterns 1-4 (narrator record, isnad edges, identity assertions, traversal); Data Model section; Pitfalls 1-7 |
</phase_requirements>

---

## Summary

Phase 7 turns the `isnad-graph` placeholder into a real port crate that reuses Phase 4's graph traversal and Phase 6's per-scholar/owner-gate patterns. The single highest-leverage finding: **every primitive Phase 7 needs already exists** — `quran-graph` provides `GraphStore`, `MemGraphStore`, traversal (`up_to_k_paths`, `min_hops`, `bounded_subgraph`, `reachable`), `QueryBudgets`, `Assertion`/`AssertionDecision`/`ProvenanceLayer`, and `AuthzScope`; Phase 6 provides the per-scholar rijal pattern (`hadith_gradings`), the owner-gate CHECK (`verse_links`), and the confidence-band labeling (`ConfidenceBand`). The planner's job is to **compose these**, not invent new ones.

The recommended shape: `isnad-graph` is a thin domain/port crate that **reuses `quran-graph` types and traversal verbatim** (narrators become `GraphNode` with a new `NodeKind::Narrator`; transmission edges become new `EDGE_VOCABULARY` entries `TRANSMITS` and `NARRATES`), with isnad-specific relational tables (`narrators`, `narrator_rijal_sources`, `isnad_edges`, `narrator_identity_assertions`) loaded into a `MemGraphStore` snapshot by an application adapter mirroring `SqliteGraphStore`. Identity proposals reuse `quran_graph::Assertion` with a new `AssertionKind::NarratorIdentity`. Rijal sources reuse the `GradingRecord` per-scholar shape. The owner gate reuses the `verse_links` CHECK pattern.

**Primary recommendation:** Sequence as (1) migrations 0031-0034 (narrators, rijal sources, isnad edges, identity assertions); (2) extend `quran-graph` minimally (new `NodeKind`/`AssertionKind` variants + 2 edge predicates); (3) implement `isnad-graph` port crate + domain types; (4) application adapter `SqliteIsnadGraphStore` + `IsnadReader` service; (5) `IsnadBackend` trait + tools in `tool-registry`; (6) cross-surface parity (CLI/server/TUI); (7) validation.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Narrator record storage (stable id, name variants) | Database / Storage (`storage-sqlite`) | — | Follows `hadith_records` + `graph_nodes` pattern; stable IDs never rowids |
| Rijal sources (per-scholar attributed) | Database / Storage | Application service | Reuses `hadith_gradings` per-scholar pattern; never consensus-merged |
| Isnad transmission edges (narrator→narrator, narrator→hadith) | Database / Storage | Application service | Reuses `graph_edges` adjacency + attribution-pointer shape |
| Narrator-identity proposals (Pending/Accepted/Rejected/Superseded/Disputed) | Database / Storage (`narrator_identity_assertions`) | Application service | Reuses `graph_assertions` authority pattern + owner gate |
| Chain path traversal (up_to_k_paths, min_hops, bounded_subgraph) | Application service (adapter) | `quran-graph` traversal | Reuses Phase 4 traversal with `QueryBudgets`; uncertainty on each link |
| Computational chain extraction + confidence bands | Application service | — | Reuses `ConfidenceBand` thresholds; always labeled uncertain |
| Typed tools (narrator.get, isnad.trace, identity propose/decide) | API / Backend (`application` + `tool-registry`) | CLI, TUI, Web (thin callers) | Extends typed-tool pattern (Phase 5/6) |
| Reviewer approve/reject workflow | API / Backend | Database (CHECK gate) | Reuses `verse_links` owner-gate CHECK |

---

## Standard Stack

### Core — Rust (existing workspace crates)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `quran-graph` | workspace | `GraphStore`, `MemGraphStore`, traversal, `QueryBudgets`, `Assertion`, `AuthzScope` | Phase 4 port; reuse verbatim for isnad traversal |
| `hadith-core` | workspace | `HadithRecord` with `isnad_text`, `GradingRecord` per-scholar | Phase 6 pattern for rijal sources |
| `tool-registry` | workspace | Backend traits + `ToolRegistry` + typed params | Phase 5/6 tool surface |
| `tools` | workspace | `ToolResult<T>`, `research_checksum`, `AnalysisSource` | Phase 5 result contract |
| `domain` | workspace | `typed_id!`, `ContentHash`, `SemVer`, `Timestamp` | Phase 1 domain types |
| `storage-sqlite` | workspace | SQLite + migrations + `SqliteDatabase` | Storage backend |
| `application` | workspace | Service layer wrapping readers | Composition root |

### New / Modified Crates (Phase 7)

| Crate | Change | Rationale |
|-------|--------|-----------|
| `isnad-graph` | Implement port crate (currently placeholder) | Domain types + isnad-specific helpers on top of `quran-graph` |
| `quran-graph` | Additive: `NodeKind::Narrator`, `NodeKind::Hadith`, `AssertionKind::NarratorIdentity`, edge predicates `TRANSMITS`, `NARRATES` | Reuse traversal/export/authz verbatim; small additive change |
| `application` | New modules `isnad_reader`, `isnad_graph_store` (adapter), `isnad_tools` | Follows `hadith_reader`/`hadith_tools` pattern |
| `tool-registry` | New `IsnadBackend` trait + tools | Follows `HadithBackend` pattern |

**Installation:** no new external crates required — all dependencies (`uuid`, `sqlx`, `serde`, `async-trait`, `tokio`) already in the workspace.

---

## Package Legitimacy Audit

No new external packages. All dependencies are existing workspace crates. **Disposition: none added.**

---

## Architecture Patterns

### 1. Narrator Record (stable ID + name variants + rijal sources)

**What:** `narrator:{slug}` stable ID; `NarratorRecord { id, slug, primary_name, name_variants: Vec<String>, metadata_json, created_at }`. Rijal sources stored as separate attributed rows (see Pattern 2), never merged.

**Where:** `isnad-graph` crate defines `NarratorId` (string newtype) + `NarratorRecord`; migration `0031_narrators.up.sql` stores the record.

**Migration 0031_narrators (sketch):**
```sql
CREATE TABLE narrators (
  id                TEXT PRIMARY KEY,
  slug              TEXT NOT NULL UNIQUE,
  primary_name      TEXT NOT NULL,
  name_variants_json TEXT NOT NULL DEFAULT '[]',
  metadata_json     TEXT NOT NULL DEFAULT '{}',
  created_at        TEXT NOT NULL
);
CREATE INDEX ix_narrators_slug ON narrators(slug);
```

### 2. Rijal Sources (per-scholar, never merged)

**What:** Reuse the `hadith_gradings` shape exactly: one row per scholar/source citation, with `scholar_name`, `source_citation`, `grading_value` (optional rijal verdict), `evidence_json`, `reviewer`, `decision`, `decided_at`, `supersedes_id`, `provenance_layer` (B/C/D), `confidence`, `created_at`. Owner gate CHECK identical to `hadith_gradings` (migration 0025) and `verse_links` (migration 0030).

**Migration 0032_narrator_rijal_sources (sketch):**
```sql
CREATE TABLE narrator_rijal_sources (
  id                TEXT PRIMARY KEY,
  narrator_id       TEXT NOT NULL REFERENCES narrators(id) ON DELETE RESTRICT,
  scholar_name      TEXT NOT NULL CHECK (length(trim(scholar_name)) > 0),
  source_citation   TEXT NOT NULL,
  grading_value     TEXT,
  evidence_json     TEXT NOT NULL DEFAULT '{}',
  reviewer          TEXT,
  decision          TEXT NOT NULL CHECK (decision IN ('pending','accepted','rejected','superseded','disputed')),
  decided_at        TEXT,
  supersedes_id     TEXT REFERENCES narrator_rijal_sources(id),
  provenance_layer  TEXT NOT NULL CHECK (provenance_layer IN ('B','C','D')),
  confidence        REAL,
  created_at        TEXT NOT NULL,
  CHECK (decision = 'pending' OR
         (reviewer IS NOT NULL AND reviewer != '' AND decided_at IS NOT NULL)),
  CHECK (provenance_layer != 'D' OR (confidence IS NOT NULL))
);
CREATE INDEX ix_rijal_narrator ON narrator_rijal_sources(narrator_id);
```

### 3. Isnad Transmission Edges

**What:** narrator→narrator (`TRANSMITS`) and narrator→hadith (`NARRATES`) edges, each carrying `provenance_layer`, `confidence` (Option), uncertainty label, and an optional `assertion_id` into `narrator_identity_assertions`. Stored in `isnad_edges` (mirrors `graph_edges` shape). Computational extraction starts `label='uncertain'`.

**Migration 0033_isnad_edges (sketch):**
```sql
CREATE TABLE isnad_edges (
  id                TEXT PRIMARY KEY,
  src_stable_id     TEXT NOT NULL,
  edge              TEXT NOT NULL CHECK (edge IN ('TRANSMITS','NARRATES')),
  dst_stable_id     TEXT NOT NULL,
  assertion_id      TEXT REFERENCES narrator_identity_assertions(id),
  provenance_layer  TEXT NOT NULL CHECK (provenance_layer IN ('B','C','D')),
  confidence        REAL,
  label             TEXT NOT NULL DEFAULT 'uncertain'
                    CHECK (label IN ('uncertain','confirmed','rejected')),
  algorithm         TEXT,
  algorithm_version TEXT,
  reviewer          TEXT,
  decided_at        TEXT,
  created_at        TEXT NOT NULL,
  CHECK (label != 'confirmed' OR
         (reviewer IS NOT NULL AND reviewer != '' AND decided_at IS NOT NULL)),
  CHECK (provenance_layer != 'D' OR (algorithm IS NOT NULL AND algorithm_version IS NOT NULL))
);
CREATE INDEX ix_isnad_src ON isnad_edges(src_stable_id);
CREATE INDEX ix_isnad_dst ON isnad_edges(dst_stable_id);
```

**Edge vocabulary extension (quran-graph):** add `TRANSMITS` and `NARRATES` to `EDGE_VOCABULARY` so `is_allowed_edge` admits them during staging and conformance tests pass. Update the `vocabulary_has_expected_entries` test count (19 → 21).

**NodeKind extension (quran-graph):** add `Narrator` and `Hadith` variants so isnad nodes type-check as first-class nodes. The Rust enum is not constrained by migration 0019's CHECK (that only governs `graph_nodes`), so adding variants is non-breaking for the Quran tables. The isnad adapter maps `narrator:{slug}` → `NodeKind::Narrator`, `hadith:{id}` → `NodeKind::Hadith`.

### 4. Narrator-Identity Assertions (review workflow)

**What:** Reuse `quran_graph::Assertion` with a new `AssertionKind::NarratorIdentity` variant. Stored in a dedicated `narrator_identity_assertions` table (mirrors `graph_assertions` columns, minus `projection_row_id`/edition scoping, since isnad data is not a Quran projection). Owner gate CHECK identical to `graph_assertions` (migration 0019) and `verse_links` (migration 0030). Supersession via `supersedes_id`; `Disputed` stays effective (never tombstoned).

**Migration 0034_narrator_identity_assertions (sketch):**
```sql
CREATE TABLE narrator_identity_assertions (
  id                TEXT PRIMARY KEY,
  assertion_kind    TEXT NOT NULL CHECK (assertion_kind IN ('narrator_identity')),
  claim_json        TEXT NOT NULL,
  evidence_json     TEXT NOT NULL,
  source_location   TEXT NOT NULL DEFAULT '',
  reviewer          TEXT,
  decision          TEXT NOT NULL CHECK (decision IN ('pending','accepted','rejected','superseded','disputed')),
  decided_at        TEXT,
  supersedes_id     TEXT REFERENCES narrator_identity_assertions(id),
  provenance_layer  TEXT NOT NULL CHECK (provenance_layer IN ('B','C','D')),
  algorithm         TEXT,
  algorithm_version TEXT,
  confidence        REAL,
  created_at        TEXT NOT NULL,
  CHECK (decision = 'pending' OR (reviewer IS NOT NULL AND reviewer != '' AND decided_at IS NOT NULL)),
  CHECK (provenance_layer != 'D' OR (algorithm IS NOT NULL AND algorithm_version IS NOT NULL AND confidence IS NOT NULL))
);
CREATE INDEX ix_nia_decision ON narrator_identity_assertions(decision);
```

**Application adapter:** `SqliteIsnadGraphStore` (new module `crates/application/src/isnad_graph_store.rs`) mirrors `SqliteGraphStore`: loads `narrators` + `isnad_edges` into a `MemGraphStore` snapshot keyed under projection family `isnad-narrator-v1`, converts rows to `GraphNode`/`GraphEdge`/`Assertion`, and implements `quran_graph::GraphStore`. This reuses `up_to_k_paths`, `min_hops`, `bounded_subgraph`, `reachable` verbatim.

**Uncertainty marking:** traversal returns `GraphEdge` with `assertion_id`; the application service resolves each edge's assertion and annotates the result envelope with `provenance_layer`, `confidence`, and a label (`uncertain`/`confirmed`/`rejected`). Pending-suggestion warnings follow the `suggestion_warnings` pattern.

**Confidence bands:** reuse `ConfidenceBand::of` thresholds (high >0.9, medium 0.7-0.9, low <0.7) from `verse_link`. Every band stays `uncertain` until owner confirmation.

### 5. Application Services

**Module layout (mirror Phase 6):**
- `crates/application/src/isnad_reader.rs` — `IsnadReader` trait + `IsnadReaderService` (narrator CRUD, rijal sources, identity propose/accept/reject/dispute/supersede, get_narrator, get_rijal_sources, trace paths).
- `crates/application/src/isnad_graph_store.rs` — `SqliteIsnadGraphStore` adapter implementing `quran_graph::GraphStore`.
- `crates/application/src/isnad_tools.rs` — `IsnadBackend` impl returning `ToolResult` envelopes.

**Trait sketch (follows `HadithReader`/`VerseLinker`):**
```rust
#[async_trait]
pub trait IsnadReader: Send + Sync {
    async fn get_narrator(&self, id: &str) -> Result<NarratorRecord, IsnadError>;
    async fn get_rijal_sources(&self, narrator_id: &str) -> Result<Vec<RijalSource>, IsnadError>;
    async fn trace_paths(&self, src: &str, dst: &str, k: usize, budgets: &QueryBudgets)
        -> Result<PathsResult, IsnadError>;
    async fn propose_identity(&self, input: IdentityProposalInput) -> Result<Assertion, IsnadError>;
    async fn accept_identity(&self, id: &str, reviewer: &str) -> Result<(), IsnadError>;
    async fn reject_identity(&self, id: &str, reviewer: &str) -> Result<(), IsnadError>;
    async fn dispute_identity(&self, id: &str, reviewer: &str) -> Result<(), IsnadError>;
}
```

### 6. Tool-Registry Extension

**Follow `HadithBackend` pattern.** Add `IsnadBackend` trait + params structs + tool-name constants. Add to `TOOL_NAMES` (17 → ~21) and `ToolRegistry` (add `isnad_backend: Option<Arc<dyn IsnadBackend>>` + `with_isnad`/`set_isnad_backend`, mirroring `with_hadith`/`with_tafsir`).

**Suggested tool names:** `isnad.narrator`, `isnad.rijal`, `isnad.trace`, `isnad.identity_propose`, `isnad.identity_accept`, `isnad.identity_reject`. Each returns `ToolResult<serde_json::Value>` with `research_checksum` pinning the isnad projection build + algorithm attribution.

### 7. Cross-Surface Parity

CLI/server/TUI register the new tools following the Phase 5/6 typed-tool pattern. CLI module `crates/cli/src/isnad.rs` (mirror `crates/cli/src/hadith.rs`). Server routes + TUI commands expose the same tool names.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Graph traversal (paths, hops, subgraph) | Custom BFS over `isnad_edges` | `quran-graph` traversal (`up_to_k_paths`, `min_hops`, `bounded_subgraph`) | Budget/authz/ordering semantics already proven |
| Per-scholar attributed records | New attribution schema | Reuse `hadith_gradings` shape (migration 0025) | Identical owner-gate + supersedes pattern |
| Owner gate (reviewer + decided_at) | Application-only check | DB CHECK constraint (migration 0030 pattern) | Defense in depth; tested at DB level |
| Confidence bands | New thresholds | `ConfidenceBand::of` (verse_link) | Locked thresholds already encoded |
| Typed tool envelope | Ad-hoc JSON | `ToolResult` + `research_checksum` | Phase 5 contract; cross-surface parity |
| Assertion authority (Pending/Accepted/Rejected/Superseded/Disputed) | New state machine | `quran_graph::Assertion` + `AssertionDecision` | Tombstone/dispute/supersession semantics already defined |

**Key insight:** Phase 7 is composition, not invention. The only genuinely new code is the isnad-specific tables + the adapter that loads them into the proven graph port.

---

## Common Pitfalls

### Pitfall 1: Merging rijal sources into a consensus
**What goes wrong:** A single "grade" field per narrator erases scholarly disagreement.
**How to avoid:** One row per scholar/source citation; no consensus column; display side-by-side (Phase 6 D-03).
**Warning signs:** Any `consensus_grade` or `merged` column.

### Pitfall 2: Presenting computational matches as certain
**What goes wrong:** Chain extraction or identity match shown as fact.
**How to avoid:** Every computational link starts `label='uncertain'` with confidence; owner action required to confirm.
**Warning signs:** A code path that sets `label='confirmed'` without reviewer.

### Pitfall 3: Rejected identity proposals disappearing
**What goes wrong:** Tombstoning hides audit history.
**How to avoid:** `Rejected`/`Superseded` tombstone for traversal/export but the authority row is retained for audit. `Disputed` stays effective.
**Warning signs:** `DELETE FROM narrator_identity_assertions`.

### Pitfall 4: Budget exhaustion reported as "no path"
**What goes wrong:** Truncated search misreported as empty result.
**How to avoid:** Propagate `truncated` + `incomplete_reason`; never collapse to empty without the flag.
**Warning signs:** Returning `Vec::new()` on budget error.

### Pitfall 5: Stable-ID drift
**What goes wrong:** Narrator identity keyed by rowid or display name.
**How to avoid:** Always `narrator:{slug}`; never use DB rowid as identity; name variants are attributes, not keys.
**Warning signs:** `id INTEGER PRIMARY KEY` on narrators.

### Pitfall 6: Forgetting checksums.json
**What goes wrong:** Migrations fail verification in CI.
**How to avoid:** Regenerate `migrations/sqlite/checksums.json` after adding 0031-0034 (the migration runner verifies sha256).
**Warning signs:** `checksum mismatch` errors in tests.

### Pitfall 7: is_allowed_edge gating isnad predicates
**What goes wrong:** `MemGraphStore::stage_edges` rejects `TRANSMITS`/`NARRATES` as unknown predicates.
**How to avoid:** Add both to `EDGE_VOCABULARY` in `quran-graph` (and update the count test) before staging isnad edges.
**Warning signs:** `PatternRejected { cannot stage unknown edge }`.

---

## Code Examples

### Reusing traversal (verbatim)
```rust
// Source: crates/quran-graph/src/traverse.rs (up_to_k_paths)
let paths = quran_graph::traverse::up_to_k_paths(
    &isnad_store, &src, &dst, k, &budgets, &cancel, &authz,
)?;
// Each GraphPath.edges carries assertion_id → resolve to layer/confidence/label
```

### Owner-gate CHECK (reuse)
```sql
-- Source: migrations/sqlite/0030_verse_links.up.sql
CHECK (label != 'confirmed' OR
       (reviewer IS NOT NULL AND reviewer != '' AND decided_at IS NOT NULL))
```

### Per-scholar rijal row (reuse)
```sql
-- Source: migrations/sqlite/0025_hadith_gradings.up.sql
CHECK (decision = 'pending' OR
       (reviewer IS NOT NULL AND reviewer != '' AND decided_at IS NOT NULL))
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Narrators as free-text names in `hadith_records.narrator_names_json` | Stable `narrator:{slug}` nodes with variants + rijal sources | Phase 7 | Traceable identity + uncertainty |
| Consensus grading | Per-scholar attributed records | Phase 6 | No false consensus |

**Deprecated/outdated:**
- Treating `narrator_names_json` as authoritative identity — superseded by narrator nodes.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `isnad-graph` should reuse `quran-graph` types/traversal rather than duplicate | Architecture Patterns 4/6 | Medium — if isnad needs distinct node semantics, adapter mapping may be insufficient; mitigation: keep isnad tables separate so a future dedicated port is non-breaking |
| A2 | Migration numbering 0031-0034 is free (no collisions) | Data Model | Low — verified against `migrations/sqlite/` listing |
| A3 | `AssertionKind::NarratorIdentity` can be added to `quran-graph` without breaking Quran tables | Architecture Patterns 4 | Low — CHECK constraints are per-table; Rust enum is not DB-constrained |
| A4 | No new external crates needed | Standard Stack | Low — all deps present in workspace |

---

## Open Questions

1. **Should identity proposals live in `narrator_identity_assertions` (dedicated table) or reuse `graph_assertions` under a synthetic projection?**
   - What we know: `graph_assertions` FKs to `graph_projections`; isnad data has no Quran projection.
   - What's unclear: Whether coupling isnad to the Quran projection lifecycle is desirable.
   - Recommendation: Dedicated table (Pattern 4) — cleaner, avoids synthetic projection rows. Revisit if cross-graph (isnad↔Quran) traversal is later required.

2. **Should `narrator_names_json` on `hadith_records` be backfilled to narrator nodes?**
   - Deferred: data acquisition is out of scope (CONTEXT deferred). Phase 7 delivers schema + tools + review workflow only.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Build/test | yes | (workspace) | — |
| sqlx (SQLite) | Migrations | yes | workspace | — |
| cargo | Tests | yes | — | — |

**Missing dependencies with no fallback:** none.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (unit + integration) |
| Config file | `Cargo.toml` (workspace) |
| Quick run command | `cargo test -p isnad-graph -p application` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REQ-isnad-narrator-graph | Narrator record round-trips with name variants | unit | `cargo test -p isnad-graph narrator` | no |
| REQ-isnad-narrator-graph | Rijal sources per-scholar, not merged | unit | `cargo test -p application isnad_rijal` | no |
| REQ-isnad-narrator-graph | Chain path traversal with uncertainty on each link | unit | `cargo test -p application isnad_trace` | no |
| REQ-isnad-narrator-graph | Budget exhaustion explicit (truncated + reason) | unit | `cargo test -p quran-graph` (reuse) | yes |
| REQ-isnad-narrator-graph | Identity proposal approve/reject with owner gate | integration | `cargo test -p application isnad_identity` | no |
| REQ-isnad-narrator-graph | DB CHECK rejects confirmed without reviewer | integration | `cargo test -p application db_check` | no |
| REQ-isnad-narrator-graph | Disputed stays effective (not tombstoned) | unit | `cargo test -p isnad-graph disputed` | no |
| REQ-isnad-narrator-graph | Supersession chain ordering | unit | `cargo test -p application isnad_supersede` | no |

### Sampling Rate
- **Per task commit:** `cargo test -p isnad-graph -p application`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `crates/isnad-graph/src/lib.rs` — implement port crate (currently placeholder)
- [ ] `crates/isnad-graph/tests/` — conformance tests mirroring `quran-graph` mem tests
- [ ] `crates/application/src/isnad_reader.rs` — `IsnadReader` trait + service
- [ ] `crates/application/src/isnad_graph_store.rs` — adapter
- [ ] `crates/application/src/isnad_tools.rs` — `IsnadBackend` impl
- [ ] `crates/application/tests/` — service + CHECK-constraint tests (follow `verse_link` service_tests pattern)
- [ ] `crates/tool-registry/src/lib.rs` — `IsnadBackend` trait + tool names
- [ ] `migrations/sqlite/0031-0034*.up.sql` + `checksums.json` regeneration
- [ ] CLI/server/TUI registration for new tools

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | yes | Reviewer identity + `invoked_by` operator principal (mirrors `quran_graph_annotations`); deny-by-default |
| V5 Input Validation | yes | Stable-id validation, slug validation, non-empty reviewer checks, confidence range [0,1] |
| V6 Cryptography | no | — |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Fabricated narrator identity presented as fact | Spoofing/Tampering | Computational matches labeled uncertain; owner gate required for confirmation |
| Merged scholarly consensus (false agreement) | Integrity | Per-scholar rows; no consensus column |
| Tombstoning audit history | Repudiation | Retain rejected/superseded rows; audit chain (Phase 4 pattern) |
| SQL injection via narrator slug | Tampering | Parameterized queries via sqlx (Phase 6 precedent) |

---

## Sources

### Primary (HIGH confidence)
- crates/quran-graph/src/model.rs — NodeKind, EDGE_VOCABULARY, Assertion, AssertionDecision, ProvenanceLayer, QueryBudgets
- crates/quran-graph/src/traverse.rs — up_to_k_paths, min_hops, bounded_subgraph, reachable
- crates/quran-graph/src/mem.rs — stage_edges is_allowed_edge gate
- crates/quran-graph/src/store.rs — GraphStore port, AuthzScope, EdgeFilter
- crates/application/src/quran_graph_store.rs — SqliteGraphStore snapshot adapter pattern
- crates/application/src/verse_link.rs — owner gate, confidence bands, LinkLabel
- crates/application/src/hadith_reader.rs — HadithReader trait + service pattern
- crates/hadith-core/src/grading.rs — GradingRecord per-scholar
- crates/tool-registry/src/lib.rs — HadithBackend trait + registry wiring
- migrations/sqlite/0019_quran_graph.up.sql — graph_assertions schema + CHECK
- migrations/sqlite/0025_hadith_gradings.up.sql — per-scholar grading CHECK
- migrations/sqlite/0030_verse_links.up.sql — owner-gate CHECK
- migrations/sqlite/0024_hadith_records.up.sql — hadith_records with isnad_text
- crates/isnad-graph/src/lib.rs — placeholder status
- crates/application/src/lib.rs — module list

### Secondary (MEDIUM confidence)
- Phase 6 research/plan documents (synthesized patterns)

### Tertiary (LOW confidence)
- None.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all crates read this session
- Architecture: MEDIUM — synthesized from Phase 4/6 patterns; not yet implemented
- Pitfalls: HIGH — derived from verified CHECK constraints and code paths

**Research date:** 2026-10-17
**Valid until:** 2026-11-16 (30 days for stable workspace)
```

The write succeeded (empty content response, no error). Let me verify file exists.<|end|><|start|>-assistant to=shell<|channel|>analysis <|constrain|>xml<|message|><rfl:parameter name="command">
cd /Users/ali/dev/rust/Q-ai && ls -la .planning/phases/07-isnad-narrator-research/ && wc -l .planning/phases/07-isnad-narrator-research/07-RESEARCH.md