-- Example queries for AMP operations

-- 1. Insert a symbol
CREATE symbols CONTENT {
    id: "symbols:rust_main_fn",
    type: "symbol",
    tenant_id: "default",
    project_id: "amp_server",
    name: "main",
    kind: "function",
    path: "src/main.rs",
    language: "rust",
    signature: "fn main() -> Result<(), Box<dyn Error>>",
    provenance: {
        agent: "rust_analyzer",
        summary: "Main function discovered during indexing"
    },
    embedding: [0.1, 0.2, 0.3] -- Truncated for example
};

-- 2. Create relationships
RELATE symbols:rust_main_fn->calls->symbols:axum_serve;
RELATE changesets:fix_cors->modifies->symbols:cors_middleware;

-- 3. Vector similarity search
SELECT *, vector::similarity::cosine(embedding, [0.1, 0.2, 0.3]) AS similarity 
FROM objects 
WHERE type = "symbol" 
ORDER BY similarity DESC 
LIMIT 10;

-- 4. Graph traversal - find all symbols that depend on a given symbol
SELECT * FROM (
    SELECT ->depends_on->objects.* AS dependencies 
    FROM symbols:rust_main_fn
) WHERE dependencies IS NOT NONE;

-- 5. Temporal query - recent changes
SELECT * FROM changesets 
WHERE created_at > time::now() - 1d 
ORDER BY created_at DESC;

-- 6. Hybrid query combining vector search with graph traversal
SELECT 
    *,
    vector::similarity::cosine(embedding, [0.1, 0.2, 0.3]) AS similarity,
    ->depends_on->objects.* AS dependencies
FROM objects 
WHERE type IN ["symbol", "decision"] 
    AND project_id = "amp_server"
    AND vector::similarity::cosine(embedding, [0.1, 0.2, 0.3]) > 0.8
ORDER BY similarity DESC
LIMIT 5;

-- 7. Find decision justifications
SELECT 
    d.*,
    <-justified_by<-changesets.* AS implementations
FROM decisions d
WHERE d.status = "accepted";

-- 8. Agent run analysis
SELECT 
    agent,
    count() AS run_count,
    avg(confidence) AS avg_confidence,
    avg(duration_ms) AS avg_duration
FROM runs 
WHERE status = "completed"
    AND created_at > time::now() - 7d
GROUP BY provenance.agent;

-- 9. Project overview query
SELECT 
    type,
    count() AS count,
    max(created_at) AS latest_update
FROM objects 
WHERE project_id = "amp_server"
GROUP BY type;

-- 10. Lease management
-- Acquire lease
CREATE leases CONTENT {
    resource: "project:amp_server:indexing",
    holder: "indexer_agent_1",
    expires_at: time::now() + 5m
};

-- Check active leases
SELECT * FROM leases 
WHERE expires_at > time::now()
    AND resource = "project:amp_server:indexing";
