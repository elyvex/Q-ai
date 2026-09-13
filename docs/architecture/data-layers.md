# Data Layers: Five Trust Layers A–E (PRD §6)

---
## 1. **Layer A** — Canonical Source**: data (Quran, hadith, tafsir, scripture, Quranic, Quran-normalization, Quran-morphology).
2. **Layer B** — Publisher Metadata (numbering, pagination, grading, attribution).
3. **Layer C** — Scholarly Annotation (tafsir claim, root analysis, evaluation).
4. **Layer D** — Computational Annotation (LLM model output, embeddings).
5. **Layer E** — User/AI Notes (bookmarks, reminders, reasoning traces).

---
## Usage in codebase:

- All data passes through `sources::sources` API, validated by manifest + audit.
- Graph stores (`provenance_records`) holds core lineage of all Layer types.
- Each layer corresponds to a domain crate (`domain`) 
  populated only for the layer it holds (§5.1).
- Domain crates maintain both aggregates (per layer) and Value Objects (per field/column).
---
## Source lifecycle: the state machine diagram from `plan.md` §22.1–22.6 in text and tables.
---
## Error codes registry: all QAI-****n codes defined in `docs/architecture/error-codes.md` (JSON: QAI-DOM-0001..0005,
QAI-DB-0001..0009, QAI-JOB-0001..0014, QAI-SEC-0001..0004,
QAI-CLI-0001..0014, QAI-QUR-0001..0005, QAI-NORM-0001..0005,
QAI-IDX-0001..0005, QAI-SEC-0001..0004).
---
## Hashing spec: sha256 algorithm tag format `sha256:<hex>`,
`canonical_json_bytes` definition (stable RFC8785, UTF-8, LF newline, NFC normalization).
