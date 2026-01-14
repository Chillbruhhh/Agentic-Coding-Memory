use surrealdb::{engine::local::Db, Surreal};
use anyhow::Result;

pub struct Database {
    pub client: Surreal<Db>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let client = if database_url == "memory" {
            Surreal::new::<surrealdb::engine::local::Mem>(()).await?
        } else if database_url.starts_with("file://") {
            let path = database_url.strip_prefix("file://").unwrap();
            Surreal::new::<surrealdb::engine::local::File>(path).await?
        } else {
            anyhow::bail!("Invalid database URL. Use 'memory' or 'file://path/to/db'");
        };

        client.use_ns("amp").use_db("main").await?;

        Ok(Self { client })
    }

    pub async fn initialize_schema(&self) -> Result<()> {
        let schema = include_str!("../../spec/schema.surql");
        
        // Execute schema in chunks (SurrealDB processes statements individually)
        for statement in schema.split(';') {
            let statement = statement.trim();
            if !statement.is_empty() && !statement.starts_with("--") {
                if let Err(e) = self.client.query(statement).await {
                    let err_msg = e.to_string();
                    // Only ignore "already exists" errors
                    if err_msg.contains("already exists") || err_msg.contains("already been defined") {
                        tracing::debug!("Schema element already exists, skipping");
                    } else {
                        tracing::error!("Critical schema error: {}", e);
                        return Err(e.into());
                    }
                }
            }
        }

        tracing::info!("Database schema initialized");
        Ok(())
    }
}
