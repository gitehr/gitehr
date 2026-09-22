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
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"example-client","version":"1.0.0"}}}' | ./target/release/gitehr mcp serve --stdio
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
      "version": "<current GitEHR version>"
    }
  }
}
```

### Python Client Library

For scripting or testing against the server without hand-writing JSON-RPC, use the stdlib-only Python client in [`clients/python/gitehr_mcp.py`](https://github.com/gitehr/gitehr/blob/main/clients/python/gitehr_mcp.py):

```python
from gitehr_mcp import GitEHRMCPClient

with GitEHRMCPClient(repo_path="/path/to/gitehr/repo") as client:
    client.initialize()
    for resource in client.list_resources():
        print(resource["uri"])
```

See [`clients/python/README.md`](https://github.com/gitehr/gitehr/blob/main/clients/python/README.md) for the full API and how to run its own test suite.

## Configuration

`gitehr mcp serve` reads an optional `.gitehr/mcp.json` in the repository being served (R35). A missing file is equivalent to everything enabled - the historical, pre-R35 behaviour - so existing repositories need no changes.

```json
// .gitehr/mcp.json
{
  "enabled": true,
  "resources": {
    "journal": { "enabled": true },
    "state": { "enabled": true },
    "documents": { "enabled": true },
    "imaging": { "enabled": false }
  },
  "tools": {
    "add_journal_entry": { "enabled": true },
    "update_state": { "enabled": false },
    "search_repository": { "enabled": true }
  }
}
```

- **`enabled`** (top-level): when `false`, `gitehr mcp serve` refuses to start and exits non-zero with a clear error, rather than silently serving nothing.
- **`resources.{journal,state,documents,imaging}.enabled`**: when `false`, that group is omitted from `resources/list` and `resources/read` on any URI under it (including individual entries, e.g. `gitehr://repo/journal/<entry>`) fails with a "disabled by .gitehr/mcp.json" error. `status` has no flag and is always available.
- **`tools.{add_journal_entry,update_state,search_repository}.enabled`**: when `false`, that tool is omitted from `tools/list` and `tools/call` fails the same way.
- The config is re-read only at server startup, matching how the repository path itself is fixed for the life of a `gitehr mcp serve` process.

Only the fields shown above are supported. Unsupported or misspelled keys, including future `transport`, `auth`, `audit`, and prompt settings described in `spec/mcp.md`, fail startup rather than being ignored. An access-control setting that parses but has no effect would create false assurance. A malformed `.gitehr/mcp.json` also fails startup with a parse error instead of silently falling back to defaults.

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
- `gitehr://repo/documents` - Non-imaging Documents list
- `gitehr://repo/imaging` - Imaging Documents and studies list

Four further URI patterns are readable but not listed by `resources/list` - read them directly by URI:
- `gitehr://repo/journal/{filename}` - Content of one journal entry (e.g. `gitehr://repo/journal/20260101T000000.000Z-abc123.md`)
- `gitehr://repo/state/{filename}` - Content of one state file (e.g. `gitehr://repo/state/demographics.json`)
- `gitehr://repo/documents/{name}` - Content of one non-imaging Document, or the manifest for a directory Document
- `gitehr://repo/imaging/{name}` - Content of one imaging Document, or the manifest for a directory study

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

### Prompts (Clinical Templates)

Prompts render source-grounded clinical note drafting instructions with variable substitution. Unlike resources and tools, prompts never touch the repository - they only generate a text template for the LLM client to act on. Prompt arguments appear only in a clearly marked JSON data section and are treated as untrusted claims. Each template instructs the client to preserve supplied source attribution and verification status without adding or upgrading it, expose conflicting claims, never use assistant-authored content or generated drafts as corroborating evidence, and never invent missing clinical details or recommendations.

#### List Prompts

```json
{
  "jsonrpc": "2.0",
  "id": 8,
  "method": "prompts/list"
}
```

Returns the five available prompts: `soap_note`, `discharge_summary`, `referral_letter`, `consultation`, `medication_review`, each with its argument list.

#### Get Prompt: SOAP Note

```json
{
  "jsonrpc": "2.0",
  "id": 9,
  "method": "prompts/get",
  "params": {
    "name": "soap_note",
    "arguments": {
      "chief_complaint": "chest pain",
      "specialty": "cardiology"
    }
  }
}
```

Returns a `description` and a `messages` array containing one `user` message whose text is the drafting instruction (Subjective/Objective/Assessment/Plan headings, tailored to the arguments given).

Arguments must be an object containing only the fields declared by `prompts/list`. Every supplied value must be a non-empty, single-line string no larger than 1,000 bytes; unknown fields and malformed values return JSON-RPC `-32602` (invalid params).

## API Reference

### Resources

| URI | Returns | MIME type |
| --- | --- | --- |
| `gitehr://repo/journal` | JSON array of journal entry filenames | `application/json` |
| `gitehr://repo/journal/{filename}` | Raw Markdown content of one journal entry | `text/markdown` |
| `gitehr://repo/state` | JSON array of state filenames (excludes `README.md`) | `application/json` |
| `gitehr://repo/state/{filename}` | Raw content of one state file | `text/plain` |
| `gitehr://repo/status` | Repository status: `version`, `encrypted`, `journal_entry_count`, `state_files` | `application/json` |
| `gitehr://repo/documents` | JSON array of non-imaging Document names | `application/json` |
| `gitehr://repo/documents/{name}` | Document bytes as base64, or a directory Document's manifest as JSON text | Guessed from extension, or `application/json` |
| `gitehr://repo/imaging` | JSON array of imaging Document and study names | `application/json` |
| `gitehr://repo/imaging/{name}` | Imaging bytes as base64, or a directory study's manifest as JSON text | Guessed from extension, or `application/json` |

