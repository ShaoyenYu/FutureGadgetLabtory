# Web (WASM / WebAssembly) Platform

This directory contains web platform configurations and templates for DuoSnake.

## Structure

- `index.html`: Responsive HTML5 canvas wrapper with Candy Kawaii UI, fullscreen support, and loading states.
- `Trunk.toml`: Configuration for the Trunk WASM bundler.

## Quick Start

From the project root:

```bash
# Live development server
trunk serve

# Release build (output to dist/)
trunk build --release
```

Or using the helper scripts in `scripts/`:
- Windows: `.\scripts\build-web.ps1 -Serve`
- Linux/macOS: `./scripts/build-web.sh serve`
