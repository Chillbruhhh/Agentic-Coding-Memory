use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod database;
mod handlers;
mod models;
mod services;

use config::Config;
use database::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub config: Arc<Config>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "amp_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Arc::new(Config::from_env()?);
    
    // Initialize database
    let db = Arc::new(Database::new(&config.database_url).await?);
    
    // Initialize database schema
    db.initialize_schema().await?;

    let state = AppState { db, config: config.clone() };

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/v1", api_routes())
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
        .route("/objects/batch", post(handlers::objects::create_objects_batch))
        .route("/objects/:id", get(handlers::objects::get_object))
        .route("/query", post(handlers::query::query))
        .route("/trace/:id", get(handlers::trace::get_trace))
        .route("/leases/acquire", post(handlers::leases::acquire_lease))
        .route("/leases/release", post(handlers::leases::release_lease))
}

async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "service": "amp-server",
        "version": env!("CARGO_PKG_VERSION")
    })))
}
