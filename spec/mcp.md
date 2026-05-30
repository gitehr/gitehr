<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Model Context Protocol (MCP) Server for GitEHR

## Goal

Expose GitEHR repositories as **Model Context Protocol (MCP)** servers, enabling LLM agents to read, write, and interact with patient medical records through a standardized protocol while maintaining GitEHR's security, audit, and version control guarantees.

## Motivation

The Model Context Protocol (MCP) provides a standardized way for LLM applications to access external data sources and tools. By implementing MCP server capabilities in GitEHR:

1. **AI-assisted clinical workflows**: LLMs can suggest diagnoses, generate clinical notes, identify drug interactions, etc.
2. **Structured data extraction**: Convert free-text journal entries into structured state data
3. **Clinical decision support**: Integrate with clinical calculators and evidence-based guidelines
4. **Interoperability**: Standard protocol for LLM-EHR integration across applications
5. **Audit trail**: All LLM interactions are logged in the journal for transparency

## Scope

### MCP Server Capabilities

GitEHR will implement the full MCP specification:

1. **Resources**: Read-only access to repository data
   - Journal entries (list, read, search)
   - State files (list, read)
   - Imaging metadata
   - Document metadata
   - Repository status and version

2. **Tools**: Actionable operations
   - Add journal entry
   - Update state file
   - Run clinical calculators
   - Verify journal integrity
   - Search repository content
   - Generate summaries

3. **Prompts**: Clinical templates and workflows
   - SOAP note template
   - Discharge summary template
   - Referral letter template
   - Consultation note template
   - Medication review template

4. **Sampling** (optional): Allow LLM to request human review before committing changes

## Architecture

### Dual-Mode Operation

GitEHR CLI can operate as **both** a traditional CLI tool **and** an MCP server:

```bash
# Traditional CLI mode (current behavior)
gitehr init
gitehr journal add "Patient note"
gitehr status

# MCP server mode (new)
gitehr mcp serve --port 3000
gitehr mcp serve --stdio  # For local LLM clients
gitehr mcp serve --config ~/.gitehr-mcp.json
```

### Workspace Structure

Add new MCP server crate to workspace:

```
gitehr/
├── Cargo.toml                    # Workspace root
├── cli/                          # Main CLI binary
├── gitehr-calculators/           # Clinical calculators
├── mcp/                          # MCP server crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs               # MCP server binary entrypoint
│   │   ├── lib.rs                # MCP library for CLI integration
│   │   ├── server.rs             # MCP protocol implementation
│   │   ├── resources.rs          # Resource handlers
│   │   ├── tools.rs              # Tool handlers
│   │   ├── prompts.rs            # Prompt templates
│   │   ├── security.rs           # Auth, encryption checks
│   │   └── audit.rs              # MCP audit logging
│   └── tests/
│       └── integration.rs
└── gui/
```

### MCP Protocol Implementation

```rust
// mcp/src/server.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,  // "2.0"
    pub id: serde_json::Value,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<McpError>,
}

pub enum McpMethod {
    // Discovery
    ListResources,
    ListTools,
    ListPrompts,
    
    // Resources
    ReadResource { uri: String },
    
    // Tools
    CallTool { name: String, arguments: serde_json::Value },
    
    // Prompts
    GetPrompt { name: String, arguments: serde_json::Value },
    
    // Sampling (optional)
    CreateMessage { messages: Vec<Message>, max_tokens: usize },
}
```

### Transport Options

1. **stdio**: Standard input/output (for local clients like Claude Desktop)
2. **HTTP/SSE**: Server-Sent Events over HTTP (for remote clients)
3. **Unix socket**: Local domain socket (Linux/macOS)
4. **Named pipe**: Windows equivalent

## MCP Resources

Resources provide **read-only** access to repository data.

### Resource URI Scheme

```
gitehr://repo/{repo_path}/journal/{entry_id}
gitehr://repo/{repo_path}/state/{filename}
gitehr://repo/{repo_path}/imaging/{file_id}
gitehr://repo/{repo_path}/status
```

