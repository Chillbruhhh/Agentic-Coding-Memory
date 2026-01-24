-- AMP SurrealDB Schema
-- Agentic Memory Protocol database schema with tables, relations, and indexes

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

-- FileChunk table for chunked file content
DEFINE TABLE file_chunks SCHEMAFULL;
DEFINE FIELD id ON file_chunks TYPE record<file_chunks>;
DEFINE FIELD file_path ON file_chunks TYPE string;
DEFINE FIELD chunk_index ON file_chunks TYPE int;
DEFINE FIELD start_line ON file_chunks TYPE int;
DEFINE FIELD end_line ON file_chunks TYPE int;
DEFINE FIELD token_count ON file_chunks TYPE int;
DEFINE FIELD content ON file_chunks TYPE string;
DEFINE FIELD content_hash ON file_chunks TYPE string;
DEFINE FIELD language ON file_chunks TYPE string;
DEFINE FIELD embedding ON file_chunks TYPE option<array<float>>;
DEFINE FIELD file_id ON file_chunks TYPE string;
DEFINE FIELD tenant_id ON file_chunks TYPE string;
DEFINE FIELD project_id ON file_chunks TYPE string;
DEFINE FIELD created_at ON file_chunks TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON file_chunks TYPE datetime DEFAULT time::now();

-- Indexes for file_chunks
DEFINE INDEX idx_file_chunks_path ON file_chunks COLUMNS file_path;
DEFINE INDEX idx_file_chunks_hash ON file_chunks COLUMNS content_hash;
DEFINE INDEX idx_file_chunks_file_id ON file_chunks COLUMNS file_id;
DEFINE INDEX idx_file_chunks_embedding ON file_chunks COLUMNS embedding MTREE DIMENSION 1536;

-- FileLog table for semantic file summaries with audit trail
DEFINE TABLE file_logs SCHEMAFULL;
DEFINE FIELD id ON file_logs TYPE record<file_logs>;
DEFINE FIELD file_path ON file_logs TYPE string;
DEFINE FIELD file_id ON file_logs TYPE string;
DEFINE FIELD summary ON file_logs TYPE string;
DEFINE FIELD purpose ON file_logs TYPE option<string>;
DEFINE FIELD key_symbols ON file_logs TYPE array<string>;
DEFINE FIELD dependencies ON file_logs TYPE array<string>;
DEFINE FIELD notes ON file_logs TYPE option<string>;
DEFINE FIELD embedding ON file_logs TYPE option<array<float>>;
DEFINE FIELD last_modified ON file_logs TYPE datetime;
DEFINE FIELD change_count ON file_logs TYPE int DEFAULT 0;
DEFINE FIELD linked_changesets ON file_logs TYPE array<string>;
-- Audit trail: concise record of changes (1-4 sentences each)
-- Structure: [{ timestamp, action, summary, run_id?, agent_id? }]
DEFINE FIELD audit_trail ON file_logs TYPE array<object> DEFAULT [];
DEFINE FIELD tenant_id ON file_logs TYPE string;
DEFINE FIELD project_id ON file_logs TYPE string;
DEFINE FIELD created_at ON file_logs TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON file_logs TYPE datetime DEFAULT time::now();

-- Indexes for file_logs
DEFINE INDEX idx_file_logs_path ON file_logs COLUMNS file_path;
DEFINE INDEX idx_file_logs_file_id ON file_logs COLUMNS file_id;
DEFINE INDEX idx_file_logs_embedding ON file_logs COLUMNS embedding MTREE DIMENSION 1536;

-- Note-specific fields (freeform agent notes/insights)
DEFINE TABLE notes AS SELECT * FROM objects WHERE type = "note";
DEFINE FIELD title ON notes TYPE string;
DEFINE FIELD content ON notes TYPE string;
DEFINE FIELD category ON notes TYPE option<string> ASSERT $value IN ["insight", "todo", "question", "warning", "reference"];
DEFINE FIELD linked_objects ON notes TYPE option<array<string>>;
DEFINE FIELD agent_id ON notes TYPE option<string>;
DEFINE FIELD run_id ON notes TYPE option<string>;
DEFINE FIELD tags ON notes TYPE option<array<string>>;

-- Indexes for notes
DEFINE INDEX idx_notes_title ON notes COLUMNS title;
DEFINE INDEX idx_notes_category ON notes COLUMNS category;
DEFINE INDEX idx_notes_agent ON notes COLUMNS agent_id;

-- FileLogs as artifacts (view from objects table for artifact queries)
DEFINE TABLE filelogs AS SELECT * FROM objects WHERE type = "filelog";
DEFINE FIELD title ON filelogs TYPE string;
DEFINE FIELD file_path ON filelogs TYPE string;
DEFINE FIELD summary ON filelogs TYPE string;
DEFINE FIELD symbols ON filelogs TYPE option<array<string>>;
DEFINE FIELD dependencies ON filelogs TYPE option<array<string>>;
DEFINE FIELD agent_id ON filelogs TYPE option<string>;
DEFINE FIELD run_id ON filelogs TYPE option<string>;
DEFINE FIELD tags ON filelogs TYPE option<array<string>>;

