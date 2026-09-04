# PRD — Q-ai: Quran, Hadith, and Comparative Scripture Research Platform

**Status:** Draft / Living Document  
**Version:** 0.3.2  
**Previous Version:** 0.3.1  
**Primary Language:** Rust  
**Primary Interfaces:** Web GUI + TUI + CLI + API  
**Deployment:** Local-first + Server  
**Architecture:** Canonical Quran Engine + Knowledge Graph + Multi-RAG + Agentic Tool Runtime  
**Product Name:** **Q-ai**

---

# 1. Product Vision

Q-ai is a research platform for studying:

- The Quran
- Quranic vocabulary, morphology, roots, and linguistic relationships
- Quran translations and tafsir
- Shia hadith collections
- Hadith chains of transmission
- Hadith grading and scholarly commentary
- Islamic history and theology
- The Torah, Hebrew Bible, New Testament, and other relevant scriptures
- Relationships, parallels, differences, and citations across these sources

Q-ai must not treat the Quran as an ordinary collection of document chunks.

The Quran must have a dedicated, lossless, structured, graph-enabled corpus engine that is richer and more reliable than standard RAG.

Hadith books, tafsir collections, translations, historical books, and comparative scriptures may use RAG, full-text search, structured indexes, and knowledge graphs according to their source structure.

The system must combine:

1. A canonical Quran corpus
2. Quran-specific linguistic indexes
3. A Quran knowledge graph
4. Hadith and isnad graphs
5. Multi-book RAG
6. Hybrid and graph retrieval
7. Smart source selection
8. Controlled agentic tool calling
9. Verifiable citations
10. Source and edition management
11. Research workspaces and reproducible research reports

The same core engine must power:

- Web GUI
- TUI
- CLI
- REST API
- Streaming API
- Agent tools
- MCP integrations

---



# 2. Product Principles

Q-ai must follow these principles:

1. **Exact text before generated interpretation**
2. **Every factual claim must be traceable**
3. **The Quran is stored structurally, not only as RAG chunks**
4. **Canonical text must be separated from translations and annotations**
5. **Translations must not be presented as the original Quran**
6. **Disputed claims must be labeled as disputed**
7. **Hadith grading must be attributed to a source or scholar**
8. **Different Islamic schools must be represented without silently merging their views**
9. **Search normalization must never modify displayed canonical text**
10. **Imported internet content is untrusted until validated**
11. **No model may fabricate a verse, hadith, chain, grading, or citation**
12. **Answers must distinguish quotation, source summary, and AI analysis**
13. **Users must be able to inspect how every answer was produced**
14. **Research results should be reproducible**
15. **Local data remains local unless the user explicitly enables a remote provider**
16. **Agents and tools use deny-by-default permissions**
17. **Religious conclusions must not be falsely presented as scholarly consensus**
18. **Q-ai assists research; it does not automatically claim religious authority or issue binding rulings**
19. **Contradictory scholarly interpretations must be presented side-by-side with attribution rather than merged into a single system-endorsed conclusion**

---



# 3. Goals

## 3.1 Primary Goals

- Provide a complete and carefully validated Quran research system.
- Support exact Arabic Quran search.
- Support Arabic search with and without diacritics.
- Support root, lemma, stem, token, phrase, and word-family exploration.
- Support Quran graph browsing and graph search.
- Support high-quality visual presentation of the Quran.
- Import and index major Shia hadith collections.
- Support works such as:
  - Al-Kafi
  - Bihar al-Anwar / Biḥār al-Anwār
  - Man La Yahduruhu al-Faqih
  - Tahdhib al-Ahkam
  - Al-Istibsar
  - Other configurable collections
- Support tafsir and Quranic commentary.
- Support Torah, Bible, and other comparative scripture collections.
- Provide smart retrieval routing among structured Quran tools, graph search, lexical search, hadith indexes, and RAG.
- Provide verse-level, hadith-level, page-level, and passage-level references.
- Allow administrators to add and update book catalogs from the internet safely.
- Preserve source versions, licensing information, hashes, and provenance.
- Support local and server deployment.
- Provide APIs suitable for research applications and external agents.

## 3.2 Secondary Goals

- Multilingual search and display.
- Arabic, Persian, and English interfaces.
- Extensible source adapters.
- Exportable research reports.
- Personal annotations and collections.
- Saved queries.
- Scholar-oriented comparison workflows.
- Reproducible graph and corpus queries.
- Offline Quran browsing and searching.
- Optional local LLM and embedding models.
- Future plugin and research-tool ecosystem.

---



# 4. Non-Goals

The initial releases will not:

- Build a foundation model from scratch.
- Declare one sectarian interpretation as universally authoritative.
- Automatically issue fatwas.
- Replace qualified scholars.
- Treat all hadith gradings as universal or uncontested.
- Silently reconcile contradictory narrations.
- Infer missing isnad members as established facts.
- Alter canonical Quran text based on model output.
- Automatically trust arbitrary internet editions.
- Automatically publish AI-generated linguistic annotations as verified scholarship.
- Provide every historical qira'ah in the first release.
- Implement unrestricted autonomous web research.
- Execute generated native code without review and isolation.
- Resolve all copyright and licensing issues through technical mechanisms alone.

---



# 5. Supported Knowledge Domains

Q-ai must model knowledge domains independently so users can include or exclude them.

```text
Q-ai Knowledge Domains
├── Quran
│   ├── Canonical Arabic text
│   ├── Orthography
│   ├── Tokens
│   ├── Lemmas
│   ├── Roots
│   ├── Morphology
│   ├── Translations
│   ├── Tafsir
│   ├── Recitation metadata
│   └── Quran graph
├── Hadith
│   ├── Shia collections
│   ├── Narration text
│   ├── Isnads
│   ├── Narrators
│   ├── Gradings
│   ├── Commentary
│   └── Hadith graph
├── Islamic Literature
│   ├── Theology
│   ├── Fiqh
│   ├── History
│   ├── Rijal
│   ├── Sira
│   └── Dictionaries
├── Comparative Scripture
│   ├── Torah
│   ├── Tanakh / Hebrew Bible
│   ├── New Testament
│   ├── Bible translations
│   ├── Apocrypha where licensed
│   └── Other configured corpora
└── User Collections
    ├── Articles
    ├── Notes
    ├── PDFs
    ├── Websites
    └── Private research material
```

Every result must identify its domain.

---



# 6. Authoritative Data Separation

Q-ai must separate data into explicit trust layers.

## 6.1 Layer A — Canonical Source Text

Examples:

- Quran Arabic text
- A published hadith edition’s original text
- A Bible edition’s original text
- A book’s source transcription

This layer must be immutable for a specific source version.

Corrections create a new version rather than silently modifying historical data.

## 6.2 Layer B — Publisher or Dataset Metadata

Examples:

- Verse numbering
- Page numbering
- Juz, hizb, rubʿ, and manzil
- Hadith numbering
- Volume and page references
- Chapter structure
- Supplied morphology
- Supplied hadith grading

## 6.3 Layer C — Scholarly Annotation

Examples:

- Tafsir claims
- Root analyses
- Narrator evaluations
- Jurisprudential classifications
- Cross-references
- Historical dates

Annotations must identify their author, school, source, and edition where available.

## 6.4 Layer D — Computational Annotation

Examples:

- Automatically inferred lemmas
- Named entities
- Topic classifications
- Embeddings
- Similarity relationships
- Machine-generated cross-references

Computational annotations must include:

- Algorithm or model
- Version
- Confidence
- Generation time
- Input source version
- Verification status

## 6.5 Layer E — User and AI Notes

AI-generated material must never be visually or structurally confused with canonical source text or verified scholarly annotation.

---



# 7. Canonical Quran Corpus Requirements

The Quran engine must preserve a validated canonical representation.

## 7.1 Required Quran Hierarchy

```text
Quran Edition
└── Surah
    └── Ayah
        └── Segment
            └── Token
                ├── Surface form
                ├── Normalized forms
                ├── Lemma candidates
                ├── Root candidates
                ├── Stem
                ├── Prefixes
                ├── Suffixes
                └── Morphological features
```

## 7.2 Quran Edition Model

Each edition must record:

```text
id
name
script
riwayah
qiraah
publisher
source_url
license
language
verse_numbering_scheme
text_hash
manifest_hash
version
imported_at
verified_at
verification_method
status
```

The initial release should support one thoroughly validated Arabic Quran edition, preferably an Uthmani-script Hafs edition, while keeping the schema ready for additional editions and qira'at.

Different readings must not be merged into one synthetic text.

## 7.3 Immutable Canonical Text

The following must never be generated or corrected by an LLM:

- Arabic verse text
- Surah order
- Verse identifiers
- Canonical token order
- Edition identity
- Recitation identity

All canonical text changes require:

- A new source version
- Checksum validation
- Structural validation
- Difference report
- Human approval
- Audit event

## 7.4 Quran Addressing

Every Quran element must have stable addressing.

Examples:

```text
quran:1:1
quran:2:255
quran:2:255:token:3
quran:hafs-uthmani:36:1-12
```

The system must support:

- Surah and ayah
- Ayah range
- Juz
- Hizb
- Rubʿ
- Page
- Sajdah markers
- Ruku where present in an edition
- Token position

---



# 8. Quran Text Normalization

Normalization is required for searching but must never replace displayed canonical text.

## 8.1 Search Forms

Each token or phrase should support separately indexed forms:

- Original Uthmani form
- Diacritic-free Arabic
- Tatweel-free Arabic
- Quranic annotation-mark-free form
- Standardized hamza form
- Optional alif normalization
- Optional alif maqsura/ya normalization
- Optional ta marbuta/ha normalization
- Optional Persian/Arabic character normalization
- Token-concatenated form
- Token-separated form
- Lemma form
- Root form
- Transliteration
- Phonetic approximation where explicitly enabled

## 8.2 User-Controlled Normalization

Users must be able to choose strictness:

```text
Exact canonical
Exact after whitespace normalization
Ignore Quranic marks
Ignore diacritics
Normalize hamza/alif
Normalize Arabic/Persian code points
Morphological search
Fuzzy spelling search
```

The results interface must show which normalization rules were used.

## 8.3 Concatenated-Word Search

Q-ai must support searches in which users:

- Enter text without diacritics
- Omit spaces
- Use joined forms
- Use separated forms
- Search across token boundaries
- Search common prefix/suffix attachment variants

Example use cases:

- Searching a phrase after removing all harakat
- Searching `بسمالله` and finding `بِسْمِ ٱللَّهِ`
- Searching words regardless of attached conjunctions or prepositions
- Searching a surface word with pronoun suffixes removed

Matches must include an explanation of the normalization and segmentation used.

---



# 9. Quran Linguistic Model

## 9.1 Token Data

A Quran token may contain:

```text
token_id
edition_id
surah
ayah
position
surface_uthmani
surface_simple
normalized
prefixes
stem
suffixes
lemma_candidates
root_candidates
part_of_speech
person
gender
number
case
mood
voice
aspect
state
dependency
morphology_source
confidence
verification_status
```

## 9.2 Multiple Analyses

Morphological interpretation may differ between datasets or scholars.

The system must:

- Store multiple analyses
- Attribute each analysis
- Avoid silently selecting one as unquestionably correct
- Allow users to compare analyses
- Permit a configured preferred dataset
- Display confidence and verification status for generated analyses

## 9.3 Arabic Word Families

The product must provide a **Kalimāt-e Ham-Khānevādeh / Word Family** feature.

It should discover and display:

- Same-root words
- Same-lemma forms
- Derived forms
- Inflectional variants
- Prefix and suffix variants
- Singular/plural relations
- Masculine/feminine relations where linguistically applicable
- Verb-form relationships
- Active and passive participles
- Verbal nouns
- Related names and adjectives

Users must be able to distinguish:

- Same exact word
- Same lemma
- Same stem
- Same root
- Computationally suggested relation
- Scholar-verified relation

---



# 10. Quran Knowledge Graph

The Quran must have a first-class graph representation independent of vector search.

## 10.1 Core Node Types

```text
QuranEdition
Surah
Ayah
AyahRange
Token
SurfaceForm
NormalizedForm
Lemma
Root
Stem
Morpheme
MorphologicalAnalysis
Phrase
NamedEntity
Person
Place
Community
Event
Object
Concept
Theme
RhetoricalFeature
GrammaticalFeature
TranslationPassage
TafsirPassage
CrossReference
RecitationVariant
ResearchClaim
Source
Edition
Scholar
Annotation
```

## 10.2 Core Edge Types

```text
CONTAINS
PRECEDES
FOLLOWS
HAS_TOKEN
HAS_LEMMA
HAS_ROOT
HAS_STEM
HAS_PREFIX
HAS_SUFFIX
HAS_ANALYSIS
DERIVED_FROM
SAME_ROOT_AS
SAME_LEMMA_AS
REFERS_TO
MENTIONS
ADDRESSES
LOCATED_IN
RELATED_TO
SIMILAR_TO
CONTRASTS_WITH
PARALLELS
EXPLAINS
INTERPRETS
TRANSLATES
CITES
QUOTED_BY
ALLUDES_TO
NARRATES
OCCURS_IN
HAS_VARIANT
SUPPORTED_BY
DISPUTED_BY
ASSERTED_BY
GENERATED_BY
```

## 10.3 Edge Provenance

Every non-structural edge must include:

```text
source_id
source_location
author_or_algorithm
version
confidence
verification_status
created_at
```

The system must distinguish:

- Deterministic structural edges
- Dataset-supplied edges
- Scholar-authored edges
- User-created edges
- Algorithmically inferred edges

## 10.4 Graph Search

Users must be able to search:

- All verses containing tokens from a root
- Concepts within N graph hops of an ayah
- Verses sharing entities and roots
- Tafsir passages interpreting the same verse
- Hadith that cite or reference an ayah
- Narrations connected to a Quranic topic
- Parallel passages in other scriptures
- Chains such as:
  `Root → Lemma → Token → Ayah → Tafsir → Scholar`
- Paths between two verses, themes, entities, or narrations
- Subgraphs filtered by source, school, date, language, or confidence

## 10.5 Graph Query Explainability

Every graph result must expose:

- Start node
- End node
- Traversed path
- Edge types
- Edge provenance
- Confidence
- Applied filters
- Query duration

## 10.6 Semi-Automated Graph Annotation

Q-ai must provide a review-oriented annotation workflow to bootstrap the graph with human-verified edges.

Requirements:

- Researchers can manually create typed edges such as `CITES`, `CONTRASTS_WITH`, `PARALLELS`, and `EXPLAINS` between existing nodes.
- Manually created edges are attributed to the creating user and classified as `Scholar-Authored` or `User-Created` per the edge provenance rules of section 10.3.
- Algorithmically suggested edges can be queued for human review, then accepted, rejected, or corrected.
- Accepted suggestions record the reviewer, decision, and timestamp.
- Review tools must show the evidence supporting a suggested edge before acceptance.
- No computational suggestion becomes a verified edge without an explicit human decision.

---



# 11. Quran Research Tools

The following must be first-class typed tools usable by the UI, API, workflows, and authorized agents.

## 11.1 Exact and Normalized Search

### `quran.search_exact`

Search the canonical surface text without linguistic expansion.

### `quran.search_normalized`

Search while applying explicit normalization options.

### `quran.search_concatenated`

Find phrases entered without whitespace or Quranic marks.

### `quran.search_phrase`

Search exact, ordered, near, or unordered phrases.

### `quran.search_regex`

Advanced controlled regular-expression search over selected normalized fields.

Regular expressions must be resource-limited.

## 11.2 Root and Morphology Tools

### `quran.root_search`

Find all tokens associated with a root.

### `quran.lemma_search`

Find all inflections of a lemma.

### `quran.word_family`

Build an explainable family of same-root or derived words.

### `quran.morphology`

Return token segmentation and analyses.

### `quran.morphology_compare`

Compare multiple morphological datasets for a token or phrase.

### `quran.pattern_search`

Search Arabic morphological patterns and verb forms.

### `quran.affix_search`

Search by prefix, suffix, attached pronoun, conjunction, article, or preposition.

## 11.3 Frequency and Distribution Tools

### `quran.frequency`

Count forms, lemmas, roots, or phrases.

### `quran.distribution`

Show distribution by surah, Makki/Madani metadata, juz, page, or passage.

### `quran.cooccurrence`

Find words, roots, entities, or themes occurring within configurable windows.

### `quran.collocation`

Rank statistically significant neighboring terms.

### `quran.first_last_occurrence`

Locate first, last, and ordered occurrences.

### `quran.interval_analysis`

Analyze verse or token distances between occurrences.

## 11.4 Context and Comparison Tools

### `quran.get_ayah`

Return canonical text and configured metadata.

### `quran.get_context`

Return surrounding verses without arbitrary chunking.