### Example: List Resources

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "resources/list"
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "resources": [
      {
        "uri": "gitehr://repo/current/journal",
        "name": "Journal Entries",
        "description": "Chronological clinical notes",
        "mimeType": "application/json"
      },
      {
        "uri": "gitehr://repo/current/state",
        "name": "Current Clinical State",
        "description": "Active problems, medications, allergies",
        "mimeType": "application/json"
      }
    ]
  }
}
```

### Example: Read Resource

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "resources/read",
  "params": {
    "uri": "gitehr://repo/current/journal/20260306T120000.000Z-abc123.md"
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "contents": [
      {
        "uri": "gitehr://repo/current/journal/20260306T120000.000Z-abc123.md",
        "mimeType": "text/markdown",
        "text": "---\nparent_hash: \"...\"\n...\n---\n\nPatient presented with..."
      }
    ]
  }
}
```

## MCP Tools

Tools allow **read-write** operations on the repository.

### Available Tools

1. **`add_journal_entry`**: Create new journal entry
2. **`update_state`**: Update state file
3. **`calculate_clinical`**: Run clinical calculator
4. **`verify_journal`**: Check journal integrity
5. **`search_repository`**: Full-text search across journal and state
6. **`summarize_journal`**: Generate summary of recent entries
7. **`extract_structured_data`**: Parse journal into structured state

### Example: Add Journal Entry Tool

**Tool Definition**:
```json
{
  "name": "add_journal_entry",
  "description": "Create a new clinical journal entry",
  "inputSchema": {
    "type": "object",
    "properties": {
      "content": {
        "type": "string",
        "description": "Markdown content of the journal entry"
      },
      "author": {
        "type": "string",
        "description": "Optional contributor ID (defaults to active user)"
      }
    },
    "required": ["content"]
  }
}
```

**Tool Call Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "add_journal_entry",
    "arguments": {
      "content": "## Consultation\n\nPatient reports improvement in symptoms..."
    }
  }
}
```

**Tool Call Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Created journal entry: journal/20260306T153000.000Z-def456.md"
      }
    ],
    "isError": false
  }
}
```

### Example: Clinical Calculator Tool

**Tool Definition**:
```json
{
  "name": "calculate_clinical",
  "description": "Run a clinical calculator and record results",
  "inputSchema": {
    "type": "object",
    "properties": {
      "calculator": {
        "type": "string",
        "enum": ["chads2", "egfr", "curb65", "wells_dvt", "growth"],
        "description": "Calculator name"
      },
      "inputs": {
        "type": "object",
        "description": "Calculator-specific inputs"
      },
      "record_in_journal": {
        "type": "boolean",
        "description": "Create journal entry with results (default: true)"
      }
    },
    "required": ["calculator", "inputs"]
  }
}
```

**Tool Call**:
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "calculate_clinical",
    "arguments": {
      "calculator": "chads2",
      "inputs": {
        "age_over_75": true,
        "hypertension": true,
        "diabetes": true
      },
      "record_in_journal": true
    }
  }
}
```

**Tool Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "CHADS2 Score: 3 (Moderate-High risk)\nAnnual stroke risk: 8.5%\nRecommendation: Anticoagulation recommended\n\nRecorded in: journal/20260306T154500.000Z-ghi789.md"
      }
    ],
    "isError": false
  }
}
```

## MCP Prompts

Prompts provide clinical note templates with variable substitution.

### Example Prompts

1. **`soap_note`**: SOAP (Subjective, Objective, Assessment, Plan) note template
2. **`discharge_summary`**: Hospital discharge summary
3. **`referral_letter`**: Specialist referral template
4. **`medication_review`**: Systematic medication review
5. **`consultation`**: General consultation note

### Example: SOAP Note Prompt

**Prompt Definition**:
```json
{
  "name": "soap_note",
  "description": "Generate a SOAP note template",
  "arguments": [
    {
      "name": "chief_complaint",
      "description": "Patient's chief complaint",
      "required": true
    },
    {
      "name": "specialty",
      "description": "Medical specialty (e.g., cardiology, pediatrics)",
      "required": false
    }
  ]
}
```

**Get Prompt Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 5,
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

