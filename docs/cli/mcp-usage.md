# GitEHR MCP Server Usage

## Overview

GitEHR now includes a Model Context Protocol (MCP) server that exposes repository data and operations to LLM applications. This enables AI-assisted clinical workflows while maintaining GitEHR's security and audit trail.

## Quick Start

### Starting the MCP Server

```bash
# From within a GitEHR repository
gitehr mcp serve --stdio

# From outside a repository (specify path)
gitehr mcp serve --stdio --repo-path /path/to/gitehr/repo
```

The server runs on stdio by default, which is the standard transport for MCP clients like Claude Desktop.

### Testing the Server

Create a simple test client to verify the server is working:

```bash
# In a GitEHR repository
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | ./target/release/gitehr mcp serve --stdio
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "resources": {},
      "tools": {},
      "prompts": {}
    },
    "serverInfo": {
      "name": "gitehr",
      "version": "0.1.7"
    }
  }
}
```

## MCP Capabilities

### Resources (Read-Only)

Resources provide read-only access to repository data.

#### List Resources

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "resources/list"
}
```

Returns available resources:
- `gitehr://repo/journal` - Journal entries list
- `gitehr://repo/state` - State files list
- `gitehr://repo/status` - Repository status

Two further URI patterns are readable but not listed by `resources/list` — read them directly by URI:
- `gitehr://repo/journal/{filename}` - Content of one journal entry (e.g. `gitehr://repo/journal/20260101T000000.000Z-abc123.md`)
- `gitehr://repo/state/{filename}` - Content of one state file (e.g. `gitehr://repo/state/demographics.json`)

Every resource URI is relative to the repository the server was started against (`--repo-path`, or the current directory) — the URI itself never contains a filesystem path.

#### Read Resource

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "resources/read",
  "params": {
    "uri": "gitehr://repo/journal"
  }
}
```

Returns JSON array of journal entry filenames, or content of specific resources.

### Tools (Read-Write)

Tools allow write operations on the repository.

#### List Tools

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/list"
}
```

Returns available tools:
- `add_journal_entry` - Create a new journal entry
- `update_state` - Update a state file
- `search_repository` - Search journal and state

#### Call Tool: Add Journal Entry

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "add_journal_entry",
    "arguments": {
      "content": "## Consultation\\n\\nPatient reports improvement in symptoms...",
      "author": "dr-jones"
    }
  }
}
```

#### Call Tool: Update State

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "tools/call",
  "params": {
    "name": "update_state",
    "arguments": {
      "filename": "medications.json",
      "content": "{\"medications\": []}"
    }
  }
}
```

#### Call Tool: Search Repository

```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "tools/call",
  "params": {
    "name": "search_repository",
    "arguments": {
      "query": "diabetes"
    }
  }
}
```

Returns list of matching files in journal/ and state/.

## API Reference

### Resources

| URI | Returns | MIME type |
| --- | --- | --- |
| `gitehr://repo/journal` | JSON array of journal entry filenames | `application/json` |
| `gitehr://repo/journal/{filename}` | Raw Markdown content of one journal entry | `text/markdown` |
| `gitehr://repo/state` | JSON array of state filenames (excludes `README.md`) | `application/json` |
| `gitehr://repo/state/{filename}` | Raw content of one state file | `text/plain` |
| `gitehr://repo/status` | Repository status: `version`, `encrypted`, `journal_entry_count`, `state_files` | `application/json` |

Only the three top-level URIs are returned by `resources/list`; the `{filename}` forms are read directly by URI and are not enumerated.

### Tools

| Tool | Parameters | Behaviour |
| --- | --- | --- |
| `add_journal_entry` | `content` (string, required) — Markdown body; `author` (string, optional) — contributor ID, defaults to the target repository's active contributor | Writes `journal/{timestamp}-{uuid}.md` with proper YAML front matter, stages it, and commits it — the same entry format `gitehr journal add` produces. Rejects empty/whitespace-only content. |
| `update_state` | `filename` (string, required), `content` (string, required) | Writes `content` verbatim to `state/{filename}`, creating `state/` if needed. Overwrites any existing file at that path. No journal entry or commit is recorded. |
| `search_repository` | `query` (string, required) | Case-insensitive substring search across `.md` files in `journal/` and every file in `state/`. Returns matching paths as `journal/{filename}` or `state/{filename}`. |

## Integration with Claude Desktop

To use GitEHR MCP server with Claude Desktop:

1. Build the gitehr binary:
```bash
cargo build --release
```

2. Add to Claude Desktop configuration (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "gitehr": {
      "command": "/path/to/gitehr/target/release/gitehr",
      "args": ["mcp", "serve", "--stdio", "--repo-path", "/path/to/your/gitehr/repo"]
    }
  }
}
```

3. Restart Claude Desktop

4. Claude will now have access to your GitEHR repository via MCP

## Example Workflows

### AI-Assisted Note Writing

1. Ask Claude: "Review recent journal entries and draft a consultation note"
2. Claude uses `resources/read` to access recent journal entries
3. Claude generates draft note
4. You review and approve
5. Claude uses `add_journal_entry` tool to save the note

### Clinical Data Extraction

1. Ask Claude: "Extract all medications from recent notes into structured state"
2. Claude uses `search_repository` to find medication mentions
3. Claude parses text and creates structured JSON
4. Claude uses `update_state` tool to save to `state/medications.json`

## Security Considerations

- Filenames in resource URIs (`journal/{filename}`, `state/{filename}`) and in the `update_state` tool are validated to a single bare path component, so `../` traversal outside the repository is rejected.
- `gitehr mcp serve` refuses to start unless `--repo-path` (or the current directory) contains a `.gitehr` directory, and refuses to start against a repository marked `.gitehr/ENCRYPTED`, since encrypted-repository support does not exist yet. Point the server only at repositories you trust.
- All operations are logged to stderr via `RUST_LOG` (future: audit entries in journal — see [Limitations](#limitations-current-implementation))
- Runs with the same file permissions as the user running the command

## Debugging

Enable trace logging:

```bash
RUST_LOG=trace gitehr mcp serve --stdio
```

This will show all MCP protocol messages in stderr.

## Limitations (Current Implementation)

- **No prompts**: Prompt templates not yet implemented
- **No authentication**: Stdio mode assumes local trust
- **No encryption support**: Server refuses to operate on encrypted repos rather than decrypting them (see [Security Considerations](#security-considerations))
- **No audit logging**: MCP operations not yet recorded in journal

These will be addressed in future releases.

## Protocol Compliance

GitEHR implements the Model Context Protocol specification:
- JSON-RPC 2.0
- Protocol version: `2024-11-05`
- Transport: stdio (HTTP/SSE planned)

See [MCP Specification](https://spec.modelcontextprotocol.io/) for full protocol details.
