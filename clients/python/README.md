<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# GitEHR MCP Python client

A minimal Python client for GitEHR's MCP server (`gitehr mcp serve --stdio`), for testing and scripting against it without hand-writing JSON-RPC. See [`docs/cli/mcp-usage.md`](../../docs/cli/mcp-usage.md) for the protocol itself.

Standard library only - no dependencies to install. Copy `gitehr_mcp.py` into your project, or run it from a checkout of this repository.

## Usage

```python
from gitehr_mcp import GitEHRMCPClient

with GitEHRMCPClient(repo_path="/path/to/gitehr/repo") as client:
    client.initialize()

    for resource in client.list_resources():
        print(resource["uri"])

    journal = client.read_resource("gitehr://repo/journal")

    for tool in client.list_tools():
        print(tool["name"])

    result = client.call_tool("search_repository", {"query": "penicillin"})

    prompt = client.get_prompt(
        "soap_note", {"chief_complaint": "chest pain", "specialty": "cardiology"}
    )
```

Pass `gitehr_bin` as the first positional argument if `gitehr` is not on `PATH`:

```python
GitEHRMCPClient("/path/to/gitehr", repo_path="/path/to/repo")
```

`call_tool` and `get_prompt` raise `McpError` (with `.code`, and `.data` when the server supplied it) on a JSON-RPC error response, so a missing tool, invalid arguments, or repository error surfaces as a normal Python exception rather than a silent malformed result.

A request that goes unanswered for 30 seconds also raises `McpError`, quoting whatever the server wrote to stderr, so a stalled server fails a test rather than hanging it. Pass `timeout=` to change that, or `timeout=None` to wait indefinitely (on Windows, where a pipe cannot be waited on, it always blocks).

## Testing the library itself

`test_gitehr_mcp.py` drives a real `gitehr` binary end to end: it builds the release binary if needed (or set `GITEHR_BIN` to reuse one you already built), bootstraps a throwaway repo with `gitehr store init`, and exercises resources, tools, and prompts through the client.

```bash
python3 clients/python/test_gitehr_mcp.py
```

CI runs it on every push against the debug binary, so the client cannot drift away from the server it documents.
