# Multi-Language Parser Implementation Plan

## Current State
**Implemented Languages:** Python, TypeScript
**Architecture:** Single `CodebaseParser` struct with hardcoded language support

## Objective
Extend AMP's codebase parser to support 20+ programming languages using Tree-sitter grammars, enabling comprehensive code intelligence across polyglot projects.

## Priority Languages (Phase 1)
Based on usage frequency and ecosystem importance:

1. **JavaScript** (`.js`, `.jsx`) - Essential for web development
2. **Rust** (`.rs`) - AMP's own implementation language
3. **Go** (`.go`) - Popular for backend services
4. **Java** (`.java`) - Enterprise standard
5. **C/C++** (`.c`, `.cpp`, `.h`, `.hpp`) - Systems programming
6. **C#** (`.cs`) - .NET ecosystem
7. **Ruby** (`.rb`) - Rails and scripting
8. **PHP** (`.php`) - Web development

## Secondary Languages (Phase 2)
9. **Swift** (`.swift`) - iOS/macOS development
10. **Kotlin** (`.kt`) - Android development
11. **Scala** (`.scala`) - JVM functional programming
12. **Elixir** (`.ex`, `.exs`) - Erlang VM
13. **Haskell** (`.hs`) - Functional programming
14. **Lua** (`.lua`) - Embedded scripting
15. **Bash** (`.sh`) - Shell scripting

## Markup & Config Languages (Phase 3)
16. **JSON** (`.json`) - Configuration
17. **YAML** (`.yaml`, `.yml`) - Configuration
18. **TOML** (`.toml`) - Configuration
19. **HTML** (`.html`) - Web markup
20. **CSS/SCSS** (`.css`, `.scss`) - Styling

## Architecture Redesign

### Current Architecture Issues
- Hardcoded language fields in struct
- Separate query creation methods per language
- Manual file extension matching
- No extensibility for new languages

### Proposed Architecture

```rust
pub struct CodebaseParser {
    languages: HashMap<String, LanguageConfig>,
}

pub struct LanguageConfig {
    language: Language,
    queries: LanguageQueries,
    extensions: Vec<String>,
    metadata: LanguageMetadata,
}

pub struct LanguageQueries {
    symbols: Query,
    imports: Query,
    exports: Query,
}

pub struct LanguageMetadata {
    name: String,
    display_name: String,
    comment_syntax: CommentSyntax,
    supports_classes: bool,
    supports_interfaces: bool,
    supports_modules: bool,
}

pub enum CommentSyntax {
    CStyle { line: &'static str, block: (&'static str, &'static str) },
    HashStyle { line: &'static str },
    Custom { line: &'static str, block: Option<(&'static str, &'static str)> },
}
```

### Language Registry Pattern

```rust
impl CodebaseParser {
    pub fn new() -> Result<Self> {
        let mut languages = HashMap::new();
        
        // Register all supported languages
        Self::register_python(&mut languages)?;
        Self::register_typescript(&mut languages)?;
        Self::register_javascript(&mut languages)?;
        Self::register_rust(&mut languages)?;
        // ... more languages
        
        Ok(Self { languages })
    }
    
    fn register_python(languages: &mut HashMap<String, LanguageConfig>) -> Result<()> {
        let language = tree_sitter_python::language();
        let queries = Self::create_python_queries(&language)?;
        
        languages.insert("python".to_string(), LanguageConfig {
            language,
            queries,
            extensions: vec!["py".to_string()],
            metadata: LanguageMetadata {
                name: "python".to_string(),
                display_name: "Python".to_string(),
                comment_syntax: CommentSyntax::HashStyle { line: "#" },
                supports_classes: true,
                supports_interfaces: false,
                supports_modules: true,
            },
        });
        
        Ok(())
    }
    
    pub fn detect_language(&self, file_path: &Path) -> Option<&str> {
        let extension = file_path.extension()?.to_str()?;
        
        for (lang_name, config) in &self.languages {
            if config.extensions.contains(&extension.to_string()) {
                return Some(lang_name);
            }
        }
        
        None
    }
}
```

## Query Pattern Templates

### Universal Symbol Patterns
Most languages share common constructs:
- Functions/Methods
- Classes/Structs
- Variables/Constants
- Types/Interfaces
- Imports/Exports

### Language-Specific Query Files
Create separate query files for maintainability:
```
amp/server/src/queries/
├── python.scm
├── typescript.scm
├── javascript.scm
├── rust.scm
├── go.scm
└── ...
```

Example `rust.scm`:
```scheme
; Functions
(function_item
  name: (identifier) @function.name) @function.definition

; Structs
(struct_item
  name: (type_identifier) @struct.name) @struct.definition

; Enums
(enum_item
  name: (type_identifier) @enum.name) @enum.definition

; Traits
(trait_item
  name: (type_identifier) @trait.name) @trait.definition

; Impl blocks
(impl_item
  type: (type_identifier) @impl.type) @impl.definition

; Use statements (imports)
(use_declaration
  argument: (scoped_identifier) @import.name)

; Pub items (exports)
(visibility_modifier) @export.modifier
```

## Implementation Steps

### Step 1: Refactor Core Architecture (2-3 hours)
1. Create `LanguageConfig` and related structs
2. Implement language registry pattern
3. Refactor `parse_file()` to use registry
4. Update tests for new architecture

