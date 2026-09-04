# Project Context

## What This Is

Q-ai is a Rust-based research platform for Islamic studies and comparative scripture:

- Canonical Quran engine with dedicated, lossless, structured, graph-enabled corpus
- Quran-specific linguistic indexes (roots, lemmas, stems, morphology)
- Knowledge graphs for Quran and hadith (isnad / narrator chains)
- Multi-book RAG for hadith, tafsir, translations, and comparative scripture
- Hybrid and graph retrieval with smart source selection
- Controlled agentic tool calling with verifiable citations
- Source and edition management with provenance tracking

## Supported Knowledge Domains

1. Quran (canonical text, morphology, translations, tafsir, graph)
2. Hadith (Shia collections, isnads, narrators, gradings, graph)
3. Islamic Literature (theology, fiqh, history, rijal, sira, dictionaries)
4. Comparative Scripture (Torah, Tanakh, New Testament, other corpora)
5. User Collections (articles, notes, PDFs)

## Interfaces

Same core engine powers:

- Web GUI
- TUI
- CLI (`qai`)
- REST API
- Streaming API
- Agent tools / MCP integrations

## Development Phases

The project is planned in phases under `docs/03-plan/phases/`. Current phase is tracked in
`docs/03-plan/current-plan.md`.
