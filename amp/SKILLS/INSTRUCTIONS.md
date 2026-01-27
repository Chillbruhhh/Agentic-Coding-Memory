# AGENTS.md Integration Guide

Instructions for integrating AMP skills into your project's AGENTS.md file.

## Recommended AGENTS.md Block

Add this to your project's `AGENTS.md` to enable AMP skill discovery:

```markdown
## Skills

Skills are stored in `amp/SKILLS/`. Load skills when relevant to your task.

### How to use skills

1. **Discover**: Read `amp/SKILLS/README.md` to see available skills
2. **Load**: When a task involves AMP tools, read `amp/SKILLS/amp-core/SKILL.md`
3. **Navigate**: Follow links in SKILL.md to reference docs as needed
4. **Progressive disclosure**: Don't load every file - only what you need

### When to load amp-core skill

Load `amp/SKILLS/amp-core/SKILL.md` when:
- Working with persistent memory or shared state
- Need to store/retrieve context across sessions
- Making architectural decisions worth recording
- Modifying code files (for provenance tracking)
- Coordinating with other agents on shared resources
- Searching existing knowledge (symbols, decisions, changesets)

### Skill file priorities

1. `SKILL.md` - Always read first (overview + navigation)
2. `references/tool-map.md` - Quick reference for which tool to use
3. `references/decision-guide.md` - When choosing between approaches
4. `references/tool-reference.md` - When need parameter details

**Task-specific guides:**
5. `references/artifact-guidelines.md` - When creating decisions/notes/changesets
6. `references/file-sync-guide.md` - When modifying files (provenance tracking)
7. `references/cache-guide.md` - When working with episodic cache
8. `references/workflows.md` - When following multi-step patterns
9. `references/examples.md` - When need concrete examples
```

## Alternative: Minimal Block

For simpler setups:

```markdown
## Skills

Load `amp/SKILLS/amp-core/SKILL.md` when working with AMP MCP tools.
Follow progressive disclosure: only read reference files as needed.
```

## Integration Points

### With Claude Code

Skills are automatically available. The agent can read skill files using the Read tool.

### With Custom Agents

Ensure your agent has filesystem access to the SKILLS directory and can follow the progressive disclosure pattern.

### With MCP Servers

Skills complement MCP tool schemas. While MCP provides tool definitions, skills provide:
- When-to-use guidance
- Workflow patterns
- Best practices
- Examples

## Best Practices

1. **Don't pre-load all skills** - Let the agent discover as needed
2. **Trust the agent** - It will navigate to relevant references
3. **Keep AGENTS.md lean** - Point to skills, don't duplicate content
4. **Update skills, not AGENTS.md** - Skill content is the source of truth
