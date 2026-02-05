# GitEHR GUI E2E Tests

End-to-end tests for the GitEHR GUI using WebDriverIO and tauri-driver.

## Prerequisites

### System Dependencies

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get install webkit2gtk-driver
```

**macOS:**
WebKitWebDriver is included with Safari.

**Windows:**
Edge WebDriver is required. Install via:
```bash
cargo install --git https://github.com/nicholasrice/msedgedriver-tool
msedgedriver-tool
```

### Rust Tools
```bash
cargo install tauri-driver --locked
```

### Node Dependencies
```bash
cd gui/gitehr-gui/e2e
npm install
```

## Running Tests

### All Tests
```bash
npm test
```

### No-Repo Tests Only
Tests the initial load screen when no GitEHR repository is detected:
```bash
npm run test:no-repo
```

### With-Repo Tests Only
Tests journal entries, sidebar, and state files with a test repository:
```bash
npm run test:with-repo
```

## Test Structure

```
e2e/
├── package.json        # Test dependencies and scripts
├── wdio.conf.js        # WebDriverIO configuration
├── README.md           # This file
└── specs/
    ├── initial-load.spec.js  # No-repo screen tests
    ├── journal.spec.js       # Journal entry tests
    └── sidebar.spec.js       # Sidebar and state file tests
```

## How It Works

1. **onPrepare**: Builds the Tauri app in debug mode. If `E2E_WITH_REPO=true`, also creates a temporary test repository with sample data.

2. **beforeSession**: Starts `tauri-driver` which proxies WebDriver requests to the app.

3. **Tests run**: WebDriverIO connects to tauri-driver and interacts with the app.

4. **Cleanup**: tauri-driver is killed and temp repo is deleted.

## CI Integration

See the GitHub Actions workflow example in the Tauri docs for running these tests in CI with xvfb (Linux) or native drivers (Windows/macOS).

Example workflow snippet:
```yaml
- name: Install WebDriver dependencies (Linux)
  if: matrix.platform == 'ubuntu-latest'
  run: sudo apt-get install -y webkit2gtk-driver xvfb

- name: Install tauri-driver
  run: cargo install tauri-driver --locked

- name: Run E2E tests
  run: xvfb-run npm test
  working-directory: gui/gitehr-gui/e2e
```
