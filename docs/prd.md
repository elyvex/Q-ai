# PRD — Rust Multi-RAG and Agentic AI Platform

**Status:** Draft / Living Document  
**Version:** 0.2.0
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

- Rust

Minimum supported Rust version should be defined once implementation begins.

Use modern stable Rust whenever possible.

---

# 6. Rust UI Requirements

The project MUST provide both:

## 6.1 TUI

The TUI should use a mature Rust terminal UI ecosystem.

Preferred stack:

- `ratatui`
- `crossterm`

The TUI should support:

- Interactive dashboards
- Tables
- Lists
- Forms
- Tabs
- Trees
- Progress bars
- Logs
- Search
- Command palette
- Configuration editors
- RAG pipeline visualization
- Job monitoring
- Server monitoring
- Keyboard shortcuts
- Mouse support where possible
- Resizable panels

The TUI should feel like a professional developer tool.

---

# 7. Web GUI

The project MUST provide a browser-based GUI.

The GUI should be usable when the application is running in:

```text
serve mode
```

The browser interface should allow management of:

- RAG systems
- Knowledge bases
- Documents
- Collections
- Embedding models
- LLM providers
- Vector databases
- Retrieval settings
- RAG pipelines
- Jobs
- Server configuration
- Users/authentication
- Logs
- Metrics
- System health

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

- Rust/WASM frontend
- Leptos
- Dioxus
- Yew
- Separate TypeScript frontend

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

- REST API
- WebSocket API where required
- Web GUI
- Health endpoints
- Metrics endpoints
- Authentication
- Background job management

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

- Name
- Description
- Documents
- Collections
- Chunking strategy
- Embedding model
- Vector database
- Retriever
- Reranker
- LLM
- Prompt templates
- Search parameters
- Metadata filters
- Pipeline configuration

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

- Vector similarity search
- Keyword search
- Full-text search
- Hybrid search
- Metadata filtering
- Multi-query retrieval
- Contextual retrieval
- Reranking

Future strategies:

- Graph retrieval
- Knowledge graph RAG
- Agentic retrieval
- Recursive retrieval
- Parent-child retrieval
- Hierarchical retrieval

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

- Qdrant
- PostgreSQL + pgvector
- SQLite-based local storage
- Elasticsearch/OpenSearch
- Weaviate
- Milvus
- Pinecone

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

- OpenAI-compatible APIs
- OpenAI
- Ollama
- Hugging Face
- Local embedding models
- Custom HTTP providers

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

- OpenAI-compatible APIs
- OpenAI
- Anthropic
- Ollama
- Local models
- Custom HTTP endpoints

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

- Markdown
- Plain text
- PDF
- HTML
- JSON
- CSV
- DOCX

Potential future formats:

- EPUB
- XLSX
- PPTX
- Images
- Audio
- Video

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

- Fixed-size chunks
- Token-based chunks
- Sentence chunks
- Paragraph chunks
- Markdown-aware chunks
- Recursive chunks
- Parent-child chunks

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

- Filtering
- Display
- Debugging
- Citations
- Retrieval
- Evaluation

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

- Retrieval precision
- Retrieval recall
- Context relevance
- Faithfulness
- Answer relevance
- Latency
- Token usage
- Cost

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

- Document indexing
- Re-indexing
- Embedding generation
- Bulk imports
- Evaluation
- Data migration

Job states:

```text
Queued
Running
Completed
Failed
Cancelled
```

Jobs must expose:

- ID
- Type
- Status
- Progress
- Started time
- Finished time
- Error
- Logs

---

# 24. Persistence

The application requires persistent metadata storage.

The storage layer must be abstracted.

Possible initial database:

- SQLite

Potential future databases:

- PostgreSQL

Persist:

- RAG configurations
- Knowledge bases
- Documents
- Provider configurations
- Pipelines
- Jobs
- Users
- Settings

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

- Job progress
- Streaming LLM responses
- Server logs
- System metrics
- RAG execution events

---

# 29. Authentication

Server mode should support authentication.

Initial requirements:

- Local single-user mode
- Optional authentication
- API tokens

Future:

- Multiple users
- Roles
- Permissions
- OAuth/OIDC

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

- Logs
- Metrics
- Traces
- Request timing
- RAG execution timing
- Token usage
- Error tracking

Prefer Rust ecosystem standards such as:

- `tracing`
- `tracing-subscriber`
- OpenTelemetry where appropriate

---

# 34. Performance

Performance requirements:

- Async architecture
- Non-blocking I/O
- Streaming responses
- Concurrent document processing
- Configurable worker pools
- Efficient memory usage
- Efficient serialization
- Cancellation support

CPU-heavy workloads should not block the async runtime.

---

# 35. Error Handling

Use typed Rust errors.

Prefer:

- `thiserror`
- `anyhow` at application boundaries where appropriate

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

