# PRD — Rust Multi-RAG AI Platform

**Status:** Draft / Living Document  
**Version:** 0.1.0  
**Language:** Rust  
**Primary Interfaces:** TUI + Web GUI  
**Deployment:** Local + Server  
**Architecture:** Modular / Extensible / Multi-RAG

---

# 1. Product Overview

Build a modern AI/RAG platform written primarily in **Rust**.

The application must provide:

- Multiple independent RAG systems
- Multiple knowledge bases
- Multiple document sources
- Multiple vector databases
- Multiple embedding providers
- Multiple LLM providers
- Configurable RAG pipelines
- Interactive terminal UI (TUI)
- Browser-based GUI
- Server mode
- Local/standalone mode
- API access
- Background jobs
- Persistent configuration
- Observability and logging
- Plugin/extensibility architecture

The same core engine must power:

1. CLI
2. TUI
3. Web GUI
4. Server/API

The architecture must avoid duplicating business logic between interfaces.

---

# 2. Goals

## Primary Goals

- Build a high-performance RAG platform in Rust.
- Support multiple RAG systems simultaneously.
- Make RAG pipelines configurable rather than hard-coded.
- Provide an excellent terminal experience.
- Provide a modern browser-based management interface.
- Allow the application to run entirely locally.
- Allow the application to run as a server.
- Allow users to manage the server from a browser.
- Support multiple LLM and embedding providers.
- Make components replaceable.
- Make the system suitable for both developers and advanced users.

## Secondary Goals

- Easy installation.
- Cross-platform support.
- Docker support.
- Strong configuration system.
- Good developer experience.
- API-first architecture.
- Future plugin ecosystem.
- Future distributed execution.

---

# 3. Non-Goals

The initial version will NOT attempt to:

- Build its own foundation model.
- Implement every vector database natively.
- Implement every document parser from scratch.
- Become a full enterprise data warehouse.
- Replace general-purpose databases.
- Support distributed clusters in the first release.

These can be added later through adapters/plugins.

---

# 4. Core Architecture

The system should follow a layered architecture.

```text
┌──────────────────────────────────────────────┐
│                  Interfaces                  │
│                                              │
│       TUI        Web GUI        CLI          │
└──────────────────────┬───────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────┐
│                 API Layer                    │
│                                              │
│              REST / WebSocket                │
└──────────────────────┬───────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────┐
│                Application                   │
│                                              │
│   RAG Manager / Project Manager / Jobs       │
└──────────────────────┬───────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────┐
│                   Core                       │
│                                              │
│ Documents                                    │
│ Chunking                                     │
│ Embeddings                                   │
│ Retrieval                                    │
│ Reranking                                    │
│ Prompting                                    │
│ Generation                                   │
│ Pipelines                                    │
└──────────────────────┬───────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────┐
│                 Adapters                     │
│                                              │
│ LLM Providers                                │
│ Embedding Providers                          │
│ Vector Databases                             │
│ Document Loaders                             │
│ Storage                                      │
└──────────────────────────────────────────────┘
````

---

# 5. Technology Stack

## Programming Language

Primary language:

* Rust

Minimum supported Rust version should be defined once implementation begins.

Use modern stable Rust whenever possible.

---

# 6. Rust UI Requirements

The project MUST provide both:

## 6.1 TUI

The TUI should use a mature Rust terminal UI ecosystem.

Preferred stack:

* `ratatui`
* `crossterm`

The TUI should support:

* Interactive dashboards
* Tables
* Lists
* Forms
* Tabs
* Trees
* Progress bars
* Logs
* Search
* Command palette
* Configuration editors
* RAG pipeline visualization
* Job monitoring
* Server monitoring
* Keyboard shortcuts
* Mouse support where possible
* Resizable panels

The TUI should feel like a professional developer tool.

---

# 7. Web GUI

The project MUST provide a browser-based GUI.

The GUI should be usable when the application is running in:

```text
serve mode
```

The browser interface should allow management of:

* RAG systems
* Knowledge bases
* Documents
* Collections
* Embedding models
* LLM providers
* Vector databases
* Retrieval settings
* RAG pipelines
* Jobs
* Server configuration
* Users/authentication
* Logs
* Metrics
* System health

The Web GUI should communicate with the Rust backend through the API.

---

# 8. GUI Architecture

The backend must remain independent from the frontend.

Preferred architecture:

```text
Browser
   │
   │ HTTP / WebSocket
   ▼