### `quran.compare_translations`

Align translations at verse or phrase level.

### `quran.compare_tafsir`

Compare tafsir passages with explicit source labels.

### `quran.parallel_verses`

Find lexical, semantic, thematic, or structural parallels.

### `quran.contrastive_search`

Find passages that use contrasting vocabulary or themes.

### `quran.cross_reference`

Retrieve documented references among verses, hadith, tafsir, and other scriptures.

## 11.5 Graph Tools

### `quran.graph_neighbors`

Return typed neighboring nodes.

### `quran.graph_path`

Find explainable paths between nodes.

### `quran.graph_subgraph`

Build a filtered research subgraph.

### `quran.graph_pattern`

Execute safe, bounded graph-pattern queries.

### `quran.entity_timeline`

Show appearances of a person, community, place, or event in canonical order.

## 11.6 Rhetorical and Structural Research Tools

These tools may use verified data or produce explicitly labeled computational suggestions.

### `quran.repetition_analysis`

Detect repeated words, phrases, and structural motifs.

### `quran.parallel_structure`

Suggest parallel or mirrored passage structures.

### `quran.rhyme_analysis`

Analyze verse-ending phonetic or orthographic patterns.

### `quran.address_shift`

Identify suggested changes in speaker, addressee, person, tense, or number.

### `quran.discourse_links`

Find conjunctions, referents, topic transitions, and passage links.

### `quran.pronoun_reference`

Return documented or computationally suggested pronoun antecedents.

### `quran.semantic_field`

Explore words associated with a configured semantic field.

### `quran.numeric_report`

Produce reproducible counts with exact counting rules.

Numeric reports must never present speculative numerology as established fact.

## 11.7 Discovery Tools

### `quran.unusual_usage`

Find rare lemmas, roots, forms, or constructions.

### `quran.hapax_search`

Find forms or lemmas occurring once under stated normalization rules.

### `quran.near_duplicate_passages`

Find passages with high lexical overlap.

### `quran.missing_expected_form`

Given a word family or pattern, show attested and unattested forms without implying theological conclusions.

### `quran.query_builder`

Convert a natural-language research request into an inspectable structured query.

### `quran.research_notebook`

Save queries, evidence, notes, graph views, and source versions as a reproducible research package.

---



# 12. Quran Tool Result Contract

Every Quran tool result must include:

```text
tool_name
tool_version
query
normalization_rules
edition_id
edition_version
results
canonical_references
analysis_sources
confidence
warnings
execution_time
reproducibility_data
```

A Quran quotation must include:

- Surah number and name
- Ayah number or range
- Arabic canonical text
- Edition identifier
- Translation identifier when displayed
- Stable deep link

The system must never construct Quran quotations from model memory when canonical retrieval is available.

## 12.1 Research Checksum

Every research query, tool plan, and research report must produce a reproducibility checksum computed over:

```text
query text and parameters
tool plan and tool versions
selected source IDs and source versions
edition IDs and edition versions
normalization rules
retrieval and reranking configuration
model provider, model name, and prompt version
```

The checksum allows a user to re-run identical research later and detect whether results changed because of corpus updates, index changes, or model updates.

Re-running with the same checksum must produce identical output for deterministic tools. Nondeterministic or model-generated parts must be labeled separately.

---



# 13. Quran Display Requirements

The Web GUI must present the Quran as a high-quality reading and research experience.

## 13.1 Reading View

Required features:

- High-quality Arabic font
- Right-to-left layout
- Correct Quranic mark rendering
- Surah headers
- Basmala handling according to edition metadata
- Ayah markers
- Page, juz, hizb, and rubʿ navigation
- Responsive desktop and mobile layout
- Light, dark, sepia, and high-contrast themes
- Adjustable Arabic font size and line spacing
- Optional word-by-word translation
- Optional full-verse translation
- Multiple translation panels
- Tafsir side panel
- Audio synchronization where licensed
- Bookmarking
- Notes
- Copy with citation
- Shareable deep links

## 13.2 Research View

Selecting a word should optionally show:

```text
Canonical token
Simple Arabic form
Transliteration
Translation gloss
Lemma
Root
Morphological segmentation
Grammar
Other occurrences
Word family
Co-occurring terms
Graph neighbors
Tafsir references
Hadith references
```

## 13.3 Citation Interaction

Clicking a citation must open the exact source location, not merely the document homepage.

Examples:

- Quran verse
- Hadith number and chapter
- Volume and page
- Tafsir passage
- Bible chapter and verse
- Source scan page where available

## 13.4 Visual Distinction

The interface must visually distinguish:

- Quran Arabic
- Translation
- Tafsir
- Hadith
- Commentary
- AI summary
- Computational suggestion
- User note

---



# 14. Hadith Corpus Requirements

Hadith must be represented using source-aware structured records, not only arbitrary RAG chunks.

## 14.1 Hadith Structure

```text
HadithCollection
└── Edition
    └── Volume
        └── Book
            └── Chapter
                └── Hadith
                    ├── Isnad
                    ├── Matn
                    ├── Translation
                    ├── Grading
                    ├── Commentary
                    └── References
```

## 14.2 Hadith Metadata

```text
hadith_id
collection_id
edition_id
volume
book
chapter
hadith_number
alternate_numbers
arabic_text
normalized_text
isnad_text
matn_text
language
translator
source_page
source_url
content_hash
license
version
```

## 14.3 Shia Collections

The architecture must support, subject to legal and reliable source availability:

- Al-Kafi
- Man La Yahduruhu al-Faqih
- Tahdhib al-Ahkam
- Al-Istibsar
- Bihar al-Anwar
- Wasā'il al-Shīʿa
- Nahj al-Balagha
- Other configured hadith and commentary collections

Bihar al-Anwar must retain volume, chapter, page, edition, and cited earlier-source metadata where available.

Al-Kafi must preserve its major divisions and chapter hierarchy.

## 14.4 Grading Model

A hadith must not have one universal `authentic` Boolean.

Gradings must record:

```text
grading_id
hadith_id
grader
school_or_methodology
grading_term
grading_explanation
source
edition
page
date
language
verification_status
```

The UI must display:

- Who graded it
- The exact grading term
- The source of the grading
- Other known gradings
- Whether views differ

## 14.5 Hadith Tools

Required tools include:

- `hadith.get`
- `hadith.search_exact`
- `hadith.search_normalized`
- `hadith.search_semantic`
- `hadith.search_by_narrator`
- `hadith.search_by_collection`
- `hadith.search_by_chapter`
- `hadith.compare_variants`
- `hadith.find_parallels`
- `hadith.get_gradings`
- `hadith.get_commentary`
- `hadith.isnad_parse`
- `hadith.isnad_graph`
- `hadith.narrator_profile`
- `hadith.narrator_paths`
- `hadith.quran_references`
- `hadith.source_trace`

---



# 15. Isnad and Narrator Graph

## 15.1 Node Types

```text
Hadith
Isnad
Narrator
NarratorIdentity
NameVariant
TeacherStudentRelationship
Collection
Scholar
Grading
RijalEntry
Place
HistoricalPeriod
```

## 15.2 Edge Types

```text
NARRATED_FROM
NARRATED_TO
APPEARS_IN_ISNAD
POSSIBLE_IDENTITY
NAME_VARIANT_OF
TEACHER_OF
STUDENT_OF
CONTEMPORARY_OF
EVALUATED_BY
GRADED_BY
RECORDED_IN
VARIANT_OF
PARALLEL_TO
```

## 15.3 Identity Uncertainty

Narrator identity resolution may be uncertain.

The graph must support:

- Confirmed identity
- Probable identity
- Possible identity
- Conflicting identity
- Unresolved identity

Automated identity matching must never silently merge narrator records.

---



# 16. Tafsir and Commentary

Tafsir must be indexed by:

- Work
- Author
- School or tradition where applicable
- Language
- Edition
- Volume and page
- Surah and ayah range
- Topic
- Quoted sources
- Referenced hadith
- Referenced Quran verses

The platform must support:

- Verse-to-tafsir lookup
- Multiple-tafsir comparison
- Chronological comparison
- School-aware filtering
- Arabic/Persian/English retrieval
- Exact quotation display
- AI-generated summaries clearly separated from original text

Tafsir passages must not be presented as Quran text.

---



# 17. Comparative Scripture

Q-ai may contain:

- Torah
- Tanakh / Hebrew Bible
- Old Testament editions
- New Testament editions
- Bible translations
- Other relevant licensed scriptures

## 17.1 Requirements

- Preserve edition and translation identity.
- Preserve chapter and verse numbering schemes.
- Support alternate numbering where required.
- Preserve original-language text when licensed and available.
- Label translations clearly.
- Never silently merge different textual traditions.
- Attribute cross-scripture relationships.
- Distinguish direct quotation, thematic parallel, lexical similarity, and AI-suggested similarity.

## 17.2 Comparative Tools

- `scripture.get_passage`
- `scripture.search_exact`
- `scripture.search_semantic`
- `scripture.compare_translations`
- `scripture.find_parallels`
- `scripture.cross_reference`
- `scripture.graph_path`
- `scripture.compare_entities`
- `scripture.compare_narratives`

Computationally discovered parallels must be labeled as suggestions unless supported by cited scholarship.

---



# 18. Multi-RAG Architecture

Q-ai must support multiple independently configured RAG systems.

Example:

```text
RAG Manager
├── Quran Tafsir RAG
├── Shia Hadith RAG
├── Rijal RAG
├── Islamic History RAG
├── Torah and Bible RAG
├── Academic Papers RAG
└── Private Research RAG
```

Each RAG may configure:

- Included sources
- Excluded sources
- Language
- School or tradition filters
- Chunking strategy
- Embedding model
- Vector store
- Full-text index
- Graph source
- Retriever
- Reranker
- Citation policy
- Trust policy
- LLM
- Prompt templates
- Access controls

---



# 19. Structure-Aware Ingestion and Chunking

Generic token chunking is insufficient for sacred and scholarly texts.

## 19.1 Quran

Quran indexing units include:

- Token
- Phrase
- Ayah
- Ayah window
- Passage
- Surah

Canonical ayah boundaries must never be lost.

## 19.2 Hadith

Hadith indexing units include:

- Full hadith
- Isnad
- Matn
- Chapter heading
- Commentary
- Edition notes

Isnad and matn boundaries must be preserved.

## 19.3 Tafsir

Tafsir chunks should align with:

- Ayah references
- Paragraphs
- Headings
- Quoted passages
- Commentary units

## 19.4 Books

General book chunking should preserve:

- Volume
- Page
- Book
- Chapter
- Section
- Paragraph
- Footnotes
- Bibliography references

Every chunk must retain a reversible link to its source location.

---



# 20. Smart Retrieval and Tool Selection

The system must contain a deterministic and model-assisted retrieval router.

## 20.1 Retrieval Methods

- Canonical Quran lookup
- Exact Arabic search
- Normalized Arabic search
- Root and lemma search
- Morphological search
- Full-text search
- BM25 or equivalent lexical search
- Vector similarity
- Hybrid retrieval
- Metadata filtering
- Graph traversal
- Knowledge Graph RAG
- Parent-child retrieval
- Multi-query retrieval
- Query decomposition
- Reranking
- Cross-encoder reranking where configured
- Citation validation

## 20.2 Smart Router

The router should classify a query before retrieval.

Example:

```text
User Query
   ↓
Intent and Constraint Analysis
   ├── Exact Quran quotation?
   ├── Root/lemma question?
   ├── Quran concept research?
   ├── Hadith lookup?
   ├── Hadith grading?
   ├── Isnad question?
   ├── Tafsir comparison?
   ├── Comparative scripture?
   └── General book research?
   ↓
Tool and Source Plan
   ↓
Execute Specialized Tools
   ↓
Merge and Rerank Evidence
   ↓
Validate Citations
   ↓
Generate Answer
```

## 20.3 Routing Rules

Examples:

- “Show Quran 2:255” → `quran.get_ayah`
- “Words from root ر ح م” → `quran.root_search`
- “Find this phrase without harakat” → `quran.search_normalized`
- “Find hadith narrated by X” → `hadith.search_by_narrator`
- “What is the grading?” → `hadith.get_gradings`
- “Compare al-Kafi and Bihar narrations” → structured hadith search plus parallel detection
- “What do tafsirs say?” → tafsir retrieval, not Quran-only search
- “Compare this account with Genesis” → Quran retrieval plus comparative scripture tools
- Broad conceptual questions → hybrid RAG and graph search

## 20.4 Tool Plan Visibility

The user must be able to inspect:

- Classified intent
- Selected tools
- Selected sources
- Excluded sources
- Filters
- Retrieval scores
- Graph paths
- Reranking
- Final context
- Citation validation

---



# 21. Research Answer Contract

Answers should use a structured evidence model.

## 21.1 Answer Sections

Where appropriate:

1. Direct answer
2. Quran evidence
3. Hadith evidence
4. Tafsir or scholarly views
5. Comparative material
6. Areas of disagreement
7. AI analysis
8. References
9. Search methodology

## 21.2 Claim-Level Citations

Citations should be attached to individual claims rather than placed only at the end.

Each citation must link to:

- Source ID
- Edition
- Exact location
- Quoted or retrieved passage
- Content hash
- Ingestion version

## 21.3 Citation Validation

Before presenting an answer, Q-ai should verify:

- The source exists.
- The location exists.
- The quotation matches the source.
- The user can access the source.
- The citation supports the associated claim.
- The source edition is identified.

Unsupported claims must be removed, qualified, or marked as analysis.

## 21.4 Disagreement Representation

When sources disagree, Q-ai must:

- Avoid hiding the disagreement.
- Attribute each position.
- Provide separate citations.
- Avoid declaring consensus without evidence.
- Allow filtering by school, scholar, era, or methodology.
- Present contradictory tafsir or grading views side-by-side with explicit labels instead of synthesizing one blended position.
- Never present an AI synthesis of conflicting views as the resolution of the disagreement.

---



# 22. Book and Source Catalog

Q-ai must provide a central source catalog.

## 22.1 Catalog Record

```text
source_id
title
alternate_titles
author
compiler
translator
editor
publisher
edition
publication_date
language
tradition
school
content_type
volumes
identifiers
source_urls
download_urls
license
copyright_status
checksum
signature
version
schema_version
last_checked_at
status
trust_level
```

## 22.2 Source States

```text
Discovered
PendingReview
Downloading
Downloaded
Validating
ValidationFailed
Staged
Approved
Indexing
Active
Deprecated
Quarantined
Removed
```

## 22.3 Internet Catalog Updates

Administrators must be able to:

- Add a catalog URL
- Discover available books
- Preview metadata
- Review licenses
- Compare versions
- Download sources
- Validate checksums
- Validate structure
- Preview textual differences
- Approve import
- Roll back an update
- Pin a specific version
- Disable automatic updates

No internet source may become active solely because an LLM recommended it.

## 22.4 Trusted Catalog Manifests

Catalog providers should publish signed or checksummed manifests.

Example:

```json
{
  "catalog_version": "1.4.0",
  "generated_at": "...",
  "sources": [
    {
      "id": "example-book",
      "version": "2.0.1",
      "url": "https://...",
      "sha256": "...",
      "license": "...",
      "format": "json"
    }
  ]
}
```

## 22.5 Update Safety

Updates must support:

- TLS validation
- Domain allowlists
- SSRF protection
- Download size limits
- Archive safety
- Malware scanning where available
- Schema validation
- Content hashing
- Duplicate detection
- License review
- Staged indexing
- Difference reports
- Rollback
- Audit logs

## 22.6 Source Genealogy

A source may be a translation, summary, abridgment, or edition of another work.

The catalog must support recording lineage per source:

```text
derives_from_source_id
derivation_type (translation | summary | edition | abridgment | commentary | original)
derivation_language
derivation_date
derivation_note
```

Lineage must be transitive and displayable, for example: "This text is an English translation (2024) of an Arabic summary (1990) of the original manuscript (1200)."

Retrieval and citation must distinguish which lineage member is actually quoted. A translation must never be presented as the original work.

---



# 23. Source Quality and Trust

Every source must have a configurable trust profile.

```text
CanonicalVerified
PublisherVerified
ScholarReviewed
CommunityReviewed
ImportedUnverified
MachineGenerated
UserProvided
Quarantined
```

Trust level must influence:

- Search filters
- Retrieval ranking
- Answer generation
- Warning labels
- Whether a source can support a definitive claim

Popularity or vector similarity must not be treated as authority.

---



# 24. User Experience

## 24.1 Main Web Navigation

```text
Home
├── Read Quran
├── Quran Search
├── Quran Graph
├── Word Families
├── Hadith
├── Narrators and Isnad
├── Tafsir
├── Comparative Scripture
├── Ask Q-ai
├── Research Workspaces
├── Saved Queries
├── Collections
├── Source Library
├── Imports and Updates
├── Agents and Tools
├── Jobs
├── Settings
└── Administration
```

