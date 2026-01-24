use axum::{
    extract::State,
    http::StatusCode,
    middleware::{from_fn_with_state, Next},
    response::{Json, Response},
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use std::time::Instant;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

mod config;
mod database;
mod handlers;
mod models;
mod services;
mod surreal_json;

use config::Config;
use database::Database;
use services::analytics::AnalyticsService;
use services::embedding::EmbeddingService;
use services::graph::GraphTraversalService;
use services::hybrid::HybridRetrievalService;
use services::settings::SettingsService;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub config: Arc<Config>,
    pub embedding_service: Arc<dyn EmbeddingService>,
    pub graph_service: Arc<GraphTraversalService>,
    pub hybrid_service: Arc<HybridRetrievalService>,
    pub analytics_service: Arc<AnalyticsService>,
    pub settings_service: Arc<SettingsService>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if it exists
    let _ = dotenvy::dotenv();

    // Initialize tracing
    let log_dir = match std::env::current_dir() {
        Ok(dir) => {
            if dir.file_name().and_then(|name| name.to_str()) == Some("server") {
                dir.parent()
                    .map(|parent| parent.join("logs"))
                    .unwrap_or_else(|| dir.join("logs"))
            } else if dir.file_name().and_then(|name| name.to_str()) == Some("amp") {
                dir.join("logs")
            } else {
                dir.join("amp").join("logs")
            }
        }
        Err(_) => std::path::PathBuf::from("amp").join("logs"),
    };
    if let Err(err) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Failed to create log directory {:?}: {}", log_dir, err);
    }
    let error_log = tracing_appender::rolling::never(&log_dir, "amp-errors.log");
    let (error_log, _error_log_guard) = tracing_appender::non_blocking(error_log);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "amp_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(error_log)
                .with_ansi(false)
                .with_filter(LevelFilter::WARN),
        )
        .init();

    // Load configuration
    let config = Arc::new(Config::from_env()?);

    // Initialize database
    let db = Arc::new(Database::new(&config.database_url).await?);

    // Initialize database schema
    db.initialize_schema().await?;

    let settings_service = Arc::new(SettingsService::new(db.client.clone()));
    tracing::info!("Settings service initialized");

    let settings = settings_service.load_settings().await.unwrap_or_default();

    let (embedding_model, embedding_dimension) = match settings.embedding_provider.as_str() {
        "openrouter" => (
            settings.openrouter_model.clone(),
            settings.openrouter_dimension as usize,
        ),
        "ollama" => (
            settings.ollama_model.clone(),
            settings.ollama_dimension as usize,
        ),
        _ => (
            settings.openai_model.clone(),
            settings.openai_dimension as usize,
        ),
    };

    // Initialize embedding service
    let embedding_service = services::embedding::create_embedding_service(
        &settings.embedding_provider,
        Some(settings.openai_api_key.clone()),
        Some(settings.openrouter_api_key.clone()),
        settings.ollama_url.clone(),
        embedding_dimension,
        embedding_model.clone(),
    );

    tracing::info!(
        "Embedding service initialized: provider={}, model={}, dimension={}, enabled={}",
        settings.embedding_provider,
        embedding_model,
        embedding_service.dimension(),
        embedding_service.is_enabled()
    );

    let graph_service = Arc::new(GraphTraversalService::new(db.clone()));
    tracing::info!("Graph traversal service initialized");

    let embedding_service_arc: Arc<dyn EmbeddingService> = Arc::from(embedding_service);
    let hybrid_service = HybridRetrievalService::new(
        db.clone(),
        embedding_service_arc.clone(),
        graph_service.clone(),
    );
    tracing::info!("Hybrid retrieval service initialized");

    let analytics_service = Arc::new(AnalyticsService::new(db.clone()));
    tracing::info!("Analytics service initialized");

    let state = AppState {
        db,
        config: config.clone(),
        embedding_service: embedding_service_arc,
        graph_service,
        hybrid_service: Arc::new(hybrid_service),
        analytics_service,
        settings_service,
    };

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/v1", api_routes())
        .layer(from_fn_with_state(state.clone(), track_latency))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("{}:{}", config.bind_address, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("AMP server listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;
    Ok(())
}

fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/objects", post(handlers::objects::create_object))
        .route(
            "/objects/batch",
            post(handlers::objects::create_objects_batch),
        )
        .route("/objects/:id", get(handlers::objects::get_object))
        .route("/objects/:id", put(handlers::objects::update_object))
        .route("/objects/:id", delete(handlers::objects::delete_object))
        .route("/query", post(handlers::query::query))
        .route("/trace/:id", get(handlers::trace::get_trace))
        .route("/leases/acquire", post(handlers::leases::acquire_lease))
        .route("/leases/release", post(handlers::leases::release_lease))
        .route("/leases/renew", post(handlers::leases::renew_lease))
        .route(
            "/relationships",
            post(handlers::relationships::create_relationship),
        )
        .route(
            "/relationships",
            get(handlers::relationships::get_relationships),
        )
        .route(
            "/relationships/:type/:id",
            delete(handlers::relationships::delete_relationship),
        )
        // Codebase parsing endpoints
        .route("/codebase/parse", post(handlers::codebase::parse_codebase))
        .route("/codebase/parse-file", post(handlers::codebase::parse_file))
        .route(
            "/codebase/delete",
            post(handlers::codebase::delete_codebase),
        )
        .route(
            "/codebase/file-logs",
            get(handlers::codebase::get_file_logs),
        )
        .route(
            "/codebase/file-logs/:path",
            get(handlers::codebase::get_file_log),
        )
        .route(
            "/codebase/file-log-objects/:path",
            get(handlers::codebase::get_file_log_object),
        )
        .route(
            "/codebase/file-contents/:path",
            get(handlers::codebase::get_file_content),
        )
        .route(
            "/codebase/update-file-log",
            post(handlers::codebase::update_file_log),
        )
        .route(
            "/codebase/sync",
            post(handlers::codebase::sync_file),
        )
        .route(
            "/codebase/ai-file-log",
            post(handlers::codebase::generate_ai_file_log),
        )
        // Analytics endpoint
        .route("/analytics", get(handlers::analytics::get_analytics))
        // Settings endpoints
        .route("/settings", get(handlers::settings::get_settings))
        .route("/settings", put(handlers::settings::update_settings))
        // Artifact endpoints - unified write across all 3 memory layers
        .route("/artifacts", post(handlers::artifacts::write_artifact))
        .route("/artifacts", get(handlers::artifacts::list_artifacts))
        .route(
            "/artifacts/:id",
            delete(handlers::artifacts::delete_artifact),
        )
        // Cache endpoints - semantic cache / unity layer (legacy)
        .route("/cache/pack", post(handlers::cache::get_pack))
        .route("/cache/write", post(handlers::cache::write_items))
        .route("/cache/gc", post(handlers::cache::gc))
        // Cache block endpoints - episodic memory (rolling window)
        .route("/cache/block/write", post(handlers::cache::block_write))
        .route("/cache/block/compact", post(handlers::cache::block_compact))
        .route("/cache/block/search", post(handlers::cache::block_search))
        .route("/cache/block/:id", get(handlers::cache::block_get))
}

async fn track_latency(
    State(state): State<AppState>,
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    let start = Instant::now();
    let response = next.run(request).await;
    let latency_ms = start.elapsed().as_secs_f32() * 1000.0;
    state.analytics_service.record_request_latency(latency_ms);
    response
}

async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "service": "amp-server",
        "version": env!("CARGO_PKG_VERSION")
    })))
}
