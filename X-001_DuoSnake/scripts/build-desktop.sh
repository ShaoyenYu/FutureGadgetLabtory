#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "=== DuoSnake Desktop (Native) Helper ==="

MODE="${1:-run}"
if [ "$MODE" = "build" ]; then
    echo "Building desktop release binary..."
    cargo build --release
else
    echo "Running desktop release..."
    cargo run --release
fi
