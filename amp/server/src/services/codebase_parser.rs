use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::{Language, Parser, Query, QueryCursor, Tree};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSymbol {
    pub name: String,
    pub symbol_type: String,
    pub start_line: usize,
    pub end_line: usize,
    pub start_byte: usize,
    pub end_byte: usize,
    pub file_path: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLog {
    pub path: String,
    pub language: String,
    pub last_indexed: String,
    pub content_hash: String,
    pub symbols: Vec<ParsedSymbol>,
    pub dependencies: FileDependencies,
    pub recent_changes: Vec<String>,
    pub linked_decisions: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDependencies {
    pub imports: Vec<String>,
    pub exports: Vec<String>,
}

pub struct CodebaseParser {
    python_language: Language,
    typescript_language: Language,
    javascript_language: Language,
    rust_language: Language,
    go_language: Language,
    csharp_language: Language,
    java_language: Language,
    c_language: Language,
    cpp_language: Language,
    ruby_language: Language,
}

struct CodeQueries {
    symbols: Query,
    imports: Query,
    exports: Query,
}

impl CodebaseParser {
    pub fn new() -> Result<Self> {
        let python_language = tree_sitter_python::language();
        let typescript_language = tree_sitter_typescript::language_typescript();
        let javascript_language = tree_sitter_javascript::language();
        let rust_language = tree_sitter_rust::language();
        let go_language = tree_sitter_go::language();
        let csharp_language = tree_sitter_c_sharp::language();
        let java_language = tree_sitter_java::language();
        let c_language = tree_sitter_c::language();
        let cpp_language = tree_sitter_cpp::language();
        let ruby_language = tree_sitter_ruby::language();

        Ok(Self {
            python_language,
            typescript_language,
            javascript_language,
            rust_language,
            go_language,
            csharp_language,
            java_language,
            c_language,
            cpp_language,
            ruby_language,
        })
    }

    fn create_python_queries(&self) -> Result<CodeQueries> {
        let symbols_query = Query::new(
            self.python_language,
            r#"
            (function_definition
              name: (identifier) @function.name) @function.definition
            
            (class_definition
              name: (identifier) @class.name) @class.definition
            
            (assignment
              left: (identifier) @variable.name) @variable.definition
            
            (function_definition
              body: (block
                (function_definition
                  name: (identifier) @method.name))) @method.definition
            "#,
        )?;

        let imports_query = Query::new(
            self.python_language,
            r#"
            (import_statement
              name: (dotted_name) @import.name)
            
            (import_from_statement
              module_name: (dotted_name) @import.module)
            
            (import_from_statement
              name: (dotted_name) @import.name)
            "#,
        )?;

        let exports_query = Query::new(
            self.python_language,
            r#"
            (function_definition
              name: (identifier) @export.name)
            
            (class_definition
              name: (identifier) @export.name)
            "#,
        )?;

        Ok(CodeQueries {
            symbols: symbols_query,
            imports: imports_query,
            exports: exports_query,
        })
    }

    fn create_typescript_queries(&self) -> Result<CodeQueries> {
        let symbols_query = Query::new(
            self.typescript_language,
            r#"
            (function_declaration
              name: (identifier) @function.name) @function.definition
            
            (class_declaration
              name: (type_identifier) @class.name) @class.definition
            
            (interface_declaration
              name: (type_identifier) @interface.name) @interface.definition
            
            (type_alias_declaration
              name: (type_identifier) @type.name) @type.definition
            
            (variable_declaration
              (variable_declarator
                name: (identifier) @variable.name)) @variable.definition
            
            (method_definition
              name: (property_identifier) @method.name) @method.definition
            
            (arrow_function) @arrow_function.definition
            
            (assignment_expression
              left: (identifier) @variable.name
              right: (arrow_function)) @variable.definition
            "#,
        )?;

        let imports_query = Query::new(
            self.typescript_language,
            r#"
            (import_statement
              source: (string) @import.source)
            
            (import_statement
              (import_clause
                (named_imports
                  (import_specifier
                    name: (identifier) @import.name))))
            "#,
        )?;

        let exports_query = Query::new(
            self.typescript_language,
            r#"
            (export_statement
              (function_declaration
                name: (identifier) @export.name))
            
            (export_statement
              (class_declaration
                name: (type_identifier) @export.name))
            
            (export_statement
              (interface_declaration
                name: (type_identifier) @export.name))
            "#,
        )?;

        Ok(CodeQueries {
            symbols: symbols_query,
            imports: imports_query,
            exports: exports_query,
        })
    }

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
              right: (arrow_function)) @variable.definition
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
              (visibility_modifier)
              name: (identifier) @export.name)

            (struct_item
              (visibility_modifier)
              name: (type_identifier) @export.name)

            (enum_item
              (visibility_modifier)
              name: (type_identifier) @export.name)
            "#,
        )?;

        Ok(CodeQueries {
            symbols: symbols_query,
            imports: imports_query,
            exports: exports_query,
        })
    }

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
              name: (identifier) @export.name)

            (type_declaration
              (type_spec
                name: (type_identifier) @export.name))
            "#,
        )?;

        Ok(CodeQueries {
            symbols: symbols_query,
            imports: imports_query,
            exports: exports_query,
        })
    }

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
              name: (identifier) @export.name)

            (method_declaration
              name: (identifier) @export.name)
            "#,
        )?;

        Ok(CodeQueries {
            symbols: symbols_query,
            imports: imports_query,
            exports: exports_query,
        })
    }

    fn create_java_queries(&self) -> Result<CodeQueries> {
        let symbols_query = Query::new(
            self.java_language,
            r#"
            (class_declaration
              name: (identifier) @class.name) @class.definition

            (interface_declaration
              name: (identifier) @interface.name) @interface.definition

            (method_declaration
              name: (identifier) @method.name) @method.definition

            (constructor_declaration
              name: (identifier) @constructor.name) @constructor.definition

            (field_declaration
              declarator: (variable_declarator
                name: (identifier) @field.name)) @field.definition

            (enum_declaration
              name: (identifier) @enum.name) @enum.definition
            "#,
        )?;

        let imports_query = Query::new(
            self.java_language,
            r#"
            (import_declaration
              (scoped_identifier) @import.name)

            (import_declaration
              (identifier) @import.name)

            (package_declaration
              (scoped_identifier) @package.name)
            "#,
        )?;

        let exports_query = Query::new(
            self.java_language,
            r#"
            (class_declaration
              name: (identifier) @export.name)

            (interface_declaration
              name: (identifier) @export.name)

            (method_declaration
              name: (identifier) @export.name)
            "#,
        )?;

        Ok(CodeQueries {
            symbols: symbols_query,
            imports: imports_query,
            exports: exports_query,
        })
    }

    fn create_c_queries(&self) -> Result<CodeQueries> {
        let symbols_query = Query::new(
            self.c_language,
            r#"
            (function_definition
              declarator: (function_declarator
                declarator: (identifier) @function.name)) @function.definition

            (struct_specifier
              name: (type_identifier) @struct.name) @struct.definition

            (enum_specifier
              name: (type_identifier) @enum.name) @enum.definition

            (declaration
              declarator: (init_declarator
                declarator: (identifier) @variable.name)) @variable.definition

            (type_definition
              declarator: (type_identifier) @typedef.name) @typedef.definition
            "#,
        )?;

        let imports_query = Query::new(
            self.c_language,
            r#"
            (preproc_include
              path: (string_literal) @include.path)

            (preproc_include
              path: (system_lib_string) @include.path)
            "#,
        )?;

        let exports_query = Query::new(
            self.c_language,
            r#"
            (function_definition
              declarator: (function_declarator
                declarator: (identifier) @export.name))
            "#,
        )?;

        Ok(CodeQueries {
            symbols: symbols_query,
            imports: imports_query,
            exports: exports_query,
        })
    }

    fn create_cpp_queries(&self) -> Result<CodeQueries> {
        let symbols_query = Query::new(
            self.cpp_language,
            r#"
            (function_definition
              declarator: (function_declarator
                declarator: (identifier) @function.name)) @function.definition

            (function_definition
              declarator: (function_declarator
                declarator: (qualified_identifier
                  name: (identifier) @function.name))) @function.definition

            (class_specifier
              name: (type_identifier) @class.name) @class.definition

            (struct_specifier
              name: (type_identifier) @struct.name) @struct.definition

            (enum_specifier
              name: (type_identifier) @enum.name) @enum.definition

            (namespace_definition
              name: (namespace_identifier) @namespace.name) @namespace.definition

            (template_declaration) @template.definition
            "#,
        )?;

        let imports_query = Query::new(
            self.cpp_language,
            r#"
            (preproc_include
              path: (string_literal) @include.path)

            (preproc_include
              path: (system_lib_string) @include.path)

            (using_declaration
              (identifier) @using.name)

            (using_declaration
              (qualified_identifier) @using.name)
            "#,
        )?;

        let exports_query = Query::new(
            self.cpp_language,
            r#"
            (function_definition
              declarator: (function_declarator
                declarator: (identifier) @export.name))

            (class_specifier
              name: (type_identifier) @export.name)
            "#,
        )?;

        Ok(CodeQueries {
            symbols: symbols_query,
            imports: imports_query,
            exports: exports_query,
        })
    }

    fn create_ruby_queries(&self) -> Result<CodeQueries> {
        let symbols_query = Query::new(
            self.ruby_language,
            r#"
            (method
              name: (identifier) @method.name) @method.definition

            (singleton_method
              name: (identifier) @method.name) @method.definition

            (class
              name: (constant) @class.name) @class.definition

            (module
              name: (constant) @module.name) @module.definition

            (assignment
              left: (identifier) @variable.name) @variable.definition

            (assignment
              left: (constant) @constant.name) @constant.definition
            "#,
        )?;

        let imports_query = Query::new(
            self.ruby_language,
            r#"
            (call
              method: (identifier) @method (#eq? @method "require")
              arguments: (argument_list (string (string_content) @import.name)))

            (call
              method: (identifier) @method (#eq? @method "require_relative")
              arguments: (argument_list (string (string_content) @import.name)))

            (call
              method: (identifier) @method (#eq? @method "load")
              arguments: (argument_list (string (string_content) @import.name)))
            "#,
        )?;

        let exports_query = Query::new(
            self.ruby_language,
            r#"
            (class
              name: (constant) @export.name)

            (module
              name: (constant) @export.name)

            (method
              name: (identifier) @export.name)
            "#,
        )?;

        Ok(CodeQueries {
            symbols: symbols_query,
            imports: imports_query,
            exports: exports_query,
        })
    }

    pub fn parse_codebase(&self, root_path: &Path) -> Result<HashMap<String, FileLog>> {
        let mut file_logs = HashMap::new();

        for entry in WalkDir::new(root_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    let ext_str = extension.to_string_lossy();
                    match ext_str.as_ref() {
                        "py" => {
                            if let Ok(file_log) = self.parse_file(path, "python") {
                                file_logs.insert(path.to_string_lossy().to_string(), file_log);
                            }
                        }
                        "ts" | "tsx" => {
                            if let Ok(file_log) = self.parse_file(path, "typescript") {
                                file_logs.insert(path.to_string_lossy().to_string(), file_log);
                            }
                        }
                        "js" | "jsx" => {
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
                        "java" => {
                            if let Ok(file_log) = self.parse_file(path, "java") {
                                file_logs.insert(path.to_string_lossy().to_string(), file_log);
                            }
                        }
                        "c" | "h" => {
                            if let Ok(file_log) = self.parse_file(path, "c") {
                                file_logs.insert(path.to_string_lossy().to_string(), file_log);
                            }
                        }
                        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => {
                            if let Ok(file_log) = self.parse_file(path, "cpp") {
                                file_logs.insert(path.to_string_lossy().to_string(), file_log);
                            }
                        }
                        "rb" | "rake" | "gemspec" => {
                            if let Ok(file_log) = self.parse_file(path, "ruby") {
                                file_logs.insert(path.to_string_lossy().to_string(), file_log);
                            }
                        }
                        _ => continue,
                    }
                }
            }
        }

        Ok(file_logs)
    }

    pub fn parse_file(&self, file_path: &Path, language: &str) -> Result<FileLog> {
        let content = std::fs::read_to_string(file_path)?;
        let content_hash = self.compute_hash(&content);

        let mut parser = Parser::new();
        let queries = match language {
            "python" => {
                parser.set_language(self.python_language)?;
                self.create_python_queries()?
            }
            "typescript" => {
                parser.set_language(self.typescript_language)?;
                self.create_typescript_queries()?
            }
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
            "java" => {
                parser.set_language(self.java_language)?;
                self.create_java_queries()?
            }
            "c" => {
                parser.set_language(self.c_language)?;
                self.create_c_queries()?
            }
            "cpp" => {
                parser.set_language(self.cpp_language)?;
                self.create_cpp_queries()?
            }
            "ruby" => {
                parser.set_language(self.ruby_language)?;
                self.create_ruby_queries()?
            }
            _ => {
                // For unsupported languages, return a basic file log without parsing
                let mut hasher = Sha256::new();
                hasher.update(&content);
                let hash = format!("{:x}", hasher.finalize());

                return Ok(FileLog {
                    path: file_path.to_string_lossy().to_string(),
                    language: language.to_string(),
                    last_indexed: chrono::Utc::now().to_rfc3339(),
                    content_hash: hash,
                    symbols: Vec::new(),
                    dependencies: FileDependencies {
                        imports: Vec::new(),
                        exports: Vec::new(),
                    },
                    recent_changes: Vec::new(),
                    linked_decisions: Vec::new(),
                    notes: vec![format!(
                        "Language '{}' not yet supported for parsing",
                        language
                    )],
                });
            }
        };

        let tree = parser
            .parse(&content, None)
            .ok_or_else(|| anyhow!("Failed to parse file: {}", file_path.display()))?;

        let symbols = self.extract_symbols(&tree, &content, &queries, file_path, language)?;
        let dependencies = self.extract_dependencies(&tree, &content, &queries)?;

        Ok(FileLog {
            path: file_path.to_string_lossy().to_string(),
            language: language.to_string(),
            last_indexed: chrono::Utc::now().to_rfc3339(),
            content_hash,
            symbols,
            dependencies,
            recent_changes: Vec::new(),
            linked_decisions: Vec::new(),
            notes: Vec::new(),
        })
    }

    fn extract_symbols(
        &self,
        tree: &Tree,
        content: &str,
        queries: &CodeQueries,
        file_path: &Path,
        language: &str,
    ) -> Result<Vec<ParsedSymbol>> {
        let mut cursor = QueryCursor::new();
        let mut symbols = Vec::new();

        let matches = cursor.matches(&queries.symbols, tree.root_node(), content.as_bytes());

        for m in matches {
            let mut symbol_name = String::new();
            let mut symbol_type = String::from("unknown");
            let mut node_for_position = None;

            for capture in m.captures {
                let node = capture.node;
                let capture_name = &queries.symbols.capture_names()[capture.index as usize];

                if capture_name.ends_with(".name") {
                    // Extract the symbol type from the capture name (e.g., "function.name" -> "function")
                    symbol_type = capture_name
                        .split('.')
                        .next()
                        .unwrap_or("unknown")
                        .to_string();
                    symbol_name = node.utf8_text(content.as_bytes())?.to_string();
                    node_for_position = Some(node);
                } else if capture_name.ends_with(".definition") && node_for_position.is_none() {
                    // Use the definition node for position if we don't have a name node yet
                    node_for_position = Some(node);
                }
            }

            if !symbol_name.is_empty() {
                if let Some(pos_node) = node_for_position {
                    symbols.push(ParsedSymbol {
                        name: symbol_name,
                        symbol_type,
                        start_line: pos_node.start_position().row,
                        end_line: pos_node.end_position().row,
                        start_byte: pos_node.start_byte(),
                        end_byte: pos_node.end_byte(),
                        file_path: file_path.to_string_lossy().to_string(),
                        language: language.to_string(),
                    });
                }
            }
        }

        Ok(symbols)
    }

    fn extract_dependencies(
        &self,
        tree: &Tree,
        content: &str,
        queries: &CodeQueries,
    ) -> Result<FileDependencies> {
        let mut cursor = QueryCursor::new();
        let mut imports = Vec::new();
        let mut exports = Vec::new();

        // Extract imports
        let import_matches = cursor.matches(&queries.imports, tree.root_node(), content.as_bytes());
        for m in import_matches {
            for capture in m.captures {
                let node = capture.node;
                if let Ok(text) = node.utf8_text(content.as_bytes()) {
                    imports.push(text.trim_matches('"').to_string());
                }
            }
        }

        // Extract exports
        let export_matches = cursor.matches(&queries.exports, tree.root_node(), content.as_bytes());
        for m in export_matches {
            for capture in m.captures {
                let node = capture.node;
                if let Ok(text) = node.utf8_text(content.as_bytes()) {
                    exports.push(text.to_string());
                }
            }
        }

        Ok(FileDependencies { imports, exports })
    }

    fn compute_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn generate_file_log_markdown(&self, file_log: &FileLog) -> String {
        let mut markdown = String::new();

        markdown.push_str("# FILE_LOG v1\n");
        markdown.push_str(&format!("path: {}\n", file_log.path));
        markdown.push_str(&format!("language: {}\n", file_log.language));
        markdown.push_str(&format!("last_indexed: {}\n", file_log.last_indexed));
        markdown.push_str(&format!(
            "content_hash: sha256:{}\n\n",
            file_log.content_hash
        ));

        markdown.push_str("## Symbols\n");
        for symbol in &file_log.symbols {
            markdown.push_str(&format!(
                "- {}: {} (lines {}-{})\n",
                symbol.symbol_type,
                symbol.name,
                symbol.start_line + 1,
                symbol.end_line + 1
            ));
        }
        markdown.push('\n');

        markdown.push_str("## Dependencies\n");
        markdown.push_str("imports:\n");
        for import in &file_log.dependencies.imports {
            markdown.push_str(&format!("- {}\n", import));
        }
        markdown.push_str("\nexports:\n");
        for export in &file_log.dependencies.exports {
            markdown.push_str(&format!("- {}\n", export));
        }
        markdown.push('\n');

        markdown.push_str("## Recent Changes\n");
        for change in &file_log.recent_changes {
            markdown.push_str(&format!("- {}\n", change));
        }
        markdown.push('\n');

        markdown.push_str("## Linked Decisions\n");
        for decision in &file_log.linked_decisions {
            markdown.push_str(&format!("- {}\n", decision));
        }
        markdown.push('\n');

        markdown.push_str("## Notes\n");
        for note in &file_log.notes {
            markdown.push_str(&format!("- {}\n", note));
        }

        markdown
    }

    pub fn chunk_file_content(
        &self,
        content: &str,
        language: &str,
    ) -> Vec<super::chunking::ChunkData> {
        let chunking_service = super::chunking::ChunkingService::new();
        chunking_service.chunk_file(content, language)
    }

    pub fn generate_filelog_summary(
        &self,
        file_path: &str,
        symbols: &[ParsedSymbol],
        language: &str,
    ) -> (String, Vec<String>, Vec<String>) {
        let filelog_gen = super::filelog_generator::FileLogGenerator::new();

        // Convert ParsedSymbol to Symbol for the generator
        let mock_symbols: Vec<crate::models::Symbol> = symbols
            .iter()
            .map(|ps| crate::models::Symbol {
                base: crate::models::BaseObject {
                    id: uuid::Uuid::new_v4(),
                    object_type: crate::models::ObjectType::Symbol,
                    tenant_id: "default".to_string(),
                    project_id: "default".to_string(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                    updated_at: chrono::Utc::now().to_rfc3339(),
                    provenance: crate::models::Provenance {
                        agent: "parser".to_string(),
                        model: None,
                        tools: None,
                        summary: "Parsed from codebase".to_string(),
                    },
                    links: vec![],
                    embedding: None,
                },
                name: ps.name.clone(),
                kind: match ps.symbol_type.as_str() {
                    "function" => crate::models::SymbolKind::Function,
                    "class" => crate::models::SymbolKind::Class,
                    "variable" => crate::models::SymbolKind::Variable,
                    "module" => crate::models::SymbolKind::Module,
                    "type" => crate::models::SymbolKind::Type,
                    _ => crate::models::SymbolKind::Function,
                },
                path: ps.file_path.clone(),
                language: ps.language.clone(),
                content_hash: None,
                signature: None,
                documentation: None,
            })
            .collect();

        let summary = filelog_gen.generate_summary(file_path, &mock_symbols, language);
        let key_symbols = filelog_gen.extract_key_symbols(&mock_symbols);
        let dependencies = filelog_gen.extract_dependencies(&mock_symbols);

        (summary, key_symbols, dependencies)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    #[test]
    fn test_parse_python_file() {
        let parser = CodebaseParser::new().unwrap();

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.py");
        std::fs::write(
            &file_path,
            r#"
def hello_world():
    print("Hello, world!")

class MyClass:
    def method(self):
        pass

import os
from typing import List
"#,
        )
        .unwrap();

        let file_log = parser.parse_file(&file_path, "python").unwrap();

        assert_eq!(file_log.language, "python");
        assert!(file_log.symbols.len() >= 3); // function, class, method
        assert!(file_log.dependencies.imports.len() >= 1);
    }

    #[test]
    fn test_parse_typescript_file() {
        let parser = CodebaseParser::new().unwrap();

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.ts");
        std::fs::write(
            &file_path,
            r#"
import { Component } from 'react';

interface User {
    name: string;
    age: number;
}

class UserComponent extends Component {
    render() {
        return null;
    }
}

export function createUser(name: string): User {
    return { name, age: 0 };
}
"#,
        )
        .unwrap();

        let file_log = parser.parse_file(&file_path, "typescript").unwrap();

        assert_eq!(file_log.language, "typescript");
        assert!(file_log.symbols.len() >= 3); // interface, class, function
    }

    #[test]
    fn test_generate_markdown() {
        let parser = CodebaseParser::new().unwrap();

        let file_log = FileLog {
            path: "test.py".to_string(),
            language: "python".to_string(),
            last_indexed: "2026-01-17T10:00:00Z".to_string(),
            content_hash: "abc123".to_string(),
            symbols: vec![ParsedSymbol {
                name: "hello".to_string(),
                symbol_type: "function".to_string(),
                start_line: 0,
                end_line: 2,
                start_byte: 0,
                end_byte: 30,
                file_path: "test.py".to_string(),
                language: "python".to_string(),
            }],
            dependencies: FileDependencies {
                imports: vec!["os".to_string()],
                exports: vec!["hello".to_string()],
            },
            recent_changes: vec!["Added hello function".to_string()],
            linked_decisions: vec!["dec_001".to_string()],
            notes: vec!["Main entry point".to_string()],
        };

        let markdown = parser.generate_file_log_markdown(&file_log);

        assert!(markdown.contains("# FILE_LOG v1"));
        assert!(markdown.contains("path: test.py"));
        assert!(markdown.contains("## Symbols"));
        assert!(markdown.contains("function: hello"));
    }

    #[test]
    fn test_parse_javascript_file() {
        let parser = CodebaseParser::new().unwrap();

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.js");
        std::fs::write(
            &file_path,
            r#"
import { useState } from 'react';

function greet(name) {
    console.log('Hello, ' + name);
}

class Calculator {
    add(a, b) {
        return a + b;
    }
}

export const PI = 3.14159;
"#,
        )
        .unwrap();

        let file_log = parser.parse_file(&file_path, "javascript").unwrap();

        assert_eq!(file_log.language, "javascript");
        assert!(file_log.symbols.len() >= 2); // function, class
        assert!(file_log.dependencies.imports.len() >= 1);
    }

    #[test]
    fn test_parse_rust_file() {
        let parser = CodebaseParser::new().unwrap();

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        std::fs::write(
            &file_path,
            r#"
use std::collections::HashMap;

fn main() {
    println!("Hello, world!");
}

pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

pub enum Direction {
    North,
    South,
    East,
    West,
}
"#,
        )
        .unwrap();

        let file_log = parser.parse_file(&file_path, "rust").unwrap();

        assert_eq!(file_log.language, "rust");
        assert!(file_log.symbols.len() >= 3); // function, struct, enum
        assert!(file_log.dependencies.imports.len() >= 1);
    }

    #[test]
    fn test_parse_go_file() {
        let parser = CodebaseParser::new().unwrap();

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.go");
        std::fs::write(
            &file_path,
            r#"
package main

import "fmt"

func main() {
    fmt.Println("Hello, World!")
}

type User struct {
    Name string
    Age  int
}

func (u *User) Greet() string {
    return "Hello, " + u.Name
}

type Greeter interface {
    Greet() string
}
"#,
        )
        .unwrap();

        let file_log = parser.parse_file(&file_path, "go").unwrap();

        assert_eq!(file_log.language, "go");
        assert!(file_log.symbols.len() >= 3); // functions, struct, interface
        assert!(file_log.dependencies.imports.len() >= 1);
    }

    #[test]
    fn test_parse_csharp_file() {
        let parser = CodebaseParser::new().unwrap();

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.cs");
        std::fs::write(
            &file_path,
            r#"
using System;
using System.Collections.Generic;

namespace MyApp
{
    public class Calculator
    {
        public int Add(int a, int b)
        {
            return a + b;
        }
    }

    public interface ICalculator
    {
        int Add(int a, int b);
    }

    public enum Operation
    {
        Add,
        Subtract,
        Multiply,
        Divide
    }
}
"#,
        )
        .unwrap();

        let file_log = parser.parse_file(&file_path, "csharp").unwrap();

        assert_eq!(file_log.language, "csharp");
        assert!(file_log.symbols.len() >= 3); // class, interface, enum
        assert!(file_log.dependencies.imports.len() >= 1);
    }

    #[test]
    fn test_parse_java_file() {
        let parser = CodebaseParser::new().unwrap();

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("Test.java");
        std::fs::write(
            &file_path,
            r#"
package com.example;

import java.util.List;
import java.util.ArrayList;

public class Calculator {
    private int value;

    public Calculator() {
        this.value = 0;
    }

    public int add(int a, int b) {
        return a + b;
    }
}

interface Computable {
    int compute(int x);
}

enum Operation {
    ADD, SUBTRACT, MULTIPLY, DIVIDE
}
"#,
        )
        .unwrap();

        let file_log = parser.parse_file(&file_path, "java").unwrap();

        assert_eq!(file_log.language, "java");
        assert!(file_log.symbols.len() >= 3); // class, interface, enum
        assert!(file_log.dependencies.imports.len() >= 1);
    }

    #[test]
    fn test_parse_c_file() {
        let parser = CodebaseParser::new().unwrap();

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.c");
        std::fs::write(
            &file_path,
            r#"
#include <stdio.h>
#include "myheader.h"

typedef struct {
    int x;
    int y;
} Point;

enum Color {
    RED,
    GREEN,
    BLUE
};

int add(int a, int b) {
    return a + b;
}

int main() {
    printf("Hello, World!\n");
    return 0;
}
"#,
        )
        .unwrap();

        let file_log = parser.parse_file(&file_path, "c").unwrap();

        assert_eq!(file_log.language, "c");
        assert!(file_log.symbols.len() >= 2); // functions, struct, enum
        assert!(file_log.dependencies.imports.len() >= 1);
    }

    #[test]
    fn test_parse_cpp_file() {
        let parser = CodebaseParser::new().unwrap();

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.cpp");
        std::fs::write(
            &file_path,
            r#"
#include <iostream>
#include <vector>

namespace math {

class Calculator {
public:
    int add(int a, int b) {
        return a + b;
    }
};

struct Point {
    double x;
    double y;
};

enum class Color {
    Red,
    Green,
    Blue
};

}

int main() {
    std::cout << "Hello, World!" << std::endl;
    return 0;
}
"#,
        )
        .unwrap();

        let file_log = parser.parse_file(&file_path, "cpp").unwrap();

        assert_eq!(file_log.language, "cpp");
        assert!(file_log.symbols.len() >= 3); // class, struct, enum, namespace, function
        assert!(file_log.dependencies.imports.len() >= 1);
    }

    #[test]
    fn test_parse_ruby_file() {
        let parser = CodebaseParser::new().unwrap();

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.rb");
        std::fs::write(
            &file_path,
            r#"
require 'json'
require_relative 'helper'

module Calculator
  VERSION = "1.0.0"

  class Basic
    def initialize(value)
      @value = value
    end

    def add(a, b)
      a + b
    end

    def self.create
      new(0)
    end
  end
end

def greet(name)
  puts "Hello, #{name}!"
end
"#,
        )
        .unwrap();

        let file_log = parser.parse_file(&file_path, "ruby").unwrap();

        assert_eq!(file_log.language, "ruby");
        assert!(file_log.symbols.len() >= 3); // module, class, methods
        assert!(file_log.dependencies.imports.len() >= 1);
    }
}
