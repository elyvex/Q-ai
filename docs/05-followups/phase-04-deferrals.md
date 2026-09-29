# Phase 4 (Quran Graph) — Deferral Ledger

> **Purpose.** Records, explicitly, everything Phase 4 deliberately did **not**
> do, with the rationale and the target phase, plus the D-01 evidence matrix
> mapping each success criterion to its implementation and repeatable check.
> Items here are **deferred, not dropped**. It mirrors the ledger shape of
> `docs/05-followups/phase-03-deferrals.md`.
>
> **Scope note.** This ledger records *scope* deferrals. The six owner gates
> (OD-11 dataset/license, OD-12 linguist, P4-X01/X02/X03/X05 ratifications) are
> **blocks**, not deferrals, and are recorded separately in
> `docs/05-followups/phase-04-owner-gates.md`. `decisions-needed.md` (OD-11/OD-12)
> and `docs/03-plan/phases/phase-04-quran-graph/tasks.md` (swimlane X) remain
> the sources of truth for those.
>
> **Rule for agents:** adding an item here is the honest alternative to silently
> omitting it. Never move an item off this list except by implementing it under a
> plan that names it.

---

## 1. Tafsir / hadith / isnad graph links → roadmap Phases 6–7

| Deferred item | Rationale | Target |
|---|---|---|
| **Tafsir** commentary links | Content scope is structural + word-root + concepts/entities + translation links (D-02); tafsir ingestion is a later corpus phase. | Phase 6–7 |
| **Hadith** collection links | Same boundary — no hadith corpus is ingested in Phase 4. | Phase 6–7 |
| **Isnad** chain links | Same boundary — chain grammar belongs to the hadith phase. | Phase 6–7 |

## 2. Interactive explorer, Web GUI, TUI → roadmap Phase 5

| Deferred item | Rationale | Target |
|---|---|---|
| **Interactive graph explorer** | Phase-4 visualization is static local rendering only (D-13): Graph JSON v1 file plus static DOT/SVG of a neighborhood/subgraph. | Phase 5 |
| **Web GUI** reading/research views | No GUI ships in Phase 4; the OpenAPI graph paths plus Envelope/ToolResult contracts are the machine-readable upstream the UI consumes. | Phase 5 |
| **TUI** cockpit | Same boundary — CLI + HTTP + tools only. | Phase 5 |

## 3. CozoDB / sqlite-graph-extension adapter spikes → optional go-no-go

| Deferred item | Rationale | Target |
|---|---|---|
| **CozoDB** adapter spike (legacy TASK-411) | Optional within the phase only as a go/no-go spike behind the same `GraphStore` port; never attempted — SQLite remains the single source of truth either way (ADR-0202). | optional spike |
| **sqlite-graph-extension** adapter spike (legacy TASK-412) | Same — never attempted; the snapshot-on-open adapter met every success criterion without it. | optional spike |

## 4. GraphML → deferred lossy view only

| Deferred item | Rationale | Target |
|---|---|---|
| **GraphML export** | Per ADR-0218 (Proposed, P4-X05 BLOCKED): Graph JSON v1 is the mandatory lossless envelope; GraphML is at most a lossy view generated from it, never the authoritative form. Not implemented. | lossy view only |

## 5. Production auth, adapters, hardening → roadmap Phases 11–12

| Deferred item | Rationale | Target |
|---|---|---|
| **Production auth / RBAC** on graph surfaces | Out of scope per CONTEXT (D-11 management scope is CLI-only); `LOCAL_PRINCIPAL` single-user interim stands. | Phase 11 |
| **Write-path adapters beyond CLI** (HTTP mutation routes, mutation agent tools) | D-11 scope fence: review and repair mutations are CLI-only by design; the fence is pinned by route/registry tests. | Phase 11–12 |
| **Operations hardening** (pooled serve handles, backup/GC runbooks) | `FileGraphBackend` opens per request (fine at research scale); `doctor-repair` + audit events are the runbook primitives Phase 12 consumes. | Phase 12 |

## 6. Additional editions / qira'at / multi-RAG / comparative scripture → their phases

| Deferred item | Rationale | Target |
|---|---|---|
| Additional Quran **qira'at / editions** | Phase 4 is single-fixture (`test-edition-min`); multi-edition expansion is a later roadmap phase. The generation-stamp/drift machinery already handles edition bumps (proven by the v2 drift test). | later phases |
| **Multi-RAG / comparative-scripture** surfaces | Untouched — graph records carry refs-only, which is the precondition those phases need. | their roadmap phases |
| **LLM-generated relationships** | The canonical path stays model-free by C0 Architecture invariant; suggestions are algorithmic (layer D) with confidence, never LLM output. | never via this path |

---

## D-01 evidence matrix (gap-closure record)

Each row: success criterion → implementation files → repeatable check →
gap-or-closed. All checks below are green at phase close (plan 04-05 full
gate); the only open items are the six BLOCKED owner gates, which no check
can close.