Rust API Server
   │
   ▼
Application Layer
   │
   ▼
RAG Core
```

The frontend technology may be selected during implementation based on the best current Rust/Web ecosystem.

Potential approaches include:

* Rust/WASM frontend
* Leptos
* Dioxus
* Yew
* Separate TypeScript frontend

The final choice must prioritize:

1. Stability
2. Developer experience
3. Performance
4. UI quality
5. Long-term maintainability

---

# 9. Execution Modes

The application MUST support multiple execution modes.

## 9.1 Interactive Mode

```bash
app
```

Launches the interactive TUI.

---

## 9.2 CLI Mode

```bash
app <command>
```

Examples:

```bash
app rag list
app rag create
app rag run
app document import
app server start
app config show
```

---

## 9.3 Server Mode

```bash
app serve
```

Starts the backend server.

Example:

```text
Browser
   │
   ▼
http://localhost:8080
   │
   ▼
Rust Server
```

The server must expose:

* REST API
* WebSocket API where required
* Web GUI
* Health endpoints
* Metrics endpoints
* Authentication
* Background job management

---

# 10. Server Management

When running:

```bash
app serve
```

the user should be able to manage the entire application from a browser.

The GUI must provide:

```text
Dashboard
├── System
├── RAG Systems
├── Knowledge Bases
├── Documents
├── Retrieval
├── Models
│   ├── LLM
│   └── Embeddings
├── Vector Stores
├── Pipelines
├── Jobs
├── Logs
├── Metrics
├── Settings
└── Users
```

---

# 11. Multi-RAG Architecture

The system MUST support multiple independent RAG systems.

Example:

```text
RAG Manager

├── Company Documentation RAG
├── Programming RAG
├── Legal Documents RAG
├── Research RAG
├── Personal Knowledge RAG
└── Custom RAG
```

Each RAG system should be independently configurable.

Each RAG system can have:

* Name
* Description
* Documents
* Collections
* Chunking strategy
* Embedding model
* Vector database
* Retriever
* Reranker
* LLM
* Prompt templates
* Search parameters
* Metadata filters
* Pipeline configuration

---

# 12. RAG Pipeline

A RAG system should support configurable pipelines.

Basic pipeline:

```text
Documents
    │
    ▼
Loader
    │
    ▼
Parser
    │
    ▼
Cleaner
    │
    ▼
Chunker
    │
    ▼
Embedding
    │
    ▼
Vector Store
```

Query pipeline:

```text
User Query
    │
    ▼
Query Processing
    │
    ▼
Query Embedding
    │
    ▼
Retriever
    │
    ▼
Reranker
    │
    ▼
Context Builder
    │
    ▼
Prompt Builder
    │
    ▼
LLM
    │
    ▼
Response
```

---

# 13. Retrieval Strategies

The system should support multiple retrieval strategies.

Initial strategies:

* Vector similarity search
* Keyword search
* Full-text search
* Hybrid search
* Metadata filtering
* Multi-query retrieval
* Contextual retrieval
* Reranking

Future strategies:

* Graph retrieval
* Knowledge graph RAG
* Agentic retrieval
* Recursive retrieval
* Parent-child retrieval
* Hierarchical retrieval

---

# 14. Multiple Vector Databases

The architecture MUST use an abstraction layer for vector storage.

Example:

```rust
trait VectorStore {
    async fn create_collection(...);
    async fn delete_collection(...);
    async fn upsert(...);
    async fn search(...);
    async fn delete(...);
}
```

Potential adapters:

* Qdrant
* PostgreSQL + pgvector
* SQLite-based local storage
* Elasticsearch/OpenSearch
* Weaviate
* Milvus
* Pinecone

The first implementation should prioritize a small number of high-quality adapters rather than implementing everything.

---

# 15. Embedding Providers

Embedding providers must use an abstraction layer.

Example:

```rust
trait EmbeddingProvider {
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}
```

Potential providers:

* OpenAI-compatible APIs
* OpenAI
* Ollama
* Hugging Face
* Local embedding models
* Custom HTTP providers

The system must support OpenAI-compatible endpoints with configurable:

```text
base_url
api_key
model
dimensions
```

---

# 16. LLM Providers

LLM access must use a provider abstraction.

Example:

```rust
trait LlmProvider {
    async fn generate(&self, request: GenerationRequest)
        -> Result<GenerationResponse>;
}
```

Support:

* OpenAI-compatible APIs
* OpenAI
* Anthropic
* Ollama
* Local models
* Custom HTTP endpoints

Provider configuration should support:

```text
name
base_url
api_key
model
temperature
max_tokens
timeout
```

---

# 17. Document Management

The system must support importing documents into knowledge bases.

Initial formats:

* Markdown
* Plain text
* PDF
* HTML
* JSON
* CSV
* DOCX

Potential future formats:

* EPUB
* XLSX
* PPTX
* Images
* Audio
* Video

Documents should have metadata:

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
```

