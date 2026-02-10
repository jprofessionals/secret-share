# Ansible VM Deployment Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the AWS SAM deploy job with an Ansible playbook that deploys SecretShare to a dev VM, and switch E2E tests back to PostgreSQL.

**Architecture:** Backend runs as a Docker container on the dev VM (built from existing Dockerfile). Frontend static files served by Caddy. PostgreSQL database already provisioned by ansible-controll. Caddy reverse proxies `/api/*` to the container and serves static files for everything else.

**Tech Stack:** Ansible, Docker, Caddy, GitHub Actions, PostgreSQL

---

### Task 1: Create Ansible inventory and config

**Files:**
- Create: `infra/ansible/ansible.cfg`
- Create: `infra/ansible/inventory.yml`

**Step 1: Create `infra/ansible/ansible.cfg`**

```ini
[defaults]
inventory = inventory.yml
host_key_checking = False
interpreter_python = /usr/bin/python3
```

Note: No `become` — the deploy user has Docker group access and passwordless sudo for Caddy reload only.

**Step 2: Create `infra/ansible/inventory.yml`**

```yaml
all:
  hosts:
    devserver:
      ansible_host: "{{ lookup('env', 'VM_HOST') | default('changeme', true) }}"
      ansible_user: deploy
```

**Step 3: Commit**

```bash
git add infra/ansible/ansible.cfg infra/ansible/inventory.yml
git commit -m "feat: add Ansible inventory and config for VM deployment"
```

---

### Task 2: Create the deploy playbook

**Files:**
- Create: `infra/ansible/deploy.yml`

**Step 1: Create `infra/ansible/deploy.yml`**

```yaml
---
- name: Deploy SecretShare to dev VM
  hosts: devserver
  vars_files:
    - "{{ ansible_controll_vars | default('../../ansible-controll/inventory/group_vars/all.yml') }}"
  vars:
    app_name: secret-share
    app_repo: https://github.com/jprofessionals/secret-share.git
    app_branch: main
    app_dir: /home/deploy/secret-share
    frontend_dir: /var/www/secret-share
    container_name: secret-share-backend
    container_port: 3000
    domain: "secret.{{ domain | default('jpro.dev') }}"
    database_url: "postgres://secret_share:{{ vault_secret_share_db_password }}@127.0.0.1:5432/secret_share_db"
    base_url: "https://{{ domain }}"
    rust_log: "info,secret_share_backend=debug"
    max_secret_days: 30
    max_secret_views: 100
    max_failed_attempts: 10

  tasks:
    - name: Clone or update repository
      ansible.builtin.git:
        repo: "{{ app_repo }}"
        dest: "{{ app_dir }}"
        version: "{{ app_branch }}"
        force: true

    - name: Build backend Docker image
      community.docker.docker_image:
        name: "{{ app_name }}-backend"
        tag: latest
        source: build
        build:
          path: "{{ app_dir }}/backend"
        force_source: true

    - name: Stop existing backend container
      community.docker.docker_container:
        name: "{{ container_name }}"
        state: absent

    - name: Start backend container
      community.docker.docker_container:
        name: "{{ container_name }}"
        image: "{{ app_name }}-backend:latest"
        state: started
        restart_policy: unless-stopped
        network_mode: host
        env:
          DATABASE_URL: "{{ database_url }}"
          BASE_URL: "{{ base_url }}"
          PORT: "{{ container_port | string }}"
          RUST_LOG: "{{ rust_log }}"
          MAX_SECRET_DAYS: "{{ max_secret_days | string }}"
          MAX_SECRET_VIEWS: "{{ max_secret_views | string }}"
          MAX_FAILED_ATTEMPTS: "{{ max_failed_attempts | string }}"

    - name: Install Node.js (if not present)
      become: true
      ansible.builtin.apt:
        name:
          - nodejs
          - npm
        state: present

    - name: Install frontend dependencies
      community.general.npm:
        path: "{{ app_dir }}/frontend"
        ci: true

    - name: Build frontend
      ansible.builtin.command:
        cmd: npm run build
        chdir: "{{ app_dir }}/frontend"
      environment:
        VITE_API_URL: ""

    - name: Create frontend directory
      become: true
      ansible.builtin.file:
        path: "{{ frontend_dir }}"
        state: directory
        owner: deploy
        group: deploy
        mode: "0755"

    - name: Deploy frontend files
      ansible.posix.synchronize:
        src: "{{ app_dir }}/frontend/dist/"
        dest: "{{ frontend_dir }}/"
        delete: true
        recursive: true
      delegate_to: "{{ inventory_hostname }}"

    - name: Write Caddy site config
      ansible.builtin.copy:
        dest: "/etc/caddy/sites/{{ app_name }}.caddy"
        content: |
          {{ domain }} {
              handle /api/* {
                  reverse_proxy localhost:{{ container_port }}
              }
              handle {
                  root * {{ frontend_dir }}
                  try_files {path} /index.html
                  file_server
              }
          }
        mode: "0644"

    - name: Wait for backend health check
      ansible.builtin.uri:
        url: "http://localhost:{{ container_port }}/health"
        status_code: 200
      register: health
      retries: 15
      delay: 2
      until: health.status == 200

    - name: Reload Caddy
      ansible.builtin.command:
        cmd: sudo systemctl reload caddy
      changed_when: true
```