- Validate all external input.
- Protect API keys.
- Avoid command injection.
- Avoid arbitrary filesystem access.
- Sanitize uploaded documents where appropriate.
- Enforce configurable file-size limits.
- Enforce request limits.
- Use secure HTTP configuration.
- Avoid leaking sensitive information through logs.
- Provide authentication for exposed server deployments.

---

# 39. Testing

Testing levels:

## Unit Tests

Test:

- Chunkers
- Parsers
- Retrievers
- Prompt builders
- Configuration
- Domain logic

## Integration Tests

Test:

- Vector stores
- LLM providers
- Embedding providers
- API
- Database

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

- `.env.example`
- Example configurations
- Example RAG projects
- Docker configuration
- Development documentation

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

- Parallel retrieval
- Weighted retrieval
- Result merging
- Reranking
- RAG routing
- Query classification

---

# 48. Agentic Features

Agentic RAG should be considered a core requirement.

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

- Configuration
- Database
- Vector store
- LLM connectivity
- Embedding connectivity
- File permissions
- Network connectivity
- Required dependencies

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

- Answer
- Sources
- Retrieved chunks
- Scores
- Execution time
- Token usage
- RAG pipeline details

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

- RAG query
- Document indexing
- Embedding jobs
- Bulk imports
- Evaluations

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

- Workspace setup
- Core domain model
- Error handling
- Configuration
- Logging
- Basic CLI

## Phase 1 — Core RAG

- Documents
- Chunking
- Embeddings
- Vector store
- Retrieval
- LLM
- Basic RAG pipeline

## Phase 2 — Multi-RAG

- RAG manager
- Multiple knowledge bases
- Multiple pipelines
- RAG configuration
- RAG selection

## Phase 3 — TUI

- Ratatui application
- Dashboard
- RAG management
- Document management
- Jobs
- Logs
- Configuration

## Phase 4 — Server

- HTTP API
- WebSocket
- Authentication
- Background jobs
- Server lifecycle

## Phase 5 — Web GUI

- Dashboard
- RAG management
- Knowledge bases
- Documents
- Query interface
- Logs
- Metrics

## Phase 6 — Advanced RAG

- Hybrid search
- Reranking
- Multi-query
- Query rewriting
- Multi-RAG routing
- Evaluation

## Phase 7 - Agentic features

- Agentic RAG
- MCP integration
- Tool calling
- Workflow builder
- AI-assisted tool creation

## Phase 8 — Production

- Docker
- PostgreSQL
- Security hardening
- Observability
- Performance optimization
- Documentation

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

- Knowledge graphs
- Graph RAG
- Visual RAG pipeline editor
- RAG marketplace
- Plugin marketplace
- Multi-user collaboration
- RBAC
- OAuth/OIDC
- Distributed workers
- GPU acceleration
- Cluster mode
- Cloud deployment
- RAG benchmarking
- Automated pipeline optimization
- AI-assisted RAG configuration

---

# 60. Open Technical Decisions

These decisions must be evaluated during implementation rather than assumed permanently:

- Web GUI framework
- Primary database
- Vector database default
- Local embedding engine
- Plugin architecture mechanism
- Authentication framework
- API framework
- Job queue implementation
- WebSocket architecture
- Configuration format extensions
- Local model runtime

Technology decisions should favor mature Rust ecosystem solutions.

---

# 61. Definition of Done

A feature is considered complete only when:

- It is implemented.
- It has appropriate tests.
- It has error handling.
- It is observable through logs where appropriate.
- It is documented.
- It works through the intended interface(s).
- It does not unnecessarily couple layers.
- `cargo fmt` succeeds.
- `cargo clippy` succeeds.
- `cargo test` succeeds.

---

# 62. Current Project Status

## Completed

- [ ] Project architecture
- [ ] Cargo workspace
- [ ] Core domain
- [ ] Configuration
- [ ] CLI
- [ ] RAG engine
- [ ] Document processing
- [ ] Embedding abstraction
- [ ] LLM abstraction
- [ ] Vector store abstraction
- [ ] Multi-RAG
- [ ] TUI
- [ ] API
- [ ] Server mode
- [ ] Web GUI
- [ ] Background jobs
- [ ] Authentication
- [ ] Observability
- [ ] Docker
- [ ] Documentation

- [ ] LLM connection manager
- [ ] Model capability discovery
- [ ] Model router
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
- [ ] Agent evaluation

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

# 64. Terminology

The following terms have specific meanings in this PRD:

## Model

An LLM, embedding model, reranker, speech model, vision model, or other inference model accessed through a provider.

## Agent

An AI runtime configured with:

- One or more models
- System instructions
- Tools
- Knowledge bases
- Memory
- Permissions
- Execution limits
- Output requirements

## Agency

A group of agents that collaborate to complete tasks.

An agency may contain:

- Supervisor agents
- Worker agents
- Research agents
- Coding agents
- Review agents
- Tool-building agents
- Custom user-defined agents

## Tool

A capability that an agent can invoke through a typed interface.

