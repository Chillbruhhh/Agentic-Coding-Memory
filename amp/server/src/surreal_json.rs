use serde_json::Value as JsonValue;
use surrealdb::Response;

pub fn take_json_values(response: &mut Response, index: usize) -> Vec<JsonValue> {
    match response.take::<Vec<JsonValue>>(index) {
        Ok(values) => values,
        Err(err) => {
            tracing::warn!("Failed to decode response values: {}", err);
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
    let Some(id_value) = map.get("id") else {
        return;
    };

    if let Some(id_str) = id_value.as_str() {
        if let Some((_, raw_id)) = id_str.split_once(':') {
            let normalized = raw_id.trim_matches('`');
            map.insert("id".to_string(), JsonValue::String(normalized.to_string()));
        }
        return;
    }

    if let Some(id_obj) = id_value.as_object() {
        if let Some(raw_id) = id_obj.get("id").and_then(|inner| inner.as_str()) {
            let normalized = raw_id.trim_matches('`');
            map.insert("id".to_string(), JsonValue::String(normalized.to_string()));
        }
    }
}

pub fn normalize_object_ids(values: &mut [JsonValue]) {
    for value in values {
        normalize_object_id(value);
    }
}