Note: Uses `network_mode: host` so the container can reach PostgreSQL on `127.0.0.1`. This also means the container binds directly to port 3000 on the host.

**Step 2: Commit**

```bash
git add infra/ansible/deploy.yml
git commit -m "feat: add Ansible deploy playbook for dev VM"
```

---

### Task 3: Add Ansible collections requirement

**Files:**
- Create: `infra/ansible/requirements.yml`

**Step 1: Create `infra/ansible/requirements.yml`**

```yaml
---
collections:
  - name: community.docker
  - name: community.general
  - name: ansible.posix
```

Needed for `community.docker.docker_container`, `community.docker.docker_image`, `community.general.npm`, and `ansible.posix.synchronize`.

**Step 2: Commit**

```bash
git add infra/ansible/requirements.yml
git commit -m "feat: add Ansible collection requirements"
```

---

### Task 4: Switch E2E tests from DynamoDB to PostgreSQL

**Files:**
- Modify: `.github/workflows/ci.yml:68-121`

**Step 1: Change E2E test job name and env var**

In `.github/workflows/ci.yml`, change:

```yaml
  test-e2e:
    name: E2E Tests (DynamoDB)
```

to:

```yaml
  test-e2e:
    name: E2E Tests (PostgreSQL)
```

And change:

```yaml
      - name: Run E2E tests
        working-directory: e2e
        run: npm test
        env:
          E2E_DATABASE: dynamodb
```

to:

```yaml
      - name: Run E2E tests
        working-directory: e2e
        run: npm test
        env:
          E2E_DATABASE: postgres
```

**Step 2: Verify locally**

Run: `cd e2e && E2E_DATABASE=postgres npm test`
Expected: E2E tests pass against PostgreSQL testcontainer.

**Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "test: switch E2E tests from DynamoDB to PostgreSQL"
```

---

### Task 5: Replace SAM deploy job with Ansible deploy

**Files:**
- Modify: `.github/workflows/ci.yml:123-191`

**Step 1: Replace the deploy job**

Replace the entire `deploy:` job (lines 123-191) with:

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
          repository: jprofessionals/ansible-dev-vm-controll
          path: ansible-controll
          token: ${{ secrets.ANSIBLE_CONTROLL_PAT }}

      - name: Install Ansible
        run: pip install ansible

      - name: Install Ansible collections
        run: ansible-galaxy collection install -r infra/ansible/requirements.yml

      - name: Set up SSH key
        run: |
          mkdir -p ~/.ssh
          echo "${{ secrets.DEPLOY_SSH_KEY }}" > ~/.ssh/id_ed25519
          chmod 600 ~/.ssh/id_ed25519
          ssh-keyscan -H ${{ vars.VM_HOST }} >> ~/.ssh/known_hosts

      - name: Set up Vault password
        run: echo "${{ secrets.VAULT_PASSWORD }}" > .vault_password

      - name: Run deploy playbook
        working-directory: infra/ansible
        run: >
          ansible-playbook deploy.yml
          --vault-password-file ../../.vault_password
          -e ansible_controll_vars=../../ansible-controll/inventory/group_vars/all.yml
        env:
          VM_HOST: ${{ vars.VM_HOST }}
```

**Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "feat: replace SAM deploy with Ansible VM deployment"
```

---

### Task 6: Update CLAUDE.md deployment section

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Update deployment commands and options**

In the Common Commands section, replace the `# Deployment` and `# AWS Serverless (SAM)` blocks with:

```bash
# Deployment (Ansible → Dev VM)
cd infra/ansible && ansible-playbook deploy.yml --vault-password-file ../../.vault_password
# Requires: VM_HOST env var, vault password file, ansible-controll vars accessible

# AWS Serverless (SAM) - alternative deployment
cd infra/sam && sam build     # Build Lambda function
cd infra/sam && sam deploy    # Deploy stack to AWS
cd infra/sam && sam validate  # Validate SAM template
cd infra/sam && sam delete    # Delete the deployed stack
```

In the Deployment Options section, update to:

```
1. **Docker Compose** - `docker-compose.yml` for local/testing
2. **Ansible (Dev VM)** - `infra/ansible/deploy.yml` for dev VM deployment (primary)
3. **AWS Lambda** - `infra/sam/template.yaml` for serverless deployment (alternative)
```

**Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md with Ansible deployment commands"
```

---

### Task 7: Validate the full workflow

**Step 1: Lint the Ansible playbook**

Run: `cd infra/ansible && ansible-playbook deploy.yml --syntax-check`
Expected: `playbook: deploy.yml` (no errors)

**Step 2: Verify CI workflow YAML is valid**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"`
Expected: No errors

**Step 3: Run E2E tests against PostgreSQL**

Run: `cd e2e && E2E_DATABASE=postgres npm test`
Expected: All tests pass

**Step 4: Commit any fixes if needed**
