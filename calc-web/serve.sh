#!/usr/bin/env bash
# Serve the calc-web calculators over HTTP.
#
# The calculators import the shared bridge as an ES module, which browsers will
# not load over file://, so a static HTTP server is needed for local dev.
#
# Usage: ./serve.sh [port]   (default 5500)
set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
PORT="${1:-5500}"

echo "Serving calc-web on http://localhost:${PORT}/index.html  (Ctrl-C to stop)"
exec python3 -m http.server "${PORT}" --directory "${DIR}"
