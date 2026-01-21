# AMP Parser Test Repository

This repository contains sample code files in all languages supported by AMP's Tree-sitter parser.

## Supported Languages

1. Python (`.py`)
2. TypeScript (`.ts`, `.tsx`)
3. JavaScript (`.js`, `.jsx`)
4. Rust (`.rs`)
5. Go (`.go`)
6. C# (`.cs`)
7. Java (`.java`)
8. C (`.c`, `.h`)
9. C++ (`.cpp`, `.hpp`)
10. Ruby (`.rb`)

## Testing the Parser

```bash
# Index this test repository
cd amp/cli
cargo run -- index ../../test-repo

# Query the indexed symbols
cargo run -- query "function" --limit 20

# View file logs
cargo run -- filelog ../../test-repo/python/sample.py
```

## Expected Symbols

Each file contains common language constructs:
- Functions/Methods
- Classes/Structs
- Variables/Constants
- Imports/Dependencies
- Exports (where applicable)

Total expected symbols: ~100+ across all files