Examples:

- Search a knowledge base
- Read a file
- Write a file
- Run an HTTP request
- Query a database
- Execute a command
- Call an MCP server
- Invoke another agent
- Start a workflow

## Tool Creation

The process of generating, validating, testing, approving, and publishing a new tool.

Generated tools must never be automatically trusted or executed with unrestricted permissions.

## Workflow

A deterministic or agent-driven graph of steps, tools, models, and approval gates.

---

# 65. Updated Product Scope

The platform must support three first-class workloads:

1. Retrieval-augmented generation
2. AI agents and multi-agent agencies
3. Tool and workflow creation

The product should provide a unified runtime:

```text
User
  │
  ▼
CLI / TUI / Web GUI / API
  │
  ▼
Conversation and Task Runtime
  │
  ├── Direct LLM Chat
  ├── RAG Pipeline
  ├── Single Agent
  ├── Multi-Agent Agency
  └── Workflow
          │
          ▼
     Tool Runtime
          │
          ├── Built-in Tools
          ├── User Tools
          ├── Generated Tools
          ├── MCP Tools
          └── Remote API Tools
```

The same agent, tool, RAG, and workflow definitions must be usable through all supported interfaces.

---

# 66. TUI LLM Connection Manager

The TUI must provide a complete interface for creating and managing LLM connections.

## Required Connection Types

Initial support should include:

- OpenAI-compatible APIs
- OpenAI
- Anthropic
- Ollama
- Local model servers
- Custom HTTP providers

Future support may include:

- Google Gemini
- AWS Bedrock
- Azure-hosted models
- llama.cpp servers
- vLLM
- Additional provider plugins

## Connection Profile

Each connection profile should support:

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
enabled
created_at
updated_at
```

Secrets must be represented by references rather than returned through APIs.

Example:

```text
api_key_reference = "keychain://providers/openai/default"
```

## TUI Connection Screens

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

## Connection Test

The user must be able to test:

- Network connectivity
- Authentication
- Model availability
- Streaming support
- Tool-calling support
- Structured-output support
- Vision support
- Context-window limits
- Embedding dimensions
- Request latency

Example:

```text
Connection: Local Ollama

✓ Server reachable
✓ Authentication not required
✓ Model available
✓ Streaming supported
✗ Tool calling not supported
✓ Context window: 32,768 tokens

Latency: 41 ms
```

Connection tests must not expose API keys in logs or error messages.

---

# 67. Unified Model Capability Interface

Provider adapters must expose model capabilities instead of assuming all models support the same features.

Conceptual Rust interface:

```rust
pub struct ModelCapabilities {
    pub chat: bool,
    pub streaming: bool,
    pub tool_calling: bool,
    pub parallel_tool_calls: bool,
    pub structured_output: bool,
    pub vision: bool,
    pub embeddings: bool,
    pub max_context_tokens: Option<u64>,
    pub max_output_tokens: Option<u64>,
}

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;
    async fn capabilities(&self, model: &str)
        -> Result<ModelCapabilities>;
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

Provider differences must be normalized without discarding provider-specific features.

Raw provider options may be passed through a namespaced extension field when necessary.

---

# 68. Model Routing and Fallback

The platform should support model aliases and routing policies.

Example aliases:

```text
fast
balanced
reasoning
coding
local
embedding-default
```

A routing policy may consider:

- Required capabilities
- Privacy requirements
- Local-only mode
- Context length
- Estimated cost
- Provider health
- Request latency
- Rate limits
- Task type
- User preference

Example:

```toml
[model_aliases.fast]
candidates = [
  "ollama/qwen-local",
  "openai-compatible/fast-model"
]

[model_aliases.reasoning]
candidates = [
  "provider-a/reasoning-model",
  "provider-b/fallback-model"
]
```

Fallback must only occur when the fallback model satisfies the task’s required capabilities and data-handling policy.

The runtime must record which model was ultimately selected and why.

---

# 69. Agent Definition

An agent must be represented by a versioned, portable definition.

Example:

```toml
[agent]
name = "research-agent"
version = "1.0.0"
description = "Researches indexed project documentation"
model = "balanced"
max_steps = 12
max_runtime_seconds = 180
max_tool_calls = 20
max_input_tokens = 50000
max_output_tokens = 8000

[instructions]
system_prompt = "./prompts/research-agent.md"

[knowledge]
rag_systems = ["project-docs", "engineering-docs"]

[memory]
conversation = true
working = true
long_term = false

[tools]
allow = [
  "rag.search",
  "document.read",
  "web.fetch"
]

[permissions]
network = ["docs.example.com"]
filesystem_read = ["./workspace"]
filesystem_write = []
command_execution = false

[approval]
required_for = [
  "network.write",
  "filesystem.write",
  "command.execute"
]
```

Agent definitions must include:

