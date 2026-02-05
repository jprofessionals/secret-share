# DynamoDB Support Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add DynamoDB as an alternative database backend with auto-detection, comprehensive testing, and CI integration.

**Architecture:** The existing `SecretRepository` trait provides the abstraction. We add `DynamoDbRepository` implementing it, reorganize PostgreSQL code into a submodule, and update startup to auto-detect which database to use based on environment variables.

**Tech Stack:** Rust, aws-sdk-dynamodb, aws-config, testcontainers (dynamodb_local), Playwright

---

## Task 1: Add AWS SDK Dependencies

**Files:**
- Modify: `backend/Cargo.toml`

**Step 1: Add dependencies**

Add to `[dependencies]` section:

```toml
# AWS SDK for DynamoDB
aws-config = { version = "1.5", features = ["behavior-version-latest"] }
aws-sdk-dynamodb = "1.54"
```

Add to `[dev-dependencies]` section (update existing line):

```toml
testcontainers-modules = { version = "0.11", features = ["postgres", "dynamodb_local"] }
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully (downloads new dependencies)

**Step 3: Commit**

```bash
git add backend/Cargo.toml
git commit -m "Add AWS SDK dependencies for DynamoDB support"
```

---

## Task 2: Reorganize PostgreSQL Code into Submodule

**Files:**
- Create: `backend/src/db/postgres/mod.rs`
- Create: `backend/src/db/postgres/secrets.rs`
- Modify: `backend/src/db/mod.rs`
- Delete content from: `backend/src/db/secrets.rs` (will be removed)

**Step 1: Create postgres module file**

Create `backend/src/db/postgres/mod.rs`:

```rust
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::error::AppError;

mod secrets;

pub struct PostgresRepository {
    pub(crate) pool: PgPool,
}

