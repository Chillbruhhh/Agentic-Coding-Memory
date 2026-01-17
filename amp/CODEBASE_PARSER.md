# AMP Codebase Parser

## Overview

The AMP Codebase Parser is a Tree-sitter based system that analyzes codebases and creates structured knowledge graphs with persistent file logs. It supports Python and TypeScript initially, with extensible architecture for additional languages.

## Features

### 🔍 **Multi-Language Parsing**
- **Python**: Functions, classes, variables, imports, exports
- **TypeScript**: Functions, classes, interfaces, types, methods, imports, exports
- **Extensible**: Easy to add new languages via Tree-sitter grammars

### 📊 **Symbol Extraction**
- Function definitions with signatures
- Class and interface declarations
- Variable assignments and type definitions
- Method definitions within classes
- Import and export statements

### 📝 **File Log Generation**
- Structured Markdown format optimized for embeddings
- Content hash tracking for change detection
- Symbol snapshots with line numbers
- Dependency mapping (imports/exports)
- Change history with linked decisions/changesets
- Notes and linked architectural decisions

### 🔗 **AMP Integration**
- Automatic Symbol object creation in AMP database
- File log objects with vector embeddings
- Links to Decision and ChangeSet objects
- Project and tenant isolation

## API Endpoints

### Parse Entire Codebase
```http
POST /v1/codebase/parse
Content-Type: application/json

{
    "root_path": "/path/to/project",
    "project_id": "my-project",
    "tenant_id": "my-tenant"
}
```

**Response:**
```json
{
    "success": true,
    "files_parsed": 15,
    "file_logs": {
        "/path/to/file.py": {
            "path": "/path/to/file.py",
            "language": "python",
            "symbols": [...],
            "dependencies": {...}
        }
    },
    "errors": []
}
```

### Parse Single File
```http
POST /v1/codebase/parse-file
Content-Type: application/json

{
    "file_path": "/path/to/file.py",
    "language": "python",
    "project_id": "my-project",
    "tenant_id": "my-tenant"
}
```

### Update File Log
```http
POST /v1/codebase/update-file-log
Content-Type: application/json

{
    "file_path": "/path/to/file.py",
    "change_description": "Added error handling",
    "changeset_id": "cs_001",
    "run_id": "run_001",
    "decision_id": "dec_001"
}
```

### Get File Logs
```http
GET /v1/codebase/file-logs?project_id=my-project&language=python&limit=10
```

### Get Specific File Log
```http
GET /v1/codebase/file-logs/{encoded_file_path}
```

## File Log Format

The file log uses a structured Markdown format optimized for vector embeddings:

```markdown
# FILE_LOG v1
path: src/auth/jwt.ts
language: typescript
last_indexed: 2026-01-17T10:30:00Z
content_hash: sha256:abc123...

## Symbols
- function: authenticateUser (lines 15-25)
- function: issueTokens (lines 27-35)
- class: JwtVerifier (lines 37-55)

## Dependencies
imports:
- react
- axios
- ./types/User

exports:
- UserService
- createUserService

## Recent Changes
- 2026-01-17 · Added error handling and logging (run: run_001, cs: cs_001)
- 2026-01-16 · Migrated to new auth system (decision: dec_001)

## Linked Decisions
- dec_001 · Use JWT + refresh token rotation

## Notes
- Main authentication module
- Requires security review for changes
```

## Usage Examples

### Basic Codebase Analysis
```bash
# Parse entire project
curl -X POST http://localhost:8105/v1/codebase/parse \\
  -H "Content-Type: application/json" \\
  -d '{
    "root_path": "/path/to/project",
    "project_id": "my-project"
  }'
```

### Track File Changes
```bash
# Update file log after making changes
curl -X POST http://localhost:8105/v1/codebase/update-file-log \\
  -H "Content-Type: application/json" \\
  -d '{
    "file_path": "/path/to/file.py",
    "change_description": "Refactored authentication logic",
    "changeset_id": "cs_123",
    "decision_id": "dec_456"
  }'
```

### Integration with AI Agents
```python
import requests

# Parse file and get structured data
response = requests.post('http://localhost:8105/v1/codebase/parse-file', json={
    'file_path': 'src/main.py',
    'language': 'python'
})

file_log = response.json()['file_log']
markdown = response.json()['markdown']

# Use markdown for AI context
ai_context = f"File analysis:\\n{markdown}"

# Update after AI makes changes
requests.post('http://localhost:8105/v1/codebase/update-file-log', json={
    'file_path': 'src/main.py',
    'change_description': 'AI refactored error handling',
    'run_id': 'ai_run_789'
})
```

## Supported Languages

### Python
- **Symbols**: Functions, classes, variables
- **Dependencies**: `import`, `from ... import`
- **Queries**: Function definitions, class definitions, assignments

### TypeScript
- **Symbols**: Functions, classes, interfaces, types, methods
- **Dependencies**: `import`, `export`
- **Queries**: Function declarations, class declarations, interface declarations

### Adding New Languages

1. Add Tree-sitter grammar dependency to `Cargo.toml`
2. Add language function to `codebase_parser.rs`
3. Create language-specific queries for symbols and dependencies
4. Update `detect_language()` function for file extension mapping

Example for adding Rust support:
```rust
extern "C" {
    fn tree_sitter_rust() -> Language;
}

fn create_rust_queries(language: &Language) -> Result<CodeQueries> {
    let symbols_query = Query::new(
        language,
        r#"
        (function_item
          name: (identifier) @function.name) @function.definition
        
        (struct_item
          name: (type_identifier) @struct.name) @struct.definition
        "#,
    )?;
    // ... more queries
}
```

## Testing

Run the comprehensive test suite:

```bash
# PowerShell
./amp/scripts/test-codebase-parser.ps1

# Bash
./amp/scripts/test-codebase-parser.sh
```

The test creates sample Python and TypeScript files, parses them, and validates:
- Symbol extraction accuracy
- Dependency detection
- Markdown generation
- File log updates
- Change tracking

## Performance Considerations

- **Incremental Parsing**: Only re-parse files when content hash changes
- **Parallel Processing**: Use `walkdir` for concurrent file processing
- **Memory Efficiency**: Stream large codebases rather than loading all in memory
- **Caching**: Store parsed results in AMP database for quick retrieval

## Integration with AMP

The codebase parser integrates seamlessly with AMP's memory system:

1. **Symbol Objects**: Each extracted symbol becomes a Symbol object in AMP
2. **File Log Objects**: File logs are stored with vector embeddings for semantic search
3. **Relationships**: Automatic relationship creation between symbols and files
4. **Change Tracking**: Links to ChangeSet and Decision objects for full traceability

This creates a comprehensive knowledge graph of your codebase that AI agents can query and reason about effectively.