**Get Prompt Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "description": "SOAP note template for chest pain (cardiology)",
    "messages": [
      {
        "role": "user",
        "content": {
          "type": "text",
          "text": "Generate a cardiology SOAP note for a patient presenting with chest pain. Include:\n\n**Subjective**: Symptom description, onset, character, duration, associated symptoms, risk factors\n\n**Objective**: Vital signs, physical exam findings, relevant investigations (ECG, troponin, etc.)\n\n**Assessment**: Differential diagnosis, risk stratification (e.g., HEART score)\n\n**Plan**: Investigations, treatment, disposition, follow-up"
        }
      }
    ]
  }
}
```

## Security and Access Control

### Authentication

MCP server supports multiple authentication modes:

1. **Local mode** (stdio): No auth required (implicit trust of local user)
2. **Token-based auth**: Bearer tokens stored in `.gitehr/mcp-tokens.json`
3. **OAuth2**: For integration with identity providers
4. **Client certificates**: Mutual TLS for high-security environments

### Token Configuration

```json
// .gitehr/mcp-tokens.json
{
  "tokens": [
    {
      "token": "mcp_abc123xyz",
      "name": "Claude Desktop",
      "created_at": "2026-03-06T12:00:00Z",
      "expires_at": null,
      "permissions": {
        "resources": ["read"],
        "tools": ["add_journal_entry", "calculate_clinical"],
        "prompts": ["all"]
      }
    }
  ]
}
```

### Encryption Awareness

MCP server must respect `.gitehr/ENCRYPTED` marker:

- If repository is encrypted, MCP requests fail with "Repository encrypted" error
- Decrypt workflow: Client must call `decrypt` tool before accessing data
- All MCP operations should re-check encryption status per request

### Audit Logging

All MCP interactions are logged in a dedicated journal entry format:

```yaml
---
parent_hash: "..."
parent_entry: "..."
timestamp: "2026-03-06T15:00:00Z"
author: "mcp-server"
mcp_audit:
  client: "Claude Desktop"
  client_version: "1.0.0"
  method: "tools/call"
  tool: "add_journal_entry"
  token_name: "Claude Desktop"
  ip_address: "127.0.0.1"
  user_agent: "MCP-Client/1.0"
---

# MCP Audit Log

**Operation**: add_journal_entry
**Client**: Claude Desktop
**Result**: Success
**Created**: journal/20260306T150000.000Z-abc123.md
```

## CLI Integration

### New Command: `gitehr mcp`

```bash
# Start MCP server on stdio (for local clients)
gitehr mcp serve --stdio

# Start MCP server on HTTP
gitehr mcp serve --port 3000

# Start with custom config
gitehr mcp serve --config ~/.gitehr-mcp.json

# Generate MCP token
gitehr mcp token create --name "Claude Desktop" --permissions read,write

# List active tokens
gitehr mcp token list

# Revoke token
gitehr mcp token revoke mcp_abc123xyz
```

### MCP Server Configuration

```json
// .gitehr/mcp.json
{
  "enabled": true,
  "transport": "stdio",  // or "http", "unix", "named_pipe"
  "port": 3000,
  "bind": "127.0.0.1",
  "auth": {
    "method": "token",  // or "none", "oauth2", "mtls"
    "token_file": ".gitehr/mcp-tokens.json"
  },
  "resources": {
    "journal": { "enabled": true, "max_entries": 1000 },
    "state": { "enabled": true },
    "imaging": { "enabled": true },
    "documents": { "enabled": true }
  },
  "tools": {
    "add_journal_entry": { "enabled": true },
    "update_state": { "enabled": true },
    "calculate_clinical": { "enabled": true },
    "verify_journal": { "enabled": true },
    "search_repository": { "enabled": true }
  },
  "prompts": {
    "enabled": true,
    "custom_prompts_dir": ".gitehr/prompts/"
  },
  "audit": {
    "log_all_requests": true,
    "create_journal_entries": true
  }
}
```

## Implementation Steps

1. **Create `gitehr-mcp` crate** - Workspace member with MCP protocol implementation
2. **Implement MCP server core** - JSON-RPC 2.0 handler, transport abstraction
3. **Add resource handlers** - Read-only access to journal, state, imaging, documents
4. **Add tool handlers** - Write operations (journal, state, calculators)
5. **Add prompt templates** - Clinical note templates with variable substitution
6. **Implement authentication** - Token-based auth, OAuth2 support
7. **Add audit logging** - Dedicated MCP audit journal entries
8. **CLI command integration** - `gitehr mcp serve` command
9. **Configuration system** - `.gitehr/mcp.json` and `.gitehr/mcp-tokens.json`
10. **Client libraries** - Python/TypeScript client for testing
11. **Documentation** - MCP integration guide, API reference
12. **Testing** - Integration tests with MCP client simulator

## Dependencies

Add to `mcp/Cargo.toml`:

```toml
[dependencies]
gitehr = { path = "../cli" }
gitehr-calculators = { path = "../gitehr-calculators" }