- Stable ID
- Name
- Description
- Version
- Model or model-routing policy
- Instructions
- RAG systems
- Tool allowlist
- Tool denylist
- Memory policy
- Execution limits
- Permission policy
- Approval policy
- Output schema
- Tags
- Owner
- Created and updated timestamps

Agent configuration changes must create a new revision.

---

# 70. Agent Runtime

The agent runtime must execute agents as observable, cancellable state machines.

Conceptual lifecycle:

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
     Continue or Finish
          ↓
Completed / Failed / Cancelled / Timed Out
```

## Runtime Requirements

The runtime must support:

- Streaming model output
- Sequential tool calls
- Parallel tool calls where supported
- Cancellation
- Deadlines
- Maximum step count
- Maximum tool-call count
- Token budgets
- Cost budgets
- Retry policies
- Structured output validation
- Human approval
- Checkpointing
- Execution replay
- Failure recovery
- Complete execution traces

Every run must receive a unique run ID.

---

# 71. Agent Run Events

Agent execution must emit normalized events.

Example event types:

```text
run.started
run.completed
run.failed
run.cancelled
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
rag.retrieval_started
rag.retrieval_completed
agent.delegated
agent.message
budget.warning
budget.exceeded
```

Events must be available through:

- Internal event bus
- TUI
- WebSocket or Server-Sent Events
- Web GUI
- Structured logs
- Audit records

Sensitive inputs and outputs must be redacted according to policy.

---

# 72. Multi-Agent Agencies

The platform must support agencies containing multiple collaborating agents.

## Agency Roles

Supported role patterns should include:

- Supervisor
- Router
- Planner
- Worker
- Researcher
- Tool builder
- Reviewer
- Safety reviewer
- Final-answer synthesizer

## Coordination Strategies

Initial strategies:

- Supervisor-and-workers
- Sequential handoff
- Parallel delegation
- Router-based delegation
- Review-and-revise

Future strategies:

- Debate
- Blackboard collaboration
- Hierarchical teams
- Dynamic agent creation
- Consensus execution

## Agency Definition

```toml
[agency]
name = "software-development-agency"
version = "1.0.0"
entry_agent = "supervisor"
max_total_steps = 40
max_runtime_seconds = 900

[[agency.agents]]
id = "supervisor"
agent = "agents/supervisor.toml"

[[agency.agents]]
id = "researcher"
agent = "agents/researcher.toml"

[[agency.agents]]
id = "tool-builder"
agent = "agents/tool-builder.toml"

[[agency.agents]]
id = "reviewer"
agent = "agents/reviewer.toml"

[[agency.routes]]
from = "supervisor"
to = "researcher"
condition = "research_required"

[[agency.routes]]
from = "tool-builder"
to = "reviewer"
condition = "tool_generated"
```

An agency must define:

- Entry agent
- Member agents
- Delegation rules
- Shared and private memory
- Maximum global budget
- Maximum run duration
- Communication policy
- Completion policy
- Failure policy
- Human approval gates

Recursive delegation must have a configurable maximum depth.

---

# 73. Tool System

Tools must use strongly typed definitions.

Conceptual interface:

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

A tool manifest must include:

```text
id
name
description
version
input_schema
output_schema
permissions
timeout
side_effect_class
network_policy
filesystem_policy
execution_runtime
publisher
signature
checksum
status
```

## Side-Effect Classes

Every tool must be classified as one of:

```text
ReadOnly
LocalWrite
ExternalWrite
CommandExecution
Privileged
```

The classification determines approval and sandbox requirements.

## Initial Built-In Tools

- `rag.search`
- `rag.query`
- `document.read`
- `document.list`
- `filesystem.read`
- `filesystem.list`
- `http.get`
- `json.transform`
- `database.query_readonly`
- `agent.delegate`
- `workflow.start`

Write-capable and command-execution tools should be optional and disabled by default.

---

# 74. Agent-Created Tools

Agents may assist users in creating tools, but generated tools must follow a controlled lifecycle.

```text
Tool Request
  ↓
Requirement and Schema Generation
  ↓
Source-Code Generation
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
Human Review
  ↓
Approval
  ↓
Signing
  ↓
Publication
  ↓
Enabled for Selected Agents
```

## Tool States

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

## Mandatory Safety Rules

Generated tools must:

- Never execute automatically after generation.
- Never inherit unrestricted host permissions.
- Declare every requested capability.
- Use an explicit input and output schema.
- Have a timeout and resource limits.
- Run tests before publication.
- Be reviewed or explicitly approved by a user.
- Be content-addressed or checksummed.
- Be versioned and immutable after publication.
- Record the model, prompt, and source inputs used to generate them.
- Be revocable.
- Be disabled by default for existing agents.

Generated tools must not:

- Read arbitrary files.
- access arbitrary network destinations,
- execute unrestricted shell commands,
- access secrets unless explicitly authorized,
- modify application configuration,
- install undeclared dependencies,
- create other tools recursively without approval.

## Initial Tool-Creation Scope

The first release should generate one of these controlled formats:

1. Declarative HTTP/OpenAPI tools
2. Sandboxed WebAssembly tools
3. Workflow-composition tools built from approved existing tools

Native dynamic libraries and unrestricted generated shell scripts are out of scope for the initial release.

---

# 75. Tool Sandbox

Untrusted tools must run outside the primary application process.

Preferred initial sandbox:

- WebAssembly/WASI runtime
- Capability-based filesystem access
- Network allowlists
- CPU and memory limits
- Execution timeout
- Output-size limit
- No inherited environment variables
- Explicit secret injection
- Temporary isolated working directory

Potential future isolation:

- Containers
- MicroVMs
- Remote isolated workers

A sandbox policy should support:

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
```