## 24.2 Search Modes

- Simple
- Quran exact
- Arabic normalized
- Root and morphology
- Hadith
- Isnad
- Tafsir
- Graph
- Comparative
- Advanced query builder

## 24.3 Research Workspace

A workspace should store:

- Research question
- Selected corpora
- Search configuration
- Saved verses
- Saved hadith
- Graph snapshots
- Notes
- AI analyses
- Citations
- Source versions
- Query history
- Exported reports

## 24.4 Export

Supported exports should include:

- Markdown
- HTML
- PDF
- JSON
- CSV
- Graph JSON
- GraphML where practical
- Citation formats such as BibTeX or CSL-JSON where metadata permits

---



# 25. TUI Requirements

The TUI should use:

- `ratatui`
- `crossterm`

Required screens:

```text
Dashboard
Quran
├── Read
├── Search
├── Roots
├── Morphology
├── Word Families
└── Graph

Hadith
├── Collections
├── Search
├── Hadith View
├── Gradings
├── Narrators
└── Isnad Graph

Tafsir
Comparative Scripture
Research Chat
Knowledge Bases
Sources
Imports and Updates
RAG Systems
Agents
Tools
Runs
Jobs
Logs
Connections
Configuration
Doctor
```

The TUI must support Arabic and right-to-left text as far as terminal capabilities allow. The Web GUI is the authoritative rich Quran reading experience when terminal rendering is insufficient.

---

# 25.1 TUI Detailed Screen Inventory

The TUI is the operational cockpit for local and remote use. Every section of Q-ai must be reachable from the TUI without dropping to a terminal escape.

## Core Screens

```text
Dashboard

Quran
├── Read
│   ├── Surah navigator
│   ├── Ayah navigator
│   ├── Juz / Hizb / Rubʿ / Page navigator
│   ├── Translation panels
│   ├── Tafsir panel
│   ├── Word inspector
│   ├── Bookmark manager
│   └── Reading notes
├── Search
│   ├── Exact
│   ├── Normalized
│   ├── Concatenated
│   ├── Phrase
│   ├── Regex
│   └── Result inspector
├── Roots
│   ├── Root index
│   ├── All occurrences
│   └── Word family
├── Morphology
│   ├── Token morphology
│   ├── Compare datasets
│   └── Multiple analyses
├── Word Families
└── Graph
    ├── Neighbor explorer
    ├── Path explorer
    ├── Subgraph view
    └── Pattern query

Hadith
├── Collections
├── Search
├── Hadith View
├── Gradings
├── Narrators
└── Isnad Graph

Tafsir
├── Works
├── Compare
└── Verse lookup

Comparative Scripture
├── Editions
├── Parallel search
└── Translation compare

Research Chat
├── Modes
├── Sources
├── Tool plan
├── Trace
├── Citations
└── Export

Knowledge Bases
├── List
├── Create
├── Documents
└── Search

Sources
├── Catalog
├── Discovered
├── Staged
├── Active
├── Versions
├── Diff
└── Quarantine

Imports and Updates
├── Discover
├── Preview
├── Approve
└── Rollback

RAG Systems
├── List
├── Create
├── Edit
├── Run
└── Inspect

Agents
├── List
├── Create
├── Edit
├── Clone
├── Run
├── Permissions
├── Knowledge
├── Tools
├── Memory
└── Versions

Agencies
├── List
├── Create
├── Members
├── Routing
├── Budgets
└── Run

Tools
├── Installed
├── Quran Tools
├── Hadith Tools
├── Create
├── Generated Drafts
├── Validate
├── Test
├── Approvals
├── Permissions
├── Versions
└── Revoke

Runs
├── Active
├── Awaiting Approval
├── Completed
├── Failed
└── Trace

Jobs
├── Active
├── Queued
├── History
└── Logs

Logs
├── Stream
├── Filter
└── Export

Connections
├── List
├── Create
├── Edit
├── Test
└── Capabilities

Configuration
├── Server
├── Database
├── Providers
├── RAG Defaults
├── Tools
└── Security

Doctor
```

## Required TUI Capabilities

- Interactive dashboards
- Tables with sorting and filtering
- Lists with virtualized scrolling
- Form editors with validation
- Tabbed views
- Tree navigation
- Progress bars for long-running jobs
- Live log tailing
- Search fields with incremental filter
- Command palette
- Configuration editors with diff against saved state
- RAG pipeline visualization
- Job monitoring with cancel
- Server monitoring with connection counts
- Keyboard shortcuts with help overlay
- Mouse support where the terminal supports it
- Resizable panels

The TUI must feel like a professional developer tool. Ascii art and emoji are not acceptable as primary UI; UTF-8 box-drawing characters are acceptable.

The TUI must support Arabic and right-to-left text as far as terminal capabilities allow. The Web GUI is the authoritative rich Quran reading experience when terminal rendering is insufficient.

---

# 25.2 Command Palette

The TUI must provide a command palette as the single entry point for navigation and action.

```text
> Search commands...

Quran
  Open reader at 2:255
  Search exact
  Search normalized
  Open word family for root رحم
  Open graph around ayah 21:107

Hadith
  Search hadith
  Get hadith
  Compare variants
  Show isnad

Tools
  List tools
  Open tool builder
  Validate tool

Agents
  List agents
  Run agent
  Inspect agent

Connections
  Test connection
  Discover models

System
  Start server
  Stop server
  Open dashboard
  View logs
  Open configuration
  Open doctor
```

Commands must be searchable by name, alias, category, or fuzzy text.

---

# 25.3 RAG Debugging View

A major feature must be transparent RAG execution. Users must be able to inspect what happened at every step.

```text
Query
 ↓
Classified intent
 ↓
Selected tools and tool versions
 ↓
Selected sources and source versions
 ↓
Normalization rules applied
 ↓
Filters applied
 ↓
Retrieval results with scores
 ↓
Graph paths traversed
 ↓
Reranking scores
 ↓
Context builder output
 ↓
Prompt sent to model
 ↓
Model response with citations
 ↓
Citation validation result
```

This view must be available in TUI and GUI for the same run.

The view must show:

- Each tool call, its arguments, and the resulting observations
- The Quran canonical reference, edition, and version for every Quran quote
- The hadith collection, edition, and number for every hadith quote
- Vector and BM25 scores
- Reranker adjustments
- Time spent at each step
- Total tokens used
- Estimated cost
- Any rejected claims
- The full prompt actually sent

Users must be able to copy any step into a reproducible report.

---

# 25.4 CLI Design

The CLI must be consistent across all commands. The executable should be named `qai` or `q-ai`.

## Command Tree

```bash
qai serve
qai doctor
qai version

qai quran get <reference>
qai quran search <text> [--ignore-diacritics] [--root <root>]
qai quran root <root>
qai quran family <word>
qai quran morphology <reference:position>
qai quran graph path <start> <end>
qai quran graph neighbors <node>

qai hadith search <text>
qai hadith get <collection>:<volume>:<book>:<hadith>
qai hadith narrator search <name>
qai hadith isnad show <hadith-id>
qai hadith grading <hadith-id>

qai source list
qai source discover <catalog-url>
qai source import <manifest>
qai source update <source-id>
qai source diff <source-id> --from v1 --to v2
qai source approve <staged-version>
qai source rollback <source-id> --to <version>

qai rag list
qai rag create <name>
qai rag delete <name>
qai rag query <name> <text>
qai rag inspect <name> <query>

qai kb list
qai kb create <name>
qai kb delete <name>

qai document import <path>
qai document list
qai document inspect <id>
qai document delete <id>

qai agent list
qai agent run <agent> <text>
qai agent inspect <run-id>

qai tool list
qai tool inspect <id>
qai tool revoke <id>

qai server start
qai server stop
qai server status

qai config get <key>
qai config set <key> <value>
qai config show

qai research export <workspace-id>
qai workspace list
qai workspace create <name>
qai workspace delete <name>
```

## Global Flags

```text
--config <path>      Use a specific config file
--data-dir <path>    Use a specific data directory
--log-level <level>  trace | debug | info | warn | error
--no-color           Disable colored output
--json               Emit machine-readable JSON
--quiet              Suppress non-essential output
--yes                Assume yes for confirmations
```

## Conventions

- All commands are verbs first, nouns second.
- Identifiers are passed positionally only when unambiguous.
- Long flags use double dashes.
- Output is human-readable by default; `--json` is always supported.
- Errors include a short code, a human message, and a suggested action.
- Destructive commands ask for confirmation unless `--yes` is provided.

---

# 25.5 Generic Document Processing

General book and research material ingestion must follow the same disciplined pipeline as canonical corpora, but with configurable chunking, formatting, and trust.

## Supported Document Formats

Initial formats:

- Markdown
- Plain text
- PDF
- HTML
- JSON
- CSV
- DOCX

Future formats:

- EPUB
- XLSX
- PPTX
- Images
- Audio
- Video

## Document Metadata

Every document must record:

```text
id
name
path
source
type
size
created_at
updated_at
hash
metadata
status
ingestion_version
```

## Document Pipeline

```text
Import
  ↓
Validate
  ↓
Parse
  ↓
Normalize
  ↓
Chunk
  ↓
Embed
  ↓
Index
  ↓
Ready
```

The UI must show processing progress in real time.

```text
Document: architecture.pdf

Parsing       ████████████████ 100%
Chunking      ████████████████ 100%
Embedding     ███████████░░░░░  72%
Indexing      ░░░░░░░░░░░░░░░░   0%

Status: Processing
```

---

# 25.6 Chunking System

Chunking must be modular and corpus-aware.

## Strategies

- Fixed-size chunks
- Token-based chunks
- Sentence chunks
- Paragraph chunks
- Markdown-aware chunks
- Recursive chunks
- Parent-child chunks
- Hadith-aware chunks (isnad and matn preserved)
- Tafsir-aware chunks (verse-aligned)
- Scripture-aware chunks (verse-aligned)
- Semantic chunks

## Configuration

```text
chunk_size
chunk_overlap
separator
max_tokens
min_tokens
strategy
preserve_boundaries
```

When corpus structure exists (Quran ayah, hadith record, Bible verse), the chunker must respect it and never split across structural boundaries unless explicitly configured.

## Metadata per Chunk

Every chunk must support:

```json
{
  "document_id": "...",
  "source": "...",
  "edition": "...",
  "title": "...",
  "page": 12,
  "section": "Architecture",
  "language": "en",
  "created_at": "...",
  "canonical_reference": "...",
  "custom": {}
}
```

Metadata is used for filtering, display, debugging, citations, retrieval, and evaluation.

---

# 25.7 RAG Projects

A user must be able to create reusable, portable RAG projects.

```text
my-rag/
├── project.toml
├── documents/
├── prompts/
├── data/
├── cache/
└── exports/
```

A project should be exportable and portable. A project record must record its corpus, embedding model, vector store, retriever, reranker, LLM, and prompt template versions.

---

# 25.8 RAG Configuration Schema

A RAG configuration describes the complete pipeline declaratively.

```toml
[rag]
name = "Programming Knowledge"
version = "1.0.0"

[loader]
type = "filesystem"
path = "./documents"

[chunking]
strategy = "recursive"
size = 800
overlap = 100

[embedding]
provider = "openai-compatible"
model = "text-embedding-3-small"
dimensions = 1536

[vector_store]
type = "qdrant"
collection = "programming"

[full_text]
enabled = true
language = "en"

[retrieval]
strategy = "hybrid"
top_k = 10
bm25_weight = 0.4
vector_weight = 0.6

[reranker]
enabled = true
model = "bge-reranker-v2-m3"
top_n = 5

[llm]
provider = "openai-compatible"
model = "gpt-4o"
temperature = 0.1
max_tokens = 4000

[prompt]
template = "default"
include_citations = true

[citation]
policy = "claim_level"
validation = "strict"
```

This format is the input to the `qai rag create` and `qai rag import` commands.

---

# 25.9 Multiple RAG Composition

The system should support querying multiple RAG systems and merging their evidence.

```text
User Query
     │
     ├── Quran Tafsir RAG
     │
     ├── Shia Hadith RAG
     │
     └── Comparative Scripture RAG
              │
              ▼
       Result Aggregator
              │
              ▼
            LLM
```

Supported strategies:

- Parallel retrieval
- Weighted retrieval
- Result merging
- Cross-RAG reranking
- RAG routing
- Query classification
- School or edition filtering
- Citation merging with attribution preserved

---

# 25.10 Import Sources

Beyond the canonical Quran pipeline, users may import additional research material.

Initial sources:

- Filesystem
- Git repositories
- GitHub
- URLs and websites
- RSS feeds
- S3
- MinIO
- Google Drive
- Notion
- Telegram
- Email
- Databases
- APIs

Each source must use an adapter model with a typed manifest.

All imported content must be marked `ImportedUnverified` until reviewed.

---

# 25.11 Generic Web GUI Navigation

Beyond the Quran-specific Web GUI of section 24.1, the Web GUI must also support the platform-level views below.

```text
Home
├── System Dashboard
├── Sources
├── RAG Systems
├── Knowledge Bases
├── Documents
├── Connections
├── Agents
├── Tools
├── Workflows
├── Runs and Approvals
├── Jobs
├── Logs
├── Metrics
├── Settings
└── Administration
```

## RAG Query UI

```text
┌─────────────────────────────────────────┐
│ Select RAG: [Programming Knowledge ▼]  │
├─────────────────────────────────────────┤
│                                         │
│ Ask a question...                       │
│                                         │
│                              [Search]   │
└─────────────────────────────────────────┘
```

Results show:

- Answer
- Sources
- Retrieved chunks
- Scores
- Execution time
- Token usage
- Estimated cost
- Pipeline step trace

## Server Dashboard

```text
SYSTEM

Status       ● Running
Version      0.3.0
Uptime       2h 31m
Mode         Local

CORPORA
Quran Editions          1
Translations            8
Tafsir Works            12
Hadith Collections      7
Other Scriptures        5
Research Books          146

INDEXES
Quran Tokens            ...
Hadith Records          ...
Graph Nodes             ...
Graph Edges             ...
Vector Records          ...
Full-Text Documents     ...

RESEARCH
Active Conversations    4
Saved Workspaces        19
Active Runs             2
Awaiting Approvals      1

HEALTH
Corpus Integrity        ✓
Citation Resolver       ✓
Graph Store             ✓
Vector Store            ✓
Source Updates          3 available

PERFORMANCE
Requests                1,294
Average Search Latency  84 ms
Average Research Latency 2.1 s
```

---

# 25.12 Storage Architecture for Generic Content

Beyond the Quran and hadith stores, Q-ai requires a metadata layer for generic content.

## Document and Chunk Storage

- SQLite (initial)
- PostgreSQL (server)
- Optional S3-compatible object store for original files

## Vector Store

Initial options:

- Local embedded vector store
- Qdrant

Future adapters:

- PostgreSQL with pgvector
- OpenSearch
- Milvus
- Weaviate
- Other providers

## Full-Text Store

- Tantivy (initial) for Arabic and Latin text
- BM25 and phrase search
- Metadata filtering

## Graph Store

The graph layer must use an abstraction with multiple backends:

- Relational adjacency tables for the initial local release
- Embedded graph-oriented storage
- PostgreSQL-based graph representation
- Dedicated graph database in later releases

A dedicated graph database must not be required for the local MVP.

---

# 25.13 Performance, Error Handling, and Engineering Baseline

This section records engineering standards that apply across all of Q-ai.

## Performance

- Async architecture
- Non-blocking I/O
- Streaming responses
- Concurrent document processing
- Configurable worker pools
- Efficient memory usage
- Efficient serialization
- Cancellation support
- Cached canonical lookups
- Cached morphology and root queries
- Incremental graph updates
- Incremental embedding updates
- Memory-bounded result sets
- Bounded graph traversal
- Query timeouts
- Canonical Quran lookup and exact verse navigation must not require an LLM or vector database

CPU-heavy workloads must not block the async runtime.

## Error Handling

Use typed Rust errors. Prefer:

- `thiserror` for library errors
- `anyhow` at application boundaries where appropriate

Errors must provide:

```text
What happened
Why it happened
Where it happened
How the user can fix it
Suggested next command
```

## Secrets

API keys and credentials must be handled securely.

Possible mechanisms:

- Environment variables
- OS keychain
- Encrypted local storage
- Secret manager integration

Never expose secrets through:

- Logs
- API responses
- TUI output
- Browser HTML
- Error messages
- Audit records

## Configuration

Configuration should support:

```text
TOML
Environment Variables
CLI arguments
```

Configuration precedence:

```text
CLI
  >
Environment
  >
Config File
  >
Defaults
```

Configuration must be validated at load time and on every change.

## Observability

