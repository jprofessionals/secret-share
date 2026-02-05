# DynamoDB Support Design

## Overview

Add DynamoDB as an alternative database backend to SecretShare, enabling AWS deployment without managed PostgreSQL and leveraging pay-per-request pricing for cost optimization.

## Configuration

### Auto-Detection Logic

At startup, the application checks:
- If `DATABASE_URL` is set and starts with `postgres://` → use PostgreSQL
- Otherwise, if `DYNAMODB_TABLE` is set → use DynamoDB
- If neither is configured → error with clear message

### Environment Variables

**PostgreSQL (existing)**
| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | Yes | PostgreSQL connection string |

**DynamoDB (new)**
| Variable | Required | Description |
|----------|----------|-------------|
| `DYNAMODB_TABLE` | Yes | Table name for secrets |
| `DYNAMODB_ENDPOINT` | No | Override endpoint (for DynamoDB Local) |
| `AWS_REGION` | Yes* | Via standard AWS SDK chain |
| AWS credentials | Yes | Via standard SDK chain (env, IAM role, config file) |

## Architecture

### Database Abstraction

The existing `SecretRepository` trait stays unchanged. A new `DynamoDbRepository` struct implements it alongside the existing PostgreSQL implementation.

### File Structure

```
backend/src/db/
├── mod.rs              # Auto-detection logic, exports
├── repository.rs       # SecretRepository trait (unchanged)
├── postgres/
│   ├── mod.rs          # PostgresRepository struct + connection
│   └── secrets.rs      # Trait implementation
└── dynamodb/
    ├── mod.rs          # DynamoDbRepository struct + client setup
    └── secrets.rs      # Trait implementation
```

### AppState Change

```rust
pub struct AppState {
    pub db: Arc<dyn SecretRepository>,  // Changed from Arc<Database>
    pub config: Arc<Config>,
}
```

## DynamoDB Table Schema

| Attribute | Type | Key | Description |
|-----------|------|-----|-------------|
| `id` | String (UUID) | Partition Key | Secret identifier |
| `encrypted_data` | Binary | - | Encrypted secret blob |
| `created_at` | String (ISO 8601) | - | Creation timestamp |
| `expires_at` | Number (Unix epoch) | - | Expiration time (TTL attribute) |
| `max_views` | Number | - | Max allowed views (null = unlimited) |
| `views` | Number | - | Current view count |
| `extendable` | Boolean | - | Whether secret can be extended |
| `failed_attempts` | Number | - | Wrong passphrase counter |

### Key Decisions

- **Partition key only** (no sort key) - Each secret is accessed by ID, no range queries needed
- **`expires_at` as Unix epoch** - Required format for DynamoDB TTL
- **Binary type for encrypted_data** - Avoids base64 encoding overhead in storage

### TTL Configuration

The table must have TTL enabled on the `expires_at` attribute. DynamoDB automatically deletes expired items (usually within minutes, guaranteed within 48 hours).

### Table Creation

- **Local development**: `DynamoDbRepository::new()` creates the table if it doesn't exist when `DYNAMODB_ENDPOINT` is set
- **Production**: Table creation handled by infrastructure (Terraform, CloudFormation, or manual)

## Implementation Details

### New Dependencies

```toml
[dependencies]
aws-config = "1.5"
aws-sdk-dynamodb = "1.54"

[dev-dependencies]
testcontainers-modules = { version = "0.11", features = ["postgres", "dynamodb_local"] }
```

### DynamoDbRepository

```rust
pub struct DynamoDbRepository {
    client: aws_sdk_dynamodb::Client,
    table_name: String,
}

impl DynamoDbRepository {
    pub async fn new(table_name: &str, endpoint: Option<&str>) -> Result<Self, AppError> {
        let mut config = aws_config::from_env();
        if let Some(endpoint) = endpoint {
            config = config.endpoint_url(endpoint);
        }
        let config = config.load().await;
        let client = aws_sdk_dynamodb::Client::new(&config);

        Ok(Self { client, table_name: table_name.to_string() })
    }
}
```

