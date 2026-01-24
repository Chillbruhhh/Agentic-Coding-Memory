use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::settings::SettingsConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiFileLogInput {
    pub path: String,
    pub language: String,
    pub content_hash: String,
    pub content: String,
    pub symbols: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiFileLogOutput {
    pub summary_markdown: String,
    pub purpose: Option<String>,
    pub key_symbols: Vec<String>,
    pub dependencies: Vec<String>,
    pub notes: Option<String>,
}

pub struct IndexLlmService {
    client: Client,
}

impl IndexLlmService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn generate_file_log(
        &self,
        settings: &SettingsConfig,
        input: AiFileLogInput,
    ) -> Result<AiFileLogOutput> {
        match settings.index_provider.as_str() {
            "openai" => {
                self.generate_openai(
                    settings,
                    input,
                    "https://api.openai.com/v1/chat/completions",
                )
                .await
            }
            "openrouter" => {
                self.generate_openai(
                    settings,
                    input,
                    "https://openrouter.ai/api/v1/chat/completions",
                )
                .await
            }
            "ollama" => self.generate_ollama(settings, input).await,
            _ => anyhow::bail!("Index model provider is disabled"),
        }
    }

    async fn generate_openai(
        &self,
        settings: &SettingsConfig,
        input: AiFileLogInput,
        base_url: &str,
    ) -> Result<AiFileLogOutput> {
        let model = match settings.index_provider.as_str() {
            "openrouter" => settings.index_openrouter_model.clone(),
            _ => settings.index_openai_model.clone(),
        };
        let api_key = match settings.index_provider.as_str() {
            "openrouter" => settings.openrouter_api_key.clone(),
            _ => settings.openai_api_key.clone(),
        };

        if api_key.trim().is_empty() {
            anyhow::bail!("API key is missing for index model provider");
        }

        let prompt = build_filelog_prompt(&input);
        let body = serde_json::json!({
            "model": model,
            "temperature": 0.2,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a precise codebase analyst. Return ONLY valid JSON."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        let mut request = self
            .client
            .post(base_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body);

        if base_url.contains("openrouter.ai") {
            request = request
                .header("HTTP-Referer", "http://localhost")
                .header("X-Title", "AMP");
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Index model error: {}", error_text);
        }

        let payload: OpenAIChatResponse = response.json().await?;
        let content = payload
            .choices
            .get(0)
            .and_then(|c| c.message.content.as_ref())
            .context("Missing model response content")?;

        parse_filelog_json(content)
    }

    async fn generate_ollama(
        &self,
        settings: &SettingsConfig,
        input: AiFileLogInput,
    ) -> Result<AiFileLogOutput> {
        let prompt = build_filelog_prompt(&input);
        let body = serde_json::json!({
            "model": settings.index_ollama_model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a precise codebase analyst. Return ONLY valid JSON."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "stream": false
        });

        let url = format!("{}/api/chat", settings.ollama_url.trim_end_matches('/'));
        let response = self.client.post(url).json(&body).send().await?;
        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Index model error: {}", error_text);
        }

        let payload: OllamaChatResponse = response.json().await?;
        parse_filelog_json(&payload.message.content)
    }
}

fn build_filelog_prompt(input: &AiFileLogInput) -> String {
    let symbols = if input.symbols.is_empty() {
        "None".to_string()
    } else {
        input.symbols.join("\n- ")
    };
    let deps = if input.dependencies.is_empty() {
        "None".to_string()
    } else {
        input.dependencies.join("\n- ")
    };

    format!(
        "Generate a FILE_LOG in JSON with keys: summary_markdown, purpose, key_symbols, dependencies, notes.\n\
summary_markdown must follow this template:\n\
# FILE_LOG v1\n\
path: {path}\n\
language: {language}\n\
last_indexed: <ISO8601>\n\
content_hash: {content_hash}\n\
\n\
## Symbols (current snapshot)\n\
- <symbol list>\n\
\n\
## Dependencies (best-effort)\n\
imports:\n\
- <import paths>\n\
exports:\n\
- <exported symbols>\n\
\n\
## Recent Changes (rolling, last N)\n\
- <if available, otherwise 'None'>\n\
\n\
## Notes / Decisions linked\n\
- <if available, otherwise 'None'>\n\
\n\
Notes must include a concise 1-2 sentence overview of what this file/directory does (even if there are no decisions).\n\
Use the file content and symbols provided. Return ONLY valid JSON.\n\
\n\
File path: {path}\n\
Language: {language}\n\
Symbols:\n\
- {symbols}\n\
\n\
Dependencies:\n\
- {deps}\n\
\n\
File content:\n\
{content}\n",
        path = input.path,
        language = input.language,
        symbols = symbols,
        deps = deps,
        content = input.content,
        content_hash = input.content_hash
    )
}

fn parse_filelog_json(raw: &str) -> Result<AiFileLogOutput> {
    let trimmed = raw.trim();
    if let Ok(parsed) = serde_json::from_str::<AiFileLogOutput>(trimmed) {
        return Ok(parsed);
    }

    let candidate =
        extract_json_block(trimmed).context("Failed to locate JSON in model response")?;
    if let Ok(parsed) = serde_json::from_str::<AiFileLogOutput>(&candidate) {
        return Ok(parsed);
    }

    let value: Value =
        serde_json::from_str(&candidate).context("Failed to parse index model JSON")?;
    coerce_filelog_value(value).context("Failed to coerce index model JSON")
}

fn extract_json_block(raw: &str) -> Option<String> {
    if let Some(start) = raw.find("```json") {
        let after = &raw[start + "```json".len()..];
        if let Some(end) = after.find("```") {
            return Some(after[..end].trim().to_string());
        }
    }

    if let Some(start) = raw.find("```") {
        let after = &raw[start + 3..];
        if let Some(end) = after.find("```") {
            return Some(after[..end].trim().to_string());
        }
    }

    let first_brace = raw.find('{')?;
    let last_brace = raw.rfind('}')?;
    if last_brace <= first_brace {
        return None;
    }
    Some(raw[first_brace..=last_brace].trim().to_string())
}

fn coerce_filelog_value(value: Value) -> Result<AiFileLogOutput> {
    let obj = value
        .as_object()
        .context("Expected JSON object for file log")?;

    let summary_markdown = coerce_string(
        obj.get("summary_markdown")
            .or_else(|| obj.get("summaryMarkdown"))
            .or_else(|| obj.get("summary"))
            .or_else(|| obj.get("file_log")),
    )
    .unwrap_or_default();

    if summary_markdown.is_empty() {
        anyhow::bail!("Missing summary_markdown in index model JSON");
    }

    let purpose = coerce_string(obj.get("purpose"));
    let notes = coerce_string(obj.get("notes"));

    let key_symbols = coerce_string_list(
        obj.get("key_symbols")
            .or_else(|| obj.get("keySymbols"))
            .or_else(|| obj.get("symbols")),
    );

    let dependencies = coerce_dependencies(obj.get("dependencies"));

    Ok(AiFileLogOutput {
        summary_markdown,
        purpose,
        key_symbols,
        dependencies,
        notes,
    })
}

fn coerce_string(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(val)) => Some(val.trim().to_string()),
        Some(Value::Number(val)) => Some(val.to_string()),
        Some(Value::Bool(val)) => Some(val.to_string()),
        Some(Value::Array(items)) => Some(
            items
                .iter()
                .filter_map(|item| item.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        ),
        _ => None,
    }
}

fn coerce_string_list(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| item.as_str())
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect(),
        Some(Value::String(val)) => val
            .lines()
            .map(|line| line.trim().trim_start_matches('-').trim())
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

fn coerce_dependencies(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| item.as_str())
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect(),
        Some(Value::Object(map)) => {
            let mut deps = Vec::new();
            if let Some(Value::Array(items)) = map.get("imports") {
                for item in items.iter().filter_map(|item| item.as_str()) {
                    let trimmed = item.trim();
                    if !trimmed.is_empty() {
                        deps.push(format!("imports: {}", trimmed));
                    }
                }
            }
            if let Some(Value::Array(items)) = map.get("exports") {
                for item in items.iter().filter_map(|item| item.as_str()) {
                    let trimmed = item.trim();
                    if !trimmed.is_empty() {
                        deps.push(format!("exports: {}", trimmed));
                    }
                }
            }
            deps
        }
        Some(Value::String(val)) => val
            .lines()
            .map(|line| line.trim().trim_start_matches('-').trim())
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

#[derive(Debug, Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaMessage,
}

#[derive(Debug, Deserialize)]
struct OllamaMessage {
    content: String,
}
