import os from 'os';
import path from 'path';
import { spawn, spawnSync, execSync } from 'child_process';
import { fileURLToPath } from 'url';
import { mkdtempSync, rmSync, existsSync } from 'fs';

const __dirname = fileURLToPath(new URL('.', import.meta.url));

let tauriDriver;
let exit = false;
let testRepoPath = null;

const isWithRepo = process.env.E2E_WITH_REPO === 'true';

export const config = {
  runner: 'local',
  host: '127.0.0.1',
  port: 4444,
  specs: isWithRepo 
    ? ['./specs/journal.spec.js', './specs/sidebar.spec.js']
    : ['./specs/initial-load.spec.js'],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      'tauri:options': {
        application: path.resolve(__dirname, '../src-tauri/target/debug/guigitehr-gui'),
      },
    },
  ],
  reporters: ['spec'],
  framework: 'mocha',
  mochaOpts: {
    ui: 'bdd',
    timeout: 60000,
  },

  onPrepare: () => {
    console.log('Building Tauri app (debug mode)...');
    spawnSync('npm', ['run', 'tauri', 'build', '--', '--debug', '--no-bundle'], {
      cwd: path.resolve(__dirname, '..'),
      stdio: 'inherit',
      shell: true,
    });

    if (isWithRepo) {
      const cliBinary = path.resolve(__dirname, '../../../target/debug/gitehr');
      
      if (!existsSync(cliBinary)) {
        console.log('Building gitehr CLI...');
        execSync('cargo build', {
          cwd: path.resolve(__dirname, '../../..'),
          stdio: 'inherit',
        });
      }

      testRepoPath = mkdtempSync(path.join(os.tmpdir(), 'gitehr-e2e-'));
      console.log(`Creating test repo at: ${testRepoPath}`);
      
      execSync(`"${cliBinary}" init`, { cwd: testRepoPath, stdio: 'inherit', shell: true });
      execSync(`"${cliBinary}" journal add "Initial test entry for E2E testing"`, { cwd: testRepoPath, stdio: 'inherit', shell: true });
      execSync(`"${cliBinary}" state set allergies "Penicillin, Sulfa drugs"`, { cwd: testRepoPath, stdio: 'inherit', shell: true });
      execSync(`"${cliBinary}" state set medications "Aspirin 81mg daily"`, { cwd: testRepoPath, stdio: 'inherit', shell: true });
    }
  },

  beforeSession: () => {
    const cwd = isWithRepo ? testRepoPath : os.tmpdir();
    
    tauriDriver = spawn(
      path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'),
      [],
      { 
        stdio: [null, process.stdout, process.stderr],
        cwd: cwd,
      }
    );

    tauriDriver.on('error', (error) => {
      console.error('tauri-driver error:', error);
      process.exit(1);
    });
    tauriDriver.on('exit', (code) => {
      if (!exit) {
        console.error('tauri-driver exited with code:', code);
        process.exit(1);
      }
    });
  },

  afterSession: () => {
    closeTauriDriver();
  },

  onComplete: () => {
    if (testRepoPath && existsSync(testRepoPath)) {
      console.log(`Cleaning up test repo: ${testRepoPath}`);
      rmSync(testRepoPath, { recursive: true, force: true });
    }
  },
};

function closeTauriDriver() {
  exit = true;
  tauriDriver?.kill();
}

function onShutdown(fn) {
  const cleanup = () => {
    try {
      fn();
    } finally {
      process.exit();
    }
  };

  process.on('exit', cleanup);
  process.on('SIGINT', cleanup);
  process.on('SIGTERM', cleanup);
  process.on('SIGHUP', cleanup);
  process.on('SIGBREAK', cleanup);
}

onShutdown(() => {
  closeTauriDriver();
  if (testRepoPath && existsSync(testRepoPath)) {
    rmSync(testRepoPath, { recursive: true, force: true });
  }
});
