-- Multi-hop Graph Traversal Examples for AMP

-- Setup: Create a function call chain for testing
CREATE symbols:func_a CONTENT {
    id: "symbols:func_a",
    type: "symbol",
    tenant_id: "default",
    project_id: "multi_hop_demo",
    name: "function_a",
    kind: "function",
    path: "src/main.rs",
    language: "rust",
    signature: "fn function_a()",
    provenance: {
        agent: "demo",
        summary: "Multi-hop traversal demo function"
    }
};

CREATE symbols:func_b CONTENT {
    id: "symbols:func_b",
    type: "symbol",
    tenant_id: "default",
    project_id: "multi_hop_demo",
    name: "function_b",
    kind: "function",
    path: "src/main.rs",
    language: "rust",
    signature: "fn function_b()",
    provenance: {
        agent: "demo",
        summary: "Multi-hop traversal demo function"
    }
};

CREATE symbols:func_c CONTENT {
    id: "symbols:func_c",
    type: "symbol",
    tenant_id: "default",
    project_id: "multi_hop_demo",
    name: "function_c",
    kind: "function",
    path: "src/main.rs",
    language: "rust",
    signature: "fn function_c()",
    provenance: {
        agent: "demo",
        summary: "Multi-hop traversal demo function"
    }
};

-- Create relationships: A -> B -> C
RELATE symbols:func_a->calls->symbols:func_b;
RELATE symbols:func_b->calls->symbols:func_c;

-- 1. COLLECT Algorithm - Get all unique nodes within depth 2
-- This returns all unique nodes reachable within 2 steps
SELECT {2+collect}.->calls->objects AS collected_nodes 
FROM symbols:func_a;

-- 2. PATH Algorithm - Get all possible paths within depth 2
-- This returns arrays showing all possible paths
SELECT {2+path}.->calls->objects AS all_paths 
FROM symbols:func_a;

-- 3. SHORTEST Algorithm - Find shortest path to specific target
-- This finds the shortest path from func_a to func_c
SELECT {..3+shortest=symbols:func_c}.->calls->objects AS shortest_path 
FROM symbols:func_a;

-- 4. Multi-hop with depth limit
-- Traverse up to 3 levels deep collecting unique nodes
SELECT {3+collect}.->calls->objects AS deep_traversal 
FROM symbols:func_a;

-- 5. Bidirectional traversal (inbound)
-- Find what calls into func_c (reverse direction)
SELECT {2+collect}.<-calls<-objects AS callers 
FROM symbols:func_c;

-- 6. Complex multi-relationship traversal
-- Follow multiple relationship types in one query
SELECT {2+collect}.->(calls|depends_on|modifies)->objects AS multi_rel 
FROM symbols:func_a;

-- 7. Filtered multi-hop traversal
-- Only traverse to symbols (functions, classes, etc.)
SELECT {3+collect}.->calls->objects AS symbols_only 
FROM symbols:func_a 
WHERE type = "symbol";

-- 8. Path with inclusive flag (include starting node)
-- Include the starting node in the path result
SELECT {2+path+inclusive}.->calls->objects AS inclusive_path 
FROM symbols:func_a;

-- 9. Open-ended shortest path (no depth limit)
-- Let the algorithm determine the depth needed
SELECT {..+shortest=symbols:func_c}.->calls->objects AS unlimited_shortest 
FROM symbols:func_a;

-- 10. Multi-hop decision justification chain
-- Follow decision justifications across multiple levels
CREATE decisions:arch_decision CONTENT {
    id: "decisions:arch_decision",
    type: "decision",
    tenant_id: "default",
    project_id: "multi_hop_demo",
    title: "Use microservices architecture",
    status: "accepted",
    provenance: {
        agent: "architect",
        summary: "Architecture decision"
    }
};

CREATE changesets:implementation CONTENT {
    id: "changesets:implementation",
    type: "changeset",
    tenant_id: "default",
    project_id: "multi_hop_demo",
    title: "Implement microservices",
    status: "completed",
    provenance: {
        agent: "developer",
        summary: "Implementation changeset"
    }
};

-- Create justification chain
RELATE changesets:implementation->justified_by->decisions:arch_decision;
RELATE symbols:func_a->implements->changesets:implementation;

-- Follow the full justification chain
SELECT {3+path}.->implements->changesets.->justified_by->decisions AS justification_chain 
FROM symbols:func_a;