### Database Initialization (main.rs)

```rust
let db: Arc<dyn SecretRepository> = if let Some(ref url) = config.database_url {
    if url.starts_with("postgres://") {
        let pg = PostgresRepository::new(url).await?;
        pg.migrate().await?;
        Arc::new(pg)
    } else {
        return Err(anyhow!("Invalid DATABASE_URL: must start with postgres://"));
    }
} else if let Some(ref table) = config.dynamodb_table {
    let dynamo = DynamoDbRepository::new(table, config.dynamodb_endpoint.as_deref()).await?;
    Arc::new(dynamo)
} else {
    return Err(anyhow!("No database configured. Set DATABASE_URL or DYNAMODB_TABLE"));
};
```

### Cleanup Binary

The cleanup binary checks which database is configured:
- **PostgreSQL**: Runs the existing cleanup query
- **DynamoDB**: Logs that TTL handles cleanup automatically, exits successfully

## Testing Strategy

### Backend Integration Tests

Tests run against both databases using feature flags:

```toml
[features]
postgres-tests = []
dynamodb-tests = []
```

**Test Structure**
```
backend/tests/
├── integration/
│   ├── mod.rs
│   ├── postgres_context.rs  # TestContext for PostgreSQL
│   └── dynamodb_context.rs  # TestContext for DynamoDB Local
├── api_create_secret.rs     # Uses macro to run against both
├── api_retrieve_secret.rs
├── api_edge_cases.rs
└── cleanup_expired.rs       # PostgreSQL only
```

**Test Macro Pattern**
```rust
// Expands to: test_create_secret_postgres, test_create_secret_dynamodb
test_both_databases!(test_create_secret, |ctx| async {
    // Test logic using ctx.client, ctx.url(), etc.
});
```

### E2E Tests

E2E tests are configurable via `E2E_DATABASE` environment variable:
- `dynamodb` (default): Starts DynamoDB Local container
- `postgres`: Starts PostgreSQL container

## CI Configuration

### GitHub Actions Workflow

```yaml
jobs:
  test-backend-postgres:
    runs-on: ubuntu-latest
    steps:
      - Checkout, Rust setup, caching
      - Run: cargo test --features postgres-tests

  test-backend-dynamodb:
    runs-on: ubuntu-latest
    steps:
      - Checkout, Rust setup, caching
      - Run: cargo test --features dynamodb-tests

  test-frontend:
    # Unchanged

  test-e2e:
    runs-on: ubuntu-latest
    steps:
      - Checkout, Rust setup, Node setup, caching
      - Build backend and frontend
      - Start DynamoDB Local container
      - Run backend with DYNAMODB_TABLE + DYNAMODB_ENDPOINT
      - Run Playwright tests
```

## Files to Create/Modify

| Action | File |
|--------|------|
| Create | `backend/src/db/dynamodb/mod.rs` |
| Create | `backend/src/db/dynamodb/secrets.rs` |
| Create | `backend/src/db/postgres/mod.rs` |
| Create | `backend/src/db/postgres/secrets.rs` |
| Create | `backend/tests/integration/dynamodb_context.rs` |
| Modify | `backend/src/db/mod.rs` |
| Modify | `backend/src/config.rs` |
| Modify | `backend/src/main.rs` |
| Modify | `backend/src/bin/cleanup.rs` |
| Modify | `backend/Cargo.toml` |
| Rename | `backend/tests/integration/helpers.rs` → `postgres_context.rs` |
| Modify | `backend/tests/*.rs` (add test macro) |
| Modify | `.github/workflows/ci.yml` |
| Modify | `e2e/fixtures/global-setup.ts` |
| Modify | `e2e/fixtures/global-teardown.ts` |

## Out of Scope

- Infrastructure code (Terraform/CloudFormation for DynamoDB table)
- AWS Lambda deployment updates
- Kubernetes deployment updates for DynamoDB
- Data migration between PostgreSQL and DynamoDB
