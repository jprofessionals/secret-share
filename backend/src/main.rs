use secret_share_backend::{config::Config, db::Database, run};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,secret_share_backend=debug".to_string()),
        )
        .init();

    tracing::info!("Starting Secret Share Backend");

    let config = Config::from_env()?;
    let db = Database::new(&config.database_url).await?;
    db.migrate().await?;

    run(config, db).await
}
