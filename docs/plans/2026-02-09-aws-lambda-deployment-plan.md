# AWS Lambda Deployment Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deploy the SecretShare backend on AWS Lambda with the frontend on S3, unified behind CloudFront, with automated CD via GitHub Actions.

**Architecture:** CloudFront routes `/api/*` to API Gateway → Lambda (Rust/Axum) → DynamoDB, and all other paths to S3 (static SvelteKit build). SAM manages all infrastructure.

**Tech Stack:** Rust, lambda_http, cargo-lambda, AWS SAM, CloudFront, S3, API Gateway, DynamoDB, GitHub Actions

**Design doc:** `docs/plans/2026-02-09-aws-lambda-deployment-design.md`

---

### Task 1: Extract `build_app` from `run()` in lib.rs

**Files:**
- Modify: `backend/src/lib.rs`

**Step 1: Refactor `run()` to extract app construction**

Split the `run()` function into two parts:
- `build_app(config: Config) -> anyhow::Result<(Router, Config)>` — creates the DB connection, AppState, and Router
- `run(config: Config)` — calls `build_app()` then binds to TCP

```rust
pub async fn build_app(config: Config) -> anyhow::Result<(Router, AppState)> {
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
        config: Arc::new(config),
    };

    let app = create_router(state.clone());
    Ok((app, state))
}

pub async fn run(config: Config) -> anyhow::Result<()> {
    let port = config.port;
    let (app, _state) = build_app(config).await?;

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

**Step 2: Verify existing tests still pass**

Run: `cd backend && cargo test --features postgres-tests 2>&1 | tail -5`
Expected: All tests pass (the refactor doesn't change behavior)

**Step 3: Commit**

```bash
git add backend/src/lib.rs
git commit -m "refactor: extract build_app() from run() for Lambda reuse"
```

---

### Task 2: Add lambda_http dependency and feature flag

**Files:**
- Modify: `backend/Cargo.toml`

**Step 1: Add lambda_http as optional dependency and lambda feature flag**

Add to `[dependencies]`:
```toml
# AWS Lambda runtime (optional, for serverless deployment)
lambda_http = { version = "0.14", optional = true }
```

Add to `[features]`:
```toml
lambda = ["lambda_http"]
```

**Step 2: Verify it compiles without the feature**

Run: `cd backend && cargo check`
Expected: Compiles successfully (lambda_http not included by default)

**Step 3: Verify it compiles with the feature**

Run: `cd backend && cargo check --features lambda`
Expected: Compiles successfully with lambda_http

**Step 4: Commit**

```bash
git add backend/Cargo.toml
git commit -m "feat: add lambda_http dependency behind lambda feature flag"
```

---

### Task 3: Create Lambda binary entry point

**Files:**
- Create: `backend/src/bin/lambda.rs`

**Step 1: Create the Lambda binary**

```rust
use secret_share_backend::{build_app, config::Config};

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    lambda_http::tracing::init_default_subscriber();

    tracing::info!("Starting Secret Share Lambda");

    let config = Config::from_env()
        .expect("Failed to load config");

    let (app, _state) = build_app(config).await
        .expect("Failed to build app");

    lambda_http::run(app).await
}
```

**Step 2: Verify it compiles**

Run: `cd backend && cargo check --features lambda --bin lambda`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add backend/src/bin/lambda.rs
git commit -m "feat: add Lambda binary entry point"
```

---

### Task 4: Create SAM template

**Files:**
- Create: `infra/sam/template.yaml`

**Step 1: Create the SAM template**

