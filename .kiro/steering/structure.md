# Project Structure

## Directory Layout
```
amp/
├── spec/                    # Protocol specifications
│   ├── openapi.yaml        # Complete API specification
│   ├── schemas/            # JSON schemas for all objects
│   │   ├── base.json       # Base object schema
│   │   ├── symbol.json     # Symbol object schema
│   │   ├── decision.json   # Decision object schema
│   │   ├── changeset.json  # ChangeSet object schema
│   │   └── run.json        # Run object schema
│   ├── schema.surql        # SurrealDB schema definition
│   └── example_queries.surql # Example database queries
├── server/                 # Rust server implementation
│   ├── src/
│   │   ├── main.rs         # Server entry point
│   │   ├── config.rs       # Configuration management
│   │   ├── database.rs     # Database connection
│   │   ├── models/         # Data models
│   │   ├── handlers/       # API request handlers
│   │   └── services/       # Business logic services
│   └── Cargo.toml          # Rust dependencies
├── sdks/                   # Generated client SDKs
│   ├── python/             # Python client library
│   └── typescript/         # TypeScript client library
├── examples/               # Usage examples
│   ├── python_basic.py     # Python SDK example
│   └── typescript_basic.ts # TypeScript SDK example
├── scripts/                # Development scripts
│   ├── dev-setup.sh        # Development environment setup
│   ├── generate-sdks.sh    # SDK generation script
│   └── demo.sh             # Complete demo script
├── Cargo.toml              # Workspace configuration
├── README.md               # Project overview
└── DEVELOPMENT.md          # Development guide
```

## File Naming Conventions
- **Rust modules**: snake_case (e.g., `object_store.rs`, `query_engine.rs`)
- **JSON schemas**: lowercase with underscores (e.g., `base.json`, `changeset.json`)
- **API endpoints**: kebab-case in URLs (e.g., `/objects:batch`, `/leases:acquire`)
- **Database tables**: lowercase plural (e.g., `symbols`, `decisions`, `changesets`)
- **Configuration files**: lowercase with extensions (e.g., `openapi.yaml`, `schema.surql`)

## Module Organization
[How code is organized into modules, packages, or components]

## Configuration Files
[Location and purpose of config files]

## Documentation Structure
[Where and how documentation is organized]

## Asset Organization
[How images, styles, and other assets are structured]

## Build Artifacts
[Where compiled/generated files are placed]

## Environment-Specific Files
[How different environments (dev, staging, prod) are handled]
