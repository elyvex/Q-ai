# Phase 7 (Isnad & Narrator Research) — Owner Gates

> **Purpose.** This record closes no decision. It records, for the human-only
> gates that block licensed narrator/rijal data ingestion and scholarly
> ratification of uncertainty policy, exactly what is open, what each gate
> blocks, and the precise action plus command that would close it. It exists so
> Phase 7 can be declared *engineered* while remaining unambiguous that the phase
> is **not** a licensed-dataset or scholar-ratified release.
>
> **Agent-uncloseable.** No agent, model, or importer may close any gate below.
> No dataset slug, publisher, license, scholar name, grading value, or sign-off
> may be invented or substituted. Every such value stays `_unassigned_` /
> `pending_owner_verification` until a human owner writes it in.
>
> **Status source of truth.** Canonical rows live in
> `docs/05-followups/decisions-needed.md`; this file is a supporting record and
> never flips a row to resolved.
>
> **Status key:** 🔴 BLOCKED · 🟢 answered (with date + where recorded).

## Gates

| ID | Status | Blocks | Owner | Evidence path | Closing action (exact) |
|----|--------|--------|-------|---------------|------------------------|
| OD-07-01 | 🔴 BLOCKED | Licensed narrator/rijal database ingestion (real data) | Data steward | `docs/05-followups/decisions-needed.md` | Owner licenses a narrator/rijal dataset and runs the approved ingestion pipeline; then `/gsd-verify-work 7`. |
| OD-07-02 | 🔴 BLOCKED | Comparative isnad across collections | Owner | `.planning/phases/07-isnad-narrator-research/07-CONTEXT.md` (<deferred>) | Deferred to Phase 9; close when Phase 9 comparative-scope gate lands. |
| OD-07-03 | 🔴 BLOCKED | Ratification that narrator identity resolution never presents uncertainty as certainty | Owner | `crates/application/src/isnad_tools.rs` | Owner reviews and ratifies the uncertain-until-owner-confirmed policy; record in `docs/05-followups/owner-decisions.md`. |
| OD-07-04 | 🔴 BLOCKED | Rijal grading vocabulary (free-text `grading_value`) ratification | Named rijal scholar | `migrations/sqlite/0032_narrator_rijal_sources.up.sql` | Named scholar ratifies the vocabulary/mapping; owner records sign-off. |

**None of the above is answered.** All remain explicitly 🔴 BLOCKED at time of
writing (2026-10-17).
