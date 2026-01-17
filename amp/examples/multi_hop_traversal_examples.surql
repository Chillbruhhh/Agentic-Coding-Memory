-- Multi-hop Graph Traversal Examples for AMP Application-Level Logic

-- Setup: Create a complex function dependency chain for testing
CREATE symbols:main_fn CONTENT {
    id: "symbols:main_fn",
    type: "symbol",
    tenant_id: "default",
    project_id: "multi_hop_demo",
    name: "main",
    kind: "function",
    path: "src/main.rs",
    language: "rust",
    signature: "fn main()",
    provenance: {
        agent: "demo",
        summary: "Main function - entry point"
    }
};

CREATE symbols:auth_service CONTENT {
    id: "symbols:auth_service",
    type: "symbol",
    tenant_id: "default",
    project_id: "multi_hop_demo",
    name: "AuthService",
    kind: "struct",
    path: "src/auth.rs",
    language: "rust",
    signature: "struct AuthService",
    provenance: {
        agent: "demo",
        summary: "Authentication service"
    }
};

CREATE symbols:database_conn CONTENT {
    id: "symbols:database_conn",
    type: "symbol",
    tenant_id: "default",
    project_id: "multi_hop_demo",
    name: "DatabaseConnection",
    kind: "struct",
    path: "src/db.rs",
    language: "rust",
    signature: "struct DatabaseConnection",
    provenance: {
        agent: "demo",
        summary: "Database connection handler"
    }
};

CREATE symbols:crypto_utils CONTENT {
    id: "symbols:crypto_utils",
    type: "symbol",
    tenant_id: "default",
    project_id: "multi_hop_demo",
    name: "crypto_utils",
    kind: "module",
    path: "src/crypto.rs",
    language: "rust",
    signature: "mod crypto_utils",
    provenance: {
        agent: "demo",
        summary: "Cryptographic utilities"
    }
};

-- Create dependency relationships: main -> auth -> db -> crypto
RELATE symbols:main_fn->calls->symbols:auth_service;
RELATE symbols:auth_service->depends_on->symbols:database_conn;
RELATE symbols:database_conn->depends_on->symbols:crypto_utils;

-- Create architectural decisions
CREATE decisions:use_jwt CONTENT {
    id: "decisions:use_jwt",
    type: "decision",
    tenant_id: "default",
    project_id: "multi_hop_demo",
    title: "Use JWT for authentication",
    status: "accepted",
    rationale: "JWT provides stateless authentication suitable for microservices",
    provenance: {
        agent: "architect",
        summary: "Authentication strategy decision"
    }
};

CREATE decisions:use_postgres CONTENT {
    id: "decisions:use_postgres",
    type: "decision",
    tenant_id: "default",
    project_id: "multi_hop_demo",
    title: "Use PostgreSQL as primary database",
    status: "accepted",
    rationale: "PostgreSQL provides ACID compliance and advanced features",
    provenance: {
        agent: "architect",
        summary: "Database selection decision"
    }
};

-- Create justification relationships
RELATE symbols:auth_service->justified_by->decisions:use_jwt;
RELATE symbols:database_conn->justified_by->decisions:use_postgres;

-- Example 1: COLLECT Algorithm - Find all dependencies of main function
-- API Call: POST /v1/query
-- {
--   "graph": {
--     "start_nodes": ["symbols:main_fn"],
--     "direction": "outbound",
--     "max_depth": 3,
--     "algorithm": "collect",
--     "relation_types": ["calls", "depends_on"]
--   }
-- }
-- Expected: Returns AuthService, DatabaseConnection, crypto_utils

-- Example 2: PATH Algorithm - Find all dependency paths from main
-- API Call: POST /v1/query
-- {
--   "graph": {
--     "start_nodes": ["symbols:main_fn"],
--     "direction": "outbound",
--     "max_depth": 3,
--     "algorithm": "path",
--     "relation_types": ["calls", "depends_on"]
--   }
-- }
-- Expected: Returns paths like [main->auth->db->crypto]