---

# 18. Document Processing

Document processing must be asynchronous.

Pipeline:

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

The UI must show processing progress.

Example:

```text
Document: architecture.pdf

Parsing       ████████████████ 100%
Chunking      ████████████████ 100%
Embedding     ███████████░░░░░  72%
Indexing      ░░░░░░░░░░░░░░░░   0%

Status: Processing
```

---

# 19. Chunking System

Chunking must be modular.

Support:

* Fixed-size chunks
* Token-based chunks
* Sentence chunks
* Paragraph chunks
* Markdown-aware chunks
* Recursive chunks
* Parent-child chunks

Configuration:

```text
chunk_size
chunk_overlap
separator
max_tokens
min_tokens
strategy
```

---

# 20. Metadata

Every document chunk should support metadata.

Example:

```json
{
  "document_id": "...",
  "source": "...",
  "title": "...",
  "page": 12,
  "section": "Architecture",
  "language": "en",
  "created_at": "...",
  "custom": {}
}
```

Metadata must be available for:

* Filtering
* Display
* Debugging
* Citations
* Retrieval
* Evaluation

---

# 21. Citations

RAG responses should provide citations whenever possible.

Example:

```text
The authentication service uses JWT tokens.

Sources:
[1] architecture.md
[2] authentication.pdf — page 14
[3] api-reference.md
```

The GUI should allow users to click a citation and inspect the source chunk.

---

# 22. RAG Evaluation

The system should eventually support evaluation of RAG pipelines.

Metrics may include:

* Retrieval precision
* Retrieval recall
* Context relevance
* Faithfulness
* Answer relevance
* Latency
* Token usage
* Cost

Users should be able to compare different RAG configurations.

Example:

```text
RAG A
Retrieval: 82%
Faithfulness: 91%
Latency: 1.8s

RAG B
Retrieval: 91%
Faithfulness: 94%
Latency: 2.4s
```

---

# 23. Background Job System

Long-running tasks must run as background jobs.

Examples:

* Document indexing
* Re-indexing
* Embedding generation
* Bulk imports
* Evaluation
* Data migration

Job states:

```text
Queued
Running
Completed
Failed
Cancelled
```

Jobs must expose:

* ID
* Type
* Status
* Progress
* Started time
* Finished time
* Error
* Logs

---

# 24. Persistence

The application requires persistent metadata storage.

The storage layer must be abstracted.

Possible initial database:

* SQLite

Potential future databases:

* PostgreSQL

Persist:

* RAG configurations
* Knowledge bases
* Documents
* Provider configurations
* Pipelines
* Jobs
* Users
* Settings

Secrets must never be stored in plaintext unless explicitly configured by the user.

---

# 25. Configuration

Configuration should support:

```text
TOML
Environment Variables
CLI arguments
```

Example conceptual configuration:

```toml
[server]
host = "127.0.0.1"
port = 8080

[database]
url = "sqlite://app.db"

[llm.default]
provider = "openai"
model = "..."

[embedding.default]
provider = "openai"
model = "..."

[rag.default]
chunk_size = 800
chunk_overlap = 100
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

---

# 26. Secrets

API keys and credentials must be handled securely.

Possible mechanisms:

* Environment variables
* OS keychain
* Encrypted local storage
* Secret manager integration

Never expose secrets through:

* Logs
* API responses
* TUI output
* Browser HTML
* Error messages

---

# 27. API

The server must expose a documented API.

Core endpoints:

```text
GET    /api/health

GET    /api/rags
POST   /api/rags
GET    /api/rags/:id
PUT    /api/rags/:id
DELETE /api/rags/:id

GET    /api/knowledge-bases
POST   /api/knowledge-bases

GET    /api/documents
POST   /api/documents
DELETE /api/documents/:id

POST   /api/query

GET    /api/jobs
GET    /api/jobs/:id

GET    /api/providers
GET    /api/models