```yaml
AWSTemplateFormatVersion: '2010-09-09'
Transform: AWS::Serverless-2016-10-31
Description: SecretShare - Secure secret sharing with end-to-end encryption

Parameters:
  Stage:
    Type: String
    Default: prod
  MaxSecretDays:
    Type: Number
    Default: 30
  MaxSecretViews:
    Type: Number
    Default: 100
  MaxFailedAttempts:
    Type: Number
    Default: 10

Globals:
  Function:
    Timeout: 30
    MemorySize: 256

Resources:
  # DynamoDB Table
  SecretsTable:
    Type: AWS::DynamoDB::Table
    Properties:
      TableName: !Sub 'secretshare-${Stage}'
      BillingMode: PAY_PER_REQUEST
      AttributeDefinitions:
        - AttributeName: id
          AttributeType: S
      KeySchema:
        - AttributeName: id
          KeyType: HASH
      TimeToLiveSpecification:
        AttributeName: expires_at
        Enabled: true

  # Lambda Function
  SecretShareFunction:
    Type: AWS::Serverless::Function
    Metadata:
      BuildMethod: rust-cargolambda
      BuildProperties:
        Binary: lambda
        CargoLambdaFlags:
          - '--features'
          - 'lambda'
    Properties:
      CodeUri: ../../backend
      Handler: bootstrap
      Runtime: provided.al2023
      Architectures:
        - arm64
      Environment:
        Variables:
          DYNAMODB_TABLE: !Ref SecretsTable
          BASE_URL: !Sub 'https://${CloudFrontDistribution.DomainName}'
          MAX_SECRET_DAYS: !Ref MaxSecretDays
          MAX_SECRET_VIEWS: !Ref MaxSecretViews
          MAX_FAILED_ATTEMPTS: !Ref MaxFailedAttempts
          RUST_LOG: info
      Policies:
        - DynamoDBCrudPolicy:
            TableName: !Ref SecretsTable
      Events:
        ApiCatchAll:
          Type: HttpApi
          Properties:
            ApiId: !Ref HttpApi
            Path: /{proxy+}
            Method: ANY
        ApiRoot:
          Type: HttpApi
          Properties:
            ApiId: !Ref HttpApi
            Path: /
            Method: ANY

  # HTTP API Gateway
  HttpApi:
    Type: AWS::Serverless::HttpApi
    Properties:
      StageName: !Ref Stage

  # S3 Bucket for frontend
  FrontendBucket:
    Type: AWS::S3::Bucket
    Properties:
      BucketName: !Sub 'secretshare-frontend-${Stage}'
      PublicAccessBlockConfiguration:
        BlockPublicAcls: true
        BlockPublicPolicy: true
        IgnorePublicAcls: true
        RestrictPublicBuckets: true

  # S3 Bucket Policy for CloudFront OAC
  FrontendBucketPolicy:
    Type: AWS::S3::BucketPolicy
    Properties:
      Bucket: !Ref FrontendBucket
      PolicyDocument:
        Version: '2012-10-17'
        Statement:
          - Sid: AllowCloudFrontServicePrincipal
            Effect: Allow
            Principal:
              Service: cloudfront.amazonaws.com
            Action: s3:GetObject
            Resource: !Sub '${FrontendBucket.Arn}/*'
            Condition:
              StringEquals:
                AWS:SourceArn: !Sub 'arn:aws:cloudfront::${AWS::AccountId}:distribution/${CloudFrontDistribution}'

  # CloudFront Origin Access Control
  CloudFrontOAC:
    Type: AWS::CloudFront::OriginAccessControl
    Properties:
      OriginAccessControlConfig:
        Name: !Sub 'secretshare-oac-${Stage}'
        OriginAccessControlOriginType: s3
        SigningBehavior: always
        SigningProtocol: sigv4

  # CloudFront Distribution
  CloudFrontDistribution:
    Type: AWS::CloudFront::Distribution
    Properties:
      DistributionConfig:
        Enabled: true
        DefaultRootObject: index.html
        HttpVersion: http2

        Origins:
          # S3 origin for frontend
          - Id: S3Origin
            DomainName: !GetAtt FrontendBucket.RegionalDomainName
            OriginAccessControlId: !Ref CloudFrontOAC
            S3OriginConfig:
              OriginAccessIdentity: ''

          # API Gateway origin
          - Id: ApiOrigin
            DomainName: !Sub '${HttpApi}.execute-api.${AWS::Region}.amazonaws.com'
            CustomOriginConfig:
              HTTPSPort: 443
              OriginProtocolPolicy: https-only
            OriginPath: !Sub '/${Stage}'

        DefaultCacheBehavior:
          TargetOriginId: S3Origin
          ViewerProtocolPolicy: redirect-to-https
          CachePolicyId: 658327ea-f89d-4fab-a63d-7e88639e58f6  # CachingOptimized
          Compress: true

        CacheBehaviors:
          - PathPattern: /api/*
            TargetOriginId: ApiOrigin
            ViewerProtocolPolicy: redirect-to-https
            CachePolicyId: 4135ea2d-6df8-44a3-9df3-4b5a84be39ad  # CachingDisabled
            OriginRequestPolicyId: b689b0a8-53d0-40ab-baf2-68738e2966ac  # AllViewerExceptHostHeader
            AllowedMethods:
              - GET
              - HEAD
              - OPTIONS
              - PUT
              - PATCH
              - POST
              - DELETE

        # SPA fallback: serve index.html for 403/404 from S3
        CustomErrorResponses:
          - ErrorCode: 403
            ResponseCode: 200
            ResponsePagePath: /index.html
            ErrorCachingMinTTL: 0
          - ErrorCode: 404
            ResponseCode: 200
            ResponsePagePath: /index.html
            ErrorCachingMinTTL: 0

Outputs:
  CloudFrontUrl:
    Description: CloudFront distribution URL
    Value: !Sub 'https://${CloudFrontDistribution.DomainName}'
  ApiUrl:
    Description: API Gateway URL
    Value: !Sub 'https://${HttpApi}.execute-api.${AWS::Region}.amazonaws.com/${Stage}'
  FrontendBucketName:
    Description: S3 bucket name for frontend
    Value: !Ref FrontendBucket
  CloudFrontDistributionId:
    Description: CloudFront distribution ID (for cache invalidation)
    Value: !Ref CloudFrontDistribution
```