The application must fail closed if sandbox policy cannot be enforced.

---

# 76. Tool Permissions and Human Approval

Before every tool call, the runtime must evaluate:

```text
Agent permissions
      +
Tool requirements
      +
User permissions
      +
Workspace policy
      +
Run-specific approval
```

Possible decisions:

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
- Requested network hosts
- Requested secrets
- Timeout
- Risk level

Approval choices:

- Deny
- Allow once
- Allow for this run
- Allow for this agent and tool version
- Edit arguments and allow

Permanent wildcard approval for privileged operations should be discouraged.

---

# 77. MCP Integration

The system should support the Model Context Protocol as an interoperability layer.

MCP support should include:

- Local stdio servers
- Remote supported transports
- Tool discovery
- Resource discovery
- Prompt discovery where applicable
- Capability negotiation
- Connection health
- Authentication configuration
- Per-server permission policy
- Tool-name collision handling
- Timeouts
- Audit logs

MCP tools must pass through the same policy and approval engine as native tools.

An MCP server must not be trusted merely because it is locally installed.

---

# 78. Agent Memory

The platform should distinguish between different memory types.

## Conversation Memory

Messages in the current conversation.

## Working Memory

Temporary state for the current run.

## Episodic Memory

Summaries of previous tasks or runs.

## Semantic Memory

Long-term facts stored using retrieval.

## Artifact Memory

Files and structured outputs created during a task.

Memory must be configurable per agent and agency.

Required controls:

- Enable or disable each memory type
- Retention period
- Maximum size
- User deletion
- Export
- Workspace isolation
- Sensitive-data classification
- Retrieval filters
- Provenance
- Summarization policy

Agents must not silently promote temporary data into long-term memory.

---

# 79. Agent and RAG Integration

RAG must be available to agents through both:

1. A pipeline-integrated retrieval step
2. Explicit tools such as `rag.search` and `rag.query`

Agents should be able to:

- Select an authorized RAG system
- Search multiple knowledge bases
- Apply metadata filters
- Inspect citations
- Retrieve source chunks
- Request another retrieval pass
- Use query rewriting
- Compare results from multiple RAG systems

Every retrieved chunk supplied to an agent must retain:

- Knowledge-base ID
- Document ID
- Chunk ID
- Source location
- Retrieval score
- Reranking score
- Access-control information
- Content hash
- Ingestion version

---

# 80. RAG Security and Prompt-Injection Defense

Retrieved documents and tool outputs must be treated as untrusted data.

The runtime must distinguish:

```text
System instructions
Developer or workspace policy
User instructions
Tool instructions
Retrieved content
Tool output
```

Retrieved text must never automatically become privileged instructions.

Required protections:

- Clear instruction/data boundaries
- Content provenance
- Tool allowlists
- Retrieval access control
- Detection and logging of suspicious instructions
- Secret redaction
- Maximum context limits
- HTML and script sanitization
- Protection against indirect prompt injection
- Confirmation before sensitive side effects
- Policy checks independent from the LLM

Prompt-injection detection may assist policy enforcement but must not replace deterministic authorization.

---

# 81. Access-Control-Aware Retrieval

All retrieval must apply access-control filters before results are returned.

The system must prevent:

- Cross-user document leakage
- Cross-workspace leakage
- Retrieval from unauthorized collections
- Citation access to unauthorized source files
- Cached-result leakage
- Agent access beyond the initiating user's permissions

Authorization must be enforced by the application and storage layers, not by prompts.

For local single-user mode, the same model should remain available with a simplified policy.

---

# 82. Data Lifecycle and Index Consistency

Document ingestion must be idempotent and version-aware.

Required behaviors:

- Content-hash duplicate detection
- Source revision tracking
- Incremental re-indexing
- Embedding-model version tracking
- Chunker-version tracking
- Parser-version tracking
- Tombstones for deletion
- Vector deletion verification
- Retry-safe jobs
- Failed-index recovery
- Reconciliation between metadata and vector stores

Deleting a document must remove or tombstone:

- Source records
- Parsed content
- Chunks
- Embeddings
- Cached retrieval results
- Generated previews

The system should expose index-health and orphan-detection commands.

---

# 83. Conversation and Session Management

