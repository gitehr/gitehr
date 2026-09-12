#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Minimal Python client for GitEHR's MCP server (R39).
#
# Speaks JSON-RPC 2.0 over the stdio transport implemented by
# `gitehr mcp serve --stdio` (see cli/src/commands/mcp/server_impl/ and
# spec/mcp.md). Stdlib only, no third-party dependencies, matching the
# rest of this repository's Python tooling (s/generate, s/demo-store).
#
# Example:
#
#   from gitehr_mcp import GitEHRMCPClient
#
#   with GitEHRMCPClient(repo_path="/path/to/repo") as client:
#       client.initialize()
#       for resource in client.list_resources():
#           print(resource["uri"])
#       journal = client.read_resource("gitehr://repo/journal")

from __future__ import annotations

import json
import selectors
import subprocess
import tempfile
from itertools import count
from typing import Any, Optional

DEFAULT_TIMEOUT = 30.0


class McpError(RuntimeError):
    """A JSON-RPC error object returned by the server."""

    def __init__(self, code: int, message: str, data: Any = None) -> None:
        super().__init__(f"MCP error {code}: {message}")
        self.code = code
        self.data = data


class GitEHRMCPClient:
    """A synchronous stdio client for `gitehr mcp serve --stdio`.

    One process per client: the server is spawned in the constructor and
    torn down by `close()` (or the `with` block). Not thread-safe - each
    request blocks until its matching response line arrives, so concurrent
    calls from multiple threads would race on the same pipes.
    """

    def __init__(
        self,
        gitehr_bin: str = "gitehr",
        repo_path: Optional[str] = None,
        timeout: Optional[float] = DEFAULT_TIMEOUT,
    ) -> None:
        args = [gitehr_bin, "mcp", "serve", "--stdio"]
        if repo_path is not None:
            args += ["--repo-path", repo_path]
        # The server's diagnostics go to a temporary file rather than a pipe.
        # An undrained stderr pipe deadlocks once it fills: the server blocks
        # writing to it while this client blocks waiting for a response that
        # can no longer be written. A file has no such limit, and is still
        # readable when a failure needs explaining.
        self._stderr = tempfile.TemporaryFile(mode="w+")
        self._process = subprocess.Popen(
            args,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=self._stderr,
            text=True,
            bufsize=1,
        )
        self._next_id = count(1)
        self._timeout = timeout

    def __enter__(self) -> "GitEHRMCPClient":
        return self

    def __exit__(self, *exc_info: object) -> None:
        self.close()

    def close(self) -> None:
        """Terminate the server process, if it is still running."""
        if self._process.poll() is None:
            self._process.terminate()
            try:
                self._process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self._process.kill()
                self._process.wait()
        for pipe in (self._process.stdin, self._process.stdout):
            if pipe is not None:
                pipe.close()
        self._stderr.close()

    # -- MCP protocol ---------------------------------------------------

    def initialize(
        self,
        client_name: str = "gitehr-mcp-python",
        client_version: str = "0.1.0",
    ) -> dict:
        """Send `initialize`, then the required `notifications/initialized`.

        Must be called once before any other request; the server rejects
        everything else until it has seen this handshake.
        """
        result = self._request(
            "initialize",
            {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": client_name, "version": client_version},
            },
        )
        self._notify("notifications/initialized")
        return result

    def list_resources(self) -> list[dict]:
        return self._request("resources/list", {})["resources"]

    def read_resource(self, uri: str) -> list[dict]:
        return self._request("resources/read", {"uri": uri})["contents"]

    def list_tools(self) -> list[dict]:
        return self._request("tools/list", {})["tools"]

    def call_tool(self, name: str, arguments: Optional[dict] = None) -> dict:
        return self._request("tools/call", {"name": name, "arguments": arguments or {}})

    def list_prompts(self) -> list[dict]:
        return self._request("prompts/list", {})["prompts"]

    def get_prompt(self, name: str, arguments: Optional[dict] = None) -> dict:
        return self._request("prompts/get", {"name": name, "arguments": arguments or {}})

    # -- JSON-RPC transport -----------------------------------------------

    def _notify(self, method: str, params: Optional[dict] = None) -> None:
        message: dict[str, Any] = {"jsonrpc": "2.0", "method": method}
        if params is not None:
            message["params"] = params
        self._write(message)

    def _request(self, method: str, params: dict) -> Any:
        request_id = next(self._next_id)
        self._write(
            {"jsonrpc": "2.0", "id": request_id, "method": method, "params": params}
        )
        response = self._read()
        if response.get("id") != request_id:
            raise McpError(
                -32603,
                f"unexpected response id {response.get('id')!r}, expected {request_id}",
            )
        error = response.get("error")
        if error is not None:
            raise McpError(error["code"], error["message"], error.get("data"))
        return response.get("result")

    def _write(self, message: dict) -> None:
        assert self._process.stdin is not None
        self._process.stdin.write(json.dumps(message) + "\n")
        self._process.stdin.flush()

    def _read(self) -> dict:
        assert self._process.stdout is not None
        if not self._wait_readable():
            raise McpError(
                -32603,
                f"gitehr mcp serve did not respond within {self._timeout}s"
                + self._stderr_suffix(),
            )

        line = self._process.stdout.readline()
        if line == "":
            raise McpError(
                -32603,
                "gitehr mcp serve exited without a response" + self._stderr_suffix(),
            )
        return json.loads(line)

    def _wait_readable(self) -> bool:
        """Whether a response arrived in time. A silent server should fail a
        test rather than hang it."""
        if self._timeout is None:
            return True
        try:
            with selectors.DefaultSelector() as selector:
                selector.register(self._process.stdout, selectors.EVENT_READ)
                return bool(selector.select(self._timeout))
        except (OSError, ValueError):
            # Windows selectors cannot wait on a pipe; block instead of
            # refusing to run at all.
            return True

    def _stderr_suffix(self) -> str:
        """Whatever the server managed to say before going quiet."""
        try:
            self._stderr.seek(0)
            message = self._stderr.read().strip()
        except (OSError, ValueError):
            return ""
        return f" (stderr: {message})" if message else ""
