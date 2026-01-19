use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsData {
    #[serde(rename = "totalObjects")]
    pub total_objects: i64,
    #[serde(rename = "totalRelationships")]
    pub total_relationships: i64,
    #[serde(rename = "objectsByType")]
    pub objects_by_type: HashMap<String, i64>,
    #[serde(rename = "languageDistribution")]
    pub language_distribution: HashMap<String, i64>,
    #[serde(rename = "recentActivity")]
    pub recent_activity: Vec<ActivityItem>,
    #[serde(rename = "systemMetrics")]
    pub system_metrics: SystemMetrics,
    #[serde(rename = "indexingStats")]
    pub indexing_stats: IndexingStats,
    #[serde(rename = "requestLatency")]
    pub request_latency: RequestLatencyData,
    #[serde(rename = "errorDistribution")]
    pub error_distribution: Vec<ErrorDistributionItem>,
    #[serde(rename = "systemEvents")]
    pub system_events: Vec<SystemEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityItem {
    pub id: String,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub action: String,
    pub timestamp: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    #[serde(rename = "memoryUsage")]
    pub memory_usage: f32,
    #[serde(rename = "cpuUsage")]
    pub cpu_usage: f32,
    #[serde(rename = "diskUsage")]
    pub disk_usage: f32,
    pub uptime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingStats {
    #[serde(rename = "filesIndexed")]
    pub files_indexed: i64,
    #[serde(rename = "symbolsExtracted")]
    pub symbols_extracted: i64,
    #[serde(rename = "lastIndexTime")]
    pub last_index_time: String,
    #[serde(rename = "indexingSpeed")]
    pub indexing_speed: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLatencyData {
    pub p99: f32,
    pub p95: f32,
    pub p50: f32,
    pub avg: f32,
    pub data_points: Vec<LatencyPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyPoint {
    pub timestamp: String,
    pub latency: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDistributionItem {
    pub label: String,
    pub count: i64,
    pub percent: f32,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    pub time: String,
    pub event: String,
    pub origin: String,
    pub status: String,
    pub alert: bool,
}