The system must support persistent and temporary conversations.

A conversation should include:

```text
id
workspace_id
user_id
title
mode
selected_model
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

Conversation modes:

- Direct model chat
- RAG chat
- Agent chat
- Agency task
- Workflow run

Users must be able to:

- Create a conversation
- Resume a conversation
- Fork a conversation
- Export a conversation
- Delete a conversation
- Retry a response
- Change models
- Inspect tool calls
- Inspect citations
- Inspect token and cost usage

---

# 84. TUI Agent Experience

The TUI must add these screens:

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
├── Create
├── Generated Drafts
├── Validate
├── Test
├── Approvals
├── Permissions
├── Versions
└── Revoke

Connections
├── Providers
├── Models
├── Test
├── Capabilities
└── Routing

Runs
├── Active
├── Awaiting Approval
├── Completed
├── Failed
└── Trace
```

## TUI Chat Layout

```text
┌──────────────────────────────────────────────────────────┐
│ Mode: Agent | Agent: Researcher | Model: balanced       │
│ RAG: project-docs | Budget: 4,210 / 20,000 tokens       │
├──────────────────────────────────────────────────────────┤
│ Conversation                                             │
│                                                          │
│ User: Find the authentication design and summarize it.  │
│                                                          │
│ Agent: Searching project documentation...               │
│                                                          │
│ Tool: rag.search                                         │
│ Status: Completed                                        │
│ Sources: 5                                               │
│                                                          │
│ Agent: The system uses... [1] [2]                       │
├──────────────────────────────────────────────────────────┤
│ Ask a question...                              [Send]   │
└──────────────────────────────────────────────────────────┘
```

Keyboard actions should include:

- Cancel run
- Approve or deny tool
- Expand tool arguments
- Inspect retrieved context
- Inspect execution trace
- Switch model
- Switch RAG
- Switch agent
- Fork conversation
- Copy response
- Open citation

---

# 85. Tool Builder UI

The TUI and Web GUI should provide a guided tool builder.

Required steps:

```text
1. Describe the desired tool
2. Define inputs and outputs
3. Choose implementation type
4. Select permissions
5. Generate implementation
6. Review generated source
7. Run validation
8. Run tests
9. Review security report
10. Approve and publish
11. Assign to agents
```

The UI must display source-code differences between generated revisions.

A generated tool may not be published if:

- Its schema is invalid.
- It does not compile.
- Required tests fail.
- Permissions are undeclared.
- The sandbox cannot enforce its policy.
- A critical security finding is unresolved.
- The approving user lacks permission.

---

# 86. Workflow Engine

The platform should support reusable workflows combining deterministic and agentic steps.

Initial node types:

- Model call
- RAG retrieval
- Agent invocation
- Tool invocation
- Condition
- Parallel branch
- Human approval
- Data transformation
- Final output

Workflow runs must support:

- Typed inputs and outputs
- Versioning
- Checkpoints
- Retries
- Cancellation
- Timeouts
- Approval gates
- Run traces
- Partial failure policies

Arbitrary cycles should be rejected unless the workflow declares an explicit maximum iteration count.

---

# 87. Budgets and Resource Governance

Every model, agent, agency, and workflow run should support budgets.

Budget types:

- Maximum input tokens
- Maximum output tokens
- Maximum total tokens
- Maximum estimated cost
- Maximum tool calls
- Maximum model calls
- Maximum retrieval calls
- Maximum runtime
- Maximum generated artifacts
- Maximum delegation depth

The runtime must stop safely when a hard budget is exceeded.

Soft limits should produce warnings.

Cost estimates must be labeled as estimates unless confirmed by provider billing data.

---

# 88. Audit Trail and Provenance

Security-relevant actions must generate immutable audit records.

Audit events include:

- Provider connection created or changed
- Secret reference changed
- Agent created or modified
- Permission changed
- Tool generated
- Tool approved
- Tool published
- Tool revoked
- Sensitive tool call approved or denied
- Document imported or deleted
- User or role changed

Generated outputs should record provenance:

```text
run_id
agent_version
agency_version
workflow_version
model_provider
model_name
prompt_template_version
tool_versions
rag_pipeline_version
document_versions
timestamps
token_usage
estimated_cost
```

Audit logs must not store plaintext secrets.

---

# 89. API Additions

Add versioned endpoints such as:

