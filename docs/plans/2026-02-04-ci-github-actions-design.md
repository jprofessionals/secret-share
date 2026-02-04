# CI GitHub Actions Design

## Overview

A GitHub Actions workflow with three parallel jobs that run all tests (backend, frontend, and E2E) on pushes to `main` and on pull requests.

## Trigger Events

```yaml
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
```

## Job Structure

### Job 1: `test-backend`

- **Runs on**: `ubuntu-latest`
- **Purpose**: Run Rust unit tests and integration tests
- **Steps**:
  1. Checkout code
  2. Install Rust toolchain (stable)
  3. Cache Cargo dependencies and build artifacts
  4. Run `cargo test` in `backend/` directory

The integration tests use testcontainers to spin up PostgreSQL automatically.

### Job 2: `test-frontend`

- **Runs on**: `ubuntu-latest`
- **Purpose**: Run Vitest unit tests
- **Steps**:
  1. Checkout code
  2. Setup Node.js 20 LTS with npm cache
  3. Run `npm ci` in `frontend/`
  4. Run `npm test` in `frontend/`

### Job 3: `test-e2e`

- **Runs on**: `ubuntu-latest`
- **Purpose**: Run Playwright browser tests against full stack
- **Steps**:
  1. Checkout code
  2. Install Rust toolchain (stable) with cache
  3. Setup Node.js 20 LTS with npm cache
  4. Install frontend dependencies (`npm ci` in `frontend/`)
  5. Install E2E dependencies (`npm ci` in `e2e/`)
  6. Install Playwright browsers
  7. Run `npm test` in `e2e/`

The E2E test setup (via `global-setup.ts`) handles:
- Starting PostgreSQL via testcontainers
- Building and starting the backend (release mode)
- Building and starting the frontend preview server

## Caching Strategy

| Component | Cache Action | What's Cached |
|-----------|--------------|---------------|
| Rust | `Swatinem/rust-cache@v2` | `~/.cargo`, `target/` |
| Node.js | `actions/setup-node` cache option | `~/.npm` |
| Playwright | `actions/cache` | `~/.cache/ms-playwright` |

## File Location

`.github/workflows/ci.yml`
