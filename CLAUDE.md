# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SecretShare is a secure secret-sharing service with end-to-end encryption. Users create secrets that are encrypted client-side before being stored, and recipients decrypt them locally using a passphrase. The backend never sees plaintext secrets.

## Tech Stack

- **Backend**: Rust with Axum, SQLx (PostgreSQL), AES-256-GCM encryption, Argon2 key derivation
- **Frontend**: Svelte 5 with SvelteKit 2, TypeScript, Vite 6, TailwindCSS 4
- **Database**: PostgreSQL 16 or DynamoDB

## Common Commands

```bash
# Development
make dev                 # Start full dev environment (PostgreSQL + backend + frontend)
make docker-up           # Start Docker Compose stack
make docker-down         # Stop Docker Compose stack
make docker-logs         # Stream Docker logs

# Building
make build-backend       # Build Rust backend (release mode)
make build-frontend      # Build Svelte frontend
make docker-build        # Build both Docker images

# Testing
make test                # Run all tests
make test-backend        # Run Rust tests only (cargo test)
make test-frontend       # Run frontend tests only (npm test)
cd backend && cargo test crypto  # Run specific test module

# Maintenance
make cleanup             # Run cleanup of expired secrets (or: cd backend && cargo run --bin cleanup)

# Database migrations
make migrate             # Run migrations via sqlx-cli
make migrate-add name=x  # Create new migration file

# Deployment
make k8s-deploy          # Deploy to Kubernetes
make k8s-status          # Check Kubernetes status
```

## Architecture

### Encryption Flow

1. Frontend generates a 3-word BIP39 passphrase (~33 bits entropy)
2. Passphrase is converted to a 256-bit key using Argon2id (64MB memory, 3 iterations)
3. Secret is encrypted with AES-256-GCM before sending to backend
4. Backend stores only the encrypted blob
5. Recipient uses passphrase to decrypt locally

### Encrypted Data Format

`[Salt(16 bytes) || Nonce(12 bytes) || Ciphertext || Auth Tag]`

### API Endpoints

- `POST /api/secrets` - Create secret (accepts encrypted data, returns id + passphrase)
- `POST /api/secrets/:id` - Retrieve secret (requires passphrase)
- `POST /api/secrets/:id/extend` - Extend secret lifetime/views (requires passphrase)
- `GET /health` - Health check

### Brute Force Protection

Wrong passphrase attempts are tracked to prevent brute force attacks:
- First 2 wrong attempts: No penalty (free attempts)
- 3rd+ wrong attempts: Each consumes a view
- When views are depleted: Secret is deleted
- For unlimited views secrets: Deleted after `MAX_FAILED_ATTEMPTS` (default: 10)
- Successful retrieval resets the failed attempts counter

### Key Source Files

- `backend/src/routes/secrets.rs` - HTTP route handlers
- `backend/src/services/secrets.rs` - Business logic (create, retrieve, extend)
- `backend/src/crypto/` - Encryption/decryption logic, passphrase generation
- `backend/src/db/` - Database operations
- `backend/migrations/` - SQLx migration files (run at startup)
- `backend/src/models/` - Data structures for secrets and API requests/responses
- `frontend/src/routes/create/CreateSecret.svelte` - Client-side encryption UI
- `frontend/src/routes/secret/[id]/ViewSecret.svelte` - Client-side decryption UI
- `backend/src/bin/cleanup.rs` - CLI for cleanup cron job

## Environment Variables

### Backend

- `DATABASE_URL` - PostgreSQL connection string
- `BASE_URL` - Public URL for share links
- `PORT` - Server port (default: 3000)
- `RUST_LOG` - Log level (e.g., `debug`, `info`)
- `MAX_SECRET_DAYS` - Maximum days a secret can exist (default: 30)
- `MAX_SECRET_VIEWS` - Maximum view count allowed (default: 100)
- `MAX_FAILED_ATTEMPTS` - Max wrong passphrase attempts for unlimited views secrets (default: 10)

### DynamoDB (alternative to PostgreSQL)

- `DYNAMODB_TABLE` - DynamoDB table name (if set, uses DynamoDB instead of PostgreSQL)
- `DYNAMODB_ENDPOINT` - Optional endpoint URL for DynamoDB Local
- AWS credentials via standard SDK chain (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, AWS_REGION, or IAM role)

### Frontend

- `VITE_API_URL` - Backend API URL

## Deployment Options

1. **Docker Compose** - `docker-compose.yml` for local/testing
2. **Kubernetes** - `infra/kubernetes.yaml` with HPA, Ingress, StatefulSet
3. **AWS Lambda** - `infra/serverless.yml` for serverless deployment
