use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
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

        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/secretshare".to_string()),
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
}
