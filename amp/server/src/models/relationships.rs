#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use surrealdb::sql::Datetime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    DependsOn,
    DefinedIn,
    Calls,
    JustifiedBy,
    Modifies,
    Implements,
    Produced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub relation_type: RelationType,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub metadata: Option<serde_json::Value>,
    pub created_at: Datetime,
}

#[derive(Debug, Deserialize)]
pub struct CreateRelationshipRequest {
    #[serde(rename = "type")]
    pub relation_type: RelationType,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub metadata: Option<serde_json::Value>,
    /// Project ID for edge isolation - prevents cross-project graph contamination
    pub project_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RelationshipResponse {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub relation_type: RelationType,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub created_at: String,
}
