# Quick Start Guide

Get started with SecretShare in 5 minutes!

## Option 1: Docker Compose (Recommended)

The fastest way to get everything up and running - no manual setup required.

```bash
# 1. Clone repository
git clone <repo-url>
cd secret-share

# 2. Start all services
docker-compose up -d

# 3. Open browser
open http://localhost
```

That's it! The service is now running at:
- **Frontend**: http://localhost
- **Backend API**: http://localhost:3000
- **Database**: localhost:5432

To stop:
```bash
docker-compose down
```

## Option 2: Make (for development)

Use `make` for local development with hot-reload.

### Prerequisites
- Docker (for PostgreSQL)
- Rust 1.75+
- Node.js 20+

### Start Development Environment

```bash
# Show all available commands
make help

# Start everything (PostgreSQL + backend + frontend)
make dev
```

Frontend starts at http://localhost:5173, backend at http://localhost:3000.

Press `Ctrl+C` to stop everything.

### Important Make Commands

| Command | Description |
|---------|-------------|
| `make help` | Show all commands |
| `make dev` | Start development environment |
| `make dev-stop` | Stop development environment manually |
| `make test` | Run all tests |
| `make test-backend` | Backend tests only |
| `make test-frontend` | Frontend tests only |
| `make test-e2e` | Run E2E tests |
| `make build-backend` | Build backend (release) |
| `make build-frontend` | Build frontend |

See [Makefile](Makefile) for complete list.

## Option 3: Manual Setup

### 1. Start PostgreSQL

```bash
docker run -d --name secretshare-db \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=secretshare \
  -p 5432:5432 \
  postgres:16-alpine
```

### 2. Start Backend

```bash
cd backend

# Create .env
cat > .env << EOF
DATABASE_URL=postgres://postgres:postgres@localhost/secretshare
BASE_URL=http://localhost:5173
PORT=3000
RUST_LOG=info,secret_share_backend=debug
EOF

# Run
cargo run
```

### 3. Start Frontend

```bash
cd frontend

# Install dependencies
npm install

# Create .env
echo "VITE_API_URL=http://localhost:3000" > .env

# Run
npm run dev
```

## Test That It Works

1. Open http://localhost (Docker) or http://localhost:5173 (Make/manual)
2. Enter a test secret
3. Click "Create Secret"
4. Copy the link and passphrase
5. Open the link in a new tab
6. Paste the passphrase
7. Verify that the secret is displayed!

## Common Problems

### Backend Can't Connect to Database

```bash
# Check that PostgreSQL is running
docker ps | grep postgres

# Check database connection
psql -h localhost -U postgres -d secretshare
```

### Frontend Can't Reach Backend

```bash
# Check that backend is running
curl http://localhost:3000/health
# Should return: OK
```

### Port Already in Use

Change port in `.env` file or `docker-compose.yml`:
- Backend: `PORT=3001`
- Frontend: Change in `vite.config.ts`

## Next Steps

- Read [README.md](README.md) for full documentation
- Read [ARCHITECTURE.md](ARCHITECTURE.md) for architecture details
- Read [CLAUDE.md](CLAUDE.md) for developer reference
- See [docker-compose.yml](docker-compose.yml) for production setup
