use serde_json::Value as JsonValue;
use surrealdb::Response;

pub fn take_json_values(response: &mut Response, index: usize) -> Vec<JsonValue> {
    // Simple approach: just try direct JSON deserialization
    match response.take::<Vec<JsonValue>>(index) {
        Ok(values) => {
            tracing::info!("Successfully decoded {} values as JSON", values.len());
            values
        }
        Err(e) => {
            tracing::warn!("Failed to decode as JSON values: {}", e);
            Vec::new()
        }
    }
}

pub fn take_json_value(response: &mut Response, index: usize) -> Option<JsonValue> {
    match response.take::<Option<JsonValue>>(index) {
        Ok(value) => value,
        Err(err) => {
            tracing::warn!("Failed to decode response value: {}", err);
            None
        }
    }
}

pub fn normalize_object_id(value: &mut JsonValue) {
    let Some(map) = value.as_object_mut() else {
        return;
    };
    
    // If we have id_string, use that as the main id
    if let Some(id_string) = map.remove("id_string") {
        if let Some(id_str) = id_string.as_str() {
            // Normalize: extract UUID from "objects:⟨uuid⟩" format if present
            let normalized = if let Some((_, raw_id)) = id_str.split_once(':') {
                raw_id.trim_matches('`').trim_matches('⟨').trim_matches('⟩').to_string()
            } else {
                id_str.trim_matches('`').trim_matches('⟨').trim_matches('⟩').to_string()
            };
            map.insert("id".to_string(), JsonValue::String(normalized));
        }
        return;
    }
    
    // Fallback to existing logic for complex ID objects
    let Some(id_value) = map.get("id") else {
        return;
    };

    if let Some(id_str) = id_value.as_str() {
        if let Some((_, raw_id)) = id_str.split_once(':') {
            // Trim both backticks AND unicode angle brackets (⟨⟩) that SurrealDB adds
            let normalized = raw_id
                .trim_matches('`')
                .trim_matches('⟨')
                .trim_matches('⟩');
            map.insert("id".to_string(), JsonValue::String(normalized.to_string()));
        }
        return;
    }

    if let Some(id_obj) = id_value.as_object() {
        if let Some(raw_id) = id_obj.get("id").and_then(|inner| inner.as_str()) {
            // Trim both backticks AND unicode angle brackets (⟨⟩) that SurrealDB adds
            let normalized = raw_id
                .trim_matches('`')
                .trim_matches('⟨')
                .trim_matches('⟩');
            map.insert("id".to_string(), JsonValue::String(normalized.to_string()));
        }
    }
}

pub fn normalize_object_ids(values: &mut [JsonValue]) {
    for value in values {
        normalize_object_id(value);
    }
}
