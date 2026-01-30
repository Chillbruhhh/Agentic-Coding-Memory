# What is AMP?

The Agentic Memory Protocol (AMP) is a vendor-neutral protocol for persistent, shared memory in AI agent development.

## The Problem

AI coding agents today operate in isolation:

- **No Memory**: Agents forget everything between sessions
- **No Coordination**: Multiple agents can't share knowledge
- **No Context**: Agents don't understand project history
- **No Traceability**: Can't track why decisions were made

This leads to:
- Repeated work and redundant analysis
- Conflicting changes from different agents
- Lost architectural context
- Inability to learn from past mistakes

## The Solution

AMP provides a unified memory substrate that enables:

### 1. Persistent Memory

Agents can store and retrieve information across sessions:

```
Session 1: Agent indexes codebase, stores symbols
Session 2: Different agent queries symbols, understands structure
Session 3: Another agent builds on previous work
```

### 2. Shared Knowledge

Multiple agents access the same memory:

```
Agent A: "I'm refactoring the auth module"
Agent B: Queries AMP, sees Agent A's work
Agent B: "I'll work on the API module instead"
```

### 3. Hybrid Retrieval

Find information using multiple methods:

- **Vector Search**: Semantic similarity ("find authentication code")
- **Graph Traversal**: Relationships ("what calls this function?")
- **Temporal Filtering**: Time-based ("changes in last week")

### 4. Coordination

Agents coordinate to avoid conflicts:

```
Agent A: Acquires lease on "auth.py"
Agent B: Tries to modify "auth.py", sees lease
Agent B: Waits or works on different file
```

## Core Concepts

### Memory Objects

AMP stores four types of memory objects:

**Symbol** - Code structure
```json
{
  "type": "symbol",
  "name": "authenticate_user",
  "kind": "function",
  "path": "src/auth.py",
  "language": "python"
}
```

**Decision** - Architectural choices
```json
{
  "type": "decision",
  "title": "Use JWT for authentication",
  "context": "Need stateless auth",
  "decision": "Implement JWT tokens",
  "consequences": "Requires token refresh logic"
}
```

**ChangeSet** - Code modifications
```json
{
  "type": "changeset",
  "description": "Add password hashing",
  "files_changed": ["src/auth.py"],
  "diff_summary": "+15 -3 lines"
}
```

**Run** - Agent executions
```json
{
  "type": "run",
  "input_summary": "Refactor authentication",
  "outputs": ["Modified 3 files", "Added tests"],
  "duration_ms": 45000
}
```

### Provenance Tracking

Every object tracks its origin:

```json
{
  "provenance": {
    "agent": "claude-code",
    "summary": "Indexed during codebase scan",
    "timestamp": "2026-01-27T10:30:00Z"
  }
}
```

## How It Works

### 1. Indexing Phase

Agent scans codebase and creates symbols:

```
amp index /path/to/project
→ Parses files with tree-sitter
→ Extracts functions, classes, imports
→ Generates embeddings
→ Stores in AMP
```

### 2. Query Phase

Agent searches for relevant information:

```
amp query "authentication logic"
→ Vector search finds semantic matches
→ Graph traversal finds related code
→ Returns ranked results with explanations
```

### 3. Modification Phase

Agent makes changes and records them:

```
Agent modifies auth.py
→ Creates ChangeSet object
→ Updates file log
→ Links to related Decision
→ Stores in AMP
```

### 4. Coordination Phase

Multiple agents work together:

```
Agent A: Acquires lease on "auth module"
Agent B: Checks leases, sees Agent A working
Agent B: Works on different module
Agent A: Releases lease when done
```

## Architecture

```
┌─────────────────────────────────────┐
│     AI Agents (Claude, Cursor)     │
│   (via MCP or direct API calls)    │
├─────────────────────────────────────┤
│         MCP Server (Port 8106)      │
│    (13 tools for AI agents)        │
├─────────────────────────────────────┤
│      AMP Server (Port 8105)         │
│           REST API                  │
├─────────────────────────────────────┤
│    Storage Layer (SurrealDB)        │
│  Vector + Graph + Document store   │
└─────────────────────────────────────┘
```

## Key Features

### Multi-Language Support

Parses 10 programming languages:
- Python, TypeScript, JavaScript
- Rust, Go, C#, Java
- C, C++, Ruby

### Flexible Deployment

AMP uses SurrealDB as its storage layer, providing:
- **Vector search** for semantic similarity
- **Graph traversal** for relationships
- **Document storage** for structured data

Configure via Docker (recommended) or the UI Settings tab.

### Embedding Providers

- **Ollama**: Local, free, private
- **OpenAI**: Cloud, high-quality
- **OpenRouter**: Cloud, multiple models

### Multiple Interfaces

- **CLI**: Command-line tool
- **Desktop UI**: Tauri application
- **MCP**: AI agent integration
- **REST API**: Direct HTTP access

## Use Cases

### Solo Development

```
Day 1: Index codebase
Day 2: Query for patterns
Day 3: Make changes, record decisions
Day 4: Review history, understand context
```

### Team Development

```
Developer A: Makes architectural decision
Developer B: Queries AMP, sees decision
Developer C: Builds on previous work
All: Share understanding of codebase
```

### AI Agent Workflows

```
Agent 1: Analyzes codebase structure
Agent 2: Identifies refactoring opportunities
Agent 3: Implements changes
Agent 4: Generates documentation
All: Coordinate via AMP
```

### Open Source Maintenance

```
Maintainer: Records design decisions
Contributor: Queries AMP for context
New contributor: Understands project quickly
All: Build shared knowledge base
```

## Benefits

### For Developers

- Faster onboarding to new codebases
- Better understanding of project history
- Reduced context switching
- Improved code quality

### For AI Agents

- Persistent memory across sessions
- Shared knowledge between agents
- Coordination to avoid conflicts
- Traceability of actions

### For Teams

- Shared understanding of architecture
- Documented decision rationale
- Reduced knowledge silos
- Better collaboration

## Comparison to Alternatives

### vs. Git

- **Git**: Version control for code
- **AMP**: Memory and context for agents
- **Together**: Git tracks changes, AMP tracks understanding

### vs. Documentation

- **Docs**: Static, manually written
- **AMP**: Dynamic, automatically generated
- **Together**: Docs explain intent, AMP provides structure

### vs. Code Search

- **Search**: Find text matches
- **AMP**: Semantic understanding + relationships
- **Together**: Search finds code, AMP understands it

### vs. Vector Databases

- **Vector DB**: Similarity search only
- **AMP**: Vector + Graph + Temporal + Coordination
- **Together**: Vector DB is one component of AMP

## Getting Started

Ready to use AMP? Start here:

1. [Quick Start Guide](../getting-started/quick-start.md) - Get running in 5 minutes
2. [Installation](../getting-started/installation.md) - Detailed setup
3. [First Steps](../getting-started/first-steps.md) - Your first project
4. [MCP Integration](../guides/agents/mcp-integration.md) - Connect AI agents

## Learn More

- [Memory Objects](memory-objects.md) - Deep dive into object types
- [Hybrid Retrieval](hybrid-retrieval.md) - How search works
- [Architecture](architecture.md) - System design details
