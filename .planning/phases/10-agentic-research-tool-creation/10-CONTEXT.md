# Phase 10: Agentic Research & Tool Creation - Context

**Gathered:** 2026-10-19
**Status:** Ready for planning

## Phase Boundary
Users can delegate research to controlled agents and create sandboxed tools; MVP acceptance passes. Agent runtime, tool sandbox, workflows, MVP acceptance workflow.

## Decisions
- Reuse existing crates: agency, agent-runtime, agent-domain, tool-sandbox, tool-sdk, tools, workflows, model-router, llm, memory, conversations.
- Retrieved content treated as untrusted data on every agent path.
- Tool lifecycle: describe → review → generate → isolated-test → security-review → approve → publish; revoke instantly.

## Deferred
- Full LLM provider integration depends on owner API keys.
- Production-grade sandbox isolation (use process-level for now).
