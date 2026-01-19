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
    let mut query = serde_json::json!({
        "text": input.query,
        "limit": 10
    });

    if input.mode == "hybrid" {
        query["hybrid"] = serde_json::json!(true);
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

    if let Some(graph_opts) = input.graph_options {
        if let Some(graph_obj) = graph_opts.as_object() {
            if !graph_obj.is_empty() {
                query["graph"] = serde_json::Value::Object(graph_obj.clone());
            }
        }
    }

    let result = client.query(query).await?;

    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

pub async fn handle_amp_trace(
    client: &crate::amp_client::AmpClient,
    input: AmpTraceInput,
) -> Result<Vec<Content>> {
    let params = serde_json::json!({
        "from_id": input.object_id,
        "depth": input.depth
    });

    let result = client.get_relationships(params).await?;

    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}
