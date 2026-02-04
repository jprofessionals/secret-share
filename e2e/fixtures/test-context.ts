import { ChildProcess } from 'child_process';
import { StartedTestContainer } from 'testcontainers';

export interface TestContext {
  postgresContainer: StartedTestContainer;
  backendProcess: ChildProcess;
  frontendProcess: ChildProcess;
  backendUrl: string;
  frontendUrl: string;
}

// Global state file path
export const STATE_FILE = '/tmp/e2e-test-context.json';
