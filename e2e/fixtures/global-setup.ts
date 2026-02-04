import { spawn, ChildProcess, execSync } from 'child_process';
import { GenericContainer, Wait } from 'testcontainers';
import * as fs from 'fs';
import * as path from 'path';

const STATE_FILE = '/tmp/e2e-test-context.json';
const ROOT_DIR = path.resolve(__dirname, '../..');

interface TestState {
  postgresPort: number;
  backendPid: number;
  frontendPid: number;
  backendUrl: string;
  frontendUrl: string;
}

async function waitForUrl(url: string, maxAttempts = 30): Promise<void> {
  for (let i = 0; i < maxAttempts; i++) {
    try {
      const response = await fetch(url);
      if (response.ok) return;
    } catch {
      // Ignore errors, keep trying
    }
    await new Promise((r) => setTimeout(r, 1000));
  }
  throw new Error(`Timeout waiting for ${url}`);
}

async function globalSetup(): Promise<void> {
  console.log('Starting E2E test infrastructure...');

  // 1. Start PostgreSQL container
  console.log('Starting PostgreSQL container...');
  const postgresContainer = await new GenericContainer('postgres:16-alpine')
    .withEnvironment({
      POSTGRES_PASSWORD: 'postgres',
      POSTGRES_DB: 'secretshare',
    })
    .withExposedPorts(5432)
    .withWaitStrategy(Wait.forLogMessage('database system is ready to accept connections'))
    .start();

  const postgresPort = postgresContainer.getMappedPort(5432);
  const postgresHost = postgresContainer.getHost();
  const databaseUrl = `postgres://postgres:postgres@${postgresHost}:${postgresPort}/secretshare`;

  console.log(`PostgreSQL running at ${postgresHost}:${postgresPort}`);
  console.log(`Database URL: ${databaseUrl}`);

  // Wait for database to be fully ready (beyond just log message)
  console.log('Waiting for PostgreSQL to be fully ready...');
  await new Promise((r) => setTimeout(r, 3000));

  // 2. Build and start backend
  console.log('Building backend...');
  execSync('cargo build --release', {
    cwd: path.join(ROOT_DIR, 'backend'),
    stdio: 'inherit',
  });

  console.log('Starting backend...');
  const backendProcess = spawn(
    './target/release/secret-share-backend',
    [],
    {
      cwd: path.join(ROOT_DIR, 'backend'),
      env: {
        ...process.env,
        DATABASE_URL: databaseUrl,
        BASE_URL: 'http://localhost:4173',
        PORT: '3000',
        RUST_LOG: 'info',
      },
      stdio: ['ignore', 'inherit', 'inherit'],
      detached: true,
    }
  );

  backendProcess.unref();

  const backendUrl = 'http://localhost:3000';
  console.log('Waiting for backend to be ready...');
  await waitForUrl(`${backendUrl}/health`);
  console.log('Backend ready');

  // 3. Build and start frontend
  console.log('Building frontend...');
  execSync('npm run build', {
    cwd: path.join(ROOT_DIR, 'frontend'),
    stdio: 'inherit',
    env: {
      ...process.env,
      VITE_API_URL: backendUrl,
    },
  });

  console.log('Starting frontend preview server...');
  const frontendProcess = spawn(
    'npx',
    ['vite', 'preview', '--port', '4173', '--host'],
    {
      cwd: path.join(ROOT_DIR, 'frontend'),
      env: {
        ...process.env,
      },
      stdio: 'pipe',
      detached: true,
    }
  );

  frontendProcess.unref();

  const frontendUrl = 'http://localhost:4173';
  console.log('Waiting for frontend to be ready...');
  await waitForUrl(frontendUrl);
  console.log('Frontend ready');

  // Save state for teardown
  const state: TestState = {
    postgresPort,
    backendPid: backendProcess.pid!,
    frontendPid: frontendProcess.pid!,
    backendUrl,
    frontendUrl,
  };

  fs.writeFileSync(STATE_FILE, JSON.stringify(state));

  console.log('E2E infrastructure ready');
}

export default globalSetup;
