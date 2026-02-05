pub mod config;
pub mod crypto;
pub mod db;
pub mod error;
pub mod models;
mod routes;
pub mod services;

use std::sync::Arc;

use crate::config::{Config, DatabaseConfig};
use crate::db::{DynamoDbRepository, PostgresRepository, SecretRepository};

pub use routes::create_router;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<dyn SecretRepository>,
    pub config: Arc<Config>,
}

pub async fn run(config: Config) -> anyhow::Result<()> {
    let db: Arc<dyn SecretRepository> = match &config.database {
        DatabaseConfig::Postgres { url } => {
            tracing::info!("Using PostgreSQL database");
            let pg = PostgresRepository::new(url).await?;
            pg.migrate().await?;
            Arc::new(pg)
        }
        DatabaseConfig::DynamoDB { table, endpoint } => {
            tracing::info!("Using DynamoDB table: {}", table);
            let dynamo = DynamoDbRepository::new(table, endpoint.as_deref()).await?;
            Arc::new(dynamo)
        }
    };

    let state = AppState {
        db,
        config: Arc::new(config.clone()),
    };

    let app = create_router(state);

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