**Step 2: Validate the template**

Run: `cd infra/sam && sam validate --lint`
Expected: Template is valid

**Step 3: Commit**

```bash
git add infra/sam/template.yaml
git commit -m "feat: add SAM template for Lambda + CloudFront + S3 + DynamoDB"
```

---

### Task 5: Create SAM config file

**Files:**
- Create: `infra/sam/samconfig.toml`

**Step 1: Create samconfig.toml**

```toml
version = 0.1

[default.global.parameters]
stack_name = "secretshare"
region = "eu-west-1"

[default.build.parameters]
beta_features = true

[default.deploy.parameters]
capabilities = "CAPABILITY_IAM"
confirm_changeset = true
resolve_s3 = true
```

**Step 2: Commit**

```bash
git add infra/sam/samconfig.toml
git commit -m "feat: add SAM deployment config"
```

---

### Task 6: Add deploy job to GitHub Actions CI workflow

**Files:**
- Modify: `.github/workflows/ci.yml`

**Step 1: Add the deploy job after the existing test jobs**

Append to the end of the `jobs:` section in `.github/workflows/ci.yml`:

```yaml
  deploy:
    name: Deploy to AWS
    needs: [test-backend-postgres, test-backend-dynamodb, test-frontend, test-e2e]
    if: github.ref == 'refs/heads/main' && github.event_name == 'push'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-lambda
        run: pip3 install cargo-lambda

      - name: Cache Cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: backend -> target

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json

      - name: Setup AWS SAM CLI
        uses: aws-actions/setup-sam@v2

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: ${{ secrets.AWS_REGION }}

      - name: Build frontend
        working-directory: frontend
        run: npm ci && npm run build
        env:
          VITE_API_URL: ''

      - name: SAM build
        working-directory: infra/sam
        run: sam build

      - name: SAM deploy
        working-directory: infra/sam
        run: sam deploy --no-confirm-changeset --no-fail-on-empty-changeset

      - name: Get stack outputs
        id: stack
        working-directory: infra/sam
        run: |
          echo "bucket=$(aws cloudformation describe-stacks --stack-name secretshare --query 'Stacks[0].Outputs[?OutputKey==`FrontendBucketName`].OutputValue' --output text)" >> "$GITHUB_OUTPUT"
          echo "distribution_id=$(aws cloudformation describe-stacks --stack-name secretshare --query 'Stacks[0].Outputs[?OutputKey==`CloudFrontDistributionId`].OutputValue' --output text)" >> "$GITHUB_OUTPUT"

      - name: Deploy frontend to S3
        run: aws s3 sync frontend/build/ "s3://${{ steps.stack.outputs.bucket }}" --delete

      - name: Invalidate CloudFront cache
        run: aws cloudfront create-invalidation --distribution-id "${{ steps.stack.outputs.distribution_id }}" --paths "/*"
```

**Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "feat: add CD deploy job to GitHub Actions workflow"
```

---

### Task 7: Update CLAUDE.md with SAM commands

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Add SAM commands to the Common Commands section**

Add after the existing "Deployment" section in the `## Common Commands` block:

```markdown
# AWS Serverless (SAM)
cd infra/sam && sam build     # Build Lambda function
cd infra/sam && sam deploy    # Deploy stack to AWS
cd infra/sam && sam validate  # Validate SAM template
cd infra/sam && sam delete    # Delete the deployed stack
```

**Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: add SAM commands to CLAUDE.md"
```

---

### Summary of tasks

| # | Task | Files | Estimated |
|---|------|-------|-----------|
| 1 | Extract `build_app()` from `run()` | `backend/src/lib.rs` | 5 min |
| 2 | Add lambda_http dependency + feature flag | `backend/Cargo.toml` | 2 min |
| 3 | Create Lambda binary entry point | `backend/src/bin/lambda.rs` | 3 min |
| 4 | Create SAM template | `infra/sam/template.yaml` | 5 min |
| 5 | Create SAM config | `infra/sam/samconfig.toml` | 2 min |
| 6 | Add deploy job to CI workflow | `.github/workflows/ci.yml` | 5 min |
| 7 | Update CLAUDE.md | `CLAUDE.md` | 2 min |