impl PostgresRepository {
    pub async fn new(database_url: &str) -> Result<Self, AppError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), AppError> {
        sqlx::migrate!()
            .run(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
```

**Step 2: Move secrets implementation**

Create `backend/src/db/postgres/secrets.rs`:

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::PostgresRepository;
use crate::db::SecretRepository;
use crate::error::AppError;
use crate::models::Secret;

#[async_trait]
impl SecretRepository for PostgresRepository {
    async fn create_secret(&self, secret: &Secret) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO secrets (id, encrypted_data, created_at, expires_at, max_views, views, extendable, failed_attempts)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&secret.id)
        .bind(&secret.encrypted_data)
        .bind(&secret.created_at)
        .bind(&secret.expires_at)
        .bind(&secret.max_views)
        .bind(&secret.views)
        .bind(&secret.extendable)
        .bind(&secret.failed_attempts)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_secret(&self, id: &Uuid) -> Result<Option<Secret>, AppError> {
        let secret = sqlx::query_as::<_, Secret>(
            r#"
            SELECT id, encrypted_data, created_at, expires_at, max_views, views, extendable, failed_attempts
            FROM secrets
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(secret)
    }

    async fn update_secret(&self, secret: &Secret) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE secrets
            SET views = $2, failed_attempts = $3
            WHERE id = $1
            "#,
        )
        .bind(&secret.id)
        .bind(&secret.views)
        .bind(&secret.failed_attempts)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn extend_secret(
        &self,
        id: &Uuid,
        new_expires_at: DateTime<Utc>,
        new_max_views: Option<i32>,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE secrets
            SET expires_at = $2, max_views = $3
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(new_expires_at)
        .bind(new_max_views)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_secret(&self, id: &Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM secrets WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn cleanup_expired(&self) -> Result<u64, AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM secrets
            WHERE expires_at < NOW()
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected())
    }
}
```

**Step 3: Update db/mod.rs**

Replace `backend/src/db/mod.rs` with:

```rust
mod repository;
pub mod postgres;

pub use repository::SecretRepository;
#[cfg(test)]
pub use repository::MockSecretRepository;
pub use postgres::PostgresRepository;

// Backward compatibility alias
pub type Database = PostgresRepository;
```

**Step 4: Delete old secrets.rs**

Delete file: `backend/src/db/secrets.rs`

**Step 5: Verify compilation and tests**

Run: `cargo test`
Expected: All 19 tests pass (no behavior change)

**Step 6: Commit**

```bash
git add backend/src/db/
git commit -m "Reorganize PostgreSQL code into db/postgres submodule"
```

---

## Task 3: Create DynamoDB Repository Structure

**Files:**
- Create: `backend/src/db/dynamodb/mod.rs`
- Create: `backend/src/db/dynamodb/secrets.rs`
- Modify: `backend/src/db/mod.rs`

**Step 1: Create DynamoDB module**

Create `backend/src/db/dynamodb/mod.rs`:

```rust
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{config::Builder, Client};

use crate::error::AppError;

mod secrets;

pub struct DynamoDbRepository {
    pub(crate) client: Client,
    pub(crate) table_name: String,
}

impl DynamoDbRepository {
    pub async fn new(table_name: &str, endpoint: Option<&str>) -> Result<Self, AppError> {
        let config = aws_config::defaults(BehaviorVersion::latest()).load().await;

        let client = if let Some(endpoint_url) = endpoint {
            let dynamo_config = Builder::from(&config)
                .endpoint_url(endpoint_url)
                .build();
            Client::from_conf(dynamo_config)
        } else {
            Client::new(&config)
        };

        let repo = Self {
            client,
            table_name: table_name.to_string(),
        };

        // Create table if using local endpoint (development/testing)
        if endpoint.is_some() {
            repo.ensure_table_exists().await?;
        }

        Ok(repo)
    }

    async fn ensure_table_exists(&self) -> Result<(), AppError> {
        use aws_sdk_dynamodb::types::{
            AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType,
        };

        // Check if table exists
        let tables = self
            .client
            .list_tables()
            .send()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if tables
            .table_names()
            .iter()
            .any(|name| name == &self.table_name)
        {
            return Ok(());
        }

        // Create table
        self.client
            .create_table()
            .table_name(&self.table_name)
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("id")
                    .key_type(KeyType::Hash)
                    .build()
                    .map_err(|e| AppError::DatabaseError(e.to_string()))?,
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("id")
                    .attribute_type(ScalarAttributeType::S)
                    .build()
                    .map_err(|e| AppError::DatabaseError(e.to_string()))?,
            )
            .provisioned_throughput(
                ProvisionedThroughput::builder()
                    .read_capacity_units(5)
                    .write_capacity_units(5)
                    .build()
                    .map_err(|e| AppError::DatabaseError(e.to_string()))?,
            )
            .send()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Wait for table to be active
        self.client
            .wait_until_table_exists()
            .table_name(&self.table_name)
            .wait(std::time::Duration::from_secs(30))
            .await
            .map_err(|e| AppError::DatabaseError(format!("Table creation timeout: {}", e)))?;

        Ok(())
    }
}
```

**Step 2: Create secrets implementation stub**

Create `backend/src/db/dynamodb/secrets.rs`:

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::DynamoDbRepository;
use crate::db::SecretRepository;
use crate::error::AppError;
use crate::models::Secret;

#[async_trait]
impl SecretRepository for DynamoDbRepository {
    async fn create_secret(&self, _secret: &Secret) -> Result<(), AppError> {
        todo!("Implement create_secret for DynamoDB")
    }

    async fn get_secret(&self, _id: &Uuid) -> Result<Option<Secret>, AppError> {
        todo!("Implement get_secret for DynamoDB")
    }

    async fn update_secret(&self, _secret: &Secret) -> Result<(), AppError> {
        todo!("Implement update_secret for DynamoDB")
    }

    async fn extend_secret(
        &self,
        _id: &Uuid,
        _expires_at: DateTime<Utc>,
        _max_views: Option<i32>,
    ) -> Result<(), AppError> {
        todo!("Implement extend_secret for DynamoDB")
    }

    async fn delete_secret(&self, _id: &Uuid) -> Result<(), AppError> {
        todo!("Implement delete_secret for DynamoDB")
    }

    async fn cleanup_expired(&self) -> Result<u64, AppError> {
        // DynamoDB uses TTL for automatic cleanup
        tracing::info!("DynamoDB handles cleanup via TTL - no action needed");
        Ok(0)
    }
}
```

**Step 3: Update db/mod.rs to export DynamoDB**

Update `backend/src/db/mod.rs`:

```rust
mod repository;
pub mod postgres;
pub mod dynamodb;

pub use repository::SecretRepository;
#[cfg(test)]
pub use repository::MockSecretRepository;
pub use postgres::PostgresRepository;
pub use dynamodb::DynamoDbRepository;

// Backward compatibility alias
pub type Database = PostgresRepository;
```

**Step 4: Verify compilation**

Run: `cargo check`
Expected: Compiles (with warnings about unused code)

**Step 5: Commit**

```bash
git add backend/src/db/
git commit -m "Add DynamoDB repository structure with stubs"
```

---

## Task 4: Implement DynamoDB create_secret

**Files:**
- Modify: `backend/src/db/dynamodb/secrets.rs`

**Step 1: Implement create_secret**

Replace the `create_secret` method in `backend/src/db/dynamodb/secrets.rs`:

```rust
use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::DynamoDbRepository;
use crate::db::SecretRepository;
use crate::error::AppError;
use crate::models::Secret;

#[async_trait]
impl SecretRepository for DynamoDbRepository {
    async fn create_secret(&self, secret: &Secret) -> Result<(), AppError> {
        let mut item = vec![
            ("id".to_string(), AttributeValue::S(secret.id.to_string())),
            (
                "encrypted_data".to_string(),
                AttributeValue::S(secret.encrypted_data.clone()),
            ),
            (
                "created_at".to_string(),
                AttributeValue::S(secret.created_at.to_rfc3339()),
            ),
            (
                "expires_at".to_string(),
                AttributeValue::N(secret.expires_at.timestamp().to_string()),
            ),
            ("views".to_string(), AttributeValue::N(secret.views.to_string())),
            (
                "extendable".to_string(),
                AttributeValue::Bool(secret.extendable),
            ),
            (
                "failed_attempts".to_string(),
                AttributeValue::N(secret.failed_attempts.to_string()),
            ),
        ];

        if let Some(max_views) = secret.max_views {
            item.push(("max_views".to_string(), AttributeValue::N(max_views.to_string())));
        }

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item.into_iter().collect()))
            .send()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // ... rest of methods unchanged
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add backend/src/db/dynamodb/secrets.rs
git commit -m "Implement DynamoDB create_secret"
```

---

## Task 5: Implement DynamoDB get_secret

**Files:**
- Modify: `backend/src/db/dynamodb/secrets.rs`

**Step 1: Implement get_secret**

Replace the `get_secret` method:

```rust
    async fn get_secret(&self, id: &Uuid) -> Result<Option<Secret>, AppError> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let Some(item) = result.item else {
            return Ok(None);
        };

        let secret = self.item_to_secret(&item)?;
        Ok(Some(secret))
    }
```

**Step 2: Add helper method to convert DynamoDB item to Secret**

Add this helper method to the `impl SecretRepository for DynamoDbRepository` block (or as a separate impl block):

```rust
impl DynamoDbRepository {
    fn item_to_secret(
        &self,
        item: &std::collections::HashMap<String, AttributeValue>,
    ) -> Result<Secret, AppError> {
        let get_s = |key: &str| -> Result<String, AppError> {
            item.get(key)
                .and_then(|v| v.as_s().ok())
                .map(|s| s.to_string())
                .ok_or_else(|| AppError::DatabaseError(format!("Missing or invalid field: {}", key)))
        };

        let get_n = |key: &str| -> Result<i64, AppError> {
            item.get(key)
                .and_then(|v| v.as_n().ok())
                .and_then(|n| n.parse().ok())
                .ok_or_else(|| AppError::DatabaseError(format!("Missing or invalid field: {}", key)))
        };

        let get_bool = |key: &str| -> Result<bool, AppError> {
            item.get(key)
                .and_then(|v| v.as_bool().ok())
                .copied()
                .ok_or_else(|| AppError::DatabaseError(format!("Missing or invalid field: {}", key)))
        };

        let id = Uuid::parse_str(&get_s("id")?)
            .map_err(|e| AppError::DatabaseError(format!("Invalid UUID: {}", e)))?;

        let created_at = DateTime::parse_from_rfc3339(&get_s("created_at")?)
            .map_err(|e| AppError::DatabaseError(format!("Invalid created_at: {}", e)))?
            .with_timezone(&Utc);

        let expires_at_timestamp = get_n("expires_at")?;
        let expires_at = DateTime::from_timestamp(expires_at_timestamp, 0)
            .ok_or_else(|| AppError::DatabaseError("Invalid expires_at timestamp".to_string()))?;

        let max_views = item
            .get("max_views")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse().ok());

        Ok(Secret {
            id,
            encrypted_data: get_s("encrypted_data")?,
            created_at,
            expires_at,
            max_views,
            views: get_n("views")? as i32,
            extendable: get_bool("extendable")?,
            failed_attempts: get_n("failed_attempts")? as i32,
        })
    }
}
```

**Step 3: Add required import**

Ensure this import is at the top of the file:

```rust
use std::collections::HashMap;
```

**Step 4: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add backend/src/db/dynamodb/secrets.rs
git commit -m "Implement DynamoDB get_secret with item conversion"
```