The application must use structured logging. Support:

- Logs
- Metrics
- Traces
- Request timing
- Retrieval timing
- Token usage
- Error tracking

Prefer Rust ecosystem standards:

- `tracing`
- `tracing-subscriber`
- OpenTelemetry where appropriate

## Dependency Principles

Dependencies must be selected based on:

1. Project maturity
2. Maintenance activity
3. Rust ecosystem adoption
4. API quality
5. Performance
6. Security
7. License compatibility
8. Cross-platform support

Avoid unnecessary dependencies. Do not introduce a dependency merely because it is convenient if a small internal abstraction is preferable.

## Security Baseline

- Validate all external input
- Protect API keys
- Avoid command injection
- Avoid arbitrary filesystem access
- Sanitize uploaded documents where appropriate
- Enforce configurable file-size limits
- Enforce request limits
- Use secure HTTP configuration
- Avoid leaking sensitive information through logs
- Provide authentication for exposed server deployments
- Bind to localhost by default
- Require TLS in production
- Read-only database credentials by default

## Testing Strategy

Unit tests cover:

- Chunkers
- Parsers
- Retrievers
- Prompt builders
- Configuration
- Domain logic

Integration tests cover:

- Vector stores
- LLM providers
- Embedding providers
- API
- Database
- Source importers
- Tool sandboxes

End-to-end tests cover the full pipeline:

```text
Import document
     ↓
Index document
     ↓
Query RAG
     ↓
Retrieve context
     ↓
Generate answer
     ↓
Return citations
```

Corpus-specific tests cover:

- Canonical integrity
- Arabic normalization round-trip
- Token order preservation
- Citation resolution
- Edition switching

## Developer Experience

The project must provide:

```bash
cargo build
cargo test
cargo clippy
cargo fmt
```

Plus:

- `.env.example`
- Example configurations
- Example RAG projects
- Docker configuration
- Development documentation
- A `qai doctor --json` mode for CI

## Plugin Architecture (Generic Adapters)

The architecture should allow future plugins.

Potential plugin types:

```text
LLM Plugin
Embedding Plugin
Vector Store Plugin
Document Loader Plugin
Retriever Plugin
Reranker Plugin
Authentication Plugin
Storage Plugin
UI Plugin
```

Plugins should be introduced only when the architecture genuinely benefits from them. Avoid premature dynamic plugin complexity.

## Docker

The server should be deployable with Docker.

```text
docker run ...
```

Potential Docker Compose stack:

```text
app
├── Rust Q-ai Server
├── PostgreSQL
├── Qdrant
└── Optional monitoring
```

The application must also work without Docker.

## Local-First Design

The application must work locally without requiring a cloud service.

```text
Q-ai
  ├── SQLite
  ├── Local filesystem
  ├── Local embeddings
  ├── Ollama
  ├── Local full-text engine
  ├── Local vector store
  └── Local graph store
```

Cloud services are optional. The `doctor` command must report any local-only fallback paths.

---

# 25.14 Non-Goals (Generic)

In addition to the Quran-specific non-goals of section 4, the generic platform will not initially:

- Build its own foundation model
- Implement every vector database natively
- Implement every document parser from scratch
- Become a full enterprise data warehouse
- Replace general-purpose databases
- Support distributed clusters in the first release

These can be added later through adapters and plugins.

---

# 25.15 Open Technical Decisions (Generic)

These decisions must be evaluated during implementation:

- Web GUI framework
- Primary database
- Vector database default
- Local embedding engine
- Plugin architecture mechanism
- Authentication framework
- API framework
- Job queue implementation
- WebSocket versus Server-Sent Events
- Configuration format extensions
- Local model runtime
- RTL terminal strategy
- WASM runtime
- Tool signing method

Technology decisions should favor mature Rust ecosystem solutions. Each decision must be recorded as an ADR per Section 60.

---



# 26. CLI Requirements

The executable should be named `q-ai` or `qai`.

Examples:

```bash
qai
qai serve
qai doctor

qai quran get 2:255
qai quran search "الرحمن"
qai quran search --ignore-diacritics "الرحمن"
qai quran root "ر ح م"
qai quran family "رحمة"
qai quran morphology 1:1:1
qai quran graph path root:رحم ayah:21:107

qai hadith search "..."
qai hadith get al-kafi:1:2:3
qai hadith narrator search "..."
qai hadith isnad show <hadith-id>

qai source list
qai source discover <catalog-url>
qai source import <manifest>
qai source update <source-id>
qai source diff <source-id> --from v1 --to v2
qai source approve <staged-version>

qai rag list
qai rag query quran-tafsir "..."
qai research export <workspace-id>
```

---



# 27. API Requirements

The API must be versioned.

## 27.1 Quran API

```text
GET  /api/v1/quran/editions
GET  /api/v1/quran/surahs
GET  /api/v1/quran/ayahs/:reference
GET  /api/v1/quran/context/:reference
POST /api/v1/quran/search/exact
POST /api/v1/quran/search/normalized
POST /api/v1/quran/search/phrase
POST /api/v1/quran/search/root
POST /api/v1/quran/search/lemma
POST /api/v1/quran/word-family
POST /api/v1/quran/morphology
POST /api/v1/quran/frequency
POST /api/v1/quran/cooccurrence
POST /api/v1/quran/graph/neighbors
POST /api/v1/quran/graph/path
POST /api/v1/quran/graph/query
```

## 27.2 Hadith API

```text
GET  /api/v1/hadith/collections
GET  /api/v1/hadith/:id
POST /api/v1/hadith/search
GET  /api/v1/hadith/:id/gradings
GET  /api/v1/hadith/:id/commentary
GET  /api/v1/hadith/:id/isnad
POST /api/v1/hadith/compare
POST /api/v1/narrators/search
GET  /api/v1/narrators/:id
POST /api/v1/isnad/path
```

## 27.3 Source API

```text
GET  /api/v1/sources
POST /api/v1/sources
GET  /api/v1/sources/:id
POST /api/v1/sources/discover
POST /api/v1/sources/:id/download
POST /api/v1/sources/:id/validate
POST /api/v1/sources/:id/stage
POST /api/v1/sources/:id/approve
POST /api/v1/sources/:id/update
GET  /api/v1/sources/:id/versions
GET  /api/v1/sources/:id/diff
POST /api/v1/sources/:id/rollback
```

## 27.4 Research and Query API

```text
POST /api/v1/query
POST /api/v1/research
GET  /api/v1/research/:id
GET  /api/v1/research/:id/evidence
GET  /api/v1/research/:id/trace
POST /api/v1/research/:id/export
```

Streaming may use WebSocket or Server-Sent Events.

---



# 28. Agentic Runtime

Agents are first-class but controlled.

## 28.1 Initial Agents

```text
Research Router
Quran Linguistics Researcher
Quran Graph Researcher
Hadith Researcher
Isnad Researcher
Tafsir Comparison Researcher
Comparative Scripture Researcher
Citation Verifier
Answer Reviewer
```

## 28.2 Suggested Research Agency

```text
User Question
    ↓
Research Router
    ├── Quran Linguistics Agent
    ├── Hadith Agent
    ├── Tafsir Agent
    ├── Graph Agent
    └── Comparative Scripture Agent
             ↓
Citation Verifier
             ↓
Disagreement and Safety Reviewer
             ↓
Final Answer Synthesizer
```

## 28.3 Agent Limits

Each run must define:

- Maximum steps
- Maximum tool calls
- Maximum retrieval calls
- Maximum tokens
- Maximum cost
- Maximum runtime
- Allowed sources
- Allowed tools
- Network policy
- Output schema

Agents must not modify canonical corpora.

---



# 29. Tool Runtime and Security

Tools must use typed manifests.

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn manifest(&self) -> ToolManifest;

    async fn execute(
        &self,
        context: ToolContext,
        input: serde_json::Value,
    ) -> Result<ToolResult>;
}
```

Every manifest must include:

```text
id
name
version
description
input_schema
output_schema
permissions
source_domains
side_effect_class
timeout
resource_limits
publisher
signature
checksum
status
```

Side-effect classes:

```text
ReadOnly
ComputationalAnnotation
LocalWrite
ExternalWrite
CommandExecution
Privileged
```

`ComputationalAnnotation` covers tools that produce machine-generated linguistic, graph, or cross-reference annotations. Their outputs must carry the Layer D computational-annotation metadata of section 6.4 and must never be written into canonical data.

Risk tiers and default approval behavior:

```text
ReadOnly                -> freely callable by authorized agents, no approval
ComputationalAnnotation -> callable, outputs flagged "Needs Review" until verified
LocalWrite              -> explicit permission required
ExternalWrite           -> explicit permission plus human approval
CommandExecution        -> disabled by default, always require human approval
Privileged              -> architecturally isolated from agent tool paths
```

Quran and hadith research tools should normally be `ReadOnly`.

Write-capable tools require explicit permission and, where necessary, human approval.

---



# 30. AI-Assisted Tool Creation

Q-ai may assist developers in creating new research tools.

Allowed initial formats:

1. Declarative query tools
2. Workflow compositions
3. Declarative HTTP/OpenAPI tools
4. Sandboxed WebAssembly tools

Lifecycle:

```text
Requirement
→ Schema
→ Implementation
→ Static Validation
→ Security Scan
→ Build
→ Tests
→ Corpus Accuracy Tests
→ Human Review
→ Signing
→ Publication
```

Generated tools must never:

- Modify Quran canonical text
- Replace source citations with generated references
- Publish automatically
- Receive unrestricted filesystem access
- Receive unrestricted network access
- Execute arbitrary shell commands by default
- Present generated morphology as verified scholarship without a label

---



# 31. Model and Provider Support

Q-ai must support abstractions for:

- LLMs
- Embeddings
- Rerankers
- Arabic NLP models
- Transliteration engines
- Morphology analyzers
- Entity extraction models
- OCR systems

Initial model providers:

- OpenAI-compatible APIs
- OpenAI
- Anthropic
- Ollama
- Local model servers
- Custom HTTP providers

Provider capabilities must be discovered rather than assumed.

Model routing must consider:

- Required tool calling
- Arabic quality
- Persian quality
- Context length
- Structured output
- Privacy
- Local-only policy
- Cost
- Latency
- Provider health

---



# 32. Storage Architecture

Q-ai requires multiple storage types behind abstractions.

## 32.1 Relational Metadata Store

Initial:

- SQLite

Future/server:

- PostgreSQL

Stores:

- Sources
- Editions
- Documents
- Quran structure
- Hadith structure
- Users
- Workspaces
- Configurations
- Jobs
- Runs
- Audit records

## 32.2 Full-Text Index

Must support:

- Arabic-aware tokenization
- Exact fields
- Normalized fields
- Phrase search
- BM25-style search
- Metadata filtering

The implementation may use Tantivy or another suitable engine after an ADR.

## 32.3 Vector Store

Initial options:

- Local embedded vector store
- Qdrant

Future adapters:

- PostgreSQL with pgvector
- OpenSearch
- Milvus
- Weaviate
- Other providers

## 32.4 Graph Store

The graph layer must use an abstraction.

Possible implementations:

- Relational adjacency tables for the initial local release
- Embedded graph-oriented storage
- PostgreSQL-based graph representation
- Dedicated graph database in later releases

A dedicated graph database must not be required for the local MVP.

## 32.5 Object and File Storage

Used for:

- Original source files
- Scans
- Parsed representations
- Audio
- Exported research reports
- Source manifests

---



# 33. Suggested Rust Workspace

```text
q-ai/
├── Cargo.toml
├── crates/
│   ├── domain/
│   ├── application/
│   ├── quran-core/
│   ├── quran-corpus/
│   ├── quran-normalization/
│   ├── quran-morphology/
│   ├── quran-search/
│   ├── quran-graph/
│   ├── hadith-core/
│   ├── hadith-ingestion/
│   ├── isnad-graph/
│   ├── scripture/
│   ├── sources/
│   ├── ingestion/
│   ├── documents/
│   ├── retrieval/
│   ├── rag/
│   ├── graph/
│   ├── embeddings/
│   ├── reranking/
│   ├── llm/
│   ├── model-router/
│   ├── agents/
│   ├── agency/
│   ├── tools/
│   ├── tool-sdk/
│   ├── tool-registry/
│   ├── tool-sandbox/
│   ├── workflows/
│   ├── policy/
│   ├── approvals/
│   ├── citations/
│   ├── provenance/
│   ├── evaluation/
│   ├── storage/
│   ├── jobs/
│   ├── api/
│   ├── server/
│   ├── tui/
│   ├── cli/
│   ├── config/
│   └── audit/
├── web/
├── migrations/
├── corpora/
├── manifests/
├── tests/
├── docs/
├── examples/
└── adr/
```

Core domain crates must not depend on the Web GUI, TUI, or concrete external providers.

---



# 34. Data Ingestion Pipeline

```text
Discover Source
    ↓
Review Metadata and License
    ↓
Download to Quarantine
    ↓
Verify Checksum and Signature
    ↓
Detect Format
    ↓
Parse
    ↓
Validate Structure
    ↓
Normalize Search Copies
    ↓
Preserve Original Text
    ↓
Generate Difference Report
    ↓
Human Approval
    ↓
Create Version
    ↓
Build Full-Text Index
    ↓
Build Structured Index
    ↓
Build Embeddings
    ↓
Build Graph
    ↓
Reconcile Indexes
    ↓
