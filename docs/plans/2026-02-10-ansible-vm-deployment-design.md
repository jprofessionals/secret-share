# Ansible VM Deployment Design

## Summary

Replace the AWS Lambda/SAM deploy job in GitHub Actions with an Ansible playbook that deploys SecretShare to a dev VM. The backend runs as a Docker container, the frontend is served as static files by Caddy, and PostgreSQL (already provisioned by ansible-controll) is the database.

Lambda, DynamoDB, and SAM infrastructure remain in the repo as an alternative deployment path. E2E tests switch back to PostgreSQL.

## Architecture

### Traffic Flow

```
User → https://secret.jpro.dev → Caddy (port 443)
  ├── /api/*  → reverse_proxy localhost:3000 (backend Docker container)
  └── /*      → file_server /var/www/secret-share (static frontend)
```

### VM Infrastructure (from ansible-controll)

Already provisioned:
- PostgreSQL with `secret_share_db` database, `secret_share` user (vault-encrypted password)
- Docker engine (deploy user in docker group)
- Caddy reverse proxy (per-project configs in `/etc/caddy/sites/`)
- `deploy` user with passwordless `sudo systemctl reload caddy`
- Domain: `jpro.dev`

## Ansible Playbook

### File Structure

```
infra/ansible/
├── deploy.yml       # Deployment playbook
├── inventory.yml    # Dev VM host
└── ansible.cfg      # Minimal config
```

### inventory.yml

```yaml
all:
  hosts:
    devserver:
      ansible_host: "{{ lookup('env', 'VM_HOST') }}"
      ansible_user: deploy
```

### deploy.yml Tasks (in order)

1. Clone/pull the repo to `/home/deploy/secret-share/`
2. Build backend Docker image from `backend/Dockerfile`
3. Stop existing backend container (if running)
4. Start new backend container on `localhost:3000` with environment variables
5. Build frontend (`npm ci && npm run build` with `VITE_API_URL=""`)
6. Copy `frontend/dist/` to `/var/www/secret-share/`
7. Write Caddy config to `/etc/caddy/sites/secret-share.caddy`
8. Reload Caddy

### Secrets Management

The playbook loads vault-encrypted variables from the ansible-controll project:

```yaml
vars_files:
  - <path-to>/ansible-controll/inventory/group_vars/all.yml
```

This provides `vault_secret_share_db_password`, used to construct:

```
DATABASE_URL=postgres://secret_share:{{ vault_secret_share_db_password }}@127.0.0.1:5432/secret_share_db
```

### Backend Container Environment

| Variable | Value |
|----------|-------|
| `DATABASE_URL` | `postgres://secret_share:<vault_password>@host.docker.internal:5432/secret_share_db` |
| `BASE_URL` | `https://secret.jpro.dev` |
| `PORT` | `3000` |
| `RUST_LOG` | `info,secret_share_backend=debug` |
| `MAX_SECRET_DAYS` | `30` |
| `MAX_SECRET_VIEWS` | `100` |
| `MAX_FAILED_ATTEMPTS` | `10` |

Note: `host.docker.internal` or `--network host` used so the container can reach PostgreSQL on the host.

### Caddy Site Config

Written to `/etc/caddy/sites/secret-share.caddy`:

```caddy
secret.jpro.dev {
    handle /api/* {
        reverse_proxy localhost:3000
    }
    handle {
        root * /var/www/secret-share
        try_files {path} /index.html
        file_server
    }
}
```

### VM File Layout

- `/home/deploy/secret-share/` - cloned repo (build workspace)
- `/var/www/secret-share/` - frontend static files (served by Caddy)

## GitHub Actions Changes

### E2E Tests: DynamoDB → PostgreSQL

Change `E2E_DATABASE` from `dynamodb` to `postgres`:

```yaml
test-e2e:
  name: E2E Tests (PostgreSQL)    # was: E2E Tests (DynamoDB)
  # ...
  - name: Run E2E tests
    env:
      E2E_DATABASE: postgres       # was: dynamodb
```

The E2E test infrastructure already supports PostgreSQL via testcontainers.

### Deploy Job: SAM → Ansible

Replace the SAM deploy job. Same conditions: runs on push to main, after all tests pass, production environment, single concurrency.

```yaml
deploy:
  name: Deploy to Dev VM
  needs: [test-backend-postgres, test-backend-dynamodb, test-frontend, test-e2e]
  if: github.ref == 'refs/heads/main' && github.event_name == 'push'
  runs-on: ubuntu-latest
  environment: production
  concurrency:
    group: deploy-production
    cancel-in-progress: false
  steps:
    - uses: actions/checkout@v4
    - uses: actions/checkout@v4
      with:
        repository: <org>/ansible-controll
        path: ansible-controll
        token: ${{ secrets.ANSIBLE_CONTROLL_PAT }}
    - name: Install Ansible
      run: pip install ansible
    - name: Set up SSH key
      run: |
        mkdir -p ~/.ssh
        echo "${{ secrets.DEPLOY_SSH_KEY }}" > ~/.ssh/id_ed25519
        chmod 600 ~/.ssh/id_ed25519
        ssh-keyscan -H ${{ vars.VM_HOST }} >> ~/.ssh/known_hosts
    - name: Set up Vault password
      run: echo "${{ secrets.VAULT_PASSWORD }}" > .vault_password
    - name: Run deploy playbook
      run: >
        ansible-playbook infra/ansible/deploy.yml
        --vault-password-file .vault_password
      env:
        VM_HOST: ${{ vars.VM_HOST }}
```

### Required GitHub Secrets/Variables

| Name | Type | Description |
|------|------|-------------|
| `DEPLOY_SSH_KEY` | Secret | Deploy user SSH private key (from ansible-controll setup) |
| `VAULT_PASSWORD` | Secret | Ansible Vault decryption password |
| `ANSIBLE_CONTROLL_PAT` | Secret | GitHub PAT to clone ansible-controll repo |
| `VM_HOST` | Variable | Dev VM IP address or hostname |

## What Stays, What Changes

### Kept (no changes)

- `infra/sam/` - SAM template and config
- `backend/src/bin/lambda.rs` - Lambda entry point
- `backend/src/db/dynamodb.rs` - DynamoDB support
- DynamoDB backend test job in CI
- All feature flags (`lambda`, `dynamodb-tests`, `postgres-tests`)

### Changed

- CI deploy job: SAM deploy replaced with Ansible deploy
- E2E tests: `E2E_DATABASE` switched from `dynamodb` to `postgres`
- E2E test job name: "E2E Tests (PostgreSQL)"
- CLAUDE.md: add Ansible deploy commands

### New files

- `infra/ansible/deploy.yml`
- `infra/ansible/inventory.yml`
- `infra/ansible/ansible.cfg`

### Removed

- SAM deploy job from `.github/workflows/ci.yml` (SAM files stay, just not auto-deployed)
