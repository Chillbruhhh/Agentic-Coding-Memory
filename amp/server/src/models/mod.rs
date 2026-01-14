use serde::{Deserialize, Serialize};
use uuid::Uuid;
use surrealdb::sql::Datetime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseObject {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub object_type: ObjectType,
    pub tenant_id: String,
    pub project_id: String,
    pub created_at: Datetime,
    pub updated_at: Datetime,
    pub provenance: Provenance,
    #[serde(default)]
    pub links: Vec<Link>,
    #[serde(default)]
    pub embedding: Option<Vec<f32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObjectType {
    Symbol,
    Decision,
    Changeset,
    Run,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub agent: String,
    pub model: Option<String>,
    pub tools: Option<Vec<String>>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    #[serde(rename = "type")]
    pub link_type: String,
    pub target: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    #[serde(flatten)]
    pub base: BaseObject,
    pub name: String,
    pub kind: SymbolKind,
    pub path: String,
    pub language: String,
    pub content_hash: Option<String>,
    pub signature: Option<String>,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SymbolKind {
    File,
    Module,
    Class,
    Function,
    Variable,
    Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    #[serde(flatten)]
    pub base: BaseObject,
    pub title: String,
    pub problem: String,
    pub options: Option<Vec<DecisionOption>>,
    pub rationale: String,
    pub outcome: String,
    pub status: Option<DecisionStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOption {
    pub name: String,
    pub description: String,
    pub pros: Option<Vec<String>>,
    pub cons: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DecisionStatus {
    Proposed,
    Accepted,
    Rejected,
    Superseded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    #[serde(flatten)]
    pub base: BaseObject,
    pub title: String,
    pub description: Option<String>,
    pub diff: Option<String>,
    pub files_changed: Vec<String>,
    pub tests: Option<Vec<TestResult>>,
    pub status: ChangeSetStatus,
    pub commit_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub status: TestStatus,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeSetStatus {
    Draft,
    Review,
    Approved,
    Merged,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    #[serde(flatten)]
    pub base: BaseObject,
    pub input_summary: String,
    pub outputs: Option<Vec<RunOutput>>,
    pub errors: Option<Vec<RunError>>,
    pub confidence: Option<f32>,
    pub duration_ms: Option<i64>,
    pub status: RunStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunOutput {
    #[serde(rename = "type")]
    pub output_type: RunOutputType,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunOutputType {
    File,
    Command,
    Response,
    Artifact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunError {
    pub message: String,
    pub code: Option<String>,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AmpObject {
    Symbol(Symbol),
    Decision(Decision),
    ChangeSet(ChangeSet),
    Run(Run),
}