Activate
```

The pipeline must be:

- Asynchronous
- Idempotent
- Cancellable
- Retry-safe
- Version-aware
- Observable
- Reversible

---



# 35. Validation Requirements

## 35.1 Quran Validation

- Expected surah count
- Valid verse identifiers for the selected numbering scheme
- No missing verses
- No duplicate verse identifiers
- Stable token order
- Unicode validity
- Source checksum
- Round-trip serialization
- Comparison against approved reference corpus
- Search normalization does not modify canonical display text

## 35.2 Hadith Validation

- Collection hierarchy integrity
- Stable source numbering
- Volume and page consistency where supplied
- Isnad and matn boundary checks
- No silent text truncation
- Edition provenance
- Alternate numbering preservation

## 35.3 Citation Validation

- Source exists
- Location resolves
- Quotation matches
- Edition exists
- User has permission
- Hash and version are available

---



# 36. Evaluation

## 36.1 Quran Evaluation

- Exact verse retrieval accuracy
- Diacritic-insensitive search recall
- Concatenated phrase search accuracy
- Root-search precision and recall
- Lemma-search accuracy
- Morphological segmentation accuracy
- Word-family explanation accuracy
- Graph path correctness
- Verse citation correctness
- Canonical text integrity

## 36.2 Hadith Evaluation

- Hadith lookup accuracy
- Collection and numbering resolution
- Isnad segmentation accuracy
- Narrator identity precision
- Grading attribution accuracy
- Variant detection quality
- Citation correctness

## 36.3 RAG Evaluation

- Retrieval precision
- Retrieval recall
- Context relevance
- Citation support
- Faithfulness
- Source diversity
- Disagreement preservation
- Access-control leakage
- Prompt-injection resistance

## 36.4 Agent Evaluation

- Tool-selection accuracy
- Invalid tool-call rate
- Citation hallucination rate
- Source-routing accuracy
- Budget compliance
- Policy compliance
- Recovery from retrieval failure

Datasets, expected outputs, and results must be versioned.

---



# 37. Security

Required controls:

- Bind to localhost by default.
- Require authentication for remote access.
- Require TLS for non-local production deployments.
- Protect API keys with environment variables, OS keychain, or encrypted storage.
- Use deny-by-default tool permissions.
- Apply SSRF protection to source importers and HTTP tools.
- Prevent path traversal and symlink escape.
- Prevent archive zip-slip attacks.
- Treat uploaded and retrieved content as untrusted.
- Sanitize HTML and scripts.
- Apply file-size and request-size limits.
- Apply timeouts, rate limits, and concurrency limits.
- Separate instructions from retrieved documents.
- Never treat book text as privileged model instructions.
- Redact secrets from prompts, logs, events, and audit records.
- Validate all structured model outputs.
- Audit source updates and canonical corpus changes.
- Fail closed if sandbox policy cannot be enforced.

---



# 38. Copyright and Licensing

Before activation, every imported source must have a recorded licensing status:

```text
PublicDomain
OpenLicense
PermissionGranted
UserOwned
MetadataOnly
Unknown
Restricted
```

The product must:

- Avoid shipping copyrighted books without permission.
- Preserve attribution required by licenses.
- Allow metadata-only catalog entries.
- Restrict export when required.
- Allow administrators to remove or disable a source.
- Record license changes between source versions.
- Display licensing warnings before internet import.

Technical availability does not imply legal permission to redistribute.

---



# 39. Observability and Provenance

Use structured logging and tracing, preferably:

- `tracing`
- `tracing-subscriber`
- OpenTelemetry where appropriate

Every research run should record:

```text
run_id
user_id
workspace_id
query
query_plan
tools_called
tool_versions
source_ids
source_versions
edition_ids
normalization_rules
retrieval_results
graph_paths
reranking_scores
model_provider
model_name
prompt_version
citations
token_usage
estimated_cost
timestamps
warnings
```

No private research content may be collected as external telemetry without explicit opt-in.

---



# 40. Performance

The system should provide:

- Async, non-blocking I/O
- Concurrent ingestion
- Streaming model responses
- Streaming search results where useful
- Cached canonical lookups
- Cached morphology and root queries
- Incremental graph updates
- Incremental embedding updates
- Configurable worker pools
- Cancellation propagation
- Memory-bounded result sets
- Bounded graph traversal
- Query timeouts

Canonical Quran lookup and exact verse navigation should not require an LLM or vector database.

---



# 41. Reliability and Backups

Provider, tool, and source operations require:

- Configurable timeouts
- Bounded retries
- Exponential backoff
- Jitter
- Circuit breakers where appropriate
- Idempotency keys for side effects
- Interrupted-job detection
- Safe checkpoints
- Index reconciliation
- Orphan detection

Backups must include:

- Metadata database
- User annotations
- Research workspaces
- Source manifests
- Corpus versions
- Graph annotations
- Configuration
- Audit data where configured

Indexes and embeddings should be rebuildable from source versions and manifests.

---



# 42. Authentication and Access Control

Initial modes:

- Local single-user
- Optional local authentication
- Server authentication
- API tokens

Future server mode:

- Multiple users
- Roles
- Workspaces
- RBAC
- OAuth/OIDC

Permissions may control:

- Corpus access
- Restricted books
- Source import
- Source approval
- Corpus updates
- Agent use
- Tool use
- Export
- Administration

Retrieval filters must be applied before results reach an LLM or agent.

---



# 43. Development Phases
## Phase 0 — Foundations

- Rust workspace
- Domain model
- Configuration
- SQLite
- Migration framework
- Logging
- CLI
- Job system
- Source manifests
- Provenance model
- Security baseline

## Phase 1 — Canonical Quran Engine

- Validated Quran edition import
- Surah and ayah model
- Token model
- Exact lookup
- Arabic normalization
- Exact and normalized search
- Phrase search
- Concatenated-word search
- Quran API
- Corpus integrity tests

## Phase 2 — Quran Linguistics

- Lemma index
- Root index
- Morphological analyses
- Word-family tool
- Frequency
- Distribution
- Co-occurrence
- Morphology comparison
- Linguistic provenance

## Phase 3 — Quran Graph

- Graph abstraction
- Quran nodes and edges
- Neighbor search
- Path search
- Subgraph view
- Entity and concept annotations
- Graph provenance
- Graph visualization

## Phase 4 — Rich Quran Web Experience

- Arabic reading view
- RTL support
- Translation panels
- Word inspector
- Tafsir panel
- Deep links
- Citation copy
- Search interface
- Graph explorer
- Saved research

## Phase 5 — Hadith and Tafsir

- Structured hadith model
- Initial Shia collection adapter
- Al-Kafi support
- Bihar al-Anwar support
- Hadith search
- Grading attribution
- Tafsir indexing
- Verse-to-tafsir links
- Citation validation

## Phase 6 — Isnad Graph

- Narrator records
- Name variants
- Isnad parsing
- Identity review workflow
- Narrator graph
- Chain paths
- Rijal integration
- Uncertainty representation

## Phase 7 — Multi-RAG and Smart Routing

- Full-text index
- Vector retrieval
- Hybrid retrieval
- Reranking
- Multi-RAG manager
- Query classification
- Smart tool selection
- Citation-aware context builder
- Retrieval debugging

## Phase 8 — Comparative Scripture

- Edition-aware scripture model
- Torah/Bible adapters
- Passage search
- Translation comparison
- Cross-reference graph
- Parallel-passage research
- Explicit computational-suggestion labels

## Phase 9 — Agentic Research

- Research router agent
- Quran research tools
- Hadith tools
- Graph tools
- Citation verifier
- Tool policy engine
- Budgets
- Run traces
- Human approvals
- MCP support

## Phase 10 — Source Catalog and Internet Updates

- Catalog discovery
- Signed/checksummed manifests
- Download quarantine
- License review
- Structural validation
- Staging
- Version diff
- Approval
- Rollback
- Scheduled update checks

## Phase 11 — Production Hardening

- PostgreSQL
- Qdrant
- Authentication
- RBAC
- TLS
- Docker
- Backups
- OpenTelemetry
- Performance testing
- Security testing
- Disaster recovery

---


## Phase 0 — Foundations

- Rust workspace
- Domain model
- Typed errors
- Configuration
- Secrets
- SQLite
- Migration framework
- Logging and tracing
- Background jobs
- Basic CLI
- Source manifest schema
- Provenance model
- Audit model

## Phase 1 — Canonical Quran Core

- Approved Quran source
- Quran edition model
- Canonical importer
- Corpus validator
- Surah and ayah model
- Token model
- Stable Quran addressing
- Exact lookup
- Quran reader API
- Canonical integrity tests

## Phase 2 — Quran Search and Linguistics

- Arabic normalization
- Diacritic-free search
- Concatenated phrase search
- Exact phrase search
- Root index
- Lemma index
- Stem index
- Morphological analyses
- Word families
- Frequency and distribution
- Co-occurrence and collocation
- Linguistic provenance

## Phase 3 — Quran Graph

- Graph abstraction
- Quran graph schema
- Structural edges
- Linguistic edges
- Entity and concept annotations
- Neighbor search
- Path search
- Bounded graph pattern search
- Subgraph export
- Graph visualization
- Edge provenance

## Phase 4 — Rich Quran Experience

- Responsive Web GUI
- High-quality Arabic rendering
- RTL layout
- Reading navigation
- Translation panels
- Word inspector
- Morphology panel
- Word-family explorer
- Tafsir panel
- Quran graph explorer
- Notes and bookmarks
- Citation deep links

## Phase 5 — Shia Hadith and Tafsir

- Structured hadith domain model
- Al-Kafi ingestion
- Bihar al-Anwar ingestion
- Additional approved Shia collections
- Hadith exact and normalized search
- Matn/isnad separation
- Grading attribution
- Commentary indexing
- Tafsir indexing
- Quran-to-hadith links
- Quran-to-tafsir links

## Phase 6 — Narrator and Isnad Research

- Narrator model
- Name variants
- Isnad parsing
- Narrator identity review
- Teacher-student relationships
- Isnad graph
- Rijal source integration
- Uncertainty representation
- Chain path search

## Phase 7 — Multi-RAG

- Document ingestion
- Full-text engine
- Embeddings
- Local vector store
- Hybrid retrieval
- Metadata filters
- Parent-child retrieval
- Reranking
- Multi-RAG manager
- Retrieval debugging
- Citation-aware context construction

## Phase 8 — Comparative Scripture and Books

- Edition-aware scripture model
- Torah and Bible source adapters
- Chapter and verse addressing
- Translation comparison
- Parallel-passage research
- Cross-scripture graph
- General research-book ingestion
- Explicit labeling of computationally suggested parallels

## Phase 9 — Agentic Research

- Model connectivity
- Capability discovery
- Model routing
- Agent definitions
- Tool calling
- Quran tools
- Hadith tools
- Tafsir tools
- Graph tools
- Citation verifier
- Permission engine
- Human approvals
- Budgets and traces

## Phase 10 — Safe Tool Creation and Workflows

- Tool manifests
- Tool registry
- Declarative tools
- WASM sandbox
- Tool-generation workflow
- Validation
- Security testing
- Human publication approval
- Workflow engine
- MCP integration

## Phase 11 — Server and Web Management

- Versioned HTTP API
- Streaming API
- Authentication
- Workspaces
- RBAC
- Source management
- Update approval inbox
- Agent management
- Tool management
- Audit viewer

## Phase 12 — Production Hardening

- PostgreSQL
- Qdrant
- TLS
- Docker
- Backups
- OpenTelemetry
- Performance tests
- Security tests
- Corpus disaster recovery
- Index reconstruction
- Deployment documentation

---

# 44. MVP Definition
The first Q-ai MVP must be intentionally focused on trustworthy Quran research.

## 44.1 MVP Includes

- Local-first Rust application
- Web GUI
- CLI
- Basic TUI
- SQLite metadata database
- One validated Quran edition
- Surah and ayah navigation
- Exact Arabic search
- Search without diacritics
- Concatenated phrase search
- Root search
- Lemma search
- Word-family explorer
- Morphological token display
- Frequency and distribution tools
- Basic Quran graph
- One or more translations with clear attribution
- Citation links
- Research chat using Quran tools
- OpenAI-compatible provider
- Ollama provider
- Tool-call inspection
- Run traces
- Corpus integrity tests
- Manual source import
- Versioned source manifests

## 44.2 MVP Excludes

- Full autonomous internet updates
- Every hadith collection
- Every Quran qira'ah
- Definitive automated narrator identity resolution
- Unrestricted generated tools
- Arbitrary shell execution
- Full multi-user collaboration
- Claims of authoritative religious rulings
- Unreviewed AI changes to canonical corpora

## 44.3 MVP Acceptance Workflow

```text
Install Q-ai
   ↓
Open the Quran reader
   ↓
Navigate to a surah and ayah
   ↓
Verify exact canonical Arabic text
   ↓
Search an Arabic phrase without diacritics
   ↓
Search the same phrase without spaces
   ↓
Select a Quran word
   ↓
Inspect its lemma, root, morphology, and word family
   ↓
View all related occurrences
   ↓
Open a Quran graph around the word or verse
   ↓
Ask Q-ai a research question
   ↓
Inspect selected tools and retrieved evidence
   ↓
Receive an answer with exact verse citations
   ↓
Open every citation at the correct source location
   ↓
Export the query and evidence
```

---


The MVP is complete when the following workflow succeeds:

```text
Install Q-ai
   ↓
Start locally
   ↓
Open the Quran reader
   ↓
Navigate to a verse
   ↓
Verify exact canonical Arabic
   ↓
Search Arabic with and without diacritics
   ↓
Search a phrase without spaces
   ↓
Inspect token morphology
   ↓
Search by lemma and root
   ↓
Explore the word family
   ↓
View a basic Quran graph
   ↓
Configure a local or remote model
   ↓
Ask a Quran research question
   ↓
Inspect called tools and retrieved evidence
   ↓
Receive a streamed answer with exact citations
   ↓
Open each citation
   ↓
Export the research result
```

The same core operations must be available through:

- CLI
- TUI
- Web GUI
- API

The MVP must include:

- Local-first execution
- One approved Quran edition
- Canonical validation
- Exact verse lookup
- Exact and normalized Arabic search
- Concatenated phrase search
- Root and lemma search
- Word-family exploration
- Morphological display
- Basic Quran graph
- Attributed translations
- Citation validation
- One local vector-store option
- SQLite
- OpenAI-compatible models
- Ollama
- Streaming
- Run cancellation
- Execution traces
- Provider diagnostics

---

# 45. Post-MVP Acceptance Workflow

```text
Select Quran, Al-Kafi, Bihar al-Anwar, and tafsir sources
   ↓
Ask a cross-source research question
   ↓
Smart router selects Quran, hadith, and tafsir tools
   ↓
Q-ai performs structured and hybrid retrieval
   ↓
Q-ai retrieves hadith with edition-aware references
   ↓
Q-ai displays attributed gradings
   ↓
Q-ai shows graph links between verses and narrations
   ↓
Citation verifier checks every source
   ↓
Answer distinguishes source quotation from AI analysis
   ↓
User opens exact verse, hadith, volume, page, and tafsir references
   ↓
User saves and exports a reproducible research report
```

---



# 46. Quality Gates

A Quran feature is complete only when:

- It cannot modify canonical text accidentally.
- It has corpus-integrity tests.
- It has Unicode and Arabic normalization tests.
- It preserves exact source addressing.
- Its outputs contain source versions.
- Generated analysis is clearly labeled.
- It works without requiring an LLM where deterministic processing is sufficient.
- It has performance limits.
- It has API, UI, and domain tests.

A hadith feature is complete only when:

- It identifies collection and edition.
- It preserves source numbering.
- It distinguishes isnad and matn where supported.
- It attributes gradings.
- It represents disagreement.
- It resolves citations to exact locations.
- It avoids presenting computational identity matching as certainty.

An agent feature is complete only when:

- Tool permissions are enforced.
- Inputs and outputs are schema-validated.
- Cancellation and timeout work.
- Resource limits are enforced.
- Tool and model versions are recorded.
- Citations are validated.
- Retrieved content is treated as untrusted.
- The user can inspect what happened.

---



# 47. Product Success Metrics
Initial metrics:

- Canonical Quran integrity test pass rate
- Exact verse retrieval accuracy
- Normalized Arabic search recall
- Concatenated phrase search accuracy
- Root and lemma search accuracy
- Citation resolution success rate
- Unsupported citation rate
- Percentage of claims with valid citations
- Search latency
- Graph-query latency
- Time to first successful Quran search
- Time to first cited research answer
- Hadith source-resolution accuracy
- Grading-attribution accuracy
- Smart-router tool-selection accuracy
- Agent invalid-tool-call rate
- Index consistency error rate
- Crash-free run rate

Target integrity requirements:

- Canonical Quran text corruption tolerance: **zero**
- Fabricated verse references: **zero in validated deterministic lookup**
- Silent source-version changes: **zero**
- Unattributed hadith grading presented as universal: **zero**

---


Metrics include:

- Time to first Quran lookup
- Time to first normalized search
- Time to first cited research answer
- Canonical integrity pass rate
- Search accuracy
- Root and lemma accuracy
- Concatenated phrase accuracy
- Citation resolution rate
- Citation-support rate
- Unsupported citation rate
- Retrieval latency
- Graph latency
- End-to-end latency
- Connection success rate
- Agent completion rate
- Tool-call success rate
- Invalid call rate
- Approval rate
- Cancellation success rate
- Index consistency error rate
- Prevented policy violations
- Crash-free run rate
- Cost per successful task

No prompts, documents, research questions, or model responses may be collected as telemetry without explicit opt-in.

---

# 48. Open Technical Decisions
The following require Architecture Decision Records:

- Initial Quran source dataset and license
- Initial Quran morphology dataset
- Arabic normalization standard
- Preferred transliteration standard
- Initial Shia hadith data sources
- Tafsir source licensing
- Quran graph storage implementation
- Full-text engine
- Local vector store
- Arabic embedding model
- Arabic reranker
- Narrator identity model
- Hadith numbering reconciliation
- Web frontend framework
- RTL terminal strategy
- WebSocket versus Server-Sent Events
- Source-manifest signing method
- Catalog trust model
- Graph export formats
- Citation-support verification method
- Public API rate limits

Each ADR must record:

- Context
- Options
- Decision
- Accuracy implications
- Religious-source implications
- Licensing implications
- Security implications
- Migration strategy
- Reversal cost

---


The following must be resolved through Architecture Decision Records:

- Approved initial Quran dataset
- Initial Quran script and recitation edition
- Quran morphology dataset
- Arabic normalization rules
- Transliteration standard
- Full-text search engine
- Local vector-store implementation
- Graph-store implementation
- Graph query representation
- Quran graph schema versioning
- Hadith source formats
- Hadith numbering reconciliation
- Narrator identity representation
- Rijal data integration
- Tafsir source licensing
- Torah and Bible source licensing
- Local Arabic embedding model
- Arabic and Persian reranker
- Web GUI framework
- RTL terminal strategy
- API framework
- WebSocket versus Server-Sent Events
- Job queue implementation
- Agent state-machine representation
- Workflow graph representation
- Policy engine
- WASM runtime
- MCP SDK and transports
- Tool-signing mechanism
- Catalog-manifest signing
- Secret storage on each operating system
- Audit-log integrity
- Citation-support verification
- Source-difference algorithms
- Conversation retention defaults

Every ADR must document:

- Context
- Options considered
- Decision
- Accuracy implications
- Corpus implications
- Licensing implications
- Security implications
- Operational implications
- Migration strategy
- Reversal cost

---

# 49. Current Project Status
## Foundations

- [ ] Rust workspace
- [ ] Core domain
- [ ] Configuration
- [ ] Persistence
- [ ] Jobs
- [ ] Logging
- [ ] CLI
- [ ] Web server
- [ ] Web GUI
- [ ] TUI

## Generic RAG Platform

- [ ] Document processing pipeline
- [ ] Chunking system
- [ ] Embedding abstraction
- [ ] LLM abstraction
- [ ] Vector store abstraction
- [ ] Full-text index
- [ ] Multi-RAG
- [ ] RAG debugging view
- [ ] RAG projects
- [ ] Generic document import sources

## Quran

- [ ] Quran edition model
- [ ] Canonical importer
- [ ] Corpus validator
- [ ] Quran navigation
- [ ] Exact search
- [ ] Normalized search
- [ ] Concatenated search
- [ ] Root index
- [ ] Lemma index
- [ ] Morphology
- [ ] Word families
- [ ] Frequency tools
- [ ] Quran graph
- [ ] Graph explorer
- [ ] Rich Quran reader

## Hadith and Tafsir

- [ ] Hadith collection model
- [ ] Al-Kafi adapter
- [ ] Bihar al-Anwar adapter
- [ ] Hadith search
- [ ] Grading attribution
- [ ] Tafsir indexing
- [ ] Isnad parser
- [ ] Narrator graph
- [ ] Rijal integration

## RAG and Agents

- [ ] Full-text retrieval
- [ ] Vector retrieval
- [ ] Hybrid retrieval
- [ ] Graph RAG
- [ ] Smart router
- [ ] Tool registry
- [ ] Agent runtime
- [ ] Citation verifier
- [ ] Research agency
- [ ] Evaluation framework

## Sources

- [ ] Source catalog
- [ ] Manifest format
- [ ] License tracking
- [ ] Internet discovery
- [ ] Quarantine
- [ ] Validation
- [ ] Update diff
- [ ] Approval
- [ ] Rollback

---


## Foundations

- [ ] Project architecture
- [ ] Cargo workspace
- [ ] Core domain
- [ ] Configuration
- [ ] SQLite persistence
- [ ] Migration framework
- [ ] CLI
- [ ] TUI
- [ ] API
- [ ] Server mode
- [ ] Web GUI
- [ ] Background jobs
- [ ] Authentication
- [ ] Observability
- [ ] Docker
- [ ] Documentation

## Quran

- [ ] Quran edition model
- [ ] Canonical importer
- [ ] Corpus validation
- [ ] Quran address resolver
- [ ] Exact search
- [ ] Normalized search
- [ ] Concatenated phrase search
- [ ] Root index
- [ ] Lemma index
- [ ] Morphology
- [ ] Word families
- [ ] Frequency tools
- [ ] Distribution tools
- [ ] Quran graph
- [ ] Quran reader
- [ ] Citation validation

## Hadith and Tafsir

- [ ] Structured hadith model
- [ ] Al-Kafi adapter
- [ ] Bihar al-Anwar adapter
- [ ] Additional approved Shia collection adapters
- [ ] Hadith search
- [ ] Hadith grading model
- [ ] Isnad parser
- [ ] Narrator identity model
- [ ] Narrator graph
- [ ] Rijal integration
- [ ] Tafsir ingestion
- [ ] Tafsir comparison

## Comparative Sources

- [ ] Scripture edition model
- [ ] Torah adapters
- [ ] Bible adapters
- [ ] Passage comparison
- [ ] Parallel-passage research
- [ ] Comparative graph

## RAG and Models

- [ ] Document processing
- [ ] Embedding abstraction
- [ ] LLM abstraction
- [ ] Reranker abstraction
- [ ] Vector-store abstraction
- [ ] Graph-store abstraction
- [ ] Multi-RAG
- [ ] Hybrid retrieval
- [ ] Smart source router
- [ ] Model connection manager
- [ ] Capability discovery
- [ ] Model fallback

## Agents and Tools

- [ ] Conversation management
- [ ] Agent domain model
- [ ] Agent runtime
- [ ] Agency runtime
- [ ] Tool registry
- [ ] Tool SDK
- [ ] Tool policy engine
- [ ] Tool sandbox
- [ ] Tool generation workflow
- [ ] Human approval system
- [ ] MCP integration
- [ ] Agent memory
- [ ] Workflow engine
- [ ] Audit trail
- [ ] Evaluation

## Source Management

- [ ] Source catalog
- [ ] Manifest schema
- [ ] License tracking
- [ ] Catalog discovery
- [ ] Download quarantine
- [ ] Structural validation
- [ ] Version differences
- [ ] Approval workflow
- [ ] Rollback
- [ ] Scheduled update checks

---

# 50. Doctor and Corpus Diagnostics

Q-ai must provide a diagnostic command:

```bash
qai doctor
```

The command must diagnose:

- Application configuration
- Metadata database
- Full-text search index
- Vector store
- Graph store
- Quran corpus integrity
- Quran edition checksums
- Quran verse and token counts
- Quran morphology datasets
- Hadith collection integrity
- Tafsir collection integrity
- Comparative scripture integrity
- Citation resolver
- Source manifests
- Orphaned source versions
- Missing files
- Index consistency
- LLM connectivity
- Embedding connectivity
- Reranker connectivity
- Model capabilities
- Filesystem permissions
- Network connectivity
- Source-catalog connectivity
- Job-worker health
- Tool sandbox
- Required dependencies

Example:

```text
Q-ai Doctor

