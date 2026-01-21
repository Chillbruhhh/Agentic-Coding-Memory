# Feature: Extend Tree-Sitter Language Support

The following plan should be complete, but its important that you validate documentation and codebase patterns and task sanity before you start implementing.

Pay special attention to naming of existing utils types and models. Import from the right files etc.

## Feature Description

Extend AMP's codebase parser to support additional programming languages beyond the current Python and TypeScript support. This enables comprehensive code intelligence, symbol extraction, and dependency analysis across polyglot projects. The implementation adds JavaScript, Rust, Go, and C# as Phase 1 priority languages, with a refactored architecture that makes adding future languages trivial.

## User Story

As a developer working on multi-language projects
I want AMP to parse and understand code in JavaScript, Rust, Go, and C#
So that I get accurate symbol indexing, dependency graphs, and AI-powered code understanding across my entire codebase

## Problem Statement

Currently AMP only parses Python and TypeScript files. All other languages are detected but return empty symbol lists with the note "Language 'X' not yet supported for parsing". This means:
- JavaScript/JSX files (huge web ecosystem) aren't parsed
- Rust files (AMP's own implementation language) aren't parsed
- Go and C# (popular backend languages) aren't parsed
- Users with polyglot codebases get incomplete knowledge graphs

## Solution Statement

Implement tree-sitter parsing support for JavaScript, Rust, Go, and C# by:
1. Adding tree-sitter grammar dependencies to Cargo.toml
2. Creating language-specific query patterns for symbol/import/export extraction
3. Extending CodebaseParser struct to handle new languages
4. Updating parse_file() and parse_codebase() match statements
5. Adding comprehensive tests for each new language

## Feature Metadata

**Feature Type**: New Capability
**Estimated Complexity**: Medium
**Primary Systems Affected**: amp/server/src/services/codebase_parser.rs, amp/server/Cargo.toml
**Dependencies**: tree-sitter-javascript, tree-sitter-rust, tree-sitter-go, tree-sitter-c-sharp

---

## CONTEXT REFERENCES

### Relevant Codebase Files IMPORTANT: YOU MUST READ THESE FILES BEFORE IMPLEMENTING!

- `amp/server/src/services/codebase_parser.rs` (lines 1-562) - Why: Core parser implementation, contains patterns for Python/TypeScript to mirror
- `amp/server/Cargo.toml` (lines 29-32) - Why: Shows tree-sitter dependency pattern
- `amp/spec/multi-language-parser-plan.md` (full file) - Why: Architecture design and query patterns for reference
- `amp/server/src/handlers/codebase.rs` (lines 642-689) - Why: Language detection function that already maps extensions
- `tree_sitter_languages.md` - Why: Reference list of available tree-sitter grammars

### New Files to Create

None - all changes are to existing files

### Files to Modify

- `amp/server/Cargo.toml` - Add new tree-sitter dependencies
- `amp/server/src/services/codebase_parser.rs` - Add language fields, queries, and parsing logic

### Relevant Documentation YOU SHOULD READ THESE BEFORE IMPLEMENTING!

- [Tree-sitter Query Syntax](https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries)
  - Specific section: Pattern syntax, captures, predicates
  - Why: Essential for writing correct query patterns
- [tree-sitter-javascript grammar](https://github.com/tree-sitter/tree-sitter-javascript)
  - Node types reference in src/node-types.json
  - Why: Shows exact node names for JavaScript queries
- [tree-sitter-rust grammar](https://github.com/nicholasklassen/tree-sitter-rust)
  - Node types for function_item, struct_item, impl_item, etc.
  - Why: Rust has unique constructs like traits and impls
- [tree-sitter-go grammar](https://github.com/tree-sitter/tree-sitter-go)
  - Node types for function_declaration, type_declaration, method_declaration
  - Why: Go has unique package/interface patterns
- [tree-sitter-c-sharp grammar](https://github.com/tree-sitter/tree-sitter-c-sharp)
  - Node types for class_declaration, method_declaration, namespace_declaration
  - Why: C# has namespaces, properties, and rich OOP constructs

### Patterns to Follow

**Language Field Pattern** (codebase_parser.rs:40-43):
```rust
pub struct CodebaseParser {
    python_language: Language,
    typescript_language: Language,
    // Add new languages here
}
```

**Language Initialization Pattern** (codebase_parser.rs:52-60):
```rust
pub fn new() -> Result<Self> {
    let python_language = tree_sitter_python::language();
    let typescript_language = tree_sitter_typescript::language_typescript();
    // Initialize new languages here
    Ok(Self { ... })
}
```

**Query Creation Pattern** (codebase_parser.rs:62-111):
```rust
fn create_python_queries(&self) -> Result<CodeQueries> {
    let symbols_query = Query::new(
        self.python_language,
        r#"
        (function_definition
          name: (identifier) @function.name) @function.definition
        // ... more patterns
        "#,
    )?;
    // imports_query, exports_query...
    Ok(CodeQueries { symbols, imports, exports })
}
```

**Parse File Match Pattern** (codebase_parser.rs:220-250):
```rust
let queries = match language {
    "python" => {
        parser.set_language(self.python_language)?;
        self.create_python_queries()?
    }
    "typescript" => {
        parser.set_language(self.typescript_language)?;
        self.create_typescript_queries()?
    }
    // Add new languages here
    _ => { /* unsupported fallback */ }
};
```

**Parse Codebase Extension Pattern** (codebase_parser.rs:195-207):
```rust
match ext_str.as_ref() {
    "py" => { /* parse python */ }
    "ts" | "tsx" => { /* parse typescript */ }
    // Add new extensions here
    _ => continue,
}
```

**Naming Conventions:**
- Language field: `{language}_language: Language`
- Query method: `create_{language}_queries(&self) -> Result<CodeQueries>`
- Use snake_case for Rust identifiers
- Query capture names: `{symbol_type}.name`, `{symbol_type}.definition`

**Error Handling:**
- Use `?` operator for fallible operations
- Query::new returns Result, propagate with `?`
- File read errors should be propagated

---

## IMPLEMENTATION PLAN

### Phase 1: Dependencies (5 minutes)

Add tree-sitter grammar crates for JavaScript, Rust, Go, and C# to Cargo.toml.

**Tasks:**
- Add 4 new tree-sitter dependencies
- Run cargo check to verify they resolve

### Phase 2: Language Fields (10 minutes)

Extend CodebaseParser struct with new language fields and initialize them.

**Tasks:**
- Add language fields to struct
- Initialize languages in new() constructor

### Phase 3: Query Implementations (45 minutes)

Create query patterns for each language to extract symbols, imports, and exports.

**Tasks:**
- Implement JavaScript queries (functions, classes, arrow functions, imports/exports)
- Implement Rust queries (functions, structs, enums, traits, impls, use statements)
- Implement Go queries (functions, types, methods, imports)
- Implement C# queries (classes, methods, properties, namespaces, using statements)

### Phase 4: Parser Integration (15 minutes)

Wire up the new languages in parse_file() and parse_codebase() methods.

**Tasks:**
- Add match arms for new languages in parse_file()
- Add extension matching in parse_codebase()

### Phase 5: Testing (30 minutes)

Add unit tests for each new language following existing test patterns.

**Tasks:**
- Test JavaScript parsing
- Test Rust parsing
- Test Go parsing
- Test C# parsing

---

## STEP-BY-STEP TASKS

IMPORTANT: Execute every task in order, top to bottom. Each task is atomic and independently testable.

### Task 1: UPDATE amp/server/Cargo.toml

- **IMPLEMENT**: Add tree-sitter dependencies for JavaScript, Rust, Go, C#
- **PATTERN**: Follow existing tree-sitter dependency format (lines 29-32)
- **LOCATION**: After line 32, add new dependencies
- **CODE**:
```toml
tree-sitter-javascript = "0.20"
tree-sitter-rust = "0.21"
tree-sitter-go = "0.20"
tree-sitter-c-sharp = "0.20"
```
- **GOTCHA**: tree-sitter-rust uses 0.21, others use 0.20. Check crates.io for latest compatible versions.
- **VALIDATE**: `cd amp/server && cargo check`

### Task 2: UPDATE CodebaseParser struct

- **IMPLEMENT**: Add language fields for JavaScript, Rust, Go, C#
- **PATTERN**: Mirror existing python_language, typescript_language fields (lines 40-43)
- **FILE**: amp/server/src/services/codebase_parser.rs
- **CODE**:
```rust
pub struct CodebaseParser {
    python_language: Language,
    typescript_language: Language,
    javascript_language: Language,
    rust_language: Language,
    go_language: Language,
    csharp_language: Language,
}
```
- **VALIDATE**: `cd amp/server && cargo check` (will fail until Task 3)

### Task 3: UPDATE CodebaseParser::new()

- **IMPLEMENT**: Initialize all new language fields
- **PATTERN**: Mirror existing initialization (lines 52-60)
- **IMPORTS**: Add at top of file if not present:
  - `// tree-sitter language imports happen via the crate bindings`
- **CODE**:
```rust
pub fn new() -> Result<Self> {
    let python_language = tree_sitter_python::language();
    let typescript_language = tree_sitter_typescript::language_typescript();
    let javascript_language = tree_sitter_javascript::language();
    let rust_language = tree_sitter_rust::language();
    let go_language = tree_sitter_go::language();
    let csharp_language = tree_sitter_c_sharp::language();

    Ok(Self {
        python_language,
        typescript_language,
        javascript_language,
        rust_language,
        go_language,
        csharp_language,
    })
}
```
- **VALIDATE**: `cd amp/server && cargo check`

### Task 4: CREATE JavaScript queries method

- **IMPLEMENT**: Add create_javascript_queries() method
- **PATTERN**: Mirror create_typescript_queries() (lines 114-181) - JavaScript is similar
- **LOCATION**: After create_typescript_queries() method
- **CODE**:
```rust
fn create_javascript_queries(&self) -> Result<CodeQueries> {
    let symbols_query = Query::new(
        self.javascript_language,
        r#"
        (function_declaration
          name: (identifier) @function.name) @function.definition

        (class_declaration
          name: (identifier) @class.name) @class.definition

        (variable_declaration
          (variable_declarator
            name: (identifier) @variable.name)) @variable.definition

        (method_definition
          name: (property_identifier) @method.name) @method.definition

        (arrow_function) @arrow_function.definition

        (assignment_expression
          left: (identifier) @variable.name
          right: [(arrow_function) (function_expression)]) @variable.definition
        "#,
    )?;

    let imports_query = Query::new(
        self.javascript_language,
        r#"
        (import_statement
          source: (string) @import.source)

        (import_statement
          (import_clause
            (named_imports
              (import_specifier
                name: (identifier) @import.name))))

        (call_expression
          function: (identifier) @func (#eq? @func "require")
          arguments: (arguments (string) @import.source))
        "#,
    )?;

    let exports_query = Query::new(
        self.javascript_language,
        r#"
        (export_statement
          (function_declaration
            name: (identifier) @export.name))

        (export_statement
          (class_declaration
            name: (identifier) @export.name))

        (export_statement
          declaration: (lexical_declaration
            (variable_declarator
              name: (identifier) @export.name)))
        "#,
    )?;

    Ok(CodeQueries {
        symbols: symbols_query,
        imports: imports_query,
        exports: exports_query,
    })
}
```
- **GOTCHA**: JavaScript uses `identifier` for class names, TypeScript uses `type_identifier`
- **VALIDATE**: `cd amp/server && cargo check`

### Task 5: CREATE Rust queries method

- **IMPLEMENT**: Add create_rust_queries() method
- **PATTERN**: Similar structure but Rust-specific node types
- **LOCATION**: After create_javascript_queries()
- **CODE**:
```rust
fn create_rust_queries(&self) -> Result<CodeQueries> {
    let symbols_query = Query::new(
        self.rust_language,
        r#"
        (function_item
          name: (identifier) @function.name) @function.definition

        (struct_item
          name: (type_identifier) @struct.name) @struct.definition

        (enum_item
          name: (type_identifier) @enum.name) @enum.definition

        (trait_item
          name: (type_identifier) @trait.name) @trait.definition

        (impl_item
          type: (type_identifier) @impl.name) @impl.definition

        (const_item
          name: (identifier) @constant.name) @constant.definition

        (static_item
          name: (identifier) @static.name) @static.definition

        (type_item
          name: (type_identifier) @type.name) @type.definition

        (mod_item
          name: (identifier) @module.name) @module.definition
        "#,
    )?;

    let imports_query = Query::new(
        self.rust_language,
        r#"
        (use_declaration
          argument: (_) @import.path)

        (extern_crate_declaration
          name: (identifier) @import.crate)
        "#,
    )?;

    let exports_query = Query::new(
        self.rust_language,
        r#"
        (function_item
          (visibility_modifier) @vis
          name: (identifier) @export.name)

        (struct_item
          (visibility_modifier) @vis
          name: (type_identifier) @export.name)

        (enum_item
          (visibility_modifier) @vis
          name: (type_identifier) @export.name)
        "#,
    )?;

    Ok(CodeQueries {
        symbols: symbols_query,
        imports: imports_query,
        exports: exports_query,
    })
}
```
- **GOTCHA**: Rust uses `type_identifier` for type names, `identifier` for function/variable names
- **VALIDATE**: `cd amp/server && cargo check`

### Task 6: CREATE Go queries method

- **IMPLEMENT**: Add create_go_queries() method
- **LOCATION**: After create_rust_queries()
- **CODE**:
```rust
fn create_go_queries(&self) -> Result<CodeQueries> {
    let symbols_query = Query::new(
        self.go_language,
        r#"
        (function_declaration
          name: (identifier) @function.name) @function.definition

        (method_declaration
          name: (field_identifier) @method.name) @method.definition

        (type_declaration
          (type_spec
            name: (type_identifier) @type.name)) @type.definition

        (const_declaration
          (const_spec
            name: (identifier) @constant.name)) @constant.definition

        (var_declaration
          (var_spec
            name: (identifier) @variable.name)) @variable.definition
        "#,
    )?;

    let imports_query = Query::new(
        self.go_language,
        r#"
        (import_declaration
          (import_spec
            path: (interpreted_string_literal) @import.path))

        (import_declaration
          (import_spec_list
            (import_spec
              path: (interpreted_string_literal) @import.path)))
        "#,
    )?;

    let exports_query = Query::new(
        self.go_language,
        r#"
        (function_declaration
          name: (identifier) @export.name
          (#match? @export.name "^[A-Z]"))

        (type_declaration
          (type_spec
            name: (type_identifier) @export.name
            (#match? @export.name "^[A-Z]")))
        "#,
    )?;

    Ok(CodeQueries {
        symbols: symbols_query,
        imports: imports_query,
        exports: exports_query,
    })
}
```
- **GOTCHA**: Go exports are determined by capitalization - names starting with uppercase are public. The `#match?` predicate may not work in all tree-sitter versions; if it fails, remove the predicate and filter in Rust code instead.
- **VALIDATE**: `cd amp/server && cargo check`

### Task 7: CREATE C# queries method

- **IMPLEMENT**: Add create_csharp_queries() method
- **LOCATION**: After create_go_queries()
- **CODE**:
```rust
fn create_csharp_queries(&self) -> Result<CodeQueries> {
    let symbols_query = Query::new(
        self.csharp_language,
        r#"
        (class_declaration
          name: (identifier) @class.name) @class.definition

        (struct_declaration
          name: (identifier) @struct.name) @struct.definition

        (interface_declaration
          name: (identifier) @interface.name) @interface.definition

        (method_declaration
          name: (identifier) @method.name) @method.definition

        (property_declaration
          name: (identifier) @property.name) @property.definition

        (field_declaration
          (variable_declaration
            (variable_declarator
              (identifier) @field.name))) @field.definition

        (enum_declaration
          name: (identifier) @enum.name) @enum.definition

        (namespace_declaration
          name: (_) @namespace.name) @namespace.definition
        "#,
    )?;

    let imports_query = Query::new(
        self.csharp_language,
        r#"
        (using_directive
          (identifier) @import.name)

        (using_directive
          (qualified_name) @import.name)
        "#,
    )?;

    let exports_query = Query::new(
        self.csharp_language,
        r#"
        (class_declaration
          (modifier) @mod (#eq? @mod "public")
          name: (identifier) @export.name)

        (method_declaration
          (modifier) @mod (#eq? @mod "public")
          name: (identifier) @export.name)
        "#,
    )?;

    Ok(CodeQueries {
        symbols: symbols_query,
        imports: imports_query,
        exports: exports_query,
    })
}
```
- **GOTCHA**: C# modifiers may need adjustment based on actual tree-sitter-c-sharp node structure. Check the grammar if queries fail.
- **VALIDATE**: `cd amp/server && cargo check`

### Task 8: UPDATE parse_file() match statement

- **IMPLEMENT**: Add match arms for new languages
- **PATTERN**: Mirror existing Python/TypeScript arms (lines 220-250)
- **LOCATION**: In parse_file() method, extend the match statement
- **CODE**: Add these arms before the `_ =>` fallback:
```rust
"javascript" => {
    parser.set_language(self.javascript_language)?;
    self.create_javascript_queries()?
}
"rust" => {
    parser.set_language(self.rust_language)?;
    self.create_rust_queries()?
}
"go" => {
    parser.set_language(self.go_language)?;
    self.create_go_queries()?
}
"csharp" => {
    parser.set_language(self.csharp_language)?;
    self.create_csharp_queries()?
}
```
- **VALIDATE**: `cd amp/server && cargo check`

### Task 9: UPDATE parse_codebase() extension matching

- **IMPLEMENT**: Add extension matching for new languages
- **PATTERN**: Mirror existing py/ts/tsx matching (lines 195-207)
- **LOCATION**: In parse_codebase() match statement
- **CODE**: Add these arms:
```rust
"js" | "jsx" | "mjs" | "cjs" => {
    if let Ok(file_log) = self.parse_file(path, "javascript") {
        file_logs.insert(path.to_string_lossy().to_string(), file_log);
    }
}
"rs" => {
    if let Ok(file_log) = self.parse_file(path, "rust") {
        file_logs.insert(path.to_string_lossy().to_string(), file_log);
    }
}
"go" => {
    if let Ok(file_log) = self.parse_file(path, "go") {
        file_logs.insert(path.to_string_lossy().to_string(), file_log);
    }
}
"cs" => {
    if let Ok(file_log) = self.parse_file(path, "csharp") {
        file_logs.insert(path.to_string_lossy().to_string(), file_log);
    }
}
```
- **VALIDATE**: `cd amp/server && cargo check`

### Task 10: ADD JavaScript test

- **IMPLEMENT**: Add test_parse_javascript_file() test
- **PATTERN**: Mirror test_parse_python_file() (lines 468-491)
- **LOCATION**: In #[cfg(test)] mod tests section
- **CODE**:
```rust
#[test]
fn test_parse_javascript_file() {
    let parser = CodebaseParser::new().unwrap();

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.js");
    std::fs::write(&file_path, r#"
import { useState } from 'react';
const axios = require('axios');

function fetchData(url) {
    return axios.get(url);
}

class DataService {
    constructor() {
        this.cache = {};
    }

    async get(key) {
        return this.cache[key];
    }
}

export const helper = () => {};
export function exported() {}
"#).unwrap();

    let file_log = parser.parse_file(&file_path, "javascript").unwrap();

    assert_eq!(file_log.language, "javascript");
    assert!(file_log.symbols.len() >= 3, "Expected at least 3 symbols, got {}", file_log.symbols.len());
    assert!(file_log.dependencies.imports.len() >= 1, "Expected at least 1 import");
}
```
- **VALIDATE**: `cd amp/server && cargo test test_parse_javascript_file`

### Task 11: ADD Rust test

- **IMPLEMENT**: Add test_parse_rust_file() test
- **CODE**:
```rust
#[test]
fn test_parse_rust_file() {
    let parser = CodebaseParser::new().unwrap();

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.rs");
    std::fs::write(&file_path, r#"
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub struct Config {
    pub name: String,
    pub value: i32,
}

pub enum Status {
    Active,
    Inactive,
}

pub trait Processor {
    fn process(&self) -> Result<(), Error>;
}

impl Processor for Config {
    fn process(&self) -> Result<(), Error> {
        Ok(())
    }
}

pub fn main() {
    println!("Hello");
}

const MAX_SIZE: usize = 100;
"#).unwrap();

    let file_log = parser.parse_file(&file_path, "rust").unwrap();

    assert_eq!(file_log.language, "rust");
    assert!(file_log.symbols.len() >= 4, "Expected at least 4 symbols (struct, enum, trait, fn), got {}", file_log.symbols.len());
    assert!(file_log.dependencies.imports.len() >= 1, "Expected at least 1 import");
}
```
- **VALIDATE**: `cd amp/server && cargo test test_parse_rust_file`

### Task 12: ADD Go test

- **IMPLEMENT**: Add test_parse_go_file() test
- **CODE**:
```rust
#[test]
fn test_parse_go_file() {
    let parser = CodebaseParser::new().unwrap();

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.go");
    std::fs::write(&file_path, r#"
package main

import (
    "fmt"
    "net/http"
)

type Server struct {
    Port int
    Host string
}

func NewServer(port int) *Server {
    return &Server{Port: port}
}

func (s *Server) Start() error {
    return http.ListenAndServe(fmt.Sprintf(":%d", s.Port), nil)
}

const MaxConnections = 100

var globalConfig = &Server{}
"#).unwrap();

    let file_log = parser.parse_file(&file_path, "go").unwrap();

    assert_eq!(file_log.language, "go");
    assert!(file_log.symbols.len() >= 3, "Expected at least 3 symbols, got {}", file_log.symbols.len());
    assert!(file_log.dependencies.imports.len() >= 1, "Expected at least 1 import");
}
```
- **VALIDATE**: `cd amp/server && cargo test test_parse_go_file`

### Task 13: ADD C# test

- **IMPLEMENT**: Add test_parse_csharp_file() test
- **CODE**:
```rust
#[test]
fn test_parse_csharp_file() {
    let parser = CodebaseParser::new().unwrap();

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.cs");
    std::fs::write(&file_path, r#"
using System;
using System.Collections.Generic;

namespace MyApp.Services
{
    public interface IUserService
    {
        User GetUser(int id);
    }

    public class UserService : IUserService
    {
        private readonly Dictionary<int, User> _cache;

        public string Name { get; set; }

        public UserService()
        {
            _cache = new Dictionary<int, User>();
        }

        public User GetUser(int id)
        {
            return _cache[id];
        }
    }

    public struct User
    {
        public int Id;
        public string Name;
    }

    public enum UserRole
    {
        Admin,
        User
    }
}
"#).unwrap();

    let file_log = parser.parse_file(&file_path, "csharp").unwrap();

    assert_eq!(file_log.language, "csharp");
    assert!(file_log.symbols.len() >= 4, "Expected at least 4 symbols (interface, class, struct, enum), got {}", file_log.symbols.len());
    assert!(file_log.dependencies.imports.len() >= 1, "Expected at least 1 import");
}
```
- **VALIDATE**: `cd amp/server && cargo test test_parse_csharp_file`

### Task 14: RUN full test suite

- **IMPLEMENT**: Run all tests to ensure no regressions
- **VALIDATE**: `cd amp/server && cargo test`

### Task 15: BUILD and verify

- **IMPLEMENT**: Full build to ensure everything compiles
- **VALIDATE**: `cd amp/server && cargo build`

---

## TESTING STRATEGY

### Unit Tests

Based on existing test patterns in codebase_parser.rs:
- Each language gets a dedicated test function
- Tests create temporary files with representative code
- Assertions verify:
  - Correct language detection
  - Symbol extraction (at least N expected symbols)
  - Import extraction (at least 1 import)
  - No panics on valid code

### Integration Tests

After unit tests pass:
- Run `amp index` on a multi-language codebase
- Verify symbols appear in the knowledge graph
- Confirm no parsing errors in logs

### Edge Cases

- Empty files (should return empty symbols list)
- Syntax errors (should gracefully degrade)
- Very large files (performance)
- Unicode identifiers (if supported by language)
- Nested structures (classes in classes, closures)

---

## VALIDATION COMMANDS

Execute every command to ensure zero regressions and 100% feature correctness.

### Level 1: Syntax & Style

```bash
cd amp/server && cargo fmt --check
cd amp/server && cargo clippy
```

### Level 2: Unit Tests

```bash
cd amp/server && cargo test test_parse_javascript_file
cd amp/server && cargo test test_parse_rust_file
cd amp/server && cargo test test_parse_go_file
cd amp/server && cargo test test_parse_csharp_file
cd amp/server && cargo test  # All tests
```

### Level 3: Build Verification

```bash
cd amp/server && cargo build
cd amp/server && cargo build --release
```

### Level 4: Manual Validation

Create test files and verify parsing:
```bash
# Test JavaScript
echo 'function test() {}' > /tmp/test.js
# Run amp-server and call parse endpoint

# Test Rust
echo 'fn main() {}' > /tmp/test.rs
# Verify through API
```

---

## ACCEPTANCE CRITERIA

- [ ] JavaScript files (.js, .jsx, .mjs, .cjs) are parsed with symbol extraction
- [ ] Rust files (.rs) are parsed with symbol extraction
- [ ] Go files (.go) are parsed with symbol extraction
- [ ] C# files (.cs) are parsed with symbol extraction
- [ ] All 4 new language tests pass
- [ ] Existing Python/TypeScript tests still pass (no regressions)
- [ ] `cargo build` succeeds without errors
- [ ] `cargo clippy` has no warnings
- [ ] Knowledge graph shows symbols from all supported languages

---

## COMPLETION CHECKLIST

- [ ] All tasks completed in order (Tasks 1-15)
- [ ] Each task validation passed immediately
- [ ] All validation commands executed successfully
- [ ] Full test suite passes (unit tests)
- [ ] No linting or type checking errors
- [ ] Manual testing confirms features work
- [ ] Acceptance criteria all met

---

## NOTES

### Design Decisions

1. **Hardcoded languages vs. registry pattern**: This implementation adds languages directly to the struct. The spec/multi-language-parser-plan.md describes a more extensible registry pattern for future phases. This approach is chosen for simplicity and to match existing patterns.

2. **Query complexity**: The queries focus on major constructs (functions, classes, imports). Edge cases like nested closures or complex generics may not be captured. This is acceptable for initial implementation.

3. **Go export detection**: Go uses capitalization for exports. The `#match?` predicate may not work; if so, filter in Rust code.

4. **C# complexity**: C# has many constructs (properties, events, delegates). Initial queries cover the most common ones.

### Future Work (Not in this plan)

- Add Phase 2 languages (Swift, Kotlin, etc.)
- Refactor to registry pattern for easier extension
- Add external query files (.scm) for maintainability
- Performance optimization for large codebases

### Risk Mitigations

- **Query syntax errors**: Validate each query independently before combining
- **Grammar version mismatches**: Pin tree-sitter versions to 0.20.x
- **Missing node types**: Check grammar source if queries don't match
