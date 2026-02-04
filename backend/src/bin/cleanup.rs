use secret_share_backend::{config::Config, db::{Database, SecretRepository}};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        )
        .init();

    let config = Config::from_env()?;
    let db = Database::new(&config.database_url).await?;

    let deleted = db.cleanup_expired().await?;
    println!("Cleanup complete: deleted {} expired secrets", deleted);

    Ok(())
}
