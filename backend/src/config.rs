use std::env;

#[derive(Debug, Clone)]
pub enum DatabaseConfig {
    Postgres { url: String },
    DynamoDB { table: String, endpoint: Option<String> },
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub base_url: String,
    pub port: u16,
    pub max_secret_days: i32,
    pub max_secret_views: i32,
    pub max_failed_attempts: i32,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        // Load .env file if present (for local development)
        dotenvy::dotenv().ok();

        let database = Self::detect_database()?;

        Ok(Config {
            database,
            base_url: env::var("BASE_URL")
                .unwrap_or_else(|_| "http://localhost:5173".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            max_secret_days: env::var("MAX_SECRET_DAYS")
                .ok()
                .and_then(|d| d.parse().ok())
                .unwrap_or(30),
            max_secret_views: env::var("MAX_SECRET_VIEWS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            max_failed_attempts: env::var("MAX_FAILED_ATTEMPTS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        })
    }

    fn detect_database() -> anyhow::Result<DatabaseConfig> {
        // Check for PostgreSQL first
        if let Ok(url) = env::var("DATABASE_URL") {
            if url.starts_with("postgres://") || url.starts_with("postgresql://") {
                return Ok(DatabaseConfig::Postgres { url });
            }
            anyhow::bail!(
                "DATABASE_URL must start with postgres:// or postgresql://, got: {}",
                &url[..url.len().min(20)]
            );
        }

        // Check for DynamoDB
        if let Ok(table) = env::var("DYNAMODB_TABLE") {
            let endpoint = env::var("DYNAMODB_ENDPOINT").ok();
            return Ok(DatabaseConfig::DynamoDB { table, endpoint });
        }

        // Default to local PostgreSQL for backward compatibility
        Ok(DatabaseConfig::Postgres {
            url: "postgres://postgres:postgres@localhost:5432/secretshare".to_string(),
        })
    }
}