---

## Task 6: Implement DynamoDB update_secret and delete_secret

**Files:**
- Modify: `backend/src/db/dynamodb/secrets.rs`

**Step 1: Implement update_secret**

Replace the `update_secret` method:

```rust
    async fn update_secret(&self, secret: &Secret) -> Result<(), AppError> {
        self.client
            .update_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(secret.id.to_string()))
            .update_expression("SET #views = :views, #failed_attempts = :failed_attempts")
            .expression_attribute_names("#views", "views")
            .expression_attribute_names("#failed_attempts", "failed_attempts")
            .expression_attribute_values(":views", AttributeValue::N(secret.views.to_string()))
            .expression_attribute_values(
                ":failed_attempts",
                AttributeValue::N(secret.failed_attempts.to_string()),
            )
            .send()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }
```

**Step 2: Implement delete_secret**

Replace the `delete_secret` method:

```rust
    async fn delete_secret(&self, id: &Uuid) -> Result<(), AppError> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }
```

**Step 3: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add backend/src/db/dynamodb/secrets.rs
git commit -m "Implement DynamoDB update_secret and delete_secret"
```

---

## Task 7: Implement DynamoDB extend_secret

**Files:**
- Modify: `backend/src/db/dynamodb/secrets.rs`

**Step 1: Implement extend_secret**

Replace the `extend_secret` method:

```rust
    async fn extend_secret(
        &self,
        id: &Uuid,
        new_expires_at: DateTime<Utc>,
        new_max_views: Option<i32>,
    ) -> Result<(), AppError> {
        let mut update_expr = "SET #expires_at = :expires_at".to_string();
        let mut expr_names = vec![("#expires_at".to_string(), "expires_at".to_string())];
        let mut expr_values = vec![(
            ":expires_at".to_string(),
            AttributeValue::N(new_expires_at.timestamp().to_string()),
        )];

        if let Some(max_views) = new_max_views {
            update_expr.push_str(", #max_views = :max_views");
            expr_names.push(("#max_views".to_string(), "max_views".to_string()));
            expr_values.push((":max_views".to_string(), AttributeValue::N(max_views.to_string())));
        }

        let mut req = self
            .client
            .update_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .update_expression(update_expr);

        for (name, value) in expr_names {
            req = req.expression_attribute_names(name, value);
        }

        for (name, value) in expr_values {
            req = req.expression_attribute_values(name, value);
        }

        req.send()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add backend/src/db/dynamodb/secrets.rs
git commit -m "Implement DynamoDB extend_secret"
```

---

## Task 8: Update Config for Database Auto-Detection

**Files:**
- Modify: `backend/src/config.rs`

**Step 1: Update Config struct**

Replace `backend/src/config.rs`:

```rust
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
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles (with errors in main.rs - expected)

**Step 3: Commit**

```bash
git add backend/src/config.rs
git commit -m "Update Config for database auto-detection"
```

---

## Task 9: Update AppState to Use Dynamic Repository

**Files:**
- Modify: `backend/src/lib.rs`

**Step 1: Update AppState and run function**

Replace `backend/src/lib.rs`:

```rust
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
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles (with errors in main.rs and tests - expected)

**Step 3: Commit**

```bash
git add backend/src/lib.rs
git commit -m "Update AppState to use dynamic SecretRepository"
```

---

## Task 10: Update main.rs

**Files:**
- Modify: `backend/src/main.rs`

**Step 1: Simplify main.rs**

Replace `backend/src/main.rs`:

```rust
use secret_share_backend::{config::Config, run};

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
    run(config).await
}
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add backend/src/main.rs
git commit -m "Simplify main.rs - database init moved to run()"
```

---

## Task 11: Update Cleanup Binary

**Files:**
- Modify: `backend/src/bin/cleanup.rs`

**Step 1: Update cleanup for both databases**

Replace `backend/src/bin/cleanup.rs`:

```rust
use secret_share_backend::{
    config::{Config, DatabaseConfig},
    db::{DynamoDbRepository, PostgresRepository, SecretRepository},
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
```

**Step 2: Verify compilation**

Run: `cargo build --bin cleanup`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add backend/src/bin/cleanup.rs
git commit -m "Update cleanup binary to handle both databases"
```

---

## Task 12: Update Integration Test Helpers for PostgreSQL

**Files:**
- Rename: `backend/tests/integration/helpers.rs` → `backend/tests/integration/postgres_context.rs`
- Modify: `backend/tests/integration/mod.rs`

**Step 1: Rename helpers.rs**

```bash
mv backend/tests/integration/helpers.rs backend/tests/integration/postgres_context.rs
```

**Step 2: Update postgres_context.rs**

Replace `backend/tests/integration/postgres_context.rs`:

```rust
use secret_share_backend::{
    config::{Config, DatabaseConfig},
    create_router,
    db::{PostgresRepository, SecretRepository},
    AppState,
};
use std::sync::Arc;
use testcontainers::{runners::AsyncRunner, ContainerAsync};
use testcontainers_modules::postgres::Postgres;
use tokio::net::TcpListener;

#[allow(dead_code)]
pub struct PostgresTestContext {
    pub base_url: String,
    pub client: reqwest::Client,
    pub db: Arc<dyn SecretRepository>,
    _container: ContainerAsync<Postgres>,
}

#[allow(dead_code)]
impl PostgresTestContext {
    pub async fn new() -> Self {
        // Start PostgreSQL container
        let container = Postgres::default()
            .start()
            .await
            .expect("Failed to start PostgreSQL container");

        let host = container.get_host().await.expect("Failed to get host");
        let port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("Failed to get port");

        let database_url = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);

        // Wait for database to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Initialize database
        let pg = PostgresRepository::new(&database_url)
            .await
            .expect("Failed to connect to database");
        pg.migrate().await.expect("Failed to run migrations");

        // Create app state with Config
        let config = Config {
            database: DatabaseConfig::Postgres {
                url: database_url.clone(),
            },
            base_url: "http://localhost".to_string(),
            port: 3000,
            max_secret_days: 30,
            max_secret_views: 100,
            max_failed_attempts: 10,
        };

        let db: Arc<dyn SecretRepository> = Arc::new(pg);
        let state = AppState {
            db: db.clone(),
            config: Arc::new(config),
        };

        // Start server on random port
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to random port");
        let addr = listener.local_addr().expect("Failed to get local address");
        let base_url = format!("http://{}", addr);

        let app = create_router(state);

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // Wait for server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        PostgresTestContext {
            base_url,
            client: reqwest::Client::new(),
            db,
            _container: container,
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

// Backward compatibility alias
pub type TestContext = PostgresTestContext;
```

**Step 3: Update mod.rs**

Replace `backend/tests/integration/mod.rs`:

```rust
pub mod postgres_context;

// Re-export for backward compatibility
pub use postgres_context::TestContext;
```

**Step 4: Verify tests pass**

Run: `cargo test`
Expected: All tests pass

**Step 5: Commit**

```bash
git add backend/tests/integration/
git commit -m "Rename helpers.rs to postgres_context.rs"
```

---

## Task 13: Create DynamoDB Test Context

**Files:**
- Create: `backend/tests/integration/dynamodb_context.rs`
- Modify: `backend/tests/integration/mod.rs`

**Step 1: Create DynamoDB test context**

Create `backend/tests/integration/dynamodb_context.rs`:

```rust
use secret_share_backend::{
    config::{Config, DatabaseConfig},
    create_router,
    db::{DynamoDbRepository, SecretRepository},
    AppState,
};
use std::sync::Arc;
use testcontainers::{runners::AsyncRunner, ContainerAsync, GenericImage};
use tokio::net::TcpListener;

#[allow(dead_code)]
pub struct DynamoDbTestContext {
    pub base_url: String,
    pub client: reqwest::Client,
    pub db: Arc<dyn SecretRepository>,
    _container: ContainerAsync<GenericImage>,
}

#[allow(dead_code)]
impl DynamoDbTestContext {
    pub async fn new() -> Self {
        // Start DynamoDB Local container
        let container = GenericImage::new("amazon/dynamodb-local", "latest")
            .with_exposed_port(8000.into())
            .start()
            .await
            .expect("Failed to start DynamoDB Local container");

        let host = container.get_host().await.expect("Failed to get host");
        let port = container
            .get_host_port_ipv4(8000)
            .await
            .expect("Failed to get port");

        let endpoint = format!("http://{}:{}", host, port);
        let table_name = "secrets-test";

        // Wait for DynamoDB to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Initialize DynamoDB repository (creates table automatically)
        let dynamo = DynamoDbRepository::new(table_name, Some(&endpoint))
            .await
            .expect("Failed to connect to DynamoDB");

        // Create app state with Config
        let config = Config {
            database: DatabaseConfig::DynamoDB {
                table: table_name.to_string(),
                endpoint: Some(endpoint),
            },
            base_url: "http://localhost".to_string(),
            port: 3000,
            max_secret_days: 30,
            max_secret_views: 100,
            max_failed_attempts: 10,
        };

        let db: Arc<dyn SecretRepository> = Arc::new(dynamo);
        let state = AppState {
            db: db.clone(),
            config: Arc::new(config),
        };

        // Start server on random port
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to random port");
        let addr = listener.local_addr().expect("Failed to get local address");
        let base_url = format!("http://{}", addr);

        let app = create_router(state);

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // Wait for server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        DynamoDbTestContext {
            base_url,
            client: reqwest::Client::new(),
            db,
            _container: container,
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}
```

**Step 2: Update mod.rs**

Replace `backend/tests/integration/mod.rs`:

```rust
pub mod postgres_context;
pub mod dynamodb_context;

// Re-export for backward compatibility
pub use postgres_context::TestContext;
```

**Step 3: Verify compilation**

Run: `cargo check --tests`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add backend/tests/integration/
git commit -m "Add DynamoDB test context"
```

---

## Task 14: Add Feature Flags for Test Selection

**Files:**
- Modify: `backend/Cargo.toml`

**Step 1: Add feature flags**

Add to `backend/Cargo.toml` after `[dev-dependencies]`:

```toml
[features]
default = []
postgres-tests = []
dynamodb-tests = []
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add backend/Cargo.toml
git commit -m "Add feature flags for test selection"
```

---

## Task 15: Create Test Macro for Dual-Database Testing

**Files:**
- Create: `backend/tests/integration/test_macro.rs`
- Modify: `backend/tests/integration/mod.rs`

**Step 1: Create test macro**

Create `backend/tests/integration/test_macro.rs`:

```rust
/// Macro to generate tests for both PostgreSQL and DynamoDB
///
/// Usage:
/// ```
/// test_both_databases!(test_name, |ctx| async move {
///     // Test code using ctx.client, ctx.url(), etc.
/// });
/// ```
#[macro_export]
macro_rules! test_both_databases {
    ($test_name:ident, $test_fn:expr) => {
        paste::paste! {
            #[cfg(feature = "postgres-tests")]
            #[tokio::test]
            async fn [<$test_name _postgres>]() {
                use crate::integration::postgres_context::PostgresTestContext;
                let ctx = PostgresTestContext::new().await;
                let test_fn = $test_fn;
                test_fn(&ctx).await;
            }

            #[cfg(feature = "dynamodb-tests")]
            #[tokio::test]
            async fn [<$test_name _dynamodb>]() {
                use crate::integration::dynamodb_context::DynamoDbTestContext;
                let ctx = DynamoDbTestContext::new().await;
                let test_fn = $test_fn;
                test_fn(&ctx).await;
            }
        }
    };
}
```

**Step 2: Add paste dependency**

Add to `[dev-dependencies]` in `backend/Cargo.toml`:

```toml
paste = "1.0"
```

**Step 3: Update mod.rs**

Replace `backend/tests/integration/mod.rs`:

```rust
pub mod postgres_context;
pub mod dynamodb_context;
#[macro_use]
pub mod test_macro;

// Re-export for backward compatibility
pub use postgres_context::TestContext;
```

**Step 4: Verify compilation**

Run: `cargo check --tests --features postgres-tests`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add backend/Cargo.toml backend/tests/integration/
git commit -m "Add test macro for dual-database testing"
```

---

## Task 16: Add TestContext Trait for Polymorphic Tests

**Files:**
- Modify: `backend/tests/integration/mod.rs`
- Modify: `backend/tests/integration/postgres_context.rs`
- Modify: `backend/tests/integration/dynamodb_context.rs`

**Step 1: Add TestContext trait**

Add to `backend/tests/integration/mod.rs`:

```rust
pub mod postgres_context;
pub mod dynamodb_context;
#[macro_use]
pub mod test_macro;

// Re-export for backward compatibility
pub use postgres_context::PostgresTestContext as TestContext;

/// Common interface for test contexts
pub trait TestContextTrait {
    fn url(&self, path: &str) -> String;
    fn client(&self) -> &reqwest::Client;
}
```

**Step 2: Implement trait for PostgresTestContext**

Add to `backend/tests/integration/postgres_context.rs`:

```rust
use super::TestContextTrait;

impl TestContextTrait for PostgresTestContext {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn client(&self) -> &reqwest::Client {
        &self.client
    }
}
```

**Step 3: Implement trait for DynamoDbTestContext**

Add to `backend/tests/integration/dynamodb_context.rs`:

```rust
use super::TestContextTrait;

impl TestContextTrait for DynamoDbTestContext {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn client(&self) -> &reqwest::Client {
        &self.client
    }
}
```

**Step 4: Verify compilation**

Run: `cargo check --tests`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add backend/tests/integration/
git commit -m "Add TestContextTrait for polymorphic test contexts"
```

---

## Task 17: Update CI Workflow for Parallel Database Tests

**Files:**
- Modify: `.github/workflows/ci.yml`

**Step 1: Update CI workflow**

Replace `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test-backend-postgres:
    name: Backend Tests (PostgreSQL)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: backend -> target

      - name: Run PostgreSQL tests
        working-directory: backend
        run: cargo test --features postgres-tests

  test-backend-dynamodb:
    name: Backend Tests (DynamoDB)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: backend -> target

      - name: Run DynamoDB tests
        working-directory: backend
        run: cargo test --features dynamodb-tests

  test-frontend:
    name: Frontend Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json

      - name: Install dependencies
        working-directory: frontend
        run: npm ci

      - name: Run tests
        working-directory: frontend
        run: npm test

  test-e2e:
    name: E2E Tests (DynamoDB)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: backend -> target

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: |
            frontend/package-lock.json
            e2e/package-lock.json

      - name: Install frontend dependencies
        working-directory: frontend
        run: npm ci

      - name: Install E2E dependencies
        working-directory: e2e
        run: npm ci

      - name: Cache Playwright browsers
        uses: actions/cache@v4
        with:
          path: ~/.cache/ms-playwright
          key: playwright-${{ runner.os }}-${{ hashFiles('e2e/package-lock.json') }}

      - name: Install Playwright browsers
        working-directory: e2e
        run: npx playwright install --with-deps chromium

      - name: Run E2E tests
        working-directory: e2e
        run: npm test
        env:
          E2E_DATABASE: dynamodb

      - name: Upload Playwright report
        uses: actions/upload-artifact@v4
        if: failure()
        with:
          name: playwright-report
          path: e2e/playwright-report/
          retention-days: 7
```

**Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "Update CI for parallel PostgreSQL and DynamoDB tests"
```

---

## Task 18: Update E2E Global Setup for Database Selection

**Files:**
- Modify: `e2e/fixtures/global-setup.ts`

**Step 1: Update global-setup.ts**

Replace `e2e/fixtures/global-setup.ts`:

```typescript
import { spawn, execSync } from 'child_process';
import { GenericContainer, Wait } from 'testcontainers';
import * as fs from 'fs';
import * as path from 'path';

const STATE_FILE = '/tmp/e2e-test-context.json';
const ROOT_DIR = path.resolve(__dirname, '../..');

interface TestState {
  database: 'postgres' | 'dynamodb';
  databasePort: number;
  backendPid: number;
  frontendPid: number;
  backendUrl: string;
  frontendUrl: string;
}

async function waitForUrl(url: string, maxAttempts = 30): Promise<void> {
  for (let i = 0; i < maxAttempts; i++) {
    try {
      const response = await fetch(url);
      if (response.ok) return;
    } catch {
      // Ignore errors, keep trying
    }
    await new Promise((r) => setTimeout(r, 1000));
  }
  throw new Error(`Timeout waiting for ${url}`);
}

async function startPostgres(): Promise<{ port: number; env: Record<string, string> }> {
  console.log('Starting PostgreSQL container...');
  const container = await new GenericContainer('postgres:16-alpine')
    .withEnvironment({
      POSTGRES_PASSWORD: 'postgres',
      POSTGRES_DB: 'secretshare',
    })
    .withExposedPorts(5432)
    .withWaitStrategy(Wait.forLogMessage('database system is ready to accept connections'))
    .start();

  const port = container.getMappedPort(5432);
  const host = container.getHost();
  const databaseUrl = `postgres://postgres:postgres@${host}:${port}/secretshare`;

  console.log(`PostgreSQL running at ${host}:${port}`);
  await new Promise((r) => setTimeout(r, 3000));

  return {
    port,
    env: { DATABASE_URL: databaseUrl },
  };
}

