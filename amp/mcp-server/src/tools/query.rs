use anyhow::Result;
use rmcp::model::Content;
use schemars::{
    JsonSchema,
    SchemaGenerator,
    schema::{InstanceType, ObjectValidation, Schema, SchemaObject, SingleOrVec},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpQueryInput {
    pub query: String,
    #[serde(default = "default_mode")]
    pub mode: String,
    #[schemars(schema_with = "schema_any_object")]
    pub filters: Option<Value>,
    #[schemars(schema_with = "schema_any_object")]
    pub graph_options: Option<Value>,
    pub graph_intersect: Option<bool>,
    pub graph_autoseed: Option<bool>,
    pub limit: Option<u64>,
}

fn default_mode() -> String {
    "hybrid".to_string()
}

fn schema_any_object(_gen: &mut SchemaGenerator) -> Schema {
    let mut schema = SchemaObject::default();
    schema.instance_type = Some(SingleOrVec::Single(Box::new(InstanceType::Object)));
    schema.object = Some(Box::new(ObjectValidation {
        additional_properties: Some(Box::new(Schema::Bool(true))),
        ..Default::default()
    }));
    Schema::Object(schema)
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpTraceInput {
    pub object_id: String,
    #[serde(default = "default_depth")]
    pub depth: i32,
}

fn default_depth() -> i32 {
    2
}

pub async fn handle_amp_query(
    client: &crate::amp_client::AmpClient,
    input: AmpQueryInput,
) -> Result<Vec<Content>> {
    let mode = input.mode.as_str();
    let is_hybrid = mode == "hybrid";
    let mut query = serde_json::json!({
        "text": input.query,
        "hybrid": is_hybrid,
        "limit": input.limit.unwrap_or(5)
    });

    if mode == "vector" || is_hybrid {
        query["vector"] = serde_json::json!(null); // Will auto-generate from text
    }

    if let Some(graph_intersect) = input.graph_intersect {
        query["graph_intersect"] = serde_json::json!(graph_intersect);
    }

    if let Some(graph_autoseed) = input.graph_autoseed {
        query["graph_autoseed"] = serde_json::json!(graph_autoseed);
    }

    if let Some(filters) = input.filters {
        if let Some(mut filters_obj) = filters.as_object().cloned() {
            if let Some(type_value) = filters_obj.get_mut("type") {
                if let Some(type_str) = type_value.as_str() {
                    *type_value = serde_json::json!([type_str]);
                }
            }
            if !filters_obj.is_empty() {
                query["filters"] = serde_json::Value::Object(filters_obj);
            }
        }
    }

    // Enable graph traversal only when we have start_nodes
    if let Some(graph_opts) = input.graph_options {
        if let Some(graph_obj) = graph_opts.as_object() {
            let has_start_nodes = graph_obj
                .get("start_nodes")
                .and_then(|v| v.as_array())
                .map(|arr| !arr.is_empty())
                .unwrap_or(false);
            if mode == "graph" && !has_start_nodes {
                anyhow::bail!("graph mode requires graph_options.start_nodes");
            }
            if has_start_nodes {
                query["graph"] = serde_json::Value::Object(graph_obj.clone());
            }
        }
    } else if mode == "graph" {
        anyhow::bail!("graph mode requires graph_options");
    }

    if mode == "text" {
        query["hybrid"] = serde_json::json!(false);
        query.as_object_mut().map(|obj| obj.remove("vector"));
        query.as_object_mut().map(|obj| obj.remove("graph"));
    }

    if mode == "vector" {
        query["hybrid"] = serde_json::json!(false);
        query.as_object_mut().map(|obj| obj.remove("graph"));
    }

    if mode == "graph" {
        query["hybrid"] = serde_json::json!(false);
        query.as_object_mut().map(|obj| obj.remove("vector"));
    }

    let result = client.query(query).await?;
    
    // Summarize RRF results with scoring details
    let summary = summarize_rrf_results(&result, &input.query)?;
    
    Ok(vec![Content::text(summary)])
}

fn summarize_rrf_results(result: &Value, query: &str) -> Result<String> {
    let mut summary = format!("Hybrid Query (RRF): {}\n\n", query);
    
    if let Some(results) = result.get("results").and_then(|r| r.as_array()) {
        summary.push_str(&format!("Found {} results (ranked by Reciprocal Rank Fusion):\n\n", results.len()));
        
        for (i, item) in results.iter().take(5).enumerate() {
            // Server returns "score" field (mapped from hybrid's total_score)
            let total_score = item.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0);
            let text_score = item.get("text_score").and_then(|s| s.as_f64());
            let vector_score = item.get("vector_score").and_then(|s| s.as_f64());
            let graph_score = item.get("graph_score").and_then(|s| s.as_f64());
            
            if let Some(obj) = item.get("object") {
                let obj_id = obj.get("id").and_then(|i| i.as_str()).unwrap_or("unknown");
                if let Some(obj_type) = obj.get("type").and_then(|t| t.as_str()) {
                    match obj_type {
                        "symbol" => {
                            let name = obj.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                            let kind = obj.get("kind").and_then(|k| k.as_str()).unwrap_or("unknown");
                            let path = obj.get("path").and_then(|p| p.as_str()).unwrap_or("unknown");
                            summary.push_str(&format!("{}. Symbol: {} ({}) in {}\n", i+1, name, kind, path));
                            summary.push_str(&format!("   id: {}\n", obj_id));
                        },
                        "decision" => {
                            let title = obj.get("title").and_then(|t| t.as_str()).unwrap_or("unknown");
                            let status = obj.get("status").and_then(|s| s.as_str()).unwrap_or("unknown");
                            summary.push_str(&format!("{}. Decision: {} ({})\n", i+1, title, status));
                            summary.push_str(&format!("   id: {}\n", obj_id));
                        },
                        "changeset" => {
                            let title = obj.get("title").and_then(|t| t.as_str()).unwrap_or("unknown");
                            let files = obj.get("files_changed").and_then(|f| f.as_array()).map(|arr| arr.len()).unwrap_or(0);
                            summary.push_str(&format!("{}. ChangeSet: {} ({} files)\n", i+1, title, files));
                            summary.push_str(&format!("   id: {}\n", obj_id));
                        },
                        "filechunk" => {
                            let path = obj.get("file_path").and_then(|p| p.as_str()).unwrap_or("unknown");
                            let lines = format!("{}-{}", 
                                obj.get("start_line").and_then(|l| l.as_u64()).unwrap_or(0),
                                obj.get("end_line").and_then(|l| l.as_u64()).unwrap_or(0)
                            );
                            summary.push_str(&format!("{}. FileChunk: {} (lines {})\n", i+1, path, lines));
                            summary.push_str(&format!("   id: {}\n", obj_id));
                        },
                        _ => {
                            let id = obj.get("id").and_then(|i| i.as_str()).unwrap_or("unknown");
                            summary.push_str(&format!("{}. {} ({})\n", i+1, obj_type, &id[..8.min(id.len())]));
                            summary.push_str(&format!("   id: {}\n", obj_id));
                        }
                    }
                }
            }
            
            // Add RRF scoring breakdown
            summary.push_str(&format!("   RRF Score: {:.4}", total_score));
            if text_score.is_some() || vector_score.is_some() || graph_score.is_some() {
                summary.push_str(" (");
                let mut parts = Vec::new();
                if let Some(ts) = text_score { parts.push(format!("text:{:.3}", ts)); }
                if let Some(vs) = vector_score { parts.push(format!("vector:{:.3}", vs)); }
                if let Some(gs) = graph_score { parts.push(format!("graph:{:.3}", gs)); }
                summary.push_str(&parts.join(", "));
                summary.push_str(")");
            }
            summary.push_str("\n\n");
        }
        
        if results.len() > 5 {
            summary.push_str(&format!("... and {} more results\n", results.len() - 5));
        }
    } else {
        summary.push_str("No results found\n");
    }
    
    Ok(summary)
}

pub async fn handle_amp_trace(
    client: &crate::amp_client::AmpClient,
    input: AmpTraceInput,
) -> Result<Vec<Content>> {
    let params = serde_json::json!({
        "from_id": input.object_id,
        "depth": input.depth.min(2)  // Limit depth to prevent massive responses
    });

    let result = client.get_relationships(params).await?;
    
    // Summarize relationships instead of returning raw JSON
    let summary = summarize_trace_results(&result, &input.object_id, input.depth)?;
    
    Ok(vec![Content::text(summary)])
}

fn summarize_trace_results(result: &Value, object_id: &str, depth: i32) -> Result<String> {
    let mut summary = format!("Trace for object: {} (depth: {})\n\n", object_id, depth);
    
    if let Some(relationships) = result.get("relationships").and_then(|r| r.as_array()) {
        summary.push_str(&format!("Found {} relationships:\n\n", relationships.len()));
        
        for (i, rel) in relationships.iter().take(10).enumerate() {
            let rel_type = rel.get("type").and_then(|t| t.as_str()).unwrap_or("unknown");
            let from_id = rel.get("from").and_then(|f| f.as_str()).unwrap_or("unknown");
            let to_id = rel.get("to").and_then(|t| t.as_str()).unwrap_or("unknown");
            
            summary.push_str(&format!("{}. {} -> {} ({})\n", i+1, 
                &from_id[..8.min(from_id.len())], 
                &to_id[..8.min(to_id.len())], 
                rel_type
            ));
        }
        
        if relationships.len() > 10 {
            summary.push_str(&format!("\n... and {} more relationships\n", relationships.len() - 10));
        }
    } else {
        summary.push_str("No relationships found\n");
    }
    
    Ok(summary)
}
