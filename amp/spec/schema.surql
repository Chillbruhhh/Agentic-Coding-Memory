-- AMP SurrealDB Schema
-- Agent Memory Protocol database schema with tables, relations, and indexes

-- Base table for all objects
DEFINE TABLE objects SCHEMALESS;

-- Indexes for efficient querying
DEFINE INDEX idx_objects_type ON objects COLUMNS type;
DEFINE INDEX idx_objects_tenant ON objects COLUMNS tenant_id;
DEFINE INDEX idx_objects_project ON objects COLUMNS project_id;
DEFINE INDEX idx_objects_created ON objects COLUMNS created_at;
DEFINE INDEX idx_objects_updated ON objects COLUMNS updated_at;

-- Vector index for semantic search (using SurrealDB's vector capabilities)
DEFINE INDEX idx_objects_embedding ON objects COLUMNS embedding MTREE DIMENSION 1536;

-- Symbol-specific fields
DEFINE TABLE symbols AS SELECT * FROM objects WHERE type = "symbol";
DEFINE FIELD name ON symbols TYPE string;
DEFINE FIELD kind ON symbols TYPE string ASSERT $value IN ["file", "module", "class", "function", "variable", "type"];
DEFINE FIELD path ON symbols TYPE string;
DEFINE FIELD language ON symbols TYPE string;
DEFINE FIELD content_hash ON symbols TYPE option<string>;
DEFINE FIELD signature ON symbols TYPE option<string>;
DEFINE FIELD documentation ON symbols TYPE option<string>;

-- Indexes for symbols
DEFINE INDEX idx_symbols_name ON symbols COLUMNS name;
DEFINE INDEX idx_symbols_kind ON symbols COLUMNS kind;
DEFINE INDEX idx_symbols_path ON symbols COLUMNS path;
DEFINE INDEX idx_symbols_language ON symbols COLUMNS language;
DEFINE INDEX idx_symbols_hash ON symbols COLUMNS content_hash;

-- Decision-specific fields
DEFINE TABLE decisions AS SELECT * FROM objects WHERE type = "decision";
DEFINE FIELD title ON decisions TYPE string;
DEFINE FIELD problem ON decisions TYPE string;
DEFINE FIELD options ON decisions TYPE option<array<object>>;
DEFINE FIELD rationale ON decisions TYPE string;
DEFINE FIELD outcome ON decisions TYPE string;
DEFINE FIELD status ON decisions TYPE option<string> ASSERT $value IN ["proposed", "accepted", "rejected", "superseded"];

-- Indexes for decisions
DEFINE INDEX idx_decisions_title ON decisions COLUMNS title;
DEFINE INDEX idx_decisions_status ON decisions COLUMNS status;

-- ChangeSet-specific fields
DEFINE TABLE changesets AS SELECT * FROM objects WHERE type = "changeset";
DEFINE FIELD title ON changesets TYPE string;
DEFINE FIELD description ON changesets TYPE option<string>;
DEFINE FIELD diff ON changesets TYPE option<string>;
DEFINE FIELD files_changed ON changesets TYPE array<string>;
DEFINE FIELD tests ON changesets TYPE option<array<object>>;
DEFINE FIELD status ON changesets TYPE string ASSERT $value IN ["draft", "review", "approved", "merged", "rejected"];
DEFINE FIELD commit_hash ON changesets TYPE option<string>;

-- Indexes for changesets
DEFINE INDEX idx_changesets_title ON changesets COLUMNS title;
DEFINE INDEX idx_changesets_status ON changesets COLUMNS status;
DEFINE INDEX idx_changesets_commit ON changesets COLUMNS commit_hash;

-- Run-specific fields
DEFINE TABLE runs AS SELECT * FROM objects WHERE type = "run";
DEFINE FIELD input_summary ON runs TYPE string;
DEFINE FIELD outputs ON runs TYPE option<array<object>>;
DEFINE FIELD errors ON runs TYPE option<array<object>>;
DEFINE FIELD confidence ON runs TYPE option<float> ASSERT $value >= 0 AND $value <= 1;
DEFINE FIELD duration_ms ON runs TYPE option<int>;
DEFINE FIELD status ON runs TYPE string ASSERT $value IN ["running", "completed", "failed", "cancelled"];

-- Indexes for runs
DEFINE INDEX idx_runs_status ON runs COLUMNS status;
DEFINE INDEX idx_runs_confidence ON runs COLUMNS confidence;

-- Relationship tables for graph traversal
DEFINE TABLE depends_on SCHEMALESS;
DEFINE FIELD in ON depends_on TYPE record<objects>;
DEFINE FIELD out ON depends_on TYPE record<objects>;
DEFINE FIELD created_at ON depends_on TYPE datetime DEFAULT time::now();

DEFINE TABLE defined_in SCHEMALESS;
DEFINE FIELD in ON defined_in TYPE record<objects>;
DEFINE FIELD out ON defined_in TYPE record<objects>;
DEFINE FIELD created_at ON defined_in TYPE datetime DEFAULT time::now();

DEFINE TABLE calls SCHEMALESS;

DEFINE TABLE justified_by SCHEMALESS;
DEFINE FIELD in ON justified_by TYPE record<objects>;
DEFINE FIELD out ON justified_by TYPE record<objects>;
DEFINE FIELD created_at ON justified_by TYPE datetime DEFAULT time::now();

DEFINE TABLE modifies SCHEMALESS;
DEFINE FIELD in ON modifies TYPE record<objects>;
DEFINE FIELD out ON modifies TYPE record<objects>;
DEFINE FIELD created_at ON modifies TYPE datetime DEFAULT time::now();

DEFINE TABLE implements SCHEMALESS;
DEFINE FIELD in ON implements TYPE record<objects>;
DEFINE FIELD out ON implements TYPE record<objects>;
DEFINE FIELD created_at ON implements TYPE datetime DEFAULT time::now();

DEFINE TABLE produced SCHEMALESS;
DEFINE FIELD in ON produced TYPE record<objects>;
DEFINE FIELD out ON produced TYPE record<objects>;
DEFINE FIELD created_at ON produced TYPE datetime DEFAULT time::now();

-- Query traces for deterministic traceability
DEFINE TABLE query_traces SCHEMAFULL;
DEFINE FIELD id ON query_traces TYPE record<query_traces>;
DEFINE FIELD query ON query_traces TYPE object;
DEFINE FIELD steps ON query_traces TYPE array<object>;
DEFINE FIELD total_time_ms ON query_traces TYPE int;
DEFINE FIELD created_at ON query_traces TYPE datetime DEFAULT time::now();

-- Coordination leases
DEFINE TABLE leases SCHEMAFULL;
DEFINE FIELD id ON leases TYPE record<leases>;
DEFINE FIELD resource ON leases TYPE string;
DEFINE FIELD holder ON leases TYPE string;
DEFINE FIELD expires_at ON leases TYPE datetime;
DEFINE FIELD created_at ON leases TYPE datetime DEFAULT time::now();

-- Indexes for leases
DEFINE INDEX idx_leases_resource ON leases COLUMNS resource;
DEFINE INDEX idx_leases_holder ON leases COLUMNS holder;
DEFINE INDEX idx_leases_expires ON leases COLUMNS expires_at;