async function startDynamoDB(): Promise<{ port: number; env: Record<string, string> }> {
  console.log('Starting DynamoDB Local container...');
  const container = await new GenericContainer('amazon/dynamodb-local')
    .withExposedPorts(8000)
    .withWaitStrategy(Wait.forListeningPorts())
    .start();

  const port = container.getMappedPort(8000);
  const host = container.getHost();
  const endpoint = `http://${host}:${port}`;

  console.log(`DynamoDB Local running at ${endpoint}`);
  await new Promise((r) => setTimeout(r, 1000));

  return {
    port,
    env: {
      DYNAMODB_TABLE: 'secrets',
      DYNAMODB_ENDPOINT: endpoint,
      AWS_ACCESS_KEY_ID: 'test',
      AWS_SECRET_ACCESS_KEY: 'test',
      AWS_REGION: 'us-east-1',
    },
  };
}

async function globalSetup(): Promise<void> {
  console.log('Starting E2E test infrastructure...');

  const database = (process.env.E2E_DATABASE || 'dynamodb') as 'postgres' | 'dynamodb';
  console.log(`Using database: ${database}`);

  // 1. Start database container
  const { port: databasePort, env: dbEnv } =
    database === 'postgres' ? await startPostgres() : await startDynamoDB();

  // 2. Build and start backend
  console.log('Building backend...');
  execSync('cargo build --release', {
    cwd: path.join(ROOT_DIR, 'backend'),
    stdio: 'inherit',
  });

  console.log('Starting backend...');
  const backendProcess = spawn('./target/release/secret-share-backend', [], {
    cwd: path.join(ROOT_DIR, 'backend'),
    env: {
      ...process.env,
      ...dbEnv,
      BASE_URL: 'http://localhost:4173',
      PORT: '3000',
      RUST_LOG: 'info',
    },
    stdio: ['ignore', 'inherit', 'inherit'],
    detached: true,
  });

  backendProcess.unref();

  const backendUrl = 'http://localhost:3000';
  console.log('Waiting for backend to be ready...');
  await waitForUrl(`${backendUrl}/health`);
  console.log('Backend ready');

  // 3. Build and start frontend
  console.log('Building frontend...');
  execSync('npm run build', {
    cwd: path.join(ROOT_DIR, 'frontend'),
    stdio: 'inherit',
    env: {
      ...process.env,
      VITE_API_URL: backendUrl,
    },
  });

  console.log('Starting frontend preview server...');
  const frontendProcess = spawn('npx', ['vite', 'preview', '--port', '4173', '--host'], {
    cwd: path.join(ROOT_DIR, 'frontend'),
    env: {
      ...process.env,
    },
    stdio: 'pipe',
    detached: true,
  });

  frontendProcess.unref();

  const frontendUrl = 'http://localhost:4173';
  console.log('Waiting for frontend to be ready...');
  await waitForUrl(frontendUrl);
  console.log('Frontend ready');

  // Save state for teardown
  const state: TestState = {
    database,
    databasePort,
    backendPid: backendProcess.pid!,
    frontendPid: frontendProcess.pid!,
    backendUrl,
    frontendUrl,
  };

  fs.writeFileSync(STATE_FILE, JSON.stringify(state));

  console.log('E2E infrastructure ready');
}

