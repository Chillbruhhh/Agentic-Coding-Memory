-- AMP Hybrid Query Examples
-- These examples demonstrate how to use the hybrid retrieval system
-- that combines text search, vector similarity, and graph traversal

-- ============================================================================
-- HYBRID QUERY BASICS
-- ============================================================================

-- Example 1: Simple hybrid text query
-- Combines text search across multiple fields with intelligent scoring
POST /v1/query
{
    "text": "authentication",
    "hybrid": true,
    "limit": 10
}

-- Example 2: Hybrid query with type filtering
-- Searches for authentication-related symbols and decisions only
POST /v1/query
{
    "text": "authentication",
    "hybrid": true,
    "filters": {
        "type": ["symbol", "decision"],
        "project_id": "my_project"
    },
    "limit": 5
}

-- ============================================================================
-- MULTI-MODAL HYBRID QUERIES
-- ============================================================================

-- Example 3: Text + Vector hybrid search
-- Provides both keyword matching and semantic similarity
POST /v1/query
{
    "text": "user login security",
    "vector": [0.1, 0.2, 0.3, ...], -- 1536-dimensional embedding
    "hybrid": true,
    "limit": 10
}

-- Example 4: Text + Graph hybrid search
-- Combines text search with relationship traversal
POST /v1/query
{
    "text": "password",
    "graph": {
        "start_nodes": ["550e8400-e29b-41d4-a716-446655440000"],
        "relation_types": ["depends_on", "calls"],
        "max_depth": 2,
        "direction": "outbound"
    },
    "hybrid": true,
    "limit": 15
}

-- Example 5: Full hybrid search (Text + Vector + Graph)
-- Uses all three search modalities for maximum coverage
POST /v1/query
{
    "text": "authentication system",
    "vector": [0.1, 0.2, 0.3, ...], -- Generated from text or provided
    "graph": {
        "start_nodes": ["550e8400-e29b-41d4-a716-446655440000"],
        "relation_types": ["implements", "justified_by"],
        "max_depth": 3,
        "direction": "both"
    },
    "hybrid": true,
    "filters": {
        "project_id": "auth_service",
        "created_after": "2024-01-01T00:00:00Z"
    },
    "limit": 20
}

-- ============================================================================
-- HYBRID SCORING EXAMPLES
-- ============================================================================

-- The hybrid system uses weighted scoring:
-- - Vector similarity: 40% weight
-- - Text relevance: 30% weight  
-- - Graph connectivity: 30% weight

-- Example response showing multi-modal scoring:
{
    "results": [
        {
            "object": { /* memory object */ },
            "score": 0.85,  -- Combined total score
            "explanation": "Text match for 'authentication' in name + Vector similarity (score: 0.92) + Graph traversal match"
        }
    ],
    "trace_id": "123e4567-e89b-12d3-a456-426614174000",
    "total_count": 5,
    "execution_time_ms": 245
}

-- ============================================================================
-- PERFORMANCE OPTIMIZATION EXAMPLES
-- ============================================================================

-- Example 6: Hybrid query with result limits
-- Limits each search method to improve performance
POST /v1/query
{
    "text": "database connection",
    "hybrid": true,
    "limit": 5,  -- Total results limit
    "filters": {
        "type": ["symbol", "changeset"]
    }
}

-- Example 7: Project-scoped hybrid search
-- Restricts search to specific project for better performance
POST /v1/query
{
    "text": "error handling",
    "hybrid": true,
    "filters": {
        "project_id": "core_service",
        "tenant_id": "team_alpha"
    },
    "limit": 10
}

-- ============================================================================
-- ADVANCED HYBRID PATTERNS
-- ============================================================================

-- Example 8: Temporal hybrid search
-- Combines hybrid retrieval with time-based filtering
POST /v1/query
{
    "text": "refactoring",
    "hybrid": true,
    "filters": {
        "created_after": "2024-06-01T00:00:00Z",
        "created_before": "2024-12-31T23:59:59Z"
    },
    "limit": 25
}

-- Example 9: Multi-hop hybrid traversal
-- Combines semantic search with deep graph traversal
POST /v1/query
{
    "text": "security vulnerability",
    "graph": {
        "start_nodes": ["550e8400-e29b-41d4-a716-446655440000"],
        "algorithm": "collect",
        "max_depth": 4,
        "direction": "both"
    },
    "hybrid": true,
    "limit": 30
}

-- ============================================================================
-- ERROR HANDLING EXAMPLES
-- ============================================================================

-- Example 10: Graceful degradation
-- If vector search fails, text and graph results are still returned
POST /v1/query
{
    "text": "memory management",
    "vector": [/* invalid or corrupted vector */],
    "hybrid": true,
    "limit": 10
}
-- Response will include partial results with explanation of what succeeded

-- Example 11: Empty hybrid query
-- Hybrid flag with no search criteria - returns empty results gracefully
POST /v1/query
{
    "hybrid": true,
    "filters": {
        "project_id": "test_project"
    },
    "limit": 10
}

-- ============================================================================
-- BACKWARD COMPATIBILITY
-- ============================================================================

-- Example 12: Non-hybrid queries continue to work unchanged
-- Existing queries without hybrid flag use original single-method logic
POST /v1/query
{
    "text": "authentication",
    "filters": {
        "type": ["symbol"]
    },
    "limit": 10
}
-- This uses the original text-only search path

-- ============================================================================
-- BEST PRACTICES
-- ============================================================================

-- 1. Use hybrid queries when you want comprehensive results across modalities
-- 2. Apply filters to improve performance and relevance
-- 3. Set appropriate limits to control response size and latency
-- 4. Combine text with vector for semantic + keyword matching
-- 5. Use graph traversal to find related concepts and dependencies
-- 6. Monitor execution times and adjust limits as needed
-- 7. Leverage project/tenant filtering for multi-tenant scenarios
