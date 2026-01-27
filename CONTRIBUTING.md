# Contributing to AMP

We welcome contributions! Please follow these guidelines to help maintain code quality and consistency.

## Getting Started

1. **Fork the repository** and clone locally
2. **Run the stack**: `cd amp && docker compose up`
3. **Verify everything works**: Open `http://localhost:8109`

## Development Workflow

1. **Create a feature branch**: `git checkout -b feature/your-feature`
2. **Make changes** following the code style below
3. **Test your changes**: `cargo test --workspace`
4. **Update documentation** if needed (README, SKILLS/, DEVLOG.md)
5. **Submit a PR** with a clear description

## Code Style

| Language | Style |
|----------|-------|
| **Rust** | `cargo fmt` before committing, follow clippy suggestions |
| **TypeScript** | ESLint + Prettier, functional components preferred |
| **Commits** | Conventional commits (`feat:`, `fix:`, `docs:`, `refactor:`) |

### Commit Message Format

```
<type>: <short description>

[optional body]

[optional footer]
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation only
- `refactor` - Code change that neither fixes a bug nor adds a feature
- `test` - Adding or updating tests
- `chore` - Maintenance tasks

**Examples:**
```
feat: add amp_file_sync project auto-creation
fix: resolve path ambiguity in file lookup
docs: update SKILLS documentation for artifacts
refactor: consolidate cache block handlers
```

## What to Contribute

### Bug Fixes
- Open an issue first describing the bug
- Reference the issue in your PR
- Include a test that reproduces the bug

### New MCP Tools
- Add to `amp/mcp-server/src/tools/`
- Register in `amp/mcp-server/src/main.rs`
- Update `SKILLS/amp-core/references/tool-reference.md`
- Update `SKILLS/amp-core/references/tool-map.md`

### Language Parsers
- Add to `amp/server/src/services/parser/`
- Follow existing parser patterns (tree-sitter based)
- Add language to supported list in README

### UI Improvements
- React components in `amp/ui/src/`
- Follow existing component patterns
- Use TailwindCSS for styling

### Documentation
- `SKILLS/` - Agent integration documentation
- `README.md` - User-facing documentation
- `DEVLOG.md` - Development timeline and decisions
- Inline comments for complex logic

## Pull Request Guidelines

1. **One feature per PR** - Keep PRs focused and reviewable
2. **Include tests** - For new functionality
3. **Update DEVLOG.md** - Document what you changed and why
4. **Link related issues** - Reference any related GitHub issues
5. **Describe your changes** - What, why, and any trade-offs

### PR Template

```markdown
## Summary
Brief description of changes

## Changes
- Change 1
- Change 2

## Testing
How to test these changes

## Related Issues
Fixes #123
```

## Architecture Notes

Understanding these constraints will help your contributions align with the project:

- **SurrealDB** is the only supported database (vector + graph + document in one)
- **Embeddings** require OpenAI, OpenRouter, or Ollama
- **MCP tools** are the primary agent interface (13 tools)
- **Settings** are managed via UI Settings tab or environment variables
- **Three memory layers**: Temporal (FileLog), Vector (Chunks), Graph (Relationships)

## Project Structure

```
amp/
├── server/       # Rust API server (Axum + SurrealDB)
├── cli/          # Terminal CLI (Ratatui + Tree-sitter)
├── ui/           # React/Tauri desktop UI
├── mcp-server/   # MCP integration for AI agents
└── spec/         # OpenAPI + JSON schemas

SKILLS/           # Agent documentation (load via amp-core skill)
```

## Testing

```bash
# Run all tests
cargo test --workspace

# Run specific component tests
cd amp/server && cargo test
cd amp/cli && cargo test
cd amp/mcp-server && cargo test

# Run with logging
RUST_LOG=debug cargo test
```

## Questions?

- **Issues**: Open a GitHub issue for bugs or feature requests
- **Documentation**: Check [SKILLS/](SKILLS/) for detailed agent integration docs
- **Architecture**: See [DEVLOG.md](DEVLOG.md) for design decisions

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (AMP Community License v1.0).
