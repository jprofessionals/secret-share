pub mod config;
pub mod crypto;
pub mod db;
pub mod error;
pub mod models;
mod routes;
pub mod services;

use std::sync::Arc;

use crate::config::Config;
use crate::db::Database;

pub use routes::create_router;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub config: Arc<Config>,
}

pub async fn run(config: Config, db: Database) -> anyhow::Result<()> {
    let state = AppState {
        db: Arc::new(db),
        config: Arc::new(config.clone()),
    };

    let app = create_router(state);

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