### Step 2: Add Query File System (1-2 hours)
1. Create `queries/` directory
2. Move existing Python/TypeScript queries to `.scm` files
3. Implement query file loader
4. Add query validation

### Step 3: Implement Phase 1 Languages (4-6 hours)
1. Add Cargo dependencies for each language
2. Create query files for each language
3. Register languages in parser
4. Write tests for each language
5. Update documentation

### Step 4: Add Language Detection (1 hour)
1. Build extension-to-language mapping
2. Implement MIME type detection (optional)
3. Add shebang detection for scripts
4. Handle ambiguous extensions (`.h` for C/C++/Objective-C)

### Step 5: Implement Phase 2 & 3 (6-8 hours)
1. Add remaining languages incrementally
2. Test with real-world codebases
3. Optimize query performance
4. Document language-specific features

## Cargo.toml Updates

```toml
# Phase 1 Languages
tree-sitter-javascript = "0.20"
tree-sitter-rust = "0.20"
tree-sitter-go = "0.20"
tree-sitter-java = "0.20"
tree-sitter-c = "0.20"
tree-sitter-cpp = "0.20"
tree-sitter-c-sharp = "0.20"
tree-sitter-ruby = "0.20"
tree-sitter-php = "0.20"

# Phase 2 Languages
tree-sitter-swift = "0.20"
tree-sitter-kotlin = "0.20"
tree-sitter-scala = "0.20"
tree-sitter-elixir = "0.20"
tree-sitter-haskell = "0.20"
tree-sitter-lua = "0.20"
tree-sitter-bash = "0.20"

# Phase 3 Languages
tree-sitter-json = "0.20"
tree-sitter-yaml = "0.20"
tree-sitter-toml = "0.20"
tree-sitter-html = "0.20"
tree-sitter-css = "0.20"
```

## Testing Strategy

### Unit Tests
- Test each language parser independently
- Verify symbol extraction accuracy
- Test import/export detection
- Validate edge cases (nested classes, closures, etc.)

### Integration Tests
- Parse real-world codebases
- Test polyglot projects (multiple languages)
- Benchmark parsing performance
- Validate memory usage

### Test Files
Create minimal test files for each language:
```
amp/server/tests/fixtures/
├── test.py
├── test.ts
├── test.js
├── test.rs
├── test.go
└── ...
```

## Performance Considerations

### Lazy Loading
- Load language parsers on-demand
- Cache parsed trees for incremental updates
- Use parallel parsing for large codebases

### Memory Management
- Limit concurrent parsers
- Stream large files instead of loading entirely
- Implement parser pool for reuse

### Optimization Targets
- Parse 1000 files/second on average hardware
- Support codebases up to 100k files
- Memory usage under 500MB for typical projects

## API Changes

### New Endpoints
```
GET /v1/languages
- Returns list of supported languages with metadata

GET /v1/parse/preview?language=rust
- Returns example parsed output for a language

POST /v1/parse/validate
- Validates code syntax without full parsing
```

### Updated Responses
```json
{
  "file_log": {
    "path": "src/main.rs",
    "language": "rust",
    "language_version": "2021",
    "parser_version": "0.20",
    "symbols": [...],
    "dependencies": {...}
  }
}
```

## Documentation Updates

### User Documentation
- List of supported languages
- Language-specific features and limitations
- Query customization guide
- Performance tuning tips

### Developer Documentation
- Adding new language support guide
- Query pattern reference
- Testing guidelines
- Contribution workflow

## Success Metrics

### Coverage
- Support 20+ languages by end of implementation
- Parse 95%+ of common language constructs
- Handle 90%+ of real-world code patterns

### Performance
- Parse speed: 1000+ files/second
- Memory efficiency: <500MB for 10k files
- Incremental parsing: <100ms for typical file changes

### Quality
- Test coverage: 80%+ for parser code
- Zero crashes on malformed input
- Graceful degradation for unsupported constructs

## Risks & Mitigations

### Risk: Tree-sitter Grammar Quality Varies
**Mitigation:** Test with real codebases, contribute fixes upstream, maintain fallback parsing

### Risk: Query Complexity for Some Languages
**Mitigation:** Start with basic queries, iterate based on user feedback, document limitations

### Risk: Performance Degradation with Many Languages
**Mitigation:** Lazy loading, parser pooling, profiling and optimization

### Risk: Maintenance Burden
**Mitigation:** Automated testing, community contributions, clear documentation

## Timeline

- **Week 1:** Architecture refactor + query system (Steps 1-2)
- **Week 2:** Phase 1 languages (Step 3)
- **Week 3:** Language detection + Phase 2 (Steps 4-5)
- **Week 4:** Phase 3 + testing + documentation
- **Week 5:** Performance optimization + polish

**Total Estimated Time:** 80-100 hours

## Next Steps

1. Review and approve this plan
2. Create GitHub issues for each phase
3. Set up feature branch: `feature/multi-language-parser`
4. Begin Step 1: Architecture refactor
5. Implement incrementally with continuous testing

---

**Status:** Planning Complete - Ready for Implementation
**Owner:** TBD
**Priority:** High
**Dependencies:** None
