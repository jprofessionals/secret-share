# SecretShare

A secure service for sharing secrets (passwords, API keys, etc.) with end-to-end encryption and self-destructing messages.

## Features

- **End-to-end encryption**: Secrets are encrypted client-side with AES-256-GCM
- **3-word passphrase**: Simple and secure sharing with three random words from the BIP39 wordlist
- **Self-destructing**: Secrets are automatically deleted after:
  - Maximum number of views
  - Expiration time (hours/days)
- **Brute force protection**: Failed password attempts count against the view limit
- **Extendable lifetime**: Secrets can be extended with more views/days
- **12-factor app**: Configured for cloud deployment
- **Scalable**: Supports Lambda, Kubernetes, and VM deployment

## Getting Started

### Quickest Path: Docker Compose

```bash
# Clone and start
git clone <repo-url>
cd secret-share
docker-compose up -d

# Open in browser
open http://localhost
```

Done! The service is now running at:
- **Frontend**: http://localhost
- **Backend API**: http://localhost:3000

### Local Development with Make

```bash
# Show all available commands
make help

# Start development environment (PostgreSQL + backend + frontend)
make dev

# Run tests
make test

# Run E2E tests
make test-e2e
```

See [Makefile](Makefile) for all available commands.

## Architecture

### Backend (Rust)
- **Framework**: Axum (async web framework)
- **Database**: PostgreSQL with SQLx
- **Encryption**: AES-256-GCM via aes-gcm crate
- **Key derivation**: Argon2 for passphrase to encryption key

### Frontend (Svelte)
- **Framework**: Svelte 5 with SvelteKit 2 and TypeScript
- **Styling**: TailwindCSS 4
- **Build**: Vite 6

### Source Code Structure

```
backend/
  src/
    routes/      # HTTP route handlers
    services/    # Business logic
    crypto/      # Encryption and passphrase
    db/          # Database operations
    models/      # Data structures
frontend/
  src/
    routes/      # Svelte pages
    lib/         # Shared components
```

## Security

### Encryption
- **Algorithm**: AES-256-GCM (authenticated encryption)
- **Key derivation**: Argon2id (memory-hard function)
- **Passphrase**: 3 random words from BIP39 wordlist (~33 bits entropy)

### Brute Force Protection
- First 2 failed password attempts: No penalty
- 3+ failed attempts: Each attempt uses one view
- When views are exhausted: Secret is deleted
- For unlimited views: Deleted after `MAX_FAILED_ATTEMPTS` (default: 10)

### Best Practices
- **Channel separation**: Share link and passphrase via different channels
- **Short lifetimes**: Set short expiration times for sensitive secrets
- **HTTPS**: Always use TLS in production

## Configuration

### Backend Environment Variables

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

### Frontend Environment Variables

```bash
VITE_API_URL=https://api.your-domain.com
```

## Testing

```bash
# All tests
make test

# Backend only
make test-backend

# Frontend only
make test-frontend

# E2E tests (requires running services)
make test-e2e

# Playwright with UI
make test-e2e-playwright-ui
```

## Deployment

### Docker Compose (recommended for testing)

```bash
docker-compose up -d
```

### Kubernetes

See [infra/kubernetes.yaml](infra/kubernetes.yaml) for manifests.

```bash
kubectl apply -f infra/kubernetes.yaml
```

### AWS Lambda

See [infra/serverless.yml](infra/serverless.yml) for serverless configuration.

## Documentation

- [QUICKSTART.md](QUICKSTART.md) - Get started in 5 minutes
- [ARCHITECTURE.md](ARCHITECTURE.md) - Detailed architecture
- [CLAUDE.md](CLAUDE.md) - Developer reference

## License

MIT License - see LICENSE file for details
