use surrealdb::{engine::any::Any, Surreal};
use anyhow::Result;

pub struct Database {
    pub client: Surreal<Any>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        tracing::info!("Connecting to database: {}", database_url);
        
        // Connect with timeout
        let connect_future = async {
            let client = surrealdb::engine::any::connect(database_url).await?;
            
            // Sign in if credentials are provided AND we're not using file/memory database
            if let (Ok(user), Ok(pass)) = (std::env::var("DB_USER"), std::env::var("DB_PASS")) {
                if !database_url.starts_with("file://") && !database_url.starts_with("memory") {
                    tracing::info!("Authenticating with database credentials");
                    client.signin(surrealdb::opt::auth::Root {
                        username: &user,
                        password: &pass,
                    }).await?;
                } else {
                    tracing::info!("Skipping authentication for file/memory database");
                }
            }
            
            tracing::info!("Selecting namespace and database...");
            client.use_ns("amp").use_db("amp").await?;
            tracing::info!("Using namespace: amp, database: amp");
            
            Ok::<_, anyhow::Error>(client)
        };
        
        let client = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            connect_future
        ).await
            .map_err(|_| anyhow::anyhow!("Database connection timeout after 10 seconds"))??;
        
        tracing::info!("Database connection established");

        Ok(Self { client })
    }

    pub async fn initialize_schema(&self) -> Result<()> {
        tracing::info!("Initializing database schema...");
        let schema = include_str!("../../spec/schema.surql");
        
        // Execute schema in chunks (SurrealDB processes statements individually)
        for (i, statement) in schema.split(';').enumerate() {
            let statement = statement.trim();
            if !statement.is_empty() && !statement.starts_with("--") {
                tracing::debug!("Executing schema statement {}", i);
                if let Err(e) = self.client.query(statement).await {
                    let err_msg = e.to_string();
                    // Only ignore "already exists" errors
                    if err_msg.contains("already exists") || err_msg.contains("already been defined") {
                        tracing::debug!("Schema element already exists, skipping");
                    } else {
                        tracing::warn!("Schema error (continuing): {}", e);
                        // Don't fail on schema errors, just log them
                    }
                }
            }
        }

        tracing::info!("Database schema initialized");
        Ok(())
    }
}
