import * as fs from 'fs';
import { execSync } from 'child_process';

const STATE_FILE = '/tmp/e2e-test-context.json';

interface TestState {
  postgresPort: number;
  backendPid: number;
  frontendPid: number;
  backendUrl: string;
  frontendUrl: string;
}

async function globalTeardown(): Promise<void> {
  console.log('Tearing down E2E test infrastructure...');

  try {
    const state: TestState = JSON.parse(fs.readFileSync(STATE_FILE, 'utf-8'));

    // Kill backend process
    try {
      process.kill(state.backendPid, 'SIGTERM');
      console.log('Backend stopped');
    } catch {
      console.log('Backend already stopped');
    }

    // Kill frontend process
    try {
      process.kill(state.frontendPid, 'SIGTERM');
      console.log('Frontend stopped');
    } catch {
      console.log('Frontend already stopped');
    }

    // Stop PostgreSQL container (Docker will handle cleanup)
    try {
      execSync(`docker stop $(docker ps -q --filter "publish=${state.postgresPort}")`, {
        stdio: 'pipe',
      });
      console.log('PostgreSQL container stopped');
    } catch {
      console.log('PostgreSQL container already stopped');
    }

    // Clean up state file
    fs.unlinkSync(STATE_FILE);
  } catch (error) {
    console.error('Error during teardown:', error);
  }
}

export default globalTeardown;
