use anyhow::Result;
use rmcp::ServerHandler;
use rmcp::service::{RequestContext, ServiceExt, RoleServer};
use rmcp::model::{ServerInfo, ServerCapabilities, Implementation, ProtocolVersion, CallToolRequestParam, CallToolResult};
use rmcp::ErrorData as McpError;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod amp_client;
mod config;
mod tools;

use amp_client::AmpClient;
use config::Config;

#[derive(Clone)]
struct AmpMcpHandler {
    client: Arc<AmpClient>,
    config: Arc<Config>,
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
            instructions: Some("AMP MCP Server - Exposes Agentic Memory Protocol tools for AI agents".to_string()),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, McpError> {
        use rmcp::model::Tool;
        use std::sync::Arc;

        // Helper to convert schema to Arc<Map> (schemars 1.0 API)
        let to_schema = |schema: schemars::Schema| -> Arc<serde_json::Map<String, serde_json::Value>> {
            let value = serde_json::to_value(schema).unwrap();
            if let Some(obj) = value.as_object() {
                Arc::new(obj.clone())
            } else {
                // Fallback to empty object schema
                let mut map = serde_json::Map::new();
                map.insert("type".to_string(), serde_json::Value::String("object".to_string()));
                Arc::new(map)
            }
        };

        Ok(rmcp::model::ListToolsResult {
            tools: vec![
                Tool {
                    name: "amp_status".into(),
                    description: Some("Get AMP server health and analytics".into()),
                    input_schema: to_schema(schemars::schema_for!(tools::discovery::AmpStatusInput)),
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
                    name: "amp_context".into(),
                    description: Some("Get high-signal memory bundle for a task".into()),
                    input_schema: to_schema(schemars::schema_for!(tools::context::AmpContextInput)),
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
                    name: "amp_write_decision".into(),
                    description: Some("Create an architectural decision record".into()),
                    input_schema: to_schema(schemars::schema_for!(tools::memory::AmpWriteDecisionInput)),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_write_changeset".into(),
                    description: Some("Document a completed work unit".into()),
                    input_schema: to_schema(schemars::schema_for!(tools::memory::AmpWriteChangesetInput)),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_run_start".into(),
                    description: Some("Start tracking an agent execution".into()),
                    input_schema: to_schema(schemars::schema_for!(tools::memory::AmpRunStartInput)),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_run_end".into(),
                    description: Some("Complete an agent execution".into()),
                    input_schema: to_schema(schemars::schema_for!(tools::memory::AmpRunEndInput)),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_filelog_get".into(),
                    description: Some("Get file log with symbols and dependencies".into()),
                    input_schema: to_schema(schemars::schema_for!(tools::files::AmpFilelogGetInput)),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_filelog_update".into(),
                    description: Some("Update file log after changes".into()),
                    input_schema: to_schema(schemars::schema_for!(tools::files::AmpFilelogUpdateInput)),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_file_content_get".into(),
                    description: Some("Get stored file content from indexed chunks".into()),
                    input_schema: to_schema(schemars::schema_for!(tools::files::AmpFileContentGetInput)),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_file_path_resolve".into(),
                    description: Some("Resolve canonical stored path for a file input".into()),
                    input_schema: to_schema(schemars::schema_for!(tools::files::AmpFilePathResolveInput)),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_lease_acquire".into(),
                    description: Some("Acquire a resource lease for coordination".into()),
                    input_schema: to_schema(schemars::schema_for!(tools::coordination::AmpLeaseAcquireInput)),
                    annotations: None,
                    icons: None,
                    meta: None,
                    title: None,
                    output_schema: None,
                },
                Tool {
                    name: "amp_lease_release".into(),
                    description: Some("Release a resource lease".into()),
                    input_schema: to_schema(schemars::schema_for!(tools::coordination::AmpLeaseReleaseInput)),
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
        let to_invalid_params = |e: serde_json::Error| McpError::invalid_params(e.to_string(), None);

        let contents = match params.name.as_ref() {
            "amp_status" => {
                tools::discovery::handle_amp_status(client).await.map_err(to_internal_error)?
            }
            "amp_list" => {
                let input: tools::discovery::AmpListInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::discovery::handle_amp_list(client, input).await.map_err(to_internal_error)?
            }
            "amp_context" => {
                let input: tools::context::AmpContextInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::context::handle_amp_context(client, input).await.map_err(to_internal_error)?
            }
            "amp_query" => {
                let input: tools::query::AmpQueryInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::query::handle_amp_query(client, input).await.map_err(to_internal_error)?
            }
            "amp_trace" => {
                let input: tools::query::AmpTraceInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::query::handle_amp_trace(client, input).await.map_err(to_internal_error)?
            }
            "amp_write_decision" => {
                let input: tools::memory::AmpWriteDecisionInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::memory::handle_write_decision(client, input).await.map_err(to_internal_error)?
            }
            "amp_write_changeset" => {
                let input: tools::memory::AmpWriteChangesetInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::memory::handle_write_changeset(client, input).await.map_err(to_internal_error)?
            }
            "amp_run_start" => {
                let input: tools::memory::AmpRunStartInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::memory::handle_run_start(client, input).await.map_err(to_internal_error)?
            }
            "amp_run_end" => {
                let input: tools::memory::AmpRunEndInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::memory::handle_run_end(client, input).await.map_err(to_internal_error)?
            }
            "amp_filelog_get" => {
                let input: tools::files::AmpFilelogGetInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::files::handle_filelog_get(client, input).await.map_err(to_internal_error)?
            }
            "amp_filelog_update" => {
                let input: tools::files::AmpFilelogUpdateInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::files::handle_filelog_update(client, input).await.map_err(to_internal_error)?
            }
            "amp_file_content_get" => {
                let input: tools::files::AmpFileContentGetInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::files::handle_file_content_get(client, input).await.map_err(to_internal_error)?
            }
            "amp_file_path_resolve" => {
                let input: tools::files::AmpFilePathResolveInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::files::handle_file_path_resolve(client, input).await.map_err(to_internal_error)?
            }
            "amp_lease_acquire" => {
                let input: tools::coordination::AmpLeaseAcquireInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::coordination::handle_lease_acquire(client, input).await.map_err(to_internal_error)?
            }
            "amp_lease_release" => {
                let input: tools::coordination::AmpLeaseReleaseInput =
                    serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
                        .map_err(to_invalid_params)?;
                tools::coordination::handle_lease_release(client, input).await.map_err(to_internal_error)?
            }
            _ => return Err(McpError::invalid_request(format!("Unknown tool: {}", params.name), None))
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
        StreamableHttpService,
        StreamableHttpServerConfig,
        session::local::LocalSessionManager,
    };
    use std::sync::Arc;

    tracing::info!("Starting MCP server with Streamable HTTP transport on port {}", port);

    // Create session manager for handling multiple client sessions
    let session_manager = Arc::new(LocalSessionManager::default());

    // Create config
    let config = StreamableHttpServerConfig::default();

    // Clone handler for the factory
    let handler_clone = handler.clone();

    // Create the streamable HTTP service with a factory function
    let service = StreamableHttpService::new(
        move || Ok(handler_clone.clone()),
        session_manager,
        config,
    );

    // Create the axum router with the service
    let app = axum::Router::new()
        .route("/mcp", axum::routing::any_service(service));

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

    // Create handler
    let handler = AmpMcpHandler {
        client: client.clone(),
        config: config.clone(),
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
