use anyhow::Result;
use std::sync::Arc;
use std::collections::{HashMap, VecDeque};
use sysinfo::{Disks, System};
use chrono::Utc;
use crate::{
    database::Database,
    models::analytics::{AnalyticsData, ActivityItem, SystemMetrics, IndexingStats, RequestLatencyData, LatencyPoint, ErrorDistributionItem, SystemEvent},
    surreal_json::take_json_values,
};

#[derive(Debug, Clone)]
struct LatencyBucket {
    timestamp: String,
    sum: f32,
    count: u32,
}

pub struct AnalyticsService {
    db: Arc<Database>,
    system: std::sync::Mutex<System>,
    latency_points: std::sync::Mutex<VecDeque<LatencyBucket>>,
}

impl AnalyticsService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            system: std::sync::Mutex::new(System::new_all()),
            latency_points: std::sync::Mutex::new(VecDeque::new()),
        }
    }

    pub fn record_request_latency(&self, latency_ms: f32) {
        let mut points = self.latency_points.lock().unwrap();
        let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        if let Some(last) = points.back_mut() {
            if last.timestamp == timestamp {
                last.sum += latency_ms;
                last.count += 1;
            } else {
                points.push_back(LatencyBucket {
                    timestamp,
                    sum: latency_ms,
                    count: 1,
                });
            }
        } else {
            points.push_back(LatencyBucket {
                timestamp,
                sum: latency_ms,
                count: 1,
            });
        }

        while points.len() > 120 {
            points.pop_front();
        }
    }

    pub async fn get_analytics(&self) -> Result<AnalyticsData> {
        // Collect all analytics data in parallel
        let (
            total_objects,
            total_relationships,
            objects_by_type,
            language_distribution,
            recent_activity,
            system_metrics,
            indexing_stats,
            request_latency,
            error_distribution,
            system_events,
        ) = tokio::try_join!(
            self.get_total_objects(),
            self.get_total_relationships(),
            self.get_objects_by_type(),
            self.get_language_distribution(),
            self.get_recent_activity(),
            self.get_system_metrics(),
            self.get_indexing_stats(),
            self.get_request_latency(),
            self.get_error_distribution(),
            self.get_system_events(),
        )?;

        Ok(AnalyticsData {
            total_objects,
            total_relationships,
            objects_by_type,
            language_distribution,
            recent_activity,
            system_metrics,
            indexing_stats,
            request_latency,
            error_distribution,
            system_events,
        })
    }

    async fn get_total_objects(&self) -> Result<i64> {
        let query = "SELECT VALUE string::concat(id) FROM objects";
        let mut result = self.db.client.query(query).await?;
        let values = take_json_values(&mut result, 0);
        let count = values.len() as i64;
        
        tracing::info!("Total objects count: {}", count);
        Ok(count)
    }

    async fn get_total_relationships(&self) -> Result<i64> {
        let relationship_tables = [
            "depends_on",
            "defined_in",
            "calls",
            "justified_by",
            "modifies",
            "implements",
            "produced",
        ];

        let mut total = 0_i64;

        for table in relationship_tables {
            let query = format!("SELECT VALUE string::concat(id) FROM {}", table);
            match self.db.client.query(&query).await {
                Ok(mut result) => {
                    let values = take_json_values(&mut result, 0);
                    let count = values.len() as i64;
                    tracing::info!("Relationships count for {}: {}", table, count);
                    total += count;
                }
                Err(err) => {
                    tracing::warn!("Failed to count relationships in {}: {}", table, err);
                }
            }
        }

        Ok(total)
    }

    async fn get_objects_by_type(&self) -> Result<HashMap<String, i64>> {
        let mut map = HashMap::new();

        // Use string::concat to force string conversion and avoid enum serialization issues
        let symbol_kind_query =
            "SELECT string::lowercase(string::concat('', kind)) AS kind, count() AS count FROM objects WHERE string::lowercase(string::concat('', type)) = 'symbol' AND kind IS NOT NULL GROUP BY kind";
        let mut result = self.db.client.query(symbol_kind_query).await?;
        let kinds: Vec<serde_json::Value> = take_json_values(&mut result, 0);
        tracing::info!("Symbol kind query returned {} rows", kinds.len());

        for kind_value in kinds {
            if let (Some(kind), Some(count)) = (
                kind_value.get("kind").and_then(|v| v.as_str()),
                kind_value.get("count").and_then(|v| v.as_i64())
            ) {
                *map.entry(kind.to_string()).or_insert(0) += count;
            }
        }

        let other_type_query =
            "SELECT string::lowercase(string::concat('', type)) AS obj_type, count() AS count FROM objects WHERE string::lowercase(string::concat('', type)) != 'symbol' GROUP BY obj_type";
        let mut result = self.db.client.query(other_type_query).await?;
        let types: Vec<serde_json::Value> = take_json_values(&mut result, 0);

        tracing::info!(
            "Non-symbol type query returned {} rows",
            types.len()
        );

        for type_value in types {
            if let (Some(obj_type), Some(count)) = (
                type_value.get("obj_type").and_then(|v| v.as_str()),
                type_value.get("count").and_then(|v| v.as_i64())
            ) {
                *map.entry(obj_type.to_string()).or_insert(0) += count;
            }
        }

        Ok(map)
    }

    async fn get_language_distribution(&self) -> Result<HashMap<String, i64>> {
        let query = "SELECT language, count() AS count FROM objects WHERE string::lowercase(string::concat('', type)) = 'symbol' AND language IS NOT NULL GROUP BY language";
        let mut result = self.db.client.query(query).await?;
        let counts: Vec<serde_json::Value> = take_json_values(&mut result, 0);

        let mut map = HashMap::new();
        for count in counts {
            if let (Some(language), Some(count_val)) = (
                count.get("language").and_then(|v| v.as_str()),
                count.get("count").and_then(|v| v.as_i64())
            ) {
                map.insert(language.to_string(), count_val);
            }
        }
        Ok(map)
    }

    async fn get_recent_activity(&self) -> Result<Vec<ActivityItem>> {
        let query = "SELECT string::concat('', id) AS id, string::concat('', type) AS type, string::concat('', created_at) AS created_at, string::concat('', updated_at) AS updated_at FROM objects ORDER BY created_at DESC LIMIT 10";
        let mut result = self.db.client.query(query).await?;
        let objects: Vec<serde_json::Value> = take_json_values(&mut result, 0);

        let mut activities = Vec::new();
        for (i, obj) in objects.iter().enumerate() {
            let _id = obj.get("id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
            let obj_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
            let created_at = obj.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string();

            activities.push(ActivityItem {
                id: format!("activity_{}", i),
                activity_type: obj_type.clone(),
                action: "Created".to_string(),
                timestamp: created_at,
                details: format!("{} object created", obj_type),
            });
        }

        Ok(activities)
    }

    async fn get_system_metrics(&self) -> Result<SystemMetrics> {
        let mut system = self.system.lock().unwrap();
        system.refresh_all();

        let memory_usage = (system.used_memory() as f32 / system.total_memory() as f32) * 100.0;
        let cpu_usage = system.global_cpu_info().cpu_usage();
        
        let disks = Disks::new_with_refreshed_list();
        let mut total_disk = 0_u64;
        let mut available_disk = 0_u64;
        for disk in disks.iter() {
            total_disk += disk.total_space();
            available_disk += disk.available_space();
        }
        let disk_usage = if total_disk > 0 {
            ((total_disk - available_disk) as f32 / total_disk as f32) * 100.0
        } else {
            0.0
        };
        
        let uptime_secs = System::uptime();
        let uptime = format!("{}h {}m", 
            uptime_secs / 3600, 
            (uptime_secs % 3600) / 60
        );

        Ok(SystemMetrics {
            memory_usage,
            cpu_usage,
            disk_usage,
            uptime,
        })
    }

    async fn get_indexing_stats(&self) -> Result<IndexingStats> {
        // Get symbol count as files indexed
        let files_query =
            "SELECT count() AS total FROM objects WHERE string::lowercase(string::concat('', type)) = 'symbol' AND string::lowercase(string::concat('', kind)) = 'file'";
        let mut result = self.db.client.query(files_query).await?;
        let files_counts: Vec<serde_json::Value> = take_json_values(&mut result, 0);
        let files_indexed = files_counts.first()
            .and_then(|v| v.get("total"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // Get total symbols as symbols extracted
        let symbols_query = "SELECT count() AS total FROM objects WHERE string::lowercase(string::concat('', type)) = 'symbol'";
        let mut result = self.db.client.query(symbols_query).await?;
        let symbols_counts: Vec<serde_json::Value> = take_json_values(&mut result, 0);
        let symbols_extracted = symbols_counts.first()
            .and_then(|v| v.get("total"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // Get most recent symbol creation time
        let time_query =
            "SELECT string::concat('', created_at) AS created_at FROM objects WHERE string::lowercase(string::concat('', type)) = 'symbol' ORDER BY created_at DESC LIMIT 1";
        let mut result = self.db.client.query(time_query).await?;
        let last_times: Vec<serde_json::Value> = take_json_values(&mut result, 0);
        let last_index_time = last_times.first()
            .and_then(|v| v.get("created_at"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(IndexingStats {
            files_indexed,
            symbols_extracted,
            last_index_time,
            indexing_speed: String::new(),
        })
    }

    async fn get_request_latency(&self) -> Result<RequestLatencyData> {
        let points = self.latency_points.lock().unwrap();
        if points.is_empty() {
            return Ok(RequestLatencyData {
                p99: 0.0,
                p95: 0.0,
                p50: 0.0,
                avg: 0.0,
                data_points: Vec::new(),
            });
        }

        let mut data_points: Vec<LatencyPoint> = points
            .iter()
            .map(|bucket| LatencyPoint {
                timestamp: bucket.timestamp.clone(),
                latency: bucket.sum / bucket.count.max(1) as f32,
            })
            .collect();

        let mut latencies: Vec<f32> = data_points.iter().map(|p| p.latency).collect();
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let percentile = |p: f32| -> f32 {
            let idx = ((latencies.len() - 1) as f32 * p).round() as usize;
            latencies.get(idx).copied().unwrap_or(0.0)
        };

        let avg = latencies.iter().copied().sum::<f32>() / latencies.len() as f32;

        Ok(RequestLatencyData {
            p99: percentile(0.99),
            p95: percentile(0.95),
            p50: percentile(0.50),
            avg,
            data_points: {
                if data_points.len() > 120 {
                    data_points.drain(..data_points.len() - 120);
                }
                data_points
            },
        })
    }

    async fn get_error_distribution(&self) -> Result<Vec<ErrorDistributionItem>> {
        Ok(Vec::new())
    }

    async fn get_system_events(&self) -> Result<Vec<SystemEvent>> {
        // Get recent activity from objects table
        let query = "SELECT string::concat('', id) AS id, string::concat('', type) AS type, string::concat('', created_at) AS created_at, string::concat('', updated_at) AS updated_at FROM objects ORDER BY created_at DESC LIMIT 20";
        let mut result = self.db.client.query(query).await?;
        let objects: Vec<serde_json::Value> = take_json_values(&mut result, 0);

        let mut events = Vec::new();
        for obj in objects {
            let obj_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
            let created_at = obj.get("created_at").and_then(|v| v.as_str()).unwrap_or("");

            // Parse timestamp and format as HH:MM:SS
            let time = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(created_at) {
                dt.format("%H:%M:%S").to_string()
            } else {
                "00:00:00".to_string()
            };

            events.push(SystemEvent {
                time,
                event: format!("{} object indexed", obj_type.to_uppercase()),
                origin: "PARSER".to_string(),
                status: "Success".to_string(),
                alert: false,
            });
        }

        // Add system startup event if no events
        if events.is_empty() {
            events.push(SystemEvent {
                time: chrono::Local::now().format("%H:%M:%S").to_string(),
                event: "System initialized".to_string(),
                origin: "CORE".to_string(),
                status: "Success".to_string(),
                alert: false,
            });
        }

        Ok(events)
    }
}
