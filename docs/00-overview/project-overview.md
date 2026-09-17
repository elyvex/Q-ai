# Q-ai — Project Overview

Q-ai is a local-first research platform for the Quran, hadith collections, Islamic literature,
and comparative scripture, built in Rust. Its load-bearing idea: the Quran gets a dedicated
lossless, structured, hash-verified corpus engine — not a bag of RAG chunks — and everything
downstream (search, graphs, tafsir links, agent reports, citations) inherits that correctness.

- **Status (2026-09-17):** developer-stage canonical Quran engine (CLI + loopback API +
  emerging normalization/search). Web GUI, TUI, graphs, hadith, multi-RAG, and agent runtime
  are planned, not built. See `README.md` (root) for verified current behavior.
- **Start here:** the full agent briefing — PRD map, architecture, codebase, pipelines,
  invariants, gates, current phase state, known traps, orchestration loop — lives in
  [`agent-briefing.md`](agent-briefing.md). Required reading before any task.
- **Companions:** `goals.md` · `non-goals.md` · `scope.md` · `glossary.md` (this folder) ·
  PRD `../01-requirements/requirements.md` · `.agent/project-context.md`.
