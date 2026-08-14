# Desktop Platform (Windows / macOS / Linux)

This directory outlines desktop-specific details for DuoSnake.

## Quick Start

From the project root:

```bash
# Debug run
cargo run

# Optimized release run
cargo run --release
```

Or using helper scripts in `scripts/`:
- Windows: `.\scripts\build-desktop.ps1`
- Linux/macOS: `./scripts/build-desktop.sh`

## Packaging Desktop Binaries

- **Windows**: Build produces `target/release/DuoSnake.exe`.
- **macOS**: Build produces binary in `target/release/DuoSnake` (can be wrapped into a `.app` bundle).
- **Linux**: Build produces ELF binary in `target/release/DuoSnake` (can be packaged via AppImage / Flatpak / deb / rpm).