export default globalSetup;
```

**Step 2: Verify TypeScript compiles**

Run: `cd e2e && npx tsc --noEmit`
Expected: No errors

**Step 3: Commit**

```bash
git add e2e/fixtures/global-setup.ts
git commit -m "Update E2E global-setup for database selection"
```

---

## Task 19: Update E2E Global Teardown

**Files:**
- Modify: `e2e/fixtures/global-teardown.ts`

**Step 1: Update global-teardown.ts**

Replace `e2e/fixtures/global-teardown.ts`:

```typescript
import * as fs from 'fs';
import { execSync } from 'child_process';

const STATE_FILE = '/tmp/e2e-test-context.json';

interface TestState {
  database: 'postgres' | 'dynamodb';
  databasePort: number;
  backendPid: number;
  frontendPid: number;
  backendUrl: string;
  frontendUrl: string;
}

async function globalTeardown(): Promise<void> {
  console.log('Tearing down E2E test infrastructure...');

  try {
    const state: TestState = JSON.parse(fs.readFileSync(STATE_FILE, 'utf-8'));

    // Kill backend process
    try {
      process.kill(state.backendPid, 'SIGTERM');
      console.log('Backend stopped');
    } catch {
      console.log('Backend already stopped');
    }

    // Kill frontend process
    try {
      process.kill(state.frontendPid, 'SIGTERM');
      console.log('Frontend stopped');
    } catch {
      console.log('Frontend already stopped');
    }

    // Stop database container
    const containerPort = state.database === 'postgres' ? state.databasePort : state.databasePort;
    try {
      execSync(`docker stop $(docker ps -q --filter "publish=${containerPort}")`, {
        stdio: 'pipe',
      });
      console.log(`${state.database} container stopped`);
    } catch {
      console.log(`${state.database} container already stopped`);
    }

    // Clean up state file
    fs.unlinkSync(STATE_FILE);
  } catch (error) {
    console.error('Error during teardown:', error);
  }
}

