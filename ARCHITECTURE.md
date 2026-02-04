# SecretShare Architecture

## Overview

SecretShare is a secure service for sharing secrets built as a modern 12-factor app. The system consists of three main components:

1. **Frontend** (Svelte) - User interface
2. **Backend** (Rust) - API and business logic
3. **Database** (PostgreSQL) - Data persistence

## Security Principles

### End-to-End Encryption

Encryption happens **before** data leaves the client:

```
1. User creates secret in frontend
2. Frontend generates 3-word passphrase
3. Frontend encrypts secret with passphrase (AES-256-GCM)
4. Only encrypted data is sent to backend
5. Backend stores encrypted data in database
6. Recipient uses passphrase to decrypt
```

**Important**: Backend/database never has access to unencrypted data.

### Cryptography Details

#### Passphrase Generation
```rust
// Generate 3 random words from BIP39 wordlist (2048 words)
// Entropy: log2(2048^3) ~ 33 bits
// Example: "abandon-ability-able"
```

#### Key Derivation
```rust
// Argon2id with salt
Passphrase -> Argon2id -> 256-bit key
```

**Parameters**:
- Memory: 64 MB
- Iterations: 3
- Parallelism: 4 threads

#### Encryption
```rust
// AES-256-GCM
Plaintext -> AES-256-GCM -> Ciphertext + Auth Tag

// Format: [Salt(16) || Nonce(12) || Ciphertext || Tag(16)]
```

### Brute Force Protection

Failed password attempts are tracked to prevent brute force attacks:

- **First 2 attempts**: No penalty (free attempts)
- **3+ attempts**: Each attempt uses one view
- **Views exhausted**: Secret is deleted
- **Unlimited views**: Deleted after `MAX_FAILED_ATTEMPTS` (default: 10)
- **Successful retrieval**: Resets counter

## Data Flow

### Creating a Secret

```
+---------+         +---------+         +----------+
| Browser |         | Backend |         | Database |
+----+----+         +----+----+         +-----+----+
     |                   |                    |
     | POST /api/secrets |                    |
     +------------------>|                    |
     | {                 |                    |
     |   encrypted_data  |                    |
     |   settings        |                    |
     | }                 |                    |
     |                   | INSERT secret      |
     |                   +------------------->|
     |                   |                    |
     |                   |<-------------------+
     |                   |                    |
     |<------------------+                    |
     | {                 |                    |
     |   id,             |                    |
     |   passphrase,     |                    |
     |   share_url       |                    |
     | }                 |                    |
```

### Retrieving a Secret

```
+---------+         +---------+         +----------+
| Browser |         | Backend |         | Database |
+----+----+         +----+----+         +-----+----+
     |                   |                    |
     |POST /api/secrets/:id                   |
     +------------------>|                    |
     | {passphrase}      |                    |
     |                   | SELECT secret      |
     |                   +------------------->|
     |                   |                    |
     |                   |<-------------------+
     |                   | encrypted_data     |
     |                   |                    |
     |                   | [Verify passphrase]|
     |                   | [Update views/     |
     |                   |  failed_attempts]  |
     |                   +------------------->|
     |                   |                    |
     |<------------------+                    |
     | encrypted_data    |                    |
     |                   |                    |
     | [Decrypt locally] |                    |
```

## Database Schema

```sql
CREATE TABLE secrets (
    id UUID PRIMARY KEY,
    encrypted_data TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    max_views INTEGER,
    views INTEGER NOT NULL DEFAULT 0,
    extendable BOOLEAN NOT NULL DEFAULT TRUE,
    failed_attempts INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_secrets_expires_at ON secrets(expires_at);
```

**Important**: `encrypted_data` never contains plaintext!

## Source Code Structure

```
backend/
  src/
    routes/
      mod.rs          # Router setup
      secrets.rs      # HTTP handlers
    services/
      mod.rs
      secrets.rs      # Business logic (create, retrieve, extend)
    crypto/
      mod.rs
      encryption.rs   # AES-256-GCM encryption
      passphrase.rs   # BIP39 passphrase generation
    db/
      mod.rs          # Database setup and migrations
      secrets.rs      # Database queries
    models/
      mod.rs
      secrets.rs      # Data structures
    config.rs         # Environment variable configuration
    error.rs          # Error handling
    lib.rs            # AppState and router
    main.rs           # Entry point
```

## API Endpoints

### `GET /health`
Health check for load balancer.

**Response**: `200 OK`

### `POST /api/secrets`
Create a new secret.

**Request**:
```json
{
  "secret": "encrypted_base64_data",
  "max_views": 1,
  "expires_in_hours": 24,
  "extendable": true
}
```

**Response**:
```json
{
  "id": "uuid",
  "passphrase": "word1-word2-word3",
  "expires_at": "2024-01-01T12:00:00Z",
  "share_url": "https://domain.com/secret/uuid"
}
```

### `POST /api/secrets/:id`
Retrieve a secret.

**Request**:
```json
{
  "passphrase": "word1-word2-word3"
}
```

**Response (success)**:
```json
{
  "secret": "encrypted_base64_data",
  "views_remaining": 0,
  "extendable": true,
  "expires_at": "2024-01-01T12:00:00Z"
}
```

**Response (wrong passphrase)**: `401 Unauthorized`

**Response (not found/deleted)**: `404 Not Found`

### `POST /api/secrets/:id/extend`
Extend a secret with more views or days.

**Request**:
```json
{
  "passphrase": "word1-word2-word3",
  "add_days": 7,
  "add_views": 5
}
```

**Response**:
```json
{
  "expires_at": "2024-01-08T12:00:00Z",
  "max_views": 10,
  "views": 2
}
```

## Environment Variables

```bash
# Required
DATABASE_URL=postgres://user:password@host:5432/dbname
BASE_URL=https://your-domain.com

# Optional
PORT=3000                    # Default: 3000
RUST_LOG=info               # Log level
MAX_SECRET_DAYS=30          # Max days for a secret
MAX_SECRET_VIEWS=100        # Max views allowed
MAX_FAILED_ATTEMPTS=10      # Max failed passwords for unlimited secrets
```

## Deployment Options

### 1. Docker Compose (Development/Testing)

The simplest way to run the project. See [docker-compose.yml](docker-compose.yml).

```bash
docker-compose up -d
```

**Advantages**:
- No manual setup
- All components in one file
- Good for local development

### 2. Kubernetes (Production)

See [infra/kubernetes.yaml](infra/kubernetes.yaml) for manifests.

**Advantages**:
- Horizontal scaling (HPA)
- Auto-healing
- Rolling updates
- Service discovery

### 3. AWS Lambda (Serverless)

See [infra/serverless.yml](infra/serverless.yml).

**Advantages**:
- No server management
- Auto-scaling
- Pay-per-use

## Development

### Make Commands

```bash
make help              # Show all commands
make dev               # Start development environment
make test              # Run all tests
make test-e2e          # Run E2E tests
make build-backend     # Build backend (release)
make build-frontend    # Build frontend
```

See [Makefile](Makefile) for complete list.

## Security Best Practices

### 1. Network Security
- TLS/HTTPS required
- Firewall rules (allow only HTTPS)
- Private subnets for database

### 2. Application Security
- CORS configured
- Input validation
- SQL injection protected (SQLx)
- Brute force protection

### 3. Data Security
- Encrypted at rest (database encryption)
- Encrypted in transit (TLS)
- No plaintext storage
- Automatic cleanup of expired data
