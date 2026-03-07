# Store Root Implementation - Complete

**Date**: 2026-03-07  
**Issue**: GUI could not create store root from scratch  
**Status**: ✅ **COMPLETE**

---

## Problem Identified

The GitEHR GUI had a "Create New Repository (Store Root)" button, but:
1. It called `gitehr init` which creates a **single patient repository**
2. It then tried to load `gitehr-mpi.json` which doesn't exist
3. Result: **Error: "MPI not found in selected store root"**

**Root cause**: No CLI command existed to create a store root with MPI.

---

## Solution Implemented

### 1. New CLI Command: `gitehr store`

Created `/gitehr-cli/src/commands/store.rs` with:

#### `gitehr store init`
```bash
cd /path/to/new-store
gitehr store init
```

**Creates**:
- `gitehr-mpi.json` - Empty Master Patient Index
- `patients/` directory - For future patient repositories
- `README.md` - Documentation

**Output**:
```
✓ Initialized GitEHR store root
  Created: gitehr-mpi.json (empty MPI)
  Created: patients/ directory
  Created: README.md

Next steps:
  1. Create patient repositories in patients/ directory
  2. Register patients in gitehr-mpi.json
  3. Open this directory in the GitEHR GUI
```

#### `gitehr store list`
```bash
cd /path/to/store
gitehr store list
```

Lists all patients registered in the MPI with their identifiers and status.

---

### 2. Updated Tauri Backend

Added new command to `gui/gitehr-gui/src-tauri/src/lib.rs`:

```rust
#[tauri::command]
fn init_store_root(path: String) -> Result<String, String> {
    let output = std::process::Command::new("gitehr")
        .arg("store")
        .arg("init")
        .current_dir(&path)
        .output()
        .map_err(|e| {
            if e.kind() == ErrorKind::NotFound {
                "GitEHR CLI not found. Install gitehr or ensure it is in PATH.".to_string()
            } else {
                format!("Failed to execute gitehr binary: {}", e)
            }
        })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
```

Registered in `invoke_handler!` list.

---

### 3. Updated Frontend API

Added to `gui/gitehr-gui/src/api/gitehr.ts`:

```typescript
export async function initStoreRoot(path: string): Promise<string> {
  return invoke<string>("init_store_root", { path });
}
```

---

### 4. Updated Frontend UI

Modified `gui/gitehr-gui/src/App.tsx`:

```typescript
const handleInitRepo = async () => {
  try {
    const folder = await pickFolder();
    if (folder) {
      setCreating(true);
      setError(null);
      try {
        await initStoreRoot(folder);  // ← Now calls store init!
        const mpiData = await getMpi(folder);
        setMpi(mpiData);
        setStoreRoot(mpiData.store_root);
      } catch (err) {
        // ... error handling
      }
    }
  } catch (err) {
    // ... error handling
  }
};
```

---

## Files Modified

### Created
1. `gitehr-cli/src/commands/store.rs` - New store module (170 lines)

### Modified
2. `gitehr-cli/src/commands/mod.rs` - Registered store module
3. `gitehr-cli/src/main.rs` - Added `Store` command enum and handler
4. `gui/gitehr-gui/src-tauri/src/lib.rs` - Added `init_store_root()` Tauri command
5. `gui/gitehr-gui/src/api/gitehr.ts` - Added `initStoreRoot()` API wrapper
6. `gui/gitehr-gui/src/App.tsx` - Updated button handler to use `initStoreRoot()`

---

## Testing

### CLI Test
```bash
$ cd /tmp && mkdir test-store && cd test-store
$ gitehr store init
✓ Initialized GitEHR store root
  Created: gitehr-mpi.json (empty MPI)
  Created: patients/ directory
  Created: README.md

$ cat gitehr-mpi.json
{
  "version": 1,
  "updated_at": "2026-03-07T17:42:00.324336756+00:00",
  "patients": []
}

$ gitehr store list
No patients registered in this store root.
Add patient repositories in the patients/ directory and register them in gitehr-mpi.json
```

### GUI Test
1. ✅ "Create New Repository (Store Root)" button now functional
2. ✅ Creates proper store root structure
3. ✅ Loads MPI successfully
4. ✅ No more "MPI not found" error

---

## Design Philosophy Maintained

✅ **100% CLI-backed**: GUI cannot do anything that CLI can't do  
✅ **Consistent**: `gitehr store init` matches pattern of other commands  
✅ **User-friendly**: Clear output and documentation  
✅ **Safe**: Checks for existing store/repo before creating  

---

## Future Enhancements

### Short-term
- [ ] Add `gitehr store add-patient` command to register patients in MPI
- [ ] Add `gitehr store remove-patient` command
- [ ] GUI workflow to create patient repos within store

### Medium-term
- [ ] Validate MPI schema
- [ ] Support patient merging/linking
- [ ] Import/export MPI

### Long-term
- [ ] Multi-organization support
- [ ] Distributed MPI sync
- [ ] Advanced patient search

---

## CLI Help Output

```
$ gitehr store --help
Usage: gitehr store <COMMAND>

Commands:
  init  Initialize a new store root (multi-patient)
  list  List all patients in the store
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

---

## MPI File Structure

```json
{
  "version": 1,
  "updated_at": "2026-03-07T17:42:00+00:00",
  "patients": [
    {
      "patient_id": "550e8400-e29b-41d4-a716-446655440000",
      "repo_path": "patients/550e8400-e29b-41d4-a716-446655440000",
      "status": "active",
      "merged_into": null,
      "updated_at": "2026-03-07T17:42:00+00:00",
      "identifiers": [
        {
          "type": "NHS_NUMBER",
          "value": "1234567890"
        },
        {
          "type": "LOCAL_ID",
          "value": "P123456"
        }
      ]
    }
  ]
}
```

---

## Success Criteria

All criteria met:

- [x] CLI command `gitehr store init` creates valid store root
- [x] CLI command `gitehr store list` displays patient data
- [x] GUI button creates store root via CLI command
- [x] GUI loads MPI after creation
- [x] No duplicate functionality between CLI and GUI
- [x] Proper error handling for edge cases
- [x] Documentation included in README.md
- [x] No breaking changes to existing functionality

---

**Implementation**: Complete  
**Status**: Ready for use  
**Next**: User can now create multi-patient stores from both CLI and GUI!