export default globalTeardown;
```

**Step 2: Commit**

```bash
git add e2e/fixtures/global-teardown.ts
git commit -m "Update E2E global-teardown for database selection"
```

---

## Task 20: Convert Existing Tests to Use Macro (api_create_secret)

**Files:**
- Modify: `backend/tests/api_create_secret.rs`

**Step 1: Update test file to use macro**

Replace `backend/tests/api_create_secret.rs`:

```rust
#[macro_use]
mod integration;

use integration::TestContextTrait;
use serde_json::json;

test_both_databases!(test_create_secret_returns_valid_response, |ctx| async move {
    let response = ctx
        .client()
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "my-api-key-12345",
            "max_views": 5,
            "expires_in_hours": 24
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");

    // Verify response structure
    assert!(body["id"].is_string());
    assert!(body["passphrase"].is_string());
    assert!(body["share_url"].is_string());
    assert!(body["expires_at"].is_string());

    // Verify passphrase is 3 words separated by dashes
    let passphrase = body["passphrase"].as_str().unwrap();
    let word_count = passphrase.split('-').count();
    assert_eq!(word_count, 3, "Passphrase should be 3 words: {}", passphrase);
});

test_both_databases!(test_create_secret_with_extendable_true, |ctx| async move {
    let response = ctx
        .client()
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "extendable-secret",
            "max_views": 5,
            "expires_in_hours": 24,
            "extendable": true
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["id"].is_string());
});