```text
GET    /api/v1/connections
POST   /api/v1/connections
GET    /api/v1/connections/:id
PUT    /api/v1/connections/:id
DELETE /api/v1/connections/:id
POST   /api/v1/connections/:id/test
GET    /api/v1/connections/:id/models

GET    /api/v1/agents
POST   /api/v1/agents
GET    /api/v1/agents/:id
PUT    /api/v1/agents/:id
DELETE /api/v1/agents/:id
POST   /api/v1/agents/:id/runs

GET    /api/v1/agencies
POST   /api/v1/agencies
GET    /api/v1/agencies/:id
PUT    /api/v1/agencies/:id
POST   /api/v1/agencies/:id/runs

GET    /api/v1/tools
POST   /api/v1/tools
GET    /api/v1/tools/:id
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

Use an idempotency key for operations that create runs, publish tools, or perform external side effects.

---

# 90. Additional Security Requirements

The product must use secure defaults.

Required controls:

- Server binds to localhost by default.
- Remote access requires explicit configuration.
- TLS is required for non-local production deployments.
- Authentication is required when exposed beyond localhost.
- Tool execution uses deny-by-default permissions.
- SSRF protection applies to HTTP tools and document importers.
- Filesystem tools prevent path traversal and symlink escapes.
- Archive extraction prevents zip-slip attacks.
- Database tools use read-only credentials by default.
- Shell execution is disabled by default.
- Secrets are scoped to specific tools and runs.
- Model inputs containing secrets are blocked or require explicit policy.
- Rate limits and request-size limits are configurable.
- Uploaded content is treated as untrusted.
- Dependencies and generated tools support vulnerability scanning.
- Audit records are protected from ordinary modification.

---

# 91. Reliability Requirements

Provider and tool calls must use:

- Configurable timeouts
- Bounded retries
- Exponential backoff
- Jitter
- Cancellation
- Concurrency limits
- Rate-limit handling
- Circuit breakers where appropriate

Retries must not repeat non-idempotent side effects unless the operation supports an idempotency key.

Crashes or restarts must not leave runs permanently marked as active.

The system must identify interrupted runs and either:

- Resume from a safe checkpoint
- Mark them as interrupted
- Request user action

---

# 92. Evaluation Requirements

Evaluation must cover RAG, agents, tools, and agencies.

## RAG Evaluation

- Retrieval recall
- Retrieval precision
- Citation correctness
- Faithfulness
- Answer relevance
- Access-control leakage
- Prompt-injection resistance

## Agent Evaluation

- Task success
- Step count
- Tool-selection accuracy
- Invalid tool-call rate
- Budget compliance
- Policy compliance
- Human-intervention rate
- Recovery from tool failure

## Tool Evaluation

- Schema compliance
- Correct output
- Error behavior
- Timeout behavior
- Permission enforcement
- Sandbox escape resistance
- Determinism where expected

## Agency Evaluation

- Delegation accuracy
- Duplicate work
- Completion rate
- Total latency
- Total token usage
- Total cost
- Final-answer quality

Evaluation datasets and results must be versioned.

---

# 93. Updated Project Structure

Add the following workspace components:

```text
crates/
├── agent-domain/
├── agent-runtime/
├── agency/
├── conversations/
├── model-router/
├── tools/
├── tool-sdk/
├── tool-registry/
├── tool-sandbox/
├── policy/
├── approvals/
├── workflows/
├── memory/
├── mcp/
├── audit/
└── evaluation/
```

Core domain crates must not depend on the TUI, web frontend, or concrete provider implementations.

---

# 94. Updated Development Phases

## Phase 0 — Foundations

- Workspace
- Domain model
- Configuration
- Secrets
- Persistence
- Logging
- CLI
- Migration framework

## Phase 1 — Model Connectivity

- OpenAI-compatible provider
- Ollama provider
- Streaming
- Capability discovery
- Connection testing
- Model routing
- TUI connection manager

## Phase 2 — Core RAG

- Document ingestion
- Versioned chunking
- Embeddings
- Local vector store
- Retrieval
- Citations
- Index reconciliation
- Basic evaluation

## Phase 3 — TUI Chat and RAG

- Conversation interface
- Model selection
- RAG selection
- Streaming output
- Citation inspection
- Run cancellation
- Trace inspection

## Phase 4 — Agent Runtime

- Agent definitions
- Tool calling
- Built-in read-only tools
- Execution events
- Budgets
- Permission engine
- Human approvals

## Phase 5 — Safe Tool Creation

- Tool manifests
- Declarative HTTP tools
- WASM sandbox
- Generation workflow
- Validation and testing
- Approval and publication
- Tool versioning and revocation

## Phase 6 — Multi-Agent Agencies

- Agency definitions
- Supervisor-and-worker pattern
- Delegation
- Shared run budget
- Agent-to-agent events
- Agency tracing

## Phase 7 — Server and Web GUI

- Versioned API
- Authentication
- RBAC
- Real-time events
- Agent and tool management
- Approval inbox
- Audit viewer

## Phase 8 — Production Hardening

- PostgreSQL
- Qdrant
- Docker
- TLS
- Backups
- OpenTelemetry
- Performance testing
- Security testing
- Disaster recovery

---

# 95. Revised MVP

The MVP should remain intentionally constrained.

## MVP Includes

- Local-first application
- TUI
- CLI
- SQLite metadata storage
- One local vector-store option
- OpenAI-compatible LLM connection
- Ollama connection
- Markdown, text, and PDF ingestion
- One configurable RAG pipeline
- Streaming RAG chat
- Citations
- One agent per run
- Built-in read-only RAG and document tools
- Tool-call inspection
- Tool approval framework
- Run cancellation
- Execution traces
- Connection diagnostics
- Secret storage through environment variables or OS keychain

## MVP Excludes

- Unrestricted generated native code
- Automatic publication of generated tools
- Arbitrary shell execution
- Dynamic creation of unrestricted agents
- Distributed workers
- Plugin marketplace
- Full multi-user collaboration
- Autonomous long-running agents
- Automatic spending without configured limits

## MVP Acceptance Workflow

```text
Install application
  ↓