-- Indexes for filelogs
DEFINE INDEX idx_filelogs_path ON filelogs COLUMNS file_path;
DEFINE INDEX idx_filelogs_agent ON filelogs COLUMNS agent_id;

-- ============================================================================
-- Semantic Cache (Episodic Memory) - Rolling window of session context
-- ============================================================================

-- Cache Blocks: Episodic memory units (rolling window of ~20 blocks)
-- Each block holds 1800-2000 tokens of structured session context
DEFINE TABLE cache_block SCHEMAFULL;
DEFINE FIELD id ON cache_block TYPE record<cache_block>;
DEFINE FIELD scope_id ON cache_block TYPE string;
DEFINE FIELD sequence ON cache_block TYPE int;
DEFINE FIELD status ON cache_block TYPE string DEFAULT "open" ASSERT $value IN ["open", "closed"];

-- Quick-access summary (~200 tokens) for efficient search
DEFINE FIELD summary ON cache_block TYPE option<string>;
DEFINE FIELD summary_embedding ON cache_block TYPE option<array<float>>;

-- Full structured content
DEFINE FIELD items ON cache_block TYPE array<object> DEFAULT [];
DEFINE FIELD token_count ON cache_block TYPE int DEFAULT 0;

-- Timestamps
DEFINE FIELD created_at ON cache_block TYPE datetime DEFAULT time::now();
DEFINE FIELD closed_at ON cache_block TYPE option<datetime>;

-- Indexes for cache_block
DEFINE INDEX idx_cache_block_scope ON cache_block COLUMNS scope_id;
DEFINE INDEX idx_cache_block_sequence ON cache_block COLUMNS scope_id, sequence;
DEFINE INDEX idx_cache_block_status ON cache_block COLUMNS scope_id, status;
DEFINE INDEX idx_cache_block_summary_embedding ON cache_block COLUMNS summary_embedding MTREE DIMENSION 1536;

-- Cache Block Item structure (stored in items array):
-- {
--   kind: "fact" | "decision" | "snippet" | "warning",
--   content: string,
--   importance: float (0-1),
--   file_ref: option<string>,  -- For snippets
--   created_at: datetime
-- }

-- Legacy cache_frame (kept for backward compatibility, will be migrated)
DEFINE TABLE cache_frame SCHEMAFULL;
DEFINE FIELD scope_id ON cache_frame TYPE string;
DEFINE FIELD summary ON cache_frame TYPE string;
DEFINE FIELD summary_embedding ON cache_frame TYPE option<array<float>>;
DEFINE FIELD version ON cache_frame TYPE int DEFAULT 0;
DEFINE FIELD token_count ON cache_frame TYPE int DEFAULT 0;
DEFINE FIELD updated_at ON cache_frame TYPE datetime DEFAULT time::now();
DEFINE FIELD ttl_expires_at ON cache_frame TYPE option<datetime>;
DEFINE FIELD created_at ON cache_frame TYPE datetime DEFAULT time::now();

-- Indexes for cache_frame
DEFINE INDEX idx_cache_frame_scope ON cache_frame COLUMNS scope_id UNIQUE;
DEFINE INDEX idx_cache_frame_ttl ON cache_frame COLUMNS ttl_expires_at;
DEFINE INDEX idx_cache_frame_embedding ON cache_frame COLUMNS summary_embedding MTREE DIMENSION 1536;

-- Legacy cache_item (kept for backward compatibility, will be migrated)
DEFINE TABLE cache_item SCHEMAFULL;
DEFINE FIELD scope_id ON cache_item TYPE string;
DEFINE FIELD artifact_id ON cache_item TYPE option<string>;
DEFINE FIELD kind ON cache_item TYPE string ASSERT $value IN ["fact", "decision", "snippet", "warning"];
DEFINE FIELD preview ON cache_item TYPE string;
DEFINE FIELD facts ON cache_item TYPE array<string> DEFAULT [];
DEFINE FIELD embedding ON cache_item TYPE option<array<float>>;
DEFINE FIELD importance ON cache_item TYPE float DEFAULT 0.5;
DEFINE FIELD access_count ON cache_item TYPE int DEFAULT 0;
DEFINE FIELD ttl_expires_at ON cache_item TYPE option<datetime>;
DEFINE FIELD version ON cache_item TYPE int DEFAULT 0;
DEFINE FIELD provenance ON cache_item TYPE object DEFAULT {};
DEFINE FIELD created_at ON cache_item TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON cache_item TYPE datetime DEFAULT time::now();

-- Indexes for cache_item
DEFINE INDEX idx_cache_item_scope ON cache_item COLUMNS scope_id;
DEFINE INDEX idx_cache_item_kind ON cache_item COLUMNS kind;
DEFINE INDEX idx_cache_item_importance ON cache_item COLUMNS importance;
DEFINE INDEX idx_cache_item_ttl ON cache_item COLUMNS ttl_expires_at;
DEFINE INDEX idx_cache_item_embedding ON cache_item COLUMNS embedding MTREE DIMENSION 1536;