CORE
✓ Configuration valid
✓ SQLite database reachable
✓ Migration version current
✓ Background workers running

QURAN
✓ Canonical edition: hafs-uthmani-1.0.0
✓ Surahs: 114
✓ Verse structure valid
✓ Canonical checksum valid
✓ Token order valid
✓ Normalized search index current
✓ Root index current
✗ Morphology index differs from source version

HADITH
✓ Al-Kafi source files available
✓ Bihar al-Anwar source files available
✓ Hadith indexes reconciled
! 42 narrator identities awaiting review

MODELS
✓ Local embedding provider
✓ Ollama reachable
✗ Remote LLM connection unavailable

Suggested action:
Run `qai quran morphology reindex`.
```

Diagnostic commands must never modify canonical data automatically.

Repair actions must be shown separately and require confirmation when they may modify indexes, metadata, source activation, or user data.

Additional commands should include:

```bash
qai doctor --json
qai doctor --quran
qai doctor --sources
qai doctor --indexes
qai doctor --models
qai doctor --tools
qai doctor --repair-preview
```

---



# 51. Server Dashboard

The Web GUI must provide an operational and research dashboard.

Example:

```text
Q-AI SYSTEM

Status                    ● Running
Version                   0.3.0
Uptime                    2h 31m
Mode                      Local
Canonical Quran Edition   Hafs Uthmani 1.0.0

CORPORA

Quran Editions            1
Translations              8
Tafsir Works              12
Hadith Collections        7
Other Scriptures          5
Research Books            146

INDEXES

Quran Tokens              ...
Hadith Records            ...
Graph Nodes               ...
Graph Edges               ...
Vector Records            ...
Full-Text Documents       ...

RESEARCH

Active Conversations      4
Saved Workspaces          19
Active Runs               2
Awaiting Approvals        1

HEALTH

Corpus Integrity          ✓
Citation Resolver         ✓
Graph Store               ✓
Vector Store              ✓
Source Updates            3 available

PERFORMANCE

Requests                  1,294
Average Search Latency    84 ms
Average Research Latency  2.1 s
```

The dashboard must provide direct access to:

- Quran reader
- Quran search
- Root and word-family research
- Quran graph
- Hadith collections
- Narrators and isnads
- Tafsir
- Comparative scripture
- Ask Q-ai
- Research workspaces
- Source updates
- Jobs
- Runs
- Approvals
- Logs
- Metrics
- System health

---



# 52. Research Query Interface

The browser query interface must support multiple research modes.

```text
┌─────────────────────────────────────────────────────────────┐
│ Mode: [Smart Research ▼]                                   │
│ Sources: [Quran] [Hadith] [Tafsir] [Scripture] [Books]    │
│ Edition policy: [Approved editions only ▼]                 │
├─────────────────────────────────────────────────────────────┤
│ Ask a question or enter Arabic text...                     │
│                                                             │
│                                            [Research]       │
└─────────────────────────────────────────────────────────────┘
```

Supported modes:

- Quran reading
- Quran exact search
- Quran normalized search
- Root search
- Lemma search
- Word-family exploration
- Morphological analysis
- Quran graph search
- Hadith lookup
- Hadith research
- Isnad research
- Narrator research
- Tafsir comparison
- Comparative scripture
- General book research
- Smart automatic routing
- Advanced query builder

Results should display:

- Direct answer
- Canonical Quran quotations
- Hadith evidence
- Tafsir evidence
- Comparative passages
- Disagreements or alternative analyses
- Claim-level citations
- Retrieved source passages
- Graph paths
- Full-text retrieval scores
- Vector scores
- Reranking scores
- Selected tools
- Selected sources
- Edition and source versions
- Normalization rules
- Execution time
- Token usage
- Estimated cost
- Research trace

Users must be able to disable generative answers and request search results only.

---



# 53. Streaming

Q-ai must support streaming for:

- LLM responses
- Research progress
- Retrieval progress
- Tool calls
- Graph traversal progress
- Source imports
- Indexing jobs
- Corpus validation
- Evaluation runs
- Export generation

Example:

```text
Researching...

✓ Query classified: Quran linguistic research
✓ Exact Quran search completed
✓ Root search completed
✓ Word-family graph completed
✓ Tafsir retrieval completed
• Validating citations...
• Generating cited summary...
```

Streaming events must use normalized event schemas and must be available through:

- TUI
- Web GUI
- WebSocket or Server-Sent Events
- Internal event bus
- Structured logs where appropriate

Canonical quotations must be retrieved and validated before being streamed as final evidence.

Partial model output must not be mistaken for a validated final answer.

---



# 54. Cancellation

Long-running operations must be cancellable.

Users must be able to cancel:

- Research queries
- Agent runs
- Agency runs
- Workflow runs
- Tool calls where the tool supports cancellation
- Document imports
- Internet downloads
- Parsing
- Embedding generation
- Graph construction
- Full-text indexing
- Re-indexing
- Corpus validation
- Evaluation
- Exports

Cancellation must propagate through the pipeline.

A cancelled operation must:

- Stop new work from being scheduled.
- Attempt to interrupt active cancellable work.
- Preserve valid completed stages.
- Remove or quarantine incomplete output.
- Avoid activating partial indexes.
- Record the cancellation in the run trace.
- Leave canonical source versions unchanged.
- support safe resumption where applicable.

---



# 55. Architecture Principles

The following principles are mandatory:

1. Separation of concerns
2. Dependency inversion
3. Explicit interfaces
4. Strong typing
5. Async-first I/O
6. Testability
7. Provider abstraction
8. Storage abstraction
9. UI independence
10. Configuration-driven behavior
11. Local-first operation
12. Observability
13. Secure defaults
14. Canonical text immutability
15. Exact source provenance
16. Edition-aware retrieval
17. Structure-aware ingestion
18. Quran tools before generic RAG when appropriate
19. Graph and lexical search as first-class retrieval methods
20. Generated analysis must be distinguishable from source text
21. Scholarly disagreement must remain attributable
22. Deterministic operations must not depend unnecessarily on an LLM
23. Search normalization must not alter display text
24. Every citation must resolve to an exact source location
25. Imported internet content is untrusted until approved
26. Corpus updates are versioned and reversible
27. Tools use deny-by-default permissions
28. Authorization is enforced outside the model
29. Every run has explicit resource limits
30. Every agent action is attributable and auditable
31. Local data is not sent remotely without explicit configuration
32. Security boundaries must never depend solely on prompts

---



# 56. Quality Requirements

## Correctness

- Canonical Quran text must match the approved source version exactly.
- Deterministic searches must be reproducible.
- Citations must resolve correctly.
- Hadith metadata must retain collection and edition identity.
- Generated analysis must never overwrite source data.

## Performance

- Exact Quran lookup should not require an LLM.
- Search indexes must support interactive response times.
- Graph queries must be bounded.
- CPU-intensive work must not block the async runtime.
- Bulk ingestion must use configurable worker pools.

## Reliability

- Provider failures must not crash Q-ai.
- Partial imports must not become active.
- Indexes must be reconstructable.
- Interrupted jobs must be detectable.
- Source updates must support rollback.

## Maintainability

- Core functionality must remain independent from interfaces.
- Corpus-specific logic must live in dedicated modules.
- Provider adapters must not leak into domain models.
- All important schemas must be versioned.

## Extensibility

Adding a new:

- Quran edition
- Translation
- Tafsir
- Hadith collection
- Scripture edition
- Model provider
- Retriever
- Vector store
- Graph store
- Source catalog
- Tool

must normally require an adapter or manifest, not changes throughout the core.

## User Experience

Users must be able to:

- Read the Quran comfortably.
- Understand what type of content they are viewing.
- Inspect sources without technical knowledge.
- Distinguish source text from AI output.
- Inspect advanced execution details when desired.

---



# 57. Future Features

Potential future roadmap:

- Additional Quran editions and recognized recitation metadata
- More detailed orthographic analysis
- Advanced Arabic syntax graphs
- Quran dependency trees
- Rhetorical structure research
- Audio recitation synchronization
- Tajwid visualization
- Pronunciation tools
- Handwritten manuscript research
- OCR for historical Arabic and Persian books
- Scan-to-text alignment
- Page-image citation overlays
- Advanced isnad comparison
- Narrator identity review collaboration
- Historical maps
- Entity timelines
- Visual research canvases
- Visual RAG pipeline editor
- Visual workflow editor
- Graph-based research notebooks
- Collaborative annotation
- Research peer review
- Dataset publishing
- Plugin ecosystem
- Tool marketplace with signed packages
- Distributed workers
- GPU acceleration
- Cluster mode
- Cloud deployment
- Automated RAG benchmarking
- AI-assisted retrieval configuration
- Multilingual query translation
- Speech input
- Recitation-based Quran navigation

Future additions must preserve canonical integrity, provenance, attribution, and user control.

---



# 58. Definition of Done

A feature is complete only when:

- It is implemented.
- It has appropriate unit tests.
- It has integration tests where required.
- It has typed error handling.
- It is observable.
- It is documented.
- It works through intended interfaces.
- It does not unnecessarily couple layers.
- Configuration is validated.
- Cancellation works where applicable.
- Timeouts are enforced.
- Sensitive information is redacted.
- Provenance is recorded.
- Version compatibility is handled.
- Access controls are enforced.
- Failure recovery is tested.
- `cargo fmt` succeeds.
- `cargo clippy` succeeds.
- `cargo test` succeeds.

Corpus-related features additionally require:

- Source-version tracking
- Checksum validation
- Round-trip tests
- Citation resolution tests
- No silent canonical changes
- Rebuild tests for derived indexes
- Difference reports for updates

---



# 59. Updated Product Scope

Q-ai supports five first-class workloads:

1. Canonical Quran reading and research
2. Shia hadith, isnad, rijal, and tafsir research
3. Comparative scripture and research-book retrieval
4. Agentic research
5. Tool and workflow creation

Unified runtime:

```text
User
  ↓
CLI / TUI / Web GUI / API
  ↓
Conversation and Research Runtime
  ├── Quran Reader
  ├── Quran Search
  ├── Quran Graph
  ├── Hadith Research
  ├── Tafsir Research
  ├── Comparative Research
  ├── Multi-RAG
  ├── Single Agent
  ├── Research Agency
  └── Workflow
          ↓
       Tool Runtime
          ├── Quran Tools
          ├── Hadith Tools
          ├── Graph Tools
          ├── RAG Tools
          ├── Source Tools
          ├── Generated Tools
          ├── MCP Tools
          └── Remote API Tools
```

Definitions must be versioned and portable across all interfaces.

---



# 60. TUI Model Connection Manager

The TUI must provide complete connection management.

Supported initial providers:

- OpenAI-compatible APIs
- OpenAI
- Anthropic
- Ollama
- Local model servers
- Custom HTTP providers

Potential future providers:

- Google Gemini
- Azure-hosted models
- AWS Bedrock
- llama.cpp servers
- vLLM
- Additional provider plugins

Each profile should include:

```text
id
name
provider
base_url
api_key_reference
organization
project
default_model
timeout
connect_timeout
max_retries
retry_backoff
requests_per_minute
tokens_per_minute
proxy
tls_settings
privacy_policy
allowed_data_classes
enabled
created_at
updated_at
```

TUI screens:

```text
Models
├── Connections
│   ├── List
│   ├── Create
│   ├── Edit
│   ├── Delete
│   ├── Test
│   └── Inspect Capabilities
├── Available Models
├── Model Routing
├── Usage
└── Health
```

Connection testing must check:

- Connectivity
- Authentication
- Model availability
- Streaming
- Tool calling
- Parallel tool calls
- Structured output
- Context window
- Embedding dimensions
- Arabic handling where testable
- Latency

API keys must never appear in output or logs.

---



# 61. Unified Model Capability Interface

Provider adapters must expose capabilities.

```rust
pub struct ModelCapabilities {
    pub chat: bool,
    pub streaming: bool,
    pub tool_calling: bool,
    pub parallel_tool_calls: bool,
    pub structured_output: bool,
    pub vision: bool,
    pub embeddings: bool,
    pub reranking: bool,
    pub max_context_tokens: Option<u64>,
    pub max_output_tokens: Option<u64>,
    pub supported_languages: Vec<String>,
}

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;

    async fn capabilities(
        &self,
        model: &str,
    ) -> Result<ModelCapabilities>;

    async fn chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse>;

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<ChatEventStream>;
}
```

Provider-specific options may be retained through namespaced extension fields.

Capabilities must be discovered or configured explicitly, not guessed by the agent.

---



# 62. Model Routing and Fallback

Suggested aliases:

```text
fast
balanced
reasoning
arabic
persian
research
local
private
embedding-default
reranker-default
```

Routing may consider:

- Required capabilities
- Arabic and Persian quality
- Tool calling
- Structured output
- Context length
- Privacy
- Local-only mode
- Data classification
- Cost
- Latency
- Provider health
- Rate limits
- Task type
- User preference

Fallback may occur only when:

- The fallback satisfies required capabilities.
- The data-handling policy allows it.
- The fallback does not send local data to a prohibited provider.
- The run budget allows it.

The trace must record which model was selected and why.

---



# 63. Agent Definition

Agents must use portable, versioned definitions.

```toml
[agent]
name = "quran-researcher"
version = "1.0.0"
description = "Performs Quran linguistic and graph research"
model = "research"
max_steps = 12
max_runtime_seconds = 180
max_tool_calls = 20
max_retrieval_calls = 15
max_input_tokens = 50000
max_output_tokens = 8000