Open TUI
  ↓
Create and test an LLM connection
  ↓
Create a knowledge base
  ↓
Import documents
  ↓
Index documents
  ↓
Create an agent
  ↓
Authorize the agent to use rag.search
  ↓
Ask the agent a question
  ↓
Inspect retrieval and tool calls
  ↓
Receive a streamed answer with citations
  ↓
Inspect execution trace and token usage
```

---

# 96. Tool-Creation Release Acceptance Criteria

Agent-assisted tool creation is complete only when a user can:

1. Describe a desired tool.
2. Review the generated input and output schemas.
3. Review requested permissions.
4. Generate the implementation.
5. Build it without modifying the host application.
6. Run automated tests in isolation.
7. Inspect validation and security results.
8. Approve or reject publication.
9. Assign the approved version to an agent.
10. Run the tool inside the sandbox.
11. Inspect its inputs, outputs, logs, duration, and resource usage.
12. Disable or revoke the tool immediately.

A generated tool that has not completed this lifecycle must not be available to production agents.

---

# 97. Product Success Metrics

Initial product metrics should include:

- Time from installation to first successful LLM response
- Time from installation to first indexed RAG query
- Percentage of answers with valid citations
- Retrieval latency
- End-to-end response latency
- Provider connection success rate
- Agent task completion rate
- Tool-call success rate
- Invalid tool-call rate
- Human approval rate
- Run cancellation success rate
- Index consistency error rate
- Number of security-policy violations prevented
- Crash-free run rate
- Cost and token usage per successful task

No user content, prompts, documents, or model responses may be collected as telemetry without explicit opt-in.

---

# 98. Additional Open Technical Decisions

The following decisions should be resolved through architecture decision records:

- TUI-to-core in-process interface versus API-only operation
- WebSocket versus Server-Sent Events for streaming
- WASM runtime selection
- MCP SDK and transport support
- Agent state-machine representation
- Workflow graph representation
- Policy engine implementation
- Tool-signing mechanism
- Secret-storage mechanism per operating system
- Local vector-store implementation
- Full-text search engine
- Model capability normalization
- Durable background-job implementation
- Conversation retention defaults
- Audit-log storage and integrity mechanism

Each decision must document:

- Context
- Options considered
- Decision
- Security implications
- Operational implications
- Migration strategy
- Reversal cost

---

# 99. Updated Architecture Principles

Add these mandatory principles:

14. Deny-by-default tool permissions  
15. Human control over external side effects  
16. Generated code is untrusted  
17. Retrieved content is untrusted  
18. Authorization is enforced outside the model  
19. Every agent action is attributable and auditable  
20. Every run has explicit resource limits  
21. Tool and agent definitions are versioned  
22. Provider capabilities are discovered, not assumed  
23. External side effects must be idempotent where possible  
24. Local data remains local unless the user explicitly selects a remote provider  
25. Security boundaries must not depend on prompt instructions  

---

# 100. Updated Definition of Done

In addition to the existing definition, an agent or tool feature is complete only when:

- Permission checks are implemented.
- Cancellation works.
- Timeouts are enforced.
- Resource limits are enforced.
- Inputs and outputs are schema-validated.
- Sensitive values are redacted.
- Audit events are generated.
- The feature works with streaming where applicable.
- Failure and retry behavior are tested.
- Prompt-injection scenarios are tested.
- Unauthorized filesystem and network access are tested.
- Tool side effects are documented.
- Agent, model, prompt, and tool versions are recorded.
- Generated code cannot bypass the sandbox.
- A user can inspect what the agent did and why.

---

# 101. PRD Change Log

## Version 0.2.0

Added:

- TUI LLM connection management
- Model capability discovery
- Model routing and fallback
- First-class agent runtime
- Multi-agent agencies
- Typed tool system
- Agent-assisted tool creation
- Sandboxed tool execution
- Human approval system
- MCP integration
- Agent memory
- Agent/RAG integration
- Prompt-injection defenses
- Access-control-aware retrieval
- Document and index lifecycle requirements
- Conversation management
- Agent-oriented TUI screens
- Workflow engine
- Budgets and governance
- Audit trails and provenance
- Agent and tool APIs
- Expanded security requirements
- Agent, tool, and agency evaluation
- Revised implementation phases
- Revised MVP and acceptance criteria
