import { spawn, execSync } from 'child_process';
import { GenericContainer, Wait } from 'testcontainers';
import * as fs from 'fs';
import * as path from 'path';

const STATE_FILE = '/tmp/e2e-test-context.json';
const ROOT_DIR = path.resolve(__dirname, '../..');

interface TestState {
  database: 'postgres' | 'dynamodb';
  databasePort: number;
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

async function startPostgres(): Promise<{ port: number; env: Record<string, string> }> {
  console.log('Starting PostgreSQL container...');
  const container = await new GenericContainer('postgres:16-alpine')
    .withEnvironment({
      POSTGRES_PASSWORD: 'postgres',
      POSTGRES_DB: 'secretshare',
    })
    .withExposedPorts(5432)
    .withWaitStrategy(Wait.forLogMessage('database system is ready to accept connections'))
    .start();

  const port = container.getMappedPort(5432);
  const host = container.getHost();
  const databaseUrl = `postgres://postgres:postgres@${host}:${port}/secretshare`;

  console.log(`PostgreSQL running at ${host}:${port}`);
  await new Promise((r) => setTimeout(r, 3000));

  return {
    port,
    env: { DATABASE_URL: databaseUrl },
  };
}

async function startDynamoDB(): Promise<{ port: number; env: Record<string, string> }> {
  console.log('Starting DynamoDB Local container...');
  const container = await new GenericContainer('amazon/dynamodb-local')
    .withExposedPorts(8000)
    .withWaitStrategy(Wait.forListeningPorts())
    .start();

  const port = container.getMappedPort(8000);
  const host = container.getHost();
  const endpoint = `http://${host}:${port}`;

  console.log(`DynamoDB Local running at ${endpoint}`);
  await new Promise((r) => setTimeout(r, 1000));

  return {
    port,
    env: {
      DYNAMODB_TABLE: 'secrets',
      DYNAMODB_ENDPOINT: endpoint,
      AWS_ACCESS_KEY_ID: 'test',
      AWS_SECRET_ACCESS_KEY: 'test',
      AWS_REGION: 'us-east-1',
    },
  };
}

async function globalSetup(): Promise<void> {
  console.log('Starting E2E test infrastructure...');

  const database = (process.env.E2E_DATABASE || 'dynamodb') as 'postgres' | 'dynamodb';
  console.log(`Using database: ${database}`);

  // 1. Start database container
  const { port: databasePort, env: dbEnv } =
    database === 'postgres' ? await startPostgres() : await startDynamoDB();

  // 2. Build and start backend
  const backendBinary = path.join(ROOT_DIR, 'backend/target/release/secret-share-backend');
  if (!fs.existsSync(backendBinary)) {
    console.log('Building backend...');
    execSync('cargo build --release', {
      cwd: path.join(ROOT_DIR, 'backend'),
      stdio: 'inherit',
    });
  } else {
    console.log('Using pre-built backend binary');
  }

  console.log('Starting backend...');
  const backendProcess = spawn('./target/release/secret-share-backend', [], {
    cwd: path.join(ROOT_DIR, 'backend'),
    env: {
      ...process.env,
      ...dbEnv,
      BASE_URL: 'http://localhost:4173',
      PORT: '3000',
      RUST_LOG: 'info',
    },
    stdio: ['ignore', 'inherit', 'inherit'],
    detached: true,
  });

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
  const frontendProcess = spawn('npx', ['vite', 'preview', '--port', '4173', '--host'], {
    cwd: path.join(ROOT_DIR, 'frontend'),
    env: {
      ...process.env,
    },
    stdio: 'pipe',
    detached: true,
  });

  frontendProcess.unref();

  const frontendUrl = 'http://localhost:4173';
  console.log('Waiting for frontend to be ready...');
  await waitForUrl(frontendUrl);
  console.log('Frontend ready');

  // Save state for teardown
  const state: TestState = {
    database,
    databasePort,
    backendPid: backendProcess.pid!,
    frontendPid: frontendProcess.pid!,
    backendUrl,
    frontendUrl,
  };

  fs.writeFileSync(STATE_FILE, JSON.stringify(state));

  console.log('E2E infrastructure ready');
}

export default globalSetup;
