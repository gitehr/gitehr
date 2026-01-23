# GitEHR

A Git-based, decentralised, multi-contributor Electronic Health Record system.

## Design Philosophy

1. **Git-based Storage**

   - Leverages Git's distributed nature and version control
   - Each change is tracked and auditable
   - Multiple contributors can work on the same record
   - Full history is preserved

2. **Immutable Journal Structure**

   - Clinical entries are stored in chronological order
   - Each entry links to its parent via cryptographic hash
   - Forms a tamper-evident chain of clinical events
   - Initial entry seeded with random data for security

3. **Clear Data Organization**

   - `/journal` - Immutable chronological entries
   - `/state` - Current clinical state that may be updated
   - `/imaging` - Medical imaging and related data
   - `/.gitehr` - Configuration and internal data

4. **Security First**

   - All entries can be cryptographically verified
   - Support for encryption at rest
   - Designed for future digital signatures
   - Transport format for secure data movement

5. **Two-Part Architecture**
   - Core CLI tool for data operations (`gitehr`)
   - Reference GUI for clinical use (`gitehr gui`)

# This repository

This repository is a monorepo combining the core CLI tool, documentation, and folder structure templates.

## Documentation

GitEHR's documentation is best viewed online at [https://gitehr.org/](https://gitehr.org/).

All docs are also available in this repository under the `docs/` directory in markdown format.
