use secret_share_backend::{
    config::{Config, DatabaseConfig},
    db::{PostgresRepository, SecretRepository},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .init();

    let config = Config::from_env()?;

    match &config.database {
        DatabaseConfig::Postgres { url } => {
            let db = PostgresRepository::new(url).await?;
            let deleted = db.cleanup_expired().await?;
            println!("Cleanup complete: deleted {} expired secrets", deleted);
        }
        DatabaseConfig::DynamoDB { table, .. } => {
            println!(
                "DynamoDB table '{}' uses TTL for automatic cleanup - no action needed",
                table
            );
        }
    }

    Ok(())
}