[knowledge]
sources = ["quran-canonical", "approved-tafsir"]

[tools]
allow = [
  "quran.get_ayah",
  "quran.search_exact",
  "quran.search_normalized",
  "quran.root_search",
  "quran.word_family",
  "quran.graph_path"
]

[permissions]
network = []
filesystem_read = []
filesystem_write = []
command_execution = false
canonical_write = false
```

Agent definitions must include:

- Stable ID
- Name
- Description
- Version
- Model policy
- Instructions
- Authorized knowledge sources
- Tool allowlist and denylist
- Memory policy
- Execution limits
- Permission policy
- Approval policy
- Output schema
- Tags
- Owner
- Timestamps

Configuration changes create a new revision.

---



# 64. Agent Runtime

Agents must execute as observable and cancellable state machines.

```text
Created
  ↓
Planning
  ↓
Model Request
  ↓
Tool Requested
  ↓
Policy Evaluation
  ├── Denied
  ├── Approval Required
  └── Allowed
          ↓
     Tool Execution
          ↓
     Observation
          ↓
     Citation Validation
          ↓
     Continue or Finish
          ↓
Completed / Failed / Cancelled / Timed Out
```

Runtime requirements:

- Streaming
- Sequential and parallel tool calls
- Cancellation
- Deadlines
- Step limits
- Tool-call limits
- Retrieval limits
- Token and cost budgets
- Retry policies
- Structured-output validation
- Human approval
- Checkpointing
- Replay
- Failure recovery
- Complete traces
- Source and edition recording

Every run receives a unique run ID.

---



# 65. Agent Run Events

Normalized events include:

```text
run.started
run.completed
run.failed
run.cancelled
run.timed_out
model.requested
model.response_delta
model.completed
tool.requested
tool.approval_required
tool.approved
tool.denied
tool.started
tool.progress
tool.completed
tool.failed
retrieval.started
retrieval.completed
quran.lookup_completed
graph.query_started
graph.query_completed
citation.validation_started
citation.validated
citation.rejected
agent.delegated
agent.message
budget.warning
budget.exceeded
```

Events must be available through:

- Internal event bus
- TUI
- Web GUI
- WebSocket or SSE
- Structured logs
- Audit records where appropriate

Sensitive content must be redacted according to policy.

---



# 66. Multi-Agent Research Agencies

Supported roles:

- Supervisor
- Router
- Planner
- Quran linguistics researcher
- Quran graph researcher
- Hadith researcher
- Isnad researcher
- Tafsir researcher
- Comparative scripture researcher
- Citation verifier
- Safety reviewer
- Final-answer synthesizer
- Tool builder
- General reviewer

Coordination strategies:

- Supervisor and workers
- Sequential handoff
- Parallel delegation
- Router-based delegation
- Review and revise

Future strategies:

- Debate
- Blackboard collaboration
- Hierarchical teams
- Consensus execution
- Dynamic specialized workers within strict policies

Agencies must define:

- Entry agent
- Members
- Delegation rules
- Shared and private memory
- Global budget
- Duration
- Communication policy
- Completion policy
- Failure policy
- Approval gates
- Maximum delegation depth

---



# 67. Tool System

Initial built-in tools should include:

```text
quran.get_ayah
quran.get_context
quran.search_exact
quran.search_normalized
quran.search_concatenated
quran.search_phrase
quran.root_search
quran.lemma_search
quran.word_family
quran.morphology
quran.frequency
quran.cooccurrence
quran.graph_neighbors
quran.graph_path
quran.graph_subgraph

hadith.get
hadith.search_exact
hadith.search_normalized
hadith.search_semantic
hadith.search_by_narrator
hadith.get_gradings
hadith.compare_variants
hadith.isnad_graph
hadith.narrator_profile

tafsir.search
tafsir.compare
scripture.get_passage
scripture.search
scripture.compare
rag.search
rag.query
document.read
document.list
source.inspect
graph.query
json.transform
database.query_readonly
agent.delegate
workflow.start
```

Tool manifests must include:

```text
id
name
description
version
input_schema
output_schema
permissions
timeout
resource_limits
side_effect_class
network_policy
filesystem_policy
source_policy
execution_runtime
publisher
signature
checksum
status
```

Write and command tools must be optional and disabled by default.

---



# 68. Agent-Created Tools

Lifecycle:

```text
Request
  ↓
Requirements and Schemas
  ↓
Source Generation
  ↓
Static Validation
  ↓
Dependency and License Check
  ↓
Security Scan
  ↓
Build
  ↓
Sandbox Tests
  ↓
Corpus Accuracy Tests
  ↓
Human Review
  ↓
Approval and Signing
  ↓
Publication
  ↓
Explicit Agent Assignment
```

Tool states:

```text
Draft
Generated
Validating
BuildFailed
TestFailed
AwaitingApproval
Approved
Published
Disabled
Revoked
Archived
```

Generated tools must:

- Declare all capabilities.
- Have typed schemas.
- Have timeouts and limits.
- Run tests.
- Be reviewed.
- Be checksummed.
- Be immutable after publication.
- Record generation provenance.
- Be revocable.
- Be disabled for existing agents by default.

They must not:

- Modify canonical Quran text.
- Modify approved source text.
- Invent source references.
- Access arbitrary files or hosts.
- Access secrets without permission.
- Execute unrestricted commands.
- Install undeclared dependencies.
- create other tools recursively without approval.

---



# 69. Tool Sandbox

Untrusted tools must execute outside the main application process.

Preferred initial sandbox:

- WebAssembly/WASI
- Capability-based filesystem access
- Network allowlists
- CPU limits
- Memory limits
- Wall-clock timeout
- Output-size limits
- No inherited environment
- Explicit secret injection
- Temporary isolated workspace

Example:

```toml
[sandbox]
memory_mb = 128
cpu_time_ms = 5000
wall_time_ms = 10000
max_output_bytes = 1048576
network_enabled = false
filesystem_read = []
filesystem_write = []
environment = []
canonical_store_write = false
```

Q-ai must fail closed when a sandbox policy cannot be enforced.

---



# 70. Tool Permissions and Human Approval

Before every tool call, evaluate:

```text
Agent permissions
+ Tool requirements
+ User permissions
+ Workspace policy
+ Source access policy
+ Run-specific approval
```

Possible outcomes:

- Allow
- Deny
- Require approval

Approval UI must display:

- Requesting agent
- Tool name and version
- Purpose
- Input arguments
- Expected side effects
- Requested files
- Requested hosts
- Requested secrets
- Affected sources
- Timeout
- Risk level

Approval choices:

- Deny
- Allow once
- Allow for this run
- Allow for this agent and tool version
- Edit arguments and allow

Canonical source modification must not be available as a normal agent permission.

---



# 71. MCP Integration

MCP support should include:

- Local stdio servers
- Supported remote transports
- Tool discovery
- Resource discovery
- Prompt discovery
- Capability negotiation
- Connection health
- Authentication
- Per-server permissions
- Name-collision handling
- Timeouts
- Audit logs

MCP tools must pass through the same:

- Schema validation
- Permission engine
- Approval process
- Source access controls
- Budgets
- Audit system

A locally installed MCP server is not automatically trusted.

---



# 72. Agent Memory

Memory types:

## Conversation Memory

Messages in the current conversation.

## Working Memory

Temporary run state and evidence.

## Episodic Memory

Summaries of previous runs.

## Semantic Memory

Long-term retrievable user-approved information.

## Artifact Memory

Files, reports, graphs, and structured outputs.

Controls:

- Enable or disable each type
- Retention period
- Maximum size
- User deletion
- Export
- Workspace isolation
- Sensitive-data classification
- Retrieval filters
- Provenance
- Summarization policy

Agents must not silently promote temporary research or religious conclusions into long-term memory.

Canonical corpora are knowledge sources, not agent memory.

---



# 73. Agent, Quran, Graph, and RAG Integration

Agents may access research through:

1. Pipeline-integrated retrieval
2. Explicit typed tools

Agents should be able to:

- Select authorized corpora
- Use exact Quran retrieval
- Use root and morphology tools
- Search hadith collections
- Inspect isnads
- Compare tafsir
- Traverse graphs
- Search multiple knowledge bases
- Apply source and edition filters
- Inspect citations
- Request another retrieval pass
- Compare retrieval systems

Every retrieved unit must retain:

```text
corpus_id
source_id
edition_id
document_id
passage_id
chunk_id
canonical_reference
source_location
retrieval_score
reranking_score
access_control
content_hash
ingestion_version
```

---



# 74. Prompt-Injection Defense

Retrieved books, web pages, source manifests, tool output, and imported documents are untrusted data.

The runtime must distinguish:

```text
System instructions
Application policy
Workspace policy
User instructions
Tool instructions
Retrieved content
Tool output
Model output
```

Protections:

- Instruction/data boundaries
- Content provenance
- Tool allowlists
- Retrieval access controls
- Suspicious-instruction detection
- Secret redaction
- Context limits
- HTML and script sanitization
- Indirect prompt-injection defenses
- Confirmation before side effects
- Deterministic authorization outside the model
- Source-manifest schema validation

Detection may support security but cannot replace authorization.

---



# 75. Access-Control-Aware Retrieval

All retrieval must apply authorization before returning results.

Prevent:

- Cross-user leakage
- Cross-workspace leakage
- Unauthorized corpus access
- Restricted-book leakage
- Unauthorized citation access
- Cached-result leakage
- Agent access beyond the initiating user
- Export of restricted source content

Authorization must be enforced in:

- Application layer
- Metadata store
- Full-text store
- Vector store
- Graph store
- Citation resolver
- Cache layer

Local single-user mode may use a simplified policy while preserving the same data model.

---



# 76. Data Lifecycle and Index Consistency

Ingestion must be idempotent and version-aware.

Requirements:

- Content-hash duplicate detection
- Source revision tracking
- Incremental re-indexing
- Embedding-model version tracking
- Chunker-version tracking
- Parser-version tracking
- Normalizer-version tracking
- Graph-builder-version tracking
- Tombstones
- Vector deletion verification
- Retry-safe jobs
- Failed-index recovery
- Cross-store reconciliation
- Canonical source pinning

Deleting or deactivating a document version must address:

- Source records
- Parsed content
- Chunks
- Embeddings
- Full-text records
- Graph nodes and edges
- Caches
- Previews

Canonical versions should normally be deprecated or deactivated rather than destructively deleted.

---



# 77. Conversation and Session Management

A conversation includes:

```text
id
workspace_id
user_id
title
mode
selected_model
selected_sources
selected_rag
selected_agent
selected_agency
messages
artifacts
run_ids
created_at
updated_at
retention_policy
```

Modes:

- Quran search
- Quran research
- Hadith research
- Tafsir comparison
- Comparative scripture
- Direct model chat
- RAG chat
- Agent chat
- Agency task
- Workflow run

Users can:

- Create
- Resume
- Fork
- Export
- Delete
- Retry a response
- Change models
- Change sources
- Inspect tool calls
- Inspect graph paths
- Inspect citations
- Inspect token and cost usage

---



# 78. TUI Agent Experience

Additional screens:

```text
Agents
├── List
├── Create
├── Edit
├── Clone
├── Run
├── Permissions
├── Knowledge
├── Tools
├── Memory
└── Versions

Agencies
├── List
├── Create
├── Members
├── Routing
├── Budgets
└── Run

Tools
├── Installed
├── Quran Tools
├── Hadith Tools
├── Create
├── Generated Drafts
├── Validate
├── Test
├── Approvals
├── Permissions
├── Versions
└── Revoke

Runs
├── Active
├── Awaiting Approval
├── Completed
├── Failed
└── Trace
```

Keyboard actions:

- Cancel run
- Approve or deny
- Expand tool arguments
- Inspect retrieved context
- Inspect graph path
- Inspect execution trace
- Switch model
- Switch corpus
- Switch agent
- Fork conversation
- Copy response
- Open citation

---



# 79. Tool Builder UI

Steps:

```text
1. Describe the tool
2. Define inputs and outputs
3. Select implementation type
4. Select source access
5. Select permissions
6. Generate implementation
7. Review source
8. Validate
9. Run tests
10. Run corpus-accuracy tests
11. Review security report
12. Approve and publish
13. Assign to agents
```

Publication must be blocked if:

- Schema is invalid.
- Compilation fails.
- Tests fail.
- Permissions are undeclared.
- Sandbox enforcement fails.
- A critical security issue remains.
- Corpus-integrity tests fail.
- The tool can alter canonical data.
- The approver lacks permission.

The UI must show differences between tool revisions.

---



# 80. Workflow Engine

Initial nodes:

- Model call
- Quran tool
- Hadith tool
- Graph query
- RAG retrieval
- Agent invocation
- Tool invocation
- Condition
- Parallel branch
- Human approval
- Data transformation
- Citation validation
- Final output

Requirements:

- Typed inputs and outputs
- Versioning
- Checkpoints
- Retries
- Cancellation
- Timeouts
- Approval gates
- Run traces
- Partial-failure policies
- Source and edition pinning

Cycles require an explicit maximum iteration count.

---



# 81. Budgets and Resource Governance

Budget types:

- Maximum input tokens
- Maximum output tokens
- Maximum total tokens
- Maximum estimated cost
- Maximum tool calls
- Maximum model calls
- Maximum retrieval calls
- Maximum graph expansions
- Maximum returned passages
- Maximum runtime
- Maximum artifacts
- Maximum delegation depth
- Maximum download size
- Maximum indexing resources

Hard limits stop safely.

Soft limits produce warnings.

Cost values must be labeled as estimates unless provider billing confirms them.

---



# 82. Audit Trail and Provenance

Audited actions include:

- Provider connections
- Secret-reference changes
- Agent changes
- Permission changes
- Tool generation and publication
- Sensitive tool approvals
- Source discovery
- Source download
- Source validation
- Source activation
- Source rollback
- Quran edition changes
- Hadith imports
- Document deletion
- User and role changes

Generated outputs should record:

```text
run_id
agent_version
agency_version
workflow_version
model_provider
model_name
prompt_version
tool_versions
rag_pipeline_version
source_versions
edition_versions
normalization_version
graph_version
timestamps
token_usage
estimated_cost
```

Audit logs must not store plaintext secrets.

---



# 83. API Additions

Add endpoints such as:

```text
GET    /api/v1/connections
POST   /api/v1/connections
POST   /api/v1/connections/:id/test

GET    /api/v1/agents
POST   /api/v1/agents
POST   /api/v1/agents/:id/runs

GET    /api/v1/agencies
POST   /api/v1/agencies
POST   /api/v1/agencies/:id/runs

GET    /api/v1/tools
POST   /api/v1/tools
POST   /api/v1/tools/generate
POST   /api/v1/tools/:id/validate
POST   /api/v1/tools/:id/test
POST   /api/v1/tools/:id/approve
POST   /api/v1/tools/:id/publish
POST   /api/v1/tools/:id/revoke

GET    /api/v1/runs
GET    /api/v1/runs/:id
POST   /api/v1/runs/:id/cancel
GET    /api/v1/runs/:id/events

GET    /api/v1/approvals
POST   /api/v1/approvals/:id/approve
POST   /api/v1/approvals/:id/deny

