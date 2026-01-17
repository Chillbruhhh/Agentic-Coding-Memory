use std::sync::Arc;
use std::collections::{HashSet, VecDeque, BinaryHeap};
use uuid::Uuid;
use serde_json::Value;
use serde::{Deserialize, Serialize};
use tokio::time::{timeout, Duration};
use thiserror::Error;
use crate::database::Database;
use crate::surreal_json::{normalize_object_ids, take_json_values};
use crate::handlers::query::{GraphQuery, GraphDirection, TraversalAlgorithm};

#[derive(Debug, Error)]
pub enum GraphTraversalError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Query timeout")]
    Timeout,
    
    #[error("Invalid graph query: {0}")]
    InvalidQuery(String),
    
    #[error("Target node not reachable")]
    TargetNotReachable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraversalResult {
    pub nodes: Vec<Value>,
    pub paths: Option<Vec<Vec<Uuid>>>,
    pub total_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathNode {
    pub id: Uuid,
    pub object: Value,
    pub depth: usize,
    pub parent: Option<Uuid>,
}

pub struct GraphTraversalService {
    db: Arc<Database>,
}

impl GraphTraversalService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
    
    pub async fn execute_multi_hop(&self, query: &GraphQuery) -> Result<TraversalResult, GraphTraversalError> {
        let max_depth = query.max_depth.unwrap_or(3);
        
        match &query.algorithm {
            Some(TraversalAlgorithm::Collect) => {
                self.execute_collect_traversal(query, max_depth).await
            }
            Some(TraversalAlgorithm::Path) => {
                self.execute_path_traversal(query, max_depth).await
            }
            Some(TraversalAlgorithm::Shortest) => {
                self.execute_shortest_path(query, max_depth).await
            }
            None => {
                Err(GraphTraversalError::InvalidQuery("Algorithm not specified for multi-hop query".to_string()))
            }
        }
    }
    
    async fn execute_collect_traversal(&self, query: &GraphQuery, max_depth: usize) -> Result<TraversalResult, GraphTraversalError> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut all_nodes = Vec::new();
        
        // Initialize with start nodes
        for start_id in &query.start_nodes {
            queue.push_back((*start_id, 0)); // (node_id, depth)
            visited.insert(*start_id);
        }
        
        let direction = query.direction.as_ref().unwrap_or(&GraphDirection::Outbound);
        let relation_list = if let Some(types) = &query.relation_types {
            types.join(", ")
        } else {
            "depends_on, defined_in, calls, justified_by, modifies, implements, produced".to_string()
        };
        
        while let Some((current_id, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            
            // Build query for current node's neighbors
            let query_str = match direction {
                GraphDirection::Outbound => {
                    format!("SELECT ->{}->objects AS connected FROM objects:`{}`", relation_list, current_id)
                }
                GraphDirection::Inbound => {
                    format!("SELECT <-{}<-objects AS connected FROM objects:`{}`", relation_list, current_id)
                }
                GraphDirection::Both => {
                    format!("SELECT ->{}->objects AS outbound, <-{}<-objects AS inbound FROM objects:`{}`", 
                        relation_list, relation_list, current_id)
                }
            };
            
            tracing::debug!("Collect traversal query at depth {}: {}", depth, query_str);
            
            let query_result = timeout(
                Duration::from_secs(5),
                self.db.client.query(query_str)
            ).await;
            
            let connected_nodes: Vec<Value> = match query_result {
                Ok(Ok(mut response)) => {
                    let raw_results: Vec<Value> = take_json_values(&mut response, 0);
                    
                    // Extract connected objects
                    let mut connected = raw_results
                        .into_iter()
                        .filter_map(|v| {
                            if let Some(obj) = v.as_object() {
                                if direction == &GraphDirection::Both {
                                    // Handle both directions
                                    let mut nodes = Vec::new();
                                    if let Some(outbound) = obj.get("outbound") {
                                        if let Some(arr) = outbound.as_array() {
                                            nodes.extend(arr.clone());
                                        } else {
                                            nodes.push(outbound.clone());
                                        }
                                    }
                                    if let Some(inbound) = obj.get("inbound") {
                                        if let Some(arr) = inbound.as_array() {
                                            nodes.extend(arr.clone());
                                        } else {
                                            nodes.push(inbound.clone());
                                        }
                                    }
                                    Some(nodes)
                                } else {
                                    // Handle single direction
                                    obj.get("connected").map(|v| {
                                        if let Some(arr) = v.as_array() {
                                            arr.clone()
                                        } else {
                                            vec![v.clone()]
                                        }
                                    })
                                }
                            } else {
                                None
                            }
                        })
                        .flatten()
                        .collect::<Vec<_>>();
                    
                    normalize_object_ids(&mut connected);
                    connected
                }
                Ok(Err(e)) => {
                    tracing::error!("Database error in collect traversal: {}", e);
                    return Err(GraphTraversalError::DatabaseError(e.to_string()));
                }
                Err(_) => {
                    tracing::error!("Timeout in collect traversal");
                    return Err(GraphTraversalError::Timeout);
                }
            };
            
            // Process connected nodes
            for node in connected_nodes {
                if let Some(node_id_str) = node.get("id").and_then(|v| v.as_str()) {
                    if let Ok(node_id) = Uuid::parse_str(node_id_str.trim_start_matches("objects:")) {
                        if !visited.contains(&node_id) {
                            visited.insert(node_id);
                            queue.push_back((node_id, depth + 1));
                            all_nodes.push(node);
                        }
                    }
                }
            }
        }
        
        Ok(TraversalResult {
            total_count: all_nodes.len(),
            nodes: all_nodes,
            paths: None,
        })
    }
    
    async fn execute_path_traversal(&self, query: &GraphQuery, max_depth: usize) -> Result<TraversalResult, GraphTraversalError> {
        let mut all_paths = Vec::new();
        let mut all_nodes = Vec::new();
        
        for start_id in &query.start_nodes {
            // Use iterative approach instead of recursive to avoid lifetime issues
            let paths = self.find_paths_iterative(*start_id, query, max_depth).await?;
            all_paths.extend(paths);
        }
        
        // Collect unique nodes from all paths
        let mut unique_nodes = HashSet::new();
        for path in &all_paths {
            for node_id in path {
                unique_nodes.insert(*node_id);
            }
        }
        
        // Fetch node objects for unique nodes
        if !unique_nodes.is_empty() {
            let node_ids: Vec<String> = unique_nodes.iter()
                .map(|id| format!("objects:`{}`", id))
                .collect();
            
            let query_str = format!("SELECT * FROM [{}]", node_ids.join(", "));
            
            let query_result = timeout(
                Duration::from_secs(5),
                self.db.client.query(query_str)
            ).await;
            
            match query_result {
                Ok(Ok(mut response)) => {
                    let mut nodes: Vec<Value> = take_json_values(&mut response, 0);
                    normalize_object_ids(&mut nodes);
                    all_nodes = nodes;
                }
                Ok(Err(e)) => {
                    tracing::error!("Database error in path traversal: {}", e);
                    return Err(GraphTraversalError::DatabaseError(e.to_string()));
                }
                Err(_) => {
                    tracing::error!("Timeout in path traversal");
                    return Err(GraphTraversalError::Timeout);
                }
            }
        }
        
        Ok(TraversalResult {
            total_count: all_nodes.len(),
            nodes: all_nodes,
            paths: Some(all_paths),
        })
    }
    
    async fn find_paths_iterative(&self, start_id: Uuid, query: &GraphQuery, max_depth: usize) -> Result<Vec<Vec<Uuid>>, GraphTraversalError> {
        let mut all_paths = Vec::new();
        let mut stack = vec![(start_id, vec![start_id], 1)]; // (current_id, path, depth)
        
        let direction = query.direction.as_ref().unwrap_or(&GraphDirection::Outbound);
        let relation_list = if let Some(types) = &query.relation_types {
            types.join(", ")
        } else {
            "depends_on, defined_in, calls, justified_by, modifies, implements, produced".to_string()
        };
        
        while let Some((current_id, current_path, depth)) = stack.pop() {
            // Add current path to results
            all_paths.push(current_path.clone());
            
            if depth >= max_depth {
                continue;
            }
            
            // Build query for current node's neighbors
            let query_str = match direction {
                GraphDirection::Outbound => {
                    format!("SELECT ->{}->objects AS connected FROM objects:`{}`", relation_list, current_id)
                }
                GraphDirection::Inbound => {
                    format!("SELECT <-{}<-objects AS connected FROM objects:`{}`", relation_list, current_id)
                }
                GraphDirection::Both => {
                    format!("SELECT ->{}->objects AS outbound, <-{}<-objects AS inbound FROM objects:`{}`", 
                        relation_list, relation_list, current_id)
                }
            };
            
            let query_result = timeout(
                Duration::from_secs(5),
                self.db.client.query(query_str)
            ).await;
            
            let connected_nodes: Vec<Uuid> = match query_result {
                Ok(Ok(mut response)) => {
                    let raw_results: Vec<Value> = take_json_values(&mut response, 0);
                    
                    raw_results
                        .into_iter()
                        .filter_map(|v| {
                            if let Some(obj) = v.as_object() {
                                let mut node_ids = Vec::new();
                                
                                if direction == &GraphDirection::Both {
                                    // Handle both directions
                                    if let Some(outbound) = obj.get("outbound") {
                                        if let Some(arr) = outbound.as_array() {
                                            for node in arr {
                                                if let Some(id_str) = node.get("id").and_then(|v| v.as_str()) {
                                                    if let Ok(node_id) = Uuid::parse_str(id_str.trim_start_matches("objects:")) {
                                                        node_ids.push(node_id);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if let Some(inbound) = obj.get("inbound") {
                                        if let Some(arr) = inbound.as_array() {
                                            for node in arr {
                                                if let Some(id_str) = node.get("id").and_then(|v| v.as_str()) {
                                                    if let Ok(node_id) = Uuid::parse_str(id_str.trim_start_matches("objects:")) {
                                                        node_ids.push(node_id);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    // Handle single direction
                                    if let Some(connected) = obj.get("connected") {
                                        let nodes = if let Some(arr) = connected.as_array() {
                                            arr.clone()
                                        } else {
                                            vec![connected.clone()]
                                        };
                                        
                                        for node in nodes {
                                            if let Some(id_str) = node.get("id").and_then(|v| v.as_str()) {
                                                if let Ok(node_id) = Uuid::parse_str(id_str.trim_start_matches("objects:")) {
                                                    node_ids.push(node_id);
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                Some(node_ids)
                            } else {
                                None
                            }
                        })
                        .flatten()
                        .collect()
                }
                Ok(Err(e)) => {
                    tracing::error!("Database error in path finding: {}", e);
                    return Err(GraphTraversalError::DatabaseError(e.to_string()));
                }
                Err(_) => {
                    tracing::error!("Timeout in path finding");
                    return Err(GraphTraversalError::Timeout);
                }
            };
            
            // Add connected nodes to stack (avoid cycles by checking if node is already in path)
            for next_id in connected_nodes {
                if !current_path.contains(&next_id) {
                    let mut new_path = current_path.clone();
                    new_path.push(next_id);
                    stack.push((next_id, new_path, depth + 1));
                }
            }
        }
        
        Ok(all_paths)
    }
    
    async fn execute_shortest_path(&self, query: &GraphQuery, max_depth: usize) -> Result<TraversalResult, GraphTraversalError> {
        let target_id = query.target_node.ok_or_else(|| {
            GraphTraversalError::InvalidQuery("Target node required for shortest path algorithm".to_string())
        })?;
        
        for start_id in &query.start_nodes {
            if let Some(path) = self.find_shortest_path(*start_id, target_id, query, max_depth).await? {
                // Fetch node objects for the path
                let node_ids: Vec<String> = path.iter()
                    .map(|id| format!("objects:`{}`", id))
                    .collect();
                
                let query_str = format!("SELECT * FROM [{}]", node_ids.join(", "));
                
                let query_result = timeout(
                    Duration::from_secs(5),
                    self.db.client.query(query_str)
                ).await;
                
                let nodes: Vec<Value> = match query_result {
                    Ok(Ok(mut response)) => {
                        let mut nodes: Vec<Value> = take_json_values(&mut response, 0);
                        normalize_object_ids(&mut nodes);
                        nodes
                    }
                    Ok(Err(e)) => {
                        tracing::error!("Database error in shortest path: {}", e);
                        return Err(GraphTraversalError::DatabaseError(e.to_string()));
                    }
                    Err(_) => {
                        tracing::error!("Timeout in shortest path");
                        return Err(GraphTraversalError::Timeout);
                    }
                };
                
                return Ok(TraversalResult {
                    total_count: nodes.len(),
                    nodes,
                    paths: Some(vec![path]),
                });
            }
        }
        
        Err(GraphTraversalError::TargetNotReachable)
    }
    
    async fn find_shortest_path(
        &self,
        start_id: Uuid,
        target_id: Uuid,
        query: &GraphQuery,
        max_depth: usize
    ) -> Result<Option<Vec<Uuid>>, GraphTraversalError> {
        use std::cmp::Reverse;
        
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct PathState {
            node_id: Uuid,
            distance: usize,
            path: Vec<Uuid>,
        }
        
        impl Ord for PathState {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.distance.cmp(&other.distance)
            }
        }
        
        impl PartialOrd for PathState {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
        
        let mut heap = BinaryHeap::new();
        let mut visited = HashSet::new();
        
        heap.push(Reverse(PathState {
            node_id: start_id,
            distance: 0,
            path: vec![start_id],
        }));
        
        let direction = query.direction.as_ref().unwrap_or(&GraphDirection::Outbound);
        let relation_list = if let Some(types) = &query.relation_types {
            types.join(", ")
        } else {
            "depends_on, defined_in, calls, justified_by, modifies, implements, produced".to_string()
        };
        
        while let Some(Reverse(current)) = heap.pop() {
            if current.node_id == target_id {
                return Ok(Some(current.path));
            }
            
            if current.distance >= max_depth || visited.contains(&current.node_id) {
                continue;
            }
            
            visited.insert(current.node_id);
            
            // Build query for current node's neighbors
            let query_str = match direction {
                GraphDirection::Outbound => {
                    format!("SELECT ->{}->objects AS connected FROM objects:`{}`", relation_list, current.node_id)
                }
                GraphDirection::Inbound => {
                    format!("SELECT <-{}<-objects AS connected FROM objects:`{}`", relation_list, current.node_id)
                }
                GraphDirection::Both => {
                    format!("SELECT ->{}->objects AS outbound, <-{}<-objects AS inbound FROM objects:`{}`", 
                        relation_list, relation_list, current.node_id)
                }
            };
            
            let query_result = timeout(
                Duration::from_secs(5),
                self.db.client.query(query_str)
            ).await;
            
            let connected_nodes: Vec<Uuid> = match query_result {
                Ok(Ok(mut response)) => {
                    let raw_results: Vec<Value> = take_json_values(&mut response, 0);
                    
                    raw_results
                        .into_iter()
                        .filter_map(|v| {
                            if let Some(obj) = v.as_object() {
                                let mut node_ids = Vec::new();
                                
                                if direction == &GraphDirection::Both {
                                    // Handle both directions
                                    if let Some(outbound) = obj.get("outbound") {
                                        if let Some(arr) = outbound.as_array() {
                                            for node in arr {
                                                if let Some(id_str) = node.get("id").and_then(|v| v.as_str()) {
                                                    if let Ok(node_id) = Uuid::parse_str(id_str.trim_start_matches("objects:")) {
                                                        node_ids.push(node_id);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if let Some(inbound) = obj.get("inbound") {
                                        if let Some(arr) = inbound.as_array() {
                                            for node in arr {
                                                if let Some(id_str) = node.get("id").and_then(|v| v.as_str()) {
                                                    if let Ok(node_id) = Uuid::parse_str(id_str.trim_start_matches("objects:")) {
                                                        node_ids.push(node_id);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    // Handle single direction
                                    if let Some(connected) = obj.get("connected") {
                                        let nodes = if let Some(arr) = connected.as_array() {
                                            arr.clone()
                                        } else {
                                            vec![connected.clone()]
                                        };
                                        
                                        for node in nodes {
                                            if let Some(id_str) = node.get("id").and_then(|v| v.as_str()) {
                                                if let Ok(node_id) = Uuid::parse_str(id_str.trim_start_matches("objects:")) {
                                                    node_ids.push(node_id);
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                Some(node_ids)
                            } else {
                                None
                            }
                        })
                        .flatten()
                        .collect()
                }
                Ok(Err(e)) => {
                    tracing::error!("Database error in shortest path: {}", e);
                    return Err(GraphTraversalError::DatabaseError(e.to_string()));
                }
                Err(_) => {
                    tracing::error!("Timeout in shortest path");
                    return Err(GraphTraversalError::Timeout);
                }
            };
            
            // Add neighbors to heap
            for next_id in connected_nodes {
                if !visited.contains(&next_id) {
                    let mut new_path = current.path.clone();
                    new_path.push(next_id);
                    
                    heap.push(Reverse(PathState {
                        node_id: next_id,
                        distance: current.distance + 1,
                        path: new_path,
                    }));
                }
            }
        }
        
        Ok(None)
    }
}