test_both_databases!(test_create_secret_with_extendable_false, |ctx| async move {
    let response = ctx
        .client()
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "non-extendable-secret",
            "extendable": false
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["id"].is_string());
});

test_both_databases!(test_create_secret_defaults_extendable_true, |ctx| async move {
    // Create without specifying extendable
    let create_response = ctx
        .client()
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "test-secret"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(create_response.status(), 200);
    let created: serde_json::Value = create_response.json().await.unwrap();
    let id = created["id"].as_str().unwrap();
    let passphrase = created["passphrase"].as_str().unwrap();

    // Retrieve and verify extendable defaults to true
    let retrieve_response = ctx
        .client()
        .post(ctx.url(&format!("/api/secrets/{}", id)))
        .json(&json!({ "passphrase": passphrase }))
        .send()
        .await
        .unwrap();

    assert_eq!(retrieve_response.status(), 200);
    let body: serde_json::Value = retrieve_response.json().await.unwrap();
    assert_eq!(body["extendable"], true);
});
```

**Step 2: Run tests with postgres feature**

Run: `cargo test --features postgres-tests test_create`
Expected: 4 tests pass

**Step 3: Run tests with dynamodb feature**

Run: `cargo test --features dynamodb-tests test_create`
Expected: 4 tests pass

**Step 4: Commit**

```bash
git add backend/tests/api_create_secret.rs
git commit -m "Convert api_create_secret tests to dual-database macro"
```

---

## Task 21: Convert api_retrieve_secret Tests

**Files:**
- Modify: `backend/tests/api_retrieve_secret.rs`

**Step 1: Update test file**

Replace `backend/tests/api_retrieve_secret.rs`:

```rust
#[macro_use]
mod integration;

use integration::TestContextTrait;
use serde_json::json;

test_both_databases!(test_retrieve_secret_success, |ctx| async move {
    // First create a secret
    let create_response = ctx
        .client()
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "my-secret-data",
            "max_views": 5
        }))
        .send()
        .await
        .expect("Failed to create secret");

    let created: serde_json::Value = create_response.json().await.unwrap();
    let id = created["id"].as_str().unwrap();
    let passphrase = created["passphrase"].as_str().unwrap();

    // Retrieve the secret
    let response = ctx
        .client()
        .post(ctx.url(&format!("/api/secrets/{}", id)))
        .json(&json!({ "passphrase": passphrase }))
        .send()
        .await
        .expect("Failed to retrieve secret");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["secret"], "my-secret-data");
    assert_eq!(body["views_remaining"], 4);
});

