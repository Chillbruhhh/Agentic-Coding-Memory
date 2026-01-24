use crate::models::Symbol;

pub struct FileLogGenerator;

impl FileLogGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_summary(&self, file_path: &str, symbols: &[Symbol], language: &str) -> String {
        let purpose = self.infer_purpose(symbols);
        let symbols_section = self.format_symbols(symbols);
        let deps_section = self.format_dependencies(symbols);

        format!(
            "# {}\n\n**Language**: {}\n\n## Purpose\n{}\n\n## Symbols\n{}\n\n## Dependencies\n{}",
            file_path, language, purpose, symbols_section, deps_section
        )
    }

    pub fn extract_key_symbols(&self, symbols: &[Symbol]) -> Vec<String> {
        symbols
            .iter()
            .map(|s| format!("{}:{}", s.kind_str(), s.name))
            .collect()
    }

    pub fn extract_dependencies(&self, symbols: &[Symbol]) -> Vec<String> {
        symbols
            .iter()
            .filter_map(|s| s.documentation.as_ref())
            .flat_map(|doc| self.extract_imports_from_doc(doc))
            .collect()
    }

    fn infer_purpose(&self, symbols: &[Symbol]) -> String {
        if symbols.is_empty() {
            return "No symbols found".to_string();
        }

        let classes = symbols
            .iter()
            .filter(|s| matches!(s.kind_str(), "class"))
            .count();
        let functions = symbols
            .iter()
            .filter(|s| matches!(s.kind_str(), "function"))
            .count();

        if classes > 0 && functions > 0 {
            format!(
                "Module containing {} classes and {} functions",
                classes, functions
            )
        } else if classes > 0 {
            format!("Class definitions ({} classes)", classes)
        } else if functions > 0 {
            format!("Function definitions ({} functions)", functions)
        } else {
            "Code file with various symbols".to_string()
        }
    }

    fn format_symbols(&self, symbols: &[Symbol]) -> String {
        if symbols.is_empty() {
            return "No symbols".to_string();
        }

        symbols
            .iter()
            .take(20)
            .map(|s| format!("- `{}` ({})", s.name, s.kind_str()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_dependencies(&self, symbols: &[Symbol]) -> String {
        let deps = self.extract_dependencies(symbols);
        if deps.is_empty() {
            return "No dependencies detected".to_string();
        }

        deps.iter()
            .take(10)
            .map(|d| format!("- {}", d))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn extract_imports_from_doc(&self, doc: &str) -> Vec<String> {
        doc.lines()
            .filter(|line| line.contains("import") || line.contains("from"))
            .map(|s| s.trim().to_string())
            .collect()
    }
}

impl Default for FileLogGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// Helper trait for Symbol
trait SymbolExt {
    fn kind_str(&self) -> &str;
}

impl SymbolExt for Symbol {
    fn kind_str(&self) -> &str {
        match self.kind {
            crate::models::SymbolKind::File => "file",
            crate::models::SymbolKind::Module => "module",
            crate::models::SymbolKind::Class => "class",
            crate::models::SymbolKind::Function => "function",
            crate::models::SymbolKind::Variable => "variable",
            crate::models::SymbolKind::Type => "type",
        }
    }
}
