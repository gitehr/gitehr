import os from 'os';
import path from 'path';
import { spawn, spawnSync, execSync } from 'child_process';
import { fileURLToPath } from 'url';
import { mkdtempSync, rmSync, existsSync, writeFileSync, readFileSync, renameSync } from 'fs';

const __dirname = fileURLToPath(new URL('.', import.meta.url));

// Fixture paths are handed from onPrepare (launcher process) to the workers
// through GITEHR_E2E_STORE - WebdriverIO's local runner spawns workers as
// separate processes, so module state set in onPrepare is not visible in
// beforeSession and friends.
const fixtureFile = () => path.join(os.tmpdir(), 'gitehr-e2e-store-path');

let tauriDriver;
let exit = false;

const isWithRepo = process.env.E2E_WITH_REPO === 'true';

export const config = {
  runner: 'local',
  host: '127.0.0.1',
  port: 4444,
  specs: isWithRepo
    ? ['./specs/journal.spec.js']
    : ['./specs/initial-load.spec.js'],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      'tauri:options': {
        application: path.resolve(__dirname, '../src-tauri/target/debug/gitehr-gui'),
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
    // Quarantine the machine's GitEHR config: the app reads
    // $HOME/.config/gitehr/config.toml and would otherwise open the
    // developer's own store instead of showing the no-repo picker or the
    // fixture store. File-level, because tauri-driver does not pass env on
    // to the launched app. Restored in onComplete.
    const realConfig = path.join(os.homedir(), '.config', 'gitehr', 'config.toml');
    const quarantined = `${realConfig}.e2e-quarantined`;
    if (existsSync(realConfig)) {
      renameSync(realConfig, quarantined);
    }

    console.log('Building Tauri app (debug mode)...');
    spawnSync('npm', ['run', 'tauri', 'build', '--', '--debug', '--no-bundle'], {
      cwd: path.resolve(__dirname, '..'),
      stdio: 'inherit',
      shell: true,
    });

    if (isWithRepo) {
      const cliBinary = path.resolve(__dirname, '../../target/debug/gitehr');

      if (!existsSync(cliBinary)) {
        console.log('Building gitehr CLI...');
        execSync('cargo build', {
          cwd: path.resolve(__dirname, '../..'),
          stdio: 'inherit',
        });
      }

      const testStorePath = mkdtempSync(path.join(os.tmpdir(), 'gitehr-e2e-'));
      console.log(`Creating test store at: ${testStorePath}`);

      execSync(`"${cliBinary}" store init e2e`, { cwd: testStorePath, stdio: 'inherit', shell: true });
      execSync(`"${cliBinary}" store link e2e NHS:1234567890`, { cwd: testStorePath, stdio: 'inherit', shell: true });
      const testRepoPath = path.join(testStorePath, 'e2e');
      execSync('git config commit.gpgsign false', { cwd: testRepoPath, stdio: 'inherit', shell: true });
      execSync(`"${cliBinary}" journal add "Initial test entry for E2E testing"`, { cwd: testRepoPath, stdio: 'inherit', shell: true });
      execSync(`"${cliBinary}" allergies add --agent Penicillin --reaction anaphylaxis --severity high`, { cwd: testRepoPath, stdio: 'inherit', shell: true });

      writeFileSync(fixtureFile(), JSON.stringify({ testStorePath, testRepoPath }));
    }
  },

  beforeSession: () => {
    // tauri-driver does not propagate env/cwd to the launched app, so the
    // app's cwd-inheritance is unreliable. We give cwd the Store root: the
    // GUI's initial check finds gitehr-mpi.json there and shows the Patient
    // Index for the fixture store (the specs then click Open on the subject).
    let cwd = os.tmpdir();

    if (isWithRepo && existsSync(fixtureFile())) {
      const fixture = JSON.parse(readFileSync(fixtureFile(), 'utf8'));
      cwd = fixture.testStorePath;
    }

    tauriDriver = spawn(
      path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'),
      [],
      {
        stdio: [null, process.stdout, process.stderr],
        cwd,
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
    const realConfig = path.join(os.homedir(), '.config', 'gitehr', 'config.toml');
    const quarantined = `${realConfig}.e2e-quarantined`;
    if (existsSync(quarantined)) {
      renameSync(quarantined, realConfig);
    }

    if (existsSync(fixtureFile())) {
      try {
        const { testStorePath } = JSON.parse(readFileSync(fixtureFile(), 'utf8'));
        if (testStorePath && existsSync(testStorePath)) {
          console.log(`Cleaning up test store: ${testStorePath}`);
          rmSync(testStorePath, { recursive: true, force: true });
        }
      } finally {
        rmSync(fixtureFile(), { force: true });
      }
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
  if (existsSync(fixtureFile())) {
    try {
      const { testStorePath } = JSON.parse(readFileSync(fixtureFile(), 'utf8'));
      if (testStorePath && existsSync(testStorePath)) {
        rmSync(testStorePath, { recursive: true, force: true });
      }
    } catch {
      // nothing to clean up
    }
    rmSync(fixtureFile(), { force: true });
  }
});