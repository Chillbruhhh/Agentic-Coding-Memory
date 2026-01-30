# AMP Skills

Skills that guide AI agents on how to use AMP (Agentic Memory Protocol) tools effectively.

## What are Skills?

Skills are structured documentation packages that help agents:
- **Discover** available tools and their purposes
- **Decide** which tool to use for a given situation
- **Execute** tools correctly with proper parameters
- **Follow** proven workflows and best practices

Skills use **progressive disclosure**: SKILL.md provides overview, references/ contains deep details.

## How to Use Skills

1. **Load the skill**: Read `amp-core/SKILL.md` when working with AMP tools
2. **Navigate as needed**: Follow links to reference docs for details
3. **Don't load everything**: Only read reference files when needed

## Available Skills

### `amp-core`
Core guidance for the AMP MCP tools covering:
- **Episodic Memory Cache** (block-based rolling window)
- **File Provenance** (sync files across all memory layers)
- **Discovery & Search** (finding knowledge)
- **Writing Artifacts** (decisions, changesets, notes)
- **Focus Tracking** (session focus and outputs)

**Load when**: Working with persistent memory, shared state, or code provenance.

## Skill Structure

```
skills/
|-- README.md              # This file
|-- INSTRUCTIONS.md        # AGENTS.md integration guide
`-- amp-core/
    |-- SKILL.md           # Main skill file (load first)
    `-- references/
        |-- cache-guide.md         # Episodic memory & two-phase retrieval
        |-- file-sync-guide.md     # File sync across all layers
        |-- tool-reference.md      # Tool parameters
        |-- tool-map.md            # Quick reference for tool choice
        |-- artifact-guidelines.md # When/why to create artifacts
        |-- workflows.md           # Step-by-step patterns
        |-- examples.md            # Real-world examples
        `-- decision-guide.md      # Flowcharts for tool selection
```

## Quick Reference

| I want to... | Load this |
|--------------|-----------|
| Use any AMP tool | `amp-core/SKILL.md` |
| Understand episodic cache | `references/cache-guide.md` |
| Sync files after edits | `references/file-sync-guide.md` |
| See tool parameters | `references/tool-reference.md` |
| Know when to create artifacts | `references/artifact-guidelines.md` |
| Follow a workflow | `references/workflows.md` |

## Key Concepts

### Three Memory Layers
1. **Temporal** (FileLog): Audit trail, symbols, dependencies
2. **Vector** (Chunks + Embeddings): Semantic search
3. **Graph** (Relationships): depends_on, defined_in, calls

### Two-Phase Cache Retrieval
1. Search block summaries (~200 tokens each)
2. Fetch full blocks only when needed

This reduces context from 2000+ tokens to 200-400 for initial search.

## Creating New Skills

Follow the Anthropic agent skills pattern:

1. Create `skill-name/SKILL.md` with YAML frontmatter:
   ```yaml
   ---
   name: skill-name
   description: When to use this skill (1-2 sentences)
   ---
   ```

2. Keep SKILL.md concise (overview + navigation)
3. Put detailed content in `references/`
4. Trust the agent to navigate as needed

## Resources

- [Anthropic: Equipping Agents for the Real World](https://www.anthropic.com/engineering/equipping-agents-for-the-real-world-with-agent-skills)