GET    /api/system
```

API design should be versioned:

```text
/api/v1/...
```

---

# 28. WebSocket

WebSockets should be used where real-time communication improves UX.

Examples:

* Job progress
* Streaming LLM responses
* Server logs
* System metrics
* RAG execution events

---

# 29. Authentication

Server mode should support authentication.

Initial requirements:

* Local single-user mode
* Optional authentication
* API tokens

Future:

* Multiple users
* Roles
* Permissions
* OAuth/OIDC

---

# 30. TUI Screens

Minimum TUI screens:

```text
Dashboard

RAG Systems
├── List
├── Create
├── Edit
├── Run
└── Inspect

Knowledge Bases
├── List
├── Create
├── Documents
└── Search

Documents
├── Import
├── Process
├── Inspect
└── Delete

Models
├── LLM
├── Embeddings
└── Rerankers

Vector Stores

Jobs

Logs

Configuration

Server

Help
```

---

# 31. Command Palette

The TUI should provide a command palette.

Example:

```text
> Search commands...

Create RAG
Import Document
Run RAG
Open Knowledge Base
View Jobs
Start Server
Stop Server
Open Configuration
Search Logs
```

---

# 32. RAG Debugging

A major feature should be transparent RAG execution.

Users should be able to inspect:

```text
Query
 ↓
Processed Query
 ↓
Embedding
 ↓
Retrieved Documents
 ↓
Similarity Scores
 ↓
Reranked Documents
 ↓
Final Context
 ↓
Prompt
 ↓
LLM Request
 ↓
LLM Response
```

This should be available in both TUI and GUI.

---

# 33. Observability

The application should use structured logging.

Support:

* Logs
* Metrics
* Traces
* Request timing
* RAG execution timing
* Token usage
* Error tracking

Prefer Rust ecosystem standards such as:

* `tracing`
* `tracing-subscriber`
* OpenTelemetry where appropriate

---

# 34. Performance

Performance requirements:

* Async architecture
* Non-blocking I/O
* Streaming responses
* Concurrent document processing
* Configurable worker pools
* Efficient memory usage
* Efficient serialization
* Cancellation support

CPU-heavy workloads should not block the async runtime.

---

# 35. Error Handling

Use typed Rust errors.

Prefer:

* `thiserror`
* `anyhow` at application boundaries where appropriate

Errors should provide:

```text
What happened
Why it happened
Where it happened
How the user can fix it
```

---

# 36. Project Structure

Preferred workspace structure:

```text
project/
│
├── Cargo.toml
├── Cargo.lock
├── README.md
│
├── crates/
│   ├── core/
│   ├── domain/
│   ├── application/
│   ├── rag/
│   ├── embeddings/
│   ├── llm/
│   ├── retrieval/
│   ├── reranking/
│   ├── documents/
│   ├── storage/
│   ├── vectorstore/
│   ├── api/
│   ├── server/
│   ├── tui/
│   ├── cli/
│   └── config/
│
├── web/
│
├── migrations/
│
├── tests/
│
├── docs/
│
└── examples/
```

Exact structure may evolve as implementation progresses.

---

# 37. Dependency Principles

Dependencies should be selected based on:

1. Project maturity
2. Maintenance activity
3. Rust ecosystem adoption
4. API quality
5. Performance
6. Security
7. License compatibility
8. Cross-platform support

Avoid unnecessary dependencies.

Do not introduce a dependency merely because it is convenient if a small internal abstraction is preferable.

---

# 38. Security

Security requirements:

* Validate all external input.
* Protect API keys.
* Avoid command injection.
* Avoid arbitrary filesystem access.
* Sanitize uploaded documents where appropriate.
* Enforce configurable file-size limits.
* Enforce request limits.
* Use secure HTTP configuration.
* Avoid leaking sensitive information through logs.
* Provide authentication for exposed server deployments.

---

# 39. Testing

Testing levels:

## Unit Tests

Test:

* Chunkers
* Parsers
* Retrievers
* Prompt builders
* Configuration
* Domain logic

## Integration Tests

Test:

* Vector stores
* LLM providers
* Embedding providers
* API
* Database

## End-to-End Tests

Test:

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

---

# 40. Developer Experience

The project should provide:

```bash
cargo build
cargo test
cargo clippy
cargo fmt
```

Development commands should be documented.

Provide:

* `.env.example`
* Example configurations
* Example RAG projects
* Docker configuration
* Development documentation

---

# 41. Docker

The server should eventually be deployable with Docker.

Example:

```bash
docker run ...
```

Potential Docker Compose stack:

```text
app
├── Rust RAG Server
├── PostgreSQL
├── Qdrant
└── Optional monitoring
```

The application should also work without Docker.

---

# 42. Local-First Design

The application should work locally without requiring a cloud service.

Possible local stack:

```text
Rust Application
      │
      ├── SQLite
      ├── Local filesystem
      ├── Local embeddings
      ├── Ollama
      └── Local vector store