# MCP protocol
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yml = "0.0.12"

# Server runtime
tokio = { version = "1", features = ["full"] }
axum = "0.7"  # HTTP server
tower = "0.5"  # Middleware
tower-http = { version = "0.6", features = ["cors", "trace"] }

# Transport
futures = "0.3"
async-stream = "0.3"

# Auth
jsonwebtoken = "9"  # JWT tokens
uuid = { version = "1", features = ["v4"] }

# Error handling
anyhow = "1"
thiserror = "2"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

## Example Use Cases

### 1. AI-Assisted Clinical Note Writing

LLM reads recent journal entries and current state, then generates a consultation note using SOAP template.

```
User → LLM: "Generate a follow-up note for Mr. Smith"
LLM → MCP: resources/read (recent journal entries)
LLM → MCP: resources/read (current medications, allergies)
LLM → MCP: prompts/get (soap_note template)
LLM → User: [Draft SOAP note]
User → LLM: "Looks good, save it"
LLM → MCP: tools/call (add_journal_entry)
```

### 2. Clinical Calculator Integration

LLM extracts patient data and runs clinical calculator automatically.

```
User → LLM: "Calculate CHADS2 score"
LLM → MCP: resources/read (demographics, problems)
LLM → MCP: tools/call (calculate_clinical, calculator="chads2")
LLM → User: "CHADS2 score is 3 (Moderate-High risk, anticoagulation recommended)"
[Journal entry automatically created]
```

### 3. Structured Data Extraction

LLM parses free-text journal entry into structured state data.

```
User → LLM: "Update the allergy list from recent notes"
LLM → MCP: resources/read (recent journal entries)
LLM → MCP: tools/call (extract_structured_data)
LLM → MCP: tools/call (update_state, file="allergies.json")
LLM → User: "Updated allergies: added penicillin allergy from March 5 note"
```

## Dual CLI/MCP Binary Design

The same `gitehr` binary can operate in both modes:

```bash
# Traditional CLI mode (detect from args)
gitehr journal add "Note"

# MCP server mode (detect from args)
gitehr mcp serve --stdio
```

Alternatively, provide both binaries in the same package:
- `gitehr` - CLI tool
- `gitehr-mcp` - MCP server (symlink or separate binary)

## Integration with GUI

The GUI can also act as an MCP client:

- **Tauri command**: `start_mcp_server()` launches embedded MCP server
- **GUI panel**: LLM chat interface with MCP tool access
- **Permissions**: User can approve/deny MCP tool calls via GUI prompts

## Standards Compliance

Implement full MCP specification from Anthropic:
- JSON-RPC 2.0 protocol
- Standard resource/tool/prompt schemas
- Server-Sent Events (SSE) for progress updates
- Sampling protocol for human-in-the-loop workflows

## Open Questions

- Should MCP server run as a persistent daemon or on-demand per request?
- Should we support MCP server clustering for multi-user deployments?
- Should MCP audit logs be in separate files or standard journal entries?
- How to handle long-running operations (e.g., bulk imports) in MCP tools?
- Should we support MCP-over-WebSocket in addition to HTTP/SSE?

## Future Enhancements

- **MCP proxy**: Multi-repository MCP server for whole-hospital deployments
- **Federated MCP**: Cross-site repository access with consent management
- **MCP plugins**: Third-party MCP tool extensions
- **Real-time subscriptions**: SSE-based notifications for repository changes
- **MCP analytics**: Track LLM usage patterns, tool effectiveness
- **MCP marketplace**: Pre-built prompt templates and tools

---

This specification establishes GitEHR as a first-class MCP server, enabling seamless LLM integration while maintaining security, auditability, and clinical data integrity.
