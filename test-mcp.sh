#!/bin/bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Simple test script for GitEHR MCP server

set -e

# Disable tracing for clean JSON output
export RUST_LOG=off

echo "Building gitehr..."
cargo build --release --quiet

echo "Creating test repository..."
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMP_DIR=$(mktemp -d)
GITEHR_BIN="$SCRIPT_DIR/target/release/gitehr"

cd "$TEMP_DIR"
"$GITEHR_BIN" init
echo "Repository created at: $TEMP_DIR"

echo ""
echo "Testing MCP server..."
echo ""

# Test 1: Initialize
echo "Test 1: Initialize"
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | \
  "$GITEHR_BIN" mcp serve --stdio | jq .

echo ""

# Test 2: List resources
echo "Test 2: List resources"
(
  echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
  echo '{"jsonrpc":"2.0","id":2,"method":"resources/list"}'
) | "$GITEHR_BIN" mcp serve --stdio | tail -1 | jq .

echo ""

# Test 3: Read journal resource
echo "Test 3: Read journal resource"
(
  echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
  echo '{"jsonrpc":"2.0","id":3,"method":"resources/read","params":{"uri":"gitehr://repo/./journal"}}'
) | "$GITEHR_BIN" mcp serve --stdio | tail -1 | jq .

echo ""

# Test 4: List tools
echo "Test 4: List tools"
(
  echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
  echo '{"jsonrpc":"2.0","id":4,"method":"tools/list"}'
) | "$GITEHR_BIN" mcp serve --stdio | tail -1 | jq .

echo ""

# Test 5: Search repository
echo "Test 5: Search repository (looking for 'Genesis')"
(
  echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
  echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"search_repository","arguments":{"query":"Genesis"}}}'
) | "$GITEHR_BIN" mcp serve --stdio | tail -1 | jq .

echo ""
echo "All tests completed successfully!"
echo "Test repository: $TEMP_DIR"
echo ""
echo "To manually test the server, run:"
echo "  cd $TEMP_DIR"
echo "  echo '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}' | gitehr mcp serve --stdio"