```

Cloud services should be optional.

---

# 43. Plugin Architecture

The architecture should allow future plugins.

Potential plugins:

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

Plugins should be introduced only when the architecture genuinely benefits from them.

Avoid premature dynamic plugin complexity.

---

# 44. Import Sources

Future document sources:

```text
Filesystem
Git repositories
GitHub
URLs
Websites
S3
MinIO
Google Drive
Notion
Telegram
Email
Databases
APIs
```

Each source should use an adapter model.

---

# 45. RAG Projects

A user should be able to create reusable RAG projects.

Example:

```text
my-rag/
├── rag.toml
├── documents/
├── prompts/
├── data/
└── cache/
```

A project should be exportable and portable.

---

# 46. RAG Configuration

A RAG configuration should describe the complete pipeline.

Conceptually:

```toml
[rag]
name = "Programming Knowledge"

[loader]
type = "filesystem"

[chunking]
strategy = "recursive"
size = 800
overlap = 100

[embedding]
provider = "openai"
model = "..."

[vector_store]
type = "qdrant"
collection = "programming"

[retrieval]
strategy = "hybrid"
top_k = 10

[reranker]
enabled = true

[llm]
provider = "openai"
model = "..."

[prompt]
template = "default"
```

---

# 47. Multiple RAG Composition

The system should eventually support querying multiple RAG systems.

Example:

```text
User Query
     │
     ├── Programming RAG
     │
     ├── Documentation RAG
     │
     └── Project RAG
              │
              ▼
       Result Aggregator
              │
              ▼
            LLM
```

Possible strategies:

* Parallel retrieval
* Weighted retrieval
* Result merging
* Reranking
* RAG routing
* Query classification

---

# 48. Agentic Features

Agentic RAG should be considered a future capability.

Possible architecture:

```text
User
 ↓
Agent
 ├── RAG 1
 ├── RAG 2
 ├── Web Search
 ├── Database
 └── Tools
 ↓
Final Answer
```

This should not complicate the initial RAG architecture.

---

# 49. CLI Design

CLI commands should be consistent.

Example:

```bash
app rag list
app rag create my-rag
app rag delete my-rag
app rag query my-rag "What is RAG?"

app kb list
app kb create docs

app document import ./docs

app server start
app server stop
app server status

app config get
app config set

app doctor
```

---

# 50. Doctor Command

Provide:

```bash
app doctor
```

It should diagnose:

* Configuration
* Database
* Vector store
* LLM connectivity
* Embedding connectivity
* File permissions
* Network connectivity
* Required dependencies

Example:

```text
✓ Configuration
✓ Database
✓ Qdrant
✓ Embedding provider
✗ LLM provider

LLM provider:
Connection refused

Suggested action:
Check base_url configuration.
```

---

# 51. Server Dashboard

The web dashboard should provide a high-level overview.

Example:

```text
SYSTEM

Status       ● Running
Version      0.1.0
Uptime       2h 31m

RAG SYSTEMS
12

DOCUMENTS
18,492

VECTOR RECORDS
2,841,229

ACTIVE JOBS
3

REQUESTS
1,294

AVG LATENCY
1.42s
```

---

# 52. RAG Query UI

The browser query interface should provide:

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

Results should show:

* Answer
* Sources
* Retrieved chunks
* Scores
* Execution time
* Token usage
* RAG pipeline details

---

# 53. Streaming

LLM responses should support streaming.

The UI should display generated text incrementally.

TUI:

```text
Generating...