GET    /api/v1/conversations
POST   /api/v1/conversations
GET    /api/v1/conversations/:id
POST   /api/v1/conversations/:id/messages
DELETE /api/v1/conversations/:id
```

Run creation, publication, source activation, updates, rollback, and external side effects require idempotency keys where applicable.

---



# 84. Additional Security Requirements

- Bind to localhost by default.
- Require explicit configuration for remote access.
- Require TLS in production.
- Require authentication outside localhost.
- Use deny-by-default tool execution.
- Protect HTTP tools and importers against SSRF.
- Prevent filesystem traversal and symlink escapes.
- Prevent archive extraction attacks.
- Use read-only database credentials by default.
- Disable shell execution by default.
- Scope secrets to tools and runs.
- Block or approve model inputs containing secrets.
- Apply rate and size limits.
- Treat uploaded content as untrusted.
- Scan dependencies and generated tools.
- Protect audit records.
- Require human approval for corpus activation.
- Prevent agents from editing canonical sources.
- Verify source hashes before activation.
- Keep update downloads in quarantine.
- Apply licensing controls before publishing or exporting content.

---



# 85. Reliability Requirements

Provider and tool calls must use:

- Timeouts
- Bounded retries
- Exponential backoff
- Jitter
- Cancellation
- Concurrency limits
- Rate-limit handling
- Circuit breakers where appropriate

Non-idempotent side effects must not be retried without idempotency support.

After crashes, Q-ai must identify interrupted runs and:

- Resume from a safe checkpoint,
- Mark them interrupted, or
- Request user action.

Source activation must be atomic.

A partially built Quran, hadith, graph, full-text, or vector index must never replace the active index.

---



# 86. Evaluation Requirements

## Quran

- Canonical integrity
- Exact lookup
- Normalized search recall
- Concatenated search accuracy
- Root and lemma accuracy
- Morphology accuracy
- Word-family quality
- Graph correctness
- Citation correctness

## Hadith

- Record lookup
- Source-number resolution
- Isnad segmentation
- Narrator identity precision
- Grading attribution
- Variant comparison
- Citation correctness

## RAG

- Retrieval recall
- Retrieval precision
- Citation support
- Faithfulness
- Answer relevance
- Access-control leakage
- Prompt-injection resistance
- Disagreement preservation

## Agents

- Task success
- Step count
- Tool-selection accuracy
- Invalid call rate
- Budget compliance
- Policy compliance
- Human intervention
- Tool-failure recovery
- Citation hallucination rate

## Tools

- Schema compliance
- Correct output
- Error behavior
- Timeout behavior
- Permission enforcement
- Sandbox resistance
- Determinism where expected
- Corpus-integrity safety

## Agencies

- Delegation accuracy
- Duplicate work
- Completion rate
- Latency
- Token usage
- Cost
- Final-answer quality

Datasets and results must be versioned.

---



# 87. Updated Project Structure

```text
crates/
├── domain/
├── application/
├── quran-core/
├── quran-corpus/
├── quran-normalization/
├── quran-morphology/
├── quran-search/
├── quran-graph/
├── hadith-core/
├── hadith-ingestion/
├── isnad-graph/
├── tafsir/
├── scripture/
├── sources/
├── ingestion/
├── retrieval/
├── rag/
├── graph/
├── embeddings/
├── reranking/
├── llm/
├── model-router/
├── agent-domain/
├── agent-runtime/
├── agency/
├── conversations/
├── tools/
├── tool-sdk/
├── tool-registry/
├── tool-sandbox/
├── policy/
├── approvals/
├── workflows/
├── memory/
├── mcp/
├── citations/
├── provenance/
├── audit/
├── evaluation/
├── storage/
├── jobs/
├── api/
├── server/
├── tui/
├── cli/
└── config/
```

Domain crates must not depend on the TUI, Web GUI, or concrete providers.

---



# 88. Updated Development Phases

The authoritative implementation order is:

1. Foundations and provenance
2. Canonical Quran corpus
3. Quran search and normalization
4. Quran morphology and word families
5. Quran graph
6. Rich Quran Web GUI
7. Shia hadith and tafsir
8. Isnad, narrator, and rijal graph
9. Multi-RAG and smart routing
10. Comparative scriptures and books
11. Agent runtime and typed tools
12. Safe tool creation and workflows
13. Source catalogs and internet updates
14. Server, authentication, and RBAC
15. Production hardening

Each phase must include tests, documentation, migrations, observability, and acceptance criteria.

---



# 89. Revised MVP

## Includes

- Local-first Q-ai
- Web GUI
- CLI
- Basic TUI
- SQLite
- Local full-text search
- One local vector-store option
- One approved Quran edition
- Quran corpus validation
- Exact lookup
- Normalized Arabic search
- Concatenated search
- Root and lemma search
- Morphology
- Word families
- Basic Quran graph
- Attributed translations
- Streaming cited research chat
- OpenAI-compatible provider
- Ollama
- One agent per run
- Read-only Quran tools
- Tool-call inspection
- Approval framework
- Cancellation
- Traces
- Connection diagnostics
- Environment-variable or OS-keychain secrets

## Excludes

- Unrestricted generated native code
- Automatic publication
- Arbitrary shell execution
- Unrestricted autonomous agents
- Distributed workers
- Plugin marketplace
- Full collaboration
- Autonomous source activation
- Unreviewed canonical changes
- Automatic spending without limits

---



# 90. Tool-Creation Release Acceptance Criteria

Tool creation is complete only when a user can:

1. Describe a tool.
2. Review schemas.
3. Review source requirements.
4. Review permissions.
5. Generate implementation.
6. Build without modifying the host.
7. Test in isolation.
8. Run corpus-accuracy tests.
9. Inspect security results.
10. Approve or reject publication.
11. Assign an approved version.
12. Execute in the sandbox.
13. Inspect inputs, outputs, logs, duration, and resources.
14. Disable or revoke it immediately.

Unpublished tools must not be available to production agents.

---



# 91. Additional Open Technical Decisions

Additional ADRs are required for:

- In-process TUI versus API-only TUI
- Streaming transport
- WASM runtime
- MCP implementation
- Agent state machine
- Workflow representation
- Policy engine
- Tool signing
- Source-manifest signing
- Secret storage
- Local vector store
- Arabic full-text tokenization
- Durable job system
- Conversation retention
- Audit integrity
- Graph query safety limits
- Corpus update atomicity
- Page-scan alignment
- Citation quotation verification
- Linguistic annotation confidence model

Each decision must document context, options, security, operations, migration, and reversal cost.

---



# 92. Updated Architecture Principles

Additional mandatory principles:

33. Deny-by-default tool permissions  
34. Human control over external side effects  
35. Generated code is untrusted  
36. Retrieved content is untrusted  
37. Authorization is enforced outside models  
38. Every action is attributable  
39. Every run has resource limits  
40. Tool and agent definitions are versioned  
41. Provider capabilities are discovered  
42. Side effects are idempotent where possible  
43. Canonical source changes require explicit approval  
44. Canonical and derived data are stored separately  
45. Search indexes are reproducible from approved sources  
46. AI output cannot become canonical data automatically  
47. Every scholarly annotation retains attribution  
48. Uncertain narrator identities remain explicitly uncertain  
49. Cross-scripture parallels distinguish sourced and computed relationships  
50. Exact retrieval takes precedence over model recollection  
51. Citation validation occurs before final publication  
52. Corpus licensing is a first-class product constraint  

---



# 93. Updated Definition of Done

An agent, tool, corpus, or retrieval feature is complete only when:

- Permission checks exist.
- Cancellation works.
- Timeouts work.
- Resource limits work.
- Schemas are validated.
- Sensitive values are redacted.
- Audit events are generated.
- Streaming works where applicable.
- Retry and failure behavior are tested.
- Prompt-injection scenarios are tested.
- Unauthorized access is tested.
- Side effects are documented.
- Model, prompt, tool, source, and edition versions are recorded.
- Generated code cannot bypass the sandbox.
- Users can inspect what happened and why.
- Citation links resolve.
- Source quotations match.
- Corpus updates are reversible.
- Derived indexes can be rebuilt.
- Canonical sources cannot be silently modified.

---



# 94. Living PRD Rules
This document is the authoritative product specification for Q-ai.

Whenever requirements change:

1. Update this PRD.
2. Increment the version.
3. Preserve completed requirements.
4. Mark obsolete requirements explicitly.
5. Do not silently remove security or provenance requirements.
6. Keep the roadmap synchronized with implementation.
7. Record major technical decisions as ADRs.
8. Record source and corpus decisions separately from software decisions.
9. Update acceptance criteria when scope changes.
10. Return the complete updated PRD when a full replacement is requested.

---


This document is the authoritative Q-ai product specification.

Whenever project requirements change:

1. Update this PRD.
2. Increment the version when appropriate.
3. Preserve completed requirements.
4. Mark obsolete requirements explicitly.
5. Keep architecture decisions documented.
6. Keep source and licensing decisions documented.
7. Keep the roadmap synchronized with implementation.
8. Do not silently remove previously agreed requirements.
9. Do not silently broaden the supported religious corpus.
10. Do not weaken corpus integrity or citation requirements.
11. Record major changes in the change log.
12. Return a complete consolidated PRD when replacement is requested.

Version 0.3.0 supersedes Version 0.2.0 after all sections are merged and reviewed.

---

# 95. Change Log

## Version 0.3.2

Added:

- Detailed TUI screen inventory and required terminal capabilities (section 25.1)
- Command palette requirements (section 25.2)
- RAG debugging view requirements (section 25.3)
- Full CLI command tree and conventions (section 25.4)
- Generic document processing pipeline and formats (section 25.5)
- Chunking system strategies and boundaries (section 25.6)
- RAG projects structure (section 25.7)
- Declarative RAG configuration schema (section 25.8)
- Multiple RAG composition strategies (section 25.9)
- Generic import sources list (section 25.10)
- Generic Web GUI navigation, RAG query UI, and server dashboard (section 25.11)
- Generic storage architecture for documents and vectors (section 25.12)
- Performance, error handling, secrets, config, observability, security, testing, developer-experience, plugin, Docker, and local-first engineering baseline (section 25.13)
- Generic platform non-goals (section 25.14)
- Generic open technical decisions (section 25.15)

These restore generic Rust Multi-RAG platform engineering requirements from Version 0.2.0 that were under-specified in Version 0.3.0.

## Version 0.3.1

Added:

- Interpretive-conflict presentation requirements (side-by-side attributed views)
- Semi-automated graph annotation and human review workflow (section 10.6)
- Source genealogy and derivation lineage in the source catalog (section 22.6)
- Risk-based tool execution tiers and the `ComputationalAnnotation` side-effect class
- Research checksums for reproducible research runs (section 12.1)

Fixed:

- section 14 subsection numbering (Grading Model renumbered, Hadith Tools now 14.5)

## Version 0.3.0

Renamed and refocused the project as **Q-ai**.

Added:

- Quran-first product vision
- Canonical Quran corpus engine
- Immutable and versioned Quran editions
- Arabic normalization
- Diacritic-free search
- Concatenated-word and phrase search
- Root, lemma, stem, and word-family research
- Quran morphology comparison
- Frequency, distribution, and co-occurrence tools
- Quran knowledge graph
- Graph search and graph visualization
- Quran rhetorical and structural discovery tools
- Rich Quran reading interface
- Structured Shia hadith support
- Al-Kafi and Bihar al-Anwar requirements
- Isnad and narrator graph
- Attributed hadith grading
- Tafsir comparison
- Torah and Bible support
- Comparative scripture tools
- Structure-aware ingestion and chunking
- Smart RAG and tool selection
- Claim-level citation validation
- Source trust levels
- Internet book catalog discovery and updates
- Signed/checksummed source manifests
- Source staging, approval, rollback, and licensing
- Research workspaces
- Quran-specific APIs and CLI commands
- Quran-first MVP and acceptance criteria
- Religious-source integrity and disagreement-handling requirements

Preserved and integrated from Version 0.2.0:

- Rust architecture
- Local and server modes
- CLI, TUI, Web GUI, and API
- Multi-RAG
- LLM and embedding provider abstractions
- Vector stores
- Background jobs
- Agent runtime
- Multi-agent agencies
- Tool calling
- Tool sandbox
- Human approvals
- MCP
- Workflows
- Observability
- Security
- Authentication
- Access-control-aware retrieval
- Audit trails
- Versioning
- Evaluation
- Docker and production deployment
- Local-first operation
## Versioning clarification

The updated Q-ai PRD should **replace Version 0.2.0**, not be maintained beside it.

However, my previous response should **not yet be treated as the final replacement**, because:

- It stopped at section 89.
- It did not fully preserve Sections 50–101 from Version 0.2.0.
- It included collections outside your requested scope.

The correct process is:

1. Use Version 0.2.0 as the historical baseline.
2. Merge Sections 1–49 from the Q-ai rewrite.
3. Remove all unwanted collections and related requirements from Sections 1–49.
4. Replace the previously generated Sections 50–51 with Sections 50–101 below.
5. Publish the result as **Q-ai PRD Version 0.3.0**, which completely supersedes Version 0.2.0.
6. Keep Version 0.2.0 only in version history or Git—not as an active companion specification.

The supported Islamic hadith scope should be **Shia collections only**, while the Quran, tafsir, Torah, Bible, and configurable comparative or research books remain supported.

---



# 96. Terminology

## Canonical Text

The exact source text belonging to a specific approved edition and version.

## Edition

A separately identified publication or dataset with its own text, numbering, publisher, license, and version.

## Quran Tool

A deterministic or computational operation designed for Quran structure, text, language, or graph research.

## Model

An LLM, embedding model, reranker, morphology model, OCR model, speech model, or other inference model.

## RAG

A retrieval pipeline that supplies external source material to a model. The canonical Quran engine is richer than and independent from generic RAG.

## Graph

A typed collection of nodes and edges with provenance, confidence, and source-version information.

## Agent

An AI runtime configured with:

- Models
- Instructions
- Tools
- Knowledge sources
- Memory
- Permissions
- Budgets
- Output requirements

## Agency

A group of collaborating agents with defined roles, routing, permissions, and budgets.

## Tool

A typed capability that an agent, workflow, UI, CLI, or API can invoke.

## Tool Creation

Generating, validating, testing, approving, signing, and publishing a tool.

## Workflow

A versioned deterministic or agent-driven graph of steps, tools, models, conditions, and approval gates.

## Research Claim

A statement connected to evidence, provenance, authorship, confidence, and possible disagreement.

## Computational Suggestion

A machine-generated relation or analysis that has not necessarily received scholarly verification.

---



# 97. PRD Change Log

## Version 0.3.0 — Q-ai Consolidated Replacement

Version 0.3.0 replaces Version 0.2.0 as the active PRD after consolidation.

Added:

- Q-ai product identity
- Quran-first research scope
- Canonical Quran corpus
- Exact and normalized Arabic search
- Search without diacritics
- Concatenated phrase search
- Root, lemma, stem, and morphology tools
- Kalimāt-e Ham-Khānevādeh / word-family research
- Frequency, distribution, collocation, and co-occurrence tools
- Quran graph and graph search
- Rich Quran reader
- Structured Shia hadith support
- Al-Kafi support
- Bihar al-Anwar support
- Isnad and narrator graph
- Attributed grading model
- Rijal integration
- Tafsir comparison
- Torah and Bible research
- Comparative scripture graph
- Smart tool and RAG routing
- Claim-level citations
- Citation verification
- Source trust levels
- Book catalog discovery
- Internet updates
- Manifest validation
- Staging, approval, version differences, and rollback
- Research workspaces
- Corpus-specific diagnostics
- Corpus integrity evaluation
- Quran-first MVP

Preserved and integrated from Version 0.2.0:

- Rust architecture
- CLI, TUI, Web GUI, and API
- Local and server operation
- Multi-RAG
- Document ingestion
- Embedding abstractions
- Vector-store abstractions
- LLM provider abstractions
- Model capability discovery
- Model routing and fallback
- Streaming
- Cancellation
- Background jobs
- Agent runtime
- Multi-agent agencies
- Typed tool system
- Agent-assisted tool creation
- Tool sandbox
- Human approvals
- MCP
- Agent memory
- Workflows
- Budgets
- Audit trails
- Access-control-aware retrieval
- Prompt-injection defenses
- Conversation management
- Evaluation
- Reliability
- Security
- Docker
- Production hardening
- Living PRD rules
- Definition of Done

Scope correction:

- The Islamic hadith corpus is limited to the requested Shia collections and configurable approved Shia sources.
- Earlier unwanted collection references are removed from the active specification.
- Generic architecture support for adding a source does not automatically place that source inside the approved product corpus.

## Version 0.2.0 — Historical Baseline

Version 0.2.0 introduced the general Rust Multi-RAG, agent, agency, tool, workflow, sandbox, model-routing, security, and evaluation architecture.

It remains historical and must not be used beside Version 0.3.0 as a second active PRD.

