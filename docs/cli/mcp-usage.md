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
- `gitehr://repo/{path}/journal` - Journal entries list
- `gitehr://repo/{path}/state` - State files list
- `gitehr://repo/{path}/status` - Repository status

#### Read Resource

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "resources/read",
  "params": {
    "uri": "gitehr://repo/./journal"
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
- `verify_journal` - Verify journal integrity
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
      "content": "## Consultation\\n\\nPatient reports improvement in symptoms..."
    }
  }
}
```

#### Call Tool: Search Repository

```json
{
  "jsonrpc": "2.0",
  "id": 6,
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

### Repository Quality Checks

1. Ask Claude: "Verify the journal integrity"
2. Claude uses `verify_journal` tool
3. Claude reports any broken hash chain links

## Security Considerations

- MCP server requires a valid GitEHR repository (`.gitehr` directory must exist)
- Respects encryption markers (will fail if repository is encrypted)
- All operations are logged (future: audit entries in journal)
- Runs with the same file permissions as the user running the command

## Debugging

Enable trace logging:

```bash
RUST_LOG=trace gitehr mcp serve --stdio
```

This will show all MCP protocol messages in stderr.

## Limitations (Current Implementation)

- **Placeholder journal creation**: The `add_journal_entry` tool currently doesn't create real journal entries (needs integration with gitehr library)
- **No prompts**: Prompt templates not yet implemented
- **No authentication**: Stdio mode assumes local trust
- **No encryption handling**: Server doesn't decrypt encrypted repos
- **No audit logging**: MCP operations not yet recorded in journal

These will be addressed in future releases.

## Protocol Compliance

GitEHR implements the Model Context Protocol specification:
- JSON-RPC 2.0
- Protocol version: `2024-11-05`
- Transport: stdio (HTTP/SSE planned)

See [MCP Specification](https://spec.modelcontextprotocol.io/) for full protocol details.
