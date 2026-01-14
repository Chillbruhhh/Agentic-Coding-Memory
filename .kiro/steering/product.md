# Product Overview

## Product Purpose
Agent Memory Protocol (AMP) is a vendor-neutral protocol for durable, unified memory in agentic software development. AMP solves the critical problem of coding agents operating in isolation without shared memory of codebase structure, decisions made, changes over time, and context that should guide future actions. By providing a standardized memory substrate, AMP enables any agent to persist, retrieve, and reason over structured project memory using hybrid retrieval methods.

## Target Users
**Primary Users**: Developers building AI agents and coding automation systems
- AI agent developers who need persistent memory across sessions
- Teams building custom coding assistants and automation pipelines  
- Developers working with multiple AI tools (Cursor, Claude Code, Codex, custom CLI agents)
- Open source maintainers managing complex codebases with AI assistance

**User Needs**:
- Shared understanding of project structure and history across different agents
- Persistent memory of architectural decisions and their rationale
- Ability to track what changed, when, and why
- Coordination between multiple agents working on the same project

## Key Features
- **Unified Project State**: Agents share consistent understanding of structure, history, and intent
- **Hybrid Memory Retrieval**: Combines vector similarity, graph traversal, and temporal filtering
- **Protocol Portability**: Language-neutral HTTP + JSON API usable from any framework
- **Deterministic Traceability**: Every retrieval explains why memory was returned and its origin
- **Coordination Primitives**: Lease-based system for multi-agent coordination
- **Four Core Memory Objects**: Symbols (code structure), Decisions (architecture choices), ChangeSets (modifications), Runs (agent executions)

## Business Objectives
**Hackathon Goals**:
- Demonstrate protocol viability with working implementation
- Show clear value proposition for agent memory coordination
- Build foundation for future enterprise adoption
- Create compelling demo showcasing hybrid retrieval capabilities

**Success Metrics**:
- Complete protocol specification with OpenAPI documentation
- Working Rust server with SurrealDB backend
- Generated Python and TypeScript SDKs
- Successful demo of agent memory persistence and retrieval

## User Journey
1. **Setup**: Developer integrates AMP client SDK into their agent or automation system
2. **Indexing**: Agent scans codebase and creates Symbol objects for functions, classes, modules
3. **Decision Making**: Agent stores Decision objects when making architectural choices
4. **Implementation**: Agent creates ChangeSet objects when modifying code
5. **Execution Tracking**: Agent logs Run objects for each execution with inputs/outputs
6. **Memory Retrieval**: Agent queries memory using text, vectors, or graph relationships
7. **Coordination**: Multiple agents coordinate using lease system for shared resources

## Success Criteria
**Hackathon Success**:
- ✅ Complete object schemas (Symbol, Decision, ChangeSet, Run)
- ✅ Full OpenAPI v1 specification with all endpoints
- ✅ SurrealDB schema with vector indexing and graph relations
- ✅ Rust server skeleton with Axum and proper routing
- ✅ Working demo script showing end-to-end workflow
- ✅ Example usage in Python and TypeScript

**Long-term Success**:
- Adoption by major AI coding tools and agent frameworks
- Community contributions and ecosystem growth
- Enterprise deployments with advanced coordination features
- Performance benchmarks showing improved agent effectiveness
