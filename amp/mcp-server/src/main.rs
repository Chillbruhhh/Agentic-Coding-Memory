use anyhow::Result;
use rmcp::model::{
    CallToolRequestParam, CallToolResult, Implementation, ProtocolVersion, ServerCapabilities,
    ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer, ServiceExt};
use rmcp::ErrorData as McpError;
use rmcp::ServerHandler;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod amp_client;
mod config;
mod tools;

use amp_client::AmpClient;
use config::Config;

/// Connection state tracked per MCP session
#[derive(Debug, Clone, Default)]
struct ConnectionState {
    /// Connection ID returned from register
    connection_id: Option<String>,
    /// Current run ID (updated when amp_run_start is called)
    run_id: Option<String>,
    /// Whether we've registered with the server
    registered: bool,
}

#[derive(Clone)]
struct AmpMcpHandler {
    client: Arc<AmpClient>,
    config: Arc<Config>,
    /// Shared connection state for this handler
    connection_state: Arc<RwLock<ConnectionState>>,
}

impl ServerHandler for AmpMcpHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_06_18,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_tool_list_changed()
                .build(),
            server_info: Implementation {
                name: self.config.server_name.clone(),
                version: self.config.server_version.clone(),
                icons: None,
                title: None,
                website_url: None,
            },
            instructions: Some(
                "AMP MCP Server - Exposes Agentic Memory Protocol tools for AI agents".to_string(),
            ),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, McpError> {
        use rmcp::model::Tool;
        use std::sync::Arc;

        // === Register connection on handshake (list_tools is called right after init) ===
        {
            let mut state = self.connection_state.write().await;
            if !state.registered {
                let agent_id = format!(
                    "mcp-{}",
                    uuid::Uuid::new_v4()
                        .to_string()
                        .split('-')
                        .next()
                        .unwrap_or("unknown")
                );
                let agent_suffix = agent_id
                    .split('-')
                    .nth(1)
                    .unwrap_or("unknown")
                    .to_string();

                let meta_label = context
                    .meta
                    .0
                    .get("agentName")
                    .and_then(|value| value.as_str())
                    .map(|value| value.to_string())
                    .or_else(|| {
                        context
                            .meta
                            .0
                            .get("agent_name")
                            .and_then(|value| value.as_str())
                            .map(|value| value.to_string())
                    });

                let client_label = meta_label
                    .or_else(|| {
                        context
                            .peer
                            .peer_info()
                            .and_then(|info| info.client_info.title.clone())
                    })
                    .or_else(|| {
                        context
                            .peer
                            .peer_info()
                            .map(|info| info.client_info.name.clone())
                    })
                    .filter(|label| !label.trim().is_empty());

                // Prefer explicit AMP_AGENT_NAME for UI labeling, fall back to client metadata.
                let base_label = std::env::var("AMP_AGENT_NAME")
                    .ok()
                    .filter(|label| !label.trim().is_empty())
                    .or(client_label)
                    .unwrap_or_else(|| self.config.server_name.clone());
                // Ensure uniqueness per connection by appending a short suffix.
                let agent_label = format!("{}-{}", base_label, agent_suffix);

                // Auto-create a run so the session appears in the UI immediately
                let run_payload = serde_json::json!({
                    "type": "run",
                    "input_summary": format!("{} session", agent_label),
                    "status": "running",
                    "provenance": {
                        "agent": agent_label.clone(),
                        "summary": "MCP session auto-created on connect"
                    }
                });

                if let Ok(run_response) = self.client.create_object(run_payload).await {
                    if let Some(run_id) = run_response.get("id").and_then(|v| v.as_str()) {
                        let clean_run_id = run_id.trim_start_matches("objects:").to_string();
                        state.run_id = Some(clean_run_id.clone());
                        tracing::info!("Auto-created run for MCP session: {}", clean_run_id);

                        let register_payload = serde_json::json!({
                            "agent_id": agent_id,
                            "agent_name": agent_label,
                            "run_id": clean_run_id,
                            "ttl_seconds": 600
                        });

                        if let Ok(response) = self.client.register_connection(register_payload).await {
                            if let Some(conn_id) = response.get("connection_id").and_then(|v| v.as_str()) {
                                state.connection_id = Some(conn_id.to_string());
                                tracing::info!("Registered connection on handshake: {} -> run: {}", conn_id, clean_run_id);
                            }
                        }
                    }
                }
                state.registered = true;
            }
        }

        // Helper to convert schema to Arc<Map> (schemars 1.0 API)
        let to_schema =
            |schema: schemars::Schema| -> Arc<serde_json::Map<String, serde_json::Value>> {
                let value = serde_json::to_value(schema).unwrap();
                if let Some(obj) = value.as_object() {
                    Arc::new(obj.clone())
                } else {
                    // Fallback to empty object schema
                    let mut map = serde_json::Map::new();
                    map.insert(
                        "type".to_string(),
                        serde_json::Value::String("object".to_string()),
                    );
                    Arc::new(map)
                }
            };

        Ok(rmcp::model::ListToolsResult {
            tools: vec![
                Tool {
                    name: "amp_status".into(),
                    description: Some("Get AMP server health and analytics".into()),
                    input_schema: to_schema(schemars::schema_for!(
                        tools::discovery::AmpStatusInput
                    )),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_list".into(),
                    description: Some("List AMP objects by type".into()),
                    input_schema: to_schema(schemars::schema_for!(tools::discovery::AmpListInput)),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_query".into(),
                    description: Some("Search AMP memory with hybrid retrieval".into()),
                    input_schema: to_schema(schemars::schema_for!(tools::query::AmpQueryInput)),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_trace".into(),
                    description: Some("Trace object provenance and relationships".into()),
                    input_schema: to_schema(schemars::schema_for!(tools::query::AmpTraceInput)),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_write_artifact".into(),
                    description: Some(
                        "Write artifact (decision, changeset, note, filelog) to all memory layers with graph relationships".into(),
                    ),
                    input_schema: to_schema(schemars::schema_for!(
                        tools::memory::AmpWriteArtifactInput
                    )),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_focus".into(),
                    description: Some("Manage agent focus/session state (list, get, set, complete, end)".into()),
                    input_schema: to_schema(schemars::schema_for!(tools::focus::AmpFocusInput)),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_filelog_get".into(),
                    description: Some("Get file log with symbols and dependencies".into()),
                    input_schema: to_schema(schemars::schema_for!(
                        tools::files::AmpFilelogGetInput
                    )),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_file_sync".into(),
                    description: Some(
                        "Sync file state across all memory layers (temporal, vector, graph) after create/edit/delete".into(),
                    ),
                    input_schema: to_schema(schemars::schema_for!(
                        tools::files::AmpFileSyncInput
                    )),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_file_content_get".into(),
                    description: Some("Get stored file content from indexed chunks".into()),
                    input_schema: to_schema(schemars::schema_for!(
                        tools::files::AmpFileContentGetInput
                    )),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_file_path_resolve".into(),
                    description: Some("Resolve canonical stored path for a file input".into()),
                    input_schema: to_schema(schemars::schema_for!(
                        tools::files::AmpFilePathResolveInput
                    )),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_cache_write".into(),
                    description: Some(
                        "Write to episodic cache (fact/decision/snippet/warning). Auto-closes block at ~1800 tokens.".into(),
                    ),
                    input_schema: to_schema(schemars::schema_for!(
                        tools::cache::AmpCacheWriteInput
                    )),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_cache_compact".into(),
                    description: Some(
                        "Close current cache block and open new one (call on conversation compact)".into(),
                    ),
                    input_schema: to_schema(schemars::schema_for!(
                        tools::cache::AmpCacheCompactInput
                    )),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_cache_read".into(),
                    description: Some(
                        "Read from episodic cache - search blocks, get specific block, or get current block. Modes: (1) query param → search closed blocks by summary, (2) block_id param → get specific block with full content, (3) neither → get current open block. Use include_content=true with query to fetch full content of matching blocks in one call.".into(),
                    ),
                    input_schema: to_schema(schemars::schema_for!(
                        tools::cache::AmpCacheReadInput
                    )),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
            ],
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        params: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let client = &self.client;

        // Helper to convert errors - use String directly
        let to_internal_error = |e: anyhow::Error| McpError::internal_error(e.to_string(), None);
        let to_invalid_params =
            |e: serde_json::Error| McpError::invalid_params(e.to_string(), None);

        // === Connection tracking: send heartbeat on each tool call ===
        {
            let state = self.connection_state.read().await;
            if let Some(conn_id) = &state.connection_id {
                let heartbeat_payload = serde_json::json!({
                    "connection_id": conn_id,
                    "run_id": state.run_id,
                    "ttl_seconds": 600
                });

                let client_clone = client.clone();
                tokio::spawn(async move {
                    if let Err(e) = client_clone.connection_heartbeat(heartbeat_payload).await {
                        tracing::debug!("Heartbeat failed (non-fatal): {}", e);
                    }
                });
            }
        }

        let contents = match params.name.as_ref() {
            "amp_status" => tools::discovery::handle_amp_status(client)
                .await
                .map_err(to_internal_error)?,
            "amp_list" => {
                let input: tools::discovery::AmpListInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::discovery::handle_amp_list(client, input)
                    .await
                    .map_err(to_internal_error)?
            }
            "amp_query" => {
                let input: tools::query::AmpQueryInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::query::handle_amp_query(client, input)
                    .await
                    .map_err(to_internal_error)?
            }
            "amp_trace" => {
                let input: tools::query::AmpTraceInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::query::handle_amp_trace(client, input)
                    .await
                    .map_err(to_internal_error)?
            }
            "amp_write_artifact" => {
                let input: tools::memory::AmpWriteArtifactInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::memory::handle_write_artifact(client, input)
                    .await
                    .map_err(to_internal_error)?
            }
            "amp_focus" => {
                let input: tools::focus::AmpFocusInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                let run_id = {
                    let state = self.connection_state.read().await;
                    state.run_id.clone()
                };
                tools::focus::handle_focus(client, run_id.as_deref(), input)
                    .await
                    .map_err(to_internal_error)?
            }
            "amp_filelog_get" => {
                let input: tools::files::AmpFilelogGetInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::files::handle_filelog_get(client, input)
                    .await
                    .map_err(to_internal_error)?
            }
            "amp_file_sync" => {
                let input: tools::files::AmpFileSyncInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::files::handle_file_sync(client, input)
                    .await
                    .map_err(to_internal_error)?
            }
            "amp_file_content_get" => {
                let input: tools::files::AmpFileContentGetInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::files::handle_file_content_get(client, input)
                    .await
                    .map_err(to_internal_error)?
            }
            "amp_file_path_resolve" => {
                let input: tools::files::AmpFilePathResolveInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::files::handle_file_path_resolve(client, input)
                    .await
                    .map_err(to_internal_error)?
            }
            "amp_cache_write" => {
                let input: tools::cache::AmpCacheWriteInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                let run_id = {
                    let state = self.connection_state.read().await;
                    state.run_id.clone()
                };
                tools::cache::handle_cache_write(client, run_id.as_deref(), input)
                    .await
                    .map_err(to_internal_error)?
            }
            "amp_cache_compact" => {
                let input: tools::cache::AmpCacheCompactInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                let run_id = {
                    let state = self.connection_state.read().await;
                    state.run_id.clone()
                };
                tools::cache::handle_cache_compact(client, run_id.as_deref(), input)
                    .await
                    .map_err(to_internal_error)?
            }
            "amp_cache_read" => {
                let input: tools::cache::AmpCacheReadInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::cache::handle_cache_read(client, input)
                    .await
                    .map_err(to_internal_error)?
            }
            _ => {
                return Err(McpError::invalid_request(
                    format!("Unknown tool: {}", params.name),
                    None,
                ))
            }
        };

        Ok(CallToolResult::success(contents))
    }
}

async fn run_stdio_transport(handler: AmpMcpHandler) -> Result<()> {
    use tokio::io::{stdin, stdout};

    tracing::info!("Starting MCP server with stdio transport");

    // Create transport from stdin/stdout
    let transport = (stdin(), stdout());

    // Start server
    let server = handler.serve(transport).await?;
    tracing::info!("MCP server started (stdio)");

    // Wait for shutdown
    server.waiting().await?;
    tracing::info!("MCP server shutdown");

    Ok(())
}

async fn run_http_transport(handler: AmpMcpHandler, port: u16) -> Result<()> {
    use rmcp::transport::streamable_http_server::{
        session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
    };
    use std::sync::Arc;

    tracing::info!(
        "Starting MCP server with Streamable HTTP transport on port {}",
        port
    );

    // Create session manager for handling multiple client sessions
    let session_manager = Arc::new(LocalSessionManager::default());

    // Create config
    let config = StreamableHttpServerConfig::default();

    // Create the streamable HTTP service with a factory function.
    // Each connection gets its own connection_state so sessions don't collapse.
    let handler_base = handler.clone();
    let service = StreamableHttpService::new(
        move || {
            Ok(AmpMcpHandler {
                client: handler_base.client.clone(),
                config: handler_base.config.clone(),
                connection_state: Arc::new(RwLock::new(ConnectionState::default())),
            })
        },
        session_manager,
        config,
    );

    // Create the axum router with the service
    let app = axum::Router::new().route("/mcp", axum::routing::any_service(service));

    // Bind and serve
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("MCP HTTP server listening on http://{}", addr);
    tracing::info!("Connect using: http://localhost:{}/mcp", port);

    axum::serve(listener, app).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting AMP MCP Server");

    // Load configuration
    let config = Arc::new(Config::from_env()?);
    tracing::info!("Configuration loaded: {:?}", config);

    // Initialize AMP client
    let client = Arc::new(AmpClient::new(
        config.amp_server_url.clone(),
        config.amp_server_timeout,
    )?);
    tracing::info!("AMP client initialized");

    // Create handler with connection state
    let handler = AmpMcpHandler {
        client: client.clone(),
        config: config.clone(),
        connection_state: Arc::new(RwLock::new(ConnectionState::default())),
    };

    tracing::info!("MCP handler created");

    // Check transport mode from environment
    let transport_mode = std::env::var("MCP_TRANSPORT").unwrap_or_else(|_| "stdio".to_string());
    let mcp_port: u16 = std::env::var("MCP_PORT")
        .unwrap_or_else(|_| "8106".to_string())
        .parse()
        .unwrap_or(8106);

    match transport_mode.as_str() {
        "sse" | "http" => {
            run_http_transport(handler, mcp_port).await?;
        }
        _ => {
            run_stdio_transport(handler).await?;
        }
    }

    Ok(())
}
