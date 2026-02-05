.PHONY: help dev dev-stop build-backend build-frontend test test-backend test-backend-postgres test-backend-dynamodb test-frontend clean test-e2e test-e2e-api test-e2e-playwright test-e2e-playwright-headed test-e2e-playwright-ui cleanup

help: ## Vis denne hjelpemeldingen
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

dev: ## Start lokal utviklingsmiljø (Ctrl+C stopper alt)
	@echo "Starting PostgreSQL..."
	@docker run -d --name secretshare-postgres \
		-e POSTGRES_PASSWORD=postgres \
		-e POSTGRES_DB=secretshare \
		-p 5432:5432 \
		postgres:16-alpine 2>/dev/null || true
	@echo "Waiting for PostgreSQL to be ready..."
	@until docker exec secretshare-postgres pg_isready -U postgres > /dev/null 2>&1; do sleep 1; done
	@trap 'trap - INT TERM; echo ""; echo "Shutting down..."; kill 0; docker stop secretshare-postgres 2>/dev/null; docker rm secretshare-postgres 2>/dev/null; exit 0' INT TERM; \
		echo "Starting backend..."; \
		(cd backend && cargo run) & \
		sleep 2; \
		echo "Starting frontend..."; \
		(cd frontend && npm run dev) & \
		wait

dev-stop: ## Stopp utviklingsmiljø manuelt
	@echo "Stopping services..."
	@-pkill -f "cargo run" 2>/dev/null || true
	@-pkill -f "vite" 2>/dev/null || true
	@-docker stop secretshare-postgres 2>/dev/null || true
	@-docker rm secretshare-postgres 2>/dev/null || true
	@echo "Done."

build-backend: ## Bygg backend (Rust)
	cd backend && cargo build --release

build-frontend: ## Bygg frontend (Svelte)
	cd frontend && npm run build

test-backend: ## Kjør backend tester
	cd backend && cargo test

test-backend-postgres: ## Kjør backend PostgreSQL integrasjonstester
	cd backend && cargo test --features postgres-tests

test-backend-dynamodb: ## Kjør backend DynamoDB integrasjonstester
	cd backend && cargo test --features dynamodb-tests -- --test-threads=1

test-frontend: ## Kjør frontend tester
	cd frontend && npm test

test: test-backend test-frontend ## Kjør alle tester

clean: ## Rydd opp build artifacts
	cd backend && cargo clean
	cd frontend && rm -rf dist node_modules

# E2E Testing
test-e2e: test-e2e-api test-e2e-playwright ## Kjør alle E2E tester

test-e2e-api: ## Kjør Rust API integrasjonstester
	cd backend && cargo test --test api_create_secret --test api_retrieve_secret --test api_edge_cases --test cleanup_expired -- --nocapture

test-e2e-playwright: ## Kjør Playwright browser tester
	cd e2e && npm test

test-e2e-playwright-headed: ## Kjør Playwright tester med synlig browser
	cd e2e && npm run test:headed

test-e2e-playwright-ui: ## Åpne Playwright UI mode
	cd e2e && npm run test:ui

# Maintenance
cleanup: ## Kjør opprydding av utløpte hemmeligheter
	cd backend && cargo run --bin cleanup

# Database migrations
migrate: ## Kjør database migrasjoner via CLI
	cd backend && sqlx migrate run

migrate-add: ## Opprett ny migrasjon (bruk: make migrate-add name=add_foo)
	cd backend && sqlx migrate add $(name)