-- Example 3: SHORTEST Algorithm - Find shortest path from main to crypto
-- API Call: POST /v1/query
-- {
--   "graph": {
--     "start_nodes": ["symbols:main_fn"],
--     "target_node": "symbols:crypto_utils",
--     "direction": "outbound",
--     "max_depth": 5,
--     "algorithm": "shortest",
--     "relation_types": ["calls", "depends_on"]
--   }
-- }
-- Expected: Returns shortest path [main->auth->db->crypto]

-- Example 4: Complex justification chain traversal
-- API Call: POST /v1/query
-- {
--   "graph": {
--     "start_nodes": ["symbols:main_fn"],
--     "direction": "outbound",
--     "max_depth": 4,
--     "algorithm": "collect",
--     "relation_types": ["calls", "depends_on", "justified_by"]
--   }
-- }
-- Expected: Returns all symbols and their justifying decisions

-- Example 5: Cycle detection test
-- Create a cycle for testing
RELATE symbols:crypto_utils->calls->symbols:main_fn;

-- API Call: POST /v1/query
-- {
--   "graph": {
--     "start_nodes": ["symbols:main_fn"],
--     "direction": "outbound",
--     "max_depth": 10,
--     "algorithm": "collect",
--     "relation_types": ["calls", "depends_on"]
--   }
-- }
-- Expected: Should not infinite loop, returns unique nodes only

-- Example 6: Bidirectional traversal
-- API Call: POST /v1/query
-- {
--   "graph": {
--     "start_nodes": ["symbols:database_conn"],
--     "direction": "both",
--     "max_depth": 2,
--     "algorithm": "collect",
--     "relation_types": ["calls", "depends_on"]
--   }
-- }
-- Expected: Returns both dependencies and dependents

-- Example 7: Multiple start nodes
-- API Call: POST /v1/query
-- {
--   "graph": {
--     "start_nodes": ["symbols:auth_service", "symbols:database_conn"],
--     "direction": "outbound",
--     "max_depth": 2,
--     "algorithm": "collect",
--     "relation_types": ["depends_on", "justified_by"]
--   }
-- }
-- Expected: Returns combined dependencies from both start points

-- Example 8: Filtered relationship types
-- API Call: POST /v1/query
-- {
--   "graph": {
--     "start_nodes": ["symbols:main_fn"],
--     "direction": "outbound",
--     "max_depth": 3,
--     "algorithm": "collect",
--     "relation_types": ["calls"]
--   }
-- }
-- Expected: Only follows "calls" relationships, not "depends_on"

-- Example 9: Performance test with larger graph
-- Create additional nodes for performance testing
CREATE symbols:service_a CONTENT {
    id: "symbols:service_a",
    type: "symbol",
    name: "ServiceA",
    kind: "struct"
};

CREATE symbols:service_b CONTENT {
    id: "symbols:service_b", 
    type: "symbol",
    name: "ServiceB",
    kind: "struct"
};

CREATE symbols:service_c CONTENT {
    id: "symbols:service_c",
    type: "symbol", 
    name: "ServiceC",
    kind: "struct"
};

-- Create interconnected relationships
RELATE symbols:service_a->calls->symbols:service_b;
RELATE symbols:service_b->calls->symbols:service_c;
RELATE symbols:service_c->calls->symbols:auth_service;
RELATE symbols:service_a->depends_on->symbols:database_conn;

-- API Call: POST /v1/query
-- {
--   "graph": {
--     "start_nodes": ["symbols:service_a"],
--     "direction": "outbound", 
--     "max_depth": 5,
--     "algorithm": "collect",
--     "relation_types": ["calls", "depends_on"]
--   }
-- }
-- Expected: Returns all reachable nodes through complex graph

-- Example 10: Error handling - unreachable target
CREATE symbols:isolated_fn CONTENT {
    id: "symbols:isolated_fn",
    type: "symbol",
    name: "isolated_function", 
    kind: "function"
};

-- API Call: POST /v1/query
-- {
--   "graph": {
--     "start_nodes": ["symbols:main_fn"],
--     "target_node": "symbols:isolated_fn",
--     "direction": "outbound",
--     "max_depth": 5,
--     "algorithm": "shortest",
--     "relation_types": ["calls", "depends_on"]
--   }
-- }
-- Expected: Should return error "Target node not reachable"
