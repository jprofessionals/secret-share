# AWS Lambda Deployment Design

## Summary

Deploy the SecretShare backend on AWS Lambda behind API Gateway, the frontend as static files on S3, and CloudFront as a unified CDN in front of both. Infrastructure managed by AWS SAM. Continuous deployment via GitHub Actions.

## Architecture

```
                  CloudFront
                  /        \
         /api/*             /*
           |                 |
      API Gateway         S3 Bucket
           |            (static frontend)
      Lambda Function
           |
      DynamoDB Table
```

All traffic enters through a single CloudFront distribution:
- Requests matching `/api/*` route to API Gateway, which invokes the Lambda function.
- All other requests route to the S3 bucket serving the static SvelteKit frontend.

DynamoDB-only for storage. PostgreSQL remains in the codebase for Docker/Kubernetes but is not part of the serverless deployment.

## Backend Changes

### New binary: `backend/src/bin/lambda.rs`

A thin Lambda entry point that:
- Initializes tracing without timestamps (CloudWatch adds them)
- Builds the Axum router via a shared `build_router()` function
- Passes the router to `lambda_http::run()`

### Router extraction refactor

Extract router creation from `run()` in `backend/src/lib.rs` into a `build_router()` function. Both `main.rs` (TCP listener) and `lambda.rs` (Lambda runtime) call this function.

### New Cargo dependencies

- `lambda_http` - Lambda runtime adapter (behind `lambda` feature flag)
- `lambda_runtime` - Core runtime (behind `lambda` feature flag)

### Feature flag: `lambda`

```toml
[features]
default = []
lambda = ["lambda_http", "lambda_runtime"]
```

The `lambda.rs` binary is only compiled with `--features lambda`. Normal `main.rs` remains unchanged.

### No changes to

Routes, services, crypto, database layer, models, or CORS configuration.

## Frontend Changes

None to the code. The frontend already uses `adapter-static` producing static files.

At build time, `VITE_API_URL` is set to `/api` (relative path) since CloudFront serves both frontend and API on the same domain.

## SAM Template: `infra/sam/template.yaml`

### Parameters

- `Stage` - Deployment stage (default: `prod`)
- `MaxSecretDays` - Max days a secret can exist (default: `30`)
- `MaxSecretViews` - Max view count (default: `100`)
- `MaxFailedAttempts` - Max wrong passphrase attempts (default: `10`)

### Resources

**DynamoDB Table**
- Partition key: `id` (String)
- TTL attribute: `expires_at`
- Billing: on-demand (pay-per-request)

**Lambda Function**
- Runtime: `provided.al2023` (custom runtime for Rust)
- Architecture: `arm64` (Graviton, cheaper and faster)
- Memory: 256MB
- Timeout: 30s
- Environment: `DYNAMODB_TABLE`, `BASE_URL`, `MAX_SECRET_DAYS`, `MAX_SECRET_VIEWS`, `MAX_FAILED_ATTEMPTS`
- IAM: DynamoDB read/write on the secrets table

**HTTP API Gateway**
- Catch-all route `/{proxy+}` forwarding to Lambda
- Also root route `/` for health checks

**S3 Bucket**
- Private (no public access)
- Accessed only via CloudFront Origin Access Control

**CloudFront Distribution**
- Default origin: S3 bucket (frontend)
- `/api/*` behavior: API Gateway origin
- Origin Access Control for S3
- Default root object: `index.html`
- Custom error response: 404 → `/index.html` (SPA fallback)

### SAM config: `infra/sam/samconfig.toml`

Stores non-secret deployment defaults: stack name, region, S3 bucket for build artifacts.

## GitHub Actions CD

### Updated workflow: `.github/workflows/ci.yml`

Add a `deploy` job to the existing CI workflow.

```yaml
deploy:
  name: Deploy to AWS
  needs: [test-backend-postgres, test-backend-dynamodb, test-frontend, test-e2e]
  if: github.ref == 'refs/heads/main'
  runs-on: ubuntu-latest
```

### Deploy job steps

1. Checkout code
2. Install Rust toolchain
3. Install cargo-lambda
4. Install Node.js, `npm ci` in `frontend/`
5. Build frontend: `npm run build` with `VITE_API_URL=/api`
6. Build Lambda: `cargo lambda build --release --features lambda --arm64`
7. SAM build & deploy: `sam build && sam deploy --no-confirm-changeset`
8. Sync frontend to S3: `aws s3 sync frontend/build/ s3://$BUCKET --delete`
9. Invalidate CloudFront cache: `aws cloudfront create-invalidation --distribution-id $DIST_ID --paths "/*"`

### Required GitHub secrets

- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_REGION`

Alternatively, use OIDC role assumption for keyless auth.

### Deploy-only on main

The deploy job only runs on pushes to `main` (not PRs), gated behind all four test jobs passing.

## Files to create/modify

| File | Action | Description |
|------|--------|-------------|
| `backend/src/bin/lambda.rs` | Create | Lambda entry point |
| `backend/src/lib.rs` | Modify | Extract `build_router()` from `run()` |
| `backend/Cargo.toml` | Modify | Add lambda deps + feature flag |
| `infra/sam/template.yaml` | Create | SAM template with all resources |
| `infra/sam/samconfig.toml` | Create | SAM deployment defaults |
| `.github/workflows/ci.yml` | Modify | Add deploy job |
| `CLAUDE.md` | Modify | Document SAM commands |
