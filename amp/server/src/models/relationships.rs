use serde::{Deserialize, Serialize};
use uuid::Uuid;
use surrealdb::sql::Datetime;

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
