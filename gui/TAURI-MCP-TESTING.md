# Tauri MCP Server Integration Testing

**Date**: 2026-03-07  
**Purpose**: Document testing of Tauri MCP server integration with GitEHR GUI

---

## Setup Status

### Workspace Configuration

✅ **Fixed**: Added `gui/src-tauri` to workspace exclusions in root `Cargo.toml`
```toml
exclude = [
    "gui/src-tauri",
]
```

✅ **Fixed**: Corrected gitehr dependency path in `gui/src-tauri/Cargo.toml`
```toml
gitehr = { path = "../../cli" }
```

✅ **Fixed**: Updated Tauri version to match npm package (2.10)
```toml
tauri = { version = "2.10", features = ["image-png"] }
```

### Compilation Status

✅ **Backend compiles successfully** - `cargo check` passed in 19.55s
- All Tauri plugins loaded correctly
- GitEHR CLI integration working
- All dependencies resolved

---

## Current Tauri Commands (Backend)

The Tauri app exposes these commands to the frontend:

### Repository Management
- `get_current_dir()` - Get current working directory
- `is_gitehr_repo(path)` - Check if path is a GitEHR repo
- `init_repo(path)` - Initialize new GitEHR repo

### MPI (Main Patient Index)
- `has_mpi(path)` - Check if MPI exists
- `get_mpi(path)` - Load MPI data

### Repository Status
- `get_status(repo_path)` - Get repo status info
  - Returns: is_gitehr_repo, version, entry count, state files, encryption status

### Journal Operations
- `get_journal_entries(repo_path, limit, offset, reverse)` - List journal entries
- `add_journal_entry(repo_path, content)` - Create new entry
- `verify_journal(repo_path)` - Verify journal integrity

### State Management
- `get_state_files(repo_path)` - List all state files
- `get_state_file(repo_path, filename)` - Read specific state file
- `update_state_file(repo_path, filename, content)` - Update state file

### Contributors
- `get_contributors(repo_path)` - List contributors
- `get_current_contributor(repo_path)` - Get active contributor
- `activate_contributor(repo_path, contributor_id)` - Switch contributor

---

## Frontend Components (React + Mantine)

**UI Framework**: Mantine 8.3.14  
**Icons**: Tabler Icons  
**Build**: Vite + TypeScript

**Key Features Implemented**:
- Repository selection (folder picker)
- MPI support (multi-patient management)
- Journal entry list with pagination
- State file viewer
- New entry creation form
- Repository status display

---

## Testing with Tauri MCP Server

### Prerequisites

1. **Tauri MCP Server Installed** (user confirmed)
2. **Claude Desktop configured** to use Tauri MCP server
3. **Dev environment ready**:
   - Node.js + npm
   - Rust toolchain
   - Tauri CLI 2.10.0

### Test Steps

#### 1. Launch Dev Server

```bash
cd /home/marcus/code/gitehr/gitehr/gui
npm run tauri dev
```

**Expected**: 
- Vite dev server starts on http://localhost:5173
- Tauri window opens with GitEHR GUI
- Backend connects to frontend

#### 2. Test MCP Server Capabilities

The Tauri MCP server should enable:

**UI Inspection**:
- View current DOM structure
- Inspect component hierarchy
- Read CSS styles and layout

**UI Modification**:
- Suggest UI improvements
- Modify styles dynamically
- Test responsive layouts

**Component Testing**:
- Verify Tauri commands work
- Test data flow (backend ↔ frontend)
- Check error handling

#### 3. Verify Integration Points

**Backend ↔ Frontend Communication**:
```typescript
// Example: Test journal entry loading
import { getJournalEntries } from './api/gitehr';

const entries = await getJournalEntries('/path/to/repo', 10, 0, true);
// Should return array of JournalEntryInfo objects
```

**Tauri Commands**:
```rust
// Backend (Rust)
#[tauri::command]
fn get_journal_entries(...) -> Result<Vec<JournalEntryInfo>, String>

// Frontend (TypeScript)
import { invoke } from '@tauri-apps/api/core';
const entries = await invoke('get_journal_entries', { ... });
```

---

## Known Issues

### Fixed
- ✅ Workspace configuration conflict
- ✅ Path dependency incorrect
- ✅ Tauri version mismatch

### Pending Investigation
- ⚠️ **Dev server startup** - Need to test if UI loads correctly
- ⚠️ **Window rendering** - Verify Tauri window opens with content
- ⚠️ **Command execution** - Test all backend commands work from UI

---

## Next Steps

1. **Start dev server** and verify UI loads
2. **Test MCP integration** - Use Claude with Tauri MCP to inspect app
3. **Verify commands** - Test each Tauri command from UI
4. **UI improvements** - Use MCP to suggest and implement enhancements
5. **Documentation** - Document MCP workflows for future development

---

## MCP Server Configuration

**Expected location**: `~/.config/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json`

Or in Claude Desktop config: `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS)

**Example configuration**:
```json
{
  "mcpServers": {
    "tauri": {
      "command": "npx",
      "args": ["-y", "@tauri-apps/mcp-server"],
      "env": {
        "TAURI_DEV_URL": "http://localhost:5173"
      }
    }
  }
}
```

---

## Troubleshooting

### Issue: "gitehr binary not found"
**Solution**: Ensure `gitehr` CLI is built and in PATH:
```bash
cd /home/marcus/code/gitehr/gitehr
cargo build --release
export PATH="$PATH:/home/marcus/code/gitehr/gitehr/target/release"
```

### Issue: "Frontend not loading"
**Solution**: Check Vite dev server is running:
```bash
# In separate terminal
cd gui
npm run dev
```

### Issue: "MCP server can't see app"
**Solution**: Verify Tauri window is open and dev URL is correct in MCP config

---

## Resources

- **Tauri Docs**: https://tauri.app/
- **Mantine UI**: https://mantine.dev/
- **Tauri MCP Server**: https://github.com/tauri-apps/mcp-server
- **GitEHR Specs**: `/home/marcus/code/gitehr/gitehr/spec/gui/gui.md`

---

**Status**: Ready for testing  
**Last Updated**: 2026-03-07