test_both_databases!(test_retrieve_decrements_view_count, |ctx| async move {
    // Create a secret with 3 views
    let create_response = ctx
        .client()
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "test-secret",
            "max_views": 3
        }))
        .send()
        .await
        .unwrap();

    let created: serde_json::Value = create_response.json().await.unwrap();
    let id = created["id"].as_str().unwrap();
    let passphrase = created["passphrase"].as_str().unwrap();

    // First retrieval
    let r1 = ctx
        .client()
        .post(ctx.url(&format!("/api/secrets/{}", id)))
        .json(&json!({ "passphrase": passphrase }))
        .send()
        .await
        .unwrap();
    let b1: serde_json::Value = r1.json().await.unwrap();
    assert_eq!(b1["views_remaining"], 2);

    // Second retrieval
    let r2 = ctx
        .client()
        .post(ctx.url(&format!("/api/secrets/{}", id)))
        .json(&json!({ "passphrase": passphrase }))
        .send()
        .await
        .unwrap();
    let b2: serde_json::Value = r2.json().await.unwrap();
    assert_eq!(b2["views_remaining"], 1);

    // Third retrieval (last view)
    let r3 = ctx
        .client()
        .post(ctx.url(&format!("/api/secrets/{}", id)))
        .json(&json!({ "passphrase": passphrase }))
        .send()
        .await
        .unwrap();
    let b3: serde_json::Value = r3.json().await.unwrap();
    assert_eq!(b3["views_remaining"], 0);

    // Fourth retrieval should fail (secret deleted)
    let r4 = ctx
        .client()
        .post(ctx.url(&format!("/api/secrets/{}", id)))
        .json(&json!({ "passphrase": passphrase }))
        .send()
        .await
        .unwrap();
    assert_eq!(r4.status(), 404);
});
```

**Step 2: Run tests**

Run: `cargo test --features postgres-tests test_retrieve`
Expected: 2 tests pass

Run: `cargo test --features dynamodb-tests test_retrieve`
Expected: 2 tests pass

**Step 3: Commit**

```bash
git add backend/tests/api_retrieve_secret.rs
git commit -m "Convert api_retrieve_secret tests to dual-database macro"
```

---

## Task 22: Convert api_edge_cases Tests

**Files:**
- Modify: `backend/tests/api_edge_cases.rs`

**Step 1: Read current file**

First read the current file content to understand what tests exist.

**Step 2: Update to use macro**

Convert all tests to use the `test_both_databases!` macro, following the same pattern as previous tasks.

**Step 3: Run tests**

Run: `cargo test --features postgres-tests`
Run: `cargo test --features dynamodb-tests`
Expected: All tests pass

**Step 4: Commit**

```bash
git add backend/tests/api_edge_cases.rs
git commit -m "Convert api_edge_cases tests to dual-database macro"
```

---

## Task 23: Keep cleanup_expired Tests PostgreSQL-Only

**Files:**
- Modify: `backend/tests/cleanup_expired.rs`

**Step 1: Add feature gate**

Update `backend/tests/cleanup_expired.rs` to only run with postgres-tests feature:

```rust
#![cfg(feature = "postgres-tests")]

mod integration;

use integration::postgres_context::PostgresTestContext;
use secret_share_backend::db::SecretRepository;
// ... rest of existing tests unchanged, but use PostgresTestContext instead of TestContext
```

**Step 2: Run tests**

Run: `cargo test --features postgres-tests cleanup`
Expected: 2 tests pass

Run: `cargo test --features dynamodb-tests cleanup`
Expected: 0 tests (none compiled)

**Step 3: Commit**

```bash
git add backend/tests/cleanup_expired.rs
git commit -m "Gate cleanup_expired tests to PostgreSQL only"
```

---

## Task 24: Run Full Test Suite

**Step 1: Run all PostgreSQL tests**

Run: `cargo test --features postgres-tests`
Expected: All tests pass

**Step 2: Run all DynamoDB tests**

Run: `cargo test --features dynamodb-tests`
Expected: All tests pass (except cleanup which is gated)

**Step 3: Run frontend tests**

Run: `cd ../frontend && npm test`
Expected: All 13 tests pass

**Step 4: Commit any fixes if needed**

---

## Task 25: Update README and Documentation

**Files:**
- Modify: `CLAUDE.md` (add DynamoDB section)

**Step 1: Add DynamoDB documentation**

Add to `CLAUDE.md` in the Environment Variables section:

```markdown
### DynamoDB (alternative to PostgreSQL)

- `DYNAMODB_TABLE` - DynamoDB table name (if set, uses DynamoDB instead of PostgreSQL)
- `DYNAMODB_ENDPOINT` - Optional endpoint URL for DynamoDB Local
- AWS credentials via standard SDK chain (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, AWS_REGION, or IAM role)
```

**Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "Document DynamoDB configuration in CLAUDE.md"
```

---

## Task 26: Final Integration Test

**Step 1: Run E2E with DynamoDB**

Run: `cd ../e2e && E2E_DATABASE=dynamodb npm test`
Expected: All E2E tests pass

**Step 2: Run E2E with PostgreSQL**

Run: `cd ../e2e && E2E_DATABASE=postgres npm test`
Expected: All E2E tests pass

**Step 3: Verify CI workflow locally (optional)**

Run: `act -j test-backend-postgres` (if act is installed)
Run: `act -j test-backend-dynamodb`

---

## Summary

After completing all tasks:

1. **New files created:**
   - `backend/src/db/postgres/mod.rs`
   - `backend/src/db/postgres/secrets.rs`
   - `backend/src/db/dynamodb/mod.rs`
   - `backend/src/db/dynamodb/secrets.rs`
   - `backend/tests/integration/postgres_context.rs`
   - `backend/tests/integration/dynamodb_context.rs`
   - `backend/tests/integration/test_macro.rs`

2. **Files modified:**
   - `backend/Cargo.toml` (AWS SDK deps, feature flags, paste)
   - `backend/src/db/mod.rs` (exports both repos)
   - `backend/src/config.rs` (DatabaseConfig enum)
   - `backend/src/lib.rs` (dynamic AppState)
   - `backend/src/main.rs` (simplified)
   - `backend/src/bin/cleanup.rs` (handles both DBs)
   - `backend/tests/integration/mod.rs`
   - `backend/tests/api_*.rs` (use macro)
   - `.github/workflows/ci.yml` (parallel jobs)
   - `e2e/fixtures/global-setup.ts` (E2E_DATABASE)
   - `e2e/fixtures/global-teardown.ts`
   - `CLAUDE.md` (documentation)

3. **Files deleted:**
   - `backend/src/db/secrets.rs` (moved to postgres/secrets.rs)