| Criterion | Implementation | Repeatable check | Gap or closed |
|---|---|---|---|
| **SC1** — neighbor view around any verse, word, root, or concept | `crates/application/src/quran_graph_store.rs` (snapshot adapter), `crates/application/src/quran_graph_api.rs` (neighbors service), `crates/cli/src/quran.rs` + `crates/application/src/quran_cli.rs` (Neighbors verb), `crates/server/src/api.rs` (neighbors route), `crates/application/src/quran_graph_tools.rs` (neighbors tool) | `cargo test -p application --test graph_neighbors` · `cargo test -p cli --test quran quran_graph_snapshots` · `cargo test -p server --test graph` · `cargo test -p application --test graph_explain` | **Closed.** Words/roots/concepts resolve through the lexicon-gated root-family + annotated projections; verses carry pinned canonical quotations. |
| **SC2** — paths between two nodes with per-edge provenance | `crates/application/src/quran_graph_api.rs` (reachability/shortest/up-to-K over `traverse` helpers), CLI `Path --mode`, HTTP path route, pattern/subgraph verbs | `cargo test -p application --test graph_paths` (11) · `graph_s2.trycmd` (8 segments) · `--test graph_explain` (parity) | **Closed.** Every edge reports structural-vs-assertion provenance with reviewer/decision/timestamp; no-path requires completeness (Option/unknown types). |
| **SC3** — export a subgraph with edge provenance intact | `crates/application/src/quran_graph_export.rs` (policy-filtered assembly), `crates/quran-graph/src/export.rs` (Graph JSON v1 + notice), DOT/SVG static rendering, CLI `Export --format` | `cargo test -p application --test graph_export` (5: travel-with-edges, byte-absence, truncation notice, document, render smoke) · export/DOT/SVG file assertions in `quran_graph_snapshots` | **Closed.** Assertions travel with edges; restricted/tombstoned material is byte-absent (proven, not commented). |
| **SC4** — queries explain why each result was returned | `Explanation` payload on every read result (snapshot identity, per-edge provenance, applied filters + authz descriptor, completion, duration ms); shared `explain_edge_with`/`describe_authz` helpers so CLI/HTTP/tools cannot drift | `cargo test -p application --test graph_explain` (4: three-surface parity + unanimous truncation) · `cargo test -p application --test graph_tools` (5: attributed envelopes) | **Closed.** D-14 confidence/algorithm fields included; file-backed reads assemble the same contract. |
| **Annotation lifecycle** (propose/suggest/accept/reject/dispute/correct + tombstones) | `crates/application/src/quran_graph_annotations.rs` (single-tx authority writes), migration `0022`, CLI `Review` group, word-root + annotated builders with AC-P4-03 re-derivation | `cargo test -p application --test graph_review` (8 + D-11 fence pin) · `cargo test -p application --test graph_build` (6 + OD BLOCKED pins) · review E2E + audit-list assertions in `quran_graph_snapshots` | **Closed** as engineering. Scholarly acceptance stays gated: OD-12 linguist BLOCKED; suggestions stay pending-labeled. |
| **Doctor + repair + GC** (D-08 operability tail) | `crates/application/src/quran_graph_doctor.rs` (six read-only `quran.graph.*` checks; confirmed quarantine/GC/rebuild with audit), CLI `Doctor` + `DoctorRepair` verbs | `cargo test -p application --test graph_doctor` (10: manifest/drift/dangling/tombstone/empty/immutability + 4 repair) · `qai quran graph doctor [--deep]` verb smoke | **Closed.** Doctor never mutates (byte-identical proof); repair is explicit, retention-gated, audited. |
| **Parity + conformance** (cross-surface trust) | Backend-generic `quran-graph` conformance (Mem × SQLite), three-surface parity suite, OpenAPI graph paths, 12-tool registry | `cargo test -p quran-graph` (49) · `cargo test -p server` (graph 10) · `cargo test -p tool-registry` · `cargo xtask arch-check` · `cargo xtask migrate-check` · `cargo fmt --check` · `cargo clippy -- -D warnings` | **Closed.** Zero new dependencies (T-04-SC); arch-check green with no allowlist changes in Phase 4 wave 5. |
| **Word-root on real data** | Gated by `require_active_dataset` (`QAI-MORPH-0004`) | `graph_build` unavailable-dataset pin | **Gap — OD-11 BLOCKED.** Synthetic fixture only until the owner closes OD-11 (see owner-gates ledger). |
| **ADR ratification** (0202/0702/0217/0218) | Implemented toward the Proposed drafts | `tasks.md` swimlane X rows | **Gap — P4-X01/X02/X03/X05 BLOCKED.** Drafts ready; ratification is owner work. |

---

*Phase: 4-Quran Graph · Deferral ledger + D-01 evidence matrix created
2026-09-29 by plan 04-05. Append-only; nothing here is silently dropped.*