Rust provides ownership-based memory
management that...
```

Web:

```text
Streaming response...
```

---

# 54. Cancellation

Long-running operations must be cancellable.

Users should be able to cancel:

* RAG query
* Document indexing
* Embedding jobs
* Bulk imports
* Evaluations

Cancellation must propagate through the pipeline.

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

---

# 56. Development Phases

## Phase 0 — Architecture

* Workspace setup
* Core domain model
* Error handling
* Configuration
* Logging
* Basic CLI

## Phase 1 — Core RAG

* Documents
* Chunking
* Embeddings
* Vector store
* Retrieval
* LLM
* Basic RAG pipeline

## Phase 2 — Multi-RAG

* RAG manager
* Multiple knowledge bases
* Multiple pipelines
* RAG configuration
* RAG selection

## Phase 3 — TUI

* Ratatui application
* Dashboard
* RAG management
* Document management
* Jobs
* Logs
* Configuration

## Phase 4 — Server

* HTTP API
* WebSocket
* Authentication
* Background jobs
* Server lifecycle

## Phase 5 — Web GUI

* Dashboard
* RAG management
* Knowledge bases
* Documents
* Query interface
* Logs
* Metrics

## Phase 6 — Advanced RAG

* Hybrid search
* Reranking
* Multi-query
* Query rewriting
* Multi-RAG routing
* Evaluation

## Phase 7 — Production

* Docker
* PostgreSQL
* Security hardening
* Observability
* Performance optimization
* Documentation

---

# 57. MVP Definition

The MVP is complete when the following workflow works:

```text
Install application
       ↓
Start application
       ↓
Create RAG
       ↓
Create knowledge base
       ↓
Import Markdown/PDF documents
       ↓
Chunk documents
       ↓
Generate embeddings
       ↓
Store vectors
       ↓
Ask question
       ↓
Retrieve relevant chunks
       ↓
Send context to LLM
       ↓
Receive streamed answer
       ↓
Display citations
```

The same RAG must be usable through:

```text
CLI
TUI
API
Browser GUI
```

---

# 58. Quality Requirements

The project should prioritize:

### Correctness

RAG results must be deterministic where configuration allows.

### Performance

Async and concurrent execution should be used appropriately.

### Reliability

Failures in external providers must not crash the entire application.

### Maintainability

Core functionality must remain independent from UI implementations.

### Extensibility

Adding a new provider should require implementing an adapter rather than modifying core RAG logic.

### UX

Both TUI and GUI must be usable by developers without reading documentation for basic operations.

---

# 59. Future Features

Potential future roadmap:

* Agentic RAG
* Knowledge graphs
* Graph RAG
* MCP integration
* Tool calling
* Workflow builder
* Visual RAG pipeline editor
* RAG marketplace
* Plugin marketplace
* Multi-user collaboration
* RBAC
* OAuth/OIDC
* Distributed workers
* GPU acceleration
* Cluster mode
* Cloud deployment
* RAG benchmarking
* Automated pipeline optimization
* AI-assisted RAG configuration

---

# 60. Open Technical Decisions

These decisions must be evaluated during implementation rather than assumed permanently:

* Web GUI framework
* Primary database
* Vector database default
* Local embedding engine
* Plugin architecture mechanism
* Authentication framework
* API framework
* Job queue implementation
* WebSocket architecture
* Configuration format extensions
* Local model runtime

Technology decisions should favor mature Rust ecosystem solutions.

---

# 61. Definition of Done

A feature is considered complete only when:

* It is implemented.
* It has appropriate tests.
* It has error handling.
* It is observable through logs where appropriate.
* It is documented.
* It works through the intended interface(s).
* It does not unnecessarily couple layers.
* `cargo fmt` succeeds.
* `cargo clippy` succeeds.
* `cargo test` succeeds.

---

# 62. Current Project Status

## Completed

* [ ] Project architecture
* [ ] Cargo workspace
* [ ] Core domain
* [ ] Configuration
* [ ] CLI
* [ ] RAG engine
* [ ] Document processing
* [ ] Embedding abstraction
* [ ] LLM abstraction
* [ ] Vector store abstraction
* [ ] Multi-RAG
* [ ] TUI
* [ ] API
* [ ] Server mode
* [ ] Web GUI
* [ ] Background jobs
* [ ] Authentication
* [ ] Observability
* [ ] Docker
* [ ] Documentation

## Current Version

`0.1.0`

## Current Phase

`Phase 0 — Architecture`

---

# 63. Living PRD Rules

This document is the authoritative product specification.

Whenever the project requirements change:

1. Update this PRD.
2. Increment the version when appropriate.
3. Preserve completed requirements.
4. Mark obsolete requirements explicitly.
5. Keep architecture decisions documented.
6. Keep the roadmap synchronized with implementation.
7. Do not silently remove previously agreed requirements.

When requesting an update to this PRD, the complete updated PRD should be returned so it can replace the previous version.

---

```
