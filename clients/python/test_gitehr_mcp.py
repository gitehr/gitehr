#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Smoke test for gitehr_mcp.GitEHRMCPClient against a real `gitehr` binary.
#
# Builds the release binary if needed, then drives a fresh `gitehr init`
# repository through the MCP handshake, resources, tools, and prompts -
# the same surface docs/cli/mcp-usage.md and test-mcp.sh exercise by hand.
#
# Usage:
#   python3 clients/python/test_gitehr_mcp.py
#   GITEHR_BIN=/path/to/gitehr python3 clients/python/test_gitehr_mcp.py

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from gitehr_mcp import GitEHRMCPClient, McpError  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parents[2]


def _resolve_gitehr_bin() -> str:
    env_bin = os.environ.get("GITEHR_BIN")
    if env_bin:
        return str(Path(env_bin).resolve())

    release_bin = REPO_ROOT / "target" / "release" / "gitehr"
    if not release_bin.exists():
        subprocess.run(
            ["cargo", "build", "--release", "--quiet", "-p", "gitehr"],
            cwd=REPO_ROOT,
            check=True,
        )
    return str(release_bin)


class GitEHRMCPClientTest(unittest.TestCase):
    def setUp(self) -> None:
        self.gitehr_bin = _resolve_gitehr_bin()
        self.store_dir = tempfile.TemporaryDirectory()
        # `store init` bootstraps a Store plus its first subject repo, the
        # only way GitEHR creates a fresh repo today; stdin is closed so the
        # auto-generated-id path is taken instead of the interactive prompt.
        subprocess.run(
            [self.gitehr_bin, "store", "init"],
            cwd=self.store_dir.name,
            check=True,
            capture_output=True,
            stdin=subprocess.DEVNULL,
        )
        store_root = Path(self.store_dir.name)
        subject_dirs = [
            entry for entry in store_root.iterdir() if (entry / ".gitehr").is_dir()
        ]
        self.assertEqual(
            len(subject_dirs), 1, f"expected one subject repo, found {subject_dirs}"
        )
        self.repo_path = subject_dirs[0]

        self.client = GitEHRMCPClient(self.gitehr_bin, repo_path=str(self.repo_path))
        self.client.initialize()

    def tearDown(self) -> None:
        self.client.close()
        self.store_dir.cleanup()

    def test_list_resources_includes_journal_and_state(self) -> None:
        uris = {resource["uri"] for resource in self.client.list_resources()}
        self.assertIn("gitehr://repo/journal", uris)
        self.assertIn("gitehr://repo/state", uris)

    def test_read_status_resource(self) -> None:
        contents = self.client.read_resource("gitehr://repo/status")
        self.assertEqual(len(contents), 1)
        self.assertEqual(contents[0]["uri"], "gitehr://repo/status")

    def test_list_tools_includes_add_journal_entry(self) -> None:
        names = {tool["name"] for tool in self.client.list_tools()}
        self.assertIn("add_journal_entry", names)
        self.assertIn("search_repository", names)

    def test_call_tool_search_repository(self) -> None:
        result = self.client.call_tool("search_repository", {"query": "test"})
        self.assertIn("content", result)

    def test_call_tool_unknown_name_raises(self) -> None:
        with self.assertRaises(McpError):
            self.client.call_tool("no_such_tool", {})

    def test_list_prompts_includes_soap_note(self) -> None:
        names = {prompt["name"] for prompt in self.client.list_prompts()}
        self.assertIn("soap_note", names)

    def test_get_prompt_soap_note(self) -> None:
        result = self.client.get_prompt(
            "soap_note", {"chief_complaint": "chest pain", "specialty": "cardiology"}
        )
        self.assertIn("messages", result)


if __name__ == "__main__":
    unittest.main()
