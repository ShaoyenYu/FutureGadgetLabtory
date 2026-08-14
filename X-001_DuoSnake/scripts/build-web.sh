#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "=== DuoSnake Web (WASM) Build Helper ==="

# 1. Ensure wasm target is installed
if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    echo "Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# 2. Check for trunk
if ! command -v trunk &> /dev/null; then
    echo "Trunk not found. Install via: cargo install trunk"
fi

MODE="${1:-serve}"
PORT="${2:-8080}"

if [ "$MODE" = "build" ]; then
    echo "Building WASM bundle for release..."
    trunk build --release
    echo "Build completed in dist/"
else
    echo "Starting Trunk dev server at http://127.0.0.1:$PORT ..."
    trunk serve --port "$PORT"
fi