Only the five top-level URIs are returned by `resources/list`; the `{filename}` and `{name}` forms are read directly by URI and are not enumerated.

### Tools

| Tool | Parameters | Behaviour |
| --- | --- | --- |
| `add_journal_entry` | `content` (string, required) — Markdown body; `author` (string, optional) — contributor ID, defaults to the target repository's active contributor | Writes `journal/{timestamp}-{uuid}.md` with proper YAML front matter **as an uncommitted draft** (ADR-0007): it is not staged or committed and is not part of the record until a human approves it with `gitehr journal drafts --approve`. Rejects empty/whitespace-only content. |
| `update_state` | `filename` (string, required), `content` (string, required) | Writes `content` verbatim to `state/{filename}`, creating `state/` if needed. Overwrites any existing file at that path. No journal entry or commit is recorded. |
| `search_repository` | `query` (string, required) | Case-insensitive substring search across `.md` files in `journal/` and every file in `state/`. Returns matching paths as `journal/{filename}` or `state/{filename}`. |

### Prompts

| Prompt | Required arguments | Optional arguments |
| --- | --- | --- |
| `soap_note` | `chief_complaint` | `specialty` |
| `discharge_summary` | `diagnosis` | `admission_date`, `discharge_date` |
| `referral_letter` | `specialty`, `reason` | `urgency` |
| `consultation` | `chief_complaint` | — |
| `medication_review` | — | `focus` |

Each prompt returns a `description` and a single `user` message of type `text` containing the drafting instruction - it does not read or write repository data.

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
3. Claude generates a draft note for you to review
4. Claude uses `add_journal_entry` to create an uncommitted MCP draft
5. You inspect the draft with `gitehr journal drafts`, then approve or reject it

### Clinical Data Extraction

1. Ask Claude: "Extract all medications from recent notes into structured state"
2. Claude uses `search_repository` to find medication mentions
3. Claude parses text and creates structured JSON
4. Claude uses `update_state` tool to save to `state/medications.json`

## Security Considerations

- Filenames in resource URIs (`journal/{filename}`, `state/{filename}`) and in the `update_state` tool are validated to a single bare path component, so `../` traversal outside the repository is rejected.
- `gitehr mcp serve` refuses to start unless `--repo-path` (or the current directory) contains a `.gitehr` directory, and refuses to start against a repository marked `.gitehr/ENCRYPTED`, since encrypted-repository support does not exist yet. The marker is re-checked on every `resources/list`, `resources/read`, `tools/list`, and `tools/call`, because the server is long-lived and a repository can be marked after it has started serving; those requests then fail with a `Repository encrypted` error. Prompts keep working, as they never touch repository data. Note that the marker does not mean the contents are encrypted - encryption at rest is unimplemented (roadmap R67/R68) and any surviving marker is a stale artefact that `gitehr decrypt` removes. Point the server only at repositories you trust.
- Stdio requests are limited to 1 MiB. Oversized requests are rejected and drained without buffering the remainder in memory.
- Operations are logged to stderr via `RUST_LOG`, but logs include only protocol metadata and byte counts, never request or response bodies that may contain clinical data.
- Every successful tool call is also recorded as a dedicated audit journal entry (front matter `mcp_audit: {method, tool, result}`, author `mcp-server`), separate from any journal entry the tool itself wrote. Client/session identity (client name/version, token, IP) is not yet tracked and so is not included - see [Limitations](#limitations-current-implementation).
- Runs with the same file permissions as the user running the command

## Debugging

Enable trace logging:

```bash
RUST_LOG=trace gitehr mcp serve --stdio
```

This shows recognized protocol method names (or `unknown`), whether a request ID was present, byte counts, lifecycle events, and error codes on stderr. Client-controlled method text and request and response bodies are deliberately omitted because they may contain clinical data.

## Limitations (Current Implementation)

- **No authentication**: Stdio mode assumes local trust
- **No encryption support**: Server refuses to operate on encrypted repos rather than decrypting them (see [Security Considerations](#security-considerations))
- **No client identity in audit entries**: audit entries record the operation and result, but not client name/version, token, or IP, since MCP authentication (R32) does not exist yet
- **`.gitehr/mcp.json` only gates resources and tools**: the `enabled` switch and per-resource/per-tool flags described in [Configuration](#configuration) work. Transport, authentication, audit, and custom-prompt settings are not implemented and are rejected rather than silently ignored

These will be addressed in future releases.

## Protocol Compliance

GitEHR implements the following Model Context Protocol server subset:
- JSON-RPC 2.0
- Protocol version: `2024-11-05`
- Transport: stdio (HTTP/SSE planned)
- Lifecycle: `initialize`, followed by the response-free `notifications/initialized` notification

See [MCP Specification](https://spec.modelcontextprotocol.io/) for full protocol details.
